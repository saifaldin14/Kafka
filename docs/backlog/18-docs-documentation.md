# DOCS: User and operator documentation

> Crates: `docs/`, docs site · Kafka counterpart: kafka.apache.org/documentation · Milestones: M0-M9

Everything users and operators read: the docs site, quickstart, concepts, references generated from code, operations and security guides, and the Kafka migration guide.

## [To Do] DOCS-F01 Documentation site
P2 · M2 · Feature · deps: FND-F10
Scope: mdBook site structure, build and publishing.

### [To Do] DOCS-0001 Decide the docs site layout, URL scheme and versioning model
P2 · M2 · Spike · deps: none · labels: docs
Done when: ADR "Documentation site" fixes the mdBook source location (`docs/site/`), which existing repository docs are published (glossary, architecture, ADRs, standards) and how they are pulled in without copies, the URL scheme (`/next/` from main, `/<X.Y>/` per release), hosting on GitHub Pages, and how many versions stay published.

### [To Do] DOCS-0002 mdBook skeleton: `book.toml`, SUMMARY, landing page and `cargo xtask docs build`
P2 · M2 · Task · deps: DOCS-0001 · labels: docs
Done when: `docs/site/book.toml` pins the mdBook version installed by xtask; `SUMMARY.md` has the parts Getting started, Concepts, Operations, Security, Reference, Kafka compatibility and migration, and Contributing; `cargo xtask docs build` builds with zero warnings in CI.

### [To Do] DOCS-0003 Navigation: part index pages, glossary integration and orphan check
P2 · M2 · Task · deps: DOCS-0002 · labels: docs
Done when: each part has an index page linking its chapters; the glossary renders from docs/glossary.md as the single source; `cargo xtask docs check` fails on a page under `docs/site/` not reachable from SUMMARY and on duplicate page titles.

### [To Do] DOCS-0004 Link and anchor checking on the built site
P2 · M2 · Task · deps: DOCS-0002 · labels: docs
Done when: CI checks every internal link and anchor in the rendered HTML, including generated reference pages, extending the FND-F10 Markdown check; external links are checked nightly with retries and an allow-list, never blocking PRs.

### [To Do] DOCS-0005 Versioned docs trees and version switcher
P2 · M2 · Task · deps: DOCS-0002 · labels: docs
Done when: the build produces `/next/` and `/<X.Y>/` trees; a generated `versions.json` drives the version switcher; released trees are never rebuilt; a test builds two versions side by side and checks the switcher links.

### [To Do] DOCS-0006 Publish pipeline to GitHub Pages
P2 · M2 · Task · deps: DOCS-0005, DOCS-0004, DOCS-0003 · labels: docs, security
Done when: pushes to main publish `/next/` and release tags publish `/<X.Y>/`; actions are SHA-pinned and the workflow has only `pages: write` and `id-token: write`; publishing fails when `cargo xtask docs check` fails; PRs get the built site as a preview artifact.

### [To Do] DOCS-0007 Documentation style guide
P2 · M2 · Task · deps: DOCS-0002 · labels: docs
Done when: `docs/site/src/contributing/style.md` covers audience per part, voice and tense, Kafka terms from docs/glossary.md, command blocks (`$` prompt, output separated), config key and metric formatting, the "Kafka difference" callout with its ADR link, labelling opt-in Alluvium features, and no screenshots of terminal output.

### [To Do] DOCS-0008 Docs review checklist and PR template section
P2 · M2 · Task · deps: DOCS-0007 · labels: docs
Done when: `docs/site/src/contributing/docs-review.md` lists the checks (commands run and output verified, Kafka differences linked to ADRs, generated references regenerated, links pass, no secrets or real hostnames in examples, style guide followed); `.github/pull_request_template.md` links it in a docs section.

### [To Do] DOCS-0009 Prose lint with Vale using the project vocabulary
P3 · M2 · Task · deps: DOCS-0007 · labels: docs
Done when: a Vale config accepts glossary terms and flags the banned synonyms listed in the style guide; it runs in CI on `docs/site/` with zero errors.

## [To Do] DOCS-F02 Quickstart
P1 · M2 · Feature · deps: CLI-F02
Scope: Install, start, produce and consume with Kafka tools.

### [To Do] DOCS-0010 Executable docs snippets: run marked command blocks in CI
P1 · M2 · Task · deps: DOCS-0002 · labels: docs, test
Done when: code blocks tagged `console,test` are extracted by `cargo xtask docs test-snippets` and run in order against a fresh node; output lines marked as expected are compared after documented normalization of ids and timestamps; a failure names the page and line.

### [To Do] DOCS-0011 Quickstart step 1: get Alluvium from a release archive, a container image or source
P1 · M2 · Task · deps: OPS-F02, OPS-F03, DOCS-0010 · labels: docs · ref: docs/getting-started/quickstart.md Step 1 @4.4.0
Done when: `docs/site/src/getting-started/quickstart.md` shows downloading the archive and checking it against `SHA256SUMS`, pulling the image by tag, and `cargo install --locked`; the archive path is a tested snippet.

### [To Do] DOCS-0012 Quickstart step 2: format storage and start a combined-mode node
P1 · M2 · Task · deps: CLI-F03, DOCS-0010 · labels: docs · ref: docs/getting-started/quickstart.md Step 2 @4.4.0
Done when: the step generates a cluster id, formats a standalone combined-mode node with `alluvium format`, starts it with `alluvium server --config config/server.properties`, and waits for `/health/ready`; startup output is shown and the commands are tested snippets.

### [To Do] DOCS-0013 Quickstart step 3: create a topic with `kafka-topics.sh`
P1 · M2 · Task · deps: API-F19, API-F03, CLI-F06, DOCS-0012 · labels: docs · ref: docs/getting-started/quickstart.md Step 3 @4.4.0
Done when: the step fetches the Kafka 4.4.0 tools, runs `kafka-topics.sh --create --topic quickstart-events --bootstrap-server localhost:9092` and `--describe`, and shows the expected output as tested snippets.

### [To Do] DOCS-0014 Quickstart step 4: write events with `kafka-console-producer.sh` and kcat
P1 · M2 · Task · deps: API-F00, CLI-F06, DOCS-0013 · labels: docs · ref: docs/getting-started/quickstart.md Step 4 @4.4.0
Done when: the step produces lines with `kafka-console-producer.sh` and keyed records with `kcat -P -K:`; both are tested snippets.

### [To Do] DOCS-0015 Quickstart step 5: read events from an assigned partition
P1 · M2 · Task · deps: API-F01, API-F02, DOCS-0014 · labels: docs · ref: docs/getting-started/quickstart.md Step 5 @4.4.0
Done when: the step reads with `kafka-console-consumer.sh --partition 0 --offset earliest`, which needs no consumer group, and with `kcat -C -e`; output matches what step 4 wrote in the snippet test.

### [To Do] DOCS-0016 Quickstart step 5 with consumer groups: consume as a group and inspect lag
P1 · M5 · Task · deps: GRP-F04, GRP-F05, DOCS-0015 · labels: docs
Done when: the primary path becomes `kafka-console-consumer.sh --group quickstart --from-beginning` followed by `kafka-consumer-groups.sh --describe --group quickstart` showing zero lag; tested snippets.

### [To Do] DOCS-0017 Quickstart step 6: look inside with metrics, diagnostics and dump tools
P1 · M2 · Task · deps: OBS-F08, CLI-F04, DOCS-0015 · labels: docs
Done when: the step greps the produce request counter from `/metrics`, reads `/debug/partitions/quickstart-events/0` to show the high watermark and flight recorder, and runs `alluvium dump log` on the segment showing that payloads are hidden unless `--payload` is given; tested snippets.

### [To Do] DOCS-0018 Quickstart step: import and export data with Kafka Connect
P2 · M6 · Task · deps: OPS-F12, DOCS-0016 · labels: docs · ref: docs/getting-started/quickstart.md Step 6 @4.4.0
Done when: the FileStream source and sink pipeline from the Kafka quickstart runs against Alluvium with `connect-standalone.sh`, as a tested snippet.

### [To Do] DOCS-0019 Quickstart step: process events with the Kafka Streams WordCount demo
P2 · M6 · Task · deps: OPS-F12, DOCS-0016 · labels: docs · ref: docs/getting-started/quickstart.md Step 7 @4.4.0
Done when: the WordCount demo runs against Alluvium and its output topic is read with the console consumer, as a tested snippet.

### [To Do] DOCS-0020 Quickstart final step: stop the node and clean up
P1 · M2 · Task · deps: DOCS-0012 · labels: docs · ref: docs/getting-started/quickstart.md Step 8 @4.4.0
Done when: the step stops the node with Ctrl-C or SIGTERM, shows the logged shutdown sequence, and removes the data directory; tested snippet.

### [To Do] DOCS-0021 Quickstart variant with docker compose
P2 · M2 · Task · deps: OPS-F03, DOCS-0010 · labels: docs
Done when: a variant page runs the same steps with the single-node compose file and the Kafka tools container, and explains why the compose file sets the plaintext opt-in (ADR-0006); tested in CI with Docker.

### [To Do] DOCS-0022 Run the quickstart in CI on Linux and macOS
P1 · M2 · Task · deps: DOCS-0011, DOCS-0012, DOCS-0013, DOCS-0014, DOCS-0015, DOCS-0017, DOCS-0020 · labels: docs, test
Done when: `cargo xtask docs test-quickstart` executes every quickstart snippet in order on Ubuntu and macOS runners for PRs touching `docs/site/` or `crates/alluvium/`, and fails with the step name.

## [To Do] DOCS-F03 Concepts guide
P1 · M9 · Feature · deps: DOCS-F01
Scope: Topics, partitions, replication, groups, transactions in Alluvium terms.

### [To Do] DOCS-0023 Concepts: Architecture for users: nodes, roles, listeners and the KRaft quorum
P1 · M9 · Task · deps: none · labels: docs
Done when: `docs/site/src/concepts/overview.md` explains process roles, controller quorum, broker and controller listeners, and how metadata flows from the controller to every broker, using only glossary terms, linking each config key to the reference anchor and the matching Kafka 4.4.0 documentation section, and passes the docs review checklist.

### [To Do] DOCS-0024 Concepts: Topics, partitions, offsets and the log
P1 · M9 · Task · deps: STOR-F08 · labels: docs
Done when: `docs/site/src/concepts/topics.md` explains topic and partition identity (names and topic ids), offsets, log start and end offsets, segments and indexes, ordering guarantees, using only glossary terms, linking each config key to the reference anchor and the matching Kafka 4.4.0 documentation section, and passes the docs review checklist.

### [To Do] DOCS-0025 Concepts: Records and batches: keys, headers, timestamps and compression
P1 · M9 · Task · deps: REC-F12 · labels: docs
Done when: `docs/site/src/concepts/records.md` explains record batch v2, `CreateTime` versus `LogAppendTime`, compression codecs and levels, and append validation errors, using only glossary terms, linking each config key to the reference anchor and the matching Kafka 4.4.0 documentation section, and passes the docs review checklist.

### [To Do] DOCS-0026 Concepts: Retention and the segment lifecycle
P1 · M9 · Task · deps: STOR-F13, STOR-F07 · labels: docs
Done when: `docs/site/src/concepts/retention.md` explains time and size retention, segment rolling, deletion timing, and how log start offset moves, using only glossary terms, linking each config key to the reference anchor and the matching Kafka 4.4.0 documentation section, and passes the docs review checklist.

### [To Do] DOCS-0027 Concepts: Log compaction and tombstones
P1 · M9 · Task · deps: STOR-F14 · labels: docs
Done when: `docs/site/src/concepts/compaction.md` explains `cleanup.policy=compact`, the cleaner's dirty ratio and lag settings, tombstone retention and what consumers can rely on, using only glossary terms, linking each config key to the reference anchor and the matching Kafka 4.4.0 documentation section, and passes the docs review checklist.

### [To Do] DOCS-0028 Concepts: Replication: leaders, ISR, high watermark, acks and min.insync.replicas
P1 · M9 · Task · deps: REPL-F05, CTRL-F08 · labels: docs
Done when: `docs/site/src/concepts/replication.md` explains leader and follower roles, ISR membership, high watermark, `acks` with `min.insync.replicas`, eligible leader replicas, unclean leader election and fetch from follower, using only glossary terms, linking each config key to the reference anchor and the matching Kafka 4.4.0 documentation section, and passes the docs review checklist.

### [To Do] DOCS-0029 Concepts: Cluster metadata: controllers, the metadata log, `metadata.version` and features
P1 · M9 · Task · deps: CTRL-F17, META-F04 · labels: docs
Done when: `docs/site/src/concepts/metadata.md` explains the metadata log and snapshots, the metadata image on brokers, finalized feature levels and why upgrades finalize `metadata.version`, using only glossary terms, linking each config key to the reference anchor and the matching Kafka 4.4.0 documentation section, and passes the docs review checklist.

### [To Do] DOCS-0030 Concepts: Producers: partitioning, batching, acks and idempotence
P1 · M9 · Task · deps: TXN-F01 · labels: docs
Done when: `docs/site/src/concepts/producers.md` explains partitioners, batching and linger, retries and ordering, producer ids, epochs and sequence numbers, using only glossary terms, linking each config key to the reference anchor and the matching Kafka 4.4.0 documentation section, and passes the docs review checklist.

### [To Do] DOCS-0031 Concepts: Consumers and consumer groups
P1 · M9 · Task · deps: GRP-F04, GRP-F05, GRP-F07 · labels: docs
Done when: `docs/site/src/concepts/consumers.md` explains the classic and consumer group protocols, server-side assignors, static membership, committed offsets and lag, using only glossary terms, linking each config key to the reference anchor and the matching Kafka 4.4.0 documentation section, and passes the docs review checklist.

### [To Do] DOCS-0032 Concepts: Transactions and exactly-once processing
P1 · M9 · Task · deps: TXN-F10, TXN-F11, TXN-F14 · labels: docs
Done when: `docs/site/src/concepts/transactions.md` explains transactional producers, the transaction coordinator, markers, last stable offset and `read_committed`, transactions v2 and two-phase commit, using only glossary terms, linking each config key to the reference anchor and the matching Kafka 4.4.0 documentation section, and passes the docs review checklist.

### [To Do] DOCS-0033 Concepts: Share groups
P1 · M9 · Task · deps: SHARE-F02, SHARE-F03 · labels: docs
Done when: `docs/site/src/concepts/share-groups.md` explains cooperative consumption, acquisition locks, delivery counts, acknowledgement types and how share groups differ from consumer groups, using only glossary terms, linking each config key to the reference anchor and the matching Kafka 4.4.0 documentation section, and passes the docs review checklist.

### [To Do] DOCS-0034 Concepts: Streams groups
P1 · M9 · Task · deps: STRM-F03 · labels: docs
Done when: `docs/site/src/concepts/streams-groups.md` explains the KIP-1071 streams rebalance protocol, topology metadata and server-side task assignment, using only glossary terms, linking each config key to the reference anchor and the matching Kafka 4.4.0 documentation section, and passes the docs review checklist.

### [To Do] DOCS-0035 Concepts: Tiered storage
P1 · M9 · Task · deps: TIER-F04, TIER-F06 · labels: docs
Done when: `docs/site/src/concepts/tiered-storage.md` explains local and remote retention, segment copy, remote reads and what changes for consumers, using only glossary terms, linking each config key to the reference anchor and the matching Kafka 4.4.0 documentation section, and passes the docs review checklist.

### [To Do] DOCS-0036 Concepts: Security model: principals, authentication and authorization
P1 · M9 · Task · deps: SEC-F08 · labels: docs
Done when: `docs/site/src/concepts/security.md` explains listeners and security protocols, how a principal is established (TLS, SASL), and how ACLs authorize each operation, using only glossary terms, linking each config key to the reference anchor and the matching Kafka 4.4.0 documentation section, and passes the docs review checklist.

### [To Do] DOCS-0037 Concepts: Quotas and throttling
P1 · M9 · Task · deps: QUOTA-F07 · labels: docs
Done when: `docs/site/src/concepts/quotas.md` explains byte-rate, request-time, controller-mutation and connection-rate quotas, quota entities, and how throttling is communicated to clients, using only glossary terms, linking each config key to the reference anchor and the matching Kafka 4.4.0 documentation section, and passes the docs review checklist.

### [To Do] DOCS-0038 Concepts: How Alluvium explains itself: events, reasons, flight recorders and metrics
P1 · M9 · Task · deps: OBS-F06, OBS-F07 · labels: docs
Done when: `docs/site/src/concepts/observability.md` explains structured events with reasons, flight recorders, metrics and their Kafka mapping, the diagnostics endpoint and offline dump tools, using only glossary terms, linking each config key to the reference anchor and the matching Kafka 4.4.0 documentation section, and passes the docs review checklist.

### [To Do] DOCS-0039 Concepts: Client telemetry
P2 · M9 · Task · deps: TELEM-F02 · labels: docs
Done when: `docs/site/src/concepts/telemetry.md` explains KIP-714 subscriptions, client metrics resources and where pushed metrics go, using only glossary terms, linking each config key to the reference anchor and the matching Kafka 4.4.0 documentation section, and passes the docs review checklist.

### [To Do] DOCS-0040 Concepts: The Kafka compatibility surface and opt-in Alluvium features
P1 · M9 · Task · deps: none · labels: docs
Done when: `docs/site/src/concepts/compat-surface.md` explains what is guaranteed identical to Kafka 4.4.0, how deviations are decided by ADR, and why every feature beyond Kafka is off by default (ADR-0010), using only glossary terms, linking each config key to the reference anchor and the matching Kafka 4.4.0 documentation section, and passes the docs review checklist.

## [To Do] DOCS-F04 Configuration reference
P1 · M2 · Feature · deps: CFG-F12
Scope: Generated reference with Kafka parity notes.

### [To Do] DOCS-0041 Render the generated configuration reference into the site, one page per category
P1 · M2 · Task · deps: DOCS-0002 · labels: docs
Done when: broker, topic, KRaft and controller, coordinator and `alluvium.*` pages are generated at docs build time from the CFG-F12 output with no second committed copy; each key has the anchor `#<key>`; a test resolves the anchor of every key in the registry.

### [To Do] DOCS-0042 Kafka parity notes per key: Kafka default, Alluvium default, status and ADR
P1 · M2 · Task · deps: DOCS-0041, CFG-F11 · labels: docs
Done when: each key shows the Kafka 4.4.0 default, the Alluvium default and a status (supported, accepted with no effect, rejected at startup, deviation) from registry metadata; the build fails when a deviation has no ADR link.

### [To Do] DOCS-0043 Drift check: recorded Kafka defaults equal Kafka 4.4.0's
P1 · M2 · Task · deps: DOCS-0042, TEST-F09 · labels: docs, test · ref: docs/configuration/broker-configs.md, docs/configuration/topic-configs.md @4.4.0
Done when: a CI job starts Kafka 4.4.0 in the differential harness, reads `kafka-configs.sh --describe --all` for a broker and a topic, and fails on any key whose recorded Kafka default differs or that the registry doesn't list.

### [To Do] DOCS-0044 Kafka keys Alluvium doesn't support and what happens when they are set
P1 · M2 · Task · deps: DOCS-0042 · labels: docs
Done when: a generated page lists unsupported Kafka keys (plugin classes, JMX, log4j and similar) with the startup outcome (error or warning); a test sets each and checks the documented outcome.

### [To Do] DOCS-0045 Broker default to topic override mapping and dynamic update modes
P1 · M2 · Task · deps: DOCS-0041, CFG-F08 · labels: docs
Done when: a generated table maps broker keys to their topic-level overrides (`log.retention.ms` to `retention.ms`) and shows each key's dynamic update mode (read-only, per-broker, cluster-wide).

### [To Do] DOCS-0046 Config sources: files, environment overrides, config providers and precedence
P1 · M2 · Task · deps: DOCS-0010, CFG-F02, CFG-F03 · labels: docs
Done when: a page documents environment override naming, `${provider:path:key}` with the file, directory and env providers, precedence, and secret redaction, with an `alluvium check-config` example run as a tested snippet.

## [To Do] DOCS-F05 Metrics reference
P1 · M2 · Feature · deps: OBS-F05
Scope: Generated reference with Kafka name mapping.

### [To Do] DOCS-0047 Generate docs/metrics.md from registered metric descriptors
P1 · M2 · Task · deps: OBS-F04 · labels: docs
Done when: `cargo xtask docs gen metrics` writes docs/metrics.md from each crate's metric descriptors (name, type, unit, labels, help, Kafka MBean names) grouped by subsystem, reproducing every row maintained so far; CI fails when the committed file is stale.

### [To Do] DOCS-0048 Runtime drift check: every metric a running node exposes is documented
P1 · M2 · Task · deps: DOCS-0047 · labels: docs, test
Done when: CI starts a node, runs the standard workload, scrapes `/metrics`, and fails on any metric family not in docs/metrics.md and on any documented family never registered.

### [To Do] DOCS-0049 Render the metrics reference into the site per subsystem
P1 · M2 · Task · deps: DOCS-0047, DOCS-0002 · labels: docs
Done when: one page per subsystem with an anchor per metric is generated at build time from the descriptors.

### [To Do] DOCS-0050 Kafka MBean to Alluvium metric index with PromQL translations
P1 · M2 · Task · deps: DOCS-0047 · labels: docs
Done when: a generated page sorted by Kafka MBean gives the Alluvium metric and a PromQL equivalent (for example `rate()` for a Kafka meter's one-minute rate), and a reason for each MBean with no equivalent.

### [To Do] DOCS-0051 Drift check against Kafka 4.4.0's monitoring documentation
P1 · M2 · Task · deps: DOCS-0050 · labels: docs, test · ref: docs/operations/monitoring.md @4.4.0
Done when: a script extracts every server-side MBean name from Kafka's monitoring.md and fails unless each is mapped or listed as intentionally absent with a reason; producer, consumer, Connect and Streams client sections are excluded explicitly.

### [To Do] DOCS-0052 Labels and cardinality bounds per metric
P1 · M2 · Task · deps: DOCS-0047 · labels: docs
Done when: each metric lists its labels and their bounds (`topic`; `partition` only with `alluvium.metrics.per.partition.enabled`; capped `client_id` on quota metrics); generation fails on a label a descriptor doesn't declare.

### [To Do] DOCS-0053 PromQL cookbook
P2 · M2 · Task · deps: DOCS-0049 · labels: docs
Done when: a page gives queries for throughput, request p99 by phase, under-replicated partitions, consumer lag and throttle time; each query is evaluated by `promtool` against a recorded scrape in CI.

## [To Do] DOCS-F06 Events reference
P1 · M2 · Feature · deps: OBS-F07
Scope: Generated event catalog.

### [To Do] DOCS-0054 Render the event catalog into the site per component
P1 · M2 · Task · deps: DOCS-0002 · labels: docs
Done when: one page per crate is generated from the OBS-F07 catalog with event name, level, trigger, fields and flight-recorder component, and an anchor per event.

### [To Do] DOCS-0055 Reason values for every event
P1 · M2 · Task · deps: DOCS-0054 · labels: docs
Done when: each event lists every `reason` variant's stable display string and meaning, extracted from the reason enums; the build fails when a variant has no doc comment.

### [To Do] DOCS-0056 Flight recorder entry reference
P1 · M2 · Task · deps: DOCS-0054, OBS-F06 · labels: docs
Done when: for partitions, groups, transaction coordinator partitions, the Raft quorum and connections, a generated page lists input kinds and state summaries as `/debug/...` prints them; CI fails when it is stale.

### [To Do] DOCS-0057 Event stability: introduced-in version and removals
P1 · M2 · Task · deps: DOCS-0054, OPS-F01 · labels: docs
Done when: each event shows the release that introduced it, taken from the OPS-F01 surface manifests; removed or renamed events are listed with the release and replacement; the check fails when catalog and manifest disagree.

### [To Do] DOCS-0058 Finding events: jq, Loki and Elasticsearch query examples
P2 · M2 · Task · deps: DOCS-0054, OPS-F13 · labels: docs
Done when: a page shows filtering by `event`, `reason`, `topic` and `partition` with jq, LogQL and Elasticsearch queries; the jq examples run against sample log lines in CI.

## [To Do] DOCS-F07 Error reference
P1 · M9 · Feature · deps: ERR-F03
Scope: Each error code: meaning, when returned, client action.

### [To Do] DOCS-0059 Generate the error reference from the ErrorCode registry
P1 · M9 · Task · deps: ERR-F01, DOCS-0002 · labels: docs
Done when: a generated page per error family lists code, name, retriable flag and default message with an anchor per name; the build fails when any registry code lacks meaning, conditions or client-action text.

### [To Do] DOCS-0060 Link each error code to the tested conditions that return it
P1 · M9 · Task · deps: DOCS-0059 · labels: docs, test
Done when: the conditions per code (API, version range, condition) are read from the ERR-F03 test metadata, so the page lists exactly the tested conditions; the check fails when a condition test isn't referenced.

### [To Do] DOCS-0061 Error reference: Partition, leadership and offset errors
P1 · M9 · Task · deps: DOCS-0059, ERR-F04 · labels: docs
Done when: meaning and client action are written for `NOT_LEADER_OR_FOLLOWER`, `UNKNOWN_TOPIC_OR_PARTITION`, `LEADER_NOT_AVAILABLE`, `OFFSET_OUT_OF_RANGE`, `FENCED_LEADER_EPOCH`, `UNKNOWN_LEADER_EPOCH`, `NOT_ENOUGH_REPLICAS`, `NOT_ENOUGH_REPLICAS_AFTER_APPEND`, `KAFKA_STORAGE_ERROR`, `UNKNOWN_TOPIC_ID`, stating whether the Java client and librdkafka retry, refresh metadata or surface the error as verified by ERR-F04; every code in the family passes the completeness check.

### [To Do] DOCS-0062 Error reference: Record, request and protocol errors
P1 · M9 · Task · deps: DOCS-0059, ERR-F04 · labels: docs
Done when: meaning and client action are written for `CORRUPT_MESSAGE`, `MESSAGE_TOO_LARGE`, `RECORD_LIST_TOO_LARGE`, `INVALID_RECORD`, `INVALID_TIMESTAMP`, `UNSUPPORTED_COMPRESSION_TYPE`, `INVALID_REQUIRED_ACKS`, `INVALID_REQUEST`, `UNSUPPORTED_VERSION`, `REQUEST_TIMED_OUT`, stating whether the Java client and librdkafka retry, refresh metadata or surface the error as verified by ERR-F04; every code in the family passes the completeness check.

### [To Do] DOCS-0063 Error reference: Group coordinator errors
P1 · M9 · Task · deps: DOCS-0059, ERR-F04 · labels: docs
Done when: meaning and client action are written for `COORDINATOR_NOT_AVAILABLE`, `NOT_COORDINATOR`, `COORDINATOR_LOAD_IN_PROGRESS`, `ILLEGAL_GENERATION`, `UNKNOWN_MEMBER_ID`, `REBALANCE_IN_PROGRESS`, `FENCED_INSTANCE_ID`, `GROUP_MAX_SIZE_REACHED`, `FENCED_MEMBER_EPOCH`, `GROUP_ID_NOT_FOUND`, `NON_EMPTY_GROUP`, `GROUP_DELETION_FAILED`, stating whether the Java client and librdkafka retry, refresh metadata or surface the error as verified by ERR-F04; every code in the family passes the completeness check.

### [To Do] DOCS-0064 Error reference: Producer and transaction errors
P1 · M9 · Task · deps: DOCS-0059, ERR-F04 · labels: docs
Done when: meaning and client action are written for `OUT_OF_ORDER_SEQUENCE_NUMBER`, `DUPLICATE_SEQUENCE_NUMBER`, `INVALID_PRODUCER_EPOCH`, `PRODUCER_FENCED`, `INVALID_TXN_STATE`, `INVALID_PRODUCER_ID_MAPPING`, `CONCURRENT_TRANSACTIONS`, `INVALID_TRANSACTION_TIMEOUT`, `TRANSACTIONAL_ID_NOT_FOUND`, `UNKNOWN_PRODUCER_ID`, stating whether the Java client and librdkafka retry, refresh metadata or surface the error as verified by ERR-F04; every code in the family passes the completeness check.

### [To Do] DOCS-0065 Error reference: Authentication and authorization errors
P1 · M9 · Task · deps: DOCS-0059, ERR-F04 · labels: docs
Done when: meaning and client action are written for `SASL_AUTHENTICATION_FAILED`, `ILLEGAL_SASL_STATE`, `UNSUPPORTED_SASL_MECHANISM`, `TOPIC_AUTHORIZATION_FAILED`, `GROUP_AUTHORIZATION_FAILED`, `CLUSTER_AUTHORIZATION_FAILED`, `TRANSACTIONAL_ID_AUTHORIZATION_FAILED` and the delegation token errors, stating whether the Java client and librdkafka retry, refresh metadata or surface the error as verified by ERR-F04; every code in the family passes the completeness check.

### [To Do] DOCS-0066 Error reference: Topic, config, reassignment and election errors
P1 · M9 · Task · deps: DOCS-0059, ERR-F04 · labels: docs
Done when: meaning and client action are written for `TOPIC_ALREADY_EXISTS`, `INVALID_PARTITIONS`, `INVALID_REPLICATION_FACTOR`, `INVALID_REPLICA_ASSIGNMENT`, `INVALID_CONFIG`, `POLICY_VIOLATION`, `TOPIC_DELETION_DISABLED`, `NO_REASSIGNMENT_IN_PROGRESS`, `ELECTION_NOT_NEEDED`, `PREFERRED_LEADER_NOT_AVAILABLE`, `THROTTLING_QUOTA_EXCEEDED`, `FEATURE_UPDATE_FAILED`, stating whether the Java client and librdkafka retry, refresh metadata or surface the error as verified by ERR-F04; every code in the family passes the completeness check.

### [To Do] DOCS-0067 Error reference: KRaft and inter-node errors
P1 · M9 · Task · deps: DOCS-0059, ERR-F04 · labels: docs
Done when: meaning and client action are written for `STALE_BROKER_EPOCH`, `BROKER_ID_NOT_REGISTERED`, `INCONSISTENT_CLUSTER_ID`, `INCONSISTENT_VOTER_SET`, `NOT_CONTROLLER`, `DUPLICATE_BROKER_REGISTRATION`, `CONTROLLER_ID_NOT_REGISTERED` and the voter-change errors, stating whether the Java client and librdkafka retry, refresh metadata or surface the error as verified by ERR-F04; every code in the family passes the completeness check.

### [To Do] DOCS-0068 Error reference: Share group, streams group, telemetry, fetch session and tiered storage errors
P1 · M9 · Task · deps: DOCS-0059, ERR-F04 · labels: docs
Done when: meaning and client action are written for the share session and record state errors, the streams topology errors (including `STREAMS_TOPOLOGY_DESCRIPTION_UPDATE_FAILED`), `UNKNOWN_SUBSCRIPTION_ID`, `TELEMETRY_TOO_LARGE`, the fetch session errors and `OFFSET_MOVED_TO_TIERED_STORAGE`, stating whether the Java client and librdkafka retry, refresh metadata or surface the error as verified by ERR-F04; every code in the family passes the completeness check.

### [To Do] DOCS-0069 Where Alluvium's errors differ from Kafka's
P1 · M9 · Task · deps: DOCS-0059, CFG-F11 · labels: docs
Done when: a page lists every case, each backed by an ADR, where Alluvium returns a different code or message than Kafka 4.4.0 (for example finite connection limits), and the check fails when a differential-test ignore rule for an error field has no entry here.

## [To Do] DOCS-F08 Operations guide
P1 · M9 · Feature · deps: OPS-F06
Scope: Deploy, scale, upgrade, monitor, troubleshoot.

### [To Do] DOCS-0070 Operations guide: Operations guide map and links to runbooks, capacity, backup and migration
P1 · M9 · Task · deps: OPS-F10, OPS-F09, OPS-F07 · labels: docs
Done when: `docs/site/src/operations/map.md` covers the operations part index ordered by task, linking the runbooks, capacity guide, backup guide, upgrade procedure and migration runbook without duplicating them, with every command as a tested snippet or taken from a tested OPS script, and passes the docs review checklist.

### [To Do] DOCS-0071 Operations guide: Deploying on hosts with the systemd packages
P1 · M9 · Task · deps: OPS-F05 · labels: docs
Done when: `docs/site/src/operations/hosts.md` covers installing packages, the file layout, configuring a broker and a controller, starting with systemd, and per-host OS settings, with every command as a tested snippet or taken from a tested OPS script, and passes the docs review checklist.

### [To Do] DOCS-0072 Operations guide: Deploying with containers and on Kubernetes
P1 · M9 · Task · deps: OPS-F03, OPS-F04 · labels: docs
Done when: `docs/site/src/operations/kubernetes.md` covers the image, compose examples, the Helm chart's topologies, storage classes, probes, security context and scraping, with every command as a tested snippet or taken from a tested OPS script, and passes the docs review checklist.

### [To Do] DOCS-0073 Operations guide: Hardware, operating system and filesystem tuning
P1 · M9 · Task · deps: OPS-F09, STOR-F17 · labels: docs · ref: docs/operations/hardware-and-os.md @4.4.0
Done when: `docs/site/src/operations/hardware.md` covers disks and filesystems (XFS, ext4, mount options), flush policy and the page cache, file descriptor and memory-map limits, network settings and clocks, with every command as a tested snippet or taken from a tested OPS script, and passes the docs review checklist.

### [To Do] DOCS-0074 Operations guide: Bootstrapping a cluster: formatting, standalone and initial controllers, adding nodes
P1 · M9 · Task · deps: SRV-F12, RAFT-F15 · labels: docs · ref: docs/operations/kraft.md Provisioning Nodes @4.4.0
Done when: `docs/site/src/operations/bootstrap.md` covers cluster ids, `alluvium format` modes, bootstrapping a standalone or multi-controller quorum, and formatting new brokers and controllers, with every command as a tested snippet or taken from a tested OPS script, and passes the docs review checklist.

### [To Do] DOCS-0075 Operations guide: Changing controller quorum membership
P1 · M9 · Task · deps: RAFT-F07, API-F80, API-F81, API-F94 · labels: docs · ref: docs/operations/kraft.md Controller membership changes @4.4.0
Done when: `docs/site/src/operations/quorum.md` covers static versus dynamic quorums, adding, removing and unregistering controllers, and replacing a controller disk, with every command as a tested snippet or taken from a tested OPS script, and passes the docs review checklist.

### [To Do] DOCS-0076 Operations guide: Expanding the cluster and reassigning partitions with throttles
P1 · M9 · Task · deps: CTRL-F10, REPL-F16 · labels: docs · ref: docs/operations/basic-kafka-operations.md Expanding your cluster @4.4.0
Done when: `docs/site/src/operations/scaling.md` covers adding brokers, generating and executing reassignments, monitoring progress, and throttling replication safely, with every command as a tested snippet or taken from a tested OPS script, and passes the docs review checklist.

### [To Do] DOCS-0077 Operations guide: Decommissioning brokers and log directories
P1 · M9 · Task · deps: CTRL-F22, STOR-F22 · labels: docs · ref: docs/operations/basic-kafka-operations.md Decommissioning brokers and log directories @4.4.0
Done when: `docs/site/src/operations/decommission.md` covers draining and unregistering a broker and moving replicas off a log directory, with every command as a tested snippet or taken from a tested OPS script, and passes the docs review checklist.

### [To Do] DOCS-0078 Operations guide: Topic operations
P1 · M9 · Task · deps: CTRL-F04, CFG-F04, API-F21 · labels: docs · ref: docs/operations/basic-kafka-operations.md Modifying topics @4.4.0
Done when: `docs/site/src/operations/topics.md` covers creating topics, changing configs, adding partitions and its effect on key ordering, increasing replication factor, and deleting records and topics, with every command as a tested snippet or taken from a tested OPS script, and passes the docs review checklist.

### [To Do] DOCS-0079 Operations guide: Leadership balancing, preferred leader election and rack awareness
P1 · M9 · Task · deps: CTRL-F08, CTRL-F07, API-F43 · labels: docs · ref: docs/operations/basic-kafka-operations.md Balancing leadership @4.4.0
Done when: `docs/site/src/operations/leadership.md` covers preferred leaders, automatic rebalancing, `kafka-leader-election.sh`, and rack-aware replica placement, with every command as a tested snippet or taken from a tested OPS script, and passes the docs review checklist.

### [To Do] DOCS-0080 Operations guide: Graceful shutdown, restarts and maintenance
P1 · M9 · Task · deps: SRV-F03, SRV-F13 · labels: docs · ref: docs/operations/basic-kafka-operations.md Graceful shutdown @4.4.0
Done when: `docs/site/src/operations/shutdown.md` covers controlled shutdown, what happens to leaders and in-flight requests, and safe maintenance using `roll-check`, with every command as a tested snippet or taken from a tested OPS script, and passes the docs review checklist.

### [To Do] DOCS-0081 Operations guide: Monitoring with metrics, dashboards and alerts
P1 · M9 · Task · deps: OPS-F11, OBS-F08 · labels: docs
Done when: `docs/site/src/operations/monitoring.md` covers scraping, the shipped dashboards and alert rules, severities, and translating existing Kafka JMX dashboards, with every command as a tested snippet or taken from a tested OPS script, and passes the docs review checklist.

### [To Do] DOCS-0082 Operations guide: Logging: runtime levels, output and shipping
P1 · M9 · Task · deps: OPS-F13, OBS-F02 · labels: docs
Done when: `docs/site/src/operations/logging.md` covers changing levels at runtime through `/debug/log-level` and `BROKER_LOGGER`, output modes, rotation and shipping, with every command as a tested snippet or taken from a tested OPS script, and passes the docs review checklist.

### [To Do] DOCS-0083 Operations guide: Troubleshooting toolkit
P1 · M9 · Task · deps: CLI-F05, SRV-F15, OBS-F08, CLI-F04 · labels: docs
Done when: `docs/site/src/operations/troubleshooting.md` covers the diagnostics endpoint, flight recorders, dump and verify tools, `debug-bundle`, and recording and replaying input journals, with every command as a tested snippet or taken from a tested OPS script, and passes the docs review checklist.

### [To Do] DOCS-0084 Operations guide: Managing consumer, share and streams groups
P1 · M9 · Task · deps: GRP-F08, SHARE-F06, STRM-F06 · labels: docs · ref: docs/operations/basic-kafka-operations.md Managing groups @4.4.0
Done when: `docs/site/src/operations/groups.md` covers describing, resetting offsets, deleting groups, protocol migration, and share and streams group tools, with every command as a tested snippet or taken from a tested OPS script, and passes the docs review checklist.

### [To Do] DOCS-0085 Operations guide: Managing transactions
P1 · M9 · Task · deps: TXN-F13 · labels: docs
Done when: `docs/site/src/operations/transactions.md` covers listing and describing transactions and producers, and finding and aborting hanging transactions, with every command as a tested snippet or taken from a tested OPS script, and passes the docs review checklist.

### [To Do] DOCS-0086 Operations guide: Operating quotas
P1 · M9 · Task · deps: QUOTA-F08 · labels: docs · ref: docs/operations/basic-kafka-operations.md Setting quotas @4.4.0
Done when: `docs/site/src/operations/quotas.md` covers setting user and client quotas, defaults, and observing throttling, with every command as a tested snippet or taken from a tested OPS script, and passes the docs review checklist.

### [To Do] DOCS-0087 Operations guide: Multiple log directories and log directory failures
P1 · M9 · Task · deps: STOR-F21, CTRL-F20 · labels: docs
Done when: `docs/site/src/operations/jbod.md` covers JBOD layout, directory assignment, failure handling and replacing a failed disk, with every command as a tested snippet or taken from a tested OPS script, and passes the docs review checklist.

### [To Do] DOCS-0088 Operations guide: Operating tiered storage
P1 · M9 · Task · deps: TIER-F08 · labels: docs · ref: docs/operations/tiered-storage.md @4.4.0
Done when: `docs/site/src/operations/tiered.md` covers enabling and disabling tiering per topic, remote storage configuration and monitoring, with every command as a tested snippet or taken from a tested OPS script, and passes the docs review checklist.

### [To Do] DOCS-0089 Operations guide: Changing configuration at runtime
P1 · M9 · Task · deps: CFG-F04 · labels: docs
Done when: `docs/site/src/operations/dynamic-config.md` covers per-broker and cluster-wide dynamic configs, what takes effect without restart, and config sources, with every command as a tested snippet or taken from a tested OPS script, and passes the docs review checklist.

## [To Do] DOCS-F09 Security guide
P1 · M4 · Feature · deps: SEC-F07
Scope: TLS, SASL, ACLs, secrets, hardening checklist.

### [To Do] DOCS-0090 Security guide: overview and secure defaults
P1 · M4 · Task · deps: none · labels: docs, security · ref: docs/security/security-overview.md @4.4.0
Done when: `docs/site/src/security/overview.md` covers the threat model summary, what is secure by default (loopback listeners, refused external plaintext, deny-by-default ACLs) and the ADR-0006 deviations from Kafka; every configuration example is exercised by a tested snippet or an existing compatibility test, and the page passes the docs review checklist.

### [To Do] DOCS-0091 Security guide: Listeners and security protocols
P1 · M4 · Task · deps: NET-F01 · labels: docs, security · ref: docs/security/listener-configuration.md @4.4.0
Done when: `docs/site/src/security/listeners.md` covers `listeners`, `advertised.listeners`, `listener.security.protocol.map`, inter-broker and controller listener names, and the plaintext opt-in; every configuration example is exercised by a tested snippet or an existing compatibility test, and the page passes the docs review checklist.

### [To Do] DOCS-0092 Security guide: Encryption with TLS
P1 · M4 · Task · deps: NET-F07 · labels: docs, security · ref: docs/security/encryption-and-authentication-using-ssl.md @4.4.0
Done when: `docs/site/src/security/tls.md` covers PEM and PKCS#12 key stores, protocol and cipher settings (rustls, TLS 1.2 and 1.3), hot reload and inter-broker hostname verification; every configuration example is exercised by a tested snippet or an existing compatibility test, and the page passes the docs review checklist.

### [To Do] DOCS-0093 Security guide: Client certificates and principal mapping
P1 · M4 · Task · deps: NET-F07, SEC-F01 · labels: docs, security
Done when: `docs/site/src/security/mtls.md` covers `ssl.client.auth` modes and `ssl.principal.mapping.rules` with worked examples; every configuration example is exercised by a tested snippet or an existing compatibility test, and the page passes the docs review checklist.

### [To Do] DOCS-0094 Security guide: SASL/PLAIN
P1 · M4 · Task · deps: SEC-F03 · labels: docs, security · ref: docs/security/authentication-using-sasl.md @4.4.0
Done when: `docs/site/src/security/sasl-plain.md` covers configuring PLAIN, why it requires TLS on non-loopback listeners, and credential storage; every configuration example is exercised by a tested snippet or an existing compatibility test, and the page passes the docs review checklist.

### [To Do] DOCS-0095 Security guide: SASL/SCRAM and credential management
P1 · M4 · Task · deps: SEC-F04, API-F51 · labels: docs, security
Done when: `docs/site/src/security/sasl-scram.md` covers bootstrapping SCRAM users at format time and managing them with `kafka-configs.sh` and AlterUserScramCredentials; every configuration example is exercised by a tested snippet or an existing compatibility test, and the page passes the docs review checklist.

### [To Do] DOCS-0096 Security guide: SASL/OAUTHBEARER with JWKS
P1 · M4 · Task · deps: SEC-F05 · labels: docs, security
Done when: `docs/site/src/security/oauthbearer.md` covers JWKS endpoints and caching, the expected audience and issuer settings required since Kafka 4.4.0, and clock-skew tolerance; every configuration example is exercised by a tested snippet or an existing compatibility test, and the page passes the docs review checklist.

### [To Do] DOCS-0097 Security guide: SASL/GSSAPI (Kerberos) support status
P2 · M4 · Task · deps: SEC-F06 · labels: docs, security
Done when: `docs/site/src/security/gssapi.md` covers the current support decision and the alternatives for Kerberos users; every configuration example is exercised by a tested snippet or an existing compatibility test, and the page passes the docs review checklist.

### [To Do] DOCS-0098 Security guide: Authorization with ACLs
P1 · M4 · Task · deps: SEC-F08, SEC-F09 · labels: docs, security · ref: docs/security/authorization-and-acls.md @4.4.0
Done when: `docs/site/src/security/acls.md` covers resource types, pattern types, operations per API, deny precedence, `super.users`, `allow.everyone.if.no.acl.found`, and `kafka-acls.sh` examples; every configuration example is exercised by a tested snippet or an existing compatibility test, and the page passes the docs review checklist.

### [To Do] DOCS-0099 Security guide: Delegation tokens
P1 · M4 · Task · deps: SEC-F10 · labels: docs, security
Done when: `docs/site/src/security/tokens.md` covers creating, renewing, expiring and describing tokens and using them from clients; every configuration example is exercised by a tested snippet or an existing compatibility test, and the page passes the docs review checklist.

### [To Do] DOCS-0100 Security guide: Re-authentication
P2 · M4 · Task · deps: SEC-F12 · labels: docs, security
Done when: `docs/site/src/security/reauth.md` covers `connections.max.reauth.ms` and client behavior during re-authentication (KIP-368); every configuration example is exercised by a tested snippet or an existing compatibility test, and the page passes the docs review checklist.

### [To Do] DOCS-0101 Security guide: Handling secrets
P1 · M4 · Task · deps: SEC-F11 · labels: docs, security
Done when: `docs/site/src/security/secrets.md` covers config providers for secrets, `password.encoder.secret` for dynamic passwords, file permissions, and redaction in logs and config dumps; every configuration example is exercised by a tested snippet or an existing compatibility test, and the page passes the docs review checklist.

### [To Do] DOCS-0102 Security guide: Securing inter-broker and controller traffic
P1 · M4 · Task · deps: SEC-F16 · labels: docs, security
Done when: `docs/site/src/security/inter-node.md` covers TLS and SASL on the inter-broker and controller listeners and inter-node principals; every configuration example is exercised by a tested snippet or an existing compatibility test, and the page passes the docs review checklist.

### [To Do] DOCS-0103 Security guide: Securing the diagnostics endpoint
P1 · M4 · Task · deps: OBS-F08 · labels: docs, security
Done when: `docs/site/src/security/diagnostics.md` covers loopback by default and mTLS or bearer-token authentication when bound elsewhere; every configuration example is exercised by a tested snippet or an existing compatibility test, and the page passes the docs review checklist.

### [To Do] DOCS-0104 Security guide: Enabling security on a running cluster
P1 · M4 · Task · deps: SEC-F16 · labels: docs, security · ref: docs/security/incorporating-security-features-in-a-running-cluster.md @4.4.0
Done when: `docs/site/src/security/running-cluster.md` covers rolling from PLAINTEXT to SASL_SSL listener by listener without downtime; every configuration example is exercised by a tested snippet or an existing compatibility test, and the page passes the docs review checklist.

### [To Do] DOCS-0105 Security guide: Auditing access with authorization events
P2 · M4 · Task · deps: SEC-F08, OBS-F07 · labels: docs, security
Done when: `docs/site/src/security/audit.md` covers `authz_decision` events, their levels, and shipping them to an audit store; every configuration example is exercised by a tested snippet or an existing compatibility test, and the page passes the docs review checklist.

### [To Do] DOCS-0106 Security guide: Production hardening checklist
P1 · M4 · Task · deps: SEC-F13 · labels: docs, security
Done when: `docs/site/src/security/hardening.md` covers a checklist covering listeners, TLS, SASL mechanisms, ACLs and super users, secrets, the diagnostics endpoint, file permissions and container or unit hardening, each item linked to its chapter; every configuration example is exercised by a tested snippet or an existing compatibility test, and the page passes the docs review checklist.

### [To Do] DOCS-0107 Security guide: Verifying release artifacts
P1 · M9 · Task · deps: SEC-F15, OPS-F03 · labels: docs, security
Done when: `docs/site/src/security/verifying-releases.md` covers checking `SHA256SUMS`, artifact and image signatures, provenance and SBOMs; every configuration example is exercised by a tested snippet or an existing compatibility test, and the page passes the docs review checklist.

## [To Do] DOCS-F10 Kafka compatibility and migration guide
P0 · M9 · Feature · deps: OPS-F08
Scope: Compatibility matrix, deviations, migration steps.

### [To Do] DOCS-0108 API compatibility matrix generated from the advertised version ranges
P0 · M9 · Task · deps: API-F18, DOCS-0002 · labels: docs
Done when: a generated table lists each API key with Kafka 4.4.0 versions, Alluvium advertised versions, listeners and status from the registry ApiVersions uses; CI fails when it disagrees with a live node's ApiVersions response or with docs/parity.md.

### [To Do] DOCS-0109 Client compatibility matrix
P0 · M9 · Task · deps: NFR-F08, TEST-F08 · labels: docs
Done when: the page lists supported clients and versions per the NFR-F08 policy with the latest compatibility test result for each, generated from the harness results.

### [To Do] DOCS-0110 Ecosystem tools matrix rendered from the ecosystem results
P0 · M9 · Task · deps: OPS-F12 · labels: docs
Done when: a page generated from `tests/compat/ecosystem/results.json` shows each tool, tested version, status and known gaps with Bug links; CI regenerates it.

### [To Do] DOCS-0111 Kafka command-line tools compatibility
P0 · M9 · Task · deps: CLI-F06 · labels: docs
Done when: a generated page lists each `kafka-*.sh` tool and command with its compatibility test status.

### [To Do] DOCS-0112 Deviations catalog
P0 · M9 · Task · deps: CFG-F11 · labels: docs
Done when: every behavior a Kafka user can observe that differs from Kafka 4.4.0 is listed with its ADR, what the user notices and the key that changes it; CI fails when an ADR marked as a deviation or a deviation-linked `alluvium.*` key is missing from the page.

### [To Do] DOCS-0113 Unsupported Kafka features and their replacements
P0 · M9 · Task · deps: CFG-F11 · labels: docs
Done when: the page covers ZooKeeper mode, message formats v0 and v1, Java plugins (authorizer, principal builder, quota callback, metric reporters, remote storage and config provider classes) with the Alluvium alternative for each, JMX replaced by Prometheus with the metric mapping, and log4j replaced by `ALLUVIUM_LOG`.

### [To Do] DOCS-0114 Kafka operator cheat sheet
P1 · M9 · Task · deps: CLI-F04, CLI-F05, DOCS-0010 · labels: docs
Done when: a table maps `kafka-server-start.sh`, `kafka-storage.sh format`, `kafka-dump-log.sh`, `kafka-metadata-shell.sh`, JMX and `log4j2.yaml` to `alluvium server`, `alluvium format`, `alluvium dump log`, `alluvium dump metadata`, `/metrics` and `ALLUVIUM_LOG`; each Alluvium command is a tested snippet.

### [To Do] DOCS-0115 Migration guide: choosing an approach and planning
P0 · M9 · Task · deps: OPS-F08 · labels: docs
Done when: the guide explains when to migrate with MirrorMaker 2, how to read the `alluvium migrate assess` report, the timeline and the go/no-go criteria, linking the cutover runbook instead of repeating it.

### [To Do] DOCS-0116 Migration guide: client and application changes
P0 · M9 · Task · deps: OPS-F08 · labels: docs
Done when: the guide covers bootstrap and security config changes for clients, Kafka Streams applications (internal topics, state, `application.id`) and Kafka Connect workers (internal topics, connector offsets).

### [To Do] DOCS-0117 Migration guide: users, credentials, ACLs and quotas
P0 · M9 · Task · deps: OPS-F08 · labels: docs, security
Done when: the guide covers copying ACLs and quotas with the migrate tools, re-provisioning SCRAM credentials, and mapping custom authorizers or principal builders to ACLs and mapping rules.

### [To Do] DOCS-0118 Migration guide: monitoring and operations changes
P1 · M9 · Task · deps: OPS-F11, OPS-F13 · labels: docs
Done when: the guide maps existing Kafka dashboards, alerts and log configuration to the shipped Alluvium dashboards, rules and logging setup, and lists operational tools that change.

### [To Do] DOCS-0119 Migration FAQ and known issues
P2 · M9 · Task · deps: DOCS-0115 · labels: docs
Done when: a FAQ answers recurring migration questions, each linking to the canonical page, and lists open known issues by Bug id.

## [To Do] DOCS-F11 Contributor deep dives
P1 · M0 · Feature · deps: none
Scope: Per-crate design docs as crates land.

### [To Do] DOCS-0120 Deep-dive template and crate index
P1 · M0 · Task · deps: none · labels: docs
Done when: docs/crates/README.md indexes crates in the order of the architecture code map; the template has sections for responsibility and layer, Kafka counterpart with parity citations, state machines (inputs, actions, states), invariants and where they are checked, events, metrics and flight recorder, error enum and Kafka error mapping, test strategy, and how to debug the crate.

### [To Do] DOCS-0121 Check that every workspace crate has a deep dive
P1 · M0 · Task · deps: DOCS-0120, FND-F04 · labels: docs
Done when: `cargo xtask docs check-crates` (part of `cargo xtask ci`) fails when a crate under `crates/` or `xtask` has no `docs/crates/<crate>.md` or is missing from architecture §4; `cargo xtask docs new-crate-doc <crate>` writes a marked stub that satisfies the check, and the check lists remaining stubs.

### [To Do] DOCS-0122 Contributor deep dive: `alluvium-types`
P1 · M0 · Task · deps: DOCS-0120, TYPE-F01, TYPE-F06, TYPE-F08 · labels: docs
Done when: docs/crates/alluvium-types.md replaces the stub and fills every template section from the crate's actual code and module docs; it links the ADRs and Kafka 4.4.0 sources the crate mirrors; a reviewer new to the crate confirms they can locate its state machine, invariants and tests from it.

### [To Do] DOCS-0123 Contributor deep dive: `xtask`
P1 · M0 · Task · deps: DOCS-0120, FND-F04, FND-F05 · labels: docs
Done when: docs/crates/xtask.md replaces the stub and fills every template section from the crate's actual code and module docs; it links the ADRs and Kafka 4.4.0 sources the crate mirrors; a reviewer new to the crate confirms they can locate its state machine, invariants and tests from it.

### [To Do] DOCS-0124 Contributor deep dive: `alluvium-protocol`
P1 · M1 · Task · deps: DOCS-0120, PROTO-F05, PROTO-F09 · labels: docs
Done when: docs/crates/alluvium-protocol.md replaces the stub and fills every template section from the crate's actual code and module docs; it links the ADRs and Kafka 4.4.0 sources the crate mirrors; a reviewer new to the crate confirms they can locate its state machine, invariants and tests from it.

### [To Do] DOCS-0125 Contributor deep dive: `alluvium-records`
P1 · M1 · Task · deps: DOCS-0120, REC-F04, REC-F10 · labels: docs
Done when: docs/crates/alluvium-records.md replaces the stub and fills every template section from the crate's actual code and module docs; it links the ADRs and Kafka 4.4.0 sources the crate mirrors; a reviewer new to the crate confirms they can locate its state machine, invariants and tests from it.

### [To Do] DOCS-0126 Contributor deep dive: `alluvium-storage`
P1 · M2 · Task · deps: DOCS-0120, STOR-F08, STOR-F09 · labels: docs
Done when: docs/crates/alluvium-storage.md replaces the stub and fills every template section from the crate's actual code and module docs; it links the ADRs and Kafka 4.4.0 sources the crate mirrors; a reviewer new to the crate confirms they can locate its state machine, invariants and tests from it.

### [To Do] DOCS-0127 Contributor deep dive: `alluvium-raft`
P1 · M2 · Task · deps: DOCS-0120, RAFT-F01, RAFT-F05 · labels: docs
Done when: docs/crates/alluvium-raft.md replaces the stub and fills every template section from the crate's actual code and module docs; it links the ADRs and Kafka 4.4.0 sources the crate mirrors; a reviewer new to the crate confirms they can locate its state machine, invariants and tests from it.

### [To Do] DOCS-0128 Contributor deep dive: `alluvium-metadata`
P1 · M2 · Task · deps: DOCS-0120, META-F03, META-F04 · labels: docs
Done when: docs/crates/alluvium-metadata.md replaces the stub and fills every template section from the crate's actual code and module docs; it links the ADRs and Kafka 4.4.0 sources the crate mirrors; a reviewer new to the crate confirms they can locate its state machine, invariants and tests from it.

### [To Do] DOCS-0129 Contributor deep dive: `alluvium-security`
P1 · M2 · Task · deps: DOCS-0120, SEC-F01 · labels: docs
Done when: docs/crates/alluvium-security.md replaces the stub and fills every template section from the crate's actual code and module docs; it links the ADRs and Kafka 4.4.0 sources the crate mirrors; a reviewer new to the crate confirms they can locate its state machine, invariants and tests from it.

### [To Do] DOCS-0130 Contributor deep dive: `alluvium-controller`
P1 · M2 · Task · deps: DOCS-0120, CTRL-F01 · labels: docs
Done when: docs/crates/alluvium-controller.md replaces the stub and fills every template section from the crate's actual code and module docs; it links the ADRs and Kafka 4.4.0 sources the crate mirrors; a reviewer new to the crate confirms they can locate its state machine, invariants and tests from it.

### [To Do] DOCS-0131 Contributor deep dive: `alluvium-replication`
P1 · M2 · Task · deps: DOCS-0120, REPL-F02, REPL-F04 · labels: docs
Done when: docs/crates/alluvium-replication.md replaces the stub and fills every template section from the crate's actual code and module docs; it links the ADRs and Kafka 4.4.0 sources the crate mirrors; a reviewer new to the crate confirms they can locate its state machine, invariants and tests from it.

### [To Do] DOCS-0132 Contributor deep dive: `alluvium-config`
P1 · M2 · Task · deps: DOCS-0120, CFG-F01, CFG-F02 · labels: docs
Done when: docs/crates/alluvium-config.md replaces the stub and fills every template section from the crate's actual code and module docs; it links the ADRs and Kafka 4.4.0 sources the crate mirrors; a reviewer new to the crate confirms they can locate its state machine, invariants and tests from it.

### [To Do] DOCS-0133 Contributor deep dive: `alluvium-observability`
P1 · M2 · Task · deps: DOCS-0120, OBS-F04, OBS-F06 · labels: docs
Done when: docs/crates/alluvium-observability.md replaces the stub and fills every template section from the crate's actual code and module docs; it links the ADRs and Kafka 4.4.0 sources the crate mirrors; a reviewer new to the crate confirms they can locate its state machine, invariants and tests from it.

### [To Do] DOCS-0134 Contributor deep dive: `alluvium-network`
P1 · M2 · Task · deps: DOCS-0120, NET-F02, NET-F05 · labels: docs
Done when: docs/crates/alluvium-network.md replaces the stub and fills every template section from the crate's actual code and module docs; it links the ADRs and Kafka 4.4.0 sources the crate mirrors; a reviewer new to the crate confirms they can locate its state machine, invariants and tests from it.

### [To Do] DOCS-0135 Contributor deep dive: `alluvium-server`
P1 · M2 · Task · deps: DOCS-0120, SRV-F01, SRV-F02 · labels: docs
Done when: docs/crates/alluvium-server.md replaces the stub and fills every template section from the crate's actual code and module docs; it links the ADRs and Kafka 4.4.0 sources the crate mirrors; a reviewer new to the crate confirms they can locate its state machine, invariants and tests from it.

### [To Do] DOCS-0136 Contributor deep dive: `alluvium`
P1 · M2 · Task · deps: DOCS-0120, CLI-F01 · labels: docs
Done when: docs/crates/alluvium.md replaces the stub and fills every template section from the crate's actual code and module docs; it links the ADRs and Kafka 4.4.0 sources the crate mirrors; a reviewer new to the crate confirms they can locate its state machine, invariants and tests from it.

### [To Do] DOCS-0137 Contributor deep dive: `alluvium-testkit`
P1 · M2 · Task · deps: DOCS-0120, TEST-F01, TEST-F04 · labels: docs
Done when: docs/crates/alluvium-testkit.md replaces the stub and fills every template section from the crate's actual code and module docs; it links the ADRs and Kafka 4.4.0 sources the crate mirrors; a reviewer new to the crate confirms they can locate its state machine, invariants and tests from it.

### [To Do] DOCS-0138 Contributor deep dive: `alluvium-group-coordinator`
P1 · M5 · Task · deps: DOCS-0120, GRP-F01, GRP-F04 · labels: docs
Done when: docs/crates/alluvium-group-coordinator.md replaces the stub and fills every template section from the crate's actual code and module docs; it links the ADRs and Kafka 4.4.0 sources the crate mirrors; a reviewer new to the crate confirms they can locate its state machine, invariants and tests from it.

### [To Do] DOCS-0139 Contributor deep dive: `alluvium-txn-coordinator`
P1 · M6 · Task · deps: DOCS-0120, TXN-F03, TXN-F04 · labels: docs
Done when: docs/crates/alluvium-txn-coordinator.md replaces the stub and fills every template section from the crate's actual code and module docs; it links the ADRs and Kafka 4.4.0 sources the crate mirrors; a reviewer new to the crate confirms they can locate its state machine, invariants and tests from it.

### [To Do] DOCS-0140 Contributor deep dive: `alluvium-share-coordinator`
P1 · M8 · Task · deps: DOCS-0120, SHARE-F02 · labels: docs
Done when: docs/crates/alluvium-share-coordinator.md replaces the stub and fills every template section from the crate's actual code and module docs; it links the ADRs and Kafka 4.4.0 sources the crate mirrors; a reviewer new to the crate confirms they can locate its state machine, invariants and tests from it.

## [To Do] DOCS-F12 API documentation publishing
P3 · M2 · Feature · deps: FND-F10
Scope: Publish rustdoc.

### [To Do] DOCS-0141 Workspace rustdoc build for publishing with a layer index page
P3 · M2 · Task · deps: none · labels: docs
Done when: `cargo xtask docs api` runs `cargo doc --workspace --no-deps --document-private-items` with `RUSTDOCFLAGS=-D warnings` and generates an index page listing crates by layer from xtask's layer table; docs/site records that crates are not published to crates.io or docs.rs.

### [To Do] DOCS-0142 Publish rustdoc under `/api/` alongside the docs site
P3 · M2 · Task · deps: DOCS-0141, DOCS-0006 · labels: docs
Done when: the publish workflow adds `/next/api/` and `/<X.Y>/api/`; deep dives link to rustdoc items and the link checker verifies those links.
