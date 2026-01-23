//! Payout Pool for managing multiple hopper devices.
//!
//! This module provides a unified interface for managing multiple payout
//! devices (hoppers) as a single pool. It handles:
//!
//! - Coordinated initialization and enable/disable control
//! - Inventory level monitoring via sensors
//! - Value-based payout with automatic hopper selection
//! - Greedy algorithm for optimal coin selection
//! - Automatic replanning when hoppers run empty
//! - Emergency stop coordination
//!
//! # Example
//!
//! ```ignore
//! use std::time::Duration;
//!
//! let pool = PayoutPool::builder()
//!     .add_hopper(hopper1, 100)  // 1.00 EUR
//!     .add_hopper(hopper2, 50)   // 0.50 EUR
//!     .add_hopper(hopper3, 20)   // 0.20 EUR
//!     .with_selection_strategy(HopperSelectionStrategy::LargestFirst)
//!     .with_polling_interval(Duration::from_millis(250))
//!     .build_and_initialize()
//!     .await?;
//!
//! // Dispense 1.70 EUR worth of coins
//! let result = pool.payout(170).await?;
//! println!("Dispensed {} cents", result.dispensed);
//! ```

mod builder;
mod config;
mod error;
mod poll_result;
mod pool;

pub use builder::PayoutPoolBuilder;
pub use config::HopperSelectionStrategy;
pub use error::{PayoutPoolError, PayoutPoolResult};
pub use poll_result::{
    DispenseProgress, HopperInventory, HopperInventoryLevel, HopperPollError, PayoutPollResult,
};
pub use pool::PayoutPool;
