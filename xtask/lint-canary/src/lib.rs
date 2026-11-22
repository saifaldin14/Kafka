//! Lint canary: code that breaks the workspace lints on purpose, to prove they fire.
//!
//! Layer: dev ([ADR-0009](../../../docs/adr/0009-crate-layering-and-dependency-rules.md)).
//! Nothing depends on this crate.
//!
//! Kafka counterpart: Kafka's checkstyle configuration (`checkstyle/` at 4.4.0), which
//! has no equivalent test of its own rules.
//!
//! A lint that is misspelled, removed or weakened in `[workspace.lints]` fails silently:
//! code simply stops being checked. The fixtures here catch that. Every line that should
//! trip a lint ends with an `expect-lint:` line comment naming the lints, as clippy's
//! JSON output spells them (`unsafe_code`, `clippy::todo`), separated by commas.
//! `cargo xtask lint-canary` runs clippy on this crate with the `canary` feature and
//! fails unless the diagnostics match the annotations exactly: each annotated lint fires
//! on its line, and nothing else fires anywhere.
//!
//! The fixtures compile only with the `canary` feature, so normal workspace builds and
//! `cargo xtask ci --step clippy` stay clean.
//!
//! # Sanctioned exceptions
//!
//! `fixtures::suppression` also holds the allowed ways around a lint, which must produce
//! no diagnostics at all:
//!
//! - A disallowed method is called only from its one sanctioned wrapper, which carries
//!   a single `#[expect(clippy::disallowed_methods, reason = "…")]`
//!   ([rust.md](../../../docs/standards/rust.md#workspace-configuration)).
//! - An invariant check is `expect("invariant: …")` under
//!   `#[expect(clippy::expect_used, reason = "…")]`
//!   ([rust.md](../../../docs/standards/rust.md#panics)).
//!
//! A bare `#[allow]`, or an `#[expect]` without a reason, is itself a fixture that must
//! fire.
//!
//! To cover a new rule, add a function that breaks it to the fixture module for its
//! topic, annotate the line, and run `cargo xtask lint-canary`. rustfmt moves a comment
//! that follows an opening `{` into the block, so keep each offending expression on a
//! line of its own.

#[cfg(feature = "canary")]
pub mod fixtures;
