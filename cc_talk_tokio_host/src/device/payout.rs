use cc_talk_core::cc_talk::Device;
use tokio::sync::mpsc;

use crate::transport::tokio_transport::TransportMessage;

pub struct PayoutDevice {
    pub device: Device,
    pub sender: mpsc::Sender<TransportMessage>,
}

impl PayoutDevice {
    pub fn new(device: Device, sender: mpsc::Sender<TransportMessage>) -> Self {
        PayoutDevice { device, sender }
    }

    pub fn get_device(&self) -> &Device {
        &self.device
    }

    pub fn get_sender(&self) -> &mpsc::Sender<TransportMessage> {
        &self.sender
    }
}
