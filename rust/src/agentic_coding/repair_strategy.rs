//! Tenth agentic recipe — classify an arbitrary failure and decide how to repair.
//!
//! This drives Formal AI to *classify an arbitrary failure and decide how to repair
//! itself*, for **every** class of failure, reachable via the agentic interface (issue
//! #558, `R558-02`).
//!
//! Issue #558 ("Auto learning") asks that a failure trace can trigger a repair run
//! that changes a **solver method**, a **data record**, or a **test**. The
//! self-healing recipe ([`super::self_heal`]) reviews a trace against its selected
//! source. Given an [`UnknownTrace`](crate::self_improvement::UnknownTrace),
//! [`crate::repair_strategy`] classifies it and composes the corresponding
//! human-gated strategy. This module makes that classifier reachable
//! *through the agentic interface*: an external agent CLI (`Codex`, `OpenCode`,
//! `Gemini`, `Agent CLI`) — or the in-repo driver — asks Formal AI to classify a
//! failure and decide which part to repair, and the deterministic planner walks a
//! write → verify → final recipe that emits the classification of the supplied
//! traces as a reviewable Links Notation plan.

use std::fmt::Write as _;

use serde::Deserialize;

use crate::repair_strategy::RepairStrategy;

use super::repair_input::{TraceInput, data};
use super::wording::agentic_message;

pub const REPAIR_STRATEGY_PATH: &str = "repair-strategies.lino";

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

#[derive(Deserialize)]
struct StrategyInput {
    traces: Vec<TraceInput>,
}

/// Classify the traces bound to this request, in their supplied order.
#[must_use]
pub fn strategies_from_task(task: &str) -> Option<Vec<RepairStrategy>> {
    let input = data::<StrategyInput>(task)?;
    if input.traces.is_empty() || input.traces.len() > 32 {
        return None;
    }
    input
        .traces
        .into_iter()
        .map(|trace| Some(RepairStrategy::classify(&trace.into_trace()?)))
        .collect()
}

fn targets(strategies: &[RepairStrategy]) -> Vec<&'static str> {
    let mut targets = Vec::new();
    for strategy in strategies {
        let slug = strategy.target.slug();
        if !targets.contains(&slug) {
            targets.push(slug);
        }
    }
    targets
}

#[must_use]
pub fn render_document(strategies: &[RepairStrategy]) -> String {
    let mut out = String::from("repair_strategies\n");
    let _ = writeln!(out, "  covers \"{}\"", targets(strategies).join(","));
    let _ = writeln!(out, "  strategy_count \"{}\"", strategies.len());
    for strategy in strategies {
        for line in strategy.links_notation().lines() {
            let _ = writeln!(out, "  {line}");
        }
    }
    out
}

#[must_use]
pub fn final_answer(strategies: &[RepairStrategy], document: &str) -> String {
    agentic_message(
        "repair_strategy_final_answer",
        &[
            ("count", &strategies.len().to_string()),
            ("targets", &targets(strategies).join(", ")),
            ("path", REPAIR_STRATEGY_PATH),
            ("document", document.trim_end()),
        ],
    )
}
