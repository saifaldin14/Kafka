<!-- stub: fill every section from the code, then delete this line -->
# `xtask`: deep dive

<!-- Fill every section from the code as it is. Write "None" for a section that doesn't apply, with the reason.
Link to code, ADRs and Kafka sources rather than copying them. Delete these comments. -->

## Responsibility and layer

What the crate owns, in two or three sentences, and what it deliberately leaves to other crates. Its layer in the [code map](../architecture.md#4-code-map) and the crates it depends on.

## Kafka counterpart

The Kafka 4.4.0 classes this crate mirrors, as links to the source at tag `4.4.0`, and the KIPs it implements. Where the code reproduces Kafka behavior, name the `// Parity:` citations a reader should start from. List every deliberate difference from Kafka with its ADR.

## State machines

For each state machine:

- **States:** the type that holds them, and what each state means.
- **Inputs:** the events it accepts (requests, timer ticks, replies, records).
- **Actions:** what it returns for its driver to carry out.
- **Driver:** where the I/O happens ([ADR-0002](../adr/0002-sans-io-state-machines-driven-by-tokio.md)).

A small diagram of the main transitions helps when there are more than a few states.

## Invariants

Each invariant the crate maintains, and where it is checked: an `expect("invariant: …")`, a debug assertion, a property test or a simulation checker. A reader should be able to go from each invariant to the line that enforces it.

## Events, metrics and flight recorder

The events the crate's transitions emit, with their `reason` enums; the metrics it exports; and what its flight recorder keeps ([debuggability standard](../standards/debuggability.md#events-for-state-transitions)).

## Errors

The crate's error enum, what each variant carries, and the one exhaustive mapping to Kafka error codes ([ADR-0004](../adr/0004-error-model-and-panic-policy.md)).

## Tests

Where each [test level](../standards/testing.md#test-levels) lives for this crate (unit, property, simulation, fuzz, compatibility) and what each covers. How to run just this crate's tests, and how to replay a failing simulation seed.

## Debugging

Starting points for the common symptoms that lead here: which logs and fields to filter on, which `/debug` routes, flight recorders and `dump` commands to read, and the questions to ask first. The general procedure is in the [debugging guide](../guides/debugging.md).
