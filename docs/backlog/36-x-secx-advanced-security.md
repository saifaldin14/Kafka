# X-SECX: Advanced security

> Crates: `alluvium-security`, `alluvium-storage` · Kafka counterpart: none (Alluvium enhancement) · Milestones: M14

Encryption at rest and field-level encryption, attribute-based access control, audit logging, credential lifecycle and data-erasure features.

## [To Do] X-SECX-F01 Advanced security design
P0 · M14 · Feature · deps: SEC-F07
Scope: Spike and ADR: policy engine and encryption architecture.

### [To Do] X-SECX-0001 Spike: advanced security architecture ADR (encryption layer, key hierarchy, policy engine, audit transport, crate placement)
P0 · M14 · Spike · deps: none · labels: security · ref: new ADR "Advanced security architecture"
Done when: ADR accepted that decides: encryption at the segment-store layer vs record-batch layer (sendfile, recovery and replication consequences); the key hierarchy (KMS root key → KEK → per-topic DEK, and per-subject keys for crypto-shredding); the policy engine (Cedar vs Rego vs native) and how it combines with ACLs; the audit transport and tamper-evidence scheme; error-code mapping strategy for key failures; crate placement for sans-I/O logic (layer 1) and KMS drivers (layer 3/4); rejected alternatives recorded.

### [To Do] X-SECX-0002 Advanced security threat model (STRIDE) with mitigation-to-task mapping
P0 · M14 · Task · deps: X-SECX-0001, SEC-F14 · labels: security, docs · ref: docs/standards/security.md
Done when: docs/security/threat-model-advanced-security.md covers disk/backup theft, stolen KMS credentials, compromised broker, malicious tenant admin, audit-log tampering, privilege escalation through tags/policies, API-key leakage and side channels; every threat maps to a mitigation and the X-SECX task IDs that test it; reviewed with the SEC-F14 checklist.

### [To Do] X-SECX-0003 Dependency review: AEAD, HKDF, SHA-256 and signing primitives from the rustls crypto provider
P0 · M14 · Task · deps: X-SECX-0001, FND-F03 · labels: security · ref: docs/standards/security.md#dependencies
Done when: the PR answers the five dependency questions for using the provider already pulled in by rustls (aws-lc-rs or ring) for AES-256-GCM, HKDF-SHA-256, HMAC-SHA-256 and Ed25519; no RustCrypto duplicates are added; `cargo deny check` passes; `forbid(unsafe_code)` holds in our crates.

### [To Do] X-SECX-0004 Key-material randomness seam: `KeyMaterialRng` backed by the OS CSPRNG, seeded only in tests
P0 · M14 · Task · deps: X-SECX-0001, TYPE-F07 · labels: security
Done when: `KeyMaterialRng` produces DEKs, salts, nonces and API-key secrets; production wiring uses the OS CSPRNG via the crypto provider; the seeded test implementation is only constructible under `cfg(test)` or in `alluvium-testkit`; a test fails the build if a production constructor accepts a seed.

### [To Do] X-SECX-0005 `alluvium.security.version` finalized feature gating advanced-security metadata records
P0 · M14 · Task · deps: X-SECX-0001, META-F06, CTRL-F17 · labels: security
Done when: feature level 0 (default) makes the controller reject every X-SECX record type with UNSUPPORTED_VERSION; level 1 enables them; `UpdateFeatures` downgrade to 0 is refused while any key, policy, tag, API-key or masking record exists; unit tests per record type.

### [To Do] X-SECX-0006 Register the `alluvium.security.*`, `alluvium.encryption.*`, `alluvium.kms.*`, `alluvium.audit.*` and `alluvium.policy.*` namespaces
P0 · M14 · Task · deps: X-SECX-0001, CFG-F11 · labels: security
Done when: each prefix is in the CFG-F11 registry linked to the advanced-security ADR with an owner feature; the registry check rejects unregistered keys under these prefixes; docs/config.md lists the namespaces.

## [To Do] X-SECX-F02 Encryption at rest
P0 · M14 · Feature · deps: X-SECX-F01, STOR-F02
Scope: Per-topic data keys, segment encryption, key rotation.

### [To Do] X-SECX-0007 Encrypted segment file format v1 specification
P0 · M14 · Task · deps: X-SECX-0001 · labels: security, docs · ref: new ADR "Advanced security architecture"
Done when: docs/formats/encrypted-segment.md specifies the header (magic, format version 1, cipher suite id, DEK id and version, per-file random salt, frame-size limit, header MAC), the frame layout (length, ciphertext, 16-byte GCM tag), the nonce construction (salt ‖ 32-bit frame counter) and the AAD (topic id, partition, segment base offset, file kind, frame index, format version); the rule that a frame is never re-sealed is stated.

### [To Do] X-SECX-0008 Encryption data model: `DataKeyId`, `KeyVersion`, `CipherSuite`, `DataKey`, `WrappedDataKey`, `EncryptionHeader`
P0 · M14 · Task · deps: X-SECX-0007, TYPE-F08 · labels: security
Done when: `DataKey` wraps `Secret<[u8; 32]>`, zeroizes on drop and has no `Clone` or plain `Debug`; `CipherSuite` has only `Aes256Gcm` in v1 with an exhaustive wire-id mapping; unit tests cover redacted `Debug`, zeroize on drop and id round-trips.

### [To Do] X-SECX-0009 Encryption header encode/decode with bounds and typed `EncryptionHeaderError`
P0 · M14 · Task · deps: X-SECX-0008 · labels: security, decoder
Done when: decode rejects unknown magic, unknown format version, unknown cipher suite, oversize frame limit and a bad header MAC with distinct variants carrying the file path; property test round-trips arbitrary headers; fuzz target `fuzz_encryption_header` is registered with TEST-F07 and runs clean.

### [To Do] X-SECX-0010 Frame seal and open (AES-256-GCM) as a sans-I/O `FrameCipher`
P0 · M14 · Task · deps: X-SECX-0008, X-SECX-0003 · labels: security
Done when: `seal(frame_index, plaintext)` and `open(frame_index, ciphertext)` bind the AAD fields; NIST GCM known-answer vectors pass; flipping any ciphertext, tag or header byte returns `FrameAuthenticationFailed { topic, partition, segment_base_offset, frame_index }`; swapping frames between positions, segments or partitions fails authentication.

### [To Do] X-SECX-0011 Nonce uniqueness across partial flushes, truncation and segment rewrites
P0 · M14 · Task · deps: X-SECX-0010 · labels: security
Done when: partial-frame flushes start a new frame with the next counter instead of re-sealing; truncation and every rewrite (compaction, re-encryption, erasure) start a file with a fresh salt; a property test over random append/flush/truncate/rewrite sequences asserts no (DEK, salt, counter) triple is used twice; the counter-overflow path rolls the segment before wrapping.

### [To Do] X-SECX-0012 Plaintext-to-ciphertext position mapping for indexes and positional reads
P0 · M14 · Task · deps: X-SECX-0010 · labels: security
Done when: offset and time indexes keep plaintext positions; an in-memory sparse frame map (bounded by frames per segment) translates a plaintext position to its frame and in-frame offset; it is rebuilt by scanning frame headers on load; a property test shows positional reads of the encrypted file equal reads of the plaintext file for arbitrary positions and lengths.

### [To Do] X-SECX-0013 Key-encryption seam: `KeyEncryptionService` trait (wrap, unwrap, describe key) with typed `KmsError`
P0 · M14 · Task · deps: X-SECX-0008 · labels: security
Done when: the driver-side trait has `wrap`, `unwrap` and `describe_key` with per-call timeouts; `KmsError` has `Unavailable`, `AccessDenied`, `KeyDisabled`, `KeyNotFound`, `Throttled` and `InvalidCiphertext`, each carrying the key reference; an in-memory fake for tests is documented as the test seam; no key material appears in `Debug` output.

### [To Do] X-SECX-0014 Local KEK provider for development and air-gapped clusters
P1 · M14 · Task · deps: X-SECX-0013, SEC-F11 · labels: security
Done when: `alluvium.kms.provider=local` reads a 256-bit KEK from a password-type config or a key file; the file must not be group- or world-readable or startup fails with a clear error; a non-loopback cluster refuses this provider unless `alluvium.kms.local.allow.production=true`; KEK versioning supports rotation; unit tests cover each rejection.

### [To Do] X-SECX-0015 DEK lifecycle state machine: Pending → Active → DecryptOnly → Destroyed
P0 · M14 · Task · deps: X-SECX-0008, X-SECX-0004 · labels: security
Done when: the sans-I/O machine guarantees at most one Active DEK per topic; transitions return events with reasons `created`, `rotated_by_age`, `rotated_by_bytes`, `kek_rotated`, `compromised`, `unreferenced` and `shredded`; Destroyed is terminal; a property test over random transition sequences holds these invariants.

### [To Do] X-SECX-0016 Metadata records for topic data keys: `TopicDataKeyRecord` and `RemoveTopicDataKeyRecord`
P0 · M14 · Task · deps: X-SECX-0015, X-SECX-0005, META-F02 · labels: security
Done when: records carry topic id, DEK id, version, wrapped DEK, KEK reference and state; the controller validates state transitions and rejects duplicates; replay builds per-topic key sets in `MetadataImage`; the snapshot round-trip test passes; plaintext DEKs never enter the metadata log (test inspects records).

### [To Do] X-SECX-0017 Controller DEK provisioning on topic creation and on enabling encryption
P0 · M14 · Task · deps: X-SECX-0016, X-SECX-0013 · labels: security
Done when: CreateTopics with `alluvium.encryption.enabled=true` waits for a wrapped DEK from the controller's KMS driver within `alluvium.encryption.kms.timeout.ms`; on KMS failure the topic is not created and the error maps per the ADR; enabling encryption on an existing topic provisions a DEK used by segments rolled afterwards; tests cover success, timeout and access-denied.

### [To Do] X-SECX-0018 Broker DEK cache with bounded size, TTL and single-flight unwrap
P0 · M14 · Task · deps: X-SECX-0016, X-SECX-0013 · labels: security
Done when: the cache is bounded by `alluvium.encryption.dek.cache.max.entries` with TTL `alluvium.encryption.dek.cache.ttl.ms`; concurrent misses for one DEK cause a single unwrap call; evicted keys are zeroized; metrics `alluvium_encryption_dek_cache_hits_total`, `_misses_total` and `alluvium_encryption_dek_unwrap_seconds` exist; tests use the fake KMS.

### [To Do] X-SECX-0019 Encrypting segment store: append path over the STOR-F02 segment store
P0 · M14 · Task · deps: X-SECX-0009, X-SECX-0011, X-SECX-0012, X-SECX-0018, STOR-F03 · labels: security
Done when: appends to encrypted partitions write the header then sealed frames; flush and fsync semantics are unchanged; the segment's DEK id is recorded in the header; a canary-payload test scans the written files and finds no plaintext; append of an unencrypted topic is byte-identical to the plain segment store.

### [To Do] X-SECX-0020 Decrypting read path with bounded buffers and sendfile bypass
P0 · M14 · Task · deps: X-SECX-0019, NET-F06 · labels: security, perf
Done when: positional reads decrypt only the frames covering the requested range using buffers bounded by `alluvium.encryption.read.buffer.bytes`; fetch responses for encrypted partitions use the copy path instead of sendfile and count `alluvium_encryption_sendfile_bypassed_total`; Fetch tests read back identical records for encrypted and plain topics.

### [To Do] X-SECX-0021 Mixed plaintext and encrypted segments in one partition
P0 · M14 · Task · deps: X-SECX-0020 · labels: security
Done when: a partition whose older segments are plaintext and newer ones encrypted serves reads, index lookups, retention and truncation across the boundary; the segment kind is detected from the header magic, never from topic config; tests cover enabling encryption mid-life and reading across the boundary.

### [To Do] X-SECX-0022 Log recovery of encrypted segments
P0 · M14 · Task · deps: X-SECX-0020, STOR-F09 · labels: security
Done when: an authentication failure in the tail frame is treated as a torn write and truncated like a CRC failure; a failure before the tail marks the segment corrupt and follows the STOR-F09 corruption path; a missing DEK leaves the partition offline with event `encryption_key_unavailable` and reason; recovery tests cover each case.

### [To Do] X-SECX-0023 Crash-consistency tests for the encrypted append path
P0 · M14 · Task · deps: X-SECX-0022, TEST-F03 · labels: security, test
Done when: the simulated disk crashes at every write point of append, flush, roll and truncate with torn writes and lost unsynced data; after recovery every acknowledged record decrypts and no frame counter is reused (checked by an invariant); the test prints its seed.

### [To Do] X-SECX-0024 DEK rotation by age and bytes with reference tracking
P0 · M14 · Task · deps: X-SECX-0017, X-SECX-0019 · labels: security
Done when: a new DEK becomes Active after `alluvium.encryption.dek.rotation.ms` or `alluvium.encryption.dek.rotation.bytes` and is used for the next rolled segment; the previous DEK becomes DecryptOnly; once retention or compaction removes the last segment that references it, it moves to Destroyed with reason `unreferenced`; a simulation test checks that no segment ever references a Destroyed DEK.

### [To Do] X-SECX-0025 KEK rotation: controller re-wraps all DEKs without rewriting data
P0 · M14 · Task · deps: X-SECX-0017 · labels: security
Done when: detecting a new KEK version (or an admin request) makes the controller re-wrap every DEK in bounded batches (`alluvium.encryption.rewrap.batch.size`), writing updated records; re-wrap resumes after controller failover; event `dek_rewrapped` and gauge `alluvium_encryption_deks_pending_rewrap` exist; a simulation test kills the active controller mid-re-wrap and asserts every DEK still unwraps.

### [To Do] X-SECX-0026 Re-encryption of segments under a new DEK after a key compromise
P1 · M14 · Task · deps: X-SECX-0024, STOR-F14 · labels: security
Done when: an admin request marks a DEK compromised; a throttled background rewriter (`alluvium.encryption.reencrypt.max.bytes.per.second`) rewrites every referencing segment under the Active DEK with fresh salts, swapping atomically like the cleaner; progress metrics exist; the compromised DEK is Destroyed when no segment references it; crash tests cover interrupted rewrites.

### [To Do] X-SECX-0027 Replication of encrypted topics
P0 · M14 · Task · deps: X-SECX-0019, REPL-F06 · labels: security
Done when: followers receive plaintext batches and encrypt them locally with the topic's Active DEK; startup fails with a clear error if an encrypted topic exists and the inter-broker listener is not TLS; a test shows leader and follower files differ byte-for-byte but decrypt to identical batches; truncation on followers works on encrypted segments.

### [To Do] X-SECX-0028 Encryption of producer-state snapshots and transaction indexes; documented plaintext files
P0 · M14 · Task · deps: X-SECX-0019, STOR-F15, STOR-F06 · labels: security
Done when: `.snapshot` and `.txnindex` files of encrypted partitions use the v1 format; offset and time indexes and checkpoint files stay plaintext, as documented in docs/formats/encrypted-segment.md with the reasoning; recovery tests cover the encrypted files.

### [To Do] X-SECX-0029 Compaction and retention on encrypted partitions
P0 · M14 · Task · deps: X-SECX-0019, STOR-F14 · labels: security
Done when: the cleaner decrypts input segments and writes cleaned output under the Active DEK with fresh salts; DEK reference counts are updated on swap; retention deletes decrement references; a compaction test over an encrypted topic matches the plaintext result record-for-record.

### [To Do] X-SECX-0030 Tiered storage of encrypted segments
P1 · M14 · Task · deps: X-SECX-0020, TIER-F03, TIER-F04 · labels: security
Done when: segments are copied to remote storage as ciphertext with their headers; remote reads decrypt with the referenced DEK; a DEK stays DecryptOnly while any remote segment references it; a test asserts uploaded objects contain no plaintext canary.

### [To Do] X-SECX-0031 `EncryptionError` and its single exhaustive Kafka error-code mapping
P0 · M14 · Task · deps: X-SECX-0020, X-SECX-0017, ERR-F02 · labels: security
Done when: one exhaustive `match` (no `_ =>` arm) maps every variant to the Kafka error code chosen in the ADR for Produce, Fetch, CreateTopics and AlterConfigs; the mapping table is in docs/formats/encrypted-segment.md; a test per variant asserts the code; no variant leaks key material in its message.

### [To Do] X-SECX-0032 Encryption config keys: topic and broker keys with validation
P0 · M14 · Task · deps: X-SECX-0024, X-SECX-0006 · labels: security
Done when: topic keys `alluvium.encryption.enabled` and `alluvium.encryption.kek.id`, and broker keys `alluvium.encryption.dek.rotation.ms`, `.dek.rotation.bytes`, `.dek.cache.max.entries`, `.dek.cache.ttl.ms`, `.read.buffer.bytes` and `.kms.timeout.ms` are defined with types, defaults, validators and docs; disabling encryption on an encrypted topic is rejected with INVALID_CONFIG; DescribeConfigs reports them.

### [To Do] X-SECX-0033 Encryption metrics, events and flight-recorder entries
P0 · M14 · Task · deps: X-SECX-0024, X-SECX-0025, OBS-F04, OBS-F06, OBS-F07 · labels: security
Done when: metrics `alluvium_encryption_bytes_encrypted_total`, `_bytes_decrypted_total`, `_auth_failures_total` and `_active_deks` exist with bounded labels; events `dek_created`, `dek_rotated`, `dek_destroyed`, `dek_rewrapped` and `encryption_key_unavailable` are in the event catalog with reasons; DEK transitions appear in the partition flight recorder; a test asserts no key bytes appear in any of them.

### [To Do] X-SECX-0034 Encrypted segment support in `alluvium verify log` and `alluvium dump log`
P1 · M14 · Task · deps: X-SECX-0020, CLI-F04 · labels: security
Done when: without key access both tools print the encryption header and frame count and verify frame structure; with `--kms-config` they decrypt and run the normal checks; DEKs are never printed; tests cover both modes and a tampered frame.

### [To Do] X-SECX-0035 Fuzz target for loading encrypted segments from disk
P0 · M14 · Task · deps: X-SECX-0022, TEST-F07 · labels: security, decoder, test
Done when: `fuzz_encrypted_segment_load` feeds arbitrary bytes as an encrypted segment with a fixed test DEK through header decode, frame-map rebuild and recovery; it runs clean with no panic and bounded allocation in the CI short run and nightly.

### [To Do] X-SECX-0036 Simulation: key rotation, KMS outages and broker crashes on encrypted topics
P0 · M14 · Task · deps: X-SECX-0024, X-SECX-0025, X-SECX-0027, TEST-F06 · labels: security, test
Done when: a seeded simulation with a fake KMS that fails and recovers asserts that no acknowledged acks=all record becomes unreadable while its KEK exists, that no plaintext canary appears on any simulated disk, and that at most one DEK per topic is Active; failures print the seed and flight recorders.

### [To Do] X-SECX-0037 Encryption benchmarks and overhead budget
P0 · M14 · Task · deps: X-SECX-0020, TEST-F11 · labels: perf, nfr
Done when: criterion benchmarks for frame seal/open and macro produce/fetch benchmarks with encryption on and off are in CI with regression thresholds; results are published in docs/benchmarks/encryption.md against the overhead budget set in the ADR; memory per encrypted partition is measured and bounded.

## [To Do] X-SECX-F03 KMS integrations
P0 · M14 · Feature · deps: X-SECX-F02
Scope: AWS KMS, GCP KMS, Azure Key Vault, HashiCorp Vault.

### [To Do] X-SECX-0038 Shared KMS driver infrastructure: timeouts, retries with backoff, concurrency limit and supervised tasks
P0 · M14 · Task · deps: X-SECX-0013, SRV-F02 · labels: security
Done when: every provider call has a timeout (`alluvium.kms.request.timeout.ms`), retries only retriable `KmsError` kinds with jittered backoff from the injected RNG, and runs under a bounded concurrency limit (`alluvium.kms.max.in.flight`); driver tasks are named and supervised; unit tests cover retry classification and limit saturation.

### [To Do] X-SECX-0039 KMS provider selection and per-provider config keys
P0 · M14 · Task · deps: X-SECX-0038, X-SECX-0006 · labels: security
Done when: `alluvium.kms.provider` (`none`, `local`, `aws`, `gcp`, `azure`, `vault`) and each provider's keys (region, endpoint, key id, credential source) are defined with validators; credential keys are password-typed; the config check rejects a provider whose required keys are missing with a message naming the key.

### [To Do] X-SECX-0040 KMS credential handling: secrets, config providers and hot reload
P0 · M14 · Task · deps: X-SECX-0039, CFG-F02, SEC-F11 · labels: security
Done when: provider credentials load through `Secret<T>` and KIP-297 config providers; credential rotation reloads without restart and is tested with each provider's fake; OBS-F11 redaction tests cover logs, metrics, `/debug/config` and errors.

### [To Do] X-SECX-0041 Dependency review: AWS KMS client (`aws-sdk-kms` vs a minimal SigV4 client on existing hyper and rustls)
P0 · M14 · Task · deps: X-SECX-0038, FND-F03 · labels: security
Done when: the review records transitive crate count, `unsafe` and FFI use, TLS stack (must be rustls), license and maintenance for both options; the chosen option is added with minimal features; `cargo deny check` passes.

### [To Do] X-SECX-0042 AWS KMS provider: wrap and unwrap with encryption context and credential sources
P0 · M14 · Task · deps: X-SECX-0041, X-SECX-0040 · labels: security
Done when: wrap uses `Encrypt` and unwrap `Decrypt` with encryption context `{cluster_id, topic_id}`; credentials come from static keys, web identity (IRSA) or IMDSv2; AWS errors map to `KmsError` kinds exhaustively; unit tests use recorded responses.

### [To Do] X-SECX-0043 AWS KMS integration tests against a pinned LocalStack container
P0 · M14 · Task · deps: X-SECX-0042, TEST-F08 · labels: security, test
Done when: the compat harness runs wrap/unwrap, key disable (KeyDisabled), access denied, throttling and endpoint outage against LocalStack pinned by digest; each maps to the expected `KmsError` and broker behavior.

### [To Do] X-SECX-0044 Dependency review: Google Cloud KMS client
P0 · M14 · Task · deps: X-SECX-0038, FND-F03 · labels: security
Done when: the review compares available Google Cloud KMS crates with a minimal REST client on existing dependencies (transitive count, `unsafe`, rustls-only, license, maintenance) and records the choice; `cargo deny check` passes.

### [To Do] X-SECX-0045 Google Cloud KMS provider with additional authenticated data and credential sources
P0 · M14 · Task · deps: X-SECX-0044, X-SECX-0040 · labels: security
Done when: wrap and unwrap use `encrypt`/`decrypt` with additional authenticated data `{cluster_id, topic_id}`; credentials come from a service-account key, workload identity federation or the metadata server; errors map exhaustively to `KmsError`; unit tests use recorded responses.

### [To Do] X-SECX-0046 Google Cloud KMS contract tests: recorded fake in CI and a manual real-KMS job
P0 · M14 · Task · deps: X-SECX-0045, TEST-F08 · labels: security, test
Done when: a fake HTTP server replays recorded GCP responses for success, permission denied, disabled key version and 429 in CI; a manually triggered job runs the same suite against a real test project; both pass.

### [To Do] X-SECX-0047 Dependency review: Azure Key Vault client
P0 · M14 · Task · deps: X-SECX-0038, FND-F03 · labels: security
Done when: the review compares the Azure SDK crates with a minimal REST client (transitive count, `unsafe`, rustls-only, license, maintenance) and records the choice; `cargo deny check` passes.

### [To Do] X-SECX-0048 Azure Key Vault and Managed HSM provider (wrapKey and unwrapKey)
P0 · M14 · Task · deps: X-SECX-0047, X-SECX-0040 · labels: security
Done when: wrap and unwrap use `wrapKey`/`unwrapKey` with RSA-OAEP-256 (Key Vault) or AES-KW (Managed HSM) per configured key type; credentials come from client secret, client certificate, managed identity or workload identity; key versions are tracked for re-wrap; errors map exhaustively; unit tests use recorded responses.

### [To Do] X-SECX-0049 Azure Key Vault contract tests: recorded fake in CI and a manual real-vault job
P0 · M14 · Task · deps: X-SECX-0048, TEST-F08 · labels: security, test
Done when: recorded-response tests cover success, forbidden, disabled key, throttling and outage in CI; a manual job runs against a real test vault; both pass.

### [To Do] X-SECX-0050 Dependency review: HashiCorp Vault client
P0 · M14 · Task · deps: X-SECX-0038, FND-F03 · labels: security
Done when: the review compares existing Vault crates with a minimal HTTP client for the Transit engine (transitive count, `unsafe`, rustls-only, license, maintenance) and records the choice; `cargo deny check` passes.

### [To Do] X-SECX-0051 HashiCorp Vault Transit provider with token lease renewal
P0 · M14 · Task · deps: X-SECX-0050, X-SECX-0040 · labels: security
Done when: wrap and unwrap use Transit `encrypt`/`decrypt` with derivation context `{cluster_id, topic_id}`; auth supports token, AppRole and Kubernetes; token leases are renewed before expiry by a supervised task and re-login happens on renewal failure; key version changes trigger re-wrap; errors map exhaustively.

### [To Do] X-SECX-0052 Vault integration tests against a pinned Vault dev-server container
P0 · M14 · Task · deps: X-SECX-0051, TEST-F08 · labels: security, test
Done when: tests cover wrap/unwrap, key rotation and re-wrap, lease expiry and renewal, sealed vault (Unavailable) and policy denial (AccessDenied).

### [To Do] X-SECX-0053 KMS outage behavior: serve from cached DEKs, defer key operations, keep partitions offline only when unavoidable
P0 · M14 · Task · deps: X-SECX-0038, X-SECX-0018, X-SECX-0024 · labels: security
Done when: during an outage cached DEKs keep serving; rotation, provisioning and re-wrap are deferred and retried; a broker starting during an outage keeps partitions whose DEK is not cached offline with event `encryption_key_unavailable` and brings them online when the KMS recovers; gauge `alluvium_kms_available` reflects state; a simulation test covers start, outage and recovery.

### [To Do] X-SECX-0054 Bounded parsing and fuzzing of KMS responses
P0 · M14 · Task · deps: X-SECX-0042, X-SECX-0045, X-SECX-0048, X-SECX-0051, TEST-F07 · labels: security, decoder, test
Done when: every provider response parser bounds body size (`alluvium.kms.max.response.bytes`) before parsing; fuzz targets `fuzz_kms_response_aws`, `_gcp`, `_azure` and `_vault` run clean; negative tests cover a KMS returning a key of the wrong length, malformed ciphertext and oversized bodies, each producing a typed error.

### [To Do] X-SECX-0055 KMS metrics, events and spans
P0 · M14 · Task · deps: X-SECX-0038, OBS-F04, OBS-F07 · labels: security
Done when: `alluvium_kms_requests_total{provider,op,outcome}` and `alluvium_kms_request_seconds{provider,op}` exist; events `kms_unavailable` and `kms_recovered` carry reasons; spans carry `kms.provider` and `kms.key_ref` but never key material or credentials (asserted by a redaction test).

### [To Do] X-SECX-0056 KMS setup guide per provider with least-privilege policies
P0 · M14 · Task · deps: X-SECX-0042, X-SECX-0045, X-SECX-0048, X-SECX-0051, DOCS-F01 · labels: docs, security
Done when: docs/security/kms.md has, per provider, the setup steps, a minimal IAM/role/policy example granting only wrap/unwrap on one key, credential source options, rotation procedure and outage behavior; every config key links to the generated reference.

## [To Do] X-SECX-F04 Client-side field encryption
P1 · M14 · Feature · deps: X-SECX-F03, X-NATIVE-F12
Scope: SDK field-level encryption driven by schema annotations.

### [To Do] X-SECX-0057 Spike: field-encryption key distribution (wrapped DEK per record header vs DEK registry)
P1 · M14 · Spike · deps: X-SCHEMA-F03 · labels: security · ref: new ADR "Client-side field encryption"
Done when: ADR accepted choosing how consumers obtain field DEKs (wrapped DEK in a record header vs a DEK registry next to the schema registry), the cipher mode (randomized AES-256-GCM in v1; deterministic mode deferred or justified), the annotation syntax per schema format and the behavior for consumers without decrypt permission.

### [To Do] X-SECX-0058 Field ciphertext envelope format v1 specification and cross-language test vectors
P1 · M14 · Task · deps: X-SECX-0057 · labels: security, docs
Done when: docs/formats/field-encryption.md defines the envelope (version, key id, nonce, ciphertext, tag), its encoding for Avro `bytes`, Protobuf `bytes` and JSON strings (base64url), and the AAD (subject, field path, schema id); `testdata/field-encryption/v1.json` holds vectors for each format.

### [To Do] X-SECX-0059 Field envelope codec with bounds and fuzz target
P1 · M14 · Task · deps: X-SECX-0058 · labels: security, decoder
Done when: decode rejects unknown version, truncated input and oversize fields with typed errors; the codec passes all v1 vectors; fuzz target `fuzz_field_envelope` runs clean.

### [To Do] X-SECX-0060 Schema annotation parsing into a `FieldEncryptionPlan`
P1 · M14 · Task · deps: X-SECX-0057, X-SCHEMA-F04, X-SCHEMA-F05, X-SCHEMA-F06 · labels: security
Done when: Avro field property `alluvium.encrypt`, a Protobuf custom field option and the JSON Schema keyword `x-alluvium-encrypt` produce a plan of field paths and key labels; annotations on unsupported positions (map keys, union branches the ADR excludes, non-string/bytes fields) are rejected with errors naming the path; unit tests per format.

### [To Do] X-SECX-0061 SDK key access: KMS providers behind cargo features and a bounded field-DEK cache
P1 · M14 · Task · deps: X-SECX-0057, X-SECX-0038 · labels: security
Done when: `alluvium-client` exposes the X-SECX-F03 providers behind cargo features (`kms-aws`, `kms-gcp`, `kms-azure`, `kms-vault`); field DEKs are generated per (subject, key label), wrapped, and cached with a bound and TTL; plaintext DEKs are zeroized on eviction; unit tests use the fake KMS.

### [To Do] X-SECX-0062 Avro serializer and deserializer integration
P1 · M14 · Task · deps: X-SECX-0060, X-SECX-0059, X-SECX-0061 · labels: security
Done when: the SDK Avro serializer encrypts annotated fields before encoding and the deserializer decrypts them; nested records and arrays of records work; round-trip tests use the registry; a test shows that bytes on the broker contain no plaintext canary.

### [To Do] X-SECX-0063 Protobuf serializer and deserializer integration
P1 · M14 · Task · deps: X-SECX-0060, X-SECX-0059, X-SECX-0061 · labels: security
Done when: annotated Protobuf fields (including nested messages and repeated fields) are encrypted and decrypted; round-trip and canary tests pass.

### [To Do] X-SECX-0064 JSON Schema serializer and deserializer integration
P1 · M14 · Task · deps: X-SECX-0060, X-SECX-0059, X-SECX-0061 · labels: security
Done when: annotated JSON fields are encrypted to base64url strings and decrypted; JSON Pointer paths into nested objects and arrays work; round-trip and canary tests pass.

### [To Do] X-SECX-0065 Consumer behavior without decrypt permission
P1 · M14 · Task · deps: X-SECX-0062, X-SECX-0063, X-SECX-0064 · labels: security
Done when: client config `field.encryption.on.missing.key` (`fail`, `ciphertext`, `null`) is honored for all three formats; `fail` raises a typed error naming the field path but not its value; tests cover each mode with a KMS that denies access.

### [To Do] X-SECX-0066 Negative tests: tampering, wrong key, schema evolution of annotations
P1 · M14 · Task · deps: X-SECX-0065 · labels: security, test
Done when: tampered ciphertext, swapped fields (AAD mismatch), wrong key id, removed annotation in a newer schema version and added annotation in a newer version all behave as specified in the ADR with typed errors; the schema compatibility check flags annotation removal on a field that has encrypted data.

### [To Do] X-SECX-0067 Documentation and vectors for implementing field encryption in other SDKs
P2 · M14 · Task · deps: X-SECX-0058, X-NATIVE-F13 · labels: docs, security
Done when: the guide documents the algorithm step by step for generated SDK implementers; the vectors are consumed by a test in each generated SDK's test suite (TypeScript, Python, Go, Java) or the gap is recorded as an issue per SDK.

### [To Do] X-SECX-0068 Field encryption benchmarks
P2 · M14 · Task · deps: X-SECX-0062, TEST-F11 · labels: perf
Done when: criterion benchmarks measure serializer overhead per encrypted field and per record for Avro, Protobuf and JSON with 1, 5 and 20 encrypted fields; results are in docs/benchmarks/field-encryption.md.

### [To Do] X-SECX-0069 Client-side field encryption guide
P1 · M14 · Task · deps: X-SECX-0065, X-SECX-0056 · labels: docs, security
Done when: docs/security/field-encryption.md covers annotating schemas, client config (off unless `field.encryption.enabled=true` and the schema is annotated), key management, rotation, permission modes and limitations (no server-side search on encrypted fields).

## [To Do] X-SECX-F05 Attribute-based access control
P1 · M14 · Feature · deps: X-SECX-F01
Scope: Principal attributes from JWT claims and certificates; resource tags.

### [To Do] X-SECX-0070 Attribute data model: `PrincipalAttributes` and `ResourceTags` with limits
P1 · M14 · Task · deps: SEC-F01 · labels: security
Done when: attributes are a bounded map of name → set of string values (`alluvium.abac.max.attributes`, `.max.values.per.attribute`, `.max.value.bytes`); tag keys and values follow a documented charset; types are deterministic (`BTreeMap`/`BTreeSet`); unit tests cover limits and validation errors.

### [To Do] X-SECX-0071 JWT claim to attribute mapping for SASL/OAUTHBEARER principals
P1 · M14 · Task · deps: X-SECX-0070, SEC-F05 · labels: security · ref: clients/src/main/java/org/apache/kafka/common/security/oauthbearer/BrokerJwtValidator.java @4.4.0
Done when: `alluvium.abac.jwt.claim.mappings` maps claim paths (nested objects, arrays flattened to sets) to attribute names; only claims from tokens that passed SEC-F05 validation are used; nesting depth and claim size are bounded; tests cover string, number, boolean, array and missing claims.

### [To Do] X-SECX-0072 Certificate attributes for mTLS principals: SAN, SPIFFE ID and subject RDNs
P1 · M14 · Task · deps: X-SECX-0070, NET-F07 · labels: security · ref: clients/src/main/java/org/apache/kafka/common/security/ssl/SslPrincipalMapper.java @4.4.0
Done when: attributes are derived from DNS and URI SANs, SPIFFE IDs (trust domain and path), and subject O, OU and CN per `alluvium.abac.certificate.attribute.mappings`; the SAN count is bounded before parsing; tests use certificates with many SANs, malformed SPIFFE IDs and missing RDNs.

### [To Do] X-SECX-0073 Static attributes for SCRAM and PLAIN users stored in metadata
P1 · M14 · Task · deps: X-SECX-0070, X-SECX-0005, CTRL-F14 · labels: security
Done when: `UserAttributesRecord` and its removal record are validated by the controller, replayed into the image and survive snapshots; SCRAM and PLAIN principals receive these attributes at authentication; unit and replay tests pass.

### [To Do] X-SECX-0074 Attribute binding to sessions and refresh on re-authentication
P1 · M14 · Task · deps: X-SECX-0071, X-SECX-0072, X-SECX-0073, SRV-F06, SEC-F12 · labels: security
Done when: attributes are resolved once at authentication and stored in the request context; KIP-368 re-authentication replaces them atomically; a session whose new token lacks a previously present attribute loses it immediately; memory per connection is bounded and measured.

### [To Do] X-SECX-0075 Resource tag records for topics, groups and transactional ids
P1 · M14 · Task · deps: X-SECX-0070, X-SECX-0005, META-F02 · labels: security
Done when: `ResourceTagRecord` and its removal record carry resource type, name or id and tags; the controller enforces `alluvium.abac.max.tags.per.resource`; topic deletion removes its tags; replay and snapshot round-trip tests pass.

### [To Do] X-SECX-0076 Tag administration: native admin API and `kafka-configs.sh`-compatible exposure
P1 · M14 · Task · deps: X-SECX-0075, X-NATIVE-F08 · labels: security
Done when: native admin endpoints set, remove and list tags; tags on topics and groups are also visible and settable through Incremental/AlterConfigs as `alluvium.tag.<key>` so `kafka-configs.sh` works; a compat test sets and reads a tag with the Java Admin client.

### [To Do] X-SECX-0077 Authorization for tag changes to prevent privilege escalation
P1 · M14 · Task · deps: X-SECX-0076, SEC-F08 · labels: security
Done when: setting or removing a tag requires ALTER_CONFIGS on the resource plus, when policies exist, a policy permit for action `TagResource` on that tag key; negative tests show a principal cannot tag a resource into a scope that would grant itself access.

### [To Do] X-SECX-0078 Attribute resolution metrics and events without attribute values
P1 · M14 · Task · deps: X-SECX-0074, OBS-F04, OBS-F07 · labels: security
Done when: `alluvium_abac_attribute_resolution_failures_total{source}` exists; event `principal_attributes_resolved` (debug level) lists attribute names and counts only; a redaction test asserts no attribute values or token contents appear in logs.

### [To Do] X-SECX-0079 Fuzz and negative tests for claim and certificate attribute extraction
P1 · M14 · Task · deps: X-SECX-0071, X-SECX-0072, TEST-F07 · labels: security, test, decoder
Done when: `fuzz_jwt_claim_mapping` and `fuzz_cert_attribute_extraction` run clean; negative tests cover deeply nested claims, huge arrays, duplicate keys and non-UTF-8 values, each producing a bounded typed error.

### [To Do] X-SECX-0080 ABAC guide: attribute sources and resource tags
P1 · M14 · Task · deps: X-SECX-0077, DOCS-F01 · labels: docs, security
Done when: docs/security/abac.md documents each attribute source and mapping key, tagging resources with both APIs, limits and the security considerations of tag administration.

## [To Do] X-SECX-F06 Policy engine
P1 · M14 · Feature · deps: X-SECX-F05
Scope: Policy language and engine with ACL compatibility.

### [To Do] X-SECX-0081 Dependency review for the policy engine chosen in the ADR
P1 · M14 · Task · deps: FND-F03 · labels: security
Done when: the review covers the chosen engine crate (for example `cedar-policy`) or records why a native engine was chosen: transitive count, `unsafe`, evaluation cost bounds, license and maintenance; `cargo deny check` passes.

### [To Do] X-SECX-0082 Policy model: effect, principal/action/resource scope and conditions
P1 · M14 · Task · deps: X-SECX-0081 · labels: security
Done when: `Policy` has id, effect (`Permit`, `Forbid`), principal scope (principal, attribute predicate), action scope (Kafka `AclOperation` values plus Alluvium actions such as `TagResource`, `ReadUnmasked` and `Erase`) and resource scope (resource type plus literal, prefix or tag predicate); types are deterministic and documented in module docs.

### [To Do] X-SECX-0083 Policy text parser with limits and positioned errors
P1 · M14 · Task · deps: X-SECX-0082 · labels: security, decoder
Done when: the parser enforces `alluvium.policy.max.bytes` and a nesting-depth limit; errors carry line and column; a property test round-trips pretty-printed policies; fuzz target `fuzz_policy_parse` runs clean.

### [To Do] X-SECX-0084 Policy validation against Kafka resource types, operations and known attributes
P1 · M14 · Task · deps: X-SECX-0083 · labels: security
Done when: policies naming unknown actions, resource types (Topic, Group, Cluster, TransactionalId, DelegationToken, User) or operations invalid for a resource type are rejected; unknown attribute names produce a warning list returned to the caller; tests per rule.

### [To Do] X-SECX-0085 Condition operators on attributes and tags: equality, set membership, presence, wildcard match
P1 · M14 · Task · deps: X-SECX-0082 · labels: security
Done when: `==`, `!=`, `in`, `containsAny`, `containsAll`, `has` and `like` (with `*` wildcard) are evaluated on principal attributes and resource tags with unit tests for empty sets, missing attributes (evaluate to false, never error-to-allow) and case sensitivity.

### [To Do] X-SECX-0086 Request-context conditions: listener, security protocol, client CIDR and API key scope
P1 · M14 · Task · deps: X-SECX-0082, SRV-F06 · labels: security
Done when: conditions on `context.listener`, `context.security_protocol`, `context.client_ip` (IPv4 and IPv6 CIDR match) and `context.api_key_id` evaluate from the request context; tests cover IPv4-mapped IPv6 addresses and malformed CIDRs (rejected at validation).

### [To Do] X-SECX-0087 Sans-I/O evaluator: default deny, forbid overrides, bounded cost, decision with matching policy ids
P1 · M14 · Task · deps: X-SECX-0084, X-SECX-0085, X-SECX-0086 · labels: security
Done when: evaluation is deterministic, denies when nothing permits, lets any matching forbid win, stops at `alluvium.policy.max.evaluation.steps` with deny and reason `evaluation_budget_exceeded`, and returns the matching policy ids; property tests show adding a forbid never grants and removing a permit never grants.

### [To Do] X-SECX-0088 Combined authorizer: StandardAuthorizer ACL semantics plus policies
P1 · M14 · Task · deps: X-SECX-0087, SEC-F07 · labels: security · ref: metadata/src/main/java/org/apache/kafka/metadata/authorizer/StandardAuthorizer.java authorize @4.4.0
Done when: the combination rule from the ADR is implemented (ACL deny or policy forbid denies; super users and `allow.everyone.if.no.acl.found` keep Kafka semantics); with zero policies every decision equals the SEC-F07 authorizer, proven by a property test over random ACL sets and requests.

### [To Do] X-SECX-0089 Policy storage as metadata records with controller validation
P1 · M14 · Task · deps: X-SECX-0084, X-SECX-0005, META-F02 · labels: security
Done when: `PolicyRecord` (id, version, text, SHA-256) and its removal record are validated (parse plus validate) by the controller before commit; the count is bounded by `alluvium.policy.max.policies`; replay compiles the policy set into the image; snapshot round-trip and invalid-policy rejection tests pass.

### [To Do] X-SECX-0090 Authorization helper integration with policy ids in `authz_decision` events
P1 · M14 · Task · deps: X-SECX-0088, X-SECX-0089, SEC-F08 · labels: security
Done when: every handler's authorization goes through the combined authorizer when `alluvium.policy.enabled=true`; Kafka's implicit DESCRIBE rules are preserved; `authz_decision` events include matching policy ids; handler tests for Produce, Fetch, CreateTopics and DescribeGroups pass with a policy granting and one forbidding.

### [To Do] X-SECX-0091 Policy administration: native admin CRUD with dry-run validation and CLI
P1 · M14 · Task · deps: X-SECX-0089, X-NATIVE-F08, CLI-F01 · labels: security
Done when: native endpoints create, update, delete, list and validate policies (dry run returns errors and warnings without committing); `alluvium policy apply|list|delete|validate` wraps them; administration requires ALTER on the cluster; tests cover each endpoint and authorization.

### [To Do] X-SECX-0092 `alluvium policy from-acls`: translate existing ACLs to equivalent policies
P2 · M14 · Task · deps: X-SECX-0088, X-SECX-0091 · labels: security
Done when: the tool exports all ACLs as policies; a property test shows that for random ACL sets the generated policies with ACLs removed produce the same decisions as the original ACLs, including prefixed and wildcard patterns and deny entries.

### [To Do] X-SECX-0093 `alluvium policy test`: offline what-if evaluation
P2 · M14 · Task · deps: X-SECX-0087, CLI-F01 · labels: security
Done when: the command evaluates request tuples (principal and attributes, action, resource and tags, context) from a file against a policy set and prints the decision and matching policy ids; exit code reflects expected-vs-actual assertions in the file.

### [To Do] X-SECX-0094 Bounded authorization decision cache invalidated on metadata change
P1 · M14 · Task · deps: X-SECX-0090 · labels: security, perf
Done when: decisions are cached by (attribute-set hash, action, resource, context class) with a bound `alluvium.policy.decision.cache.max.entries`; any policy, tag, ACL or attribute change invalidates it by image version; a test shows no stale grant after a forbid is added.

### [To Do] X-SECX-0095 Authorization latency benchmarks with policies
P1 · M14 · Task · deps: X-SECX-0094, TEST-F11 · labels: perf, nfr
Done when: criterion benchmarks measure decision latency for 0, 100 and 10,000 policies, cached and uncached, against the ACL-only baseline; the budget from the ADR is enforced in CI.

### [To Do] X-SECX-0096 Policy engine metrics and events
P1 · M14 · Task · deps: X-SECX-0090, OBS-F04, OBS-F07 · labels: security
Done when: `alluvium_policy_decisions_total{effect}`, `alluvium_policy_evaluation_seconds` and `alluvium_policy_budget_exceeded_total` exist; events `policy_created`, `policy_updated` and `policy_deleted` carry policy id and version; cardinality is bounded (no policy-id labels).

### [To Do] X-SECX-0097 Policy language reference and ACL migration guide
P1 · M14 · Task · deps: X-SECX-0091, DOCS-F01 · labels: docs, security
Done when: docs/security/policies.md documents every construct, operator and context attribute with examples, the ACL combination rule, limits and the migration path from ACLs (opt-in via `alluvium.policy.enabled`).

## [To Do] X-SECX-F07 Audit log
P0 · M14 · Feature · deps: X-SECX-F01
Scope: Admin operations, authorization and authentication events to a tamper-evident audit stream.

### [To Do] X-SECX-0098 Audit event schema v1 and versioning rules
P0 · M14 · Task · deps: none · labels: security, docs
Done when: docs/security/audit-events.md defines the envelope (event id, sequence, time, node id, category, type, principal, client address, listener, API key and version, correlation id, resource, operation, outcome, reason, policy ids, previous-event hash) and the compatibility rules for adding fields; a Rust type mirrors it with a serialization round-trip test.

### [To Do] X-SECX-0099 Audit emitter with a bounded queue and an explicit overflow policy
P0 · M14 · Task · deps: X-SECX-0098, SRV-F02 · labels: security
Done when: emitters enqueue into a queue bounded by `alluvium.audit.queue.max.events`; `alluvium.audit.overflow.policy` is `block` (requests wait, with a timeout returning REQUEST_TIMED_OUT) or `drop_and_count` for non-mandatory categories; admin and deny events are never dropped; queue depth, capacity and drops are exported.

### [To Do] X-SECX-0100 Audit topic `__alluvium_audit`: creation, replication settings and protection
P0 · M14 · Task · deps: X-SECX-0099, CTRL-F04 · labels: security
Done when: the topic is auto-created with RF and min ISR from `alluvium.audit.topic.replication.factor` and `.min.insync.replicas` and written with acks=all; only principals in `alluvium.audit.readers` (and super users) can read; DeleteRecords, deletion, config changes below a minimum retention and produce by clients are denied with events; tests cover each denial.

### [To Do] X-SECX-0101 Tamper evidence: per-partition hash chain and sequence numbers
P0 · M14 · Task · deps: X-SECX-0100, X-SECX-0003 · labels: security
Done when: each event carries a per-partition sequence and the SHA-256 of the previous event; the chain head survives leader change and broker restart (recovered from the log tail); a property test shows any modification, deletion, insertion or reordering is detected by recomputation.

### [To Do] X-SECX-0102 Signed audit checkpoints every N events or T milliseconds
P0 · M14 · Task · deps: X-SECX-0101, X-SECX-0013 · labels: security
Done when: every `alluvium.audit.checkpoint.events` events or `.checkpoint.ms` a checkpoint event signs (sequence, chain head) with HMAC-SHA-256 or Ed25519 using a key protected by the KMS seam; signing key rotation is supported; tests cover verification across a key rotation.

### [To Do] X-SECX-0103 `alluvium audit verify`: chain and signature verification with gap reporting
P0 · M14 · Task · deps: X-SECX-0102, CLI-F01 · labels: security
Done when: the command reads the audit topic (or files from the file sink), verifies chains and checkpoint signatures, and reports missing sequences, reordering, truncation after the last checkpoint and bad signatures with exit codes per class; tests use deliberately tampered fixtures for each class.

### [To Do] X-SECX-0104 JSON-lines file sink with rotation and fsync policy
P1 · M14 · Task · deps: X-SECX-0101 · labels: security
Done when: `alluvium.audit.file.path` enables a sink written off the Tokio workers with 0600 permissions, size/time rotation and `alluvium.audit.file.fsync` (`every_event`, `interval`); rotated files keep the chain; tests cover rotation and a full disk (events count as failed, broker keeps serving per overflow policy).

### [To Do] X-SECX-0105 Authentication audit events: SASL success and failure per mechanism, TLS handshake failure, re-authentication
P0 · M14 · Task · deps: X-SECX-0099, NET-F08, SEC-F12 · labels: security
Done when: events `authentication_succeeded`, `authentication_failed` (with mechanism and failure reason, never the credential), `tls_handshake_failed` and `reauthentication_succeeded`/`_failed` are emitted; failed-auth events are rate-limited per client address with a suppressed count; tests per mechanism.

### [To Do] X-SECX-0106 Authorization audit events: every deny and configured allows
P0 · M14 · Task · deps: X-SECX-0099, SEC-F08 · labels: security
Done when: every deny emits `authorization_denied` with operation, resource and matching policy ids; allows are emitted for operations listed in `alluvium.audit.authorization.allowed.operations` (default: none) with optional sampling; implicit DESCRIBE checks are marked; tests cover deny, sampled allow and implicit checks.

### [To Do] X-SECX-0107 Topic and partition administration audit events
P0 · M14 · Task · deps: X-SECX-0099, CTRL-F04, CTRL-F05, CTRL-F06 · labels: security
Done when: CreateTopics, DeleteTopics, CreatePartitions and DeleteRecords emit events with resource, request summary (no record data) and outcome, from the node that handled the request and the controller that committed it; tests per API.

### [To Do] X-SECX-0108 Configuration change audit events with redacted sensitive values
P0 · M14 · Task · deps: X-SECX-0099, CTRL-F11 · labels: security
Done when: AlterConfigs and IncrementalAlterConfigs (broker, topic, group, client-metrics) emit old and new values with password-type and `alluvium.*` sensitive keys redacted; a redaction test sets a password config and asserts the value never appears in the audit stream.

### [To Do] X-SECX-0109 ACL, policy and tag change audit events
P0 · M14 · Task · deps: X-SECX-0099, SEC-F09, X-SECX-F06, X-SECX-F05 · labels: security
Done when: CreateAcls, DeleteAcls (with matched bindings), policy create/update/delete (id, version, SHA-256) and tag changes emit events; tests per operation.

### [To Do] X-SECX-0110 Credential audit events: SCRAM, delegation tokens, API keys and certificate rotation
P0 · M14 · Task · deps: X-SECX-0099, SEC-F10, X-SECX-F08 · labels: security
Done when: AlterUserScramCredentials, delegation token create/renew/expire/describe, API key create/revoke/rotate/expire and certificate reload events are emitted without secrets; tests per operation assert no secret material.

### [To Do] X-SECX-0111 Consumer group and offset administration audit events
P0 · M14 · Task · deps: X-SECX-0099, GRP-F08, GRP-F07 · labels: security
Done when: DeleteGroups, OffsetDelete, administrative offset resets (OffsetCommit from an empty group by an admin tool) and member removal emit events; tests use `kafka-consumer-groups.sh` operations in the compat harness.

### [To Do] X-SECX-0112 Cluster operation audit events: features, broker registration, reassignment, leader election, quotas
P0 · M14 · Task · deps: X-SECX-0099, CTRL-F17, CTRL-F10, CTRL-F08, QUOTA-F08 · labels: security
Done when: UpdateFeatures, broker register/unregister/fence, AlterPartitionReassignments, ElectLeaders and AlterClientQuotas emit events with outcome; tests per operation.

### [To Do] X-SECX-0113 Key management and erasure audit events
P0 · M14 · Task · deps: X-SECX-0099, X-SECX-F02, X-SECX-F10 · labels: security
Done when: DEK create/rotate/rewrap/destroy, KEK rotation detection, re-encryption start/finish and erasure request lifecycle events are emitted without key material; tests per event.

### [To Do] X-SECX-0114 Audit category selection: include/exclude categories and mandatory events
P0 · M14 · Task · deps: X-SECX-0105, X-SECX-0106 · labels: security
Done when: `alluvium.audit.events.include` and `.exclude` select categories; admin, credential, key and deny events cannot be excluded (config validation error); a dynamic config update takes effect without restart; tests cover each rule.

### [To Do] X-SECX-0115 Strict mode: admin operations acknowledged only after their audit event is durable
P1 · M14 · Task · deps: X-SECX-0100, X-SECX-0107 · labels: security
Done when: with `alluvium.audit.mode=strict` admin responses wait for the audit event's acks=all append (bounded by `alluvium.audit.strict.timeout.ms`, failing with REQUEST_TIMED_OUT); a simulation with crashes asserts no admin operation is acknowledged without a durable audit event.

### [To Do] X-SECX-0116 Audit pipeline metrics and events
P0 · M14 · Task · deps: X-SECX-0099, X-SECX-0100, OBS-F04, OBS-F07 · labels: security
Done when: `alluvium_audit_events_total{category}`, `_dropped_total{category}`, `_queue_depth`, `_append_seconds` and `_sink_errors_total{sink}` exist; event `audit_sink_unavailable` with reason fires once per outage; tests assert metric changes.

### [To Do] X-SECX-0117 Tamper-detection negative test suite
P0 · M14 · Task · deps: X-SECX-0103 · labels: security, test
Done when: tests modify, delete, insert, reorder and replay events, truncate after and before checkpoints, and forge a checkpoint with a wrong key; `audit verify` detects each with the right class.

### [To Do] X-SECX-0118 Audit overhead benchmarks
P0 · M14 · Task · deps: X-SECX-0106, TEST-F11 · labels: perf, nfr
Done when: macro benchmarks measure produce/fetch throughput and p99 with auditing off, admin-only, and all authorization allows audited; results and the budget from the ADR are in docs/benchmarks/audit.md.

### [To Do] X-SECX-0119 Audit guide and SIEM integration
P0 · M14 · Task · deps: X-SECX-0103, X-SECX-0114 · labels: docs, security
Done when: docs/security/audit.md covers enabling (`alluvium.audit.enabled`), categories, sinks, verification procedure, retention guidance and shipping to a SIEM, with a field mapping table to OCSF classes.

## [To Do] X-SECX-F08 Credential lifecycle
P1 · M14 · Feature · deps: X-SECX-F01
Scope: Certificate rotation automation and API key rotation. Key creation, scopes and expiry are in X-NATIVE-F09.

### [To Do] X-SECX-0120 Certificate expiry monitoring per listener and trust anchor
P1 · M14 · Task · deps: NET-F07, OBS-F04 · labels: security
Done when: gauge `alluvium_tls_certificate_expiry_seconds{listener,kind}` covers server, client and CA certificates; event `tls_certificate_expiring` fires when below `alluvium.tls.expiry.warning.ms`; tests use certificates with near and past expiry.

### [To Do] X-SECX-0121 Validated certificate reload with rollback
P1 · M14 · Task · deps: X-SECX-0120 · labels: security
Done when: before NET-F07 hot reload swaps material, the new key is checked to match the certificate, the chain validates against the trust store, and the certificate is not expired or not yet valid; failures keep the old material and emit `tls_certificate_reload_rejected` with reason; tests per failure.

### [To Do] X-SECX-0122 Trust bundle rotation with overlapping CAs
P1 · M14 · Task · deps: X-SECX-0121 · labels: security
Done when: trust stores accept multiple CA bundles during rotation; a compat test rotates the cluster CA (add new CA, reissue leaf certs, remove old CA) across a rolling restart with live clients and no failed connections.

### [To Do] X-SECX-0123 Dependency review: ACME client crate
P2 · M14 · Task · deps: X-SECX-0121, FND-F03 · labels: security
Done when: the review covers the candidate ACME crate (transitive count, `unsafe`, rustls-only, license, maintenance) or justifies a minimal RFC 8555 client; `cargo deny check` passes.

### [To Do] X-SECX-0124 Automated certificate issuance and renewal: Vault PKI and ACME issuers
P2 · M14 · Task · deps: X-SECX-0123, X-SECX-0051 · labels: security
Done when: `alluvium.tls.issuer` (`none`, `vault-pki`, `acme`) obtains certificates for listeners, renews at two thirds of lifetime with jittered retries, writes key material with 0600 permissions and triggers validated reload; tests use a Vault dev server and an ACME test server (Pebble) in the compat harness.

### [To Do] X-SECX-0125 API key model and secret format
P1 · M14 · Task · deps: X-SECX-0004 · labels: security
Done when: `ApiKey` has id, owner principal, scopes (operations and resource patterns), created_at, expires_at, created_by and a SHA-256 hash of a 256-bit secret; secrets use the scannable form `alv_<id>_<secret>_<checksum>`; verification is constant-time; unit tests cover format, checksum typo detection and hashing.

### [To Do] X-SECX-0126 API key metadata records with controller validation
P1 · M14 · Task · deps: X-SECX-0125, X-SECX-0005, META-F02 · labels: security
Done when: `ApiKeyRecord` and `RemoveApiKeyRecord` are validated (max keys per principal `alluvium.apikey.max.per.principal`, max lifetime `alluvium.apikey.max.lifetime.ms`, expiry in the future); replay and snapshot round-trip tests pass; secrets never appear in records.

### [To Do] X-SECX-0127 API key authentication over SASL/PLAIN and native bearer tokens
P1 · M14 · Task · deps: X-SECX-0126, SEC-F03, X-NATIVE-F09 · labels: security
Done when: on listeners with `alluvium.apikey.sasl.enabled=true`, SASL/PLAIN username = key id and password = secret authenticates as principal `ApiKey:<id>` linked to its owner; native APIs accept `Authorization: Bearer alv_…`; a compat test authenticates the Java client with an API key; wrong, expired and revoked keys fail with SASL_AUTHENTICATION_FAILED.

### [To Do] X-SECX-0128 Scope enforcement: API key rights are the intersection of key scopes and owner rights
P1 · M14 · Task · deps: X-SECX-0127, SEC-F08 · labels: security
Done when: authorization for an API key principal requires both a scope match and an owner grant (ACL or policy); property tests over random scopes and ACLs show a key never exceeds its owner's rights; owner deletion disables the key.

### [To Do] X-SECX-0129 Expiry enforcement on new and long-lived connections
P1 · M14 · Task · deps: X-SECX-0127, SEC-F12 · labels: security
Done when: expired keys fail authentication; the KIP-368 session lifetime is capped at the key's expiry so existing connections must re-authenticate; expired keys are removed by a controller task after `alluvium.apikey.expired.retention.ms`; tests use the simulated clock.

### [To Do] X-SECX-0130 Revocation propagation and session termination within a bound
P1 · M14 · Task · deps: X-SECX-0127 · labels: security
Done when: revoking a key closes every connection authenticated with it within `alluvium.apikey.revocation.max.delay.ms` after the broker applies the metadata change; a simulation test measures the bound; event `api_key_sessions_terminated` carries the count.

### [To Do] X-SECX-0131 API key administration: native admin and `alluvium apikey` CLI
P1 · M14 · Task · deps: X-SECX-0126, X-NATIVE-F08, CLI-F01 · labels: security
Done when: create returns the secret exactly once; list and describe never return secrets; revoke and rotate (new key with an overlap window, old key expiring at `rotate_overlap`) work; a principal may manage only its own keys unless it has ALTER on the cluster; tests per operation.

### [To Do] X-SECX-0132 Authentication failure throttling per key id and client address
P1 · M14 · Task · deps: X-SECX-0127 · labels: security
Done when: repeated failures for a key id or client address are delayed with exponential backoff up to `alluvium.apikey.auth.failure.max.delay.ms` using a bounded table (`.auth.failure.tracking.max.entries`); metric `alluvium_apikey_auth_failures_total` exists; tests cover throttling and table eviction.

### [To Do] X-SECX-0133 Negative tests and fuzzing for API key parsing and authentication
P1 · M14 · Task · deps: X-SECX-0127, TEST-F07 · labels: security, test, decoder
Done when: `fuzz_api_key_parse` runs clean; tests cover truncated keys, bad checksum, timing (constant-time comparison verified by a statistical test or code review checklist), reuse of revoked keys and keys of deleted owners.

### [To Do] X-SECX-0134 Credential lifecycle metrics and events
P1 · M14 · Task · deps: X-SECX-0131, X-SECX-0120, OBS-F07 · labels: security
Done when: `alluvium_apikeys_active`, `alluvium_apikeys_expiring_soon` and certificate reload counters exist; events `api_key_created`, `api_key_revoked`, `api_key_expired` and `tls_certificate_reloaded` are in the catalog with reasons and no secrets.

### [To Do] X-SECX-0135 Credential lifecycle guide
P1 · M14 · Task · deps: X-SECX-0131, X-SECX-0122 · labels: docs, security
Done when: docs/security/credentials.md covers certificate expiry monitoring, CA rotation runbook, automated issuance, API keys (enable, scopes, expiry, rotation, revocation) and secret scanning patterns for `alv_` keys.

## [To Do] X-SECX-F09 Data masking
P2 · M14 · Feature · deps: X-SECX-F01
Scope: Masking for inspection tools and SQL results.

### [To Do] X-SECX-0136 Masking policy model and `MaskingPolicyRecord`
P2 · M14 · Task · deps: X-SECX-0005, META-F02 · labels: security
Done when: a masking policy has id, topic pattern (literal or prefix), schema field paths and a mask function per path; records are validated by the controller, replayed and snapshotted; tests cover validation errors.

### [To Do] X-SECX-0137 Mask functions: redact, nullify, partial reveal and keyed hash
P2 · M14 · Task · deps: X-SECX-0136, X-SECX-0038 · labels: security
Done when: `redact` (fixed token), `nullify`, `partial(keep_last=n)` and `keyed_hash` (HMAC-SHA-256 with a key protected by the KMS seam) are type-aware (string, bytes, numeric) and documented; a property test asserts masked output never contains the original value for values longer than the revealed part.

### [To Do] X-SECX-0138 Schema-aware field resolution for Avro, Protobuf and JSON payloads
P2 · M14 · Task · deps: X-SECX-0136, X-SCHEMA-F04, X-SCHEMA-F05, X-SCHEMA-F06 · labels: security
Done when: field paths resolve against the registered schema of each record's schema id; masked payloads are re-encoded with the same schema; tests per format with nested records, arrays and maps.

### [To Do] X-SECX-0139 Fail-closed behavior for undecodable or unknown payloads
P2 · M14 · Task · deps: X-SECX-0138 · labels: security
Done when: a payload with an unknown schema id, a decode error or no schema on a topic covered by a masking policy is fully redacted with a marker, never shown in plaintext; tests cover each case.

### [To Do] X-SECX-0140 `ReadUnmasked` action and default masking for everyone else
P2 · M14 · Task · deps: X-SECX-0136, X-SECX-F06 · labels: security
Done when: only principals granted `ReadUnmasked` on the topic (by policy, or super users) see unmasked output in masking-aware tools; everyone else gets masked output; tests cover policy grant, forbid and super user.

### [To Do] X-SECX-0141 Masking engine API for inspection tools and SQL result sets
P2 · M14 · Task · deps: X-SECX-0137, X-SECX-0139, X-SECX-0140 · labels: security
Done when: a documented API takes (principal context, topic, record or Arrow column batch) and returns masked output; it is the single masking implementation used by X-SQL and X-OBSX message inspection; unit tests cover record and columnar inputs.

### [To Do] X-SECX-0142 Masking in `alluvium dump log --print-payloads`
P2 · M14 · Task · deps: X-SECX-0141, CLI-F04 · labels: security
Done when: when a masking policy file or the cluster's policies are supplied, printed payloads are masked; `--unmasked` requires an explicit flag and is logged to the audit file sink when present; tests cover both.

### [To Do] X-SECX-0143 Fuzz the masking path with arbitrary payloads
P2 · M14 · Task · deps: X-SECX-0138, TEST-F07 · labels: security, test, decoder
Done when: `fuzz_masking_payload` feeds arbitrary bytes with arbitrary schema ids through masking; it runs clean and never returns unmasked bytes for covered fields (asserted with canaries).

### [To Do] X-SECX-0144 Masking metrics and events
P2 · M14 · Task · deps: X-SECX-0141, OBS-F04 · labels: security
Done when: `alluvium_masking_fields_masked_total{function}` and `alluvium_masking_fail_closed_total{reason}` exist; masking policy changes emit `masking_policy_changed`; tests assert metric changes.

### [To Do] X-SECX-0145 Masking benchmarks
P3 · M14 · Task · deps: X-SECX-0141, TEST-F11 · labels: perf
Done when: criterion benchmarks measure records per second for each mask function and format with 1 and 10 masked fields; results are in docs/benchmarks/masking.md.

### [To Do] X-SECX-0146 Data masking guide
P2 · M14 · Task · deps: X-SECX-0141, DOCS-F01 · labels: docs, security
Done when: docs/security/masking.md covers policies, functions, `ReadUnmasked`, fail-closed behavior and where masking applies (dump tools, SQL results, message inspection) and where it does not (Kafka Fetch).

## [To Do] X-SECX-F10 Right to erasure
P1 · M14 · Feature · deps: X-SECX-F02
Scope: Delete by key across topics; crypto-shredding per subject.

### [To Do] X-SECX-0147 Erasure request model and state machine
P1 · M14 · Task · deps: X-SECX-0005 · labels: security
Done when: `ErasureRequest` has id, subject selector (record key or subject id), topic selector, requester and deadline; the sans-I/O machine moves Pending → Locating → Erasing → Verifying → Completed or Failed with reasons; every transition returns an event; a property test checks terminal states are final.

### [To Do] X-SECX-0148 Erasure metadata records and controller orchestration
P1 · M14 · Task · deps: X-SECX-0147, META-F02 · labels: security
Done when: `ErasureRequestRecord` and progress records per partition are validated and replayed; the controller assigns work to partition leaders and survives failover (resumes from records); simulation test kills the controller mid-request.

### [To Do] X-SECX-0149 Locating affected segments: key-index lookup with bounded full-scan fallback
P1 · M14 · Task · deps: X-SECX-0148, X-KV-F04 · labels: security
Done when: partitions with a key index use X-KV lookups to list segments containing the key; others are scanned at `alluvium.erasure.scan.max.bytes.per.second`; the result is a per-partition list of segment base offsets; tests compare both methods on the same data.

### [To Do] X-SECX-0150 Erasure on compacted topics: tombstones plus forced cleaning of affected segments
P1 · M14 · Task · deps: X-SECX-0149, STOR-F14 · labels: security
Done when: a tombstone is appended for each key and the affected segments are cleaned regardless of `min.compaction.lag.ms`; after `delete.retention.ms` the tombstone is removed per Kafka rules; a test verifies no record with the key remains in any segment.

### [To Do] X-SECX-0151 Erasure on non-compacted topics: offset-preserving segment rewrite
P1 · M14 · Task · deps: X-SECX-0149, STOR-F14 · labels: security
Done when: affected segments are rewritten without matching records, keeping offsets of the rest (gaps as in compaction), swapped atomically like the cleaner, and indexes rebuilt; control batches are kept; crash-consistency tests cover every swap step.

### [To Do] X-SECX-0152 Erasure on followers and tiered segments
P1 · M14 · Task · deps: X-SECX-0151, TIER-F03 · labels: security
Done when: every replica applies the same erasure from the metadata record deterministically; remote tiered segments containing the key are rewritten and re-uploaded with remote metadata updated; a test compares all replicas and the remote copy after erasure.

### [To Do] X-SECX-0153 Per-subject keys for crypto-shredding
P1 · M14 · Task · deps: X-SECX-0147, X-SECX-0015, X-SECX-0013 · labels: security
Done when: topics with `alluvium.erasure.subject.header` set encrypt each record value with a per-subject DEK selected by that header; subject DEKs are wrapped by the topic KEK and stored as records; the subject DEK cache is bounded; unit tests cover lookup and creation.

### [To Do] X-SECX-0154 Crypto-shredding: destroy a subject key and define read behavior
P1 · M14 · Task · deps: X-SECX-0153 · labels: security
Done when: shredding destroys the subject DEK in metadata (and the KMS reference when per-subject KEKs are used); fetches of shredded records return them with a null value and header `alluvium.shredded=true` in a rebuilt batch with a valid CRC, as specified in the ADR; a compat test shows the Java consumer reads shredded records without error.

### [To Do] X-SECX-0155 Post-erasure verification scan and signed erasure report
P1 · M14 · Task · deps: X-SECX-0152, X-SECX-0154 · labels: security
Done when: verification scans every replica (and remote segments) for the key or subject and fails the request if any match remains; a completion report (request id, partitions, segment counts, timestamps) is signed like audit checkpoints and retrievable by the requester.

### [To Do] X-SECX-0156 Erasure administration: native admin, CLI and authorization
P1 · M14 · Task · deps: X-SECX-0148, X-NATIVE-F08, CLI-F01 · labels: security
Done when: `alluvium erasure submit|status|list` and native endpoints exist; submitting requires the `Erase` action on each topic (policy) or cluster ALTER; status reports per-partition progress; tests cover authorization denial.

### [To Do] X-SECX-0157 Erasure of records inside transactions
P1 · M14 · Task · deps: X-SECX-0151, TXN-F11 · labels: security
Done when: erasing records from committed or aborted transactions keeps control batches and the aborted-transaction index valid; LSO and read_committed fetches are unaffected; tests cover both transaction outcomes.

### [To Do] X-SECX-0158 Compatibility: Kafka consumers across erased offset gaps
P1 · M14 · Task · deps: X-SECX-0151, X-SECX-0154, TEST-F08 · labels: test
Done when: the Java client and librdkafka consume across erased gaps and shredded records with correct positions and committed offsets; tests run in the compat harness.

### [To Do] X-SECX-0159 Erasure I/O throttling and progress metrics
P1 · M14 · Task · deps: X-SECX-0151 · labels: security, perf
Done when: rewrites are throttled by `alluvium.erasure.rewrite.max.bytes.per.second`; metrics `alluvium_erasure_requests{state}`, `_bytes_rewritten_total` and `_request_duration_seconds` exist; events per state transition carry reasons.

### [To Do] X-SECX-0160 Simulation: erasure under leader changes, crashes and concurrent produce
P1 · M14 · Task · deps: X-SECX-0152, TEST-F06 · labels: security, test
Done when: a seeded simulation submits erasures while producing to the same partitions with leader changes and crashes; invariants: no erased record remains after Completed, no unrelated acknowledged record is lost, offsets of kept records never change.

### [To Do] X-SECX-0161 Right-to-erasure guide
P1 · M14 · Task · deps: X-SECX-0155, X-SECX-0156 · labels: docs, security
Done when: docs/security/erasure.md covers delete-by-key, crypto-shredding setup, deadlines, verification reports, effects on consumers, and the caveat that mirrors and backups need separate erasure.

## [To Do] X-SECX-F11 Advanced security testing and compliance documentation
P0 · M14 · Feature · deps: X-SECX-F02, X-SECX-F07
Scope: Penetration test plan, fuzzing, compliance mapping.

### [To Do] X-SECX-0162 Advanced security is off by default
P0 · M14 · Task · deps: X-SECX-F03, X-SECX-F05, X-SECX-F06, X-SECX-F08, X-SECX-F09, X-SECX-F10 · labels: test, security
Done when: test `advanced_security_off_by_default` starts a node with default config and asserts: `alluvium.security.version` is 0, segments are byte-identical to the plaintext format, no KMS client is created, no audit topic exists, authorization equals SEC-F07 decisions, API-key SASL is disabled and no masking applies.

### [To Do] X-SECX-0163 Kafka compatibility and differential suites pass with advanced security enabled but unused
P0 · M14 · Task · deps: X-SECX-0162, TEST-F08, TEST-F09 · labels: test, security
Done when: the full compat and differential suites run in CI with feature level 1, a local KEK provider, the policy engine on with zero policies and API-key SASL enabled but no keys; all pass with no new ignore rules except a documented one for Alluvium internal topics.

### [To Do] X-SECX-0164 Nightly fuzz campaign for every X-SECX decoder and parser
P0 · M14 · Task · deps: X-SECX-F03, X-SECX-F04, X-SECX-F05, X-SECX-F06, X-SECX-F08, X-SECX-F09, TEST-F07 · labels: security, test
Done when: all X-SECX fuzz targets (encryption header, encrypted segment load, KMS responses, field envelope, JWT claims, certificate attributes, policy parser, API keys, masking) run nightly with committed corpora and a crash-triage owner; coverage per target is reported.

### [To Do] X-SECX-0165 Security negative-test suite: bypass, downgrade and replay attempts
P0 · M14 · Task · deps: X-SECX-F06, X-SECX-F08 · labels: security, test
Done when: tests attempt policy/ACL bypass through tag manipulation, encrypted-format version downgrade (rejected), DEK substitution between topics (AAD failure), audit event replay, use of revoked or expired API keys and reading audit topics without permission; each is rejected and audited.

### [To Do] X-SECX-0166 Penetration test plan
P0 · M14 · Task · deps: X-SECX-0002 · labels: security, docs
Done when: docs/security/pentest-plan.md lists scope, threat-model items to exercise, environments, rules of engagement and reporting; reviewed per SEC-F14.

### [To Do] X-SECX-0167 Penetration test execution and finding triage
P1 · M14 · Task · deps: X-SECX-0166, X-SECX-0163 · labels: security
Done when: an external or independent internal team executes the plan; each finding has a backlog `Bug` item with severity; critical and high findings are fixed or have an ADR accepting the risk before the M14 exit.

### [To Do] X-SECX-0168 Compliance mapping documentation
P0 · M14 · Task · deps: X-SECX-F10 · labels: docs, security
Done when: docs/security/compliance.md maps SOC 2, ISO 27001 Annex A, GDPR (Art. 17 and 32), HIPAA and PCI DSS controls to Alluvium features, configuration and evidence (audit reports, erasure reports, key rotation logs), with explicit gaps.

### [To Do] X-SECX-0169 Migration and opt-in guide for advanced security
P0 · M14 · Task · deps: X-SECX-F06, X-SECX-F08 · labels: docs, security
Done when: docs/security/advanced-security-migration.md covers raising `alluvium.security.version`, enabling encryption on existing topics, moving from ACLs to policies, enabling audit and API keys, and the downgrade path (what must be undone first); every step has a verification command.

### [To Do] X-SECX-0170 Upgrade and downgrade test with advanced security
P1 · M14 · Task · deps: X-SECX-0169, TEST-F14 · labels: test, security
Done when: a rolling upgrade from the previous Alluvium release to this one, then enabling features, passes with live clients; a downgrade attempt with encrypted topics is refused with a clear error; the documented downgrade path succeeds.

### [To Do] X-SECX-0171 All-features secret and payload redaction test
P0 · M14 · Task · deps: X-SECX-F08, OBS-F11 · labels: security, test
Done when: with every X-SECX feature enabled and exercised, OBS-F11 scanners find no DEKs, KEK credentials, API-key secrets, JWTs or record payload canaries in logs, metrics, `/debug/*` output, audit events or crash reports.

### [To Do] X-SECX-0172 Combined advanced-security benchmark report
P1 · M14 · Task · deps: X-SECX-0037, X-SECX-0095, X-SECX-0118 · labels: perf, docs
Done when: docs/benchmarks/advanced-security.md reports throughput, p99 latency and memory with encryption, policies and audit combined versus all off, on the standard workloads from NFR-F01, with the hardware and versions used.

### [To Do] X-SECX-0173 Security guide updates for advanced security
P0 · M14 · Task · deps: X-SECX-0169, DOCS-F09 · labels: docs, security
Done when: the DOCS-F09 security guide links every X-SECX guide, adds advanced-security items to the hardening checklist and states which features are opt-in.
