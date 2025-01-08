use heapless::Vec;
use packed_struct::prelude::*;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum NdefRecordError {
    #[error("Provided buffer is too small")]
    BufferTooSmall,
    #[error("Invalid header, could not unpack")]
    InvalidHeader,
    #[error("Parsed payload length is bigger than provided generic type parameter")]
    PayloadLengthMismatch,
    #[error("Append elements to Vec failed")]
    VecCapacityError,
}

#[derive(PrimitiveEnum_u8, Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
pub enum TypeNameFormat {
    Empty = 0x00,
    WellKnown = 0x01,
    MimeMediaType = 0x02,
    AbsoluteUri = 0x03,
    External = 0x04,
    Unknown = 0x05,
    Unchanged = 0x06,
    Reserved = 0x07,
}

/// NDEF message record header
#[derive(PackedStruct, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
#[packed_struct(size_bytes = "1", bit_numbering = "lsb0")]
pub struct NdefRecordHeader {
    /// Type Name Format (TNF) field that defines how to interpret the type field
    #[packed_field(bits = "0..3", ty = "enum")]
    pub type_name_format: TypeNameFormat,
    /// Indicates whether the record contains an ID field
    #[packed_field(bits = "3")]
    pub id_present: bool,
    /// The Short Record (SR) bit flag determines the length of the payload record.
    /// If SR is true, the payload length is one byte, otherwise it’s four bytes.
    /// Payload length is required, but may be zero.
    #[packed_field(bits = "4")]
    pub short: bool,
    /// The Chunk Bit Flag indicates whether the payload is a sequence of chunks.
    #[packed_field(bits = "5")]
    pub chunk: bool,
    /// The Message End Bit Flag indicates whether the record is the last one in the message.
    #[packed_field(bits = "6")]
    pub message_end: bool,
    /// The Message Begin Bit Flag indicates whether the record is the first one in the message.
    #[packed_field(bits = "7")]
    pub message_begin: bool,
}

impl NdefRecordHeader {
    /// Getter for the messaged_end field ot the header
    pub fn message_end(&self) -> bool {
        self.message_end
    }
}

/// An NDEF (NFC Data Exchange Format) record.
///
/// Represents a single NDEF record with configurable payload size. Suitable for `no_std` environments.
///
/// # Type Parameters
/// * `MAX_PAYLOAD_SIZE`: The maximum payload size in bytes.
///
/// # Fields
/// - `header`: Contains flags and type name format.
/// - `type_length`: Length of the type field.
/// - `payload_length`: Length of the payload field.
/// - `id_length`: Optional length of the ID field.
/// - `record_type`: Type field identifying the record type.
/// - `id`: Optional ID field for linking records.
/// - `payload`: The actual payload data.
///
/// # Example
/// ```ignore
/// let record = NdefRecord::<256>::from_bytes(&bytes).unwrap();
/// ```
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
pub struct NdefRecord<const MAX_PAYLOAD_SIZE: usize> {
    /// The NDEF record header containing flags and type name format
    pub header: NdefRecordHeader,
    /// Length of the record type field in bytes
    pub type_length: u8,
    /// Length of the payload field in bytes
    pub payload_length: u32,
    /// Length of the ID field in bytes, if present
    pub id_length: Option<u8>,
    /// The record type field identifying the type of the record
    pub record_type: Vec<u8, 255>, // Record type length is one byte (0-255)
    /// The optional ID field used to link NDEF records
    pub id: Option<Vec<u8, 255>>, // ID length is one byte (0-255)
    /// The payload data of the record
    pub payload: Vec<u8, MAX_PAYLOAD_SIZE>, // Adjust capacity as needed
}

impl<const MAX_PAYLOAD_SIZE: usize> NdefRecord<MAX_PAYLOAD_SIZE> {
    /// Calculate the total size needed for the serialized record
    pub fn serialized_size(&self) -> usize {
        let payload_length_size = if self.header.short { 1 } else { 4 };
        let id_length_size = if self.header.id_present { 1 } else { 0 };

        2 + // header + type_length
            payload_length_size +
            id_length_size +
            self.record_type.len() +
            self.id.as_ref().map_or(0, |id| id.len()) +
            self.payload.len()
    }

    /// Parses an NDEF record from a byte slice.
    ///
    /// # Parameters
    /// - `bytes`: A slice of bytes containing the NDEF record data.
    ///
    /// # Errors
    /// Returns a `NdefRecordError` if the input is too small, invalid, or not an NDEF type.
    ///
    /// # Example
    /// ```ignore
    /// let bytes = [0x03, 0x10, /* NDEF record bytes */];
    /// let ndef_record = NdefRecord::<1024>::from_bytes(&bytes)?;
    /// ```
    pub fn from_bytes(bytes: &[u8]) -> Result<(Self, usize), NdefRecordError> {
        // Minimum size check (header + type_length + payload_length)
        if bytes.len() < 3 {
            return Err(NdefRecordError::BufferTooSmall);
        }

        // Parse header first to determine SR flag
        let header = NdefRecordHeader::unpack(&[bytes[0]]).map_err(|_| NdefRecordError::InvalidHeader)?;

        let type_length = bytes[1];

        // Parse payload length based on SR flag
        let (payload_length, payload_length_size) = if header.short {
            (bytes[2] as u32, 1)
        } else {
            if bytes.len() < 6 {
                return Err(NdefRecordError::BufferTooSmall);
            }
            let mut payload_len_bytes = [0u8; 4];
            payload_len_bytes.copy_from_slice(&bytes[2..6]);
            (u32::from_be_bytes(payload_len_bytes), 4)
        };

        // Check if payload length exceeds Vec capacity
        if payload_length as usize > MAX_PAYLOAD_SIZE {
            return Err(NdefRecordError::PayloadLengthMismatch);
        }

        let mut offset = 2 + payload_length_size;

        // Handle ID length if present
        let id_length = if header.id_present {
            if bytes.len() < offset + 1 {
                return Err(NdefRecordError::BufferTooSmall);
            }
            let id_len = bytes[offset];
            offset += 1;
            Some(id_len)
        } else {
            None
        };

        // Parse record type
        let record_type_end = offset + type_length as usize;
        if bytes.len() < record_type_end {
            return Err(NdefRecordError::BufferTooSmall);
        }
        let mut record_type = Vec::new();
        record_type
            .extend_from_slice(&bytes[offset..record_type_end])
            .map_err(|_| NdefRecordError::VecCapacityError)?;
        offset = record_type_end;

        // Parse ID if present
        let id = if let Some(id_len) = id_length {
            let id_end = offset + id_len as usize;
            if bytes.len() < id_end {
                return Err(NdefRecordError::BufferTooSmall);
            }
            let mut id_buf = Vec::new();
            id_buf
                .extend_from_slice(&bytes[offset..id_end])
                .map_err(|_| NdefRecordError::VecCapacityError)?;
            offset = id_end;
            Some(id_buf)
        } else {
            None
        };

        // Parse payload
        let payload_end = offset + payload_length as usize;
        if bytes.len() < payload_end {
            return Err(NdefRecordError::BufferTooSmall);
        }
        let mut payload = Vec::new();
        payload
            .extend_from_slice(&bytes[offset..payload_end])
            .map_err(|_| NdefRecordError::VecCapacityError)?;

        let bytes_processed = payload_end; // The total number of bytes processed

        Ok((
            Self {
                header,
                type_length,
                payload_length,
                id_length,
                record_type,
                id,
                payload,
            },
            bytes_processed,
        ))
    }

    /// Serializes the NDEF record to bytes, writing to a provided buffer.
    ///
    /// # Parameters
    /// - `buffer`: A mutable byte slice to write the serialized data.
    ///
    /// # Returns
    /// The number of bytes written on success.
    ///
    /// # Errors
    /// Returns `NdefRecordError::BufferTooSmall` if the buffer is too small.
    /// Returns `NdefRecordError::InvalidHeader` if the header is invalid.
    ///
    /// # Example
    /// ```ignore
    /// let mut buffer = [0u8; 128];
    /// let bytes_written = ndef_record.to_bytes(&mut buffer)?;
    /// ```
    pub fn to_bytes(&self, buffer: &mut [u8]) -> Result<usize, NdefRecordError> {
        let required_size = self.serialized_size();

        if buffer.len() < required_size {
            return Err(NdefRecordError::BufferTooSmall);
        }

        let mut offset = 0;

        // Write header
        buffer[offset] = self.header.pack().map_err(|_| NdefRecordError::InvalidHeader)?[0];
        offset += 1;

        // Write type length
        buffer[offset] = self.type_length;
        offset += 1;

        // Write payload length
        if self.header.short {
            buffer[offset] = self.payload_length as u8;
            offset += 1;
        } else {
            buffer[offset..offset + 4].copy_from_slice(&self.payload_length.to_be_bytes());
            offset += 4;
        }

        // Write ID length if present
        if let Some(id_length) = self.id_length {
            buffer[offset] = id_length;
            offset += 1;
        }

        // Write record type
        buffer[offset..offset + self.record_type.len()].copy_from_slice(&self.record_type);
        offset += self.record_type.len();

        // Write ID if present
        if let Some(id) = &self.id {
            buffer[offset..offset + id.len()].copy_from_slice(id);
            offset += id.len();
        }

        // Write payload
        buffer[offset..offset + self.payload.len()].copy_from_slice(&self.payload);
        offset += self.payload.len();

        // Pad remaining buffer with zeros
        if buffer.len() > offset {
            buffer[offset..].fill(0);
        }

        Ok(offset)
    }
}

#[cfg(test)]
mod tests {
    use heapless::Vec;
    use packed_struct::prelude::*;

    use super::{NdefRecord, NdefRecordHeader, TypeNameFormat};
    use crate::tlv::{NdefTlv, Tag, TL};

    #[test]
    fn test_parse_ndef_record_header() {
        let bytes = [0xD4];
        let ndef_record_header = NdefRecordHeader::unpack(&bytes).unwrap();
        assert_eq!(ndef_record_header.type_name_format, TypeNameFormat::External);
        assert!(!ndef_record_header.id_present);
        assert!(ndef_record_header.short);
        assert!(!ndef_record_header.chunk);
        assert!(ndef_record_header.message_begin);
        assert!(ndef_record_header.message_end);
    }

    #[test]
    fn test_record_deserialization() {
        const TYPE_LEN: usize = 28;
        const PAYLOAD_LEN: usize = 8;

        let buffer = [
            0xd4, 0x1c, 0x8, 0x74, 0x68, 0x65, 0x72, 0x6d, 0x69, 0x67, 0x6f, 0x2e, 0x63, 0x6f, 0x6d, 0x3a, 0x72, 0x75, 0x73,
            0x74, 0x70, 0x6f, 0x73, 0x74, 0x63, 0x61, 0x72, 0x64, 0x2d, 0x76, 0x31, 0xd, 0xe, 0xa, 0xd, 0xb, 0xe, 0xe, 0xf,
            0xfe, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
        ];

        let (parsed, _) = NdefRecord::<1024>::from_bytes(&buffer).unwrap();

        // Check header
        assert_eq!(parsed.header.type_name_format, TypeNameFormat::External);
        assert!(!parsed.header.id_present);
        assert!(parsed.header.short);
        assert!(!parsed.header.chunk);
        assert!(parsed.header.message_begin);
        assert!(parsed.header.message_end);

        // Check type length
        assert_eq!(parsed.type_length, TYPE_LEN as u8);

        // Check payload length
        assert_eq!(parsed.payload_length, PAYLOAD_LEN as u32);

        // Check ID length
        assert_eq!(parsed.id_length, None);

        // Check record type
        assert_eq!(
            parsed.record_type,
            [
                0x74, 0x68, 0x65, 0x72, 0x6d, 0x69, 0x67, 0x6f, 0x2e, 0x63, 0x6f, 0x6d, 0x3a, 0x72, 0x75, 0x73, 0x74, 0x70,
                0x6f, 0x73, 0x74, 0x63, 0x61, 0x72, 0x64, 0x2d, 0x76, 0x31
            ]
        );

        // Check ID
        assert_eq!(parsed.id, None);

        // Check payload
        assert_eq!(parsed.payload, [0xd, 0xe, 0xa, 0xd, 0xb, 0xe, 0xe, 0xf]);
    }

    #[test]
    fn test_parse_ndef_message_with_two_text_records() {
        const TYPE_LEN: usize = 1;
        const PAYLOAD_LEN: usize = 8;

        // NDEF message with two text records:
        // - First 4 bytes: Capability Container (CC) descriptor
        // - Followed by TLV blocks containing 2 records:
        //   1. Text record with payload "Hello"
        //   2. Text record with payload "World"
        let buffer = [
            0xe1, 0x40, 0x40, 0x1, 0x3, 0x18, 0x91, 0x1, 0x8, 0x54, 0x2, 0x65, 0x6e, 0x48, 0x65, 0x6c, 0x6c, 0x6f, 0x51, 0x1,
            0x8, 0x54, 0x2, 0x65, 0x6e, 0x57, 0x6f, 0x72, 0x6c, 0x64, 0xfe, 0x0, 0x72, 0x64, 0x2d, 0x76, 0x31, 0x0, 0x0, 0x0,
            0xfe, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
            0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
            0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
            0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
        ];

        let tlv = NdefTlv::<1024, 2>::from_bytes(&buffer[4..]).unwrap();
        let mut records = tlv.value.unwrap();
        assert_eq!(records.len(), 2);

        // Test second record deserialization
        let second_record = records.pop().unwrap();
        // Check header
        assert_eq!(second_record.header.type_name_format, TypeNameFormat::WellKnown);
        assert!(!second_record.header.id_present);
        assert!(second_record.header.short);
        assert!(!second_record.header.chunk);
        assert!(!second_record.header.message_begin);
        assert!(second_record.header.message_end);
        // Check type length
        assert_eq!(second_record.type_length, TYPE_LEN as u8);
        // Check payload length
        assert_eq!(second_record.payload_length, PAYLOAD_LEN as u32);
        // Check ID length
        assert_eq!(second_record.id_length, None);
        // Check record type
        assert_eq!(second_record.record_type, [0x54]);
        // Check ID
        assert_eq!(second_record.id, None);
        // Check payload
        assert_eq!(second_record.payload, [0x2, 0x65, 0x6e, 0x57, 0x6f, 0x72, 0x6c, 0x64]);

        // Test first record deserialization
        let first_record = records.pop().unwrap();
        // Check header
        assert_eq!(first_record.header.type_name_format, TypeNameFormat::WellKnown);
        assert!(!first_record.header.id_present);
        assert!(first_record.header.short);
        assert!(!first_record.header.chunk);
        assert!(first_record.header.message_begin);
        assert!(!first_record.header.message_end);

        // Check type length
        assert_eq!(first_record.type_length, TYPE_LEN as u8);
        // Check payload length
        assert_eq!(first_record.payload_length, PAYLOAD_LEN as u32);
        // Check ID length
        assert_eq!(first_record.id_length, None);
        // Check record type
        assert_eq!(first_record.record_type, [0x54]);
        // Check ID
        assert_eq!(first_record.id, None);
        // Check payload
        assert_eq!(first_record.payload, [0x2, 0x65, 0x6e, 0x48, 0x65, 0x6C, 0x6C, 0x6F]);
    }

    #[test]
    fn test_serialize_ndef_message_with_two_text_records() {
        const TYPE_LEN: usize = 1;
        const PAYLOAD_LEN: usize = 8;

        let mut records = Vec::new();

        // Create two NDEF records
        let first_record: NdefRecord<32> = NdefRecord {
            header: NdefRecordHeader {
                type_name_format: TypeNameFormat::WellKnown,
                id_present: false,
                short: true,
                chunk: false,
                message_end: false,
                message_begin: true,
            },
            type_length: TYPE_LEN as u8,
            payload_length: PAYLOAD_LEN as u32,
            id_length: None,
            record_type: Vec::from_slice(&[0x54]).unwrap(),
            id: None,
            payload: Vec::from_slice(&[0x2, 0x65, 0x6e, 0x48, 0x65, 0x6C, 0x6C, 0x6F]).unwrap(), // "Hello"
        };
        records.push(first_record).unwrap();

        let second_record: NdefRecord<32> = NdefRecord {
            header: NdefRecordHeader {
                type_name_format: TypeNameFormat::WellKnown,
                id_present: false,
                short: true,
                chunk: false,
                message_end: true,
                message_begin: false,
            },
            type_length: TYPE_LEN as u8,
            payload_length: PAYLOAD_LEN as u32,
            id_length: None,
            record_type: Vec::from_slice(&[0x54]).unwrap(),
            id: None,
            payload: Vec::from_slice(&[0x2, 0x65, 0x6e, 0x57, 0x6f, 0x72, 0x6c, 0x64]).unwrap(), // "World"
        };
        records.push(second_record).unwrap();

        let tlv = NdefTlv::<32, 2> {
            tl: TL {
                tag: Tag::Ndef,
                length: Some(24), // Length will be the sum of both records
            },
            value: Some(records),
        };

        // Calculate the required buffer size
        let mut buffer = [0u8; 256];
        let bytes_written = tlv.to_bytes(&mut buffer).unwrap();

        // Expected serialized buffer based on the provided buffer in your test case
        let expected_buffer = [
            0x3, 0x18, 0x91, 0x1, 0x8, 0x54, 0x2, 0x65, 0x6e, 0x48, 0x65, 0x6c, 0x6c, 0x6f, 0x51, 0x1, 0x8, 0x54, 0x2, 0x65,
            0x6e, 0x57, 0x6f, 0x72, 0x6c, 0x64, 0xfe, 0x0, 0x72, 0x64, 0x2d, 0x76, 0x31, 0x0, 0x0, 0x0, 0xfe, 0x0, 0x0, 0x0,
            0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
            0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
            0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
        ];

        // Assert that the serialized buffer matches the expected output
        assert_eq!(&buffer[..bytes_written], &expected_buffer[..bytes_written]);
    }
}
