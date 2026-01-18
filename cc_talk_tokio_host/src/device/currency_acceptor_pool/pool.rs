#![allow(dead_code)]

use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use cc_talk_core::cc_talk::{
    BillEvent, BillRouteCode, CoinAcceptorError, CoinEvent, CurrencyToken,
};
use tokio::sync::{mpsc, oneshot};
use tracing::{debug, error, info, instrument, trace, warn};

use crate::{
    device::{
        base::{DeviceCommon, PollingError},
        bill_validator::BillValidator,
        coin_validator::CoinValidator,
    },
    util::DropGuard,
};

use super::{
    PoolError, PoolResult,
    builder::CurrencyAcceptorPoolBuilder,
    config::{BillRoutingMode, DenominationRange, DeviceValueMap},
    device_id::DeviceId,
    poll_result::{CurrencyCredit, PendingBill, PoolPollError, PoolPollResult},
};

type PoolPollReceiver = mpsc::Receiver<PoolPollResult>;

/// Result of a completed payment acceptance.
#[derive(Debug, Clone)]
pub struct PaymentResult {
    /// Total value received in smallest currency units.
    pub total_received: u32,
    /// Individual credits that contributed to this payment.
    pub credits: Vec<CurrencyCredit>,
    /// Whether the full target amount was reached.
    pub target_reached: bool,
}

/// Progress update during payment acceptance.
#[derive(Debug, Clone)]
pub struct PaymentProgress {
    /// The credit that was just received.
    pub credit: CurrencyCredit,
    /// Total value received so far.
    pub total_received: u32,
    /// Target value being collected.
    pub target_value: u32,
    /// Remaining value needed to reach target.
    pub remaining: u32,
}

/// A pool of currency acceptor devices for unified payment handling.
///
/// `CurrencyAcceptorPool` manages multiple coin validators and bill validators
/// as a single unit, providing coordinated control over:
///
/// - Master inhibit states
/// - Denomination filtering
/// - Polling and event aggregation
/// - Bill escrow routing
/// - Value-based payment acceptance
///
/// # Cloning
///
/// `CurrencyAcceptorPool` implements [`Clone`] and shares its internal state
/// (including polling locks and device references) across clones.
#[derive(Debug, Clone)]
pub struct CurrencyAcceptorPool {
    coin_validators: Vec<CoinValidator>,
    bill_validators: Vec<BillValidator>,
    /// Maps position -> value for each coin validator
    coin_value_maps: Vec<DeviceValueMap>,
    /// Maps position -> value for each bill validator
    bill_value_maps: Vec<DeviceValueMap>,
    denomination_range: DenominationRange,
    bill_routing_mode: BillRoutingMode,
    polling_interval: Duration,
    is_polling: Arc<Mutex<bool>>,
    initialized: Arc<Mutex<bool>>,
}

impl CurrencyAcceptorPool {
    /// Creates a new builder for constructing a `CurrencyAcceptorPool`.
    #[must_use]
    pub fn builder() -> CurrencyAcceptorPoolBuilder {
        CurrencyAcceptorPoolBuilder::new()
    }

    /// Creates a new pool with the given configuration.
    ///
    /// Prefer using [`CurrencyAcceptorPool::builder()`] for construction.
    pub(crate) fn new(
        coin_validators: Vec<CoinValidator>,
        bill_validators: Vec<BillValidator>,
        denomination_range: DenominationRange,
        bill_routing_mode: BillRoutingMode,
        polling_interval: Duration,
    ) -> Self {
        let coin_count = coin_validators.len();
        let bill_count = bill_validators.len();

        debug!(
            coin_validators = coin_count,
            bill_validators = bill_count,
            denomination_min = denomination_range.min,
            denomination_max = denomination_range.max,
            bill_routing_mode = ?bill_routing_mode,
            polling_interval_ms = polling_interval.as_millis() as u64,
            "creating currency acceptor pool"
        );

        Self {
            coin_validators,
            bill_validators,
            coin_value_maps: vec![DeviceValueMap::new(); coin_count],
            bill_value_maps: vec![DeviceValueMap::new(); bill_count],
            denomination_range,
            bill_routing_mode,
            polling_interval,
            is_polling: Arc::new(Mutex::new(false)),
            initialized: Arc::new(Mutex::new(false)),
        }
    }

    /// Returns the number of coin validators in the pool.
    #[must_use]
    pub fn coin_validator_count(&self) -> usize {
        self.coin_validators.len()
    }

    /// Returns the number of bill validators in the pool.
    #[must_use]
    pub fn bill_validator_count(&self) -> usize {
        self.bill_validators.len()
    }

    /// Returns the total number of devices in the pool.
    #[must_use]
    pub fn device_count(&self) -> usize {
        self.coin_validators.len() + self.bill_validators.len()
    }

    /// Returns `true` if the pool has been initialized.
    #[must_use]
    pub fn is_initialized(&self) -> bool {
        *self.initialized.lock().expect("should not be poisoned")
    }

    /// Returns the configured denomination range.
    #[must_use]
    pub const fn denomination_range(&self) -> DenominationRange {
        self.denomination_range
    }

    /// Returns the configured bill routing mode.
    #[must_use]
    pub const fn bill_routing_mode(&self) -> BillRoutingMode {
        self.bill_routing_mode
    }

    /// Returns the configured polling interval.
    #[must_use]
    pub const fn polling_interval(&self) -> Duration {
        self.polling_interval
    }

    /// Initializes the pool by reading currency IDs from all devices
    /// and configuring inhibits based on the denomination range.
    ///
    /// This method:
    /// 1. Reads coin/bill IDs from all positions on all devices
    /// 2. Builds value maps for position -> currency value lookup
    /// 3. Sets inhibits to only accept denominations within the configured range
    /// 4. Enables master inhibit on all devices (call `enable()` to start accepting)
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The pool has no devices
    /// - All devices fail to respond
    #[instrument(skip(self), fields(coin_validators = self.coin_validators.len(), bill_validators = self.bill_validators.len()))]
    pub async fn initialize(&mut self) -> PoolResult<()> {
        if self.device_count() == 0 {
            error!("pool initialization failed: no devices configured");
            return Err(PoolError::NoDevices);
        }

        info!(
            coin_validators = self.coin_validators.len(),
            bill_validators = self.bill_validators.len(),
            "initializing currency acceptor pool"
        );

        // Initialize coin validators
        for (idx, cv) in self.coin_validators.iter().enumerate() {
            debug!(device_idx = idx, "initializing coin validator");
            let value_map = &mut self.coin_value_maps[idx];
            let mut inhibits = [true; 16]; // Start with all inhibited
            let mut enabled_count = 0;

            for position in 0..16u8 {
                if let Ok(token) = cv.request_coin_id(position).await
                    && let Some(value) = Self::extract_value(&token)
                {
                    value_map.insert(position, value);
                    // Enable positions within denomination range
                    if self.denomination_range.contains(value) {
                        inhibits[position as usize] = false;
                        enabled_count += 1;
                        trace!(device_idx = idx, position, value, "coin position enabled");
                    } else {
                        trace!(
                            device_idx = idx,
                            position, value, "coin position inhibited (outside denomination range)"
                        );
                    }
                }
            }

            // Set coin inhibits based on denomination range
            if let Err(e) = cv.set_coin_inhibits(inhibits).await {
                warn!(device_idx = idx, error = %e, "failed to set coin inhibits");
            }
            // Enable master inhibit (device is disabled until enable() is called)
            if let Err(e) = cv.enable_master_inhibit().await {
                warn!(device_idx = idx, error = %e, "failed to enable master inhibit on coin validator");
            }

            info!(
                device_idx = idx,
                positions_configured = value_map.len(),
                positions_enabled = enabled_count,
                "coin validator initialized"
            );
        }

        // Initialize bill validators
        for (idx, bv) in self.bill_validators.iter().enumerate() {
            debug!(device_idx = idx, "initializing bill validator");
            let value_map = &mut self.bill_value_maps[idx];
            let mut inhibits = [true; 16]; // Start with all inhibited
            let mut enabled_count = 0;

            for position in 0..16u8 {
                if let Ok(token) = bv.request_bill_id(position).await
                    && let Some(value) = Self::extract_value(&token)
                {
                    value_map.insert(position, value);
                    // Enable positions within denomination range
                    if self.denomination_range.contains(value) {
                        inhibits[position as usize] = false;
                        enabled_count += 1;
                        trace!(device_idx = idx, position, value, "bill position enabled");
                    } else {
                        trace!(
                            device_idx = idx,
                            position, value, "bill position inhibited (outside denomination range)"
                        );
                    }
                }
            }

            // Set bill inhibits based on denomination range
            // Bill IDs are 1-indexed, so position 0 in the inhibits array
            // corresponds to bill ID 1. Rotate left by 1 to align correctly.
            inhibits.rotate_left(1);
            if let Err(e) = bv.set_bill_inhibits(inhibits).await {
                warn!(device_idx = idx, error = %e, "failed to set bill inhibits");
            }
            // Configure escrow based on routing mode
            let use_escrow = self.bill_routing_mode == BillRoutingMode::Manual;
            if let Err(e) = bv.set_operating_mode(true, use_escrow).await {
                warn!(device_idx = idx, error = %e, use_escrow, "failed to set bill operating mode");
            }
            // Enable master inhibit (device is disabled until enable() is called)
            if let Err(e) = bv.enable_master_inhibit().await {
                warn!(device_idx = idx, error = %e, "failed to enable master inhibit on bill validator");
            }

            info!(
                device_idx = idx,
                positions_configured = value_map.len(),
                positions_enabled = enabled_count,
                use_escrow,
                "bill validator initialized"
            );
        }

        *self.initialized.lock().expect("should not be poisoned") = true;
        info!("currency acceptor pool initialization complete");
        Ok(())
    }

    /// Enables all devices in the pool to accept currency.
    ///
    /// This disables the master inhibit on all devices, allowing them to
    /// accept coins/bills according to their individual inhibit settings.
    #[instrument(skip(self))]
    pub async fn enable(&self) -> PoolResult<()> {
        debug!("enabling all devices in pool");
        for (idx, cv) in self.coin_validators.iter().enumerate() {
            if let Err(e) = cv.disable_master_inhibit().await {
                warn!(device_idx = idx, error = %e, "failed to disable master inhibit on coin validator");
            }
        }
        for (idx, bv) in self.bill_validators.iter().enumerate() {
            if let Err(e) = bv.disable_master_inhibit().await {
                warn!(device_idx = idx, error = %e, "failed to disable master inhibit on bill validator");
            }
        }
        info!("pool enabled - accepting currency");
        Ok(())
    }

    /// Disables all devices in the pool.
    ///
    /// This enables the master inhibit on all devices, causing them to
    /// reject all currency.
    #[instrument(skip(self))]
    pub async fn disable(&self) -> PoolResult<()> {
        debug!("disabling all devices in pool");
        for (idx, cv) in self.coin_validators.iter().enumerate() {
            if let Err(e) = cv.enable_master_inhibit().await {
                warn!(device_idx = idx, error = %e, "failed to enable master inhibit on coin validator");
            }
        }
        for (idx, bv) in self.bill_validators.iter().enumerate() {
            if let Err(e) = bv.enable_master_inhibit().await {
                warn!(device_idx = idx, error = %e, "failed to enable master inhibit on bill validator");
            }
        }
        info!("pool disabled - rejecting currency");
        Ok(())
    }

    /// Polls all devices in the pool and returns aggregated results.
    ///
    /// This method polls each coin and bill validator, processing their
    /// events and converting position indices to currency values.
    ///
    /// # Bill Routing
    ///
    /// For bill validators, the behavior depends on `bill_routing_mode`:
    /// - `AutoStack`: Confirmed credits are added to the result
    /// - `AutoReturn`: Bills in escrow are automatically returned
    /// - `Manual`: Pending credits are added to `pending_bills` for manual routing
    pub async fn poll(&self) -> PoolPollResult {
        let mut result = PoolPollResult::new();

        // Poll coin validators
        for (idx, cv) in self.coin_validators.iter().enumerate() {
            let device_id = DeviceId::CoinValidator(idx);

            match cv.poll().await {
                Ok(poll_result) => {
                    for event in poll_result.events.iter() {
                        match event {
                            CoinEvent::Credit(credit) => {
                                let position = credit.credit;
                                if let Some(&value) = self.coin_value_maps[idx].get(&position) {
                                    info!(
                                        device = %device_id,
                                        position,
                                        value,
                                        "coin credit received"
                                    );
                                    result.add_credit(CurrencyCredit::new(
                                        value, device_id, position,
                                    ));
                                } else {
                                    warn!(
                                        device = %device_id,
                                        position,
                                        "coin credit received for unknown position"
                                    );
                                }
                            }
                            CoinEvent::Error(e) => {
                                if e.is_fraud_related() {
                                    warn!(device = %device_id, error = %e, "coin fraud attempt detected");
                                } else if e.is_hardware_issue() {
                                    error!(device = %device_id, error = %e, "coin validator hardware issue");
                                } else if *e != CoinAcceptorError::NullEvent {
                                    warn!(device = %device_id, error = %e, "coin rejected");
                                }
                            }
                            CoinEvent::Reset => {
                                trace!(device = %device_id, "coin validator reset detected");
                            }
                        }
                    }
                }
                Err(e) => {
                    debug!(device = %device_id, error = %e, "coin validator poll error");
                    result.add_error(PoolPollError::new(device_id, e));
                }
            }
        }

        // Poll bill validators
        for (idx, bv) in self.bill_validators.iter().enumerate() {
            let device_id = DeviceId::BillValidator(idx);

            match bv.poll().await {
                Ok(poll_result) => {
                    for event in poll_result.events.iter() {
                        match event {
                            BillEvent::Credit(bill_type) => {
                                if let Some(&value) = self.bill_value_maps[idx].get(bill_type) {
                                    info!(
                                        device = %device_id,
                                        bill_type,
                                        value,
                                        "bill credit received"
                                    );
                                    result.add_credit(CurrencyCredit::new(
                                        value, device_id, *bill_type,
                                    ));
                                } else {
                                    warn!(
                                        device = %device_id,
                                        bill_type,
                                        "bill credit received for unknown position"
                                    );
                                }
                            }
                            BillEvent::PendingCredit(bill_type) => {
                                self.handle_pending_bill(bv, idx, *bill_type, &mut result)
                                    .await;
                            }
                            BillEvent::Reject(reason) => {
                                warn!(device = %device_id, reason = %reason, "bill rejected");
                            }
                            BillEvent::FraudAttempt(reason) => {
                                warn!(device = %device_id, reason = %reason, "bill fraud attempt detected");
                            }
                            BillEvent::FatalError(reason) => {
                                error!(device = %device_id, reason = %reason, "bill validator fatal error");
                            }
                            BillEvent::Status(reason) => {
                                info!(device = %device_id, reason = %reason, "bill validator status");
                            }
                        }
                    }
                }
                Err(e) => {
                    debug!(device = %device_id, error = %e, "bill validator poll error");
                    result.add_error(PoolPollError::new(device_id, e));
                }
            }
        }

        result
    }

    /// Handles a pending bill based on the configured routing mode.
    async fn handle_pending_bill(
        &self,
        bv: &BillValidator,
        device_idx: usize,
        bill_type: u8,
        result: &mut PoolPollResult,
    ) {
        let device_id = DeviceId::BillValidator(device_idx);
        let value = self.bill_value_maps[device_idx]
            .get(&bill_type)
            .copied()
            .unwrap_or(0);

        match self.bill_routing_mode {
            BillRoutingMode::AutoStack => {
                info!(
                    device = %device_id,
                    bill_type,
                    value,
                    "auto-stacking pending bill"
                );
                if let Err(e) = bv.route_bill(BillRouteCode::Stack).await {
                    error!(device = %device_id, error = %e, "failed to auto-stack bill");
                }
            }
            BillRoutingMode::AutoReturn => {
                info!(
                    device = %device_id,
                    bill_type,
                    value,
                    "auto-returning pending bill"
                );
                if let Err(e) = bv.route_bill(BillRouteCode::Return).await {
                    error!(device = %device_id, error = %e, "failed to auto-return bill");
                }
            }
            BillRoutingMode::Manual => {
                info!(
                    device = %device_id,
                    bill_type,
                    value,
                    "bill held in escrow for manual routing"
                );
                result.add_pending_bill(PendingBill::new(value, device_id, bill_type));
            }
        }
    }

    /// Routes a pending bill to accept or reject it.
    ///
    /// This is only relevant when using `BillRoutingMode::Manual`.
    ///
    /// # Arguments
    ///
    /// * `pending_bill` - The pending bill to route
    /// * `accept` - `true` to accept (stack) the bill, `false` to reject (return) it
    #[instrument(skip(self, pending_bill), fields(device = %pending_bill.source, bill_type = pending_bill.bill_type, value = pending_bill.value))]
    pub async fn route_pending_bill(
        &self,
        pending_bill: &PendingBill,
        accept: bool,
    ) -> PoolResult<()> {
        let DeviceId::BillValidator(idx) = pending_bill.source else {
            error!("route_pending_bill called with non-bill-validator source");
            return Err(PoolError::BillRoutingFailed(
                "source is not a bill validator".to_string(),
            ));
        };

        let Some(bv) = self.bill_validators.get(idx) else {
            error!(device_idx = idx, "bill validator not found");
            return Err(PoolError::BillRoutingFailed(format!(
                "bill validator {} not found",
                idx
            )));
        };

        let route_code = if accept {
            BillRouteCode::Stack
        } else {
            BillRouteCode::Return
        };

        info!(
            accept,
            route = if accept { "stack" } else { "return" },
            "routing pending bill"
        );

        bv.route_bill(route_code).await.map_err(|e| {
            error!(error = %e, "bill routing failed");
            PoolError::BillRoutingFailed(e.to_string())
        })?;

        Ok(())
    }

    /// Resets all devices in the pool.
    #[instrument(skip(self))]
    pub async fn reset(&self) -> PoolResult<()> {
        info!("resetting all devices in pool");
        for (idx, cv) in self.coin_validators.iter().enumerate() {
            if let Err(e) = cv.reset_device().await {
                warn!(device_idx = idx, error = %e, "failed to reset coin validator");
            }
        }
        for (idx, bv) in self.bill_validators.iter().enumerate() {
            if let Err(e) = bv.reset_device().await {
                warn!(device_idx = idx, error = %e, "failed to reset bill validator");
            }
        }
        info!("all devices reset");
        Ok(())
    }

    /// Accepts a payment for a specific target value.
    ///
    /// This method enables the pool, polls for currency credits, and accumulates
    /// received value until the target is reached or the timeout expires.
    /// The pool is always disabled when this method returns, regardless of success or error.
    ///
    /// # Arguments
    ///
    /// * `target_value` - The target payment amount in smallest currency units
    /// * `timeout` - Maximum duration to wait for payment
    ///
    /// # Returns
    ///
    /// Returns a `PaymentResult` containing the total received and individual credits.
    /// If the target is not reached within the timeout, returns an error with the
    /// partial payment information including all credits received.
    ///
    /// # Example
    ///
    /// ```ignore
    /// // Accept 5.00 (500 cents) with 30 second timeout
    /// let result = pool.accept_payment(500, Duration::from_secs(30)).await?;
    /// println!("Received: {} cents", result.total_received);
    /// ```
    pub async fn accept_payment(
        &self,
        target_value: u32,
        timeout: Duration,
    ) -> PoolResult<PaymentResult> {
        let (_cancel_tx, cancel_rx) = oneshot::channel();
        self.accept_payment_with_progress(target_value, timeout, cancel_rx, None)
            .await
    }

    /// Accepts a payment with cancellation support.
    ///
    /// Similar to [`accept_payment`](Self::accept_payment), but can be cancelled
    /// by sending a signal through the provided channel.
    /// The pool is always disabled when this method returns, regardless of success or error.
    ///
    /// # Arguments
    ///
    /// * `target_value` - The target payment amount in smallest currency units
    /// * `timeout` - Maximum duration to wait for payment
    /// * `cancel_rx` - A oneshot receiver that cancels the payment when triggered
    ///
    /// # Returns
    ///
    /// Returns a `PaymentResult` on success. Returns `PoolError::PaymentCancelled`
    /// or `PoolError::PaymentTimeout` if cancelled/timed out, with partial payment
    /// information including all credits received.
    pub async fn accept_payment_with_cancel(
        &self,
        target_value: u32,
        timeout: Duration,
        cancel_rx: oneshot::Receiver<()>,
    ) -> PoolResult<PaymentResult> {
        self.accept_payment_with_progress(target_value, timeout, cancel_rx, None)
            .await
    }

    /// Accepts a payment with progress reporting and cancellation support.
    ///
    /// This is the most flexible payment acceptance method, providing both
    /// progress updates and cancellation support.
    /// The pool is always disabled when this method returns, regardless of success or error.
    ///
    /// # Arguments
    ///
    /// * `target_value` - The target payment amount in smallest currency units
    /// * `timeout` - Maximum duration to wait for payment
    /// * `cancel_rx` - A oneshot receiver that cancels the payment when triggered
    /// * `progress_tx` - Optional sender for receiving progress updates
    ///
    /// # Progress Updates
    ///
    /// If `progress_tx` is provided, a [`PaymentProgress`] is sent each time a
    /// credit is received, allowing the caller to update UI or track payment status.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let (cancel_tx, cancel_rx) = oneshot::channel();
    /// let (progress_tx, mut progress_rx) = mpsc::channel(16);
    ///
    /// // Spawn a task to handle progress updates
    /// tokio::spawn(async move {
    ///     while let Some(progress) = progress_rx.recv().await {
    ///         println!("Received {} cents, {} remaining",
    ///             progress.credit.value, progress.remaining);
    ///     }
    /// });
    ///
    /// let result = pool.accept_payment_with_progress(
    ///     500,
    ///     Duration::from_secs(30),
    ///     cancel_rx,
    ///     Some(progress_tx),
    /// ).await?;
    /// ```
    #[instrument(skip(self, cancel_rx, progress_tx), fields(target_value, timeout_secs = timeout.as_secs()))]
    pub async fn accept_payment_with_progress(
        &self,
        target_value: u32,
        timeout: Duration,
        mut cancel_rx: oneshot::Receiver<()>,
        progress_tx: Option<mpsc::Sender<PaymentProgress>>,
    ) -> PoolResult<PaymentResult> {
        info!(
            target_value,
            timeout_ms = timeout.as_millis() as u64,
            "starting payment acceptance"
        );

        // Enable the pool
        self.enable().await?;

        let result = self
            .accept_payment_inner(target_value, timeout, &mut cancel_rx, progress_tx)
            .await;

        // Always disable the pool, regardless of success or error
        let _ = self.disable().await;

        match &result {
            Ok(payment) => {
                info!(
                    total_received = payment.total_received,
                    credits_count = payment.credits.len(),
                    "payment completed successfully"
                );
            }
            Err(PoolError::PaymentTimeout {
                target,
                received,
                credits,
            }) => {
                warn!(
                    target,
                    received,
                    credits_count = credits.len(),
                    "payment timed out"
                );
            }
            Err(PoolError::PaymentCancelled {
                target,
                received,
                credits,
            }) => {
                info!(
                    target,
                    received,
                    credits_count = credits.len(),
                    "payment cancelled"
                );
            }
            Err(e) => {
                error!(error = %e, "payment failed with unexpected error");
            }
        }

        result
    }

    /// Internal payment acceptance logic.
    async fn accept_payment_inner(
        &self,
        target_value: u32,
        timeout: Duration,
        cancel_rx: &mut oneshot::Receiver<()>,
        progress_tx: Option<mpsc::Sender<PaymentProgress>>,
    ) -> PoolResult<PaymentResult> {
        let start = std::time::Instant::now();
        let mut total_received = 0u32;
        let mut credits = Vec::new();

        loop {
            // Check for cancellation
            if cancel_rx.try_recv().is_ok() {
                debug!(total_received, "payment cancellation signal received");
                return Err(PoolError::PaymentCancelled {
                    target: target_value,
                    received: total_received,
                    credits,
                });
            }

            // Check timeout
            if start.elapsed() >= timeout {
                if total_received >= target_value {
                    return Ok(PaymentResult {
                        total_received,
                        credits,
                        target_reached: true,
                    });
                }

                debug!(
                    elapsed_ms = start.elapsed().as_millis() as u64,
                    total_received, "payment timeout reached"
                );
                return Err(PoolError::PaymentTimeout {
                    target: target_value,
                    received: total_received,
                    credits,
                });
            }

            // Poll for credits
            let poll_result = self.poll().await;

            for credit in poll_result.credits {
                total_received += credit.value;
                let remaining = target_value.saturating_sub(total_received);

                debug!(
                    credit_value = credit.value,
                    total_received, remaining, "credit added to payment"
                );

                // Send progress update if channel provided
                if let Some(tx) = &progress_tx {
                    let progress = PaymentProgress {
                        credit: credit.clone(),
                        total_received,
                        target_value,
                        remaining,
                    };
                    // Don't fail payment if progress receiver is dropped
                    let _ = tx.try_send(progress);
                }

                credits.push(credit);
            }

            // Check if target reached
            if total_received >= target_value {
                debug!(total_received, target_value, "payment target reached");
                return Ok(PaymentResult {
                    total_received,
                    credits,
                    target_reached: true,
                });
            }

            // Sleep before next poll
            tokio::time::sleep(self.polling_interval).await;
        }
    }

    /// Starts background polling for currency events.
    ///
    /// Spawns a background task that continuously polls all devices and sends
    /// results through a channel.
    ///
    /// # Arguments
    ///
    /// * `channel_size` - Capacity of the result channel
    ///
    /// # Returns
    ///
    /// On success, returns a guard wrapping a receiver channel. Poll results
    /// are sent through this channel. When the guard is dropped, the background
    /// polling task is automatically stopped.
    ///
    /// # Errors
    ///
    /// Returns [`PollingError::AlreadyLeased`] if background polling is already active.
    #[must_use = "nothing happens if the result is not used"]
    pub fn try_background_polling(
        &self,
        channel_size: usize,
    ) -> Result<DropGuard<PoolPollReceiver, impl FnOnce(PoolPollReceiver)>, PollingError> {
        let mut is_polling = self.is_polling.lock().expect("should not be poisoned");
        if *is_polling {
            warn!("background polling already active");
            return Err(PollingError::AlreadyLeased);
        }
        *is_polling = true;

        info!(
            channel_size,
            polling_interval_ms = self.polling_interval.as_millis() as u64,
            "starting background polling"
        );

        let (tx, rx) = mpsc::channel(channel_size);

        let is_polling_arc = Arc::clone(&self.is_polling);
        let pool_clone = self.clone();
        let (stop_signal, mut stop_receiver) = oneshot::channel();

        let handle = tokio::spawn(async move {
            loop {
                let poll_result = pool_clone.poll().await;
                if tx.send(poll_result).await.is_err() {
                    error!(
                        "unable to send poll result, receiver may have been dropped. Stopping background polling."
                    );
                    break;
                }

                if stop_receiver.try_recv().is_ok() {
                    info!("received stop signal, stopping background polling task.");
                    break;
                }

                tokio::time::sleep(pool_clone.polling_interval).await;
            }
        });

        let rx_with_guard = DropGuard::new(rx, move |_| {
            if stop_signal.send(()).is_err() {
                warn!("failed to send stop signal to background polling task, aborting it...");
                handle.abort();
            }
            let mut is_polling = is_polling_arc.lock().expect("should not be poisoned");
            *is_polling = false;
            info!("background polling stopped");
        });

        Ok(rx_with_guard)
    }

    /// Extracts the value in smallest currency units from a `CurrencyToken`.
    fn extract_value(token: &CurrencyToken) -> Option<u32> {
        match token {
            CurrencyToken::Token => None,
            CurrencyToken::Currency(value) => Some(value.smallest_unit_value()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cc_talk_core::cc_talk::{Category, ChecksumType, Device};
    use tokio::sync::mpsc;

    fn create_test_pool() -> CurrencyAcceptorPool {
        let (tx, _rx) = mpsc::channel(1);

        let cv_device = Device::new(2, Category::CoinAcceptor, ChecksumType::Crc8);
        let cv = CoinValidator::new(cv_device, tx.clone());

        let bv_device = Device::new(40, Category::BillValidator, ChecksumType::Crc8);
        let bv = BillValidator::new(bv_device, tx);

        CurrencyAcceptorPool::new(
            vec![cv],
            vec![bv],
            DenominationRange::new(50, 10000),
            BillRoutingMode::AutoStack,
            Duration::from_millis(100),
        )
    }

    #[test]
    fn pool_device_counts() {
        let pool = create_test_pool();

        assert_eq!(pool.coin_validator_count(), 1);
        assert_eq!(pool.bill_validator_count(), 1);
        assert_eq!(pool.device_count(), 2);
    }

    #[test]
    fn pool_configuration() {
        let pool = create_test_pool();

        assert_eq!(pool.denomination_range().min, 50);
        assert_eq!(pool.denomination_range().max, 10000);
        assert_eq!(pool.bill_routing_mode(), BillRoutingMode::AutoStack);
        assert_eq!(pool.polling_interval(), Duration::from_millis(100));
    }

    #[test]
    fn pool_not_initialized_by_default() {
        let pool = create_test_pool();
        assert!(!pool.is_initialized());
    }

    #[tokio::test]
    async fn try_background_polling_returns_already_leased_when_called_twice() {
        let pool = create_test_pool();

        let first_guard = pool
            .try_background_polling(1)
            .expect("first call should succeed");

        let result = pool.try_background_polling(1);
        assert!(matches!(result, Err(PollingError::AlreadyLeased)));

        drop(first_guard);
    }

    #[tokio::test]
    async fn try_background_polling_can_restart_after_drop() {
        let pool = create_test_pool();

        let guard = pool
            .try_background_polling(1)
            .expect("first call should succeed");
        drop(guard);

        let new_guard = pool
            .try_background_polling(1)
            .expect("should be able to start polling again");
        drop(new_guard);
    }
}
