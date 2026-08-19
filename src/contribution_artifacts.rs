//! The process artifacts a contribution carries (issue #1021).
//!
//! A branch that only changes code cannot reach `main` here. The repository
//! also asks for a changelog fragment (`scripts/check-changelog-fragment.rs`)
//! and a pull-request body that closes the issue it answers
//! (`scripts/check-pull-request-link.rs`). Issue #1021 observed that Formal AI
//! writes neither, so every change it authors stalls until a human writes the
//! paperwork by hand — which is exactly the last step of the loop that keeps
//! `data/meta/self-hosting-ledger.lino` reading `0.00% self-authored`.
//!
//! [`compose`] closes that gap. It takes what only the author of a change
//! knows — which issue, what broke, why, what changed, how it is covered — and
//! renders the two artifacts the gates read. Every word of their wording comes
//! from [`crate::seed::contribution_artifact_vocabulary`], so this module holds
//! no prose of its own (R379) and a maintainer retunes the phrasing by editing
//! `data/seed/contribution-artifacts.lino`.

use crate::seed::{contribution_artifact_vocabulary, ContributionArtifactVocabulary};

/// What the author of a change supplies; everything else is derived.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Contribution {
    /// Issue number the change answers.
    pub issue: u64,
    /// `owner/repo` the issue lives in.
    pub repository: String,
    /// Short identifier for the fragment's file name (`behaviour_range`).
    pub slug: String,
    /// `YYYYMMDD_HHMMSS`, supplied by the caller so composition stays pure.
    pub timestamp: String,
    /// Release bump: one of [`ContributionArtifactVocabulary::bumps`].
    pub bump: String,
    /// Changelog category name: one of the seeded category keys.
    pub category: String,
    /// One-line summary; becomes the changelog entry and the request's title.
    pub title: String,
    /// What was broken, in the author's words.
    pub problem: String,
    /// Why it was broken.
    pub cause: String,
    /// What the change does about it.
    pub change: String,
    /// Commands that verify the change, one per line of the body.
    pub verification: Vec<String>,
}

/// The rendered artifacts, ready to be written and posted.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ContributionArtifacts {
    /// Repository-relative path the fragment belongs at.
    pub changelog_fragment_path: String,
    /// The fragment's full text, frontmatter included.
    pub changelog_fragment: String,
    /// Title for the pull request.
    pub pull_request_title: String,
    /// Body for the pull request, closing keyword first.
    pub pull_request_body: String,
}

/// Render the artifacts for `contribution`, or `None` when it names a bump or a
/// category the seed does not define.
///
/// Rejecting an unknown bump is not pedantry: `scripts/check-release-needed.rs`
/// reads that field to decide the next version, and a fragment carrying a bump
/// it does not recognise is a release that silently does not happen.
#[must_use]
pub fn compose(contribution: &Contribution) -> Option<ContributionArtifacts> {
    compose_with(contribution, &contribution_artifact_vocabulary())
}

/// [`compose`] against an explicit vocabulary, for tests that vary the seed.
#[must_use]
pub fn compose_with(
    contribution: &Contribution,
    vocab: &ContributionArtifactVocabulary,
) -> Option<ContributionArtifacts> {
    if !vocab.bumps.iter().any(|bump| bump == &contribution.bump) {
        return None;
    }
    let category = vocab
        .categories
        .iter()
        .find(|candidate| candidate.name == contribution.category)?;
    if contribution.title.trim().is_empty() || contribution.repository.trim().is_empty() {
        return None;
    }

    let mut path = String::new();
    path.push_str(&vocab.changelog_directory);
    path.push('/');
    path.push_str(&contribution.timestamp);
    path.push('_');
    path.push_str(&issue_token(contribution, vocab));
    path.push('_');
    path.push_str(&contribution.slug);
    path.push('.');
    path.push_str(&vocab.changelog_extension);

    let mut fragment = String::new();
    fragment.push_str(&vocab.changelog_fence);
    fragment.push('\n');
    fragment.push_str(&vocab.bump_field);
    fragment.push(':');
    fragment.push(' ');
    fragment.push_str(&contribution.bump);
    fragment.push('\n');
    fragment.push_str(&vocab.changelog_fence);
    fragment.push_str("\n\n");
    fragment.push_str(&vocab.changelog_heading_marker);
    fragment.push(' ');
    fragment.push_str(&category.heading);
    fragment.push_str("\n\n");
    fragment.push_str(&vocab.entry_marker);
    fragment.push(' ');
    fragment.push_str(contribution.title.trim());
    fragment.push('\n');

    Some(ContributionArtifacts {
        changelog_fragment_path: path,
        changelog_fragment: fragment,
        pull_request_title: contribution.title.trim().to_owned(),
        pull_request_body: pull_request_body(contribution, vocab),
    })
}

/// The `issue_1021` element of a fragment's file name.
///
/// The word is the one `scripts/check-fragment-release-map.rs` looks for, so it
/// is named in seed data next to the rest of the fragment's shape.
fn issue_token(contribution: &Contribution, vocab: &ContributionArtifactVocabulary) -> String {
    let mut token = vocab.issue_token.clone();
    token.push('_');
    token.push_str(&contribution.issue.to_string());
    token
}

/// Render the body: the closing line first, then one section per seeded
/// section, in seed order.
///
/// The closing line leads because `scripts/check-pull-request-link.rs` reads
/// the whole body but a reviewer reads the first line, and both need to see the
/// same thing: which issue merging this closes.
fn pull_request_body(
    contribution: &Contribution,
    vocab: &ContributionArtifactVocabulary,
) -> String {
    let url = vocab
        .issue_url
        .replace("{repository}", &contribution.repository)
        .replace("{issue}", &contribution.issue.to_string());
    let mut body = String::new();
    body.push_str(&vocab.closing_keyword);
    body.push(' ');
    body.push_str(&url);
    body.push('\n');
    for section in &vocab.sections {
        let content = match section.name.as_str() {
            "problem" => contribution.problem.trim().to_owned(),
            "cause" => contribution.cause.trim().to_owned(),
            "change" => contribution.change.trim().to_owned(),
            "verification" => verification_section(contribution, vocab),
            _ => String::new(),
        };
        if content.is_empty() {
            continue;
        }
        body.push('\n');
        body.push_str(&vocab.pull_request_heading_marker);
        body.push(' ');
        body.push_str(&section.heading);
        body.push_str("\n\n");
        body.push_str(&content);
        body.push('\n');
    }
    body
}

/// The verification section: a lead sentence, then the commands as a list.
fn verification_section(
    contribution: &Contribution,
    vocab: &ContributionArtifactVocabulary,
) -> String {
    let commands: Vec<String> = contribution
        .verification
        .iter()
        .map(|command| {
            let mut entry = vocab.entry_marker.clone();
            entry.push_str(" `");
            entry.push_str(command.trim());
            entry.push('`');
            entry
        })
        .collect();
    if commands.is_empty() {
        return String::new();
    }
    let mut section = vocab.verification_lead.clone();
    section.push_str(":\n\n");
    section.push_str(&commands.join("\n"));
    section
}
