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
- **Error codes too.** `generated::error_code` holds `ErrorCode`, one variant per constant of Kafka's `Errors.java`, and [docs/errors.md](../errors.md) lists every code. `xtask/src/codegen/errors.rs` reads the Java source, and a golden test compares the result with the table Kafka's client jar reports at runtime (`cargo xtask golden errors`).
- **API keys too.** `generated::api_key` holds `ApiKey`, one variant per API ([API keys](#api-keys)).
- **Golden output for Kafka's test schemas.** The six schemas Kafka's own generator tests use (`schemas/kafka-4.4.0/clients-test/`) generate into `crates/alluvium-protocol/src/test_messages/`, compiled only for tests. Together they use every schema feature: each field type, nullable and tagged fields with defaults, nullable structs, keyed arrays and records. A generator change shows as a reviewable diff of these files, and the tests of the generated code use them.
- **Compact formatting.** Generated modules are formatted by rustfmt with every width heuristic at the full line width, so a field's read fits on one line. `cargo fmt` skips them (`#[rustfmt::skip]` where they are declared), and `codegen --check` keeps them exact.
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

**Derives.** Every struct derives `Clone` and `PartialEq`, and `Debug` unless it has a sensitive field (see [Redaction](#redaction)). It doesn't derive `Eq`, because `float64` fields follow IEEE 754 equality, as Kafka's `equals` does.

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

## API keys

`ApiKey` has one variant per API, in API key order, named as Kafka names it (`Produce`, `ApiVersions`). It mirrors Kafka's generated `ApiMessageType` and its `ApiKeys` enum.

- **What each API knows.** From the request schema, as in Kafka:
  - its versions, flexible versions and deprecated versions;
  - its listeners;
  - whether its latest version is unstable.
- **Removed APIs are included.** The four whose schemas have no valid versions keep their variants, with no versions, so their keys stay reserved.
- **Generation checks.** Generation fails when:
  - two requests, or two responses, have one key;
  - an API lacks a request or a response;
  - its request and response disagree on valid or flexible versions. One flexible range per API then serves both headers.
- **Flags from Kafka's jar.** `is_cluster_action` and `is_forwardable` aren't schema properties: Kafka passes them to `ApiKeys`' constructors. Generation reads them from the registry `cargo xtask golden api-keys` recorded from Kafka's jar, in `tests/golden/protocol/registry/api_keys.json`. It fails unless that file lists exactly the schemas' API keys under the same names.
- **Hand-written rules on top.** [`api_key.rs`](../../crates/alluvium-protocol/src/api_key.rs) adds the rules Kafka's `ApiKeys` applies beyond the table:
  - `from_id`;
  - `latest_version(enable_unstable)` and `is_version_enabled`, where every `ApiVersions` version is enabled (KIP-511);
  - `is_version_deprecated`;
  - `served_on(listener)`.

## Decoding

Every message implements `Message::decode(reader, version)`, which mirrors Kafka's generated `read`.

**Version check.** A version outside `VERSIONS` fails as `UnsupportedVersion` ("can't read version V of M"). Kafka's callers make this check before reading; here `decode` makes it, so nested structs never see a version their message lacks.

**Fields.** The value starts at its `Default`, and each field present in the version is read over its default, in schema order. A field absent from the version keeps its default.

The versions a field is present, nullable, flexible or tagged in become conditions on `version`. A condition is written only where it varies within the struct's versions, so a field present in every version reads unconditionally. Adjacent fields present in the same versions read in one block.

**Strings, bytes, records and arrays.**
- Their length prefix is compact (an unsigned varint plus one) in the field's flexible versions, and classic elsewhere.
- In a version where the field isn't nullable, a null fails as `UnexpectedNull`, naming the field: Kafka's "non-nullable field … was serialized as null".
- Array elements are never null.

**Structs.**
- A nullable struct is preceded by a marker: an `int8`, negative for null, or in a tagged field an unsigned varint, zero for null.
- Struct arrays are read through `Reader::read_array`, so a lying count fails before anything is allocated.

**Tagged fields.** In a flexible version, a tagged-field section follows the struct's fields:
- A known tag is read from exactly its declared bytes ([ADR-0020](../adr/0020-known-tagged-fields-read-within-their-declared-size.md)).
- A known tag outside its tagged versions fails as `TagNotValidForVersion`.
- A repeated known tag keeps its last value.
- Every unknown tag is kept, in order, in `unknown_tagged_fields`.

**Errors.** Every error names the path of the field it happened in, such as `TopicData.Name`.

**No panics.** A property test decodes arbitrary input as every message at every version (`DECODE_EVERY_VERSION`, generated for tests) and requires each decode to return.

## Encoding

Every message implements `Message::encode(writer, version)`, which mirrors Kafka's generated `write`, and `Message::encoded_size(version)`.

**Version check.** A version outside `VERSIONS` fails as `UnsupportedVersion` ("can't write version V of M").

**Absent fields.** A field the version doesn't have must hold its default unless it is `ignorable`; otherwise writing would lose it, and the encode fails as `NonDefaultFieldUnsupported` ("attempted to write a non-default F at version V").
- These checks run first, in `check_absent_fields`, so a failed encode writes nothing.
- An `ignorable` field the version lacks is dropped. ProduceResponse's `RecordErrors` before v8 is an example.
- As in Kafka, a null counts as non-default unless the field's default is null.

**Present fields** are written in schema order, framed as in decoding. A `None` where the version needs a value fails as `NullForNonNullable`, naming the field. Nullable structs get the same markers as in decoding.

**Tagged fields** are written only when they differ from their defaults.
- `RawTaggedFieldWriter` writes the unknown ones between them, so the section is in ascending tag order.
- An unknown field that reuses a tag the struct defines fails as `RawTagIsDefined`.
- Unknown fields in a version without tagged fields fail as `TaggedFieldsUnsupported`.
- A decoded message re-encodes to the same bytes, unknown tags included, at every level.

**Size.** `encoded_size` encodes into a counting `Writer`, which stores nothing, so it always equals the length `encode` writes. It also reports the bytes of records and `zeroCopy` byte fields.

**Frames.** A `Writer::sharing()` writer never copies records or `zeroCopy` byte fields.
- Each becomes a segment of its own that holds the message's `Bytes`, as in Kafka's `SendBuilder`.
- `Writer::into_frame` returns an `EncodedFrame`: those shared segments, with the owned bytes between them coalesced, ready for a vectored write.
- `Writer::new()` copies them instead. That is cheaper when the caller wants one buffer (`into_bytes`): sharing costs a few hundred instructions per value, and copying a small batch costs less.

**Infallible structs.** A struct whose encoding can't fail, because it has only fixed-size fields that are present in every version, gets a `write_fields` that returns nothing.

**Data types.** `encode_version_prefixed` and `decode_version_prefixed` write and read an `int16` version before the body, as Kafka's `MessageUtil.toVersionPrefixedBytes` does for control record keys and consumer protocol payloads.

**Tests.** A generated table encodes every message's default at every version, checks its size, and decodes it back. Each message module also has a property test, `round_trips_at_every_version`, which encodes arbitrary values at each version (from a generated `arbitrary(version)` strategy, unknown tagged fields included), checks `encoded_size`, and decodes them back unchanged.

**Golden vectors.** `golden_tests` checks every case `cargo xtask golden messages` recorded from Kafka 4.4.0 (tests/golden/protocol/messages/):
- **Every message Kafka wrote:** it must decode at its version, consume all its input, and encode back to the same bytes, with `encoded_size` equal to their length. Data types also round-trip with a version prefix.
- **Every downgrade:** it must encode as Kafka did, or fail where Kafka throws, with the matching error.
- **Dispatch:** the generated `TRANSCODE` table maps each schema name to its type.

## Errors

Kafka throws a `RuntimeException` or `UnsupportedVersionException` whose message is its only detail. Here each failure is a variant of `DecodeErrorKind` (inside a `DecodeError` with the byte offset and field path) or `EncodeError`. The tables give the Kafka 4.4.0 message each variant corresponds to; "—" means Kafka has no such check.

Sources: `MessageDataGenerator`, `FieldSpec`, `ByteBufferAccessor`, `ByteUtils` and `RawTaggedFieldWriter`, in `generator/` and `clients/` at tag `4.4.0`.

| `DecodeErrorKind` | Kafka 4.4.0 |
|---|---|
| `Truncated` | `BufferUnderflowException`, or "Error reading byte array of N byte(s): only M byte(s) available" |
| `VarintTooLong` | "Varint is too long, the most significant bit in the 5th byte is set, converted value: X" |
| `VarlongTooLong` | "Varlong is too long, most significant bit in the 10th byte is set, converted value: X" |
| `UnexpectedNull` | "non-nullable field F was serialized as null" |
| `ArrayLongerThanInput` | "Tried to allocate a collection of size N, but there are only M bytes remaining." |
| `LimitExceeded` (array length) | "Tried to read a collection of size N, which exceeds the maximum allowed size of 1000000." |
| `LimitExceeded` (string length) | "string field F had invalid length N" |
| `LimitExceeded` (tagged-field count) | "Tried to read N tagged fields, which exceeds the maximum allowed count of 10000." |
| `LimitExceeded` (allocation in bytes) | — ([DecodeLimits](../../crates/alluvium-protocol/src/limits.rs)) |
| `NegativeTaggedFieldCount` | "Invalid negative number of tagged fields N" |
| `TaggedFieldCountExceedsInput` | "Tried to read N tagged fields, but there are only M bytes remaining." |
| `NegativeTaggedFieldSize` | `NegativeArraySizeException`, from `Readable.readUnknownTaggedField` |
| `UnsupportedVersion` | "Can't read version V of M" |
| `TagNotValidForVersion` | "Tag N is not valid for version V" |
| `TaggedFieldSizeMismatch` | — ([ADR-0020](../adr/0020-known-tagged-fields-read-within-their-declared-size.md)) |

| `EncodeError` | Kafka 4.4.0 |
|---|---|
| `StringTooLong` | "'F' field is too long to be serialized" |
| `UnsortedRawTags` | "Invalid raw tag field list: tag N comes after tag P, but is not higher than it." |
| `LengthTooLong` | — (Kafka's lengths are `int`s, so they can't exceed a prefix) |
| `UnsupportedVersion` | "Can't write version V of M" |
| `NullForNonNullable` | `NullPointerException` |
| `NonDefaultFieldUnsupported` | "Attempted to write a non-default F at version V" |
| `TaggedFieldsUnsupported` | "Tagged fields were set, but version V of this message does not support them." |
| `RawTagIsDefined` | "Attempted to use tag N as an undefined tag." |

**When several errors apply.** Kafka sizes a message before writing it (`MessageUtil.toByteBufferAccessor`).
- **Reported first, while sizing:**
  - unknown tagged fields where the version has none;
  - a null where the field isn't nullable;
  - a string that is too long.
- **Reported only while writing:** a non-default field the version lacks.

A failed `encode` reruns into `Writer::sizing()`, a counting writer that skips the absent-field checks, so it reports the error Kafka would.

One difference remains, and only when several fields the version lacks hold values. Kafka names the first in depth-first write order; Alluvium names the first in the outermost struct that has one. The error kind and version are the same. The golden vectors check this for every downgrade Kafka refuses.

## Keyed arrays

A struct with `mapKey` fields makes every array of it keyed. Kafka generates a collection class for such an array, with `find` and `findAll` taking the key fields.

The generated array stays a plain `Vec` in wire order. The element struct gets two associated functions over a slice, taking the key fields in declaration order:

```rust
let first = CreatableTopic::find(&request.topics, "orders");
let all: Vec<&CreatableTopic> = CreatableTopic::find_all(&request.topics, "orders").collect();
```

`find` returns the first match in wire order, and `find_all` every match in wire order. Duplicate keys are kept, as in Kafka's `ImplicitLinkedHashMultiCollection`.

Keys are compared by value. A string key is passed as `&str`, or `Option<&str>` when nullable. A key of another type (bytes, records, a struct or an array) is a generator error.

## Known values

Some integer fields hold values from a known set, such as error codes. [`xtask/codegen/known_values.toml`](../../xtask/codegen/known_values.toml) names them:

| Fields | Accessor returns |
|---|---|
| `ErrorCode`, `AcknowledgeErrorCode`, `PartitionErrorCode`, `TopicConfigErrorCode` | `Result<ErrorCode, UnknownErrorCode>` |
| `IsolationLevel` | `Result<IsolationLevel, DeliveryError>` |
| `Acks` | `Result<Acks, DeliveryError>` |

How the accessors work:
- Each field with one of those names, in any message, gets an accessor of the same name: `partition.error_code()` beside the raw `partition.error_code`.
- The struct keeps the raw integer, so a value the enum doesn't know is still decoded and written back unchanged.
- An entry that matches no field, or a field of another type, is a generator error.
- The coordinator key type and the ACL enums are added by the tasks that define them (GRP-0039, SEC-0051).

## Redaction

[`xtask/codegen/sensitive_fields.toml`](../../xtask/codegen/sensitive_fields.toml) lists the fields that hold credentials:
- SASL `AuthBytes`;
- SCRAM `Salt` and `SaltedPassword`;
- delegation token `Hmac`.

A struct with such a field gets a hand-shaped `Debug` that prints `***` for it, and everything else as the derived `Debug` would. An entry naming a message or field that isn't generated is a generator error.

Records need no entry: `RecordsField` always prints a `PayloadSummary` (length and checksum), never the records.

## Worked example: ProduceRequest

The schema [`ProduceRequest.json`](../../schemas/kafka-4.4.0/clients/ProduceRequest.json) starts like this:

```json
"apiKey": 0, "type": "request", "name": "ProduceRequest",
"validVersions": "3-13", "flexibleVersions": "9+",
"fields": [
  { "name": "TransactionalId", "type": "string", "versions": "3+", "nullableVersions": "3+",
    "default": "null", "about": "The transactional ID, or null if the producer is not transactional." },
  { "name": "Acks", "type": "int16", "versions": "0+", "about": "The number of acknowledgments ..." },
  { "name": "TimeoutMs", "type": "int32", "versions": "0+", "about": "The timeout to await a response in milliseconds." },
  { "name": "TopicData", "type": "[]TopicProduceData", "versions": "0+", "fields": [ ... ] }
]
```

It generates [`produce_request.rs`](../../crates/alluvium-protocol/src/generated/produce_request.rs), whose message struct is:

```rust
/// The `ProduceRequest` request, API key 0: versions 3-13, flexible versions 9+.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ProduceRequest {
    /// The transactional ID, or null if the producer is not transactional.
    ///
    /// Versions 3+, nullable 3+.
    pub transactional_id: Option<String>,
    /// The number of acknowledgments ...
    ///
    /// Versions 0+.
    pub acks: i16,
    /// The timeout to await a response in milliseconds.
    ///
    /// Versions 0+.
    pub timeout_ms: i32,
    /// Each topic to produce to.
    ///
    /// Versions 0+.
    pub topic_data: Vec<TopicProduceData>,
    /// Tagged fields this version doesn't define, kept to be written back unchanged.
    pub unknown_tagged_fields: Vec<crate::RawTaggedField>,
}
```

How each part of the schema shows up in that module:
- **Nullable.** `TransactionalId` is nullable, so it is an `Option`. Its `"default": "null"` is `None`, the same as the derived `Default`.
- **Known value.** `Acks` gets an `acks()` accessor returning `Result<Acks, DeliveryError>`.
- **Unknown tagged fields.** Version 9 and later are flexible, so the struct keeps unknown tagged fields.
- **Constants.** `VERSIONS` is `Versions::new(3, 13)` and `FLEXIBLE_VERSIONS` is `Versions::new(9, i16::MAX)`.
- **Nested structs.** `TopicProduceData` and `PartitionProduceData` follow in the same module.
- **No derived `Default` for topics.** `TopicProduceData` holds a `uuid` `TopicId`, and `Uuid` has no `Default`, so the generator writes its `Default` impl with `Uuid::ZERO`.
- **Records.** `PartitionProduceData`'s `Records` field is `Option<RecordsField>`.
