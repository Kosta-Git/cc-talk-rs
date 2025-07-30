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
            0 => SorterPath::NotSupported,
            _ => SorterPath::Path(value),
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
}
impl CoinEvent {
    pub fn new(result_a: u8, result_b: u8) -> Self {
        match result_a {
            0 => CoinEvent::Error(
                CoinAcceptorError::try_from(result_b).unwrap_or(CoinAcceptorError::NullEvent),
            ),
            _ => CoinEvent::Credit(CoinCredit {
                credit: result_a,
                sorter_path: SorterPath::from(result_b),
            }),
        }
    }

    pub fn is_error(&self) -> bool {
        matches!(self, CoinEvent::Error(_))
    }

    pub fn is_credit(&self) -> bool {
        matches!(self, CoinEvent::Credit(_))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoinAcceptorPollResult {
    pub event_counter: u8,
    pub events: heapless::Vec<CoinEvent, 5>,
}
impl CoinAcceptorPollResult {
    pub fn new(event_counter: u8) -> Self {
        CoinAcceptorPollResult {
            event_counter,
            events: heapless::Vec::new(),
        }
    }

    pub fn add_event(&mut self, event: CoinEvent) {
        self.events.push(event).ok();
    }

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
impl TryFrom<&[u8]> for CoinAcceptorPollResult {
    type Error = CoinAcceptorPollResultError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.is_empty() {
            return Err(CoinAcceptorPollResultError::InvalidPayload);
        }

        let announced_events = value[0];
        if announced_events > 5 {
            return Err(CoinAcceptorPollResultError::TooManyEvents);
        }

        let expected_len = (announced_events as usize * 2) + 1;
        if value.len() < expected_len {
            return Err(CoinAcceptorPollResultError::NotEnoughEvents);
        }

        let mut events = heapless::Vec::new();
        for i in 0..announced_events {
            let index_base = (i * 2) as usize + 1;
            let result_a = value[index_base];
            let result_b = value[index_base + 1];
            let _ = events.insert(i as usize, CoinEvent::new(result_a, result_b));
        }

        Ok(CoinAcceptorPollResult {
            event_counter: announced_events,
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
            CoinAcceptorPollResult::try_from(&buffer[..]).expect("should parse zero events");

        assert_eq!(result.event_counter, 0);
        assert!(result.is_empty());
    }

    #[test]
    fn should_error_on_empty() {
        let buffer = [];
        let result = CoinAcceptorPollResult::try_from(&buffer[..]);
        assert_eq!(
            result,
            Err(CoinAcceptorPollResultError::InvalidPayload),
            "should error on empty buffer"
        );
    }

    #[test]
    fn too_many_events_errors() {
        let buffer = [6u8]; // We expect 6 events, even if 1byte is provided
        let result = CoinAcceptorPollResult::try_from(&buffer[..]);
        assert_eq!(
            result,
            Err(CoinAcceptorPollResultError::TooManyEvents),
            "should error on >5 events"
        );
    }

    #[test]
    fn error_on_unexpected_len() {
        let buffer = [3u8, 1, 2, 3, 4]; // We expect 3 events, but only 4 bytes provided
        let result = CoinAcceptorPollResult::try_from(&buffer[..]);
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
            CoinAcceptorPollResult::try_from(&buffer[..]).expect("should parse three events");

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
