//! Eleventh agentic recipe — recompile Formal AI and reattach the improved worker to
//! the UI, reachable via the agentic interface (issue #558, `R558-06`).
//!
//! Issue #558 ("Auto learning") asks that Formal AI be able to *"recompile and reattach
//! the improved code to the UI"* — the final step of the self-change loop, once a change
//! is accepted. [`crate::rebuild_plan`] composes that as a deterministic, human-gated
//! plan derived from an already-accepted change ([`crate::change_request::AcceptedChange`],
//! which only exists after a green benchmark gate *and* a human approval). This module
//! makes it reachable *through the agentic interface*: an external agent CLI (`Codex`,
//! `OpenCode`, `Gemini`, `Agent CLI`) — or the in-repo driver — asks Formal AI to
//! rebuild and reattach the improved worker to the UI, and the deterministic planner
//! walks a write → verify → final recipe that emits the ordered, reversible
//! rebuild-and-reattach pipeline as Links Notation, exactly like the change-request,
//! source-graph, explain, and repair-strategy recipes emit their documents.
//!
//! Like [`super::source_graph`], [`super::explain`], and [`super::change_request`], the
//! emitted document depends on the whole source tree (the reattached artifacts'
//! content ids change with every edit), so it is asserted *live* in the issue-#558 tests
//! and never pinned byte-for-byte in a committed `data/meta/*.lino`. The reattached
//! artifacts are grounded against the real repository bytes and the owned manifest, so
//! the recipe cannot answer with a fabricated artifact. Nothing here rebuilds or
//! restarts anything: the plan is the reviewable *product*, so the "recompile and
//! reattach" guardrail (observable, testable, reversible, human-approved) is preserved.
//! Neural inference stays a NON-GOAL — the plan is a deterministic function of the
//! accepted change and the embedded artifacts.

use std::sync::OnceLock;

use crate::rebuild_plan::{canonical_rebuild_plan, RebuildPlan};

/// The workspace path the planner writes the generated rebuild-and-reattach plan to.
pub const REBUILD_PATH: &str = "rebuild-and-reattach.lino";

/// The canonical rebuild-and-reattach plan, computed once per process.
///
/// Building it accepts the canonical change through the same green-gate-plus-approval
/// the ledger and change request enforce, then grounds every reattached artifact; the
/// recipe touches it several times per run (planner write step, verify step, final
/// answer) and a server may serve it repeatedly, so memoising keeps the loop responsive
/// without changing its deterministic result.
fn cached_rebuild_plan() -> &'static RebuildPlan {
    static PLAN: OnceLock<RebuildPlan> = OnceLock::new();
    PLAN.get_or_init(canonical_rebuild_plan)
}

/// A *differently worded* request for the rebuild-and-reattach recipe.
///
/// The router recognises the intent from the words, not a hardcoded string.
/// Deliberately avoids the source-graph keyword "recompile" (and its "entire"/"whole"/
/// "all" fallback), the change-request "capability"/"feature"/"support" fallback, the
/// self-healing "heal"/"learn a" fallback, and the ledger/explain/repair-strategy
/// keywords, so the recipes never collide: this recipe keys on *reattaching* the rebuilt
/// worker to the UI.
pub const REBUILD_TASK: &str =
    "An improvement to Formal AI was accepted — now rebuild it and reattach the improved \
     WebAssembly worker to the UI: give me the ordered, reversible steps to regenerate \
     the worker, reattach it to the browser UI, hot-swap the local server, and verify \
     the UI uses the accepted version.";

/// Keywords that mark a user turn as the rebuild-and-reattach recipe.
///
/// Deliberately narrow: every keyword pins the "reattach the rebuilt worker to the UI"
/// intent, and none overlaps the other recipes' keywords (in particular the source-graph
/// recipe owns "recompile", which is absent here — this recipe keys on "reattach").
const REBUILD_KEYWORDS: [&str; 6] = [
    "reattach",
    "rebuild and reattach",
    "reattach the improved",
    "reattach it to the ui",
    "rebuild the wasm worker",
    "hot-swap the local server",
];

/// Whether `prompt` asks the rebuild-and-reattach recipe (issue #558's `R558-06`).
#[must_use]
pub fn is_rebuild_task(prompt: &str) -> bool {
    let lower = prompt.to_lowercase();
    REBUILD_KEYWORDS
        .iter()
        .any(|keyword| lower.contains(keyword))
}

/// Render the rebuild-and-reattach plan (Links Notation) for the canonical accepted
/// change. Deterministic and ends with exactly one trailing newline.
///
/// The output is what the Agent CLI writes to [`REBUILD_PATH`]. It is asserted *live*
/// (never committed byte-for-byte) in the issue-#558 tests, because the reattached
/// artifacts' content ids change with every source edit — committing it would force a
/// regenerate on every unrelated PR.
#[must_use]
pub fn render_document() -> String {
    format!("{}\n", cached_rebuild_plan().links_notation().trim_end())
}

/// The canonical rebuild-and-reattach plan backing the recipe. Exposed so tests can
/// assert the plan grounds its artifacts without rebuilding it.
#[must_use]
pub fn plan() -> RebuildPlan {
    cached_rebuild_plan().clone()
}

/// The self-contained final answer: a natural-language summary plus the generated
/// rebuild-and-reattach plan inline.
#[must_use]
pub fn final_answer(document: &str) -> String {
    let plan = cached_rebuild_plan();
    format!(
        "Composed the rebuild-and-reattach plan for accepted change `{change}` (approved by \
         {reviewer}): a {steps}-step, reversible pipeline that recompiles Formal AI, regenerates \
         the WebAssembly worker, reattaches it to {artifacts} grounded UI artifacts, hot-swaps the \
         local server, and verifies the UI uses the accepted version. Nothing is rebuilt or \
         restarted — the plan is the reviewable product, and every step is observable and \
         reversible, so keeping the swap stays a human decision. Neural inference is not used; the \
         plan is a deterministic function of the accepted change and the grounded artifacts.\n\n\
         Generated document ({REBUILD_PATH}):\n\n{document}",
        change = plan.change_id,
        reviewer = plan.reviewer,
        steps = plan.steps.len(),
        artifacts = plan.artifacts.len(),
        document = document.trim_end(),
    )
}
