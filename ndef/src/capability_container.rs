use packed_struct::prelude::*;

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
    pub write_access: WriteAccess,
    #[packed_field(bits = "2..4", ty = "enum")]
    pub read_access: ReadAccess,
    #[packed_field(bits = "4..6")]
    pub minor_version: u8,
    #[packed_field(bits = "6..8")]
    pub major_version: u8,
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

#[cfg(test)]
mod tests {
    use packed_struct::prelude::*;

    use super::{CapabilityContainer, MagicNumber, ReadAccess, WriteAccess};

    #[test]
    fn test_parse_cc() {
        let bytes = [0xE1, 0x40, 0x40, 0x01];
        let cc = CapabilityContainer::unpack(&bytes).unwrap();
        assert_eq!(cc.magic_number, MagicNumber::E1);
        assert_eq!(cc.version_access_condition.write_access, WriteAccess::Always);
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
}
