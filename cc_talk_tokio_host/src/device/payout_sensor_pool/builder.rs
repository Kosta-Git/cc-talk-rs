use std::time::Duration;

use crate::device::payout::PayoutDevice;

use super::pool::PayoutSensorPool;

/// Builder for constructing a [`PayoutSensorPool`].
#[derive(Debug)]
pub struct PayoutSensorPoolBuilder {
    hoppers: Vec<PayoutDevice>,
    polling_interval: Duration,
    channel_size: usize,
}

impl PayoutSensorPoolBuilder {
    /// Creates a new builder with default settings.
    #[must_use]
    pub fn new() -> Self {
        Self {
            hoppers: Vec::new(),
            polling_interval: Duration::from_millis(500),
            channel_size: 16,
        }
    }

    /// Adds a single hopper to the pool.
    #[must_use]
    pub fn add_hopper(mut self, hopper: PayoutDevice) -> Self {
        self.hoppers.push(hopper);
        self
    }

    /// Adds multiple hoppers to the pool.
    #[must_use]
    pub fn add_hoppers(mut self, hoppers: impl IntoIterator<Item = PayoutDevice>) -> Self {
        self.hoppers.extend(hoppers);
        self
    }

    /// Sets the polling interval for background sensor monitoring.
    ///
    /// Defaults to 500ms.
    #[must_use]
    pub const fn polling_interval(mut self, interval: Duration) -> Self {
        self.polling_interval = interval;
        self
    }

    /// Sets the channel buffer size for sensor events.
    ///
    /// Defaults to 16.
    #[must_use]
    pub const fn channel_size(mut self, size: usize) -> Self {
        self.channel_size = size;
        self
    }

    /// Builds the [`PayoutSensorPool`].
    #[must_use]
    pub fn build(self) -> PayoutSensorPool {
        PayoutSensorPool::new(self.hoppers, self.polling_interval, self.channel_size)
    }
}

impl Default for PayoutSensorPoolBuilder {
    fn default() -> Self {
        Self::new()
    }
}
