//! Small deterministic typo helpers for command and concept matching.
//!
//! These helpers intentionally stay conservative: callers decide which
//! candidates are safe to correct, while this module only reports close token
//! distance including the common adjacent-transposition typo.

/// Compute a restricted Damerau-Levenshtein distance between two strings.
///
/// Adjacent transpositions count as a single edit, so `calcualte` is one edit
/// away from `calculate`.
#[must_use]
pub fn typo_distance(a: &str, b: &str) -> usize {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let m = a_chars.len();
    let n = b_chars.len();
    let mut dp = vec![vec![0usize; n + 1]; m + 1];
    for (i, row) in dp.iter_mut().enumerate() {
        row[0] = i;
    }
    for (j, cell) in dp[0].iter_mut().enumerate() {
        *cell = j;
    }
    for i in 1..=m {
        for j in 1..=n {
            let substitution = usize::from(a_chars[i - 1] != b_chars[j - 1]);
            let mut best = (dp[i - 1][j] + 1)
                .min(dp[i][j - 1] + 1)
                .min(dp[i - 1][j - 1] + substitution);
            if i > 1
                && j > 1
                && a_chars[i - 1] == b_chars[j - 2]
                && a_chars[i - 2] == b_chars[j - 1]
            {
                best = best.min(dp[i - 2][j - 2] + 1);
            }
            dp[i][j] = best;
        }
    }
    dp[m][n]
}

#[must_use]
pub fn is_close_token_typo(actual: &str, expected: &str) -> bool {
    let actual = actual.to_lowercase();
    let expected = expected.to_lowercase();
    let actual_len = actual.chars().count();
    let expected_len = expected.chars().count();
    actual_len.min(expected_len) >= 4 && typo_distance(&actual, &expected) == 1
}

#[cfg(test)]
mod tests {
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
}
