# ADR-0017: A per-decode allocation budget

- **Status:** Accepted
- **Date:** 2026-11-22

## Context

Kafka bounds what one request can make the broker allocate in two ways:
- the frame itself, capped by `socket.request.max.bytes` (default 100 MiB, [`SocketServerConfigs`](https://github.com/apache/kafka/blob/4.4.0/server/src/main/java/org/apache/kafka/network/SocketServerConfigs.java));
- each declared count, which its generated readers check against the bytes left and against [`MessageUtil`](https://github.com/apache/kafka/blob/4.4.0/clients/src/main/java/org/apache/kafka/common/protocol/MessageUtil.java)'s `MAX_ARRAY_LENGTH` (1,000,000) and `MAX_TAGGED_FIELD_COUNT` (10,000). Alluvium makes the same checks (PROTO-0106 and PROTO-0109).

Nothing bounds the total memory the decoded message takes. A decoded element can be many times larger in memory than on the wire. A one-byte struct becomes a struct holding, at least, a vector for its unknown tagged fields. Arrays nested inside array elements multiply the effect. One frame of the maximum size can therefore decode into gigabytes. Kafka runs that decode until the JVM runs out of heap.

The [security standard](../standards/security.md#untrusted-input) requires every length to be bounded by a configured limit before allocating, and design principle 4 of the [architecture](../architecture.md#3-design-principles) requires every per-client allocation to be bounded.

## Decision

1. Each decode has an **allocation budget**: `DecodeLimits::allocation_budget`, in bytes. Every heap allocation a decode makes is charged against it *before* it is made:
   - an array of `n` elements of type `T` is charged `n × size_of::<T>()`, at least one byte per element, when its count is read;
   - a string is charged its decoded UTF-8 length;
   - a tagged-field section is charged its count × `size_of::<RawTaggedField>()`.

   Byte, records and tagged-field values are slices of the frame, which the network layer has already accounted for, so they cost nothing.
2. A charge that exceeds what is left fails the decode with `DecodeErrorKind::LimitExceeded { limit: AllocationBudget, requested, allowed }`, where `allowed` is the budget left. That is a request Alluvium can't parse, and it is handled like every other: the connection is closed (SRV-0036).
3. The default budget is **100 MiB (104,857,600 bytes)**, Kafka's default `socket.request.max.bytes`. The server sets each request's budget to its listener's `socket.request.max.bytes` (SRV-0036), so:
   - an operator who allows larger frames allows larger decodes;
   - a request's memory is at most its frame plus that much again;
   - no new configuration key is needed.
4. This is a deviation from Kafka, of the kind [ADR-0006](0006-secure-defaults.md) allows: a resource limit, not a change in protocol behavior. A request is rejected only if its decoded collections and strings exceed the size of the largest frame the listener accepts. In real requests the bulk of a large frame is records, which are not charged. Arrays are capped at 1,000,000 elements, and a request near that is far below the budget unless its elements are large. No Kafka client sends such a request.

## Consequences

- Decoding memory is bounded by a number an operator can reason about. The fuzz targets can check every input against it (PROTO-0134).
- A decode never holds more than its budget, with one brief exception. When an array has decoded its first 1000 elements (Kafka's preallocation cap), the rest of its charged count is allocated in one step, and those first elements are copied across. Only during that copy are both buffers held. Doubling instead would hold half the array twice over, and could go over the budget.
- The `Reader` is not `Clone`: a clone would duplicate the budget left.
- Generated decoders must allocate only through the `Reader`'s array, string and tagged-field readers, so that every allocation is charged. PROTO-0112 enforces this with a generator test.
- A decode's allocation is now testable. `tests/allocation.rs` in `alluvium-protocol` measures peak heap use with a counting allocator and asserts it stays within the budget.
- PROTO-0114 lists the budget beside Kafka's limits in the security standard.

## Alternatives considered

- **No budget, as in Kafka.** This leaves the memory exhaustion the security standard rules out.
- **A budget as a multiple of the frame length.** It needs a ratio between wire size and memory size. That ratio is exactly what an attacker controls, and no single value bounds hostile input while accepting every valid one.
- **A new configuration key for the budget.** It adds a key operators would need to keep in step with `socket.request.max.bytes`, to express what the frame limit already says.
- **Lower defaults for Kafka's per-count limits.** These would reject valid requests Kafka accepts, and still not bound the total, because the counts multiply.
