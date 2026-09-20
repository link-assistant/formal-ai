//! Issue #559: the method registry as first-class link data and live selection
//! data (R331).
//!
//! The meta algorithm resolves every atomic work-unit leaf with a *method* — one
//! of the solver's executable methods. The executable catalogue is still Rust,
//! but selection now goes through this registry: prelude methods, the ordered
//! handler table, and contextual overrides are all represented as method records
//! serialized to Links Notation.
//!
//! This module derives that registry *from the live code through the link
//! store* — the specialized table's precedence is the rank-link order the
//! store declares, joined with the function pointers that actually run, and
//! the execution and learned-method documents are read as the store's
//! documents (plan 09 leaf 41, stage 3) — so the data can never drift from the
//! handlers that actually run. A grounding test
//! (`tests/unit/specification/method_registry.rs`) pins the derived names against
//! the source, the same discipline the meta-recipe files use.
//!
//! The registry is recorded as a `method_registry` loop event so it is observable
//! in the event log, and the live solver dispatch also uses the registry ordering.

use std::collections::BTreeMap;
use std::fmt::Write as _;

use crate::event_log::EventLog;
use crate::links_format::format_lino_record;
use crate::seed::parser::parse_lino;
use crate::solver_dispatch::{
    CONTEXTUAL_HANDLER_NAMES, PRELUDE_METHOD_NAMES, specialized_handlers,
};

const METHOD_EXECUTION_LINO: &str = include_str!("../data/seed/method-execution.lino");
const METHOD_EXECUTION_PATH: &str = "data/seed/method-execution.lino";
const LEARNED_METHODS_PATH: &str = "data/seed/learned-methods.lino";

/// A pre-handler runtime selected by a method-record attribute.
///
/// The enum is a closed catalogue of native kernels. Which method invokes one
/// is data in `method-execution.lino`, so adding or moving a method never adds
/// a name branch to the dispatcher.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MethodRuntimeKind {
    Diagnostic,
    NaturalLanguageTool,
    BehaviorRules,
    FeatureCapability,
    PlaywrightScript,
}

/// A response-language-aware native kernel selected by a method attribute.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResponseLanguageVariant {
    ConceptLookup,
    PatternInference,
}

/// When the generic project lookup fallback runs for a method.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectLookupPhase {
    BeforeRuntime,
    Fallback,
}

/// Execution attributes attached to a registry method by seed data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct MethodExecution {
    /// Optional native pre-handler runtime.
    pub runtime: Option<MethodRuntimeKind>,
    /// Optional forced-response-language renderer.
    pub response_language: Option<ResponseLanguageVariant>,
    /// Optional generic project lookup and the phase at which it runs.
    pub project_lookup: Option<ProjectLookupPhase>,
    /// Whether definition fusion may run before this method.
    pub definition_fusion: bool,
}

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
    /// Uniform execution policy parsed from `data/seed/method-execution.lino`.
    pub execution: MethodExecution,
}

/// Review status of a learned method after its behavior-delta measurement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LearnedMethodStatus {
    /// Its held-out effect qualified, so the executable record may dispatch.
    Adopted,
    /// Review found a change but no verified improvement. The record remains
    /// discoverable as experience and may not alter answers.
    AdoptedNotEffective,
}

impl LearnedMethodStatus {
    /// Stable seed/trace spelling.
    #[must_use]
    pub const fn slug(self) -> &'static str {
        match self {
            Self::Adopted => "adopted",
            Self::AdoptedNotEffective => "adopted_not_effective",
        }
    }

    /// Only a method with a qualifying observed effect is dispatchable.
    #[must_use]
    pub const fn permits_dispatch(self) -> bool {
        matches!(self, Self::Adopted)
    }
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
    /// The review outcome backed by the adoption-effect ledger.
    pub status: LearnedMethodStatus,
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

/// One observed execution of an adopted learned method.
///
/// The answer is the same recipe projection returned by live dispatch.  The
/// verification bit is derived from the execution log: every operation the
/// learned record declares must be represented by an observed recorder event.
/// This gives behavior-delta measurement a checkable postcondition instead of
/// upgrading an arbitrary byte change to an improvement.
#[derive(Debug, Clone)]
pub struct LearnedMethodExecution {
    pub answer: String,
    pub executed_steps: Vec<String>,
    pub skipped_steps: Vec<String>,
    pub operations_verified: bool,
}

impl LearnedMethod {
    /// The learned operations as a recipe program the existing interpreter runs
    /// (issue #1138 B7, plan 07 leaf 4).
    ///
    /// A `LearnedMethod`'s `operations` are already the recorder event-kind
    /// vocabulary `src/recipe_interpreter.rs` dispatches on, so no second
    /// interpreter is written.
    ///
    /// # Errors
    /// Returns the unbound operation's name when one of the learned operations
    /// binds to no recorder.
    pub fn to_recipe_program(&self) -> Result<crate::recipe_interpreter::RecipeProgram, String> {
        let recipe = crate::recipe_interpreter::RecipeProgram::from_repo();
        let mut selected_orders = Vec::new();
        for operation in &self.operations {
            // `need:status` and `method_registry:count` are the *facets* of one
            // recorded event, so an operation binds through the event kind its
            // base names. A facet adds no second step.
            let base = operation.split(':').next().unwrap_or(operation);
            let Some(step) = recipe.steps.iter().find(|step| step_emits(step, base)) else {
                // A slug, not a sentence: the surface renders the refusal from
                // seed prose, and what the caller needs from the error is the
                // operation that did not bind (R379).
                return Err(format!("unbound_operation:{operation}"));
            };
            selected_orders.push(step.order);
        }

        // A recipe is an ordered data-flow program, not a bag of recorders. A
        // learned suffix such as `need -> ... -> reasoning_standard` still
        // needs the earlier frame/work-unit stages that provide its inputs.
        // Close over every executable prefix step through the last learned
        // operation. This is deliberately structural: a newly learned suffix
        // gets its prerequisites from recipe order, with no method-name or
        // benchmark-specific branch.
        let last_order = selected_orders.into_iter().max().unwrap_or(0);
        let steps = recipe
            .steps
            .into_iter()
            .filter(|step| step.is_executable() && step.order <= last_order)
            .collect();
        Ok(crate::recipe_interpreter::RecipeProgram { steps })
    }

    /// Whether every learned operation binds to a known recorder. A method that
    /// does not bind stays in the registry event as data, is never dispatched,
    /// and the unbound operation is named in the trace. Silent skipping is
    /// forbidden.
    #[must_use]
    pub fn is_executable(&self) -> bool {
        self.to_recipe_program().is_ok()
    }

    /// Execute this abstraction through the shared recipe interpreter and
    /// verify its declared operations against the resulting event log.
    ///
    /// # Errors
    ///
    /// Returns the same stable binding or dependency error live dispatch
    /// records when the learned program cannot execute.
    pub fn execute(
        &self,
        formalization: &crate::intent_formalization::IntentFormalization,
        config: crate::solver::SolverConfig,
    ) -> Result<LearnedMethodExecution, String> {
        let program = self.to_recipe_program()?;
        let trace = program.execute(
            formalization,
            config.max_decomposition_depth,
            config.recursion_mode,
            config.selection_mode,
            config.skill_mode,
        )?;
        let operations_verified = self.operations.iter().all(|operation| {
            let base = operation.split(':').next().unwrap_or(operation);
            trace
                .log
                .events()
                .iter()
                .any(|event| event_kind_matches_base(event.kind, base))
        });
        let mut answer = program.to_links_notation();
        for id in &trace.executed {
            let _ = write!(answer, "\n  executed {id}");
        }
        for id in &trace.skipped {
            let _ = write!(answer, "\n  skipped {id}");
        }
        Ok(LearnedMethodExecution {
            answer,
            executed_steps: trace.executed,
            skipped_steps: trace.skipped,
            operations_verified,
        })
    }

    fn to_links_notation(&self) -> String {
        let mut pairs = vec![
            ("record_type", "learned_method".to_owned()),
            ("name", self.name.clone()),
            ("status", self.status.slug().to_owned()),
            ("algorithm_id", self.algorithm_id.clone()),
            ("evidence_id", self.evidence_id.clone()),
        ];
        for operation in &self.operations {
            pairs.push(("operation", operation.clone()));
        }
        // An answer that used a learned method says so: the trace event carries
        // the `method:learned` marker beside the record, and a method that could
        // not bind carries the operation that failed rather than vanishing.
        pairs.push(("trace_event", format!("method:learned:{}", self.name)));
        match self.to_recipe_program() {
            Ok(program) => pairs.push(("bound_steps", program.steps.len().to_string())),
            Err(reason) => pairs.push(("unbound_operation", reason)),
        }
        format_lino_record(&self.name, &pairs)
    }
}

impl Method {
    #[must_use]
    fn to_links_notation(&self) -> String {
        let mut pairs: Vec<(&str, String)> = vec![
            ("record_type", "method".to_owned()),
            ("name", self.name.clone()),
            ("order", self.order.to_string()),
            ("surface", self.surface.slug().to_owned()),
        ];
        if let Some(runtime) = self.execution.runtime {
            pairs.push(("runtime", runtime.slug().to_owned()));
        }
        if let Some(variant) = self.execution.response_language {
            pairs.push(("response_language", variant.slug().to_owned()));
        }
        if let Some(phase) = self.execution.project_lookup {
            pairs.push(("project_lookup", phase.slug().to_owned()));
        }
        if self.execution.definition_fusion {
            pairs.push(("definition_fusion", "true".to_owned()));
        }
        format_lino_record(&self.name, &pairs)
    }
}

impl MethodRuntimeKind {
    const fn slug(self) -> &'static str {
        match self {
            Self::Diagnostic => "diagnostic",
            Self::NaturalLanguageTool => "natural_language_tool",
            Self::BehaviorRules => "behavior_rules",
            Self::FeatureCapability => "feature_capability",
            Self::PlaywrightScript => "playwright_script",
        }
    }

    /// Native function that implements this data-selected runtime.
    ///
    /// Keeping this mapping on the closed runtime type lets introspection and
    /// dispatch share the same executable identity.  Callers therefore do not
    /// need method-name exceptions for prelude methods whose entry point is
    /// selected by `method-execution.lino`.
    #[must_use]
    pub const fn entry_point(self) -> &'static str {
        match self {
            Self::Diagnostic => "try_diagnostic",
            Self::NaturalLanguageTool => "try_natural_language_tool_request",
            Self::BehaviorRules => "try_behavior_rules_with_runtime",
            Self::FeatureCapability => "try_feature_capability",
            Self::PlaywrightScript => "try_playwright_script",
        }
    }
}

impl ResponseLanguageVariant {
    const fn slug(self) -> &'static str {
        match self {
            Self::ConceptLookup => "concept_lookup",
            Self::PatternInference => "pattern_inference",
        }
    }
}

impl ProjectLookupPhase {
    const fn slug(self) -> &'static str {
        match self {
            Self::BeforeRuntime => "before_runtime",
            Self::Fallback => "fallback",
        }
    }
}

/// Whether one recipe step's recorder emits the event kind `base` names.
///
/// The recipe binds a step to a recorder (`record_need_ledger`); a learned
/// operation names the *event kind* that recorder emits (`need:status`). The two
/// meet at the recorder's name with its `record_` prefix removed, and a base
/// that is a prefix of that name binds too -- `need` is how the ledger's events
/// are spelled in a trace.
fn step_emits(step: &crate::recipe_interpreter::RecipeStep, base: &str) -> bool {
    let Some(recorder) = &step.records else {
        return false;
    };
    let kind = recorder.strip_prefix("record_").unwrap_or(recorder);
    kind == base || kind.starts_with(&format!("{base}_"))
}

/// The full catalogue of methods the solver can route an atomic leaf to.
///
/// Built from the live dispatch constants via [`MethodRegistry::shared`],
/// so the data is grounded in the code by construction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MethodRegistry {
    /// Every method, prelude first, then the specialized table, then contextual
    /// overrides.
    pub methods: Vec<Method>,
    /// Promoted learned abstractions, kept out of compiled dispatch until an
    /// implementation supplies an executable handler.
    pub learned_methods: Vec<LearnedMethod>,
    /// Selection heuristics, loaded from `data/meta/selection-heuristics.lino`
    /// (issue #1138 B12, plan 12 leaf 2).
    ///
    /// A heuristic is never a route target: [`Self::method_for_route`] never
    /// returns one. One registry with three collections keeps R344's single
    /// dispatch authority true — plan 07 owns the execution of
    /// `learned_methods` and plan 12 owns `heuristics`, and neither grows the
    /// struct without the other's declaration (plan 00 section 9 R16).
    pub heuristics: Vec<crate::selection_heuristics::HeuristicMethod>,
}

impl MethodRegistry {
    /// The registry every production call site shares, built once.
    ///
    /// Construction enters through the projected link store
    /// ([`crate::seed_links::network`]), the same boot projection the rule
    /// interpreter reads since plan 09 leaf 40, so one store serves every
    /// routing decision (plan 09 leaf 41, stage 3 — the closing leaf of the
    /// read-path migration).
    #[must_use]
    pub fn shared() -> &'static Self {
        static CELL: std::sync::OnceLock<MethodRegistry> = std::sync::OnceLock::new();
        CELL.get_or_init(Self::from_store)
    }

    /// Derive the registry from the link store.
    ///
    /// The specialized surface's precedence is the rank-link order
    /// [`crate::seed::handler_precedence`] reads from the store; the execution
    /// attributes and the shipped learned-method seed are read as the store's
    /// documents. The function pointers stay code — they cannot be data — and
    /// [`crate::solver_dispatch::specialized_handlers`] still asserts the
    /// store-declared order is an exact permutation of them.
    fn from_store() -> Self {
        let network = crate::seed_links::network();
        let learned_seed = network
            .document_text(LEARNED_METHODS_PATH)
            .unwrap_or(crate::seed::LEARNED_METHODS_LINO);
        Self::from_store_with_learned_seed(learned_seed)
            .expect("the store's method documents must be valid")
    }

    /// Derive compiled methods from the live dispatch constants and parse an
    /// explicit promoted-method seed.
    ///
    /// The execution attributes always read the store's method-execution
    /// document, the same text production reads; only the promoted-method seed
    /// is injected, which keeps parsing independently testable while production
    /// uses only the store via [`Self::shared`].
    ///
    /// # Errors
    ///
    /// Returns an error when an adopted record is incomplete, duplicated, or
    /// contains fewer than two ordered operations.
    pub fn from_store_with_learned_seed(learned_seed: &str) -> Result<Self, String> {
        let network = crate::seed_links::network();
        let execution_seed = network
            .document_text(METHOD_EXECUTION_PATH)
            .unwrap_or(METHOD_EXECUTION_LINO);
        let specialized = specialized_handlers();
        let execution = parse_method_execution(execution_seed)?;
        let mut methods = Vec::with_capacity(
            PRELUDE_METHOD_NAMES.len() + specialized.len() + CONTEXTUAL_HANDLER_NAMES.len(),
        );
        for (order, name) in PRELUDE_METHOD_NAMES.iter().enumerate() {
            methods.push(Method {
                name: (*name).to_owned(),
                order,
                surface: MethodSurface::Prelude,
                execution: execution_for(&execution, name),
            });
        }
        for (order, (name, _handler)) in specialized.iter().enumerate() {
            methods.push(Method {
                name: (*name).to_owned(),
                order,
                surface: MethodSurface::Specialized,
                execution: execution_for(&execution, name),
            });
        }
        for (order, name) in CONTEXTUAL_HANDLER_NAMES.iter().enumerate() {
            methods.push(Method {
                name: (*name).to_owned(),
                order,
                surface: MethodSurface::Contextual,
                execution: execution_for(&execution, name),
            });
        }
        let learned_methods = parse_learned_methods(learned_seed)?;
        Ok(Self {
            methods,
            learned_methods,
            heuristics: crate::selection_heuristics::shipped_catalog(),
        })
    }

    /// The heuristics for `role`, in declared precedence order, filtered by
    /// `situation`.
    ///
    /// Empty is a reportable state: the core then falls back to the
    /// deterministic identity ordering and says so in the trace, emitting
    /// `heuristic:none` with the role and the situation. It never silently
    /// falls back (issue #1138 B12, plan 12 leaf 2).
    #[must_use]
    pub fn heuristics_for(
        &self,
        role: crate::selection_heuristics::HeuristicRole,
        situation: &str,
    ) -> Vec<&crate::selection_heuristics::HeuristicMethod> {
        let mut selected: Vec<&crate::selection_heuristics::HeuristicMethod> = self
            .heuristics
            .iter()
            .filter(|heuristic| heuristic.role == role && heuristic.applies_in(situation))
            .collect();
        selected.sort_by_key(|heuristic| heuristic.order);
        selected
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

    /// Resolve explicit `method:<content-addressed-name>` references from a
    /// prompt against the learned registry.
    ///
    /// This is the generic invocation surface for learned methods. Names come
    /// from seed data, and tokenization merely preserves the link punctuation;
    /// adding a learned item therefore makes its explicit reference
    /// discoverable without adding a Rust condition for that item or language.
    #[must_use]
    pub fn explicit_learned_method_relevants(&self, prompt: &str) -> Vec<String> {
        let tokens: Vec<&str> = prompt
            .split(|character: char| {
                !(character.is_alphanumeric() || matches!(character, ':' | '_' | '-'))
            })
            .filter(|token| !token.is_empty())
            .collect();
        self.learned_methods
            .iter()
            .map(|method| format!("method:{}", method.name))
            .filter(|reference| tokens.contains(&reference.as_str()))
            .collect()
    }

    /// Execution attributes for one method name.
    ///
    /// Several surfaces may carry the same name; the seed declares attributes
    /// by method identity, so every copy must agree. An undeclared method uses
    /// the uniform native-handler path.
    #[must_use]
    pub fn execution_for(&self, name: &str) -> MethodExecution {
        self.methods
            .iter()
            .find(|method| method.name == name)
            .map_or_else(MethodExecution::default, |method| method.execution)
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
        // The fourth loop (issue #1138 B7, plan 07 leaf 5). Learned methods rank
        // after every compiled method, always: adopted knowledge nothing can
        // reach is not knowledge, and knowledge that outranks a compiled handler
        // is a second dispatch authority. A learned method whose operations bind
        // to no recorder is *named* rather than silently skipped -- it stays in
        // the registry event as data and never enters the ordering, which is what
        // the `method:learned` trace event reports.
        for relevant in relevants {
            let Some(name) = relevant.strip_prefix("method:") else {
                continue;
            };
            let Some(learned) = self.learned_method(name) else {
                continue;
            };
            if learned.status.permits_dispatch() && learned.is_executable() {
                push_unique(&mut ordered, learned.name.clone());
            }
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
            ("heuristic_count", self.heuristics.len().to_string()),
        ];
        for method in &self.methods {
            pairs.push(("method", method.name.clone()));
        }
        for heuristic in &self.heuristics {
            pairs.push(("heuristic", heuristic.name.clone()));
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
        for heuristic in &self.heuristics {
            out.push('\n');
            out.push_str(&heuristic.to_links_notation());
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
        // Proposal documents never reach the registry. An empirically
        // ineffective adoption remains available as experience, but cannot
        // dispatch; losing it would erase the negative result and invite the
        // same proposal again.
        let status = node.find_child_value("status");
        let status = match status {
            "" | "adopted" => LearnedMethodStatus::Adopted,
            "adopted_not_effective" => LearnedMethodStatus::AdoptedNotEffective,
            _ => return Err(format!("learned_method_not_adopted:{}", node.id)),
        };
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
            status,
            algorithm_id,
            evidence_id,
            operations,
            support_trace_ids: child_values(node, "support_trace"),
            held_out_trace_ids: child_values(node, "held_out_trace"),
        });
    }
    Ok(methods)
}

fn parse_method_execution(seed: &str) -> Result<BTreeMap<String, MethodExecution>, String> {
    let document = parse_lino(seed);
    let root = document
        .children
        .iter()
        .find(|node| node.name == "method_execution")
        .ok_or_else(|| String::from("method_execution_missing_root"))?;
    let mut attributes = BTreeMap::new();
    for method in root.children.iter().filter(|node| node.name == "method") {
        if method.id.trim().is_empty() {
            return Err(String::from("method_execution_empty_method"));
        }
        let mut execution = MethodExecution::default();
        for attribute in &method.children {
            match attribute.name.as_str() {
                "runtime" => {
                    execution.runtime = Some(match attribute.id.as_str() {
                        "diagnostic" => MethodRuntimeKind::Diagnostic,
                        "natural_language_tool" => MethodRuntimeKind::NaturalLanguageTool,
                        "behavior_rules" => MethodRuntimeKind::BehaviorRules,
                        "feature_capability" => MethodRuntimeKind::FeatureCapability,
                        "playwright_script" => MethodRuntimeKind::PlaywrightScript,
                        other => return Err(format!("method_execution_unknown_runtime:{other}")),
                    });
                }
                "response_language" => {
                    execution.response_language = Some(match attribute.id.as_str() {
                        "concept_lookup" => ResponseLanguageVariant::ConceptLookup,
                        "pattern_inference" => ResponseLanguageVariant::PatternInference,
                        other => {
                            return Err(format!(
                                "method_execution_unknown_response_language:{other}"
                            ));
                        }
                    });
                }
                "project_lookup" => {
                    execution.project_lookup = Some(match attribute.id.as_str() {
                        "before_runtime" => ProjectLookupPhase::BeforeRuntime,
                        "fallback" => ProjectLookupPhase::Fallback,
                        other => {
                            return Err(format!("method_execution_unknown_project_phase:{other}"));
                        }
                    });
                }
                "definition_fusion" if attribute.id == "true" => {
                    execution.definition_fusion = true;
                }
                other => return Err(format!("method_execution_unknown_attribute:{other}")),
            }
        }
        if attributes.insert(method.id.clone(), execution).is_some() {
            return Err(format!("method_execution_duplicate_method:{}", method.id));
        }
    }
    Ok(attributes)
}

fn execution_for(attributes: &BTreeMap<String, MethodExecution>, name: &str) -> MethodExecution {
    attributes.get(name).copied().unwrap_or_default()
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

fn event_kind_matches_base(kind: &str, base: &str) -> bool {
    kind == base
        || kind
            .strip_prefix(base)
            .is_some_and(|suffix| suffix.starts_with(':') || suffix.starts_with('_'))
}

/// Build the method registry and emit it as a loop event plus its Links Notation
/// trace.
///
/// Emit the live method catalogue as one `method_registry` event (the serialized
/// catalogue, which itself enumerates every method) and a compact
/// `method_registry:count`, so the catalogue is observable in the event log
/// without emitting one event per method on every solve. The same registry
/// ordering drives `meta_method_dispatch`.
pub(crate) fn record_method_registry(log: &mut EventLog) -> &'static MethodRegistry {
    let registry = MethodRegistry::shared();
    log.append("method_registry", registry.to_links_notation());
    log.append("method_registry:count", registry.method_count().to_string());
    registry
}
