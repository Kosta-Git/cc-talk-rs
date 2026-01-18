use std::collections::HashMap;

/// Filter for accepted denominations by value range.
///
/// Only denominations with values between `min` and `max` (inclusive)
/// will be accepted by the pool.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DenominationRange {
    /// Minimum denomination value to accept (in smallest currency units).
    pub min: u32,
    /// Maximum denomination value to accept (in smallest currency units).
    pub max: u32,
}

impl DenominationRange {
    /// Creates a new denomination range.
    ///
    /// # Arguments
    ///
    /// * `min` - Minimum value in smallest currency units (e.g., cents).
    /// * `max` - Maximum value in smallest currency units.
    #[must_use]
    pub const fn new(min: u32, max: u32) -> Self {
        Self { min, max }
    }

    /// Checks if a value falls within this range.
    #[must_use]
    pub const fn contains(&self, value: u32) -> bool {
        value >= self.min && value <= self.max
    }
}

impl Default for DenominationRange {
    fn default() -> Self {
        Self {
            min: 0,
            max: u32::MAX,
        }
    }
}

/// Bill routing mode for controlling how validated bills are handled.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BillRoutingMode {
    /// Automatically stack (accept) validated bills.
    #[default]
    AutoStack,
    /// Automatically return validated bills to the customer.
    AutoReturn,
    /// Hold bills in escrow for manual routing decision.
    /// Use `route_pending_bill` to accept or reject.
    Manual,
}

/// Maps position indices (0-15) to currency values for a single device.
///
/// This is populated during initialization by reading coin/bill IDs from devices.
pub type DeviceValueMap = HashMap<u8, u32>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn denomination_range_contains() {
        let range = DenominationRange::new(50, 10000);

        assert!(!range.contains(49));
        assert!(range.contains(50));
        assert!(range.contains(100));
        assert!(range.contains(10000));
        assert!(!range.contains(10001));
    }

    #[test]
    fn denomination_range_default_accepts_all() {
        let range = DenominationRange::default();

        assert!(range.contains(0));
        assert!(range.contains(1));
        assert!(range.contains(u32::MAX));
    }

    #[test]
    fn bill_routing_mode_default_is_auto_stack() {
        assert_eq!(BillRoutingMode::default(), BillRoutingMode::AutoStack);
    }
}
