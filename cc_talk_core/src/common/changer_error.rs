#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ChangerError {
    /// Requires refill
    HopperEmpty = 1,
    /// Remove hopper shelf and clear jam
    HopperJam = 2,
    /// Alert security
    HopperFraud = 3,
    /// Service callout
    HopperFault = 4,
    /// Remove coin acceptor and clear jam
    CoinAcceptorJam = 101,
    /// Alert security
    CoinAcceptorFraudAttempt = 102,
    /// service callout
    CoinAcceptorFault = 103,
    /// check connector
    CoinAcceptorToManifoldOptoFault = 104,
    /// Empty cashbox, money time!!!
    CashboxFull = 251,
    /// Insert cashbox
    CashboxMissing = 252,
    Other = 255,
}

impl From<ChangerError> for u8 {
    fn from(error: ChangerError) -> Self {
        error as Self
    }
}

impl From<u8> for ChangerError {
    fn from(value: u8) -> Self {
        match value {
            1 => Self::HopperEmpty,
            2 => Self::HopperJam,
            3 => Self::HopperFraud,
            4 => Self::HopperFault,
            101 => Self::CoinAcceptorJam,
            102 => Self::CoinAcceptorFraudAttempt,
            103 => Self::CoinAcceptorFault,
            104 => Self::CoinAcceptorToManifoldOptoFault,
            251 => Self::CashboxFull,
            252 => Self::CashboxMissing,
            _ => Self::Other,
        }
    }
}
