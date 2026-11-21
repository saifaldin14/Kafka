# X-SCHEMA: Built-in schema registry and validation

> Crates: `alluvium-schema` (new) · Kafka counterpart: none (Alluvium enhancement) · Milestones: M10

A built-in schema registry compatible with the Confluent Schema Registry REST API, plus broker-side validation that rejects records not matching registered schemas.

## [To Do] X-SCHEMA-F01 Schema registry design
P0 · M10 · Feature · deps: X-NATIVE-F01
Scope: Spike and ADR: API compatibility scope, storage model, id allocation.

### [To Do] X-SCHEMA-0001 Schema registry design spike: Confluent REST v1 compatibility scope, storage (compacted internal topic versus KRaft records), id allocation, consistency and listener placement
P0 · M10 · Spike · deps: none · labels: docs · ref: https://docs.confluent.io/platform/current/schema-registry/develop/api.html
Done when: an ADR "Built-in schema registry" is accepted with: the in-scope endpoint table (root, server metadata, schemas, subjects, versions, compatibility, config, mode, contexts) and out-of-scope list (exporters, importers, dek-registry, associations, tags, provenance) with reasons; the storage decision with record-size and replay-time estimates for 100 000 schemas; int32 ids per context that are never reused; read-your-writes consistency; supported record wire formats for broker validation; a dedicated HTTP listener with root paths.

### [To Do] X-SCHEMA-0002 Compatibility matrix docs/schema-registry/compatibility.md: every Confluent endpoint, query parameter, request field and error code with supported, partial or unsupported status
P0 · M10 · Task · deps: X-SCHEMA-0001 · labels: docs · ref: https://docs.confluent.io/platform/current/schema-registry/develop/api.html
Done when: the matrix covers every endpoint in the Confluent API reference and every code in Confluent's `Errors.java` (40401-50004); each partial entry explains the difference; it is linked from the ADR.

### [To Do] X-SCHEMA-0003 Dependency selection for Avro schemas and binary decoding, Protobuf schema parsing and descriptors, JSON Schema validation and a linear-time regex engine
P0 · M10 · Spike · deps: X-SCHEMA-0001 · labels: security · ref: docs/standards/security.md#dependencies
Done when: the five dependency-review questions are answered per candidate (for example apache-avro, protox, jsonschema, regex) in the ADR appendix; parsers that can recurse without a bound or allocate from input without limits are rejected or wrapped by a pre-validation pass, with the decision recorded.

### [To Do] X-SCHEMA-0004 Threat model rows for the registry: schema-parsing bombs per format, ReDoS through JSON Schema `pattern`, reference cycles and fan-out, subject enumeration, unauthenticated writes
P0 · M10 · Task · deps: X-SCHEMA-0001 · labels: security, docs · ref: docs/standards/security.md
Done when: docs/standards/security.md has one row per threat with its defense and implementing task id; the SEC-F14 checklist has a registry section.

## [To Do] X-SCHEMA-F02 Schema storage
P0 · M10 · Feature · deps: X-SCHEMA-F01, STOR-F14
Scope: Durable storage (compacted internal topic or metadata records), recovery, caching.

### [To Do] X-SCHEMA-0005 Internal topic `__alluvium_schemas`: created by the controller on the first registry write (1 partition, `cleanup.policy=compact`, replication factor `alluvium.schema.registry.topic.replication.factor`, `min.insync.replicas=2`)
P0 · M10 · Task · deps: CTRL-F04 · ref: ADR Built-in schema registry (X-SCHEMA-0001)
Done when: an enabled-but-unused registry creates no topic; Kafka clients cannot produce to, delete or alter the topic (tests per API); creation failure surfaces as error 50001 with a logged cause.

### [To Do] X-SCHEMA-0006 Record key format v1: typed keys SCHEMA(context, subject, version), CONFIG(scope), MODE(scope), DELETE_SUBJECT, CLEAR_SUBJECT, CONTEXT and ID_ALLOCATION, versioned binary encoding
P0 · M10 · Task · deps: none · labels: decoder · ref: core/src/main/java/io/confluent/kafka/schemaregistry/storage/SchemaRegistryKeyType.java (confluentinc/schema-registry)
Done when: encode and decode round-trip under proptest; the decoder bounds every length before allocating and returns typed errors with the byte offset; unknown key types are rejected, not skipped.

### [To Do] X-SCHEMA-0007 Record value format v1: schema (id, guid, type, schema text, references, metadata, ruleSet, deleted flag, registration timestamp), config and mode values; tombstones for hard deletes
P0 · M10 · Task · deps: none · labels: decoder · ref: ADR Built-in schema registry (X-SCHEMA-0001)
Done when: round-trip property tests pass; values above `alluvium.schema.registry.schema.max.bytes` plus overhead are rejected at decode; a tombstone decodes to a typed delete.

### [To Do] X-SCHEMA-0008 Fuzz targets and golden encodings for `__alluvium_schemas` keys and values
P0 · M10 · Task · deps: X-SCHEMA-0006, X-SCHEMA-0007, TEST-F07 · labels: test, security, decoder · ref: TEST-F07
Done when: `fuzz/fuzz_targets/schema_registry_key.rs` and `schema_registry_value.rs` run per PR; golden files for each record type are committed and checked.

### [To Do] X-SCHEMA-0009 Format documentation docs/formats/schemas-topic.md with hex examples and version rules
P1 · M10 · Task · deps: X-SCHEMA-0006, X-SCHEMA-0007 · labels: docs · ref: STOR-F19
Done when: every key and value type has a hex example generated by a test; the evolution rule for format v2 is stated.

### [To Do] X-SCHEMA-0010 Registry state machine (sans-I/O) in `alluvium-schema`: subjects to versions, id to schema, guid to id, canonical-hash index and referencedBy index, with invariant checks on apply
P0 · M10 · Task · deps: none · ref: core/src/main/java/io/confluent/kafka/schemaregistry/storage/KafkaSchemaRegistry.java (confluentinc/schema-registry)
Done when: the crate is registered in the layer table; `apply(record)` updates all indexes; `debug_assert!` invariants (ids unique per context, versions increasing per subject) hold in unit tests; no I/O or clock reads.

### [To Do] X-SCHEMA-0011 Replay: rebuild state from `__alluvium_schemas` from the log start offset; a compacted log replays to the same state as the uncompacted log
P0 · M10 · Task · deps: X-SCHEMA-0010, X-SCHEMA-0006, X-SCHEMA-0007 · ref: STOR-F14
Done when: a property test generates command sequences, compacts the resulting log with the STOR-F14 cleaner rules and compares states; replay of corrupt records stops with a typed error naming the offset.

### [To Do] X-SCHEMA-0012 Registry writer on the leader of `__alluvium_schemas-0` using the GRP-F01 coordinator runtime: commands to records, completion at the high watermark, fencing on leader-epoch change
P0 · M10 · Task · deps: X-SCHEMA-0010, GRP-F01, GRP-F10 · ref: GRP-F01
Done when: a registration completes only after its record is committed; a writer that loses leadership fails pending commands with 50004; the new leader loads state before accepting writes (GRP-F10 loading errors map to 50004).

### [To Do] X-SCHEMA-0013 Write forwarding: brokers that are not the writer forward write requests over the inter-node client
P0 · M10 · Task · deps: X-SCHEMA-0012, NET-F10 · ref: core/src/main/java/io/confluent/kafka/schemaregistry/rest/exceptions/Errors.java (confluentinc/schema-registry)
Done when: writes succeed through any broker; forwarding failures return 50003 and an unknown writer returns 50004; the forwarded request keeps the original principal for authorization and audit.

### [To Do] X-SCHEMA-0014 Read replicas on every broker: tail the topic into a local state copy; reads after a write wait until local replay reaches the write's offset
P0 · M10 · Task · deps: X-SCHEMA-0011 · ref: ADR Built-in schema registry (X-SCHEMA-0001)
Done when: a client writing through broker A and reading through broker B sees its write (test); the wait is bounded by `alluvium.schema.registry.read.wait.ms` and times out with 50002; replay lag is exposed for X-SCHEMA-F14.

### [To Do] X-SCHEMA-0015 Crash-consistency and failover tests: writer crash between append and high watermark, leader change during registration
P0 · M10 · Task · deps: X-SCHEMA-0012, X-SCHEMA-0014, TEST-F03, TEST-F06 · labels: test · ref: TEST-F06
Done when: simulation over 1000 seeds shows no id issued twice, no acknowledged registration lost and replicas converging; failing seeds replay with `ALLUVIUM_SIM_SEED`.

## [To Do] X-SCHEMA-F03 Subjects and versions
P0 · M10 · Feature · deps: X-SCHEMA-F02
Scope: Subjects, versions, global ids, soft and hard deletes, naming strategies.

### [To Do] X-SCHEMA-0016 Subject names and qualified-subject parser `:.<context>:<subject>` with default context `.`, length and character validation
P0 · M10 · Task · deps: none · labels: security · ref: client/src/main/java/io/confluent/kafka/schemaregistry/utils/QualifiedSubject.java (confluentinc/schema-registry)
Done when: invalid names return 42208 INVALID_SUBJECT; the parser has a fuzz target; tests port Confluent's QualifiedSubject cases (with attribution).

### [To Do] X-SCHEMA-0017 Schema id allocation: monotonically increasing int32 per context, never reused after hard delete, persisted with ID_ALLOCATION records
P0 · M10 · Task · deps: none · ref: core/src/main/java/io/confluent/kafka/schemaregistry/id/IncrementalIdGenerator.java (confluentinc/schema-registry)
Done when: property test: ids are unique and increasing across arbitrary register and delete sequences; exhaustion of int32 returns 50001 and emits an event; allocation survives writer failover (test).

### [To Do] X-SCHEMA-0018 Schema GUIDs: a UUID per distinct schema from the injected RNG, stable across subjects and contexts that share the schema
P0 · M10 · Task · deps: X-SCHEMA-0017 · ref: https://docs.confluent.io/platform/current/schema-registry/develop/api.html
Done when: the same schema registered twice keeps its guid; guids are returned only with `Confluent-Accept-Unknown-Properties: true`; simulation runs produce identical guids for identical seeds.

### [To Do] X-SCHEMA-0019 Register semantics: an identical schema (after normalization when requested) under a subject returns the existing id and version; the same schema in another subject reuses the id
P0 · M10 · Task · deps: X-SCHEMA-0017 · ref: core/src/main/java/io/confluent/kafka/schemaregistry/storage/KafkaSchemaRegistry.java register (confluentinc/schema-registry)
Done when: tests cover identical text, whitespace-only differences (not identical without normalize), differing references and differing metadata; no new version is created for duplicates.

### [To Do] X-SCHEMA-0020 Version assignment: next version is the highest version plus one (counting soft-deleted versions); explicit `version` only in IMPORT mode; `version: -1` forces a new latest copy of an existing schema
P0 · M10 · Task · deps: X-SCHEMA-0019 · ref: https://docs.confluent.io/platform/current/schema-registry/develop/api.html
Done when: tests cover each rule; versions outside [1, 2^31-1] return 42202 INVALID_VERSION.

### [To Do] X-SCHEMA-0021 Soft delete of a subject: all versions marked deleted and returned; the subject is hidden unless `deleted=true`; a repeat soft delete returns 40404
P0 · M10 · Task · deps: X-SCHEMA-0020 · ref: core/src/main/java/io/confluent/kafka/schemaregistry/rest/resources/SubjectsResource.java deleteSubject (confluentinc/schema-registry)
Done when: tests cover listing with `deleted` and `deletedOnly`, and lookup of soft-deleted versions with `deleted=true`.

### [To Do] X-SCHEMA-0022 Hard delete of a subject (`permanent=true`): requires a prior soft delete (40405 otherwise), writes tombstones, removes subject config and mode overrides
P0 · M10 · Task · deps: X-SCHEMA-0021 · ref: core/src/main/java/io/confluent/kafka/schemaregistry/rest/resources/SubjectsResource.java deleteSubject (confluentinc/schema-registry)
Done when: after hard delete the subject cannot be found even with `deleted=true`; its ids are never reallocated (test); a schema still used by another subject remains retrievable by id.

### [To Do] X-SCHEMA-0023 Version deletes: soft delete (including `latest`) and permanent delete (explicit version only, 40407 when not soft-deleted first; `latest` with `permanent=true` performs only a soft delete)
P0 · M10 · Task · deps: X-SCHEMA-0021 · ref: core/src/main/java/io/confluent/kafka/schemaregistry/rest/resources/SubjectVersionsResource.java deleteSchemaVersion (confluentinc/schema-registry)
Done when: tests cover each rule and 40406 for an already soft-deleted version.

### [To Do] X-SCHEMA-0024 Re-registration after soft delete: registering a soft-deleted schema creates a new version with the original id
P0 · M10 · Task · deps: X-SCHEMA-0021 · ref: https://docs.confluent.io/platform/current/schema-registry/schema-deletion-guidelines.html
Done when: tests match Confluent's documented behaviour for subject-level and version-level soft deletes.

### [To Do] X-SCHEMA-0025 Schema `metadata` and `ruleSet`: stored and returned verbatim with size limits, included in schema identity as Confluent does; no server-side rule execution
P1 · M10 · Task · deps: X-SCHEMA-0019 · ref: https://docs.confluent.io/platform/current/schema-registry/fundamentals/data-contracts.html
Done when: round-trip tests preserve both objects; oversized objects return 42210 or 42201 with the field named; the docs state that rules run in client serializers only.

### [To Do] X-SCHEMA-0026 Subject naming strategies as a shared resolver: TopicNameStrategy (`<topic>-key`, `<topic>-value`), RecordNameStrategy (fully qualified record name), TopicRecordNameStrategy (`<topic>-<record name>`)
P0 · M10 · Task · deps: X-SCHEMA-0016 · ref: schema-serializer/src/main/java/io/confluent/kafka/serializers/subject/TopicNameStrategy.java (confluentinc/schema-registry)
Done when: table tests match Confluent's strategies for Avro, Protobuf and JSON Schema record names; the resolver is used by broker-side validation and documented.

### [To Do] X-SCHEMA-0027 Registry limits: `alluvium.schema.registry.schema.max.bytes` (42209 SCHEMA_TOO_LARGE), `alluvium.schema.registry.max.subjects` and `alluvium.schema.registry.max.versions.per.subject`
P0 · M10 · Task · deps: X-SCHEMA-0020 · labels: security · ref: core/src/main/java/io/confluent/kafka/schemaregistry/rest/exceptions/Errors.java (confluentinc/schema-registry)
Done when: each limit is enforced before parsing or writing; keys are registered with defaults and docs; tests cover each boundary.

### [To Do] X-SCHEMA-0028 Property tests: arbitrary register and delete sequences keep the invariants (ids unique, versions monotonic per subject, hard-deleted ids never reissued, soft-deleted versions recoverable)
P0 · M10 · Task · deps: X-SCHEMA-0022, X-SCHEMA-0023, X-SCHEMA-0024, X-SCHEMA-0018, X-SCHEMA-0027 · labels: test · ref: TEST-F16
Done when: the proptest model compares the state machine to a simple reference model over 10 000 cases.

## [To Do] X-SCHEMA-F04 Avro support
P0 · M10 · Feature · deps: X-SCHEMA-F03
Scope: Parsing, canonical form, normalization, compatibility rules.

### [To Do] X-SCHEMA-0029 Avro schema parser: primitives, records, enums, arrays, maps, unions, fixed and logical types, names and namespaces, recursive named references, with depth and size limits
P0 · M10 · Task · deps: none · labels: decoder, security · ref: https://avro.apache.org/docs/1.12.0/specification/
Done when: invalid schemas return 42201 with a message naming the path; depth over `alluvium.schema.registry.max.depth` is rejected without stack overflow; tests port Confluent AvroSchema parse cases.

### [To Do] X-SCHEMA-0030 Avro default validation: field defaults must match the field type (union defaults match the first branch), as Confluent validates them
P0 · M10 · Task · deps: X-SCHEMA-0029 · ref: client/src/main/java/io/confluent/kafka/schemaregistry/avro/AvroSchema.java (confluentinc/schema-registry)
Done when: tests cover defaults for every type, including nested records and logical types.

### [To Do] X-SCHEMA-0031 Avro canonical form and normalization: Parsing Canonical Form fingerprint for identity, and `normalize=true` matching Confluent's AvroSchema normalization
P0 · M10 · Task · deps: X-SCHEMA-0029 · ref: https://avro.apache.org/docs/1.12.0/specification/#parsing-canonical-form-for-schemas
Done when: golden tests from Confluent fixtures produce identical normalized text; fingerprints are stable across runs.

### [To Do] X-SCHEMA-0032 `format=resolved` output: referenced named types inlined
P1 · M10 · Task · deps: X-SCHEMA-0029 · ref: https://docs.confluent.io/platform/current/schema-registry/develop/api.html
Done when: output equals Confluent's resolved format for fixtures with references (requires X-SCHEMA-F08 resolution in tests through a stub).

### [To Do] X-SCHEMA-0033 Fuzz target for the Avro schema parser with a corpus from Confluent and Avro test schemas
P0 · M10 · Task · deps: X-SCHEMA-0029, TEST-F07 · labels: test, security, decoder · ref: TEST-F07
Done when: the target runs per PR; deeply nested and huge-name inputs are in the corpus.

### [To Do] X-SCHEMA-0034 Avro binary decoder with writer-to-reader schema resolution, bounded allocation and a fuzz target
P0 · M10 · Task · deps: X-SCHEMA-0029 · labels: decoder, security · ref: https://avro.apache.org/docs/1.12.0/specification/#schema-resolution
Done when: decoding rejects lengths beyond the remaining bytes before allocating; trailing bytes are reported; the fuzz target runs per PR; used by FULL validation and compatibility property tests.

### [To Do] X-SCHEMA-0035 Avro compatibility rule: primitive promotions (int to long, float or double; long to float or double; float to double; string to and from bytes); other primitive changes are TYPE_MISMATCH
P0 · M10 · Task · deps: X-SCHEMA-0029 · ref: https://avro.apache.org/docs/1.12.0/specification/#schema-resolution
Done when: a table test covers every primitive pair in both directions; property test: data written with the writer decodes with the reader whenever the rule passes.

### [To Do] X-SCHEMA-0036 Avro compatibility rule: record fields; reader fields missing from the writer need defaults (READER_FIELD_MISSING_DEFAULT_VALUE), writer-only fields are skipped, field aliases match
P0 · M10 · Task · deps: X-SCHEMA-0029 · ref: https://avro.apache.org/docs/1.12.0/specification/#schema-resolution
Done when: tests cover added and removed fields with and without defaults and aliases; property test with generated records passes.

### [To Do] X-SCHEMA-0037 Avro compatibility rule: named types (records, enums, fixed) must match by full name or alias (NAME_MISMATCH)
P0 · M10 · Task · deps: X-SCHEMA-0029 · ref: https://avro.apache.org/docs/1.12.0/specification/#aliases
Done when: tests cover namespace changes, aliases on reader types and unqualified aliases.

### [To Do] X-SCHEMA-0038 Avro compatibility rule: enums; writer symbols missing from the reader fail (MISSING_ENUM_SYMBOLS) unless the reader declares an enum default
P0 · M10 · Task · deps: X-SCHEMA-0029 · ref: https://avro.apache.org/docs/1.12.0/specification/#enums
Done when: tests cover added and removed symbols with and without a default in both directions.

### [To Do] X-SCHEMA-0039 Avro compatibility rule: unions; every writer branch must resolve to a reader branch (MISSING_UNION_BRANCH); union-to-non-union and non-union-to-union resolution
P0 · M10 · Task · deps: X-SCHEMA-0029 · ref: https://avro.apache.org/docs/1.12.0/specification/#schema-resolution
Done when: tests cover nullable promotion, branch reordering, and nested unions; property test passes.

### [To Do] X-SCHEMA-0040 Avro compatibility rule: fixed sizes must match (FIXED_SIZE_MISMATCH)
P0 · M10 · Task · deps: X-SCHEMA-0029 · ref: https://avro.apache.org/docs/1.12.0/specification/#schema-resolution
Done when: tests cover size changes and fixed with logical types (decimal, duration).

### [To Do] X-SCHEMA-0041 Avro compatibility rule: arrays and maps recurse into items and values; recursive schemas terminate through a visited set of (reader, writer) pairs
P0 · M10 · Task · deps: X-SCHEMA-0029 · ref: https://avro.apache.org/docs/1.12.0/specification/#schema-resolution
Done when: tests cover nested containers and a self-referencing linked-list schema without infinite recursion.

### [To Do] X-SCHEMA-0042 Avro compatibility rule: logical types are checked on their underlying types, matching Confluent's behaviour
P1 · M10 · Task · deps: X-SCHEMA-0029 · ref: https://avro.apache.org/docs/1.12.0/specification/#logical-types
Done when: tests show decimal precision changes and timestamp-millis to timestamp-micros changes report the same result as Confluent.

### [To Do] X-SCHEMA-0043 Verbose Avro incompatibility messages in Confluent's format (`{errorType:"…", description:"…", additionalInfo:"…"}`) with the failing path
P0 · M10 · Task · deps: X-SCHEMA-0035, X-SCHEMA-0036, X-SCHEMA-0037, X-SCHEMA-0038, X-SCHEMA-0039, X-SCHEMA-0040, X-SCHEMA-0041 · ref: client/src/main/java/io/confluent/kafka/schemaregistry/avro/Difference.java (confluentinc/schema-registry)
Done when: golden tests for each incompatibility type compare message text with Confluent's for the same pair.

### [To Do] X-SCHEMA-0044 Avro compatibility property test: if the checker reports writer-to-reader compatible, random data encoded with the writer decodes with the reader
P0 · M10 · Task · deps: X-SCHEMA-0043, X-SCHEMA-0034 · labels: test · ref: TEST-F16
Done when: a schema-pair generator covers every type; 10 000 cases pass; counterexamples are persisted in `proptest-regressions/`.

### [To Do] X-SCHEMA-0045 Differential test against a pinned Confluent Schema Registry container: `is_compatible` agrees for a corpus of 500 Avro schema pairs
P1 · M10 · Task · deps: X-SCHEMA-0043, TEST-F08 · labels: test · ref: TEST-F08
Done when: the test runs in `cargo xtask compat`; every disagreement is either fixed or listed in the compatibility matrix with a reason.

## [To Do] X-SCHEMA-F05 Protobuf support
P0 · M10 · Feature · deps: X-SCHEMA-F03
Scope: Parsing, references, normalization, compatibility rules.

### [To Do] X-SCHEMA-0046 Protobuf schema parser: proto2 and proto3 syntax, imports, options, nested messages and enums, maps and oneofs, with depth and size limits
P0 · M10 · Task · deps: none · labels: decoder, security · ref: protobuf-provider/src/main/java/io/confluent/kafka/schemaregistry/protobuf/ProtobufSchema.java (confluentinc/schema-registry)
Done when: invalid schemas return 42201 with line and column; limits are enforced; tests port Confluent ProtobufSchema parse cases.

### [To Do] X-SCHEMA-0047 Bundled imports: `google/protobuf/*.proto` well-known types and `confluent/meta.proto` and `confluent/type/decimal.proto` resolve without references
P0 · M10 · Task · deps: X-SCHEMA-0046 · ref: https://protobuf.dev/reference/protobuf/google.protobuf/
Done when: bundled files are vendored with licenses; a schema importing each resolves; other unresolved imports return 42201 naming the import.

### [To Do] X-SCHEMA-0048 Descriptor output: FileDescriptorProto generation, `format=serialized` (base64 descriptor) and `format=ignore_extensions`
P0 · M10 · Task · deps: X-SCHEMA-0046 · ref: https://docs.confluent.io/platform/current/schema-registry/develop/api.html
Done when: serialized output decodes with protoc's descriptor parser in a test; ignore_extensions output matches Confluent fixtures.

### [To Do] X-SCHEMA-0049 Protobuf normalization and canonical text output matching Confluent's ProtobufSchema normalization
P0 · M10 · Task · deps: X-SCHEMA-0046 · ref: protobuf-provider/src/main/java/io/confluent/kafka/schemaregistry/protobuf/ProtobufSchema.java normalize (confluentinc/schema-registry)
Done when: golden tests from Confluent fixtures produce identical normalized text; identity uses the canonical text.

### [To Do] X-SCHEMA-0050 Message indexes: path of a message type within the file encoded as a zigzag-varint array with the single-byte `0` shorthand
P0 · M10 · Task · deps: X-SCHEMA-0046 · labels: decoder · ref: schema-serializer/src/main/java/io/confluent/kafka/serializers/schema/id/SchemaId.java (confluentinc/schema-registry)
Done when: encode and decode round-trip; indexes resolve to message descriptors; invalid indexes return typed errors; a fuzz target covers the decoder.

### [To Do] X-SCHEMA-0051 Fuzz target for the Protobuf schema parser with a corpus from Confluent fixtures
P0 · M10 · Task · deps: X-SCHEMA-0046, TEST-F07 · labels: test, security, decoder · ref: TEST-F07
Done when: the target runs per PR; deeply nested messages and import cycles are in the corpus.

### [To Do] X-SCHEMA-0052 Protobuf payload decoder against a descriptor: bounded, unknown fields allowed, wire-type checks, nesting limit
P0 · M10 · Task · deps: X-SCHEMA-0050 · labels: decoder, security · ref: https://protobuf.dev/programming-guides/encoding/
Done when: malformed varints, truncated length-delimited fields and wrong wire types return typed errors; a fuzz target runs per PR; used by FULL validation and property tests.

### [To Do] X-SCHEMA-0053 Protobuf compatibility rule: PACKAGE_CHANGED is incompatible
P0 · M10 · Task · deps: X-SCHEMA-0046 · ref: protobuf-provider/src/main/java/io/confluent/kafka/schemaregistry/protobuf/diff/Difference.java (confluentinc/schema-registry)
Done when: tests cover package rename and removal in both directions.

### [To Do] X-SCHEMA-0054 Protobuf compatibility rules for messages: MESSAGE_ADDED (compatible), MESSAGE_REMOVED and MESSAGE_MOVED (incompatible)
P0 · M10 · Task · deps: X-SCHEMA-0046 · ref: protobuf-provider/src/main/java/io/confluent/kafka/schemaregistry/protobuf/diff/MessageSchemaDiff.java (confluentinc/schema-registry)
Done when: table tests cover top-level and nested messages for every compatibility direction.

### [To Do] X-SCHEMA-0055 Protobuf compatibility rules for enums: ENUM_ADDED, ENUM_REMOVED, ENUM_CONST_ADDED, ENUM_CONST_CHANGED and ENUM_CONST_REMOVED
P0 · M10 · Task · deps: X-SCHEMA-0046 · ref: protobuf-provider/src/main/java/io/confluent/kafka/schemaregistry/protobuf/diff/EnumSchemaDiff.java (confluentinc/schema-registry)
Done when: table tests classify each change as Confluent does; payload property test confirms compatible changes decode.

### [To Do] X-SCHEMA-0056 Protobuf compatibility rules for field presence: FIELD_ADDED, FIELD_REMOVED, FIELD_NAME_CHANGED, REQUIRED_FIELD_ADDED and REQUIRED_FIELD_REMOVED
P0 · M10 · Task · deps: X-SCHEMA-0046 · ref: protobuf-provider/src/main/java/io/confluent/kafka/schemaregistry/protobuf/diff/FieldSchemaDiff.java (confluentinc/schema-registry)
Done when: table tests cover proto2 required fields and proto3 fields, reserved numbers and renames.

### [To Do] X-SCHEMA-0057 Protobuf compatibility rules for field types: FIELD_KIND_CHANGED, FIELD_SCALAR_KIND_CHANGED (wire-compatible groups), FIELD_NAMED_TYPE_CHANGED, FIELD_NUMERIC_LABEL_CHANGED and FIELD_STRING_OR_BYTES_LABEL_CHANGED
P0 · M10 · Task · deps: X-SCHEMA-0046 · ref: protobuf-provider/src/main/java/io/confluent/kafka/schemaregistry/protobuf/diff/FieldSchemaDiff.java (confluentinc/schema-registry)
Done when: a table test covers every scalar pair (int32, uint32, int64, uint64 and bool; sint32 and sint64; fixed32 and sfixed32; fixed64 and sfixed64; string and bytes) and label changes; property test decodes old payloads after compatible changes.

### [To Do] X-SCHEMA-0058 Protobuf compatibility rules for oneofs: ONEOF_ADDED, ONEOF_REMOVED, ONEOF_FIELD_ADDED, ONEOF_FIELD_REMOVED, ONEOF_FIELD_MOVED_TO_TOP_LEVEL, MULTIPLE_FIELDS_MOVED_TO_ONEOF and FIELD_MOVED_TO_EXISTING_ONEOF
P0 · M10 · Task · deps: X-SCHEMA-0046 · ref: protobuf-provider/src/main/java/io/confluent/kafka/schemaregistry/protobuf/diff/MessageSchemaDiff.java (confluentinc/schema-registry)
Done when: table tests classify each change as Confluent does, including moving a single field into a new oneof (compatible).

### [To Do] X-SCHEMA-0059 Verbose Protobuf incompatibility messages with the difference type and full path, matching Confluent's text
P0 · M10 · Task · deps: X-SCHEMA-0053, X-SCHEMA-0054, X-SCHEMA-0055, X-SCHEMA-0056, X-SCHEMA-0057, X-SCHEMA-0058 · ref: protobuf-provider/src/main/java/io/confluent/kafka/schemaregistry/protobuf/diff/Difference.java (confluentinc/schema-registry)
Done when: golden tests compare messages for each difference type.

### [To Do] X-SCHEMA-0060 Protobuf compatibility property test: payloads written with the old schema decode with the new schema whenever the checker reports compatibility
P0 · M10 · Task · deps: X-SCHEMA-0059, X-SCHEMA-0052 · labels: test · ref: TEST-F16
Done when: 10 000 generated schema-change cases pass; regressions are persisted.

### [To Do] X-SCHEMA-0061 Differential test against a pinned Confluent Schema Registry container for 500 Protobuf schema pairs
P1 · M10 · Task · deps: X-SCHEMA-0059, TEST-F08 · labels: test · ref: TEST-F08
Done when: `is_compatible` and verbose difference types agree, or disagreements are documented in the compatibility matrix.

## [To Do] X-SCHEMA-F06 JSON Schema support
P0 · M10 · Feature · deps: X-SCHEMA-F03
Scope: Drafts, references, compatibility rules.

### [To Do] X-SCHEMA-0062 JSON Schema parser: drafts 4, 6, 7, 2019-09 and 2020-12 selected by `$schema` (default draft 7), keyword model, depth and size limits
P0 · M10 · Task · deps: none · labels: decoder, security · ref: https://json-schema.org/specification
Done when: invalid schemas return 42201 with a JSON pointer; unknown `$schema` values are rejected; limits are enforced without recursion overflow.

### [To Do] X-SCHEMA-0063 Local `$ref` resolution: `#/definitions`, `$defs`, `$id` base URIs and anchors, with cycle detection and a resolution budget
P0 · M10 · Task · deps: X-SCHEMA-0062 · labels: security · ref: https://json-schema.org/draft/2020-12/json-schema-core#name-schema-references
Done when: recursive schemas validate without infinite loops; unresolvable local references return 42201; tests cover draft differences.

### [To Do] X-SCHEMA-0064 Payload validator for FULL validation: all assertion keywords of the supported drafts, `format` as annotation, a linear-time engine for `pattern`, bounded evaluation steps
P0 · M10 · Task · deps: X-SCHEMA-0063 · labels: security · ref: https://json-schema.org/draft/2020-12/json-schema-validation
Done when: the official JSON-Schema-Test-Suite passes for supported drafts (skips listed); a catastrophic-backtracking pattern runs in linear time; evaluation over budget returns a typed error.

### [To Do] X-SCHEMA-0065 JSON Schema normalization and canonical form matching Confluent's JsonSchema normalization
P1 · M10 · Task · deps: X-SCHEMA-0062 · ref: json-schema-provider/src/main/java/io/confluent/kafka/schemaregistry/json/JsonSchema.java (confluentinc/schema-registry)
Done when: golden tests from Confluent fixtures produce identical normalized text.

### [To Do] X-SCHEMA-0066 Fuzz targets for the JSON Schema parser and validator
P0 · M10 · Task · deps: X-SCHEMA-0062, X-SCHEMA-0064, TEST-F07 · labels: test, security, decoder · ref: TEST-F07
Done when: targets run per PR with a corpus from the JSON-Schema-Test-Suite.

### [To Do] X-SCHEMA-0067 JSON Schema compatibility rule: types (TYPE_EXTENDED, TYPE_NARROWED, TYPE_CHANGED)
P0 · M10 · Task · deps: X-SCHEMA-0062 · ref: json-schema-provider/src/main/java/io/confluent/kafka/schemaregistry/json/diff/SchemaDiff.java (confluentinc/schema-registry)
Done when: table tests classify each change per direction as Confluent does.

### [To Do] X-SCHEMA-0068 JSON Schema compatibility rules for objects: open and closed content models, properties added or removed, required properties, additionalProperties, min and max properties, dependencies
P0 · M10 · Task · deps: X-SCHEMA-0062 · ref: json-schema-provider/src/main/java/io/confluent/kafka/schemaregistry/json/diff/ObjectSchemaDiff.java (confluentinc/schema-registry)
Done when: table tests cover every object difference type in Confluent's Difference enum.

### [To Do] X-SCHEMA-0069 JSON Schema compatibility rules for arrays: items, additionalItems, min and max items, uniqueItems, min and max contains
P0 · M10 · Task · deps: X-SCHEMA-0062 · ref: json-schema-provider/src/main/java/io/confluent/kafka/schemaregistry/json/diff/ArraySchemaDiff.java (confluentinc/schema-registry)
Done when: table tests cover every array difference type.

### [To Do] X-SCHEMA-0070 JSON Schema compatibility rules for numbers and strings: minimum, maximum, exclusive bounds, multipleOf, min and max length, pattern
P0 · M10 · Task · deps: X-SCHEMA-0062 · ref: json-schema-provider/src/main/java/io/confluent/kafka/schemaregistry/json/diff/NumberSchemaDiff.java (confluentinc/schema-registry)
Done when: table tests cover every numeric and string difference type, including pattern changes treated as Confluent does.

### [To Do] X-SCHEMA-0071 JSON Schema compatibility rules for enum and const
P0 · M10 · Task · deps: X-SCHEMA-0062 · ref: json-schema-provider/src/main/java/io/confluent/kafka/schemaregistry/json/diff/EnumSchemaDiff.java (confluentinc/schema-registry)
Done when: table tests cover enum values added and removed and const changes.

### [To Do] X-SCHEMA-0072 JSON Schema compatibility rules for combinators: oneOf, anyOf and allOf (sum and product types), and not
P0 · M10 · Task · deps: X-SCHEMA-0062 · ref: json-schema-provider/src/main/java/io/confluent/kafka/schemaregistry/json/diff/CombinedSchemaDiff.java (confluentinc/schema-registry)
Done when: table tests cover COMBINED_TYPE, SUM_TYPE, PRODUCT_TYPE and NOT_TYPE differences.

### [To Do] X-SCHEMA-0073 JSON Schema compatibility rules for unevaluatedProperties and unevaluatedItems (2019-09 and later)
P1 · M10 · Task · deps: X-SCHEMA-0062 · ref: json-schema-provider/src/main/java/io/confluent/kafka/schemaregistry/json/diff/Difference.java (confluentinc/schema-registry)
Done when: table tests cover each difference type for drafts that support the keywords.

### [To Do] X-SCHEMA-0074 Verbose JSON Schema incompatibility messages with difference type and JSON pointer, matching Confluent's text
P0 · M10 · Task · deps: X-SCHEMA-0067, X-SCHEMA-0068, X-SCHEMA-0069, X-SCHEMA-0070, X-SCHEMA-0071, X-SCHEMA-0072 · ref: json-schema-provider/src/main/java/io/confluent/kafka/schemaregistry/json/diff/Difference.java (confluentinc/schema-registry)
Done when: golden tests compare messages per difference type.

### [To Do] X-SCHEMA-0075 JSON Schema compatibility property test: documents valid under the writer schema validate under the reader whenever the checker reports compatibility
P0 · M10 · Task · deps: X-SCHEMA-0074, X-SCHEMA-0064 · labels: test · ref: TEST-F16
Done when: 10 000 generated cases pass; regressions persisted.

### [To Do] X-SCHEMA-0076 Differential test against a pinned Confluent Schema Registry container for 500 JSON Schema pairs
P1 · M10 · Task · deps: X-SCHEMA-0074, TEST-F08 · labels: test · ref: TEST-F08
Done when: results agree or disagreements are documented in the compatibility matrix.

## [To Do] X-SCHEMA-F07 Compatibility checking
P0 · M10 · Feature · deps: X-SCHEMA-F04, X-SCHEMA-F05, X-SCHEMA-F06
Scope: BACKWARD, FORWARD, FULL (and TRANSITIVE variants), NONE, per subject and global.

### [To Do] X-SCHEMA-0077 Compatibility level model and resolution: subject config, then context default, then global, then `alluvium.schema.registry.compatibility.default` (BACKWARD)
P0 · M10 · Task · deps: none · ref: https://docs.confluent.io/platform/current/schema-registry/fundamentals/schema-evolution.html
Done when: invalid levels return 42203; resolution is unit-tested for each layer; `defaultToGlobal` semantics are supported for reads.

### [To Do] X-SCHEMA-0078 Format-agnostic checker: dispatch by schema type, reader and writer direction, candidate selection (latest non-deleted version or all non-deleted versions)
P0 · M10 · Task · deps: X-SCHEMA-0077 · ref: client/src/main/java/io/confluent/kafka/schemaregistry/CompatibilityChecker.java (confluentinc/schema-registry)
Done when: soft-deleted versions are excluded from candidates; a schema type change across versions is incompatible unless the level is NONE; unit tests cover each candidate rule.

### [To Do] X-SCHEMA-0079 BACKWARD and BACKWARD_TRANSITIVE: the new schema reads data written with the latest, or every previous, version
P0 · M10 · Task · deps: X-SCHEMA-0078 · ref: client/src/main/java/io/confluent/kafka/schemaregistry/CompatibilityChecker.java (confluentinc/schema-registry)
Done when: tests for each format show a change compatible with the latest but not an older version passes BACKWARD and fails BACKWARD_TRANSITIVE.

### [To Do] X-SCHEMA-0080 FORWARD and FORWARD_TRANSITIVE: the latest, or every previous, version reads data written with the new schema
P0 · M10 · Task · deps: X-SCHEMA-0078 · ref: client/src/main/java/io/confluent/kafka/schemaregistry/CompatibilityChecker.java (confluentinc/schema-registry)
Done when: tests for each format cover the transitive distinction.

### [To Do] X-SCHEMA-0081 FULL and FULL_TRANSITIVE: both directions against the latest, or every previous, version
P0 · M10 · Task · deps: X-SCHEMA-0079, X-SCHEMA-0080 · ref: client/src/main/java/io/confluent/kafka/schemaregistry/CompatibilityChecker.java (confluentinc/schema-registry)
Done when: tests for each format show FULL equals BACKWARD and FORWARD combined.

### [To Do] X-SCHEMA-0082 NONE: compatibility checks skipped, schema type changes allowed
P0 · M10 · Task · deps: X-SCHEMA-0078 · ref: https://docs.confluent.io/platform/current/schema-registry/fundamentals/schema-evolution.html
Done when: tests register an incompatible change and a type change under NONE successfully.

### [To Do] X-SCHEMA-0083 `compatibilityGroup`: only versions with the same value of the named metadata property are checked
P2 · M10 · Task · deps: X-SCHEMA-0078 · ref: https://docs.confluent.io/platform/current/schema-registry/fundamentals/data-contracts.html#configuration-enhancements
Done when: tests show versions in another group are ignored; an absent property forms its own group.

### [To Do] X-SCHEMA-0084 Normalization in checks and registration: `normalize` query parameter and the config-level `normalize` default
P0 · M10 · Task · deps: X-SCHEMA-0078 · ref: https://docs.confluent.io/platform/current/schema-registry/fundamentals/serdes-develop/index.html#schema-normalization
Done when: tests show a whitespace-only change is a duplicate with normalization on and a new version with it off.

### [To Do] X-SCHEMA-0085 Cross-format property tests for level algebra: FULL equals BACKWARD and FORWARD; transitive implies non-transitive; BACKWARD(a, b) equals FORWARD(b, a); NONE always passes
P0 · M10 · Task · deps: X-SCHEMA-0081, X-SCHEMA-0082 · labels: test · ref: TEST-F16
Done when: properties hold for 10 000 generated cases per format.

## [To Do] X-SCHEMA-F08 Schema references
P1 · M10 · Feature · deps: X-SCHEMA-F03
Scope: Cross-subject references and resolution.

### [To Do] X-SCHEMA-0086 Reference model (name, subject, version) validated at registration: referenced versions must exist and not be hard-deleted (42214), depth limit `alluvium.schema.registry.reference.max.depth`, cycles rejected (42220)
P1 · M10 · Task · deps: none · labels: security · ref: https://docs.confluent.io/platform/current/schema-registry/fundamentals/serdes-develop/index.html#referenced-schemas
Done when: tests cover missing subject, missing version, `latest` resolution at registration time, depth overflow and cycles.

### [To Do] X-SCHEMA-0087 referencedBy index maintained on replay; deleting a referenced version or subject is rejected with 42206 REFERENCE_EXISTS
P1 · M10 · Task · deps: X-SCHEMA-0086 · ref: core/src/main/java/io/confluent/kafka/schemaregistry/rest/exceptions/Errors.java (confluentinc/schema-registry)
Done when: soft and hard deletes of referenced schemas fail; deleting the referrer first allows the delete (test).

### [To Do] X-SCHEMA-0088 Avro references: named types from referenced schemas resolved in dependency order
P1 · M10 · Task · deps: X-SCHEMA-0086, X-SCHEMA-F04 · ref: https://docs.confluent.io/platform/current/schema-registry/fundamentals/serdes-develop/serdes-avro.html#multiple-event-types-in-the-same-topic
Done when: a schema using a type defined in a referenced subject parses, normalizes and checks compatibility; an undefined name returns 42201.

### [To Do] X-SCHEMA-0089 Protobuf references: `import` paths resolved to referenced subjects by reference name
P1 · M10 · Task · deps: X-SCHEMA-0086, X-SCHEMA-F05 · ref: https://docs.confluent.io/platform/current/schema-registry/fundamentals/serdes-develop/serdes-protobuf.html
Done when: multi-file schemas with transitive imports resolve; a missing import returns 42201 naming it.

### [To Do] X-SCHEMA-0090 JSON Schema references: external `$ref` URLs resolved to referenced schemas by reference name
P1 · M10 · Task · deps: X-SCHEMA-0086, X-SCHEMA-F06 · ref: https://docs.confluent.io/platform/current/schema-registry/fundamentals/serdes-develop/serdes-json.html
Done when: external references validate payloads and participate in compatibility checks; unknown external URLs are never fetched over the network.

### [To Do] X-SCHEMA-0091 Compatibility with references: checks run on fully resolved schemas, including changes that only bump a reference version
P1 · M10 · Task · deps: X-SCHEMA-0088, X-SCHEMA-0089, X-SCHEMA-0090, X-SCHEMA-F07 · ref: client/src/main/java/io/confluent/kafka/schemaregistry/CompatibilityChecker.java (confluentinc/schema-registry)
Done when: tests show an incompatible change inside a referenced schema is detected when the referrer is re-registered.

### [To Do] X-SCHEMA-0092 Schema identity with references: dedup and id reuse consider the reference list
P1 · M10 · Task · deps: X-SCHEMA-0086 · ref: core/src/main/java/io/confluent/kafka/schemaregistry/storage/KafkaSchemaRegistry.java (confluentinc/schema-registry)
Done when: identical text with different references gets a different id; identical text and references reuse the id.

### [To Do] X-SCHEMA-0093 Reference property tests: random reference DAGs resolve, cycles are always rejected, deletes never orphan a referrer
P1 · M10 · Task · deps: X-SCHEMA-0087, X-SCHEMA-0091, X-SCHEMA-0092 · labels: test · ref: TEST-F16
Done when: 10 000 generated graphs pass.

## [To Do] X-SCHEMA-F09 Registry REST API
P0 · M10 · Feature · deps: X-SCHEMA-F03, X-NATIVE-F04
Scope: Confluent-compatible endpoints: subjects, schemas/ids, config, mode, compatibility.

### [To Do] X-SCHEMA-0094 Registry HTTP listener `alluvium.schema.registry.listeners` (default empty) on the X-NATIVE HTTP stack with Confluent root paths, started only when `alluvium.schema.registry.enable=true`
P0 · M10 · Task · deps: none · labels: security · ref: ADR Built-in schema registry (X-SCHEMA-0001)
Done when: both keys default to off; the listener applies X-NATIVE HTTP limits and hardening; plaintext non-loopback is refused unless explicitly allowed; startup fails with a clear error if listeners are set but the registry is disabled.

### [To Do] X-SCHEMA-0095 Content types: accept and produce `application/vnd.schemaregistry.v1+json`, `application/vnd.schemaregistry+json`, `application/json` and `application/octet-stream`, with weighted `Accept` negotiation
P0 · M10 · Task · deps: X-SCHEMA-0094 · ref: https://docs.confluent.io/platform/current/schema-registry/develop/api.html#content-types
Done when: tests cover each type and q-values; responses default to `application/vnd.schemaregistry.v1+json`.

### [To Do] X-SCHEMA-0096 Error body `{"error_code": …, "message": …}` with HTTP status per Confluent error code, from one exhaustive mapping of registry errors
P0 · M10 · Task · deps: X-SCHEMA-0094 · ref: core/src/main/java/io/confluent/kafka/schemaregistry/rest/exceptions/Errors.java (confluentinc/schema-registry)
Done when: every supported code (40401-40409, 40901, 42201-42211, 42214, 42220, 50001-50004) has a golden test; the mapping has no `_ =>` arm.

### [To Do] X-SCHEMA-0097 `Confluent-Accept-Unknown-Properties: true` gates the extended fields `guid`, `ts` and `deleted`
P0 · M10 · Task · deps: X-SCHEMA-0094 · ref: https://docs.confluent.io/platform/current/schema-registry/develop/api.html#extended-response-fields
Done when: tests on each endpoint listed in the Confluent reference show the fields only with the header.

### [To Do] X-SCHEMA-0098 Shared query parameter parsing: `offset`, `limit` (-1 means no limit, capped by `alluvium.schema.registry.max.page.size`), booleans, `format`, `deleted`, `deletedOnly`, `subjectPrefix`
P0 · M10 · Task · deps: X-SCHEMA-0094 · labels: security · ref: https://docs.confluent.io/platform/current/schema-registry/develop/api.html
Done when: invalid values return 422 or 400 as Confluent does (golden tests); the page cap is applied to every list endpoint.

### [To Do] X-SCHEMA-0099 Out-of-scope endpoints (exporters, importers, dek-registry, associations, tags, provenance, deployment metadata) return 404 with a Confluent-format body naming the unsupported feature
P0 · M10 · Task · deps: X-SCHEMA-0096 · ref: ADR Built-in schema registry (X-SCHEMA-0001)
Done when: a test calls each documented out-of-scope path and checks the body; the compatibility matrix links here.

### [To Do] X-SCHEMA-0100 `GET /` and `POST /`: no-op root resource returning `{}`
P1 · M10 · Task · deps: X-SCHEMA-0094 · ref: core/src/main/java/io/confluent/kafka/schemaregistry/rest/resources/RootResource.java (confluentinc/schema-registry)
Done when: both return 200 with `{}` for authenticated principals; used by client connectivity checks in tests.

### [To Do] X-SCHEMA-0101 `GET /v1/metadata/id`: Kafka cluster id and registry cluster id
P1 · M10 · Task · deps: X-SCHEMA-0094 · ref: core/src/main/java/io/confluent/kafka/schemaregistry/rest/resources/ServerMetadataResource.java getClusterId (confluentinc/schema-registry)
Done when: the Kafka cluster id equals Metadata's cluster id; the response shape matches Confluent's `ServerClusterId`.

### [To Do] X-SCHEMA-0102 `GET /v1/metadata/version`: server version and commit id
P2 · M10 · Task · deps: X-SCHEMA-0094 · ref: core/src/main/java/io/confluent/kafka/schemaregistry/rest/resources/ServerMetadataResource.java getSchemaRegistryVersion (confluentinc/schema-registry)
Done when: values come from the build metadata; shape matches Confluent's response.

### [To Do] X-SCHEMA-0103 `GET /schemas/types`: `["JSON","PROTOBUF","AVRO"]` for the enabled formats
P0 · M10 · Task · deps: X-SCHEMA-0094 · ref: core/src/main/java/io/confluent/kafka/schemaregistry/rest/resources/SchemasResource.java getSchemaTypes (confluentinc/schema-registry)
Done when: the list reflects enabled formats; golden test.

### [To Do] X-SCHEMA-0104 `GET /schemas` with `subjectPrefix`, `aliases`, `deleted`, `latestOnly`, `ruleType`, `offset` and `limit`
P0 · M10 · Task · deps: X-SCHEMA-0098 · ref: core/src/main/java/io/confluent/kafka/schemaregistry/rest/resources/SchemasResource.java getSchemas (confluentinc/schema-registry)
Done when: each filter has a test; output order matches Confluent (by subject then version).

### [To Do] X-SCHEMA-0105 `GET /schemas/ids/{id}` with `format` and `subject`: schema, schemaType, references and extended fields
P0 · M10 · Task · deps: X-SCHEMA-0098 · ref: core/src/main/java/io/confluent/kafka/schemaregistry/rest/resources/SchemasResource.java getSchemaById (confluentinc/schema-registry)
Done when: unknown ids return 40403; `subject` searches contexts starting with the default; `format=resolved` and `serialized` behave per format; golden tests.

### [To Do] X-SCHEMA-0106 `GET /schemas/ids/{id}/schema`: unescaped schema text only
P0 · M10 · Task · deps: X-SCHEMA-0105 · ref: core/src/main/java/io/confluent/kafka/schemaregistry/rest/resources/SchemasResource.java getOnlySchemaById (confluentinc/schema-registry)
Done when: the response body is the raw schema string; 40403 for unknown ids.

### [To Do] X-SCHEMA-0107 `GET /schemas/ids/{id}/subjects` with `subject`, `deleted`, `offset` and `limit`
P0 · M10 · Task · deps: X-SCHEMA-0105 · ref: core/src/main/java/io/confluent/kafka/schemaregistry/rest/resources/SchemasResource.java getAllSubjectsById (confluentinc/schema-registry)
Done when: subjects sharing an id are listed; soft-deleted only with `deleted=true`; 40403 for unknown ids.

### [To Do] X-SCHEMA-0108 `GET /schemas/ids/{id}/versions` with `subject` and `deleted`: subject-version pairs
P0 · M10 · Task · deps: X-SCHEMA-0105 · ref: core/src/main/java/io/confluent/kafka/schemaregistry/rest/resources/SchemasResource.java getAllVersionsById (confluentinc/schema-registry)
Done when: tests cover shared ids and soft-deleted versions.

### [To Do] X-SCHEMA-0109 `GET /schemas/guids/{guid}`: schema by GUID
P1 · M10 · Task · deps: X-SCHEMA-0105 · ref: core/src/main/java/io/confluent/kafka/schemaregistry/rest/resources/SchemasResource.java getSchemaByGuid (confluentinc/schema-registry)
Done when: malformed GUIDs return 422 or 404 as Confluent does; 40403 for unknown GUIDs.

### [To Do] X-SCHEMA-0110 `GET /schemas/guids/{guid}/ids`: ids (one per context) for a GUID
P1 · M10 · Task · deps: X-SCHEMA-0109 · ref: core/src/main/java/io/confluent/kafka/schemaregistry/rest/resources/SchemasResource.java getAllIdsByGuid (confluentinc/schema-registry)
Done when: a schema registered in two contexts returns both ids.

### [To Do] X-SCHEMA-0111 `GET /subjects` with `subjectPrefix`, `deleted`, `deletedOnly`, `offset` and `limit`
P0 · M10 · Task · deps: X-SCHEMA-0098 · ref: core/src/main/java/io/confluent/kafka/schemaregistry/rest/resources/SubjectsResource.java getAllSubjects (confluentinc/schema-registry)
Done when: without `subjectPrefix` subjects of all contexts are listed, with `subjectPrefix=` only the default context, as Confluent documents; tests cover each flag.

### [To Do] X-SCHEMA-0112 `GET /subjects/{subject}/versions` with `deleted`, `deletedOnly`, `offset` and `limit`
P0 · M10 · Task · deps: X-SCHEMA-0098 · ref: core/src/main/java/io/confluent/kafka/schemaregistry/rest/resources/SubjectVersionsResource.java getAllVersions (confluentinc/schema-registry)
Done when: 40401 for unknown subjects; ascending order; tests cover soft-deleted versions.

### [To Do] X-SCHEMA-0113 `GET /subjects/{subject}/versions/{version}` with `latest` or `-1`, `format` and `deleted`
P0 · M10 · Task · deps: X-SCHEMA-0098 · ref: core/src/main/java/io/confluent/kafka/schemaregistry/rest/resources/SubjectVersionsResource.java getSchemaByVersion (confluentinc/schema-registry)
Done when: 40401, 40402 and 42202 are returned for unknown subject, unknown version and invalid version; extended fields follow the header rule.

### [To Do] X-SCHEMA-0114 `GET /subjects/{subject}/versions/{version}/schema`: unescaped schema text only
P0 · M10 · Task · deps: X-SCHEMA-0113 · ref: core/src/main/java/io/confluent/kafka/schemaregistry/rest/resources/SubjectVersionsResource.java getVersionSchemaOnly (confluentinc/schema-registry)
Done when: golden tests per format; error codes as for get-version.

### [To Do] X-SCHEMA-0115 `GET /subjects/{subject}/versions/{version}/referencedby` with `offset` and `limit`: ids of schemas referencing this version
P1 · M10 · Task · deps: X-SCHEMA-0113, X-SCHEMA-F08 · ref: core/src/main/java/io/confluent/kafka/schemaregistry/rest/resources/SubjectVersionsResource.java getReferencedBy (confluentinc/schema-registry)
Done when: tests cover zero, one and many referrers and pagination.

### [To Do] X-SCHEMA-0116 `POST /subjects/{subject}/versions` request handling: body fields (schema, schemaType, references, metadata, ruleSet, version, id), `normalize`, size and parse limits, duplicate lookup
P0 · M10 · Task · deps: X-SCHEMA-0096, X-SCHEMA-0095 · labels: security · ref: core/src/main/java/io/confluent/kafka/schemaregistry/rest/resources/SubjectVersionsResource.java register (confluentinc/schema-registry)
Done when: an empty schema returns 422 "schema may not be empty"; unknown schemaType returns 42201; an identical schema returns the existing id without a write; limits are checked before parsing.

### [To Do] X-SCHEMA-0117 Registration write: compatibility gate (40901 INCOMPATIBLE_SCHEMA with verbose messages), reference validation, id and version allocation, response `{"id": …}` plus extended fields
P0 · M10 · Task · deps: X-SCHEMA-0116, X-SCHEMA-F07, X-SCHEMA-F08 · ref: core/src/main/java/io/confluent/kafka/schemaregistry/rest/resources/SubjectVersionsResource.java register (confluentinc/schema-registry)
Done when: the Java `CachedSchemaRegistryClient.register` call succeeds against Alluvium in a compat test; incompatible schemas return 40901 with messages; concurrent registrations of the same schema return one id.

### [To Do] X-SCHEMA-0118 `POST /subjects/{subject}`: check whether a schema is registered under a subject, with `normalize`, `format` and `deleted`
P0 · M10 · Task · deps: X-SCHEMA-0116 · ref: core/src/main/java/io/confluent/kafka/schemaregistry/rest/resources/SubjectsResource.java lookUpSchemaUnderSubject (confluentinc/schema-registry)
Done when: returns subject, id, version and schema when found; 40401 for unknown subject and 40403 when the schema is absent; tests cover normalized lookups.

### [To Do] X-SCHEMA-0119 `GET /subjects/{subject}/metadata` with repeated `key` and `value` pairs, `format` and `deleted`: latest version whose metadata properties match
P1 · M10 · Task · deps: X-SCHEMA-0113 · ref: core/src/main/java/io/confluent/kafka/schemaregistry/rest/resources/SubjectsResource.java getLatestWithMetadata (confluentinc/schema-registry)
Done when: tests cover multiple pairs, no match (40403) and unequal key and value counts (422).

### [To Do] X-SCHEMA-0120 `DELETE /subjects/{subject}` with `permanent`: soft or hard delete returning the deleted versions
P0 · M10 · Task · deps: X-SCHEMA-0096 · ref: core/src/main/java/io/confluent/kafka/schemaregistry/rest/resources/SubjectsResource.java deleteSubject (confluentinc/schema-registry)
Done when: semantics follow the X-SCHEMA-F03 delete rules; the response lists the deleted versions; golden tests cover 40401, 40404 and 40405.

### [To Do] X-SCHEMA-0121 `DELETE /subjects/{subject}/versions/{version}` with `permanent`: returns the deleted version number
P0 · M10 · Task · deps: X-SCHEMA-0096 · ref: core/src/main/java/io/confluent/kafka/schemaregistry/rest/resources/SubjectVersionsResource.java deleteSchemaVersion (confluentinc/schema-registry)
Done when: semantics follow X-SCHEMA-F03 version delete rules; golden tests for each error.

### [To Do] X-SCHEMA-0122 `POST /compatibility/subjects/{subject}/versions/{version}` with `normalize` and `verbose`: `{"is_compatible": …, "messages": […]}`
P0 · M10 · Task · deps: X-SCHEMA-0116, X-SCHEMA-F07 · ref: core/src/main/java/io/confluent/kafka/schemaregistry/rest/resources/CompatibilityResource.java testVersionCompatibility (confluentinc/schema-registry)
Done when: uses the subject's effective level against the given version; `messages` only with `verbose=true`; 40401, 40402, 42201 and 42202 covered.

### [To Do] X-SCHEMA-0123 `POST /compatibility/subjects/{subject}/versions` with `normalize` and `verbose`: the same check registration would perform
P0 · M10 · Task · deps: X-SCHEMA-0116, X-SCHEMA-F07 · ref: core/src/main/java/io/confluent/kafka/schemaregistry/rest/resources/CompatibilityResource.java testSubjectCompatibility (confluentinc/schema-registry)
Done when: results equal the registration gate for the same input (property test); a non-existent subject is compatible, as in Confluent.

### [To Do] X-SCHEMA-0124 `GET /config`: compatibilityLevel, normalize, compatibilityGroup, defaultMetadata, overrideMetadata, defaultRuleSet, overrideRuleSet
P0 · M10 · Task · deps: X-SCHEMA-0096 · ref: core/src/main/java/io/confluent/kafka/schemaregistry/rest/resources/ConfigResource.java getGlobalConfig (confluentinc/schema-registry)
Done when: defaults are returned when nothing is set; golden test.

### [To Do] X-SCHEMA-0125 `PUT /config`: update the global compatibility level and related settings, echoing the request
P0 · M10 · Task · deps: X-SCHEMA-0124, X-SCHEMA-F07 · ref: core/src/main/java/io/confluent/kafka/schemaregistry/rest/resources/ConfigResource.java updateGlobalConfig (confluentinc/schema-registry)
Done when: invalid levels return 42203; changes are persisted as CONFIG records and visible on all brokers (test through two brokers).

### [To Do] X-SCHEMA-0126 `DELETE /config`: revert global configuration to defaults, returning the deleted level
P1 · M10 · Task · deps: X-SCHEMA-0125 · ref: core/src/main/java/io/confluent/kafka/schemaregistry/rest/resources/ConfigResource.java deleteGlobalConfig (confluentinc/schema-registry)
Done when: subsequent `GET /config` returns the defaults; golden test.

### [To Do] X-SCHEMA-0127 `GET /config/{subject}` with `defaultToGlobal`, including `alias`
P0 · M10 · Task · deps: X-SCHEMA-0124 · ref: core/src/main/java/io/confluent/kafka/schemaregistry/rest/resources/ConfigResource.java getSubjectConfig (confluentinc/schema-registry)
Done when: without an override and `defaultToGlobal=false` it returns 40408; with `defaultToGlobal=true` the effective config; golden tests.

### [To Do] X-SCHEMA-0128 `PUT /config/{subject}`: set a subject override (compatibility, normalize, alias, group, metadata and ruleSet defaults)
P0 · M10 · Task · deps: X-SCHEMA-0127, X-SCHEMA-0125 · ref: core/src/main/java/io/confluent/kafka/schemaregistry/rest/resources/ConfigResource.java updateSubjectConfig (confluentinc/schema-registry)
Done when: the override takes effect for the next registration; an `alias` makes lookups on the alias subject resolve to the target (test).

### [To Do] X-SCHEMA-0129 `DELETE /config/{subject}`: remove a subject override
P1 · M10 · Task · deps: X-SCHEMA-0128 · ref: core/src/main/java/io/confluent/kafka/schemaregistry/rest/resources/ConfigResource.java deleteSubjectConfig (confluentinc/schema-registry)
Done when: returns the deleted level; 40401 for unknown subjects; subsequent checks use the global level.

### [To Do] X-SCHEMA-0130 `GET /mode`: global mode (default READWRITE)
P1 · M10 · Task · deps: X-SCHEMA-0096 · ref: core/src/main/java/io/confluent/kafka/schemaregistry/rest/resources/ModeResource.java getGlobalMode (confluentinc/schema-registry)
Done when: golden test for default and after change.

### [To Do] X-SCHEMA-0131 `PUT /mode` with `force`: set the global mode when `alluvium.schema.registry.mode.mutability=true`
P1 · M10 · Task · deps: X-SCHEMA-0130 · ref: core/src/main/java/io/confluent/kafka/schemaregistry/rest/resources/ModeResource.java updateGlobalMode (confluentinc/schema-registry)
Done when: invalid modes return 42204; with mutability off the call returns 42205; IMPORT on a non-empty registry requires `force=true`; persisted as MODE records.

### [To Do] X-SCHEMA-0132 `DELETE /mode` with `recursive`: remove the global override, optionally with subject overrides in the default context
P2 · M10 · Task · deps: X-SCHEMA-0131 · ref: core/src/main/java/io/confluent/kafka/schemaregistry/rest/resources/ModeResource.java deleteGlobalMode (confluentinc/schema-registry)
Done when: tests cover recursive and non-recursive deletes.

### [To Do] X-SCHEMA-0133 `GET /mode/{subject}` with `defaultToGlobal`
P1 · M10 · Task · deps: X-SCHEMA-0130 · ref: core/src/main/java/io/confluent/kafka/schemaregistry/rest/resources/ModeResource.java getSubjectMode (confluentinc/schema-registry)
Done when: without an override and `defaultToGlobal=false` it returns 40409; golden tests.

### [To Do] X-SCHEMA-0134 `PUT /mode/{subject}` with `force`, including qualified context names (`:.ctx:`) for context-level modes
P1 · M10 · Task · deps: X-SCHEMA-0133, X-SCHEMA-0131 · ref: core/src/main/java/io/confluent/kafka/schemaregistry/rest/resources/ModeResource.java updateSubjectMode (confluentinc/schema-registry)
Done when: tests cover subject and context targets and the IMPORT `force` rule.

### [To Do] X-SCHEMA-0135 `DELETE /mode/{subject}` with `recursive`
P2 · M10 · Task · deps: X-SCHEMA-0134 · ref: core/src/main/java/io/confluent/kafka/schemaregistry/rest/resources/ModeResource.java deleteSubjectMode (confluentinc/schema-registry)
Done when: recursive deletes clear subject overrides in the same context; 40401 for unknown subjects.

## [To Do] X-SCHEMA-F10 Modes and contexts
P1 · M10 · Feature · deps: X-SCHEMA-F09
Scope: READWRITE, READONLY, IMPORT; schema contexts.

### [To Do] X-SCHEMA-0136 Mode resolution: subject override, then context, then global, default READWRITE; mode checked once per write request
P1 · M10 · Task · deps: none · ref: core/src/main/java/io/confluent/kafka/schemaregistry/storage/Mode.java (confluentinc/schema-registry)
Done when: unit tests cover each layer; every write endpoint calls the same check (test enumerates write handlers).

### [To Do] X-SCHEMA-0137 READWRITE mode: reads and writes allowed; explicit ids and versions in registration rejected (42205)
P1 · M10 · Task · deps: X-SCHEMA-0136 · ref: core/src/main/java/io/confluent/kafka/schemaregistry/storage/Mode.java (confluentinc/schema-registry)
Done when: tests show an explicit id outside IMPORT is rejected.

### [To Do] X-SCHEMA-0138 READONLY mode: registrations, deletes and config writes rejected with 42205; READONLY_OVERRIDE accepted as the deprecated global form
P1 · M10 · Task · deps: X-SCHEMA-0136 · ref: core/src/main/java/io/confluent/kafka/schemaregistry/storage/Mode.java (confluentinc/schema-registry)
Done when: a test sweeps every write endpoint under READONLY; mode changes themselves stay allowed so operators can leave READONLY.

### [To Do] X-SCHEMA-0139 IMPORT mode: registration with explicit id and version, no compatibility checks, conflicting id content rejected (42207), allocator advanced past imported ids
P1 · M10 · Task · deps: X-SCHEMA-0136 · ref: https://docs.confluent.io/platform/current/schema-registry/installation/migrate.html
Done when: imported ids and versions are preserved exactly; after leaving IMPORT the next allocated id exceeds every imported id (test).

### [To Do] X-SCHEMA-0140 FORWARD mode rejected with 42204 and a message stating it is not supported
P2 · M10 · Task · deps: X-SCHEMA-0136 · ref: core/src/main/java/io/confluent/kafka/schemaregistry/storage/Mode.java (confluentinc/schema-registry)
Done when: PUT with FORWARD at every scope returns 42204.

### [To Do] X-SCHEMA-0141 Per-context schema id spaces: ids allocated per context; lookups by id default to the default context; `?subject=` searches contexts starting with the default
P1 · M10 · Task · deps: none · ref: https://docs.confluent.io/platform/current/schema-registry/schema-contexts-cp.html
Done when: the same id in two contexts refers to different schemas (test); X-SCHEMA-F03 allocator tests run per context.

### [To Do] X-SCHEMA-0142 `GET /contexts` with `contextPrefix`, `offset` and `limit`
P1 · M10 · Task · deps: X-SCHEMA-0141 · ref: core/src/main/java/io/confluent/kafka/schemaregistry/rest/resources/ContextsResource.java listContexts (confluentinc/schema-registry)
Done when: the default context `.` is always listed; contexts appear after their first registration; golden tests.

### [To Do] X-SCHEMA-0143 `DELETE /contexts/{context}`: delete an empty context (42211 CONTEXT_NOT_EMPTY otherwise)
P1 · M10 · Task · deps: X-SCHEMA-0142 · ref: core/src/main/java/io/confluent/kafka/schemaregistry/rest/resources/ContextsResource.java deleteContext (confluentinc/schema-registry)
Done when: tests cover empty, non-empty and default-context deletes (the default context cannot be deleted).

### [To Do] X-SCHEMA-0144 Context-level config and mode through `/config/:.ctx:` and `/mode/:.ctx:`, and `subjectPrefix=:.ctx:` listing
P1 · M10 · Task · deps: X-SCHEMA-0141 · ref: https://docs.confluent.io/platform/current/schema-registry/schema-contexts-cp.html
Done when: a context-level compatibility applies to subjects in it without overrides; listing returns only that context's subjects.

### [To Do] X-SCHEMA-0145 Mode and context property tests: no write succeeds in READONLY; IMPORT preserves ids exactly; contexts never share ids
P1 · M10 · Task · deps: X-SCHEMA-0137, X-SCHEMA-0138, X-SCHEMA-0139, X-SCHEMA-0144 · labels: test · ref: TEST-F16
Done when: proptest sequences across modes and contexts pass 10 000 cases.

## [To Do] X-SCHEMA-F11 Broker-side validation
P0 · M10 · Feature · deps: X-SCHEMA-F03, REC-F12
Scope: Topic configs requiring valid schema ids for key and value; optional full payload validation; clear errors.

### [To Do] X-SCHEMA-0146 Topic configs `alluvium.key.schema.validation` and `alluvium.value.schema.validation` (NONE by default, ID, SUBJECT, FULL) and `alluvium.key.subject.name.strategy` and `alluvium.value.subject.name.strategy`
P0 · M10 · Task · deps: CFG-F05 · ref: ADR Built-in schema registry (X-SCHEMA-0001)
Done when: keys are registered with docs; setting a mode other than NONE while the registry is disabled fails with INVALID_CONFIG; the keys are not reported to Kafka DescribeConfigs unless set (CFG-F11 rule).

### [To Do] X-SCHEMA-0147 Schema id extraction: prefix format (magic 0x0, big-endian int32 id, Protobuf message indexes) and header format (`__key_schema_id`, `__value_schema_id` with magic 0x1 and a 16-byte GUID)
P0 · M10 · Task · deps: TEST-F07 · labels: decoder, security · ref: schema-serializer/src/main/java/io/confluent/kafka/serializers/schema/id/DualSchemaIdDeserializer.java (confluentinc/schema-registry)
Done when: both formats decode with typed errors; when the id header is present its last occurrence takes precedence over the payload prefix, as Confluent's DualSchemaIdDeserializer does; the fuzz target runs per PR.

### [To Do] X-SCHEMA-0148 Append-path hook after LogValidator checks: per-record iteration of batches for validated topics only, bounded decompression, control batches, null keys and tombstone values skipped
P0 · M10 · Task · deps: X-SCHEMA-0146, REPL-F03 · labels: perf · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/LogValidator.java @4.4.0
Done when: unvalidated topics take the unchanged path (benchmark shows no regression); compressed batches are decompressed at most once; the hook runs before offsets are assigned.

### [To Do] X-SCHEMA-0149 Broker-side schema lookup: id and GUID to parsed schema from the local registry replica, bounded by `alluvium.schema.registry.cache.max.bytes`; a miss waits for replay up to `alluvium.schema.registry.read.wait.ms`
P0 · M10 · Task · deps: X-SCHEMA-F02 · labels: perf · ref: ADR Built-in schema registry (X-SCHEMA-0001)
Done when: a cold miss after a fresh registration succeeds once replay catches up; misses beyond the wait reject the batch with a retriable error; cache hit ratio is exported.

### [To Do] X-SCHEMA-0150 Validation mode ID: every validated key or value carries a schema id or GUID that exists in the registry
P0 · M10 · Task · deps: X-SCHEMA-0147, X-SCHEMA-0148, X-SCHEMA-0149 · ref: ADR Built-in schema registry (X-SCHEMA-0001)
Done when: missing magic byte, unknown id and truncated prefix each reject with a distinct reason; tests cover keys and values.

### [To Do] X-SCHEMA-0151 Validation mode SUBJECT: the id is registered under the subject given by the topic's naming strategy
P0 · M10 · Task · deps: X-SCHEMA-0150 · ref: schema-serializer/src/main/java/io/confluent/kafka/serializers/subject/strategy/SubjectNameStrategy.java (confluentinc/schema-registry)
Done when: tests cover TopicNameStrategy, RecordNameStrategy and TopicRecordNameStrategy for each format; an id registered only under another subject is rejected.

### [To Do] X-SCHEMA-0152 Validation mode FULL for Avro: the payload decodes completely with the registered schema
P0 · M10 · Task · deps: X-SCHEMA-0151, X-SCHEMA-F04 · ref: ADR Built-in schema registry (X-SCHEMA-0001)
Done when: truncated, trailing-byte and type-mismatched payloads are rejected; valid payloads from KafkaAvroSerializer pass.

### [To Do] X-SCHEMA-0153 Validation mode FULL for Protobuf: message indexes resolve and the payload decodes against the message descriptor
P0 · M10 · Task · deps: X-SCHEMA-0151, X-SCHEMA-F05 · ref: ADR Built-in schema registry (X-SCHEMA-0001)
Done when: wrong indexes, wire-type mismatches and truncated payloads are rejected; valid KafkaProtobufSerializer payloads pass.

### [To Do] X-SCHEMA-0154 Validation mode FULL for JSON Schema: the payload parses as JSON (bounded depth) and validates against the schema
P0 · M10 · Task · deps: X-SCHEMA-0151, X-SCHEMA-F06 · ref: ADR Built-in schema registry (X-SCHEMA-0001)
Done when: invalid JSON and schema violations are rejected with the failing JSON pointer; valid KafkaJsonSchemaSerializer payloads pass.

### [To Do] X-SCHEMA-0155 Rejection reporting: the batch fails with INVALID_RECORD (87) and per-record `record_errors` (batch index, message naming topic, side, mode and reason) in ProduceResponse v8 and later
P0 · M10 · Task · deps: X-SCHEMA-0150, API-F00 · ref: clients/src/main/java/org/apache/kafka/common/requests/ProduceResponse.java RecordError @4.4.0
Done when: older Produce versions get INVALID_RECORD without record errors per the ERR-F02 downgrade rules; native produce maps to INVALID_ARGUMENT with field violations; messages never include payload bytes.

### [To Do] X-SCHEMA-0156 Validation observability: `alluvium_schema_validation_failures_total{topic,side,reason}`, validated-records counter, debug-level rejection log without payload
P1 · M10 · Task · deps: X-SCHEMA-0150, OBS-F04 · ref: docs/standards/debuggability.md
Done when: counters are asserted in tests; labels are bounded (topic, two sides, reason enum).

### [To Do] X-SCHEMA-0157 Compatibility test: Confluent serializer producers pass validation; a raw-bytes producer receives InvalidRecordException with the message, for each mode
P0 · M10 · Task · deps: X-SCHEMA-0152, X-SCHEMA-0153, X-SCHEMA-0154, X-SCHEMA-0155, TEST-F08 · labels: test · ref: TEST-F08
Done when: Java and librdkafka producers are covered in `cargo xtask compat`.

### [To Do] X-SCHEMA-0158 Benchmark: append throughput per validation mode versus NONE with 1 KiB records
P1 · M10 · Task · deps: X-SCHEMA-0152, X-SCHEMA-0153, X-SCHEMA-0154, TEST-F11 · labels: perf · ref: TEST-F11
Done when: overhead per mode is recorded in docs/schema-registry/performance.md; CI tracks regressions.

## [To Do] X-SCHEMA-F12 Registry security
P0 · M10 · Feature · deps: X-SCHEMA-F09, SEC-F07
Scope: Subject-level ACLs and authentication.

### [To Do] X-SCHEMA-0159 Registry authentication: the X-NATIVE per-listener methods (mTLS, bearer, API key) plus HTTP Basic verified against SCRAM credentials for Confluent `basic.auth` clients
P0 · M10 · Task · deps: X-NATIVE-F09, SEC-F04 · labels: security · ref: https://docs.confluent.io/platform/current/schema-registry/security/index.html
Done when: Basic credentials are checked by deriving the SCRAM StoredKey with constant-time comparison; Basic is refused on plaintext non-loopback listeners; tests cover each method and failure.

### [To Do] X-SCHEMA-0160 ACL resource types SUBJECT and SCHEMA_REGISTRY: Alluvium resource codes in ACL records gated by a feature level, never returned to Kafka DescribeAcls, managed through native admin and `alluvium admin acls`
P0 · M10 · Task · deps: SEC-F09, META-F06, X-NATIVE-F08 · labels: security · ref: ADR Built-in schema registry (X-SCHEMA-0001)
Done when: Kafka `kafka-acls.sh --list` output is unchanged when such ACLs exist (test); native ListAcls shows them; prefixed subject patterns work.

### [To Do] X-SCHEMA-0161 Operation mapping table: each endpoint to (resource, operation), for example reads READ SUBJECT, register WRITE SUBJECT, deletes DELETE SUBJECT, `/config` ALTER_CONFIGS or DESCRIBE_CONFIGS, `/mode` ALTER on SCHEMA_REGISTRY
P0 · M10 · Task · deps: X-SCHEMA-0160 · labels: security · ref: ADR Built-in schema registry (X-SCHEMA-0001)
Done when: one exhaustive match covers every route (test fails on an unmapped route); the table is generated into docs/schema-registry/security.md; `authz_decision` events carry the subject.

### [To Do] X-SCHEMA-0162 Listing filters: `/subjects`, `/schemas` and `/contexts` return only entries the principal may DESCRIBE
P0 · M10 · Task · deps: X-SCHEMA-0161 · labels: security · ref: ADR Built-in schema registry (X-SCHEMA-0001)
Done when: tests with prefixed ACLs show partial listings; counts in pagination reflect filtered results.

### [To Do] X-SCHEMA-0163 Id and GUID lookups authorized when the principal can READ any subject the schema is registered under; otherwise 40403 without existence leaks
P0 · M10 · Task · deps: X-SCHEMA-0161 · labels: security · ref: ADR Built-in schema registry (X-SCHEMA-0001)
Done when: tests show identical responses for unauthorized and non-existent ids.

### [To Do] X-SCHEMA-0164 Schema-parsing resource limits enforced before and during parsing for every format: bytes, nesting depth, number of types and fields, references, evaluation steps
P0 · M10 · Task · deps: none · labels: security · ref: docs/standards/security.md#untrusted-input
Done when: bomb corpora per format (deep nesting, huge enums, exponential `$ref` fan-out) are rejected within a fixed CPU budget measured in tests.

### [To Do] X-SCHEMA-0165 Registry security tests: unauthenticated access on non-loopback refused, cross-subject reads denied, Basic brute-force delay, Authorization headers redacted
P0 · M10 · Task · deps: X-SCHEMA-0159, X-SCHEMA-0162, X-SCHEMA-0163, OBS-F11 · labels: security, test · ref: SEC-F13
Done when: each case has a named test; the OBS-F11 checker runs on registry test logs.

## [To Do] X-SCHEMA-F13 Registry performance
P1 · M10 · Feature · deps: X-SCHEMA-F02
Scope: Caching and lookup performance.

### [To Do] X-SCHEMA-0166 Parsed-schema cache: LRU bounded by bytes for parsed models and compatibility-ready forms
P1 · M10 · Task · deps: none · labels: perf · ref: ADR Built-in schema registry (X-SCHEMA-0001)
Done when: the cache never exceeds `alluvium.schema.registry.cache.max.bytes` (property test); hit and miss counters are exported; eviction is deterministic in simulation.

### [To Do] X-SCHEMA-0167 Compatibility result memoization keyed by (reader fingerprint, writer fingerprint, direction), bounded
P2 · M10 · Task · deps: X-SCHEMA-0166, X-SCHEMA-F07 · labels: perf · ref: ADR Built-in schema registry (X-SCHEMA-0001)
Done when: repeated registration checks against unchanged histories skip recomputation (counter test); results are identical with the memo disabled (property test).

### [To Do] X-SCHEMA-0168 Constant-time lookups for id, GUID, (subject, version) and canonical hash, with memory per schema measured
P1 · M10 · Task · deps: none · labels: perf · ref: ADR Built-in schema registry (X-SCHEMA-0001)
Done when: a benchmark shows lookup latency independent of registry size up to 100 000 schemas; memory per schema is documented.

### [To Do] X-SCHEMA-0169 Replay performance: 100 000 schemas replay within the budget recorded in docs/nfr.md
P1 · M10 · Task · deps: X-SCHEMA-0168 · labels: perf, nfr · ref: NFR-F06
Done when: a benchmark measures replay time and peak memory; the budget is enforced in the nightly job.

### [To Do] X-SCHEMA-0170 Registry benchmarks: registration throughput, lookup-by-id p99 and compatibility check latency for large schemas (1000 fields)
P1 · M10 · Task · deps: X-SCHEMA-0166, X-SCHEMA-0168, TEST-F11 · labels: perf · ref: TEST-F11
Done when: criterion results and HTTP macro results are in docs/schema-registry/performance.md with regression thresholds.

### [To Do] X-SCHEMA-0171 Memory budget test: 100 000 registered schemas stay within the configured cache budget plus documented index overhead
P1 · M10 · Task · deps: X-SCHEMA-0166, X-SCHEMA-0168 · labels: nfr · ref: NFR-F05
Done when: the nightly test asserts RSS against the documented formula.

## [To Do] X-SCHEMA-F14 Registry observability
P1 · M10 · Feature · deps: X-SCHEMA-F09, OBS-F04
Scope: Metrics and events.

### [To Do] X-SCHEMA-0172 Endpoint metrics `alluvium_schema_registry_requests_total{endpoint,status}` and `alluvium_schema_registry_request_duration_seconds{endpoint}`
P1 · M10 · Task · deps: none · ref: docs/standards/debuggability.md#metrics
Done when: endpoint labels come from the route table (bounded); tests assert values through the explicit registry.

### [To Do] X-SCHEMA-0173 State gauges and counters: subjects, schemas, soft-deleted versions, contexts, registrations, compatibility failures
P1 · M10 · Task · deps: none · ref: docs/standards/debuggability.md#metrics
Done when: gauges match state after replay (test); counters increment on the writer only.

### [To Do] X-SCHEMA-0174 Events with reasons: `schema_registered`, `subject_soft_deleted`, `subject_hard_deleted`, `compatibility_changed`, `mode_changed`, `context_deleted`, plus a registry-writer flight recorder
P1 · M10 · Task · deps: OBS-F06, OBS-F07 · ref: docs/standards/debuggability.md#events-for-state-transitions
Done when: events are in the crate's `events.rs` catalog with triggers; schema text never appears in events; the flight recorder dumps on writer errors.

### [To Do] X-SCHEMA-0175 Replay lag gauge per broker and writer transitions `schema_registry_writer_elected` and `schema_registry_writer_resigned` with reasons
P1 · M10 · Task · deps: X-SCHEMA-0174 · ref: docs/standards/debuggability.md
Done when: lag returns to zero after writes (test); transitions appear on leader change in simulation.

### [To Do] X-SCHEMA-0176 Diagnostics route `/debug/schema-registry`: writer identity, replay offsets, cache statistics, mode and global config
P2 · M10 · Task · deps: X-SCHEMA-0174, OBS-F08 · ref: OBS-F08
Done when: the route is authenticated per OBS-F08 and contains no schema text.

### [To Do] X-SCHEMA-0177 Registry metrics and events documented in docs/metrics.md and the events catalog, with alert suggestions (replay lag, writer flapping)
P1 · M10 · Task · deps: X-SCHEMA-0172, X-SCHEMA-0173, X-SCHEMA-0174, X-SCHEMA-0175 · labels: docs · ref: DOCS-F05
Done when: generated docs include every registry metric and event.

## [To Do] X-SCHEMA-F15 Registry testing
P0 · M10 · Feature · deps: X-SCHEMA-F09, TEST-F08
Scope: Confluent serializer compatibility tests and compatibility-rule property tests.

### [To Do] X-SCHEMA-0178 Java serializer compatibility: KafkaAvroSerializer and Deserializer, KafkaProtobufSerializer and Deserializer, KafkaJsonSchemaSerializer and Deserializer (pinned Confluent version) round-trip through the Alluvium registry and brokers
P0 · M10 · Task · deps: none · labels: test · ref: https://docs.confluent.io/platform/current/schema-registry/fundamentals/serdes-develop/index.html
Done when: each pair runs with auto-registration on and off and with header-based ids; tests run in `cargo xtask compat`.

### [To Do] X-SCHEMA-0179 Non-Java serializer compatibility: confluent-kafka-python and confluent-kafka-go Avro, Protobuf and JSON serializers round-trip
P1 · M10 · Task · deps: none · labels: test · ref: https://github.com/confluentinc/confluent-kafka-python
Done when: pinned client versions run in the compat harness.

### [To Do] X-SCHEMA-0180 Differential REST tests against a pinned Confluent Schema Registry container: scripted call sequences per endpoint, responses diffed with documented ignore rules
P0 · M10 · Task · deps: TEST-F09 · labels: test · ref: TEST-F09
Done when: every in-scope endpoint has at least one scenario; ignore rules (timestamps, server version) are listed in the test; all scenarios pass.

### [To Do] X-SCHEMA-0181 Schema Registry Maven plugin goals `register`, `test-compatibility` and `download` work against Alluvium
P2 · M10 · Task · deps: none · labels: test · ref: https://docs.confluent.io/platform/current/schema-registry/develop/maven-plugin.html
Done when: a compat scenario runs the plugin with a pinned version.

### [To Do] X-SCHEMA-0182 Simulation of registry writer failover under network and disk faults with invariant checks (unique ids, contiguous versions, no lost acknowledged registration, replicas converge)
P0 · M10 · Task · deps: TEST-F06 · labels: test · ref: TEST-F06
Done when: 1000 seeds pass nightly; failing seeds print the replay command.

### [To Do] X-SCHEMA-0183 Fuzz targets for REST request bodies (register, lookup, compatibility, config, mode) and path and query parsing
P0 · M10 · Task · deps: TEST-F07 · labels: test, security, decoder · ref: TEST-F07
Done when: targets run per PR with a corpus from the differential scenarios.

### [To Do] X-SCHEMA-0184 Off by default: a default node has no registry listener, no `__alluvium_schemas` topic, and topics default to validation NONE
P0 · M10 · Task · deps: X-SCHEMA-F10, X-SCHEMA-F11 · labels: test · ref: ADR-0010
Done when: a test inspects sockets, Metadata topic lists and topic configs on a default node.

### [To Do] X-SCHEMA-0185 Kafka compatibility and differential suites pass with the registry enabled but unused
P0 · M10 · Task · deps: X-SCHEMA-0184, TEST-F09 · labels: test · ref: ADR-0010
Done when: a CI job variant enables the registry and its listener without registering schemas; results equal the default job's.

## [To Do] X-SCHEMA-F16 Registry documentation and migration
P1 · M10 · Feature · deps: X-SCHEMA-F09
Scope: Docs and import from an existing Confluent registry.

### [To Do] X-SCHEMA-0186 User guide docs/schema-registry/: enabling the registry, client configuration (`schema.registry.url`, auth), formats, compatibility, references and contexts
P1 · M10 · Task · deps: none · labels: docs · ref: DOCS-F01
Done when: every example is executed by a docs test against a dev node.

### [To Do] X-SCHEMA-0187 Broker-side validation guide: modes, naming strategies, error handling in producers, performance cost
P1 · M10 · Task · deps: X-SCHEMA-F11 · labels: docs · ref: X-SCHEMA-F11
Done when: the guide shows the producer-side exception for each mode and links the benchmark results.

### [To Do] X-SCHEMA-0188 Migration tool `alluvium schema-registry import --from <url>`: copies subjects, versions (including soft-deleted), ids, configs and modes through IMPORT mode, with dry run and a verification report
P1 · M10 · Task · deps: X-SCHEMA-F10, CLI-F01 · ref: https://docs.confluent.io/platform/current/schema-registry/installation/migrate.html
Done when: an integration test migrates a populated Confluent registry container and every id resolves to identical schema text; credentials come from files; the tool is resumable after interruption.

### [To Do] X-SCHEMA-0189 Import from a Confluent `_schemas` topic (for example mirrored with MirrorMaker 2): decoder for Confluent's JSON keys and values with a fuzz target
P2 · M10 · Task · deps: X-SCHEMA-F10, TEST-F07 · labels: decoder · ref: core/src/main/java/io/confluent/kafka/schemaregistry/storage/SchemaKey.java (confluentinc/schema-registry)
Done when: SCHEMA, CONFIG, MODE, DELETE_SUBJECT and CLEAR_SUBJECT records are replayed through IMPORT mode; unknown key types are reported, not ignored silently.

### [To Do] X-SCHEMA-0190 Post-migration verification: for every source id, Alluvium returns identical schema text and subject-version pairs; serializers keep working with only the URL changed
P1 · M10 · Task · deps: X-SCHEMA-0188 · labels: test · ref: ADR Built-in schema registry (X-SCHEMA-0001)
Done when: the verification command exits non-zero on any mismatch and prints a diff summary.

### [To Do] X-SCHEMA-0191 Opt-in and rollback guide: enabling the registry on an existing cluster, running beside an external Confluent registry, cutover and rollback
P1 · M10 · Task · deps: X-SCHEMA-0188 · labels: docs · ref: ADR-0010
Done when: the guide covers READONLY on the source during cutover, IMPORT on Alluvium, and reverse migration through the REST API.
