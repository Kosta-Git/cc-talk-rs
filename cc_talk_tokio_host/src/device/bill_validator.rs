#![allow(dead_code)]

use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use cc_talk_core::cc_talk::{
    BillRouteCode, BillRoutingError, BillValidatorPollResult, BitMask, CurrencyToken, Device,
};
use cc_talk_host::{command::Command, device::device_commands::*};
use tokio::sync::{mpsc, oneshot};
use tracing::{debug, error, info, instrument, trace, warn};

use crate::{
    device::base::PollingError, transport::tokio_transport::TransportMessage, util::DropGuard,
};

use super::base::{CommandError, DeviceCommon, DeviceResult};

/// A ccTalk bill validator device driver.
///
/// This struct provides methods to communicate with and control a bill validator
/// over the ccTalk protocol. It supports bill acceptance, inhibit control, escrow
/// operations, and background polling for bill events.
///
/// # Cloning
///
/// `BillValidator` implements [`Clone`] and shares its internal state across clones.
/// This means that polling state and event counters are synchronized between all
/// cloned instances.
#[derive(Debug, Clone)]
pub struct BillValidator {
    /// The underlying ccTalk device configuration.
    pub device: Device,
    /// Channel sender for communicating with the transport layer.
    pub sender: mpsc::Sender<TransportMessage>,
    event_counter: Arc<Mutex<u8>>,
    is_polling: Arc<Mutex<bool>>,
}

type PollResultReceiver = mpsc::Receiver<DeviceResult<BillValidatorPollResult>>;

impl BillValidator {
    /// Creates a new `BillValidator` instance.
    ///
    /// # Arguments
    ///
    /// * `device` - The ccTalk device configuration containing address and checksum type.
    /// * `sender` - A channel sender for communicating with the transport layer.
    pub fn new(device: Device, sender: mpsc::Sender<TransportMessage>) -> Self {
        debug!(
            address = device.address(),
            category = ?device.category(),
            "creating bill validator"
        );
        Self {
            device,
            sender,
            event_counter: Arc::new(Mutex::new(0)),
            is_polling: Arc::new(Mutex::new(false)),
        }
    }

    /// Returns the current event counter value.
    ///
    /// The event counter tracks the number of bill events that have occurred.
    /// It is automatically updated when calling [`poll`](Self::poll).
    pub fn event_counter(&self) -> u8 {
        *self.event_counter.lock().expect("should not be poisoned")
    }

    /// Sets the master inhibit status of the bill validator.
    ///
    /// When master inhibit is enabled (`true`), the bill validator will reject all bills.
    /// When disabled (`false`), bills will be accepted according to individual bill inhibit settings.
    ///
    /// # Arguments
    ///
    /// * `inhibit` - `true` to enable master inhibit (reject all bills), `false` to disable.
    #[instrument(skip(self), fields(inhibit), level = "debug")]
    pub async fn set_master_inhibit(&self, inhibit: bool) -> DeviceResult<()> {
        debug!(inhibit, "setting master inhibit status");
        let mask_value = !inhibit;
        let mut bitmask = BitMask::<1>::new(1).map_err(|_| CommandError::BufferOverflow)?;
        bitmask
            .set_bit(0, mask_value)
            .map_err(|_| CommandError::BufferOverflow)?;
        let command = ModifyMasterInhibitStatusCommand::<1>::build(bitmask)
            .map_err(|_| CommandError::BufferOverflow)?;
        let response_packet = self.send_command(command).await?;
        let bitmask = BitMask::<1>::new(1).map_err(|_| CommandError::BufferOverflow)?;
        ModifyMasterInhibitStatusCommand::<1>::build(bitmask)
            .map_err(|_| CommandError::BufferOverflow)?
            .parse_response(response_packet.get_data()?)
            .map_err(CommandError::from)?;
        info!(inhibit, "master inhibit status set");
        Ok(())
    }

    /// Enables the master inhibit, causing the validator to reject all bills.
    ///
    /// This is a convenience method equivalent to `set_master_inhibit(true)`.
    pub async fn enable_master_inhibit(&self) -> DeviceResult<()> {
        debug!("enabling master inhibit");
        self.set_master_inhibit(true).await
    }

    /// Disables the master inhibit, allowing the validator to accept bills.
    ///
    /// This is a convenience method equivalent to `set_master_inhibit(false)`.
    /// Note that individual bill inhibits may still prevent specific bills from being accepted.
    pub async fn disable_master_inhibit(&self) -> DeviceResult<()> {
        debug!("disabling master inhibit");
        self.set_master_inhibit(false).await
    }

    /// Returns the master inhibit status of the bill validator.
    ///
    /// Returns `true` if master inhibit is enabled (rejecting all bills),
    /// `false` if disabled (accepting bills).
    #[instrument(skip(self), level = "debug")]
    pub async fn get_master_inhibit_status(&self) -> DeviceResult<bool> {
        trace!("requesting master inhibit status");
        let response_packet = self
            .send_command(RequestMasterInhibitStatusCommand::<1>)
            .await?;
        let status = RequestMasterInhibitStatusCommand::<1>
            .parse_response(response_packet.get_data()?)
            .map_err(CommandError::from)
            .map(|bytes| bytes[0] == 0)?;
        debug!(inhibited = status, "master inhibit status received");
        Ok(status)
    }

    /// Checks if master inhibit is currently enabled.
    ///
    /// Returns `true` if the validator is rejecting all bills.
    pub async fn is_master_inhibit_enabled(&self) -> DeviceResult<bool> {
        self.get_master_inhibit_status().await
    }

    /// Checks if master inhibit is currently disabled.
    ///
    /// Returns `true` if the validator is accepting bills (subject to individual inhibits).
    pub async fn is_master_inhibit_disabled(&self) -> DeviceResult<bool> {
        let status = self.get_master_inhibit_status().await?;
        Ok(!status)
    }

    /// Requests the bill operating mode of the bill validator.
    ///
    /// # Returns
    ///
    /// A tuple of `(stacker_available, escrow_available)`:
    /// - `stacker_available` - `true` if the stacker is available for use.
    /// - `escrow_available` - `true` if escrow mode is available.
    #[instrument(skip(self), level = "debug")]
    pub async fn request_operating_mode(&self) -> DeviceResult<(bool, bool)> {
        trace!("requesting bill operating mode");
        let response_packet = self.send_command(RequestBillOperatingModeCommand).await?;
        let result = RequestBillOperatingModeCommand
            .parse_response(response_packet.get_data()?)
            .map_err(CommandError::from)?;
        debug!(
            stacker_available = result.0,
            escrow_available = result.1,
            "bill operating mode received"
        );
        Ok(result)
    }

    /// Sets the bill operating mode of the bill validator.
    ///
    /// # Arguments
    ///
    /// * `use_stacker` - `true` to enable the stacker for storing accepted bills.
    /// * `use_escrow` - `true` to enable escrow mode, allowing bills to be held
    ///   before final acceptance or rejection.
    #[instrument(skip(self), fields(use_stacker, use_escrow), level = "debug")]
    pub async fn set_operating_mode(
        &self,
        use_stacker: bool,
        use_escrow: bool,
    ) -> DeviceResult<()> {
        debug!(use_stacker, use_escrow, "setting bill operating mode");
        let command = ModifyBillOperatingModeCommand::new(use_stacker, use_escrow);
        let response_packet = self.send_command(command).await?;
        ModifyBillOperatingModeCommand::new(use_stacker, use_escrow)
            .parse_response(response_packet.get_data()?)
            .map_err(CommandError::from)?;
        info!(use_stacker, use_escrow, "bill operating mode set");
        Ok(())
    }

    /// Requests the bill ID (currency token) for a specific bill position.
    ///
    /// # Arguments
    ///
    /// * `id` - The bill position (0-15).
    ///
    /// # Returns
    ///
    /// The currency token identifying the bill type at this position.
    #[instrument(skip(self), fields(id), level = "trace")]
    pub async fn request_bill_id(&self, id: u8) -> DeviceResult<CurrencyToken> {
        trace!(bill_position = id, "requesting bill ID");
        let response_packet = self.send_command(RequestBillIdCommand::new(id)).await?;
        let token = RequestBillIdCommand::new(id)
            .parse_response(response_packet.get_data()?)
            .map_err(CommandError::from)?;
        trace!(bill_position = id, token = ?token, "bill ID received");
        Ok(token)
    }

    /// Requests bill IDs for all 16 bill positions.
    ///
    /// # Returns
    ///
    /// A vector of tuples containing the bill position and its currency token
    /// (or `None` if the request failed for that position).
    #[instrument(skip(self), level = "debug")]
    pub async fn request_all_bill_id(&self) -> DeviceResult<Vec<(u8, Option<CurrencyToken>)>> {
        debug!("requesting all bill IDs");
        let mut bills = std::vec::Vec::with_capacity(16);
        for i in 0..16 {
            if let Ok(bill) = self.request_bill_id(i).await {
                bills.push((i, Some(bill)));
            } else {
                bills.push((i, None));
            }
        }
        let configured_count = bills.iter().filter(|(_, b)| b.is_some()).count();
        debug!(configured_count, "all bill IDs request complete");
        Ok(bills)
    }

    /// Sets the inhibit status for each of the 16 bill positions.
    ///
    /// # Arguments
    ///
    /// * `inhibits` - An array of 16 boolean values where `true` disables the bill
    ///   and `false` enables it.
    #[instrument(skip(self), level = "debug")]
    pub async fn set_bill_inhibits(&self, inhibits: [bool; 16]) -> DeviceResult<()> {
        let enabled_count = inhibits.iter().filter(|&&i| !i).count();
        debug!(enabled_count, "setting bill inhibits");
        let mut bitmask = BitMask::<2>::new(16).map_err(|_| CommandError::BufferOverflow)?;
        for (i, disable) in inhibits.iter().enumerate() {
            bitmask
                // Invert value since 0 is disabled and 1 is enabled
                .set_bit(i, !*disable)
                .map_err(|_| CommandError::BufferOverflow)?;
        }
        let command = ModifyInhibitStatusCommand::<2>::build(bitmask)
            .map_err(|_| CommandError::BufferOverflow)?;
        let response_packet = self.send_command(command).await?;
        let bitmask = BitMask::<2>::new(16).map_err(|_| CommandError::BufferOverflow)?;
        ModifyInhibitStatusCommand::<2>::build(bitmask)
            .map_err(|_| CommandError::BufferOverflow)?
            .parse_response(response_packet.get_data()?)
            .map_err(CommandError::from)?;
        info!(enabled_count, "bill inhibits set");
        Ok(())
    }

    /// Sets the same inhibit status for all 16 bill positions.
    ///
    /// # Arguments
    ///
    /// * `inhibit` - `true` to disable all bills, `false` to enable all bills.
    pub async fn set_all_bill_inhibits(&self, inhibit: bool) -> DeviceResult<()> {
        debug!(inhibit, "setting all bill inhibits");
        let inhibits = [inhibit; 16];
        self.set_bill_inhibits(inhibits).await
    }

    /// Requests the inhibit status for each of the 16 bill positions.
    ///
    /// # Returns
    ///
    /// A vector of 16 boolean values where `true` means the bill is disabled
    /// and `false` means the bill is enabled.
    #[instrument(skip(self), level = "debug")]
    pub async fn get_bill_inhibits(&self) -> DeviceResult<Vec<bool>> {
        trace!("requesting bill inhibits");
        let response_packet = self.send_command(RequestInhibitStatusCommand::<2>).await?;
        let inhibits = RequestInhibitStatusCommand::<2>
            .parse_response(response_packet.get_data()?)
            .map_err(CommandError::from)
            .map(|mask| {
                let mut vec = std::vec::Vec::with_capacity(16);
                for byte in mask.iter() {
                    for i in 0..8 {
                        vec.push(byte & (1 << i) == 0);
                    }
                }
                vec
            })?;
        let enabled_count = inhibits.iter().filter(|&&i| !i).count();
        debug!(enabled_count, "bill inhibits received");
        Ok(inhibits)
    }

    /// Routes a bill that is currently held in escrow.
    ///
    /// This method is used to accept or reject a bill that has been validated
    /// and is being held in escrow. The bill can be sent to the stacker (accepted)
    /// or returned to the customer (rejected).
    ///
    /// # Arguments
    ///
    /// * `route_code` - The routing destination for the bill (e.g., accept to stacker,
    ///   return to customer).
    ///
    /// # Returns
    ///
    /// Returns `Ok(None)` if the routing was successful, or `Ok(Some(error))` if
    /// there was a routing error (e.g., stacker full, bill jammed).
    #[instrument(skip(self), fields(route_code = ?route_code), level = "info")]
    pub async fn route_bill(
        &self,
        route_code: BillRouteCode,
    ) -> DeviceResult<Option<BillRoutingError>> {
        info!(route_code = ?route_code, "routing bill");
        let command = RouteBillCommand::new(route_code);
        let response_packet = self.send_command(command).await?;
        let result = RouteBillCommand::new(route_code)
            .parse_response(response_packet.get_data()?)
            .map_err(CommandError::from)?;
        match &result {
            Some(err) => warn!(route_code = ?route_code, error = ?err, "bill routing failed"),
            None => info!(route_code = ?route_code, "bill routed successfully"),
        }
        Ok(result)
    }

    /// Polls the bill validator for buffered bill events.
    ///
    /// This method reads the event buffer from the bill validator and returns
    /// any new bill credits or errors since the last poll. The internal event
    /// counter is automatically updated.
    ///
    /// For continuous polling, consider using [`try_background_polling`](Self::try_background_polling)
    /// which handles the polling loop automatically.
    pub async fn poll(&self) -> DeviceResult<BillValidatorPollResult> {
        trace!("polling bill validator");
        let response_packet = self
            .send_command(ReadBufferedBillEventsCommand::default())
            .await?;
        let result = ReadBufferedBillEventsCommand::new(self.event_counter())
            .parse_response(response_packet.get_data()?)
            .map_err(CommandError::from)
            .inspect(|result| {
                self.event_counter
                    .lock()
                    .expect("should not be poisoned")
                    .clone_from(&result.event_counter);
            })?;
        if !result.events.is_empty() {
            debug!(
                event_counter = result.event_counter,
                events_count = result.events.len(),
                "bill validator poll returned events"
            );
        }
        Ok(result)
    }

    /// Returns the recommended polling priority (interval) for this device.
    ///
    /// The polling priority indicates how frequently the device should be polled
    /// for events. Use [`PollingPriority::as_duration`] to convert to a [`Duration`].
    #[instrument(skip(self), level = "debug")]
    pub async fn get_polling_priority(&self) -> DeviceResult<PollingPriority> {
        trace!("requesting polling priority");
        let response_packet = self.send_command(RequestPollingPriorityCommand).await?;
        let priority = RequestPollingPriorityCommand
            .parse_response(response_packet.get_data()?)
            .map_err(CommandError::from)?;
        debug!(priority = ?priority, "polling priority received");
        Ok(priority)
    }

    /// Starts background polling for bill events.
    ///
    /// This method spawns a background task that continuously polls the bill validator
    /// at the specified interval and sends results through a channel.
    ///
    /// # Arguments
    ///
    /// * `interval` - The duration between poll requests. Use [`get_polling_priority`](Self::get_polling_priority)
    ///   to get the device-recommended interval.
    /// * `channel_size` - Capacity of the result channel. If the consumer is slower than the
    ///   polling rate the channel might fill up and cause the polling task to block.
    ///
    /// # Returns
    ///
    /// On success, returns a guard wrapping a receiver channel. Poll results
    /// are sent through this channel. When the guard is dropped, the background polling
    /// task is automatically aborted and the polling lock is released.
    ///
    /// # Errors
    ///
    /// Returns [`PollingError::AlreadyLeased`] if background polling is already active
    /// on this instance or any of its clones.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let mut poll_rx = validator.try_background_polling(Duration::from_millis(100), 32)?;
    ///
    /// while let Some(result) = poll_rx.recv().await {
    ///     match result {
    ///         Ok(poll) => println!("Events: {:?}", poll.events),
    ///         Err(e) => eprintln!("Poll error: {}", e),
    ///     }
    /// }
    /// // Polling stops automatically when poll_rx is dropped
    /// ```
    #[must_use = "nothing happens if the result is not used"]
    pub fn try_background_polling(
        &self,
        interval: Duration,
        channel_size: usize,
    ) -> Result<DropGuard<PollResultReceiver, impl FnOnce(PollResultReceiver)>, PollingError> {
        let mut is_polling = self.is_polling.lock().expect("should not be poisoned");
        if *is_polling {
            warn!("background polling already active");
            return Err(PollingError::AlreadyLeased);
        }
        *is_polling = true;

        info!(
            channel_size,
            interval_ms = interval.as_millis() as u64,
            "starting bill validator background polling"
        );

        let (tx, rx) = mpsc::channel(channel_size);

        let is_polling_arc = Arc::clone(&self.is_polling);
        let bv_clone = self.clone();
        let (stop_signal, mut stop_receiver) = oneshot::channel();
        let handle = tokio::spawn(async move {
            loop {
                let poll_result = bv_clone.poll().await;
                if tx.send(poll_result).await.is_err() {
                    error!(
                        "unable to send poll result, receiver may have been dropped. Stopping background polling."
                    );
                    break;
                }

                if stop_receiver.try_recv().is_ok() {
                    info!("received stop signal, stopping bill validator background polling task");
                    break;
                }

                tokio::time::sleep(interval).await;
            }
        });

        let rx_with_guard = DropGuard::new(rx, move |_| {
            if stop_signal.send(()).is_err() {
                warn!("failed to send stop signal to background polling task, aborting it...");
                handle.abort();
            }
            let mut is_polling = is_polling_arc.lock().expect("should not be poisoned");
            *is_polling = false;
            info!("bill validator background polling stopped");
        });

        Ok(rx_with_guard)
    }
}

impl DeviceCommon for BillValidator {
    fn get_device(&self) -> &Device {
        &self.device
    }

    fn get_sender(&self) -> &mpsc::Sender<TransportMessage> {
        &self.sender
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cc_talk_core::cc_talk::{Category, ChecksumType};

    fn create_test_validator() -> BillValidator {
        let (tx, _rx) = mpsc::channel(1);
        let device = Device::new(40, Category::BillValidator, ChecksumType::Crc8);
        BillValidator::new(device, tx)
    }

    #[tokio::test]
    async fn try_background_polling_returns_already_leased_when_called_twice() {
        let validator = create_test_validator();

        // NOTE: This has to be named, and used later, to prevent it from being dropped instantly.
        let first_guard = validator
            .try_background_polling(Duration::from_millis(100), 1)
            .expect("first call should succeed");

        let result = validator.try_background_polling(Duration::from_millis(100), 1);
        assert!(matches!(result, Err(PollingError::AlreadyLeased)));
        drop(first_guard);
    }

    #[tokio::test]
    async fn try_background_polling_can_restart_after_drop() {
        let validator = create_test_validator();

        // Make sure to drop the guard
        let guard = validator
            .try_background_polling(Duration::from_millis(100), 1)
            .expect("first call should succeed");
        drop(guard);

        let new_lease = validator
            .try_background_polling(Duration::from_millis(100), 1)
            .expect("should be able to start polling again after drop");
        drop(new_lease);
    }

    #[tokio::test]
    async fn cloned_instances_share_polling_lock() {
        let validator = create_test_validator();
        let cloned = validator.clone();

        let guard = validator
            .try_background_polling(Duration::from_millis(100), 1)
            .expect("first call should succeed");

        // Cloned instance should also see the lock as held
        let result = cloned.try_background_polling(Duration::from_millis(100), 1);
        assert!(matches!(result, Err(PollingError::AlreadyLeased)));
        drop(guard);

        let new_guard = cloned
            .try_background_polling(Duration::from_millis(100), 1)
            .expect("clone should be able to start polling after original's guard dropped");
        drop(new_guard);
    }
}
