//! The crate layer table: which layer each crate belongs to.
//!
//! [`LAYERS`] must list exactly the crates in the code-map table of
//! docs/architecture.md §4, with the same layers. `cargo xtask check-layers` enforces
//! this, and uses the table to check the dependency rules of
//! [ADR-0009](../../docs/adr/0009-crate-layering-and-dependency-rules.md).
//!
//! The table includes crates that don't exist yet: it is data, and a crate's layer is
//! decided before its first code lands.

use std::fmt;

/// A crate's position in the dependency order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Layer {
    /// Layers 0 (codecs and domain types) to 5 (the binary).
    Numbered(u8),
    /// Test and tooling crates: may depend on anything; only tests may depend on them.
    Dev,
}

impl Layer {
    /// The binary's layer, which nothing may depend on.
    pub(crate) const BINARY: Layer = Layer::Numbered(5);

    /// True for layers 0 to 2, which must stay free of async runtimes and networking.
    pub(crate) fn is_runtime_free(self) -> bool {
        matches!(self, Layer::Numbered(n) if n <= 2)
    }

    /// Parses the layer column of docs/architecture.md §4 (`0` to `5`, or `dev`).
    pub(crate) fn parse(text: &str) -> Option<Layer> {
        match text {
            "dev" => Some(Layer::Dev),
            _ => text.parse().ok().filter(|n| *n <= 5).map(Layer::Numbered),
        }
    }
}

impl fmt::Display for Layer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Layer::Numbered(n) => write!(f, "{n}"),
            Layer::Dev => write!(f, "dev"),
        }
    }
}

/// Every crate and its layer, in the order of docs/architecture.md §4.
pub(crate) const LAYERS: &[(&str, Layer)] = &[
    ("alluvium-types", Layer::Numbered(0)),
    ("alluvium-protocol", Layer::Numbered(0)),
    ("alluvium-records", Layer::Numbered(0)),
    ("alluvium-storage", Layer::Numbered(1)),
    ("alluvium-raft", Layer::Numbered(1)),
    ("alluvium-metadata", Layer::Numbered(1)),
    ("alluvium-capture", Layer::Numbered(1)),
    ("alluvium-security", Layer::Numbered(1)),
    ("alluvium-controller", Layer::Numbered(2)),
    ("alluvium-replication", Layer::Numbered(2)),
    ("alluvium-group-coordinator", Layer::Numbered(2)),
    ("alluvium-txn-coordinator", Layer::Numbered(2)),
    ("alluvium-share-coordinator", Layer::Numbered(2)),
    ("alluvium-config", Layer::Numbered(3)),
    ("alluvium-observability", Layer::Numbered(3)),
    ("alluvium-network", Layer::Numbered(3)),
    ("alluvium-server", Layer::Numbered(4)),
    ("alluvium", Layer::Numbered(5)),
    ("alluvium-testkit", Layer::Dev),
    ("xtask", Layer::Dev),
    ("lint-canary", Layer::Dev),
];

/// The layer of `crate_name`, if it is in the table.
pub(crate) fn layer_of(crate_name: &str) -> Option<Layer> {
    LAYERS
        .iter()
        .find(|(name, _)| *name == crate_name)
        .map(|(_, layer)| *layer)
}

#[cfg(test)]
mod tests {
    use super::{LAYERS, Layer, layer_of};

    #[test]
    fn every_crate_appears_once() {
        for (index, (name, _)) in LAYERS.iter().enumerate() {
            assert!(
                LAYERS[index + 1..].iter().all(|(other, _)| other != name),
                "{name} is listed twice"
            );
        }
    }

    #[test]
    fn layers_parse_and_display_round_trip() {
        for text in ["0", "3", "5", "dev"] {
            assert_eq!(
                Layer::parse(text).map(|layer| layer.to_string()).as_deref(),
                Some(text)
            );
        }
        assert_eq!(Layer::parse("6"), None);
        assert_eq!(Layer::parse("x"), None);
    }

    #[test]
    fn only_layers_0_to_2_are_runtime_free() {
        assert!(Layer::Numbered(2).is_runtime_free());
        assert!(!Layer::Numbered(3).is_runtime_free());
        assert!(!Layer::Dev.is_runtime_free());
        assert_eq!(layer_of("xtask"), Some(Layer::Dev));
        assert_eq!(layer_of("nope"), None);
    }
}
