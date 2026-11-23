//! Kafka traffic captured outside Alluvium, read back for debugging (PROTO-F13), for
//! `alluvium dump frame`:
//! - hex and text dumps of a frame's bytes ([`input`]);
//! - packet captures: pcap and pcapng files read into TCP segments ([`packets`]), each
//!   connection's streams put in order and split into Kafka frames ([`streams`]), and
//!   requests paired with their responses ([`exchanges`]), so every frame decodes with
//!   `alluvium-protocol`'s frame dump.
//!
//! Layer: 1 ([ADR-0009](../../../docs/adr/0009-crate-layering-and-dependency-rules.md)).
//! It depends on `alluvium-protocol` and does no I/O: callers pass the bytes of a file or
//! of standard input.
//!
//! Kafka counterpart: none. Kafka leaves captures to Wireshark's Kafka dissector.
//!
//! # Invariants
//!
//! - Input is untrusted: reading never panics. A malformed dump is an error naming the
//!   line and column where it goes wrong, and a malformed packet or stream is a problem
//!   reported beside what could be read.
//! - This crate denies `clippy::indexing_slicing` and `clippy::arithmetic_side_effects`,
//!   so input-derived indexes and arithmetic are checked (docs/standards/security.md).
#![deny(clippy::indexing_slicing, clippy::arithmetic_side_effects)]

pub mod exchanges;
pub mod input;
pub mod packets;
pub mod streams;

use packets::{CaptureError, PacketProblem};
use streams::Connection;

/// A packet capture's connections, and the packets that couldn't be read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Capture {
    /// The TCP connections, in the order their first segments were captured.
    pub connections: Vec<Connection>,
    /// The packets that couldn't be read as TCP over IP.
    pub problems: Vec<PacketProblem>,
    /// The packets that are not TCP over IP, skipped.
    pub skipped: usize,
}

/// The Kafka connections in `capture`, a pcap or pcapng file.
///
/// # Errors
///
/// [`CaptureError`] if `capture` isn't a pcap or pcapng file, or ends inside a header or
/// block.
pub fn read_capture(capture: &[u8]) -> Result<Capture, CaptureError> {
    let segments = packets::read_segments(capture)?;
    Ok(Capture {
        connections: streams::connections(&segments.segments),
        problems: segments.problems,
        skipped: segments.skipped,
    })
}
