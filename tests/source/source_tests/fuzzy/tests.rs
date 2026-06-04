use super::{is_close_token_typo, typo_distance};

#[test]
fn typo_distance_counts_adjacent_transposition_as_one_edit() {
    assert_eq!(typo_distance("calcualte", "calculate"), 1);
}

#[test]
fn close_token_typo_requires_meaningful_token_length() {
    assert!(is_close_token_typo("calcuate", "calculate"));
    assert!(!is_close_token_typo("cn", "can"));
}
