//! Issue #559 (R344): the registry-vs-legacy dispatch parity certificate.
//!
//! The per-request selection comparison (R339, [`crate::selection`]) proves the
//! data-driven [`MethodRegistry`] never contradicts the legacy dispatch authority
//! *on the leaves a given prompt happens to produce*. That is necessary but not
//! sufficient after **retiring** the direct solver loop: to keep
//! [`specialized_handler_name`](crate::intent_formalization::specialized_handler_name)
//! as a reliable audit baseline, it must agree with the registry across the
//! *entire route vocabulary the system can ever emit*, not just the routes seen
//! on one request.
//!
//! This module is that corpus-wide proof. It enumerates every route slug any
//! authority knows about — grounded in live data, never a hand-kept list:
//!
//! * every registered method name (a method name is itself a route that resolves
//!   to itself), from [`MethodRegistry::from_dispatch`];
//! * every route→method alias (R336), from [`crate::route_method_alias`];
//! * every classifier route slug, from [`crate::seed::intent_routing`]; and
//! * the `write_program` intent the classifier emits directly.
//!
//! For each route it resolves both authorities and classifies their agreement with
//! the very same rule the per-request comparison uses
//! ([`SelectionAgreement::classify`]). The retire-parity certificate is the single
//! fact [`DispatchParity::is_retire_safe`]: **zero contradictions** across the whole
//! corpus. A contradiction means the registry would pick a different real method
//! than a valid legacy selection, or fail to resolve a route the legacy resolves —
//! either way, retiring the table would change behaviour. While that count is zero
//! the registry remains a behaviour-preserving replacement for the old authority.
//!
//! Like the method registry it audits, this certificate is derived from the live
//! code by construction, so it can never drift; the grounding test
//! (`tests/unit/specification/dispatch_parity.rs`) pins the zero-contradiction
//! invariant. It is pure analysis: it changes neither routing nor any answer (R13).

use crate::event_log::EventLog;
use crate::intent_formalization::specialized_handler_name;
use crate::links_format::format_lino_record;
use crate::method_registry::MethodRegistry;
use crate::selection::SelectionAgreement;

/// How the two authorities resolve one route slug across the whole vocabulary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RouteParity {
    /// The route slug under audit.
    pub route: String,
    /// The method the legacy authority names, kept only when it is a real
    /// registered method (its catch-all returns the slug verbatim for routes with
    /// no handler, which must not masquerade as a selection).
    pub legacy_method: Option<String>,
    /// The method the registry resolves (alias-aware).
    pub registry_method: Option<String>,
    /// How the two authorities agree on this route.
    pub agreement: SelectionAgreement,
}

impl RouteParity {
    /// Resolve both authorities for one route slug and classify their agreement,
    /// by the same rule the per-request selection comparison uses.
    #[must_use]
    fn resolve(route: &str, registry: &MethodRegistry) -> Self {
        let legacy_method = specialized_handler_name(route)
            .filter(|name| registry.methods.iter().any(|method| method.name == *name))
            .map(str::to_owned);
        let registry_method = registry
            .method_for_route(route)
            .map(|method| method.name.clone());
        let agreement =
            SelectionAgreement::classify(legacy_method.as_deref(), registry_method.as_deref());
        Self {
            route: route.to_owned(),
            legacy_method,
            registry_method,
            agreement,
        }
    }

    #[must_use]
    fn to_links_notation(&self) -> String {
        let mut pairs: Vec<(&str, String)> = vec![
            ("record_type", "route_parity".to_owned()),
            ("route", self.route.clone()),
            ("agreement", self.agreement.slug().to_owned()),
        ];
        if let Some(method) = &self.legacy_method {
            pairs.push(("legacy_method", method.clone()));
        }
        if let Some(method) = &self.registry_method {
            pairs.push(("registry_method", method.clone()));
        }
        format_lino_record(&self.route, &pairs)
    }
}

/// The corpus-wide registry-vs-legacy dispatch parity certificate (R344).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DispatchParity {
    /// One entry per distinct route slug in the corpus, in stable sorted order.
    pub routes: Vec<RouteParity>,
}

impl DispatchParity {
    /// Audit the full route vocabulary against the live method registry.
    #[must_use]
    pub fn audit() -> Self {
        let registry = MethodRegistry::from_dispatch();
        let mut corpus: Vec<String> = Vec::new();
        // A method name is itself a route that must resolve to itself.
        for method in &registry.methods {
            corpus.push(method.name.clone());
        }
        // Every route→method alias is a route the registry must resolve.
        for alias in crate::route_method_alias::aliases() {
            corpus.push(alias.route.clone());
        }
        // Every route slug the classifier can emit from the seed routing data.
        for intent in crate::seed::intent_routing().intents {
            if !intent.slug.is_empty() {
                corpus.push(intent.slug);
            }
        }
        // The one intent the classifier emits without a seed route record.
        corpus.push(crate::coding::WRITE_PROGRAM_INTENT.to_owned());

        corpus.sort_unstable();
        corpus.dedup();

        let routes = corpus
            .iter()
            .map(|route| RouteParity::resolve(route, &registry))
            .collect();
        Self { routes }
    }

    /// Total distinct route slugs audited.
    #[must_use]
    pub const fn route_count(&self) -> usize {
        self.routes.len()
    }

    /// Routes on which both authorities name the same registered method.
    #[must_use]
    pub fn agreement_count(&self) -> usize {
        self.count(SelectionAgreement::Agree)
    }

    /// Routes the registry resolves but the legacy authority could not (an alias
    /// rescue — extra coverage, not a regression).
    #[must_use]
    pub fn rescue_count(&self) -> usize {
        self.count(SelectionAgreement::RegistryRescues)
    }

    /// Routes on which the registry would pick a different real method than a valid
    /// legacy selection, or fail to resolve a route the legacy resolves. The
    /// retire-parity invariant requires this to be zero.
    #[must_use]
    pub fn contradiction_count(&self) -> usize {
        self.count(SelectionAgreement::Contradict)
    }

    /// Routes neither authority resolves (an honestly unrouted slug, e.g. a
    /// greeting); shared blockage is agreement, not divergence.
    #[must_use]
    pub fn unresolved_count(&self) -> usize {
        self.count(SelectionAgreement::Unresolved)
    }

    /// The retire-parity certificate: the registry is a behaviour-preserving
    /// drop-in for the legacy dispatch authority exactly when no route contradicts.
    #[must_use]
    pub fn is_retire_safe(&self) -> bool {
        self.contradiction_count() == 0
    }

    fn count(&self, agreement: SelectionAgreement) -> usize {
        self.routes
            .iter()
            .filter(|route| route.agreement == agreement)
            .count()
    }

    /// Serialize the certificate as a `dispatch_parity` header plus one record per
    /// route.
    #[must_use]
    pub fn to_links_notation(&self) -> String {
        let pairs: Vec<(&str, String)> = vec![
            ("record_type", "dispatch_parity".to_owned()),
            ("route_count", self.route_count().to_string()),
            ("agreement_count", self.agreement_count().to_string()),
            ("rescue_count", self.rescue_count().to_string()),
            ("unresolved_count", self.unresolved_count().to_string()),
            (
                "contradiction_count",
                self.contradiction_count().to_string(),
            ),
            ("retire_safe", self.is_retire_safe().to_string()),
        ];
        let mut out = format_lino_record("dispatch_parity", &pairs);
        for route in &self.routes {
            out.push('\n');
            out.push_str(&route.to_links_notation());
        }
        out
    }
}

/// Emit the corpus-wide dispatch-parity certificate as a trace event.
///
/// Pure analysis (R344): it appends one `dispatch_parity` event (the serialized
/// certificate, which enumerates every audited route) and a compact
/// `dispatch_parity:contradictions` count so a regression in retire-parity surfaces
/// as a single auditable number. It changes neither routing nor any answer (R13).
pub fn record_dispatch_parity(log: &mut EventLog) -> DispatchParity {
    let parity = DispatchParity::audit();
    log.append("dispatch_parity", parity.to_links_notation());
    log.append(
        "dispatch_parity:contradictions",
        parity.contradiction_count().to_string(),
    );
    parity
}
