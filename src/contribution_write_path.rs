//! The rung of the mutating-action ladder that publishes a change (#944, #1021).
//!
//! Issue #943 records what a missing ladder costs: a harness reached `gh issue
//! create` and filed issues nobody asked for. Issue #1021 pins the two rungs
//! that matter for a contribution and asks that both be tested:
//!
//! 1. **Publishing a change** — `gh pr create`, `gh pr edit`, `git push` — is
//!    refused by default and permitted only when the operator sets the opt-in
//!    named in `data/seed/contribution-artifacts.lino`. The default is refusal
//!    because the operator, not Formal AI, decides when a branch becomes
//!    something other people see.
//! 2. **Actions nobody delegated** — `gh issue create`, `gh pr merge`,
//!    `gh repo delete` — are refused in *both* states. An opt-in that also
//!    unlocks these is not an opt-in to publishing, and merging stays a human
//!    decision.
//!
//! The ladder governs the write path Formal AI takes *on its own behalf* --
//! [`plan_publication`], the steps that turn a composed contribution into a pull
//! request -- and not a command an operator names. Issue #749 pinned `execute
//! git push` as explicit passthrough and issue #687 pinned "report this on
//! GitHub" as `gh issue create`; those are delegated by the person typing them,
//! and refusing them would be the over-refusal issue #824 reports.
//!
//! The ladder decides in terms of the action it recognises; it renders no
//! explanation, so it holds no prose (R379). A caller that needs to say why
//! looks the reason up by its [`WritePathRefusal`] slug.

use std::env;

use crate::seed::{contribution_artifact_vocabulary, WritePathVocabulary};

/// What the ladder says about one command.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WritePathDecision {
    /// Not a mutating action the ladder governs; the caller proceeds as before.
    Unaffected,
    /// A governed action, permitted because the opt-in is present.
    Permitted,
    /// A governed action, refused.
    Refused(WritePathRefusal),
}

/// Why a governed action was refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WritePathRefusal {
    /// Publishing is delegated by the operator, and was not delegated here.
    OptInAbsent,
    /// The action is off the ladder entirely: no opt-in reaches it.
    NeverDelegated,
}

impl WritePathRefusal {
    /// Stable identifier a caller logs or looks a wording up by.
    #[must_use]
    pub const fn slug(self) -> &'static str {
        match self {
            Self::OptInAbsent => "write_path_opt_in_absent",
            Self::NeverDelegated => "write_path_never_delegated",
        }
    }
}

/// Whether the operator's opt-in is present in the environment.
#[must_use]
pub fn opted_in() -> bool {
    opted_in_with(&contribution_artifact_vocabulary().write_path)
}

/// [`opted_in`] against an explicit vocabulary.
#[must_use]
pub fn opted_in_with(vocab: &WritePathVocabulary) -> bool {
    !vocab.opt_in_variable.is_empty()
        && env::var(&vocab.opt_in_variable).as_deref() == Ok(vocab.opt_in_value.as_str())
}

/// Decide `command` against the ladder, reading the opt-in from the environment.
#[must_use]
pub fn decide(command: &str) -> WritePathDecision {
    let vocab = contribution_artifact_vocabulary().write_path;
    decide_with(command, &vocab, opted_in_with(&vocab))
}

/// Decide `command` against an explicit vocabulary and opt-in state, so both
/// states are reachable from one process.
#[must_use]
pub fn decide_with(
    command: &str,
    vocab: &WritePathVocabulary,
    opted_in: bool,
) -> WritePathDecision {
    let words: Vec<String> = command
        .to_lowercase()
        .split_whitespace()
        .map(ToOwned::to_owned)
        .collect();
    if vocab.refused.iter().any(|action| names(&words, action)) {
        return WritePathDecision::Refused(WritePathRefusal::NeverDelegated);
    }
    if vocab.opt_in.iter().any(|action| names(&words, action)) {
        return if opted_in {
            WritePathDecision::Permitted
        } else {
            WritePathDecision::Refused(WritePathRefusal::OptInAbsent)
        };
    }
    WritePathDecision::Unaffected
}

/// Whether `command` may run: refusals are the only thing that stops it, and an
/// action the ladder does not govern is not its business.
#[must_use]
pub fn permits(command: &str) -> bool {
    !matches!(decide(command), WritePathDecision::Refused(_))
}

/// Whether the command's words carry the action's words in a row.
///
/// A run rather than a prefix, because a command reaches the action through a
/// shell (`cd repo && gh pr create`) as readily as it opens with it, and a
/// ladder that only reads the first word is a ladder with a step missing.
fn names(words: &[String], action: &str) -> bool {
    let action: Vec<&str> = action.split_whitespace().collect();
    if action.is_empty() || words.len() < action.len() {
        return false;
    }
    words
        .windows(action.len())
        .any(|window| window.iter().zip(&action).all(|(word, part)| word == part))
}

/// The pull request a composed contribution would open, and the branch it is on.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Publication {
    /// `owner/name` of the repository the pull request targets.
    pub repository: String,
    /// Branch the contribution was committed to.
    pub branch: String,
    /// Pull-request title, as composed by [`crate::contribution_artifacts`].
    pub title: String,
    /// Path of the file holding the composed pull-request body.
    pub body_file: String,
}

/// The commands that publish `publication`, or the rung that stopped them.
///
/// This is the write path issue #1021 asks to be tested in both states: with the
/// opt-in absent every step is refused, and with it present the same steps are
/// permitted -- while an action on the never-delegated rung stays refused in
/// both.
///
/// # Errors
///
/// Returns the refusal of the first step the ladder stops.
pub fn plan_publication(publication: &Publication) -> Result<Vec<String>, WritePathRefusal> {
    let vocab = contribution_artifact_vocabulary().write_path;
    let opted_in = opted_in_with(&vocab);
    plan_publication_with(publication, &vocab, opted_in)
}

/// [`plan_publication`] against an explicit vocabulary and opt-in state.
///
/// # Errors
///
/// Returns the refusal of the first step the ladder stops.
pub fn plan_publication_with(
    publication: &Publication,
    vocab: &WritePathVocabulary,
    opted_in: bool,
) -> Result<Vec<String>, WritePathRefusal> {
    let mut commands = Vec::with_capacity(vocab.publication.len());
    for step in &vocab.publication {
        let command = step
            .replace("{repository}", &as_one_argument(&publication.repository))
            .replace("{branch}", &as_one_argument(&publication.branch))
            .replace("{title}", &as_one_argument(&publication.title))
            .replace("{body_file}", &as_one_argument(&publication.body_file));
        match decide_with(&command, vocab, opted_in) {
            WritePathDecision::Refused(refusal) => return Err(refusal),
            WritePathDecision::Permitted | WritePathDecision::Unaffected => {
                commands.push(command);
            }
        }
    }
    Ok(commands)
}

/// Render a slot value so a shell passes it to the command as exactly one
/// argument.
///
/// A composed pull-request title is a sentence, so the unquoted substitution
/// this started as produced `gh pr create --title Code the full reported range
/// ...`: a title truncated at its first space, with the rest of the sentence
/// arriving as positional arguments. Values that are already one shell word are
/// left alone -- a quoted branch name reads as noise in a log -- and anything
/// else is single-quoted, with embedded quotes closed and reopened the way
/// `shlex.quote` does.
fn as_one_argument(value: &str) -> String {
    let is_bare =
        |character: char| character.is_ascii_alphanumeric() || "@%+=:,./-_".contains(character);
    if !value.is_empty() && value.chars().all(is_bare) {
        return value.to_owned();
    }
    format!("'{}'", value.replace('\'', r"'\''"))
}
