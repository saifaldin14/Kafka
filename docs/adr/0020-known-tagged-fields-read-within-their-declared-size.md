# ADR-0020: Known tagged fields are read within their declared size

- **Status:** Accepted
- **Date:** 2026-11-22

## Context

In a flexible version, a struct ends with a tagged-field section. Each field there is a tag, a size, and a value of that size.

Kafka's generated readers handle a tag they know by reading the field's value straight from the input. They never compare what they read with the declared size ([`MessageDataGenerator.generateClassReader`](https://github.com/apache/kafka/blob/4.4.0/generator/src/main/java/org/apache/kafka/message/MessageDataGenerator.java)). A size that doesn't match the value therefore goes unnoticed:
- The read can run into the next field, or stop short of the declared bytes.
- Decoding then fails later, at some other field, or succeeds with the wrong values.

Correct clients always write the size of the value they wrote, so only malformed input can reach this case. For such input, Kafka's error, if there is one, points away from the actual fault.

## Decision

A generated decoder reads a known tagged field from a sub-reader that holds exactly the declared bytes, through `Reader::read_tagged_value`.

A value that needs more than its declared bytes fails as `Truncated`. One that leaves some unread fails as `TaggedFieldSizeMismatch { tag, declared, read }`. Both are reported at the field's offset, with its field path.

Unknown tags are unaffected: they are kept raw, as in Kafka.

## Consequences

- Malformed input fails at the field that is wrong, with an error that says so. A tagged field also can't read into the bytes that follow it, which bounds what a fuzzer can make one field's decoder consume.
- Well-formed input decodes exactly as in Kafka.
- [docs/parity.md](../parity.md) lists this as a deviation for malformed input.
- The decode tests check both directions of a mismatch.

## Alternatives considered

- **Read as Kafka does, ignoring the size.** Keeps Kafka's exact failure points for malformed input, but makes errors misleading and lets one field's decoder read another's bytes. No correct client depends on this behavior.
- **Skip to the declared end after reading, without an error.** Hides malformed input instead of reporting it, and would accept values Kafka rejects.
