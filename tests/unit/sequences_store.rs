//! Unit tests for the link-native doublet store (issue #531).
//!
//! These live in `tests/` rather than inline in `src/` so the shipped crate
//! stays free of `#[cfg(test)]` scaffolding (see the `source_test_placement`
//! CI gate). They exercise the public `formal_ai::sequences::store` surface.

use formal_ai::sequences::{Doublet, SequenceStore, NULL_LINK};

#[test]
fn null_link_resolves_to_zero_doublet() {
    let store = SequenceStore::new();
    assert!(store.is_empty());
    assert_eq!(store.null(), NULL_LINK);
    assert_eq!(store.get(NULL_LINK), Some(Doublet::new(0, 0)));
    assert!(store.is_valid(NULL_LINK));
    assert!(!store.is_point(NULL_LINK));
}

#[test]
fn points_are_self_referential_and_unique() {
    let mut store = SequenceStore::new();
    let a = store.create_point();
    let b = store.create_point();
    assert_ne!(a, b);
    assert!(store.is_point(a));
    assert!(store.is_point(b));
    assert_eq!(store.get(a), Some(Doublet::new(a, a)));
    assert_eq!(store.expand(a), vec![a]);
}

#[test]
fn get_or_create_deduplicates_pairs() {
    let mut store = SequenceStore::new();
    let a = store.create_point();
    let b = store.create_point();
    let first = store.get_or_create(a, b);
    let second = store.get_or_create(a, b);
    assert_eq!(first, second, "identical pairs must resolve to one address");
    assert_eq!(store.len(), 3, "two points and one composite");
    assert_eq!(store.search(a, b), Some(first));
    assert_eq!(store.search(b, a), None);
}

#[test]
fn expand_reproduces_the_original_elements() {
    let mut store = SequenceStore::new();
    let a = store.create_point();
    let b = store.create_point();
    let c = store.create_point();
    // Build ((a b) c) and confirm it flattens back to a, b, c.
    let ab = store.get_or_create(a, b);
    let abc = store.get_or_create(ab, c);
    assert_eq!(store.expand(abc), vec![a, b, c]);
}

#[test]
fn self_pairing_a_point_is_a_distinct_composite() {
    let mut store = SequenceStore::new();
    let a = store.create_point();
    // Pairing an atom with itself is the length-two sequence `a a`, a
    // different link from the atom `a`, and it must expand losslessly.
    let aa = store.get_or_create(a, a);
    assert_ne!(aa, a, "the self-pair is not the point");
    assert!(
        !store.is_point(aa),
        "the self-pair is a composite, not a point"
    );
    assert!(store.is_point(a));
    assert_eq!(store.expand(aa), vec![a, a]);
    // Repeated requests still deduplicate to the same composite.
    assert_eq!(store.get_or_create(a, a), aa);
    assert_eq!(store.len(), 2, "one point and one composite");
}
