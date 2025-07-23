use cc_talk_core::{ChecksumType, Header, cc_talk::MAX_BLOCK_LENGTH};

use crate::device_impl::{DeviceImpl, SimplePayoutDevice};

pub struct PayoutDevice<T>
where
    T: DeviceImpl + SimplePayoutDevice,
{
    implementation: T,
    internal_buffer: heapless::Vec<u8, MAX_BLOCK_LENGTH>,
}

impl<T> PayoutDevice<T>
where
    T: DeviceImpl + SimplePayoutDevice,
{
    pub fn new(implementation: T) -> Self {
        Self {
            implementation,
            internal_buffer: heapless::Vec::new(),
        }
    }

    pub async fn process_packet(
        &mut self,
        source: u8,
        destination: u8,
        header: Header,
        checksum_type: ChecksumType,
        payload: &[u8],
    ) -> Option<(u8, Header, &[u8], usize)> {
        if !self.implementation.is_for_me(destination) {
            return None;
        }

        let return_address = if checksum_type == ChecksumType::Crc16 {
            1
        } else {
            source
        };

        match header {
            Header::SimplePoll => Some((return_address, Header::Reply, &[], 0)),
            Header::RequestManufacturerId => {
                let name = self.implementation.manufacturer().abbreviated_name();
                create_return(return_address, Header::Reply, name.as_bytes())
            }
            Header::RequestEquipementCategoryId => {
                create_return(return_address, Header::Reply, "Payout".as_bytes())
            }
            Header::RequestProductCode => {
                let product_code = self.implementation.product_code();
                create_return(return_address, Header::Reply, product_code.as_bytes())
            }
            Header::RequestSerialNumber => {
                let serial_number = self.implementation.serial_number();
                self.internal_buffer[0] = serial_number.fix();
                self.internal_buffer[1] = serial_number.minor();
                self.internal_buffer[2] = serial_number.major();

                create_return(return_address, Header::Reply, &self.internal_buffer[..3])
            }
            Header::RequestSoftwareRevision => {
                let software_revision = self.implementation.software_revision();
                create_return(return_address, Header::Reply, software_revision.as_bytes())
            }
            Header::RequestPayoutStatus => {
                let status = self.implementation.request_payout_status().await;
                let status: [u8; 4] = status.into();
                self.internal_buffer[..4].copy_from_slice(&status);
                create_return(return_address, Header::Reply, &self.internal_buffer[..4])
            }
            Header::RequestDataStorageAvailability => {
                let data_storage = self.implementation.data_storage_availability();
                let data_storage_bytes: [u8; 5] = data_storage.into();
                self.internal_buffer[..5].copy_from_slice(&data_storage_bytes);
                create_return(return_address, Header::Reply, &self.internal_buffer[..5])
            }
            Header::RequestBuildCode => {
                let build_code = self.implementation.build_code();
                create_return(return_address, Header::Reply, build_code.as_bytes())
            }
            Header::EmergencyStop => {
                self.implementation.emergency_stop().await;
                create_return(return_address, Header::Reply, &[])
            }
            Header::RequestHopperCoin => {
                let coin = self.implementation.request_hopper_coin();
                create_return(return_address, Header::Reply, coin.as_bytes())
            }
            Header::RequestHopperDispenseCount => {
                let dispense_count = self.implementation.request_hopper_dispense_count().await;
                self.internal_buffer[..3].copy_from_slice(&dispense_count.to_le_bytes()[..3]);
                create_return(return_address, Header::Reply, &self.internal_buffer[..3])
            }
            Header::DispenseHopperCoins => {
                if payload.is_empty() {
                    return Some((return_address, Header::NACK, &[], 0));
                }

                let count = *payload.last().unwrap_or(&0u8);
                if count == 0 {
                    return Some((return_address, Header::NACK, &[], 0));
                }

                let status = self.implementation.request_payout_status().await;
                self.implementation.dispense_hopper_coins(count).await;

                self.internal_buffer[0] = status.event_counter;
                create_return(return_address, Header::Reply, &self.internal_buffer[..1])
            }
            Header::RequestHopperStatus => {
                let status = self.implementation.request_payout_status().await;
                let status_bytes: [u8; 4] = status.into();
                self.internal_buffer[..4].copy_from_slice(&status_bytes);
                create_return(return_address, Header::Reply, &self.internal_buffer[..4])
            }
            Header::EnableHopper => {
                if payload.is_empty() {
                    return Some((return_address, Header::NACK, &[], 0));
                }
                let enable = payload[0] == 0xA5;
                self.implementation.enable_payout(enable).await;
                create_return(return_address, Header::Reply, &[])
            }
            Header::TestHopper => {
                let (register_1, register_2, register_3) = self.implementation.test().await;
                self.internal_buffer[0] = register_1;
                self.internal_buffer[1] = register_2;
                self.internal_buffer[2] = register_3;
                create_return(return_address, Header::Reply, &self.internal_buffer[..3])
            }
            Header::RequestCommsRevision => {
                let (major, minor, patch) = self.implementation.comms_revision();
                self.internal_buffer[0] = major;
                self.internal_buffer[1] = minor;
                self.internal_buffer[2] = patch;
                create_return(return_address, Header::Reply, &self.internal_buffer[..3])
            }
            Header::ResetDevice => {
                self.implementation.reset().await;
                Some((return_address, Header::Reply, &[], 0))
            }
            _ => Some((return_address, Header::NACK, &[], 0)),
        }
    }
}

#[inline(always)]
fn create_return(
    address: u8,
    header: Header,
    payload: &[u8],
) -> Option<(u8, Header, &[u8], usize)> {
    Some((address, header, payload, payload.len()))
}
