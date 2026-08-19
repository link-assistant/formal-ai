//! Contribution-artifact vocabulary loaded from
//! `data/seed/contribution-artifacts.lino` (issue #1021).
//!
//! A change is not finished when the code compiles. This repository also asks for
//! a changelog fragment and a pull-request body that closes the issue it answers,
//! and issue #1021 observed that Formal AI produces neither — so a change it
//! authors cannot reach `main` without a human writing the paperwork by hand.
//!
//! The wording of that paperwork is natural language, so it lives here rather
//! than in [`crate::contribution_artifacts`], which composes it. That is what
//! keeps the generator itself R379-clean: it knows the *shape* of a fragment and
//! of a body, and reads every word of them from seed data.

use super::parser::{parse_lino, LinoNode};
use super::CONTRIBUTION_ARTIFACTS_LINO;

/// One changelog category and the heading it renders as.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ChangelogCategory {
    /// Stable key a caller selects the category by (`fixed`).
    pub name: String,
    /// The heading the fragment carries (`Fixed`).
    pub heading: String,
}

/// One pull-request body section and the heading it renders as.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PullRequestSection {
    /// Stable key the generator fills the section by (`problem`).
    pub name: String,
    /// The heading the body carries (`What is broken`).
    pub heading: String,
}

/// Everything the process-artifact generator needs that is natural language.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ContributionArtifactVocabulary {
    /// Directory changelog fragments are written to (`changelog.d`).
    pub changelog_directory: String,
    /// Fragment file extension, without the dot (`md`).
    pub changelog_extension: String,
    /// Fence that opens and closes the fragment's frontmatter (`---`).
    pub changelog_fence: String,
    /// Frontmatter field naming the release bump (`bump`).
    pub bump_field: String,
    /// Marker that opens one changelog entry (`-`).
    pub entry_marker: String,
    /// Markdown heading marker for a fragment's category (`###`).
    pub changelog_heading_marker: String,
    /// Word that introduces the issue number in a fragment's file name.
    pub issue_token: String,
    /// Accepted release bumps, in increasing order of blast radius.
    pub bumps: Vec<String>,
    /// Accepted changelog categories and their headings.
    pub categories: Vec<ChangelogCategory>,
    /// GitHub closing keyword the body must use (`Fixes`).
    pub closing_keyword: String,
    /// Issue URL template with `{repository}` and `{issue}` slots.
    pub issue_url: String,
    /// Markdown heading marker for body sections (`##`).
    pub pull_request_heading_marker: String,
    /// Body sections in the order they are written.
    pub sections: Vec<PullRequestSection>,
    /// Sentence that introduces the verification list.
    pub verification_lead: String,
}

/// Parse `data/seed/contribution-artifacts.lino`.
#[must_use]
pub fn contribution_artifact_vocabulary() -> ContributionArtifactVocabulary {
    let tree = parse_lino(CONTRIBUTION_ARTIFACTS_LINO);
    let mut vocab = ContributionArtifactVocabulary::default();
    let Some(root) = tree.children.first() else {
        return vocab;
    };
    for group in &root.children {
        match group.name.as_str() {
            "changelog" => read_changelog(group, &mut vocab),
            "pull_request" => read_pull_request(group, &mut vocab),
            _ => {}
        }
    }
    vocab
}

fn read_changelog(group: &LinoNode, vocab: &mut ContributionArtifactVocabulary) {
    group
        .find_child_value("directory")
        .clone_into(&mut vocab.changelog_directory);
    group
        .find_child_value("extension")
        .clone_into(&mut vocab.changelog_extension);
    group
        .find_child_value("fence")
        .clone_into(&mut vocab.changelog_fence);
    group
        .find_child_value("bump_field")
        .clone_into(&mut vocab.bump_field);
    group
        .find_child_value("entry_marker")
        .clone_into(&mut vocab.entry_marker);
    group
        .find_child_value("heading_marker")
        .clone_into(&mut vocab.changelog_heading_marker);
    group
        .find_child_value("issue_token")
        .clone_into(&mut vocab.issue_token);
    vocab.bumps = children_of(group, "bumps")
        .filter(|node| node.name == "bump")
        .map(|node| node.id.clone())
        .collect();
    vocab.categories = children_of(group, "categories")
        .filter(|node| node.name == "category")
        .map(|node| ChangelogCategory {
            name: node.id.clone(),
            heading: node.find_child_value("heading").to_owned(),
        })
        .collect();
}

fn read_pull_request(group: &LinoNode, vocab: &mut ContributionArtifactVocabulary) {
    group
        .find_child_value("closing_keyword")
        .clone_into(&mut vocab.closing_keyword);
    group
        .find_child_value("issue_url")
        .clone_into(&mut vocab.issue_url);
    group
        .find_child_value("heading_marker")
        .clone_into(&mut vocab.pull_request_heading_marker);
    group
        .find_child_value("verification_lead")
        .clone_into(&mut vocab.verification_lead);
    vocab.sections = children_of(group, "sections")
        .filter(|node| node.name == "section")
        .map(|node| PullRequestSection {
            name: node.id.clone(),
            heading: node.find_child_value("heading").to_owned(),
        })
        .collect();
}

fn children_of<'a>(group: &'a LinoNode, name: &str) -> impl Iterator<Item = &'a LinoNode> {
    group
        .children
        .iter()
        .find(|child| child.name == name)
        .into_iter()
        .flat_map(|node| node.children.iter())
}
