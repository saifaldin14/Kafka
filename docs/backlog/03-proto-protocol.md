# PROTO: Kafka protocol and code generation

> Crates: `alluvium-protocol` (layer 0), `xtask codegen` · Kafka counterpart: `clients/src/main/resources/common/message`, `generator/`, `common/protocol` · Milestones: M1-M2

Byte-exact Kafka 4.4.0 wire protocol: vendored schemas and a code generator producing committed, readable Rust codecs for every request, response and header version, with bounded allocation and fuzzing.

## [Done] PROTO-F01 Vendored schemas
P0 · M1 · Feature · deps: FND-F08
Scope: `schemas/kafka-4.4.0/` copied unmodified from tag 4.4.0 with LICENSE/NOTICE, checksum manifest and an update procedure.

### [Done] PROTO-0001 Vendor `clients/src/main/resources/common/message` from Kafka tag 4.4.0 into `schemas/kafka-4.4.0/clients/` with Kafka's LICENSE and NOTICE
P0 · M1 · Task · deps: none · labels: docs · ref: clients/src/main/resources/common/message @4.4.0
Done when: all 203 files (request, response, header and data schemas plus README.md) are byte-identical to tag 4.4.0 (commit 1156b2752aaa28eafc0b4c4d387e63356793268d); `schemas/kafka-4.4.0/` holds Kafka's LICENSE and NOTICE; a README states the source tag, commit and the no-edit rule.
Notes: The directory at the tag holds 204 files: 203 JSON schemas and Kafka's README.md on the schema format; all 204 are vendored. Twenty schemas have no final newline and ten have trailing whitespace, so `.editorconfig` turns those fixes off under `schemas/kafka-*/`, and every committed blob was compared byte for byte with the tag.

### [Done] PROTO-0002 Vendor metadata, group, transaction and share coordinator and raft schemas from tag 4.4.0 into `schemas/kafka-4.4.0/`
P0 · M1 · Task · deps: none · ref: metadata/src/main/resources/common/metadata @4.4.0
Done when: metadata (27), group-coordinator (44), transaction-coordinator (2), share-coordinator (4) and raft `QuorumStateData.json` copied unmodified under paths mirroring Kafka's modules; `streams/…/SubscriptionInfoData.json` is deliberately not vendored (client-side only) and the README says why.

### [Done] PROTO-0003 Schema manifest `schemas/kafka-4.4.0/MANIFEST.sha256` and `cargo xtask schemas verify` step in `cargo xtask ci`
P0 · M1 · Task · deps: PROTO-0001, PROTO-0002 · labels: security · ref: ADR-0008
Done when: the manifest lists every vendored file with its SHA-256 and the tag commit; `verify` fails naming the file on any added, removed or modified schema (tested with a tampered fixture directory); the step runs in CI.

### [Done] PROTO-0004 `cargo xtask schemas vendor --tag <tag>`: fetch schema directories from an apache/kafka tag and rewrite the manifest
P2 · M1 · Task · deps: PROTO-0003
Done when: the command sparse-clones the tag into `target/`, replaces `schemas/kafka-<tag>/`, regenerates the manifest, and `--dry-run` prints added, removed and changed schema files; never runs in CI (network).

### [Done] PROTO-0005 Document the Kafka schema update procedure in `docs/guides/updating-kafka-schemas.md`
P2 · M1 · Task · deps: PROTO-0004 · labels: docs
Done when: the guide covers vendoring, manifest, reviewing the generated diff, regenerating golden vectors, updating docs/parity.md and when an ADR is needed; linked from ADR-0008 and docs/parity.md.

## [Done] PROTO-F02 Schema parser and model
P0 · M1 · Feature · deps: PROTO-F01, FND-F04
Scope: Parse Kafka's JSON-with-comments schemas into a typed model: versions, flexible/tagged/nullable versions, defaults, map keys, common structs, entity types, `latestVersionUnstable`.

### [Done] PROTO-0006 Schema loader for Kafka's JSON-with-comments files: strip `//` and `/* */` comments, reject trailing tokens, typed `SchemaError` with file, line and JSON pointer
P0 · M1 · Task · deps: none · ref: generator/src/main/java/org/apache/kafka/message/MessageGenerator.java JSON_SERDE (ALLOW_COMMENTS, FAIL_ON_TRAILING_TOKENS) @4.4.0
Done when: comment markers inside string literals are preserved; unterminated block comments and trailing tokens are errors with file:line:column; every vendored file loads; unit tests cover each case.

### [Done] PROTO-0007 Version range parsing (`N`, `N-M`, `N+`, `none`, empty → default) with intersect, subtract and contains
P0 · M1 · Task · deps: none · ref: generator/src/main/java/org/apache/kafka/message/Versions.java parse/intersect/subtract @4.4.0
Done when: negative bounds are rejected; `N+` ends at i16::MAX; property tests show intersect is commutative, `a.subtract(b)` ⊆ a and contains agrees with the bounds; VersionsTest cases ported.

### [Done] PROTO-0008 Typed schema model `MessageSpec`, `StructSpec`, `FieldSpec` covering every property Kafka's generator accepts, unknown properties rejected
P0 · M1 · Task · deps: PROTO-0006, PROTO-0007 · ref: generator/src/main/java/org/apache/kafka/message/MessageSpec.java, FieldSpec.java @4.4.0
Done when: message properties (name, validVersions, deprecatedVersions, apiKey, type, commonStructs, flexibleVersions, listeners, latestVersionUnstable) and field properties (name, versions, fields, type, mapKey, nullableVersions, default, ignorable, entityType, about, taggedVersions, flexibleVersions, tag, zeroCopy) are modeled; an unknown key is a SchemaError naming file and pointer; single values are accepted where Kafka accepts arrays (ACCEPT_SINGLE_VALUE_AS_ARRAY).
Notes: Properties are read as Jackson reads them: unknown properties fail, JSON `null` means absent, text properties also take numbers and booleans (30 vendored defaults are numbers, 4 booleans), and list properties take a single value. Numeric and boolean properties are strict, since no vendored schema writes them as text. The listeners rule's message names `listeners`, where Kafka's still says `requestScope`.

### [Done] PROTO-0009 Field type parsing: bool, int8, int16, uint16, int32, uint32, int64, float64, string, uuid, bytes, records, struct names and `[]T` arrays
P0 · M1 · Task · deps: PROTO-0008 · ref: generator/src/main/java/org/apache/kafka/message/FieldType.java parse @4.4.0
Done when: arrays of arrays, unknown primitive names and lower-case struct names are SchemaErrors; `can_be_nullable` is true only for string, bytes, records, struct and arrays; tests per type.

### [Done] PROTO-0010 Message-level validation: message type (request, response, header, metadata, data, coordinator-key, coordinator-value), apiKey, listeners and required flexibleVersions
P0 · M1 · Task · deps: PROTO-0008 · ref: generator/src/main/java/org/apache/kafka/message/MessageSpec.java MessageSpec @4.4.0
Done when: each rejection in MessageSpec's constructor (missing flexibleVersions, listeners on non-requests, latestVersionUnstable on non-requests, coordinator-key without apiKey or with versions other than 0 or with flexibleVersions, coordinator-value without apiKey) is a SchemaError with a fixture test.

### [Done] PROTO-0011 Field name validation (`[A-Za-z][A-Za-z0-9]*`) and per-struct uniqueness of names and tags with tags contiguous from 0
P0 · M1 · Task · deps: PROTO-0008 · ref: generator/src/main/java/org/apache/kafka/message/StructSpec.java, FieldSpec.java VALID_FIELD_NAMES @4.4.0
Done when: invalid names, duplicate names, duplicate tags and a gap in tag ids are SchemaErrors naming struct and field; tests port StructSpecTest.

### [Done] PROTO-0012 Field `versions` semantics: required unless `taggedVersions` is given (then defaulting to it); per-version presence = field versions ∩ parent struct versions
P0 · M1 · Task · deps: PROTO-0008 · ref: generator/src/main/java/org/apache/kafka/message/FieldSpec.java FieldSpec, StructRegistry.java addStructSpecs @4.4.0
Done when: `present_versions(field, parent)` returns the right set for subset ranges such as `0-2` and `3+`; a field with no versions and no taggedVersions is an error; tested on ProduceRequest `TransactionalId` (3+) and a removed `0-2` field fixture.

### [Done] PROTO-0013 Message `flexibleVersions` and per-field `flexibleVersions` overrides (string and bytes only)
P0 · M1 · Task · deps: PROTO-0008, PROTO-0009 · ref: clients/src/main/resources/common/message/RequestHeader.json ClientId; generator/src/main/java/org/apache/kafka/message/FieldSpec.java @4.4.0
Done when: an override on any other type is a SchemaError; effective flexible versions per field are exposed; RequestHeader `ClientId` with override `none` resolves to classic encoding in header v2 (unit test).

### [Done] PROTO-0014 `nullableVersions` parsing and validation
P0 · M1 · Task · deps: PROTO-0009 · ref: generator/src/main/java/org/apache/kafka/message/FieldSpec.java nullableVersions @4.4.0
Done when: nullable ranges on bool, int, float or uuid fields are SchemaErrors; nullable ranges are exposed per field; ProduceResponse `ErrorMessage` (nullable 8+) is modeled correctly in a test.

### [Done] PROTO-0015 `tag` and `taggedVersions` rules
P0 · M1 · Task · deps: PROTO-0012, PROTO-0013, PROTO-0014 · ref: generator/src/main/java/org/apache/kafka/message/FieldSpec.java checkTagInvariants; generator/src/main/java/org/apache/kafka/message/MessageDataGenerator.java generateClassReader @4.4.0
Done when: fixture tests reject: negative tag; tag without taggedVersions and vice versa; taggedVersions not open-ended; taggedVersions not a subset of versions; nullable versions covering only part of the tagged versions; tagged mapKey fields; taggedVersions outside the flexible versions.

### [Done] PROTO-0016 Default values for int8, int16, uint16, int32, uint32 and int64: decimal and `0x` hex with range checks
P0 · M1 · Task · deps: PROTO-0009 · ref: generator/src/main/java/org/apache/kafka/message/FieldSpec.java fieldDefault @4.4.0
Done when: empty default means 0; out-of-range and non-numeric defaults are SchemaErrors naming the field; hex parses with the same base rules as Kafka (no octal); tests per type at both bounds.
Notes: Kafka checks an integer default as decimal or hex but then writes the text into Java as a literal. So three forms are rejected here instead of being given a value that differs from Kafka's generated code: a leading zero (`010` is octal 8 in Java), a sign after `0x` (not a Java literal), and a `uint32` default above `i32::MAX` (an `int` literal that is too large, or negative in hex). The vendored schemas' defaults (`-1`, `0x7fffffff` and the like) are unaffected.

### [Done] PROTO-0017 Default values for bool, float64, uuid, string, bytes, records, struct and array fields, including `null`
P0 · M1 · Task · deps: PROTO-0009, PROTO-0014 · ref: generator/src/main/java/org/apache/kafka/message/FieldSpec.java fieldDefault @4.4.0
Done when: bool accepts true/false case-insensitively; uuid defaults are base64url and decode to 16 bytes; float64 parses; bytes, struct and array accept only empty or `null`; `null` is allowed only when the field is nullable in all its versions; each rejection has a fixture test.
Notes: `"default": null` reaches Kafka's constructor as Java null, so it means "no default" (the type's empty value), not null; four vendored fields write it. Only the text `"null"` asks for null. A `records` default is ignored, as in Kafka. Uuid defaults are checked with `alluvium_types::Uuid`, the parser of Kafka's `Uuid.fromString`, which the generated code calls. float64 defaults accept Java's decimal forms, `NaN` and `Infinity`, but not hex floats; no vendored schema has a float64 default.

### [Done] PROTO-0018 `mapKey` parsing and detection of struct arrays with keys
P0 · M1 · Task · deps: PROTO-0008 · ref: generator/src/main/java/org/apache/kafka/message/StructRegistry.java isStructArrayWithKeys @4.4.0
Done when: fields marked mapKey are exposed per struct; `is_struct_array_with_keys` matches Kafka's StructRegistry for every vendored schema (count asserted in a test).

### [Done] PROTO-0019 `ignorable` flag exposed per field for encode-time checks
P0 · M1 · Task · deps: PROTO-0008 · ref: generator/src/main/java/org/apache/kafka/message/FieldSpec.java ignorable @4.4.0
Done when: the flag is in the model with its default false; ProduceResponse `RecordErrors` (ignorable) and a non-ignorable field are asserted in a test.

### [Done] PROTO-0020 `entityType` parsing (unknown, transactionalId, producerId, groupId, topicName, brokerId) with type verification
P0 · M1 · Task · deps: PROTO-0009 · ref: generator/src/main/java/org/apache/kafka/message/EntityType.java verifyTypeMatches @4.4.0
Done when: a mismatched type (for example topicName on int32) is a SchemaError; arrays whose element type matches are accepted; values are exposed in the model; EntityTypeTest cases ported.

### [Done] PROTO-0021 `commonStructs` and the struct registry: registration, lookup, inline structs with intersected parent versions
P0 · M1 · Task · deps: PROTO-0009, PROTO-0012 · ref: generator/src/main/java/org/apache/kafka/message/StructRegistry.java register/findStruct @4.4.0
Done when: errors for lower-case struct names, a common struct defined twice, an inline re-specification of a common struct, an inline struct defined twice in a message and an unknown struct reference; StructRegistryTest cases ported.
Notes: As in Kafka, a struct-typed message field with no inline fields and no common struct of that name defines an empty struct, so an unknown struct can only be named from a common struct's fields, which Kafka never registers. That case is reported as Kafka's `findStruct` would report it.

### [Done] PROTO-0022 `latestVersionUnstable` and `deprecatedVersions` in the model
P0 · M1 · Task · deps: PROTO-0010, PROTO-0007 · ref: generator/src/main/java/org/apache/kafka/message/MessageSpec.java latestVersionUnstable @4.4.0
Done when: both are exposed per message; InitProducerIdRequest is the only 4.4.0 schema with latestVersionUnstable=true (asserted); deprecated ranges parse with the version syntax.

### [Done] PROTO-0023 `zeroCopy` flag (bytes fields only)
P0 · M1 · Task · deps: PROTO-0009 · ref: generator/src/main/java/org/apache/kafka/message/FieldSpec.java zeroCopy @4.4.0
Done when: zeroCopy on a non-bytes field is a SchemaError; EnvelopeRequest `RequestData` and PushTelemetryRequest `Metrics` are flagged in the model (test).

### [Done] PROTO-0024 Parse every vendored 4.4.0 schema and assert the inventory
P0 · M1 · Task · deps: PROTO-0015, PROTO-0016, PROTO-0017, PROTO-0018, PROTO-0019, PROTO-0020, PROTO-0021, PROTO-0022, PROTO-0023, PROTO-0011
Done when: a test loads all vendored files with zero errors and asserts the number of request schemas, unique API keys, matching request/response pairs per API key, and the valid-version ranges of Produce (3-13) and Fetch (4-18).
Notes: 281 schemas load (the 282 vendored files minus Kafka's README). There are 95 request schemas; 91 have valid versions, and four ZooKeeper-era APIs keep their key with `none`. That gives 91 request and 91 response APIs with no unpaired key, and 76 keyed struct arrays. An independent script over the JSON gave the same numbers. `schemas verify` prints this inventory and fails on an unpaired API key.

## [Done] PROTO-F03 Primitive codecs
P0 · M1 · Feature · deps: TYPE-F01, TYPE-F03
Scope: All wire primitives: fixed ints, (u)varint/varlong, float64, bool, uuid, (compact/nullable) strings and bytes, (compact) arrays, tagged-field sections, records fields.

### [Done] PROTO-0025 Wire reader and writer over `Bytes`/`BytesMut` with `DecodeError` carrying byte offset, field path and kind
P0 · M1 · Task · deps: none · labels: decoder · ref: clients/src/main/java/org/apache/kafka/common/protocol/ByteBufferAccessor.java, Readable.java, Writable.java @4.4.0
Done when: reads never panic and return `DecodeError::Truncated { offset, needed, remaining }` on underflow; the module compiles under `deny(clippy::indexing_slicing, clippy::arithmetic_side_effects)`; module docs cite ByteBufferAccessor.

### [Done] PROTO-0026 int8 codec (i8)
P0 · M1 · Task · deps: PROTO-0025 · labels: decoder · ref: clients/src/main/java/org/apache/kafka/common/protocol/Readable.java, Writable.java @4.4.0
Done when: all 256 values round-trip; truncated input yields DecodeError at the right offset.

### [Done] PROTO-0027 int16 codec (i16 big-endian)
P0 · M1 · Task · deps: PROTO-0025 · labels: decoder · ref: clients/src/main/java/org/apache/kafka/common/protocol/Readable.java, Writable.java @4.4.0
Done when: property round-trip; byte order checked against a fixed vector; truncated input yields DecodeError at the right offset.

### [Done] PROTO-0028 int32 codec (i32 big-endian)
P0 · M1 · Task · deps: PROTO-0025 · labels: decoder · ref: clients/src/main/java/org/apache/kafka/common/protocol/Readable.java, Writable.java @4.4.0
Done when: property round-trip; byte order checked against a fixed vector; truncated input yields DecodeError at the right offset.

### [Done] PROTO-0029 int64 codec (i64 big-endian)
P0 · M1 · Task · deps: PROTO-0025 · labels: decoder · ref: clients/src/main/java/org/apache/kafka/common/protocol/Readable.java, Writable.java @4.4.0
Done when: property round-trip; byte order checked against a fixed vector; truncated input yields DecodeError at the right offset.

### [Done] PROTO-0030 uint16 codec (u16 big-endian)
P0 · M1 · Task · deps: PROTO-0025 · labels: decoder · ref: clients/src/main/java/org/apache/kafka/common/protocol/Readable.java, Writable.java @4.4.0
Done when: 0..=65535 round-trip; the schema type maps to u16 so Kafka's writeUnsignedShort range check is enforced by the type; truncated input yields DecodeError at the right offset.

### [Done] PROTO-0031 uint32 codec (u32 big-endian)
P0 · M1 · Task · deps: PROTO-0025 · labels: decoder · ref: clients/src/main/java/org/apache/kafka/common/protocol/Readable.java, Writable.java @4.4.0
Done when: property round-trip over u32 including values above i32::MAX; truncated input yields DecodeError at the right offset.

### [Done] PROTO-0032 float64 codec: IEEE 754 big-endian, bit-exact
P0 · M1 · Task · deps: PROTO-0025 · labels: decoder · ref: clients/src/main/java/org/apache/kafka/common/utils/internals/ByteUtils.java readDouble/writeDouble @4.4.0
Done when: property round-trip compares `to_bits` including NaN payloads, ±0.0, ±infinity and subnormals.

### [Done] PROTO-0033 boolean codec: any non-zero byte decodes as true; encode writes 0x00 or 0x01
P0 · M1 · Task · deps: PROTO-0025 · labels: decoder · ref: generator/src/main/java/org/apache/kafka/message/MessageDataGenerator.java primitiveReadExpression @4.4.0
Done when: decoding 0x02 and 0xFF yields true as in Kafka's generated `readByte() != 0`; re-encoding normalizes to 0x01 (documented, tested).

### [Done] PROTO-0034 uuid codec: 16 bytes, most significant 64 bits first, using TYPE-F03 `Uuid`
P0 · M1 · Task · deps: PROTO-0025, TYPE-F03 · labels: decoder · ref: clients/src/main/java/org/apache/kafka/common/protocol/Readable.java readUuid @4.4.0
Done when: round-trip property test; zero uuid encodes as 16 zero bytes; byte order checked against a vector produced by Kafka's `Uuid.fromString`.

### [Done] PROTO-0035 Unsigned varint (u32) codec with Kafka's 5-byte limit and `size_of`
P0 · M1 · Task · deps: PROTO-0025 · labels: decoder · ref: clients/src/main/java/org/apache/kafka/common/utils/internals/ByteUtils.java readUnsignedVarint/writeUnsignedVarint/sizeOfUnsignedVarint @4.4.0
Done when: decoding matches Kafka bit for bit: overlong encodings are accepted and bits above bit 3 of the 5th byte are discarded; a 5th byte with the continuation bit set is `DecodeError::VarintTooLong`; encoding is canonical; property: size_of == encoded length; ByteUtilsTest vectors ported.

### [Done] PROTO-0036 Signed varint (zigzag i32) codec
P0 · M1 · Task · deps: PROTO-0035 · labels: decoder · ref: clients/src/main/java/org/apache/kafka/common/utils/internals/ByteUtils.java readVarint/writeVarint @4.4.0
Done when: i32::MIN, −1, 0 and i32::MAX round-trip; property round-trip; size_of matches; ByteUtilsTest vectors ported.

### [Done] PROTO-0037 Signed varlong (zigzag i64) codec with the 10-byte limit
P0 · M1 · Task · deps: PROTO-0025 · labels: decoder · ref: clients/src/main/java/org/apache/kafka/common/utils/internals/ByteUtils.java readVarlong/writeVarlong/sizeOfVarlong @4.4.0
Done when: a continuation bit on the 10th byte is `DecodeError::VarlongTooLong`; i64 extremes round-trip; property round-trip; size_of matches; ByteUtilsTest vectors ported.

### [Done] PROTO-0038 string codec (int16 length + UTF-8)
P0 · M1 · Task · deps: PROTO-0027 · labels: decoder · ref: clients/src/main/java/org/apache/kafka/common/protocol/Readable.java readString @4.4.0
Done when: negative length is `UnexpectedNull` for the non-nullable reader; length beyond remaining is Truncated; invalid UTF-8 decodes with U+FFFD replacement exactly like Java's `new String(bytes, UTF_8)` (vectors in PROTO-F10); encoding a string over 32767 bytes is `EncodeError::StringTooLong`.

### [Done] PROTO-0039 nullable string codec
P0 · M1 · Task · deps: PROTO-0038 · labels: decoder · ref: generator/src/main/java/org/apache/kafka/message/MessageDataGenerator.java generateVariableLengthReader @4.4.0
Done when: any negative length decodes as None (Kafka treats every negative length as null); None encodes as −1; property round-trip.

### [Done] PROTO-0040 compact string codec (unsigned varint length + 1)
P0 · M1 · Task · deps: PROTO-0035, PROTO-0038 · labels: decoder · ref: generator/src/main/java/org/apache/kafka/message/MessageDataGenerator.java generateVariableLengthReader @4.4.0
Done when: the length is computed with Java int semantics (u32 reinterpreted as i32, minus 1, wrapping): 0 and 0x80000001–0xFFFFFFFF mean null and fail for the non-nullable reader, 0x80000000 becomes i32::MAX and fails as Truncated; property round-trip.

### [Done] PROTO-0041 compact nullable string codec
P0 · M1 · Task · deps: PROTO-0040 · labels: decoder · ref: generator/src/main/java/org/apache/kafka/message/MessageDataGenerator.java generateVariableLengthReader @4.4.0
Done when: null encodes as length 0; every value Kafka reads as null decodes as None; property round-trip.

### [Done] PROTO-0042 bytes codec (int32 length) decoding to a zero-copy `Bytes` slice
P0 · M1 · Task · deps: PROTO-0028 · labels: decoder · ref: clients/src/main/java/org/apache/kafka/common/protocol/Readable.java readArray/readByteBuffer @4.4.0
Done when: negative length is `UnexpectedNull`; length beyond remaining is Truncated; the decoded slice shares the input allocation (pointer-range test); property round-trip.

### [Done] PROTO-0043 nullable bytes codec
P0 · M1 · Task · deps: PROTO-0042 · labels: decoder · ref: generator/src/main/java/org/apache/kafka/message/MessageDataGenerator.java generateVariableLengthReader @4.4.0
Done when: any negative length decodes as None; None encodes as −1; property round-trip.

### [Done] PROTO-0044 compact bytes codec (unsigned varint length + 1)
P0 · M1 · Task · deps: PROTO-0035, PROTO-0042 · labels: decoder · ref: generator/src/main/java/org/apache/kafka/message/MessageDataGenerator.java generateVariableLengthReader @4.4.0
Done when: same Java int length semantics as compact strings; zero-copy slice; property round-trip.

### [Done] PROTO-0045 compact nullable bytes codec
P0 · M1 · Task · deps: PROTO-0044 · labels: decoder · ref: generator/src/main/java/org/apache/kafka/message/MessageDataGenerator.java generateVariableLengthReader @4.4.0
Done when: null encodes as length 0; property round-trip including empty vs null.

### [Done] PROTO-0046 records field framing: int32 or compact length, null, raw `Bytes` slice
P0 · M1 · Task · deps: PROTO-0043, PROTO-0045 · labels: decoder · ref: clients/src/main/java/org/apache/kafka/common/protocol/Readable.java readRecords; Writable.java writeRecords @4.4.0
Done when: classic and compact framings decode to a slice of the frame without parsing it; negative length is null as in Kafka's readRecords; property round-trip.

### [Done] PROTO-0047 Array length prefix (int32, −1 = null) with element count checked against remaining bytes before allocation
P0 · M1 · Task · deps: PROTO-0028 · labels: decoder, security · ref: generator/src/main/java/org/apache/kafka/message/MessageDataGenerator.java generateVariableLengthReader @4.4.0
Done when: a length greater than the remaining bytes fails before any allocation (Kafka's "Tried to allocate a collection of size" check); null handling per nullability; property round-trip of the prefix.

### [Done] PROTO-0048 Compact array length prefix (unsigned varint − 1) with the same remaining-bytes check
P0 · M1 · Task · deps: PROTO-0035, PROTO-0047 · labels: decoder, security · ref: generator/src/main/java/org/apache/kafka/message/MessageDataGenerator.java generateVariableLengthReader @4.4.0
Done when: Java int wrapping semantics as for compact strings; zero means null; property round-trip.

### [Done] PROTO-0049 Tagged-field section codec: unsigned varint count, then (tag, size, bytes) entries as `RawTaggedField`
P0 · M1 · Task · deps: PROTO-0035, PROTO-0042 · labels: decoder · ref: clients/src/main/java/org/apache/kafka/common/protocol/types/RawTaggedField.java, types/RawTaggedFieldWriter.java @4.4.0
Done when: a count that is negative as a Java int or exceeds the remaining bytes is an error; each size is bounded by the remaining bytes; the writer emits entries in ascending tag order and rejects unsorted or duplicate raw tags (RawTaggedFieldWriter's "Invalid raw tag field list"); size computation matches output (property).

### [Done] PROTO-0050 Fuzz target `wire_primitives`: fuzzer-chosen sequences of primitive reads
P0 · M1 · Task · deps: PROTO-0032, PROTO-0033, PROTO-0034, PROTO-0036, PROTO-0037, PROTO-0039, PROTO-0041, PROTO-0046, PROTO-0048, PROTO-0049, PROTO-0026, PROTO-0029, PROTO-0030, PROTO-0031, TEST-F07 · labels: test, security, decoder
Done when: the target exercises every primitive reader, runs clean for the CI fuzz budget and never allocates more than the input length times a constant.

### [Done] PROTO-0051 Criterion benchmarks for varint, compact string and bytes codecs
P2 · M1 · Task · deps: PROTO-0036, PROTO-0037, PROTO-0040, PROTO-0044, TEST-F11 · labels: perf
Done when: benchmarks run in the benchmark CI job with baselines recorded.

### [Done] PROTO-0157 Fixed-width reads without a new handle on the input, and one validation pass for valid strings
P1 · M1 · Task · deps: PROTO-0051 · labels: perf
Done when: the PROTO-0051 gated benchmarks show fewer instructions for varint and string reads, and none more elsewhere; the decode tests pass unchanged.

## [Done] PROTO-F04 Generator: message types
P0 · M1 · Feature · deps: PROTO-F02
Scope: Generate structs, nested structs, enums for known values, defaults and version metadata.

### [Done] PROTO-0052 `cargo xtask codegen` output pipeline: one module per schema under `crates/alluvium-protocol/src/generated/`, rustfmt, `@generated` header citing the schema path and Kafka 4.4.0
P0 · M1 · Task · deps: none · ref: ADR-0008
Done when: output is written in sorted order with a generated `mod.rs`; each file states it is generated and must not be edited; running twice yields identical bytes; a fixture schema produces a compiling module.

### [Done] PROTO-0053 Identifier mapping from Kafka CamelCase to Rust (`TransactionalId` → `transactional_id`, struct names kept), keyword escaping and collision detection
P0 · M1 · Task · deps: PROTO-0052
Done when: Rust keywords get a documented suffix; two Kafka names mapping to one identifier is a codegen error; a test over all vendored schemas finds no collision.

### [Done] PROTO-0054 Field type mapping: bool, i8, i16, u16, i32, u32, i64, f64, `Uuid`, `String`, `Bytes`, records field, `Vec<T>`, `Option<T>` for fields nullable in any version
P0 · M1 · Task · deps: PROTO-0052, PROTO-0101, TYPE-F03
Done when: the mapping table is in the generator's module docs and docs/protocol/codegen.md; records fields use the PROTO-F08 type; uuid uses alluvium-types `Uuid`.

### [Done] PROTO-0055 Generate top-level message structs with rustdoc from `about` plus versions, nullable and tagged annotations per field
P0 · M1 · Task · deps: PROTO-0053, PROTO-0054
Done when: every vendored request, response and header schema produces a struct; field docs state versions such as `versions 3+, nullable 3+`; output compiles.

### [Done] PROTO-0056 Generate inline nested structs and struct-array element types inside each message's module
P0 · M1 · Task · deps: PROTO-0055 · ref: generator/src/main/java/org/apache/kafka/message/MessageDataGenerator.java generateSubclasses @4.4.0
Done when: inline names unique within a message cannot clash across messages; nested types carry their own docs; output compiles for all schemas.

### [Done] PROTO-0057 Generate `commonStructs` once per message module and reuse them for every referencing field
P0 · M1 · Task · deps: PROTO-0056 · ref: generator/src/main/java/org/apache/kafka/message/StructRegistry.java @4.4.0
Done when: a common struct used by several fields (for example in ConsumerGroupHeartbeat or DescribeQuorum) is emitted once; test asserts single definition.

### [Done] PROTO-0058 `Default` impls with Kafka defaults: implicit (0, false, empty string, empty bytes, zero uuid, empty array, null records) and custom `default` values
P0 · M1 · Task · deps: PROTO-0055 · ref: clients/src/main/resources/common/message/README.md (Deserializing Messages, Custom Default Values) @4.4.0
Done when: custom decimal, hex and `null` defaults appear in `Default`; a test compares defaults for every field of the vendored schemas against a table derived from the schema model.

### [Done] PROTO-0059 `unknown_tagged_fields: Vec<RawTaggedField>` on every struct that appears in a flexible version
P0 · M1 · Task · deps: PROTO-0055 · ref: generator/src/main/java/org/apache/kafka/message/MessageDataGenerator.java generateUnknownTaggedFieldsAccessor @4.4.0
Done when: the field exists exactly on structs with at least one flexible version and is excluded from equality only where Kafka excludes it (documented); accessors documented.

### [Done] PROTO-0060 Version metadata constants per message: API key, lowest and highest valid version, flexible versions, deprecated versions, latest-version-unstable
P0 · M1 · Task · deps: PROTO-0055 · ref: generator/src/main/java/org/apache/kafka/message/MessageDataGenerator.java generateShortAccessor; clients/src/main/java/org/apache/kafka/common/protocol/ApiMessage.java @4.4.0
Done when: constants are generated for every request, response and data schema; unit test cross-checks them against the schema model.

### [Done] PROTO-0061 mapKey arrays: generate key types and `find`/`find_all` helpers that keep wire order and duplicates
P1 · M1 · Task · deps: PROTO-0056 · ref: generator/src/main/java/org/apache/kafka/message/MessageDataGenerator.java generateHashSetFindMethod/generateHashSetFindAllMethod @4.4.0
Done when: helpers exist for every struct array with keys; duplicates are kept like Kafka's ImplicitLinkedHashMultiCollection; tests on CreateTopicsRequest topics.

### [Done] PROTO-0062 Typed accessors for known-value integer fields via `xtask/codegen/known_values.toml` (error codes, isolation level, acks, coordinator key type, ACL resource/pattern/operation/permission types)
P1 · M1 · Task · deps: PROTO-0055, ERR-F01, TYPE-F12
Done when: the overlay is validated (field exists, integer type matches); wire structs keep raw integers so unknown values round-trip; accessors return `Result<Enum, Error>` with each enum's own error type, for ERR-F01 `ErrorCode` and the TYPE-F12 `IsolationLevel` and `Acks`. The coordinator key type and the ACL enums don't exist in M1; GRP-0039 and SEC-0051 add their overlay entries when they define them.

### [Done] PROTO-0063 Redacting `Debug` for generated types: records fields print a `PayloadSummary`; fields listed in `xtask/codegen/sensitive_fields.toml` print `***`
P0 · M1 · Task · deps: PROTO-0055, TYPE-F09 · labels: security
Done when: the list covers SaslAuthenticateRequest/Response `AuthBytes`, AlterUserScramCredentialsRequest `Salt` and `SaltedPassword`, and `Hmac` in CreateDelegationTokenResponse, DescribeDelegationTokenResponse, RenewDelegationTokenRequest and ExpireDelegationTokenRequest; unknown entries are codegen errors; tests assert no secret or payload bytes in Debug output.

### [Done] PROTO-0064 Generate `data` schemas used by records and coordinators: ControlRecordTypeSchema, EndTxnMarker, LeaderChangeMessage, SnapshotHeaderRecord, SnapshotFooterRecord, KRaftVersionRecord, VotersRecord, ConsumerProtocolAssignment, ConsumerProtocolSubscription, DefaultPrincipalData
P0 · M1 · Task · deps: PROTO-0056
Done when: the types are generated without API keys under `generated::data`; zeroCopy fields of the consumer protocol schemas are `Bytes`; output compiles.

### [Done] PROTO-0065 Generated code passes clippy `-D warnings` with the decoder lints and contains no lint suppressions
P0 · M1 · Task · deps: PROTO-0058, PROTO-0057, PROTO-0064, PROTO-0059, PROTO-0060 · labels: security
Done when: `cargo clippy -p alluvium-protocol -- -D warnings` passes with `deny(clippy::indexing_slicing, clippy::arithmetic_side_effects)` and missing_docs; the generator never emits `#[allow]` (test greps output).

### [Done] PROTO-0066 Generator as a library: input directories, output crate path and message-type filter, so META-F01 and coordinator record codegen reuse it
P1 · M1 · Task · deps: PROTO-0052
Done when: `codegen::generate(inputs, out_dir, filter)` is the single entry point used by `cargo xtask codegen`; a test generates a metadata fixture into a temp dir.

### [Done] PROTO-0067 `docs/protocol/codegen.md`: how each schema feature maps to generated Rust types
P1 · M1 · Task · deps: PROTO-0058, PROTO-0061, PROTO-0062, PROTO-0063 · labels: docs
Done when: covers versions, nullable, defaults, common and inline structs, mapKey, known-value accessors, unknown tagged fields and redaction, with one worked example (ProduceRequest).

## [To Do] PROTO-F05 Generator: versioned encode/decode
P0 · M1 · Feature · deps: PROTO-F03, PROTO-F04
Scope: Per-version encode/decode, size computation, flexible versions, tagged fields, unknown tagged-field preservation, ignorable-field semantics.

### [To Do] PROTO-0068 `Message` trait: `decode(reader, version)`, `encode(writer, version)`, `encoded_size(version)`; versions outside the valid range fail with `UnsupportedVersion`
P0 · M1 · Task · deps: none · ref: clients/src/main/java/org/apache/kafka/common/protocol/Message.java, ApiMessage.java @4.4.0
Done when: decoding or encoding an out-of-range version returns `UnsupportedVersion { message, version }` (Kafka's "Can't read version"/"Can't write version"); the trait is implemented by every generated message.

### [To Do] PROTO-0069 Decode generation for fixed-size fields: present fields read per version, absent fields set to their defaults
P0 · M1 · Task · deps: PROTO-0068 · labels: decoder · ref: generator/src/main/java/org/apache/kafka/message/MessageDataGenerator.java generateClassReader @4.4.0
Done when: for every message and version, fields outside their versions hold defaults after decode (generated test); decode never panics.

### [To Do] PROTO-0070 Decode generation for strings, bytes and records: classic vs compact chosen by the field's effective flexible versions; nullability enforced per version
P0 · M1 · Task · deps: PROTO-0069 · labels: decoder · ref: generator/src/main/java/org/apache/kafka/message/MessageDataGenerator.java generateVariableLengthReader @4.4.0
Done when: a null in a non-nullable version is `DecodeError::NullForNonNullable { field, version }` (Kafka's "non-nullable field … was serialized as null"); RequestHeader v2 `ClientId` decodes as a classic nullable string; a decoded ProduceRequest's records point into the frame allocation (pointer-range test).

### [To Do] PROTO-0071 Decode generation for nested structs, nullable structs and struct arrays
P0 · M1 · Task · deps: PROTO-0070 · labels: decoder · ref: generator/src/main/java/org/apache/kafka/message/MessageDataGenerator.java generateStructReader @4.4.0
Done when: nullable structs read an int8 marker (negative = null) in mandatory position and an unsigned varint marker (≤ 0 = null) when tagged, as Kafka does; struct arrays preallocate within PROTO-F09 limits.

### [To Do] PROTO-0072 Decode generation for arrays of primitives (`[]int32`, `[]int64`, `[]string`, `[]uuid`, …)
P0 · M1 · Task · deps: PROTO-0070 · labels: decoder · ref: generator/src/main/java/org/apache/kafka/message/MessageDataGenerator.java generateVariableLengthReader @4.4.0
Done when: every primitive element type used in the vendored schemas decodes in classic and compact form; property round-trip per element type.

### [To Do] PROTO-0073 Decode generation for tagged-field sections: known tags dispatched within their tagged versions, unknown tags kept as raw bytes
P0 · M1 · Task · deps: PROTO-0071, PROTO-0049 · labels: decoder · ref: generator/src/main/java/org/apache/kafka/message/MessageDataGenerator.java generateClassReader @4.4.0
Done when: a known tag outside its tagged versions fails with "Tag N is not valid for version V" as in Kafka; each known tag decodes from a sub-reader bounded by its declared size and a size mismatch is a DecodeError (stricter than Kafka for malformed input only; noted in docs/parity.md); a repeated known tag keeps the last value and repeated unknown tags are all kept, as in Kafka.

### [To Do] PROTO-0074 Encode generation for fixed and variable-length fields per version, rejecting `None` in non-nullable versions
P0 · M1 · Task · deps: PROTO-0068 · ref: generator/src/main/java/org/apache/kafka/message/MessageDataGenerator.java generateClassWriter/generateVariableLengthWriter @4.4.0
Done when: `None` in a version where the field is not nullable is `EncodeError::NullForNonNullable` (Kafka throws NullPointerException); classic vs compact chosen as in decode.

### [To Do] PROTO-0075 Encode generation for nested, nullable and array structs and primitive arrays
P0 · M1 · Task · deps: PROTO-0074 · ref: generator/src/main/java/org/apache/kafka/message/MessageDataGenerator.java generateClassWriter @4.4.0
Done when: nullable struct markers match decode (int8 or unsigned varint when tagged); decode(encode(x)) == x for generated fixtures.

### [To Do] PROTO-0076 Ignorable-field semantics on encode: a non-default value in a field absent from the target version fails unless the field is `ignorable`
P0 · M1 · Task · deps: PROTO-0074, PROTO-0019 · ref: generator/src/main/java/org/apache/kafka/message/MessageDataGenerator.java generateClassWriter; clients/src/main/resources/common/message/README.md (Ignorable Fields) @4.4.0
Done when: the error is `EncodeError::NonDefaultFieldUnsupported { field, version }`; ignorable fields are silently dropped; tests: ProduceResponse `RecordErrors` (ignorable) at v7 is dropped, a non-ignorable fixture field fails.

### [To Do] PROTO-0077 Encode generation for tagged fields: only non-default values written, ascending tag order merged with unknown raw tags, rejected in non-flexible versions
P0 · M1 · Task · deps: PROTO-0075, PROTO-0049 · ref: generator/src/main/java/org/apache/kafka/message/MessageDataGenerator.java generateClassWriter/generateCheckForUnsupportedNumTaggedFields; clients/src/main/java/org/apache/kafka/common/protocol/types/RawTaggedFieldWriter.java @4.4.0
Done when: setting a tagged field or unknown tag in a non-flexible version fails with `EncodeError::TaggedFieldsUnsupported` (Kafka's "Tagged fields were set, but version V of this message does not support them"); a fixture with known tags 0 and 2 and unknown tag 1 encodes in order 0, 1, 2.

### [To Do] PROTO-0078 Per-version `encoded_size` generation equal to the encoder's output, with zero-copy bytes counted separately
P0 · M1 · Task · deps: PROTO-0077 · ref: generator/src/main/java/org/apache/kafka/message/MessageDataGenerator.java generateClassMessageSize; clients/src/main/java/org/apache/kafka/common/protocol/MessageSizeAccumulator.java @4.4.0
Done when: property test: `encoded_size(v) == encode(v).len()` for every message and version; records and zeroCopy bytes are reported as zero-copy bytes for PROTO-F08; no serialization cache is needed (strings are already UTF-8).

### [To Do] PROTO-0079 Unknown tagged field preservation: byte-exact decode → encode round trip at root and nested levels
P0 · M1 · Task · deps: PROTO-0073, PROTO-0077 · ref: clients/src/main/java/org/apache/kafka/common/protocol/types/RawTaggedField.java @4.4.0
Done when: fixtures with unknown tags around known ones re-encode to identical bytes; accessors let callers read and set raw tags.

### [To Do] PROTO-0080 Version-prefixed encoding helpers for `data` types (int16 version, then the body at that version)
P0 · M1 · Task · deps: PROTO-0075, PROTO-0071, PROTO-0064 · ref: clients/src/main/java/org/apache/kafka/common/protocol/MessageUtil.java toVersionPrefixedBytes @4.4.0
Done when: `encode_version_prefixed`/`decode_version_prefixed` exist; a negative version is a DecodeError; used by control record keys and consumer protocol payloads; round-trip test.

### [To Do] PROTO-0081 Generated proptest strategies for every message and version respecting presence, nullability and tagged fields
P0 · M1 · Task · deps: PROTO-0068, TEST-F16 · labels: test
Done when: strategies are generated beside each message behind `cfg(test)`/a `proptest` feature using TEST-F16 building blocks; absent fields are always default.

### [To Do] PROTO-0082 Round-trip property tests for every message and every valid version
P0 · M1 · Task · deps: PROTO-0081, PROTO-0078, PROTO-0079, PROTO-0076, PROTO-0072 · labels: test
Done when: one generated test per message decodes(encode(m)) == m for each valid version and checks `encoded_size`; the suite runs within the nextest time budget.

### [To Do] PROTO-0083 Port Kafka's message tests: MessageTest, SimpleExampleMessageTest, NullableStructMessageTest, SimpleArraysMessageTest
P1 · M1 · Task · deps: PROTO-0082, PROTO-0135 · labels: test · ref: clients/src/test/java/org/apache/kafka/common/message/MessageTest.java @4.4.0
Done when: tricky cases (tagged defaults, nullable structs, unknown tags, version downgrades) pass against the generated fixtures with `// Ported from:` attribution.

### [To Do] PROTO-0084 Criterion benchmarks: ProduceRequest v13 and FetchResponse v18 decode and encode with 1, 100 and 1000 partitions
P2 · M1 · Task · deps: PROTO-0082, TEST-F11 · labels: perf
Done when: benchmarks run in the benchmark CI job with recorded baselines.

### [To Do] PROTO-0085 Document encode/decode semantics in docs/protocol/codegen.md: version checks, nullability, tagged and unknown fields, ignorable fields and error types
P1 · M1 · Task · deps: PROTO-0082, PROTO-0067 · labels: docs
Done when: each DecodeError and EncodeError variant is listed with its Kafka equivalent message.

## [To Do] PROTO-F06 Request and response headers
P0 · M1 · Feature · deps: PROTO-F05
Scope: Request header v1-v2 (the versions valid in Kafka 4.4.0), response header v0-v1, the ApiVersions response-header exception, header version selection per API version.

### [To Do] PROTO-0086 Header version selection: request header v2 when the API version is flexible else v1; response header v1 when flexible else v0; ApiVersions responses always v0 (KIP-511)
P0 · M1 · Task · deps: PROTO-0093 · ref: generator/src/main/java/org/apache/kafka/message/ApiMessageTypeGenerator.java generateHeaderVersion @4.4.0
Done when: a generated function covers every API key; flexible ranges are open-ended so versions above the supported range still get a header version (an ApiVersions v127 request header decodes as v2); unknown API keys return None; table test against the PROTO-F10 ApiKeys export.

### [To Do] PROTO-0087 Request header decode: peek API key and version, select the header version, decode v1/v2, map a null client id to empty
P0 · M1 · Task · deps: PROTO-0086 · labels: decoder · ref: clients/src/main/java/org/apache/kafka/common/requests/RequestHeader.java parse @4.4.0
Done when: returns the header and its encoded size; unknown API key → `HeaderError::UnknownApiKey`; API without valid versions → `HeaderError::UnsupportedApi`; any other failure carries the best-guess API key for NET-F04's error response; RequestHeaderTest cases ported.

### [To Do] PROTO-0088 Request header encode for inter-node clients (v1, and v2 with the classic nullable `ClientId` and a tagged-field section)
P0 · M1 · Task · deps: PROTO-0086 · ref: clients/src/main/resources/common/message/RequestHeader.json @4.4.0
Done when: encoded bytes equal Kafka's for golden vectors; client id null and empty are distinct on the wire.

### [To Do] PROTO-0089 Response header encode v0/v1 (v1 adds an empty tagged-field section)
P0 · M1 · Task · deps: PROTO-0086 · ref: clients/src/main/resources/common/message/ResponseHeader.json; clients/src/main/java/org/apache/kafka/common/requests/RequestHeader.java toResponseHeader @4.4.0
Done when: correlation id copied from the request; ApiVersions always uses v0; golden bytes match.

### [To Do] PROTO-0090 Response header decode for inter-node clients returning the correlation id for the caller to match
P0 · M1 · Task · deps: PROTO-0086 · labels: decoder · ref: clients/src/main/resources/common/message/ResponseHeader.json @4.4.0
Done when: decode v0 and v1; a truncated header is a DecodeError; matching is left to the caller (NET-F10).

### [To Do] PROTO-0091 Header plus body composition into one output with exact size precomputation (length prefix left to NET-F03)
P0 · M1 · Task · deps: PROTO-0089, PROTO-0088, PROTO-0102
Done when: computed size equals bytes produced for every golden response; zero-copy records chunks from PROTO-F08 are preserved.

### [To Do] PROTO-0092 Header tests over every API key and version: header version table, null/empty/non-ASCII client ids, ApiVersions exception
P0 · M1 · Task · deps: PROTO-0087, PROTO-0090, PROTO-0091 · labels: test · ref: clients/src/test/java/org/apache/kafka/common/requests/RequestHeaderTest.java @4.4.0
Done when: tests cover all 4.4.0 API keys at min, max and max+1 versions.

## [To Do] PROTO-F07 API key registry
P0 · M1 · Feature · deps: PROTO-F04
Scope: Generated `ApiKey` table with version ranges, listener scoping (broker/controller), unstable-version gating.

### [To Do] PROTO-0093 Generate the `ApiKey` enum from request schemas: id, name, valid version range and flexible versions
P0 · M1 · Task · deps: none · ref: generator/src/main/java/org/apache/kafka/message/ApiMessageTypeGenerator.java registerMessageType @4.4.0
Done when: `ApiKey::from_id` returns None for unknown ids; two requests or two responses with one API key are codegen errors (as in ApiMessageTypeGenerator); request and response valid versions must match (codegen check).

### [To Do] PROTO-0094 Listener scoping from `listeners`: broker and controller API sets and `apis_for_listener`
P0 · M1 · Task · deps: PROTO-0093 · ref: clients/src/main/java/org/apache/kafka/common/protocol/ApiKeys.java apisForListener; generator/src/main/java/org/apache/kafka/message/RequestListenerType.java @4.4.0
Done when: sets match Kafka's `ApiKeys.apisForListener` for BROKER and CONTROLLER (checked against the PROTO-F10 export).

### [To Do] PROTO-0095 Unstable-version gating: `latest_version(enable_unstable)` and `is_version_enabled`
P0 · M1 · Task · deps: PROTO-0093 · ref: clients/src/main/java/org/apache/kafka/common/protocol/ApiKeys.java latestVersion(boolean)/isVersionEnabled @4.4.0
Done when: with unstable disabled InitProducerId's latest version is excluded; the flag is a plain bool fed from `unstable.api.versions.enable` (key defined by CFG); unit tests both ways.

### [To Do] PROTO-0096 API flags overlay `xtask/codegen/api_flags.toml` with clusterAction and forwardable per API key
P0 · M1 · Task · deps: PROTO-0093 · ref: clients/src/main/java/org/apache/kafka/common/protocol/ApiKeys.java clusterAction/forwardable @4.4.0
Done when: the overlay must list every API key exactly once (codegen error otherwise); values equal Kafka's ApiKeys enum (checked against the PROTO-F10 export).

### [To Do] PROTO-0097 Deprecated version lookup `is_version_deprecated` for the deprecated-request metric
P0 · M1 · Task · deps: PROTO-0093, PROTO-0022 · ref: clients/src/main/java/org/apache/kafka/common/protocol/ApiKeys.java isVersionDeprecated @4.4.0
Done when: matches ApiKeys.isVersionDeprecated for every API and version (table test).

### [To Do] PROTO-0098 ApiVersions key list builder: generated ranges ∩ an explicit implemented-and-tested table, per listener, with Produce's min version 0 quirk on the broker listener
P0 · M1 · Task · deps: PROTO-0094, PROTO-0095 · ref: clients/src/main/java/org/apache/kafka/common/protocol/ApiKeys.java toApiVersion, PRODUCE_API_VERSIONS_RESPONSE_MIN_VERSION @4.4.0
Done when: APIs whose latest < oldest are omitted; only APIs in the implemented table appear (rule 15); Produce advertises min 0 on the broker listener (KAFKA-18659); consumed by API-F18; unit tests.

### [To Do] PROTO-0099 Registry parity test against the Kafka 4.4.0 ApiKeys export (id, name, oldest/latest version, listeners, flags, deprecated range)
P1 · M1 · Task · deps: PROTO-0096, PROTO-0097, PROTO-0098, PROTO-0125 · labels: test
Done when: every Kafka API key is present with identical values; a mismatch names the API and attribute.

### [To Do] PROTO-0100 Generate `docs/protocol/apis.md`: API key, name, valid and flexible versions, listeners, unstable and deprecated versions
P2 · M1 · Task · deps: PROTO-0098 · labels: docs · ref: clients/src/main/java/org/apache/kafka/common/protocol/Protocol.java @4.4.0
Done when: the page is generated by codegen and covered by the drift check.

## [In Progress] PROTO-F08 Zero-copy records fields
P0 · M1 · Feature · deps: PROTO-F03
Scope: Records fields decode to `Bytes` slices without copying; encode from shared buffers.

### [Done] PROTO-0101 `RecordsField` type holding a slice of the request frame; decode never copies record payloads
P0 · M1 · Task · deps: PROTO-0046 · labels: perf · ref: clients/src/main/java/org/apache/kafka/common/protocol/Readable.java readRecords @4.4.0
Done when: reading a records field leaves its bytes pointing into the frame allocation (pointer-range test); null records map to None; Debug prints a PayloadSummary; the type documents that it keeps the frame alive.

### [To Do] PROTO-0102 Chunked encoder output (`EncodedFrame`): owned header/body bytes interleaved with shared records `Bytes` for vectored writes
P0 · M1 · Task · deps: PROTO-0101 · labels: perf · ref: clients/src/main/java/org/apache/kafka/common/protocol/SendBuilder.java @4.4.0
Done when: encoding a FetchResponse with 100 partitions copies no record bytes (counting-allocator test); total length equals `encoded_size`; adjacent owned segments are coalesced.

### [To Do] PROTO-0103 Deferred records chunks: a placeholder with declared length that the network driver fills from a file region (sendfile path)
P0 · M1 · Task · deps: PROTO-0102 · labels: perf · ref: clients/src/main/java/org/apache/kafka/common/record/internal/MultiRecordsSend.java @4.4.0
Done when: the encoder accepts `RecordsField::Deferred { len, token }` and accounts its length; the protocol crate performs no I/O; NET consumes the placeholder.

### [To Do] PROTO-0104 Retained-frame accounting: decoded requests report how many frame bytes they keep alive (`retained_bytes`)
P2 · M1 · Task · deps: PROTO-0101 · labels: nfr
Done when: the value equals the frame length when any records slice is held and zero otherwise; documented for NET memory accounting.

## [In Progress] PROTO-F09 Decode limits
P0 · M1 · Feature · deps: none
Scope: `DecodeLimits` bounding array lengths, string lengths, nesting depth and total allocation before allocating.

### [Done] PROTO-0105 `DecodeLimits` and a per-decode budget with `DecodeError::LimitExceeded { limit, requested, path }`
P0 · M1 · Task · deps: PROTO-0025 · labels: security · ref: clients/src/main/java/org/apache/kafka/common/protocol/MessageUtil.java MAX_ARRAY_LENGTH/MAX_TAGGED_FIELD_COUNT/MAX_PREALLOCATED_ARRAY_CAPACITY @4.4.0
Done when: defaults equal Kafka's constants where Kafka has one; limits are plain values chosen by the caller (no config parsing in layer 0); unit tests.

### [Done] PROTO-0106 Rule: array length ≤ 1,000,000 elements, checked before allocation
P0 · M1 · Task · deps: PROTO-0105, PROTO-0047, PROTO-0048 · labels: security · ref: clients/src/main/java/org/apache/kafka/common/protocol/MessageUtil.java MAX_ARRAY_LENGTH @4.4.0
Done when: a declared length above the limit fails with Kafka's "exceeds the maximum allowed size" semantics before any allocation (test with an 8-byte input).

### [Done] PROTO-0107 Rule: preallocate at most 1000 elements and grow as elements decode
P0 · M1 · Task · deps: PROTO-0105 · labels: security · ref: clients/src/main/java/org/apache/kafka/common/protocol/MessageUtil.java MAX_PREALLOCATED_ARRAY_CAPACITY @4.4.0
Done when: a valid array of 5000 elements decodes; `Vec::with_capacity` never exceeds 1000 for a declared length (test).

### [Done] PROTO-0108 Rule: string length ≤ 32767 bytes for classic and compact strings
P0 · M1 · Task · deps: PROTO-0105, PROTO-0040 · labels: security · ref: generator/src/main/java/org/apache/kafka/message/MessageDataGenerator.java generateVariableLengthReader @4.4.0
Done when: a compact string declaring 40000 bytes fails with "string field had invalid length" semantics before allocation.

### [Done] PROTO-0109 Rule: tagged-field count ≤ 10,000 and ≤ remaining bytes
P0 · M1 · Task · deps: PROTO-0105, PROTO-0049 · labels: security · ref: clients/src/main/java/org/apache/kafka/common/protocol/MessageUtil.java MAX_TAGGED_FIELD_COUNT; generator/src/main/java/org/apache/kafka/message/MessageDataGenerator.java generateClassReader @4.4.0
Done when: 10,001 declared tags fail before reading entries; test for each bound.

### [Done] PROTO-0110 Rule: static nesting depth — codegen computes each message's maximum struct depth and fails above the documented limit
P0 · M1 · Task · deps: PROTO-0105, PROTO-0056 · labels: security
Done when: the deepest vendored message is reported in codegen output; a fixture deeper than the limit fails codegen; no recursive types exist so no runtime recursion is possible (documented).

### [Done] PROTO-0111 Rule: total allocation budget per decode charging `len × size_of::<T>()` for every Vec and String
P0 · M1 · Task · deps: PROTO-0105 · labels: security
Done when: a 1 MiB frame of maximal empty-struct arrays fails once the budget is exhausted; peak allocation stays under the budget (counting-allocator test); the default budget is justified in docs.

### [To Do] PROTO-0112 Thread `DecodeLimits` through all generated decoders and header decoding
P0 · M1 · Task · deps: PROTO-0106, PROTO-0107, PROTO-0108, PROTO-0109, PROTO-0111, PROTO-0073, PROTO-0072, PROTO-0087 · labels: security
Done when: every generated decode takes the limits; no allocation path in generated code bypasses the budget (generator test greps output for unchecked `with_capacity`).

### [To Do] PROTO-0113 Adversarial-input tests: maximal declared lengths in tiny inputs fail without large allocation
P0 · M1 · Task · deps: PROTO-0112 · labels: security, test
Done when: cases: int32 array length 2^31−1, compact length 0xFFFFFFFF, 10,001 tagged fields, deeply repeated nested arrays; each fails with the expected error and peak allocation under 64 KiB.

### [To Do] PROTO-0114 Document decode limits and their Kafka counterparts in docs/standards/security.md and docs/protocol/codegen.md
P1 · M1 · Task · deps: PROTO-0112 · labels: docs, security
Done when: each limit, its default, its Kafka constant and the error returned are listed.

## [To Do] PROTO-F10 Golden vectors from Kafka
P1 · M1 · Feature · deps: PROTO-F05
Scope: A pinned JVM harness that serializes messages with Kafka 4.4.0's own classes; golden files for every API version; byte-for-byte comparison tests.

### [To Do] PROTO-0115 Pinned JVM harness `tools/kafka-golden/` on `org.apache.kafka:kafka-clients:4.4.0`, run in a digest-pinned Temurin container by `cargo xtask golden`
P1 · M1 · Task · deps: none · ref: ADR-0008
Done when: no host JVM is needed; dependency checksums are verified; the harness source carries Apache-2.0 attribution where it adapts Kafka code.

### [To Do] PROTO-0116 Harness: deterministic messages for every API, version and direction (defaults-only, fully populated with path-derived values, and nulls where nullable)
P1 · M1 · Task · deps: PROTO-0115 · ref: clients/src/main/java/org/apache/kafka/common/protocol/MessageUtil.java toByteBufferAccessor @4.4.0
Done when: values are derived from the field path so reruns are identical; output per case is Kafka's bytes plus its JsonConverter JSON.

### [To Do] PROTO-0117 Golden file layout `tests/golden/protocol/<api>/<request|response>_v<N>_<variant>.bin` with a manifest of SHA-256 and Kafka version
P1 · M1 · Task · deps: PROTO-0116
Done when: files and manifest are committed; the manifest is verified by tests.

### [To Do] PROTO-0118 Rust golden tests: decode each golden file, re-encode byte for byte, `encoded_size` equals the file length
P1 · M1 · Task · deps: PROTO-0117, PROTO-0082 · labels: test
Done when: every golden case passes; failures report API, version, variant and first differing offset.

### [To Do] PROTO-0119 Reverse check: the harness decodes Rust-encoded random messages with Kafka's classes and re-encodes them byte for byte
P1 · M1 · Task · deps: PROTO-0118, PROTO-0081 · labels: test
Done when: 1000 proptest messages per API and version pass the harness `verify` mode; mismatches name API, version and offset.

### [To Do] PROTO-0120 Golden vectors for request header v1/v2 and response header v0/v1, including null client id and the ApiVersions v0 response header
P1 · M1 · Task · deps: PROTO-0117, PROTO-0092 · labels: test
Done when: Rust header encode and decode match the files.

### [To Do] PROTO-0121 Golden vectors for data types (control record key, EndTxnMarker, LeaderChangeMessage, snapshot header/footer, KRaftVersionRecord, VotersRecord, consumer protocol, DefaultPrincipalData)
P1 · M1 · Task · deps: PROTO-0117, PROTO-0080 · labels: test
Done when: Rust decode/encode of each file is byte-identical, using version-prefixed helpers where Kafka uses them.

### [To Do] PROTO-0122 Golden vectors with unknown tagged fields set through Kafka's RawTaggedField at root and nested levels
P1 · M1 · Task · deps: PROTO-0117, PROTO-0079 · labels: test
Done when: Rust preserves and re-emits them byte for byte.

### [To Do] PROTO-0123 Negative vectors: cases where Kafka's encoder throws (non-default non-ignorable field, tagged field in a non-flexible version, null in a non-nullable version) recorded as expected errors
P1 · M1 · Task · deps: PROTO-0117, PROTO-0076, PROTO-0077 · labels: test
Done when: Rust returns the matching EncodeError kind for every recorded case.

### [To Do] PROTO-0124 String decoding parity vectors: invalid UTF-8 sequences decoded by Java and re-encoded
P1 · M1 · Task · deps: PROTO-0117, PROTO-0038 · labels: test, decoder
Done when: Rust string decoding produces the same replacement output for every vector (truncated sequences, overlong forms, surrogates, invalid bytes).

### [To Do] PROTO-0125 Harness export of the ApiKeys table (id, name, oldest/latest, listeners, clusterAction, forwardable, deprecated range) and default broker and controller ApiVersionsResponses
P1 · M1 · Task · deps: PROTO-0115 · ref: clients/src/main/java/org/apache/kafka/common/requests/ApiVersionsResponse.java; clients/src/main/java/org/apache/kafka/common/protocol/ApiKeys.java @4.4.0
Done when: JSON and binary files are committed and used by PROTO-F07 and PROTO-F14 tests.

### [To Do] PROTO-0126 CI: golden tests on every run; a nightly job regenerates goldens with the harness and fails on any diff
P1 · M1 · Task · deps: PROTO-0118, PROTO-0119 · labels: test
Done when: the nightly job is green on main and posts the diff on failure.

### [To Do] PROTO-0127 `docs/testing/golden-vectors.md`: regenerate, add a variant, read a mismatch report
P2 · M1 · Task · deps: PROTO-0126 · labels: docs
Done when: a new engineer can regenerate goldens by following the page.

## [To Do] PROTO-F11 Generated fuzz targets
P0 · M1 · Feature · deps: PROTO-F05, TEST-F07
Scope: One fuzz target per request type and header version, generated by codegen.

### [To Do] PROTO-0128 Codegen emits one cargo-fuzz target per request API: the fuzzer picks a valid version, the body is decoded, and on success encode → decode must be identical
P0 · M1 · Task · deps: PROTO-0082, PROTO-0112 · labels: security, test, decoder
Done when: targets exist for every request API, build in CI, are covered by the drift check, and run with RSS and malloc limits; no panic or hang in the CI budget.

### [To Do] PROTO-0129 Fuzz targets for every response type (inter-node clients decode responses from peers)
P1 · M1 · Task · deps: PROTO-0128 · labels: security, test, decoder
Done when: same oracle as request targets; generated and built in CI.

### [To Do] PROTO-0130 Fuzz targets for request header v1/v2 and response header v0/v1
P0 · M1 · Task · deps: PROTO-0092, PROTO-0112 · labels: security, test, decoder
Done when: targets decode headers with limits; no panic in the CI budget.

### [To Do] PROTO-0131 Fuzz targets for ConsumerProtocolAssignment, ConsumerProtocolSubscription and DefaultPrincipalData (control records are fuzzed by REC-F14)
P1 · M1 · Task · deps: PROTO-0080, PROTO-0112 · labels: security, test, decoder
Done when: targets decode version-prefixed payloads with limits; no panic in the CI budget.

### [To Do] PROTO-0132 Seed corpora for every generated target from PROTO-F10 golden vectors
P1 · M1 · Task · deps: PROTO-0128, PROTO-0129, PROTO-0130, PROTO-0117 · labels: test
Done when: each target's corpus contains the golden cases for its API; coverage of the first CI run is recorded.

### [To Do] PROTO-0133 Register generated targets with the TEST-F07 runner: short per-PR runs, long nightly runs, crash triage
P0 · M1 · Task · deps: PROTO-0128, PROTO-0130 · labels: test
Done when: new targets appear in CI automatically when codegen adds them.

### [To Do] PROTO-0134 Allocation oracle in fuzz builds: per-input allocation never exceeds the DecodeLimits budget
P0 · M1 · Task · deps: PROTO-0128, PROTO-0111 · labels: security, test
Done when: a counting allocator fails the target on budget overrun; a deliberate overrun fixture is caught.

## [In Progress] PROTO-F12 Generator testing
P1 · M1 · Feature · deps: PROTO-F04
Scope: Golden-output tests for the generator, drift check `cargo xtask codegen --check`.

### [Done] PROTO-0135 Vendor Kafka's generator test schemas (AllTypeMessage, NullableStructMessage, SimpleArraysMessage, SimpleExampleMessage, SimpleKeyedArraysMessage, SimpleRecordsMessage) as fixtures
P1 · M1 · Task · deps: none · labels: test · ref: clients/src/test/resources/common/message @4.4.0
Done when: fixtures are unmodified copies listed in the schema manifest and generate compiling code.

### [Done] PROTO-0136 Golden-output tests: generator output for each fixture compared with committed expected files (`cargo xtask codegen` refreshes them)
P1 · M1 · Task · deps: PROTO-0135 · labels: test
Done when: a change to generated code shows as a reviewable diff of the expected files.

### [Done] PROTO-0137 Negative schema fixtures: one per SchemaError variant, ported from Kafka's VersionsTest, StructSpecTest, StructRegistryTest, EntityTypeTest, MessageGeneratorTest and MessageDataGeneratorTest
P1 · M1 · Task · deps: PROTO-0135, PROTO-0024 · labels: test · ref: generator/src/test/java/org/apache/kafka/message/StructSpecTest.java @4.4.0
Done when: every SchemaError variant has a fixture asserting its message; attribution comments present.

### [Done] PROTO-0138 `cargo xtask codegen --check`: regenerate in memory, compare with committed output, print the first differing file and line, exit non-zero; run in `cargo xtask ci`
P0 · M1 · Task · deps: none · labels: test · ref: ADR-0008
Done when: hand-editing a generated file fails CI with a clear message; the check covers generated code, fuzz targets and generated docs.

### [Done] PROTO-0139 Determinism test: codegen output is independent of filesystem order and repeated runs, and contains no timestamps or absolute paths
P1 · M1 · Task · deps: PROTO-0138 · labels: test
Done when: two runs over shuffled input order produce identical bytes.

### [To Do] PROTO-0140 AllTypeMessage end-to-end: generated code for every field type round-trips across all fixture versions
P1 · M1 · Task · deps: PROTO-0135, PROTO-0082 · labels: test
Done when: round-trip and size tests pass for AllTypeMessage at every version.

## [To Do] PROTO-F13 Wire capture decoding
P2 · M2 · Feature · deps: PROTO-F06, CLI-F01
Scope: `alluvium dump frame`: decode hex dumps or pcap payloads of Kafka frames for debugging.

### [To Do] PROTO-0141 Generated JSON rendering of messages with Kafka's JsonConverter field names; records as size and CRC summary; sensitive fields redacted
P2 · M2 · Task · deps: none · ref: generator/src/main/java/org/apache/kafka/message/JsonConverterGenerator.java; server/src/main/java/org/apache/kafka/network/RequestConvertToJson.java @4.4.0
Done when: output for golden vectors equals Kafka's JsonConverter JSON after documented normalization of records and redacted fields; redaction tests.

### [To Do] PROTO-0142 Frame decoding library: size prefix, request header, body by API key and version; responses decoded given API key and version
P2 · M2 · Task · deps: PROTO-0141 · labels: decoder
Done when: errors report byte offset and field path; truncated frames are reported, not fatal; a fuzz target `frame_dump` runs clean.

### [To Do] PROTO-0143 Input parsing: raw binary, plain hex, `xxd` output and Wireshark hex streams
P2 · M2 · Task · deps: PROTO-0142
Done when: each format has fixture tests; malformed input yields an error with line and column.

### [To Do] PROTO-0144 pcap/pcapng input: TCP payload extraction, in-order stream reassembly per connection, Kafka frame splitting
P3 · M2 · Task · deps: PROTO-0142
Done when: a captured Java-client session decodes end to end; out-of-order or missing segments are reported; any new dependency passes the checklist.

### [To Do] PROTO-0145 Request/response pairing by correlation id within a connection so responses decode without `--api/--version`
P2 · M2 · Task · deps: PROTO-0143
Done when: a capture of an ApiVersions + Metadata + Produce exchange decodes every response automatically.

### [To Do] PROTO-0146 `alluvium dump frame` command: human output, `--json`, `--api/--version` overrides, documented exit codes
P2 · M2 · Task · deps: PROTO-0145 · labels: security
Done when: command tests over golden frames for every output mode; payloads are never printed.

### [To Do] PROTO-0147 `docs/operations/wire-captures.md`: capture with tcpdump/Wireshark and decode with `alluvium dump frame`
P2 · M2 · Task · deps: PROTO-0146 · labels: docs
Done when: the page includes a worked example and the redaction guarantees.

## [To Do] PROTO-F14 Version negotiation
P1 · M2 · Feature · deps: PROTO-F07
Scope: Pick the highest mutually supported version per peer from ApiVersions, for inter-node clients.

### [To Do] PROTO-0148 `NodeApiVersions` from an ApiVersionsResponse: per-API peer range, unknown peer API keys kept for diagnostics
P1 · M2 · Task · deps: PROTO-0125 · ref: clients/src/main/java/org/apache/kafka/clients/NodeApiVersions.java @4.4.0
Done when: built from the Kafka 4.4.0 broker and controller responses exported by the harness; unit tests.

### [To Do] PROTO-0149 Version selection `latest_usable_version(api, allowed)`: highest version in local ∩ peer ∩ allowed, else `UnsupportedVersion { api, local, peer }`
P1 · M2 · Task · deps: PROTO-0148 · ref: clients/src/main/java/org/apache/kafka/clients/NodeApiVersions.java latestUsableVersion @4.4.0
Done when: property: the result is within every range; an empty intersection is the typed error; NodeApiVersionsTest cases ported.

### [To Do] PROTO-0150 ApiVersions handshake state machine: send the highest local version; on UNSUPPORTED_VERSION with the peer's ApiVersions range retry at the highest common version (KIP-511)
P1 · M2 · Task · deps: PROTO-0149 · ref: clients/src/main/java/org/apache/kafka/clients/NetworkClient.java handleApiVersionsResponse @4.4.0
Done when: sans-I/O machine with inputs (response) and outputs (send version N, negotiated, failed); tests for a v0-3 peer and a peer returning no range.

### [To Do] PROTO-0151 Negotiation outcome with stable `Display` (peer node, chosen version per API) for the driver's negotiation event
P1 · M2 · Task · deps: PROTO-0150 · labels: observability
Done when: NET-F10 can log it as one structured event; snapshot test of the Display output.

### [To Do] PROTO-0152 `docs/protocol/version-negotiation.md`: rules, fallback and error cases
P2 · M2 · Task · deps: PROTO-0150 · labels: docs
Done when: the page links the KIP-511 fallback and the implemented-table rule.

## [To Do] PROTO-F15 Alluvium-internal API key range
P2 · M1 · Feature · deps: PROTO-F05 · labels: foundation-for:X-ELASTIC, foundation-for:X-DISKLESS, foundation-for:X-TENANT, foundation-for:X-AUTO
Scope: A reserved API key block and schema location for Alluvium-only inter-node APIs, which are never advertised to or accepted from Kafka clients.

### [To Do] PROTO-0153 Spike: reserve the internal API key block and schema location
P2 · M1 · Spike · deps: none
Done when: an accepted ADR fixes the reserved API key block (outside any range Kafka could plausibly use), the schema directory `schemas/alluvium/messages/`, versioning rules, and the rule that internal APIs are never listed in ApiVersions responses to Kafka clients.

### [To Do] PROTO-0154 Codegen reads Alluvium internal message schemas and rejects key collisions
P2 · M1 · Task · deps: PROTO-0153
Done when: `cargo xtask codegen` generates codecs for `schemas/alluvium/messages/` with the same rules as Kafka schemas; a schema whose key collides with a Kafka key or falls outside the reserved block fails generation with a clear error; golden tests cover one sample schema.

### [To Do] PROTO-0155 Router accepts internal APIs only on inter-node listeners from internal principals
P2 · M2 · Task · deps: PROTO-0154, SRV-F04 · labels: security
Done when: an internal API request on a client listener, or from a non-internal principal, gets the same response Kafka gives for an unknown API key and an `internal_api_rejected` event; tests cover both cases.

### [To Do] PROTO-0156 Prove Kafka clients see no internal APIs
P2 · M2 · Task · deps: PROTO-0155, TEST-F09 · labels: test
Done when: a differential test shows Alluvium's ApiVersions response to a Kafka client is identical to Kafka 4.4.0's apart from documented fields; fuzz targets exist for every internal schema.
