#![allow(dead_code)]

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::Duration,
};

use tokio::sync::mpsc;
use tracing::{debug, error, info, instrument, trace, warn};

use crate::device::{base::DeviceCommon, payout::PayoutDevice};

use super::{
    PayoutPoolError, PayoutPoolResult,
    builder::PayoutPoolBuilder,
    config::HopperSelectionStrategy,
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
/// - Hopper enable/disable states
/// - Inventory level monitoring via sensors
/// - Value-based payout operations with automatic hopper selection
/// - Emergency stop coordination
///
/// # Example
///
/// ```ignore
/// let pool = PayoutPool::builder()
///     .add_hopper(hopper1, 100)  // 1.00 EUR
///     .add_hopper(hopper2, 50)   // 0.50 EUR
///     .add_hopper(hopper3, 20)   // 0.20 EUR
///     .with_selection_strategy(HopperSelectionStrategy::LargestFirst)
///     .build_and_initialize()
///     .await?;
///
/// // Dispense 1.70 EUR = 1x100 + 1x50 + 1x20
/// let result = pool.payout(170).await?;
/// println!("Dispensed {} cents in {} coins", result.dispensed, result.coins_count());
/// ```
#[derive(Debug)]
pub struct PayoutPool {
    hoppers: Vec<PayoutDevice>,
    /// Maps hopper address -> coin value
    hopper_values: HashMap<u8, u32>,
    selection_strategy: HopperSelectionStrategy,
    polling_interval: Duration,
    initialized: Arc<Mutex<bool>>,
    is_dispensing: Arc<Mutex<bool>>,
}

impl PayoutPool {
    /// Creates a new builder for constructing a `PayoutPool`.
    #[must_use]
    pub fn builder() -> PayoutPoolBuilder {
        PayoutPoolBuilder::new()
    }

    /// Creates a new pool with the given configuration.
    ///
    /// Prefer using [`PayoutPool::builder()`] for construction.
    pub(crate) fn new(
        hoppers: Vec<(PayoutDevice, u32)>,
        selection_strategy: HopperSelectionStrategy,
        polling_interval: Duration,
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
            "creating payout pool"
        );

        Self {
            hoppers: hopper_devices,
            hopper_values,
            selection_strategy,
            polling_interval,
            initialized: Arc::new(Mutex::new(false)),
            is_dispensing: Arc::new(Mutex::new(false)),
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
        *self.initialized.lock().expect("should not be poisoned")
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

    /// Initializes the pool by verifying all hoppers are responsive.
    ///
    /// This method polls each hopper to ensure it's connected and working.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The pool has no hoppers
    /// - All hoppers fail to respond
    #[instrument(skip(self), fields(hopper_count = self.hoppers.len()))]
    pub async fn initialize(&mut self) -> PayoutPoolResult<()> {
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

        *self.initialized.lock().expect("should not be poisoned") = true;
        info!(
            successful,
            total = self.hoppers.len(),
            "payout pool initialization complete"
        );
        Ok(())
    }

    /// Enables all hoppers in the pool.
    #[instrument(skip(self))]
    pub async fn enable_all(&self) -> PayoutPoolResult<()> {
        debug!("enabling all hoppers");
        for hopper in &self.hoppers {
            let address = hopper.device.address();
            if let Err(e) = hopper.enable_hopper().await {
                warn!(address, error = %e, "failed to enable hopper");
            }
        }
        info!("all hoppers enabled");
        Ok(())
    }

    /// Disables all hoppers in the pool.
    #[instrument(skip(self))]
    pub async fn disable_all(&self) -> PayoutPoolResult<()> {
        debug!("disabling all hoppers");
        for hopper in &self.hoppers {
            let address = hopper.device.address();
            if let Err(e) = hopper.disable_hopper().await {
                warn!(address, error = %e, "failed to disable hopper");
            }
        }
        info!("all hoppers disabled");
        Ok(())
    }

    /// Enables a specific hopper by address.
    #[instrument(skip(self), fields(address))]
    pub async fn enable_hopper(&self, address: u8) -> PayoutPoolResult<()> {
        let hopper = self.get_hopper(address)?;
        hopper
            .enable_hopper()
            .await
            .map_err(|e| PayoutPoolError::CommandError { address, error: e })?;
        info!(address, "hopper enabled");
        Ok(())
    }

    /// Disables a specific hopper by address.
    #[instrument(skip(self), fields(address))]
    pub async fn disable_hopper(&self, address: u8) -> PayoutPoolResult<()> {
        let hopper = self.get_hopper(address)?;
        hopper
            .disable_hopper()
            .await
            .map_err(|e| PayoutPoolError::CommandError { address, error: e })?;
        info!(address, "hopper disabled");
        Ok(())
    }

    /// Polls all hoppers for their inventory status.
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

    /// Calculates the maximum value that can be dispensed with current hoppers.
    ///
    /// Note: This is a theoretical maximum assuming unlimited coins in each hopper.
    /// Actual availability depends on hopper inventory.
    #[must_use]
    pub fn can_payout(&self, value: u32) -> bool {
        // Check if we can make exact change with available denominations
        self.generate_payout_plan(value, &self.hopper_values).1 == 0
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
    /// Uses the configured selection strategy to choose hoppers and
    /// automatically replans if a hopper runs empty.
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
        let (progress_tx, _progress_rx) = mpsc::channel(16);
        self.payout_with_progress(value, progress_tx).await
    }

    /// Dispenses the specified value with progress updates.
    ///
    /// # Arguments
    ///
    /// * `value` - The total value to dispense
    /// * `progress_tx` - Channel to receive progress updates
    #[instrument(skip(self, progress_tx), fields(value))]
    pub async fn payout_with_progress(
        &self,
        value: u32,
        progress_tx: mpsc::Sender<DispenseProgress>,
    ) -> PayoutPoolResult<DispenseProgress> {
        // Check if already dispensing
        {
            let mut is_dispensing = self.is_dispensing.lock().expect("should not be poisoned");
            if *is_dispensing {
                return Err(PayoutPoolError::PayoutInProgress);
            }
            *is_dispensing = true;
        }

        let result = self.payout_inner(value, progress_tx).await;

        // Clear dispensing flag
        *self.is_dispensing.lock().expect("should not be poisoned") = false;

        result
    }

    // TODO: Rework this to have a bg sensor polling and inventory tracking task also a bg payout
    // task
    /// Internal payout implementation.
    async fn payout_inner(
        &self,
        value: u32,
        progress_tx: mpsc::Sender<DispenseProgress>,
    ) -> PayoutPoolResult<DispenseProgress> {
        info!(value, "starting payout");

        let mut progress = DispenseProgress::new(value);
        let mut available_hoppers = self.hopper_values.clone();

        // Generate initial plan
        let (mut plan, remainder) = self.generate_payout_plan(value, &available_hoppers);

        if remainder > 0 {
            warn!(
                value,
                remainder, "cannot dispense exact amount, some value will remain"
            );
        }

        // Execute the plan
        while !plan.is_empty() {
            // Get the next hopper to dispense from (based on strategy, we process largest first)
            let (&address, &count) = plan.iter().next().expect("plan is not empty");
            plan.remove(&address);

            let Some(hopper) = self.hoppers.iter().find(|h| h.device.address() == address) else {
                warn!(address, "hopper not found in pool");
                continue;
            };

            let coin_value = self.hopper_values.get(&address).copied().unwrap_or(0);
            progress.active_hopper = Some(address);

            info!(address, count, coin_value, "dispensing from hopper");

            // Send progress update
            let _ = progress_tx.try_send(progress.clone());

            // Dispense coins from this hopper
            let dispensed = self
                .dispense_from_hopper(hopper, count, coin_value, &mut progress)
                .await;

            if dispensed < count {
                // Hopper ran empty or failed - mark as empty and replan
                warn!(
                    address,
                    requested = count,
                    dispensed,
                    "hopper ran empty or failed"
                );
                progress.empty_hoppers.push(address);
                available_hoppers.remove(&address);

                // Replan with remaining value
                if progress.remaining > 0 && !available_hoppers.is_empty() {
                    let (new_plan, _) =
                        self.generate_payout_plan(progress.remaining, &available_hoppers);
                    plan = new_plan;
                    info!(
                        remaining = progress.remaining,
                        hoppers_available = available_hoppers.len(),
                        "replanning payout"
                    );
                }
            }

            // Send progress update
            let _ = progress_tx.try_send(progress.clone());
        }

        progress.mark_done();
        let _ = progress_tx.try_send(progress.clone());

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
    ) -> u8 {
        let address = hopper.device.address();
        let mut dispensed: u8 = 0;
        let mut failures: u8 = 0;

        // Initiate the dispense
        if let Err(e) = hopper.payout_serial_number(count).await {
            error!(address, count, error = %e, "failed to initiate dispense");
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
                        let _ = hopper.emergency_stop().await;
                    }
                }
            }
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
    /// Returns (plan, remainder) where plan maps hopper address to coin count,
    /// and remainder is the value that couldn't be dispensed.
    fn generate_payout_plan(
        &self,
        value: u32,
        available_hoppers: &HashMap<u8, u32>,
    ) -> (HashMap<u8, u8>, u32) {
        let mut plan = HashMap::new();
        let mut remaining = value;

        // Sort hoppers by value based on strategy
        let mut sorted_hoppers: Vec<_> = available_hoppers.iter().collect();
        match self.selection_strategy {
            HopperSelectionStrategy::LargestFirst => {
                sorted_hoppers.sort_by(|a, b| b.1.cmp(a.1));
            }
            HopperSelectionStrategy::SmallestFirst => {
                sorted_hoppers.sort_by(|a, b| a.1.cmp(b.1));
            }
            HopperSelectionStrategy::BalanceInventory => {
                // For balance inventory, we'd need actual inventory data
                // For now, fall back to largest first
                sorted_hoppers.sort_by(|a, b| b.1.cmp(a.1));
            }
        }

        for (&address, &coin_value) in sorted_hoppers {
            if coin_value == 0 || remaining == 0 {
                continue;
            }

            let quantity = remaining / coin_value;
            if quantity > 0 {
                // Cap at u8::MAX per dispense command
                let capped_quantity = quantity.min(u8::MAX as u32) as u8;
                plan.insert(address, capped_quantity);
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

        // 170 = 1x100 + 1x50 + 1x20
        let (plan, remainder) = pool.generate_payout_plan(170, &pool.hopper_values);
        assert_eq!(remainder, 0);
        assert_eq!(plan.get(&3), Some(&1)); // 100
        assert_eq!(plan.get(&4), Some(&1)); // 50
        assert_eq!(plan.get(&5), Some(&1)); // 20

        // 250 = 2x100 + 1x50
        let (plan, remainder) = pool.generate_payout_plan(250, &pool.hopper_values);
        assert_eq!(remainder, 0);
        assert_eq!(plan.get(&3), Some(&2)); // 100
        assert_eq!(plan.get(&4), Some(&1)); // 50

        // 175 = 1x100 + 1x50 + 1x20 + 5 remainder
        let (plan, remainder) = pool.generate_payout_plan(175, &pool.hopper_values);
        assert_eq!(remainder, 5);
        assert_eq!(plan.get(&3), Some(&1)); // 100
        assert_eq!(plan.get(&4), Some(&1)); // 50
        assert_eq!(plan.get(&5), Some(&1)); // 20
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
        );

        // 100 = 5x20 with smallest first
        let (plan, remainder) = pool.generate_payout_plan(100, &pool.hopper_values);
        assert_eq!(remainder, 0);
        assert_eq!(plan.get(&5), Some(&5)); // 5x20 = 100
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
}
