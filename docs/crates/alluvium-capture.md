# `alluvium-capture`: deep dive

## Responsibility and layer

The crate reads Kafka traffic captured outside Alluvium back into Kafka frames, for debugging with `alluvium dump frame` (PROTO-F13). It reads two kinds of input:
- **Dumps of one frame's bytes**: raw, hex and `xxd` ([`input`](../../crates/alluvium-capture/src/input.rs)).
- **Packet captures**: pcap and pcapng files, as tcpdump and Wireshark write them, in three steps:
  1. [`packets`](../../crates/alluvium-capture/src/packets.rs) reads the file into TCP segments;
  2. [`streams`](../../crates/alluvium-capture/src/streams.rs) puts each connection's two directions in order and splits them into frames;
  3. [`exchanges`](../../crates/alluvium-capture/src/exchanges.rs) pairs each request with its response by correlation id, so the response decodes at its request's API and version without the user naming them.

  `read_capture` in [`lib.rs`](../../crates/alluvium-capture/src/lib.rs) does steps 1 and 2.

Decoding a frame is `alluvium-protocol`'s job (`dump_request_frame`, `dump_response_frame`), which `exchanges` calls. Reading files and printing results is the CLI's.

Layer 1 in the [code map](../architecture.md#4-code-map): one concept, pure functions over bytes in memory, no I/O. It depends on `alluvium-protocol`.

## Kafka counterpart

None. Kafka leaves captures to Wireshark's Kafka dissector. The formats are those of the tools people copy frames from or capture with:
- Wireshark's "Copy as Hex Stream";
- `xxd` and `xxd -p`;
- pcap and pcapng files with these link types:
  - Ethernet, with VLAN tags;
  - Linux cooked v1 and v2, as `tcpdump -i any` writes them;
  - BSD loopback;
  - raw IP.

## Reading packet captures

- **Packets.**
  - IPv4 and IPv6 carry TCP. IPv6 extension headers are not followed.
  - Any other packet is counted as skipped, not reported as a problem.
  - IP fragments are reported, not reassembled. TCP sets don't-fragment, so they are rare.
- **Streams.**
  - A connection is the pair of its endpoints.
  - The client is the side that sent the first SYN. A capture that starts mid-connection takes the higher port as the client, and says so (`roles_from_handshake` is false).
  - Each direction is put in order by sequence number, read relative to the next byte expected. Retransmissions add only the bytes not yet seen, even one that starts before the first byte captured. Segments that arrive early wait for the bytes before them. Sequence numbers may wrap, and a stream may outgrow 4 GiB.
  - A byte range that was never captured ends that direction's stream at the gap, because the frames after it can't be found.
- **Frames.** A direction's bytes are split at each 4-byte size prefix. A last frame that the capture cut short is kept, so its header can still be read.
- **Pairing.** Kafka's clients give each request a new correlation id, so a response pairs with the request whose id it carries. A response that matches no request is kept in `unmatched`, with its id. A request without a response is one the capture ended before, or a `Produce` with `acks=0`, which Kafka doesn't answer.

## State machines

None. Every function reads its whole input at once.

## Invariants

- **Input is untrusted, and reading never panics.**
  - The crate denies `clippy::indexing_slicing` and `clippy::arithmetic_side_effects`.
  - `input::tests::no_input_panics` feeds arbitrary bytes to every dump format.
  - The `capture_input` and `capture_pcap` fuzz targets do the same for dumps and for packet captures.
- **A malformed dump is an `InputError` naming its line and column**, counted from 1, columns in characters. `malformed_dumps_name_the_line_and_column` covers each problem.
- **What a dump holds reads back the same.** `written_dumps_read_back` writes arbitrary bytes as hex and as `xxd` would, then reads them back.
- **A damaged capture reports what it lost beside what it could read.** Only a file that isn't pcap or pcapng, or whose structure is broken, is an error. A bad packet is a `PacketProblem`, and a bad stream is a `StreamProblem`.
- **Memory is linear in the capture.** The file is already in memory, and the reader copies a payload byte at most three times: into its segment, into a segment waiting its turn, and into its stream.
- **A direction's frames tile its stream.** Each frame starts where the one before it ends; the `capture_pcap` fuzz target checks this.
- **Records are never printed.** Frame dumps write each record batch as its size and CRC-32C only.

## Events, metrics and flight recorder

None. The crate is a library for offline tools, with no state to observe.

## Errors

`input::InputError` holds the line, the column and an `InputProblem`:
- `NotText`: a text format given bytes that aren't UTF-8;
- `NotHex`: a character that isn't a hex digit;
- `OddDigits`: a hex digit without its pair;
- `NoOffset`: an `xxd` line without its `offset:`;
- `WrongOffset`: an `xxd` line whose offset isn't the count of the bytes before it.

`packets::CaptureError` ends the read:
- `UnknownFormat`: the file is neither pcap nor pcapng;
- `Truncated`: the file ends inside a header or a block;
- `BadBlockLength`: a pcapng block's length is impossible: too short for its fields or its packet, or unequal to the copy that ends the block.

`packets::PacketProblem` names a packet, counted from 1, and its `PacketIssue`:
- `CutShort`: the snapshot length cut the packet;
- `Truncated`: one of its headers is cut;
- `BadHeader`: an IP or TCP header with an impossible version or length;
- `Fragment`: an IP fragment;
- `UnsupportedLinkType`;
- `UnknownInterface`: a pcapng packet naming an interface the section doesn't describe.

`streams::StreamProblem` names a direction:
- `MidStream`: the capture started after its first byte;
- `OutOfOrder`: segments arrived out of order, which is not an error, but explains a slow connection;
- `Gap`: bytes that were never captured;
- `IncompleteFrame`: the capture ends inside a frame;
- `IncompleteSize`: the capture ends inside a frame's size prefix;
- `NegativeSize`: a size prefix below zero, so the stream is not Kafka.

The errors never reach a Kafka client, so there is no error-code mapping.

## Tests

- **Unit and property tests:** `cargo nextest run -p alluvium-capture`.
  - Each dump format is tested on a fixture of the same frame bytes.
  - Packets are tested in each file byte order and in every link type but Ethernet without a tag and Linux cooked v2, which the real captures below cover.
  - Streams are tested with handshakes, wrapping sequence numbers, overlapping retransmissions, gaps and cut frames.
  - Pairing is tested with responses out of order and unmatched.
- **End to end:** [`tests/captures.rs`](../../crates/alluvium-capture/tests/captures.rs) reads two real captures of Kafka 4.4.0's console producer creating a topic and producing to it, one as Ethernet pcap and one as Linux cooked pcapng. Every request and response decodes, and both files give the same session. [`record.sh`](../../crates/alluvium-capture/tests/captures/record.sh) re-records them with Docker.
- **Fuzz:**
  - `cargo xtask fuzz capture_input` parses arbitrary input in each dump format, and checks that a hex dump of the bytes it parsed reads back the same.
  - `cargo xtask fuzz capture_pcap` reads arbitrary files as captures and pairs their exchanges. Its seeds are the two real captures.

## Debugging

A dump that doesn't parse reports the line and column where it goes wrong. Two common causes:
- **An `xxd` dump with lines missing**, which fails as `WrongOffset` on the line after the gap.
- **A detected format that is wrong.** `detect` takes text of hex digits and spaces as hex, and anything else that isn't an `xxd` dump as raw bytes. Pass the format explicitly to override it.

A capture that decodes badly usually reports why. Common causes:
- **`CutShort` packets.** The capture's snapshot length was too small; capture with `tcpdump -s 0`.
- **A `Gap`.** The capture dropped packets; tcpdump prints how many "dropped by kernel". Capture to a file, not a terminal, and filter to the Kafka port.
- **`MidStream` with the roles swapped.** Without the handshake, the higher port is taken as the client. That's wrong for a broker listening on a high port; start the capture before the client connects.
- **TLS.** Encrypted traffic doesn't decode; it typically fails as `NegativeSize` or an implausible size. Capture on a plaintext listener.
