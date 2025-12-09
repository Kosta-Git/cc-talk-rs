#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ChangerPollResult {
    pub event_counter: u8,
    pub paid: u32,
    pub unpaid: u32,
}
impl ChangerPollResult {
    #[must_use]
    pub const fn new(event_counter: u8, paid: u32, unpaid: u32) -> Self {
        Self {
            event_counter,
            paid,
            unpaid,
        }
    }

    #[must_use]
    pub const fn next_event_counter(&self) -> u8 {
        if self.event_counter == u8::MAX {
            1
        } else {
            self.event_counter.wrapping_add(1)
        }
    }
}

impl TryFrom<&[u8]> for ChangerPollResult {
    type Error = &'static str;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.len() != 9 {
            return Err("Invalid length for ChangerPollResult");
        }
        Ok(Self {
            event_counter: value[0],
            paid: u32::from_le_bytes([value[1], value[2], value[3], value[4]]),
            unpaid: u32::from_le_bytes([value[5], value[6], value[7], value[8]]),
        })
    }
}
