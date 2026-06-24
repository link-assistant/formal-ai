//! Issue #559 Phase 3: the method registry as first-class link data (R331).
//!
//! The meta algorithm resolves every atomic work-unit leaf with a *method* — one
//! of the solver's specialized handlers. Today that catalogue lives only as Rust:
//! the ordered [`SPECIALIZED_HANDLERS`](crate::solver_dispatch::SPECIALIZED_HANDLERS)
//! table plus the five contextual overrides in
//! [`try_contextual_override`](crate::solver_dispatch::try_contextual_override).
//! To let the meta algorithm later *reason about and modify* its own catalogue
//! (the core ask of issue #559), the catalogue must also exist as data the engine
//! can read: a registry of method records serialized to Links Notation.
//!
//! This module derives that registry *from the live code* — it reads the two
//! source-of-truth constants directly, so the data can never drift from the
//! handlers that actually run. A grounding test
//! (`tests/unit/specification/method_registry.rs`) pins the derived names against
//! the source, the same discipline the meta-recipe files use.
//!
//! Phase 3 is trace-only: the registry is recorded as a `method_registry` loop
//! event so it is observable in the event log, but selection still flows through
//! the existing dispatch (R13). Later phases let the registry *drive* selection.

use crate::event_log::EventLog;
use crate::links_format::format_lino_record;
use crate::solver_dispatch::{CONTEXTUAL_HANDLER_NAMES, SPECIALIZED_HANDLERS};

/// How a method is reached during dispatch.
///
/// The two surfaces are distinct on purpose: a contextual override runs *before*
/// the ordered table when the conversation context calls for it (for example,
/// `numeric_list` as a history-aware override), while the specialized surface is
/// the ordered first-match table. Several names appear on both surfaces, so the
/// registry keeps them as separate records rather than collapsing them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MethodSurface {
    /// A handler in the ordered [`SPECIALIZED_HANDLERS`] table.
    Specialized,
    /// A context-dependent override evaluated by `try_contextual_override`.
    Contextual,
}

impl MethodSurface {
    /// Stable lowercase slug used in the Links Notation trace.
    #[must_use]
    pub const fn slug(self) -> &'static str {
        match self {
            Self::Specialized => "specialized",
            Self::Contextual => "contextual",
        }
    }
}

/// One resolvable method (a named handler) in the catalogue.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Method {
    /// The handler name, exactly as it appears in the dispatch code.
    pub name: String,
    /// Dispatch precedence within its surface (0-based, lower wins first).
    pub order: usize,
    /// Which dispatch surface reaches this method.
    pub surface: MethodSurface,
}

impl Method {
    #[must_use]
    fn to_links_notation(&self) -> String {
        let pairs: Vec<(&str, String)> = vec![
            ("record_type", "method".to_owned()),
            ("name", self.name.clone()),
            ("order", self.order.to_string()),
            ("surface", self.surface.slug().to_owned()),
        ];
        format_lino_record(&self.name, &pairs)
    }
}

/// The full catalogue of methods the solver can route an atomic leaf to.
///
/// Built from the live dispatch constants via [`MethodRegistry::from_dispatch`],
/// so the data is grounded in the code by construction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MethodRegistry {
    /// Every method, specialized table first (in precedence order), then the
    /// contextual overrides.
    pub methods: Vec<Method>,
}

impl MethodRegistry {
    /// Derive the registry from the live dispatch constants.
    #[must_use]
    pub fn from_dispatch() -> Self {
        let mut methods =
            Vec::with_capacity(SPECIALIZED_HANDLERS.len() + CONTEXTUAL_HANDLER_NAMES.len());
        for (order, (name, _handler)) in SPECIALIZED_HANDLERS.iter().enumerate() {
            methods.push(Method {
                name: (*name).to_owned(),
                order,
                surface: MethodSurface::Specialized,
            });
        }
        for (order, name) in CONTEXTUAL_HANDLER_NAMES.iter().enumerate() {
            methods.push(Method {
                name: (*name).to_owned(),
                order,
                surface: MethodSurface::Contextual,
            });
        }
        Self { methods }
    }

    /// Total number of method records.
    #[must_use]
    pub const fn method_count(&self) -> usize {
        self.methods.len()
    }

    /// Resolve a route slug to the catalogued method that serves it.
    ///
    /// A route slug usually names a method directly (the meta-language intent
    /// vocabulary and the dispatch vocabulary coincide). When it does not — for
    /// example the `write_program` intent served by the `write_script` method —
    /// the resolver consults the route→method alias link data
    /// ([`crate::route_method_alias`]) so the meta core can still name the method
    /// a leaf resolves to. This is the single route→method resolution authority
    /// the evidence join (R334) uses; keeping it here means selection and audit
    /// share one bridge between the two vocabularies.
    #[must_use]
    pub fn method_for_route(&self, route: &str) -> Option<&Method> {
        if let Some(method) = self.methods.iter().find(|method| method.name == route) {
            return Some(method);
        }
        let aliased = crate::route_method_alias::method_for_alias(route)?;
        self.methods.iter().find(|method| method.name == aliased)
    }

    /// Number of methods on a given dispatch surface.
    #[must_use]
    pub fn count_on(&self, surface: MethodSurface) -> usize {
        self.methods.iter().filter(|m| m.surface == surface).count()
    }

    /// Serialize the registry and every method to Links Notation (R311).
    #[must_use]
    pub fn to_links_notation(&self) -> String {
        let mut pairs: Vec<(&str, String)> = vec![
            ("record_type", "method_registry".to_owned()),
            ("method_count", self.method_count().to_string()),
            (
                "specialized_count",
                self.count_on(MethodSurface::Specialized).to_string(),
            ),
            (
                "contextual_count",
                self.count_on(MethodSurface::Contextual).to_string(),
            ),
        ];
        for method in &self.methods {
            pairs.push(("method", method.name.clone()));
        }
        let mut out = format_lino_record("method_registry", &pairs);
        for method in &self.methods {
            out.push('\n');
            out.push_str(&method.to_links_notation());
        }
        out
    }
}

/// Build the method registry and emit it as a loop event plus its Links Notation
/// trace.
///
/// Trace-only (Phase 3): it appends one `method_registry` event (the serialized
/// catalogue, which itself enumerates every method) and a compact
/// `method_registry:count`, so the catalogue is observable in the event log
/// without emitting one event per method on every solve. Selection is unchanged
/// — every leaf is still resolved by the existing dispatch (R13).
pub(crate) fn record_method_registry(log: &mut EventLog) -> MethodRegistry {
    let registry = MethodRegistry::from_dispatch();
    log.append("method_registry", registry.to_links_notation());
    log.append("method_registry:count", registry.method_count().to_string());
    registry
}
