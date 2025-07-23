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

impl core::convert::From<u8> for HopperStatus {
    fn from(status: u8) -> Self {
        Self {
            low_level_supported: status & 0b0001_0000 > 0, // 1 == supported and fitted
            higher_than_low_level: status & 0b0000_0001 != (1 << 0), // 0 == higher than level
            high_level_supported: status & 0b0010_0000 > 0, // 1 == supported and fitted
            higher_than_high_level: status & 0b0000_0010 > 0, // 1 == higher than level
        }
    }
}

impl core::convert::From<HopperStatus> for u8 {
    fn from(status: HopperStatus) -> Self {
        let mut result = 0u8;
        if status.low_level_supported {
            result |= 0b0001_0000; // 1 == supported and fitted
        }
        if !status.higher_than_low_level {
            result |= 0b0000_0001; // 0 == higher than level
        }
        if status.high_level_supported {
            result |= 0b0010_0000; // 1 == supported and fitted
        }
        if status.higher_than_high_level {
            result |= 0b0000_0010; // 1 == higher than level
        }
        result
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct HopperDispenseStatus {
    pub event_counter: u8,
    pub coins_remaining: u8,
    pub paid: u8,
    pub unpaid: u8,
}

impl HopperDispenseStatus {
    pub fn new(event_counter: u8, coins_remaining: u8, paid: u8, unpaid: u8) -> Self {
        Self {
            event_counter,
            coins_remaining,
            paid,
            unpaid,
        }
    }

    pub fn next_event_counter(&self) -> u8 {
        match self.event_counter {
            u8::MAX => 1, // 0 should only be used on reset.
            _ => self.event_counter + 1,
        }
    }
}

impl core::convert::From<[u8; 4]> for HopperDispenseStatus {
    fn from(status: [u8; 4]) -> Self {
        Self {
            event_counter: status[0],
            coins_remaining: status[1],
            paid: status[2],
            unpaid: status[3],
        }
    }
}

impl core::convert::From<HopperDispenseStatus> for [u8; 4] {
    fn from(status: HopperDispenseStatus) -> Self {
        [
            status.event_counter,
            status.coins_remaining,
            status.paid,
            status.unpaid,
        ]
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
        assert!(status.higher_than_low_level);
        assert!(status.higher_than_high_level);
    }

    #[test]
    fn hopper_status_to_u8() {
        let status = HopperStatus::new(true, true, true, true);
        let mask: u8 = status.into();
        assert_eq!(
            mask, 0b0011_0010,
            "expected {:0b}, got {:0b}",
            0b0011_0010, mask
        );
    }
}
