# Adding a Kafka feature

This is the path for implementing any Kafka API, API version, config or semantic. The goal is behavior that matches Kafka 4.4.0 exactly, built in a way the next person can debug.

## 1. Study Kafka first

Collect these before writing code, and link them in the PR:

| What | Where in Kafka at tag `4.4.0` |
|---|---|
| Message schema and version history | `clients/src/main/resources/common/message/<Api>Request.json` and `…Response.json`. The comments at the top explain each version bump |
| Broker request handling | `core/src/main/scala/kafka/server/KafkaApis.scala` → `handle<Api>Request` |
| Controller handling | `core/src/main/scala/kafka/server/ControllerApis.scala`, `metadata/src/main/java/org/apache/kafka/controller/` |
| Storage behavior | `storage/src/main/java/org/apache/kafka/storage/internals/log/` |
| Replication | `core/src/main/scala/kafka/cluster/Partition.scala`, `core/src/main/scala/kafka/server/ReplicaManager.scala` |
| Groups, transactions, share groups | `group-coordinator/`, `transaction-coordinator/`, `share-coordinator/` |
| Configs and defaults | `server-common/…/server/config/ServerConfigs.java` and `ServerLogConfigs.java`, `server/…/ReplicationConfigs.java`, `server/…/network/SocketServerConfigs.java`, `storage/…/LogConfig.java`, and the other `*Configs.java` classes |
| Error codes | `clients/src/main/java/org/apache/kafka/common/protocol/Errors.java` |
| Authorization per API | The `authorize(...)` calls inside the handler in `KafkaApis.scala` |
| Client expectations | The Java client's handling of the response (`clients/…/internals/`) shows which errors are retriable |
| Design rationale | The KIP(s) listed in the schema comments |

Write down the edge cases you find: each error code and the condition that triggers it, per-version differences, and timeouts. They become test cases.

## 2. Plan

- Update the API's row in [docs/parity.md](../parity.md) to **In progress**, with a link to the issue.
- If the change crosses crates, deviates from Kafka, or needs a new abstraction, write an ADR first.
- Build exactly what Kafka 4.4.0 does for this feature, no more. Extra options, abstractions or "nice to have" behavior make the system harder to test and debug ([more is not better](../../.agents/skills/alluvium-engineering/SKILL.md#more-is-not-better-braess-paradox)).
- Split the work into PRs that each leave `main` green and correct. A typical split is: codegen, then state machine, then driver and handler, then compatibility tests and advertising.

## 3. Build bottom-up

1. **Protocol**: run `cargo xtask codegen` for any new schemas. Do not hand-edit generated code.
2. **Domain logic** in the owning layer 1–2 crate, as a sans-I/O state machine or pure function:
   - Inputs and outputs (actions, transitions with `reason`) are explicit types.
   - Add `// Parity:` citations on every non-obvious rule.
   - Add unit tests for each edge case from step 1, property tests for invariants, and a simulation test if peers or timers are involved.
3. **Driver** in `alluvium-server`, if a new background component is needed: a supervised task, bounded inputs, transitions recorded and logged.
4. **Handler** in `alluvium-server/src/handlers/<api>.rs`:
   - Authorize exactly as Kafka does.
   - Decode, then call the domain logic.
   - Map errors per version.
   - Respect `throttle_time_ms` and quotas.
   - Implement every version in Kafka 4.4.0's `validVersions`. Version-specific logic is an explicit `match` on the version.
5. **Config**: new keys use Kafka's names, defaults and validation rules. Each key is documented with its source.
6. **Observability**: add request metrics (automatic for handlers), new events in `events.rs`, new fields in the [debuggability](../standards/debuggability.md) field table, and flight-recorder entries.
7. **Security review**: check input bounds, authorization, the secrets path, and that untrusted input cannot reach a panic.

## 4. Prove parity

- Write compatibility tests using at least the Java client and librdkafka.
- Write a differential test against Kafka 4.4.0 covering the happy path and each error case that can be triggered from a client.
- Only then add the API's versions to the `ApiVersions` response. Until then, clients see the API as unsupported, which is the correct and safe behavior.

## 5. Finish

- Mark the API **Done** in [docs/parity.md](../parity.md) with the supported version range.
- Update module docs, the [glossary](../glossary.md) for new terms, and the [architecture](../architecture.md) if any flows changed.
- Check the [definition of done](../../.agents/skills/alluvium-engineering/SKILL.md#definition-of-done).

## Improving on Kafka

Improvements ("superior" behavior) come only after the area reaches parity. They need an ADR and an opt-in config, and they must not change behavior that existing clients can observe under default settings. Differential tests keep running with the improvement turned off.
