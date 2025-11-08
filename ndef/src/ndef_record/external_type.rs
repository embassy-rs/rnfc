use heapless::Vec;
use thiserror::Error;

use crate::ndef_record::{NdefRecord, NdefRecordHeader, TypeNameFormat};

#[derive(Error, Debug)]
pub enum Error {
    #[error("Provided domain or type is too empty")]
    InvalidFormat,
    #[error("Provided type is too long, maximum length is 255 bytes")]
    TypeTooLong,
    #[error("Provided payload is too long, maximum length is {0} bytes")]
    PayloadTooLong(usize),
    #[error("Vector operations failed")]
    VectorFull,
}

impl<const MAX_PAYLOAD_SIZE: usize> NdefRecord<MAX_PAYLOAD_SIZE> {
    /// Creates a new external type NDEF record
    ///
    /// # Arguments
    ///
    /// * `domain` - Domain name (e.g., "example.com")
    /// * `type_` - Type name within the domain (e.g., "mytype")
    /// * `data` - Payload data
    ///
    /// # Note
    /// By default, the record is considered to be the first and last record in the message.
    /// If you have multiple records in a message, you should modify the header accordingly.
    ///
    /// # Returns
    ///
    /// Returns a Result containing the NdefRecord if successful, or an Error if:
    /// - Domain or type is empty
    /// - Combined length of domain:type exceeds 255 bytes
    /// - Payload exceeds MAX_PAYLOAD_SIZE
    /// - Vector operations fail
    pub fn new_external(domain: &[u8], type_: &[u8], data: &[u8]) -> Result<Self, Error> {
        // Domain and type must be non-empty
        if domain.is_empty() || type_.is_empty() {
            return Err(Error::InvalidFormat);
        }

        // Calculate total type length (domain:type)
        let total_type_length = domain.len() + 1 + type_.len(); // +1 for ':'
        if total_type_length > 255 {
            return Err(Error::TypeTooLong);
        }

        // Check payload length
        if data.len() > MAX_PAYLOAD_SIZE {
            return Err(Error::PayloadTooLong(MAX_PAYLOAD_SIZE));
        }

        // Create record type vector (domain:type)
        let mut record_type: Vec<u8, 255> = Vec::from_slice(domain).map_err(|_| Error::VectorFull)?;
        record_type.push(b':').map_err(|_| Error::VectorFull)?;
        record_type.extend_from_slice(type_).map_err(|_| Error::VectorFull)?;

        // Create payload vector
        let payload: Vec<u8, MAX_PAYLOAD_SIZE> = Vec::from_slice(data).map_err(|_| Error::VectorFull)?;

        Ok(Self {
            header: NdefRecordHeader::new(true, true, false, true, false, TypeNameFormat::External),
            type_length: total_type_length as u8,
            payload_length: data.len() as u32,
            id_length: None,
            record_type,
            id: None,
            payload,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_external_type() {
        let record = NdefRecord::<1024>::new_external(b"example.com", b"mytype", b"Hello, world!").unwrap();

        assert_eq!(record.header.type_name_format, TypeNameFormat::External);
        assert!(!record.header.id_present);
        assert!(record.header.short);
        assert!(!record.header.chunk);
        assert!(record.header.message_begin);
        assert!(record.header.message_end);

        // Check type length
        assert_eq!(record.type_length, 18);

        // Check payload length
        assert_eq!(record.payload_length, 13);

        // Check ID length
        assert_eq!(record.id_length, None);

        // Check record type
        assert_eq!(
            record.record_type,
            [0x65, 0x78, 0x61, 0x6D, 0x70, 0x6C, 0x65, 0x2E, 0x63, 0x6F, 0x6D, 0x3A, 0x6D, 0x79, 0x74, 0x79, 0x70, 0x65]
        );

        // Check ID
        assert_eq!(record.id, None);

        // Check payload
        assert_eq!(
            record.payload,
            [0x48, 0x65, 0x6c, 0x6c, 0x6f, 0x2c, 0x20, 0x77, 0x6f, 0x72, 0x6c, 0x64, 0x21]
        );
    }
}
