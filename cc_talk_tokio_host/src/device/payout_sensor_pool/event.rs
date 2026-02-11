use cc_talk_core::cc_talk::HopperStatus;

use crate::device::{base::CommandError, payout_pool::HopperInventoryLevel};

/// Events emitted by the [`super::PayoutSensorPool`] during background polling.
#[derive(Debug, Clone)]
pub enum SensorEvent {
    /// Periodic inventory snapshot of all hoppers.
    InventoryUpdate {
        /// Successful sensor readings from this polling cycle.
        inventories: Vec<HopperSensorReading>,
        /// Errors encountered while polling individual hoppers.
        errors: Vec<HopperSensorError>,
    },
    /// A hopper's inventory level changed since the last poll.
    LevelChanged {
        /// The ccTalk address of the hopper.
        address: u8,
        /// The previous inventory level.
        previous: HopperInventoryLevel,
        /// The current inventory level.
        current: HopperInventoryLevel,
    },
    /// A hopper was marked empty.
    MarkedEmpty {
        /// The ccTalk address of the hopper.
        address: u8,
    },
    /// A hopper was marked non-empty (manually or by auto-recovery).
    MarkedNonEmpty {
        /// The ccTalk address of the hopper.
        address: u8,
        /// The reason the hopper was marked non-empty.
        reason: RecoveryReason,
    },
}

/// A single hopper's sensor reading.
#[derive(Debug, Clone)]
pub struct HopperSensorReading {
    /// The ccTalk address of the hopper.
    pub address: u8,
    /// The inventory level derived from the sensor status.
    pub level: HopperInventoryLevel,
    /// The raw sensor status from the device.
    pub status: HopperStatus,
}

/// Error polling a specific hopper.
#[derive(Debug, Clone)]
pub struct HopperSensorError {
    /// The address of the hopper that produced the error.
    pub address: u8,
    /// The error that occurred.
    pub error: CommandError,
}

/// Reason a hopper was marked as non-empty.
#[derive(Debug, Clone)]
pub enum RecoveryReason {
    /// User explicitly called `mark_non_empty`.
    Manual,
    /// Sensor detected level at or above the recovery threshold.
    SensorRecovery {
        /// The level that triggered recovery.
        level: HopperInventoryLevel,
    },
}
