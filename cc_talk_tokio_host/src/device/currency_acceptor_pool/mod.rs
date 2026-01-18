//! Currency Acceptor Pool for managing multiple coin and bill validators.
//!
//! This module provides a unified interface for managing multiple currency acceptor
//! devices (coin validators and bill validators) as a single pool. It handles:
//!
//! - Coordinated initialization and inhibit control
//! - Unified polling across all devices
//! - Denomination filtering by value range
//! - Bill escrow handling with configurable routing modes
//! - Value-based payment acceptance
//!
//! # Example
//!
//! ```ignore
//! use std::time::Duration;
//!
//! let pool = CurrencyAcceptorPool::builder()
//!     .add_coin_validator(coin_validator)
//!     .add_bill_validator(bill_validator)
//!     .with_denomination_range(50, 10000)  // Accept 0.50 to 100.00
//!     .with_bill_routing_mode(BillRoutingMode::AutoStack)
//!     .with_polling_interval(Duration::from_millis(100))
//!     .build_and_initialize()
//!     .await?;
//!
//! // Accept a payment of 500 cents (5.00)
//! let result = pool.accept_payment(500, Duration::from_secs(30)).await?;
//! println!("Received {} cents", result.total_received);
//! ```

mod builder;
mod config;
mod device_id;
mod poll_result;
mod pool;

pub use builder::CurrencyAcceptorPoolBuilder;
pub use config::{BillRoutingMode, DenominationRange, DeviceValueMap};
pub use device_id::DeviceId;
pub use poll_result::{CurrencyCredit, PendingBill, PoolPollError, PoolPollResult};
pub use pool::{CurrencyAcceptorPool, PaymentProgress, PaymentResult};

use crate::device::base::{CommandError, PollingError};
use thiserror::Error;

/// Errors that can occur when operating the currency acceptor pool.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum PoolError {
    /// A command to a device failed.
    #[error("device command error: {0}")]
    CommandError(#[from] CommandError),

    /// Background polling is already active.
    #[error("polling error: {0}")]
    PollingError(#[from] PollingError),

    /// The pool has no devices configured.
    #[error("pool has no devices")]
    NoDevices,

    /// Failed to read currency ID from a device position.
    #[error("failed to read currency ID at position {position} from {device}")]
    CurrencyIdReadFailed { device: String, position: u8 },

    /// A bill routing operation failed.
    #[error("bill routing failed: {0}")]
    BillRoutingFailed(String),

    /// Payment was not completed within the timeout.
    #[error("payment timeout: received {received} of {target} requested")]
    PaymentTimeout {
        target: u32,
        received: u32,
        credits: Vec<CurrencyCredit>,
    },

    /// Payment was cancelled.
    #[error("payment cancelled: received {received} of {target} requested")]
    PaymentCancelled {
        target: u32,
        received: u32,
        credits: Vec<CurrencyCredit>,
    },

    /// All devices failed during a critical operation.
    #[error("all devices failed")]
    AllDevicesFailed,
}

/// Result type for pool operations.
pub type PoolResult<T> = Result<T, PoolError>;
