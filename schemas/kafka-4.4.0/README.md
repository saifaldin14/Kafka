# Apache Kafka 4.4.0 schemas, vendored

The files under this directory are copied unmodified from Apache Kafka at tag `4.4.0`, commit `1156b2752aaa28eafc0b4c4d387e63356793268d`. `cargo xtask codegen` generates Alluvium's protocol code from them ([ADR-0008](../../docs/adr/0008-protocol-code-generated-from-kafka-schemas.md)).

**Never edit a file here.** A change to a schema means a new Kafka version: vendor the new tag into its own directory instead. Behavior that differs from Kafka belongs in Alluvium's code, with an ADR, never in these files.

They are Apache Kafka material under the Apache License 2.0. [`LICENSE`](LICENSE) and [`NOTICE`](NOTICE) are Kafka's own, from the same tag, and Alluvium's [`NOTICE`](../../NOTICE) lists this directory.

| Directory | Kafka source at the tag |
|---|---|
| `clients/` | `clients/src/main/resources/common/message/`: request, response, header and data schemas, and Kafka's README on the schema format |
