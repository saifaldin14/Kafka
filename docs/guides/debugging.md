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

## 6. Afterwards

- If the investigation took long because evidence was missing, open an issue to add that evidence (event, metric, flight-recorder detail, dump-tool output).
- If the same kind of bug could exist elsewhere, search for it, or open an issue listing where to look.
