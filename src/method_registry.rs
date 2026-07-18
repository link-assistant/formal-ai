//! Issue #559: the method registry as first-class link data and live selection
//! data (R331).
//!
//! The meta algorithm resolves every atomic work-unit leaf with a *method* — one
//! of the solver's executable methods. The executable catalogue is still Rust,
//! but selection now goes through this registry: prelude methods, the ordered
//! handler table, and contextual overrides are all represented as method records
//! serialized to Links Notation.
//!
//! This module derives that registry *from the live code* — it reads the two
//! source-of-truth constants directly, so the data can never drift from the
//! handlers that actually run. A grounding test
//! (`tests/unit/specification/method_registry.rs`) pins the derived names against
//! the source, the same discipline the meta-recipe files use.
//!
//! The registry is recorded as a `method_registry` loop event so it is observable
//! in the event log, and the live solver dispatch also uses the registry ordering.

use crate::event_log::EventLog;
use crate::links_format::format_lino_record;
use crate::solver_dispatch::{
    CONTEXTUAL_HANDLER_NAMES, PRELUDE_METHOD_NAMES, SPECIALIZED_HANDLERS,
};

/// How a method is reached during dispatch.
///
/// The surfaces are distinct on purpose: prelude methods run before the regular
/// table, contextual overrides can replace a regular handler when conversation
/// context calls for it (for example, `numeric_list` as a history-aware
/// override), and the specialized surface is the ordered first-match table.
/// Several names appear on more than one surface, so the registry keeps them as
/// separate records rather than collapsing them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MethodSurface {
    /// A method that runs before the ordered handler table.
    Prelude,
    /// A handler in the ordered `SPECIALIZED_HANDLERS` table.
    Specialized,
    /// A context-dependent override evaluated by `try_contextual_override`.
    Contextual,
}

impl MethodSurface {
    /// Stable lowercase slug used in the Links Notation trace.
    #[must_use]
    pub const fn slug(self) -> &'static str {
        match self {
            Self::Prelude => "prelude",
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
    /// Every method, prelude first, then the specialized table, then contextual
    /// overrides.
    pub methods: Vec<Method>,
}

impl MethodRegistry {
    /// Derive the registry from the live dispatch constants.
    #[must_use]
    pub fn from_dispatch() -> Self {
        let mut methods = Vec::with_capacity(
            PRELUDE_METHOD_NAMES.len()
                + SPECIALIZED_HANDLERS.len()
                + CONTEXTUAL_HANDLER_NAMES.len(),
        );
        for (order, name) in PRELUDE_METHOD_NAMES.iter().enumerate() {
            methods.push(Method {
                name: (*name).to_owned(),
                order,
                surface: MethodSurface::Prelude,
            });
        }
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

    /// Ordered method names for one formalized impulse.
    ///
    /// The order preserves the historical prelude-first behaviour, then promotes
    /// any method named by the impulse's `route:`/`handler:` relevants, resolving
    /// those labels through the same alias-aware registry bridge that evidence and
    /// reasoning use. Finally it appends the full regular handler table in
    /// precedence order. Contextual methods are not appended as independent
    /// choices: their names also appear in the regular table, where the executor
    /// can decide whether the richer contextual variant applies.
    #[must_use]
    pub fn ordered_method_names_for_relevants(&self, relevants: &[String]) -> Vec<String> {
        let mut ordered = Vec::new();
        for method in self
            .methods
            .iter()
            .filter(|method| method.surface == MethodSurface::Prelude)
        {
            push_unique(&mut ordered, method.name.clone());
        }
        for relevant in relevants {
            let Some(route) = relevant
                .strip_prefix("route:")
                .or_else(|| relevant.strip_prefix("handler:"))
            else {
                continue;
            };
            if let Some(method) = self.method_for_route(route) {
                push_unique(&mut ordered, method.name.clone());
            }
        }
        for method in self
            .methods
            .iter()
            .filter(|method| method.surface == MethodSurface::Specialized)
        {
            push_unique(&mut ordered, method.name.clone());
        }
        ordered
    }

    /// Serialize the registry and every method to Links Notation (R311).
    #[must_use]
    pub fn to_links_notation(&self) -> String {
        let mut pairs: Vec<(&str, String)> = vec![
            ("record_type", "method_registry".to_owned()),
            ("method_count", self.method_count().to_string()),
            (
                "prelude_count",
                self.count_on(MethodSurface::Prelude).to_string(),
            ),
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

fn push_unique(values: &mut Vec<String>, value: String) {
    if !values.contains(&value) {
        values.push(value);
    }
}

/// Build the method registry and emit it as a loop event plus its Links Notation
/// trace.
///
/// Emit the live method catalogue as one `method_registry` event (the serialized
/// catalogue, which itself enumerates every method) and a compact
/// `method_registry:count`, so the catalogue is observable in the event log
/// without emitting one event per method on every solve. The same registry
/// ordering drives `meta_method_dispatch`.
pub(crate) fn record_method_registry(log: &mut EventLog) -> MethodRegistry {
    let registry = MethodRegistry::from_dispatch();
    log.append("method_registry", registry.to_links_notation());
    log.append("method_registry:count", registry.method_count().to_string());
    registry
}
