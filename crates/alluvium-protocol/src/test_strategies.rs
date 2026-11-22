//! Helpers the generated `arbitrary` strategies use, so each field's strategy is one
//! short expression: whether the field is present in the version, whether it may be
//! null, and a strategy for its type. The value strategies are TEST-F16's shared ones
//! (`alluvium_types::strategies`), with lengths kept short so even the deepest message
//! stays small.

use std::fmt::Debug;

use proptest::prelude::*;
use proptest::strategy::BoxedStrategy;

/// `strategy` where the field is `present` in the version; otherwise always `default`.
pub(crate) fn present<T: Clone + Debug + 'static>(
    present: bool,
    strategy: impl Strategy<Value = T> + 'static,
    default: T,
) -> BoxedStrategy<T> {
    if present {
        strategy.boxed()
    } else {
        Just(default).boxed()
    }
}

/// Values of `strategy`, and also `None` where the field is `nullable` in the version.
pub(crate) fn nullable<T: Debug + 'static>(
    nullable: bool,
    strategy: impl Strategy<Value = T> + 'static,
) -> BoxedStrategy<Option<T>> {
    if nullable {
        proptest::option::of(strategy).boxed()
    } else {
        strategy.prop_map(Some).boxed()
    }
}

pub(crate) use alluvium_types::strategies::{
    boundary_i8, boundary_i16, boundary_i32, boundary_i64, boundary_u16, boundary_u32, uuid,
};

/// A `float64` that isn't NaN: NaN never equals itself, so a round trip can't compare it.
pub(crate) fn float64() -> impl Strategy<Value = f64> {
    any::<f64>().prop_filter("not NaN", |value| !value.is_nan())
}

/// A short `string`.
pub(crate) fn string() -> impl Strategy<Value = String> {
    alluvium_types::strategies::string_up_to(8)
}

/// A short `bytes` value.
pub(crate) fn bytes() -> impl Strategy<Value = bytes::Bytes> {
    alluvium_types::strategies::bytes_up_to(8).prop_map(bytes::Bytes::from)
}

/// A short `records` value.
pub(crate) fn records() -> impl Strategy<Value = crate::RecordsField> {
    bytes().prop_map(crate::RecordsField::new)
}

/// A short array of `element`s.
pub(crate) fn array<T: Debug>(element: impl Strategy<Value = T>) -> impl Strategy<Value = Vec<T>> {
    proptest::collection::vec(element, 0..3)
}

/// Up to two unknown tagged fields, with tags from `first_tag`: above every tag the
/// struct defines.
pub(crate) fn unknown_tagged_fields(
    first_tag: u32,
) -> impl Strategy<Value = Vec<crate::RawTaggedField>> {
    proptest::collection::btree_map(
        first_tag..first_tag.saturating_add(8),
        alluvium_types::strategies::bytes_up_to(4),
        0..3,
    )
    .prop_map(|fields| {
        fields
            .into_iter()
            .map(|(tag, data)| crate::RawTaggedField {
                tag,
                data: data.into(),
            })
            .collect()
    })
}
