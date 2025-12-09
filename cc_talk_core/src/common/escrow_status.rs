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
            0 => Ok(Self::Idle),
            1 => Ok(Self::Operating),
            2 => Ok(Self::FaultCondition),
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
            0 => Ok(Self::EmptyOrNotFull),
            255 => Ok(Self::Full),
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
            0 => Self::NoFault,
            10 => Self::FailureToOpenAcceptFlap,
            11 => Self::FailureToOpenAcceptFlapFromHome,
            12 => Self::FailureToOpenAcceptFlapFromUnknown,
            20 => Self::FailureToCloseAcceptFlap,
            21 => Self::FailureToCloseAcceptFlapAfterAccept,
            22 => Self::FailureToCloseAcceptFlapAfterFailedAccept,
            23 => Self::FailureToCloseAcceptFlapAfterFromUnknown,
            30 => Self::FailureToOpenReturnFlap,
            31 => Self::FailureToOpenReturnFlapFromHome,
            32 => Self::FailureToOpenReturnFlapFromUnknown,
            40 => Self::FailureToCloseReturnFlap,
            41 => Self::FailureToCloseReturnFlapAfterReturn,
            42 => Self::FailureToCloseReturnFlapAfterFailedReturn,
            43 => Self::FailureToCloseReturnFlapFromUnknown,
            100 => Self::SupplyUnderVoltage,
            101 => Self::SupplyOverVoltage,
            200 => Self::FraudulentManipulationDetected,
            250 => Self::OverCurrentOrJammed,
            _ => Self::Other,
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
            0 => Ok(Self::None),
            1 => Ok(Self::Recommended),
            2 => Ok(Self::Overdue),
            _ => Err("Invalid value for EscrowServiceStatus"),
        }
    }
}
