# ADR-0013: Key hash space

- **Status:** Accepted
- **Date:** 2026-11-21

## Context

Elastic partitions (X-ELASTIC) split and merge a topic's partitions while keeping each key's records in order. They route a keyed record by a hash of its key and a mapping of hash ranges to physical partitions. The hash function and the bounds of its space must be fixed before any range is stored: ranges end up in metadata records (META-F08), in client metadata and in the lineage of sealed partitions, so changing either later would need a migration of every elastic topic.

Kafka already hashes keys, in the producer. `BuiltInPartitioner.partitionForKey` (`clients/src/main/java/org/apache/kafka/clients/producer/internals/BuiltInPartitioner.java` at tag `4.4.0`) computes `Utils.toPositive(Utils.murmur2(key)) % numPartitions`:

- `murmur2` is a 32-bit MurmurHash2 with Kafka's seed `0x9747b28c`;
- `toPositive` clears the sign bit, leaving a value in `[0, 2^31)`;
- the partition is that value modulo the partition count.

franz-go's default partitioner uses the same `murmur2`. librdkafka uses it only when configured with `partitioner=murmur2_random`; its default, `consistent_random`, hashes with CRC32. Records without a key are not hashed: the client's sticky partitioner places them.

`alluvium_types::murmur2` and `partition_for_key` reproduce this bit for bit. The tests check 10,000 hashes and 10,000 partition choices computed by Kafka 4.4.0 itself (TYPE-0039, TYPE-0040).

Kafka's partition choice is a modulo, so the keys of one Kafka partition are spread over the whole hash space: partition `p` of `N` holds the hashes congruent to `p` modulo `N`, not a contiguous range.

## Decision

1. **The key hash is Kafka's.** `KeyHash::of(key)` is `toPositive(murmur2(key))`, computed by `alluvium_types::murmur2`. It is the value every Kafka client computes, so a broker can compute it for any keyed record and agree with the producer that chose the partition.

2. **The space is `[0, 2^31)`.** A `KeyHash` is a `u32` below `2^31`. A `KeyRange` is half-open, `[start, end)`, with `end` up to `2^31`, so the full space is one range, `KeyRange::FULL = [0, 2^31)`, and an empty range is `start == end`. There are more than two billion points, so splitting stays possible long after any practical partition count.

3. **Null keys are never hashed.** A record with a null key has no `KeyHash` and belongs to no range by key: the native SDK and the broker place it on any active partition, as Kafka's sticky partitioner does. The empty key is a key: it hashes like any other byte string, as in Kafka.

4. **Hash ranges and Kafka partitions are separate domains.** Because Kafka partitions by modulo, a contiguous `KeyRange` is not a set of Kafka partitions, and no range layout can make Kafka's `partitionForKey` land on the right physical partition. So:
   - native elastic topics map `KeyRange`s to physical partitions;
   - the virtual partition view for Kafka clients (X-ELASTIC-F10) fixes a virtual partition count `N`, keeps Kafka's own `partitionForKey(key, N)` as the virtual partition, and maps ranges of virtual partition *indexes* to physical partitions. It never maps hash ranges.

5. **Later elastic ADRs may extend this but not change it.** X-ELASTIC-F01 owns the elastic design (mapping epochs, lineage, seal protocol, client modes). It may add to this decision, for example a range encoding in metadata records, but not change the hash function or the space: stored ranges and lineage depend on both.

Nothing here changes behavior visible to Kafka clients: `KeyHash` and `KeyRange` are internal types, and elastic topics are opt-in and off by default ([ADR-0010](0010-alluvium-is-its-own-platform.md)).

## Consequences

- Broker-side re-routing of Kafka producers' keyed records (X-ELASTIC-F07) hashes each key with the Java producer's function, so a record goes to the range a Java or franz-go producer would expect. A librdkafka producer on its default CRC32 partitioner chose its partition differently; re-routing still keeps each key in one place, but not the place that producer picked, which X-ELASTIC-F07's documentation must say.
- The golden tests of TYPE-0039 and TYPE-0040 are the compatibility guarantee: a change to `murmur2` that altered any Kafka-computed hash fails them before it reaches a stored range.
- A 31-bit MurmurHash2 is not the best-distributed hash available, and murmur2 is not resistant to chosen collisions. Ranges are split by load (X-ELASTIC-F12), not by assuming uniform hashes. A client that deliberately sends colliding keys only concentrates its own load on one range, which quotas already bound.
- TYPE-0042 implements `KeyHash` and `KeyRange` with these bounds; TYPE-0043 validates that a set of ranges covers the space exactly. X-ELASTIC-0089 keeps the virtual view over partition indexes.

## Alternatives considered

- **A wider hash, such as a 64-bit xxHash or MurmurHash3.** Better distribution and collision resistance, but it would differ from the hash every Kafka producer computes, so the broker couldn't tell which range a Kafka client meant, and re-routing would move records away from where their producers put them. The extra width buys nothing at realistic range counts.
- **Kafka's hash without `toPositive`, as a signed 32-bit space.** Same information, but ranges over a signed space wrap awkwardly around zero, and Kafka itself only ever uses the positive form.
- **Hash ranges for the virtual partition view.** Impossible to keep compatible: Kafka clients choose `hash % N`, which no contiguous range layout reproduces. Mapping partition indexes keeps the clients' own choice intact.
- **Hashing null keys as the empty key.** It would put every keyless record on one range and turn the sticky partitioner's spreading into a hot spot. Kafka never hashes null keys either.
