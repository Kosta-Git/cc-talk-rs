use std::{collections::HashMap, time::Duration};

use tokio::sync::{mpsc, watch};

use crate::device::{payout::PayoutDevice, payout_pool::HopperInventoryLevel};

/// Represents an action to be performed by the pool sensor task.
enum TaskAction {
    /// Enable the sensor polling.
    Enable,
    /// Disable the sensor polling.
    Disable,
}

/// Handle to the pool sensor task.
#[derive(Debug)]
struct PoolSensorTask {
    action_channel: mpsc::Sender<TaskAction>,
    inventory_status_watch: watch::Receiver<HashMap<u8, HopperInventoryLevel>>,
}

impl PoolSensorTask {
    #[must_use]
    const fn new(hoppers: Vec<PayoutDevice>, polling_interval: Duration) -> Self {
        let (action_tx, mut action_rx) = mpsc::channel::<TaskAction>(10);
        let (inventory_status_tx, inventory_status_rx) =
            watch::channel::<HashMap<u8, HopperInventoryLevel>>(HashMap::new());

        tokio::spawn(async move {
            let mut enabled = false;

            loop {
                tokio::select! {
                    Some(action) = action_receiver.recv() => {
                        match action {
                            TaskAction::Enable => {
                                enabled = true;
                            }
                            TaskAction::Disable => {
                                enabled = false;
                            }
                        }
                    }
                    _ = tokio::time::sleep(polling_interval) => {
                        if enabled {
                            let mut inventory_status = HashMap::new();
                            for hopper in &hoppers {
                                if let Ok(level) = hopper.get_inventory_level().await {
                                    inventory_status.insert(hopper.address(), level);
                                }
                            }
                            let _ = inventory_status_watch.send(inventory_status);
                        }
                    }
                }
            }
        });

        Self {
            action_channel,
            inventory_status_watch,
        }
    }

    /// Enable the sensor polling.
    pub async fn enable(&self) -> Result<(), mpsc::error::SendError<TaskAction>> {
        self.action_channel.send(TaskAction::Enable).await
    }

    /// Disable the sensor polling.
    pub async fn disable(&self) -> Result<(), mpsc::error::SendError<TaskAction>> {
        self.action_channel.send(TaskAction::Disable).await
    }

    /// Retrieve the latest inventory state.
    pub async fn lastest_state(&mut self) -> HashMap<u8, HopperInventoryLevel> {
        self.inventory_status_watch.borrow().clone()
    }
}
