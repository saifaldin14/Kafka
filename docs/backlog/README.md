# Alluvium backlog

This is the complete list of work for Alluvium, from the first commit to the last planned enhancement. Every item has a Jira-style status. Use this page to find out what to work on next, how to record progress, and how to add work.

- **Parity track (M0–M9):** everything needed for full compatibility with Apache Kafka 4.4.0, functional and non-functional. Foundations are built so that Kafka's limitations can be lifted later without rework. These tasks carry a `foundation-for:<epic>` label.
- **Enhancement track (M10–M18):** features that go beyond Kafka. Any change clients could notice is opt-in ([ADR-0010](../adr/0010-alluvium-is-its-own-platform.md)).

## Choosing the next task

Run:

```bash
cargo xtask backlog next          # the top 10 ready tasks
cargo xtask backlog next --all    # every ready task, in order
```

The tool applies these rules. If it isn't available, apply them by hand:

1. **Milestone first.** Find the lowest milestone that still has tasks not in `Done` or `Won't Do`. Work in that milestone comes first.
2. **Ready only.** A task is ready when its status is `To Do` and every dependency is satisfied (see [Dependencies](#dependencies)).
3. **Order:**
   - priority (`P0` first);
   - then build order (the number in the [epic table](#epics) and in the epic's file name);
   - then ID.
4. **Skip claimed work.** Tasks `In Progress` or `In Review` belong to someone.
5. **Work ahead only when blocked.** If no task in the current milestone is ready, take the first ready task from the next milestone, using the same ordering.
6. **Spikes come before what they decide.** Tasks of type `Spike` produce an ADR. Tasks that depend on them stay blocked until the ADR is accepted.

## Recording progress

| Status | Meaning | What to do |
|---|---|---|
| `To Do` | Not started | — |
| `In Progress` | Someone is working on it | Change the status and add `Owner: <name or agent>` below the `Done when` line |
| `In Review` | A PR is open | Add `PR: <link>` |
| `Blocked` | Cannot proceed | Add `Blocked: <reason and what unblocks it>` |
| `Done` | Merged, and the `Done when` condition holds | Keep the PR link |
| `Won't Do` | Deliberately dropped | Add `Won't do: <reason>`. Link an ADR if it's a design decision |

Update the status in the same PR as the work. A **feature's** status follows its tasks:
- `Done` when all its tasks are `Done` or `Won't Do`;
- `In Progress` once any task has started;
- otherwise `To Do`.

`cargo xtask backlog check` enforces this.

## Item format

Each epic has one file, named `<NN>-<code>-<slug>.md`, where NN is its build-order number. Inside, each feature is a `##` heading, and each task is a `###` heading under its feature. Each item has a metadata line, and each task has a `Done when` line:

```markdown
## [To Do] STOR-F04 Offset index
P0 · M2 · Feature · deps: STOR-F03
Scope: Sparse offset index (relative offset, position): lookup, append, truncation, sanity checks, resize.

### [To Do] STOR-0031 Offset index lookup: binary search for the greatest entry <= target offset
P0 · M2 · Task · deps: STOR-0030 · labels: decoder · ref: storage/…/OffsetIndex.java lookup @4.4.0
Done when: lookup returns the floor entry for any target; property test against a linear scan; empty and full index handled.
```

**Heading:** `[<Status>] <ID> <Title>`. The title is specific enough to act on without opening anything else.

**Metadata line:** fields separated by ` · `, in this order:

| Field | Values | Required |
|---|---|---|
| Priority | `P0` critical path · `P1` needed for the milestone · `P2` should have · `P3` nice to have | yes |
| Milestone | `M0`–`M18` ([table](#milestones)). A feature's milestone is the earliest milestone among its tasks | yes |
| Type | `Feature` · `Task` · `Spike` (research that ends in an ADR) · `Story` (user-visible slice) · `Bug` | yes |
| `deps:` | Comma-separated IDs, or `none` | yes |
| `labels:` | Comma-separated, lowercase: the labels in [`.github/labels.yml`](../../.github/labels.yml), or `foundation-for:<EPIC>`, which exists only in the backlog | no |
| `ref:` | The Kafka 4.4.0 source path and symbol, a KIP, or an ADR | no |

**`Done when:`** (tasks only) — an observable, testable completion condition. "Implemented" is not a condition.

**Optional trailing lines:** `Owner:`, `PR:`, `Blocked:`, `Won't do:`, `Notes:`.

### IDs

- Epic codes are listed in the [epic table](#epics).
- Features are `<EPIC>-Fnn`. In the API epic, the feature number is the Kafka API key, so `API-F00` is Produce.
- Tasks are `<EPIC>-nnnn`, four digits. They are numbered in the order they were added, so a task's place in the file shows its logical position, not its ID.
- **IDs are never reused or renumbered.** A dropped item becomes `Won't Do`. It is never deleted.

### Dependencies

- A dependency on a **task** is satisfied when that task is `Done` or `Won't Do`.
- A dependency on a **feature** is satisfied when every task of that feature whose milestone is at or before the dependent task's milestone is `Done` or `Won't Do`. Later-milestone tasks in a feature therefore never block earlier work.
- Dependencies across epics point at **features**. Dependencies within an epic may point at tasks.
- A task never depends on a task in a later milestone. `cargo xtask backlog check` rejects that, and it also rejects dependency cycles.
- Every task implicitly depends on its feature's `deps`.

## Granularity

A task is one reviewable change:
- one concern;
- ideally under ~400 changed lines, excluding generated code;
- with its own tests and doc updates (see the [definition of done](../../.agents/skills/alluvium-engineering/SKILL.md#definition-of-done)).

If a task turns out bigger, split it into new tasks, add them, and mark the original `Won't Do` with `Won't do: split into X, Y`.

## Adding work

1. Find the owning epic and feature. If none fits, add a feature to the right epic, using the next free `Fnn`.
2. Add the task under its feature in logical order, using the next free task number in that epic. `cargo xtask backlog stats` shows the highest numbers.
3. Fill in every required field, and write a `Done when` line that someone else could verify.
4. Run `cargo xtask backlog check` (CI runs it too).

Bugs found later are added as type `Bug` under the feature that owns the broken behavior.

Ownership rules that decide where a task goes:
- CFG defines config keys; other epics consume them.
- API owns Kafka request handlers; domain epics own the logic.
- TEST owns harnesses; other epics use them.
- CLI owns dump and verify commands for parity-track formats. Each enhancement epic owns the dump and verify tooling for the formats it introduces.

## Milestones

| Milestone | Scope | Exit criteria |
|---|---|---|
| **M0: Skeleton and tooling** | Workspace, toolchain, lints, dependency policy, `xtask`, CI, backlog tooling, `alluvium-types` | `cargo xtask ci` passes in CI; the layer check rejects a deliberately wrong dependency |
| **M1: Codecs** | Vendored schemas, code generator, all protocol codecs, record batches, CRC32C, compression, fuzz targets | Round-trip property tests and fuzzing clean; codecs decode traffic captured from real clients and match Kafka's golden bytes |
| **M2: Single-node broker** | Network, storage, single-voter KRaft, controller basics, server runtime, config, observability, dump tools, compatibility and differential harnesses | Java client (with `enable.idempotence=false` until M6) and librdkafka produce and consume against one node; `kafka-topics.sh` works |
| **M3: Multi-node cluster** | Multi-voter KRaft, broker lifecycle, replication, leader election, reassignment, simulation harness | Simulation passes across thousands of seeds; no acknowledged `acks=all` write lost under the fault plans |
| **M4: Security and quotas** | SASL, mTLS principals, ACL authorizer, delegation tokens, quotas and throttling | Authenticated clients and `kafka-acls.sh` work; authorization differential tests pass |
| **M5: Consumer groups** | Compaction, `__consumer_offsets`, classic and consumer group protocols, offset APIs | Groups rebalance correctly with Java and librdkafka; `kafka-consumer-groups.sh` works |
| **M6: Idempotence and transactions** | Producer state, transaction coordinator, markers, LSO, transactions v2 | An exactly-once Kafka Streams app runs correctly; transaction differential tests pass |
| **M7: Admin completeness** | Dynamic config, DeleteRecords, JBOD, remaining admin APIs | All `kafka-*.sh` admin tools work |
| **M8: Newer group types** | Share groups, streams groups, client telemetry | Compatibility tests with clients that support them |
| **M9: Tiered storage and parity GA** | KIP-405, upgrades, operations, large-scale and soak testing | Full parity proven; the release is production ready |
| **M10: Native APIs and schemas** | Native gRPC/HTTP APIs and SDK, built-in schema registry | Native SDK and Confluent serializers work against Alluvium |
| **M11: Queues and large messages** | Queue semantics, large messages | Queue invariants hold in simulation; large payloads flow with bounded memory |
| **M12: Elastic partitions and key lookups** | Split/merge with ordering, key index and lookups | Per-key order holds across split/merge in simulation; lookups match the log |
| **M13: Diskless topics** | Object-storage-native topics | Diskless topics pass the same compatibility tests as classic topics |
| **M14: Tenancy and advanced security** | Namespaces, isolation, encryption, ABAC, audit | Isolation holds under adversarial load; encryption and audit verified |
| **M15: Geo-replication** | Cluster links, mirror topics, failover | Failover and failback runbooks pass in simulation and in real clusters |
| **M16: Transforms and SQL** | WASM transforms, derived topics, SQL | Transforms are sandboxed and observable; SQL results verified |
| **M17: Autopilot and observability extras** | Balancing, self-healing, operator, tracing, lag in time | Autopilot keeps target balance under churn without SLO breaches |
| **M18: Low latency and embedded** | io_uring, group commit, latency gates, dev and embedded modes | Latency targets met; one-command dev mode works |

## Epics

Epics are numbered in the order they should be built. Each epic's file name starts with its number (`01-type-domain-types.md`), so a directory listing shows the build order. The number is also the tie-breaker when choosing the next task.

The order comes from a simulated run of the whole backlog: every task scheduled as soon as its dependencies allow (milestone first, then priority), and each epic ranked by when the middle of its work is built. Epics that span many milestones, such as TEST, DOCS and ERR, start early but are mostly built later. Renumber only when the plan changes substantially; `cargo xtask backlog check` verifies the numbers match the file names.

| # | Code | Epic | Milestones |
|---|---|---|---|
| 1 | TYPE | [Domain types](01-type-domain-types.md) | M0-M2 |
| 2 | FND | [Workspace, tooling and CI](02-fnd-foundation.md) | M0-M1 |
| 3 | PROTO | [Kafka protocol and code generation](03-proto-protocol.md) | M1-M2 |
| 4 | REC | [Record batches and compression](04-rec-records.md) | M1-M5 |
| 5 | META | [Metadata records and image](05-meta-metadata.md) | M2-M6 |
| 6 | TEST | [Test infrastructure](06-test-test-infrastructure.md) | M1-M9 |
| 7 | STOR | [Log storage](07-stor-storage.md) | M2-M7 |
| 8 | SRV | [Server runtime and request pipeline](08-srv-server.md) | M2-M7 |
| 9 | NET | [Network layer](09-net-network.md) | M2-M7 |
| 10 | OBS | [Observability](10-obs-observability.md) | M2-M9 |
| 11 | CTRL | [Controller](11-ctrl-controller.md) | M2-M7 |
| 12 | REPL | [Replication and partitions](12-repl-replication.md) | M2-M7 |
| 13 | NFR | [Non-functional requirements](13-nfr-non-functional.md) | M0-M9 |
| 14 | RAFT | [KRaft consensus](14-raft-kraft.md) | M2-M3 |
| 15 | CLI | [CLI and tool compatibility](15-cli-cli-and-tools.md) | M2-M9 |
| 16 | CFG | [Configuration](16-cfg-configuration.md) | M2-M9 |
| 17 | SEC | [Security](17-sec-security.md) | M2-M9 |
| 18 | DOCS | [User and operator documentation](18-docs-documentation.md) | M0-M9 |
| 19 | QUOTA | [Quotas and throttling](19-quota-quotas.md) | M4 |
| 20 | API | [Kafka API handlers](20-api-client-apis.md) | M2-M9 |
| 21 | ERR | [Kafka error codes](21-err-error-codes.md) | M1-M9 |
| 22 | GRP | [Group coordinator](22-grp-group-coordinator.md) | M5-M7 |
| 23 | TXN | [Idempotence and transactions](23-txn-transactions.md) | M6 |
| 24 | OPS | [Packaging and operations](24-ops-operations.md) | M2-M9 |
| 25 | STRM | [Streams groups](25-strm-streams-groups.md) | M8 |
| 26 | TELEM | [Client telemetry](26-telem-client-telemetry.md) | M8 |
| 27 | SHARE | [Share groups](27-share-share-groups.md) | M8-M9 |
| 28 | TIER | [Tiered storage](28-tier-tiered-storage.md) | M9 |
| 29 | X-SCHEMA | [Built-in schema registry and validation](29-x-schema-schema-registry.md) | M10 |
| 30 | X-NATIVE | [Native HTTP and gRPC APIs and SDK](30-x-native-native-apis.md) | M10 |
| 31 | X-LARGE | [Large messages](31-x-large-large-messages.md) | M11 |
| 32 | X-QUEUE | [Queue semantics](32-x-queue-queues.md) | M11 |
| 33 | X-ELASTIC | [Elastic partitions](33-x-elastic-elastic-partitions.md) | M12 |
| 34 | X-KV | [Key lookups and queryable topics](34-x-kv-key-lookups.md) | M12-M13 |
| 35 | X-DISKLESS | [Diskless object-storage-native topics](35-x-diskless-diskless.md) | M13 |
| 36 | X-SECX | [Advanced security](36-x-secx-advanced-security.md) | M14 |
| 37 | X-TENANT | [Multi-tenancy](37-x-tenant-multi-tenancy.md) | M14 |
| 38 | X-GEO | [Geo-replication](38-x-geo-geo-replication.md) | M15 |
| 39 | X-SQL | [Built-in SQL and stream processing](39-x-sql-sql.md) | M16-M17 |
| 40 | X-WASM | [Server-side filtering and WASM transforms](40-x-wasm-transforms.md) | M16 |
| 41 | X-AUTO | [Operational autopilot](41-x-auto-autopilot.md) | M17 |
| 42 | X-OBSX | [Observability extras](42-x-obsx-observability-extras.md) | M17 |
| 43 | X-LOWLAT | [Low-latency mode](43-x-lowlat-low-latency.md) | M18 |
| 44 | X-EMBED | [Embedded and lightweight mode](44-x-embed-embedded-mode.md) | M18 |

## Totals

Generated by `cargo xtask backlog stats --write`. Don't edit by hand.

<!-- backlog-stats:start -->
| Epic | Features | Tasks | Done | Open | Highest task id |
|---|---|---|---|---|---|
| TYPE | 12 | 48 | 48 | 0 | TYPE-0048 |
| FND | 11 | 69 | 69 | 0 | FND-0069 |
| PROTO | 15 | 156 | 34 | 122 | PROTO-0156 |
| REC | 15 | 157 | 0 | 157 | REC-0157 |
| META | 9 | 131 | 0 | 131 | META-0131 |
| TEST | 17 | 131 | 1 | 130 | TEST-0131 |
| STOR | 25 | 279 | 0 | 279 | STOR-0279 |
| SRV | 16 | 149 | 0 | 149 | SRV-0149 |
| NET | 15 | 128 | 0 | 128 | NET-0128 |
| OBS | 13 | 154 | 0 | 154 | OBS-0154 |
| CTRL | 23 | 191 | 0 | 191 | CTRL-0191 |
| REPL | 17 | 136 | 0 | 136 | REPL-0136 |
| NFR | 13 | 67 | 1 | 66 | NFR-0067 |
| RAFT | 15 | 159 | 0 | 159 | RAFT-0159 |
| CLI | 7 | 170 | 0 | 170 | CLI-0170 |
| CFG | 12 | 507 | 0 | 507 | CFG-0507 |
| SEC | 16 | 141 | 0 | 141 | SEC-0141 |
| DOCS | 12 | 143 | 5 | 138 | DOCS-0143 |
| QUOTA | 10 | 66 | 0 | 66 | QUOTA-0066 |
| API | 91 | 1136 | 0 | 1136 | API-1136 |
| ERR | 4 | 189 | 0 | 189 | ERR-0189 |
| GRP | 13 | 182 | 0 | 182 | GRP-0182 |
| TXN | 16 | 137 | 0 | 137 | TXN-0137 |
| OPS | 13 | 213 | 0 | 213 | OPS-0213 |
| STRM | 7 | 65 | 0 | 65 | STRM-0065 |
| TELEM | 5 | 33 | 0 | 33 | TELEM-0033 |
| SHARE | 11 | 110 | 0 | 110 | SHARE-0110 |
| TIER | 10 | 97 | 0 | 97 | TIER-0097 |
| X-SCHEMA | 16 | 191 | 0 | 191 | X-SCHEMA-0191 |
| X-NATIVE | 18 | 207 | 0 | 207 | X-NATIVE-0207 |
| X-LARGE | 10 | 75 | 0 | 75 | X-LARGE-0075 |
| X-QUEUE | 15 | 111 | 0 | 111 | X-QUEUE-0111 |
| X-ELASTIC | 16 | 143 | 0 | 143 | X-ELASTIC-0143 |
| X-KV | 11 | 90 | 0 | 90 | X-KV-0090 |
| X-DISKLESS | 15 | 139 | 0 | 139 | X-DISKLESS-0139 |
| X-SECX | 11 | 173 | 0 | 173 | X-SECX-0173 |
| X-TENANT | 12 | 97 | 0 | 97 | X-TENANT-0097 |
| X-GEO | 13 | 123 | 0 | 123 | X-GEO-0123 |
| X-SQL | 11 | 114 | 0 | 114 | X-SQL-0114 |
| X-WASM | 11 | 110 | 0 | 110 | X-WASM-0110 |
| X-AUTO | 12 | 162 | 0 | 162 | X-AUTO-0162 |
| X-OBSX | 11 | 143 | 0 | 143 | X-OBSX-0143 |
| X-LOWLAT | 10 | 112 | 0 | 112 | X-LOWLAT-0112 |
| X-EMBED | 7 | 70 | 0 | 70 | X-EMBED-0070 |
| **Total** | **642** | **7204** | **158** | **7046** | |

| Milestone | Tasks | Done | Open |
|---|---|---|---|
| M0 | 119 | 119 | 0 |
| M1 | 278 | 38 | 240 |
| M2 | 1656 | 1 | 1655 |
| M3 | 557 | 0 | 557 |
| M4 | 581 | 0 | 581 |
| M5 | 482 | 0 | 482 |
| M6 | 371 | 0 | 371 |
| M7 | 170 | 0 | 170 |
| M8 | 519 | 0 | 519 |
| M9 | 411 | 0 | 411 |
| M10 | 398 | 0 | 398 |
| M11 | 186 | 0 | 186 |
| M12 | 232 | 0 | 232 |
| M13 | 140 | 0 | 140 |
| M14 | 270 | 0 | 270 |
| M15 | 123 | 0 | 123 |
| M16 | 221 | 0 | 221 |
| M17 | 308 | 0 | 308 |
| M18 | 182 | 0 | 182 |
<!-- backlog-stats:end -->

## Searching

```bash
grep -rn '^### \[To Do\]' docs/backlog/*.md           # all open tasks
grep -rn '^### \[In Progress\]' docs/backlog/*.md     # work underway
grep -rn 'foundation-for:X-ELASTIC' docs/backlog/*.md  # groundwork for an enhancement
grep -rn ' · M2 · ' docs/backlog/07-stor-storage.md       # a milestone within an epic
cargo xtask backlog show STOR-0031                     # one item with its dependencies
```
