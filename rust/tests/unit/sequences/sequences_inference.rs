//! Unit tests for the high-level pattern-inference reports (issue #531).
//! Externalised from `src/sequences/inference.rs`.

use formal_ai::sequences::{
    Grid, LinkAddress, SequencePattern, SequenceStore, SymbolTable, infer_grid_patterns,
    infer_sequence_patterns,
};

fn atoms(store: &mut SequenceStore, symbols: &mut SymbolTable, values: &[u64]) -> Vec<LinkAddress> {
    values.iter().map(|&v| symbols.scalar(store, v)).collect()
}

#[test]
fn reports_repetition_and_compression() {
    let mut store = SequenceStore::new();
    let mut symbols = SymbolTable::new();
    let seq = atoms(&mut store, &mut symbols, &[1, 2, 1, 2, 1, 2]);
    let report = infer_sequence_patterns(&mut store, &seq);
    assert_eq!(report.length, 6);
    assert_eq!(report.distinct, 2);
    assert!(report.repetition.is_some());
    assert!(report.has_structure());
    assert!(report.compression.is_lossless(&store));
    assert!(report.summary().contains("repetition"));
}

#[test]
fn reports_palindrome() {
    let mut store = SequenceStore::new();
    let mut symbols = SymbolTable::new();
    let seq = atoms(&mut store, &mut symbols, &[1, 2, 3, 2, 1]);
    let report = infer_sequence_patterns(&mut store, &seq);
    assert!(report.palindrome);
    assert!(report.has_structure());
    assert!(report.summary().to_lowercase().contains("palindrome"));
}

#[test]
fn aperiodic_sequence_reports_little_structure() {
    let mut store = SequenceStore::new();
    let mut symbols = SymbolTable::new();
    let seq = atoms(&mut store, &mut symbols, &[1, 2, 3, 4]);
    let report = infer_sequence_patterns(&mut store, &seq);
    assert!(!report.has_structure());
    assert_eq!(report.classification, SequencePattern::Aperiodic);
}

#[test]
fn reports_grid_symmetry() {
    let mut store = SequenceStore::new();
    let grid = Grid::new(2, 3, vec![1, 2, 1, 3, 4, 3]).unwrap();
    let report = infer_grid_patterns(&mut store, &grid);
    assert_eq!((report.rows, report.cols), (2, 3));
    assert!(report.symmetries.horizontal);
    assert!(report.has_structure());
    assert!(report.summary().contains("left-right mirror"));
}

/// The symmetry list is joined by the `pattern_grid_symmetry_separator` seed
/// template (`data/seed/multilingual-responses-pattern.lino`): a comma in
/// English, the enumeration comma in Chinese. The JavaScript twin is
/// `rust/tests/web/issue-0531-pattern-inference.test.mjs`.
#[test]
fn grid_symmetries_join_with_the_seeded_separator() {
    let mut store = SequenceStore::new();
    let mut symbols = SymbolTable::new();
    let cells = atoms(&mut store, &mut symbols, &[1, 2, 1, 1, 2, 1]);
    let grid = Grid::new(2, 3, cells).unwrap();
    let report = infer_grid_patterns(&mut store, &grid);
    let english = report.summary_in("en");
    let chinese = report.summary_in("zh");
    assert_eq!(
        english.lines().nth(1),
        Some("Symmetric under: left-right mirror, top-bottom mirror, 180-degree rotation.")
    );
    assert_eq!(
        chinese.lines().nth(1),
        Some("对称于：左右镜像、上下镜像、180度旋转。")
    );
}
