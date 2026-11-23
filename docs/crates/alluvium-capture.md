# `alluvium-capture`: deep dive

## Responsibility and layer

The crate reads Kafka traffic captured outside Alluvium back into Kafka frames, for debugging with `alluvium dump frame` (PROTO-F13). So far that means dumps of a frame's bytes: raw, hex and `xxd` ([`input`](../../crates/alluvium-capture/src/input.rs)). Decoding a frame once it has the bytes is `alluvium-protocol`'s job (`dump_request_frame`). Reading files and printing results is the CLI's.

Layer 1 in the [code map](../architecture.md#4-code-map): one concept, pure functions over bytes in memory, no I/O. It depends on no other Alluvium crate yet.

## Kafka counterpart

None. Kafka leaves captures to Wireshark's Kafka dissector. The formats are those of the tools people copy frames from:
- Wireshark's "Copy as Hex Stream";
- `xxd` and `xxd -p`.

## State machines

None. Every function reads its whole input at once.

## Invariants

- **Input is untrusted, and parsing never panics.**
  - The crate denies `clippy::indexing_slicing` and `clippy::arithmetic_side_effects`.
  - `input::tests::no_input_panics` feeds arbitrary bytes to every format, and so does the `capture_input` fuzz target.
- **A malformed dump is an `InputError` naming its line and column**, counted from 1, columns in characters. `malformed_dumps_name_the_line_and_column` covers each problem.
- **What a dump holds reads back the same.** `written_dumps_read_back` writes arbitrary bytes as hex and as `xxd` would, then reads them back.

## Events, metrics and flight recorder

None. The crate is a library for offline tools, with no state to observe.

## Errors

`input::InputError` holds the line, the column and an `InputProblem`:
- `NotText`: a text format given bytes that aren't UTF-8;
- `NotHex`: a character that isn't a hex digit;
- `OddDigits`: a hex digit without its pair;
- `NoOffset`: an `xxd` line without its `offset:`;
- `WrongOffset`: an `xxd` line whose offset isn't the count of the bytes before it.

The errors never reach a Kafka client, so there is no error-code mapping.

## Tests

- **Unit and property tests:** `cargo nextest run -p alluvium-capture`. Each format is tested on a fixture of the same frame bytes.
- **Fuzz:** `cargo xtask fuzz capture_input` parses arbitrary input in each format, and checks that a hex dump of the bytes it parsed reads back the same.

## Debugging

A dump that doesn't parse reports the line and column where it goes wrong. Two common causes:
- **An `xxd` dump with lines missing**, which fails as `WrongOffset` on the line after the gap.
- **A detected format that is wrong.** `detect` takes text of hex digits and spaces as hex, and anything else that isn't an `xxd` dump as raw bytes. Pass the format explicitly to override it.
