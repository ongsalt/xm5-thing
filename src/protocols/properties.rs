use crate::{platforms::traits::DeviceCommunication, protocols::mdr::MDRPacket};

#[derive(Debug, Clone, Copy)]
pub struct HeadphoneProperties {}

impl HeadphoneProperties {
    pub async fn new<D: DeviceCommunication>(communication: &mut D) -> Self {
        Self {}
    }

    pub fn update(&mut self, packet: MDRPacket) {
        
    }
}
