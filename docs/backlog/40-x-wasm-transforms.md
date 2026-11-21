# X-WASM: Server-side filtering and WASM transforms

> Crates: `alluvium-transform` (new, layer 4) · Kafka counterpart: none (Alluvium enhancement) · Milestones: M16

Sandboxed WebAssembly transforms at produce and consume time, derived topics for Kafka clients, and server-side filtering.

## [To Do] X-WASM-F01 Transform runtime design
P0 · M16 · Feature · deps: none
Scope: Spike and ADR: WASM runtime choice and the dependency and unsafe exception.

### [To Do] X-WASM-0001 Spike: transform runtime ADR (engine, ABI style, execution placement, determinism)
P0 · M16 · Spike · deps: none · ref: new ADR "WebAssembly transform runtime"
Done when: ADR accepted that compares wasmtime (Cranelift JIT, pooling allocator), wasmer and an interpreter (wasmi) on safety record, sandbox controls (fuel, epochs, memory limits), performance and dependency weight; decides core-module ABI vs component model, no WASI (no ambient authority), where transforms run (leader append path, derived-topic tasks, native fetch path), the worker-thread execution model off the Tokio workers, and how determinism is preserved; rejected alternatives recorded.

### [To Do] X-WASM-0002 ADR: dependency and `unsafe` exception for the WebAssembly engine
P0 · M16 · Task · deps: X-WASM-0001 · labels: security · ref: docs/standards/security.md#unsafe-code
Done when: ADR accepted naming `alluvium-transform` as the only crate allowed to depend on the engine, stating that our code keeps `forbid(unsafe_code)` and listing engine APIs we must never call (for example `Module::deserialize` of untrusted bytes, which is `unsafe`); it records the engine's JIT, executable memory, guard pages and Spectre mitigations and the conditions that would require revisiting the decision.

### [To Do] X-WASM-0003 Dependency review: the WebAssembly engine with minimal features
P0 · M16 · Task · deps: X-WASM-0002, FND-F03 · labels: security
Done when: the engine is added to `[workspace.dependencies]` with only the needed features (compiler backend, pooling allocator; no WASI, no cache, no wat in production builds); the review records transitive crate count, `unsafe` and platform code, license (Apache-2.0 WITH LLVM-exception added to the `deny.toml` allow-list with a reason) and maintenance; `cargo deny check` passes; `xtask check-layers` restricts the engine to `alluvium-transform`.

### [To Do] X-WASM-0004 Transform threat model
P0 · M16 · Task · deps: X-WASM-0001, SEC-F14 · labels: security, docs
Done when: docs/security/threat-model-transforms.md covers malicious modules (sandbox escape, CPU and memory exhaustion, compiler bugs, side channels between deployments), malicious records fed to modules, module supply chain, privilege escalation through routing and derived topics, and data exfiltration through logs; each threat maps to mitigations and X-WASM test task IDs.

### [To Do] X-WASM-0005 `alluvium.transform.version` finalized feature gating transform metadata records
P0 · M16 · Task · deps: X-WASM-0001, META-F06, CTRL-F17
Done when: at level 0 (default) the controller rejects every X-WASM record type with UNSUPPORTED_VERSION; level 1 enables them; downgrade is refused while modules or deployments exist; unit tests per record type.

### [To Do] X-WASM-0006 Register the `alluvium.transform.*` and `alluvium.filter.*` config namespaces
P0 · M16 · Task · deps: X-WASM-0001, CFG-F11
Done when: both prefixes are in the CFG-F11 registry linked to the transform ADR; the registry check rejects unregistered keys under them; docs/config.md lists them.

## [To Do] X-WASM-F02 Sandbox
P0 · M16 · Feature · deps: X-WASM-F01
Scope: Fuel or epoch limits, memory limits, no ambient authority.

### [To Do] X-WASM-0007 Hardened engine configuration
P0 · M16 · Task · deps: X-WASM-0003 · labels: security
Done when: one function builds the engine config: threads, shared memory, multi-memory and other proposals the ADR rejects are disabled; WASI is not linked; maximum wasm stack is `alluvium.transform.max.stack.bytes`; guard pages and the pooling allocator with fixed slots are enabled; a test asserts each setting and that modules using a disabled proposal fail validation.

### [To Do] X-WASM-0008 Module validation at deploy time
P0 · M16 · Task · deps: X-WASM-0007 · labels: security
Done when: modules larger than `alluvium.transform.max.module.bytes`, failing wasm validation, using disabled proposals, missing required exports, exporting a wrong signature, importing anything outside the declared ABI or declaring more than `alluvium.transform.max.memory.pages` initial pages are rejected with a typed error naming the reason; tests use one WAT fixture per rejection.

### [To Do] X-WASM-0009 No ambient authority: import allow-list linker
P0 · M16 · Task · deps: X-WASM-0008 · labels: security
Done when: the linker defines only the ABI host functions of the module's declared ABI version; any other import fails instantiation; no clock, randomness, filesystem, environment or network function exists; a test instantiates modules importing WASI functions and fails each.

### [To Do] X-WASM-0010 CPU limits: fuel per record and epoch deadline per batch
P0 · M16 · Task · deps: X-WASM-0007 · labels: security
Done when: each invocation gets `alluvium.transform.max.fuel.per.record` fuel and each batch an epoch deadline of `alluvium.transform.max.time.per.batch.ms` advanced by a supervised ticker task; exhaustion traps with `TransformError::BudgetExceeded { kind: Fuel | Deadline }`; tests with an infinite-loop module hit each limit.

### [To Do] X-WASM-0011 Memory and table limits through a resource limiter
P0 · M16 · Task · deps: X-WASM-0007 · labels: security
Done when: linear memory per instance is capped at `alluvium.transform.max.memory.bytes`, tables at `alluvium.transform.max.table.elements` and instances per deployment at `alluvium.transform.max.instances`; `memory.grow` beyond the cap returns -1 (and traps if the module requires it); tests with growth bombs stay within limits.

### [To Do] X-WASM-0012 Compilation on a bounded pool with timeout and an in-memory artifact cache
P0 · M16 · Task · deps: X-WASM-0008 · labels: security
Done when: modules compile on a dedicated bounded pool with timeout `alluvium.transform.compile.timeout.ms`; compiled modules are cached in memory keyed by (module SHA-256, engine version, config hash) with a size bound; compiled artifacts are never deserialized from disk; compile time and cache metrics exist.

### [To Do] X-WASM-0013 Transform worker pool off the Tokio workers with bounded queues
P0 · M16 · Task · deps: X-WASM-0012, SRV-F02
Done when: invocations run on `alluvium.transform.worker.threads` supervised OS threads fed by a bounded queue (`alluvium.transform.queue.max.batches`); submitters receive an explicit `QueueFull` error for backpressure; queue depth, capacity and rejections are exported; a test shows Tokio workers are never blocked by a long-running module.

### [To Do] X-WASM-0014 Trap handling and instance recycling
P0 · M16 · Task · deps: X-WASM-0013, X-WASM-0010, X-WASM-0011 · labels: security
Done when: every trap (unreachable, out-of-bounds, stack overflow, budget exceeded, host error) becomes a typed `TransformError` with deployment, module version and trap code; the instance is discarded and a fresh one is created for the next invocation; no trap aborts the process; tests per trap kind.

### [To Do] X-WASM-0015 Instance isolation between deployments and invocations
P0 · M16 · Task · deps: X-WASM-0014 · labels: security
Done when: instances are never shared between deployments; per-deployment instance reuse resets nothing implicitly, so the ADR's reuse rule (fresh instance per batch, or reuse with documented state carry-over) is enforced; a test shows data written to memory by one deployment is never visible to another.

### [To Do] X-WASM-0016 Sandbox negative-test suite with hostile modules
P0 · M16 · Task · deps: X-WASM-0014, X-WASM-0009 · labels: security, test
Done when: hostile WAT modules covering infinite loops, deep recursion, memory and table growth bombs, huge data segments, forbidden imports, oversized outputs, out-of-range host-call pointers and lengths, and reentrant host calls are each contained with the expected typed error and bounded resource use; the suite runs in CI.

### [To Do] X-WASM-0017 Fuzzing: generated modules and host-call arguments
P0 · M16 · Task · deps: X-WASM-0014, TEST-F07 · labels: security, test, decoder
Done when: `fuzz_transform_sandbox` runs wasm-smith-generated modules through validation, compilation, instantiation and invocation with tight limits and `fuzz_transform_host_calls` calls host functions with arbitrary pointers and lengths; both run clean nightly with no host panic or limit escape.

### [To Do] X-WASM-0018 Sandbox metrics
P0 · M16 · Task · deps: X-WASM-0014, OBS-F04
Done when: `alluvium_transform_fuel_consumed_total{deployment}`, `_traps_total{deployment,kind}`, `_compile_seconds`, `_instance_memory_bytes{deployment}` and `_instances{deployment}` exist with a cap on deployment label values; tests assert updates.

### [To Do] X-WASM-0019 Invocation overhead benchmarks
P0 · M16 · Task · deps: X-WASM-0013, TEST-F11 · labels: perf
Done when: criterion benchmarks measure instantiation, per-record and per-batch invocation overhead for a no-op module and fuel-metering overhead; results and budgets are in docs/benchmarks/transforms.md with CI regression thresholds.

## [To Do] X-WASM-F03 Transform ABI
P0 · M16 · Feature · deps: X-WASM-F01
Scope: Versioned record-in/record-out interface with headers and errors.

### [To Do] X-WASM-0020 Transform ABI v1 specification
P0 · M16 · Task · deps: none · labels: docs
Done when: docs/transforms/abi-v1.md specifies guest exports (`alluvium_abi_version`, `alluvium_alloc`, `alluvium_dealloc`, `alluvium_init`, `alluvium_transform`), host imports in module `alluvium_v1` (`emit_record`, `emit_to_output`, `log`, `fail`), memory ownership, the record encoding, status codes, limits and the versioning and compatibility rules.

### [To Do] X-WASM-0021 ABI record encoding: host encoder for inputs and bounded decoder for guest outputs
P0 · M16 · Task · deps: X-WASM-0020 · labels: security, decoder
Done when: the encoding carries key, value, headers, timestamp, timestamp type, and (inputs only) topic, partition and offset; the decoder bounds key, value, header count and header sizes by `alluvium.transform.max.output.record.bytes` before allocating; property round-trip tests pass; fuzz target `fuzz_transform_abi_output` runs clean.

### [To Do] X-WASM-0022 Guest memory protocol: `alluvium_alloc`, `alluvium_dealloc` and pointer validation
P0 · M16 · Task · deps: X-WASM-0020, X-WASM-0009 · labels: security
Done when: the host writes inputs only into buffers returned by `alluvium_alloc`; every guest-supplied (pointer, length) is checked against current memory bounds with checked arithmetic before access; a bad pointer returns `AbiError::OutOfBounds` and traps the invocation; tests cover overflow and zero-length cases.

### [To Do] X-WASM-0023 ABI version negotiation
P0 · M16 · Task · deps: X-WASM-0020, X-WASM-0008
Done when: `alluvium_abi_version` is called at deploy validation; versions outside the host's supported set are rejected with an error listing supported versions; golden modules per supported version are kept in `testdata/transforms/abi/` and run in CI.

### [To Do] X-WASM-0024 `alluvium_init` with per-deployment configuration
P0 · M16 · Task · deps: X-WASM-0022
Done when: the deployment's config map (bounded by `alluvium.transform.max.config.bytes`) is passed once per instance; a non-zero status fails instance creation with the guest's error message (bounded length); tests cover success, failure and oversized config.

### [To Do] X-WASM-0025 `alluvium_transform` per-record contract: zero, one or many outputs
P0 · M16 · Task · deps: X-WASM-0021, X-WASM-0022
Done when: each input record is passed in the ABI encoding; outputs emitted during the call form the result (0 = filter, 1 = map, N = flat-map bounded by `alluvium.transform.max.outputs.per.input`); status codes map to `TransformOutcome::{Ok, Fail { code, message }}`; tests use WAT modules for each shape.

### [To Do] X-WASM-0026 `emit_record` host import with output validation
P0 · M16 · Task · deps: X-WASM-0025 · labels: security
Done when: `emit_record` decodes and validates each output (sizes, header count, total output bytes per invocation `alluvium.transform.max.output.bytes`) and returns a status to the guest instead of trapping on recoverable violations; tests cover each limit.

### [To Do] X-WASM-0027 `emit_to_output` host import: routing to outputs declared at deploy time
P0 · M16 · Task · deps: X-WASM-0026 · labels: security
Done when: guests route records only to output indexes declared in the deployment (never arbitrary topic names); an undeclared index returns an error status; deployers must hold WRITE on every declared output topic; tests cover valid, undeclared and unauthorized outputs.

### [To Do] X-WASM-0028 Header semantics and reserved `alluvium.` header prefix
P0 · M16 · Task · deps: X-WASM-0026 · labels: security
Done when: guests can read, add, replace and remove headers on outputs, but headers starting with `alluvium.` set by guests are rejected so provenance and DLQ headers cannot be spoofed; tests cover each operation and the reserved prefix.

### [To Do] X-WASM-0029 Output timestamps and key changes
P0 · M16 · Task · deps: X-WASM-0026, REC-F12
Done when: output CreateTime timestamps are validated against the destination topic's `message.timestamp.*` rules; LogAppendTime topics restamp outputs; a changed key on produce-time routing re-partitions with the default murmur2 partitioner; tests cover each rule.

### [To Do] X-WASM-0030 `log` host import: rate-limited, bounded and off by default
P1 · M16 · Task · deps: X-WASM-0022 · labels: security
Done when: guest log calls are dropped unless the deployment enables logging; when enabled they are truncated to `alluvium.transform.log.max.bytes`, rate-limited per deployment, tagged with deployment and module version, and written at debug level to a separate target; the docs warn that guests could log payloads.

### [To Do] X-WASM-0031 Guest failure reporting via `fail` and status codes
P0 · M16 · Task · deps: X-WASM-0025
Done when: `fail(code, message)` ends the invocation with `TransformOutcome::Fail` carrying a bounded message; error codes are namespaced so they cannot collide with host errors; tests cover messages longer than the bound and invalid UTF-8.

### [To Do] X-WASM-0032 ABI conformance kit of reference guest modules
P0 · M16 · Task · deps: X-WASM-0027, X-WASM-0028, X-WASM-0031, X-WASM-0024
Done when: `testdata/transforms/conformance/` holds WAT modules and expected outputs for every export, import and status code; the host passes the kit in CI; SDKs reuse the same expectations.

### [To Do] X-WASM-0033 Optional batch-mode export for performance
P3 · M16 · Task · deps: X-WASM-0032 · labels: perf
Done when: modules exporting `alluvium_transform_batch` receive a whole batch per call with identical semantics; conformance runs in both modes produce identical outputs; benchmarks show the gain or the export is dropped by ADR amendment.

## [To Do] X-WASM-F04 Module registry and deployment
P0 · M16 · Feature · deps: X-WASM-F03
Scope: Upload, version and roll back modules.

### [To Do] X-WASM-0034 Module and deployment metadata records
P0 · M16 · Task · deps: X-WASM-0005, META-F02
Done when: `TransformModuleRecord` (name, version, SHA-256, ABI version, size) and `TransformDeploymentRecord` (name, module version, mode, input topics, outputs, failure policy, config, state, activation epoch) are validated, replayed and snapshotted; counts are bounded by `alluvium.transform.max.modules` and `.max.deployments`; tests cover each validation error.

### [To Do] X-WASM-0035 Module blob storage in an internal compacted topic, content-addressed
P0 · M16 · Task · deps: X-WASM-0034, CTRL-F04
Done when: module bytes are stored in `__alluvium_transforms` keyed by SHA-256 with RF from config; brokers load blobs by hash and verify the hash before compiling; a hash mismatch rejects the blob and emits `transform_module_corrupt`; tests cover corruption.

### [To Do] X-WASM-0036 Module upload: chunked native API with hash verification and CLI
P0 · M16 · Task · deps: X-WASM-0035, X-NATIVE-F08, CLI-F01
Done when: a streaming native upload limited to `alluvium.transform.max.module.bytes` validates and compiles the module before committing the record; `alluvium transform upload <name> <file>` wraps it; duplicate content reuses the stored blob; tests cover oversize, invalid and duplicate uploads.

### [To Do] X-WASM-0037 Optional module signature verification
P1 · M16 · Task · deps: X-WASM-0036 · labels: security
Done when: with `alluvium.transform.require.signed.modules=true`, uploads must carry an Ed25519 signature over the module hash from a key in `alluvium.transform.trusted.signing.keys`; unsigned or wrongly signed modules are rejected; `alluvium transform sign` creates signatures; tests cover each case.

### [To Do] X-WASM-0038 Authorization: `Transform` resource type and module permissions
P0 · M16 · Task · deps: X-WASM-0034, SEC-F08 · labels: security
Done when: CREATE, ALTER, DESCRIBE and DELETE on `Transform` resources are enforced; deploying also requires READ on inputs and WRITE on outputs; without policies, cluster ALTER/DESCRIBE ACLs are required as documented; tests per operation.

### [To Do] X-WASM-0039 Deploy: bind a module version to topics with mode, outputs, config and failure policy
P0 · M16 · Task · deps: X-WASM-0036, X-WASM-0038
Done when: `alluvium transform deploy` and the native endpoint create a deployment after validating topics, outputs, mode (`produce`, `derived`, `filter`) and config; validation failures name the field; tests per validation.

### [To Do] X-WASM-0040 Rolling activation across brokers with an activation epoch
P0 · M16 · Task · deps: X-WASM-0039, X-WASM-0012
Done when: brokers compile the new version, report readiness, and switch at a deployment activation epoch so each partition uses exactly one version per batch; the deployment state machine Deploying → Active → Draining → Retired (or Failed with reason) records per-broker readiness; a simulation with broker crashes and leader moves converges.

### [To Do] X-WASM-0041 Rollback and automatic rollback on error rate
P0 · M16 · Task · deps: X-WASM-0040
Done when: `alluvium transform rollback` switches to the previous version through the same activation path; during rollout, an error ratio above `alluvium.transform.rollout.auto.rollback.error.ratio` over `.rollout.evaluation.ms` triggers automatic rollback with event reason `error_ratio_exceeded`; tests use the simulated clock.

### [To Do] X-WASM-0042 Undeploy, delete and garbage collection of module blobs
P0 · M16 · Task · deps: X-WASM-0040
Done when: undeploy drains in-flight work and retires the deployment; deleting a module version in use is refused; unreferenced blobs are tombstoned after `alluvium.transform.module.retention.ms`; tests cover each rule.

### [To Do] X-WASM-0043 Describe and list with per-broker status
P0 · M16 · Task · deps: X-WASM-0040
Done when: `alluvium transform list|describe` and native endpoints show module versions, deployments, state, per-broker readiness and last error; tests cover a partially rolled-out deployment.

### [To Do] X-WASM-0044 Deployment lifecycle events and flight recorder
P0 · M16 · Task · deps: X-WASM-0040, OBS-F06, OBS-F07
Done when: events `transform_deployed`, `transform_activated`, `transform_rolled_back` and `transform_retired` carry reasons; deployment transitions go to a per-deployment flight recorder readable at `/debug/transforms`; tests assert event output.

## [To Do] X-WASM-F05 Produce-time transforms
P0 · M16 · Feature · deps: X-WASM-F03
Scope: Validate, enrich and route records on append.

### [To Do] X-WASM-0045 Produce-time transform placement in the leader append path
P0 · M16 · Task · deps: X-WASM-F04, X-WASM-F02, REC-F12, REPL-F03
Done when: for topics with an active produce-mode deployment, the leader runs the transform after append validation and before offset assignment; control batches are never transformed; followers and mirrors never re-run transforms; tests show the stored data equals the transform output.

### [To Do] X-WASM-0046 Validate mode: per-record rejection with KIP-467 record errors
P0 · M16 · Task · deps: X-WASM-0045 · ref: KIP-467
Done when: a failing record rejects the whole batch with INVALID_RECORD; Produce v8+ responses include `record_errors` with batch index and the guest message (bounded) and `error_message`; older versions get INVALID_RECORD only; compat tests show the Java producer and librdkafka surface the error.

### [To Do] X-WASM-0047 Enrich mode: one-to-one map preserving idempotence
P0 · M16 · Task · deps: X-WASM-0045
Done when: the output count must equal the input count (otherwise the batch fails); the batch is rebuilt with the same producer id, epoch, base sequence and record count, recompressed with the topic's compression type and level, its CRC recomputed and `max.message.bytes` re-checked; an idempotent-producer test with retries shows duplicates are still detected.

### [To Do] X-WASM-0048 Route mode per the ADR: redirect records to declared output topics
P1 · M16 · Task · deps: X-WASM-0045, X-WASM-0027
Done when: routed records are appended to declared outputs and the Produce response is sent only after every destination append meets the request's acks; partial failure semantics and idempotence limits follow the ADR and are documented; tests cover success, a failed destination and acks=all.

### [To Do] X-WASM-0049 Produce-path backpressure from a saturated transform pool
P0 · M16 · Task · deps: X-WASM-0045, X-WASM-0013
Done when: when the worker queue is full, produce requests for transformed topics wait in the delayed-produce purgatory up to their timeout and then fail with REQUEST_TIMED_OUT, without blocking other topics; queue-full events are counted; a test saturates one deployment and shows untransformed topics unaffected.

### [To Do] X-WASM-0050 Decompression bounds and recompression on the transform path
P0 · M16 · Task · deps: X-WASM-0047, REC-F10
Done when: inputs are decompressed with REC-F10 bounds; outputs are recompressed with the topic's codec and level; a decompression bomb fails the batch with CORRUPT_MESSAGE before reaching the guest; tests per codec.

### [To Do] X-WASM-0051 Transforms on transactional and idempotent batches
P0 · M16 · Task · deps: X-WASM-0047
Done when: enrich and validate work on transactional batches without changing producer metadata; route mode on transactional batches is rejected at deploy unless the ADR allows it; tests with the Java transactional producer pass.

### [To Do] X-WASM-0052 Topic config for produce-time transforms
P0 · M16 · Task · deps: X-WASM-0045, X-WASM-0006
Done when: topic config `alluvium.transform.produce.deployment` binds a deployment, is validated (deployment exists, mode `produce`), and is reported by DescribeConfigs; removing it stops transforming new batches; tests cover each validation.

### [To Do] X-WASM-0053 Produce-time invariants under simulation
P0 · M16 · Task · deps: X-WASM-0047, X-WASM-0046, TEST-F06 · labels: test
Done when: a seeded simulation with leader changes, crashes and rollouts asserts every appended batch is the transform output of an accepted input, enrich preserves record counts and sequences, and no batch is transformed twice.

### [To Do] X-WASM-0054 Compatibility: Kafka producers against transformed topics
P0 · M16 · Task · deps: X-WASM-0046, X-WASM-0047, TEST-F08 · labels: test
Done when: Java client and librdkafka producers (idempotent and not) produce to validate- and enrich-mode topics; accepted, rejected and retried batches behave as specified; consumers read the transformed records.

### [To Do] X-WASM-0055 Produce-time transform benchmarks
P0 · M16 · Task · deps: X-WASM-0047, TEST-F11 · labels: perf
Done when: macro benchmarks report produce throughput and p99 latency for no transform, a no-op enrich and a JSON field-enrich module; budgets from the ADR are enforced in CI.

## [To Do] X-WASM-F06 Derived topics
P0 · M16 · Feature · deps: X-WASM-F03
Scope: Filtered and projected views consumable by Kafka clients.

### [To Do] X-WASM-0056 Derived topic model and metadata
P0 · M16 · Task · deps: X-WASM-F04
Done when: `DerivedTopicRecord` (derived topic id, source topic, deployment, start position, state) is validated (same partition count as the source, source exists, deployment mode `derived`), replayed and snapshotted; tests cover each validation error.

### [To Do] X-WASM-0057 Derived topics are read-only for clients
P0 · M16 · Task · deps: X-WASM-0056, REPL-F03
Done when: Produce, DeleteRecords and CreatePartitions on derived topics fail with the read-only error code chosen in the ADR for every API version; tests per API.

### [To Do] X-WASM-0058 Materialization task per derived partition
P0 · M16 · Task · deps: X-WASM-0056, X-WASM-0013, REPL-F13
Done when: the derived partition's leader runs a supervised task that reads the source partition through the service-layer read path from its checkpointed position, runs the transform on the worker pool and appends outputs; bounded in-flight bytes `alluvium.transform.derived.max.in.flight.bytes`; leadership changes move the task; tests cover leader moves.

### [To Do] X-WASM-0059 Exactly-once materialization: outputs and source position committed atomically
P0 · M16 · Task · deps: X-WASM-0058
Done when: each derived append atomically records the source offset range it covers (using the mechanism chosen in the ADR, such as a per-partition checkpoint committed with the append), so on restart the task resumes after the last appended range; crash tests at every write point show no duplicate and no gap in source coverage.

### [To Do] X-WASM-0060 Filter and projection semantics and partitioning
P0 · M16 · Task · deps: X-WASM-0058
Done when: filtered records produce no output; projected records keep the source partition (no repartitioning) even if the key changes, as documented; timestamps follow the ABI rules; tests per case.

### [To Do] X-WASM-0061 Derived topics as normal topics for Kafka clients
P0 · M16 · Task · deps: X-WASM-0058, X-WASM-0057
Done when: derived topics appear in Metadata, ListOffsets and DescribeConfigs; consumer groups, offsets and read_committed work; a compat test consumes a derived topic with the Java client and librdkafka.

### [To Do] X-WASM-0062 Backfill from the source log start or from latest
P0 · M16 · Task · deps: X-WASM-0058
Done when: `start.position` (`earliest` or `latest`) chooses the first source offset; backfill is throttled by `alluvium.transform.derived.backfill.max.bytes.per.second`; progress is reported; tests cover both positions.

### [To Do] X-WASM-0063 Source retention overtaking the materializer
P0 · M16 · Task · deps: X-WASM-0058
Done when: if the source log start passes the task's position, the task skips to the new start, emits `derived_topic_gap` with the skipped range and counts it; tests use retention on the simulated clock.

### [To Do] X-WASM-0064 Module upgrades on derived topics with a recorded switch offset and optional rebuild
P0 · M16 · Task · deps: X-WASM-0059, X-WASM-0040
Done when: a new module version applies from a switch offset recorded per partition; `alluvium transform rebuild <derived-topic>` recreates the derived topic from `earliest` under the new version; tests cover both.

### [To Do] X-WASM-0065 Authorization for derived topics
P0 · M16 · Task · deps: X-WASM-0056, SEC-F08 · labels: security
Done when: creating a derived topic requires READ on the source and CREATE on the derived topic; reading it requires READ on the derived topic only, so admins can expose filtered views; tests cover each case.

### [To Do] X-WASM-0066 Virtual derived topics evaluated at fetch time with source offsets
P2 · M16 · Task · deps: X-WASM-0060, REPL-F13
Done when: with `mode=virtual`, fetches of the derived topic run the transform over source batches and return rebuilt batches that keep source offsets (filtered records become gaps); a bounded result cache avoids recomputation; a compat test shows Kafka consumers handle the gaps.

### [To Do] X-WASM-0067 Derived topic simulation
P0 · M16 · Task · deps: X-WASM-0059, TEST-F06 · labels: test
Done when: a seeded simulation with crashes and leader changes asserts each derived partition equals the transform applied to its source partition exactly once, in order.

### [To Do] X-WASM-0068 Derived topic lag metrics and events
P0 · M16 · Task · deps: X-WASM-0058, OBS-F04
Done when: `alluvium_transform_derived_lag_records{topic}` and `_lag_seconds{topic}` exist; events `derived_topic_state_changed` carry reasons; tests assert values.

### [To Do] X-WASM-0069 Derived topic throughput benchmarks
P0 · M16 · Task · deps: X-WASM-0058, TEST-F11 · labels: perf
Done when: benchmarks report materialization throughput and CPU for filter and projection modules and the impact on source produce latency; results are in docs/benchmarks/transforms.md.

## [To Do] X-WASM-F07 Server-side filtering
P1 · M16 · Feature · deps: X-WASM-F03, X-NATIVE-F06
Scope: Predicate pushdown for native consumers.

### [To Do] X-WASM-0070 Filter predicate expression grammar and parser
P1 · M16 · Task · deps: X-WASM-0006 · labels: decoder
Done when: the grammar covers comparisons on key, headers, timestamp and schema fields, `AND`/`OR`/`NOT`, `IN`, `LIKE` and `IS NULL`; the parser bounds expression size and depth (`alluvium.filter.max.expression.bytes`, `.max.depth`) and returns positioned errors; fuzz target `fuzz_filter_expression` runs clean.

### [To Do] X-WASM-0071 Schema-aware field access for predicates
P1 · M16 · Task · deps: X-WASM-0070, X-SCHEMA-F04, X-SCHEMA-F05, X-SCHEMA-F06
Done when: field paths resolve against each record's registered schema for Avro, Protobuf and JSON; undecodable records evaluate to false and are counted; tests per format.

### [To Do] X-WASM-0072 Predicate evaluation on the native consume path with continuation cursors
P1 · M16 · Task · deps: X-WASM-0071
Done when: native consume requests with a filter return only matching records and a cursor past the last scanned offset; scanning stops at `alluvium.filter.max.records.scanned.per.request` or `.max.bytes.scanned.per.request` and returns a partial response with the cursor; tests cover empty matches across many batches.

### [To Do] X-WASM-0073 WASM filter modules as predicates
P2 · M16 · Task · deps: X-WASM-0072, X-WASM-0025
Done when: a request may name a deployment of mode `filter`; the module's keep/drop decision filters records with the same cursor semantics; tests compare expression and module filters on the same data.

### [To Do] X-WASM-0074 Scanned bytes count toward fetch quotas
P1 · M16 · Task · deps: X-WASM-0072, QUOTA-F03 · labels: security
Done when: bytes scanned (not just bytes returned) are charged to the principal's fetch quota, so filters cannot bypass quotas; a test shows throttling with a filter matching nothing.

### [To Do] X-WASM-0075 Authorization and side-channel note for filtered reads
P1 · M16 · Task · deps: X-WASM-0072 · labels: security
Done when: filtered reads require READ on the topic like unfiltered reads; the docs note that response timing can reveal match rates; tests cover denial.

### [To Do] X-WASM-0076 Filtering equals client-side filtering
P1 · M16 · Task · deps: X-WASM-0072 · labels: test
Done when: a property test over random records and predicates shows the server-filtered stream equals filtering the full stream client-side, including across partial responses.

### [To Do] X-WASM-0077 Filter metrics
P1 · M16 · Task · deps: X-WASM-0072, OBS-F04
Done when: `alluvium_filter_records_scanned_total`, `_records_matched_total` and `_partial_responses_total` exist; tests assert updates.

### [To Do] X-WASM-0078 Server-side filtering benchmarks
P2 · M16 · Task · deps: X-WASM-0072, TEST-F11 · labels: perf
Done when: benchmarks report network bytes saved and broker CPU per scanned GB at selectivities 1%, 10% and 50%; results are in docs/benchmarks/transforms.md.

## [To Do] X-WASM-F08 Transform failure policies
P0 · M16 · Feature · deps: X-WASM-F05
Scope: DLQ, skip, fail.

### [To Do] X-WASM-0079 Failure policy model and error classification
P0 · M16 · Task · deps: X-WASM-0031
Done when: deployments declare `failure.policy` (`fail`, `skip`, `dlq`) with optional per-class overrides for guest failure, trap, budget exceeded and output validation; the classification of every `TransformError` variant is one exhaustive match; tests per class.

### [To Do] X-WASM-0080 `fail` policy for produce-time and derived deployments
P0 · M16 · Task · deps: X-WASM-0079, X-WASM-0046, X-WASM-0058
Done when: produce-time failures reject the batch with INVALID_RECORD and record errors; derived failures pause the derived partition (state Failed with reason) until rollback, redeploy or `alluvium transform resume`; tests cover both.

### [To Do] X-WASM-0081 `skip` policy semantics
P0 · M16 · Task · deps: X-WASM-0079
Done when: produce-time skip passes the original record through unchanged and derived skip drops it, as documented; each skip is counted per deployment and class; tests cover both modes.

### [To Do] X-WASM-0082 `dlq` policy: dead-letter topic with provenance headers
P0 · M16 · Task · deps: X-WASM-0079, X-WASM-0028
Done when: failed inputs are written to `alluvium.transform.dlq.topic` with headers `alluvium.dlq.error.class`, `.error.message` (bounded), `.deployment`, `.module.version`, `.source.topic`, `.source.partition` and `.source.offset`; deployers need WRITE on the DLQ; if the DLQ write fails the `fail` policy applies; tests cover each path.

### [To Do] X-WASM-0083 Automatic pause on sustained error rate
P1 · M16 · Task · deps: X-WASM-0081, X-WASM-0082
Done when: if the failure ratio exceeds `alluvium.transform.circuit.breaker.error.ratio` over `.window.ms`, the deployment pauses with event reason `circuit_breaker_open`; resume is manual; tests use the simulated clock.

### [To Do] X-WASM-0084 Failure policy metrics and events
P0 · M16 · Task · deps: X-WASM-0080, X-WASM-0081, X-WASM-0082, OBS-F04
Done when: `alluvium_transform_failures_total{deployment,class,action}` exists; events `transform_record_failed` (sampled, no payload) and `transform_paused` carry reasons; tests assert output.

### [To Do] X-WASM-0085 Failure policy tests for every class and mode
P0 · M16 · Task · deps: X-WASM-0080, X-WASM-0081, X-WASM-0082 · labels: test
Done when: a matrix test covers each policy × error class × mode (produce-time, derived) with WAT fixtures and checks outputs, DLQ contents, errors and metrics.

## [To Do] X-WASM-F09 Transform SDKs
P1 · M16 · Feature · deps: X-WASM-F03
Scope: Rust, TinyGo and AssemblyScript SDKs and a local test harness.

### [To Do] X-WASM-0086 Rust guest SDK crate: record types, export macro and ABI bindings
P1 · M16 · Task · deps: X-WASM-0032
Done when: `alluvium-transform-sdk` (`no_std` + `alloc`, target `wasm32-unknown-unknown`) exposes `Record`, `Headers`, `Output` and an `export_transform!` macro implementing ABI v1; it passes the conformance kit; docs.rs-style module docs explain usage.

### [To Do] X-WASM-0087 Rust SDK examples: filter, mask, enrich and route
P1 · M16 · Task · deps: X-WASM-0086
Done when: `examples/transforms/rust/` contains the four examples, each built in CI and tested with the local harness against fixtures.

### [To Do] X-WASM-0088 TinyGo guest SDK
P1 · M16 · Task · deps: X-WASM-0032
Done when: Go package `alluvium.dev/transform` implements ABI v1 with `//export` functions and TinyGo-compatible memory management; it passes the conformance kit with a pinned TinyGo version in CI; examples for filter and enrich are included.

### [To Do] X-WASM-0089 AssemblyScript guest SDK
P1 · M16 · Task · deps: X-WASM-0032
Done when: an npm package implements ABI v1 bindings; it passes the conformance kit with a pinned AssemblyScript compiler in CI; examples for filter and enrich are included.

### [To Do] X-WASM-0090 Local test harness: `alluvium transform test`
P1 · M16 · Task · deps: X-WASM-0032, X-WASM-0014, CLI-F01
Done when: the command runs a module against JSON or binary record fixtures using the production sandbox configuration and prints outputs, errors, fuel and memory per record; expected-output files make it usable in CI; results are deterministic across runs.

### [To Do] X-WASM-0091 Project templates: `alluvium transform init --lang rust|tinygo|assemblyscript`
P2 · M16 · Task · deps: X-WASM-0086, X-WASM-0088, X-WASM-0089
Done when: each template builds, passes `alluvium transform test` and includes a README; a CI job instantiates and builds all three.

### [To Do] X-WASM-0092 SDK versioning and release process
P2 · M16 · Task · deps: X-WASM-0086, X-WASM-0088, X-WASM-0089, SEC-F15
Done when: SDK versions map to ABI versions in a documented table; release jobs publish to crates.io, the Go module proxy and npm with provenance; the process is in docs/transforms/sdk-releases.md.

### [To Do] X-WASM-0093 SDK guides per language
P1 · M16 · Task · deps: X-WASM-0087, X-WASM-0088, X-WASM-0089 · labels: docs
Done when: docs/transforms/sdk-{rust,tinygo,assemblyscript}.md cover setup, the record API, routing, errors, limits, testing with the harness and deployment.

## [To Do] X-WASM-F10 Transform resource isolation
P0 · M16 · Feature · deps: X-WASM-F02
Scope: Per-transform quotas and isolation.

### [To Do] X-WASM-0094 Per-deployment CPU, memory and output quotas
P0 · M16 · Task · deps: X-WASM-0013
Done when: each deployment has `quota.fuel.per.second`, `quota.output.bytes.per.second` and a memory share; exceeding a rate quota delays its work (never others') and counts throttle time; tests use the simulated clock.

### [To Do] X-WASM-0095 Weighted fair scheduling on the worker pool
P0 · M16 · Task · deps: X-WASM-0094
Done when: the worker pool schedules batches by deployment weight (and by namespace when X-TENANT is enabled) using a deterministic fair-queue algorithm; a property test shows long-run shares within 5% of weights for backlogged deployments.

### [To Do] X-WASM-0096 Separate pools for latency-critical and background work
P0 · M16 · Task · deps: X-WASM-0013
Done when: produce-time and filter work runs on `alluvium.transform.foreground.threads`; derived-topic work runs on `alluvium.transform.background.threads`; a test saturates derived work and shows produce-time latency unaffected.

### [To Do] X-WASM-0097 Admission control for total memory and instances
P0 · M16 · Task · deps: X-WASM-0094
Done when: deploys are rejected when the summed memory reservations exceed `alluvium.transform.total.memory.bytes` or instances exceed `alluvium.transform.total.instances`, with an error naming the limit; tests cover rejection and release on undeploy.

### [To Do] X-WASM-0098 Per-namespace transform limits
P2 · M16 · Task · deps: X-WASM-0097, X-TENANT-F04
Done when: namespaces have `max.transform.deployments`, `max.transform.modules` and `max.transform.memory.bytes`; exceeding them rejects deploys with a quota error; tests cover each limit.

### [To Do] X-WASM-0099 Noisy-neighbor test
P0 · M16 · Task · deps: X-WASM-0095, X-WASM-0096 · labels: test, nfr
Done when: a benchmark runs a fuel-maxing hostile module beside a normal deployment and asserts the normal deployment's p99 stays within the budget from the ADR.

### [To Do] X-WASM-0100 Isolation metrics and events
P0 · M16 · Task · deps: X-WASM-0094, OBS-F04
Done when: `alluvium_transform_throttle_seconds_total{deployment,kind}` and `alluvium_transform_pool_queue_depth{pool}` exist; event `transform_throttled` fires once per throttling episode with the quota kind.

## [To Do] X-WASM-F11 Transform observability, testing and documentation
P0 · M16 · Feature · deps: X-WASM-F05
Scope: Metrics, tests, benchmarks, guide.

### [To Do] X-WASM-0101 Transforms are off by default
P0 · M16 · Task · deps: X-WASM-F06, X-WASM-F07, X-WASM-F08, X-WASM-F10 · labels: test
Done when: test `transforms_off_by_default` starts a node with default config and asserts `alluvium.transform.version` is 0, the engine is never initialized, no worker threads exist, the produce path is unchanged (identical batches) and no internal transform topic is created.

### [To Do] X-WASM-0102 Kafka compatibility and differential suites pass with transforms enabled but unused
P0 · M16 · Task · deps: X-WASM-0101, TEST-F08, TEST-F09 · labels: test
Done when: the full compat and differential suites run in CI with feature level 1 and the worker pool started but no deployments; all pass with no new ignore rules except a documented one for Alluvium internal topics.

### [To Do] X-WASM-0103 Transform metrics catalog and docs/metrics.md entries
P0 · M16 · Task · deps: X-WASM-F06, OBS-F04
Done when: every X-WASM metric (invocations, latency histogram, records in, out and dropped, failures by class, fuel, memory, compile time, lag, throttling) is documented with labels and cardinality bounds; a test lists registered metrics and compares with the doc.

### [To Do] X-WASM-0104 Transform spans
P0 · M16 · Task · deps: X-WASM-F06, OBS-F03
Done when: each batch invocation runs in span `transform.invoke` with standard fields plus `deployment` and `module_version`; spans never carry payloads; a test asserts span fields.

### [To Do] X-WASM-0105 End-to-end transform simulation
P0 · M16 · Task · deps: X-WASM-F06, X-WASM-F08, TEST-F06 · labels: test
Done when: a seeded simulation combines rollouts, rollbacks, produce-time transforms, derived topics, failure policies, crashes and leader changes and checks all X-WASM invariants every step for thousands of seeds nightly.

### [To Do] X-WASM-0106 Nightly fuzz campaign for every X-WASM decoder
P0 · M16 · Task · deps: X-WASM-F02, X-WASM-F07, TEST-F07 · labels: security, test
Done when: the sandbox, host-call, ABI output and filter expression fuzz targets run nightly with committed corpora and a triage owner; coverage per target is reported.

### [To Do] X-WASM-0107 Transform benchmark report and performance budget
P0 · M16 · Task · deps: X-WASM-0019, X-WASM-F06 · labels: perf, docs
Done when: docs/benchmarks/transforms.md reports invocation overhead, produce-time impact, derived throughput and filtering savings on the standard hardware, with the budgets enforced by CI benchmarks.

### [To Do] X-WASM-0108 Transforms guide: writing, deploying and operating
P0 · M16 · Task · deps: X-WASM-F06, X-WASM-F08, X-WASM-F09 · labels: docs
Done when: docs/transforms/guide.md covers concepts (modes, derived topics, filtering), the lifecycle (upload, deploy, rollout, rollback), failure policies, limits and quotas, monitoring, and troubleshooting with `/debug/transforms`.

### [To Do] X-WASM-0109 Opt-in and migration guide from Connect SMTs and stateless Kafka Streams apps
P1 · M16 · Task · deps: X-WASM-0108 · labels: docs
Done when: docs/transforms/migration.md explains enabling the feature level, and maps common Connect Single Message Transforms and stateless Streams topologies (filter, mapValues, branch) to transforms with worked examples.

### [To Do] X-WASM-0110 Sandbox guarantees and security documentation
P0 · M16 · Task · deps: X-WASM-F02, X-WASM-F10 · labels: docs, security
Done when: docs/security/transforms.md documents sandbox guarantees, limits and their config keys, module signing, authorization for modules, deployments and derived topics, and residual risks from the threat model.
