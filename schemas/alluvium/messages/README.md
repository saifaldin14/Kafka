# Alluvium internal API schemas

The schemas of Alluvium's internal APIs: requests between Alluvium nodes that Kafka has no API for. They are never advertised to Kafka clients and never accepted from them ([ADR-0021](../../../docs/adr/0021-alluvium-internal-api-keys.md)). There are none yet.

A schema here uses Kafka's JSON schema format, and `cargo xtask codegen` generates it by the same rules as Kafka's schemas, into `crates/alluvium-protocol/src/internal/`. Codegen also enforces these rules, and fails naming the schema that breaks one:

- A request or response has an API key from 32000 to 32767. 32000 is reserved for `AlluviumApiVersions`, which the first internal API adds.
- No key is one Kafka's schemas use, removed APIs included, and no two internal requests or responses share one.
- Each request has a response with its key, and each response a request.
- Every schema, data schemas included, is flexible in every version: `"flexibleVersions": "0+"`.
- A retired schema, one without valid versions, stays here and keeps its key.

When the first schema lands, declare the module in the protocol crate's `lib.rs` as the other generated modules are, with no other attribute:

```rust
#[rustfmt::skip]
pub mod internal;
```

Codegen fails until it is declared, so that no generated code goes uncompiled.

Versions only grow. A release supports every version the previous release sends, and a version is removed only in a release after one in which no node sends it.

[`../test/`](../test/) holds a sample internal request and response, generated into the test-only `test_messages` module so that how internal APIs are generated is reviewed as code.
