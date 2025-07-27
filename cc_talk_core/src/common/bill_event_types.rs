/// Bill validator events
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum BillEvent {
    /// Bill correctly sent to cashbox/escrow.
    /// Contains the bill type as u8.
    Credit(u8),
    /// Bill validated and held in escrow.
    /// Contains the bill type as u8.
    PendingCredit(u8),
    /// Bill was rejected, reason as [crate::common::bill_event_types::BillEvent].
    Reject(BillEventReason),
    /// Fraud attempt detected, reason as [crate::common::bill_event_types::BillEvent].
    FraudAttempt(BillEventReason),
    /// Fatal error, reason as [crate::common::bill_event_types::BillEvent].
    FatalError(BillEventReason),
    /// General status update, reason as [crate::common::bill_event_types::BillEvent].
    Status(BillEventReason),
}

impl core::fmt::Display for BillEvent {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            BillEvent::Credit(bill_type) => write!(f, "Credit: {}", bill_type),
            BillEvent::PendingCredit(bill_type) => write!(f, "Pending Credit: {}", bill_type),
            BillEvent::Reject(reason) => write!(f, "Reject: {}", reason),
            BillEvent::FraudAttempt(reason) => write!(f, "Fraud Attempt: {}", reason),
            BillEvent::FatalError(reason) => write!(f, "Fatal Error: {}", reason),
            BillEvent::Status(reason) => write!(f, "Status: {}", reason),
        }
    }
}

impl BillEvent {
    /// Takes a single event from ReadBufferedBillEvents (two bytes result A and result B)
    /// and returns an Option<BillEvent>.
    pub fn from_result(a: u8, b: u8) -> Option<BillEvent> {
        match a {
            1..=255 => BillEvent::when_result_a(a, b),
            0 => BillEvent::when_result_b(b),
        }
    }

    fn when_result_a(a: u8, b: u8) -> Option<BillEvent> {
        match b {
            0 => Some(BillEvent::Credit(a)),
            1 => Some(BillEvent::PendingCredit(a)),
            _ => None,
        }
    }

    fn when_result_b(b: u8) -> Option<BillEvent> {
        use BillEvent::*;
        use BillEventReason::*;

        match b {
            0 => Some(Status(MasterInhibitActive)),
            1 => Some(Status(BillReturnedFromEscrow)),
            2 => Some(Reject(InvalidBillValidationFailed)),
            3 => Some(Reject(InvalidBillTransportFailed)),
            4 => Some(Reject(InhibitedBillViaSerial)),
            5 => Some(Reject(InhibitedBillViaDipSwitch)),
            6 => Some(FatalError(BillJammedInTrasport)),
            7 => Some(FatalError(BillJammedInStacker)),
            8 => Some(FraudAttempt(BillPulledBackwards)),
            9 => Some(FraudAttempt(BillTamper)),
            10 => Some(Status(StackerOk)),
            11 => Some(Status(StackerRemoved)),
            12 => Some(Status(StackerInserted)),
            13 => Some(FatalError(StackerFaulty)),
            14 => Some(Status(StackerFull)),
            15 => Some(FatalError(StackerJammed)),
            16 => Some(FatalError(BillJammedInTransportSafe)),
            17 => Some(FraudAttempt(OptoFraudDetected)),
            18 => Some(FraudAttempt(StringFraudDetected)),
            19 => Some(FatalError(AntiStringMechanismFaulty)),
            20 => Some(Status(BarCodeDetected)),
            21 => Some(Status(UnknownBillTypeStacked)),
            _ => None,
        }
    }
}

/// Bill event in case the event type is not Credit or PendingCredit.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum BillEventReason {
    MasterInhibitActive = 0,
    BillReturnedFromEscrow = 1,
    InvalidBillValidationFailed = 2,
    InvalidBillTransportFailed = 3,
    InhibitedBillViaSerial = 4,
    InhibitedBillViaDipSwitch = 5,
    BillJammedInTrasport = 6,
    BillJammedInStacker = 7,
    BillPulledBackwards = 8,
    BillTamper = 9,
    StackerOk = 10,
    StackerRemoved = 11,
    StackerInserted = 12,
    StackerFaulty = 13,
    StackerFull = 14,
    StackerJammed = 15,
    BillJammedInTransportSafe = 16,
    OptoFraudDetected = 17,
    StringFraudDetected = 18,
    AntiStringMechanismFaulty = 19,
    BarCodeDetected = 20,
    UnknownBillTypeStacked = 21,
}

impl core::fmt::Display for BillEventReason {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            BillEventReason::MasterInhibitActive => write!(f, "Master inhibit active"),
            BillEventReason::BillReturnedFromEscrow => write!(f, "Bill returned from escrow"),
            BillEventReason::InvalidBillValidationFailed => {
                write!(f, "Invalid bill validation failed")
            }
            BillEventReason::InvalidBillTransportFailed => {
                write!(f, "Invalid bill transport failed")
            }
            BillEventReason::InhibitedBillViaSerial => write!(f, "Inhibited bill via serial"),
            BillEventReason::InhibitedBillViaDipSwitch => {
                write!(f, "Inhibited bill via dip switch")
            }
            BillEventReason::BillJammedInTrasport => write!(f, "Bill jammed in transport"),
            BillEventReason::BillJammedInStacker => write!(f, "Bill jammed in stacker"),
            BillEventReason::BillPulledBackwards => write!(f, "Bill pulled backwards"),
            BillEventReason::BillTamper => write!(f, "Bill tamper detected"),
            BillEventReason::StackerOk => write!(f, "Stacker ok"),
            BillEventReason::StackerRemoved => write!(f, "Stacker removed"),
            BillEventReason::StackerInserted => write!(f, "Stacker inserted"),
            BillEventReason::StackerFaulty => write!(f, "Stacker faulty"),
            BillEventReason::StackerFull => write!(f, "Stacker full"),
            BillEventReason::StackerJammed => write!(f, "Stacker jammed"),
            BillEventReason::BillJammedInTransportSafe => {
                write!(f, "Bill jammed in transport safe")
            }
            BillEventReason::OptoFraudDetected => write!(f, "Opto fraud detected"),
            BillEventReason::StringFraudDetected => write!(f, "String fraud detected"),
            BillEventReason::AntiStringMechanismFaulty => write!(f, "Anti-string mechanism faulty"),
            BillEventReason::BarCodeDetected => write!(f, "Bar code detected"),
            BillEventReason::UnknownBillTypeStacked => write!(f, "Unknown bill type stacked"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BillValidatorPollResult {
    pub event_counter: u8,
    pub events: heapless::Vec<BillEvent, 5>,
}
impl BillValidatorPollResult {
    pub fn new(event_counter: u8) -> Self {
        BillValidatorPollResult {
            event_counter,
            events: heapless::Vec::new(),
        }
    }

    pub fn add_event(&mut self, event: BillEvent) {
        self.events.push(event).ok();
    }

    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BillValidatorPollResultError {
    NotEnoughEvents,
    TooManyEvents,
    InvalidPayload,
}
impl TryFrom<&[u8]> for BillValidatorPollResult {
    type Error = BillValidatorPollResultError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.is_empty() {
            return Err(BillValidatorPollResultError::InvalidPayload);
        }

        let announced_events = value[0];
        if announced_events > 5 {
            return Err(BillValidatorPollResultError::TooManyEvents);
        }

        let expected_len = (announced_events as usize * 2) + 1;
        if value.len() != expected_len {
            return Err(BillValidatorPollResultError::NotEnoughEvents);
        }

        let mut events = heapless::Vec::new();
        for i in 0..announced_events {
            let index_base = (i * 2) as usize + 1;
            let result_a = value[index_base];
            let result_b = value[index_base + 1];
            if let Some(event) = BillEvent::from_result(result_a, result_b) {
                events.push(event).ok();
            }
        }

        Ok(BillValidatorPollResult {
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
            BillValidatorPollResult::try_from(&buffer[..]).expect("Failed to parse zero events");
        assert_eq!(result.event_counter, 0);
        assert!(result.is_empty());
    }

    #[test]
    fn should_error_on_empty() {
        let buffer = [];
        let result = BillValidatorPollResult::try_from(&buffer[..]);
        assert!(matches!(
            result,
            Err(BillValidatorPollResultError::InvalidPayload)
        ));
    }

    #[test]
    fn too_many_events_errors() {
        let buffer = [6u8];
        let result = BillValidatorPollResult::try_from(&buffer[..]);
        assert!(matches!(
            result,
            Err(BillValidatorPollResultError::TooManyEvents)
        ));
    }

    #[test]
    fn error_on_unexpected_len() {
        let buffer = [3u8, 1, 2, 3, 4];
        let result = BillValidatorPollResult::try_from(&buffer[..]);
        assert!(matches!(
            result,
            Err(BillValidatorPollResultError::NotEnoughEvents)
        ));
    }

    #[test]
    fn prse_events() {
        let buffer = [
            5u8, //  5 events
            1, 0, // credit 1
            255, 1, // pending credit 255
            0, 1, // status returned from escrow
            0, 2, // rject due to validation fail
            0, 19, // fatal error anti string mechanism faulty
        ];

        let result =
            BillValidatorPollResult::try_from(&buffer[..]).expect("Failed to parse events");

        assert_eq!(result.event_counter, 5);
        assert_eq!(result.events.len(), 5);
        assert_eq!(result.events[0], BillEvent::Credit(1));
        assert_eq!(result.events[1], BillEvent::PendingCredit(255));
        assert_eq!(
            result.events[2],
            BillEvent::Status(BillEventReason::BillReturnedFromEscrow)
        );
        assert_eq!(
            result.events[3],
            BillEvent::Reject(BillEventReason::InvalidBillValidationFailed)
        );
        assert_eq!(
            result.events[4],
            BillEvent::FatalError(BillEventReason::AntiStringMechanismFaulty)
        );
    }
}
