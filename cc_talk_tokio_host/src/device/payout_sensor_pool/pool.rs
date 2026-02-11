use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex},
    time::Duration,
};

use tokio::sync::{mpsc, oneshot};
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
#[derive(Debug, Clone)]
pub struct PayoutSensorPool {
    hoppers: Vec<PayoutDevice>,
    /// Per-hopper empty flags.
    empty_hoppers: Arc<Mutex<HashSet<u8>>>,
    /// Last known inventory level per hopper.
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
            empty_hoppers: Arc::new(Mutex::new(HashSet::new())),
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

    /// Marks a hopper as empty.
    ///
    /// # Errors
    ///
    /// Returns [`PayoutSensorPoolError::HopperNotFound`] if the address
    /// is not in the pool.
    pub fn mark_empty(&self, address: u8) -> PayoutSensorPoolResult<()> {
        if !self.has_hopper(address) {
            return Err(PayoutSensorPoolError::HopperNotFound(address));
        }

        self.empty_hoppers
            .lock()
            .expect("should not be poisoned")
            .insert(address);

        info!(address, "hopper marked empty");
        Ok(())
    }

    /// Marks a hopper as non-empty.
    ///
    /// # Errors
    ///
    /// Returns [`PayoutSensorPoolError::HopperNotFound`] if the address
    /// is not in the pool.
    pub fn mark_non_empty(&self, address: u8) -> PayoutSensorPoolResult<()> {
        if !self.has_hopper(address) {
            return Err(PayoutSensorPoolError::HopperNotFound(address));
        }

        self.empty_hoppers
            .lock()
            .expect("should not be poisoned")
            .remove(&address);

        info!(address, "hopper marked non-empty");
        Ok(())
    }

    /// Returns `true` if the hopper is currently marked as empty.
    #[must_use]
    pub fn is_empty(&self, address: u8) -> bool {
        self.empty_hoppers
            .lock()
            .expect("should not be poisoned")
            .contains(&address)
    }

    /// Returns the set of all hopper addresses currently marked as empty.
    #[must_use]
    pub fn empty_hoppers(&self) -> HashSet<u8> {
        self.empty_hoppers
            .lock()
            .expect("should not be poisoned")
            .clone()
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
    pub fn try_start_polling(&self) -> Result<SensorPollGuard, PollingError> {
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
            loop {
                let mut inventories = Vec::new();
                let mut errors = Vec::new();

                for hopper in &pool_clone.hoppers {
                    let address = hopper.device.address();

                    match hopper.get_sensor_status().await {
                        Ok((_level_raw, status)) => {
                            let level = HopperInventoryLevel::from(status);

                            // Detect level changes.
                            let previous = {
                                let mut last = pool_clone
                                    .last_levels
                                    .lock()
                                    .expect("should not be poisoned");
                                last.insert(address, level)
                            };

                            if let Some(prev) = previous
                                && prev != level
                            {
                                trace!(address, %prev, %level, "hopper level changed");
                                let _ = tx
                                    .send(SensorEvent::LevelChanged {
                                        address,
                                        previous: prev,
                                        current: level,
                                    })
                                    .await;
                            }

                            // Auto-recovery: if hopper is marked empty and level >= threshold.
                            let is_marked_empty = pool_clone
                                .empty_hoppers
                                .lock()
                                .expect("should not be poisoned")
                                .contains(&address);

                            if is_marked_empty && level >= RECOVERY_THRESHOLD {
                                pool_clone
                                    .empty_hoppers
                                    .lock()
                                    .expect("should not be poisoned")
                                    .remove(&address);

                                info!(address, %level, "hopper auto-recovered from empty state");
                                let _ = tx
                                    .send(SensorEvent::MarkedNonEmpty {
                                        address,
                                        reason: RecoveryReason::SensorRecovery { level },
                                    })
                                    .await;
                            }

                            inventories.push(HopperSensorReading {
                                address,
                                level,
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

        sensor.mark_non_empty(3).unwrap();
        assert!(!sensor.is_empty(3));
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
    fn empty_hoppers_returns_correct_set() {
        let sensor = create_sensor_pool();

        assert!(sensor.empty_hoppers().is_empty());

        sensor.mark_empty(3).unwrap();
        sensor.mark_empty(5).unwrap();

        let empty = sensor.empty_hoppers();
        assert_eq!(empty.len(), 2);
        assert!(empty.contains(&3));
        assert!(empty.contains(&5));
        assert!(!empty.contains(&4));
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

        let _guard = sensor
            .try_start_polling()
            .expect("first call should succeed");
        let result = sensor.try_start_polling();
        assert!(matches!(result, Err(PollingError::AlreadyLeased)));
    }

    #[tokio::test]
    async fn try_start_polling_can_restart_after_drop() {
        let sensor = create_sensor_pool();

        {
            let _guard = sensor
                .try_start_polling()
                .expect("first call should succeed");
            // Guard is dropped here.
        }

        // Give the runtime a moment to process the drop.
        tokio::time::sleep(Duration::from_millis(10)).await;

        let _guard = sensor
            .try_start_polling()
            .expect("should succeed after guard dropped");
    }
}
