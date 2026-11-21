# ADR-0010: Alluvium is its own platform; Kafka 4.4 is a compatibility surface

- **Status:** Accepted
- **Date:** 2026-11-20
- **Supersedes:** [ADR-0001](0001-kafka-4-4-compatibility-is-the-parity-contract.md)

## Context

ADR-0001 made Kafka 4.4.0 parity the definition of the project, with improvements allowed only after each area reached parity. The project's direction has since changed. Alluvium is a new streaming platform, not a Kafka clone. Its own model goes beyond Kafka's well-known limitations:
- partitions that can't shrink, and adding partitions breaks key ordering;
- no per-message queue semantics;
- small message limits;
- storage tied to brokers;
- no built-in schemas, tenancy, lookups or processing;
- heavy operations.

Retrofitting a design built only for parity would be expensive. Kafka's ecosystem still matters, though: existing clients, tools and frameworks must keep working unchanged.

## Decision

1. **Alluvium is its own platform.** It has its own data model, native APIs and features. It is not defined by Kafka.
2. **Kafka 4.4.0 compatibility is a first-class, permanent protocol surface**, and the correctness benchmark for the semantics Alluvium shares with Kafka. On that surface, ADR-0001's rules still apply in full:
   - same API version ranges, wire encoding, error codes per version, ordering, durability and delivery semantics, and config key defaults;
   - proven by real-client compatibility tests and differential tests against Kafka 4.4.0;
   - every deviation needs an ADR.
3. **Foundations without Kafka's limitations are built from the start.** During the parity milestones (M0–M9), core types and interfaces are designed so enhancements can be added without rework. These tasks are labelled `foundation-for:<epic>` in the [backlog](../backlog/README.md). They include:
   - namespaces in the identity and metadata model;
   - key ranges over a hash space;
   - a segment-store abstraction;
   - a protocol-agnostic service layer;
   - a per-record delivery state store;
   - an append observer hook;
   - reserved record-header conventions.
4. **Enhancements are opt-in.** Every feature beyond Kafka:
   - is off by default;
   - doesn't change anything a Kafka client can observe under default settings;
   - has its own ADR;
   - keeps the Kafka compatibility and differential suites passing when it is enabled but unused.
5. **Order of work:** the parity track (M0–M9) comes first, because it proves correctness and builds the foundations. The enhancement track (M10–M18) follows. An enhancement may start earlier once its foundations and the parity work it depends on are done.
6. **Still out of scope:**
   - ZooKeeper mode and the APIs removed in Kafka 4.0;
   - message formats v0/v1;
   - JMX (replaced by Prometheus, with a name mapping);
   - mixed Kafka/Alluvium clusters;
   - reimplementing the Kafka Streams and Kafka Connect runtimes. They remain clients; built-in processing is provided separately by X-SQL and X-WASM.

## Consequences

- The backlog has two tracks, with foundation work inside the parity milestones.
- Core interfaces carry a little more generality than parity alone would need: for example, a segment store trait with one implementation at first. This is a deliberate exception to "no speculative abstraction". Each such interface is justified by a named enhancement epic and labelled `foundation-for`.
- Kafka users can adopt Alluvium without changes, then opt into native features one at a time.
- Two API surfaces (Kafka and native) share one service layer, so behavior and authorization cannot drift between them.

## Alternatives considered

- **Keep ADR-0001 (parity first, improvements later per area).** Rejected: the foundations for elastic partitions, queues, diskless storage and tenancy touch core types and metadata. Adding them after parity would mean rewriting finished work.
- **Let enhancements change default behavior.** Rejected: it would break drop-in compatibility, which is the main adoption path.
- **Native APIs only, no Kafka protocol.** Rejected: it gives up the Kafka ecosystem.
