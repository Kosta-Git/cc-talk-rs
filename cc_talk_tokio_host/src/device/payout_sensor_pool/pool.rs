use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::Duration,
};

use tokio::sync::{self, mpsc, oneshot};
use tracing::{debug, error, info, trace, warn};

use crate::{
    device::{base::PollingError, payout::PayoutDevice, payout_pool::HopperInventoryLevel},
    util::DropGuard,
};

use super::{
    builder::PayoutSensorPoolBuilder,
    error::{PayoutSensorPoolError, PayoutSensorPoolResult},
    event::{HopperSensorError, HopperSensorReading, RecoveryReason, SensorEvent},
};

/// The inventory level at or above which a hopper is automatically recovered
/// from the empty state.
const RECOVERY_THRESHOLD: HopperInventoryLevel = HopperInventoryLevel::Medium;

/// Guard returned by [`PayoutSensorPool::try_start_polling`].
///
/// Wraps a receiver for [`SensorEvent`]s. When dropped, the background
/// polling task is automatically stopped.
pub type SensorPollGuard =
    DropGuard<mpsc::Receiver<SensorEvent>, Box<dyn FnOnce(mpsc::Receiver<SensorEvent>) + Send>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PollingStatus {
    Paused,
    Running,
}

/// Standalone sensor monitoring for a set of [`PayoutDevice`] instances.
///
/// `PayoutSensorPool` provides continuous inventory monitoring with
/// empty-state management. It polls `PayoutDevice::get_sensor_status()`
/// directly, without any dependency on [`crate::device::payout_pool::PayoutPool`].
///
/// # Cloning
///
/// `PayoutSensorPool` implements [`Clone`] and shares its internal state
/// across clones.
#[derive(Clone)]
pub struct PayoutSensorPool {
    hoppers: Vec<PayoutDevice>,
    /// Last known inventory level per hopper.
    ///
    /// When a hopper is marked empty, its level is set to
    /// [`HopperInventoryLevel::Empty`] and remains sticky until recovery
    /// threshold is met or [`mark_non_empty`](Self::mark_non_empty) is called.
    last_levels: Arc<Mutex<HashMap<u8, HopperInventoryLevel>>>,
    /// Whether background polling is active.
    is_polling: Arc<Mutex<bool>>,
    polling_interval: Duration,
    channel_size: usize,
}

impl PayoutSensorPool {
    /// Creates a new builder.
    #[must_use]
    pub fn builder() -> PayoutSensorPoolBuilder {
        PayoutSensorPoolBuilder::new()
    }

    /// Creates a new `PayoutSensorPool` with the given configuration.
    ///
    /// Prefer using [`PayoutSensorPool::builder()`] for construction.
    pub(crate) fn new(
        hoppers: Vec<PayoutDevice>,
        polling_interval: Duration,
        channel_size: usize,
    ) -> Self {
        Self {
            hoppers,
            last_levels: Arc::new(Mutex::new(HashMap::new())),
            is_polling: Arc::new(Mutex::new(false)),
            polling_interval,
            channel_size,
        }
    }

    /// Returns the number of hoppers in the pool.
    #[must_use]
    pub fn hopper_count(&self) -> usize {
        self.hoppers.len()
    }

    /// Returns the ccTalk addresses of all hoppers in the pool.
    #[must_use]
    pub fn hopper_addresses(&self) -> Vec<u8> {
        self.hoppers.iter().map(|h| h.device.address()).collect()
    }

    /// Returns `true` if the pool contains a hopper with the given address.
    fn has_hopper(&self, address: u8) -> bool {
        self.hoppers.iter().any(|h| h.device.address() == address)
    }

    /// Marks a hopper as empty by setting its inventory level to
    /// [`HopperInventoryLevel::Empty`].
    ///
    /// The level remains `Empty` until the sensor detects a level at or above
    /// the recovery threshold, or [`mark_non_empty`](Self::mark_non_empty) is
    /// called.
    ///
    /// # Errors
    ///
    /// Returns [`PayoutSensorPoolError::HopperNotFound`] if the address
    /// is not in the pool.
    pub fn mark_empty(&self, address: u8) -> PayoutSensorPoolResult<()> {
        if !self.has_hopper(address) {
            return Err(PayoutSensorPoolError::HopperNotFound(address));
        }

        self.last_levels
            .lock()
            .expect("should not be poisoned")
            .insert(address, HopperInventoryLevel::Empty);

        info!(address, "hopper marked empty");
        Ok(())
    }

    /// Clears the empty state of a hopper by resetting its inventory level
    /// to [`HopperInventoryLevel::Unknown`].
    ///
    /// The actual level will be updated on the next sensor poll.
    ///
    /// # Errors
    ///
    /// Returns [`PayoutSensorPoolError::HopperNotFound`] if the address
    /// is not in the pool.
    pub fn mark_non_empty(&self, address: u8) -> PayoutSensorPoolResult<()> {
        if !self.has_hopper(address) {
            return Err(PayoutSensorPoolError::HopperNotFound(address));
        }

        self.last_levels
            .lock()
            .expect("should not be poisoned")
            .insert(address, HopperInventoryLevel::Unknown);

        info!(address, "hopper marked non-empty");
        Ok(())
    }

    /// Returns `true` if the hopper's inventory level is
    /// [`HopperInventoryLevel::Empty`].
    #[must_use]
    pub fn is_empty(&self, address: u8) -> bool {
        self.last_levels
            .lock()
            .expect("should not be poisoned")
            .get(&address)
            .is_some_and(|level| *level == HopperInventoryLevel::Empty)
    }

    /// Returns the last polled inventory level for a specific hopper.
    #[must_use]
    pub fn last_inventory(&self, address: u8) -> Option<HopperInventoryLevel> {
        self.last_levels
            .lock()
            .expect("should not be poisoned")
            .get(&address)
            .copied()
    }

    /// Returns the last polled inventory levels for all hoppers.
    #[must_use]
    pub fn last_inventories(&self) -> HashMap<u8, HopperInventoryLevel> {
        self.last_levels
            .lock()
            .expect("should not be poisoned")
            .clone()
    }

    /// Starts background sensor polling.
    ///
    /// Spawns a background task that continuously polls all hoppers for
    /// inventory levels via [`PayoutDevice::get_sensor_status()`] and sends
    /// [`SensorEvent`]s through a channel.
    ///
    /// The returned [`SensorPollGuard`] wraps a receiver channel. When the
    /// guard is dropped, the background polling task is automatically stopped.
    ///
    /// # Errors
    ///
    /// Returns [`PollingError::AlreadyLeased`] if background polling is
    /// already active.
    #[must_use = "nothing happens if the result is not used"]
    pub fn try_start_polling(
        &self,
        polling_status: sync::watch::Receiver<PollingStatus>,
    ) -> Result<SensorPollGuard, PollingError> {
        let mut is_polling = self.is_polling.lock().expect("should not be poisoned");
        if *is_polling {
            warn!("sensor background polling already active");
            return Err(PollingError::AlreadyLeased);
        }
        *is_polling = true;

        info!(
            channel_size = self.channel_size,
            polling_interval_ms = self.polling_interval.as_millis() as u64,
            "starting sensor background polling"
        );

        let (tx, rx) = mpsc::channel(self.channel_size);
        let (stop_signal, mut stop_receiver) = oneshot::channel::<()>();

        let pool_clone = self.clone();

        let handle = tokio::spawn(async move {
            let mut polling_status = polling_status;
            loop {
                let mut inventories = Vec::new();
                let mut errors = Vec::new();

                if *polling_status.borrow() == PollingStatus::Paused {
                    if polling_status.changed().await.is_err() {
                        info!("polling status sender dropped, stopping sensor background polling");
                        break;
                    }
                    continue;
                }

                for hopper in &pool_clone.hoppers {
                    let address = hopper.device.address();

                    match hopper.get_sensor_status().await {
                        Ok((_level_raw, status)) => {
                            let sensor_level = HopperInventoryLevel::from(status);

                            let previous = {
                                let last = pool_clone
                                    .last_levels
                                    .lock()
                                    .expect("should not be poisoned");
                                last.get(&address).copied()
                            };

                            let was_empty = previous == Some(HopperInventoryLevel::Empty);

                            // If the hopper was marked empty, only update
                            // its level when the sensor reports at or above
                            // the recovery threshold.
                            let effective_level = if was_empty
                                && sensor_level < RECOVERY_THRESHOLD
                            {
                                HopperInventoryLevel::Empty
                            } else {
                                sensor_level
                            };

                            // Store the effective level.
                            {
                                let mut last = pool_clone
                                    .last_levels
                                    .lock()
                                    .expect("should not be poisoned");
                                last.insert(address, effective_level);
                            }

                            // Detect level changes.
                            if let Some(prev) = previous
                                && prev != effective_level
                            {
                                trace!(address, %prev, %effective_level, "hopper level changed");
                                let _ = tx
                                    .send(SensorEvent::LevelChanged {
                                        address,
                                        previous: prev,
                                        current: effective_level,
                                    })
                                    .await;
                            }

                            // Auto-recovery: hopper was empty and sensor
                            // now reports at or above the threshold.
                            if was_empty && sensor_level >= RECOVERY_THRESHOLD {
                                info!(address, %sensor_level, "hopper auto-recovered from empty state");
                                let _ = tx
                                    .send(SensorEvent::MarkedNonEmpty {
                                        address,
                                        reason: RecoveryReason::SensorRecovery {
                                            level: sensor_level,
                                        },
                                    })
                                    .await;
                            }

                            inventories.push(HopperSensorReading {
                                address,
                                level: effective_level,
                                status,
                            });
                        }
                        Err(e) => {
                            debug!(address, %e, "hopper sensor poll error");
                            errors.push(HopperSensorError { address, error: e });
                        }
                    }
                }

                // Emit the full inventory update.
                if tx
                    .send(SensorEvent::InventoryUpdate {
                        inventories,
                        errors,
                    })
                    .await
                    .is_err()
                {
                    error!("sensor event receiver dropped, stopping background polling");
                    break;
                }

                // Check stop signal.
                if stop_receiver.try_recv().is_ok() {
                    info!("received stop signal, stopping sensor background polling");
                    break;
                }

                tokio::time::sleep(pool_clone.polling_interval).await;
            }
        });

        let is_polling_arc = Arc::clone(&self.is_polling);
        let cleanup: Box<dyn FnOnce(mpsc::Receiver<SensorEvent>) + Send> = Box::new(move |_| {
            if stop_signal.send(()).is_err() {
                warn!("failed to send stop signal to sensor polling task, aborting it...");
                handle.abort();
            }
            let mut is_polling = is_polling_arc.lock().expect("should not be poisoned");
            *is_polling = false;
            info!("sensor background polling stopped");
        });
        let rx_with_guard = DropGuard::new(rx, cleanup);

        Ok(rx_with_guard)
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

    fn create_sensor_pool() -> PayoutSensorPool {
        let h1 = create_test_hopper(3);
        let h2 = create_test_hopper(4);
        let h3 = create_test_hopper(5);

        PayoutSensorPool::builder()
            .add_hopper(h1)
            .add_hopper(h2)
            .add_hopper(h3)
            .build()
    }

    #[test]
    fn mark_empty_and_non_empty() {
        let sensor = create_sensor_pool();

        assert!(!sensor.is_empty(3));
        sensor.mark_empty(3).unwrap();
        assert!(sensor.is_empty(3));
        assert_eq!(
            sensor.last_inventory(3),
            Some(HopperInventoryLevel::Empty)
        );

        sensor.mark_non_empty(3).unwrap();
        assert!(!sensor.is_empty(3));
        assert_eq!(
            sensor.last_inventory(3),
            Some(HopperInventoryLevel::Unknown)
        );
    }

    #[test]
    fn mark_empty_returns_error_for_unknown_hopper() {
        let sensor = create_sensor_pool();
        let result = sensor.mark_empty(99);
        assert!(matches!(
            result,
            Err(PayoutSensorPoolError::HopperNotFound(99))
        ));
    }

    #[test]
    fn mark_non_empty_returns_error_for_unknown_hopper() {
        let sensor = create_sensor_pool();
        let result = sensor.mark_non_empty(99);
        assert!(matches!(
            result,
            Err(PayoutSensorPoolError::HopperNotFound(99))
        ));
    }

    #[test]
    fn is_empty_returns_false_by_default() {
        let sensor = create_sensor_pool();

        assert!(!sensor.is_empty(3));
        assert!(!sensor.is_empty(4));
        assert!(!sensor.is_empty(5));
    }

    #[test]
    fn mark_empty_sets_level_to_empty() {
        let sensor = create_sensor_pool();

        sensor.mark_empty(3).unwrap();
        sensor.mark_empty(5).unwrap();

        assert!(sensor.is_empty(3));
        assert!(!sensor.is_empty(4));
        assert!(sensor.is_empty(5));

        assert_eq!(
            sensor.last_inventory(3),
            Some(HopperInventoryLevel::Empty)
        );
        assert_eq!(sensor.last_inventory(4), None);
        assert_eq!(
            sensor.last_inventory(5),
            Some(HopperInventoryLevel::Empty)
        );
    }

    #[test]
    fn hopper_count_and_addresses() {
        let sensor = create_sensor_pool();

        assert_eq!(sensor.hopper_count(), 3);

        let mut addresses = sensor.hopper_addresses();
        addresses.sort();
        assert_eq!(addresses, vec![3, 4, 5]);
    }

    #[tokio::test]
    async fn try_start_polling_returns_already_leased_when_called_twice() {
        let sensor = create_sensor_pool();
        let (_tx, rx) = sync::watch::channel(PollingStatus::Running);

        let _guard = sensor
            .try_start_polling(rx.clone())
            .expect("first call should succeed");
        let result = sensor.try_start_polling(rx);
        assert!(matches!(result, Err(PollingError::AlreadyLeased)));
    }

    #[tokio::test]
    async fn try_start_polling_can_restart_after_drop() {
        let sensor = create_sensor_pool();
        let (_tx, rx) = sync::watch::channel(PollingStatus::Running);

        {
            let _guard = sensor
                .try_start_polling(rx.clone())
                .expect("first call should succeed");
            // Guard is dropped here.
        }

        // Give the runtime a moment to process the drop.
        tokio::time::sleep(Duration::from_millis(10)).await;

        let _guard = sensor
            .try_start_polling(rx)
            .expect("should succeed after guard dropped");
    }
}
