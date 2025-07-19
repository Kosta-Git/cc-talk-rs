use crate::{ChecksumType, cc_talk::Packet};

/// Deserializes a ccTalk packet and verifies its checksum.
/// Returns the reply to address if successful, or an error if the checksum is invalid or the
/// packet is malformed.
pub fn deserialize<B>(
    packet: &mut Packet<B>,
    checksum_type: ChecksumType,
) -> Result<u8, DeserializationError>
where
    B: AsRef<[u8]> + AsMut<[u8]>,
{
    match checksum_type {
        ChecksumType::Crc8 => {
            let checksum = packet
                .get_checksum()
                .map_err(|_| DeserializationError::BufferTooSmall)?;
            let expected_checksum = crate::common::checksum::crc8(packet.as_slice());

            if checksum != expected_checksum {
                return Err(DeserializationError::ChecksumMismatch);
            }

            Ok(packet
                .get_source()
                .map_err(|_| DeserializationError::InvalidPacket)?)
        }
        ChecksumType::Crc16 => {
            let checksum_msb = packet
                .get_checksum()
                .map_err(|_| DeserializationError::BufferTooSmall)?;
            let checksum_lsb = packet
                .get_source()
                .map_err(|_| DeserializationError::BufferTooSmall)?;

            let checksum = (checksum_msb as u16) << 8 | (checksum_lsb as u16);
            let expected_checksum = crate::common::checksum::crc16(packet.as_slice());
            if checksum != expected_checksum {
                return Err(DeserializationError::ChecksumMismatch);
            }

            Ok(1u8) // Default return address for CRC16
        }
    }
}

pub enum DeserializationError {
    BufferTooSmall,
    InvalidPacket,
    UnsupportedChecksumType,
    ChecksumMismatch,
}
