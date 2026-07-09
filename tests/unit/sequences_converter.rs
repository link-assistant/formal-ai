//! Unit tests for the balanced converter, sequence index, and frequency cache
//! (issue #531). Externalised from `src/sequences/converter.rs`.

use formal_ai::sequences::{
    balanced_convert, LinkAddress, LinkFrequenciesCache, SequenceIndex, SequenceStore, SymbolTable,
    NULL_LINK,
};

fn points(
    store: &mut SequenceStore,
    symbols: &mut SymbolTable,
    values: &[u64],
) -> Vec<LinkAddress> {
    values.iter().map(|&v| symbols.scalar(store, v)).collect()
}

#[test]
fn balanced_convert_edge_cases() {
    let mut store = SequenceStore::new();
    assert_eq!(balanced_convert(&mut store, &[]), NULL_LINK);
    let a = store.create_point();
    assert_eq!(balanced_convert(&mut store, &[a]), a);
}

#[test]
fn balanced_convert_round_trips_and_deduplicates() {
    let mut store = SequenceStore::new();
    let mut symbols = SymbolTable::new();
    let seq = points(&mut store, &mut symbols, &[1, 2, 3, 4, 5]);
    let root = balanced_convert(&mut store, &seq);
    assert_eq!(
        store.expand(root),
        seq,
        "balanced tree expands to the input"
    );

    // Converting the same sequence again yields the same root (structural
    // deduplication), allocating no new links.
    let before = store.len();
    let root_again = balanced_convert(&mut store, &seq);
    assert_eq!(root, root_again);
    assert_eq!(store.len(), before, "second conversion reuses every link");
}

#[test]
fn balanced_convert_pairs_before_carrying_odd_tail() {
    let mut store = SequenceStore::new();
    let mut symbols = SymbolTable::new();
    let seq = points(&mut store, &mut symbols, &[1, 2, 3]);
    let root = balanced_convert(&mut store, &seq);
    // Layer one pairs (1,2) and carries 3 -> [(1 2), 3]; root = ((1 2) 3).
    let root_doublet = store.get(root).unwrap();
    assert_eq!(root_doublet.target, seq[2]);
    assert_eq!(store.expand(root_doublet.source), vec![seq[0], seq[1]]);
}

#[test]
fn sequence_index_add_and_might_contain() {
    let mut store = SequenceStore::new();
    let mut symbols = SymbolTable::new();
    let seq = points(&mut store, &mut symbols, &[1, 2, 3]);
    assert!(!SequenceIndex::might_contain(&store, &seq));
    let already = SequenceIndex::add(&mut store, &seq);
    assert!(!already, "first add reports the sequence was not indexed");
    assert!(SequenceIndex::might_contain(&store, &seq));
    assert!(
        SequenceIndex::add(&mut store, &seq),
        "second add reports it was already indexed"
    );
}

#[test]
fn frequencies_count_adjacent_pairs() {
    let mut store = SequenceStore::new();
    let mut symbols = SymbolTable::new();
    // A B A B: the pair (A,B) occurs twice, (B,A) once.
    let seq = points(&mut store, &mut symbols, &[10, 20, 10, 20]);
    let mut cache = LinkFrequenciesCache::new();
    cache.increment_sequence(&store, &seq);
    assert_eq!(cache.get(seq[0], seq[1]).frequency, 2);
    assert_eq!(cache.get(seq[1], seq[2]).frequency, 1);
    assert_eq!(cache.get(seq[2], seq[3]).frequency, 2, "same (A,B) doublet");
    assert_eq!(cache.get(seq[1], seq[0]).frequency, 1);
    // Unobserved pair reports a zero entry.
    assert_eq!(cache.get(seq[0], seq[0]).frequency, 0);
}
