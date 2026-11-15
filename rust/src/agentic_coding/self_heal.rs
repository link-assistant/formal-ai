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
//! the document is a deterministic function of the canonical failure case.

use std::sync::OnceLock;

use crate::self_healing::{canonical_case, RepairCase};

/// The workspace path the planner writes the generated repair-case document to.
pub const SELF_HEAL_PATH: &str = "self-healing-case.lino";

/// The canonical repair case, computed once per process.
///
/// Building it parses a real module through the CST/AST engine (a source → links
/// round-trip), which is deliberately non-trivial; the recipe touches it several
/// times per run (planner write step, verify step, final answer) and a server may
/// serve it repeatedly, so memoising keeps the loop responsive without changing its
/// deterministic result.
fn cached_case() -> &'static RepairCase {
    static CASE: OnceLock<RepairCase> = OnceLock::new();
    CASE.get_or_init(canonical_case)
}

/// A *differently worded* request for the self-healing recipe.
///
/// The router recognises the intent from the words, not a hardcoded string.
/// Deliberately avoids the self-AST keywords ("cst/ast", "abstract-syntax",
/// "reason about itself") so the two self-inspection recipes never collide.
pub const SELF_HEAL_TASK: &str =
    "When you cannot answer an input, run your self-healing loop: reason about the \
     failure, map it onto the source that would change with a source-to-links \
     round-trip, learn a benchmark-gated lesson, and record the repair case in \
     Links Notation for human approval.";

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

/// Render the self-healing repair-case document (Links Notation) for the canonical
/// worked case. Deterministic and ends with exactly one trailing newline.
///
/// The output is what the Agent CLI writes to [`SELF_HEAL_PATH`] and what
/// `data/meta/self-healing-case.lino` is committed as, asserted byte-for-byte in
/// the issue-#558 tests.
#[must_use]
pub fn render_document() -> String {
    format!("{}\n", cached_case().links_notation().trim_end())
}

/// The canonical worked repair case backing the recipe. Exposed so tests can assert
/// the loop reached a reviewable, human-gated outcome without rebuilding it.
#[must_use]
pub fn case() -> RepairCase {
    cached_case().clone()
}

/// The self-contained final answer: a natural-language summary plus the generated
/// repair-case document inline.
#[must_use]
pub fn final_answer(document: &str) -> String {
    let case = cached_case();
    format!(
        "Ran the self-healing loop on an input the system could not answer directly: reasoned about \
         the failure trace, mapped it onto {module} with a verified source-to-links round-trip, \
         learned a benchmark-gated lesson, and recorded an auditable repair case ({outcome}). \
         Nothing was applied — adoption stays a human-approved review step.\n\n\
         Generated document ({SELF_HEAL_PATH}):\n\n{document}",
        module = case.source_round_trip.module_path,
        outcome = case.outcome.slug(),
        document = document.trim_end(),
    )
}
