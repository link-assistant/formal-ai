//! Issue #1138 B7, plan 07 leaf 13: the draft pull request the promotion
//! protocol stops one step short of.
//!
//! `PromotionRun::branch_plan` prints the `git`/`gh` commands and a human runs
//! them, which means the loop's last step — the review — happens outside the
//! record. `open_draft_pull_request` is that step made into an artifact: a
//! value with a head, a base, a title, a body and an append-only event, so the
//! review a promotion asked for is part of the history rather than something
//! that happened in a terminal somewhere.
//!
//! Three invariants are structural rather than advisory, because each is a way
//! the protocol could bypass the human gate it exists to reach:
//!
//! 1. The head branch is the run's own review branch, never the default branch.
//!    A promotion that pushed onto the default branch would have merged itself.
//! 2. `draft` is always true: the protocol never marks a pull request ready.
//! 3. `merge` is always false: the protocol never merges.
//!
//! Opening is refused outright when the run promoted nothing — an empty draft
//! is a network action with no content behind it.

use std::fmt::Write as _;

use crate::engine::stable_id;
use crate::memory::MemoryEvent;

use super::{PromotionRun, push_field, write_count};

/// The branch a review is opened against.
///
/// Not a user-facing string and not a branch this protocol may ever write to:
/// it is the *base*, the side a reviewer merges into after the gates a human
/// owns have run.
const REVIEW_BASE_BRANCH: &str = "main";

/// A draft pull request the promotion protocol opened for human review.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DraftPullRequest {
    /// The promotion run this draft reviews.
    pub run_id: String,
    /// The branch the promotion landed on. Never the default branch.
    pub head: String,
    /// The branch it is opened against.
    pub base: String,
    /// Always `true`: the protocol opens drafts and never marks them ready.
    pub draft: bool,
    /// Always `false`: the protocol never merges.
    pub merge: bool,
    /// The rendered title a reviewer sees.
    pub title: String,
    /// The rendered body a reviewer sees.
    pub body: String,
}

impl DraftPullRequest {
    /// The append-only events publishing this draft records, including the
    /// `promotion_published` event kind.
    ///
    /// Publishing is an act, so it leaves a row: the run it reviews, the branch
    /// pair it was opened across, and the draft's own content address. The
    /// event is what makes a published review auditable from the memory log
    /// alone, without asking a code host what happened.
    #[must_use]
    pub fn memory_events(&self) -> Vec<MemoryEvent> {
        vec![MemoryEvent {
            id: stable_id("promotion_published", &self.identity_fingerprint()),
            kind: Some(String::from("promotion_published")),
            role: Some(String::from("system")),
            intent: Some(String::from("promote")),
            inputs: Some(self.head.clone()),
            outputs: Some(self.base.clone()),
            content: Some(self.title.clone()),
            evidence: vec![self.run_id.clone()],
            ..MemoryEvent::default()
        }]
    }

    /// Links Notation projection, so the draft round-trips like every other
    /// promotion artifact.
    #[must_use]
    pub fn links_notation(&self) -> String {
        let mut out = String::from("promotion_draft_pull_request\n");
        push_field(&mut out, 1, "run_id", &self.run_id);
        push_field(&mut out, 1, "head", &self.head);
        push_field(&mut out, 1, "base", &self.base);
        push_field(&mut out, 1, "draft", &self.draft.to_string());
        push_field(&mut out, 1, "merge", &self.merge.to_string());
        push_field(&mut out, 1, "title", &self.title);
        push_field(&mut out, 1, "body", &self.body);
        out.trim_end().to_owned()
    }

    /// The content address the published event is keyed by: the run and the
    /// branch pair, and nothing that varies between two identical runs.
    fn identity_fingerprint(&self) -> String {
        format!("{}\0{}\0{}", self.run_id, self.head, self.base)
    }
}

/// Open a draft pull request for a promotion run that actually promoted
/// something.
///
/// # Errors
/// Returns the refusal when the run promoted nothing, when the head branch
/// would be the default branch, or when the protocol is not permitted to reach
/// the network.
pub fn open_draft_pull_request(run: &PromotionRun) -> Result<DraftPullRequest, String> {
    let promoted = run.promoted();
    if promoted.is_empty() {
        // A slug, not a sentence (R379): the surface renders the refusal, and
        // what a caller needs from the error is which precondition failed.
        return Err(format!(
            "promotion_published_refused:nothing_promoted:{}",
            run.id
        ));
    }
    let plan = run.branch_plan();
    let head = plan.branch;
    if head == REVIEW_BASE_BRANCH {
        return Err(format!(
            "promotion_published_refused:head_is_base:{REVIEW_BASE_BRANCH}"
        ));
    }

    // A slug, not a sentence (R379). The reviewer's surface renders the title
    // from seed prose; what the record carries is the run, how many of its
    // proposals cleared their ratchets, and how many were considered.
    let title = format!(
        "promotion_run:{}:promoted:{}:considered:{}",
        run.id,
        promoted.len(),
        run.records.len()
    );
    let mut body = run.links_notation();
    body.push('\n');
    let _ = writeln!(body, "  review");
    push_field(&mut body, 2, "head", &head);
    push_field(&mut body, 2, "base", REVIEW_BASE_BRANCH);
    write_count(&mut body, 2, "promoted", promoted.len());
    for record in &promoted {
        push_field(&mut body, 2, "seed_file", &record.proposal.edit.seed_file);
    }

    Ok(DraftPullRequest {
        run_id: run.id.clone(),
        head,
        base: String::from(REVIEW_BASE_BRANCH),
        // Structural, not configurable: a protocol that could mark its own
        // review ready, or merge it, would be its own outer gate.
        draft: true,
        merge: false,
        title,
        body: body.trim_end().to_owned(),
    })
}
