#![no_std]
//! # NDEF `no_std` Parser
//!
//! This crate provides a simple, `no_std` compatible parser for NFC Data Exchange Format (NDEF) messages.
//! It supports parsing and serializing Capability Containers (CC) and NDEF records using fixed-size buffers.
//!
//! Designed for embedded environments where memory constraints are critical and allocators are unavailable.
//!
//! ## Features
//! - **NFC Forum Type 5 Tag (T5T)** parsing.
//! - **Capability Container (CC)** decoding.
//! - **NDEF TLV (Type-Length-Value)** handling.
//!
//! ## Example Usage
//!
//! This example demonstrates decoding a Capability Container and an NDEF TLV record:
//!
//! ```ignore
//! use ndef::{CapabilityContainer, NdefTlv, NdefRecord};
//!
//! let cc_bytes = [0xE1, 0x40, 0x40, 0x01];
//! match CapabilityContainer::unpack(&cc_bytes) {
//!     Ok(cc) => println!("{:?}", cc),
//!     Err(e) => println!("Error parsing Capability Container: {:?}", e),
//! }
//!
//! let ndef_tlv_bytes = [0x03, 0x27, /* NDEF record data */];
//! const MAX_PAYLOAD_SIZE: usize = 1024;
//!
//! match NdefTlv::<MAX_PAYLOAD_SIZE>::from_bytes(&ndef_tlv_bytes) {
//!     Ok(ndef_tlv) => println!("{:?}", ndef_tlv),
//!     Err(e) => println!("Error parsing NDEF TLV: {:?}", e),
//! }
//! ```
//!
//! ## NFC Concepts
//!
//! - **Capability Container (CC)**: Stores metadata about the NFC Forum Type 5 Tag (T5T).
//! - **NDEF (NFC Data Exchange Format)**: A standardized format for exchanging data over NFC.
//! - **TLV (Type-Length-Value)**: A flexible encoding structure used within NFC tags.
//! - **NDEF Record**: A structured unit within an NDEF message containing a payload, type information, and metadata.
//!
//! ## Notes
//!
//! - **Work in Progress**: This crate may not be fully functional or stable.
//! - **Compatibility**: Initially built for **ST25DV tags**, but PRs are welcome to improve compatibility with other NFC tags.
//! - **Multiple NDEF Records**: Support for parsing multiple NDEF records in a single buffer is planned.
//! - **Payload Decoding**: Future support for decoding payloads of known types (e.g., URI, MIME) is planned.
//!
//! ## Contributing
//!
//! Contributions are welcome! To improve compatibility or add features, please submit a PR.

use heapless::Vec;
use packed_struct::prelude::*;
use thiserror::Error;

/// TLV error types
#[derive(Error, Debug)]
pub enum TlvError {
    #[error("Invalid TLV tag")]
    InvalidTag,
    #[error("Invalid TLV length")]
    InvalidLength,
    #[error("Provided buffer is too small")]
    BufferTooSmall,
    #[error("Unsupported Tag type, only NDEF is supported")]
    NotNdefType,
}

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

/// Magic number of the Capability Container (CC).
///
/// Indicates the type of the Capability Container. Typically, values are:
/// - `E1` (0xE1): Standard CC.
/// - `E2` (0xE2): Extended CC for the ST25DV (might be used for other tags?).
#[derive(PrimitiveEnum_u8, Clone, Copy, Debug, PartialEq)]
pub enum MagicNumber {
    E1 = 0xE1,
    E2 = 0xE2,
}

/// Write access permissions for memory.
///
/// Represents the rights associated with writing data to the memory area.
#[derive(PrimitiveEnum_u8, Clone, Copy, Debug, PartialEq)]
pub enum WriteAccess {
    Always = 0x00,
    RFU = 0x01,
    Proprietary = 0x02,
    Never = 0x03,
}

/// Read access permissions for memory.
///
/// Represents the rights associated with reading data from the memory area.
#[derive(PrimitiveEnum_u8, Clone, Copy, Debug, PartialEq)]
pub enum ReadAccess {
    Always = 0x00,
    RFU1 = 0x01,
    Proprietary = 0x02,
    RFU2 = 0x03,
}

/// Represents the version and access conditions of the Capability Container.
///
/// This byte encodes the version and read/write permissions.
/// For version 1.0 with all accesses granted, this byte value is 40h (commonly used).
#[derive(PackedStruct)]
#[packed_struct(size_bytes = "1", bit_numbering = "lsb0")]
pub struct VersionAccessCondition {
    #[packed_field(bits = "0..2", ty = "enum")]
    write_access: WriteAccess,
    #[packed_field(bits = "2..4", ty = "enum")]
    read_access: ReadAccess,
    #[packed_field(bits = "4..6")]
    minor_version: u8,
    #[packed_field(bits = "6..8")]
    major_version: u8,
}

/// Additional feature information byte description
#[derive(PackedStruct)]
#[packed_struct(size_bytes = "1", bit_numbering = "lsb0")]
pub struct AdditionalFeatureInformation {
    /// Support Read Multiple Block command
    #[packed_field(bits = "0")]
    mbread: bool,
    /// These bits are reserved for future use and must be set to 0.
    #[packed_field(bits = "1..3")]
    rfu1: u8,
    /// Support Lock Block command
    #[packed_field(bits = "3")]
    lock_block: bool,
    /// Support Special Frame
    #[packed_field(bits = "4")]
    special_frame: bool,
    /// These bits are reserved for future use and must be set to 0.
    #[packed_field(bits = "5..8")]
    rfu2: u8,
}

/// Capability Container (CC) for NFC Forum Type 5 Tags.
///
/// The Capability Container stores metadata about the NFC tag, including versioning, access conditions,
/// and supported features. This structure currently supports the four-byte format.
///
/// **Note:** Eight-byte CC decoding is planned but not yet implemented.
// TODO: add an eight byte CC decoder
#[derive(PackedStruct)]
#[packed_struct(size_bytes = "4", bit_numbering = "msb0")]
pub struct CapabilityContainer {
    #[packed_field(bytes = "0", ty = "enum")]
    magic_number: MagicNumber,
    #[packed_field(bytes = "1")]
    version_access_condition: VersionAccessCondition,
    #[packed_field(bytes = "2")]
    mlen: u8,
    #[packed_field(bytes = "3")]
    addional_feature_information: AdditionalFeatureInformation,
}

/// Tag type part of NDEF TLV
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Tag {
    Null = 0x00,
    Ndef = 0x03,
    Proprietary = 0xFD,
    Terminator = 0xFE,
}

impl TryFrom<u8> for Tag {
    type Error = TlvError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x00 => Ok(Tag::Null),
            0x03 => Ok(Tag::Ndef),
            0xFD => Ok(Tag::Proprietary),
            0xFE => Ok(Tag::Terminator),
            _ => Err(TlvError::InvalidTag),
        }
    }
}

/// Type-Length (part of TLV) structure
pub struct TL {
    tag: Tag,
    /// In the case of an NDEF tag, the length field indicates the length of the NDEF record.
    length: Option<u32>,
}

/// TLV (Type-Length-Value) structure specifically for NDEF messages.
///
/// This structure represents a Type-Length-Value (TLV) block containing an NDEF record.
/// The TLV format is used to encapsulate NDEF messages in NFC Forum Type 5 Tags.
///
/// # Type Parameters
/// * `MAX_PAYLOAD_SIZE`: Maximum size in bytes for the NDEF record payload
///
/// # Fields
/// * `tl`: Type and Length fields of the TLV block
/// * `value`: The NDEF record contained in this TLV block
pub struct NdefTlv<const MAX_PAYLOAD_SIZE: usize> {
    /// Type and Length bytes of the TLV block
    pub tl: TL,
    /// The NDEF record value
    pub value: Option<NdefRecord<MAX_PAYLOAD_SIZE>>,
}

impl<const MAX_PAYLOAD_SIZE: usize> NdefTlv<MAX_PAYLOAD_SIZE> {
    /// Parses a TLV structure from a byte slice.
    ///
    /// # Parameters
    /// - `bytes`: A slice of bytes containing the TLV data.
    ///
    /// # Errors
    /// Returns a `TlvError` if the input is too small, invalid, or not an NDEF type.
    ///
    /// # Example
    /// ```ignore
    /// let bytes = [0x03, 0x10, /* NDEF record bytes */];
    /// let ndef_tlv = NdefTlv::<1024>::from_bytes(&bytes)?;
    /// ```
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, TlvError> {
        // The TLV block must be at least 1 bytes long (Terminator TLV only contains the tag byte)
        if bytes.is_empty() {
            return Err(TlvError::BufferTooSmall);
        }

        // Handle terminator TLV
        if bytes[0] == Tag::Terminator as u8 {
            return Ok(Self {
                tl: TL {
                    tag: Tag::Terminator,
                    length: None,
                },
                value: None,
            });
        }

        // Parse TL bytes
        // If the length field = 0xFF, we should parsed the extended field length and populate it
        let tl = if bytes[1] == 0xFF {
            if bytes.len() < 4 {
                return Err(TlvError::BufferTooSmall);
            }
            let length = ((bytes[1] as u32) << 16) | ((bytes[2] as u32) << 8) | (bytes[3] as u32);
            TL {
                tag: bytes[0].try_into().map_err(|_| TlvError::InvalidTag)?,
                length: Some(length),
            }
        } else {
            TL {
                tag: bytes[0].try_into().map_err(|_| TlvError::InvalidTag)?,
                length: Some(bytes[1] as u32),
            }
        };

        // We only support NDEF tags
        if tl.tag != Tag::Ndef {
            return Err(TlvError::NotNdefType);
        }

        // Parse NDEF record length
        let value_length = tl.length.ok_or(TlvError::InvalidLength)? as usize;
        if bytes.len() < 2 + value_length {
            return Err(TlvError::BufferTooSmall);
        }

        // Parse NDEF record
        let value = Some(
            NdefRecord::from_bytes(&bytes[2..2 + value_length])
                .map_err(|_| TlvError::InvalidLength)?,
        );

        Ok(Self { tl, value })
    }

    /// Get the total size of the TLV structure
    pub fn total_size(&self) -> Result<usize, TlvError> {
        let length = self.tl.length.ok_or(TlvError::InvalidLength)?;
        Ok(2 + length as usize)
    }

    /// Serializes the TLV structure to bytes, writing to a provided buffer.
    ///
    /// # Parameters
    /// - `buffer`: A mutable byte slice to write the serialized data.
    ///
    /// # Returns
    /// The number of bytes written on success.
    ///
    /// # Errors
    /// Returns `TlvError::BufferTooSmall` if the buffer is too small.
    ///
    /// # Example
    /// ```ignore
    /// let mut buffer = [0u8; 128];
    /// let bytes_written = ndef_tlv.to_bytes(&mut buffer)?;
    /// ```
    pub fn to_bytes(&self, buffer: &mut [u8]) -> Result<usize, TlvError> {
        let required_size = self.total_size()?;

        // Check if the buffer is too small
        if buffer.len() < required_size {
            return Err(TlvError::BufferTooSmall);
        }

        let mut offset = 0;

        // Write the tag
        buffer[offset] = self.tl.tag as u8;
        offset += 1;

        // Handle length field
        if let Some(length) = self.tl.length {
            if length > 0xFE {
                // Extended length (0xFF followed by 3-byte length)
                buffer[offset] = 0xFF;
                offset += 1;
                buffer[offset..offset + 2].copy_from_slice(&length.to_be_bytes()[1..3]);
                offset += 2;
            } else {
                // Regular length
                buffer[offset] = length as u8;
                offset += 1;
            }

            // Write the NDEF record value if present
            if let Some(record) = &self.value {
                let value_buffer = &mut buffer[offset..];
                let bytes_written = record.to_bytes(value_buffer).unwrap();
                offset += bytes_written;
            }
        } else {
            return Err(TlvError::InvalidLength);
        }

        // Insert terminator TLV
        buffer[offset] = Tag::Terminator as u8;
        offset += 1;

        Ok(offset)
    }
}

#[derive(PrimitiveEnum_u8, Clone, Copy, Debug, PartialEq)]
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
#[derive(PackedStruct)]
#[packed_struct(size_bytes = "1", bit_numbering = "lsb0")]
#[derive(PartialEq)]
pub struct NdefRecordHeader {
    /// Type Name Format (TNF) field that defines how to interpret the type field
    #[packed_field(bits = "0..3", ty = "enum")]
    type_name_format: TypeNameFormat,
    /// Indicates whether the record contains an ID field
    #[packed_field(bits = "3")]
    id_present: bool,
    /// The Short Record (SR) bit flag determines the length of the payload record.
    /// If SR is true, the payload length is one byte, otherwise it’s four bytes.
    /// Payload length is required, but may be zero.
    #[packed_field(bits = "4")]
    short: bool,
    /// The Chunk Bit Flag indicates whether the payload is a sequence of chunks.
    #[packed_field(bits = "5")]
    chunk: bool,
    /// The Message End Bit Flag indicates whether the record is the last one in the message.
    #[packed_field(bits = "6")]
    message_end: bool,
    /// The Message Begin Bit Flag indicates whether the record is the first one in the message.
    #[packed_field(bits = "7")]
    message_begin: bool,
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
#[derive(PartialEq)]
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
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, NdefRecordError> {
        // Minimum size check (header + type_length + payload_length)
        if bytes.len() < 3 {
            return Err(NdefRecordError::BufferTooSmall);
        }

        // Parse header first to determine SR flag
        let header =
            NdefRecordHeader::unpack(&[bytes[0]]).map_err(|_| NdefRecordError::InvalidHeader)?;

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

        Ok(Self {
            header,
            type_length,
            payload_length,
            id_length,
            record_type,
            id,
            payload,
        })
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
        buffer[offset] = self
            .header
            .pack()
            .map_err(|_| NdefRecordError::InvalidHeader)?[0];
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
    use super::*;

    #[test]
    fn test_parse_cc() {
        let bytes = [0xE1, 0x40, 0x40, 0x01];
        let cc = CapabilityContainer::unpack(&bytes).unwrap();
        assert_eq!(cc.magic_number, MagicNumber::E1);
        assert_eq!(
            cc.version_access_condition.write_access,
            WriteAccess::Always
        );
        assert_eq!(cc.version_access_condition.read_access, ReadAccess::Always);
        assert_eq!(cc.version_access_condition.minor_version, 0);
        assert_eq!(cc.version_access_condition.major_version, 1);
        assert_eq!(cc.mlen, 64);
        assert!(cc.addional_feature_information.mbread);
        assert_eq!(cc.addional_feature_information.rfu1, 0);
        assert!(!cc.addional_feature_information.lock_block);
        assert!(!cc.addional_feature_information.special_frame);
        assert_eq!(cc.addional_feature_information.rfu2, 0);
    }

    #[test]
    fn test_parse_ndef_tlv() {
        let bytes = [
            0x3, 0x27, 0xd4, 0x1c, 0x8, 0x74, 0x68, 0x65, 0x72, 0x6d, 0x69, 0x67, 0x6f, 0x2e, 0x63,
            0x6f, 0x6d, 0x3a, 0x72, 0x75, 0x73, 0x74, 0x70, 0x6f, 0x73, 0x74, 0x63, 0x61, 0x72,
            0x64, 0x2d, 0x76, 0x31, 0xd, 0xe, 0xa, 0xd, 0xb, 0xe, 0xe, 0xf, 0xfe, 0x0, 0x0, 0x0,
            0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
        ];
        let tlv = NdefTlv::<1024>::from_bytes(&bytes).unwrap();

        assert_eq!(tlv.tl.tag, Tag::Ndef);
        assert_eq!(tlv.tl.length, Some(39));

        let total_tlv_size = tlv.total_size().expect("total size should be present");
        let value = tlv.value.expect("value should be present");
        // Check header
        assert_eq!(value.header.type_name_format, TypeNameFormat::External);
        assert!(!value.header.id_present);
        assert!(value.header.short);
        assert!(!value.header.chunk);
        assert!(value.header.message_begin);
        assert!(value.header.message_end);

        // Check type length
        assert_eq!(value.type_length, 28);

        // Check payload length
        assert_eq!(value.payload_length, 8);

        // Check ID length
        assert_eq!(value.id_length, None);

        // Check record type
        assert_eq!(
            value.record_type,
            [
                0x74, 0x68, 0x65, 0x72, 0x6d, 0x69, 0x67, 0x6f, 0x2e, 0x63, 0x6f, 0x6d, 0x3a, 0x72,
                0x75, 0x73, 0x74, 0x70, 0x6f, 0x73, 0x74, 0x63, 0x61, 0x72, 0x64, 0x2d, 0x76, 0x31
            ]
        );

        // Check ID
        assert_eq!(value.id, None);

        assert_eq!(total_tlv_size, 41);
    }

    #[test]
    fn test_build_ndef_tlv() {
        let record_type: Vec<u8, 255> = Vec::from_slice(&[
            0x74, 0x68, 0x65, 0x72, 0x6d, 0x69, 0x67, 0x6f, 0x2e, 0x63, 0x6f, 0x6d, 0x3a, 0x72,
            0x75, 0x73, 0x74, 0x70, 0x6f, 0x73, 0x74, 0x63, 0x61, 0x72, 0x64, 0x2d, 0x76, 0x31,
        ])
        .unwrap();
        let payload: Vec<u8, 1024> =
            Vec::from_slice(&[0xd, 0xe, 0xa, 0xd, 0xb, 0xe, 0xe, 0xf]).unwrap();
        let tlv = NdefTlv::<1024> {
            tl: TL {
                tag: Tag::Ndef,
                length: Some(39),
            },
            value: Some(NdefRecord {
                header: NdefRecordHeader {
                    type_name_format: TypeNameFormat::External,
                    id_present: false,
                    short: true,
                    chunk: false,
                    message_begin: true,
                    message_end: true,
                },
                type_length: 28,
                payload_length: 8,
                id_length: None,
                record_type,
                id: None,
                payload,
            }),
        };

        let mut buffer = [0u8; 60];
        let _ = tlv.to_bytes(&mut buffer).unwrap();
        assert_eq!(
            buffer,
            [
                0x3, 0x27, 0xd4, 0x1c, 0x8, 0x74, 0x68, 0x65, 0x72, 0x6d, 0x69, 0x67, 0x6f, 0x2e,
                0x63, 0x6f, 0x6d, 0x3a, 0x72, 0x75, 0x73, 0x74, 0x70, 0x6f, 0x73, 0x74, 0x63, 0x61,
                0x72, 0x64, 0x2d, 0x76, 0x31, 0xd, 0xe, 0xa, 0xd, 0xb, 0xe, 0xe, 0xf, 0xfe, 0x0,
                0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
                0x0
            ]
        );
    }

    #[test]
    fn test_parse_ndef_tlv_errors() {
        // Test buffer too small
        let bytes = [0x03, 0x04, 0x01];
        assert!(matches!(
            NdefTlv::<1024>::from_bytes(&bytes),
            Err(TlvError::BufferTooSmall)
        ));

        // Test wrong tag type
        let bytes = [0x00, 0x04, 0x01, 0x02, 0x03, 0x04];
        assert!(matches!(
            NdefTlv::<1024>::from_bytes(&bytes),
            Err(TlvError::NotNdefType)
        ));
    }

    #[test]
    fn test_parse_ndef_record_header() {
        let bytes = [0xD4];
        let ndef_record_header = NdefRecordHeader::unpack(&bytes).unwrap();
        assert_eq!(
            ndef_record_header.type_name_format,
            TypeNameFormat::External
        );
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
            0xd4, 0x1c, 0x8, 0x74, 0x68, 0x65, 0x72, 0x6d, 0x69, 0x67, 0x6f, 0x2e, 0x63, 0x6f,
            0x6d, 0x3a, 0x72, 0x75, 0x73, 0x74, 0x70, 0x6f, 0x73, 0x74, 0x63, 0x61, 0x72, 0x64,
            0x2d, 0x76, 0x31, 0xd, 0xe, 0xa, 0xd, 0xb, 0xe, 0xe, 0xf, 0xfe, 0x0, 0x0, 0x0, 0x0,
            0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
        ];

        let parsed = NdefRecord::<1024>::from_bytes(&buffer).unwrap();

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
                0x74, 0x68, 0x65, 0x72, 0x6d, 0x69, 0x67, 0x6f, 0x2e, 0x63, 0x6f, 0x6d, 0x3a, 0x72,
                0x75, 0x73, 0x74, 0x70, 0x6f, 0x73, 0x74, 0x63, 0x61, 0x72, 0x64, 0x2d, 0x76, 0x31
            ]
        );

        // Check ID
        assert_eq!(parsed.id, None);

        // Check payload
        assert_eq!(parsed.payload, [0xd, 0xe, 0xa, 0xd, 0xb, 0xe, 0xe, 0xf]);
    }
}
