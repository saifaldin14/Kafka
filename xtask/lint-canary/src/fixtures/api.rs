//! Public API hygiene: public items are documented, and `pub` means reachable.

pub fn missing_docs() {} // expect-lint: missing_docs

mod private {
    pub const UNREACHABLE_PUB: u32 = 1; // expect-lint: unreachable_pub
}

/// Reads the private module's constant, so it is not dead code.
pub fn reads_private() -> u32 {
    private::UNREACHABLE_PUB
}
