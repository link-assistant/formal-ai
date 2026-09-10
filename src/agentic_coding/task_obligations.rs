//! A coding task that names several artifacts is finished when all of them are.
//!
//! Issue #1099: one prompt named two files -- *"Two files. First, edit the
//! tracked file `self-hosting-attribution.rs`: ... Second, write a new file
//! `fragment.md` whose first three lines are ..."*. Formal AI edited the first,
//! answered `Final("Added ... and observed the result.")`, and the session
//! ended in five seconds with the second file never written. The whole prompt
//! had arrived; the second clause was read and dropped, not truncated.
//!
//! The cause is structural: every composer here takes a request and returns one
//! plan for one target, so "the task" and "the first artifact named by the
//! task" were the same thing. This module separates them. A request is split
//! into obligations at its own enumeration cues (seeded, four languages), each
//! obligation is planned by the existing composers exactly as a standalone
//! request would be, and the session answers `Final` only when no obligation is
//! outstanding.
//!
//! Splitting is deliberately conservative: a request with no enumeration cue,
//! or whose clauses do not each name an artifact, yields one obligation and
//! behaves exactly as before. Reading a single-artifact request as two would
//! invent work the user never asked for, which is a worse failure than the one
//! this fixes.

use super::general_planner::compose_general_change_plan;
use super::progress::Progress;
use crate::protocol::ChatMessage;
use crate::seed;

/// One artifact a request obliges the session to produce.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Obligation {
    /// The clause as the user wrote it, planned as a request in its own right.
    pub request: String,
    /// The workspace path this clause names.
    pub target: String,
}

/// Split a request into the artifacts it obliges, in the order it names them.
///
/// Returns `None` when the request names fewer than two artifacts, so callers
/// keep their existing single-target path rather than routing every request
/// through a list of one.
#[must_use]
pub fn obligations(request: &str) -> Option<Vec<Obligation>> {
    let clauses = split_at_enumeration_cues(request);
    if clauses.len() < 2 {
        return None;
    }
    let mut obligations: Vec<Obligation> = Vec::new();
    for clause in clauses {
        // A clause counts only when the ordinary composer can read an artifact
        // out of it. Prose between two named files ("Then tell me what
        // changed") names nothing and adds no obligation.
        let Some(plan) = compose_general_change_plan(&clause) else {
            continue;
        };
        if obligations
            .iter()
            .any(|existing| existing.target == plan.target)
        {
            continue;
        }
        obligations.push(Obligation {
            request: clause,
            target: plan.target,
        });
    }
    (obligations.len() > 1).then_some(obligations)
}

/// The clauses of a request, cut before each enumeration cue that opens one.
///
/// A cue is only a cut when it opens a clause -- at the start of a sentence, or
/// just after one ends. "First" inside "the first line must be exactly `---`"
/// is describing a file's contents, not enumerating an obligation, and the
/// #1099 prompt contains exactly that phrase in its second clause.
fn split_at_enumeration_cues(request: &str) -> Vec<String> {
    let cues = seed::lexicon().words_for_role(seed::ROLE_ENUMERATION_CUE);
    if cues.is_empty() {
        return vec![request.to_owned()];
    }
    let mut boundaries = vec![0_usize];
    for (index, _) in request.char_indices() {
        if index == 0 || !opens_a_clause(request, index) {
            continue;
        }
        let rest = request[index..].to_lowercase();
        if cues.iter().any(|cue| starts_with_cue(&rest, cue)) {
            boundaries.push(index);
        }
    }
    boundaries.push(request.len());
    boundaries.dedup();
    boundaries
        .windows(2)
        .map(|window| request[window[0]..window[1]].trim().to_owned())
        .filter(|clause| !clause.is_empty())
        .collect()
}

/// Whether the byte at `index` begins a clause: the text before it ends a
/// sentence, or is nothing but whitespace.
fn opens_a_clause(request: &str, index: usize) -> bool {
    let before = request[..index].trim_end();
    before.is_empty() || before.ends_with(['.', '!', '?', ';', ':', '\n'])
}

/// Whether `rest` opens with `cue` as a whole word (or, for scripts without
/// word spacing, as its own leading run of characters).
fn starts_with_cue(rest: &str, cue: &str) -> bool {
    let Some(after) = rest.strip_prefix(cue) else {
        return false;
    };
    after
        .chars()
        .next()
        .is_none_or(|character| !character.is_alphanumeric())
}

/// The first obligation this session has not yet satisfied.
///
/// An obligation is satisfied once the workspace has been written at its
/// target. That is the same evidence the single-target path already requires
/// before it answers, so a multi-artifact request is held to exactly the
/// standard each of its clauses would be held to alone -- no more, and no less.
#[must_use]
pub fn outstanding(request: &str, messages: &[ChatMessage]) -> Option<Obligation> {
    let obligations = obligations(request)?;
    let progress = Progress::scan(messages);
    obligations
        .into_iter()
        .find(|obligation| !progress.successful_write_for(&obligation.target))
}

