# Protocol code generation

`cargo xtask codegen` generates Rust types from Kafka 4.4.0's message schemas ([ADR-0008](../adr/0008-protocol-code-generated-from-kafka-schemas.md)). This page says how each schema feature maps to the generated code.

The generator is [`xtask/src/codegen.rs`](../../xtask/src/codegen.rs). It reads the schemas with [`message_spec`](../../xtask/src/message_spec.rs), which applies the rules of Kafka's own generator.

## Output

- **One module per schema that has valid versions**, under [`crates/alluvium-protocol/src/generated/`](../../crates/alluvium-protocol/src/generated/mod.rs).
  - Request, response and header schemas go at the top level.
  - `data` schemas go under `generated::data`. These are payloads carried inside records and other messages, such as `LeaderChangeMessage` and `ConsumerProtocolAssignment`.
  - A schema without valid versions, such as the four ZooKeeper-era APIs, gets no module. Kafka generates nothing for these either.
- **A header in every file.** It names the schema and Kafka version and says not to edit the file. To change generated code, change the schema or the generator, then run `cargo xtask codegen`.
- **The same bytes on every run.**
  - Files are formatted with the pinned `rustfmt` and written in sorted order.
  - A run writes only the files whose contents changed, and removes files it no longer generates.
- **Error codes too.** `generated::error_code` holds `ErrorCode`, one variant per constant of Kafka's `Errors.java`, and [docs/errors.md](../errors.md) lists every code. `xtask/src/codegen/errors.rs` reads the Java source, and a golden test compares the result with the table Kafka's client jar reports at runtime (`cargo xtask schemas dump-errors`).
- **Checked in CI.** `cargo xtask codegen --check` fails, naming the file and line, when committed output differs from what the generator writes.
- **Usable as a library.** `codegen::generate(inputs, out_dir, filter)` takes the schema directories, the output directory and a filter of message kinds. The metadata record generator (META-F01) reuses it.

## Names

| Schema | Rust |
|---|---|
| Message and struct names | Kept: `ProduceRequest`, `TopicProduceData` |
| Field names | Snake case: `TransactionalId` → `transactional_id`, `KRaftVersion` → `k_raft_version` |
| Field names that are Rust keywords | A `_` suffix: `Type` → `type_`, `Match` → `match_` |

These are generator errors:
- two fields of one struct that map to the same identifier;
- a field that maps to `unknown_tagged_fields`, the name of the generated field;
- a struct named like a prelude type or derive (`Option`, `Vec`, `String`, `Default`…).

Each message is its own module, so inline struct names never clash across messages.

## Field types

| Schema type | Rust type |
|---|---|
| `bool` | `bool` |
| `int8`, `int16`, `uint16`, `int32`, `uint32`, `int64` | `i8`, `i16`, `u16`, `i32`, `u32`, `i64` |
| `float64` | `f64` |
| `uuid` | `alluvium_types::Uuid` |
| `string` | `String` |
| `bytes` (with or without `zeroCopy`) | `bytes::Bytes` |
| `records` | `Option<RecordsField>` |
| `[]T` | `Vec<T>` |
| A struct name | The generated struct |

**Nullable fields.** A field that is nullable in any of its versions becomes an `Option` of its type.

**Records fields are always an `Option`.** Kafka defaults every records field to null, even the non-nullable `UnalignedRecords` of `FetchSnapshotResponse`.

**No payload copies.** `bytes` and `records` values are slices of the frame they arrived in, sharing its allocation, as [`RecordsField`](../../crates/alluvium-protocol/src/records.rs) documents.

**Derives.** Every struct derives `Debug`, `Clone` and `PartialEq`. It doesn't derive `Eq`, because `float64` fields follow IEEE 754 equality, as Kafka's `equals` does.

## Structs

- **Where they go.** Inline structs (a field with `fields`) and `commonStructs` become structs in the message's module. They follow the message struct, in the order the fields name them.
- **Common structs.** A common struct used by several fields is emitted once.
- **Docs.**
  - Each struct's docs say where it comes from.
  - Each field's docs are the schema's `about` text followed by its versions, such as `Versions 3+, nullable 3+.`.
  - `tagged 1+ (tag 0)` and `ignorable` are added where they apply.
- **Nesting limit.** A message may nest structs at most 8 levels deep (`MAX_NESTING_DEPTH`); deeper is a generator error.
  - `cargo xtask codegen` prints the deepest message. In Kafka 4.4.0 that is `StreamsGroupDescribeResponse`, with 6 levels.
  - A struct can't contain itself, so generated decoders recurse at most this deep on any input.

## Defaults

Every struct implements `Default` with the values Kafka gives a field that isn't on the wire:

| Schema | Default |
|---|---|
| No `default` | `0`, `false`, `0.0`, empty string, empty bytes, `Uuid::ZERO`, empty array, a default struct |
| No `default`, nullable field | The same value in `Some`: Kafka defaults nullable fields to empty, not null |
| `records` | `None` |
| `"default": "null"` | `None` |
| A decimal or `0x` hex number, `true`, a string | That value: `"0x7fffffff"` → `2_147_483_647` |

If every field's default is the one its Rust type's own `Default` gives, the struct derives `Default`. Otherwise the generator writes the impl.

## Unknown tagged fields

A struct that is on the wire in at least one flexible version has `unknown_tagged_fields: Vec<RawTaggedField>`. It holds the tagged fields a decoder doesn't know, so they are written back unchanged.

Structs that appear only in non-flexible versions don't have this field.

Kafka's generated classes have the field on every struct, and Kafka's `equals` compares it, treating null as equal to empty. The derived `PartialEq` compares it the same way.

## Version constants

Each message struct has:

| Constant | Value |
|---|---|
| `API_KEY: i16` | The API key, for schemas that have one |
| `VERSIONS: Versions` | The valid versions |
| `FLEXIBLE_VERSIONS: Versions` | The versions that use the flexible encoding |
| `DEPRECATED_VERSIONS: Versions` | The versions Kafka has deprecated |
| `LATEST_VERSION_UNSTABLE: bool` | Requests only: whether the highest version is served only when unstable versions are enabled |

`Versions` is an inclusive range of versions, which may be empty (`Versions::NONE`).
