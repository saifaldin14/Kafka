# ADR-0007: Record batches stored in wire format

- **Status:** Accepted
- **Date:** 2026-11-20

## Context

Kafka stores record batches on disk in exactly the format clients send and receive (record batch v2, "magic 2"). This allows:

- serving fetches with `sendfile` (zero-copy) on plaintext connections;
- passing CRCs through end to end;
- avoiding re-encoding on the hot path.

Alluvium needs the same properties to match Kafka's performance.

The rest of Kafka's on-disk layout (segment file naming, index formats, checkpoint files) is an internal detail. Being compatible with it would only matter for reusing Kafka log directories, which is not a goal (ADR-0010).

## Decision

1. **Record batches are stored byte-for-byte as record batch v2**, including the CRC and compression. The broker assigns offsets by rewriting the batch base offset, and the leader epoch (and, for `LogAppendTime`, the timestamp) as Kafka does. Records are never re-encoded unless validation requires it, exactly when Kafka's `LogValidator` would.
2. **Only magic v2 is accepted** (Kafka 4.0+, KIP-724).
3. **The rest of the on-disk layout is ours.** It starts Kafka-like (segment `.log` files plus offset and time indexes) and is documented in `docs/formats/` with a format version. The specific formats are decided in an ADR during M2.
4. **Every on-disk format has a dump and verify tool** (ADR-0005).

## Consequences

- The fetch path can be zero-copy on plaintext connections.
- Storage must handle untrusted bytes. Batches are validated on append, and recovery re-validates them (see [security](../standards/security.md#durability-as-a-security-property)).
- Kafka log directories can't be mounted directly into Alluvium. Migration happens through replication tools such as MirrorMaker 2.

## Alternatives considered

- **Our own internal record format.** Rejected: costs a conversion on every produce and fetch, and loses zero-copy.
- **Full Kafka on-disk compatibility.** Rejected: constrains storage improvements for a migration path that replication already covers.
