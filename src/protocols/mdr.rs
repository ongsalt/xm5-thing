use num_enum::{IntoPrimitive, TryFromPrimitive};

#[derive(Debug, Clone, Copy, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
enum MDRPacketType {
    ConnectGetProtocolInfo = 0x00,
    ConnectRetProtocolInfo = 0x01,
    ConnectGetCapabilityInfo = 0x02,
    ConnectRetCapabilityInfo = 0x03,
}
