//! Unit tests for unique symbol allocation (issue #531). Externalised from
//! `src/sequences/symbols.rs` to keep production code free of inline tests.

use formal_ai::sequences::{SequenceStore, SymbolTable};

#[test]
fn scalars_are_deduplicated() {
    let mut store = SequenceStore::new();
    let mut symbols = SymbolTable::new();
    let a = symbols.scalar(&mut store, 7);
    let b = symbols.scalar(&mut store, 7);
    let c = symbols.scalar(&mut store, 8);
    assert_eq!(a, b);
    assert_ne!(a, c);
    assert_eq!(symbols.len(), 2);
    assert_eq!(symbols.scalar_lookup(7), Some(a));
    assert_eq!(symbols.scalar_lookup(9), None);
}

#[test]
fn unicode_symbols_reuse_repeated_characters() {
    let mut store = SequenceStore::new();
    let mut symbols = SymbolTable::new();
    let points = symbols.unicode_symbols(&mut store, "abba");
    assert_eq!(points.len(), 4);
    assert_eq!(points[0], points[3], "both 'a's share a point");
    assert_eq!(points[1], points[2], "both 'b's share a point");
    assert_ne!(points[0], points[1]);
    // Only two distinct code points were allocated.
    assert_eq!(symbols.len(), 2);
}

#[test]
fn markers_are_stable_by_name() {
    let mut store = SequenceStore::new();
    let mut symbols = SymbolTable::new();
    let first = symbols.marker(&mut store, "unicode_sequence");
    let second = symbols.marker(&mut store, "unicode_sequence");
    let other = symbols.marker(&mut store, "text");
    assert_eq!(first, second);
    assert_ne!(first, other);
    assert!(store.is_point(first));
}
