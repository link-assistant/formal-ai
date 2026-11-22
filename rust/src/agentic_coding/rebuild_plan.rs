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
//! source-links, explain, and repair-strategy recipes emit their documents.
//!
//! Like [`super::source_links`], [`super::explain`], and [`super::change_request`], the
//! emitted document depends on the whole source tree (the reattached artifacts'
//! content ids change with every edit), so it is asserted *live* in the issue-#558 tests
//! and never pinned byte-for-byte in a committed `data/meta/*.lino`. The reattached
//! artifacts are grounded against the real repository bytes and the owned manifest, so
//! the recipe cannot answer with a fabricated artifact. Nothing here rebuilds or
//! restarts anything: the plan is the reviewable *product*, so the "recompile and
//! reattach" guardrail (observable, testable, reversible, human-approved) is preserved.
//! Neural inference stays a NON-GOAL — the plan is a deterministic function of the
//! accepted change and the embedded artifacts.

use serde::Deserialize;

use crate::change_request::AcceptedChange;
use crate::rebuild_plan::RebuildPlan;

use super::repair_input::{data, owned_source};

pub const REBUILD_PATH: &str = "rebuild-and-reattach.lino";

/// Keywords that mark a user turn as the rebuild-and-reattach recipe.
///
/// Deliberately narrow: every keyword pins the "reattach the rebuilt worker to the UI"
/// intent, and none overlaps the other recipes' keywords (in particular the source-links
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

#[derive(Deserialize)]
struct RebuildInput {
    change_id: String,
}

/// Join the requested change id to an approval recorded by the operator.
#[must_use]
pub fn plan_from_task(task: &str) -> Option<RebuildPlan> {
    let input = data::<RebuildInput>(task)?;
    let path = std::env::var_os("FORMAL_AI_ACCEPTED_CHANGES")?;
    let text = crate::learning_ledger::read_review_document(&path).ok()?;
    plan_from_accepted_changes(&input.change_id, &text)
}

/// Resolve an accepted change from already authenticated operator records.
#[must_use]
pub fn plan_from_accepted_changes(change_id: &str, records: &str) -> Option<RebuildPlan> {
    let records: Vec<AcceptedChange> = serde_json::from_str(records).ok()?;
    let mut matching = records
        .iter()
        .filter(|record| record.change_id == change_id);
    let accepted = matching.next()?;
    let suffix = change_id.strip_prefix("change_request_")?;
    if suffix.len() != 16
        || !suffix
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
        || matching.next().is_some()
        || accepted.benchmark_passed == 0
        || accepted.benchmark_passed as u64 > (1_u64 << 53) - 1
        || [
            &accepted.request,
            &accepted.requirement,
            &accepted.test,
            &accepted.benchmark_suite,
            &accepted.reviewer,
        ]
        .iter()
        .any(|value| value.trim().is_empty())
    {
        return None;
    }
    owned_source(&accepted.target_module)?;
    Some(RebuildPlan::for_accepted_change(accepted))
}

#[must_use]
pub fn render_document(plan: &RebuildPlan) -> String {
    format!("{}\n", plan.links_notation().trim_end())
}

#[must_use]
pub fn final_answer(plan: &RebuildPlan, document: &str) -> String {
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
