use crate::{
    cc_talk::{ChecksumType, Packet},
    common::checksum,
};

/// Deserializes a ccTalk packet and verifies its checksum.
/// Returns the reply to address if successful, or an error if the checksum is invalid or the
/// packet is malformed.
///
/// # Errors
///
/// Returns a `DeserializationError` if:
/// - The buffer is too small to contain a valid packet.
/// - The checksum does not match the expected value.
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
            let expected_checksum = checksum::crc8(packet.as_slice());

            if checksum != expected_checksum {
                return Err(DeserializationError::ChecksumMismatch(
                    u16::from(expected_checksum),
                    u16::from(checksum),
                ));
            }

            Ok(packet
                .get_source()
                .map_err(|_| DeserializationError::InvalidPacket)?)
        }
        ChecksumType::Crc16 => {
            let most_significant_bits = packet
                .get_checksum()
                .map(u16::from)
                .map_err(|_| DeserializationError::BufferTooSmall)?;
            let least_significant_bits = packet
                .get_source()
                .map(u16::from)
                .map_err(|_| DeserializationError::BufferTooSmall)?;

            let checksum = most_significant_bits << 8 | least_significant_bits;
            let expected_checksum = checksum::crc16(packet.as_slice());
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

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum DeserializationError {
    #[error("buffer too small")]
    BufferTooSmall,
    #[error("invalid packet")]
    InvalidPacket,
    #[error("unsupported checksum type")]
    UnsupportedChecksumType,
    /// Checksum mismatch between the packet and the expected checksum.
    /// .0 is the expected checksum, .1 is the actual checksum.
    #[error("checksum mismatch: expected {0}, got {1}")]
    ChecksumMismatch(u16, u16),
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn simple_checksum_verify_test() {
        let buffer: [u8; 5] = [1, 0, 2, 0, 253];
        let mut packet = Packet::new(buffer);
        let result = deserialize(&mut packet, ChecksumType::Crc8);

        assert!(result.is_ok());
        assert_eq!(result.expect("is_ok"), 2);
    }
}
