# Debugging guide

This is the procedure for investigating a bug, from first report to merged fix. The [debuggability standard](../standards/debuggability.md) describes the tools. This guide describes how to use them.

## 1. Capture the evidence

Before changing anything, collect:

- **Version and commit**, and the client library and version.
- **`alluvium debug-bundle`** from the affected nodes: redacted config with sources, metrics snapshot, flight recorders, log-dir listing.
- **Logs around the event**, filtered by `conn_id` and `correlation_id` (for one request) or `topic` and `partition` (for one partition).
- **The client's view**: the error code it received, its retries, its config (acks, idempotence, isolation level, timeouts).

## 2. Locate the component

Start from the symptom:

| Symptom | Look first at |
|---|---|
| Consumer doesn't see produced records | Partition `high_watermark` and `last_stable_offset` (`/debug/partitions/…`), consumer `isolation_level`, ISR size against `min.insync.replicas` |
| `NOT_LEADER_OR_FOLLOWER` / `FENCED_LEADER_EPOCH` loops | Metadata image version on each broker, `leader_epoch` events, partition flight recorder |
| Produce latency spikes | Request metrics by phase. High *remote time* means waiting on followers (replication lag); high *local time* means storage; high *queue time* means overloaded request handling |
| `NOT_ENOUGH_REPLICAS` | `isr_shrink` events and their `reason` |
| `OUT_OF_ORDER_SEQUENCE_NUMBER`, `INVALID_PRODUCER_EPOCH` | Producer state for the producer id (`dump producer-state`), transaction coordinator flight recorder |
| Rebalance storms | Group flight recorder: member joins and leaves with reasons (session timeout, poll timeout, epoch fencing) |
| Data missing after a restart | Recovery events (`log_recovery_truncated`), `alluvium verify log`, fsync policy |
| Stuck or slow node | `/debug/tasks`, queue depth metrics, `tokio-console` (development builds) |
| Crash | The panic report in the log: backtrace, span stack, flight recorder of the panicking component |

## 3. Narrow it down

- **Raise log detail for that module only**, at runtime, without restarting: `PUT /debug/log-level` with a filter such as `info,alluvium_replication=debug,alluvium_server::handlers::fetch=trace`. Restore it afterwards.
- **Read the flight recorder** of the affected partition, group or quorum. It shows the sequence of transitions and why each happened.
- **Inspect on-disk state** with `alluvium dump log`, `dump index` and `verify log`.
- **Compare with Kafka**: if the behavior is client-visible, run the scenario as a differential test. If Kafka behaves the same way, it isn't a parity bug. It may still be a bug in the client's expectations.

## 4. Reproduce deterministically

Pick the lowest level that reproduces the bug:

1. **Unit or property test** against the state machine, feeding it the inputs from the flight recorder or input journal.
2. **Simulation test** with a fault plan that recreates the conditions (for example, a network partition during leader election). Run seeds until it fails, then keep the seed.
3. **Compatibility test** if the bug depends on a specific client's behavior.

A bug that can't be reproduced isn't understood yet. If you're stuck, add the missing observability (event, field, metric) first. That's progress too.

## 5. Fix

1. Commit the failing test.
2. Find the **root cause**: the first point where reality diverged from the intended behavior, not the place it crashed.
3. Fix it in the component that owns the broken rule. Don't add a workaround in a caller.
4. Ask **why it escaped** and close that gap: an invariant check, a property, a simulation checker, a fuzz target, a lint, or a missing log field.
5. In the PR template, fill in the root cause, why it escaped, and the reproducing test.

## Fuzz crashes

A fuzz crash is a bug with its reproduction already attached: an input that makes a target panic, hang or allocate past its limits ([fuzz/README.md](../../fuzz/README.md)). Triage it like any other bug, in this order.

1. **Get the input.**
   - From a PR: download the `fuzz-crash-<target>` artifact of the failing run, with `gh run download <run-id> --name fuzz-crash-<target> --dir crash/`.
   - From the nightly run: the `Fuzz crash: <target>` issue names the run and the commit, and carries an input of up to 4 KiB base64-encoded. Decode it with `base64 -d > crash.bin`, or download the artifact as above.
2. **Reproduce it** at the commit the issue names: `cargo xtask fuzz <target> <input>`. That run uses the same limits as CI, so a timeout or allocation crash reproduces too.
3. **Minimize it**: `cargo +<nightly from fuzz/rust-toolchain.toml> fuzz tmin --fuzz-dir fuzz <target> <input>`. It writes `fuzz/artifacts/<target>/minimized-from-<hash>`. Check the result with `cargo xtask fuzz <target> <minimized>`, because `tmin` doesn't apply the standard limits.
4. **Find the root cause** (section 5). Decide whose rule is broken: the decoder's, or the target's assertion about the decoder. Read the Kafka source before deciding that a decoder is wrong.
5. **Write the failing unit test** in the crate that owns the broken rule, beside the code. Name it for the behavior, as every test is named (testing standard, rule 8), and put the crash file or issue in a comment. Commit it before the fix.
6. **Fix it**, then add the minimized input to `fuzz/seeds/<target>/` as `crash-<what it shows>`. Every later run then starts from it.
7. **Close the issue** from the PR that fixes it.

**A worked example.** Writing the `protocol_wire_primitives` target (PROTO-0050) turned up a crash within seconds.
- **Get and minimize the input.** It was a tagged-field section whose second tag was lower than its first: `13 02 ce 00 01 00 00 00`. The 0x13 selects the tagged-field reader; then come a count of 2, tag 78 with one byte, and tag 0 with none.
- **Reproduce it.** `cargo xtask fuzz protocol_wire_primitives <input>` failed at the target's round-trip assertion: `write_tagged_fields` refused the section the reader had accepted (`UnsortedRawTags`).
- **Find the root cause.** Kafka 4.4.0's `Readable.readUnknownTaggedField` appends tags in the order read and checks none, while `RawTaggedFieldWriter` writes only ascending tags. So the reader and writer were both right, and the target's assumption, that whatever decodes re-encodes, was wrong for this one reader.
- **Write the test.** `tags_are_read_in_any_order_as_kafka_reads_them` in `alluvium-protocol` pins the reading behavior, and `read_tagged_fields`'s doc now states it.
- **Fix it.** The target checks a section read out of order for the writer's refusal, instead of a round trip.
- **Seed it.** The input is the seed `fuzz/seeds/protocol_wire_primitives/crash-unordered-tags`.

## 6. Afterwards

- If the investigation took long because evidence was missing, open an issue to add that evidence (event, metric, flight-recorder detail, dump-tool output).
- If the same kind of bug could exist elsewhere, search for it, or open an issue listing where to look.
