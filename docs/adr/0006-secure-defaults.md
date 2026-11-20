# ADR-0006: Secure defaults

- **Status:** Accepted
- **Date:** 2026-11-20

## Context

Kafka's defaults favor ease of first use: plaintext listeners, unlimited connections, no authentication. Many real-world exposed-Kafka incidents trace back to deployments that kept those defaults. Alluvium aims to be secure by default without breaking clients: protocol behavior stays identical, and only the operator's starting configuration differs.

## Decision

1. **Memory safety:** `unsafe_code = "forbid"` workspace-wide. Exceptions need their own ADR ([security](../standards/security.md#unsafe-code)).
2. **TLS via rustls only.** No OpenSSL (it is banned in `deny.toml`).
3. **Loopback by default:** listeners and the diagnostics endpoint bind `127.0.0.1` unless configured otherwise.
4. **No accidental plaintext exposure:** a plaintext listener on a non-loopback address is refused at startup unless `alluvium.allow.plaintext.external=true`. **This deviates from Kafka.** Kafka accepts such listeners silently. The opt-in flag restores Kafka's behavior.
5. **Finite connection limits by default** (`max.connections`, `max.connections.per.ip`). **This deviates from Kafka**, whose defaults are unlimited. Operators can set Kafka's values explicitly.
6. **Authorization follows Kafka:** with an authorizer enabled, `allow.everyone.if.no.acl.found=false`, and denials are logged.
7. **Supply chain:** `cargo deny` in CI, `Cargo.lock` committed, a review checklist for new dependencies, GitHub Actions pinned by SHA.

## Consequences

- Deployments copied from Kafka that use external plaintext listeners fail fast with a clear error naming the flag, rather than running insecurely.
- Clients are unaffected: all deviations are in startup validation and resource limits, not in protocol behavior.
- Each deviation is listed in [parity.md](../parity.md) under "Intentional deviations".

## Alternatives considered

- **Copy Kafka's defaults exactly.** Rejected: it reproduces a known class of incidents.
- **Require TLS and SASL everywhere, including loopback.** Rejected: too much friction for local development, for little security gain on loopback.
