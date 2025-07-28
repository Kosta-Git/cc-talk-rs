#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum LampControl {
    Automatic = 0,
    ManualOff = 1,
    ManualOn = 2,
    ManualFlash(u8) = 10,
}

impl From<LampControl> for u8 {
    fn from(value: LampControl) -> Self {
        match value {
            LampControl::Automatic => 0,
            LampControl::ManualOff => 1,
            LampControl::ManualOn => 2,
            LampControl::ManualFlash(value) => value,
        }
    }
}

impl TryFrom<u8> for LampControl {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(LampControl::Automatic),
            1 => Ok(LampControl::ManualOff),
            2 => Ok(LampControl::ManualOn),
            v if (10..=255).contains(&value) => Ok(LampControl::ManualFlash(v)),
            _ => Err("value not in valid range for LampControl"),
        }
    }
}
