# ADR-0016: Layer 0 crates may depend on alluvium-types

- **Status:** Accepted
- **Date:** 2026-11-22

## Context

[ADR-0009](0009-crate-layering-and-dependency-rules.md) lets a crate depend only on lower layers, and forbids same-layer dependencies unless an allow-list entry backed by an ADR permits one. The [code map](../architecture.md#4-code-map) puts three crates in layer 0: `alluvium-types` (the domain types), `alluvium-protocol` and `alluvium-records` (the codecs).

The codecs decode into the domain types. A Kafka message carries topic ids as `Uuid`, and its fields are node ids, offsets, epochs and topic names. `alluvium-types` was written for exactly this: its crate doc says every crate may depend on it. The layer rules said otherwise, and `check-layers` rejected the first edge, `alluvium-protocol` → `alluvium-types`, when the protocol crate was created (PROTO-0025).

## Decision

1. A layer 0 crate may depend on `alluvium-types`. That is the only same-layer dependency layer 0 allows: the codecs do not depend on each other.
2. `alluvium-types` depends on no other workspace crate, so the edge can never form a cycle.
3. Each edge is an entry in `xtask`'s same-layer allow-list citing this ADR, added in the change that first uses it. `check-layers` already rejects an entry that is unused, so the list stays exactly the edges that exist.

## Consequences

- Codecs decode straight into validated domain types, with one definition of each type.
- The allow-list gains one entry per codec crate: `alluvium-protocol` now, and `alluvium-records` when its first code lands.
- Architecture §5 rule 1 names this exception.

## Alternatives considered

- **A layer below 0 for `alluvium-types`.** This would renumber every layer in the architecture, the layer table and the ADRs, to express one fact this ADR states directly.
- **Codecs with their own copies of the types**, such as a raw 16-byte uuid. That duplicates knowledge the [engineering skill](../../.agents/skills/alluvium-engineering/SKILL.md) says lives in one place, and every caller would convert between the copies.
- **Moving the codecs to layer 1.** Layer 1 is for primitives built on decoded data (storage, Raft, metadata). Storage and metadata depend on the codecs, so they would become same-layer edges instead.
