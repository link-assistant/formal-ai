//! Link-native sequence substrate and pattern inference (issue #531).
//!
//! This module reimplements, in dependency order, the parts of
//! `linksplatform/Data.Doublets.Sequences` needed to ground pattern inference
//! in links rather than ad hoc strings, then builds associative deduplication
//! and 1D/2D pattern inference on top of it:
//!
//! - [`store`] — a self-contained associative doublet store (`GetOrCreate` /
//!   `SearchOrDefault` primitives, exact expansion);
//! - [`symbols`] — unique point allocation for scalar, unicode, and marker atoms;
//! - [`converter`] — the deterministic `BalancedVariantConverter`,
//!   `SequenceIndex`, and `LinkFrequenciesCache` ports;
//! - [`compression`] — associative deduplication (repeated-pair replacement)
//!   with an auditable, losslessly-reversible trace;
//! - [`patterns_1d`] — 1D sequence/text pattern inference (repetition, period,
//!   palindrome, reversal, translation);
//! - [`grid_2d`] — 2D grid transforms (rotation, reflection, transpose,
//!   symmetry) projected onto the same sequence machinery;
//! - [`inference`] — a high-level report that runs the above over a sequence or
//!   grid and summarises the structure it found.
//!
//! Everything is a link: atoms are self-referential points, sequences are
//! balanced doublet trees, and deduplication reuses shared sub-structure. The
//! design follows Links meta-theory (references to links, `L -> L^2`) so the
//! same substrate can later be persisted through the crate's doublet backend.

pub mod compression;
pub mod converter;
pub mod grid_2d;
pub mod inference;
pub mod patterns_1d;
pub mod store;
pub mod symbols;

pub use compression::{compress, CompressionResult, CompressionStep};
pub use converter::{balanced_convert, LinkFrequenciesCache, LinkFrequency, SequenceIndex};
pub use grid_2d::{Grid, GridSymmetry, GridTransform};
pub use inference::{
    infer_grid_patterns, infer_sequence_patterns, GridPatternReport, SequencePatternReport,
};
pub use patterns_1d::{
    detect_palindrome, detect_period, detect_repetition, RepetitionPattern, SequencePattern,
};
pub use store::{Doublet, LinkAddress, SequenceStore, NULL_LINK};
pub use symbols::SymbolTable;
