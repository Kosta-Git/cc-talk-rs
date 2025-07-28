#![allow(dead_code)]

use std::sync::mpsc;

use cc_talk_core::cc_talk::Header;
use cc_talk_host::command::Command;
use tokio::sync::oneshot;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransportError {
    Timeout,
    Nack,
    BufferOverflow,
}

pub struct CcTalkTokioTransport {
    receiver: mpsc::Receiver<TransportMessage>,
}

pub struct TransportMessage {
    pub address: u8,
    pub checksum_type: u8,
    pub header: Header,
    pub data: Vec<u8>,
    pub respond_to: oneshot::Sender<Result<Vec<u8>, TransportError>>,
}

impl CcTalkTokioTransport {
    pub fn new(receiver: mpsc::Receiver<TransportMessage>) -> Self {
        CcTalkTokioTransport { receiver }
    }

    pub fn receiver(&self) -> &mpsc::Receiver<TransportMessage> {
        &self.receiver
    }
}
