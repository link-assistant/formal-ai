//! Unit tests for 1D sequence pattern inference (issue #531). Externalised from
//! `src/sequences/patterns_1d.rs`. The classification helpers used here
//! (`classify_sequence`, `is_constant`, `is_reverse`, `detect_translation`) are
//! public on the module but not re-exported at the `sequences` root, so they are
//! imported by their full module path.

use formal_ai::sequences::patterns_1d::{
    classify_sequence, detect_translation, is_constant, is_reverse,
};
use formal_ai::sequences::{
    detect_palindrome, detect_period, detect_repetition, LinkAddress, SequencePattern,
};

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
