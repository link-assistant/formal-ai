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
//! as Links Notation, exactly like the self-healing, self-AST, source-graph, and
//! diagram recipes emit their self-inspection documents.
//!
//! Promotion into the ledger already required *both* a green benchmark gate and an
//! explicit human approval ([`crate::learning_ledger::LearningLedger::promote`]), so
//! the emitted document is a record of an *already-approved* decision — nothing here
//! adopts anything new. The "recompile and reattach" guardrail (observable,
//! testable, reversible, human-approved) is preserved. Neural inference stays a
//! NON-GOAL — the document is a deterministic function of the canonical ledger.

use std::sync::OnceLock;

use crate::learning_ledger::{canonical_ledger, LearningLedger};

/// The workspace path the planner writes the generated ledger document to.
pub const LEDGER_PATH: &str = "learning-ledger.lino";

/// The canonical, human-approved learning ledger, computed once per process.
///
/// Building it promotes the canonical self-healing case, which parses a real module
/// through the CST/AST engine; the recipe touches it several times per run (planner
/// write step, verify step, final answer) and a server may serve it repeatedly, so
/// memoising keeps the loop responsive without changing its deterministic result.
fn cached_ledger() -> &'static LearningLedger {
    static LEDGER: OnceLock<LearningLedger> = OnceLock::new();
    LEDGER.get_or_init(canonical_ledger)
}

/// A *differently worded* request for the learning-ledger recipe.
///
/// The router recognises the intent from the words, not a hardcoded string.
/// Deliberately avoids the self-healing keywords ("self-heal", "auto learning",
/// "repair case") so the two auto-learning recipes never collide: this recipe is
/// the *promotion* step that follows an already-reviewed repair case.
pub const LEDGER_TASK: &str =
    "Promote the approved lesson into your learning ledger and record the approved \
     learning record in Links Notation so a repeated failure is answered from the \
     ledger next time.";

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

/// Render the learning-ledger document (Links Notation) for the canonical approved
/// ledger. Deterministic and ends with exactly one trailing newline.
///
/// The output is what the Agent CLI writes to [`LEDGER_PATH`] and what
/// `data/meta/learning-ledger.lino` is committed as, asserted byte-for-byte in the
/// issue-#558 tests.
#[must_use]
pub fn render_document() -> String {
    format!("{}\n", cached_ledger().links_notation().trim_end())
}

/// The canonical approved ledger backing the recipe. Exposed so tests can assert the
/// promotion reached a recorded, human-approved lesson without rebuilding it.
#[must_use]
pub fn ledger() -> LearningLedger {
    cached_ledger().clone()
}

/// The self-contained final answer: a natural-language summary plus the generated
/// ledger document inline.
#[must_use]
pub fn final_answer(document: &str) -> String {
    let ledger = cached_ledger();
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
