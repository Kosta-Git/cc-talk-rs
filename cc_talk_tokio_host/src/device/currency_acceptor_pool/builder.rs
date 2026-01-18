use std::time::Duration;

use crate::device::{bill_validator::BillValidator, coin_validator::CoinValidator};

use super::{
    PoolResult,
    config::{BillRoutingMode, DenominationRange},
    pool::CurrencyAcceptorPool,
};

/// Builder for constructing a [`CurrencyAcceptorPool`].
///
/// # Example
///
/// ```ignore
/// let pool = CurrencyAcceptorPool::builder()
///     .add_coin_validator(coin_validator)
///     .add_bill_validator(bill_validator)
///     .with_denomination_range(50, 10000)
///     .with_bill_routing_mode(BillRoutingMode::AutoStack)
///     .with_polling_interval(Duration::from_millis(100))
///     .build_and_initialize()
///     .await?;
/// ```
#[derive(Debug, Default)]
pub struct CurrencyAcceptorPoolBuilder {
    coin_validators: Vec<CoinValidator>,
    bill_validators: Vec<BillValidator>,
    denomination_range: DenominationRange,
    bill_routing_mode: BillRoutingMode,
    polling_interval: Duration,
}

impl CurrencyAcceptorPoolBuilder {
    /// Creates a new builder with default settings.
    ///
    /// Default configuration:
    /// - No devices
    /// - Accept all denominations (0 to u32::MAX)
    /// - Auto-stack bills
    /// - 100ms polling interval
    #[must_use]
    pub fn new() -> Self {
        Self {
            coin_validators: Vec::new(),
            bill_validators: Vec::new(),
            denomination_range: DenominationRange::default(),
            bill_routing_mode: BillRoutingMode::default(),
            polling_interval: Duration::from_millis(100),
        }
    }

    /// Adds a coin validator to the pool.
    #[must_use]
    pub fn add_coin_validator(mut self, validator: CoinValidator) -> Self {
        self.coin_validators.push(validator);
        self
    }

    /// Adds multiple coin validators to the pool.
    #[must_use]
    pub fn add_coin_validators(
        mut self,
        validators: impl IntoIterator<Item = CoinValidator>,
    ) -> Self {
        self.coin_validators.extend(validators);
        self
    }

    /// Adds a bill validator to the pool.
    #[must_use]
    pub fn add_bill_validator(mut self, validator: BillValidator) -> Self {
        self.bill_validators.push(validator);
        self
    }

    /// Adds multiple bill validators to the pool.
    #[must_use]
    pub fn add_bill_validators(
        mut self,
        validators: impl IntoIterator<Item = BillValidator>,
    ) -> Self {
        self.bill_validators.extend(validators);
        self
    }

    /// Sets the denomination range filter.
    ///
    /// Only coins/bills with values between `min` and `max` (inclusive)
    /// will be accepted.
    ///
    /// # Arguments
    ///
    /// * `min` - Minimum value in smallest currency units (e.g., cents)
    /// * `max` - Maximum value in smallest currency units
    #[must_use]
    pub fn with_denomination_range(mut self, min: u32, max: u32) -> Self {
        self.denomination_range = DenominationRange::new(min, max);
        self
    }

    /// Sets the denomination range using a `DenominationRange` struct.
    #[must_use]
    pub fn with_denomination_range_config(mut self, range: DenominationRange) -> Self {
        self.denomination_range = range;
        self
    }

    /// Sets the bill routing mode.
    ///
    /// - `AutoStack` - Automatically accept validated bills
    /// - `AutoReturn` - Automatically return validated bills
    /// - `Manual` - Hold bills in escrow for manual decision
    #[must_use]
    pub fn with_bill_routing_mode(mut self, mode: BillRoutingMode) -> Self {
        self.bill_routing_mode = mode;
        self
    }

    /// Convenience method to enable auto-stacking of bills.
    ///
    /// Equivalent to `with_bill_routing_mode(BillRoutingMode::AutoStack)`.
    #[must_use]
    pub fn with_auto_stack_bills(mut self, auto_stack: bool) -> Self {
        self.bill_routing_mode = if auto_stack {
            BillRoutingMode::AutoStack
        } else {
            BillRoutingMode::Manual
        };
        self
    }

    /// Sets the polling interval for background polling.
    #[must_use]
    pub fn with_polling_interval(mut self, interval: Duration) -> Self {
        self.polling_interval = interval;
        self
    }

    /// Builds the pool without initializing it.
    ///
    /// You must call [`CurrencyAcceptorPool::initialize`] before using the pool.
    #[must_use]
    pub fn build(self) -> CurrencyAcceptorPool {
        CurrencyAcceptorPool::new(
            self.coin_validators,
            self.bill_validators,
            self.denomination_range,
            self.bill_routing_mode,
            self.polling_interval,
        )
    }

    /// Builds and initializes the pool.
    ///
    /// This is the recommended way to create a ready-to-use pool.
    ///
    /// # Errors
    ///
    /// Returns an error if initialization fails (e.g., no devices, all devices fail).
    pub async fn build_and_initialize(self) -> PoolResult<CurrencyAcceptorPool> {
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

    fn create_test_coin_validator() -> CoinValidator {
        let (tx, _rx) = mpsc::channel(1);
        let device = Device::new(2, Category::CoinAcceptor, ChecksumType::Crc8);
        CoinValidator::new(device, tx)
    }

    fn create_test_bill_validator() -> BillValidator {
        let (tx, _rx) = mpsc::channel(1);
        let device = Device::new(40, Category::BillValidator, ChecksumType::Crc8);
        BillValidator::new(device, tx)
    }

    #[test]
    fn builder_default() {
        let builder = CurrencyAcceptorPoolBuilder::new();
        let pool = builder.build();

        assert_eq!(pool.coin_validator_count(), 0);
        assert_eq!(pool.bill_validator_count(), 0);
        assert_eq!(pool.denomination_range(), DenominationRange::default());
        assert_eq!(pool.bill_routing_mode(), BillRoutingMode::AutoStack);
        assert_eq!(pool.polling_interval(), Duration::from_millis(100));
    }

    #[test]
    fn builder_add_devices() {
        let cv = create_test_coin_validator();
        let bv = create_test_bill_validator();

        let pool = CurrencyAcceptorPool::builder()
            .add_coin_validator(cv)
            .add_bill_validator(bv)
            .build();

        assert_eq!(pool.coin_validator_count(), 1);
        assert_eq!(pool.bill_validator_count(), 1);
    }

    #[test]
    fn builder_add_multiple_devices() {
        let cv1 = create_test_coin_validator();
        let cv2 = create_test_coin_validator();
        let bv1 = create_test_bill_validator();
        let bv2 = create_test_bill_validator();

        let pool = CurrencyAcceptorPool::builder()
            .add_coin_validators(vec![cv1, cv2])
            .add_bill_validators(vec![bv1, bv2])
            .build();

        assert_eq!(pool.coin_validator_count(), 2);
        assert_eq!(pool.bill_validator_count(), 2);
    }

    #[test]
    fn builder_denomination_range() {
        let pool = CurrencyAcceptorPool::builder()
            .with_denomination_range(50, 10000)
            .build();

        assert_eq!(pool.denomination_range().min, 50);
        assert_eq!(pool.denomination_range().max, 10000);
    }

    #[test]
    fn builder_bill_routing_mode() {
        let pool = CurrencyAcceptorPool::builder()
            .with_bill_routing_mode(BillRoutingMode::Manual)
            .build();

        assert_eq!(pool.bill_routing_mode(), BillRoutingMode::Manual);
    }

    #[test]
    fn builder_auto_stack_bills() {
        let pool_auto = CurrencyAcceptorPool::builder()
            .with_auto_stack_bills(true)
            .build();
        assert_eq!(pool_auto.bill_routing_mode(), BillRoutingMode::AutoStack);

        let pool_manual = CurrencyAcceptorPool::builder()
            .with_auto_stack_bills(false)
            .build();
        assert_eq!(pool_manual.bill_routing_mode(), BillRoutingMode::Manual);
    }

    #[test]
    fn builder_polling_interval() {
        let pool = CurrencyAcceptorPool::builder()
            .with_polling_interval(Duration::from_millis(50))
            .build();

        assert_eq!(pool.polling_interval(), Duration::from_millis(50));
    }
}
