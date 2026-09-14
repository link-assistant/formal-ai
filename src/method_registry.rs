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
use crate::seed::parser::parse_lino;
use crate::solver_dispatch::{
    CONTEXTUAL_HANDLER_NAMES, PRELUDE_METHOD_NAMES, specialized_handlers,
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
    /// A handler in the ordered specialized-handler table, whose precedence is
    /// read from `data/seed/handler-precedence.lino`.
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

/// One promoted method abstraction learned from solved-problem event logs.
///
/// Learned entries remain separate from `methods`: the latter is the compiled,
/// executable dispatch catalogue, while these records are adopted knowledge
/// that can inform later method construction without pretending a Rust handler
/// exists. Only the benchmark-promotion seed is loaded here; proposal documents
/// never reach the registry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LearnedMethod {
    /// Stable registry name derived from the algorithm schema.
    pub name: String,
    /// Underlying algorithm candidate identity.
    pub algorithm_id: String,
    /// Integrity identity for support and held-out observations.
    pub evidence_id: String,
    /// Event-log operations in learned order.
    pub operations: Vec<String>,
    /// Traces used for schema inference.
    pub support_trace_ids: Vec<String>,
    /// Traces withheld for validation.
    pub held_out_trace_ids: Vec<String>,
}

impl LearnedMethod {
    fn to_links_notation(&self) -> String {
        let mut pairs = vec![
            ("record_type", "learned_method".to_owned()),
            ("name", self.name.clone()),
            ("algorithm_id", self.algorithm_id.clone()),
            ("evidence_id", self.evidence_id.clone()),
        ];
        for operation in &self.operations {
            pairs.push(("operation", operation.clone()));
        }
        format_lino_record(&self.name, &pairs)
    }
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
    /// Promoted learned abstractions, kept out of compiled dispatch until an
    /// implementation supplies an executable handler.
    pub learned_methods: Vec<LearnedMethod>,
}

impl MethodRegistry {
    /// Derive the registry from the live dispatch constants.
    ///
    /// The specialized surface is built from `specialized_handlers`, whose
    /// precedence comes from `data/seed/handler-precedence.lino` joined with the
    /// registered function pointers — so the registry reflects the seed-declared
    /// order while staying grounded in the code that actually runs.
    #[must_use]
    pub fn from_dispatch() -> Self {
        Self::from_dispatch_with_learned_seed(crate::seed::LEARNED_METHODS_LINO)
            .expect("the embedded learned-method seed must be valid")
    }

    /// Derive compiled methods and parse an explicit promoted-method seed.
    ///
    /// This injected form keeps parsing independently testable; production uses
    /// only the checked-in seed via [`Self::from_dispatch`].
    ///
    /// # Errors
    ///
    /// Returns an error when an adopted record is incomplete, duplicated, or
    /// contains fewer than two ordered operations.
    pub fn from_dispatch_with_learned_seed(learned_seed: &str) -> Result<Self, String> {
        let specialized = specialized_handlers();
        let mut methods = Vec::with_capacity(
            PRELUDE_METHOD_NAMES.len() + specialized.len() + CONTEXTUAL_HANDLER_NAMES.len(),
        );
        for (order, name) in PRELUDE_METHOD_NAMES.iter().enumerate() {
            methods.push(Method {
                name: (*name).to_owned(),
                order,
                surface: MethodSurface::Prelude,
            });
        }
        for (order, (name, _handler)) in specialized.iter().enumerate() {
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
        let learned_methods = parse_learned_methods(learned_seed)?;
        Ok(Self {
            methods,
            learned_methods,
        })
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

    /// Look up one promoted learned abstraction by its registry name.
    #[must_use]
    pub fn learned_method(&self, name: &str) -> Option<&LearnedMethod> {
        self.learned_methods
            .iter()
            .find(|method| method.name == name)
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
            ("learned_count", self.learned_methods.len().to_string()),
        ];
        for method in &self.methods {
            pairs.push(("method", method.name.clone()));
        }
        let mut out = format_lino_record("method_registry", &pairs);
        for method in &self.methods {
            out.push('\n');
            out.push_str(&method.to_links_notation());
        }
        for method in &self.learned_methods {
            out.push('\n');
            out.push_str(&method.to_links_notation());
        }
        out
    }
}

fn parse_learned_methods(seed: &str) -> Result<Vec<LearnedMethod>, String> {
    let document = parse_lino(seed);
    let mut methods = Vec::new();
    for node in document
        .children
        .iter()
        .filter(|node| node.name == "learned_method")
    {
        if node.id.trim().is_empty() {
            return Err(String::from("learned_method_empty_name"));
        }
        if methods
            .iter()
            .any(|method: &LearnedMethod| method.name == node.id)
        {
            return Err(format!("duplicate_learned_method:{}", node.id));
        }
        if node.find_child_value("status") != "adopted" {
            return Err(format!("learned_method_not_adopted:{}", node.id));
        }
        let algorithm_id = node.find_child_value("algorithm_id").to_owned();
        let evidence_id = node.find_child_value("evidence_id").to_owned();
        if algorithm_id.trim().is_empty() || evidence_id.trim().is_empty() {
            return Err(format!("learned_method_identity_missing:{}", node.id));
        }
        let operations = child_values(node, "operation");
        if operations.len() < 2 {
            return Err(format!("learned_method_too_short:{}", node.id));
        }
        methods.push(LearnedMethod {
            name: node.id.clone(),
            algorithm_id,
            evidence_id,
            operations,
            support_trace_ids: child_values(node, "support_trace"),
            held_out_trace_ids: child_values(node, "held_out_trace"),
        });
    }
    Ok(methods)
}

fn child_values(node: &crate::seed::parser::LinoNode, name: &str) -> Vec<String> {
    node.children
        .iter()
        .filter(|child| child.name == name)
        .map(|child| child.id.clone())
        .collect()
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
