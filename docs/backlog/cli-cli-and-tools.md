# CLI: CLI and tool compatibility

> Crates: `alluvium` binary (layer 5) · Kafka counterpart: `bin/kafka-*.sh`, `tools/` · Milestones: M2-M9

The `alluvium` command line, and compatibility of every Kafka command-line tool against Alluvium.

## [To Do] CLI-F01 CLI framework
P0 · M2 · Feature · deps: FND-F01
Scope: Argument parsing, subcommands, output module (the only stdout writer), exit codes, `--json`.

### [To Do] CLI-0001 Create the `alluvium` binary crate (layer 5) with the clap command tree
P0 · M2 · Task · deps: none · ref: docs/adr/0009-crate-layering-and-dependency-rules.md
Done when: the crate has a `//!` doc and is registered as layer 5; `clap` (derive) is added with its dependency justification; `alluvium --help` lists `server`, `format`, `storage`, `dump`, `verify`, `check-config` and `debug-bundle`; `--version` prints the Alluvium version, git commit and the Kafka protocol version (4.4.0); a help snapshot test passes.

### [To Do] CLI-0002 Output module: the only writer to stdout
P0 · M2 · Task · deps: CLI-0001
Done when: all command results go through `Output` (human or JSON mode); clippy `disallowed-macros`/`disallowed-methods` forbid `print!`, `println!` and `std::io::stdout` in the crate outside `output.rs` (a fixture proves the lint fires); diagnostics and logs go to stderr.

### [To Do] CLI-0003 `--json`: versioned JSON documents and JSON lines for streaming commands
P0 · M2 · Task · deps: CLI-0002
Done when: every command result type serializes to a document carrying `"schema": "alluvium.<command>/v1"`; streaming commands (dumps) emit one JSON object per line; committed JSON Schema files are compared in a test so a breaking change requires a version bump; snapshot tests per command.

### [To Do] CLI-0004 Exit codes: one documented enum
P0 · M2 · Task · deps: CLI-0001
Done when: `ExitCode` defines 0 success, 1 runtime failure, 2 usage error, 3 verification found problems, 4 invalid configuration or storage state; every command maps its errors through one exhaustive match; a test runs each command into each reachable code; docs/cli.md has the table.

### [To Do] CLI-0005 Error reporting: cause chains on stderr, error documents with `--json`
P0 · M2 · Task · deps: CLI-0002, CLI-0004
Done when: errors print `error: <message>` and `caused by:` lines to stderr; with `--json` an `{ "error": { "kind", "message", "causes" } }` document goes to stdout; backtraces only with `RUST_BACKTRACE`; `anyhow` is used only in this binary; tests.

### [To Do] CLI-0006 Logging for CLI commands: stderr, `-v`/`-q` and the OBS-F01 formats
P1 · M2 · Task · deps: CLI-0001, OBS-F01
Done when: offline commands install the OBS-F01 subscriber writing pretty output to stderr, with `-v`, `-vv` and `-q` mapping to debug, trace and warn filters and `ALLUVIUM_LOG` still winning; `server` uses the configured format; a test shows dump output on stdout stays clean while logging.

### [To Do] CLI-0007 Runtime construction only in `main`, only for commands that need it
P0 · M2 · Task · deps: CLI-0001, FND-F04 · ref: docs/architecture.md
Done when: the Tokio runtime is built in exactly one function in `main.rs` for `server` and online `debug-bundle`; offline commands run synchronously; an xtask check fails if `tokio::runtime::Builder` appears anywhere else in the workspace.

### [To Do] CLI-0008 CLI integration test harness: run the built binary in temporary directories
P1 · M2 · Task · deps: CLI-0001 · labels: test
Done when: a test helper runs `alluvium` with arguments, environment and a temporary working directory and asserts stdout, stderr and exit code, with snapshot support (dependency justified); help, version and error-path tests use it.

### [To Do] CLI-0009 docs/cli.md: commands, global flags, exit codes and JSON policy
P1 · M2 · Task · deps: CLI-0003, CLI-0004 · labels: docs
Done when: the page documents every subcommand and global flag, the exit code table and the JSON schema versioning rule; a test fails when a clap subcommand is missing from the page.

## [To Do] CLI-F02 `server` command
P0 · M2 · Feature · deps: CLI-F01, SRV-F01
Scope: Start a node from config, with clear startup validation messages.

### [To Do] CLI-0010 `alluvium server <config>`: load config, assemble the node and run until shutdown
P0 · M2 · Task · deps: CFG-F02, SRV-F03 · ref: core/src/main/scala/kafka/Kafka.scala @4.4.0
Done when: the command loads the file through CFG-F02, maps observability settings onto OBS-F01, assembles the node through SRV-F01 and blocks until SRV-F03's shutdown completes; exit 0 after a SIGTERM-initiated clean shutdown, 1 after a fatal error; an integration test starts a loopback single node in temporary directories and stops it with SIGTERM.

### [To Do] CLI-0011 `--override key=value` with `kafka.Kafka` semantics
P1 · M2 · Task · deps: CLI-0010 · labels: security · ref: core/src/main/scala/kafka/Kafka.scala getPropsFromArgs @4.4.0
Done when: repeatable overrides apply on top of the file as Kafka's getPropsFromArgs does and appear with their source in check-config; malformed overrides are usage errors (exit 2); password-type keys are refused on the command line with a hint to use the file, an environment variable or a config provider (secrets never in arguments); tests.

### [To Do] CLI-0012 Startup validation: report every configuration error before binding anything
P0 · M2 · Task · deps: CLI-0010, CFG-F11 · ref: docs/adr/0006-secure-defaults.md
Done when: all validation errors are collected and printed together with key, value source, reason and a fix hint, then the process exits 4 before any listener binds or file is written; tests cover missing `process.roles` and `node.id`, missing `log.dirs`, a controller listener absent from `listeners`, and plaintext on a non-loopback address without `alluvium.allow.plaintext.external` (message names the key and ADR-0006).

### [To Do] CLI-0013 Refuse to start on unformatted or mismatched storage, with a `format` hint
P0 · M2 · Task · deps: CLI-0010, SRV-F11
Done when: a missing `meta.properties`, a cluster id or node id mismatch, or duplicate directory ids stop startup with exit 4 and a message naming the directory and pointing to `alluvium format`; tests per case.

### [To Do] CLI-0014 Startup and shutdown events from the binary
P1 · M2 · Task · deps: CLI-0010, OBS-F07
Done when: `node_starting` (version, commit, node id, roles, config path, number of non-default keys, never values), `node_started` (elapsed_ms) and `node_stopped` (exit reason) are declared in the binary's events.rs and emitted; a captured-log test asserts them.

### [To Do] CLI-0015 Process setup order: subscriber, panic hook, runtime, node
P0 · M2 · Task · deps: CLI-0010, SRV-F16
Done when: `main` installs the OBS-F01 subscriber, then the SRV-F16 panic hook, then builds the runtime (worker threads from config), then starts the node; the release profile uses `panic = "abort"` (checked by a test reading the build profile); a forced early panic in a child process produces the SRV-F16 crash report.

## [To Do] CLI-F03 `format` command
P0 · M2 · Feature · deps: CLI-F01, SRV-F12
Scope: Format storage directories (cluster id, bootstrap).

### [To Do] CLI-0016 `alluvium format`: kafka-storage.sh format flags mapped onto SRV-F12
P0 · M2 · Task · deps: none · ref: core/src/main/scala/kafka/tools/StorageTool.scala @4.4.0
Done when: `--config`, `--cluster-id`, `--release-version`, `--feature name=level` (repeatable), `--ignore-formatted`, `--standalone`, `--initial-controllers` and `--no-initial-controllers` behave as in StorageTool; human output lists each formatted directory with its metadata.version; JSON output lists directories, directory ids and the cluster id; tests.

### [To Do] CLI-0017 Format flag validation with Kafka's rules and messages
P0 · M2 · Task · deps: CLI-0016 · ref: core/src/main/scala/kafka/tools/StorageTool.scala, metadata/src/main/java/org/apache/kafka/metadata/storage/Formatter.java @4.4.0
Done when: mutually exclusive flags, the controller rules for static (`controller.quorum.voters`) and dynamic (`controller.quorum.bootstrap.servers`) quorums, cluster ids that are not 16-byte base64 UUIDs, and `--release-version` combined with a metadata.version `--feature` all fail with StorageTool's wording and exit 2; one test per rule.

### [To Do] CLI-0018 Release version and feature level resolution
P0 · M2 · Task · deps: CLI-0016, META-F06
Done when: `--release-version` maps to metadata.version and META-F06's default feature levels; explicit `--feature` values override defaults and are validated against dependencies; unstable levels require the unstable-versions setting; errors list the supported range; tests across releases 3.3 to 4.4.

### [To Do] CLI-0019 `--initial-controllers` parsing: `id@host:port:directory-id` lists
P0 · M2 · Task · deps: CLI-0016, RAFT-F15 · ref: raft/src/main/java/org/apache/kafka/raft/DynamicVoters.java @4.4.0
Done when: the list parses as in DynamicVoters (unique ids, valid host and port, base64 directory ids); this node's entry must match `node.id` and supplies its directory id; errors name the offending entry; tests.

### [To Do] CLI-0020 Re-format safety: already formatted and mixed directories
P0 · M2 · Task · deps: CLI-0016
Done when: formatted directories cause an error listing them unless `--ignore-formatted`, which skips them; directories with different cluster ids are always an error; a partially formatted set formats only the rest; tests.

### [To Do] CLI-0021 SCRAM bootstrap credentials from a file instead of command-line passwords
P1 · M4 · Task · deps: CLI-0016, SEC-F04 · labels: security · ref: metadata/src/main/java/org/apache/kafka/metadata/storage/ScramParser.java @4.4.0
Done when: `--add-scram-file <path>` reads entries in Kafka's `SCRAM-SHA-256=[name=alice,password=...]` syntax (and the salt, saltedpassword and iterations forms) per ScramParser; inline `--add-scram` values containing `password=` are refused with an explanation, recorded as an ADR-0006 deviation in docs/parity.md; tests.

### [To Do] CLI-0022 `alluvium storage info`: meta.properties of every log directory
P1 · M2 · Task · deps: CLI-0004 · ref: core/src/main/scala/kafka/tools/StorageTool.scala @4.4.0
Done when: like `kafka-storage.sh info`, the command prints each directory's meta.properties (version, cluster id, node id, directory id) and flags missing, unformatted or inconsistent directories, exiting 3 when it finds problems; JSON output; tests.

### [To Do] CLI-0023 `alluvium storage random-uuid`
P2 · M2 · Task · deps: CLI-0001, TYPE-F03 · ref: core/src/main/scala/kafka/tools/StorageTool.scala @4.4.0
Done when: prints a new Kafka-compatible base64 UUID that is never all zeros or reserved (TYPE-F03 rules); a test checks the format over many runs.

### [To Do] CLI-0024 `alluvium storage version-mapping` and `feature-dependencies`
P1 · M2 · Task · deps: CLI-0003, META-F06 · ref: core/src/main/scala/kafka/tools/StorageTool.scala @4.4.0
Done when: output for each `--release-version` and `--feature` equals committed goldens captured from Kafka 4.4.0's `kafka-storage.sh version-mapping` and `feature-dependencies`; JSON output; tests.

### [To Do] CLI-0025 Document `format` and `storage` with the kafka-storage.sh mapping
P2 · M2 · Task · deps: CLI-0016, CLI-0022, CLI-0024 · labels: docs
Done when: docs/cli.md maps every kafka-storage.sh subcommand and flag to its Alluvium equivalent, including the SCRAM-file deviation, with examples for standalone and initial-controllers bootstraps.

## [To Do] CLI-F04 Offline dump and verify tools
P0 · M2 · Feature · deps: CLI-F01, STOR-F09, META-F05
Scope: Dump log, offset index, time index, transaction index, producer state, checkpoints, metadata log and snapshots, quorum state; verify log.

### [To Do] CLI-0026 `alluvium dump` command group: file type detection and shared options
P0 · M2 · Task · deps: none · ref: tools/src/main/java/org/apache/kafka/tools/DumpLogSegments.java @4.4.0
Done when: subcommands `log`, `index`, `time-index`, `txn-index`, `producer-state`, `checkpoint`, `meta-properties`, `metadata` and `quorum-state` exist; types are detected from file names (`.log`, `.index`, `.timeindex`, `.txnindex`, `.snapshot`, checkpoint names, `quorum-state`) with a `--type` override; unknown types are usage errors; `--max-bytes` bounds how much is read; output streams; tests.

### [To Do] CLI-0027 `dump log <segment>`: batch-level output
P0 · M2 · Task · deps: CLI-0026, STOR-F03 · ref: tools/src/main/java/org/apache/kafka/tools/DumpLogSegments.java @4.4.0
Done when: each batch prints baseOffset, lastOffset, count, baseSequence, lastSequence, producerId, producerEpoch, partitionLeaderEpoch, isTransactional, isControl, deleteHorizonMs, position, timestamp type and value, size, magic, compression codec, crc and isValid, as DumpLogSegments does; an invalid batch is reported with its position and dumping stops; golden tests on fixture segments.

### [To Do] CLI-0028 `dump log --records` and `--payload`
P0 · M2 · Task · deps: CLI-0027, REC-F11 · labels: security · ref: tools/src/main/java/org/apache/kafka/tools/DumpLogSegments.java @4.4.0
Done when: record lines show offset, timestamp, keySize, valueSize, sequence and header keys; keys and values print only with `--payload` (escaped UTF-8 or `--payload-format hex`); control records decode as EndTransactionMarker (type, coordinator epoch), LeaderChangeMessage, SnapshotHeader/Footer, KRaftVersionRecord and VotersRecord; golden tests.

### [To Do] CLI-0029 `dump log --decode metadata` for `__cluster_metadata` segments
P0 · M2 · Task · deps: CLI-0028, META-F01 · labels: security · ref: tools/src/main/java/org/apache/kafka/tools/DumpLogSegments.java --cluster-metadata-decoder @4.4.0
Done when: records decode through META-F01 into named records with versions; SCRAM keys and salts, token ids and sensitive config values are always redacted; golden test on a fixture metadata segment.

### [To Do] CLI-0030 `dump log --decode offsets` for `__consumer_offsets`
P1 · M5 · Task · deps: CLI-0028, GRP-F02 · ref: tools/src/main/java/org/apache/kafka/tools/DumpLogSegments.java --offsets-decoder @4.4.0
Done when: offset commit, group metadata and consumer-group (KIP-848) record keys and values decode by type and version; member metadata and assignment bytes print as `PayloadSummary` unless `--payload`; golden test.

### [To Do] CLI-0031 `dump log --decode transactions` for `__transaction_state`
P1 · M6 · Task · deps: CLI-0028, TXN-F03 · ref: tools/src/main/java/org/apache/kafka/tools/DumpLogSegments.java --transaction-log-decoder @4.4.0
Done when: TransactionLogKey and TransactionLogValue decode with state, producer id and epoch, timeout and partitions; golden test.

### [To Do] CLI-0032 `dump log --decode share-state` for `__share_group_state`
P1 · M8 · Task · deps: CLI-0028, SHARE-F05 · ref: tools/src/main/java/org/apache/kafka/tools/DumpLogSegments.java --share-group-state-decoder @4.4.0
Done when: share snapshot and update records decode with group, topic, partition, start offset and state batches; golden test.

### [To Do] CLI-0033 `dump log --decode remote-log-metadata` for `__remote_log_metadata`
P2 · M9 · Task · deps: CLI-0028, TIER-F02 · ref: tools/src/main/java/org/apache/kafka/tools/DumpLogSegments.java --remote-log-metadata-decoder @4.4.0
Done when: remote log segment metadata, update and partition delete records decode with segment ids, offsets and states; golden test.

### [To Do] CLI-0034 `dump index <file>`: offset index entries and sanity check
P0 · M2 · Task · deps: CLI-0026, STOR-F04 · ref: tools/src/main/java/org/apache/kafka/tools/DumpLogSegments.java @4.4.0
Done when: entries print as absolute offset and position; `--verify` applies the `--index-sanity-check` rules (monotonic entries, size a multiple of the entry size, within maximum size); tests on valid and corrupt fixtures.

### [To Do] CLI-0035 `dump time-index <file>`: timestamp index entries
P0 · M2 · Task · deps: CLI-0026, STOR-F05 · ref: tools/src/main/java/org/apache/kafka/tools/DumpLogSegments.java @4.4.0
Done when: entries print as timestamp and offset; `--verify` checks non-decreasing timestamps and sizes; tests.

### [To Do] CLI-0036 `dump txn-index <file>`: aborted transaction entries
P1 · M6 · Task · deps: CLI-0026, STOR-F06 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/TransactionIndex.java @4.4.0
Done when: entries print version, producerId, firstOffset, lastOffset and lastStableOffset; corrupt entries are reported with position; tests.

### [To Do] CLI-0037 `dump producer-state <snapshot>`
P1 · M6 · Task · deps: CLI-0026, STOR-F15 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/ProducerStateManager.java @4.4.0
Done when: entries print producerId, producerEpoch, coordinatorEpoch, lastSequence, lastOffset, offsetDelta, timestamp and currentTxnFirstOffset; the snapshot CRC is verified and a mismatch is an error; tests.

### [To Do] CLI-0038 `dump checkpoint <file>`: offset and leader-epoch checkpoint files
P0 · M2 · Task · deps: CLI-0026, STOR-F10, STOR-F11 · ref: storage/src/main/java/org/apache/kafka/storage/internals/checkpoint/OffsetCheckpointFile.java, storage/src/main/java/org/apache/kafka/storage/internals/checkpoint/LeaderEpochCheckpointFile.java @4.4.0
Done when: recovery-point-offset-checkpoint, log-start-offset-checkpoint, replication-offset-checkpoint, cleaner-offset-checkpoint and leader-epoch-checkpoint print version and entries; malformed files report the line number; tests.

### [To Do] CLI-0039 `dump meta-properties` and `partition.metadata`
P1 · M2 · Task · deps: CLI-0026, SRV-F11 · ref: server-common/src/main/java/org/apache/kafka/metadata/properties/MetaProperties.java, storage/src/main/java/org/apache/kafka/storage/internals/checkpoint/PartitionMetadataFile.java @4.4.0
Done when: meta.properties prints version, cluster.id, node.id and directory.id; partition.metadata prints version and topic id; malformed files are errors with the line; tests.

### [To Do] CLI-0040 `dump metadata <log dir>`: metadata log segments and snapshots
P0 · M2 · Task · deps: CLI-0029, RAFT-F06
Done when: the command walks `__cluster_metadata-0`, printing batches (offsets, epoch, append time) with decoded, redacted records; `--snapshot <file>` prints the header, footer and records of a `.checkpoint`; tests on fixtures.

### [To Do] CLI-0041 `dump metadata --image`: replay to an image and print its tree
P1 · M2 · Task · deps: CLI-0040, META-F04
Done when: the latest snapshot plus log records up to `--offset` replay through META-F04 and print the META-F03 tree with kafka-metadata-shell paths; `--path` selects a subtree; output is redacted; tests.

### [To Do] CLI-0042 `dump quorum-state <file>`
P1 · M2 · Task · deps: CLI-0026, RAFT-F09 · ref: raft/src/main/resources/common/message/QuorumStateData.json, raft/src/main/java/org/apache/kafka/raft/FileQuorumStateStore.java @4.4.0
Done when: fields print for each data version (leaderId, leaderEpoch, votedId, votedDirectoryId, and current voters in version 0); unknown versions are errors; tests.

### [To Do] CLI-0043 `verify log <partition dir>`: CRCs and batch structure
P0 · M2 · Task · deps: CLI-0026, REC-F03 · ref: tools/src/main/java/org/apache/kafka/tools/DumpLogSegments.java @4.4.0
Done when: every batch's CRC32C is checked, along with magic 2, lastOffsetDelta against the record count and length fields; problems list file, position and offset (first 100, then a count) and the command exits 3; a clean directory exits 0; tests with corrupted fixtures.

### [To Do] CLI-0044 `verify log`: offset order across batches and segments
P0 · M2 · Task · deps: CLI-0043
Done when: segment base offsets match file names, batch offsets strictly increase across segments, and gaps are reported unless the topic is compacted (`--compacted`); tests.

### [To Do] CLI-0045 `verify log`: index and log consistency
P0 · M2 · Task · deps: CLI-0044, STOR-F04, STOR-F05 · ref: tools/src/main/java/org/apache/kafka/tools/DumpLogSegments.java @4.4.0
Done when: each offset index entry points at a batch boundary containing that offset, time index entries are non-decreasing and resolve to existing offsets, and index sizes are within limits (the `--verify-index-only` and `--index-sanity-check` checks); mismatches are reported; tests.

### [To Do] CLI-0046 `verify log`: leader-epoch checkpoint against batch leader epochs
P1 · M2 · Task · deps: CLI-0044, STOR-F11
Done when: the leader-epoch-checkpoint start offsets must match the first batch of each partitionLeaderEpoch in the log; mismatches are reported; tests.

### [To Do] CLI-0047 Dump and verify never panic on corrupt input
P0 · M2 · Task · deps: CLI-0028, CLI-0034, CLI-0035, CLI-0038, CLI-0040, CLI-0042, CLI-0045 · labels: security, test
Done when: a property test truncates and flips bytes in fixtures of every file type; every command returns an error naming the file and position with exit 1 or 3 and never panics or allocates beyond `--max-bytes`.

### [To Do] CLI-0048 Bounded memory when dumping large segments
P1 · M2 · Task · deps: CLI-0028 · labels: nfr, perf
Done when: dumping a 1 GiB segment with records stays under 64 MiB resident memory (measured in a test or benchmark), because batches stream one at a time.

### [To Do] CLI-0049 Offline tools guide in docs/guides/debugging.md
P1 · M2 · Task · deps: CLI-0041, CLI-0046 · labels: docs
Done when: the guide covers verifying a partition after a crash, dumping around an offset, inspecting metadata and quorum state, and links each command to its docs/formats page; examples run in a docs test.

## [To Do] CLI-F05 Config check and debug bundle
P1 · M2 · Feature · deps: CLI-F01, CFG-F03
Scope: `check-config` and `debug-bundle`.

### [To Do] CLI-0050 `alluvium check-config <file>`: validate and print effective values with sources
P1 · M2 · Task · deps: CFG-F02
Done when: the command loads and validates the file (all errors at once, exit 4) and prints every key with its effective value and source (default, file, env, override), password values redacted; JSON output; tests.

### [To Do] CLI-0051 check-config warnings: unknown keys, deprecated keys and Alluvium deviations
P1 · M2 · Task · deps: CLI-0050
Done when: unknown keys are warned about like Kafka's "not a known config" log line, deprecated keys name their replacement, and deviation keys (plaintext on external listeners, connection limits) link their ADR; warnings keep exit 0; tests.

### [To Do] CLI-0052 `alluvium debug-bundle`: archive format, manifest and size limit
P1 · M2 · Task · deps: CLI-0004
Done when: the command writes `alluvium-debug-<node>-<timestamp>.tar.gz` (or `--output`) with file mode 0600 and a manifest.json listing each item with size, collection time and any collection error (partial bundles are still written); `--max-size` (default 64 MiB) truncates with a note; tests.

### [To Do] CLI-0053 debug-bundle offline contents: config, versions, host facts and log-dir listing
P1 · M2 · Task · deps: CLI-0052, CLI-0050
Done when: the bundle holds the redacted effective config with sources, version information (version, commit, rustc, target), OS and kernel, CPU count, memory, file-descriptor limits, log-dir listings (names, sizes, mtimes; never file contents), meta.properties and quorum-state; tests.

### [To Do] CLI-0054 debug-bundle online contents from the diagnostics endpoint
P1 · M2 · Task · deps: CLI-0052, OBS-F08
Done when: with `--diagnostics-url` and a token from a file or environment variable the bundle adds `/metrics`, `/debug/tasks`, `/debug/raft`, `/debug/config`, `/debug/log-level` and a bounded set of `/debug/partitions` (listed or the first N under-replicated), each with a timeout; failures are recorded in the manifest; tests against a fake endpoint.

### [To Do] CLI-0055 debug-bundle redaction test
P0 · M2 · Task · deps: CLI-0053, CLI-0054, OBS-F11 · labels: security, test
Done when: with canary secrets in the config file, environment and config-provider files and canary record payloads in segments, the OBS-F11 scanner finds no canary anywhere in the bundle.

### [To Do] CLI-0056 Document check-config and debug-bundle
P2 · M2 · Task · deps: CLI-0051, CLI-0055 · labels: docs
Done when: docs/guides/debugging.md lists exactly what the bundle collects, how to review it before sharing, and how to read check-config output.

## [To Do] CLI-F06 Kafka tool compatibility
P1 · M2 · Feature · deps: TEST-F08
Scope: One task per `kafka-*.sh` tool: compatibility tests for its commands.

### [To Do] CLI-0057 Tool compatibility matrix and per-tool test layout
P1 · M2 · Task · deps: TEST-F09 · labels: test, docs · ref: bin @4.4.0
Done when: docs/compat/tools.md lists all 44 scripts in Kafka 4.4.0's bin/ (kafka-*, connect-*, trogdor) with status (tested, replaced, not applicable), milestone and owning task, and notes the tools removed in Kafka 4.0 (zookeeper-*); per-tool tests live under `tests/compat/tools/` on the TEST-F08 `kafka-*.sh` runner with TEST-F09 normalization; a test fails when a bin/ script has no row.

### [To Do] CLI-0058 `kafka-topics.sh --create` against Alluvium
P0 · M2 · Task · deps: CLI-0057, API-F19, API-F03, API-F18 · labels: test · ref: tools/src/main/java/org/apache/kafka/tools/TopicCommand.java @4.4.0
Done when: creating with `--partitions`/`--replication-factor`, `--replica-assignment`, `--config k=v` and `--if-not-exists` succeeds; TOPIC_ALREADY_EXISTS, INVALID_REPLICATION_FACTOR, INVALID_PARTITIONS, INVALID_CONFIG and invalid topic names produce Kafka's messages. Output and exit code match Kafka 4.4.0 for the same scenario after normalization (TEST-F09 differential).

### [To Do] CLI-0059 `kafka-topics.sh --list` against Alluvium
P0 · M2 · Task · deps: CLI-0057, API-F03 · labels: test · ref: tools/src/main/java/org/apache/kafka/tools/TopicCommand.java @4.4.0
Done when: `--list` with and without `--exclude-internal` and with a `--topic` regex lists the expected topics. Output and exit code match Kafka 4.4.0 for the same scenario after normalization (TEST-F09 differential).

### [To Do] CLI-0060 `kafka-topics.sh --describe` against Alluvium
P0 · M2 · Task · deps: CLI-0057, API-F75, API-F32, API-F03 · labels: test · ref: tools/src/main/java/org/apache/kafka/tools/TopicCommand.java @4.4.0
Done when: `--describe` by name and by `--topic-id`, `--topics-with-overrides` and `--partition-size-limit-per-response` show partitions, leaders, replicas, ISR and configs; the reassignment lookup falls back cleanly while ListPartitionReassignments is not advertised. Output and exit code match Kafka 4.4.0 for the same scenario after normalization (TEST-F09 differential).

### [To Do] CLI-0061 `kafka-topics.sh --describe` health filters on a multi-node cluster
P1 · M3 · Task · deps: CLI-0057, API-F46, API-F75 · labels: test · ref: tools/src/main/java/org/apache/kafka/tools/TopicCommand.java @4.4.0
Done when: `--under-replicated-partitions`, `--under-min-isr-partitions`, `--at-min-isr-partitions` and `--unavailable-partitions` list the right partitions on a 3-node cluster with a stopped broker, and adding/removing replicas show during an active reassignment. Output and exit code match Kafka 4.4.0 for the same scenario after normalization (TEST-F09 differential).

### [To Do] CLI-0062 `kafka-topics.sh --alter --partitions` against Alluvium
P0 · M2 · Task · deps: CLI-0057, API-F37 · labels: test · ref: tools/src/main/java/org/apache/kafka/tools/TopicCommand.java @4.4.0
Done when: increasing partitions (with and without `--replica-assignment`) succeeds, decreasing returns INVALID_PARTITIONS with Kafka's message, and `--if-exists` skips missing topics. Output and exit code match Kafka 4.4.0 for the same scenario after normalization (TEST-F09 differential).

### [To Do] CLI-0063 `kafka-topics.sh --delete` against Alluvium
P0 · M2 · Task · deps: CLI-0057, API-F20 · labels: test · ref: tools/src/main/java/org/apache/kafka/tools/TopicCommand.java @4.4.0
Done when: deleting by name and regex removes topics, `--if-exists` skips missing ones, and internal topics are refused by the tool as with Kafka. Output and exit code match Kafka 4.4.0 for the same scenario after normalization (TEST-F09 differential).

### [To Do] CLI-0064 `kafka-configs.sh --describe --entity-type topics`
P1 · M2 · Task · deps: CLI-0057, API-F32 · labels: test · ref: tools/src/main/java/org/apache/kafka/tools/ConfigCommand.java @4.4.0
Done when: describing one or all topics, with `--all` showing defaults and synonyms, matches Kafka. Output and exit code match Kafka 4.4.0 for the same scenario after normalization (TEST-F09 differential).

### [To Do] CLI-0065 `kafka-configs.sh --alter --entity-type topics`
P1 · M7 · Task · deps: CLI-0057, API-F44 · labels: test · ref: tools/src/main/java/org/apache/kafka/tools/ConfigCommand.java @4.4.0
Done when: `--add-config`, `--delete-config` and `--add-config-file` change topic configs; invalid values return INVALID_CONFIG with Kafka's message. Output and exit code match Kafka 4.4.0 for the same scenario after normalization (TEST-F09 differential).

### [To Do] CLI-0066 `kafka-configs.sh --entity-type brokers` (per broker and cluster default)
P1 · M7 · Task · deps: CLI-0057, API-F44, API-F32, CFG-F04 · labels: test · ref: tools/src/main/java/org/apache/kafka/tools/ConfigCommand.java @4.4.0
Done when: describe and alter of dynamic broker configs by `--entity-name <id>` and `--entity-default`, including listener-prefixed keys and `--all`, match Kafka. Output and exit code match Kafka 4.4.0 for the same scenario after normalization (TEST-F09 differential).

### [To Do] CLI-0067 `kafka-configs.sh --entity-type broker-loggers`
P1 · M7 · Task · deps: CLI-0057, API-F44, API-F32, OBS-F02 · labels: test · ref: tools/src/main/java/org/apache/kafka/tools/ConfigCommand.java @4.4.0
Done when: describing loggers and setting and deleting `kafka.request.logger` and the root logger level follow OBS-F02's mapping; invalid loggers and levels fail with Kafka's errors. Output and exit code match Kafka 4.4.0 for the same scenario after normalization (TEST-F09 differential).

### [To Do] CLI-0068 `kafka-configs.sh` client quotas for users, clients and ips
P1 · M4 · Task · deps: CLI-0057, API-F48, API-F49 · labels: test · ref: tools/src/main/java/org/apache/kafka/tools/ConfigCommand.java @4.4.0
Done when: adding and deleting producer_byte_rate, consumer_byte_rate, request_percentage, controller_mutation_rate and connection_creation_rate for named and default entities, and describing them, match Kafka. Output and exit code match Kafka 4.4.0 for the same scenario after normalization (TEST-F09 differential).

### [To Do] CLI-0069 `kafka-configs.sh` SCRAM credentials for users
P1 · M4 · Task · deps: CLI-0057, API-F50, API-F51, SEC-F04 · labels: test · ref: tools/src/main/java/org/apache/kafka/tools/ConfigCommand.java @4.4.0
Done when: adding SCRAM-SHA-256 and SCRAM-SHA-512 credentials with iterations, deleting a mechanism, and describing users match Kafka, and the new credentials authenticate. Output and exit code match Kafka 4.4.0 for the same scenario after normalization (TEST-F09 differential).

### [To Do] CLI-0070 `kafka-configs.sh --entity-type groups`
P2 · M7 · Task · deps: CLI-0057, API-F44, API-F32, GRP-F05 · labels: test · ref: tools/src/main/java/org/apache/kafka/tools/ConfigCommand.java @4.4.0
Done when: describing and altering group configs such as `consumer.session.timeout.ms` and `consumer.heartbeat.interval.ms` match Kafka, including validation errors. Output and exit code match Kafka 4.4.0 for the same scenario after normalization (TEST-F09 differential).

### [To Do] CLI-0071 `kafka-configs.sh --entity-type client-metrics`
P2 · M8 · Task · deps: CLI-0057, API-F44, API-F32, TELEM-F03 · labels: test · ref: tools/src/main/java/org/apache/kafka/tools/ConfigCommand.java @4.4.0
Done when: describing and altering client-metrics resources (metrics, interval.ms, match) match Kafka. Output and exit code match Kafka 4.4.0 for the same scenario after normalization (TEST-F09 differential).

### [To Do] CLI-0072 `kafka-configs.sh --bootstrap-controller`
P2 · M7 · Task · deps: CLI-0057, API-F44, API-F32 · labels: test · ref: tools/src/main/java/org/apache/kafka/tools/ConfigCommand.java @4.4.0
Done when: describe and alter through the controller listener for brokers, broker defaults and broker-loggers match Kafka. Output and exit code match Kafka 4.4.0 for the same scenario after normalization (TEST-F09 differential).

### [To Do] CLI-0073 `kafka-acls.sh --add` against Alluvium
P0 · M4 · Task · deps: CLI-0057, API-F30 · labels: test · ref: tools/src/main/java/org/apache/kafka/tools/AclCommand.java @4.4.0
Done when: adding allow and deny ACLs with principals, hosts and operations for topic, group, cluster, transactional-id, delegation-token and user resources, literal and prefixed patterns and the `*` wildcard, matches Kafka. Output and exit code match Kafka 4.4.0 for the same scenario after normalization (TEST-F09 differential).

### [To Do] CLI-0074 `kafka-acls.sh --producer` and `--consumer` convenience options
P1 · M4 · Task · deps: CLI-0057, API-F30 · labels: test · ref: tools/src/main/java/org/apache/kafka/tools/AclCommand.java @4.4.0
Done when: `--producer` (with `--idempotent` and `--transactional-id`) and `--consumer --group` create the same ACL sets as Kafka. Output and exit code match Kafka 4.4.0 for the same scenario after normalization (TEST-F09 differential).

### [To Do] CLI-0075 `kafka-acls.sh --list` against Alluvium
P0 · M4 · Task · deps: CLI-0057, API-F29 · labels: test · ref: tools/src/main/java/org/apache/kafka/tools/AclCommand.java @4.4.0
Done when: listing with resource and principal filters and `--resource-pattern-type` any, match, literal and prefixed matches Kafka. Output and exit code match Kafka 4.4.0 for the same scenario after normalization (TEST-F09 differential).

### [To Do] CLI-0076 `kafka-acls.sh --remove` against Alluvium
P0 · M4 · Task · deps: CLI-0057, API-F31 · labels: test · ref: tools/src/main/java/org/apache/kafka/tools/AclCommand.java @4.4.0
Done when: removing with `--force` and filters deletes exactly the ACLs Kafka would. Output and exit code match Kafka 4.4.0 for the same scenario after normalization (TEST-F09 differential).

### [To Do] CLI-0077 `kafka-acls.sh --bootstrap-controller`
P2 · M4 · Task · deps: CLI-0057, API-F29, API-F30, API-F31 · labels: test · ref: tools/src/main/java/org/apache/kafka/tools/AclCommand.java @4.4.0
Done when: add, list and remove through the controller listener match Kafka. Output and exit code match Kafka 4.4.0 for the same scenario after normalization (TEST-F09 differential).

### [To Do] CLI-0078 `kafka-broker-api-versions.sh` against Alluvium
P0 · M2 · Task · deps: CLI-0057, API-F18 · labels: test · ref: tools/src/main/java/org/apache/kafka/tools/BrokerApiVersionsCommand.java @4.4.0
Done when: every API shows the advertised range and usable version, and unadvertised APIs show UNSUPPORTED; the expected output per milestone is committed and equals parity.md's advertised ranges.

### [To Do] CLI-0079 `kafka-client-metrics.sh --alter` and `--delete`
P2 · M8 · Task · deps: CLI-0057, API-F44, TELEM-F03 · labels: test · ref: tools/src/main/java/org/apache/kafka/tools/ClientMetricsCommand.java @4.4.0
Done when: creating with `--name` or `--generate-name`, `--metrics`, `--interval` and `--match`, then deleting, match Kafka. Output and exit code match Kafka 4.4.0 for the same scenario after normalization (TEST-F09 differential).

### [To Do] CLI-0080 `kafka-client-metrics.sh --describe` and `--list`
P2 · M8 · Task · deps: CLI-0057, API-F32, API-F74 · labels: test · ref: tools/src/main/java/org/apache/kafka/tools/ClientMetricsCommand.java @4.4.0
Done when: describe by name and list all client-metrics resources match Kafka. Output and exit code match Kafka 4.4.0 for the same scenario after normalization (TEST-F09 differential).

### [To Do] CLI-0081 `kafka-cluster.sh cluster-id`
P1 · M2 · Task · deps: CLI-0057, API-F60 · labels: test · ref: tools/src/main/java/org/apache/kafka/tools/ClusterTool.java @4.4.0
Done when: prints the formatted cluster id. Output and exit code match Kafka 4.4.0 for the same scenario after normalization (TEST-F09 differential).

### [To Do] CLI-0082 `kafka-cluster.sh api-versions`
P2 · M2 · Task · deps: CLI-0057, API-F18 · labels: test · ref: tools/src/main/java/org/apache/kafka/tools/ClusterTool.java @4.4.0
Done when: prints the same ranges as kafka-broker-api-versions.sh for each node. Output and exit code match Kafka 4.4.0 for the same scenario after normalization (TEST-F09 differential).

### [To Do] CLI-0083 `kafka-cluster.sh list-endpoints`
P1 · M3 · Task · deps: CLI-0057, API-F60, CTRL-F03 · labels: test · ref: tools/src/main/java/org/apache/kafka/tools/ClusterTool.java @4.4.0
Done when: lists broker endpoints, `--include-fenced-brokers` adds fenced ones, and `--bootstrap-controller` lists controller endpoints. Output and exit code match Kafka 4.4.0 for the same scenario after normalization (TEST-F09 differential).

### [To Do] CLI-0084 `kafka-cluster.sh unregister`
P1 · M3 · Task · deps: CLI-0057, API-F64 · labels: test · ref: tools/src/main/java/org/apache/kafka/tools/ClusterTool.java @4.4.0
Done when: unregistering a stopped broker removes it from DescribeCluster; unregistering a live broker fails as in Kafka. Output and exit code match Kafka 4.4.0 for the same scenario after normalization (TEST-F09 differential).

### [To Do] CLI-0085 `kafka-cluster.sh unregister-controller`
P2 · M3 · Task · deps: CLI-0057, API-F94 · labels: test · ref: tools/src/main/java/org/apache/kafka/tools/ClusterTool.java @4.4.0
Done when: unregistering a removed controller succeeds and errors match Kafka. Output and exit code match Kafka 4.4.0 for the same scenario after normalization (TEST-F09 differential).

### [To Do] CLI-0086 `kafka-console-consumer.sh` with `--partition` and `--offset`
P0 · M2 · Task · deps: CLI-0057, API-F01, API-F02, API-F03 · labels: test · ref: tools/src/main/java/org/apache/kafka/tools/consumer/ConsoleConsumer.java @4.4.0
Done when: consuming without a group from earliest, latest and a given offset, with `--max-messages`, `--timeout-ms` and formatter properties (print.key, print.headers, print.timestamp, print.partition, print.offset), prints the same records as Kafka. Output and exit code match Kafka 4.4.0 for the same scenario after normalization (TEST-F09 differential).

### [To Do] CLI-0087 `kafka-console-consumer.sh --group` with both group protocols
P0 · M5 · Task · deps: CLI-0057, API-F11, API-F14, API-F68, API-F08, API-F09, API-F10 · labels: test · ref: tools/src/main/java/org/apache/kafka/tools/consumer/ConsoleConsumer.java @4.4.0
Done when: consuming with `--group` (group.protocol classic and consumer), `--from-beginning` and `--include` regex commits offsets and resumes after restart as with Kafka. Output and exit code match Kafka 4.4.0 for the same scenario after normalization (TEST-F09 differential).

### [To Do] CLI-0088 `kafka-console-consumer.sh --isolation-level read_committed`
P1 · M6 · Task · deps: CLI-0057, API-F01, TXN-F11 · labels: test · ref: tools/src/main/java/org/apache/kafka/tools/consumer/ConsoleConsumer.java @4.4.0
Done when: with a transactional producer committing and aborting, read_committed shows only committed records and read_uncommitted shows all. Output and exit code match Kafka 4.4.0 for the same scenario after normalization (TEST-F09 differential).

### [To Do] CLI-0089 `kafka-console-consumer.sh` formatters on `__consumer_offsets`
P1 · M5 · Task · deps: CLI-0057, GRP-F02 · labels: test · ref: tools/src/main/java/org/apache/kafka/tools/consumer/OffsetsMessageFormatter.java, tools/src/main/java/org/apache/kafka/tools/consumer/GroupMetadataMessageFormatter.java @4.4.0
Done when: OffsetsMessageFormatter and GroupMetadataMessageFormatter print the same records for the same group activity as on Kafka, proving the record formats are compatible. Output and exit code match Kafka 4.4.0 for the same scenario after normalization (TEST-F09 differential).

### [To Do] CLI-0090 `kafka-console-consumer.sh` TransactionLogMessageFormatter on `__transaction_state`
P1 · M6 · Task · deps: CLI-0057, TXN-F03 · labels: test · ref: tools/src/main/java/org/apache/kafka/tools/consumer/TransactionLogMessageFormatter.java @4.4.0
Done when: the formatter prints the same transaction log records for the same transactional workload as on Kafka. Output and exit code match Kafka 4.4.0 for the same scenario after normalization (TEST-F09 differential).

### [To Do] CLI-0091 `kafka-console-consumer.sh` ShareGroupStateMessageFormatter on `__share_group_state`
P2 · M8 · Task · deps: CLI-0057, SHARE-F05 · labels: test · ref: tools/src/main/java/org/apache/kafka/tools/consumer/group/share/ShareGroupStateMessageFormatter.java @4.4.0
Done when: the formatter prints the same share-state records for the same share workload as on Kafka. Output and exit code match Kafka 4.4.0 for the same scenario after normalization (TEST-F09 differential).

### [To Do] CLI-0092 `kafka-console-producer.sh` against Alluvium
P0 · M2 · Task · deps: CLI-0057, API-F00, API-F03 · labels: test · ref: tools/src/main/java/org/apache/kafka/tools/ConsoleProducer.java @4.4.0
Done when: producing with `--request-required-acks` 0, 1 and all, parse.key and key.separator, parse.headers, and each `--compression-codec` (gzip, snappy, lz4, zstd) with idempotence disabled stores records that a consumer reads back identically. Output and exit code match Kafka 4.4.0 for the same scenario after normalization (TEST-F09 differential).

### [To Do] CLI-0093 `kafka-console-producer.sh` with the default idempotent producer
P1 · M6 · Task · deps: CLI-0057, API-F22, API-F00 · labels: test · ref: tools/src/main/java/org/apache/kafka/tools/ConsoleProducer.java @4.4.0
Done when: with Kafka 4.4.0's default `enable.idempotence=true` and `--sync`, producing succeeds and retries do not duplicate records. Output and exit code match Kafka 4.4.0 for the same scenario after normalization (TEST-F09 differential).

### [To Do] CLI-0094 `kafka-console-share-consumer.sh` against Alluvium
P1 · M8 · Task · deps: CLI-0057, API-F76, API-F78, API-F79 · labels: test · ref: tools/src/main/java/org/apache/kafka/tools/consumer/ConsoleShareConsumer.java @4.4.0
Done when: consuming with `--group`, default acceptance, `--release` and `--reject` delivers and redelivers records as on Kafka. Output and exit code match Kafka 4.4.0 for the same scenario after normalization (TEST-F09 differential).

### [To Do] CLI-0095 `kafka-consumer-groups.sh --list`
P0 · M5 · Task · deps: CLI-0057, API-F16 · labels: test · ref: tools/src/main/java/org/apache/kafka/tools/consumer/group/ConsumerGroupCommand.java @4.4.0
Done when: listing with `--state` and `--type` filters shows classic and consumer groups as on Kafka. Output and exit code match Kafka 4.4.0 for the same scenario after normalization (TEST-F09 differential).

### [To Do] CLI-0096 `kafka-consumer-groups.sh --describe --offsets`
P0 · M5 · Task · deps: CLI-0057, API-F15, API-F69, API-F09, API-F02 · labels: test · ref: tools/src/main/java/org/apache/kafka/tools/consumer/group/ConsumerGroupCommand.java @4.4.0
Done when: describing one group and `--all-groups` shows current offset, log end offset and lag per partition. Output and exit code match Kafka 4.4.0 for the same scenario after normalization (TEST-F09 differential).

### [To Do] CLI-0097 `kafka-consumer-groups.sh --describe --members` and `--state`
P1 · M5 · Task · deps: CLI-0057, API-F15, API-F69 · labels: test · ref: tools/src/main/java/org/apache/kafka/tools/consumer/group/ConsumerGroupCommand.java @4.4.0
Done when: members (with `--verbose` assignments) and group state for classic and consumer groups match Kafka. Output and exit code match Kafka 4.4.0 for the same scenario after normalization (TEST-F09 differential).

### [To Do] CLI-0098 `kafka-consumer-groups.sh --reset-offsets` to absolute positions
P0 · M5 · Task · deps: CLI-0057, API-F08, API-F02, API-F09, API-F15 · labels: test · ref: tools/src/main/java/org/apache/kafka/tools/consumer/group/ConsumerGroupCommand.java @4.4.0
Done when: `--to-earliest`, `--to-latest`, `--to-offset`, `--to-current`, `--to-datetime` and `--by-duration` with `--dry-run` and `--execute` for `--all-topics` and `topic:partitions` match Kafka, including the error for an active group. Output and exit code match Kafka 4.4.0 for the same scenario after normalization (TEST-F09 differential).

### [To Do] CLI-0099 `kafka-consumer-groups.sh --reset-offsets` with `--shift-by`, `--from-file` and `--export`
P1 · M5 · Task · deps: CLI-0057, API-F08, API-F02 · labels: test · ref: tools/src/main/java/org/apache/kafka/tools/consumer/group/ConsumerGroupCommand.java @4.4.0
Done when: relative shifts clamp at the log boundaries, CSV plans import and export, as on Kafka. Output and exit code match Kafka 4.4.0 for the same scenario after normalization (TEST-F09 differential).

### [To Do] CLI-0100 `kafka-consumer-groups.sh --delete`
P1 · M5 · Task · deps: CLI-0057, API-F42 · labels: test · ref: tools/src/main/java/org/apache/kafka/tools/consumer/group/ConsumerGroupCommand.java @4.4.0
Done when: deleting empty groups succeeds and non-empty groups fail with GROUP_NOT_EMPTY's message. Output and exit code match Kafka 4.4.0 for the same scenario after normalization (TEST-F09 differential).

### [To Do] CLI-0101 `kafka-consumer-groups.sh --delete-offsets`
P2 · M5 · Task · deps: CLI-0057, API-F47 · labels: test · ref: tools/src/main/java/org/apache/kafka/tools/consumer/group/ConsumerGroupCommand.java @4.4.0
Done when: deleting a topic's committed offsets for an empty group, and the error for a subscribed topic, match Kafka. Output and exit code match Kafka 4.4.0 for the same scenario after normalization (TEST-F09 differential).

### [To Do] CLI-0102 `kafka-consumer-perf-test.sh` against Alluvium
P2 · M5 · Task · deps: CLI-0057, API-F01, API-F11, API-F68 · labels: test · ref: tools/src/main/java/org/apache/kafka/tools/ConsumerPerformance.java @4.4.0
Done when: the tool consumes a fixed message count with a group and reports its usual columns; results are archived as an NFR-F01 baseline input.

### [To Do] CLI-0103 `kafka-delegation-tokens.sh --create` and `--describe`
P1 · M4 · Task · deps: CLI-0057, API-F38, API-F41 · labels: test · ref: tools/src/main/java/org/apache/kafka/tools/DelegationTokenCommand.java @4.4.0
Done when: over a SASL connection, creating with `--max-life-time-period` and `--renewer-principal` and describing by owner match Kafka, and the token authenticates. Output and exit code match Kafka 4.4.0 for the same scenario after normalization (TEST-F09 differential).

### [To Do] CLI-0104 `kafka-delegation-tokens.sh --renew` and `--expire`
P1 · M4 · Task · deps: CLI-0057, API-F39, API-F40 · labels: test · ref: tools/src/main/java/org/apache/kafka/tools/DelegationTokenCommand.java @4.4.0
Done when: renewing and expiring by `--hmac` with periods behave as on Kafka, and an expired token no longer authenticates. Output and exit code match Kafka 4.4.0 for the same scenario after normalization (TEST-F09 differential).

### [To Do] CLI-0105 `kafka-delete-records.sh` against Alluvium
P1 · M7 · Task · deps: CLI-0057, API-F21 · labels: test · ref: tools/src/main/java/org/apache/kafka/tools/DeleteRecordsCommand.java @4.4.0
Done when: an `--offset-json-file` plan advances log start offsets and prints low watermarks; offsets past the high watermark fail with OFFSET_OUT_OF_RANGE. Output and exit code match Kafka 4.4.0 for the same scenario after normalization (TEST-F09 differential).

### [To Do] CLI-0106 `kafka-dump-log.sh`: replaced by `alluvium dump` and `verify log`
P1 · M2 · Task · deps: CLI-0057, CLI-0028, CLI-0045 · labels: test, docs · ref: tools/src/main/java/org/apache/kafka/tools/DumpLogSegments.java @4.4.0
Done when: docs/compat/tools.md maps every DumpLogSegments flag (`--print-data-log`, `--deep-iteration`, `--verify-index-only`, `--index-sanity-check`, the decoders) to its Alluvium command, explaining why Kafka's tool cannot read Alluvium's directories (ADR-0007); a differential test produces the same records on both systems and shows `kafka-dump-log.sh --deep-iteration` on Kafka and `alluvium dump log --records --json` on Alluvium agree on batch boundaries, counts, producer fields and CRCs.

### [To Do] CLI-0107 `kafka-e2e-latency.sh` against Alluvium
P2 · M2 · Task · deps: CLI-0057, API-F00, API-F01, API-F02 · labels: test · ref: tools/src/main/java/org/apache/kafka/tools/EndToEndLatency.java @4.4.0
Done when: the tool completes with acks=all and reports latency percentiles; results are archived as an NFR-F01 baseline input.

### [To Do] CLI-0108 `kafka-features.sh describe`
P1 · M2 · Task · deps: CLI-0057, API-F18 · labels: test · ref: tools/src/main/java/org/apache/kafka/tools/FeatureCommand.java @4.4.0
Done when: supported and finalized levels for every feature equal Kafka's for the same release version. Output and exit code match Kafka 4.4.0 for the same scenario after normalization (TEST-F09 differential).

### [To Do] CLI-0109 `kafka-features.sh upgrade`
P1 · M3 · Task · deps: CLI-0057, API-F57 · labels: test · ref: tools/src/main/java/org/apache/kafka/tools/FeatureCommand.java @4.4.0
Done when: upgrading by `--metadata`, `--feature name=level` and `--release-version`, with `--dry-run`, behaves as on Kafka. Output and exit code match Kafka 4.4.0 for the same scenario after normalization (TEST-F09 differential).

### [To Do] CLI-0110 `kafka-features.sh downgrade` and `disable`
P2 · M3 · Task · deps: CLI-0057, API-F57 · labels: test · ref: tools/src/main/java/org/apache/kafka/tools/FeatureCommand.java @4.4.0
Done when: safe downgrades succeed, unsafe ones need `--unsafe`, and disabling a feature works, with Kafka's error messages. Output and exit code match Kafka 4.4.0 for the same scenario after normalization (TEST-F09 differential).

### [To Do] CLI-0111 `kafka-features.sh version-mapping` and `feature-dependencies` against Alluvium's tables
P2 · M2 · Task · deps: CLI-0057, CLI-0024 · labels: test · ref: tools/src/main/java/org/apache/kafka/tools/FeatureCommand.java @4.4.0
Done when: the tool's offline output for every release version equals `alluvium storage version-mapping` and `feature-dependencies` output.

### [To Do] CLI-0112 `kafka-get-offsets.sh` against Alluvium
P1 · M2 · Task · deps: CLI-0057, API-F02, API-F03 · labels: test · ref: tools/src/main/java/org/apache/kafka/tools/GetOffsetShell.java @4.4.0
Done when: `--time` -1, -2, -3, -4 and -5 and timestamps, `--topic-partitions` patterns and `--exclude-internal-topics` print the same offsets as Kafka. Output and exit code match Kafka 4.4.0 for the same scenario after normalization (TEST-F09 differential).

### [To Do] CLI-0113 `kafka-groups.sh --list` for classic and consumer groups
P1 · M5 · Task · deps: CLI-0057, API-F16 · labels: test · ref: tools/src/main/java/org/apache/kafka/tools/GroupsCommand.java @4.4.0
Done when: listing with `--group-type`, `--protocol` and `--consumer` matches Kafka. Output and exit code match Kafka 4.4.0 for the same scenario after normalization (TEST-F09 differential).

### [To Do] CLI-0114 `kafka-groups.sh --list` for share and streams groups
P2 · M8 · Task · deps: CLI-0057, API-F16, SHARE-F01, STRM-F01 · labels: test · ref: tools/src/main/java/org/apache/kafka/tools/GroupsCommand.java @4.4.0
Done when: `--share`, `--streams` and the matching `--group-type` values list those groups as on Kafka. Output and exit code match Kafka 4.4.0 for the same scenario after normalization (TEST-F09 differential).

### [To Do] CLI-0115 `kafka-jmx.sh`: replaced by `/metrics`
P2 · M2 · Task · deps: CLI-0057, OBS-F08 · labels: docs · ref: tools/src/main/java/org/apache/kafka/tools/JmxTool.java @4.4.0
Done when: docs/compat/tools.md explains that Alluvium has no JMX (ADR-0005) and shows, for the JmxTool invocations in Kafka's docs, the equivalent `/metrics` query using docs/metrics.md names.

### [To Do] CLI-0116 `kafka-leader-election.sh --election-type PREFERRED`
P1 · M3 · Task · deps: CLI-0057, API-F43 · labels: test · ref: tools/src/main/java/org/apache/kafka/tools/LeaderElectionCommand.java @4.4.0
Done when: `--all-topic-partitions`, `--topic`/`--partition` and `--path-to-json-file` elect preferred leaders and report ELECTION_NOT_NEEDED as on Kafka. Output and exit code match Kafka 4.4.0 for the same scenario after normalization (TEST-F09 differential).

### [To Do] CLI-0117 `kafka-leader-election.sh --election-type UNCLEAN`
P2 · M3 · Task · deps: CLI-0057, API-F43 · labels: test · ref: tools/src/main/java/org/apache/kafka/tools/LeaderElectionCommand.java @4.4.0
Done when: an unclean election on a partition whose ISR is offline succeeds and is reported as on Kafka. Output and exit code match Kafka 4.4.0 for the same scenario after normalization (TEST-F09 differential).

### [To Do] CLI-0118 `kafka-log-dirs.sh --describe`
P1 · M7 · Task · deps: CLI-0057, API-F35 · labels: test · ref: tools/src/main/java/org/apache/kafka/tools/LogDirsCommand.java @4.4.0
Done when: `--broker-list` and `--topic-list` produce the same JSON structure, sizes and offset lags as on Kafka. Output and exit code match Kafka 4.4.0 for the same scenario after normalization (TEST-F09 differential).

### [To Do] CLI-0119 `kafka-metadata-quorum.sh describe --status`
P1 · M3 · Task · deps: CLI-0057, API-F55 · labels: test · ref: tools/src/main/java/org/apache/kafka/tools/MetadataQuorumCommand.java @4.4.0
Done when: through `--bootstrap-server` and `--bootstrap-controller`, ClusterId, LeaderId, LeaderEpoch, HighWatermark, MaxFollowerLag, MaxFollowerLagTimeMs, CurrentVoters and CurrentObservers are reported as on Kafka. Output and exit code match Kafka 4.4.0 for the same scenario after normalization (TEST-F09 differential).

### [To Do] CLI-0120 `kafka-metadata-quorum.sh describe --replication`
P1 · M3 · Task · deps: CLI-0057, API-F55 · labels: test · ref: tools/src/main/java/org/apache/kafka/tools/MetadataQuorumCommand.java @4.4.0
Done when: per-replica log end offset, lag and fetch times, with and without `--human-readable`, match Kafka. Output and exit code match Kafka 4.4.0 for the same scenario after normalization (TEST-F09 differential).

### [To Do] CLI-0121 `kafka-metadata-quorum.sh add-controller`
P1 · M3 · Task · deps: CLI-0057, API-F80 · labels: test · ref: tools/src/main/java/org/apache/kafka/tools/MetadataQuorumCommand.java @4.4.0
Done when: adding a formatted controller to a dynamic quorum (with `--dry-run`) succeeds as on Kafka. Output and exit code match Kafka 4.4.0 for the same scenario after normalization (TEST-F09 differential).

### [To Do] CLI-0122 `kafka-metadata-quorum.sh remove-controller`
P1 · M3 · Task · deps: CLI-0057, API-F81 · labels: test · ref: tools/src/main/java/org/apache/kafka/tools/MetadataQuorumCommand.java @4.4.0
Done when: removing a voter by `--controller-id` and `--controller-directory-id` succeeds and errors match Kafka. Output and exit code match Kafka 4.4.0 for the same scenario after normalization (TEST-F09 differential).

### [To Do] CLI-0123 `kafka-metadata-shell.sh`: replaced by `alluvium dump metadata --image`
P1 · M2 · Task · deps: CLI-0057, CLI-0041 · labels: test, docs · ref: shell/src/main/java/org/apache/kafka/shell/MetadataShell.java @4.4.0
Done when: docs/compat/tools.md maps the shell's ls, cat, find and tree commands to `alluvium dump metadata --image --path`; a differential test shows `find /` and `cat` of selected nodes on Kafka's snapshot match Alluvium's tree for the same scenario.

### [To Do] CLI-0124 `kafka-producer-perf-test.sh` against Alluvium
P1 · M2 · Task · deps: CLI-0057, API-F00 · labels: test · ref: tools/src/main/java/org/apache/kafka/tools/ProducerPerformance.java @4.4.0
Done when: runs with `--num-records`, `--record-size`, `--throughput` and acks=all (idempotence disabled) complete and report the usual summary; results are archived as an NFR-F01 baseline input.

### [To Do] CLI-0125 `kafka-producer-perf-test.sh` with transactions
P2 · M6 · Task · deps: CLI-0057, API-F22, API-F24, API-F26 · labels: test · ref: tools/src/main/java/org/apache/kafka/tools/ProducerPerformance.java @4.4.0
Done when: runs with `--transactional-id` and `--transaction-duration-ms` commit transactions and the records are visible to read_committed consumers.

### [To Do] CLI-0126 `kafka-reassign-partitions.sh --generate`
P1 · M3 · Task · deps: CLI-0057, API-F03, API-F60 · labels: test · ref: tools/src/main/java/org/apache/kafka/tools/reassign/ReassignPartitionsCommand.java @4.4.0
Done when: plans for `--topics-to-move-json-file` and `--broker-list`, with and without `--disable-rack-aware`, have the same shape as Kafka's. Output and exit code match Kafka 4.4.0 for the same scenario after normalization (TEST-F09 differential).

### [To Do] CLI-0127 `kafka-reassign-partitions.sh --execute`
P1 · M3 · Task · deps: CLI-0057, API-F45 · labels: test · ref: tools/src/main/java/org/apache/kafka/tools/reassign/ReassignPartitionsCommand.java @4.4.0
Done when: executing a plan (and `--additional`) moves replicas and data on a 3-node cluster. Output and exit code match Kafka 4.4.0 for the same scenario after normalization (TEST-F09 differential).

### [To Do] CLI-0128 `kafka-reassign-partitions.sh --verify --preserve-throttles`
P1 · M3 · Task · deps: CLI-0057, API-F46 · labels: test · ref: tools/src/main/java/org/apache/kafka/tools/reassign/ReassignPartitionsCommand.java @4.4.0
Done when: verify reports in-progress and complete states as on Kafka. Output and exit code match Kafka 4.4.0 for the same scenario after normalization (TEST-F09 differential).

### [To Do] CLI-0129 `kafka-reassign-partitions.sh --cancel`
P2 · M3 · Task · deps: CLI-0057, API-F45 · labels: test · ref: tools/src/main/java/org/apache/kafka/tools/reassign/ReassignPartitionsCommand.java @4.4.0
Done when: cancelling an in-progress reassignment restores the original replicas. Output and exit code match Kafka 4.4.0 for the same scenario after normalization (TEST-F09 differential).

### [To Do] CLI-0130 `kafka-reassign-partitions.sh --list`
P2 · M3 · Task · deps: CLI-0057, API-F46 · labels: test · ref: tools/src/main/java/org/apache/kafka/tools/reassign/ReassignPartitionsCommand.java @4.4.0
Done when: in-progress reassignments are listed as on Kafka. Output and exit code match Kafka 4.4.0 for the same scenario after normalization (TEST-F09 differential).

### [To Do] CLI-0131 `kafka-reassign-partitions.sh --throttle` and throttle clearing
P1 · M7 · Task · deps: CLI-0057, API-F44, REPL-F16 · labels: test · ref: tools/src/main/java/org/apache/kafka/tools/reassign/ReassignPartitionsCommand.java @4.4.0
Done when: `--throttle` and `--replica-alter-log-dirs-throttle` set the leader/follower replication throttle configs, throttled moves respect the rate, and `--verify` without `--preserve-throttles` clears them. Output and exit code match Kafka 4.4.0 for the same scenario after normalization (TEST-F09 differential).

### [To Do] CLI-0132 `kafka-reassign-partitions.sh` with log directory moves
P2 · M7 · Task · deps: CLI-0057, API-F34, API-F35 · labels: test · ref: tools/src/main/java/org/apache/kafka/tools/reassign/ReassignPartitionsCommand.java @4.4.0
Done when: plans with `log_dirs` move replicas between directories on one broker, and verify reports completion. Output and exit code match Kafka 4.4.0 for the same scenario after normalization (TEST-F09 differential).

### [To Do] CLI-0133 `kafka-replica-verification.sh` against Alluvium
P2 · M3 · Task · deps: CLI-0057, API-F01, API-F02, API-F03 · labels: test · ref: tools/src/main/java/org/apache/kafka/tools/ReplicaVerificationTool.java @4.4.0
Done when: the deprecated tool runs on a 3-node cluster and reports no divergence for healthy replicas.

### [To Do] CLI-0134 `kafka-run-class.sh`: not applicable
P3 · M2 · Task · deps: CLI-0057 · labels: docs · ref: bin/kafka-run-class.sh @4.4.0
Done when: docs/compat/tools.md explains it is Kafka's JVM launcher, lists its environment variables (KAFKA_HEAP_OPTS, KAFKA_OPTS, KAFKA_JMX_OPTS, KAFKA_LOG4J_OPTS) and their Alluvium counterparts or why none is needed.

### [To Do] CLI-0135 `kafka-server-start.sh`: replaced by `alluvium server`
P2 · M2 · Task · deps: CLI-0057, CLI-0011, CLI-0012, CLI-0016 · labels: test, docs · ref: bin/kafka-server-start.sh, config/server.properties, core/src/main/scala/kafka/Kafka.scala @4.4.0
Done when: docs/compat/tools.md maps `kafka-server-start.sh [-daemon] config --override k=v` to `alluvium server` (daemon mode via systemd or containers); a test formats and starts Alluvium with Kafka 4.4.0's sample config/server.properties plus the harness overrides, where the only required change is the documented plaintext deviation (explained by the startup error) and unsupported keys only warn.

### [To Do] CLI-0136 `kafka-server-stop.sh`: replaced by SIGTERM
P2 · M2 · Task · deps: CLI-0057, CLI-0010 · labels: test, docs · ref: bin/kafka-server-stop.sh @4.4.0
Done when: docs/compat/tools.md shows stopping with SIGTERM (`kill`, systemd, container stop); a test sends SIGTERM to a running node and sees an orderly shutdown with exit 0.

### [To Do] CLI-0137 `kafka-share-consumer-perf-test.sh` against Alluvium
P2 · M8 · Task · deps: CLI-0057, API-F76, API-F78, API-F79 · labels: test · ref: tools/src/main/java/org/apache/kafka/tools/ShareConsumerPerformance.java @4.4.0
Done when: the tool consumes a fixed message count through a share group and reports its summary.

### [To Do] CLI-0138 `kafka-share-groups.sh --list`
P1 · M8 · Task · deps: CLI-0057, API-F16, API-F77 · labels: test · ref: tools/src/main/java/org/apache/kafka/tools/consumer/group/ShareGroupCommand.java @4.4.0
Done when: listing share groups with `--state` matches Kafka. Output and exit code match Kafka 4.4.0 for the same scenario after normalization (TEST-F09 differential).

### [To Do] CLI-0139 `kafka-share-groups.sh --describe`
P1 · M8 · Task · deps: CLI-0057, API-F77, API-F90 · labels: test · ref: tools/src/main/java/org/apache/kafka/tools/consumer/group/ShareGroupCommand.java @4.4.0
Done when: `--members`, `--offsets`, `--state` and `--verbose` output match Kafka. Output and exit code match Kafka 4.4.0 for the same scenario after normalization (TEST-F09 differential).

### [To Do] CLI-0140 `kafka-share-groups.sh --reset-offsets`
P1 · M8 · Task · deps: CLI-0057, API-F91 · labels: test · ref: tools/src/main/java/org/apache/kafka/tools/consumer/group/ShareGroupCommand.java @4.4.0
Done when: `--to-earliest`, `--to-latest`, `--to-datetime`, `--to-offset` and `--to-current` with `--dry-run` and `--execute` match Kafka. Output and exit code match Kafka 4.4.0 for the same scenario after normalization (TEST-F09 differential).

### [To Do] CLI-0141 `kafka-share-groups.sh --delete`
P2 · M8 · Task · deps: CLI-0057, API-F42, API-F86 · labels: test · ref: tools/src/main/java/org/apache/kafka/tools/consumer/group/ShareGroupCommand.java @4.4.0
Done when: deleting an empty share group removes it and its state; deleting an active one fails as on Kafka. Output and exit code match Kafka 4.4.0 for the same scenario after normalization (TEST-F09 differential).

### [To Do] CLI-0142 `kafka-share-groups.sh --delete-offsets`
P2 · M8 · Task · deps: CLI-0057, API-F92 · labels: test · ref: tools/src/main/java/org/apache/kafka/tools/consumer/group/ShareGroupCommand.java @4.4.0
Done when: deleting a topic's share-group offsets matches Kafka. Output and exit code match Kafka 4.4.0 for the same scenario after normalization (TEST-F09 differential).

### [To Do] CLI-0143 `kafka-storage.sh`: replaced by `alluvium format` and `alluvium storage`
P1 · M2 · Task · deps: CLI-0057, CLI-0016, CLI-0018, CLI-0022, CLI-0023, CLI-0024 · labels: test, docs · ref: core/src/main/scala/kafka/tools/StorageTool.scala @4.4.0
Done when: docs/compat/tools.md maps format, info, random-uuid, version-mapping and feature-dependencies with every flag; a differential test runs kafka-storage.sh format and `alluvium format` with the same flags and shows the bootstrap metadata (feature levels and records) is equivalent.

### [To Do] CLI-0144 `kafka-streams-application-reset.sh` against Alluvium
P1 · M5 · Task · deps: CLI-0057, API-F15, API-F08, API-F09, API-F02, API-F20 · labels: test · ref: tools/src/main/java/org/apache/kafka/tools/StreamsResetter.java @4.4.0
Done when: for a stopped Streams application, `--input-topics`, `--intermediate-topics` and `--internal-topics` with `--to-earliest`, `--to-offset`, `--by-duration` and `--dry-run` reset offsets and delete internal topics as on Kafka. Output and exit code match Kafka 4.4.0 for the same scenario after normalization (TEST-F09 differential).

### [To Do] CLI-0145 `kafka-streams-groups.sh --list`
P1 · M8 · Task · deps: CLI-0057, API-F16, API-F89 · labels: test · ref: tools/src/main/java/org/apache/kafka/tools/streams/StreamsGroupCommand.java @4.4.0
Done when: listing streams groups with `--state` matches Kafka. Output and exit code match Kafka 4.4.0 for the same scenario after normalization (TEST-F09 differential).

### [To Do] CLI-0146 `kafka-streams-groups.sh --describe`
P1 · M8 · Task · deps: CLI-0057, API-F89 · labels: test · ref: tools/src/main/java/org/apache/kafka/tools/streams/StreamsGroupCommand.java @4.4.0
Done when: `--members`, `--offsets`, `--state` and `--verbose` output match Kafka. Output and exit code match Kafka 4.4.0 for the same scenario after normalization (TEST-F09 differential).

### [To Do] CLI-0147 `kafka-streams-groups.sh --reset-offsets`
P1 · M8 · Task · deps: CLI-0057, API-F89, API-F08 · labels: test · ref: tools/src/main/java/org/apache/kafka/tools/streams/StreamsGroupCommand.java @4.4.0
Done when: absolute and relative resets for input topics with `--dry-run` and `--execute` match Kafka. Output and exit code match Kafka 4.4.0 for the same scenario after normalization (TEST-F09 differential).

### [To Do] CLI-0148 `kafka-streams-groups.sh --delete` and internal topic deletion
P2 · M8 · Task · deps: CLI-0057, API-F42, API-F20 · labels: test · ref: tools/src/main/java/org/apache/kafka/tools/streams/StreamsGroupCommand.java @4.4.0
Done when: deleting an empty streams group, with `--delete-all-internal-topics` or `--delete-internal-topic`, removes the group and topics as on Kafka. Output and exit code match Kafka 4.4.0 for the same scenario after normalization (TEST-F09 differential).

### [To Do] CLI-0149 `kafka-streams-groups.sh --delete-offsets`
P2 · M8 · Task · deps: CLI-0057, API-F47 · labels: test · ref: tools/src/main/java/org/apache/kafka/tools/streams/StreamsGroupCommand.java @4.4.0
Done when: deleting input-topic offsets for an empty streams group matches Kafka. Output and exit code match Kafka 4.4.0 for the same scenario after normalization (TEST-F09 differential).

### [To Do] CLI-0150 `kafka-transactions.sh list`
P1 · M6 · Task · deps: CLI-0057, API-F66 · labels: test · ref: tools/src/main/java/org/apache/kafka/tools/TransactionsCommand.java @4.4.0
Done when: listing with state, producer-id, `--duration-filter` and `--transactional-id-pattern` filters matches Kafka. Output and exit code match Kafka 4.4.0 for the same scenario after normalization (TEST-F09 differential).

### [To Do] CLI-0151 `kafka-transactions.sh describe`
P1 · M6 · Task · deps: CLI-0057, API-F65 · labels: test · ref: tools/src/main/java/org/apache/kafka/tools/TransactionsCommand.java @4.4.0
Done when: describing a transactional id shows coordinator, state, epoch, timeout and partitions as on Kafka. Output and exit code match Kafka 4.4.0 for the same scenario after normalization (TEST-F09 differential).

### [To Do] CLI-0152 `kafka-transactions.sh describe-producers`
P1 · M6 · Task · deps: CLI-0057, API-F61 · labels: test · ref: tools/src/main/java/org/apache/kafka/tools/TransactionsCommand.java @4.4.0
Done when: active producers for a partition (with `--broker-id`) are listed as on Kafka. Output and exit code match Kafka 4.4.0 for the same scenario after normalization (TEST-F09 differential).

### [To Do] CLI-0153 `kafka-transactions.sh abort`
P1 · M6 · Task · deps: CLI-0057, API-F27, API-F61 · labels: test · ref: tools/src/main/java/org/apache/kafka/tools/TransactionsCommand.java @4.4.0
Done when: aborting a hanging transaction by `--start-offset`, or by producer id, epoch and coordinator epoch, writes the marker and advances the last stable offset. Output and exit code match Kafka 4.4.0 for the same scenario after normalization (TEST-F09 differential).

### [To Do] CLI-0154 `kafka-transactions.sh find-hanging`
P1 · M6 · Task · deps: CLI-0057, API-F66, API-F61, API-F65 · labels: test · ref: tools/src/main/java/org/apache/kafka/tools/TransactionsCommand.java @4.4.0
Done when: a transaction left open beyond `--max-transaction-timeout` is found by `--broker-id` and by topic and partition. Output and exit code match Kafka 4.4.0 for the same scenario after normalization (TEST-F09 differential).

### [To Do] CLI-0155 `kafka-transactions.sh forceTerminateTransaction`
P2 · M6 · Task · deps: CLI-0057, API-F22, TXN-F14 · labels: test · ref: tools/src/main/java/org/apache/kafka/tools/TransactionsCommand.java @4.4.0
Done when: force-terminating a prepared two-phase-commit transaction aborts it as on Kafka. Output and exit code match Kafka 4.4.0 for the same scenario after normalization (TEST-F09 differential).

### [To Do] CLI-0156 `kafka-verifiable-consumer.sh` against Alluvium
P2 · M5 · Task · deps: CLI-0057, API-F11, API-F68, API-F08 · labels: test · ref: tools/src/main/java/org/apache/kafka/tools/VerifiableConsumer.java @4.4.0
Done when: the tool's JSON event stream (partitions assigned, records consumed, offsets committed) for both group protocols has the same event sequence shape as on Kafka, so TEST-F15 system tests can use it.

### [To Do] CLI-0157 `kafka-verifiable-producer.sh` against Alluvium
P2 · M2 · Task · deps: CLI-0057, API-F00 · labels: test · ref: tools/src/main/java/org/apache/kafka/tools/VerifiableProducer.java @4.4.0
Done when: with `--acks` -1, 0 and 1 and `--max-messages`, the JSON event stream reports every send acknowledged with offsets.

### [To Do] CLI-0158 `kafka-verifiable-share-consumer.sh` against Alluvium
P2 · M8 · Task · deps: CLI-0057, API-F76, API-F78, API-F79 · labels: test · ref: tools/src/main/java/org/apache/kafka/tools/VerifiableShareConsumer.java @4.4.0
Done when: the JSON event stream for each acknowledgement mode matches Kafka's event sequence shape.

### [To Do] CLI-0159 `connect-distributed.sh` workers against Alluvium
P1 · M5 · Task · deps: CLI-0057, API-F11, API-F14, API-F19, API-F32, STOR-F14 · labels: test · ref: connect/runtime/src/main/java/org/apache/kafka/connect/cli/ConnectDistributed.java @4.4.0
Done when: a 2-worker Connect cluster creates its compacted internal topics, runs FileStream source and sink connectors, and rebalances tasks when a worker joins and leaves.

### [To Do] CLI-0160 `connect-distributed.sh` with exactly-once source support
P2 · M6 · Task · deps: CLI-0057, API-F22, API-F24, API-F26, API-F28 · labels: test · ref: connect/runtime/src/main/java/org/apache/kafka/connect/cli/ConnectDistributed.java @4.4.0
Done when: with `exactly.once.source.support=enabled` a source connector's records and offsets commit atomically and survive a worker crash without duplicates.

### [To Do] CLI-0161 `connect-standalone.sh` against Alluvium
P2 · M5 · Task · deps: CLI-0057, API-F00, API-F01, API-F11 · labels: test · ref: connect/runtime/src/main/java/org/apache/kafka/connect/cli/ConnectStandalone.java @4.4.0
Done when: a standalone worker runs FileStream source and sink connectors end to end.

### [To Do] CLI-0162 `connect-internal-topics.sh create` against Alluvium
P2 · M2 · Task · deps: CLI-0057, API-F19, API-F03 · labels: test · ref: tools/src/main/java/org/apache/kafka/tools/ConnectInternalTopics.java @4.4.0
Done when: the tool creates Connect's config, offset and status topics from a worker config with the expected partitions, replication and `cleanup.policy=compact`.

### [To Do] CLI-0163 `connect-mirror-maker.sh` from Kafka 4.4.0 into Alluvium
P1 · M7 · Task · deps: CLI-0057, API-F19, API-F32, API-F44, API-F30, API-F08 · labels: test · ref: connect/mirror/src/main/java/org/apache/kafka/connect/mirror/MirrorMaker.java @4.4.0
Done when: MirrorMaker 2 replicates topics with configs, data, consumer offsets (checkpoints) and ACLs from a Kafka source to an Alluvium target, and consumers fail over with translated offsets.

### [To Do] CLI-0164 `connect-mirror-maker.sh` from Alluvium into Kafka 4.4.0
P2 · M7 · Task · deps: CLI-0057, API-F32, API-F29, API-F09, API-F01 · labels: test · ref: connect/mirror/src/main/java/org/apache/kafka/connect/mirror/MirrorMaker.java @4.4.0
Done when: MirrorMaker 2 replicates topics, configs, offsets and ACLs from an Alluvium source to a Kafka target.

### [To Do] CLI-0165 `connect-plugin-path.sh`: not applicable
P3 · M2 · Task · deps: CLI-0057 · labels: docs · ref: tools/src/main/java/org/apache/kafka/tools/ConnectPluginPath.java @4.4.0
Done when: docs/compat/tools.md records that the tool only inspects and rewrites local Connect plugin manifests and never contacts a broker, so it needs no Alluvium support.

### [To Do] CLI-0166 `trogdor.sh`: workloads against Alluvium, faults not applicable
P3 · M9 · Task · deps: CLI-0057, API-F00, API-F01, API-F11 · labels: test, docs · ref: trogdor/src/main/java/org/apache/kafka/trogdor/coordinator/Coordinator.java @4.4.0
Done when: Trogdor's ProduceBench and ConsumeBench workloads run against Alluvium from a Kafka 4.4.0 Trogdor agent; docs/compat/tools.md records that process-kill and network faults target Kafka processes and are replaced by TEST-F12 chaos tests.

## [To Do] CLI-F07 Shell completions and man pages
P3 · M9 · Feature · deps: CLI-F01
Scope: Generated completions and man pages.

### [To Do] CLI-0167 `alluvium completions <shell>` for bash, zsh, fish and PowerShell
P3 · M9 · Task · deps: none
Done when: completions are generated from the clap definitions (dependency justified) and covered by snapshot tests; `--help` documents installation.

### [To Do] CLI-0168 Man pages generated from the clap definitions
P3 · M9 · Task · deps: FND-F04
Done when: `cargo xtask man` writes one page per subcommand; CI fails when committed pages are stale; pages ship in release artifacts.

### [To Do] CLI-0169 Generate the docs/cli.md reference from the clap definitions
P3 · M9 · Task · deps: CLI-0168 · labels: docs
Done when: the hand-written command reference is replaced by generated sections with a CI drift check; narrative sections stay hand-written.

### [To Do] CLI-0170 Install completions and man pages with packages and the container image
P3 · M9 · Task · deps: CLI-0167, CLI-0168, OPS-F05, OPS-F03
Done when: systemd packages and the container image install completions and man pages in standard locations; a packaging test checks they are present.
