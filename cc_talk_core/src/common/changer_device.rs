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
        device as Self
    }
}

impl From<u8> for ChangerDevice {
    fn from(value: u8) -> Self {
        match value {
            1 => Self::Hopper1,
            2 => Self::Hopper2,
            3 => Self::Hopper3,
            4 => Self::Hopper4,
            5 => Self::Hopper5,
            6 => Self::Hopper6,
            7 => Self::Hopper7,
            8 => Self::Hopper8,
            100 => Self::CoinAcceptor,
            200 => Self::Cashbox,
            255 => Self::System,
            _ => Self::Unknown,
        }
    }
}
