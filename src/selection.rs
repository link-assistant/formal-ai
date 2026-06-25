//! Issue #559 (R339): registry-driven method selection trace.
//!
//! The meta core resolves every atomic work-unit leaf with a *method* through a
//! single, data-driven authority:
//! [`MethodRegistry::method_for_route`](crate::method_registry::MethodRegistry::method_for_route)
//! (R331/R336). The route→method alias link data (R336) lets a leaf whose route
//! slug differs from the method name still resolve (e.g. `write_program` →
//! `write_script`). There is no second authority: the legacy hardcoded route
//! mapper was removed once the registry became the sole dispatch authority, so
//! this module records *what the registry selects*, not a comparison.
//!
//! For every atomic leaf it records, as Links Notation, the method the registry
//! resolves (or marks the leaf `unresolved` when no method serves it). The
//! selection step of the algorithm is thus an inspectable white-box artifact:
//! given a work-unit tree, the trace shows exactly which method each leaf was
//! dispatched to.
//!
//! Recording is governed by [`SelectionMode`]: the default [`SelectionMode::Off`]
//! records nothing, leaving the trace exactly as it was before this artifact
//! existed; [`SelectionMode::Record`] emits one `selection` event (a header plus
//! one record per leaf).

use crate::event_log::EventLog;
use crate::links_format::format_lino_record;
use crate::meta_frame::WorkUnit;
use crate::method_registry::MethodRegistry;

/// Whether the meta core records the per-leaf method-selection artifact. The live
/// solver always dispatches through the registry; this knob only changes trace
/// verbosity, never routing or any answer (R13).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SelectionMode {
    /// Record nothing, reproducing the pre-artifact trace exactly.
    #[default]
    Off,
    /// Record the registry-resolved method for every atomic leaf.
    Record,
}

impl SelectionMode {
    /// Whether a selection artifact should be emitted at all.
    #[must_use]
    pub const fn emits_artifact(self) -> bool {
        matches!(self, Self::Record)
    }

    /// The stable slug used in traces and config parsing.
    #[must_use]
    pub const fn slug(self) -> &'static str {
        match self {
            Self::Off => "off",
            Self::Record => "record",
        }
    }

    /// Parse a slug back into a mode, accepting the canonical spellings.
    #[must_use]
    pub fn from_slug(slug: &str) -> Option<Self> {
        match slug.trim().to_ascii_lowercase().as_str() {
            "off" => Some(Self::Off),
            "record" => Some(Self::Record),
            _ => None,
        }
    }
}

/// The method the registry selects for one atomic work-unit leaf.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LeafSelection {
    /// The leaf whose method is being selected.
    pub unit_id: String,
    /// Recursion depth of the leaf (0 at the root).
    pub depth: u8,
    /// The route slug the leaf carries, when one was recognized.
    pub route: Option<String>,
    /// The method the registry resolves for the leaf's route (alias-aware), or
    /// `None` when no method serves it (an honestly blocked leaf).
    pub method: Option<String>,
}

impl LeafSelection {
    /// Resolve the registry method for one leaf.
    #[must_use]
    fn resolve(unit: &WorkUnit, registry: &MethodRegistry) -> Self {
        let route = unit.route.clone();
        let method = route
            .as_deref()
            .and_then(|route| registry.method_for_route(route))
            .map(|method| method.name.clone());
        Self {
            unit_id: unit.unit_id.clone(),
            depth: unit.depth,
            route,
            method,
        }
    }

    /// Whether the registry resolved a method for this leaf.
    #[must_use]
    pub const fn is_resolved(&self) -> bool {
        self.method.is_some()
    }

    /// Render one leaf's selection as a `leaf_selection` record. A resolved leaf
    /// names its `method`; an unresolved leaf is marked `unresolved` so a blocked
    /// leaf is recorded rather than dropped.
    #[must_use]
    fn to_links_notation(&self) -> String {
        let mut pairs: Vec<(&str, String)> = vec![
            ("record_type", "leaf_selection".to_owned()),
            ("unit_id", self.unit_id.clone()),
            ("depth", self.depth.to_string()),
        ];
        if let Some(route) = &self.route {
            pairs.push(("route", route.clone()));
        }
        match &self.method {
            Some(method) => pairs.push(("method", method.clone())),
            None => pairs.push(("method", "unresolved".to_owned())),
        }
        format_lino_record(&self.unit_id, &pairs)
    }
}

/// The per-leaf method selection across a work-unit tree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MethodSelection {
    /// The root unit the selection was built for.
    pub root_id: String,
    /// One entry per atomic leaf, in source order.
    pub leaves: Vec<LeafSelection>,
}

impl MethodSelection {
    /// Build the selection for a work-unit tree, resolving the registry method for
    /// every atomic leaf.
    #[must_use]
    pub fn for_unit(root: &WorkUnit, registry: &MethodRegistry) -> Self {
        let mut leaves = Vec::new();
        collect_leaf_selections(root, registry, &mut leaves);
        Self {
            root_id: root.unit_id.clone(),
            leaves,
        }
    }

    /// Number of atomic leaves selected.
    #[must_use]
    pub const fn leaf_count(&self) -> usize {
        self.leaves.len()
    }

    /// Leaves for which the registry resolved a method.
    #[must_use]
    pub fn resolved_count(&self) -> usize {
        self.leaves.iter().filter(|leaf| leaf.is_resolved()).count()
    }

    /// Leaves for which no method was resolved (honestly blocked leaves).
    #[must_use]
    pub fn unresolved_count(&self) -> usize {
        self.leaf_count() - self.resolved_count()
    }

    /// Render the selection as a `selection` header plus one record per leaf.
    #[must_use]
    pub fn to_links_notation(&self) -> String {
        let pairs: Vec<(&str, String)> = vec![
            ("record_type", "selection".to_owned()),
            ("root_id", self.root_id.clone()),
            ("leaf_count", self.leaf_count().to_string()),
            ("resolved_count", self.resolved_count().to_string()),
            ("unresolved_count", self.unresolved_count().to_string()),
        ];
        let mut out = format_lino_record("selection", &pairs);
        for leaf in &self.leaves {
            out.push('\n');
            out.push_str(&leaf.to_links_notation());
        }
        out
    }
}

/// Collect a selection entry for every atomic leaf, in source order.
fn collect_leaf_selections(
    unit: &WorkUnit,
    registry: &MethodRegistry,
    leaves: &mut Vec<LeafSelection>,
) {
    if unit.children.is_empty() {
        leaves.push(LeafSelection::resolve(unit, registry));
        return;
    }
    for child in &unit.children {
        collect_leaf_selections(child, registry, leaves);
    }
}

/// Emit the per-leaf method selection as an optional trace event, gated by `mode`.
///
/// Returns `None` when `mode` is [`SelectionMode::Off`], so the default leaves the
/// trace exactly as it was before this artifact existed. When emitted, it appends
/// one `selection` event (the serialized header plus every leaf). It is pure
/// analysis: it changes neither routing nor any answer (R13).
pub(crate) fn record_selection(
    log: &mut EventLog,
    root: &WorkUnit,
    registry: &MethodRegistry,
    mode: SelectionMode,
) -> Option<MethodSelection> {
    if !mode.emits_artifact() {
        return None;
    }
    let selection = MethodSelection::for_unit(root, registry);
    log.append("selection", selection.to_links_notation());
    Some(selection)
}
