#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum TeachModeStatus {
    Unknown = 0,
    Aborted = 252,
    Error = 253,
    InProgress = 254,
    Completed = 255,
}

impl From<u8> for TeachModeStatus {
    fn from(value: u8) -> Self {
        match value {
            252 => Self::Aborted,
            253 => Self::Error,
            254 => Self::InProgress,
            255 => Self::Completed,
            _ => Self::Unknown,
        }
    }
}
