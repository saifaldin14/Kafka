# Alluvium

Alluvium is a Kafka-compatible streaming platform written in Rust. Existing Kafka clients, tools and frameworks (Java client, librdkafka, Kafka Streams, Kafka Connect, `kafka-*.sh` tools) are meant to work against it unchanged.

*Alluvium* is the sediment a river lays down in layers over time. Here, that's the append-only log.

## Status

**Design phase. There is no code yet.** The architecture, standards and decisions are written down first, so the code starts from a consistent foundation.

- Parity target: **Apache Kafka 4.4.0**
- Next milestone: **M0, workspace skeleton and tooling** ([docs/parity.md](docs/parity.md#milestones))

## Goals

1. **Parity first.** Wire-protocol and semantic compatibility with Kafka 4.4.0, verified by running real clients and by differential tests against real Kafka.
2. **Then better.** Improvements come only after parity in an area. They are opt-in, and they never change what existing clients observe.
3. **Built to be debugged.** Deterministic state machines, replayable simulation, structured tracing, flight recorders and offline inspection tools.
4. **Secure and bounded by default.** No `unsafe`, rustls, loopback-only defaults, bounded resources everywhere.

## Where to start

| You want to… | Read |
|---|---|
| Understand the system | [docs/architecture.md](docs/architecture.md) |
| Get set up and contribute | [CONTRIBUTING.md](CONTRIBUTING.md) |
| Know the rules | [.agents/skills/alluvium-engineering/SKILL.md](.agents/skills/alluvium-engineering/SKILL.md) |
| See why something was decided | [docs/adr/](docs/adr/README.md) |
| See what's built and what's next | [docs/parity.md](docs/parity.md) |

## License

No license has been chosen yet. All rights reserved.

Alluvium uses Apache Kafka's protocol schemas and studies its source, both of which are Apache-2.0 licensed. See [ADR-0008](docs/adr/0008-protocol-code-generated-from-kafka-schemas.md) for the attribution rules.
