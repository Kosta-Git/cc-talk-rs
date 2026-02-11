use std::{collections::HashSet, time::Duration};

use tokio::sync::mpsc;

use crate::device::payout::PayoutDevice;

use super::{
    PayoutPoolResult,
    config::HopperSelectionStrategy,
    event::PayoutPoolEvent,
    pool::PayoutPool,
};

/// Default capacity for the event notification channel.
const DEFAULT_EVENT_CHANNEL_SIZE: usize = 16;

/// Builder for constructing a [`PayoutPool`].
///
/// # Example
///
/// ```ignore
/// let (pool, mut event_rx) = PayoutPool::builder()
///     .add_hopper(hopper1, 100)  // 1.00 EUR coin
///     .add_hopper(hopper2, 50)   // 0.50 EUR coin
///     .with_selection_strategy(HopperSelectionStrategy::LargestFirst)
///     .with_polling_interval(Duration::from_millis(250))
///     .build();
///
/// // Consume events in a background task
/// tokio::spawn(async move {
///     while let Some(event) = event_rx.recv().await {
///         println!("Pool event: {:?}", event);
///     }
/// });
/// ```
#[derive(Debug)]
pub struct PayoutPoolBuilder {
    hoppers: Vec<(PayoutDevice, u32)>,
    selection_strategy: HopperSelectionStrategy,
    polling_interval: Duration,
    initially_disabled: HashSet<u8>,
    event_channel_size: usize,
}

impl Default for PayoutPoolBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl PayoutPoolBuilder {
    /// Creates a new builder with default settings.
    ///
    /// Default configuration:
    /// - No hoppers
    /// - Largest-first selection strategy
    /// - 250ms polling interval
    /// - No initially disabled hoppers
    /// - Event channel capacity of 16
    #[must_use]
    pub fn new() -> Self {
        Self {
            hoppers: Vec::new(),
            selection_strategy: HopperSelectionStrategy::default(),
            polling_interval: Duration::from_millis(250),
            initially_disabled: HashSet::new(),
            event_channel_size: DEFAULT_EVENT_CHANNEL_SIZE,
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

    /// Sets the hopper addresses that should be initially disabled.
    ///
    /// Disabled hoppers are part of the pool but will not be used for
    /// payout operations until explicitly enabled.
    #[must_use]
    pub fn with_disabled_hoppers(mut self, addresses: impl IntoIterator<Item = u8>) -> Self {
        self.initially_disabled = addresses.into_iter().collect();
        self
    }

    /// Sets the capacity of the event notification channel.
    ///
    /// The default capacity is 16, which is sufficient for typical hopper
    /// dispensing rates of up to 8 coins per second.
    #[must_use]
    pub fn with_event_channel_size(mut self, size: usize) -> Self {
        self.event_channel_size = size;
        self
    }

    /// Builds the pool without initializing it.
    ///
    /// Returns the pool and a receiver for pool events. You must call
    /// [`PayoutPool::initialize`] before using the pool for payout operations.
    ///
    /// # Returns
    ///
    /// A tuple of `(PayoutPool, mpsc::Receiver<PayoutPoolEvent>)`.
    #[must_use]
    pub fn build(self) -> (PayoutPool, mpsc::Receiver<PayoutPoolEvent>) {
        let (event_tx, event_rx) = mpsc::channel(self.event_channel_size);

        let pool = PayoutPool::new(
            self.hoppers,
            self.selection_strategy,
            self.polling_interval,
            self.initially_disabled,
            event_tx,
        );

        (pool, event_rx)
    }

    /// Builds and initializes the pool.
    ///
    /// This is the recommended way to create a ready-to-use pool.
    ///
    /// # Returns
    ///
    /// A tuple of `(PayoutPool, mpsc::Receiver<PayoutPoolEvent>)`.
    ///
    /// # Errors
    ///
    /// Returns an error if initialization fails (e.g., no hoppers, all hoppers fail).
    pub async fn build_and_initialize(
        self,
    ) -> PayoutPoolResult<(PayoutPool, mpsc::Receiver<PayoutPoolEvent>)> {
        let (pool, event_rx) = self.build();
        pool.initialize().await?;
        Ok((pool, event_rx))
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
        let (pool, _rx) = builder.build();

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

        let (pool, _rx) = PayoutPool::builder().add_hopper(hopper, 100).build();

        assert_eq!(pool.hopper_count(), 1);
    }

    #[test]
    fn builder_add_multiple_hoppers() {
        let h1 = create_test_hopper(3);
        let h2 = create_test_hopper(4);
        let h3 = create_test_hopper(5);

        let (pool, _rx) = PayoutPool::builder()
            .add_hoppers(vec![(h1, 100), (h2, 50)])
            .add_hopper(h3, 20)
            .build();

        assert_eq!(pool.hopper_count(), 3);
    }

    #[test]
    fn builder_selection_strategy() {
        let (pool, _rx) = PayoutPool::builder()
            .with_selection_strategy(HopperSelectionStrategy::SmallestFirst)
            .build();

        assert_eq!(
            pool.selection_strategy(),
            &HopperSelectionStrategy::SmallestFirst
        );
    }

    #[test]
    fn builder_polling_interval() {
        let (pool, _rx) = PayoutPool::builder()
            .with_polling_interval(Duration::from_millis(100))
            .build();

        assert_eq!(pool.polling_interval(), Duration::from_millis(100));
    }

    #[test]
    fn builder_disabled_hoppers() {
        let h1 = create_test_hopper(3);
        let h2 = create_test_hopper(4);

        let (pool, _rx) = PayoutPool::builder()
            .add_hopper(h1, 100)
            .add_hopper(h2, 50)
            .with_disabled_hoppers(vec![3])
            .build();

        assert!(pool.is_hopper_disabled(3));
        assert!(!pool.is_hopper_disabled(4));
    }

    #[test]
    fn builder_event_channel_size() {
        let (pool, _rx) = PayoutPool::builder()
            .with_event_channel_size(32)
            .build();

        // Verify pool was created (channel size isn't directly observable)
        assert_eq!(pool.hopper_count(), 0);
    }

    #[test]
    fn builder_returns_event_receiver() {
        let h1 = create_test_hopper(3);

        let (pool, mut event_rx) = PayoutPool::builder().add_hopper(h1, 100).build();

        // Disable a hopper and verify event is received
        pool.disable_hopper(3).expect("should succeed");

        let event = event_rx.try_recv().expect("should receive event");
        assert!(matches!(
            event,
            PayoutPoolEvent::HopperDisabled { address: 3 }
        ));
    }
}
