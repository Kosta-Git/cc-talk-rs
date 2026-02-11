//! Standalone sensor monitoring for payout hopper devices.
//!
//! This module provides background sensor polling for a set of
//! [`PayoutDevice`](super::payout::PayoutDevice) instances, independent of
//! [`PayoutPool`](super::payout_pool::PayoutPool). It handles:
//!
//! - Continuous background polling of hopper sensor levels
//! - Inventory level change detection with event notifications
//! - Manual and automatic empty-state tracking per hopper
//! - Auto-recovery when a previously-empty hopper is refilled
//!
//! # Example
//!
//! ```ignore
//! use std::time::Duration;
//!
//! let sensor_pool = PayoutSensorPool::builder()
//!     .add_hopper(hopper1)
//!     .add_hopper(hopper2)
//!     .add_hopper(hopper3)
//!     .polling_interval(Duration::from_millis(500))
//!     .build();
//!
//! // Start background polling — returns a guard with an event receiver
//! let mut guard = sensor_pool.try_start_polling()?;
//!
//! while let Some(event) = guard.recv().await {
//!     match event {
//!         SensorEvent::LevelChanged { address, previous, current, .. } => {
//!             println!("Hopper {address}: {previous} -> {current}");
//!         }
//!         SensorEvent::InventoryUpdate { inventories, errors } => {
//!             for reading in &inventories {
//!                 println!("Hopper {}: {}", reading.address, reading.level);
//!             }
//!         }
//!         _ => {}
//!     }
//! }
//! // Polling stops automatically when the guard is dropped.
//! ```

mod builder;
mod error;
mod event;
mod pool;

pub use builder::PayoutSensorPoolBuilder;
pub use error::{PayoutSensorPoolError, PayoutSensorPoolResult};
pub use event::{HopperSensorError, HopperSensorReading, RecoveryReason, SensorEvent};
pub use pool::{PayoutSensorPool, SensorPollGuard};
