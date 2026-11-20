# ADR-0001: Kafka 4.4 compatibility is the parity contract

- **Status:** Accepted
- **Date:** 2026-11-20

## Context

Alluvium's value depends on working with the existing Kafka ecosystem: client libraries in every language, Kafka Streams, Kafka Connect, admin tools, monitoring and operational knowledge. "Feature parity" needs a precise, testable definition, or it turns into an endless list of guesses.

Apache Kafka 4.4.0 is the latest release. Kafka 4.0 removed ZooKeeper mode and its APIs (keys 4–7), and message formats v0/v1 (KIP-724), which considerably shrinks the surface.

## Decision

1. **Target: Apache Kafka 4.4.0.** A move to a newer Kafka version is made through a new ADR.
2. **Client-facing compatibility is exact.** For every API in [parity.md](../parity.md) with a broker listener, Alluvium supports the same version range as Kafka 4.4.0 with the same:
   - wire encoding;
   - error codes for each version;
   - ordering, durability and delivery semantics;
   - defaults for Kafka config keys.
3. **Inter-node protocols reuse Kafka's messages** (KRaft `Vote`/`BeginQuorumEpoch`/`Fetch`, `BrokerRegistration`, `AlterPartition`, replication `Fetch`) where the semantics match. This keeps admin tools such as `kafka-metadata-quorum.sh` working and avoids designing new protocols. **Mixed Kafka/Alluvium clusters are not a goal.**
4. **Parity is proven, not claimed.** An API is "Done" when compatibility tests with real clients and differential tests against Kafka 4.4.0 pass ([testing](../standards/testing.md)).
5. **Out of scope:** ZooKeeper mode, removed APIs, message formats v0/v1, JMX (replaced by Prometheus with a name mapping), Streams and Connect runtimes (these are clients).
6. **Improvements come after parity**, per area. They are opt-in, need an ADR, and don't change default client-observable behavior.

## Consequences

- Every behavior question has an answer: read Kafka 4.4.0. Code cites it with `// Parity:`.
- Intentional deviations (for example the secure defaults in ADR-0006) must be ADRs, so they are visible and countable.
- A Docker-based Kafka 4.4.0 test environment is required from M2 onward.

## Alternatives considered

- **Track Kafka trunk.** Rejected: a moving target that can't be verified.
- **A new protocol with a Kafka compatibility layer.** Rejected: two protocols to maintain and debug, and the compatibility layer would always lag.
- **A protocol-compatible subset only.** Rejected: "parity" would become a matter of opinion, and gaps would show up in production clients.
