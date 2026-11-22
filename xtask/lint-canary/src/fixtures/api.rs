//! Public API hygiene: public items are documented and printable with `{:?}`, `pub`
//! means reachable, and paths are no longer than the prelude needs.

pub fn missing_docs() {} // expect-lint: missing_docs

mod private {
    pub const UNREACHABLE_PUB: u32 = 1; // expect-lint: unreachable_pub
}

/// Reads the private module's constant, so it is not dead code.
pub fn reads_private() -> u32 {
    private::UNREACHABLE_PUB
}

/// A public type that can't be printed with `{:?}`.
pub struct MissingDebug; // expect-lint: missing_debug_implementations

/// A path spelled out although the prelude has the name.
pub fn unused_qualifications() -> Vec<u8> {
    std::vec::Vec::new() // expect-lint: unused_qualifications
}
