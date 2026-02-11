use std::{
    collections::{HashMap, HashSet},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

use tokio::sync::mpsc;
use tracing::{debug, error, info, instrument, trace, warn};

use crate::device::{base::DeviceCommon, payout::PayoutDevice};

use super::{
    PayoutPoolError, PayoutPoolResult,
    builder::PayoutPoolBuilder,
    config::HopperSelectionStrategy,
    event::PayoutEvent,
    poll_result::{
        DispenseProgress, HopperInventory, HopperInventoryLevel, HopperPollError, PayoutPollResult,
    },
};

/// Maximum number of consecutive failures before giving up on a hopper.
const MAX_FAILURES: u8 = 5;

/// A pool of hopper devices for unified payout handling.
///
/// `PayoutPool` manages multiple payout devices (hoppers) as a single unit,
/// providing coordinated control over:
///
/// - Pool-level hopper enable/disable (no hardware commands)
/// - Inventory level monitoring via sensors
/// - Value-based payout operations with automatic hopper selection
/// - Per-payment async event notifications
/// - Automatic payout plan rebalancing when hoppers run empty
/// - Emergency stop coordination
///
/// # Cloning
///
/// `PayoutPool` implements [`Clone`] and shares its internal state
/// (including disabled hopper set and dispensing lock) across clones.
///
/// # Example
///
/// ```ignore
/// let pool = PayoutPool::builder()
///     .add_hopper(hopper1, 100)  // 1.00 EUR
///     .add_hopper(hopper2, 50)   // 0.50 EUR
///     .add_hopper(hopper3, 20)   // 0.20 EUR
///     .selection_strategy(HopperSelectionStrategy::LargestFirst)
///     .build();
///
/// pool.initialize().await?;
///
/// // Dispense 1.70 EUR = 1x100 + 1x50 + 1x20
/// let result = pool.payout(170).await?;
/// println!("Dispensed {} cents in {} coins", result.dispensed, result.coins_count());
/// ```
#[derive(Debug, Clone)]
pub struct PayoutPool {
    hoppers: Vec<PayoutDevice>,
    /// Maps hopper address -> coin value.
    hopper_values: HashMap<u8, u32>,
    /// Set of hopper addresses that are disabled at the pool level.
    disabled_hoppers: Arc<Mutex<HashSet<u8>>>,
    selection_strategy: HopperSelectionStrategy,
    polling_interval: Duration,
    initialized: Arc<AtomicBool>,
    is_dispensing: Arc<AtomicBool>,
}

impl PayoutPool {
    /// Creates a new builder for constructing a `PayoutPool`.
    #[must_use]
    pub fn builder() -> PayoutPoolBuilder {
        PayoutPoolBuilder::default()
    }

    /// Creates a new pool with the given configuration.
    ///
    /// Prefer using [`PayoutPool::builder()`] for construction.
    pub(crate) fn new(
        hoppers: Vec<(PayoutDevice, u32)>,
        selection_strategy: HopperSelectionStrategy,
        polling_interval: Duration,
        initially_disabled: HashSet<u8>,
    ) -> Self {
        let mut hopper_values = HashMap::new();
        let mut hopper_devices = Vec::with_capacity(hoppers.len());

        for (hopper, value) in hoppers {
            let address = hopper.device.address();
            hopper_values.insert(address, value);
            hopper_devices.push(hopper);
        }

        debug!(
            hopper_count = hopper_devices.len(),
            selection_strategy = ?selection_strategy,
            polling_interval_ms = polling_interval.as_millis() as u64,
            initially_disabled = ?initially_disabled,
            "creating payout pool"
        );

        Self {
            hoppers: hopper_devices,
            hopper_values,
            disabled_hoppers: Arc::new(Mutex::new(initially_disabled)),
            selection_strategy,
            polling_interval,
            initialized: Arc::new(AtomicBool::new(false)),
            is_dispensing: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Returns the number of hoppers in the pool.
    #[must_use]
    pub fn hopper_count(&self) -> usize {
        self.hoppers.len()
    }

    /// Returns `true` if the pool has been initialized.
    #[must_use]
    pub fn is_initialized(&self) -> bool {
        self.initialized.load(Ordering::Acquire)
    }

    /// Returns the configured selection strategy.
    #[must_use]
    pub const fn selection_strategy(&self) -> &HopperSelectionStrategy {
        &self.selection_strategy
    }

    /// Returns the configured polling interval.
    #[must_use]
    pub const fn polling_interval(&self) -> Duration {
        self.polling_interval
    }

    /// Returns the addresses of all hoppers in the pool.
    #[must_use]
    pub fn hopper_addresses(&self) -> Vec<u8> {
        self.hoppers.iter().map(|h| h.device.address()).collect()
    }

    /// Returns a reference to the hopper value map (address -> coin value).
    #[must_use]
    pub const fn hopper_values(&self) -> &HashMap<u8, u32> {
        &self.hopper_values
    }

    /// Returns the coin value for a hopper by address.
    #[must_use]
    pub fn get_hopper_value(&self, address: u8) -> Option<u32> {
        self.hopper_values.get(&address).copied()
    }

    // --- Pool-level enable/disable ---

    /// Disables a hopper at the pool level.
    ///
    /// A disabled hopper will not be used for payout operations.
    /// This does not send any hardware commands to the device.
    ///
    /// # Errors
    ///
    /// Returns [`PayoutPoolError::HopperNotFound`] if the address is not in the pool.
    #[instrument(skip(self), fields(address))]
    pub fn disable_hopper(&self, address: u8) -> PayoutPoolResult<()> {
        self.get_hopper(address)?;
        let mut disabled = self
            .disabled_hoppers
            .lock()
            .expect("should not be poisoned");
        disabled.insert(address);
        info!(address, "hopper disabled");
        Ok(())
    }

    /// Enables a hopper at the pool level.
    ///
    /// Re-includes a previously disabled hopper in payout operations.
    /// This does not send any hardware commands to the device.
    ///
    /// # Errors
    ///
    /// Returns [`PayoutPoolError::HopperNotFound`] if the address is not in the pool.
    #[instrument(skip(self), fields(address))]
    pub fn enable_hopper(&self, address: u8) -> PayoutPoolResult<()> {
        self.get_hopper(address)?;
        let mut disabled = self
            .disabled_hoppers
            .lock()
            .expect("should not be poisoned");
        disabled.remove(&address);
        info!(address, "hopper enabled");
        Ok(())
    }

    /// Returns `true` if the hopper is disabled at the pool level.
    #[must_use]
    pub fn is_hopper_disabled(&self, address: u8) -> bool {
        self.disabled_hoppers
            .lock()
            .expect("should not be poisoned")
            .contains(&address)
    }

    /// Returns the set of currently disabled hopper addresses.
    #[must_use]
    pub fn disabled_hoppers(&self) -> HashSet<u8> {
        self.disabled_hoppers
            .lock()
            .expect("should not be poisoned")
            .clone()
    }

    /// Returns the available hoppers filtered to exclude disabled and extra exclusions.
    ///
    /// Returns `(address, coin_value)` pairs sorted by the selection strategy.
    fn available_hopper_values(&self, extra_exclusions: &HashSet<u8>) -> Vec<(u8, u32)> {
        let disabled = self
            .disabled_hoppers
            .lock()
            .expect("should not be poisoned");
        let mut hoppers: Vec<(u8, u32)> = self
            .hopper_values
            .iter()
            .filter(|(addr, _)| !disabled.contains(addr) && !extra_exclusions.contains(addr))
            .map(|(&addr, &val)| (addr, val))
            .collect();

        match self.selection_strategy {
            HopperSelectionStrategy::LargestFirst | HopperSelectionStrategy::BalanceInventory => {
                hoppers.sort_by(|a, b| b.1.cmp(&a.1));
            }
            HopperSelectionStrategy::SmallestFirst => {
                hoppers.sort_by(|a, b| a.1.cmp(&b.1));
            }
        }

        hoppers
    }

    /// Initializes the pool by verifying all hoppers are responsive.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The pool has no hoppers
    /// - All hoppers fail to respond
    #[instrument(skip(self), fields(hopper_count = self.hoppers.len()))]
    pub async fn initialize(&self) -> PayoutPoolResult<()> {
        if self.hoppers.is_empty() {
            error!("pool initialization failed: no hoppers configured");
            return Err(PayoutPoolError::NoHoppers);
        }

        info!(
            hopper_count = self.hoppers.len(),
            "initializing payout pool"
        );

        let mut successful = 0;

        for hopper in &self.hoppers {
            let address = hopper.device.address();
            debug!(address, "verifying hopper");

            match hopper.simple_poll().await {
                Ok(()) => {
                    successful += 1;
                    debug!(address, "hopper verified");
                }
                Err(e) => {
                    warn!(address, error = %e, "hopper verification failed");
                }
            }
        }

        if successful == 0 {
            error!("pool initialization failed: all hoppers failed");
            return Err(PayoutPoolError::AllHoppersFailed);
        }

        self.initialized.store(true, Ordering::Release);
        info!(
            successful,
            total = self.hoppers.len(),
            "payout pool initialization complete"
        );
        Ok(())
    }

    /// Polls all hoppers for their inventory status.
    ///
    /// Polls all hoppers including disabled ones, since physical
    /// monitoring is still useful regardless of pool-level state.
    #[instrument(skip(self))]
    pub async fn poll_inventories(&self) -> PayoutPollResult {
        let mut result = PayoutPollResult::new();

        for hopper in &self.hoppers {
            let address = hopper.device.address();
            let value = self.hopper_values.get(&address).copied().unwrap_or(0);

            match hopper.get_sensor_status().await {
                Ok((_level, status)) => {
                    let inventory_level = HopperInventoryLevel::from(status);
                    trace!(address, level = %inventory_level, "hopper inventory polled");
                    result.add_inventory(HopperInventory::new(
                        address,
                        value,
                        inventory_level,
                        status,
                    ));
                }
                Err(e) => {
                    debug!(address, error = %e, "hopper inventory poll failed");
                    result.add_error(HopperPollError::new(address, e));
                }
            }
        }

        result
    }

    /// Gets the inventory status for a specific hopper.
    #[instrument(skip(self), fields(address))]
    pub async fn get_hopper_inventory(&self, address: u8) -> PayoutPoolResult<HopperInventory> {
        let hopper = self.get_hopper(address)?;
        let value = self.hopper_values.get(&address).copied().unwrap_or(0);

        let (_level, status) = hopper
            .get_sensor_status()
            .await
            .map_err(|e| PayoutPoolError::CommandError { address, error: e })?;

        let inventory_level = HopperInventoryLevel::from(status);
        Ok(HopperInventory::new(
            address,
            value,
            inventory_level,
            status,
        ))
    }

    /// Calculates whether the requested value can be dispensed with enabled hoppers.
    ///
    /// Note: This is a theoretical check assuming unlimited coins in each hopper.
    /// Actual availability depends on hopper inventory.
    #[must_use]
    pub fn can_payout(&self, value: u32) -> bool {
        let available = self.available_hopper_values(&HashSet::new());
        self.generate_payout_plan(value, &available).1 == 0
    }

    /// Executes an emergency stop on all hoppers.
    #[instrument(skip(self))]
    pub async fn emergency_stop(&self) -> PayoutPoolResult<()> {
        error!("emergency stop triggered on all hoppers");
        for hopper in &self.hoppers {
            let address = hopper.device.address();
            if let Err(e) = hopper.emergency_stop().await {
                error!(address, error = %e, "emergency stop failed on hopper");
            }
        }
        warn!("emergency stop completed");
        Ok(())
    }

    /// Dispenses the specified value from the pool.
    ///
    /// Uses the configured selection strategy to choose hoppers, respects
    /// disabled hoppers, and automatically replans if a hopper runs empty.
    /// Hoppers are dispensed sequentially (never in parallel) to avoid
    /// voltage issues on the ccTalk bus.
    ///
    /// # Arguments
    ///
    /// * `value` - The total value to dispense (in smallest currency units)
    ///
    /// # Returns
    ///
    /// Returns the final dispense progress showing what was actually dispensed.
    #[instrument(skip(self), fields(value))]
    pub async fn payout(&self, value: u32) -> PayoutPoolResult<DispenseProgress> {
        self.payout_guarded(value, None).await
    }

    /// Dispenses the specified value with event notifications.
    ///
    /// Events are sent through the provided channel during the operation.
    /// The caller creates and owns the channel, controlling buffer size.
    ///
    /// # Arguments
    ///
    /// * `value` - The total value to dispense
    /// * `event_tx` - Channel to receive payout events
    #[instrument(skip(self, event_tx), fields(value))]
    pub async fn payout_with_events(
        &self,
        value: u32,
        event_tx: mpsc::Sender<PayoutEvent>,
    ) -> PayoutPoolResult<DispenseProgress> {
        self.payout_guarded(value, Some(event_tx)).await
    }

    /// Guards payout with the dispensing lock.
    async fn payout_guarded(
        &self,
        value: u32,
        event_tx: Option<mpsc::Sender<PayoutEvent>>,
    ) -> PayoutPoolResult<DispenseProgress> {
        if self
            .is_dispensing
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            return Err(PayoutPoolError::PayoutInProgress);
        }

        let result = self.payout_inner(value, &event_tx).await;

        self.is_dispensing.store(false, Ordering::Release);

        result
    }

    /// Internal payout implementation.
    ///
    /// Hoppers are dispensed sequentially to avoid voltage issues on the
    /// ccTalk bus.
    async fn payout_inner(
        &self,
        value: u32,
        event_tx: &Option<mpsc::Sender<PayoutEvent>>,
    ) -> PayoutPoolResult<DispenseProgress> {
        info!(value, "starting payout");

        let mut progress = DispenseProgress::new(value);
        let mut exhausted_hoppers = HashSet::new();

        // Build available hoppers (respecting disabled)
        let available_hoppers = self.available_hopper_values(&exhausted_hoppers);

        // Generate initial plan (Vec preserves strategy order)
        let (mut plan, remainder) = self.generate_payout_plan(value, &available_hoppers);

        if remainder > 0 {
            warn!(
                value,
                remainder, "cannot dispense exact amount, some value will remain"
            );
        }

        // Execute the plan — hoppers are processed in strategy order
        while let Some((address, count)) = plan.first().copied() {
            plan.remove(0);

            let Some(hopper) = self.hoppers.iter().find(|h| h.device.address() == address) else {
                warn!(address, "hopper not found in pool");
                continue;
            };

            let coin_value = self.hopper_values.get(&address).copied().unwrap_or(0);
            progress.active_hopper = Some(address);

            info!(address, count, coin_value, "dispensing from hopper");

            // Emit progress event
            emit_event(event_tx, PayoutEvent::Progress(progress.clone()));

            // Dispense coins from this hopper
            let dispensed = self
                .dispense_from_hopper(hopper, count, coin_value, &mut progress, event_tx)
                .await;

            if dispensed < count {
                // Hopper ran empty or failed — mark as exhausted and replan
                warn!(
                    address,
                    requested = count,
                    dispensed,
                    "hopper ran empty or failed"
                );
                progress.empty_hoppers.push(address);
                exhausted_hoppers.insert(address);

                // Emit hopper empty event
                emit_event(
                    event_tx,
                    PayoutEvent::HopperEmpty {
                        address,
                        coin_value,
                    },
                );

                // Replan with remaining value, excluding exhausted AND disabled
                if progress.remaining > 0 {
                    let available = self.available_hopper_values(&exhausted_hoppers);
                    if !available.is_empty() {
                        let (new_plan, _) =
                            self.generate_payout_plan(progress.remaining, &available);

                        emit_event(
                            event_tx,
                            PayoutEvent::PlanRebalanced {
                                exhausted_hopper: address,
                                remaining_value: progress.remaining,
                                new_plan: new_plan.clone(),
                            },
                        );

                        plan = new_plan;
                        info!(
                            remaining = progress.remaining,
                            hoppers_available = available.len(),
                            "replanning payout"
                        );
                    }
                }
            }

            // Emit progress event
            emit_event(event_tx, PayoutEvent::Progress(progress.clone()));
        }

        progress.mark_done();

        info!(
            requested = value,
            dispensed = progress.dispensed,
            coins = progress.coins_count(),
            "payout completed"
        );

        Ok(progress)
    }

    /// Dispenses coins from a single hopper, polling for completion.
    async fn dispense_from_hopper(
        &self,
        hopper: &PayoutDevice,
        count: u8,
        coin_value: u32,
        progress: &mut DispenseProgress,
        event_tx: &Option<mpsc::Sender<PayoutEvent>>,
    ) -> u8 {
        let address = hopper.device.address();
        let mut dispensed: u8 = 0;
        let mut failures: u8 = 0;

        if let Err(e) = hopper.enable_hopper().await {
            error!(address, count, error = %e, "failed to enable hopper");
            emit_event(event_tx, PayoutEvent::HopperError { address, error: e });
            return 0;
        }

        // Initiate the dispense
        if let Err(e) = hopper.payout_serial_number(count).await {
            error!(address, count, error = %e, "failed to initiate dispense");
            emit_event(event_tx, PayoutEvent::HopperError { address, error: e });
            return 0;
        }

        // Poll until complete or max failures
        let mut interval = tokio::time::interval(self.polling_interval);
        let mut remaining = count;

        while remaining > 0 && failures < MAX_FAILURES {
            interval.tick().await;

            match hopper.get_payout_status().await {
                Ok(status) => {
                    failures = 0;

                    // Track newly dispensed coins
                    let newly_paid = status.paid.saturating_sub(dispensed);
                    for _ in 0..newly_paid {
                        progress.coin_dispensed(coin_value);
                    }
                    dispensed = status.paid;
                    remaining = status.coins_remaining;

                    trace!(
                        address,
                        paid = status.paid,
                        remaining = status.coins_remaining,
                        unpaid = status.unpaid,
                        "dispense status"
                    );
                }
                Err(e) => {
                    failures += 1;
                    warn!(
                        address,
                        failures,
                        max = MAX_FAILURES,
                        error = %e,
                        "dispense status poll failed"
                    );

                    if failures >= MAX_FAILURES {
                        error!(address, "max failures reached, stopping dispense");
                        emit_event(event_tx, PayoutEvent::HopperError { address, error: e });
                        let _ = hopper.emergency_stop().await;
                    }
                }
            }
        }

        if let Err(e) = hopper.disable_hopper().await {
            error!(address, count, error = %e, "failed to disable hopper");
        }

        debug!(
            address,
            dispensed,
            requested = count,
            "hopper dispense complete"
        );
        dispensed
    }

    /// Generates a payout plan using the greedy algorithm.
    ///
    /// Returns `(plan, remainder)` where plan is a list of `(hopper_address, coin_count)`
    /// pairs in strategy order, and remainder is the value that couldn't be dispensed.
    fn generate_payout_plan(
        &self,
        value: u32,
        available_hoppers: &[(u8, u32)],
    ) -> (Vec<(u8, u8)>, u32) {
        let mut plan = Vec::new();
        let mut remaining = value;

        // available_hoppers is already sorted by selection strategy
        for &(address, coin_value) in available_hoppers {
            if coin_value == 0 || remaining == 0 {
                continue;
            }

            let quantity = remaining / coin_value;
            if quantity > 0 {
                // Cap at u8::MAX per dispense command
                let capped_quantity = quantity.min(u8::MAX as u32) as u8;
                plan.push((address, capped_quantity));
                remaining -= u32::from(capped_quantity) * coin_value;
            }
        }

        (plan, remaining)
    }

    /// Gets a reference to a hopper by address.
    fn get_hopper(&self, address: u8) -> PayoutPoolResult<&PayoutDevice> {
        self.hoppers
            .iter()
            .find(|h| h.device.address() == address)
            .ok_or(PayoutPoolError::HopperNotFound(address))
    }
}

/// Conditionally emits an event if a sender is available.
fn emit_event(event_tx: &Option<mpsc::Sender<PayoutEvent>>, event: PayoutEvent) {
    if let Some(tx) = event_tx {
        let _ = tx.try_send(event);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cc_talk_core::cc_talk::{Category, ChecksumType, Device};
    use tokio::sync::mpsc;

    fn create_test_pool() -> PayoutPool {
        let (tx, _rx) = mpsc::channel(1);

        let h1 = PayoutDevice::new(
            Device::new(3, Category::Payout, ChecksumType::Crc8),
            tx.clone(),
        );
        let h2 = PayoutDevice::new(
            Device::new(4, Category::Payout, ChecksumType::Crc8),
            tx.clone(),
        );
        let h3 = PayoutDevice::new(Device::new(5, Category::Payout, ChecksumType::Crc8), tx);

        PayoutPool::new(
            vec![(h1, 100), (h2, 50), (h3, 20)],
            HopperSelectionStrategy::LargestFirst,
            Duration::from_millis(250),
            HashSet::new(),
        )
    }

    #[test]
    fn pool_hopper_count() {
        let pool = create_test_pool();
        assert_eq!(pool.hopper_count(), 3);
    }

    #[test]
    fn pool_hopper_addresses() {
        let pool = create_test_pool();
        let addresses = pool.hopper_addresses();
        assert_eq!(addresses.len(), 3);
        assert!(addresses.contains(&3));
        assert!(addresses.contains(&4));
        assert!(addresses.contains(&5));
    }

    #[test]
    fn pool_hopper_values() {
        let pool = create_test_pool();
        assert_eq!(pool.get_hopper_value(3), Some(100));
        assert_eq!(pool.get_hopper_value(4), Some(50));
        assert_eq!(pool.get_hopper_value(5), Some(20));
        assert_eq!(pool.get_hopper_value(99), None);
    }

    #[test]
    fn pool_not_initialized_by_default() {
        let pool = create_test_pool();
        assert!(!pool.is_initialized());
    }

    #[test]
    fn generate_payout_plan_largest_first() {
        let pool = create_test_pool();
        let available = pool.available_hopper_values(&HashSet::new());

        // 170 = 1x100 + 1x50 + 1x20
        let (plan, remainder) = pool.generate_payout_plan(170, &available);
        assert_eq!(remainder, 0);
        assert!(plan.contains(&(3, 1))); // 100
        assert!(plan.contains(&(4, 1))); // 50
        assert!(plan.contains(&(5, 1))); // 20

        // 250 = 2x100 + 1x50
        let (plan, remainder) = pool.generate_payout_plan(250, &available);
        assert_eq!(remainder, 0);
        assert!(plan.contains(&(3, 2))); // 100
        assert!(plan.contains(&(4, 1))); // 50

        // 175 = 1x100 + 1x50 + 1x20 + 5 remainder
        let (plan, remainder) = pool.generate_payout_plan(175, &available);
        assert_eq!(remainder, 5);
        assert!(plan.contains(&(3, 1))); // 100
        assert!(plan.contains(&(4, 1))); // 50
        assert!(plan.contains(&(5, 1))); // 20
    }

    #[test]
    fn generate_payout_plan_preserves_strategy_order() {
        let pool = create_test_pool();
        let available = pool.available_hopper_values(&HashSet::new());

        // With LargestFirst, the plan should be ordered: 100, 50, 20
        let (plan, _) = pool.generate_payout_plan(170, &available);
        assert_eq!(plan.len(), 3);
        assert_eq!(plan[0].0, 3); // 100-cent hopper first
        assert_eq!(plan[1].0, 4); // 50-cent hopper second
        assert_eq!(plan[2].0, 5); // 20-cent hopper third
    }

    #[test]
    fn generate_payout_plan_smallest_first() {
        let (tx, _rx) = mpsc::channel(1);

        let h1 = PayoutDevice::new(
            Device::new(3, Category::Payout, ChecksumType::Crc8),
            tx.clone(),
        );
        let h2 = PayoutDevice::new(
            Device::new(4, Category::Payout, ChecksumType::Crc8),
            tx.clone(),
        );
        let h3 = PayoutDevice::new(Device::new(5, Category::Payout, ChecksumType::Crc8), tx);

        let pool = PayoutPool::new(
            vec![(h1, 100), (h2, 50), (h3, 20)],
            HopperSelectionStrategy::SmallestFirst,
            Duration::from_millis(250),
            HashSet::new(),
        );

        let available = pool.available_hopper_values(&HashSet::new());

        // 100 = 5x20 with smallest first
        let (plan, remainder) = pool.generate_payout_plan(100, &available);
        assert_eq!(remainder, 0);
        assert!(plan.contains(&(5, 5))); // 5x20 = 100
    }

    #[test]
    fn can_payout_exact_amount() {
        let pool = create_test_pool();

        assert!(pool.can_payout(170)); // 100 + 50 + 20
        assert!(pool.can_payout(100)); // 100
        assert!(pool.can_payout(20)); // 20
        assert!(!pool.can_payout(5)); // Can't make 5 with 100, 50, 20
        assert!(!pool.can_payout(15)); // Can't make 15
    }

    #[test]
    fn disable_and_enable_hopper() {
        let pool = create_test_pool();

        // Initially all enabled
        assert!(!pool.is_hopper_disabled(3));
        assert!(pool.disabled_hoppers().is_empty());

        // Disable hopper 3
        pool.disable_hopper(3).expect("should succeed");
        assert!(pool.is_hopper_disabled(3));
        assert!(!pool.is_hopper_disabled(4));
        assert_eq!(pool.disabled_hoppers().len(), 1);

        // Re-enable hopper 3
        pool.enable_hopper(3).expect("should succeed");
        assert!(!pool.is_hopper_disabled(3));
        assert!(pool.disabled_hoppers().is_empty());
    }

    #[test]
    fn disable_hopper_not_found() {
        let pool = create_test_pool();
        let result = pool.disable_hopper(99);
        assert!(matches!(result, Err(PayoutPoolError::HopperNotFound(99))));
    }

    #[test]
    fn enable_hopper_not_found() {
        let pool = create_test_pool();
        let result = pool.enable_hopper(99);
        assert!(matches!(result, Err(PayoutPoolError::HopperNotFound(99))));
    }

    #[test]
    fn can_payout_respects_disabled_hoppers() {
        let pool = create_test_pool();

        // With all hoppers: 170 = 100 + 50 + 20
        assert!(pool.can_payout(170));

        // Disable the 100-cent hopper
        pool.disable_hopper(3).expect("should succeed");

        // Now 170 = 3x50 + 1x20 = 150 + 20 = 170
        assert!(pool.can_payout(170));

        // Disable the 50-cent hopper too
        pool.disable_hopper(4).expect("should succeed");

        // Now only 20-cent hopper: 170 is not divisible by 20
        assert!(!pool.can_payout(170));
        // But 100 = 5x20
        assert!(pool.can_payout(100));
    }

    #[test]
    fn available_hopper_values_excludes_disabled() {
        let pool = create_test_pool();

        // All available
        let available = pool.available_hopper_values(&HashSet::new());
        assert_eq!(available.len(), 3);

        // Disable hopper 4
        pool.disable_hopper(4).expect("should succeed");
        let available = pool.available_hopper_values(&HashSet::new());
        assert_eq!(available.len(), 2);
        assert!(!available.iter().any(|(a, _)| *a == 4));
        assert!(available.iter().any(|(a, _)| *a == 3));
        assert!(available.iter().any(|(a, _)| *a == 5));
    }

    #[test]
    fn available_hopper_values_excludes_extra() {
        let pool = create_test_pool();

        let mut extra = HashSet::new();
        extra.insert(3);
        extra.insert(5);

        let available = pool.available_hopper_values(&extra);
        assert_eq!(available.len(), 1);
        assert!(available.iter().any(|(a, _)| *a == 4));
    }

    #[test]
    fn initially_disabled_hoppers() {
        let (tx, _rx) = mpsc::channel(1);

        let h1 = PayoutDevice::new(
            Device::new(3, Category::Payout, ChecksumType::Crc8),
            tx.clone(),
        );
        let h2 = PayoutDevice::new(Device::new(4, Category::Payout, ChecksumType::Crc8), tx);

        let mut initially_disabled = HashSet::new();
        initially_disabled.insert(3);

        let pool = PayoutPool::new(
            vec![(h1, 100), (h2, 50)],
            HopperSelectionStrategy::LargestFirst,
            Duration::from_millis(250),
            initially_disabled,
        );

        assert!(pool.is_hopper_disabled(3));
        assert!(!pool.is_hopper_disabled(4));
        assert_eq!(pool.disabled_hoppers().len(), 1);
    }

    #[test]
    fn pool_is_clone() {
        let pool = create_test_pool();
        let pool2 = pool.clone();

        // Both should share the same state
        pool.disable_hopper(3).expect("should succeed");
        assert!(pool2.is_hopper_disabled(3));
    }
}
