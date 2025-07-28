#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ChangerDevice {
    Hopper1 = 1,
    Hopper2 = 2,
    Hopper3 = 3,
    Hopper4 = 4,
    Hopper5 = 5,
    Hopper6 = 6,
    Hopper7 = 7,
    Hopper8 = 8,
    CoinAcceptor = 100,
    Cashbox = 200,
    System = 255,
    Unknown = 0,
}

impl From<ChangerDevice> for u8 {
    fn from(device: ChangerDevice) -> Self {
        device as u8
    }
}

impl From<u8> for ChangerDevice {
    fn from(value: u8) -> Self {
        match value {
            1 => ChangerDevice::Hopper1,
            2 => ChangerDevice::Hopper2,
            3 => ChangerDevice::Hopper3,
            4 => ChangerDevice::Hopper4,
            5 => ChangerDevice::Hopper5,
            6 => ChangerDevice::Hopper6,
            7 => ChangerDevice::Hopper7,
            8 => ChangerDevice::Hopper8,
            100 => ChangerDevice::CoinAcceptor,
            200 => ChangerDevice::Cashbox,
            255 => ChangerDevice::System,
            _ => ChangerDevice::Unknown,
        }
    }
}

