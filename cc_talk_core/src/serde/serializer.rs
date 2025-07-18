use crate::{
    Device,
    cc_talk::{Packet, SOURCE_OFFSET},
};

pub fn serialize<B>(device: &Device, packet: &mut Packet<B>) -> Result<(), SerializationError>
where
    B: AsMut<[u8]> + AsRef<[u8]>,
{
    assert!(!device.encrypted());
    match device.checksum_type() {
        crate::ChecksumType::Crc8 => {
            let checksum = crate::common::checksum::crc8(packet.as_slice());
            let checksum_index = packet
                .get_checksum_offset()
                .map_err(|_| SerializationError::BufferTooSmall)?;

            packet
                .write_byte(checksum_index as usize, checksum)
                .map_err(|_| SerializationError::BufferTooSmall)?;

            Ok(())
        }
        crate::ChecksumType::Crc16 => {
            let checksum = crate::common::checksum::crc16(packet.as_slice());
            let checksum_index = packet
                .get_checksum_offset()
                .map_err(|_| SerializationError::BufferTooSmall)?;

            let checksum_lsb = (checksum & 0xFF) as u8;
            let checksum_msb = ((checksum >> 8) & 0xFF) as u8;

            packet
                .write_byte(SOURCE_OFFSET, checksum_lsb)
                .map_err(|_| SerializationError::BufferTooSmall)?;

            packet
                .write_byte(checksum_index as usize, checksum_msb)
                .map_err(|_| SerializationError::BufferTooSmall)?;

            Ok(())
        }
    }
}

pub enum SerializationError {
    BufferTooSmall,
}
