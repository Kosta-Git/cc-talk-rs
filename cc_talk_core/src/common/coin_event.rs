use crate::CoinAcceptorError;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg(feature = "defmt")]
#[derive(defmt::Format)]
pub enum SorterPath {
    NotSupported,
    Path(u8),
}
impl TryFrom<u8> for SorterPath {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(SorterPath::NotSupported),
            _ => Ok(SorterPath::Path(value)),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg(feature = "defmt")]
#[derive(defmt::Format)]
pub struct CoinCredit {
    pub credit: u8,
    pub sorter_path: SorterPath,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg(feature = "defmt")]
#[derive(defmt::Format)]
pub enum CoinEvent {
    Error(CoinAcceptorError),
    Credit(CoinCredit),
}
impl CoinEvent {
    pub fn new(result_a: u8, result_b: u8) -> Self {
        match result_a {
            0 => CoinEvent::Error(CoinAcceptorError::try_from(result_b).unwrap()),
            _ => CoinEvent::Credit(CoinCredit {
                credit: result_a,
                sorter_path: SorterPath::try_from(result_b).unwrap_or(SorterPath::NotSupported),
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
