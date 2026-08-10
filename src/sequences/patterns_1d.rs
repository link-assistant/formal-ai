//! One-dimensional pattern inference over link sequences.
//!
//! These detectors operate purely on element equality, so they apply equally to
//! symbolic sequences, event streams, and text (once a string is projected to
//! unicode-symbol points). They are the 1D half of Phase 4 in the solution plan:
//! repetition, periodicity, palindromes, reversal, and translation.
//!
//! All functions are total and allocation-light; combinatorial search is bounded
//! by the sequence length so inference stays predictable.

use super::store::LinkAddress;

/// A sequence expressed as `repetitions` copies of a `base` block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepetitionPattern {
    /// The length of the repeating block.
    pub period: usize,
    /// How many times the block repeats to form the whole sequence.
    pub repetitions: usize,
    /// The repeating block itself.
    pub base: Vec<LinkAddress>,
}

/// The primary structural classification of a single sequence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SequencePattern {
    /// The sequence is empty.
    Empty,
    /// Every element is identical.
    Constant,
    /// The sequence is an exact tiling of a shorter block (period divides length).
    Repetition(RepetitionPattern),
    /// The sequence has a repeating period that does not tile it exactly.
    Periodic {
        /// The smallest period shorter than the sequence.
        period: usize,
    },
    /// No non-trivial repeating structure was found.
    Aperiodic,
}

/// Detect an exact tiling: the smallest block whose repetition reproduces the
/// whole sequence. Returns [`None`] when only the whole sequence tiles itself
/// (i.e. there is no shorter repeating block).
#[must_use]
pub fn detect_repetition(sequence: &[LinkAddress]) -> Option<RepetitionPattern> {
    let length = sequence.len();
    if length < 2 {
        return None;
    }
    for period in 1..length {
        if !length.is_multiple_of(period) {
            continue;
        }
        if tiles_exactly(sequence, period) {
            return Some(RepetitionPattern {
                period,
                repetitions: length / period,
                base: sequence[..period].to_vec(),
            });
        }
    }
    None
}

/// Whether `sequence` is `sequence[..period]` repeated `length / period` times.
fn tiles_exactly(sequence: &[LinkAddress], period: usize) -> bool {
    sequence
        .iter()
        .enumerate()
        .all(|(index, element)| *element == sequence[index % period])
}

/// Detect the smallest period `p` (with `p < len`) such that
/// `sequence[i] == sequence[i + p]` for every valid `i`. Returns [`None`] when
/// the only period is the full length (an aperiodic sequence).
#[must_use]
pub fn detect_period(sequence: &[LinkAddress]) -> Option<usize> {
    let length = sequence.len();
    if length < 2 {
        return None;
    }
    (1..length).find(|&period| has_period(sequence, period))
}

/// Whether `sequence[i] == sequence[i + period]` holds for every valid `i`.
fn has_period(sequence: &[LinkAddress], period: usize) -> bool {
    sequence
        .iter()
        .skip(period)
        .zip(sequence.iter())
        .all(|(later, earlier)| later == earlier)
}

/// Whether `sequence` reads identically forwards and backwards. Empty and
/// single-element sequences are trivially palindromic.
#[must_use]
pub fn detect_palindrome(sequence: &[LinkAddress]) -> bool {
    let mut left = 0;
    let mut right = sequence.len();
    while left + 1 < right {
        right -= 1;
        if sequence[left] != sequence[right] {
            return false;
        }
        left += 1;
    }
    true
}

/// Whether every element of `sequence` is identical (and it is non-empty).
#[must_use]
pub fn is_constant(sequence: &[LinkAddress]) -> bool {
    sequence
        .first()
        .is_some_and(|first| sequence.iter().all(|element| element == first))
}

/// Whether `candidate` is the exact reversal of `original`.
#[must_use]
pub fn is_reverse(original: &[LinkAddress], candidate: &[LinkAddress]) -> bool {
    original.len() == candidate.len() && original.iter().rev().eq(candidate.iter())
}

/// Detect whether `candidate` is a cyclic translation (rotation) of `original`.
///
/// Returns the shift amount, where a shift of `0` means the two are already
/// equal, or [`None`] when `candidate` is not any rotation of `original`.
#[must_use]
pub fn detect_translation(original: &[LinkAddress], candidate: &[LinkAddress]) -> Option<usize> {
    let length = original.len();
    if length != candidate.len() {
        return None;
    }
    if length == 0 {
        return Some(0);
    }
    (0..length).find(|&shift| is_rotation_by(original, candidate, shift))
}

/// Whether rotating `original` left by `shift` yields `candidate`.
fn is_rotation_by(original: &[LinkAddress], candidate: &[LinkAddress], shift: usize) -> bool {
    let length = original.len();
    (0..length).all(|index| original[(index + shift) % length] == candidate[index])
}

/// Classify the dominant structure of `sequence` into a single label. The order
/// prefers the most specific description: constant, then exact repetition, then
/// bare periodicity, then aperiodic.
#[must_use]
pub fn classify_sequence(sequence: &[LinkAddress]) -> SequencePattern {
    if sequence.is_empty() {
        return SequencePattern::Empty;
    }
    if is_constant(sequence) {
        return SequencePattern::Constant;
    }
    if let Some(repetition) = detect_repetition(sequence) {
        return SequencePattern::Repetition(repetition);
    }
    if let Some(period) = detect_period(sequence) {
        return SequencePattern::Periodic { period };
    }
    SequencePattern::Aperiodic
}
