# Alluvium

Alluvium is a streaming platform written in Rust. It is fully compatible with Apache Kafka: existing Kafka clients, tools and frameworks (Java client, librdkafka, Kafka Streams, Kafka Connect, `kafka-*.sh` tools) are meant to work against it unchanged. Beyond that, it is designed from the start without Kafka's limitations. It plans elastic partitions, queue semantics, large messages, diskless object-storage topics, multi-tenancy, built-in schemas, geo-replication, transforms, SQL, key lookups and native HTTP/gRPC APIs, all opt-in.

*Alluvium* is the sediment a river lays down in layers over time. Here, that's the append-only log.

## Status

**Design phase. There is no code yet.** The architecture, standards and decisions are written down first, so the code starts from a consistent foundation.

- Compatibility target: **Apache Kafka 4.4.0**
- Next milestone: **M0, workspace skeleton and tooling**
- Plan: [docs/backlog/](docs/backlog/README.md), with milestones M0–M18. Run `python3 docs/backlog/tools/backlog.py next` to see what to work on.

## Goals

1. **Kafka compatible.** Wire-protocol and semantic compatibility with Kafka 4.4.0, verified by running real clients and by differential tests against real Kafka.
2. **Better than Kafka.** Foundations are built without Kafka's limitations from the start. Enhancements are opt-in and never change what Kafka clients observe ([ADR-0010](docs/adr/0010-alluvium-is-its-own-platform.md)).
3. **Built to be debugged.** Deterministic state machines, replayable simulation, structured tracing, flight recorders and offline inspection tools.
4. **Secure and bounded by default.** No `unsafe`, rustls, loopback-only defaults, bounded resources everywhere.

## Where to start

| You want to… | Read |
|---|---|
| Understand the system | [docs/architecture.md](docs/architecture.md) |
| Get set up and contribute | [CONTRIBUTING.md](CONTRIBUTING.md) |
| Know the rules | [.agents/skills/alluvium-engineering/SKILL.md](.agents/skills/alluvium-engineering/SKILL.md) |
| See why something was decided | [docs/adr/](docs/adr/README.md) |
| See what's planned, built and next | [docs/backlog/](docs/backlog/README.md) |

## License

No license has been chosen yet. All rights reserved.

Alluvium uses Apache Kafka's protocol schemas and studies its source, both of which are Apache-2.0 licensed. See [ADR-0008](docs/adr/0008-protocol-code-generated-from-kafka-schemas.md) for the attribution rules.
