//! Kafka traffic captured outside Alluvium, read back for debugging (PROTO-F13): hex and
//! text dumps of a frame's bytes ([`input`]), for `alluvium dump frame`.
//!
//! Layer: 1 ([ADR-0009](../../../docs/adr/0009-crate-layering-and-dependency-rules.md)).
//! It does no I/O: callers pass the bytes of a file or of standard input, and get back
//! the bytes the dump holds.
//!
//! Kafka counterpart: none. Kafka leaves captures to Wireshark's Kafka dissector.
//!
//! # Invariants
//!
//! - Input is untrusted: parsing never panics, and a malformed dump is an error naming
//!   the line and column where it goes wrong.
//! - This crate denies `clippy::indexing_slicing` and `clippy::arithmetic_side_effects`,
//!   so input-derived indexes and arithmetic are checked (docs/standards/security.md).
#![deny(clippy::indexing_slicing, clippy::arithmetic_side_effects)]

pub mod input;
