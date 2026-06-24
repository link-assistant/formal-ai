//! Issue #559: the upward construction pass and the `recursion_mode` knob.
//!
//! The meta core's downward pass ([`crate::meta_frame::WorkUnit`]) decomposes a
//! request into a bounded work-unit tree, and the downward reasoning
//! ([`crate::meta_reasoning`], R337) explains *why* each unit was split or judged
//! atomic. Decomposition is only half of a recursive algorithm: the other half is
//! **construction** — composing the children's results back up into the parent's
//! answer, leaf to root. This module records that upward pass explicitly.
//!
//! The construction is a post-order (bottom-up) walk of the same tree: each leaf
//! is constructed directly from the method that resolves it (the base case), and
//! each parent is constructed by composing its already-constructed children in
//! source order (the recursive case), terminating at the root. Serialized to
//! Links Notation as `construction_step` records under one `upward_construction`
//! header, it makes the compositional direction as inspectable as the
//! decompositional one.
//!
//! Which directions the meta core emits is governed by [`RecursionMode`]:
//!
//! * [`RecursionMode::Down`] (the default) — emit the downward reasoning only, so
//!   the trace is exactly what shipped before this knob existed (behavior-
//!   preserving, R13);
//! * [`RecursionMode::Up`] — emit the upward construction only;
//! * [`RecursionMode::Both`] — emit both directions.
//!
//! The work-unit decomposition events (`work_unit:enter` / `work_unit:exit`) are
//! structural and always emitted; the knob gates only the directional *reasoning*
//! artifacts, none of which change routing or the answer.

use crate::event_log::EventLog;
use crate::links_format::format_lino_record;
use crate::meta_frame::WorkUnit;
use crate::method_registry::MethodRegistry;

/// Which direction(s) of recursive reasoning the meta core emits.
///
/// `Down` is the default and reproduces the pre-knob trace exactly, so enabling
/// the upward pass is always an explicit opt-in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RecursionMode {
    /// Emit the downward decomposition reasoning only (default, behavior-preserving).
    #[default]
    Down,
    /// Emit the upward construction reasoning only.
    Up,
    /// Emit both the downward and the upward reasoning.
    Both,
}

impl RecursionMode {
    /// Whether the downward decomposition reasoning (R337) should be emitted.
    #[must_use]
    pub const fn emits_downward(self) -> bool {
        matches!(self, Self::Down | Self::Both)
    }

    /// Whether the upward construction pass should be emitted.
    #[must_use]
    pub const fn emits_upward(self) -> bool {
        matches!(self, Self::Up | Self::Both)
    }

    /// The stable slug used in traces and config parsing.
    #[must_use]
    pub const fn slug(self) -> &'static str {
        match self {
            Self::Down => "down",
            Self::Up => "up",
            Self::Both => "both",
        }
    }

    /// Parse a slug back into a mode, accepting the canonical spellings.
    #[must_use]
    pub fn from_slug(slug: &str) -> Option<Self> {
        match slug.trim().to_ascii_lowercase().as_str() {
            "down" => Some(Self::Down),
            "up" => Some(Self::Up),
            "both" => Some(Self::Both),
            _ => None,
        }
    }
}

/// How one work unit's answer is constructed during the upward pass.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConstructionStep {
    /// The unit whose answer this step constructs.
    pub unit_id: String,
    /// Recursion depth of the unit (0 at the root).
    pub depth: u8,
    /// Post-order position (1-based): children are always constructed before
    /// their parent, and the root is last.
    pub order: usize,
    /// `leaf_method` (base case) or `compose` (recursive case).
    pub kind: String,
    /// The method whose result a leaf is constructed from, when one resolves.
    pub method: Option<String>,
    /// The child unit ids composed into a parent's answer, in source order.
    pub inputs: Vec<String>,
    /// Why the answer is constructed this way (human-readable).
    pub rationale: String,
}

/// The upward construction pass: a post-order list of construction steps.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpwardConstruction {
    /// The root unit the construction terminates at.
    pub root_id: String,
    /// One step per unit, in post-order (leaves first, root last).
    pub steps: Vec<ConstructionStep>,
}

impl UpwardConstruction {
    /// Build the upward construction for a work-unit tree, resolving leaf methods
    /// through the same `method_for_route` bridge the evidence join uses.
    #[must_use]
    pub fn for_unit(root: &WorkUnit, registry: &MethodRegistry) -> Self {
        let mut steps = Vec::new();
        visit_post_order(root, registry, &mut steps);
        Self {
            root_id: root.unit_id.clone(),
            steps,
        }
    }

    /// Number of construction steps (one per unit).
    #[must_use]
    pub const fn step_count(&self) -> usize {
        self.steps.len()
    }

    /// Render the construction as a header plus one record per step.
    #[must_use]
    pub fn to_links_notation(&self) -> String {
        let header = format_lino_record(
            "upward_construction",
            &[
                ("record_type", "upward_construction".to_owned()),
                ("root_id", self.root_id.clone()),
                ("step_count", self.steps.len().to_string()),
            ],
        );
        let mut out = header;
        for step in &self.steps {
            out.push('\n');
            out.push_str(&step.to_links_notation());
        }
        out
    }
}

impl ConstructionStep {
    /// Render one construction step as a `construction_step` record.
    #[must_use]
    fn to_links_notation(&self) -> String {
        let mut pairs: Vec<(&str, String)> = vec![
            ("record_type", "construction_step".to_owned()),
            ("unit_id", self.unit_id.clone()),
            ("depth", self.depth.to_string()),
            ("order", self.order.to_string()),
            ("kind", self.kind.clone()),
            ("rationale", self.rationale.clone()),
        ];
        if let Some(method) = &self.method {
            pairs.push(("method", method.clone()));
        }
        for input in &self.inputs {
            pairs.push(("input", input.clone()));
        }
        format_lino_record(&self.unit_id, &pairs)
    }
}

/// Append each unit's construction step in post-order (children before parent).
fn visit_post_order(unit: &WorkUnit, registry: &MethodRegistry, steps: &mut Vec<ConstructionStep>) {
    for child in &unit.children {
        visit_post_order(child, registry, steps);
    }
    let order = steps.len() + 1;
    let step = if unit.children.is_empty() {
        let method = unit
            .route
            .as_ref()
            .and_then(|route| registry.method_for_route(route))
            .map(|method| method.name.clone());
        let rationale = method.as_ref().map_or_else(
            || {
                "Base case: no method resolves this leaf, so its answer is a blocked \
                 marker — nothing is constructed."
                    .to_owned()
            },
            |method| {
                format!(
                    "Base case: construct this leaf's answer directly from the result of \
                     method `{method}`."
                )
            },
        );
        ConstructionStep {
            unit_id: unit.unit_id.clone(),
            depth: unit.depth,
            order,
            kind: "leaf_method".to_owned(),
            method,
            inputs: Vec::new(),
            rationale,
        }
    } else {
        let inputs = unit
            .children
            .iter()
            .map(|child| child.unit_id.clone())
            .collect::<Vec<_>>();
        let rationale = format!(
            "Recursive case: compose the {} already-constructed children in source order \
             into this unit's answer.",
            inputs.len()
        );
        ConstructionStep {
            unit_id: unit.unit_id.clone(),
            depth: unit.depth,
            order,
            kind: "compose".to_owned(),
            method: None,
            inputs,
            rationale,
        }
    };
    steps.push(step);
}

/// Emit the upward construction pass as a trace-only event, gated by `mode`.
///
/// Returns `None` when `mode` does not request the upward direction, so the
/// default ([`RecursionMode::Down`]) leaves the trace exactly as it was before
/// this pass existed (R13). When emitted, it appends one `upward_construction`
/// event (the serialized header plus every step) and a compact
/// `upward_construction:steps`.
pub(crate) fn record_upward_construction(
    log: &mut EventLog,
    root: &WorkUnit,
    registry: &MethodRegistry,
    mode: RecursionMode,
) -> Option<UpwardConstruction> {
    if !mode.emits_upward() {
        return None;
    }
    let construction = UpwardConstruction::for_unit(root, registry);
    log.append("upward_construction", construction.to_links_notation());
    log.append(
        "upward_construction:steps",
        construction.step_count().to_string(),
    );
    Some(construction)
}
