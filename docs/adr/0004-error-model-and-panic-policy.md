# ADR-0004: Error model and panic policy

- **Status:** Accepted
- **Date:** 2026-11-20

## Context

Errors in Alluvium must (a) map to the right Kafka error code per API version, so clients retry or fail correctly, and (b) contain enough context to debug from one log line. Stringly-typed errors or a catch-all `anyhow` lose both.

Panics are the other half. Continuing after an invariant violation in a durable log risks silent data corruption, which is worse than a crash followed by recovery. Kafka itself halts the broker on some storage failures for the same reason.

## Decision

1. **Library crates use typed `thiserror` enums** whose variants carry identifying fields. `anyhow` is allowed only in the binary, `xtask` and tests.
2. **One exhaustive mapping to `ErrorCode` per crate**, with no wildcard arm. Version-dependent codes are chosen in the handler.
3. **Errors are logged once**, by the code that handles them.
4. **Input never causes a panic.** Network and disk input produce errors.
5. **A panic is a bug, and it is fatal.** The release profile uses `panic = "abort"` and `overflow-checks = true`. The supervisor treats a task panic as fatal in all builds. The panic hook logs the backtrace, the active span stack, and the relevant flight recorder before aborting.
6. **Invariants** are asserted (`assert!` where violation would corrupt data, `debug_assert!` on hot paths). `expect` is used only with an `"invariant: …"` message.
7. **Storage I/O failures** take the affected log directory offline and move its partitions elsewhere, following Kafka's `LogDirFailureChannel` behavior. They do not panic.

## Consequences

- One log line identifies what failed and where.
- Adding an error variant forces a decision about the client-visible error code.
- A bug can crash a node. That is deliberate: replication and fast recovery handle node loss, while corrupted state is not recoverable. Panic-freedom on input is enforced by fuzzing, so a crash cannot be triggered by a client.

## Alternatives considered

- **`panic = "unwind"` and catching panics per task.** Rejected: the panicking task may have left shared state half-updated.
- **`anyhow` everywhere.** Rejected: loses the structure needed for error-code mapping and exhaustive handling.
