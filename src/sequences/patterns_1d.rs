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

#[cfg(test)]
mod tests {
    use super::*;

    fn seq(values: &[LinkAddress]) -> Vec<LinkAddress> {
        values.to_vec()
    }

    #[test]
    fn detects_exact_repetition() {
        let s = seq(&[1, 2, 1, 2, 1, 2]);
        let pattern = detect_repetition(&s).expect("repeating block");
        assert_eq!(pattern.period, 2);
        assert_eq!(pattern.repetitions, 3);
        assert_eq!(pattern.base, vec![1, 2]);
        assert_eq!(classify_sequence(&s), SequencePattern::Repetition(pattern));
    }

    #[test]
    fn constant_sequence_classifies_as_constant() {
        let s = seq(&[7, 7, 7]);
        assert!(is_constant(&s));
        assert_eq!(classify_sequence(&s), SequencePattern::Constant);
    }

    #[test]
    fn periodic_without_exact_tiling() {
        // Period 2 (A B A B A) but length 5 is not a multiple of 2.
        let s = seq(&[1, 2, 1, 2, 1]);
        assert_eq!(detect_repetition(&s), None);
        assert_eq!(detect_period(&s), Some(2));
        assert_eq!(
            classify_sequence(&s),
            SequencePattern::Periodic { period: 2 }
        );
    }

    #[test]
    fn aperiodic_sequence() {
        let s = seq(&[1, 2, 3, 4]);
        assert_eq!(classify_sequence(&s), SequencePattern::Aperiodic);
        assert_eq!(detect_period(&s), None);
    }

    #[test]
    fn palindrome_detection() {
        assert!(detect_palindrome(&seq(&[1, 2, 3, 2, 1])));
        assert!(detect_palindrome(&seq(&[1, 2, 2, 1])));
        assert!(!detect_palindrome(&seq(&[1, 2, 3])));
        assert!(detect_palindrome(&seq(&[])));
        assert!(detect_palindrome(&seq(&[9])));
    }

    #[test]
    fn reversal_and_translation() {
        let original = seq(&[1, 2, 3, 4]);
        assert!(is_reverse(&original, &seq(&[4, 3, 2, 1])));
        assert!(!is_reverse(&original, &seq(&[1, 2, 3, 4])));
        // Rotate left by 1: [2,3,4,1].
        assert_eq!(detect_translation(&original, &seq(&[2, 3, 4, 1])), Some(1));
        assert_eq!(detect_translation(&original, &seq(&[1, 2, 3, 4])), Some(0));
        assert_eq!(detect_translation(&original, &seq(&[4, 3, 2, 1])), None);
    }
}
