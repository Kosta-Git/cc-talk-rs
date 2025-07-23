#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct HopperStatus {
    pub low_level_supported: bool,
    /// True => higher or equal to low level
    /// False => lower than low level
    pub higher_than_low_level: bool,
    pub high_level_supported: bool,
    /// True => higher or equal to high level
    /// False => lower than high level
    pub higher_than_high_level: bool,
}

impl HopperStatus {
    pub fn new(
        low_level_supported: bool,
        higher_than_low_level: bool,
        high_level_supported: bool,
        higher_than_high_level: bool,
    ) -> Self {
        Self {
            low_level_supported,
            higher_than_low_level,
            high_level_supported,
            higher_than_high_level,
        }
    }
}

impl From<u8> for HopperStatus {
    fn from(status: u8) -> Self {
        Self {
            low_level_supported: status & 0b0001_0000 > 0, // 1 == supported and fitted
            higher_than_low_level: status & 0b0000_0001 != (1 << 0), // 0 == higher than level
            high_level_supported: status & 0b0010_0000 > 0, // 1 == supported and fitted
            higher_than_high_level: status & 0b0000_0010 > 0, // 1 == higher than level
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn u8_to_hopper_status() {
        let status = HopperStatus::from(0b0011_0010);
        assert!(status.low_level_supported);
        assert!(status.high_level_supported);
    }
}
