use thiserror::Error;

/// Errors returned by [`super::PayoutSensorPool`] operations.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum PayoutSensorPoolError {
    /// The given ccTalk address does not match any hopper in the pool.
    #[error("hopper not found: address {0}")]
    HopperNotFound(u8),
}

/// Convenience alias for results from [`super::PayoutSensorPool`] operations.
pub type PayoutSensorPoolResult<T> = Result<T, PayoutSensorPoolError>;
