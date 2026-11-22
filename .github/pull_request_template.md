## What and why

<!-- What this changes and the reason. Name the backlog task(s), e.g. STOR-0031. -->

## Kafka reference

<!-- For Kafka behavior: links to Kafka 4.4.0 source (tag 4.4.0) and KIPs. Write "n/a" if none applies. -->

- **Closely ported Kafka code or tests:** <!-- "none", or the files. Each has the header and `// Ported from:` comments from CONTRIBUTING.md#attributing-kafka-material and is listed in NOTICE. -->

## Bug fixes only

- **Root cause:**
- **Why it escaped** (missing test, invariant or lint, and what this PR adds to close the gap):
- **Failing test that reproduces it:**

## Checklist

- [ ] Every addition is needed for the goal and is the best available choice; nothing is added "just in case" (see "More is not better" in the engineering skill)
- [ ] `cargo xtask ci` passes
- [ ] Tests at the right level (unit/property, simulation, fuzz, compatibility)
- [ ] Errors carry context; state transitions emit events with a `reason`; hot paths have metrics
- [ ] No new `unsafe`, unbounded queue, untimed network wait, unnamed task, logged secret, or input-reachable panic
- [ ] `// Parity:` citations added for Kafka-mirroring code
- [ ] Backlog task status updated; new work found along the way added as tasks; `cargo xtask backlog check` passes
- [ ] Docs describing the changed behavior are updated (module docs, architecture, standards, ADRs, glossary)
- [ ] New dependencies are justified (see `docs/standards/security.md#dependencies`)
