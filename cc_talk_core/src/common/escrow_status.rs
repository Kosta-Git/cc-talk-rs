#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum EscrowOperatingStatus {
    Idle = 0,
    Operating = 1,
    FaultCondition = 2,
}

impl TryFrom<u8> for EscrowOperatingStatus {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(EscrowOperatingStatus::Idle),
            1 => Ok(EscrowOperatingStatus::Operating),
            2 => Ok(EscrowOperatingStatus::FaultCondition),
            _ => Err("Invalid value for EscrowOperatingStatus"),
        }
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum EscrowLevelStatus {
    EmptyOrNotFull = 0,
    Full = 255,
}

impl TryFrom<u8> for EscrowLevelStatus {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(EscrowLevelStatus::EmptyOrNotFull),
            255 => Ok(EscrowLevelStatus::Full),
            _ => Err("Invalid value for EscrowLevelStatus"),
        }
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum EscrowFaultCode {
    NoFault = 0,
    FailureToOpenAcceptFlap = 10,
    FailureToOpenAcceptFlapFromHome = 11,
    FailureToOpenAcceptFlapFromUnknown = 12,
    FailureToCloseAcceptFlap = 20,
    FailureToCloseAcceptFlapAfterAccept = 21,
    FailureToCloseAcceptFlapAfterFailedAccept = 22,
    FailureToCloseAcceptFlapAfterFromUnknown = 23,
    FailureToOpenReturnFlap = 30,
    FailureToOpenReturnFlapFromHome = 31,
    FailureToOpenReturnFlapFromUnknown = 32,
    FailureToCloseReturnFlap = 40,
    FailureToCloseReturnFlapAfterReturn = 41,
    FailureToCloseReturnFlapAfterFailedReturn = 42,
    FailureToCloseReturnFlapFromUnknown = 43,
    SupplyUnderVoltage = 100,
    SupplyOverVoltage = 101,
    FraudulentManipulationDetected = 200,
    OverCurrentOrJammed = 250,
    Other = 255,
}

impl From<u8> for EscrowFaultCode {
    fn from(value: u8) -> Self {
        match value {
            0 => EscrowFaultCode::NoFault,
            10 => EscrowFaultCode::FailureToOpenAcceptFlap,
            11 => EscrowFaultCode::FailureToOpenAcceptFlapFromHome,
            12 => EscrowFaultCode::FailureToOpenAcceptFlapFromUnknown,
            20 => EscrowFaultCode::FailureToCloseAcceptFlap,
            21 => EscrowFaultCode::FailureToCloseAcceptFlapAfterAccept,
            22 => EscrowFaultCode::FailureToCloseAcceptFlapAfterFailedAccept,
            23 => EscrowFaultCode::FailureToCloseAcceptFlapAfterFromUnknown,
            30 => EscrowFaultCode::FailureToOpenReturnFlap,
            31 => EscrowFaultCode::FailureToOpenReturnFlapFromHome,
            32 => EscrowFaultCode::FailureToOpenReturnFlapFromUnknown,
            40 => EscrowFaultCode::FailureToCloseReturnFlap,
            41 => EscrowFaultCode::FailureToCloseReturnFlapAfterReturn,
            42 => EscrowFaultCode::FailureToCloseReturnFlapAfterFailedReturn,
            43 => EscrowFaultCode::FailureToCloseReturnFlapFromUnknown,
            100 => EscrowFaultCode::SupplyUnderVoltage,
            101 => EscrowFaultCode::SupplyOverVoltage,
            200 => EscrowFaultCode::FraudulentManipulationDetected,
            250 => EscrowFaultCode::OverCurrentOrJammed,
            _ => EscrowFaultCode::Other,
        }
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum EscrowServiceStatus {
    None = 0,
    Recommended = 1,
    Overdue = 2,
}

impl TryFrom<u8> for EscrowServiceStatus {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(EscrowServiceStatus::None),
            1 => Ok(EscrowServiceStatus::Recommended),
            2 => Ok(EscrowServiceStatus::Overdue),
            _ => Err("Invalid value for EscrowServiceStatus"),
        }
    }
}

