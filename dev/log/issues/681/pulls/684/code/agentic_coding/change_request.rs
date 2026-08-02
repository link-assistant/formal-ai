//! Ninth agentic recipe — driving Formal AI to *turn a user's change request into a
//! reviewable pull request*, through the same human-gated repair loop, reachable via
//! the agentic interface (issue #558).
//!
//! Issue #558 ("Auto learning") asks that *"users must be able to ask for changes in
//! the AI system through this mechanism"* (`R558-08`'s sibling `R558-07`): a
//! natural-language change request should flow through the *same* human-gated loop the
//! self-healing slices use, producing a requirement, a test, and a patch offered as a
//! reviewable pull request. [`crate::change_request`] composes that flow. This module
//! makes it reachable *through the agentic interface*: an external agent CLI (`Codex`,
//! `OpenCode`, `Gemini`, `Agent CLI`) — or the in-repo driver — asks Formal AI to
//! change itself, and the deterministic planner walks a write → verify → final recipe
//! that emits the reviewable pull request as Links Notation, exactly like the
//! self-healing, ledger, source-graph, self-AST, explain, and diagram recipes emit
//! their documents.
//!
//! Like [`super::source_graph`] and [`super::explain`], the emitted document depends
//! on the whole source tree (the target module's manifest content id changes with
//! every edit), so it is asserted *live* in the issue-#558 tests and never pinned
//! byte-for-byte in a committed `data/meta/*.lino`. The target module is verified
//! against the owned manifest at construction ([`crate::change_request::ChangeRequest::for_module`]
//! panics on an unknown path), so the recipe cannot answer with a fabricated target.
//! Neural inference stays a NON-GOAL — the requirement, test, and patch plan are
//! deterministic functions of the request and its grounded target.

use std::sync::OnceLock;

use crate::change_request::{canonical_change_request, ChangeRequest};

/// The workspace path the planner writes the generated pull-request document to.
pub const CHANGE_PATH: &str = "requested-change.lino";

/// The canonical reviewable change request, computed once per process.
///
/// Building it grounds the target module against the owned manifest; the recipe
/// touches it several times per run (planner write step, verify step, final answer)
/// and a server may serve it repeatedly, so memoising keeps the loop responsive
/// without changing its deterministic result.
fn cached_change_request() -> &'static ChangeRequest {
    static REQUEST: OnceLock<ChangeRequest> = OnceLock::new();
    REQUEST.get_or_init(canonical_change_request)
}

/// A *differently worded* request for the self-change recipe.
///
/// The router recognises the intent from the words, not a hardcoded string.
/// Deliberately avoids the self-healing keywords ("repair loop", "auto learning"), the
/// ledger keywords ("promote", "ledger"), the explain keywords ("how … works"), and
/// the source-graph keywords ("recompile", whole-source-to-links) so the recipes never
/// collide: this recipe is the *user-initiated change* to Formal AI itself.
pub const CHANGE_TASK: &str =
    "I want to change Formal AI itself: please add a new capability to the system, and \
     route my request through the same human-gated review loop that produces a \
     requirement, a test, and a patch plan I can review as a pull request.";

/// Keywords that mark a user turn as the self-change recipe.
///
/// Deliberately narrow: every keyword pins the "change Formal AI itself" intent so an
/// ordinary "change this text" or "add these numbers" request does not match, and none
/// overlaps the other recipes' keywords.
const CHANGE_KEYWORDS: [&str; 6] = [
    "change formal ai",
    "modify formal ai",
    "add a capability to formal ai",
    "add a new capability to the system",
    "change the ai system",
    "change formal ai itself",
];

/// Whether `prompt` asks the user-initiated self-change recipe (issue #558's
/// `R558-07`).
#[must_use]
pub fn is_change_request_task(prompt: &str) -> bool {
    let lower = prompt.to_lowercase();
    // A dedicated keyword routes here directly.
    if CHANGE_KEYWORDS
        .iter()
        .any(|keyword| lower.contains(keyword))
    {
        return true;
    }
    // Scoped fallback: an explicit request to *change the system itself* (add/modify a
    // capability or feature), so an ordinary "change this line" request and the sibling
    // recipes never match.
    let targets_the_system = lower.contains("formal ai")
        || lower.contains("the ai system")
        || lower.contains("the system itself");
    let asks_to_change = (lower.contains("change")
        || lower.contains("modify")
        || lower.contains("add a"))
        && (lower.contains("capability") || lower.contains("feature") || lower.contains("support"));
    targets_the_system && asks_to_change
}

/// Render the reviewable pull-request document (Links Notation) for the canonical
/// change request. Deterministic and ends with exactly one trailing newline.
///
/// The output is what the Agent CLI writes to [`CHANGE_PATH`]. It is asserted *live*
/// (never committed byte-for-byte) in the issue-#558 tests, because the target
/// module's manifest content id changes with every source edit — committing it would
/// force a regenerate on every unrelated PR.
#[must_use]
pub fn render_document() -> String {
    format!("{}\n", cached_change_request().links_notation().trim_end())
}

/// The canonical change request backing the recipe. Exposed so tests can assert the
/// proposal grounds its target without rebuilding it.
#[must_use]
pub fn change_request() -> ChangeRequest {
    cached_change_request().clone()
}

/// The self-contained final answer: a natural-language summary plus the generated
/// reviewable pull-request document inline.
#[must_use]
pub fn final_answer(document: &str) -> String {
    let request = cached_change_request();
    format!(
        "Turned your change request into a reviewable pull request: it derives the requirement \
         \"{requirement}\", proposes the test `{test}`, and a {steps}-step patch plan against the \
         grounded module {target}. Nothing is applied — the change merges only through the same \
         human-gated loop: a green benchmark gate and your explicit approval. Neural inference is \
         not used; the requirement, test, and patch plan are a deterministic function of your \
         request and its grounded target.\n\n\
         Generated document ({CHANGE_PATH}):\n\n{document}",
        requirement = request.derived_requirement,
        test = request.proposed_test,
        steps = request.patch_plan.len(),
        target = request.target_module,
        document = document.trim_end(),
    )
}
