use std::time::Instant;

use freya::prelude::*;

pub const SIGNIFICANT_DISPLACEMENT: f32 = 0.001;

#[derive(Debug, Clone, Copy)]
pub struct SpringSpec {
    pub damping: f32,
    pub stiffness: f32,
}

impl SpringSpec {
    pub fn new(damping: f32, stiffness: f32) -> Self {
        Self { damping, stiffness }
    }
}

impl Default for SpringSpec {
    fn default() -> Self {
        Self {
            damping: 1.0,
            stiffness: 400.0,
        }
    }
}

// based on android animatale
#[derive(Debug, Clone, Copy)]
pub struct SpringAnimator {
    value: f32,
    spec: SpringSpec,
    target: f32,
    simulation: Option<SpringSimulation>,
    last_velocity: f32,
}

impl SpringAnimator {
    pub fn new(value: f32) -> Self {
        Self {
            target: value,
            spec: SpringSpec::default(),
            value,
            simulation: None,
            last_velocity: 0.,
        }
    }

    pub fn set_spec(&mut self, spec: SpringSpec) {
        self.spec = spec
    }

    // Need to be called every frame | dt is in second
    pub fn update(&mut self, dt: f32) -> f32 {
        if let Some(ref mut simulation) = self.simulation {
            let motion = simulation.update(self.value, self.last_velocity, dt);
            self.last_velocity = motion.velocity;
            self.value = motion.displacement;
            if (self.target - motion.displacement).abs() < SIGNIFICANT_DISPLACEMENT
                && motion.velocity.abs() < SIGNIFICANT_DISPLACEMENT
                && simulation
                    .get_acceleration(motion.displacement, motion.velocity)
                    .abs()
                    < SIGNIFICANT_DISPLACEMENT
            {
                self.value = self.target;
                self.simulation = None
            }
        }

        self.value
    }

    pub fn animate_to(&mut self, target: f32) {
        if target == self.target {
            return;
        }
        self.target = target;
        self.simulation = {
            let mut simulation = SpringSimulation::new(target);
            simulation.set_spec(&self.spec);
            Some(simulation)
        };
    }

    pub fn animate_by(&mut self, offset: f32) {
        self.animate_to(self.target + offset)
    }

    pub fn is_animating(&self) -> bool {
        self.simulation.is_some()
    }

    pub fn target(&self) -> f32 {
        self.target
    }

    pub fn value(&self) -> f32 {
        self.value
    }
}

pub struct Motion {
    displacement: f32,
    velocity: f32,
}

// From jetpack compose SpringSimulation.kt
#[derive(Debug, Clone, Copy)]
pub struct SpringSimulation {
    final_position: f32,
    natural_freq: f32,
    damping_ratio: f32, // Damping ratio must be non-negative
}

// TODO: refactor this - we dont need 2 seperate struct for this shit
impl SpringSimulation {
    fn new(final_position: f32) -> Self {
        Self {
            final_position,
            natural_freq: 1.,
            damping_ratio: 1.,
        }
    }

    fn set_spec(&mut self, spec: &SpringSpec) {
        self.set_stiffness(spec.stiffness);
        self.damping_ratio = spec.damping;
    }

    fn stiffness(&self) -> f32 {
        self.natural_freq * self.natural_freq
    }

    fn set_stiffness(&mut self, stiffness: f32) -> Result<(), &'static str> {
        if stiffness <= 0. {
            return Err("Spring stiffness constant must be positive");
        }
        self.natural_freq = stiffness.sqrt();
        Ok(())
    }

    fn get_acceleration(&self, last_displacement: f32, last_velocity: f32) -> f32 {
        let adjusted_displacement = last_displacement - self.final_position;

        let k = self.natural_freq * self.natural_freq;
        let c = 2. * self.natural_freq * self.damping_ratio;

        -k * adjusted_displacement - c * last_velocity
    }

    // TODO: refactor this
    fn update(&mut self, last_displacement: f32, last_velocity: f32, dt: f32) -> Motion {
        let adjusted_displacement = last_displacement - self.final_position;
        let k = self.damping_ratio * self.damping_ratio;
        let r = -self.damping_ratio * self.natural_freq;

        let mut displacement = 0.;
        let mut current_velocity = 0.;

        if self.damping_ratio > 1. {
            // Over damping
            let s = self.natural_freq * (k - 1.).sqrt();
            let gamma_plus = r + s;
            let gamma_minus = r - s;

            // Overdamped
            let coeff_b =
                (gamma_minus * adjusted_displacement - last_velocity) / (gamma_minus - gamma_plus);
            let coeff_a = adjusted_displacement - coeff_b;
            displacement = coeff_a * (gamma_minus * dt).exp() + coeff_b * (gamma_plus * dt).exp();
            current_velocity = coeff_a * gamma_minus * (gamma_minus * dt).exp()
                + coeff_b * gamma_plus * (gamma_plus * dt).exp();
        } else if self.damping_ratio == 1. {
            // Critically damped
            let coeff_a = adjusted_displacement;
            let coeff_b = last_velocity + self.natural_freq * adjusted_displacement;
            let n_fd_t = -self.natural_freq * dt;
            displacement = (coeff_a + coeff_b * dt) * n_fd_t.exp();
            current_velocity = ((coeff_a + coeff_b * dt) * n_fd_t.exp() * (-self.natural_freq))
                + coeff_b * n_fd_t.exp()
        } else {
            let damped_freq = self.natural_freq * (1. - k).sqrt();
            // Underdamped
            let cos_coeff = adjusted_displacement;
            let sin_coeff = (1. / damped_freq) * ((-r * adjusted_displacement) + last_velocity);
            let d_fd_t = damped_freq * dt;
            displacement = (r * dt).exp() * (cos_coeff * d_fd_t.cos() + sin_coeff * d_fd_t.sin());
            current_velocity = displacement * r
                + ((r * dt).exp()
                    * (-damped_freq * cos_coeff * d_fd_t.sin()
                        + damped_freq * sin_coeff * (d_fd_t.cos())));
        }

        Motion {
            displacement: displacement + self.final_position,
            velocity: current_velocity,
        }
    }
}

#[derive(Clone, Copy)]
pub struct UseSpring {
    platform: UsePlatform,
    task: Option<Task>,
    spring: SpringAnimator,
    value: Signal<f32>,
    target: Signal<f32>,
    is_animating: Signal<bool>
}

impl UseSpring {
    pub fn new(spring: SpringAnimator, platform: UsePlatform) -> UseSpring {
        let value = use_signal(|| spring.value());
        let target = use_signal(|| spring.target());
        let is_animating = use_signal(|| spring.is_animating());

        UseSpring {
            spring,
            platform,
            target,
            is_animating,
            value,
            task: None
        }
    }

    pub fn set_spec(&mut self, spec: SpringSpec) {
        self.spring.set_spec(spec);
    }

    pub fn animate_to(&mut self, target: f32) {
        self.spring.animate_to(target);
    }

    pub fn value(&self) -> Signal<f32> {
        self.value
    }

    pub fn target(&self) -> Signal<f32> {
        self.target
    }

    pub fn is_animating(&self) -> Signal<bool> {
        self.is_animating
    }

    pub fn run(&mut self) {
        if self.spring.is_animating() {
            // wont launch new task
            return;
        }

        // this should be redundant
        let Some(task) = self.task else {
            return;
        };
        let platform = self.platform;

        // fuck this, i will come back after learning async rust
        let animation_task = spawn(async move {
            platform.request_animation_frame();
            let mut last_updated = Instant::now();

            loop {
                let now = Instant::now();
                let dt = (now - last_updated).as_millis();

                // self.spring.update(dt as f32 / 1000f32);
            }
    
            
        });

        self.task = Some(animation_task);
    }
}

pub fn use_spring(initial_value: f32) -> UseSpring {
    let platform = use_platform();
    let mut spring = SpringAnimator::new(initial_value);

    let mut u = UseSpring::new(spring, platform);

    u.run();

    u
}
