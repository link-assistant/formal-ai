//! Unit tests for associative deduplication / Re-Pair compression (issue #531).
//! Externalised from `src/sequences/compression.rs`.

use formal_ai::sequences::{compress, LinkAddress, SequenceStore, SymbolTable};

fn atoms(store: &mut SequenceStore, symbols: &mut SymbolTable, values: &[u64]) -> Vec<LinkAddress> {
    values.iter().map(|&v| symbols.scalar(store, v)).collect()
}

#[test]
fn compresses_repeated_pair_and_expands_exactly() {
    let mut store = SequenceStore::new();
    let mut symbols = SymbolTable::new();
    // A B A B C A B  -> the pair (A,B) occurs three times.
    let seq = atoms(&mut store, &mut symbols, &[1, 2, 1, 2, 3, 1, 2]);
    let result = compress(&mut store, &seq);
    assert!(result.is_compressed());
    assert!(result.is_lossless(&store), "expansion must reproduce input");
    assert_eq!(result.expand(&store), seq);
    // First step replaces (A,B) at its three occurrences.
    let first = result.steps[0];
    assert_eq!((first.source, first.target), (seq[0], seq[1]));
    assert_eq!(first.occurrences, 3);
    // Compressed form is shorter than the original.
    assert!(result.sequence.len() < seq.len());
    assert!(result.compression_ratio() < 1.0);
}

#[test]
fn no_repetition_leaves_sequence_untouched() {
    let mut store = SequenceStore::new();
    let mut symbols = SymbolTable::new();
    let seq = atoms(&mut store, &mut symbols, &[1, 2, 3, 4]);
    let result = compress(&mut store, &seq);
    assert!(!result.is_compressed());
    assert_eq!(result.sequence, seq);
    assert!(result.is_lossless(&store));
    assert!((result.compression_ratio() - 1.0).abs() < f64::EPSILON);
}

#[test]
fn overlapping_run_counts_non_overlapping() {
    let mut store = SequenceStore::new();
    let mut symbols = SymbolTable::new();
    // A A A A -> pair (A,A) non-overlapping count is 2.
    let seq = atoms(&mut store, &mut symbols, &[1, 1, 1, 1]);
    let result = compress(&mut store, &seq);
    assert_eq!(result.steps[0].occurrences, 2);
    assert!(result.is_lossless(&store));
}

#[test]
fn tie_break_is_deterministic() {
    let mut store = SequenceStore::new();
    let mut symbols = SymbolTable::new();
    // Two disjoint pairs each occur twice: (1,2) and (3,4). The smaller
    // (source,target) pair by address must be chosen first, deterministically.
    let seq = atoms(&mut store, &mut symbols, &[1, 2, 3, 4, 1, 2, 3, 4]);
    let result = compress(&mut store, &seq);
    let first = result.steps[0];
    assert_eq!((first.source, first.target), (seq[0], seq[1]));
    assert!(result.is_lossless(&store));
}
