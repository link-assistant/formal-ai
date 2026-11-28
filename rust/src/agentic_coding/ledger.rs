//! Seventh agentic recipe — driving Formal AI to *promote an approved lesson into
//! its learning ledger* and record the approved learning record in our data
//! (issue #558).
//!
//! Issue #558 asks the system, after a self-healing lesson passes its tests and a
//! human accepts it, to promote the improvement *"to mainline history as an approved
//! learning record"*. [`crate::learning_ledger`] realises that terminal, human-gated
//! promotion step. This module makes it reachable *through the agentic interface*:
//! an external agent CLI (`Codex`, `OpenCode`, `Gemini`, `Agent CLI`) — or the
//! in-repo driver — asks Formal AI to record its approved learning ledger, and the
//! deterministic planner walks a write → verify → final recipe that emits the ledger
//! as Links Notation, exactly like the self-healing, self-AST, source-links, and
//! diagram recipes emit their self-inspection documents.
//!
//! Promotion into the ledger already required *both* a green benchmark gate and an
//! explicit human approval ([`crate::learning_ledger::LearningLedger::promote`]), so
//! the emitted document is a record of an *already-approved* decision — nothing here
//! adopts anything new. The "recompile and reattach" guardrail (observable,
//! testable, reversible, human-approved) is preserved. Neural inference stays a
//! NON-GOAL — the document records the lessons configured for this runtime.

use crate::learning_ledger::LearningLedger;

pub const LEDGER_PATH: &str = "learning-ledger.lino";

/// Keywords that mark a user turn as the learning-ledger promotion recipe.
///
/// Deliberately narrow and disjoint from the self-healing keywords (which own
/// "auto learning" / "auto-learning") so the promotion recipe never captures a
/// self-healing request or vice versa.
const LEDGER_KEYWORDS: [&str; 5] = [
    "learning ledger",
    "promotion ledger",
    "promote the lesson",
    "promote the approved",
    "approved learning record",
];

/// Whether `prompt` asks the system to promote an approved lesson into / record its
/// learning ledger (issue #558's promotion step).
#[must_use]
pub fn is_ledger_task(prompt: &str) -> bool {
    let lower = prompt.to_lowercase();
    // A dedicated keyword, or an explicit "promote … lesson/learning" pairing, routes
    // here. Kept narrow so ordinary "save this" requests do not match.
    LEDGER_KEYWORDS
        .iter()
        .any(|keyword| lower.contains(keyword))
        || (lower.contains("promote")
            && (lower.contains("lesson") || lower.contains("learning"))
            && lower.contains("ledger"))
}

/// Render the approved records supplied by the operator.
#[must_use]
pub fn render_document(ledger: &LearningLedger) -> String {
    format!("{}\n", ledger.links_notation().trim_end())
}

/// Summarize the ledger and include its serialized contents.
#[must_use]
pub fn final_answer(ledger: &LearningLedger, document: &str) -> String {
    format!(
        "Recorded the approved learning ledger: {count} lesson(s) that passed their benchmark gate \
         and were approved by a human are now durable learning records, so a repeated failure is \
         answered from the ledger instead of re-derived. Promotion stayed human-gated — only \
         green, approved, faithfully round-tripping cases were recorded.\n\n\
         Generated document ({LEDGER_PATH}):\n\n{document}",
        count = ledger.len(),
        document = document.trim_end(),
    )
}
