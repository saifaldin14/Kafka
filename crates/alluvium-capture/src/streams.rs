//! TCP streams from a capture's segments, and the Kafka frames in them (PROTO-0144).
//!
//! Each connection has two directions, each its own byte stream. The bytes are put in
//! order by sequence number:
//! - a retransmitted segment adds only the bytes not yet seen, even if it starts before
//!   the first byte captured;
//! - a segment that arrives early waits for the bytes before it; of two at the same
//!   place, the longer is kept.
//!
//! Sequence numbers are read relative to the next byte expected, so they may wrap, and a
//! stream may outgrow 4 GiB. Memory is linear in the capture: a payload byte is copied
//! at most twice, into a segment waiting its turn and into its stream.
//!
//! Each direction is then split into frames at their `int32` size prefixes: the
//! client's into requests and the server's into responses. What can't be read is a
//! [`StreamProblem`]:
//! - bytes never captured end the direction at the gap;
//! - a capture that began mid-connection may start inside a frame;
//! - a frame the capture holds only part of is kept as far as it goes, and a size prefix
//!   it holds only part of is reported.
//!
//! Client and server: the side that sent SYN is the client. In a capture that began
//! after the handshake, the side with the higher port is taken to be the client, since
//! clients use ephemeral ports and servers well-known ones.

use std::collections::BTreeMap;
use std::fmt;
use std::net::SocketAddr;

use bytes::Bytes;

use crate::packets::TcpSegment;

/// One Kafka frame from a stream: its bytes, size prefix included, and where it starts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StreamFrame {
    /// Its offset in its direction's stream.
    pub offset: u64,
    /// The packet its first byte arrived in, from 1.
    pub packet: usize,
    /// The frame, from its size prefix to its last byte, or as far as the capture has it.
    pub bytes: Bytes,
}

/// One TCP connection's frames.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Connection {
    /// The client's address.
    pub client: SocketAddr,
    /// The server's address.
    pub server: SocketAddr,
    /// Whether client and server were told apart by the SYN, rather than guessed from the
    /// ports.
    pub roles_from_handshake: bool,
    /// The client's frames: requests.
    pub requests: Vec<StreamFrame>,
    /// The server's frames: responses.
    pub responses: Vec<StreamFrame>,
    /// What couldn't be read.
    pub problems: Vec<StreamProblem>,
}

/// A direction of a connection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    /// From client to server.
    Requests,
    /// From server to client.
    Responses,
}

impl fmt::Display for Direction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Direction::Requests => "requests",
            Direction::Responses => "responses",
        })
    }
}

/// What couldn't be read from a direction of a connection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum StreamProblem {
    /// The capture began after the connection opened, so the first frame may be cut.
    #[error("the {direction} stream was captured from mid-connection")]
    MidStream {
        /// The direction.
        direction: Direction,
    },
    /// Segments arrived out of order, and were put in order.
    #[error("{count} {direction} segment(s) arrived out of order")]
    OutOfOrder {
        /// The direction.
        direction: Direction,
        /// The segments that arrived ahead of bytes before them.
        count: usize,
    },
    /// Bytes were never captured; the direction isn't read past them.
    #[error("{missing} {direction} byte(s) at stream offset {offset} were never captured")]
    Gap {
        /// The direction.
        direction: Direction,
        /// Where the missing bytes start.
        offset: u64,
        /// How many bytes are missing before the next captured ones.
        missing: u64,
    },
    /// The last frame of a direction ends before its size says.
    #[error(
        "the {direction} frame at stream offset {offset} has {present} of its {declared} bytes"
    )]
    IncompleteFrame {
        /// The direction.
        direction: Direction,
        /// Where the frame starts.
        offset: u64,
        /// The bytes its size prefix declares, after the prefix.
        declared: u64,
        /// The bytes of it captured, after the prefix.
        present: u64,
    },
    /// The stream ends inside a frame's size prefix.
    #[error(
        "the {direction} stream ends at offset {offset} after {present} of a size prefix's 4 bytes"
    )]
    IncompleteSize {
        /// The direction.
        direction: Direction,
        /// The offset of the size prefix in its stream.
        offset: u64,
        /// The bytes of it captured.
        present: usize,
    },
    /// A size prefix is negative, so the stream can't be split past it.
    #[error("the {direction} size prefix at stream offset {offset} is {size}")]
    NegativeSize {
        /// The direction.
        direction: Direction,
        /// Where the prefix is.
        offset: u64,
        /// The size read.
        size: i32,
    },
}

/// One direction's bytes, put in order.
#[derive(Default)]
struct Stream {
    /// The sequence number of the stream's first byte.
    start: Option<u32>,
    /// Whether the stream started at its SYN.
    from_syn: bool,
    /// The bytes in order so far, from the start.
    bytes: Vec<u8>,
    /// The packet each run of bytes arrived in: (offset, packet).
    packets: Vec<(u64, usize)>,
    /// Segments ahead of the bytes in order, by offset, each with its packet.
    early: BTreeMap<u64, (Vec<u8>, usize)>,
    out_of_order: usize,
}

impl Stream {
    fn add(&mut self, segment: &TcpSegment) {
        let first_byte = segment.sequence.wrapping_add(u32::from(segment.syn));
        let start = *self.start.get_or_insert_with(|| {
            self.from_syn = segment.syn;
            first_byte
        });
        if segment.payload.is_empty() {
            return;
        }
        // The distance from the next byte expected, signed, so that sequence numbers wrap
        // and a stream can outgrow 4 GiB, as long as each segment is within 2 GiB of it.
        let next = start.wrapping_add(low_32_bits(self.len()));
        let ahead = first_byte.wrapping_sub(next).cast_signed();
        if let Some(ahead) = u64::try_from(ahead).ok().filter(|ahead| *ahead > 0) {
            self.out_of_order = self.out_of_order.saturating_add(1);
            let early = self
                .early
                .entry(self.len().saturating_add(ahead))
                .or_insert_with(|| (Vec::new(), segment.packet));
            // A retransmission can carry more than the segment it repeats.
            if segment.payload.len() > early.0.len() {
                *early = (segment.payload.clone(), segment.packet);
            }
            return;
        }
        let behind = usize::try_from(ahead.unsigned_abs()).unwrap_or(usize::MAX);
        self.append(behind, &segment.payload, segment.packet);
        loop {
            let len = self.len();
            let Some(entry) = self.early.first_entry() else {
                break;
            };
            let Some(behind) = len.checked_sub(*entry.key()) else {
                break;
            };
            let (payload, packet) = entry.remove();
            self.append(
                usize::try_from(behind).unwrap_or(usize::MAX),
                &payload,
                packet,
            );
        }
    }

    fn len(&self) -> u64 {
        u64::try_from(self.bytes.len()).unwrap_or(u64::MAX)
    }

    /// Appends `payload`, whose first `seen` bytes are already in order.
    fn append(&mut self, seen: usize, payload: &[u8], packet: usize) {
        if let Some(new) = payload.get(seen..).filter(|new| !new.is_empty()) {
            self.packets.push((self.len(), packet));
            self.bytes.extend_from_slice(new);
        }
    }

    /// The packet the byte at `offset` arrived in.
    fn packet_at(&self, offset: u64) -> usize {
        let index = self.packets.partition_point(|(start, _)| *start <= offset);
        index
            .checked_sub(1)
            .and_then(|index| self.packets.get(index))
            .map_or(0, |(_, packet)| *packet)
    }

    /// The stream's frames, and what couldn't be read.
    fn frames(
        mut self,
        direction: Direction,
        problems: &mut Vec<StreamProblem>,
    ) -> Vec<StreamFrame> {
        if self.start.is_some() && !self.from_syn {
            problems.push(StreamProblem::MidStream { direction });
        }
        if self.out_of_order > 0 {
            problems.push(StreamProblem::OutOfOrder {
                direction,
                count: self.out_of_order,
            });
        }
        if let Some((&next, _)) = self.early.first_key_value() {
            problems.push(StreamProblem::Gap {
                direction,
                offset: self.len(),
                missing: next.saturating_sub(self.len()),
            });
        }
        let bytes = Bytes::from(std::mem::take(&mut self.bytes));
        let mut frames = Vec::new();
        let mut at = 0_usize;
        while let Some(prefix) = bytes.get(at..).and_then(<[u8]>::first_chunk::<4>) {
            let offset = u64::try_from(at).unwrap_or(u64::MAX);
            let size = i32::from_be_bytes(*prefix);
            let Ok(declared) = usize::try_from(size) else {
                problems.push(StreamProblem::NegativeSize {
                    direction,
                    offset,
                    size,
                });
                break;
            };
            let end = at.saturating_add(4).saturating_add(declared);
            let present = bytes.len().min(end);
            if present < end {
                problems.push(StreamProblem::IncompleteFrame {
                    direction,
                    offset,
                    declared: u64::try_from(declared).unwrap_or(u64::MAX),
                    present: u64::try_from(present.saturating_sub(at).saturating_sub(4))
                        .unwrap_or(u64::MAX),
                });
            }
            frames.push(StreamFrame {
                offset,
                packet: self.packet_at(offset),
                bytes: bytes.slice(at..present),
            });
            at = present;
        }
        let rest = bytes.len().saturating_sub(at);
        if (1..4).contains(&rest) {
            problems.push(StreamProblem::IncompleteSize {
                direction,
                offset: u64::try_from(at).unwrap_or(u64::MAX),
                present: rest,
            });
        }
        frames
    }
}

/// The low 32 bits of `value`, as sequence numbers count bytes.
fn low_32_bits(value: u64) -> u32 {
    u32::try_from(value & u64::from(u32::MAX)).unwrap_or(0)
}

/// The connections in `segments`, in the order their first segments were captured.
pub fn connections(segments: &[TcpSegment]) -> Vec<Connection> {
    // Each connection's two streams, keyed by its endpoints in a fixed order, with its
    // client once known and the position of its first segment.
    struct Building {
        first: usize,
        client: Option<SocketAddr>,
        streams: BTreeMap<SocketAddr, Stream>,
    }
    let mut building: BTreeMap<(SocketAddr, SocketAddr), Building> = BTreeMap::new();
    for (position, segment) in segments.iter().enumerate() {
        let key = if segment.source <= segment.destination {
            (segment.source, segment.destination)
        } else {
            (segment.destination, segment.source)
        };
        let connection = building.entry(key).or_insert_with(|| Building {
            first: position,
            client: None,
            streams: BTreeMap::new(),
        });
        if segment.opens {
            connection.client.get_or_insert(segment.source);
        }
        connection
            .streams
            .entry(segment.source)
            .or_default()
            .add(segment);
    }
    let mut built: Vec<(usize, Connection)> = building
        .into_iter()
        .map(|((low, high), mut connection)| {
            let (client, roles_from_handshake) = match connection.client {
                Some(client) => (client, true),
                None if low.port() > high.port() => (low, false),
                None => (high, false),
            };
            let server = if client == low { high } else { low };
            let mut problems = Vec::new();
            let mut frames = |side: SocketAddr, direction| {
                connection
                    .streams
                    .remove(&side)
                    .map(|stream| stream.frames(direction, &mut problems))
                    .unwrap_or_default()
            };
            let requests = frames(client, Direction::Requests);
            let responses = frames(server, Direction::Responses);
            (
                connection.first,
                Connection {
                    client,
                    server,
                    roles_from_handshake,
                    requests,
                    responses,
                    problems,
                },
            )
        })
        .collect();
    built.sort_by_key(|(first, _)| *first);
    built
        .into_iter()
        .map(|(_, connection)| connection)
        .collect()
}

#[cfg(test)]
mod tests {
    use std::net::SocketAddr;

    use super::{Direction, StreamProblem, connections};
    use crate::packets::TcpSegment;

    fn address(port: u16) -> SocketAddr {
        SocketAddr::from(([10, 0, 0, 1], port))
    }

    /// A segment from `from` to `to` at `sequence`.
    fn segment(from: u16, to: u16, sequence: u32, payload: &[u8]) -> TcpSegment {
        TcpSegment {
            packet: 0,
            source: address(from),
            destination: address(to),
            sequence,
            opens: false,
            syn: false,
            payload: payload.to_vec(),
        }
    }

    fn syn(from: u16, to: u16, sequence: u32, opens: bool) -> TcpSegment {
        TcpSegment {
            opens,
            syn: true,
            ..segment(from, to, sequence, b"")
        }
    }

    /// A frame of `body`, with its size prefix.
    fn frame(body: &[u8]) -> Vec<u8> {
        let mut frame = u32::try_from(body.len()).unwrap().to_be_bytes().to_vec();
        frame.extend_from_slice(body);
        frame
    }

    fn bodies(frames: &[super::StreamFrame]) -> Vec<Vec<u8>> {
        frames
            .iter()
            .map(|frame| frame.bytes.get(4..).unwrap().to_vec())
            .collect()
    }

    #[test]
    fn segments_are_put_in_order_and_split_into_frames() {
        let mut stream = frame(b"first");
        stream.extend(frame(b"second"));
        let (head, tail) = stream.split_at(7);
        // The handshake, then the second half before the first, then the first again.
        let segments = [
            syn(5000, 9092, u32::MAX, true),
            syn(9092, 5000, 77, false),
            segment(5000, 9092, 7, tail),
            segment(5000, 9092, 0, head),
            segment(5000, 9092, 0, &stream),
            segment(9092, 5000, 78, &frame(b"reply")),
        ];
        let [connection] = connections(&segments).try_into().unwrap();
        assert_eq!(
            (connection.client, connection.server),
            (address(5000), address(9092))
        );
        assert!(connection.roles_from_handshake);
        assert_eq!(
            bodies(&connection.requests),
            [b"first".to_vec(), b"second".to_vec()]
        );
        assert_eq!(bodies(&connection.responses), [b"reply".to_vec()]);
        assert_eq!(connection.requests[1].offset, 9);
        assert_eq!(
            connection.problems,
            [StreamProblem::OutOfOrder {
                direction: Direction::Requests,
                count: 1
            }]
        );
    }

    #[test]
    fn overlapping_retransmissions_keep_every_byte() {
        let mut stream = frame(b"abc");
        stream.extend(frame(b"de"));
        let segments = [
            syn(5000, 9092, u32::MAX - 1, true),
            // Two segments at offset 4 arrive early, the second carrying more.
            segment(5000, 9092, 3, stream.get(4..5).unwrap()),
            segment(5000, 9092, 3, stream.get(4..).unwrap()),
            segment(5000, 9092, u32::MAX, stream.get(..4).unwrap()),
        ];
        let [connection] = connections(&segments).try_into().unwrap();
        assert_eq!(
            bodies(&connection.requests),
            [b"abc".to_vec(), b"de".to_vec()]
        );
        assert_eq!(
            connection.problems,
            [StreamProblem::OutOfOrder {
                direction: Direction::Requests,
                count: 2
            }]
        );
    }

    #[test]
    fn a_retransmission_from_before_the_capture_adds_what_it_carries_beyond() {
        let mut stream = frame(b"abc");
        stream.extend(frame(b"de"));
        let mut retransmission = vec![0xee, 0xee];
        retransmission.extend_from_slice(&stream);
        let segments = [
            segment(40000, 9092, 1000, stream.get(..4).unwrap()),
            segment(40000, 9092, 998, &retransmission),
        ];
        let [connection] = connections(&segments).try_into().unwrap();
        assert_eq!(
            bodies(&connection.requests),
            [b"abc".to_vec(), b"de".to_vec()]
        );
        assert_eq!(
            connection.problems,
            [StreamProblem::MidStream {
                direction: Direction::Requests
            }]
        );
    }

    #[test]
    fn a_gap_ends_the_stream_and_a_mid_stream_capture_guesses_roles() {
        let mut stream = frame(b"one");
        stream.extend(frame(b"two"));
        let segments = [
            segment(9092, 40000, 500, &frame(b"reply")),
            segment(40000, 9092, 1000, stream.get(..7).unwrap()),
            segment(40000, 9092, 1010, stream.get(10..).unwrap()),
        ];
        let [connection] = connections(&segments).try_into().unwrap();
        assert_eq!(connection.client, address(40000));
        assert!(!connection.roles_from_handshake);
        assert_eq!(bodies(&connection.requests), [b"one".to_vec()]);
        assert_eq!(
            connection.problems,
            [
                StreamProblem::MidStream {
                    direction: Direction::Requests
                },
                StreamProblem::OutOfOrder {
                    direction: Direction::Requests,
                    count: 1
                },
                StreamProblem::Gap {
                    direction: Direction::Requests,
                    offset: 7,
                    missing: 3
                },
                StreamProblem::MidStream {
                    direction: Direction::Responses
                },
            ]
        );
    }

    #[test]
    fn cut_frames_cut_size_prefixes_and_negative_sizes_are_reported() {
        let mut requests = frame(b"whole");
        requests.extend_from_slice(&[0, 0, 0, 9, 1, 2]);
        let mut cut_prefix = frame(b"x");
        cut_prefix.extend_from_slice(&[0, 0]);
        let segments = [
            segment(40000, 9092, 0, &requests),
            segment(9092, 40000, 0, &[0xff, 0xff, 0xff, 0xff, 0]),
        ];
        let [connection] = connections(&segments).try_into().unwrap();
        assert_eq!(connection.requests.len(), 2);
        assert_eq!(connection.requests[1].bytes.as_ref(), [0, 0, 0, 9, 1, 2]);
        assert_eq!(
            connection.responses.len(),
            0,
            "the response frame never starts"
        );
        assert!(
            connection
                .problems
                .contains(&StreamProblem::IncompleteFrame {
                    direction: Direction::Requests,
                    offset: 9,
                    declared: 9,
                    present: 2
                })
        );
        assert!(connection.problems.contains(&StreamProblem::NegativeSize {
            direction: Direction::Responses,
            offset: 0,
            size: -1
        }));
        let [cut] = connections(&[segment(40000, 9092, 0, &cut_prefix)])
            .try_into()
            .unwrap();
        assert_eq!(bodies(&cut.requests), [b"x".to_vec()]);
        assert!(cut.problems.contains(&StreamProblem::IncompleteSize {
            direction: Direction::Requests,
            offset: 5,
            present: 2
        }));
    }
}
