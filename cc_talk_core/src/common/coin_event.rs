use crate::cc_talk::CoinAcceptorError;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum SorterPath {
    NotSupported,
    Path(u8),
}

impl From<u8> for SorterPath {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::NotSupported,
            _ => Self::Path(value),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CoinCredit {
    pub credit: u8,
    pub sorter_path: SorterPath,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum CoinEvent {
    Error(CoinAcceptorError),
    Credit(CoinCredit),
    Reset,
}
impl CoinEvent {
    #[must_use]
    pub fn new(result_a: u8, result_b: u8) -> Self {
        match result_a {
            0 => Self::Error(
                CoinAcceptorError::try_from(result_b).unwrap_or(CoinAcceptorError::NullEvent),
            ),
            _ => Self::Credit(CoinCredit {
                credit: result_a,
                sorter_path: SorterPath::from(result_b),
            }),
        }
    }

    #[must_use]
    pub const fn is_error(&self) -> bool {
        matches!(self, Self::Error(_))
    }

    #[must_use]
    pub const fn is_credit(&self) -> bool {
        matches!(self, Self::Credit(_))
    }
}

const MAX_COIN_EVENT_SIZE: usize = 5;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoinAcceptorPollResult {
    pub event_counter: u8,
    pub lost_events: u8,
    pub events: heapless::Vec<CoinEvent, MAX_COIN_EVENT_SIZE>,
}
impl CoinAcceptorPollResult {
    #[must_use]
    pub const fn new(event_counter: u8) -> Self {
        Self {
            event_counter,
            events: heapless::Vec::new(),
            lost_events: 0,
        }
    }

    pub fn add_event(&mut self, event: CoinEvent) {
        self.events.push(event).ok();
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CoinAcceptorPollResultError {
    NotEnoughEvents,
    TooManyEvents,
    InvalidPayload,
}
impl TryFrom<(&[u8], u8)> for CoinAcceptorPollResult {
    type Error = CoinAcceptorPollResultError;

    #[allow(clippy::cast_possible_truncation)]
    fn try_from(value: (&[u8], u8)) -> Result<Self, Self::Error> {
        let (value, event_counter) = value;
        if value.is_empty() {
            return Err(CoinAcceptorPollResultError::InvalidPayload);
        }

        let received_event_counter = value[0];

        if received_event_counter == 0 {
            let mut events = heapless::Vec::new();
            events.push(CoinEvent::Reset).ok();
            return Ok(Self {
                event_counter,
                events,
                lost_events: 0,
            });
        }

        let events_to_parse = if received_event_counter >= event_counter {
            received_event_counter - event_counter
        } else {
            (255 - event_counter) + received_event_counter
        };

        let lost_events = events_to_parse.saturating_sub(MAX_COIN_EVENT_SIZE as u8);

        let events_to_parse = if events_to_parse > MAX_COIN_EVENT_SIZE as u8 {
            MAX_COIN_EVENT_SIZE as u8
        } else {
            events_to_parse
        };

        let expected_len = (events_to_parse as usize * 2) + 1;
        if value.len() < expected_len {
            return Err(CoinAcceptorPollResultError::NotEnoughEvents);
        }

        let mut events = heapless::Vec::new();
        for i in 0..events_to_parse {
            let index_base = (i * 2) as usize + 1;
            let result_a = value[index_base];
            let result_b = value[index_base + 1];
            let _ = events.insert(i as usize, CoinEvent::new(result_a, result_b));
        }

        Ok(Self {
            event_counter: received_event_counter,
            lost_events,
            events,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_zero_events() {
        let buffer = [0u8];

        let result =
            CoinAcceptorPollResult::try_from((&buffer[..], 0)).expect("should parse zero events");

        assert_eq!(result.event_counter, 0);
        assert_eq!(result.lost_events, 0);
        assert_eq!(result.events.len(), 1);
        assert_eq!(result.events[0], CoinEvent::Reset);
    }

    #[test]
    fn should_error_on_empty() {
        let buffer = [];
        let result = CoinAcceptorPollResult::try_from((&buffer[..], 0));
        assert_eq!(
            result,
            Err(CoinAcceptorPollResultError::InvalidPayload),
            "should error on empty buffer"
        );
    }

    #[test]
    fn event_lost() {
        let buffer = [6u8, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5]; // One event lost
        let result =
            CoinAcceptorPollResult::try_from((&buffer[..], 0)).expect("should parse events");
        assert_eq!(result.lost_events, 1, "one event should be lost");
        assert_eq!(result.event_counter, 6, "event counter should be 6");
        assert_eq!(result.events.len(), 5, "should have 5 events");
    }

    #[test]
    fn error_on_unexpected_len() {
        let buffer = [3u8, 1, 1, 2, 2]; // We expect 3 events, but only 4 bytes provided
        let result = CoinAcceptorPollResult::try_from((&buffer[..], 0));
        assert_eq!(
            result,
            Err(CoinAcceptorPollResultError::NotEnoughEvents),
            "should error on unexpected length"
        );
    }

    #[test]
    fn parse_events() {
        let buffer = [3u8, 1, 2, 3, 4, 5, 6]; // We expect 3 events
        let result =
            CoinAcceptorPollResult::try_from((&buffer[..], 0)).expect("should parse three events");

        assert_eq!(result.event_counter, 3);
        assert_eq!(result.events.len(), 3);
        assert_eq!(
            result.events[0],
            CoinEvent::Credit(CoinCredit {
                credit: 1,
                sorter_path: SorterPath::Path(2)
            })
        );
        assert_eq!(
            result.events[1],
            CoinEvent::Credit(CoinCredit {
                credit: 3,
                sorter_path: SorterPath::Path(4)
            })
        );
        assert_eq!(
            result.events[2],
            CoinEvent::Credit(CoinCredit {
                credit: 5,
                sorter_path: SorterPath::Path(6)
            })
        );
    }
}
