# ADR-0005: Observability model

- **Status:** Accepted
- **Date:** 2026-11-20

## Context

Kafka operators rely on JMX metrics, `log4j` logs (including the request and authorizer loggers), and tools like `kafka-dump-log.sh`. Alluvium must be at least as diagnosable, in a way that fits Rust and modern infrastructure, and it must make "why did this happen?" answerable without code changes.

## Decision

1. **`tracing`** for logs and spans, with a fixed span hierarchy and a standard field vocabulary ([debuggability](../standards/debuggability.md)). The output is JSON in production. The filter can be changed at runtime, including through Kafka's `BROKER_LOGGER` config resource.
2. **Metrics with `prometheus-client`**, using an explicitly passed registry with no global state. Metrics that have Kafka equivalents are documented with their Kafka JMX names. Label cardinality is bounded.
3. **State transitions are first-class.** Layer 1–2 state machines return transitions with `reason` enums. Drivers log them as named events and record them in per-component **flight recorders**, which are bounded ring buffers exposed via the diagnostics endpoint and dumped on errors and panics.
4. **A diagnostics endpoint**, on a separate listener that is loopback-only by default, serves metrics, health, effective config with sources, task lists, component state and flight recorders, and log-level control.
5. **Offline tools** in the main binary dump and verify every on-disk format.
6. **Optional input journaling** for state machines, for exact replay of production behavior (metadata only, off by default).

## Consequences

- Every feature PR includes its events, metrics and flight-recorder entries. This is part of the definition of done.
- Operators coming from Kafka can translate their dashboards using the documented mapping.
- Flight recorders cost a small fixed amount of memory per component. Their size is configurable.

## Alternatives considered

- **The `metrics` facade crate.** Rejected: it relies on a global recorder, which makes tests that assert on metrics harder and hides dependencies.
- **OpenTelemetry SDK as the primary API.** Deferred: `tracing` can export to OTLP through a layer when needed, without making the OpenTelemetry SDK a core dependency.
- **JMX compatibility.** Rejected: it requires a JVM bridge and isn't the modern standard.
