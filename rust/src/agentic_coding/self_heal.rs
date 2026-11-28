//! Fifth agentic recipe — driving Formal AI to *run its own self-healing loop*
//! and store the resulting repair case in our data (issue #558).
//!
//! Issue #558 ("Auto learning") asks the system, when it cannot answer an input,
//! to reason about the failure, map it onto the source that would change, propose a
//! fix, gate it against a benchmark, and — only with human approval — promote the
//! lesson. [`crate::self_healing`] realises that closed loop as an auditable,
//! proposal-only [`RepairCase`]. This module makes
//! the loop reachable *through the agentic interface*: an external agent CLI
//! (`Codex`, `OpenCode`, `Gemini`, `Agent CLI`) — or the in-repo driver — asks Formal AI
//! to self-heal, and the deterministic planner walks a write → verify → final
//! recipe that emits the repair case as Links Notation, exactly like the self-AST
//! and diagram recipes emit their self-inspection documents.
//!
//! Nothing here applies a change: the recipe's product is the reviewable repair
//! case, so the "recompile and reattach" guardrail (observable, testable,
//! reversible, human-approved) is preserved. Neural inference stays a NON-GOAL —
//! the document follows the supplied failure trace and source module.

use serde::Deserialize;

use crate::self_healing::{RepairCase, SourceRoundTrip};
use crate::self_improvement::BenchmarkGateReport;

use super::repair_input::{TraceInput, data, owned_source};
use super::wording::agentic_message;

pub const SELF_HEAL_PATH: &str = "self-healing-case.lino";

/// Keywords that mark a user turn as the self-healing recipe.
const SELF_HEAL_KEYWORDS: [&str; 6] = [
    "self-healing",
    "self-heal",
    "auto-learning",
    "auto learning",
    "repair case",
    "repair loop",
];

/// Self-directed repair phrasings — the object of the fix/debug/heal is the system
/// *itself*, not an arbitrary user file.
///
/// Issue #676: plain-language requests such as "Can you fix it yourself?" should run
/// the self-healing loop, but the narrow keyword set above missed them. These phrases
/// all pin the action to the assistant ("yourself", "your own", "on your own"), so
/// they stay clear of ordinary "fix this file" / "fix the bug in main.rs" requests.
const SELF_REPAIR_PHRASES: [&str; 16] = [
    "fix it yourself",
    "fix yourself",
    "fix your self",
    "fix your own",
    "fix them yourself",
    "fix this yourself",
    "solve it yourself",
    "solve this yourself",
    "debug yourself",
    "debug it yourself",
    "heal yourself",
    "repair yourself",
    "correct yourself",
    "fix on your own",
    "fix it on your own",
    "learn from your mistake",
];

/// Whether `prompt` asks the system to run its self-healing / auto-learning loop
/// (issue #558, extended for issue #676).
#[must_use]
pub fn is_self_heal_task(prompt: &str) -> bool {
    let lower = prompt.to_lowercase();
    // A dedicated keyword, a self-directed repair phrasing, or the phrase pairing a
    // failure with self-repair routes here. Kept narrow so ordinary "fix this"
    // requests (with no self-reference) do not match.
    SELF_HEAL_KEYWORDS
        .iter()
        .any(|keyword| lower.contains(keyword))
        || SELF_REPAIR_PHRASES
            .iter()
            .any(|phrase| lower.contains(phrase))
        || ((lower.contains("cannot answer")
            || lower.contains("can't answer")
            || lower.contains("failure"))
            && (lower.contains("heal") || lower.contains("learn a")))
}

#[derive(Deserialize)]
struct RepairInput {
    trace: TraceInput,
    source_module: String,
}

/// Ground a supplied trace in owned source and record the proposal for review.
pub fn case_from_task(task: &str) -> Result<RepairCase, &'static str> {
    let input = data::<RepairInput>(task).ok_or("self_heal_input_needed")?;
    let trace = input.trace.into_trace().ok_or("self_heal_input_needed")?;
    let source = owned_source(&input.source_module).ok_or("self_heal_input_needed")?;
    let source_round_trip = SourceRoundTrip::for_module(input.source_module, source);
    if !source_round_trip.faithful {
        return Err("self_heal_source_unavailable");
    }
    Ok(RepairCase::from_trace(
        trace,
        source_round_trip,
        BenchmarkGateReport::absent_for_issue_362(),
    ))
}

#[must_use]
pub fn render_document(case: &RepairCase) -> String {
    format!("{}\n", case.links_notation().trim_end())
}

#[must_use]
pub fn final_answer(case: &RepairCase, document: &str) -> String {
    agentic_message(
        "self_heal_final_answer",
        &[
            ("module", &case.source_round_trip.module_path),
            ("outcome", case.outcome.slug()),
            ("path", SELF_HEAL_PATH),
            ("document", document.trim_end()),
        ],
    )
}
