#!/usr/bin/env rust-script
//! Enforce GitHub's closing-keyword syntax in pull-request descriptions.
//!
//! Issue #960 (R234-4, <https://github.com/link-assistant/formal-ai/pull/234#issuecomment-4528554549>):
//! "we should use proper `Fixes https://github.com/link-assistant/formal-ai/issues/146`
//! syntax in pull request description to fully close all the issues. Word
//! `Addresses` is not recognized by GitHub as explicit link to the issue, that
//! will cause it to automatically close on pull request merge."
//!
//! A description that says "Addresses #146" reads to a human exactly like a
//! link and to GitHub like plain prose: the issue silently stays open after the
//! merge. This check reads the description and fails when it carries no
//! recognised closing keyword, or when it uses an unrecognised word (Addresses,
//! Relates to, Part of, See, Refs) where a closing keyword belongs.
//!
//! Recognised keywords are GitHub's own list:
//! <https://docs.github.com/en/issues/tracking-your-work-with-issues/using-issues/linking-a-pull-request-to-an-issue>
//!
//! Usage:
//!   PR_BODY="$(gh pr view 975 --json body --jq .body)" \
//!     rust-script scripts/check-pull-request-link.rs      # check a body
//!   rust-script scripts/check-pull-request-link.rs FILE   # check a file
//!
//! Run the inline unit tests with:
//!   rust-script --test scripts/check-pull-request-link.rs
//!
//! ```cargo
//! [package]
//! edition = "2024"
//!
//! [dependencies]
//! ```

#[cfg(not(test))]
use std::{fs, process::exit};

/// GitHub's closing keywords. Any of these, followed by an issue reference,
/// closes the issue when the pull request merges.
const CLOSING_KEYWORDS: &[&str] = &[
    "close", "closes", "closed", "fix", "fixes", "fixed", "resolve", "resolves", "resolved",
];

/// Words contributors reach for that look like links but close nothing.
const NON_CLOSING_KEYWORDS: &[&str] = &[
    "addresses",
    "address",
    "addressing",
    "relates to",
    "related to",
    "part of",
    "refs",
    "ref",
    "see",
];

/// Host of the issue URLs this repository links to.
const ISSUE_URL_FRAGMENT: &str = "github.com/";

#[derive(Debug, Clone, PartialEq, Eq)]
enum Problem {
    /// No closing keyword anywhere in the body.
    MissingClosingKeyword,
    /// A non-closing word is used where a closing keyword belongs.
    NonClosingKeyword { line: usize, keyword: String },
}

/// An issue reference: `#146` or a full issue URL.
fn starts_with_issue_reference(rest: &str) -> bool {
    let rest = rest.trim_start();
    if let Some(number) = rest.strip_prefix('#') {
        return number.chars().next().is_some_and(|c| c.is_ascii_digit());
    }
    // `owner/repo#146` and full URLs.
    if rest.starts_with("http") {
        return rest.contains(ISSUE_URL_FRAGMENT) && rest.contains("/issues/");
    }
    rest.split_whitespace().next().is_some_and(|token| {
        token
            .split_once('#')
            .is_some_and(|(_, number)| number.chars().next().is_some_and(char::is_numeric))
    })
}

/// Does `line` use `keyword` as a link verb followed by an issue reference?
fn links_with(line: &str, keyword: &str) -> bool {
    let lowered = line.to_lowercase();
    let mut search_from = 0;

    while let Some(found) = lowered[search_from..].find(keyword) {
        let at = search_from + found;
        let after = at + keyword.len();
        let preceded_by_word = at > 0
            && lowered[..at]
                .chars()
                .next_back()
                .is_some_and(char::is_alphanumeric);
        let followed_by_word = lowered[after..]
            .chars()
            .next()
            .is_some_and(char::is_alphanumeric);
        if !preceded_by_word && !followed_by_word && starts_with_issue_reference(&line[after..]) {
            return true;
        }
        search_from = after;
    }

    false
}

fn check_body(body: &str) -> Vec<Problem> {
    let mut problems = Vec::new();
    let mut closes = false;

    let mut in_code_fence = false;

    for (index, line) in body.lines().enumerate() {
        // A description quotes the run it is reporting, so a fenced block can
        // legitimately contain the very wording this check rejects. Quoted
        // output is evidence, not a link: skip it in both directions.
        if line.trim_start().starts_with("```") {
            in_code_fence = !in_code_fence;
            continue;
        }
        if in_code_fence {
            continue;
        }

        for keyword in CLOSING_KEYWORDS {
            if links_with(line, keyword) {
                closes = true;
            }
        }
        for keyword in NON_CLOSING_KEYWORDS {
            if links_with(line, keyword) {
                problems.push(Problem::NonClosingKeyword {
                    line: index + 1,
                    keyword: (*keyword).to_string(),
                });
            }
        }
    }

    if !closes {
        problems.insert(0, Problem::MissingClosingKeyword);
    }

    problems
}

#[cfg(not(test))]
fn describe(problem: &Problem) -> String {
    match problem {
        Problem::MissingClosingKeyword => format!(
            "The description links no issue with a GitHub closing keyword ({}).\n  \
             Add a line such as: Fixes https://github.com/link-assistant/formal-ai/issues/146",
            CLOSING_KEYWORDS.join(", ")
        ),
        Problem::NonClosingKeyword { line, keyword } => format!(
            "line {line}: `{keyword} #N` is not recognised by GitHub and will not close the issue on merge.\n  \
             Use a closing keyword instead: Fixes #N / Fixes <issue url>."
        ),
    }
}

#[cfg(not(test))]
fn main() {
    let body = std::env::args()
        .nth(1)
        .and_then(|path| fs::read_to_string(path).ok())
        .or_else(|| std::env::var("PR_BODY").ok())
        .unwrap_or_default();

    if body.trim().is_empty() {
        println!("No pull-request description to check (set PR_BODY or pass a file); skipping.");
        exit(0);
    }

    println!("\nChecking the pull-request description links its issue (R234-4)...\n");

    let problems = check_body(&body);
    if problems.is_empty() {
        println!("Description closes its issue with a recognised GitHub keyword\n");
        exit(0);
    }

    for problem in &problems {
        println!("  {}", describe(problem));
    }
    println!(
        "\nSee CONTRIBUTING.md -> Pull Request Process for the required linking syntax.\n"
    );
    exit(1);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The exact body shape this repository's PR template produces.
    #[test]
    fn fixes_with_a_full_issue_url_passes() {
        let body = "## Summary\n\nFixes https://github.com/link-assistant/formal-ai/issues/146\n";

        assert_eq!(check_body(body), Vec::new());
    }

    #[test]
    fn fixes_with_a_short_reference_passes() {
        let body = "Fixes #960\n";

        assert_eq!(check_body(body), Vec::new());
    }

    #[test]
    fn other_closing_keywords_pass() {
        for body in ["Closes #1", "resolves #1", "Fixed #1"] {
            assert_eq!(check_body(body), Vec::new(), "{body} should close its issue");
        }
    }

    /// The reported failure: reads like a link, closes nothing.
    #[test]
    fn addresses_is_rejected() {
        let body = "## Summary\n\nAddresses #146\n";

        assert_eq!(
            check_body(body),
            vec![
                Problem::MissingClosingKeyword,
                Problem::NonClosingKeyword {
                    line: 3,
                    keyword: "addresses".to_string(),
                },
            ]
        );
    }

    #[test]
    fn addresses_with_a_full_url_is_rejected() {
        let body = "Addresses https://github.com/link-assistant/formal-ai/issues/146\n";

        assert_eq!(
            check_body(body),
            vec![
                Problem::MissingClosingKeyword,
                Problem::NonClosingKeyword {
                    line: 1,
                    keyword: "addresses".to_string(),
                },
            ]
        );
    }

    /// A body may reference sibling issues loosely as long as it also closes
    /// its own — flagging the loose word is still useful, so both are reported.
    #[test]
    fn a_body_with_no_issue_link_at_all_is_rejected() {
        let body = "## Summary\n\n- refactored the parser\n";

        assert_eq!(check_body(body), vec![Problem::MissingClosingKeyword]);
    }

    /// "fix" inside prose is not a link verb.
    #[test]
    fn prose_mentioning_fix_without_a_reference_does_not_count_as_a_link() {
        let body = "This fixes the parser crash.\n";

        assert_eq!(check_body(body), vec![Problem::MissingClosingKeyword]);
    }

    #[test]
    fn keyword_inside_a_longer_word_is_ignored() {
        let body = "Prefixes #146 is not a keyword\nFixes #146\n";

        assert_eq!(check_body(body), Vec::new());
    }

    /// A description that reports running this very check quotes its own
    /// output; the quote must not be read as the description's own wording.
    #[test]
    fn wording_quoted_inside_a_code_fence_is_not_read_as_a_link() {
        let body = "Fixes #960\n\n```console\n$ check pr-body-loose.md  # \"Addresses #1\"\n```\n";

        assert_eq!(check_body(body), Vec::new());
    }

    /// The converse: a code fence cannot supply the closing keyword either.
    #[test]
    fn a_closing_keyword_only_inside_a_code_fence_does_not_count() {
        let body = "## Summary\n\n```\nFixes #960\n```\n";

        assert_eq!(check_body(body), vec![Problem::MissingClosingKeyword]);
    }

    #[test]
    fn owner_repo_short_reference_is_recognised() {
        let body = "Fixes link-assistant/formal-ai#146\n";

        assert_eq!(check_body(body), Vec::new());
    }
}
