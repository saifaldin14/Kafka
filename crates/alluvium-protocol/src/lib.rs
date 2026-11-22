//! Kafka's wire protocol: the primitive encodings every request, response and header
//! field is made of, decoded from untrusted bytes without panicking.
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
//! - This crate denies `clippy::indexing_slicing` and `clippy::arithmetic_side_effects`,
//!   so input-derived indexes and arithmetic are checked (docs/standards/security.md).
//!
//! Errors carry no Kafka error code: Kafka closes the connection on a request it can't
//! parse, and the network layer decides that.
#![deny(clippy::indexing_slicing, clippy::arithmetic_side_effects)]

mod error;
mod framed;
mod reader;
mod tagged_fields;
mod varint;
mod writer;

pub use error::{DecodeError, DecodeErrorKind, EncodeError};
pub use framed::Framing;
pub use reader::Reader;
pub use tagged_fields::RawTaggedField;
pub use varint::{size_of_unsigned_varint, size_of_varint, size_of_varlong};
pub use writer::Writer;
