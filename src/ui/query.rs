// Characteristic
pub struct Optimistic<T: Clone> {
    value: Option<T>,
    pending: bool,
}

impl<T: Clone> Optimistic<T> {
    pub fn new(value: Option<T>) -> Self {
        Self {
            pending: value.is_none(),
            value,
        }
    }

    pub fn pending(&self) -> bool {
        self.pending
    }

    pub fn set(&mut self, new_value: T) {
        self.value = Some(new_value);
        self.pending = true;
    }

    pub fn get(&self) -> Option<T> {
        self.value.clone()
    }

    // We should do timeout here too
    pub fn on_notify(&mut self, actual_new_value: T) {
        self.value = Some(actual_new_value);
        self.pending = false;
    }
}


