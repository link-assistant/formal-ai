//! High-level pattern-inference reports over sequences and grids.
//!
//! These functions run the whole substrate — deduplication plus 1D/2D
//! detectors — over an input and summarise the structure found. They are what
//! the solver handler and benchmarks call: one entry point in, one auditable
//! report out, including a human-readable [`SequencePatternReport::summary`].

use std::collections::BTreeSet;
use std::fmt::Write as _;

use super::compression::{compress, CompressionResult};
use super::grid_2d::{Grid, GridSymmetry, GridTransform};
use super::patterns_1d::{
    classify_sequence, detect_palindrome, detect_period, detect_repetition, RepetitionPattern,
    SequencePattern,
};
use super::store::{LinkAddress, SequenceStore};

/// A structural report for a single sequence.
#[derive(Debug, Clone)]
pub struct SequencePatternReport {
    /// The number of elements in the sequence.
    pub length: usize,
    /// The number of distinct elements.
    pub distinct: usize,
    /// The primary structural classification.
    pub classification: SequencePattern,
    /// Whether the sequence is a palindrome.
    pub palindrome: bool,
    /// The smallest repeating period shorter than the sequence, if any.
    pub period: Option<usize>,
    /// An exact repeating-block tiling, if any.
    pub repetition: Option<RepetitionPattern>,
    /// The associative-deduplication result and trace.
    pub compression: CompressionResult,
}

impl SequencePatternReport {
    /// Whether any non-trivial structure (repetition, period, palindrome, or
    /// compressible substructure) was detected.
    #[must_use]
    pub const fn has_structure(&self) -> bool {
        self.palindrome
            || self.period.is_some()
            || self.repetition.is_some()
            || self.compression.is_compressed()
            || matches!(self.classification, SequencePattern::Constant)
    }

    /// A short, human-readable description of the inferred structure.
    #[must_use]
    pub fn summary(&self) -> String {
        let mut lines = Vec::new();
        lines.push(format!(
            "Sequence of {} element(s), {} distinct.",
            self.length, self.distinct
        ));
        match &self.classification {
            SequencePattern::Empty => lines.push("The sequence is empty.".to_owned()),
            SequencePattern::Constant => {
                lines.push("Every element is identical (constant sequence).".to_owned());
            }
            SequencePattern::Repetition(pattern) => lines.push(format!(
                "It is a repetition: a block of {} element(s) repeated {} times.",
                pattern.period, pattern.repetitions
            )),
            SequencePattern::Periodic { period } => {
                lines.push(format!("It is periodic with period {period}."));
            }
            SequencePattern::Aperiodic => {
                lines.push("It has no exact repeating period.".to_owned());
            }
        }
        if self.palindrome && self.length > 1 {
            lines.push("It reads the same forwards and backwards (palindrome).".to_owned());
        }
        if self.compression.is_compressed() {
            let ratio = self.compression.compression_ratio();
            lines.push(format!(
                "Associative deduplication replaced {} repeated pair(s), compressing to {:.0}% of the original length (lossless).",
                self.compression.steps.len(),
                ratio * 100.0
            ));
        } else {
            lines.push("No repeated adjacent pairs to deduplicate.".to_owned());
        }
        let mut summary = String::new();
        for (index, line) in lines.iter().enumerate() {
            if index > 0 {
                summary.push('\n');
            }
            summary.push_str(line);
        }
        summary
    }
}

/// Run the full 1D pattern-inference pipeline over `sequence`.
#[must_use]
pub fn infer_sequence_patterns(
    store: &mut SequenceStore,
    sequence: &[LinkAddress],
) -> SequencePatternReport {
    let distinct = sequence.iter().copied().collect::<BTreeSet<_>>().len();
    let classification = classify_sequence(sequence);
    let palindrome = detect_palindrome(sequence);
    let period = detect_period(sequence);
    let repetition = detect_repetition(sequence);
    let compression = compress(store, sequence);
    debug_assert!(
        compression.is_lossless(store),
        "compression must round-trip losslessly"
    );
    SequencePatternReport {
        length: sequence.len(),
        distinct,
        classification,
        palindrome,
        period,
        repetition,
        compression,
    }
}

/// A structural report for a grid.
#[derive(Debug, Clone)]
pub struct GridPatternReport {
    /// The grid's row count.
    pub rows: usize,
    /// The grid's column count.
    pub cols: usize,
    /// The symmetries the grid exhibits.
    pub symmetries: GridSymmetry,
    /// The non-identity transforms that leave the grid invariant.
    pub invariant_transforms: Vec<GridTransform>,
    /// Pattern inference over the row-major projection of the grid.
    pub row_major: SequencePatternReport,
}

impl GridPatternReport {
    /// Whether the grid shows any symmetry or compressible structure.
    #[must_use]
    pub const fn has_structure(&self) -> bool {
        self.symmetries.any() || self.row_major.has_structure()
    }

    /// A short, human-readable description of the inferred grid structure.
    #[must_use]
    pub fn summary(&self) -> String {
        let mut summary = format!("Grid {}x{}.", self.rows, self.cols);
        let mut symmetries = Vec::new();
        if self.symmetries.horizontal {
            symmetries.push("left-right mirror");
        }
        if self.symmetries.vertical {
            symmetries.push("top-bottom mirror");
        }
        if self.symmetries.rotational_180 {
            symmetries.push("180-degree rotation");
        }
        if self.symmetries.diagonal {
            symmetries.push("main-diagonal reflection");
        }
        if self.symmetries.anti_diagonal {
            symmetries.push("anti-diagonal reflection");
        }
        if symmetries.is_empty() {
            let _ = write!(summary, "\nNo symmetry detected.");
        } else {
            let _ = write!(summary, "\nSymmetric under: {}.", symmetries.join(", "));
        }
        let _ = write!(summary, "\n{}", self.row_major.summary());
        summary
    }
}

/// Run the full 2D pattern-inference pipeline over `grid`.
#[must_use]
pub fn infer_grid_patterns(store: &mut SequenceStore, grid: &Grid) -> GridPatternReport {
    let symmetries = grid.symmetries();
    let invariant_transforms = grid.invariant_transforms();
    let row_major = infer_sequence_patterns(store, &grid.row_major());
    GridPatternReport {
        rows: grid.rows(),
        cols: grid.cols(),
        symmetries,
        invariant_transforms,
        row_major,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sequences::symbols::SymbolTable;

    fn atoms(
        store: &mut SequenceStore,
        symbols: &mut SymbolTable,
        values: &[u64],
    ) -> Vec<LinkAddress> {
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
}
