use crate::{cc_talk::Packet, ChecksumType};

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
                return Err(DeserializationError::ChecksumMismatch(
                    expected_checksum as u16,
                    checksum as u16,
                ));
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
                return Err(DeserializationError::ChecksumMismatch(
                    expected_checksum,
                    checksum,
                ));
            }

            Ok(1u8) // Default return address for CRC16
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeserializationError {
    BufferTooSmall,
    InvalidPacket,
    UnsupportedChecksumType,
    /// Cehcksum mismatch between the packet and the expected checksum.
    /// .0 is the expected checksum, .1 is the actual checksum.
    ChecksumMismatch(u16, u16),
}

#[cfg(test)]
mod test {
    use std::{println, string::String};

    use super::*;

    #[test]
    fn simple_checksum_verify_test() {
        let buffer: [u8; 5] = [1, 0, 2, 0, 253];
        let mut packet = Packet::new(buffer);
        let result = deserialize(&mut packet, ChecksumType::Crc8);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 2);
    }
}
