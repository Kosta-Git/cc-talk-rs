use crate::cc_talk::{ChecksumType, Device, Packet, SOURCE_OFFSET};

/// Serializes a ccTalk packet by calculating and inserting the appropriate checksum.
///
/// # Errors
///
/// Other errors will return a `SerializationError` if:
/// - Buffer is too small to write the checksum.
///
/// # Panics
///
/// Panics if the device is encrypted, as encrypted devices are not supported.
pub fn serialize<B>(device: &Device, packet: &mut Packet<B>) -> Result<(), SerializationError>
where
    B: AsMut<[u8]> + AsRef<[u8]>,
{
    assert!(
        !device.encrypted(),
        "encrypted devices are currently not supported."
    );

    match device.checksum_type() {
        ChecksumType::Crc8 => {
            let checksum = crate::common::checksum::crc8(packet.as_slice());
            let checksum_index = packet
                .get_checksum_offset()
                .map_err(|_| SerializationError::BufferTooSmall)?;

            packet
                .write_byte(checksum_index as usize, checksum)
                .map_err(|_| SerializationError::BufferTooSmall)?;

            Ok(())
        }
        ChecksumType::Crc16 => {
            let checksum = crate::common::checksum::crc16(packet.as_slice());
            let checksum_index = packet
                .get_checksum_offset()
                .map_err(|_| SerializationError::BufferTooSmall)?;

            let least_significant_bits = (checksum & 0xFF) as u8;
            let most_significant_bits = ((checksum >> 8) & 0xFF) as u8;

            packet
                .write_byte(SOURCE_OFFSET, least_significant_bits)
                .map_err(|_| SerializationError::BufferTooSmall)?;

            packet
                .write_byte(checksum_index as usize, most_significant_bits)
                .map_err(|_| SerializationError::BufferTooSmall)?;

            Ok(())
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum SerializationError {
    #[error("buffer too small for serialization")]
    BufferTooSmall,
}

#[cfg(test)]
mod test {
    use crate::cc_talk::{Category, Device};

    use super::*;

    #[test]
    fn simple_checksum_verify_test() {
        let buffer: [u8; 5] = [1, 0, 2, 0, 0];
        let mut packet = Packet::new(buffer);
        let device = Device::new(1, Category::Unknown, ChecksumType::Crc8);
        let result = serialize(&device, &mut packet);

        assert!(result.is_ok());
        assert!(packet.get_checksum().is_ok());
        assert_eq!(packet.get_checksum().expect("is_ok"), 253);
    }
}
