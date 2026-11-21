# X-LOWLAT: Low-latency mode

> Crates: `alluvium-storage`, `alluvium-network`, `alluvium-server` · Kafka counterpart: none (Alluvium enhancement) · Milestones: M18

A mode tuned for tail latency: io_uring storage, group commit, zero-allocation request paths, CPU pinning and latency regression gates.

## [To Do] X-LOWLAT-F01 Low-latency design
P0 · M18 · Feature · deps: STOR-F18
Scope: Spike and ADR: revisit thread-per-core and io_uring with benchmark evidence.

### [To Do] X-LOWLAT-0001 Low-latency mode ADR (proposed): tail-latency sources, target percentiles, candidate techniques and the evaluation plan
P0 · M18 · Spike · deps: none · labels: perf · ref: ADR-0002
Done when: ADR merged in Proposed status covering io_uring, group commit, thread-per-core, allocation-free paths, preallocation and pinning, naming the scenarios and percentiles (p99, p99.9, p99.99) to be judged, the evidence each candidate must produce, the opt-in model (every technique off by default) and the decision criteria; references architecture §10 and the STOR-F18 decision.

### [To Do] X-LOWLAT-0002 Baseline tail-latency profile of the parity stack: attribute p99.9 to fsync, page faults, allocation, scheduler wakeups and lock contention with on-CPU and off-CPU flame graphs and request phase metrics
P0 · M18 · Spike · deps: X-LOWLAT-0001, TEST-F11 · labels: perf
Done when: report appended to the ADR with reproducible commands, hardware and kernel recorded, and a ranked list of tail-latency contributors per scenario.

### [To Do] X-LOWLAT-0003 Thread-per-core prototype: partitions sharded to cores with per-core I/O versus the Tokio multi-thread runtime plus I/O pool, measured on the same workloads
P0 · M18 · Spike · deps: X-LOWLAT-0001 · labels: perf
Done when: prototype branch and results (p99.9 and throughput per core count) appended to the ADR, with the cost of cross-core requests (multi-partition produce and fetch) measured.

### [To Do] X-LOWLAT-0004 io_uring crate evaluation and unsafe exception: io-uring vs rustix-based vs tokio-uring, unsafe audit, maintenance, kernel support, confinement to one module
P0 · M18 · Spike · deps: X-LOWLAT-0001 · labels: security · ref: docs/standards/security.md
Done when: dependency justification per the security standard (unsafe usage, maintainers, transitive count, advisories) and an ADR-0006-style exception scoped to one module, accepted or rejected with reasons.

### [To Do] X-LOWLAT-0005 Accept the low-latency ADR with evidence: decision for each candidate technique, the opt-in keys, and the kernel and platform support matrix
P0 · M18 · Spike · deps: X-LOWLAT-0002, X-LOWLAT-0003, X-LOWLAT-0004
Done when: ADR status Accepted; each later X-LOWLAT feature cites the section it implements; rejected techniques recorded with their numbers.

### [To Do] X-LOWLAT-0006 Publish low-latency mode targets in docs/nfr.md (p50, p99, p99.9, p99.99 per scenario) beside the parity targets
P0 · M18 · Task · deps: X-LOWLAT-0005, NFR-F01 · labels: nfr, docs
Done when: targets table merged with hardware assumptions; each target maps to a benchmark scenario in X-LOWLAT-F08.

## [To Do] X-LOWLAT-F02 io_uring storage backend
P0 · M18 · Feature · deps: X-LOWLAT-F01
Scope: Linux io_uring implementation of the fs trait.

### [To Do] X-LOWLAT-0007 io_uring capability detection: kernel version, opcode probe (`IORING_REGISTER_PROBE`), `kernel.io_uring_disabled` sysctl and seccomp denial, with fallback to the I/O pool backend
P0 · M18 · Task · deps: none
Done when: event `io_backend_selected` with reason (`selected`, `kernel_too_old`, `opcode_missing`, `disabled_by_sysctl`, `denied_by_seccomp`); tests with an injectable probe.

### [To Do] X-LOWLAT-0008 `alluvium.storage.io.backend` = `pool` (default), `io_uring` or `auto`; `io_uring` on non-Linux builds rejected at config validation
P0 · M18 · Task · deps: X-LOWLAT-0007, CFG-F11 · labels: config
Done when: `io_uring` with a failed probe refuses startup with a clear error; `auto` falls back with the event; tests per platform.

### [To Do] X-LOWLAT-0009 Ring ownership: one ring per storage I/O thread, SQ size `alluvium.storage.io.uring.entries`, bounded in-flight submissions with backpressure to callers, CQ overflow detection
P0 · M18 · Task · deps: X-LOWLAT-0008
Done when: in-flight never exceeds the bound (test with a stalled device mock); CQ overflow sets a flag that is handled without losing completions.

### [To Do] X-LOWLAT-0010 io_uring operation: openat with O_CLOEXEC and close, using registered file slots (`IORING_REGISTER_FILES`) with fallback when slots run out
P0 · M18 · Task · deps: X-LOWLAT-0009
Done when: slot exhaustion falls back to unregistered descriptors and is counted; open/close conformance tests pass.

### [To Do] X-LOWLAT-0011 io_uring operation: positional write and writev for segment and index appends, with short-write continuation
P0 · M18 · Task · deps: X-LOWLAT-0010
Done when: short writes continue until complete (fault-injected test); appends land at the expected positions.

### [To Do] X-LOWLAT-0012 io_uring operation: positional read with registered buffers (`IORING_REGISTER_BUFFERS`) for fetch reads and recovery scans
P0 · M18 · Task · deps: X-LOWLAT-0010
Done when: registered buffer pool bounded; reads beyond EOF return the short length; conformance tests pass.

### [To Do] X-LOWLAT-0013 io_uring operation: fsync and fdatasync (`IORING_FSYNC_DATASYNC`) for segment flushes and checkpoint durability
P0 · M18 · Task · deps: X-LOWLAT-0010
Done when: conformance tests prove ordering with prior writes; fdatasync used for data files and fsync for directories.

### [To Do] X-LOWLAT-0014 io_uring operation chain: linked write then fdatasync (`IOSQE_IO_LINK`) for durable appends, with partial-failure handling
P0 · M18 · Task · deps: X-LOWLAT-0011, X-LOWLAT-0013
Done when: A failed write cancels the linked fsync and both surface typed errors; test with fault injection.

### [To Do] X-LOWLAT-0015 io_uring operation: fallocate for segment preallocation
P1 · M18 · Task · deps: X-LOWLAT-0010
Done when: preallocated size reported by statx; unsupported filesystems fall back to the pool path with a counter.

### [To Do] X-LOWLAT-0016 io_uring operation: renameat for checkpoint atomic replace and segment `.deleted` renames, followed by directory fsync
P0 · M18 · Task · deps: X-LOWLAT-0010, X-LOWLAT-0013
Done when: checkpoint replace sequence (write tmp, fsync, rename, dir fsync) passes the crash-consistency cases that apply.

### [To Do] X-LOWLAT-0017 io_uring operation: unlinkat for segment and index deletion
P0 · M18 · Task · deps: X-LOWLAT-0010
Done when: deletion conformance tests pass; ENOENT on an already-deleted file is a typed error the caller handles.

### [To Do] X-LOWLAT-0018 io_uring operation: statx for file size and recovery checks
P0 · M18 · Task · deps: X-LOWLAT-0010
Done when: sizes match `std::fs::metadata` in conformance tests.

### [To Do] X-LOWLAT-0019 Error mapping: CQE errno to typed `FsError { op, path, errno }`; EINTR and EAGAIN retried; ENOSPC to the disk-full path; EIO takes the log dir offline
P0 · M18 · Task · deps: X-LOWLAT-0014, X-LOWLAT-0016, X-LOWLAT-0017, X-LOWLAT-0018, X-LOWLAT-0012, STOR-F25, STOR-F21
Done when: one exhaustive mapping with no wildcard arm; test per errno class.

### [To Do] X-LOWLAT-0020 The `fs` trait conformance suite passes on the io_uring backend on ext4, xfs and tmpfs in Linux CI
P0 · M18 · Task · deps: X-LOWLAT-0019 · labels: test
Done when: same suite as the pool backend with no backend-specific skips; CI job on a kernel that supports every used opcode.

### [To Do] X-LOWLAT-0021 Crash consistency on io_uring: process kill during linked write and fsync and during checkpoint replace, verified with the recovery invariants
P0 · M18 · Task · deps: X-LOWLAT-0014, X-LOWLAT-0016, STOR-F20 · labels: test
Done when: repeated kill-and-recover runs show no acknowledged durable write lost and no corrupt checkpoint.

### [To Do] X-LOWLAT-0022 Metrics: `alluvium_storage_uring_inflight`, `alluvium_storage_uring_sq_full_total`, `alluvium_storage_uring_op_seconds{op}`, `alluvium_storage_uring_fallback_total{reason}`
P0 · M18 · Task · deps: X-LOWLAT-0009 · labels: observability
Done when: values asserted in tests; rows in docs/metrics.md.

### [To Do] X-LOWLAT-0023 Benchmarks: append plus fdatasync latency and random read latency (p50 to p99.99) versus the pool backend on NVMe
P0 · M18 · Task · deps: X-LOWLAT-0020, TEST-F11 · labels: perf
Done when: results published and compared with the ADR evidence; regression tracked in the benchmark job.

### [To Do] X-LOWLAT-0024 Docs: kernel and container requirements (seccomp profiles, `kernel.io_uring_disabled`), enabling and rollback
P1 · M18 · Task · deps: X-LOWLAT-0020 · labels: docs
Done when: page lists the minimum kernel and each opcode used; container seccomp example tested.

## [To Do] X-LOWLAT-F03 Group commit
P0 · M18 · Feature · deps: STOR-F17, X-LOWLAT-F01
Scope: Batched fsync for strict durability at low latency.

### [To Do] X-LOWLAT-0025 `alluvium.log.flush.mode` = `kafka` (default, parity `flush.messages`/`flush.ms`) or `group-commit`, with validation against topic flush overrides
P0 · M18 · Task · deps: X-LOWLAT-0001, CFG-F11 · labels: config
Done when: conflicting topic overrides in group-commit mode are rejected with INVALID_CONFIG and a message naming the key; default behavior unchanged (test).

### [To Do] X-LOWLAT-0026 Group commit coordinator (sans-I/O, in `alluvium-storage`): dirty segments per log dir, commit-cycle decisions, and completion of appends whose offsets are durable
P0 · M18 · Task · deps: X-LOWLAT-0025 · ref: ADR-0002
Done when: unit tests for cycle start, completion and appends arriving during a cycle; invariants: an append completes only after a cycle that started after it.

### [To Do] X-LOWLAT-0027 Group commit policy: immediate - fsync after every append (per-batch durability baseline)
P0 · M18 · Task · deps: X-LOWLAT-0026
Done when: policy selectable by `alluvium.log.group.commit.policy=immediate`; test shows one cycle per append.

### [To Do] X-LOWLAT-0028 Group commit policy: fixed window - one commit cycle every `alluvium.log.group.commit.max.delay.us`
P0 · M18 · Task · deps: X-LOWLAT-0026
Done when: test with the simulated clock: appends in one window share one cycle; no append waits longer than the window plus fsync time.

### [To Do] X-LOWLAT-0029 Group commit policy: size-triggered - commit when pending bytes reach `alluvium.log.group.commit.max.batch.bytes` or the window ends
P0 · M18 · Task · deps: X-LOWLAT-0028
Done when: test shows early cycles under heavy load and window-bound cycles under light load.

### [To Do] X-LOWLAT-0030 Group commit policy: adaptive - commit immediately when idle and accumulate while a cycle is in flight, bounded by the maximum delay
P0 · M18 · Task · deps: X-LOWLAT-0026
Done when: property test: at most one cycle in flight per log dir; latency at low load equals the immediate policy within one fsync.

### [To Do] X-LOWLAT-0031 Commit cycle execution: parallel fdatasync of every dirty file on the log dir with `alluvium.log.group.commit.max.parallel.fsyncs`, and directory fsync when files were created
P0 · M18 · Task · deps: X-LOWLAT-0026
Done when: concurrency bound respected (test); new segment files are durable in their directory before appends complete.

### [To Do] X-LOWLAT-0032 Durable offset per partition: published `durable_offset` (at most the LEO) advanced after commit cycles, never moving backwards except on truncation
P0 · M18 · Task · deps: X-LOWLAT-0031
Done when: invariant checked with `debug_assert!` and a property test; exposed on `/debug/partitions/{topic}/{partition}`.

### [To Do] X-LOWLAT-0033 Produce completion in group-commit mode: acks=1 completes at the leader's durable offset; acks=all waits for the HW and leader durability through a `durable_offset` delayed-produce watch key
P0 · M18 · Task · deps: X-LOWLAT-0032, REPL-F08
Done when: tests for acks=0, 1 and all; REQUEST_TIMED_OUT when durability does not arrive within the timeout.

### [To Do] X-LOWLAT-0034 Follower durability: followers make fetched data durable before sending their next fetch, so the leader's HW implies durability on every ISR member with no protocol change
P0 · M18 · Task · deps: X-LOWLAT-0032, REPL-F06
Done when: simulation shows the HW never passes an offset that is not durable on every ISR member; follower fetch latency impact recorded.

### [To Do] X-LOWLAT-0035 Crash-consistency tests: losing all unsynced data on all replicas at once loses no acknowledged write in group-commit mode
P0 · M18 · Task · deps: X-LOWLAT-0033, X-LOWLAT-0034, TEST-F03 · labels: test
Done when: new simulation checker for durable acknowledgements; thousands of seeds pass; the same fault plan shows losses in `kafka` mode (documented contrast).

### [To Do] X-LOWLAT-0036 fsync failure handling: EIO from fsync takes the log dir offline without retrying, and pending appends fail with KAFKA_STORAGE_ERROR
P0 · M18 · Task · deps: X-LOWLAT-0031, STOR-F21
Done when: fault-injected test; event with reason `fsync_failed`; no append completes after the failed cycle.

### [To Do] X-LOWLAT-0037 Observability: `alluvium_log_group_commit_batch_bytes`, `alluvium_log_group_commit_fsync_seconds`, `alluvium_log_group_commit_wait_seconds`, durable-lag gauge, event `flush_mode_changed`
P0 · M18 · Task · deps: X-LOWLAT-0032 · labels: observability
Done when: values asserted in tests; rows in docs/metrics.md and docs/events.md.

### [To Do] X-LOWLAT-0038 Benchmark: acks=all p99 and p99.9 with group commit (each policy) versus per-append fsync versus Kafka's default flush; throughput at a fixed p99.9
P0 · M18 · Task · deps: X-LOWLAT-0033, X-LOWLAT-0030, X-LOWLAT-0029, TEST-F11 · labels: perf
Done when: results published; the default policy chosen in the ADR is the best measured at the target load.

### [To Do] X-LOWLAT-0039 Docs: durability model in group-commit mode versus Kafka's default, policy selection and recommended settings
P1 · M18 · Task · deps: X-LOWLAT-0033, X-LOWLAT-0034 · labels: docs
Done when: page states exactly which failures each mode survives.

## [To Do] X-LOWLAT-F04 Zero-allocation request path
P1 · M18 · Feature · deps: X-LOWLAT-F01
Scope: Buffer pools and allocation-free hot paths.

### [To Do] X-LOWLAT-0040 Allocation counting harness: test-only counting global allocator and a per-request allocation report for produce and fetch benchmarks
P1 · M18 · Task · deps: none · labels: test, perf
Done when: report lists allocation sites per request (top 20 by count); harness excluded from release builds.

### [To Do] X-LOWLAT-0041 Buffer pool core: size-classed, bounded total bytes, reusing `BytesMut` without unsafe, returned on drop, with hit and miss counters
P1 · M18 · Task · deps: X-LOWLAT-0040
Done when: unit tests for bound enforcement and reuse; property test: bytes in use never exceed the bound.

### [To Do] X-LOWLAT-0042 Buffer pool: network frame read buffers by frame-size class, bounded by `alluvium.network.buffer.pool.bytes`
P1 · M18 · Task · deps: X-LOWLAT-0041, NET-F03
Done when: steady-state produce traffic shows zero frame-buffer allocations in the harness; bound respected.

### [To Do] X-LOWLAT-0043 Buffer pool: response encoding buffers
P1 · M18 · Task · deps: X-LOWLAT-0041, NET-F06
Done when: steady-state responses encode into pooled buffers; harness shows zero encode allocations.

### [To Do] X-LOWLAT-0044 Buffer pool: decompression scratch buffers for record validation, bounded by `message.max.bytes`
P1 · M18 · Task · deps: X-LOWLAT-0041, REC-F10
Done when: compressed produce in steady state allocates no decompression buffers; decompression bombs still rejected.

### [To Do] X-LOWLAT-0045 Buffer pool: fetch read buffers for TLS and memory-backend paths where sendfile is not used
P1 · M18 · Task · deps: X-LOWLAT-0041
Done when: TLS fetch in steady state shows zero read-buffer allocations.

### [To Do] X-LOWLAT-0046 Borrowed request header decode: client_id and topic names as slices of the frame instead of owned strings
P1 · M18 · Task · deps: X-LOWLAT-0040, PROTO-F07
Done when: decoded header borrows from the frame; fuzz target for the header decoder still passes; harness shows the allocations gone.

### [To Do] X-LOWLAT-0047 Topic name and id interning from the metadata image (`Arc<str>`) for partition lookups without allocation
P1 · M18 · Task · deps: X-LOWLAT-0040, META-F09
Done when: lookup by name allocates nothing (harness); interned names released when the topic is deleted.

### [To Do] X-LOWLAT-0048 Allocation-free observability on hot paths: pre-resolved metric handles per partition and API, span fields without `format!`, disabled events cost-free
P1 · M18 · Task · deps: X-LOWLAT-0040, OBS-F04 · labels: observability
Done when: harness shows zero allocations from metrics and tracing at default log level.

### [To Do] X-LOWLAT-0049 Allocation-free produce steady state for small batches
P1 · M18 · Task · deps: X-LOWLAT-0042, X-LOWLAT-0043, X-LOWLAT-0044, X-LOWLAT-0046, X-LOWLAT-0047, X-LOWLAT-0048 · labels: perf
Done when: harness asserts zero allocations per produce request after warmup in the benchmark workload.

### [To Do] X-LOWLAT-0050 Allocation-free fetch steady state for caught-up consumers and followers
P1 · M18 · Task · deps: X-LOWLAT-0042, X-LOWLAT-0043, X-LOWLAT-0045, X-LOWLAT-0046, X-LOWLAT-0047, X-LOWLAT-0048 · labels: perf
Done when: harness asserts zero allocations per fetch request after warmup.

### [To Do] X-LOWLAT-0051 Pool exhaustion: fall back to allocation counted in `alluvium_buffer_pool_miss_total{pool}`, never unbounded pool growth
P1 · M18 · Task · deps: X-LOWLAT-0041
Done when: test drives exhaustion and asserts the counter and the memory bound.

### [To Do] X-LOWLAT-0052 Allocation regression test in CI: steady-state produce and fetch assert zero allocations per request
P1 · M18 · Task · deps: X-LOWLAT-0049, X-LOWLAT-0050 · labels: test
Done when: test fails with the allocation site report when a change adds an allocation.

### [To Do] X-LOWLAT-0053 Benchmark: p99.9 and throughput with pools enabled versus disabled
P1 · M18 · Task · deps: X-LOWLAT-0049, X-LOWLAT-0050, TEST-F11 · labels: perf
Done when: results published; improvement demonstrated or the pool removed per the more-is-not-better rule.

### [To Do] X-LOWLAT-0054 Docs: buffer pool sizing keys and memory accounting
P2 · M18 · Task · deps: X-LOWLAT-0051 · labels: docs
Done when: each pool key documented with its memory impact.

## [To Do] X-LOWLAT-F05 Segment preallocation and warmup
P1 · M18 · Feature · deps: STOR-F03, X-LOWLAT-F01
Scope: Preallocated segments and index warmup.

### [To Do] X-LOWLAT-0055 Ahead-of-time next segment: prepare the next segment and index files (`.prepared` suffix) in the background when the active segment passes `alluvium.log.segment.prepare.threshold.ratio`
P1 · M18 · Task · deps: X-LOWLAT-0001, CFG-F11
Done when: prepared files exist before roll in a test; key default off; at most one prepared segment per partition.

### [To Do] X-LOWLAT-0056 Roll uses the prepared segment by renaming it to the final base-offset name instead of creating files on the append path
P1 · M18 · Task · deps: X-LOWLAT-0055, STOR-F07
Done when: roll performs no file creation when a prepared segment exists (counting fs test); falls back to normal creation otherwise.

### [To Do] X-LOWLAT-0057 Recovery: prepared files are ignored and deleted on startup; docs/formats/ updated with the `.prepared` state
P1 · M18 · Task · deps: X-LOWLAT-0055, STOR-F09, STOR-F19
Done when: recovery test with leftover prepared files; format docs updated.

### [To Do] X-LOWLAT-0058 Preallocate prepared segments to `segment.bytes` with fallocate, keeping the `log.preallocate` semantics for trimming at roll
P1 · M18 · Task · deps: X-LOWLAT-0055
Done when: prepared files have the expected allocated size; the roll trims to the real size as with `log.preallocate`.

### [To Do] X-LOWLAT-0059 Index warmup: on becoming leader and at startup, load the tail of offset and time indexes for active segments into memory
P1 · M18 · Task · deps: X-LOWLAT-0001, STOR-F04, STOR-F05
Done when: first fetch after leader change performs no index read from disk (counting fs test).

### [To Do] X-LOWLAT-0060 Tail warmup: read the last `alluvium.log.warmup.tail.bytes` of the active segment into page cache (posix_fadvise WILLNEED or read) on leadership change
P1 · M18 · Task · deps: X-LOWLAT-0059
Done when: warmup issued in the background; test with the counting fs shows bounded reads.

### [To Do] X-LOWLAT-0061 Warmup bounds: total warmup bytes and concurrent warmups capped; warmup never delays becoming leader
P1 · M18 · Task · deps: X-LOWLAT-0060
Done when: becoming leader completes before warmup in a test; concurrency limit respected.

### [To Do] X-LOWLAT-0062 Crash tests: crash during prepare and during the rename at roll, with recovery invariants
P1 · M18 · Task · deps: X-LOWLAT-0056, X-LOWLAT-0057, TEST-F03 · labels: test
Done when: crash at every write point of prepare and roll recovers with no lost or duplicated segment.

### [To Do] X-LOWLAT-0063 Metrics: `alluvium_log_roll_seconds`, `alluvium_log_prepared_segment_total{result}`, `alluvium_log_warmup_bytes_total`, `alluvium_log_warmup_seconds`
P1 · M18 · Task · deps: X-LOWLAT-0056, X-LOWLAT-0061 · labels: observability
Done when: values asserted in tests; rows in docs/metrics.md.

### [To Do] X-LOWLAT-0064 Benchmark: p99.99 produce latency across segment rolls and first-fetch latency after leader change, with and without each technique
P1 · M18 · Task · deps: X-LOWLAT-0056, X-LOWLAT-0060, TEST-F11 · labels: perf
Done when: results published; each technique kept only if it shows a measured improvement.

### [To Do] X-LOWLAT-0065 Docs: preallocation and warmup keys and when to use them
P2 · M18 · Task · deps: X-LOWLAT-0064 · labels: docs
Done when: page includes the benchmark numbers.

## [To Do] X-LOWLAT-F06 CPU and NUMA pinning
P2 · M18 · Feature · deps: X-LOWLAT-F01
Scope: Pinning and busy-poll options.

### [To Do] X-LOWLAT-0066 CPU and NUMA topology discovery from Linux sysfs, with fixture tests; other platforms report unsupported
P2 · M18 · Task · deps: none
Done when: parser handles offline CPUs and missing NUMA nodes from fixtures; malformed files return typed errors.

### [To Do] X-LOWLAT-0067 Dependency evaluation for affinity and memory-policy system calls (for example `core_affinity` or `nix`) with no new unsafe in our crates
P2 · M18 · Spike · deps: none · labels: security · ref: docs/standards/security.md
Done when: justification recorded per the dependency policy; `cargo deny check` passes.

### [To Do] X-LOWLAT-0068 Pinning option: Tokio worker threads pinned to `alluvium.runtime.worker.cpus`
P2 · M18 · Task · deps: X-LOWLAT-0066, X-LOWLAT-0067 · labels: config
Done when: workers report their CPU on `/debug/runtime`; test on Linux CI asserts the affinity mask.

### [To Do] X-LOWLAT-0069 Pinning option: storage I/O threads and rings pinned to `alluvium.storage.io.cpus`
P2 · M18 · Task · deps: X-LOWLAT-0066, X-LOWLAT-0067 · labels: config
Done when: affinity asserted in a Linux test.

### [To Do] X-LOWLAT-0070 Pinning option: background work (log cleaner, retention, checkpoints) pinned to `alluvium.runtime.background.cpus` to keep it off latency-critical cores
P2 · M18 · Task · deps: X-LOWLAT-0066, X-LOWLAT-0067 · labels: config
Done when: affinity asserted in a Linux test; background throughput unchanged within noise.

### [To Do] X-LOWLAT-0071 Pinning option: NUMA-local placement of workers, I/O threads and their buffer pools on `alluvium.runtime.numa.node`
P2 · M18 · Task · deps: X-LOWLAT-0068, X-LOWLAT-0069 · labels: config
Done when: memory policy applied through the chosen dependency; numactl alternative documented; test on a two-node CI host or a documented manual check.

### [To Do] X-LOWLAT-0072 Busy-poll option: `SO_BUSY_POLL` from `alluvium.network.socket.busy.poll.us` on accepted sockets
P2 · M18 · Task · deps: X-LOWLAT-0067, NET-F12 · labels: config
Done when: socket option asserted in a Linux test; requires the documented capability and fails clearly without it.

### [To Do] X-LOWLAT-0073 Busy-poll option: io_uring SQPOLL via `alluvium.storage.io.uring.sqpoll` with `alluvium.storage.io.uring.sqpoll.idle.ms`
P2 · M18 · Task · deps: X-LOWLAT-0067, X-LOWLAT-F02 · labels: config
Done when: SQPOLL ring created when enabled; fallback with reason when not permitted.

### [To Do] X-LOWLAT-0074 Validation: CPUs exist and are online, sets are disjoint, counts match thread counts; startup refused with clear errors
P2 · M18 · Task · deps: X-LOWLAT-0068, X-LOWLAT-0069, X-LOWLAT-0070
Done when: test per invalid configuration with the error message asserted.

### [To Do] X-LOWLAT-0075 `/debug/runtime`: thread-to-CPU mapping, NUMA node and busy-poll settings
P2 · M18 · Task · deps: X-LOWLAT-0068 · labels: observability
Done when: endpoint output asserted in a test.

### [To Do] X-LOWLAT-0076 Benchmark on a two-socket host: p99.9 with and without pinning and busy-poll options
P2 · M18 · Task · deps: X-LOWLAT-0074, TEST-F11 · labels: perf
Done when: results published; options without measured benefit documented as such.

### [To Do] X-LOWLAT-0077 Docs: pinning and busy-poll options with OS prerequisites (isolcpus, nohz_full, capabilities)
P2 · M18 · Task · deps: X-LOWLAT-0076 · labels: docs
Done when: each key documented with its prerequisite and measured effect.

## [To Do] X-LOWLAT-F07 Latency profile
P1 · M18 · Feature · deps: X-LOWLAT-F01
Scope: Low-latency configuration profile.

### [To Do] X-LOWLAT-0078 Config profile mechanism: `alluvium.profile` (`none` default) applies documented defaults only for keys not set explicitly; the profile appears as the value source in check-config and /debug/config
P1 · M18 · Task · deps: CFG-F03 · labels: config
Done when: describeConfigs reports profile values with an existing Kafka config source (no new wire enum), as decided in the ADR; tests for precedence over defaults and under explicit values.

### [To Do] X-LOWLAT-0079 Low-latency profile contents: io backend `auto`, lower `replica.fetch.wait.max.ms`, `replica.fetch.min.bytes=1`, segment prepare and warmup on, pool sizes; each value with rationale and benchmark reference
P1 · M18 · Task · deps: X-LOWLAT-0078
Done when: profile table generated from code into docs; a test fails if a profile key is undocumented.

### [To Do] X-LOWLAT-0080 Conflict warnings: explicit values that defeat the profile logged at startup with event `profile_override`
P2 · M18 · Task · deps: X-LOWLAT-0079
Done when: test with an explicit large `replica.fetch.wait.max.ms` emits the event with key and values.

### [To Do] X-LOWLAT-0081 Profile change rules: the profile is static (restart required); a dynamic attempt is rejected with INVALID_CONFIG
P1 · M18 · Task · deps: X-LOWLAT-0078
Done when: test with IncrementalAlterConfigs.

### [To Do] X-LOWLAT-0082 Off by default: with default config `alluvium.profile=none`, `alluvium.storage.io.backend=pool`, `alluvium.log.flush.mode=kafka`, no pinning and no prepare or warmup
P1 · M18 · Task · deps: X-LOWLAT-0079 · labels: test
Done when: effective config of a default node equals the parity defaults snapshot; a counting test shows no io_uring probe and no prepared files.

### [To Do] X-LOWLAT-0083 Client recommendations for low latency: producer `linger.ms`, `acks`, `max.in.flight.requests.per.connection`, consumer `fetch.min.bytes`, `fetch.max.wait.ms`
P2 · M18 · Task · deps: X-LOWLAT-0079 · labels: docs
Done when: doc table with the measured effect of each setting from the benchmark suite.

## [To Do] X-LOWLAT-F08 Latency benchmark suite
P0 · M18 · Feature · deps: TEST-F11, X-LOWLAT-F01
Scope: Coordinated-omission-safe load generation; p99.9 and p99.99.

### [To Do] X-LOWLAT-0084 Open-loop load generator `alluvium-latency-bench` (dev crate): fixed arrival schedule, latency from intended send time (coordinated-omission safe), HdrHistogram recording
P0 · M18 · Task · deps: X-LOWLAT-0001 · labels: perf, test
Done when: generator self-test with a delayed fake broker shows latency including queueing delay; Kafka-protocol client pinned and used for both Alluvium and Kafka.

### [To Do] X-LOWLAT-0085 Benchmark environment spec: instance types, kernel, filesystem and mount options, NIC settings; machine metadata captured with every result
P0 · M18 · Task · deps: X-LOWLAT-0084
Done when: result files include the spec hash; runs on a different spec are not compared.

### [To Do] X-LOWLAT-0086 Result format and report: JSON with p50, p90, p99, p99.9, p99.99 and max, latency over time, and a Markdown report
P0 · M18 · Task · deps: X-LOWLAT-0084
Done when: golden-file test of the report renderer; JSON schema versioned.

### [To Do] X-LOWLAT-0087 Kafka 4.4.0 comparison runner: the same scenarios against Kafka with equivalent documented config
P0 · M18 · Task · deps: X-LOWLAT-0084, TEST-F09
Done when: one command runs a scenario against both systems and produces a side-by-side report.

### [To Do] X-LOWLAT-0088 Scenario: produce acks=1, 1 KiB records, 50% of saturation throughput, 100 partitions
P0 · M18 · Task · deps: X-LOWLAT-0085, X-LOWLAT-0086
Done when: scenario definition checked in; repeatable results within the documented noise band.

### [To Do] X-LOWLAT-0089 Scenario: produce acks=all, RF=3, `min.insync.replicas=2`
P0 · M18 · Task · deps: X-LOWLAT-0085, X-LOWLAT-0086
Done when: scenario definition checked in with three-node topology.

### [To Do] X-LOWLAT-0090 Scenario: end-to-end produce-to-consume latency using generator timestamps in the payload
P0 · M18 · Task · deps: X-LOWLAT-0085, X-LOWLAT-0086
Done when: consumer-side histogram corrected for coordinated omission.

### [To Do] X-LOWLAT-0091 Scenario: tail fetch latency for caught-up consumers while another consumer replays from the beginning (page-cache pressure isolation)
P0 · M18 · Task · deps: X-LOWLAT-0085, X-LOWLAT-0086
Done when: replay throughput and tail latency reported together.

### [To Do] X-LOWLAT-0092 Scenario: 10k partitions at low per-partition rate
P1 · M18 · Task · deps: X-LOWLAT-0085, X-LOWLAT-0086
Done when: scenario reports latency and broker CPU.

### [To Do] X-LOWLAT-0093 Scenario: strict durability with each group commit policy versus per-append fsync
P0 · M18 · Task · deps: X-LOWLAT-0085, X-LOWLAT-0086, X-LOWLAT-F03
Done when: runs every policy in one invocation.

### [To Do] X-LOWLAT-0094 Scenario: compacted topic with the cleaner running
P1 · M18 · Task · deps: X-LOWLAT-0085, X-LOWLAT-0086
Done when: cleaner throughput and produce tail latency reported.

### [To Do] X-LOWLAT-0095 Scenario: TLS listener with the same load as the acks=1 scenario
P1 · M18 · Task · deps: X-LOWLAT-0085, X-LOWLAT-0086
Done when: TLS handshake excluded from steady-state measurement and reported separately.

### [To Do] X-LOWLAT-0096 Scenario: latency during leader failover and rolling restart (maximum latency and recovery time)
P0 · M18 · Task · deps: X-LOWLAT-0085, X-LOWLAT-0086
Done when: fault timing scripted; recovery time to baseline p99 reported.

### [To Do] X-LOWLAT-0097 Scenario: latency-throughput curve up to saturation
P1 · M18 · Task · deps: X-LOWLAT-0085, X-LOWLAT-0086
Done when: curve plotted in the report; saturation point recorded.

### [To Do] X-LOWLAT-0098 Docs: running the latency suite locally and in CI, and reading the reports
P1 · M18 · Task · deps: X-LOWLAT-0088, X-LOWLAT-0090 · labels: docs
Done when: page explains coordinated omission and why the generator is open-loop.

## [To Do] X-LOWLAT-F09 Latency regression gates
P0 · M18 · Feature · deps: X-LOWLAT-F08
Scope: CI gates on tail latency.

### [To Do] X-LOWLAT-0099 Gate framework: per-scenario thresholds relative to stored baselines (`bench/latency/baselines.json`), repeated runs and noise-aware comparison
P0 · M18 · Task · deps: none · labels: test
Done when: comparison uses the median of N runs with a bootstrap confidence interval; unit tests with synthetic results for pass, fail and inconclusive.

### [To Do] X-LOWLAT-0100 Baseline management: updates only through a PR containing before and after reports, tied to the environment spec
P0 · M18 · Task · deps: X-LOWLAT-0099
Done when: CI rejects baseline changes without an attached report; procedure documented.

### [To Do] X-LOWLAT-0101 PR smoke gate: short acks=all and end-to-end scenarios on a dedicated runner for changes under storage, network and server crates
P0 · M18 · Task · deps: X-LOWLAT-0099
Done when: gate runs in under 15 minutes; a deliberately slowed build fails it (gate self-test).

### [To Do] X-LOWLAT-0102 Nightly gate across all scenarios with trend history
P0 · M18 · Task · deps: X-LOWLAT-0099
Done when: history stored as CI artifacts; regressions open an issue with the report.

### [To Do] X-LOWLAT-0103 Kafka-relative gate: Alluvium p99.9 at most Kafka 4.4.0 p99.9 per scenario on the same hardware, weekly
P1 · M18 · Task · deps: X-LOWLAT-0102
Done when: weekly job publishes the comparison; failures block the release checklist.

### [To Do] X-LOWLAT-0104 Regression triage artifacts: flame graphs and request phase metric diffs attached to failing gate runs
P1 · M18 · Task · deps: X-LOWLAT-0102
Done when: failing run artifacts include both; triage steps documented.

### [To Do] X-LOWLAT-0105 Compatibility and differential suites pass with the low-latency profile, io_uring backend and group-commit mode enabled
P0 · M18 · Task · deps: X-LOWLAT-F02, X-LOWLAT-F03, X-LOWLAT-F07, TEST-F08, TEST-F09 · labels: test
Done when: linux CI job runs `cargo xtask compat` with these settings; differential diffs unchanged versus defaults.

### [To Do] X-LOWLAT-0106 Docs: latency gate policy, what blocks a merge, how to update baselines
P1 · M18 · Task · deps: X-LOWLAT-0101, X-LOWLAT-0102 · labels: docs
Done when: page linked from CONTRIBUTING guidance for performance changes.

## [To Do] X-LOWLAT-F10 Low-latency documentation
P1 · M18 · Feature · deps: X-LOWLAT-F07
Scope: Tuning guide.

### [To Do] X-LOWLAT-0107 Tuning guide: OS settings (CPU isolation, nohz_full, transparent huge pages, swappiness, IRQ affinity, NIC coalescing), filesystem and mount options
P1 · M18 · Task · deps: none · labels: docs
Done when: each recommendation cites its measured effect from the benchmark suite or is marked as advisory.

### [To Do] X-LOWLAT-0108 Tuning guide: Alluvium settings (profile, io backend, group commit, pools, preallocation, pinning) with expected effects
P1 · M18 · Task · deps: X-LOWLAT-0107 · labels: docs
Done when: every X-LOWLAT key appears with its default and its effect.

### [To Do] X-LOWLAT-0109 Client tuning for low latency with the Java client and librdkafka
P2 · M18 · Task · deps: X-LOWLAT-0108 · labels: docs
Done when: settings table with measured effects.

### [To Do] X-LOWLAT-0110 Migration and opt-in: enabling low-latency mode on an existing cluster with rolling restarts, verification and rollback
P1 · M18 · Task · deps: X-LOWLAT-0108 · labels: docs
Done when: procedure tested on a three-node cluster; rollback restores defaults without data loss.

### [To Do] X-LOWLAT-0111 Troubleshooting tail latency with request phase metrics, flight recorders and the benchmark suite
P1 · M18 · Task · deps: X-LOWLAT-0108 · labels: docs
Done when: walkthrough of a real regression from the nightly gate.

### [To Do] X-LOWLAT-0112 Published benchmark results page generated from the nightly gate
P2 · M18 · Task · deps: X-LOWLAT-0108, X-LOWLAT-F09 · labels: docs
Done when: page regenerated by CI; shows Alluvium and Kafka side by side.
