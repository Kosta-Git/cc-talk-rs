/// Bill validator events
#[derive(Debug, Clone, PartialEq, Eq)]
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
