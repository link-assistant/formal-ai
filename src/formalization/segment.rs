//! Script-aware sentence and clause segmentation with exact character spans
//! (issue #1138, plan 04 L2).
//!
//! Terminators are declared in the seed, never as literals in Rust, and a
//! segment's span selects exactly its own text — the defect recorded at
//! `docs/case-studies/issue-710/plans/07:371-377` (a span that swallowed the
//! preceding separator whitespace) is fixed here rather than documented again.

/// One segmented unit of source text with exact character offsets into the
/// original string.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Segment {
    pub text: String,
    pub start: usize,
    pub end: usize,
    pub script: Script,
}

/// The writing system a segment is in, decided from its characters, never from
/// a language flag supplied by a caller.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Script {
    Latin,
    Cyrillic,
    Devanagari,
    Han,
    Other,
}

/// Segment `text` into sentences at the terminators the seed declares for each
/// script (`.` `!` `?` `。` `！` `？` `।` `॥`, and the Spanish inverted pairs),
/// keeping exact spans.
#[must_use]
pub fn sentences(_text: &str) -> Vec<Segment> {
    todo!("plan 04 leaf L2")
}

/// Segment one sentence into clauses at the seeded clause separators, so a
/// requirement carrying several obligations emits several needs.
#[must_use]
pub fn clauses(_sentence: &Segment) -> Vec<Segment> {
    todo!("plan 04 leaf L2")
}
