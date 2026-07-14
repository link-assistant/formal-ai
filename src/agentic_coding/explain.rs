//! Eighth agentic recipe — driving Formal AI to *explain how it itself works*,
//! grounded in its own source, data, and tests, reachable through the agentic
//! interface (issue #558).
//!
//! Issue #558 ("Auto learning") asks that a user be able to *"ask how Formal AI
//! itself works"* and receive an answer *"grounded in its source and data"* rather
//! than prose docs alone (`R558-08`). [`crate::self_explanation`] composes that
//! grounded answer: an ordered set of topics, each citing the *real* artifacts it
//! rests on (source files resolved through the owned manifest, generated data
//! artifacts, and the tests that lock the behaviour). This module makes it reachable
//! *through the agentic interface*: an external agent CLI (`Codex`, `OpenCode`,
//! `Gemini`, `Agent CLI`) — or the in-repo driver — asks Formal AI how it works, and
//! the deterministic planner walks a write → verify → final recipe that emits the
//! grounded explanation as Links Notation, exactly like the self-healing, ledger,
//! source-links, self-AST, and diagram recipes emit their self-inspection documents.
//!
//! Like [`super::source_links`], the emitted document depends on the whole source
//! tree (each source citation's `content_id` and the manifest id change with every
//! edit), so it is asserted *live* in the issue-#558 tests and never pinned
//! byte-for-byte in a committed `data/meta/*.lino`. Every source citation is verified
//! against the owned manifest at construction ([`crate::self_explanation::Citation::source`]
//! panics on an unknown path), so the recipe cannot answer with a fabricated source
//! reference. Neural inference stays a NON-GOAL — the explanation is a deterministic
//! function of the embedded source and a fixed set of cited paths.

use std::sync::OnceLock;

use crate::self_explanation::{canonical_explanation, SystemExplanation};

/// The workspace path the planner writes the generated explanation document to.
pub const EXPLAIN_PATH: &str = "how-formal-ai-works.lino";

/// The canonical grounded explanation, computed once per process.
///
/// Building it resolves every source citation against the owned manifest; the recipe
/// touches it several times per run (planner write step, verify step, final answer)
/// and a server may serve it repeatedly, so memoising keeps the loop responsive
/// without changing its deterministic result.
fn cached_explanation() -> &'static SystemExplanation {
    static EXPLANATION: OnceLock<SystemExplanation> = OnceLock::new();
    EXPLANATION.get_or_init(canonical_explanation)
}

/// A *differently worded* request for the self-explanation recipe.
///
/// The router recognises the intent from the words, not a hardcoded string.
/// Deliberately avoids the source-links keywords ("recompile", whole-source-to-links
/// "and back") and the ledger keywords so the self-inspection recipes never collide.
pub const EXPLAIN_TASK: &str =
    "Explain how Formal AI itself works, and ground the answer in its own source \
     files, data artifacts, and tests rather than prose documentation.";

/// Keywords that mark a user turn as the self-explanation recipe.
///
/// Deliberately narrow: every keyword pins the "how <the system> works" intent so an
/// ordinary "explain this text" or "how do I …" request does not match, and none
/// overlaps the other self-inspection recipes' keywords.
const EXPLAIN_KEYWORDS: [&str; 5] = [
    "how formal ai works",
    "how does formal ai work",
    "how formal ai itself works",
    "how the system itself works",
    "explain how formal ai",
];

/// Whether `prompt` asks how Formal AI itself works with a grounded answer (issue
/// #558's `R558-08`).
#[must_use]
pub fn is_explain_task(prompt: &str) -> bool {
    let lower = prompt.to_lowercase();
    // A dedicated keyword routes here directly.
    if EXPLAIN_KEYWORDS
        .iter()
        .any(|keyword| lower.contains(keyword))
    {
        return true;
    }
    // Scoped fallback: an explicit "how does <the system> work" question that asks for
    // a grounded answer (source plus data or tests), not prose docs — so ordinary
    // "how do I …" requests and the sibling recipes never match.
    let asks_how_it_works = lower.contains("how")
        && (lower.contains("work") || lower.contains("explain"))
        && (lower.contains("formal ai")
            || lower.contains("the system")
            || lower.contains("yourself"));
    let grounded = lower.contains("source") && (lower.contains("data") || lower.contains("test"));
    asks_how_it_works && grounded
}

/// Render the grounded self-explanation document (Links Notation). Deterministic and
/// ends with exactly one trailing newline.
///
/// The output is what the Agent CLI writes to [`EXPLAIN_PATH`]. It is asserted *live*
/// (never committed byte-for-byte) in the issue-#558 tests, because the source
/// citations' content ids and the manifest id change with every source edit —
/// committing it would force a regenerate on every unrelated PR.
#[must_use]
pub fn render_document() -> String {
    format!("{}\n", cached_explanation().links_notation().trim_end())
}

/// The grounded explanation backing the recipe. Exposed so tests can assert the
/// citations resolve without rebuilding it.
#[must_use]
pub fn explanation() -> SystemExplanation {
    cached_explanation().clone()
}

/// The self-contained final answer: a natural-language summary plus the generated
/// grounded explanation document inline.
#[must_use]
pub fn final_answer(document: &str) -> String {
    let explanation = cached_explanation();
    format!(
        "Here is how Formal AI works, grounded in its own source, data, and tests: across {sections} \
         topics with {citations} citations, every claim points at a real artifact — source files \
         content-addressed through the owned manifest, the data artifacts they generate, and the \
         tests that lock the behaviour — rather than prose docs. The answer is a deterministic \
         function of the embedded source, so it cannot cite anything the repository does not ship.\n\n\
         Generated document ({EXPLAIN_PATH}):\n\n{document}",
        sections = explanation.section_count(),
        citations = explanation.citation_count(),
        document = document.trim_end(),
    )
}
