//! TCP segments from a packet capture (PROTO-0144): a pcap or pcapng file, its link
//! layer, then IPv4 or IPv6, then TCP.
//!
//! Supported link types: Ethernet, with 802.1Q and 802.1ad tags; Linux cooked capture
//! v1 and v2 (`tcpdump -i any`); raw IP; and BSD loopback (`lo0` on macOS). A packet
//! that isn't TCP over IP is skipped. A packet that can't be read, such as one cut short
//! by the capture's snapshot length or a fragment of an IP datagram, is reported as a
//! [`PacketProblem`], and reading goes on. Only a file that isn't a capture at all is an
//! error.

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};

/// One TCP segment from a capture.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TcpSegment {
    /// The packet's position in the capture, counting from 1 as Wireshark does.
    pub packet: usize,
    /// The sender.
    pub source: SocketAddr,
    /// The receiver.
    pub destination: SocketAddr,
    /// The sequence number of the first payload byte, or of the SYN.
    pub sequence: u32,
    /// Whether the segment opens a connection: SYN without ACK, sent by the client.
    pub opens: bool,
    /// Whether the segment carries SYN, which takes a sequence number before the payload.
    pub syn: bool,
    /// The TCP payload.
    pub payload: Vec<u8>,
}

/// A packet that couldn't be read as TCP over IP.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("packet {packet}: {problem}")]
pub struct PacketProblem {
    /// The packet's position in the capture, from 1.
    pub packet: usize,
    /// What is wrong with it.
    pub problem: PacketIssue,
}

/// What is wrong with a packet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum PacketIssue {
    /// The capture holds fewer bytes of the packet than were on the wire.
    #[error("only {captured} of its {original} bytes were captured")]
    CutShort {
        /// The bytes captured.
        captured: usize,
        /// The bytes on the wire.
        original: usize,
    },
    /// A header ends before its fields do.
    #[error("its {layer} header is truncated")]
    Truncated {
        /// The header: `ethernet`, `ipv4`, `tcp`…
        layer: &'static str,
    },
    /// A header field holds a value no valid packet has.
    #[error("its {layer} header has an impossible {field}")]
    BadHeader {
        /// The header: `ipv4`, `ipv6` or `tcp`.
        layer: &'static str,
        /// The field.
        field: &'static str,
    },
    /// A fragment of an IP datagram, which isn't reassembled.
    #[error("it is a fragment of an IP datagram, which this reader doesn't reassemble")]
    Fragment,
    /// A link type this reader doesn't support.
    #[error("its link type {link_type} isn't supported")]
    UnsupportedLinkType {
        /// The link type, as `LINKTYPE_` numbers it.
        link_type: u32,
    },
    /// A pcapng packet naming an interface the section hasn't described.
    #[error("it names interface {interface}, which the section doesn't describe")]
    UnknownInterface {
        /// The interface id.
        interface: u32,
    },
}

/// Why a file isn't a capture.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum CaptureError {
    /// The file starts with neither pcap's nor pcapng's magic number.
    #[error("the file is neither pcap nor pcapng: it starts with {start:02x?}")]
    UnknownFormat {
        /// The first bytes of the file.
        start: Vec<u8>,
    },
    /// The file ends inside its file header or a block.
    #[error("the capture ends inside {what} at byte {offset}")]
    Truncated {
        /// What was being read.
        what: &'static str,
        /// The offset of its start.
        offset: usize,
    },
    /// A pcapng block whose length can't be right: too short for its fields or its
    /// packet, not a multiple of 4, or unequal to the copy of it that ends the block.
    #[error("the pcapng block at byte {offset} has length {length}")]
    BadBlockLength {
        /// The offset of the block.
        offset: usize,
        /// The length it declares.
        length: u32,
    },
}

/// The TCP segments of a capture, and the packets that couldn't be read.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Segments {
    /// The segments, in capture order.
    pub segments: Vec<TcpSegment>,
    /// The packets that couldn't be read.
    pub problems: Vec<PacketProblem>,
    /// The packets that are not TCP over IP, skipped.
    pub skipped: usize,
}

/// The TCP segments of `capture`, a pcap or pcapng file.
///
/// # Errors
///
/// [`CaptureError`] if `capture` isn't a pcap or pcapng file, or ends inside a header or
/// block.
pub fn read_segments(capture: &[u8]) -> Result<Segments, CaptureError> {
    let mut segments = Segments::default();
    match capture.first_chunk::<4>() {
        Some(&[0x0a, 0x0d, 0x0d, 0x0a]) => read_pcapng(capture, &mut segments)?,
        Some(magic) if pcap_order(*magic).is_some() => read_pcap(capture, &mut segments)?,
        _ => {
            return Err(CaptureError::UnknownFormat {
                start: capture
                    .get(..capture.len().min(8))
                    .unwrap_or_default()
                    .to_vec(),
            });
        }
    }
    Ok(segments)
}

/// Reads fixed-size fields in one byte order from a slice.
#[derive(Clone, Copy)]
struct Fields<'a> {
    bytes: &'a [u8],
    little_endian: bool,
}

impl<'a> Fields<'a> {
    fn big(bytes: &'a [u8]) -> Fields<'a> {
        Fields {
            bytes,
            little_endian: false,
        }
    }

    fn array<const N: usize>(&self, at: usize) -> Option<[u8; N]> {
        let end = at.checked_add(N)?;
        self.bytes.get(at..end)?.try_into().ok()
    }

    fn u8(&self, at: usize) -> Option<u8> {
        self.bytes.get(at).copied()
    }

    fn u16(&self, at: usize) -> Option<u16> {
        let bytes = self.array(at)?;
        Some(if self.little_endian {
            u16::from_le_bytes(bytes)
        } else {
            u16::from_be_bytes(bytes)
        })
    }

    fn u32(&self, at: usize) -> Option<u32> {
        let bytes = self.array(at)?;
        Some(if self.little_endian {
            u32::from_le_bytes(bytes)
        } else {
            u32::from_be_bytes(bytes)
        })
    }

    fn rest(&self, at: usize) -> Option<&'a [u8]> {
        self.bytes.get(at..)
    }
}

/// Whether a pcap file with this magic number is little-endian.
// The pcap file format: https://www.ietf.org/archive/id/draft-ietf-opsawg-pcap-04.html
fn pcap_order(magic: [u8; 4]) -> Option<bool> {
    match magic {
        // Microsecond and nanosecond timestamps, written on a little-endian machine.
        [0xd4, 0xc3, 0xb2, 0xa1] | [0x4d, 0x3c, 0xb2, 0xa1] => Some(true),
        [0xa1, 0xb2, 0xc3, 0xd4] | [0xa1, 0xb2, 0x3c, 0x4d] => Some(false),
        _ => None,
    }
}

fn truncated(what: &'static str, offset: usize) -> CaptureError {
    CaptureError::Truncated { what, offset }
}

fn read_pcap(capture: &[u8], out: &mut Segments) -> Result<(), CaptureError> {
    let little_endian = capture
        .first_chunk::<4>()
        .and_then(|magic| pcap_order(*magic))
        .ok_or(truncated("the pcap file header", 0))?;
    let header = Fields {
        bytes: capture,
        little_endian,
    };
    // The low 16 bits; the high ones say whether frames end with a frame check sequence,
    // which the IP packet's own length leaves out.
    let link_type = header.u32(20).ok_or(truncated("the pcap file header", 0))? & 0xffff;
    let mut offset = 24;
    let mut packet = 0_usize;
    while offset < capture.len() {
        packet = packet.saturating_add(1);
        let record = Fields {
            bytes: capture.get(offset..).unwrap_or_default(),
            little_endian,
        };
        let (Some(captured), Some(original)) = (record.u32(8), record.u32(12)) else {
            return Err(truncated("a pcap packet header", offset));
        };
        let captured = usize::try_from(captured).unwrap_or(usize::MAX);
        let start = offset.saturating_add(16);
        let end = start.saturating_add(captured);
        let data = capture
            .get(start..end)
            .ok_or(truncated("a pcap packet", offset))?;
        read_packet(out, packet, link_type, data, original);
        offset = end;
    }
    Ok(())
}

/// The pcapng block types read; others are skipped.
const SECTION_HEADER: u32 = 0x0a0d_0d0a;
const INTERFACE_DESCRIPTION: u32 = 1;
const SIMPLE_PACKET: u32 = 3;
const ENHANCED_PACKET: u32 = 6;

// The pcapng format: https://www.ietf.org/archive/id/draft-ietf-opsawg-pcapng-01.html
fn read_pcapng(capture: &[u8], out: &mut Segments) -> Result<(), CaptureError> {
    let mut offset = 0;
    let mut little_endian = true;
    // Each interface of the current section: its link type and snapshot length.
    let mut interfaces: Vec<(u32, u32)> = Vec::new();
    let mut packet = 0_usize;
    while offset < capture.len() {
        let rest = capture.get(offset..).unwrap_or_default();
        // In the section's byte order: the section header's own type reads the same in
        // either.
        let block_type = Fields {
            bytes: rest,
            little_endian,
        }
        .u32(0)
        .ok_or(truncated("a pcapng block", offset))?;
        if block_type == SECTION_HEADER {
            // The byte-order magic decides the order of the whole section.
            little_endian = match rest.get(8..12) {
                Some([0x4d, 0x3c, 0x2b, 0x1a]) => true,
                Some([0x1a, 0x2b, 0x3c, 0x4d]) => false,
                _ => return Err(truncated("a pcapng section header", offset)),
            };
            interfaces.clear();
        }
        let fields = Fields {
            bytes: rest,
            little_endian,
        };
        let length = fields.u32(4).ok_or(truncated("a pcapng block", offset))?;
        let size = usize::try_from(length).unwrap_or(usize::MAX);
        let bad_length = || CaptureError::BadBlockLength { offset, length };
        if size < 12 || !size.is_multiple_of(4) {
            return Err(bad_length());
        }
        let whole = Fields {
            bytes: rest
                .get(..size)
                .ok_or(truncated("a pcapng block", offset))?,
            little_endian,
        };
        let body_end = size.saturating_sub(4);
        if whole.u32(body_end) != Some(length) {
            return Err(bad_length());
        }
        // The block without its trailing length, so no field can be read from it.
        let block = Fields {
            bytes: whole.bytes.get(..body_end).unwrap_or_default(),
            little_endian,
        };
        match block_type {
            SECTION_HEADER => {
                // The version and the section length.
                block.u32(16).ok_or_else(bad_length)?;
            }
            INTERFACE_DESCRIPTION => {
                let link_type = block.u16(8).map(u32::from);
                let snapshot = block.u32(12);
                let (Some(link_type), Some(snapshot)) = (link_type, snapshot) else {
                    return Err(bad_length());
                };
                interfaces.push((link_type, snapshot));
            }
            ENHANCED_PACKET => {
                packet = packet.saturating_add(1);
                let (Some(interface), Some(captured), Some(original)) =
                    (block.u32(8), block.u32(20), block.u32(24))
                else {
                    return Err(bad_length());
                };
                let data = packet_data(block.bytes, 28, captured).ok_or_else(bad_length)?;
                match interfaces.get(usize::try_from(interface).unwrap_or(usize::MAX)) {
                    Some(&(link_type, _)) => read_packet(out, packet, link_type, data, original),
                    None => out.problems.push(PacketProblem {
                        packet,
                        problem: PacketIssue::UnknownInterface { interface },
                    }),
                }
            }
            SIMPLE_PACKET => {
                packet = packet.saturating_add(1);
                let original = block.u32(8).ok_or_else(bad_length)?;
                // A simple packet belongs to the section's first interface, and holds at
                // most its snapshot length.
                let Some(&(link_type, snapshot)) = interfaces.first() else {
                    out.problems.push(PacketProblem {
                        packet,
                        problem: PacketIssue::UnknownInterface { interface: 0 },
                    });
                    offset = offset.saturating_add(size);
                    continue;
                };
                let kept = original.min(if snapshot == 0 { u32::MAX } else { snapshot });
                let data = packet_data(block.bytes, 12, kept).ok_or_else(bad_length)?;
                read_packet(out, packet, link_type, data, original);
            }
            _ => {}
        }
        offset = offset.saturating_add(size);
    }
    Ok(())
}

/// The `len` bytes of packet data at `start` in a pcapng block's `body`, if they and the
/// padding to 4 bytes after them end inside it.
fn packet_data(body: &[u8], start: usize, len: u32) -> Option<&[u8]> {
    let len = usize::try_from(len).ok()?;
    start
        .checked_add(len.checked_next_multiple_of(4)?)
        .filter(|padded_end| *padded_end <= body.len())?;
    body.get(start..start.checked_add(len)?)
}

/// The link types read, as `LINKTYPE_` numbers them.
const ETHERNET: u32 = 1;
const NULL: u32 = 0;
const RAW: u32 = 101;
const LINUX_SLL: u32 = 113;
const IPV4: u32 = 228;
const IPV6: u32 = 229;
const LINUX_SLL2: u32 = 276;

/// Reads one packet of `link_type`, `original` bytes on the wire, into `out`.
fn read_packet(out: &mut Segments, packet: usize, link_type: u32, data: &[u8], original: u32) {
    let original = usize::try_from(original).unwrap_or(usize::MAX);
    let problem = |problem| PacketProblem { packet, problem };
    if data.len() < original {
        out.problems.push(problem(PacketIssue::CutShort {
            captured: data.len(),
            original,
        }));
        return;
    }
    match network_layer(link_type, data).and_then(|ip| ip.map(|ip| tcp(packet, ip)).transpose()) {
        Ok(Some(segment)) => out.segments.push(segment),
        Ok(None) => out.skipped = out.skipped.saturating_add(1),
        Err(issue) => out.problems.push(problem(issue)),
    }
}

/// An IP packet's addresses and its TCP segment's bytes.
#[derive(Clone, Copy)]
struct IpPacket<'a> {
    source: IpAddr,
    destination: IpAddr,
    tcp: &'a [u8],
}

/// The IP packet in a link-layer frame, if it is TCP over IP.
fn network_layer(link_type: u32, data: &[u8]) -> Result<Option<IpPacket<'_>>, PacketIssue> {
    let frame = Fields::big(data);
    let cut = |layer| PacketIssue::Truncated { layer };
    // The ethertype and the IP packet after it.
    let (ethertype, ip) = match link_type {
        ETHERNET => {
            let mut at = 12;
            let mut ethertype = frame.u16(at).ok_or(cut("ethernet"))?;
            // 802.1Q and 802.1ad tags, each four bytes before the real ethertype.
            while matches!(ethertype, 0x8100 | 0x88a8) {
                at = at.saturating_add(4);
                ethertype = frame.u16(at).ok_or(cut("ethernet"))?;
            }
            (
                ethertype,
                frame.rest(at.saturating_add(2)).ok_or(cut("ethernet"))?,
            )
        }
        LINUX_SLL => (
            frame.u16(14).ok_or(cut("linux cooked"))?,
            frame.rest(16).ok_or(cut("linux cooked"))?,
        ),
        LINUX_SLL2 => (
            frame.u16(0).ok_or(cut("linux cooked"))?,
            frame.rest(20).ok_or(cut("linux cooked"))?,
        ),
        NULL => {
            // The address family, in the capturing machine's byte order.
            let family = frame.array::<4>(0).ok_or(cut("loopback"))?;
            let family = u32::from_le_bytes(family).min(u32::from_be_bytes(family));
            let ethertype = match family {
                2 => 0x0800,
                24 | 28 | 30 => 0x86dd,
                _ => return Ok(None),
            };
            (ethertype, frame.rest(4).ok_or(cut("loopback"))?)
        }
        RAW | IPV4 | IPV6 => match data.first().map(|byte| byte >> 4) {
            Some(4) => (0x0800, data),
            Some(6) => (0x86dd, data),
            _ => return Ok(None),
        },
        other => return Err(PacketIssue::UnsupportedLinkType { link_type: other }),
    };
    match ethertype {
        0x0800 => ipv4(ip),
        0x86dd => ipv6(ip),
        _ => Ok(None),
    }
}

/// TCP's IP protocol number.
const TCP: u8 = 6;

fn ipv4(packet: &[u8]) -> Result<Option<IpPacket<'_>>, PacketIssue> {
    let fields = Fields::big(packet);
    let cut = PacketIssue::Truncated { layer: "ipv4" };
    let first = fields.u8(0).ok_or(cut)?;
    let bad = |field| PacketIssue::BadHeader {
        layer: "ipv4",
        field,
    };
    if first >> 4 != 4 {
        return Err(bad("version"));
    }
    let header = usize::from(first & 0x0f).saturating_mul(4);
    if header < 20 {
        return Err(bad("header length"));
    }
    let total = usize::from(fields.u16(2).ok_or(cut)?);
    if total < header {
        return Err(bad("total length"));
    }
    let fragment = fields.u16(6).ok_or(cut)?;
    if fields.u8(9).ok_or(cut)? != TCP {
        return Ok(None);
    }
    // More fragments, or a fragment offset.
    if fragment & 0x3fff != 0 {
        return Err(PacketIssue::Fragment);
    }
    let source = Ipv4Addr::from(fields.array::<4>(12).ok_or(cut)?);
    let destination = Ipv4Addr::from(fields.array::<4>(16).ok_or(cut)?);
    // The total length leaves out the padding a short Ethernet frame carries.
    let tcp = packet.get(header..total).ok_or(cut)?;
    Ok(Some(IpPacket {
        source: source.into(),
        destination: destination.into(),
        tcp,
    }))
}

fn ipv6(packet: &[u8]) -> Result<Option<IpPacket<'_>>, PacketIssue> {
    let fields = Fields::big(packet);
    let cut = PacketIssue::Truncated { layer: "ipv6" };
    if fields.u8(0).ok_or(cut)? >> 4 != 6 {
        return Err(PacketIssue::BadHeader {
            layer: "ipv6",
            field: "version",
        });
    }
    let payload = usize::from(fields.u16(4).ok_or(cut)?);
    // Extension headers aren't followed: a packet with one is skipped.
    if fields.u8(6).ok_or(cut)? != TCP {
        return Ok(None);
    }
    let source = Ipv6Addr::from(fields.array::<16>(8).ok_or(cut)?);
    let destination = Ipv6Addr::from(fields.array::<16>(24).ok_or(cut)?);
    let tcp = packet
        .get(40..40_usize.saturating_add(payload))
        .ok_or(cut)?;
    Ok(Some(IpPacket {
        source: source.into(),
        destination: destination.into(),
        tcp,
    }))
}

fn tcp(packet: usize, ip: IpPacket<'_>) -> Result<TcpSegment, PacketIssue> {
    let fields = Fields::big(ip.tcp);
    let cut = PacketIssue::Truncated { layer: "tcp" };
    let source_port = fields.u16(0).ok_or(cut)?;
    let destination_port = fields.u16(2).ok_or(cut)?;
    let sequence = fields.u32(4).ok_or(cut)?;
    let header = usize::from(fields.u8(12).ok_or(cut)? >> 4).saturating_mul(4);
    if header < 20 {
        return Err(PacketIssue::BadHeader {
            layer: "tcp",
            field: "data offset",
        });
    }
    let flags = fields.u8(13).ok_or(cut)?;
    let (syn, ack) = (flags & 0x02 != 0, flags & 0x10 != 0);
    let payload = ip.tcp.get(header..).ok_or(cut)?;
    Ok(TcpSegment {
        packet,
        source: SocketAddr::new(ip.source, source_port),
        destination: SocketAddr::new(ip.destination, destination_port),
        sequence,
        opens: syn && !ack,
        syn,
        payload: payload.to_vec(),
    })
}

#[cfg(test)]
mod tests {
    use std::net::SocketAddr;

    use super::{CaptureError, PacketIssue, PacketProblem, read_segments};

    const CLIENT: [u8; 4] = [10, 0, 0, 1];
    const SERVER: [u8; 4] = [10, 0, 0, 2];

    fn length(bytes: &[u8], plus: usize) -> u16 {
        u16::try_from(bytes.len().checked_add(plus).unwrap()).unwrap()
    }

    /// A TCP segment from port 40000 to 9092 with `flags` and `payload`.
    fn tcp(sequence: u32, flags: u8, payload: &[u8]) -> Vec<u8> {
        let mut segment = Vec::new();
        segment.extend_from_slice(&40000_u16.to_be_bytes());
        segment.extend_from_slice(&9092_u16.to_be_bytes());
        segment.extend_from_slice(&sequence.to_be_bytes());
        segment.extend_from_slice(&[0, 0, 0, 0, 0x50, flags, 0xff, 0xff, 0, 0, 0, 0]);
        segment.extend_from_slice(payload);
        segment
    }

    /// An IPv4 packet carrying `protocol`'s `payload`, with the fragment field `fragment`.
    fn ipv4(protocol: u8, fragment: u16, payload: &[u8]) -> Vec<u8> {
        let mut packet = vec![0x45, 0];
        packet.extend_from_slice(&length(payload, 20).to_be_bytes());
        packet.extend_from_slice(&[0, 0]);
        packet.extend_from_slice(&fragment.to_be_bytes());
        packet.extend_from_slice(&[64, protocol, 0, 0]);
        packet.extend_from_slice(&CLIENT);
        packet.extend_from_slice(&SERVER);
        packet.extend_from_slice(payload);
        packet
    }

    fn ipv6(payload: &[u8]) -> Vec<u8> {
        let mut packet = vec![0x60, 0, 0, 0];
        packet.extend_from_slice(&length(payload, 0).to_be_bytes());
        packet.extend_from_slice(&[6, 64]);
        packet.extend_from_slice(&[0xfe; 16]);
        packet.extend_from_slice(&[0xfd; 16]);
        packet.extend_from_slice(payload);
        packet
    }

    /// An Ethernet frame with an 802.1Q tag, carrying `packet`.
    fn tagged_ethernet(packet: &[u8]) -> Vec<u8> {
        let mut frame = vec![0; 12];
        frame.extend_from_slice(&[0x81, 0x00, 0, 5, 0x08, 0x00]);
        frame.extend_from_slice(packet);
        // Short frames are padded, past the IP packet's own length.
        frame.extend_from_slice(&[0; 6]);
        frame
    }

    /// A pcap file of `packets`, each `(captured, original length)`.
    fn pcap(little_endian: bool, link_type: u32, packets: &[(Vec<u8>, u32)]) -> Vec<u8> {
        let word = |value: u32| {
            if little_endian {
                value.to_le_bytes()
            } else {
                value.to_be_bytes()
            }
        };
        let mut file = word(0xa1b2_c3d4).to_vec();
        file.extend_from_slice(&[0; 16]);
        file.extend_from_slice(&word(link_type));
        for (data, original) in packets {
            file.extend_from_slice(&[0; 8]);
            file.extend_from_slice(&word(u32::try_from(data.len()).unwrap()));
            file.extend_from_slice(&word(*original));
            file.extend_from_slice(data);
        }
        file
    }

    fn whole(packet: Vec<u8>) -> (Vec<u8>, u32) {
        let original = u32::try_from(packet.len()).unwrap();
        (packet, original)
    }

    /// A big-endian pcapng file: one interface of `link_type`, and `packets` in simple
    /// packet blocks.
    fn pcapng(link_type: u16, packets: &[Vec<u8>]) -> Vec<u8> {
        let block = |kind: u32, body: &[u8]| {
            let mut padded = body.to_vec();
            padded.resize(body.len().next_multiple_of(4), 0);
            let total = u32::try_from(padded.len().checked_add(12).unwrap()).unwrap();
            let mut block = kind.to_be_bytes().to_vec();
            block.extend_from_slice(&total.to_be_bytes());
            block.extend_from_slice(&padded);
            block.extend_from_slice(&total.to_be_bytes());
            block
        };
        let mut file = block(
            0x0a0d_0d0a,
            &[
                0x1a, 0x2b, 0x3c, 0x4d, 0, 1, 0, 0, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            ],
        );
        let mut interface = link_type.to_be_bytes().to_vec();
        interface.extend_from_slice(&[0, 0, 0, 0, 0, 0]);
        file.extend(block(1, &interface));
        for packet in packets {
            let mut body = u32::try_from(packet.len()).unwrap().to_be_bytes().to_vec();
            body.extend_from_slice(packet);
            file.extend(block(3, &body));
        }
        file
    }

    #[test]
    fn a_big_endian_pcap_of_tagged_ethernet_reads() {
        let packet = tagged_ethernet(&ipv4(6, 0x4000, &tcp(7, 0x18, b"kafka")));
        let segments = read_segments(&pcap(false, 1, &[whole(packet)])).unwrap();
        assert_eq!(segments.problems, []);
        let [segment] = segments.segments.as_slice() else {
            panic!("one segment: {segments:?}");
        };
        assert_eq!(segment.source, SocketAddr::from((CLIENT, 40000)));
        assert_eq!(segment.destination, SocketAddr::from((SERVER, 9092)));
        assert_eq!(
            (segment.sequence, segment.payload.as_slice()),
            (7, &b"kafka"[..])
        );
        assert!(!segment.syn && !segment.opens);
    }

    #[test]
    fn a_big_endian_pcapng_of_simple_blocks_reads() {
        let file = pcapng(
            101,
            &[ipv6(&tcp(1, 0x02, b"")), ipv4(6, 0, &tcp(9, 0x18, b"x"))],
        );
        let segments = read_segments(&file).unwrap();
        assert_eq!(segments.problems, []);
        let opens: Vec<bool> = segments
            .segments
            .iter()
            .map(|segment| segment.opens)
            .collect();
        assert_eq!(opens, [true, false]);
        assert!(segments.segments[0].source.is_ipv6());
    }

    #[test]
    fn loopback_frames_read_in_either_byte_order() {
        let mut little = 2_u32.to_le_bytes().to_vec();
        little.extend(ipv4(6, 0, &tcp(1, 0x18, b"a")));
        let mut big = 30_u32.to_be_bytes().to_vec();
        big.extend(ipv6(&tcp(2, 0x18, b"b")));
        let segments = read_segments(&pcap(true, 0, &[whole(little), whole(big)])).unwrap();
        assert_eq!(segments.segments.len(), 2, "{segments:?}");
    }

    #[test]
    fn linux_cooked_v1_frames_read() {
        let mut frame = vec![0; 14];
        frame.extend_from_slice(&[0x86, 0xdd]);
        frame.extend(ipv6(&tcp(3, 0x18, b"cooked")));
        let segments = read_segments(&pcap(true, 113, &[whole(frame)])).unwrap();
        assert_eq!(segments.problems, []);
        let [segment] = segments.segments.as_slice() else {
            panic!("one segment: {segments:?}");
        };
        assert_eq!(segment.payload, b"cooked");
    }

    #[test]
    fn unreadable_packets_are_reported_and_others_skipped() {
        let packets = [
            whole(ipv4(6, 0, &tcp(1, 0x18, b"ok"))),
            (ipv4(6, 0, &tcp(1, 0x18, b"cut")), 1500),
            whole(ipv4(6, 0x2000, &tcp(1, 0x18, b"fragment"))),
            whole(ipv4(17, 0, b"udp")),
            whole(ipv4(6, 0, &[0; 10])),
        ];
        let segments = read_segments(&pcap(true, 101, &packets)).unwrap();
        assert_eq!(segments.segments.len(), 1);
        assert_eq!(segments.skipped, 1);
        let problem = |packet, problem| PacketProblem { packet, problem };
        assert_eq!(
            segments.problems,
            [
                problem(
                    2,
                    PacketIssue::CutShort {
                        captured: 43,
                        original: 1500
                    }
                ),
                problem(3, PacketIssue::Fragment),
                problem(5, PacketIssue::Truncated { layer: "tcp" }),
            ]
        );
        let other = read_segments(&pcap(true, 147, &[whole(vec![1])])).unwrap();
        assert_eq!(
            other.problems,
            [problem(
                1,
                PacketIssue::UnsupportedLinkType { link_type: 147 }
            )]
        );
    }

    #[test]
    fn impossible_header_lengths_are_reported() {
        let mut short_ip = ipv4(6, 0, &tcp(1, 0x18, b"x"));
        short_ip[0] = 0x44;
        let mut short_tcp = tcp(1, 0x18, b"x");
        short_tcp[12] = 0x00;
        let mut wrong_version = ipv4(6, 0, &tcp(1, 0x18, b"x"));
        wrong_version[0] = 0x65;
        // Ethernet, whose ethertype names IPv4 whatever the packet's version says.
        let packets = [short_ip, ipv4(6, 0, &short_tcp), wrong_version]
            .map(|packet| whole(tagged_ethernet(&packet)));
        let segments = read_segments(&pcap(true, 1, &packets)).unwrap();
        let issues: Vec<PacketIssue> = segments
            .problems
            .iter()
            .map(|problem| problem.problem)
            .collect();
        let bad = |layer, field| PacketIssue::BadHeader { layer, field };
        assert_eq!(
            issues,
            [
                bad("ipv4", "header length"),
                bad("tcp", "data offset"),
                bad("ipv4", "version")
            ]
        );
    }

    #[test]
    fn the_high_bits_of_a_pcap_link_type_are_not_the_link_type() {
        // Ethernet frames ending with a 4-byte frame check sequence.
        let mut frame = tagged_ethernet(&ipv4(6, 0, &tcp(1, 0x18, b"fcs")));
        frame.extend_from_slice(&[0xde, 0xad, 0xbe, 0xef]);
        let segments = read_segments(&pcap(true, 0x2400_0001, &[whole(frame)])).unwrap();
        assert_eq!(segments.problems, []);
        assert_eq!(segments.segments[0].payload, b"fcs");
    }

    #[test]
    fn pcapng_blocks_must_hold_what_they_declare() {
        let file = pcapng(101, &[ipv4(6, 0, &tcp(1, 0x18, b"x"))]);
        let packet_block = 28 + 20;
        // The trailing copy of the packet block's length, changed.
        let mut unequal = file.clone();
        let last = unequal.len() - 1;
        unequal[last] ^= 4;
        assert_eq!(
            read_segments(&unequal),
            Err(CaptureError::BadBlockLength {
                offset: packet_block,
                length: 60
            })
        );
        // A simple packet whose original length runs past its block.
        let mut long = file;
        long[packet_block + 11] = 0xff;
        assert!(matches!(
            read_segments(&long),
            Err(CaptureError::BadBlockLength { offset, .. }) if offset == packet_block
        ));
    }

    #[test]
    fn a_file_that_isnt_a_capture_is_an_error() {
        assert_eq!(
            read_segments(b"hello"),
            Err(CaptureError::UnknownFormat {
                start: b"hello".to_vec()
            })
        );
        let mut file = pcap(true, 1, &[whole(vec![0; 20])]);
        file.truncate(30);
        assert_eq!(
            read_segments(&file),
            Err(CaptureError::Truncated {
                what: "a pcap packet header",
                offset: 24
            })
        );
        let mut bad = pcapng(1, &[]);
        bad.extend_from_slice(&[0, 0, 0, 3, 0, 0, 0, 5]);
        assert!(matches!(
            read_segments(&bad),
            Err(CaptureError::BadBlockLength { length: 5, .. })
        ));
    }
}
