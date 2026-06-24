//! Issue #559 (R339): registry-driven method selection, compared to the legacy
//! route mapper.
//!
//! The meta core resolves every atomic work-unit leaf with a *method* (one of the
//! solver's specialized handlers). Two authorities can name that method:
//!
//! * the **legacy** authority
//!   ([`specialized_handler_name`](crate::intent_formalization::specialized_handler_name)) —
//!   the hardcoded `match` the solver has always used to map a route slug to a
//!   handler name; and
//! * the **registry** authority
//!   ([`MethodRegistry::method_for_route`](crate::method_registry::MethodRegistry::method_for_route)) —
//!   the data-driven resolver that consults the route→method alias link data
//!   (R336), so it can name a handler for routes whose slug differs from the
//!   method name (e.g. `write_program` → `write_script`).
//!
//! The live solver now drives selection from the registry. The old route mapper is
//! retained as a parity baseline: wherever it names a real handler, the registry
//! must name the same one. This module records that comparison per leaf as Links
//! Notation, classifying each leaf as:
//!
//! * `agree` — both authorities name the same registered method;
//! * `registry_rescues` — the legacy names no real method (its catch-all returns a
//!   slug with no handler), but the registry resolves one through an alias;
//! * `contradict` — both name a real method, but a *different* one (a regression
//!   the zero-contradiction tests forbid); or
//! * `unresolved` — neither authority resolves a method (an honestly blocked leaf).
//!
//! Selection is governed by [`SelectionMode`]: the default
//! [`SelectionMode::Legacy`] records nothing for trace compatibility.
//! [`SelectionMode::Registry`] records the registry-driven choice per leaf, and
//! [`SelectionMode::Compare`] records the full per-leaf comparison plus the
//! divergence and contradiction counts.

use crate::event_log::EventLog;
use crate::intent_formalization::specialized_handler_name;
use crate::links_format::format_lino_record;
use crate::meta_frame::WorkUnit;
use crate::method_registry::MethodRegistry;

/// Which selection artifact the meta core records. The live solver always uses
/// registry dispatch; this knob only changes trace verbosity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SelectionMode {
    /// Record nothing for compatibility with the pre-comparison trace.
    #[default]
    Legacy,
    /// Record the registry-driven method choice per leaf.
    Registry,
    /// Record the full legacy-vs-registry comparison and divergence counts.
    Compare,
}

impl SelectionMode {
    /// Whether a selection artifact should be emitted at all.
    #[must_use]
    pub const fn emits_artifact(self) -> bool {
        matches!(self, Self::Registry | Self::Compare)
    }

    /// Whether the legacy authority and the per-leaf agreement are recorded
    /// alongside the registry choice (only in [`Self::Compare`]).
    #[must_use]
    pub const fn records_comparison(self) -> bool {
        matches!(self, Self::Compare)
    }

    /// The stable slug used in traces and config parsing.
    #[must_use]
    pub const fn slug(self) -> &'static str {
        match self {
            Self::Legacy => "legacy",
            Self::Registry => "registry",
            Self::Compare => "compare",
        }
    }

    /// Parse a slug back into a mode, accepting the canonical spellings.
    #[must_use]
    pub fn from_slug(slug: &str) -> Option<Self> {
        match slug.trim().to_ascii_lowercase().as_str() {
            "legacy" => Some(Self::Legacy),
            "registry" => Some(Self::Registry),
            "compare" => Some(Self::Compare),
            _ => None,
        }
    }
}

/// How the legacy and registry authorities agree (or disagree) on one leaf.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectionAgreement {
    /// Both authorities name the same registered method.
    Agree,
    /// The legacy names no real method; the registry resolves one via an alias.
    RegistryRescues,
    /// Both name a real method, but a different one (a forbidden regression).
    Contradict,
    /// Neither authority resolves a method (an honestly blocked leaf).
    Unresolved,
}

impl SelectionAgreement {
    /// Stable lowercase slug used in the Links Notation trace.
    #[must_use]
    pub const fn slug(self) -> &'static str {
        match self {
            Self::Agree => "agree",
            Self::RegistryRescues => "registry_rescues",
            Self::Contradict => "contradict",
            Self::Unresolved => "unresolved",
        }
    }

    /// Classify a leaf from the two authorities' resolved method names.
    ///
    /// Shared with the corpus-wide dispatch-parity audit (R344,
    /// [`crate::dispatch_parity`]) so the per-request comparison and the
    /// retire-parity certificate classify agreement by exactly the same rule.
    #[must_use]
    pub(crate) fn classify(legacy: Option<&str>, registry: Option<&str>) -> Self {
        match (legacy, registry) {
            (None, None) => Self::Unresolved,
            (None, Some(_)) => Self::RegistryRescues,
            (Some(_), None) => Self::Contradict,
            (Some(legacy), Some(registry)) => {
                if legacy == registry {
                    Self::Agree
                } else {
                    Self::Contradict
                }
            }
        }
    }
}

/// The method each authority selects for one atomic work-unit leaf.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LeafSelection {
    /// The leaf whose method is being selected.
    pub unit_id: String,
    /// Recursion depth of the leaf (0 at the root).
    pub depth: u8,
    /// The route slug the leaf carries, when one was recognized.
    pub route: Option<String>,
    /// The method the legacy authority names, kept only when it is a real
    /// registered method (its catch-all "name" with no handler is dropped).
    pub legacy_method: Option<String>,
    /// The method the registry authority resolves (alias-aware).
    pub registry_method: Option<String>,
    /// How the two authorities agree on this leaf.
    pub agreement: SelectionAgreement,
}

impl LeafSelection {
    /// Resolve both authorities for one leaf and classify their agreement.
    #[must_use]
    fn resolve(unit: &WorkUnit, registry: &MethodRegistry) -> Self {
        let route = unit.route.clone();
        // The legacy authority only counts when it names a method that actually
        // exists in the catalogue; its catch-all returns the slug verbatim even
        // for routes with no handler, which must not masquerade as a selection.
        let legacy_method = route
            .as_deref()
            .and_then(specialized_handler_name)
            .filter(|name| registry.methods.iter().any(|method| method.name == *name))
            .map(str::to_owned);
        let registry_method = route
            .as_deref()
            .and_then(|route| registry.method_for_route(route))
            .map(|method| method.name.clone());
        let agreement =
            SelectionAgreement::classify(legacy_method.as_deref(), registry_method.as_deref());
        Self {
            unit_id: unit.unit_id.clone(),
            depth: unit.depth,
            route,
            legacy_method,
            registry_method,
            agreement,
        }
    }

    /// Render one leaf's selection as a `leaf_selection` record. In
    /// [`SelectionMode::Registry`] only the registry choice is shown (the registry
    /// drives); in [`SelectionMode::Compare`] the legacy choice and the agreement
    /// are shown alongside it.
    #[must_use]
    fn to_links_notation(&self, mode: SelectionMode) -> String {
        let mut pairs: Vec<(&str, String)> = vec![
            ("record_type", "leaf_selection".to_owned()),
            ("unit_id", self.unit_id.clone()),
            ("depth", self.depth.to_string()),
        ];
        if let Some(route) = &self.route {
            pairs.push(("route", route.clone()));
        }
        if let Some(method) = &self.registry_method {
            pairs.push(("registry_method", method.clone()));
        }
        if mode.records_comparison() {
            if let Some(method) = &self.legacy_method {
                pairs.push(("legacy_method", method.clone()));
            }
            pairs.push(("agreement", self.agreement.slug().to_owned()));
        }
        format_lino_record(&self.unit_id, &pairs)
    }
}

/// The per-leaf method-selection comparison across a work-unit tree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectionComparison {
    /// The root unit the comparison was built for.
    pub root_id: String,
    /// One entry per atomic leaf, in source order.
    pub leaves: Vec<LeafSelection>,
}

impl SelectionComparison {
    /// Build the comparison for a work-unit tree, resolving both authorities for
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

    /// Number of atomic leaves compared.
    #[must_use]
    pub const fn leaf_count(&self) -> usize {
        self.leaves.len()
    }

    /// Leaves on which both authorities name the same registered method.
    #[must_use]
    pub fn agreement_count(&self) -> usize {
        self.count(SelectionAgreement::Agree)
    }

    /// Leaves the registry resolves but the legacy could not.
    #[must_use]
    pub fn rescue_count(&self) -> usize {
        self.count(SelectionAgreement::RegistryRescues)
    }

    /// Leaves on which the two authorities name a different real method — a
    /// regression the zero-contradiction invariant forbids.
    #[must_use]
    pub fn contradiction_count(&self) -> usize {
        self.count(SelectionAgreement::Contradict)
    }

    /// Leaves where the two authorities differ (rescues plus contradictions); the
    /// shared `unresolved` (both blocked) is agreement, not divergence.
    #[must_use]
    pub fn divergence_count(&self) -> usize {
        self.rescue_count() + self.contradiction_count()
    }

    fn count(&self, agreement: SelectionAgreement) -> usize {
        self.leaves
            .iter()
            .filter(|leaf| leaf.agreement == agreement)
            .count()
    }

    /// Render the comparison as a `selection` header plus one record per leaf.
    #[must_use]
    pub fn to_links_notation(&self, mode: SelectionMode) -> String {
        let mut pairs: Vec<(&str, String)> = vec![
            ("record_type", "selection".to_owned()),
            ("mode", mode.slug().to_owned()),
            ("root_id", self.root_id.clone()),
            ("leaf_count", self.leaf_count().to_string()),
        ];
        if mode.records_comparison() {
            pairs.push(("agreement_count", self.agreement_count().to_string()));
            pairs.push(("rescue_count", self.rescue_count().to_string()));
            pairs.push((
                "contradiction_count",
                self.contradiction_count().to_string(),
            ));
        }
        let mut out = format_lino_record("selection", &pairs);
        for leaf in &self.leaves {
            out.push('\n');
            out.push_str(&leaf.to_links_notation(mode));
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

/// Emit the method-selection comparison as an optional trace event, gated by
/// `mode`.
///
/// Returns `None` when `mode` is [`SelectionMode::Legacy`], so the default leaves
/// the trace exactly as it was before this comparison existed. When emitted, it
/// appends one `selection` event (the serialized header plus every leaf) and, in
/// [`SelectionMode::Compare`], a compact `selection:contradictions` count so a
/// regression surfaces as a single auditable number.
pub(crate) fn record_selection(
    log: &mut EventLog,
    root: &WorkUnit,
    registry: &MethodRegistry,
    mode: SelectionMode,
) -> Option<SelectionComparison> {
    if !mode.emits_artifact() {
        return None;
    }
    let comparison = SelectionComparison::for_unit(root, registry);
    log.append("selection", comparison.to_links_notation(mode));
    if mode.records_comparison() {
        log.append(
            "selection:contradictions",
            comparison.contradiction_count().to_string(),
        );
    }
    Some(comparison)
}
