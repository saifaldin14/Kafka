//! Byte-slicing a `str`, which panics when the index is not on a char boundary.

/// Slicing by byte index.
pub fn string_slice(text: &str) -> &str {
    &text[..1] // expect-lint: clippy::string_slice
}
