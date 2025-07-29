#![allow(dead_code)]

use cc_talk_core::cc_talk::{
    Category, ChecksumType, DATA_LENGTH_OFFSET, Device, Header, MAX_BLOCK_LENGTH, Packet,
    deserializer::deserialize, serializer::serialize,
};
use cc_talk_host::command::Command;
use std::time::Duration;
use thiserror::Error;
use tokio::{
    io::{self, AsyncReadExt, AsyncWriteExt},
    net::UnixStream,
    sync::{mpsc, oneshot},
    time::timeout,
};
use tracing::{error, info, trace};

use super::retry::RetryConfig;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum TransportError {
    #[error("Timeout")]
    Timeout,
    #[error("NACK")]
    Nack,
    #[error("Buffer overflow")]
    BufferOverflow,
    #[error("Packet creation error")]
    PacketCreationError,
    #[error("Socket write error")]
    SocketWriteError,
    #[error("Socket read error")]
    SocketReadError,
    #[error("Checksum error")]
    ChecksumError,
    #[error("Max retries exceeded")]
    MaxRetriesExceeded,
}

pub struct CcTalkTokioTransport {
    receiver: mpsc::Receiver<TransportMessage>,
    socket_path: String,
    timeout: Duration,
    retry_config: RetryConfig,
    minimum_delay: Duration,
    send_buffer: Vec<u8>,
    receive_buffer: Vec<u8>,
}

pub struct TransportMessage {
    pub address: u8,
    pub checksum_type: ChecksumType,
    pub header: Header,
    pub data: Vec<u8>,
    pub respond_to: oneshot::Sender<Result<Vec<u8>, TransportError>>,
}

impl TransportMessage {
    pub fn new<T>(
        device: &Device,
        command: T,
        respond_to: oneshot::Sender<Result<Vec<u8>, TransportError>>,
    ) -> Self
    where
        T: Command,
    {
        TransportMessage {
            address: device.address(),
            checksum_type: *device.checksum_type(),
            header: command.header(),
            data: command.data().to_vec(),
            respond_to,
        }
    }
}

#[derive(Debug)]
struct Message<'a> {
    pub address: u8,
    pub checksum_type: ChecksumType,
    pub header: Header,
    pub data: &'a [u8],
}

impl<'a> Message<'a> {
    fn from(transport_message: &'a TransportMessage) -> Self {
        Message {
            address: transport_message.address,
            checksum_type: transport_message.checksum_type,
            header: transport_message.header,
            data: &transport_message.data,
        }
    }
}

impl CcTalkTokioTransport {
    pub fn new(
        receiver: mpsc::Receiver<TransportMessage>,
        socket_path: String,
        timeout: Duration,
        minimum_delay: Duration,
        retry_config: RetryConfig,
    ) -> Self {
        CcTalkTokioTransport {
            receiver,
            socket_path,
            timeout,
            minimum_delay,
            retry_config,
            send_buffer: vec![0; MAX_BLOCK_LENGTH],
            receive_buffer: vec![0; MAX_BLOCK_LENGTH],
        }
    }

    pub async fn run(mut self) -> io::Result<()> {
        let mut socket = match UnixStream::connect(&self.socket_path).await {
            Ok(socket) => {
                info!("connected to socket at {}", &self.socket_path);
                socket
            }
            Err(error) => {
                error!("unable to connect to socket: {}", error);
                return Err(error);
            }
        };

        loop {
            if let Some(transport_message) = self.receiver.recv().await {
                trace!(
                    "received message for {}, header: {}",
                    transport_message.address, transport_message.header as u8
                );

                let mut retry_instance = self.retry_config.create_retry_instance();
                let mut response_data: Option<Vec<u8>> = None;
                let message = Message::from(&transport_message);
                while retry_instance.can_retry() {
                    match handle_message(
                        &message,
                        &mut self.send_buffer,
                        &mut self.receive_buffer,
                        self.timeout,
                        &mut socket,
                    )
                    .await
                    {
                        Ok(data) => {
                            response_data = Some(data);
                            break;
                        }
                        Err((error_code, error_message)) => {
                            error!("{} handling message. Info: {}", error_code, error_message);
                            retry_instance.evaluate_and_wait(error_code).await;
                        }
                    }
                }

                if let Some(data) = response_data {
                    transport_message.respond_to.send(Ok(data)).ok();
                } else {
                    error!(
                        "too many retries for message to {}, header: {}",
                        transport_message.address, transport_message.header as u8
                    );
                    transport_message
                        .respond_to
                        .send(Err(retry_instance.last_error()))
                        .ok();
                }

                if !self.minimum_delay.is_zero() {
                    tokio::time::sleep(self.minimum_delay).await;
                }
            }
        }
    }
}

fn handle_error(message: TransportMessage, error: TransportError, error_message: &str) {
    error!("{}: {:?}", error_message, error);
    if message.respond_to.send(Err(error)).is_err() {
        error!("failed to send error response for message - receiver dropped");
    }
}

fn build_packet(message: &Message, packet: &mut Packet<&mut [u8]>) -> Result<(), TransportError> {
    packet
        .set_destination(message.address)
        .map_err(|_| TransportError::BufferOverflow)?;
    packet
        .set_source(1)
        .map_err(|_| TransportError::BufferOverflow)?;
    packet
        .set_header(message.header)
        .map_err(|_| TransportError::BufferOverflow)?;
    packet
        .set_data(message.data)
        .map_err(|_| TransportError::BufferOverflow)?;

    Ok(())
}

async fn handle_send(
    message: &Message<'_>,
    send_packet: &mut Packet<&mut [u8]>,
    socket: &mut UnixStream,
    write_timeout: Duration,
) -> Result<(), (TransportError, &'static str)> {
    trace!("building packet for message");
    if let Err(error) = build_packet(message, send_packet) {
        return Err((error, "failed to build packet"));
    }

    trace!("serializing packet");
    if serialize(
        &Device::new(message.address, Category::Unknown, message.checksum_type),
        send_packet,
    )
    .is_err()
    {
        return Err((
            TransportError::PacketCreationError,
            "failed to serialize packet",
        ));
    }

    let packet_length = send_packet.get_logical_size();
    trace!(
        "writing packet of length {}, {:?}",
        packet_length,
        &send_packet.as_slice()[..packet_length]
    );
    match timeout(
        write_timeout,
        socket.write_all(&send_packet.as_slice()[..packet_length]),
    )
    .await
    {
        Ok(Ok(_)) => {
            trace!("packet sent successfully");
            let _ = socket.flush().await;
            let _ = socket
                .read_exact(&mut send_packet.as_mut_slice()[..packet_length])
                .await;
            Ok(())
        }
        Ok(Err(_)) => Err((
            TransportError::SocketWriteError,
            "failed to write to socket",
        )),
        Err(_) => Err((TransportError::Timeout, "timeout writing to socket")),
    }
}

async fn read_packet_header(
    read_buffer: &mut [u8],
    read_timeout: Duration,
    socket: &mut UnixStream,
) -> Result<usize, (TransportError, &'static str)> {
    match timeout(read_timeout, socket.read_exact(&mut read_buffer[..5])).await {
        Ok(Ok(read_bytes)) => {
            trace!("read response header ({} bytes)", read_bytes);
            Ok(read_bytes)
        }
        Ok(Err(_)) => Err((
            TransportError::SocketReadError,
            "failed to read response header",
        )),
        Err(_) => Err((TransportError::Timeout, "timeout reading response header")),
    }
}

async fn read_full_packet(
    read_buffer: &mut [u8],
    read_timeout: Duration,
    socket: &mut UnixStream,
) -> Result<usize, (TransportError, &'static str)> {
    let data_length = read_buffer[DATA_LENGTH_OFFSET] as usize;
    trace!(
        "data length: {}, buffer {:?}",
        data_length,
        &read_buffer[..5]
    );
    if data_length > 0 {
        return match timeout(
            read_timeout,
            socket.read_exact(&mut read_buffer[5..(5 + data_length)]),
        )
        .await
        {
            Ok(Ok(bytes_read)) => {
                trace!("read {} bytes of response data", data_length);
                Ok(bytes_read)
            }
            Ok(Err(_)) => Err((
                TransportError::SocketReadError,
                "failed to read response data",
            )),
            Err(_) => Err((TransportError::Timeout, "timeout reading response data")),
        };
    }
    Ok(0)
}

async fn handle_message(
    message: &Message<'_>,
    send_buffer: &mut [u8],
    read_buffer: &mut [u8],
    rw_timeout: Duration,
    socket: &mut UnixStream,
) -> Result<Vec<u8>, (TransportError, &'static str)> {
    let mut send_packet = Packet::new(send_buffer);

    if let Err((error_code, error_message)) =
        handle_send(message, &mut send_packet, socket, rw_timeout).await
    {
        return Err((error_code, error_message));
    }

    let mut bytes_read = match read_packet_header(read_buffer, rw_timeout, socket).await {
        Ok(bytes_read) => bytes_read,
        Err((error_code, error_message)) => return Err((error_code, error_message)),
    };

    bytes_read += match read_full_packet(read_buffer, rw_timeout, socket).await {
        Ok(bytes_read) => bytes_read,
        Err((error_code, error_message)) => {
            return Err((error_code, error_message));
        }
    };

    let mut response_packet = Packet::new(&mut read_buffer[..bytes_read]);
    if deserialize(&mut response_packet, message.checksum_type).is_err() {
        return Err((
            TransportError::ChecksumError,
            "failed to deserialize response packet",
        ));
    }

    Ok(read_buffer[..bytes_read].to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;
    use cc_talk_core::cc_talk::{ChecksumType, Header, MAX_BLOCK_LENGTH};
    use std::path::Path;
    use std::time::Duration;
    use tempfile::TempDir;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::UnixListener;
    use tokio::sync::{mpsc, oneshot};

    fn create_test_socket_path() -> (TempDir, String) {
        let temp_dir = TempDir::new().unwrap();
        let socket_path = temp_dir
            .path()
            .join("test.sock")
            .to_string_lossy()
            .to_string();
        (temp_dir, socket_path)
    }

    fn create_test_transport(
        receiver: mpsc::Receiver<TransportMessage>,
        socket_path: String,
    ) -> CcTalkTokioTransport {
        CcTalkTokioTransport {
            receiver,
            socket_path,
            retry_config: RetryConfig {
                max_retries: 0,
                retry_delay: Duration::from_millis(100),
                retry_on_timeout: true,
                retry_on_checksum_error: true,
                retry_on_nack: false,
                retry_on_socket_error: true,
            },
            timeout: Duration::from_millis(100),
            minimum_delay: Duration::from_millis(0),
            send_buffer: vec![0u8; MAX_BLOCK_LENGTH],
            receive_buffer: vec![0u8; MAX_BLOCK_LENGTH],
        }
    }

    async fn base_mock_device<F>(socket_path: String, replier: F)
    where
        F: AsyncFn(UnixStream) -> (),
    {
        if Path::new(&socket_path).exists() {
            std::fs::remove_file(&socket_path).ok();
        }

        let listener = UnixListener::bind(&socket_path).unwrap();

        while let Ok((stream, _)) = listener.accept().await {
            let _ = replier(stream).await;
        }
    }

    async fn mock_device_ack_responder(socket_path: String) {
        base_mock_device(socket_path, |mut stream: UnixStream| async move {
            let mut buffer = [0u8; 256];

            while let Ok(n) = stream.read(&mut buffer).await {
                if n == 0 {
                    break;
                }

                let request = &buffer[..n];
                if n >= 5 {
                    let dest = request[0];
                    let src = request[2];

                    let mut response = vec![src, 0x00, dest, 0x00]; // dest, len=0, src, header=Reply

                    let checksum: u16 = response.iter().map(|&b| b as u16).sum();
                    response.push((256 - (checksum % 256)) as u8);

                    let _ = stream.write_all(&response).await;
                }
            }
        })
        .await;
    }

    async fn mock_device_no_response(socket_path: String) {
        base_mock_device(socket_path, |mut stream: UnixStream| async move {
            let mut buffer = [0u8; 256];
            let _ = stream.read(&mut buffer).await.unwrap();
            tokio::time::sleep(Duration::from_secs(10)).await;
        })
        .await;
    }

    async fn mock_device_nack_responder(socket_path: String) {
        base_mock_device(socket_path, |mut stream: UnixStream| async move {
            let mut buffer = [0u8; 256];

            while let Ok(n) = stream.read(&mut buffer).await {
                if n == 0 {
                    break;
                }

                let request = &buffer[..n];
                if n >= 5 {
                    let dest = request[0];
                    let src = request[2];

                    let mut response = vec![src, 0x00, dest, Header::NACK as u8];

                    let checksum: u16 = response.iter().map(|&b| b as u16).sum();
                    response.push((256 - (checksum % 256)) as u8);

                    let _ = stream.write_all(&response).await;
                }
            }
        })
        .await;
    }

    #[tokio::test]
    async fn test_successful_simple_poll() {
        let (_temp_dir, socket_path) = create_test_socket_path();
        let (tx, rx) = mpsc::channel(10);

        let device_socket_path = socket_path.clone();
        tokio::spawn(async move {
            mock_device_ack_responder(device_socket_path).await;
        });

        let transport_socket_path = socket_path.clone();
        let transport_handle = tokio::spawn(async move {
            let transport = create_test_transport(rx, transport_socket_path);
            transport.run().await
        });

        tokio::time::sleep(Duration::from_millis(10)).await;

        let (response_tx, response_rx) = oneshot::channel();
        let message = TransportMessage {
            address: 2,
            checksum_type: ChecksumType::Crc8,
            header: Header::SimplePoll,
            data: vec![],
            respond_to: response_tx,
        };

        tx.send(message).await.unwrap();

        let response = tokio::time::timeout(Duration::from_millis(200), response_rx)
            .await
            .expect("Response timeout")
            .expect("Response channel error")
            .expect("Transport error");

        assert_eq!(response.len(), 5); // dest + len + src + header + checksum
        assert_eq!(response[0], 1); // dest = host address
        assert_eq!(response[1], 0); // len = 0
        assert_eq!(response[2], 2); // src = device address
        assert_eq!(response[3], 0); // header = Reply

        transport_handle.abort();
    }

    #[tokio::test]
    async fn test_command_with_data() {
        let (_temp_dir, socket_path) = create_test_socket_path();
        let (tx, rx) = mpsc::channel(10);

        // Mock device that echoes data back
        let device_socket_path = socket_path.clone();
        tokio::spawn(async move {
            if Path::new(&device_socket_path).exists() {
                std::fs::remove_file(&device_socket_path).ok();
            }

            let listener = UnixListener::bind(&device_socket_path).unwrap();

            while let Ok((mut stream, _)) = listener.accept().await {
                tokio::spawn(async move {
                    let mut buffer = [0u8; 256];

                    while let Ok(n) = stream.read(&mut buffer).await {
                        if n == 0 {
                            break;
                        }

                        let request = &buffer[..n];
                        if n >= 5 {
                            let dest = request[0];
                            let data_len = request[1];
                            let src = request[2];

                            // Echo back the data in response
                            let mut response = vec![src, data_len, dest, 0x00]; // header = Reply

                            // Add the original data
                            if data_len > 0 {
                                response.extend_from_slice(&request[4..4 + data_len as usize]);
                            }

                            // Calculate checksum
                            let checksum: u16 = response.iter().map(|&b| b as u16).sum();
                            response.push((256 - (checksum % 256)) as u8);

                            let _ = stream.write_all(&response).await;
                        }
                    }
                });
            }
        });

        let transport_socket_path = socket_path.clone();
        let transport_handle = tokio::spawn(async move {
            let transport = create_test_transport(rx, transport_socket_path);
            transport.run().await
        });

        tokio::time::sleep(Duration::from_millis(10)).await;

        let (response_tx, response_rx) = oneshot::channel();
        let test_data = vec![0x12, 0x34, 0x56];
        let message = TransportMessage {
            address: 3,
            checksum_type: ChecksumType::Crc8,
            header: Header::ModifyInhibitStatus,
            data: test_data.clone(),
            respond_to: response_tx,
        };

        tx.send(message).await.unwrap();

        let response = tokio::time::timeout(Duration::from_millis(200), response_rx)
            .await
            .expect("Response timeout")
            .expect("Response channel error")
            .expect("Transport error");

        assert_eq!(response.len(), 5 + test_data.len()); // header + data + checksum
        assert_eq!(response[0], 1); // dest = host
        assert_eq!(response[1], test_data.len() as u8); // data length
        assert_eq!(response[2], 3); // src = device
        assert_eq!(response[3], 0); // header = Reply
        assert_eq!(&response[4..4 + test_data.len()], &test_data[..]); // echoed data

        transport_handle.abort();
    }

    #[tokio::test]
    async fn test_timeout_error() {
        let (_temp_dir, socket_path) = create_test_socket_path();
        let (tx, rx) = mpsc::channel(10);

        let device_socket_path = socket_path.clone();
        tokio::spawn(async move {
            mock_device_no_response(device_socket_path).await;
        });

        let transport_socket_path = socket_path.clone();
        let transport_handle = tokio::spawn(async move {
            let transport = create_test_transport(rx, transport_socket_path);
            transport.run().await
        });

        tokio::time::sleep(Duration::from_millis(10)).await;

        let (response_tx, response_rx) = oneshot::channel();
        let message = TransportMessage {
            address: 2,
            checksum_type: ChecksumType::Crc8,
            header: Header::SimplePoll,
            data: vec![],
            respond_to: response_tx,
        };

        tx.send(message).await.unwrap();

        let result = tokio::time::timeout(Duration::from_millis(300), response_rx)
            .await
            .expect("Response timeout")
            .expect("Response channel error");

        assert_eq!(result, Err(TransportError::Timeout));

        transport_handle.abort();
    }

    #[tokio::test]
    async fn test_nack_response() {
        let (_temp_dir, socket_path) = create_test_socket_path();
        let (tx, rx) = mpsc::channel(10);

        let device_socket_path = socket_path.clone();
        tokio::spawn(async move {
            mock_device_nack_responder(device_socket_path).await;
        });

        let transport_socket_path = socket_path.clone();
        let transport_handle = tokio::spawn(async move {
            let transport = create_test_transport(rx, transport_socket_path);
            transport.run().await
        });

        tokio::time::sleep(Duration::from_millis(10)).await;

        let (response_tx, response_rx) = oneshot::channel();
        let message = TransportMessage {
            address: 2,
            checksum_type: ChecksumType::Crc8,
            header: Header::SimplePoll,
            data: vec![],
            respond_to: response_tx,
        };

        tx.send(message).await.unwrap();

        let response = tokio::time::timeout(Duration::from_millis(200), response_rx)
            .await
            .expect("Response timeout")
            .expect("Response channel error")
            .expect("Transport error");

        assert_eq!(response[3], Header::NACK as u8);

        transport_handle.abort();
    }

    #[tokio::test]
    async fn test_connection_failure() {
        let (_temp_dir, socket_path) = create_test_socket_path();
        let (_, rx) = mpsc::channel(10);

        let transport_socket_path = socket_path.clone();
        let transport_result = tokio::spawn(async move {
            let transport = create_test_transport(rx, transport_socket_path);
            transport.run().await
        });

        let result = tokio::time::timeout(Duration::from_millis(100), transport_result)
            .await
            .expect("Transport should fail quickly")
            .expect("Join error");

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_multiple_commands() {
        let (_temp_dir, socket_path) = create_test_socket_path();
        let (tx, rx) = mpsc::channel(10);

        let device_socket_path = socket_path.clone();
        tokio::spawn(async move {
            mock_device_ack_responder(device_socket_path).await;
        });

        let transport_socket_path = socket_path.clone();
        let transport_handle = tokio::spawn(async move {
            let transport = create_test_transport(rx, transport_socket_path);
            transport.run().await
        });

        tokio::time::sleep(Duration::from_millis(10)).await;

        let mut response_receivers = vec![];

        for i in 2..5 {
            let (response_tx, response_rx) = oneshot::channel();
            let message = TransportMessage {
                address: i,
                checksum_type: ChecksumType::Crc8,
                header: Header::SimplePoll,
                data: vec![],
                respond_to: response_tx,
            };

            tx.send(message).await.unwrap();
            response_receivers.push(response_rx);
        }

        for (i, response_rx) in response_receivers.into_iter().enumerate() {
            let response = tokio::time::timeout(Duration::from_millis(200), response_rx)
                .await
                .expect("Response timeout")
                .expect("Response channel error")
                .expect("Transport error");

            assert_eq!(response[2], (i + 2) as u8); // src address
        }

        transport_handle.abort();
    }

    #[tokio::test]
    async fn test_packet_building() {
        let (response_tx, _response_rx) = oneshot::channel();
        let message = TransportMessage {
            address: 5,
            checksum_type: ChecksumType::Crc8,
            header: Header::RequestStatus,
            data: vec![0x01, 0x02],
            respond_to: response_tx,
        };

        let mut buffer = vec![0u8; MAX_BLOCK_LENGTH];
        let mut packet = Packet::new(buffer.as_mut_slice());

        let message = Message::from(&message);
        let result = build_packet(&message, &mut packet);
        assert!(result.is_ok());

        assert_eq!(packet.get_destination().unwrap(), 5);
        assert_eq!(packet.get_source().unwrap(), 1);
        assert_eq!(packet.get_header().unwrap(), Header::RequestStatus);
        assert_eq!(packet.get_data().unwrap(), &[0x01, 0x02]);
    }

    #[tokio::test]
    async fn test_error_handling() {
        let (response_tx, response_rx) = oneshot::channel();
        let message = TransportMessage {
            address: 2,
            checksum_type: ChecksumType::Crc8,
            header: Header::SimplePoll,
            data: vec![],
            respond_to: response_tx,
        };

        handle_error(message, TransportError::Timeout, "test error");

        let result = response_rx.await.expect("Response channel error");
        assert!(matches!(result, Err(TransportError::Timeout)));
    }
}
