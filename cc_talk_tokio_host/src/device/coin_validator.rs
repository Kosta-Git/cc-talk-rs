#![allow(dead_code)]

use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use cc_talk_core::cc_talk::{BitMask, CoinAcceptorPollResult, CurrencyToken, Device, SorterPath};
use cc_talk_host::{command::Command, device::device_commands::*};
use tokio::sync::{mpsc, oneshot};
use tracing::{debug, error, info, instrument, trace, warn};

use crate::{
    device::base::PollingError, transport::tokio_transport::TransportMessage, util::DropGuard,
};

use super::base::{CommandError, DeviceCommon, DeviceResult};

/// A ccTalk coin validator device driver.
///
/// This struct provides methods to communicate with and control a coin validator
/// over the ccTalk protocol. It supports coin acceptance, inhibit control, sorter
/// path configuration, and background polling for coin events.
///
/// # Cloning
///
/// `CoinValidator` implements [`Clone`] and shares its internal state across clones.
/// This means that polling state and event counters are synchronized between all
/// cloned instances.
#[derive(Debug, Clone)]
pub struct CoinValidator {
    /// The underlying ccTalk device configuration.
    pub device: Device,
    /// Channel sender for communicating with the transport layer.
    pub sender: mpsc::Sender<TransportMessage>,
    event_counter: Arc<Mutex<u8>>,
    is_polling: Arc<Mutex<bool>>,
}

type PollResultReceiver = mpsc::Receiver<DeviceResult<CoinAcceptorPollResult>>;

impl CoinValidator {
    /// Creates a new `CoinValidator` instance.
    ///
    /// # Arguments
    ///
    /// * `device` - The ccTalk device configuration containing address and checksum type.
    /// * `sender` - A channel sender for communicating with the transport layer.
    pub fn new(device: Device, sender: mpsc::Sender<TransportMessage>) -> Self {
        debug!(
            address = device.address(),
            category = ?device.category(),
            "creating coin validator"
        );
        CoinValidator {
            device,
            sender,
            event_counter: Arc::new(Mutex::new(0)),
            is_polling: Arc::new(Mutex::new(false)),
        }
    }

    /// Returns the current event counter value.
    ///
    /// The event counter tracks the number of coin events that have occurred.
    /// It is automatically updated when calling [`poll`](Self::poll).
    pub fn event_counter(&self) -> u8 {
        *self.event_counter.lock().expect("should not be poisoned")
    }

    /// Sets the master inhibit status of the coin validator.
    ///
    /// When master inhibit is enabled (`true`), the coin validator will reject all coins.
    /// When disabled (`false`), coins will be accepted according to individual coin inhibit settings.
    ///
    /// # Arguments
    ///
    /// * `inhibit` - `true` to enable master inhibit (reject all coins), `false` to disable.
    #[instrument(skip(self), fields(inhibit), level = "debug")]
    pub async fn set_master_inhibit(&self, inhibit: bool) -> DeviceResult<()> {
        debug!(inhibit, "setting master inhibit status");
        // TODO: This is a bit goofy, the api for this should be simplified.
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

    /// Enables the master inhibit, causing the validator to reject all coins.
    ///
    /// This is a convenience method equivalent to `set_master_inhibit(true)`.
    pub async fn enable_master_inhibit(&self) -> DeviceResult<()> {
        debug!("enabling master inhibit");
        self.set_master_inhibit(true).await
    }

    /// Disables the master inhibit, allowing the validator to accept coins.
    ///
    /// This is a convenience method equivalent to `set_master_inhibit(false)`.
    /// Note that individual coin inhibits may still prevent specific coins from being accepted.
    pub async fn disable_master_inhibit(&self) -> DeviceResult<()> {
        debug!("disabling master inhibit");
        self.set_master_inhibit(false).await
    }

    /// Returns the master inhibit status of the coin validator.
    ///
    /// Returns `true` if master inhibit is enabled (rejecting all coins),
    /// `false` if disabled (accepting coins).
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
    /// Returns `true` if the validator is rejecting all coins.
    pub async fn is_master_inhibit_enabled(&self) -> DeviceResult<bool> {
        self.get_master_inhibit_status().await
    }

    /// Checks if master inhibit is currently disabled.
    ///
    /// Returns `true` if the validator is accepting coins (subject to individual inhibits).
    pub async fn is_master_inhibit_disabled(&self) -> DeviceResult<bool> {
        let status = self.get_master_inhibit_status().await?;
        Ok(!status)
    }

    /// Sets the default sorter path for accepted coins.
    ///
    /// The sorter path determines which physical output path coins are directed to
    /// after acceptance.
    ///
    /// # Arguments
    ///
    /// * `new_default_path` - The sorter path number (device-specific range).
    #[instrument(skip(self), fields(new_default_path), level = "debug")]
    pub async fn set_default_sorter_path(&self, new_default_path: u8) -> DeviceResult<()> {
        debug!(path = new_default_path, "setting default sorter path");
        let command = ModifyDefaultSorterPathCommand::new(new_default_path);
        let response_packet = self.send_command(command).await?;
        ModifyDefaultSorterPathCommand::new(new_default_path)
            .parse_response(response_packet.get_data()?)
            .map_err(CommandError::from)?;
        info!(path = new_default_path, "default sorter path set");
        Ok(())
    }

    /// Returns the current default sorter path.
    #[instrument(skip(self), level = "debug")]
    pub async fn get_default_sorter_path(&self) -> DeviceResult<SorterPath> {
        trace!("requesting default sorter path");
        let response_packet = self.send_command(RequestDefaultSorterPathCommand).await?;
        let path = RequestDefaultSorterPathCommand
            .parse_response(response_packet.get_data()?)
            .map_err(CommandError::from)?;
        debug!(path = ?path, "default sorter path received");
        Ok(path)
    }

    /// Sets the sorter override status for each of the 8 sorter paths.
    /// The `overrides` array should contain 8 boolean values, where each value corresponds
    /// True: sorter override to a different or default path.
    /// False: no action
    #[instrument(skip(self), level = "debug")]
    pub async fn modify_sorter_override_status(&self, overrides: [bool; 8]) -> DeviceResult<()> {
        debug!(overrides = ?overrides, "modifying sorter override status");
        let mut bitmask = BitMask::<1>::new(8).map_err(|_| CommandError::BufferOverflow)?;
        for (i, should_override) in overrides.iter().enumerate() {
            bitmask
                // Invert value since 0 is override and 1 is no override
                .set_bit(i, !*should_override)
                .map_err(|_| CommandError::BufferOverflow)?;
        }

        let command = ModifySorterOverrideStatusCommand::build(bitmask)
            .map_err(|_| CommandError::BufferOverflow)?;

        let response_packet = self.send_command(command).await?;
        let bitmask = BitMask::<1>::new(8).map_err(|_| CommandError::BufferOverflow)?;
        ModifySorterOverrideStatusCommand::build(bitmask)
            .map_err(|_| CommandError::BufferOverflow)?
            .parse_response(response_packet.get_data()?)
            .map_err(CommandError::from)?;
        info!(overrides = ?overrides, "sorter override status modified");
        Ok(())
    }

    /// Requests the sorter override status for each of the 8 sorter paths.
    /// The returned BitMask will have 8 bits, where each bit corresponds to a sorter
    /// 1: sorter override to a different or default path.
    /// 0: no override
    #[instrument(skip(self), level = "debug")]
    pub async fn request_sorter_override_status(&self) -> DeviceResult<BitMask<1>> {
        trace!("requesting sorter override status");
        let response_packet = self
            .send_command(RequestSorterOverrideStatusCommand)
            .await?;
        let mask = RequestSorterOverrideStatusCommand
            .parse_response(response_packet.get_data()?)
            .map_err(CommandError::from)
            .map(|mut mask| {
                mask.flip();
                mask
            })?;
        debug!(mask = ?mask, "sorter override status received");
        Ok(mask)
    }

    /// Sets the sorter path for a specific coin position.
    ///
    /// # Arguments
    ///
    /// * `coin_position` - The coin position (0-15).
    /// * `path` - The sorter path to assign to this coin.
    #[instrument(skip(self), fields(coin_position, path), level = "debug")]
    pub async fn set_coin_sorter_path(&self, coin_position: u8, path: u8) -> DeviceResult<()> {
        debug!(coin_position, path, "setting coin sorter path");
        let command = ModifySorterPathCommand::new(coin_position, path);
        let response_packet = self.send_command(command).await?;
        ModifySorterPathCommand::new(coin_position, path)
            .parse_response(response_packet.get_data()?)
            .map_err(CommandError::from)?;
        trace!(coin_position, path, "coin sorter path set");
        Ok(())
    }

    /// Returns the sorter path configured for a specific coin position.
    ///
    /// # Arguments
    ///
    /// * `coin_position` - The coin position (0-15).
    #[instrument(skip(self), fields(coin_position), level = "debug")]
    pub async fn get_coin_sorter_path(&self, coin_position: u8) -> DeviceResult<SorterPath> {
        trace!(coin_position, "requesting coin sorter path");
        let response_packet = self
            .send_command(RequestSorterPathCommand::new(coin_position))
            .await?;
        let path = RequestSorterPathCommand::new(coin_position)
            .parse_response(response_packet.get_data()?)
            .map_err(CommandError::from)?;
        trace!(coin_position, path = ?path, "coin sorter path received");
        Ok(path)
    }

    /// Polls the coin validator for buffered credit and error events.
    ///
    /// This method reads the event buffer from the coin validator and returns
    /// any new coin credits or errors since the last poll. The internal event
    /// counter is automatically updated.
    ///
    /// For continuous polling, consider using [`try_background_polling`](Self::try_background_polling)
    /// which handles the polling loop automatically.
    pub async fn poll(&self) -> DeviceResult<CoinAcceptorPollResult> {
        trace!("polling coin validator");
        let response_packet = self
            .send_command(ReadBufferedCreditOrErrorCodeCommand::default())
            .await?;
        let result = ReadBufferedCreditOrErrorCodeCommand::new(self.event_counter())
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
                "coin validator poll returned events"
            );
        }
        Ok(result)
    }

    /// Requests the coin ID (currency token) for a specific coin position.
    ///
    /// # Arguments
    ///
    /// * `coin_position` - The coin position (0-15).
    ///
    /// # Returns
    ///
    /// The currency token identifying the coin type at this position.
    #[instrument(skip(self), fields(coin_position), level = "trace")]
    pub async fn request_coin_id(&self, coin_position: u8) -> DeviceResult<CurrencyToken> {
        trace!(coin_position, "requesting coin ID");
        let response_packet = self
            .send_command(RequestCoinIdCommand::new(coin_position))
            .await?;
        let token = RequestCoinIdCommand::new(coin_position)
            .parse_response(response_packet.get_data()?)
            .map_err(CommandError::from)?;
        trace!(coin_position, token = ?token, "coin ID received");
        Ok(token)
    }

    /// Requests coin IDs for a range of coin positions.
    ///
    /// # Arguments
    ///
    /// * `number_of_coins` - The number of coin positions to query (starting from 0).
    ///
    /// # Returns
    ///
    /// A vector of tuples containing the coin position and its currency token
    /// (or `None` if the request failed for that position).
    #[instrument(skip(self), fields(number_of_coins), level = "debug")]
    pub async fn request_coin_id_range(
        &self,
        number_of_coins: u8,
    ) -> DeviceResult<Vec<(u8, Option<CurrencyToken>)>> {
        debug!(number_of_coins, "requesting coin ID range");
        let mut coins = std::vec::Vec::with_capacity(number_of_coins as usize);
        for i in 0..number_of_coins {
            if let Ok(coin) = self.request_coin_id(i).await {
                coins.push((i, Some(coin)));
            } else {
                coins.push((i, None));
            }
        }
        let configured_count = coins.iter().filter(|(_, c)| c.is_some()).count();
        debug!(
            number_of_coins,
            configured_count, "coin ID range request complete"
        );
        Ok(coins)
    }

    /// Requests coin IDs for all 16 coin positions.
    ///
    /// This is a convenience method equivalent to `request_coin_id_range(16)`.
    pub async fn request_all_coin_id(&self) -> DeviceResult<Vec<(u8, Option<CurrencyToken>)>> {
        debug!("requesting all coin IDs");
        self.request_coin_id_range(16).await
    }

    /// Sets the inhibit status for each of the 16 coin positions.
    /// True: coin is DISABLED
    /// False: coin is ENABLED
    #[instrument(skip(self), level = "debug")]
    pub async fn set_coin_inhibits(&self, inhibits: [bool; 16]) -> DeviceResult<()> {
        let enabled_count = inhibits.iter().filter(|&&i| !i).count();
        debug!(enabled_count, "setting coin inhibits");
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
        info!(enabled_count, "coin inhibits set");
        Ok(())
    }

    /// Sets the same inhibit status for all 16 coin positions.
    ///
    /// # Arguments
    ///
    /// * `inhibit` - `true` to disable all coins, `false` to enable all coins.
    pub async fn set_all_coin_inhibits(&self, inhibit: bool) -> DeviceResult<()> {
        debug!(inhibit, "setting all coin inhibits");
        let inhibits = [inhibit; 16];
        self.set_coin_inhibits(inhibits).await
    }

    /// Requests the inhibit status for each of the 16 coin positions.
    ///
    /// # Returns
    ///
    /// A vector of 16 boolean values where `true` means the coin is disabled
    /// and `false` means the coin is enabled.
    #[instrument(skip(self), level = "debug")]
    pub async fn get_coin_inhibits(&self) -> DeviceResult<Vec<bool>> {
        trace!("requesting coin inhibits");
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
        debug!(enabled_count, "coin inhibits received");
        Ok(inhibits)
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

    /// Starts background polling for coin events.
    ///
    /// This method spawns a background task that continuously polls the coin validator
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
            "starting coin validator background polling"
        );

        let (tx, rx) = mpsc::channel(channel_size);

        let is_polling_arc = Arc::clone(&self.is_polling);
        let cv_clone = self.clone();
        let (stop_signal, mut stop_receiver) = oneshot::channel();
        let handle = tokio::spawn(async move {
            loop {
                let poll_result = cv_clone.poll().await;
                if tx.send(poll_result).await.is_err() {
                    error!(
                        "unable to send poll result, receiver may have been dropped. Stopping background polling."
                    );
                    break;
                }

                if stop_receiver.try_recv().is_ok() {
                    info!("received stop signal, stopping coin validator background polling task");
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
            info!("coin validator background polling stopped");
        });

        Ok(rx_with_guard)
    }
}

impl DeviceCommon for CoinValidator {
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

    fn create_test_validator() -> CoinValidator {
        let (tx, _rx) = mpsc::channel(1);
        let device = Device::new(2, Category::CoinAcceptor, ChecksumType::Crc8);
        CoinValidator::new(device, tx)
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
