use cc_talk_core::cc_talk::HopperStatus;

use crate::device::base::CommandError;

/// Inventory level of a hopper based on sensor readings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HopperInventoryLevel {
    /// Hopper is empty or below low level threshold.
    Empty,
    /// Hopper is above low level but below high level.
    Low,
    /// Hopper is at medium level (above low, below high).
    Medium,
    /// Hopper is above high level threshold.
    High,
    /// Sensor status could not be determined.
    #[default]
    Unknown,
}

impl From<HopperStatus> for HopperInventoryLevel {
    fn from(status: HopperStatus) -> Self {
        if !(status.high_level_supported || status.higher_than_high_level) {
            return Self::Unknown;
        }

        if status.high_level_supported && status.higher_than_high_level {
            Self::High
        } else if status.low_level_supported {
            if status.higher_than_low_level {
                Self::Medium
            } else {
                Self::Low
            }
        } else {
            Self::Empty
        }
    }
}

impl std::fmt::Display for HopperInventoryLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Empty => write!(f, "Empty"),
            Self::Low => write!(f, "Low"),
            Self::Medium => write!(f, "Medium"),
            Self::High => write!(f, "High"),
            Self::Unknown => write!(f, "Unknown"),
        }
    }
}

/// Inventory status of a single hopper.
#[derive(Debug, Clone)]
pub struct HopperInventory {
    /// The ccTalk address of the hopper.
    pub address: u8,
    /// The coin value this hopper dispenses (in smallest currency units).
    pub value: u32,
    /// The inventory level based on sensor readings.
    pub level: HopperInventoryLevel,
    /// The raw sensor status from the device.
    pub status: HopperStatus,
}

impl HopperInventory {
    /// Creates a new hopper inventory status.
    #[must_use]
    pub const fn new(
        address: u8,
        value: u32,
        level: HopperInventoryLevel,
        status: HopperStatus,
    ) -> Self {
        Self {
            address,
            value,
            level,
            status,
        }
    }
}

/// Progress update during a payout operation.
#[derive(Debug, Clone)]
pub struct DispenseProgress {
    /// Total value requested for payout (in smallest currency units).
    pub requested: u32,
    /// Total value dispensed so far.
    pub dispensed: u32,
    /// Individual coin values that have been dispensed.
    pub coins_dispensed: Vec<u32>,
    /// Value remaining to dispense.
    pub remaining: u32,
    /// The hopper currently dispensing (by address), if any.
    pub active_hopper: Option<u8>,
    /// Hoppers that have run empty during this payout (by address).
    pub empty_hoppers: Vec<u8>,
    /// Whether the payout operation is complete.
    pub done: bool,
}

impl DispenseProgress {
    /// Creates a new dispense progress for a requested value.
    #[must_use]
    pub fn new(requested: u32) -> Self {
        Self {
            requested,
            dispensed: 0,
            coins_dispensed: Vec::new(),
            remaining: requested,
            active_hopper: None,
            empty_hoppers: Vec::new(),
            done: false,
        }
    }

    /// Returns the number of coins dispensed so far.
    #[must_use]
    pub fn coins_count(&self) -> usize {
        self.coins_dispensed.len()
    }

    /// Marks a coin as dispensed and updates progress.
    pub fn coin_dispensed(&mut self, value: u32) {
        self.dispensed += value;
        self.remaining = self.requested.saturating_sub(self.dispensed);
        self.coins_dispensed.push(value);
    }

    /// Marks the payout as complete.
    pub fn mark_done(&mut self) {
        self.done = true;
        self.active_hopper = None;
    }
}

/// Error that occurred while polling a specific hopper.
#[derive(Debug, Clone)]
pub struct HopperPollError {
    /// The address of the hopper that produced the error.
    pub address: u8,
    /// The error that occurred.
    pub error: CommandError,
}

impl HopperPollError {
    /// Creates a new hopper poll error.
    #[must_use]
    pub const fn new(address: u8, error: CommandError) -> Self {
        Self { address, error }
    }
}

impl std::fmt::Display for HopperPollError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Hopper[{}]: {}", self.address, self.error)
    }
}

impl std::error::Error for HopperPollError {}

/// Result of polling all hoppers in the pool for inventory status.
#[derive(Debug, Clone, Default)]
pub struct PayoutPollResult {
    /// Inventory status of each hopper.
    pub inventories: Vec<HopperInventory>,
    /// Errors that occurred while polling individual hoppers.
    pub errors: Vec<HopperPollError>,
}

impl PayoutPollResult {
    /// Creates a new empty poll result.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            inventories: Vec::new(),
            errors: Vec::new(),
        }
    }

    /// Adds an inventory status to the result.
    pub fn add_inventory(&mut self, inventory: HopperInventory) {
        self.inventories.push(inventory);
    }

    /// Adds an error to the result.
    pub fn add_error(&mut self, error: HopperPollError) {
        self.errors.push(error);
    }

    /// Returns `true` if any errors occurred during polling.
    #[must_use]
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// Returns the inventory for a specific hopper by address.
    #[must_use]
    pub fn get_by_address(&self, address: u8) -> Option<&HopperInventory> {
        self.inventories.iter().find(|inv| inv.address == address)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hopper_inventory_level_from_status() {
        // Both sensors, both above
        let status = HopperStatus::new(true, true, true, true);
        assert_eq!(
            HopperInventoryLevel::from(status),
            HopperInventoryLevel::High
        );

        // Both sensors, above low but below high
        let status = HopperStatus::new(true, true, true, false);
        assert_eq!(
            HopperInventoryLevel::from(status),
            HopperInventoryLevel::Medium
        );

        // Below low level
        let status = HopperStatus::new(true, false, true, false);
        assert_eq!(
            HopperInventoryLevel::from(status),
            HopperInventoryLevel::Low
        );

        // No sensors
        let status = HopperStatus::new(false, false, false, false);
        assert_eq!(
            HopperInventoryLevel::from(status),
            HopperInventoryLevel::Unknown
        );
    }

    #[test]
    fn dispense_progress_tracking() {
        let mut progress = DispenseProgress::new(500);

        assert_eq!(progress.requested, 500);
        assert_eq!(progress.dispensed, 0);
        assert_eq!(progress.remaining, 500);
        assert!(!progress.done);

        progress.coin_dispensed(200);
        assert_eq!(progress.dispensed, 200);
        assert_eq!(progress.remaining, 300);
        assert_eq!(progress.coins_count(), 1);

        progress.coin_dispensed(100);
        assert_eq!(progress.dispensed, 300);
        assert_eq!(progress.remaining, 200);
        assert_eq!(progress.coins_count(), 2);

        progress.mark_done();
        assert!(progress.done);
    }

    #[test]
    fn payout_poll_result_queries() {
        let mut result = PayoutPollResult::new();

        let status = HopperStatus::new(true, true, true, true);
        result.add_inventory(HopperInventory::new(
            3,
            100,
            HopperInventoryLevel::High,
            status,
        ));
        result.add_inventory(HopperInventory::new(
            4,
            50,
            HopperInventoryLevel::Low,
            status,
        ));

        assert_eq!(result.get_by_address(3).map(|i| i.value), Some(100));
        assert_eq!(result.get_by_address(4).map(|i| i.value), Some(50));
        assert!(result.get_by_address(5).is_none());
    }
}
