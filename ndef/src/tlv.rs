use heapless::Vec;
use thiserror::Error;

use crate::ndef_record::{NdefRecord, NdefRecordError};

/// TLV error types
#[derive(Error, Debug)]
pub enum TlvError {
    #[error("Invalid TLV tag")]
    InvalidTag,
    #[error("Input buffer is empty")]
    EmptyInputBuffer,
    #[error("Incomplete input buffer")]
    IncompleteInputBuffer,
    #[error("Invalid TLV length")]
    InvalidLength,
    #[error("Maximum number of NDEF records exceeded, provide a larger MAX_RECORDS")]
    MaxRecordsExceeded,
    #[error("Provided buffer is too small, provided: {provided}, required: {required}")]
    BufferTooSmall { provided: usize, required: usize },
    #[error("Unsupported Tag type, only NDEF is supported")]
    NotNdefType,
    #[error("Invalid NDEF record")]
    NdefRecordError(#[from] NdefRecordError),
    #[error("Too many NDEF records, maximum is {0}")]
    TooManyRecords(usize),
    #[error("Vector operations failed")]
    VectorFull,
}

/// Tag type part of NDEF TLV
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
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
#[derive(Debug)]
#[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
pub struct TL {
    pub tag: Tag,
    /// In the case of an NDEF tag, the length field indicates the length of the NDEF record.
    pub length: Option<u32>,
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
#[derive(Debug)]
#[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
pub struct NdefTlv<const MAX_PAYLOAD_SIZE: usize, const MAX_RECORDS: usize> {
    /// Type and Length bytes of the TLV block
    pub tl: TL,
    /// The NDEF record value
    pub value: Option<Vec<NdefRecord<MAX_PAYLOAD_SIZE>, MAX_RECORDS>>,
}

impl<const MAX_PAYLOAD_SIZE: usize, const MAX_RECORDS: usize> NdefTlv<MAX_PAYLOAD_SIZE, MAX_RECORDS> {
    /// Creates a new NDEF TLV structure containing one or more NDEF records
    ///
    /// # Arguments
    ///
    /// * `records` - Slice of NDEF records to include in the TLV
    ///
    /// # Returns
    ///
    /// Returns a Result containing the NdefTlv if successful, or an Error if:
    /// - The number of records exceeds MAX_RECORDS
    /// - Vector operations fail
    /// - The total length of all records exceeds the maximum allowed TLV length
    pub fn new(records: &[NdefRecord<MAX_PAYLOAD_SIZE>]) -> Result<Self, TlvError> {
        if records.is_empty() {
            return Ok(Self {
                tl: TL {
                    tag: Tag::Ndef,
                    length: None,
                },
                value: None,
            });
        }

        if records.len() > MAX_RECORDS {
            return Err(TlvError::TooManyRecords(MAX_RECORDS));
        }

        // Create vector of records, updating message begin/end flags
        let mut ndef_records: Vec<NdefRecord<MAX_PAYLOAD_SIZE>, MAX_RECORDS> = Vec::new();

        for (index, record) in records.iter().enumerate() {
            let mut record = record.clone();

            // Update header flags for position in message
            let is_first = index == 0;
            let is_last = index == records.len() - 1;

            if is_first {
                record.header.message_begin = true;
            }

            if is_last {
                record.header.message_end = true;
            }

            ndef_records.push(record).map_err(|_| TlvError::VectorFull)?;
        }

        // Calculate total length of all records
        let total_length: u32 = ndef_records.iter().map(|record| record.serialized_size() as u32).sum();

        Ok(Self {
            tl: TL {
                tag: Tag::Ndef,
                length: Some(total_length),
            },
            value: Some(ndef_records),
        })
    }

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
            #[cfg(feature = "defmt-03")]
            defmt::trace!("Buffer is empty");
            return Err(TlvError::EmptyInputBuffer);
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
                #[cfg(feature = "defmt-03")]
                defmt::trace!("Buffer too small for extended length field");
                return Err(TlvError::IncompleteInputBuffer);
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
            #[cfg(feature = "defmt-03")]
            defmt::trace!("Buffer too small for NDEF record, need {} bytes", 2 + value_length);
            return Err(TlvError::BufferTooSmall {
                provided: bytes.len(),
                required: 2 + value_length,
            });
        }

        // Parse NDEF record
        #[cfg(feature = "defmt-03")]
        defmt::trace!("Attempting to parse NDEF records");
        let mut vec: Vec<NdefRecord<MAX_PAYLOAD_SIZE>, MAX_RECORDS> = Vec::new();
        let mut offset = 2; // Start after initial 2 bytes
        let mut total_bytes_processed = 0;

        while total_bytes_processed < value_length {
            let remaining_bytes = &bytes[offset..];
            let (record, bytes_processed) = NdefRecord::from_bytes(remaining_bytes)?;

            if vec.push(record).is_err() {
                return Err(TlvError::MaxRecordsExceeded);
            }

            offset += bytes_processed;
            total_bytes_processed += bytes_processed;

            if vec.last().unwrap().header.message_end() {
                break;
            }
        }

        let value = Some(vec);

        #[cfg(feature = "defmt-03")]
        defmt::trace!("Successfully parsed {} NDEF records from TLV", vec.len());

        Ok(Self { tl, value })
    }

    /// Get the total size of the TLV structure
    fn total_size(&self) -> Result<usize, TlvError> {
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
            return Err(TlvError::BufferTooSmall {
                provided: buffer.len(),
                required: required_size,
            });
        }

        let mut offset = 0;

        // Write the tag
        buffer[offset] = self.tl.tag as u8;
        offset += 1;

        // Handle length field
        if let Some(length) = self.tl.length {
            if length > 0xFE {
                // Extended length (0xFF followed by 2-byte length)
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
            if let Some(records) = &self.value {
                for record in records {
                    let value_buffer = &mut buffer[offset..];

                    let bytes_written = record.to_bytes(value_buffer)?;

                    offset += bytes_written;
                }
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

#[cfg(test)]
mod tests {
    use heapless::Vec;

    use super::{NdefTlv, Tag, TlvError, TL};
    use crate::ndef_record::{NdefRecord, NdefRecordHeader, TypeNameFormat};

    #[test]
    fn test_parse_ndef_tlv() {
        let bytes = [
            0x3, 0x27, 0xd4, 0x1c, 0x8, 0x74, 0x68, 0x65, 0x72, 0x6d, 0x69, 0x67, 0x6f, 0x2e, 0x63, 0x6f, 0x6d, 0x3a, 0x72,
            0x75, 0x73, 0x74, 0x70, 0x6f, 0x73, 0x74, 0x63, 0x61, 0x72, 0x64, 0x2d, 0x76, 0x31, 0xd, 0xe, 0xa, 0xd, 0xb, 0xe,
            0xe, 0xf, 0xfe, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
        ];
        let tlv = NdefTlv::<1024, 1>::from_bytes(&bytes).unwrap();

        assert_eq!(tlv.tl.tag, Tag::Ndef);
        assert_eq!(tlv.tl.length, Some(39));

        let total_tlv_size = tlv.total_size().expect("total size should be present");
        let value = tlv.value.expect("value should be present").pop().unwrap();
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
                0x74, 0x68, 0x65, 0x72, 0x6d, 0x69, 0x67, 0x6f, 0x2e, 0x63, 0x6f, 0x6d, 0x3a, 0x72, 0x75, 0x73, 0x74, 0x70,
                0x6f, 0x73, 0x74, 0x63, 0x61, 0x72, 0x64, 0x2d, 0x76, 0x31
            ]
        );

        // Check ID
        assert_eq!(value.id, None);

        assert_eq!(total_tlv_size, 41);
    }

    #[test]
    fn test_build_ndef_tlv() {
        let record_type: Vec<u8, 255> = Vec::from_slice(&[
            0x74, 0x68, 0x65, 0x72, 0x6d, 0x69, 0x67, 0x6f, 0x2e, 0x63, 0x6f, 0x6d, 0x3a, 0x72, 0x75, 0x73, 0x74, 0x70, 0x6f,
            0x73, 0x74, 0x63, 0x61, 0x72, 0x64, 0x2d, 0x76, 0x31,
        ])
        .unwrap();
        let payload: Vec<u8, 1024> = Vec::from_slice(&[0xd, 0xe, 0xa, 0xd, 0xb, 0xe, 0xe, 0xf]).unwrap();

        let record = NdefRecord {
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
        };
        let mut records = Vec::new();
        records.push(record).unwrap();
        let tlv = NdefTlv::<1024, 1> {
            tl: TL {
                tag: Tag::Ndef,
                length: Some(39),
            },
            value: Some(records),
        };

        let mut buffer = [0u8; 60];
        let _ = tlv.to_bytes(&mut buffer).unwrap();
        assert_eq!(
            buffer,
            [
                0x3, 0x27, 0xd4, 0x1c, 0x8, 0x74, 0x68, 0x65, 0x72, 0x6d, 0x69, 0x67, 0x6f, 0x2e, 0x63, 0x6f, 0x6d, 0x3a, 0x72,
                0x75, 0x73, 0x74, 0x70, 0x6f, 0x73, 0x74, 0x63, 0x61, 0x72, 0x64, 0x2d, 0x76, 0x31, 0xd, 0xe, 0xa, 0xd, 0xb,
                0xe, 0xe, 0xf, 0xfe, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0
            ]
        );
    }

    #[test]
    fn test_parse_ndef_tlv_errors() {
        // Test buffer too small
        let bytes = [0x03, 0x04, 0x01];
        assert!(matches!(
            NdefTlv::<1024, 1>::from_bytes(&bytes),
            Err(TlvError::BufferTooSmall {
                provided: _,
                required: _
            })
        ));

        // Test wrong tag type
        let bytes = [0x00, 0x04, 0x01, 0x02, 0x03, 0x04];
        assert!(matches!(NdefTlv::<1024, 1>::from_bytes(&bytes), Err(TlvError::NotNdefType)));
    }
}
