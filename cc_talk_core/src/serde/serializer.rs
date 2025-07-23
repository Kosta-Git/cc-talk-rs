use crate::{
    cc_talk::{Packet, SOURCE_OFFSET},
    Device,
};

pub fn serialize<B>(device: &Device, packet: &mut Packet<B>) -> Result<(), SerializationError>
where
    B: AsMut<[u8]> + AsRef<[u8]>,
{
    assert!(
        !device.encrypted(),
        "encrypted devices are currently not supported."
    );

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

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum SerializationError {
    BufferTooSmall,
}

impl core::fmt::Display for SerializationError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            SerializationError::BufferTooSmall => write!(f, "Buffer too small for serialization"),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn simple_checksum_verify_test() {
        let buffer: [u8; 5] = [1, 0, 2, 0, 0];
        let mut packet = Packet::new(buffer);
        let device = Device::new(1, crate::Category::Unknown, crate::ChecksumType::Crc8);
        let result = serialize(&device, &mut packet);

        assert!(result.is_ok());
        assert!(packet.get_checksum().is_ok());
        assert_eq!(packet.get_checksum().unwrap(), 253);
    }
}
