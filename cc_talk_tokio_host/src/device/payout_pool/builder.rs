use std::{collections::HashSet, time::Duration};

use derive_builder::Builder;

use crate::device::payout::PayoutDevice;

use super::{PayoutPoolResult, config::HopperSelectionStrategy, pool::PayoutPool};

/// Internal configuration struct used by `derive_builder` to generate
/// [`PayoutPoolBuilder`].
#[derive(Debug, Clone, Builder)]
#[builder(
    name = "PayoutPoolBuilder",
    vis = "pub",
    pattern = "owned",
    build_fn(skip),
    derive(Debug)
)]
#[allow(dead_code)]
pub(crate) struct PayoutPoolConfig {
    #[builder(setter(custom), default)]
    hoppers: Vec<(PayoutDevice, u32)>,

    #[builder(default)]
    selection_strategy: HopperSelectionStrategy,

    #[builder(default = "Duration::from_millis(250)")]
    polling_interval: Duration,

    #[builder(setter(custom), default)]
    initially_disabled: HashSet<u8>,
}

impl PayoutPoolBuilder {
    /// Adds a hopper to the pool with its coin value.
    ///
    /// # Arguments
    ///
    /// * `hopper` - The payout device
    /// * `value` - The coin value this hopper dispenses (in smallest currency units, e.g., cents)
    #[must_use]
    pub fn add_hopper(mut self, hopper: PayoutDevice, value: u32) -> Self {
        self.hoppers
            .get_or_insert_with(Vec::new)
            .push((hopper, value));
        self
    }

    /// Adds multiple hoppers to the pool.
    ///
    /// Each tuple contains a payout device and its coin value.
    #[must_use]
    pub fn add_hoppers(mut self, hoppers: impl IntoIterator<Item = (PayoutDevice, u32)>) -> Self {
        self.hoppers.get_or_insert_with(Vec::new).extend(hoppers);
        self
    }

    /// Sets the hopper addresses that should be initially disabled.
    ///
    /// Disabled hoppers are part of the pool but will not be used for
    /// payout operations until explicitly enabled.
    #[must_use]
    pub fn with_disabled_hoppers(mut self, addresses: impl IntoIterator<Item = u8>) -> Self {
        self.initially_disabled = Some(addresses.into_iter().collect());
        self
    }

    /// Builds the pool.
    ///
    /// You must call [`PayoutPool::initialize`] before using the pool for payout operations.
    #[must_use]
    pub fn build(self) -> PayoutPool {
        PayoutPool::new(
            self.hoppers.unwrap_or_default(),
            self.selection_strategy.unwrap_or_default(),
            self.polling_interval.unwrap_or(Duration::from_millis(250)),
            self.initially_disabled.unwrap_or_default(),
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
        let pool = self.build();
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
        let pool = PayoutPoolBuilder::default().build();

        assert_eq!(pool.hopper_count(), 0);
        assert_eq!(
            pool.selection_strategy(),
            &HopperSelectionStrategy::LargestFirst
        );
        assert_eq!(pool.polling_interval(), Duration::from_millis(250));
    }

    #[test]
    fn builder_add_hopper() {
        let hopper = create_test_hopper(3);

        let pool = PayoutPool::builder().add_hopper(hopper, 100).build();

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
            .selection_strategy(HopperSelectionStrategy::SmallestFirst)
            .build();

        assert_eq!(
            pool.selection_strategy(),
            &HopperSelectionStrategy::SmallestFirst
        );
    }

    #[test]
    fn builder_polling_interval() {
        let pool = PayoutPool::builder()
            .polling_interval(Duration::from_millis(100))
            .build();

        assert_eq!(pool.polling_interval(), Duration::from_millis(100));
    }

    #[test]
    fn builder_disabled_hoppers() {
        let h1 = create_test_hopper(3);
        let h2 = create_test_hopper(4);

        let pool = PayoutPool::builder()
            .add_hopper(h1, 100)
            .add_hopper(h2, 50)
            .with_disabled_hoppers(vec![3])
            .build();

        assert!(pool.is_hopper_disabled(3));
        assert!(!pool.is_hopper_disabled(4));
    }
}
