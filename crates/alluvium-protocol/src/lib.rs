//! Kafka's wire protocol: the primitive encodings every request, response and header
//! field is made of, decoded from untrusted bytes without panicking, and the message
//! types in [`generated`], one module per Kafka 4.4.0 schema.
//!
//! Layer: 0 ([ADR-0009](../../../docs/adr/0009-crate-layering-and-dependency-rules.md)).
//! It depends only on `alluvium-types` and does no I/O: a [`Reader`] reads from bytes
//! already in memory, and a [`Writer`] appends to a growable buffer.
//!
//! Kafka counterpart: `clients/.../common/protocol/` at 4.4.0 (`Readable`, `Writable`,
//! `ByteBufferAccessor`, `MessageUtil`), `clients/.../common/utils/internals/ByteUtils.java`
//! (varints), and the length and null checks Kafka's generator writes into every message
//! class (`generator/.../MessageDataGenerator.java`).
//!
//! # Invariants
//!
//! - Decoding never panics and never reads past its input: every read checks the
//!   remaining bytes first and returns a [`DecodeError`] naming the byte offset.
//! - A declared size is checked against the bytes remaining and the caller's
//!   [`DecodeLimits`] before anything is allocated for it.
//! - This crate denies `clippy::indexing_slicing` and `clippy::arithmetic_side_effects`,
//!   so input-derived indexes and arithmetic are checked (docs/standards/security.md).
//!
//! Errors carry no Kafka error code: Kafka closes the connection on a request it can't
//! parse, and the network layer decides that.
#![deny(clippy::indexing_slicing, clippy::arithmetic_side_effects)]

mod api_key;
mod api_versions;
mod array;
#[cfg(test)]
mod decode_tests;
#[cfg(test)]
mod encode_tests;
mod error;
mod error_code;
mod framed;
#[rustfmt::skip]
pub mod generated;
#[cfg(test)]
mod golden_tests;
#[cfg(test)]
mod kafka_message_tests;
mod limits;
mod message;
mod reader;
mod records;
mod tagged_fields;
#[cfg(test)]
#[rustfmt::skip]
pub mod test_messages;
#[cfg(test)]
mod test_strategies;
mod varint;
mod versions;
mod writer;

pub use api_key::{ApiKey, Listener};
pub use api_versions::advertised_api_versions;
pub use error::{DecodeError, DecodeErrorKind, EncodeError, WithinField};
pub use error_code::{ErrorCode, UnknownErrorCode};
pub use framed::Framing;
pub use limits::{DecodeLimits, Limit};
pub use message::{EncodedSize, Message, decode_version_prefixed, encode_version_prefixed};
pub use reader::Reader;
pub use records::RecordsField;
pub use tagged_fields::{RawTaggedField, RawTaggedFieldWriter};
pub use varint::{size_of_unsigned_varint, size_of_varint, size_of_varlong};
pub use versions::Versions;
pub use writer::Writer;
