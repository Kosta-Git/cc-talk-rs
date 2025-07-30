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

#[cfg(feature = "std")]
impl core::fmt::Display for HopperStatus {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let low_status = if self.low_level_supported {
            if self.higher_than_low_level {
                "🟢 Above low level"
            } else {
                "🔴 Below low level"
            }
        } else {
            "➖ Low level not supported"
        };

        let high_status = if self.high_level_supported {
            if self.higher_than_high_level {
                "🟢 Above high level"
            } else {
                "🟡 Below high level"
            }
        } else {
            "➖ High level not supported"
        };

        write!(
            f,
            "📊 Hopper Level Status\n\
            ┌─ 🔻 Low Level: {}\n\
            └─ 🔺 High Level: {}",
            low_status, high_status
        )
    }
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

#[cfg(feature = "std")]
impl core::fmt::Display for HopperDispenseStatus {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "🪙 Coin Hopper Status\n\
             ┌─ 🎫 Event ID: {}\n\
             ├─ 💰 Dispensed: {} coins\n\
             ├─ ⏳ Not dispensed: {} coins\n\
             └─ 📦 Remaining in hopper: {} coins",
            self.event_counter, self.paid, self.unpaid, self.coins_remaining
        )
    }
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

    pub fn payout_requested(&self, coin_count: u8) -> HopperDispenseStatus {
        HopperDispenseStatus {
            event_counter: self.next_event_counter(),
            coins_remaining: self.coins_remaining.saturating_add(coin_count),
            paid: 0,
            unpaid: 0,
        }
    }

    pub fn coin_paid(&self, coin_count: u8) -> HopperDispenseStatus {
        HopperDispenseStatus {
            event_counter: self.next_event_counter(),
            coins_remaining: self.coins_remaining.saturating_sub(coin_count),
            paid: self.paid.saturating_add(coin_count),
            unpaid: self.unpaid,
        }
    }

    pub fn coin_unpaid(&self, coin_count: u8) -> HopperDispenseStatus {
        HopperDispenseStatus {
            event_counter: self.next_event_counter(),
            coins_remaining: self.coins_remaining.saturating_sub(coin_count),
            paid: self.paid,
            unpaid: self.unpaid.saturating_add(coin_count),
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct HopperDispenseValueStatus {
    pub event_counter: u8,
    pub value_remaining: u16,
    pub paid: u16,
    pub unpaid: u16,
}

impl HopperDispenseValueStatus {
    pub fn new(event_counter: u8, remaining: u16, paid: u16, unpaid: u16) -> Self {
        Self {
            event_counter,
            value_remaining: remaining,
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

    pub fn payout_requested(&self, value: u16) -> HopperDispenseValueStatus {
        HopperDispenseValueStatus {
            event_counter: self.next_event_counter(),
            value_remaining: self.value_remaining.saturating_add(value),
            paid: 0,
            unpaid: 0,
        }
    }

    pub fn paid(&self, value: u16) -> HopperDispenseValueStatus {
        HopperDispenseValueStatus {
            event_counter: self.next_event_counter(),
            value_remaining: self.value_remaining.saturating_sub(value),
            paid: self.paid.saturating_add(value),
            unpaid: self.unpaid,
        }
    }

    pub fn unpaid(&self, value: u16) -> HopperDispenseValueStatus {
        HopperDispenseValueStatus {
            event_counter: self.next_event_counter(),
            value_remaining: self.value_remaining.saturating_sub(value),
            paid: self.paid,
            unpaid: self.unpaid.saturating_add(value),
        }
    }
}

impl core::convert::From<[u8; 7]> for HopperDispenseValueStatus {
    fn from(status: [u8; 7]) -> Self {
        let value = u16::from_le_bytes([status[1], status[2]]);
        let paid = u16::from_le_bytes([status[3], status[4]]);
        let unpaid = u16::from_le_bytes([status[5], status[6]]);
        Self {
            event_counter: status[0],
            value_remaining: value,
            paid,
            unpaid,
        }
    }
}

impl core::convert::From<HopperDispenseValueStatus> for [u8; 7] {
    fn from(status: HopperDispenseValueStatus) -> Self {
        let remaining = status.value_remaining.to_le_bytes();
        let paid = status.paid.to_le_bytes();
        let unpaid = status.unpaid.to_le_bytes();
        [
            status.event_counter,
            remaining[0],
            remaining[1],
            paid[0],
            paid[1],
            unpaid[0],
            unpaid[1],
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
