use crate::device::base::CommandError;
use thiserror::Error;

/// Errors that can occur when operating the payout pool.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum PayoutPoolError {
    /// The pool has not been initialized.
    #[error("pool not initialized")]
    NotInitialized,

    /// The pool has already been initialized.
    #[error("pool already initialized")]
    AlreadyInitialized,

    /// The pool has no hoppers configured.
    #[error("pool has no hoppers")]
    NoHoppers,

    /// Cannot dispense the requested amount with available hoppers.
    #[error("insufficient hoppers: requested {requested}, can dispense {available}")]
    InsufficientHoppers { requested: u32, available: u32 },

    /// The specified hopper was not found in the pool.
    #[error("hopper not found: address {0}")]
    HopperNotFound(u8),

    /// A command to a hopper device failed.
    #[error("hopper {address} command error: {error}")]
    CommandError { address: u8, error: CommandError },

    /// The payout operation timed out.
    #[error("payout timeout: dispensed {dispensed} of {requested} requested")]
    Timeout { requested: u32, dispensed: u32 },

    /// The payout was stopped by an emergency stop command.
    #[error("emergency stop: dispensed {dispensed} of {requested} requested")]
    EmergencyStopped { requested: u32, dispensed: u32 },

    /// A payout operation is already in progress.
    #[error("payout already in progress")]
    PayoutInProgress,

    /// All hoppers failed during the operation.
    #[error("all hoppers failed")]
    AllHoppersFailed,

    /// The specified hopper is disabled.
    #[error("hopper {0} is disabled")]
    HopperDisabled(u8),
}

impl From<CommandError> for PayoutPoolError {
    fn from(error: CommandError) -> Self {
        Self::CommandError { address: 0, error }
    }
}

/// Result type for payout pool operations.
pub type PayoutPoolResult<T> = Result<T, PayoutPoolError>;
