use cc_talk_core::{
    cc_talk::{
        deserializer::deserialize, serializer::serialize, Packet, PacketError, MAX_BLOCK_LENGTH,
    },
    Header,
};

use crate::{
    device_impl::{DeviceImpl, SimplePayoutDevice},
    log::{error, info},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum FrameError {
    MemoryError,
    FrameNotValid,
    SerializationError,
}

impl From<PacketError> for FrameError {
    fn from(error: PacketError) -> Self {
        match error {
            PacketError::OutOfBounds => FrameError::MemoryError,
            PacketError::DataLengthMismatch => FrameError::FrameNotValid,
            PacketError::InvalidHeader(_) => FrameError::FrameNotValid,
            PacketError::InvalidPacket => FrameError::FrameNotValid,
        }
    }
}

pub struct PayoutDevice<T>
where
    T: DeviceImpl + SimplePayoutDevice,
{
    implementation: T,
}

impl<T> PayoutDevice<T>
where
    T: DeviceImpl + SimplePayoutDevice,
{
    pub fn new(implementation: T) -> Self {
        Self { implementation }
    }

    /// Process a ccTalk frame.
    ///
    /// `frame` has to be a valid ccTalk frame, which means it has to be at least 5 bytes long.
    ///
    /// `reply_buffer` is a buffer that will be used to store the reply packet. It should be
    /// MAX_BLOCK_LENGTH bytes long.
    ///
    /// The result will be the size of the reply packet, or an error if something went wrong.
    pub async fn on_frame(
        &self,
        frame: &mut [u8],
        reply_buffer: &mut [u8],
    ) -> Result<usize, FrameError> {
        match self.validate(frame) {
            Some((packet, reply_address)) => {
                let header = packet.get_header()?;
                let payload = packet.get_data()?;
                let mut reply_packet = Packet::new(reply_buffer);

                reply_packet.set_source(self.implementation.address())?;
                reply_packet.set_destination(reply_address)?;
                self.process_packet(header, payload, &mut reply_packet)
                    .await?;

                match serialize(&self.implementation.device(), &mut reply_packet) {
                    Ok(()) => Ok(reply_packet.get_logical_size()),
                    Err(error) => {
                        error!("failed to serialize reply packet: {:?}", error);
                        Err(FrameError::SerializationError)
                    }
                }
            }
            None => Err(FrameError::FrameNotValid),
        }
    }

    fn validate<'a>(&self, buffer: &'a mut [u8]) -> Option<(Packet<&'a mut [u8]>, u8)> {
        let mut p = Packet::new(&mut buffer[..]);

        let destination = p.get_destination().unwrap_or(0u8);
        if !self.implementation.is_for_me(destination) {
            return None;
        }

        match deserialize(&mut p, self.implementation.checksum_type()) {
            Ok(reply_addr) => Some((p, reply_addr)),
            Err(error) => {
                // If we have a checksom error, or something similar, its better to not reply.
                error!("failed to deserialize packet: {:?}", error);
                None
            }
        }
    }

    async fn process_packet(
        &self,
        header: Header,
        payload: &[u8],
        packet: &mut Packet<&mut [u8]>,
    ) -> Result<(), PacketError> {
        packet.set_header(Header::Reply)?;

        match header {
            Header::SimplePoll => packet.set_data(&[]),
            Header::RequestManufacturerId => packet.set_data(
                self.implementation
                    .manufacturer()
                    .abbreviated_name()
                    .as_bytes(),
            ),
            Header::RequestEquipementCategoryId => packet.set_data("Payout".as_bytes()),
            Header::RequestProductCode => {
                packet.set_data(self.implementation.product_code().as_bytes())
            }
            Header::RequestSerialNumber => {
                let serial_number = self.implementation.serial_number();
                packet.set_data(
                    [
                        serial_number.fix(),
                        serial_number.minor(),
                        serial_number.major(),
                    ]
                    .as_ref(),
                )
            }
            Header::RequestSoftwareRevision => {
                packet.set_data(self.implementation.software_revision().as_bytes())
            }
            Header::RequestPayoutStatus => {
                let status = self.implementation.request_payout_status().await;
                let status: [u8; 4] = status.into();
                packet.set_data(&status)
            }
            Header::RequestDataStorageAvailability => {
                let data_storage = self.implementation.data_storage_availability();
                let data_storage_bytes: [u8; 5] = data_storage.into();
                packet.set_data(&data_storage_bytes)
            }
            Header::RequestBuildCode => {
                packet.set_data(self.implementation.build_code().as_bytes())
            }
            Header::EmergencyStop => {
                self.implementation.emergency_stop().await;
                packet.set_data(&[])
            }
            Header::RequestHopperCoin => {
                packet.set_data(self.implementation.request_hopper_coin().as_bytes())
            }
            Header::RequestHopperDispenseCount => {
                let dispense_count = self.implementation.request_hopper_dispense_count().await;
                packet.set_data(&dispense_count.to_le_bytes()[..3])
            }
            Header::DispenseHopperCoins => {
                if payload.is_empty() {
                    packet.set_header(Header::NACK)?;
                    return packet.set_data(&[]);
                }

                let count = *payload.last().unwrap_or(&0u8);
                if count == 0 {
                    packet.set_header(Header::NACK)?;
                    return packet.set_data(&[]);
                }

                let status = self.implementation.request_payout_status().await;
                self.implementation.dispense_hopper_coins(count).await;

                packet.set_data(&[status.event_counter])
            }
            Header::RequestHopperStatus => {
                let status = self.implementation.request_payout_status().await;
                let status_bytes: [u8; 4] = status.into();
                packet.set_data(&status_bytes)
            }
            Header::EnableHopper => {
                if payload.is_empty() {
                    packet.set_header(Header::NACK)?;
                    return packet.set_data(&[]);
                }
                let enable = payload[0] == 0xA5;
                self.implementation.enable_payout(enable).await;
                packet.set_data(&[])
            }
            Header::TestHopper => {
                let (register_1, register_2, register_3) = self.implementation.test().await;
                packet.set_data(&[register_1, register_2, register_3])
            }
            Header::RequestCommsRevision => {
                let (major, minor, patch) = self.implementation.comms_revision();
                packet.set_data(&[major, minor, patch])
            }
            Header::ResetDevice => {
                self.implementation.reset().await;
                packet.set_data(&[])
            }
            _ => {
                packet.set_header(Header::NACK)?;
                packet.set_data(&[])
            }
        }
    }
}
