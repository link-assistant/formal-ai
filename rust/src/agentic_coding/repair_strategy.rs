//! Tenth agentic recipe — classify an arbitrary failure and decide how to repair.
//!
//! This drives Formal AI to *classify an arbitrary failure and decide how to repair
//! itself*, for **every** class of failure, reachable via the agentic interface (issue
//! #558, `R558-02`).
//!
//! Issue #558 ("Auto learning") asks that a failure trace can trigger a repair run
//! that changes a **solver method**, a **data record**, or a **test**. The
//! self-healing recipe ([`super::self_heal`]) executes the *solver-method* path end to
//! end, but on a single canonical failure. [`crate::repair_strategy`] generalises the
//! *front* of the loop: given an arbitrary [`UnknownTrace`](crate::self_improvement::UnknownTrace),
//! it classifies which of the three classes the repair belongs to and composes the
//! grounded, human-gated strategy for it. This module makes that classifier reachable
//! *through the agentic interface*: an external agent CLI (`Codex`, `OpenCode`,
//! `Gemini`, `Agent CLI`) — or the in-repo driver — asks Formal AI to classify a
//! failure and decide which part to repair, and the deterministic planner walks a
//! write → verify → final recipe that emits the three canonical strategies (one per
//! target class) as Links Notation, exactly like the self-healing, ledger, and diagram
//! recipes emit their documents.
//!
//! Unlike the source-links, explain, and change-request recipes, the emitted document
//! depends only on the three self-contained canonical failure traces (never on the
//! whole source tree), so it is committed byte-for-byte as
//! `data/meta/repair-strategies.lino` and asserted against a fresh render in the
//! issue-#558 tests — like [`super::self_heal`]'s repair case. Nothing here applies a
//! change: each strategy is a reviewable *plan*, so the "recompile and reattach"
//! guardrail (observable, testable, reversible, human-approved) is preserved. Neural
//! inference stays a NON-GOAL — the classification and the plan are deterministic
//! functions of the traces.

use std::fmt::Write as _;
use std::sync::OnceLock;

use crate::repair_strategy::{canonical_strategies, RepairStrategy};

/// The workspace path the planner writes the generated repair-strategies document to,
/// and the committed byte-for-byte artifact under `data/meta/`.
pub const REPAIR_STRATEGY_PATH: &str = "repair-strategies.lino";

/// The three canonical strategies (one per repair target), computed once per process.
///
/// Classifying them scans real failure traces and composes a grounded plan for each;
/// the recipe touches them several times per run (planner write step, verify step,
/// final answer) and a server may serve them repeatedly, so memoising keeps the loop
/// responsive without changing its deterministic result.
fn cached_strategies() -> &'static [RepairStrategy] {
    static STRATEGIES: OnceLock<Vec<RepairStrategy>> = OnceLock::new();
    STRATEGIES.get_or_init(canonical_strategies)
}

/// A *differently worded* request for the general repair-classification recipe.
///
/// The router recognises the intent from the words, not a hardcoded string.
/// Deliberately avoids the self-healing keywords ("self-heal", "repair case", "repair
/// loop", and any pairing of "failure" with "heal"/"learn a"), the self-AST keywords
/// ("cst/ast"), the ledger keywords ("promote", "ledger"), the explain keywords
/// ("how … works"), and the change-request keywords ("change Formal AI", a
/// capability/feature/support change) so the recipes never collide: this recipe is the
/// *general classifier* that decides which part to repair for every failure class.
pub const REPAIR_STRATEGY_TASK: &str =
    "Classify a failure and decide which part to repair: given a failure trace the \
     system could not answer, determine whether the repair is a solver method, a data \
     record, or a test change, then compose the grounded, human-gated repair strategy \
     for it — for every class of failure, not just one.";

/// Keywords that mark a user turn as the general repair-classification recipe.
///
/// Deliberately narrow: every keyword pins the "classify the failure, decide which part
/// to repair" intent, and none overlaps the other recipes' keywords (in particular the
/// self-healing recipe owns "repair case"/"repair loop", which are absent here).
const REPAIR_STRATEGY_KEYWORDS: [&str; 6] = [
    "classify the failure",
    "classify a failure",
    "repair strategy",
    "repair strategies",
    "which part to repair",
    "solver method, a data record, or a test",
];

/// Whether `prompt` asks the general repair-classification recipe (issue #558's
/// `R558-02`).
#[must_use]
pub fn is_repair_strategy_task(prompt: &str) -> bool {
    let lower = prompt.to_lowercase();
    REPAIR_STRATEGY_KEYWORDS
        .iter()
        .any(|keyword| lower.contains(keyword))
}

/// Render the repair-strategies document (Links Notation) for the three canonical
/// strategies, one per target class. Deterministic and ends with exactly one trailing
/// newline.
///
/// The output is what the Agent CLI writes to [`REPAIR_STRATEGY_PATH`] and what
/// `data/meta/repair-strategies.lino` is committed as, asserted byte-for-byte in the
/// issue-#558 tests. The document depends only on the self-contained canonical traces,
/// so committing it does not couple it to unrelated source edits.
#[must_use]
pub fn render_document() -> String {
    let strategies = cached_strategies();
    let mut out = String::from("repair_strategies\n");
    out.push_str("  covers \"solver_method,data_record,test\"\n");
    let _ = writeln!(out, "  strategy_count \"{}\"", strategies.len());
    for strategy in strategies {
        for line in strategy.links_notation().lines() {
            out.push_str("  ");
            out.push_str(line);
            out.push('\n');
        }
    }
    format!("{}\n", out.trim_end())
}

/// The canonical strategies backing the recipe. Exposed so tests can assert the
/// classifier covers every target class without rebuilding them.
#[must_use]
pub fn strategies() -> Vec<RepairStrategy> {
    cached_strategies().to_vec()
}

/// The self-contained final answer: a natural-language summary plus the generated
/// repair-strategies document inline.
#[must_use]
pub fn final_answer(document: &str) -> String {
    let strategies = cached_strategies();
    let targets = strategies
        .iter()
        .map(|strategy| strategy.target.slug())
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        "Classified {count} failure traces, one per repair class ({targets}), and composed a \
         grounded, human-gated repair strategy for each: the general repair loop now decides \
         *which part* to change — a solver method, a data record, or a test — for every failure, \
         not just one canonical case. Nothing is applied — each strategy is a reviewable plan that \
         merges only through the same human-gated loop. Neural inference is not used; every \
         classification and plan is a deterministic function of the trace.\n\n\
         Generated document ({REPAIR_STRATEGY_PATH}):\n\n{document}",
        count = strategies.len(),
        document = document.trim_end(),
    )
}
