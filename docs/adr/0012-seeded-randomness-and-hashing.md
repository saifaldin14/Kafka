# ADR-0012: Seeded randomness and hashing

- **Status:** Accepted
- **Date:** 2026-11-21

## Context

[ADR-0002](0002-sans-io-state-machines-driven-by-tokio.md) and [ADR-0003](0003-deterministic-simulation-testing.md) require all randomness to come from an injected, seeded RNG, so a simulation seed replays a run exactly. `clippy.toml` already bans `rand`'s unseeded entry points and `std`'s randomly seeded `HashMap`/`HashSet`, but nothing yet says what replaces them.

Requirements:

- **Replay:** the same seed gives the same output on every supported platform (x86_64 and aarch64, Linux and macOS) and across dependency upgrades, since a stored failing seed must still replay after an upgrade.
- **Independence:** components get their own streams, so adding randomness to one component doesn't change what another draws in the same simulated run.
- **Production unpredictability:** election timeouts, retry jitter and member ids must not be predictable from outside. Hash maps keyed by client-controlled strings (topic names, group ids, client ids) must resist hash flooding, which is why `std` seeds its `RandomState` randomly.
- **Secrets:** SCRAM salts and nonces, and delegation token secrets, need a cryptographically secure generator in production.

Points of reference:

- `rand` 0.10 documents `StdRng` as non-portable: "any future library version may replace the algorithm". It recommends a generator named after its algorithm, such as `ChaCha12Rng`, for reproducible output. Its distributions (`random_range` and the like) may change their output in any breaking release, which for `rand` 0.x is every minor version.
- `std`'s `HashMap` hashes with SipHash-1-3 under a random 128-bit key.
- Kafka uses `java.util.Random` and `ThreadLocalRandom` for jitter and `SecureRandom` for SCRAM; none of it is seedable for replay.

## Decision

1. **One RNG type, `SeededRng`, in `alluvium-types`, generating the ChaCha12 keystream.**
   - It uses the `chacha20` crate's `ChaCha12Rng`, with the `rng` feature and no default features, from a 256-bit key.
   - There is no RNG trait: there is one implementation, and tests make it deterministic by seeding it, so no test seam is needed.
   - Its methods are ours, written on top of raw keystream words, so output depends only on ChaCha12 and this crate, never on `rand`'s distributions:
     - `next_u32`, `next_u64`, `fill_bytes`;
     - `below(bound)`, unbiased, using Lemire's multiply-and-reject method;
     - `fork(label)`.
   - `Debug` prints `SeededRng { .. }` and never the key or position.

2. **Seeds.**
   - Simulation and tests: `SeededRng::from_seed(seed: u64)`. The `u64` is expanded to the 256-bit key with four SplitMix64 outputs, little-endian. The expansion is fixed in our code, not delegated to `rand_core`'s `seed_from_u64`, whose output may change.
   - Production: `SeededRng::from_os_entropy()` reads 32 bytes with `getrandom::fill`, the only OS-entropy call in the workspace, under one `#[expect(clippy::disallowed_methods, reason = "…")]`. The binary calls it once at startup and forks everything else from it. If the OS can't provide entropy, startup fails with a typed error; there is no weaker fallback.
   - As with `SystemClock` ([ADR-0011](0011-time-model-and-clock-injection.md)), layers 0–2 never call `from_os_entropy`; their state machines receive a `SeededRng` (or values drawn from one) from their driver.

3. **Forks are independent, and adding one changes nothing else.**
   - `fork(label)` derives the child's key from the parent's key and the label only. It is the first 32 bytes of ChaCha12 under the parent's key, on a stream number chosen by the label's SipHash-1-3 (with a fixed key, and never 0, which is the parent's own stream).
   - Forking never consumes the parent's stream, so adding or removing a fork doesn't shift any other stream.
   - ChaCha is a pseudorandom function of its key, so seeing one stream's output reveals nothing about its parent or siblings.
   - Labels name the component, plus an instance where there are many (`"raft"`, `"group-coordinator/payments"`).

4. **The keyed hasher is SipHash-1-3 with a 128-bit key from the RNG.**
   - It is the `siphasher` crate's `sip::SipHasher13`, the same algorithm and flooding resistance as `std`'s `HashMap`, but keyed from `SeededRng`, so iteration order replays in simulation.
   - `alluvium_types::collections::{SeededHashMap, SeededHashSet}` take a `HashSeed`, which is drawn with `HashSeed::from_rng(&mut SeededRng)`.
   - `HashSeed`'s `Debug` prints `HashSeed(***)`.
   - Iteration order stays an implementation detail. It must never reach output or decisions; where order is observable, use `BTreeMap`.

5. **Logging.**
   - Simulation and test seeds are always printed on failure, with the replay command (debuggability standard).
   - Production RNG keys and hash seeds are never logged, printed, exported or put in metrics or debug endpoints. Knowing them would let an attacker build colliding keys (hash flooding) or predict election timeouts and jitter. The redacting `Debug` impls make this hold even when a struct holding one is printed.

6. **Secrets use the same RNG.**
   - ChaCha12 keyed from OS entropy is a cryptographically secure generator; it is how `rand` builds `ThreadRng`. Forks of the production root may therefore generate salts, nonces and token secrets, and in simulation those become deterministic, which tests want.
   - TLS is the exception: rustls draws from its own crypto provider, and simulation doesn't model TLS randomness.

7. **The lints cover the remaining entropy sources.**
   - `clippy.toml` adds `getrandom::fill`, `getrandom::fill_uninit`, `getrandom::u32` and `getrandom::u64`, each with a lint-canary fixture.
   - The reasons on `rand::make_rng` and `rand::rngs::SysRng` point to `SeededRng::from_os_entropy`.

## Consequences

- One type serves simulation replay, production unpredictability and secret generation, so there is one thing to learn, test and audit.
- Golden tests (TYPE-0029) pin the first outputs of fixed seeds and forks. They run on Linux and macOS, x86_64 and aarch64, covering `chacha20`'s SIMD backends. A dependency upgrade that changed the output would fail them before any stored seed broke.
- Three dependencies enter `alluvium-types`, each justified in its task per security.md#dependencies:
  - `chacha20`, from RustCrypto, already in `Cargo.lock` through `rand`; its unsafe code is limited to its SIMD backends and buffering.
  - `siphasher`, with no dependencies.
  - `getrandom`, maintained by the `rand` project; its unsafe code is the OS calls.
- Plan changes:
  - TYPE-0029 uses `SeededRng::from_os_entropy` (not `ProcessRng`).
  - TYPE-0030 uses `HashSeed`.
  - FND-0064 adds the `getrandom` bans and updates the two reasons.

## Alternatives considered

- **`rand`'s `StdRng` and distributions.** Rejected: `StdRng` may change algorithm and the distributions may change output between `rand` releases, so stored seeds could stop replaying.
- **xoshiro256++ (or another small non-cryptographic generator), implemented in our crate.** Rejected:
  - Its state can be recovered from its outputs, so production hash keys and jitter would be predictable from observed behavior.
  - We would need a second, cryptographic generator for secrets, which means two RNG types and two seeding paths.
- **ChaCha20 or ChaCha8.** ChaCha20 costs more for no practical security gain over 12 rounds. ChaCha8 has a smaller margin, and `rand` itself settled on 12.
- **An `Rng` trait with production and simulation implementations.** Rejected: a single implementation, deterministic by seeding, needs no seam. A trait would also invite a second, non-portable implementation.
- **`ahash` or `foldhash` for hash maps.** Rejected: `ahash`'s output isn't stable across versions and platforms, and `foldhash` documents only minimal resistance to hash flooding.
- **`std`'s deprecated `SipHasher::new_with_keys`.** Rejected: it is deprecated (SipHash-2-4) and would need a lint exception at every use.
- **A fork that draws the child's seed from the parent's stream.** Rejected: adding a fork would shift every later draw and fork, so unrelated changes would invalidate stored seeds.
