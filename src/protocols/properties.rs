use crate::{platforms::traits::DeviceCommunication, protocols::mdr::MDRPacket};

#[derive(Debug, Clone)]
pub struct HeadphoneProperties {
    pub placeholder_text: String
}

impl HeadphoneProperties {
    pub async fn new<D: DeviceCommunication>(communication: &mut D) -> Self {
        Self {
            placeholder_text: "".to_owned()
        }
    }

    pub fn update(&mut self, packet: MDRPacket) {
        
    }
}
