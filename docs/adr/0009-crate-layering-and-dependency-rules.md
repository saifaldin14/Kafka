# ADR-0009: Crate layering and dependency rules

- **Status:** Accepted
- **Date:** 2026-11-20

## Context

Large codebases lose their structure one convenient import at a time. Once storage knows about networking, or a coordinator reaches into the server, every change has a wide impact and every bug has many suspects. In Kafka, the `core` module grew into this shape over the years and is still being split into separate modules.

## Decision

1. The workspace is organized into the layers in [architecture §4–5](../architecture.md#4-code-map). A crate may depend only on lower layers.
2. **Layers 0–2 are runtime-free and I/O-free** (except storage through its `fs` trait). They don't depend on Tokio or any networking crate.
3. **`cargo xtask check-layers`** reads `cargo metadata` and fails if:
   - any dependency between workspace crates violates the layers;
   - any layer 0–2 crate depends, directly or transitively, on a crate in a deny-list (`tokio`, `mio`, `hyper`, and so on).

   Each crate's layer is declared in one table in `xtask`.
4. **Same-layer dependencies** are forbidden unless listed in `xtask`'s allow-list with an ADR reference.
5. **Crates are created when their first code lands**, not ahead of time. Splitting a crate later is preferred over creating empty structure early.
6. **No catch-all crates** (`common`, `utils`). Shared code moves to the lowest crate that owns the concept.

## Consequences

- Each crate can be understood, tested and reviewed on its own terms. A bug in the storage layer can be investigated without knowing the network code.
- Some code that would be convenient to share must instead be placed carefully, or passed in as data. This is intended.
- The layer table in `xtask` and the table in `architecture.md` must agree. `check-layers` verifies that both list the same crates.

## Alternatives considered

- **A single crate with modules.** Rejected: Rust enforces no boundaries between modules of one crate, and compile times grow.
- **Rules enforced only by review.** Rejected: review misses gradual erosion. A machine check doesn't.
