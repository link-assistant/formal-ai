//! High-level pattern-inference reports over sequences and grids.
//!
//! These functions run the whole substrate — deduplication plus 1D/2D
//! detectors — over an input and summarise the structure found. They are what
//! the solver handler and benchmarks call: one entry point in, one auditable
//! report out, including a human-readable [`SequencePatternReport::summary`].

use std::collections::BTreeSet;

use crate::seed;

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
        self.summary_in("en")
    }

    /// A localized description rendered from Links Notation seed templates.
    #[must_use]
    pub fn summary_in(&self, language: &str) -> String {
        let length = self.length.to_string();
        let distinct = self.distinct.to_string();
        let mut lines = vec![render_pattern(
            "pattern_sequence_count",
            language,
            &[("length", &length), ("distinct", &distinct)],
        )];
        match &self.classification {
            SequencePattern::Empty => {
                lines.push(pattern_template("pattern_sequence_empty", language));
            }
            SequencePattern::Constant => {
                lines.push(pattern_template("pattern_sequence_constant", language));
            }
            SequencePattern::Repetition(pattern) => {
                let period = pattern.period.to_string();
                let repetitions = pattern.repetitions.to_string();
                lines.push(render_pattern(
                    "pattern_sequence_repetition",
                    language,
                    &[("period", &period), ("repetitions", &repetitions)],
                ));
            }
            SequencePattern::Periodic { period } => {
                let period = period.to_string();
                lines.push(render_pattern(
                    "pattern_sequence_periodic",
                    language,
                    &[("period", &period)],
                ));
            }
            SequencePattern::Aperiodic => {
                lines.push(pattern_template("pattern_sequence_aperiodic", language));
            }
        }
        if self.palindrome && self.length > 1 {
            lines.push(pattern_template("pattern_sequence_palindrome", language));
        }
        if self.compression.is_compressed() {
            let pairs = self.compression.steps.len().to_string();
            let percent = format!("{:.0}", self.compression.compression_ratio() * 100.0);
            lines.push(render_pattern(
                "pattern_sequence_compression",
                language,
                &[("pairs", &pairs), ("percent", &percent)],
            ));
        } else {
            lines.push(pattern_template(
                "pattern_sequence_no_compression",
                language,
            ));
        }
        lines.join("\n")
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
        self.summary_in("en")
    }

    /// A localized description rendered from Links Notation seed templates.
    #[must_use]
    pub fn summary_in(&self, language: &str) -> String {
        let rows = self.rows.to_string();
        let cols = self.cols.to_string();
        let mut lines = vec![render_pattern(
            "pattern_grid_shape",
            language,
            &[("rows", &rows), ("cols", &cols)],
        )];
        let mut symmetries = Vec::new();
        if self.symmetries.horizontal {
            symmetries.push(pattern_template("pattern_grid_horizontal", language));
        }
        if self.symmetries.vertical {
            symmetries.push(pattern_template("pattern_grid_vertical", language));
        }
        if self.symmetries.rotational_180 {
            symmetries.push(pattern_template("pattern_grid_rotation", language));
        }
        if self.symmetries.diagonal {
            symmetries.push(pattern_template("pattern_grid_diagonal", language));
        }
        if self.symmetries.anti_diagonal {
            symmetries.push(pattern_template("pattern_grid_anti_diagonal", language));
        }
        if symmetries.is_empty() {
            lines.push(pattern_template("pattern_grid_no_symmetry", language));
        } else {
            let separator = if language == "zh" { "、" } else { ", " };
            let joined = symmetries.join(separator);
            lines.push(render_pattern(
                "pattern_grid_symmetry",
                language,
                &[("symmetries", &joined)],
            ));
        }
        lines.push(self.row_major.summary_in(language));
        lines.join("\n")
    }
}

fn pattern_template(intent: &str, language: &str) -> String {
    seed::localized_response(intent, language).unwrap_or_default()
}

fn render_pattern(intent: &str, language: &str, values: &[(&str, &str)]) -> String {
    let mut rendered = pattern_template(intent, language);
    for (name, value) in values {
        rendered = rendered.replace(&format!("{{{name}}}"), value);
    }
    rendered
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
