#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ChangerError {
    /// Requires refill
    #[error("hopper is empty - requires refill")]
    HopperEmpty = 1,
    /// Remove hopper shelf and clear jam
    #[error("hopper jam - remove hopper shelf and clear jam")]
    HopperJam = 2,
    /// Alert security
    #[error("hopper fraud detected - alert security")]
    HopperFraud = 3,
    /// Service callout
    #[error("hopper fault - service callout required")]
    HopperFault = 4,
    /// Remove coin acceptor and clear jam
    #[error("coin acceptor jam - remove coin acceptor and clear jam")]
    CoinAcceptorJam = 101,
    /// Alert security
    #[error("coin acceptor fraud attempt - alert security")]
    CoinAcceptorFraudAttempt = 102,
    /// service callout
    #[error("coin acceptor fault - service callout required")]
    CoinAcceptorFault = 103,
    /// check connector
    #[error("coin acceptor to manifold opto fault - check connector")]
    CoinAcceptorToManifoldOptoFault = 104,
    /// Empty cashbox, money time!!!
    #[error("cashbox is full - empty cashbox")]
    CashboxFull = 251,
    /// Insert cashbox
    #[error("cashbox is missing - insert cashbox")]
    CashboxMissing = 252,
    #[error("other changer error")]
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
