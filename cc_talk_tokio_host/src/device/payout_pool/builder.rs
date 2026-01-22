use std::time::Duration;

use crate::device::payout::PayoutDevice;

use super::{
    PayoutPoolResult,
    config::HopperSelectionStrategy,
    pool::PayoutPool,
};

/// Builder for constructing a [`PayoutPool`].
///
/// # Example
///
/// ```ignore
/// let pool = PayoutPool::builder()
///     .add_hopper(hopper1, 100)  // 1.00 EUR coin
///     .add_hopper(hopper2, 50)   // 0.50 EUR coin
///     .with_selection_strategy(HopperSelectionStrategy::LargestFirst)
///     .with_polling_interval(Duration::from_millis(250))
///     .build_and_initialize()
///     .await?;
/// ```
#[derive(Debug, Default)]
pub struct PayoutPoolBuilder {
    hoppers: Vec<(PayoutDevice, u32)>,
    selection_strategy: HopperSelectionStrategy,
    polling_interval: Duration,
}

impl PayoutPoolBuilder {
    /// Creates a new builder with default settings.
    ///
    /// Default configuration:
    /// - No hoppers
    /// - Largest-first selection strategy
    /// - 250ms polling interval
    #[must_use]
    pub fn new() -> Self {
        Self {
            hoppers: Vec::new(),
            selection_strategy: HopperSelectionStrategy::default(),
            polling_interval: Duration::from_millis(250),
        }
    }

    /// Adds a hopper to the pool with its coin value.
    ///
    /// # Arguments
    ///
    /// * `hopper` - The payout device
    /// * `value` - The coin value this hopper dispenses (in smallest currency units, e.g., cents)
    #[must_use]
    pub fn add_hopper(mut self, hopper: PayoutDevice, value: u32) -> Self {
        self.hoppers.push((hopper, value));
        self
    }

    /// Adds multiple hoppers to the pool.
    ///
    /// Each tuple contains a payout device and its coin value.
    #[must_use]
    pub fn add_hoppers(mut self, hoppers: impl IntoIterator<Item = (PayoutDevice, u32)>) -> Self {
        self.hoppers.extend(hoppers);
        self
    }

    /// Sets the hopper selection strategy.
    ///
    /// - `LargestFirst` - Use highest value coins first (default, minimizes coin count)
    /// - `SmallestFirst` - Use lowest value coins first
    /// - `BalanceInventory` - Prefer hoppers with highest inventory
    #[must_use]
    pub fn with_selection_strategy(mut self, strategy: HopperSelectionStrategy) -> Self {
        self.selection_strategy = strategy;
        self
    }

    /// Sets the polling interval for status checks during dispense operations.
    #[must_use]
    pub fn with_polling_interval(mut self, interval: Duration) -> Self {
        self.polling_interval = interval;
        self
    }

    /// Builds the pool without initializing it.
    ///
    /// You must call [`PayoutPool::initialize`] before using the pool.
    #[must_use]
    pub fn build(self) -> PayoutPool {
        PayoutPool::new(
            self.hoppers,
            self.selection_strategy,
            self.polling_interval,
        )
    }

    /// Builds and initializes the pool.
    ///
    /// This is the recommended way to create a ready-to-use pool.
    ///
    /// # Errors
    ///
    /// Returns an error if initialization fails (e.g., no hoppers, all hoppers fail).
    pub async fn build_and_initialize(self) -> PayoutPoolResult<PayoutPool> {
        let mut pool = self.build();
        pool.initialize().await?;
        Ok(pool)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cc_talk_core::cc_talk::{Category, ChecksumType, Device};
    use tokio::sync::mpsc;

    fn create_test_hopper(address: u8) -> PayoutDevice {
        let (tx, _rx) = mpsc::channel(1);
        let device = Device::new(address, Category::Payout, ChecksumType::Crc8);
        PayoutDevice::new(device, tx)
    }

    #[test]
    fn builder_default() {
        let builder = PayoutPoolBuilder::new();
        let pool = builder.build();

        assert_eq!(pool.hopper_count(), 0);
        assert_eq!(pool.selection_strategy(), &HopperSelectionStrategy::LargestFirst);
        assert_eq!(pool.polling_interval(), Duration::from_millis(250));
    }

    #[test]
    fn builder_add_hopper() {
        let hopper = create_test_hopper(3);

        let pool = PayoutPool::builder()
            .add_hopper(hopper, 100)
            .build();

        assert_eq!(pool.hopper_count(), 1);
    }

    #[test]
    fn builder_add_multiple_hoppers() {
        let h1 = create_test_hopper(3);
        let h2 = create_test_hopper(4);
        let h3 = create_test_hopper(5);

        let pool = PayoutPool::builder()
            .add_hoppers(vec![(h1, 100), (h2, 50)])
            .add_hopper(h3, 20)
            .build();

        assert_eq!(pool.hopper_count(), 3);
    }

    #[test]
    fn builder_selection_strategy() {
        let pool = PayoutPool::builder()
            .with_selection_strategy(HopperSelectionStrategy::SmallestFirst)
            .build();

        assert_eq!(
            pool.selection_strategy(),
            &HopperSelectionStrategy::SmallestFirst
        );
    }

    #[test]
    fn builder_polling_interval() {
        let pool = PayoutPool::builder()
            .with_polling_interval(Duration::from_millis(100))
            .build();

        assert_eq!(pool.polling_interval(), Duration::from_millis(100));
    }
}
