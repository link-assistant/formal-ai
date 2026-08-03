//! Associative deduplication: repeated-pair compression with a reversible trace.
//!
//! This is the first pattern-inference step the issue asks for. It is a clean,
//! deterministic Re-Pair-style algorithm rather than a port of the upstream
//! `CompressingConverter` pointer arithmetic (whose C# and C++ sources disagree
//! on the max-frequency comparison — see the case study). The behaviour is
//! specified precisely so it can be tested exactly:
//!
//! 1. count the non-overlapping occurrences of every adjacent pair;
//! 2. pick the pair with the highest count (at least two), breaking ties by the
//!    lexicographically smallest `(source, target)` so the choice is stable;
//! 3. materialise that pair as a single composite link via
//!    [`SequenceStore::get_or_create`] and replace every non-overlapping
//!    occurrence, left to right;
//! 4. record the substitution as a [`CompressionStep`];
//! 5. repeat until no adjacent pair repeats.
//!
//! Each replacement strictly shortens the working sequence, so the loop always
//! terminates. Because replacements go through `get_or_create`, the result is a
//! set of links whose [`SequenceStore::expand`] reproduces the original element
//! sequence exactly: the compression is losslessly reversible, which every test
//! asserts.

use std::collections::HashMap;

use super::store::{LinkAddress, SequenceStore};

/// One repeated-pair substitution recorded during compression.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CompressionStep {
    /// The source link of the replaced pair.
    pub source: LinkAddress,
    /// The target link of the replaced pair.
    pub target: LinkAddress,
    /// The composite link the pair was replaced with.
    pub replacement: LinkAddress,
    /// How many non-overlapping occurrences were replaced.
    pub occurrences: usize,
}

/// The outcome of compressing a sequence: the trace plus the compressed form.
#[derive(Debug, Clone)]
pub struct CompressionResult {
    /// The original element sequence, kept for lossless-expansion checks.
    pub original: Vec<LinkAddress>,
    /// The compressed sequence of links (each may be an atom or a composite).
    pub sequence: Vec<LinkAddress>,
    /// The ordered substitutions applied, most-frequent-first by construction.
    pub steps: Vec<CompressionStep>,
}

impl CompressionResult {
    /// Expand the compressed sequence back into the original element sequence.
    #[must_use]
    pub fn expand(&self, store: &SequenceStore) -> Vec<LinkAddress> {
        let mut output = Vec::with_capacity(self.original.len());
        for &link in &self.sequence {
            output.extend(store.expand(link));
        }
        output
    }

    /// Whether expanding the compressed form reproduces the original exactly.
    /// This is the lossless-round-trip guarantee that makes the compression a
    /// valid deduplication rather than lossy summarisation.
    #[must_use]
    pub fn is_lossless(&self, store: &SequenceStore) -> bool {
        self.expand(store) == self.original
    }

    /// The ratio of compressed length to original length in `(0, 1]`; smaller is
    /// more compressible. An empty original reports `1.0` (nothing to compress).
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn compression_ratio(&self) -> f64 {
        if self.original.is_empty() {
            return 1.0;
        }
        self.sequence.len() as f64 / self.original.len() as f64
    }

    /// Whether any repeated structure was found and deduplicated.
    #[must_use]
    pub const fn is_compressed(&self) -> bool {
        !self.steps.is_empty()
    }
}

/// Compress `sequence` by repeatedly replacing the most frequent adjacent pair.
///
/// See the module documentation for the exact rule. The returned
/// [`CompressionResult`] carries the trace and satisfies
/// [`CompressionResult::is_lossless`] for the same `store`.
#[must_use]
pub fn compress(store: &mut SequenceStore, sequence: &[LinkAddress]) -> CompressionResult {
    let original = sequence.to_vec();
    let mut current = original.clone();
    let mut steps = Vec::new();

    while let Some((pair, occurrences)) = most_frequent_pair(&current) {
        let replacement = store.get_or_create(pair.0, pair.1);
        current = replace_pair(&current, pair, replacement);
        steps.push(CompressionStep {
            source: pair.0,
            target: pair.1,
            replacement,
            occurrences,
        });
    }

    CompressionResult {
        original,
        sequence: current,
        steps,
    }
}

/// Find the adjacent pair with the most non-overlapping occurrences (at least
/// two), breaking ties by the lexicographically smallest `(source, target)`.
/// Returns [`None`] when no adjacent pair repeats.
fn most_frequent_pair(sequence: &[LinkAddress]) -> Option<((LinkAddress, LinkAddress), usize)> {
    // Distinct candidate pairs are collected first so each is counted with a
    // dedicated non-overlapping scan (overlapping runs like `A A A` must count
    // as one, not two).
    let mut seen: HashMap<(LinkAddress, LinkAddress), usize> = HashMap::new();
    for window in sequence.windows(2) {
        seen.entry((window[0], window[1])).or_insert(0);
    }

    let mut best: Option<((LinkAddress, LinkAddress), usize)> = None;
    for &pair in seen.keys() {
        let count = count_non_overlapping(sequence, pair);
        if count < 2 {
            continue;
        }
        best = match best {
            Some((_, best_count)) if best_count > count => best,
            Some((best_pair, best_count)) if best_count == count && best_pair < pair => best,
            _ => Some((pair, count)),
        };
    }
    best
}

/// Count non-overlapping occurrences of `pair` scanning left to right.
fn count_non_overlapping(sequence: &[LinkAddress], pair: (LinkAddress, LinkAddress)) -> usize {
    let mut count = 0;
    let mut index = 0;
    while index + 1 < sequence.len() {
        if sequence[index] == pair.0 && sequence[index + 1] == pair.1 {
            count += 1;
            index += 2;
        } else {
            index += 1;
        }
    }
    count
}

/// Replace every non-overlapping occurrence of `pair` with `replacement`,
/// scanning left to right.
fn replace_pair(
    sequence: &[LinkAddress],
    pair: (LinkAddress, LinkAddress),
    replacement: LinkAddress,
) -> Vec<LinkAddress> {
    let mut output = Vec::with_capacity(sequence.len());
    let mut index = 0;
    while index < sequence.len() {
        if index + 1 < sequence.len() && sequence[index] == pair.0 && sequence[index + 1] == pair.1
        {
            output.push(replacement);
            index += 2;
        } else {
            output.push(sequence[index]);
            index += 1;
        }
    }
    output
}
