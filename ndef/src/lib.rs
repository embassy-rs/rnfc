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
//! match NdefTlv::<MAX_PAYLOAD_SIZE, MAX_RECORDS>::from_bytes(&ndef_tlv_bytes) {
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
//! - **Payload Decoding**: Future support for decoding payloads of known types (e.g., URI, MIME) is planned.
//!
//! ## Contributing
//!
//! Contributions are welcome! To improve compatibility or add features, please submit a PR.

pub mod capability_container;
pub mod ndef_record;
pub mod tlv;
