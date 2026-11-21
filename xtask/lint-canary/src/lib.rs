//! Lint canary: code that breaks the workspace lints on purpose, to prove they fire.
//!
//! Layer: dev ([ADR-0009](../../../docs/adr/0009-crate-layering-and-dependency-rules.md)).
//! Nothing depends on this crate.
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
//! To cover a new rule, add a function that breaks it to the fixture module for its
//! topic, annotate the line, and run `cargo xtask lint-canary`.

#[cfg(feature = "canary")]
pub mod fixtures;
