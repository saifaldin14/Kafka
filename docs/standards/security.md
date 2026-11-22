# Security standard

Alluvium stores other people's data and listens on the network. Security is a default, not an add-on. Decisions are in [ADR-0006](../adr/0006-secure-defaults.md).

## Threat model (summary)

| Threat | Main defenses |
|---|---|
| Malformed or malicious requests (crash, memory exhaustion, CPU exhaustion) | Panic-free decoders, size limits checked before allocating, fuzzing, per-connection and per-IP limits, quotas |
| Unauthenticated or unauthorized access | Authentication required before any API except `ApiVersions`/`SaslHandshake`/`SaslAuthenticate`; ACLs that deny by default |
| Eavesdropping or tampering on the network | TLS via rustls; plaintext only on loopback unless explicitly allowed |
| Credential theft | SCRAM stored as salted hashes; `Secret<T>`; secrets never in logs, metrics, debug output or command-line arguments |
| Corrupted or tampered data on disk | CRC32C validation on recovery and replication paths; `verify log`; disk input treated as untrusted |
| Hash-flooding denial of service | SipHash-1-3 hash maps keyed per process from OS entropy; the keys are never logged ([ADR-0012](../adr/0012-seeded-randomness-and-hashing.md)) |
| Malicious or compromised dependency | Minimal dependencies, `cargo deny`, lockfile, review of new crates |
| Exposed diagnostics | Diagnostics endpoint loopback-only by default; authentication required otherwise |

## Untrusted input

**All bytes from the network and from disk are untrusted.** Decoders follow these rules:

1. **Never panic.** Return a `DecodeError` that identifies what failed and at which offset in the input.
2. **Bound before allocating.** A length read from input is checked against both the remaining bytes and a configured limit *before* any `Vec::with_capacity` or `reserve`. A 4-byte array length of 2³¹ must not allocate anything.
3. **Checked arithmetic** on every value derived from input (offsets, lengths, deltas). The decoder crates deny `clippy::arithmetic_side_effects` and `clippy::indexing_slicing`.
4. **No unbounded recursion.** Nested structures have a depth limit.
5. **Decompression is bounded.** The decompressed size is capped by `message.max.bytes` and by the batch's declared size. A decompression bomb returns an error, not an out-of-memory kill.
6. **A fuzz target for every decoder** (request header, every request body via the generated codec, record batch, each compression codec, segment and index recovery, metadata records). Fuzzing runs in CI on every PR for a short time and nightly for longer.

## Network defaults

| Setting | Default | Note |
|---|---|---|
| Listener bind address | `127.0.0.1` | Binding elsewhere is an explicit operator choice |
| Plaintext on a non-loopback address | **Refused at startup** unless `alluvium.allow.plaintext.external=true` | Deviation from Kafka, see ADR-0006 |
| `socket.request.max.bytes` | 104857600 (Kafka default) | Checked against the frame length before reading the body |
| `max.connections`, `max.connections.per.ip` | Finite defaults (set in M2) | Kafka's default is unlimited; ours are finite. Recorded as an ADR-0006 deviation |
| Max in-flight requests per connection | Bounded; the connection is muted when reached | |
| Authentication timeout | Bounded | Unauthenticated connections are closed |
| `connections.max.reauth.ms` | Supported (KIP-368) | |

## TLS

- **rustls only.** No OpenSSL. TLS 1.3 is preferred; TLS 1.2 is allowed with rustls's default cipher suites. Older versions are not supported.
- Certificates and keys reload without restart (as with Kafka's dynamic SSL config).
- mTLS client authentication maps certificate subjects to principals with Kafka-compatible `ssl.principal.mapping.rules`.

## Authentication (SASL)

| Mechanism | Plan | Notes |
|---|---|---|
| `PLAIN` | M4 | Refused on plaintext non-loopback listeners unless explicitly allowed |
| `SCRAM-SHA-256`, `SCRAM-SHA-512` | M4 | Credentials stored as salted, iterated hashes in metadata, like Kafka |
| `OAUTHBEARER` | M4 | JWT validation with JWKS. No network calls on the request path; keys are cached and refreshed in the background |
| `GSSAPI` (Kerberos) | Later | Needs a decision on FFI (backlog spike SEC-F06) |

- SASL mechanisms are sans-I/O state machines in `alluvium-security`, fed bytes by `alluvium-network`.
- Credential comparisons use constant-time equality (`subtle`).
- Delegation tokens (KIP-48) follow Kafka semantics.

## Authorization

- A Kafka-compatible ACL authorizer: resource types, pattern types (literal, prefixed), operations, `ALLOW`/`DENY` with deny precedence, `super.users`, and `allow.everyone.if.no.acl.found=false` by default, as in Kafka.
- **Every handler authorizes before it acts**, using the same operation and resource mapping as Kafka's `KafkaApis` for that API (cite it with `// Parity:`). A handler that skips authorization fails review.
- The authorizer is a pure function of (ACL image, principal, host, operation, resource). It is unit- and property-tested.
- Decisions are logged as `authz_decision` events: denials at `info`, allows at `debug`. This matches Kafka's `kafka.authorizer.logger`.

## Secrets

- Secrets are held in `Secret<T>`. It redacts `Debug` and `Display`, zeroizes on drop, and exposes the value only through an explicit `expose_secret()`.
- Secrets come from files or environment variables, never command-line arguments, which other users can see in `ps`.
- Config dumps (`check-config`, `/debug/config`, `DescribeConfigs`) redact sensitive keys, as Kafka marks them `PASSWORD`.

## `unsafe` code

- `unsafe_code = "forbid"` workspace-wide.
- An exception requires an ADR naming the crate, the reason, and why safe alternatives fail. The crate then overrides the lint locally. Every `unsafe` block has a `// SAFETY:` comment, is covered by tests run under Miri where possible, and is fuzzed if reachable from input.
- Dependencies that use `unsafe` are acceptable when they are widely used and maintained (`bytes`, `tokio`, `rustls`, compression codecs). That is part of the dependency review below.

## Dependencies

Before adding a dependency, the PR answers these questions. `cargo xtask deps-diff <base-ref>` lists the crates a change adds, removes or upgrades in `Cargo.lock`, with each one's license and transitive count, and CI writes that list to the `deny` job's summary when a PR changes `Cargo.lock`.

1. What does it do, and why can't std or an existing dependency do it in reasonable code?
2. Is it maintained (recent releases, responsive maintainers) and widely used?
3. Does it use `unsafe` or FFI? Is that well-tested?
4. How many transitive dependencies does it add (`cargo tree -e normal -p <crate>`)?
5. Is its license allowed by `deny.toml`?

[`deny.toml`](../../deny.toml) policy, checked by the `deny` and `advisories` steps of `cargo xtask ci`:

- **Scope:** every dependency on the supported targets, including optional ones (`all-features = true`).
- **Advisories:** vulnerable, unsound, unmaintained and yanked crates are errors. Each ignore has a reason and an expiry date, written as `{ id = "RUSTSEC-…", reason = "…", expiry = "YYYY-MM-DD" }`. The `manifests` step requires both fields; cargo-deny rejects a malformed date and reports the advisory again once the date passes.
- **Licenses:** an allow-list of permissive licenses (MIT, Apache-2.0, BSD-2/3-Clause, ISC, Unicode-3.0, Zlib). An exception names the crate, the license and the reason.
- **Bans:** duplicate versions warn and wildcard versions are errors. `openssl`, `openssl-sys` and `native-tls` are denied, to keep TLS on rustls (ADR-0006).
- **Sources:** crates.io only. No other registries and no git dependencies (the `[sources]` section of `deny.toml`).

Fixture tests in `xtask/tests/deny.rs` prove that a banned crate, a wildcard version and a GPL-licensed dependency are rejected. In CI, `deny` is a required job. `advisories` runs on every PR without blocking it, and [`advisories.yml`](../../.github/workflows/advisories.yml) runs it daily: a failure opens a tracking issue naming the advisory ids, or comments on the open one.

`Cargo.lock` is committed. Dependency updates are separate PRs: Dependabot opens one PR per crate each week ([`.github/dependabot.yml`](../../.github/dependabot.yml)). CI pins GitHub Actions by commit SHA, with the version in a comment; `cargo xtask check-workflows` enforces that, along with job timeouts and explicit permissions, and Dependabot proposes SHA bumps weekly.

## Durability as a security property

- CRC32C is validated where Kafka validates it: on produce (record validation), on log recovery, on follower fetch, and by `verify log`.
- After an unclean shutdown, recovery validates the active segments and truncates at the first invalid batch, recording an event with the truncated range.
- The fsync policy is explicit and documented (backlog STOR-F17, which ends in an ADR). Acknowledgement semantics never claim more durability than the policy provides.
