# NDEF `no_std` Parser Crate

This crate provides a lightweight, `no_std` compatible parser for NFC Data Exchange Format (NDEF) messages, specifically designed for embedded environments with limited memory.
It supports parsing and serializing Capability Containers (CC) and NDEF messages using fixed-size buffers.

## Features

- **NFC Forum Type 5 Tag (T5T) Support**: Parse NFC Type 5 tags commonly used in embedded NFC applications.
- **Capability Container (CC) Parsing**: Decode Capability Containers, which store metadata about the NFC tag.
- **NDEF Message Handling**: Parse and handle NDEF messages that use Type-Length-Value (TLV) structures.

## Usage Example

### Parsing a Capability Container (CC)

```rust
use ndef_parser::{CapabilityContainer, NdefError};

let cc_bytes = [0xE1, 0x40, 0x40, 0x01];
match CapabilityContainer::unpack(&cc_bytes) {
    Ok(cc) => println!("Parsed Capability Container: {:?}", cc),
    Err(e) => eprintln!("Error: {}", e),
}
```

### Parsing an NDEF TLV

```rust
use ndef_parser::{NdefTlv, NdefError};

let ndef_tlv_bytes = [0x03, 0x27, /* NDEF record data */];
// 1024 is the maximum size of an NDEF record payload, adjust as needed
match NdefTlv::<1024>::from_bytes(&ndef_tlv_bytes) {
    Ok(ndef_tlv) => println!("Parsed NDEF TLV: {:?}", ndef_tlv),
    Err(e) => eprintln!("Error: {}", e),
}
```

## NFC Concepts Overview

- **Capability Container (CC)**: Metadata structure that defines the capabilities of an NFC tag, including supported operations and memory layout.
- **NDEF Record**: Individual data record within an NDEF message, consisting of a header and payload.
- **TLV (Type-Length-Value)**: Encoded structure used for representing data in NDEF messages.

## Planned Features

- **Payload Decoding**: Support for decoding specific payload types like URIs, MIME types, and more.
- **Extended Tag Compatibility**: Improve compatibility with a wider range of NFC tags beyond ST25DV.

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT License ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

## Contributions

Contributions are welcome! Feel free to submit a PR to improve functionality, compatibility, or documentation.
