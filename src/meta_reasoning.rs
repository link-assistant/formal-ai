//! Issue #559: white-box recursive reasoning for every meta-core work unit.
//!
//! The recursive downward pass ([`crate::meta_frame::WorkUnit`]) records *what*
//! the meta core did at each node: the span, its depth, whether it was atomic,
//! and its route. This module records *why* — a human-readable line of reasoning
//! attached to every recursive step, so the white box is fully inspectable by
//! users and developers, not just by the people who wrote the predicate.
//!
//! For each unit the reasoning mirrors the recursion in both directions:
//!
//! * the **downward** thought — what was observed and why the unit was decomposed
//!   or judged atomic (and, for an atomic leaf, which method resolves it); and
//! * the **upward** thought — how the unit's answer is composed once its children
//!   (if any) are solved, the construction step of the bidirectional recursion.
//!
//! The reasoning is a parallel tree to the work-unit tree, serialized to Links
//! Notation as `work_unit_reasoning` records and emitted as one trace-only loop
//! event. It changes neither routing nor the answer (R13); it makes the existing
//! recursion legible. Method names are resolved through the same
//! [`MethodRegistry::method_for_route`](crate::method_registry::MethodRegistry::method_for_route)
//! bridge the evidence join uses, so the reasoning and the audit agree.

use crate::event_log::EventLog;
use crate::links_format::format_lino_record;
use crate::meta_frame::{AtomicityReason, WorkUnit};
use crate::method_registry::MethodRegistry;

/// A single recursive step's reasoning, parallel to one [`WorkUnit`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkUnitReasoning {
    /// The work unit this reasoning explains.
    pub unit_id: String,
    /// Recursion depth (0 at the root), so the trace reads as a tree.
    pub depth: u8,
    /// What the meta core observed about the span at this step.
    pub observation: String,
    /// The decision taken, as a stable slug (`decompose`, `direct_method`,
    /// `single_need`, `depth_bound`).
    pub decision: String,
    /// Why that decision follows from the observation (the downward thought).
    pub downward_rationale: String,
    /// The method that resolves this unit, when it is an atomic leaf with a
    /// recognized, registered route.
    pub method: Option<String>,
    /// How this unit's answer is constructed once its children are solved (the
    /// upward thought).
    pub upward_rationale: String,
    /// Reasoning for each child, in source order (empty for a leaf).
    pub children: Vec<Self>,
}

impl WorkUnitReasoning {
    /// Reason about a work-unit tree, resolving leaf methods through `registry`.
    #[must_use]
    pub fn for_unit(unit: &WorkUnit, registry: &MethodRegistry) -> Self {
        let children = unit
            .children
            .iter()
            .map(|child| Self::for_unit(child, registry))
            .collect::<Vec<_>>();
        let method = unit
            .route
            .as_ref()
            .filter(|_| unit.atomic)
            .and_then(|route| registry.method_for_route(route))
            .map(|method| method.name.clone());
        Self {
            unit_id: unit.unit_id.clone(),
            depth: unit.depth,
            observation: observe(unit),
            decision: decision_slug(unit.reason).to_owned(),
            downward_rationale: downward_rationale(unit, method.as_deref()),
            method,
            upward_rationale: upward_rationale(unit),
            children,
        }
    }

    /// Total number of reasoning steps (this node plus its descendants).
    #[must_use]
    pub fn step_count(&self) -> usize {
        1 + self.children.iter().map(Self::step_count).sum::<usize>()
    }

    /// Render this reasoning and its descendants as Links Notation records.
    #[must_use]
    pub fn to_links_notation(&self) -> String {
        let mut pairs: Vec<(&str, String)> = vec![
            ("record_type", "work_unit_reasoning".to_owned()),
            ("unit_id", self.unit_id.clone()),
            ("depth", self.depth.to_string()),
            ("observation", self.observation.clone()),
            ("decision", self.decision.clone()),
            ("downward_rationale", self.downward_rationale.clone()),
            ("upward_rationale", self.upward_rationale.clone()),
        ];
        if let Some(method) = &self.method {
            pairs.push(("method", method.clone()));
        }
        for child in &self.children {
            pairs.push(("child", child.unit_id.clone()));
        }
        let mut out = format_lino_record(&self.unit_id, &pairs);
        for child in &self.children {
            out.push('\n');
            out.push_str(&child.to_links_notation());
        }
        out
    }
}

/// Map a unit's atomicity reason to the reasoning decision slug.
const fn decision_slug(reason: AtomicityReason) -> &'static str {
    match reason {
        AtomicityReason::NotAtomic => "decompose",
        AtomicityReason::DirectMethod => "direct_method",
        AtomicityReason::SingleNeed => "single_need",
        AtomicityReason::DepthBound => "depth_bound",
    }
}

/// What the meta core observed about this span before deciding.
fn observe(unit: &WorkUnit) -> String {
    let route = unit.route.as_deref().map_or_else(
        || "no route was recognized".to_owned(),
        |route| format!("the route `{route}` was recognized"),
    );
    format!(
        "At depth {}, observed a span of {} characters; {route}.",
        unit.depth,
        unit.source_span.chars().count()
    )
}

/// The downward thought: why the unit was decomposed or judged atomic.
fn downward_rationale(unit: &WorkUnit, method: Option<&str>) -> String {
    match unit.reason {
        AtomicityReason::NotAtomic => format!(
            "The span carries more than one need, so it is not solvable by a single \
             method: decompose it into {} sub-units and reason about each recursively.",
            unit.children.len()
        ),
        AtomicityReason::DirectMethod => method.map_or_else(
            || {
                "The span is directly solvable by a recognized route, but no registered \
                 method resolves it yet — record the route so the gap is visible rather than \
                 decomposing further."
                    .to_owned()
            },
            |method| {
                format!(
                    "The span is directly solvable: its route resolves to the registered \
                     method `{method}`. Invoke that method; no further decomposition is needed."
                )
            },
        ),
        AtomicityReason::SingleNeed => "The span cannot be split further and matches no \
             known route, so it is an irreducible single need. Record it for an honest \
             blocked status instead of forcing an answer."
            .to_owned(),
        AtomicityReason::DepthBound => "The recursion depth bound was reached, so stop \
             decomposing to guarantee termination and treat this unit as a leaf."
            .to_owned(),
    }
}

/// The upward thought: how this unit's answer is constructed from its children.
fn upward_rationale(unit: &WorkUnit) -> String {
    if unit.atomic {
        match unit.reason {
            AtomicityReason::DirectMethod => "Return the method's result directly; there are \
                 no children to compose."
                .to_owned(),
            _ => "Return a blocked marker upward: this leaf has no resolving method, so there \
                 is nothing to compose."
                .to_owned(),
        }
    } else {
        format!(
            "Once all {} children are solved, compose their results in source order into this \
             unit's answer; the answer is complete iff every child's is.",
            unit.children.len()
        )
    }
}

/// Build the recursive reasoning for the work-unit tree and emit it as one
/// trace-only loop event plus a compact step count.
///
/// Trace-only (R337): it appends one `work_unit_reasoning` event (the serialized
/// reasoning tree, which enumerates every step) and a compact
/// `work_unit_reasoning:steps`, so the white-box reasoning is observable in the
/// event log without changing routing or the answer (R13).
pub(crate) fn record_work_unit_reasoning(
    log: &mut EventLog,
    root: &WorkUnit,
    registry: &MethodRegistry,
) -> WorkUnitReasoning {
    let reasoning = WorkUnitReasoning::for_unit(root, registry);
    log.append("work_unit_reasoning", reasoning.to_links_notation());
    log.append(
        "work_unit_reasoning:steps",
        reasoning.step_count().to_string(),
    );
    reasoning
}
