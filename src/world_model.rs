//! Symbolic world models and contexts — issue #649.
//!
//! Issue #649 asks the project to reason with **symbolic world models** on the
//! links network rather than embeddings: throughout a dialogue build a
//! **current-state** context and a **target-state** context, expose their
//! **difference**, let the user and agent **synchronize** the target, **merge**
//! and **split** context models, keep **each context a links network**, use
//! [relative-meta-logic](https://github.com/link-foundation/relative-meta-logic)
//! so that **statements are dependent** and **changing the world recalculates all
//! statement probabilities**, and ultimately **predict the consequences of an
//! action** by simulating it.
//!
//! The [design case study](../docs/case-studies/issue-649/README.md) shows the
//! feature is an *audit-and-wire* task over machinery that already exists. This
//! module supplies the missing connections:
//!
//! * A first-class [`Context`] = an id, a links network ([`SubstitutionGraph`]),
//!   and a set of dependent [`Statement`]s. This is the STRIPS *state* / *goal*
//!   container, re-expressed over doublets.
//! * An [`Action`] = a set of add/delete link edits (STRIPS *effects*).
//! * [`Context::recalculate`] — a JTMS-style fixpoint that re-fires
//!   [`StatementAssessment`] for every dependent statement whenever the context
//!   changes, so *any* edit recalculates *all* statement probabilities and the
//!   values converge deterministically on the relative-meta-logic decimal grid.
//! * [`Context::difference`] — the STRIPS *goal − state* delta between two
//!   contexts (links to add / remove, plus functional conflicts).
//! * [`Context::predict`] — *predict = apply the action to a clone, recalculate,
//!   and diff*, so the real world model is never mutated (issue-649's headline
//!   requirement).
//! * [`Context::merge_from`] / [`Context::split_off`] — ATMS-style context
//!   combination and separation, reusing union-by-id semantics.
//! * [`WorldModel`] — the per-dialogue holder of the `current`, `target`, and
//!   shared `general` contexts.
//!
//! Everything here is pure symbolic arithmetic over caller-supplied links and
//! evidence: no clocks, no randomness, no network. Values are snapped to the RML
//! decimal grid so the trace is byte-for-byte reproducible.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

use crate::engine::stable_id;
use crate::relative_meta_logic::{
    RelativeEvidence, SourceTier, Stance, StatementAssessment, TruthValue, ASSUMED_TRUE_PRIOR,
};
use crate::substitution::{SubstitutionGraph, SubstitutionLink};

/// Upper bound on relaxation passes in [`Context::recalculate`].
///
/// The cascade re-evaluates every statement from the current values of its
/// dependencies; because truth values are snapped to a finite decimal grid a
/// monotone cascade converges in a handful of passes. This constant is a
/// non-termination backstop for the pathological case of a negative-feedback
/// dependency cycle (a statement whose dependency contradicts it), which can
/// oscillate rather than settle. It is multiplied by the statement count so a
/// deep dependency chain still relaxes fully before the guard trips.
const MAX_RECALCULATION_PASSES_PER_STATEMENT: usize = 4;

/// How one statement depends on another inside a context.
///
/// A dependency is a directed edge from the dependent statement to the statement
/// it relies on, tagged with the [`Stance`] the dependency takes: a
/// [`Stance::Supports`] edge raises the dependent's probability as the relied-on
/// statement becomes more true (a JTMS *positive justification*), a
/// [`Stance::Contradicts`] edge lowers it (a *negative justification*).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dependency {
    /// The id of the statement this one depends on.
    pub on: String,
    /// Whether the relied-on statement supports or contradicts this one.
    pub stance: Stance,
}

impl Dependency {
    /// A positive justification: `id` becomes more true as `on` does.
    #[must_use]
    pub fn supports(on: impl Into<String>) -> Self {
        Self {
            on: on.into(),
            stance: Stance::Supports,
        }
    }

    /// A negative justification: `id` becomes less true as `on` becomes true.
    #[must_use]
    pub fn contradicts(on: impl Into<String>) -> Self {
        Self {
            on: on.into(),
            stance: Stance::Contradicts,
        }
    }
}

/// A dependent statement inside a [`Context`].
///
/// A statement carries its own assumed-true `prior` and base `evidence` (weighed
/// exactly as [`StatementAssessment`] does elsewhere), plus zero or more
/// [`Dependency`] edges onto other statements in the same context. Its `truth`
/// is the value [`Context::recalculate`] last computed; it is graph-visible via
/// [`Context::links_notation`].
#[derive(Debug, Clone, PartialEq)]
pub struct Statement {
    /// Content-addressed id, stable over the statement text.
    pub id: String,
    /// The statement text.
    pub text: String,
    /// The assumed-true prior before any evidence or dependency is weighed.
    pub prior: TruthValue,
    /// Base evidence weighed against this statement.
    pub evidence: Vec<RelativeEvidence>,
    /// Edges onto the statements this one depends on.
    pub dependencies: Vec<Dependency>,
    /// The probability last computed by [`Context::recalculate`].
    pub truth: TruthValue,
}

impl Statement {
    /// Build a statement with the module default [`ASSUMED_TRUE_PRIOR`] and no
    /// dependencies. Its initial `truth` is its own evidence-only posterior.
    #[must_use]
    pub fn new(text: impl Into<String>) -> Self {
        let text = text.into();
        let id = stable_id("world_statement", &text);
        let prior = TruthValue::new(ASSUMED_TRUE_PRIOR);
        let truth = StatementAssessment::assess(&text, prior, &[]).posterior;
        Self {
            id,
            text,
            prior,
            evidence: Vec::new(),
            dependencies: Vec::new(),
            truth,
        }
    }

    /// Attach base evidence, returning `self` for chaining.
    #[must_use]
    pub fn with_evidence(mut self, evidence: RelativeEvidence) -> Self {
        self.evidence.push(evidence);
        self
    }

    /// Attach a dependency edge, returning `self` for chaining.
    #[must_use]
    pub fn with_dependency(mut self, dependency: Dependency) -> Self {
        self.dependencies.push(dependency);
        self
    }

    /// Override the assumed-true prior, returning `self` for chaining.
    #[must_use]
    pub fn with_prior(mut self, prior: impl Into<TruthValue>) -> Self {
        self.prior = prior.into();
        self
    }
}

/// An action to simulate against a context: a set of link edits.
///
/// This is the STRIPS *effect* set re-expressed over doublets — `remove` is the
/// delete list, `add` is the add list. [`Context::predict`] applies it to a
/// clone so the real world model is never touched.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Action {
    /// A human-readable name for the action (e.g. `"open the door"`).
    pub name: String,
    /// Links the action deletes from the state.
    pub remove: Vec<SubstitutionLink>,
    /// Links the action adds to the state.
    pub add: Vec<SubstitutionLink>,
}

impl Action {
    /// Start an empty action with a name.
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            remove: Vec::new(),
            add: Vec::new(),
        }
    }

    /// Add an "assert this link" effect, returning `self` for chaining.
    #[must_use]
    pub fn adding(mut self, from: impl Into<String>, to: impl Into<String>) -> Self {
        self.add.push(SubstitutionLink::new(from, to));
        self
    }

    /// Add a "retract this link" effect, returning `self` for chaining.
    #[must_use]
    pub fn removing(mut self, from: impl Into<String>, to: impl Into<String>) -> Self {
        self.remove.push(SubstitutionLink::new(from, to));
        self
    }

    /// A content-addressed id, stable over the action's name and effects.
    #[must_use]
    pub fn id(&self) -> String {
        let mut canonical = format!("name:{};", self.name);
        for link in &self.remove {
            let _ = write!(canonical, "remove:{};", link.pattern_text());
        }
        for link in &self.add {
            let _ = write!(canonical, "add:{};", link.pattern_text());
        }
        stable_id("world_action", &canonical)
    }
}

/// A named world/context model: a links network plus its dependent statements.
///
/// Each context is *always* a links network — the [`SubstitutionGraph`] is the
/// network — satisfying issue-649's "each context is always a links network"
/// requirement. Statements live alongside it and their dependency structure is
/// mirrored into the graph so the whole context serializes to Links Notation.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Context {
    /// The context id.
    pub id: String,
    /// The world-state links network (STRIPS atoms).
    links: SubstitutionGraph,
    /// The dependent statements, keyed by id for stable iteration and merge.
    statements: BTreeMap<String, Statement>,
}

impl Context {
    /// An empty context with the given id.
    #[must_use]
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            links: SubstitutionGraph::new(),
            statements: BTreeMap::new(),
        }
    }

    /// Assert a world-state atom (a link). Returns `true` if newly added.
    pub fn assert_link(&mut self, from: &str, to: &str) -> bool {
        self.links.insert_link(from, to)
    }

    /// Retract a world-state atom (a link). Returns `true` if it was present.
    pub fn retract_link(&mut self, from: &str, to: &str) -> bool {
        self.links.remove_link(from, to)
    }

    /// Whether the state atom `from -> to` holds.
    #[must_use]
    pub fn holds(&self, from: &str, to: &str) -> bool {
        self.links.contains_link(from, to)
    }

    /// Every world-state atom currently in the context, sorted.
    #[must_use]
    pub fn links(&self) -> Vec<SubstitutionLink> {
        self.links.links()
    }

    /// Read-only view of the statements, keyed by id.
    #[must_use]
    pub const fn statements(&self) -> &BTreeMap<String, Statement> {
        &self.statements
    }

    /// Look up a statement by id.
    #[must_use]
    pub fn statement(&self, id: &str) -> Option<&Statement> {
        self.statements.get(id)
    }

    /// Insert (or replace) a statement, then recalculate the whole context so
    /// the new statement's dependencies and every dependent value settle. Returns
    /// the id of the inserted statement.
    pub fn add_statement(&mut self, statement: Statement) -> String {
        let id = statement.id.clone();
        self.statements.insert(id.clone(), statement);
        let _ = self.recalculate();
        id
    }

    /// Re-evaluate every statement to a fixpoint (JTMS-style cascade).
    ///
    /// Each pass recomputes every statement's [`TruthValue`] from its own
    /// evidence plus the *current* values of the statements it depends on
    /// (supporting dependencies raise it, contradicting ones lower it), reusing
    /// [`StatementAssessment::assess`]. Passes repeat until no snapped value
    /// changes (converged) or the `MAX_RECALCULATION_PASSES_PER_STATEMENT`
    /// bound trips. The dependency structure is mirrored into the links network
    /// so the context stays a single inspectable graph.
    pub fn recalculate(&mut self) -> RecalculationReport {
        let ids: Vec<String> = self.statements.keys().cloned().collect();
        let before: BTreeMap<String, TruthValue> = ids
            .iter()
            .map(|id| (id.clone(), self.statements[id].truth))
            .collect();
        let limit = ids
            .len()
            .saturating_mul(MAX_RECALCULATION_PASSES_PER_STATEMENT)
            .max(MAX_RECALCULATION_PASSES_PER_STATEMENT);

        let mut iterations = 0;
        let mut converged = false;
        while iterations < limit {
            iterations += 1;
            let snapshot: BTreeMap<String, TruthValue> = ids
                .iter()
                .map(|id| (id.clone(), self.statements[id].truth))
                .collect();
            let mut changed = false;
            for id in &ids {
                let recomputed = self.assess_with_dependencies(id, &snapshot);
                if recomputed != self.statements[id].truth {
                    if let Some(statement) = self.statements.get_mut(id) {
                        statement.truth = recomputed;
                    }
                    changed = true;
                }
            }
            if !changed {
                converged = true;
                break;
            }
        }

        self.sync_statement_links();

        let updated = ids
            .iter()
            .filter_map(|id| {
                let after = self.statements[id].truth;
                let previous = before.get(id).copied().unwrap_or(after);
                (after != previous).then(|| StatementChange {
                    statement_id: id.clone(),
                    text: self.statements[id].text.clone(),
                    before: previous,
                    after,
                })
            })
            .collect();

        RecalculationReport {
            iterations,
            converged,
            updated,
        }
    }

    /// Assess one statement from its own evidence plus the current truth values
    /// of its dependencies, expressed as synthesized full-trust evidence so the
    /// existing relative-meta-logic kernel does the combination.
    fn assess_with_dependencies(
        &self,
        id: &str,
        snapshot: &BTreeMap<String, TruthValue>,
    ) -> TruthValue {
        let statement = &self.statements[id];
        let mut evidence = statement.evidence.clone();
        for dependency in &statement.dependencies {
            if let Some(value) = snapshot.get(&dependency.on) {
                // A dependency is a first-party justification: its strength is the
                // relied-on statement's own current probability, weighed at full
                // trust so a certainly-true supporter fully supports.
                evidence.push(RelativeEvidence::new(
                    format!("statement:{}", dependency.on),
                    SourceTier::OriginalFirstParty,
                    dependency.stance,
                    *value,
                ));
            }
        }
        StatementAssessment::assess(&statement.text, statement.prior, &evidence).posterior
    }

    /// Mirror statement existence, dependency edges, and truth values into the
    /// links network so the context is one inspectable links graph.
    fn sync_statement_links(&mut self) {
        // Drop stale statement-layer links, then re-emit from the current state.
        for link in self.links.links() {
            if is_statement_layer_link(&link) {
                self.links.remove_link(&link.from, &link.to);
            }
        }
        for statement in self.statements.values() {
            self.links.insert_link(&statement.id, "world:statement");
            self.links
                .insert_link(&statement.id, &format!("truth:{}", statement.truth));
            for dependency in &statement.dependencies {
                self.links.insert_link(
                    &statement.id,
                    &format!("{}:{}", dependency.stance.slug(), dependency.on),
                );
            }
        }
    }

    /// Apply an action's link edits in place, then recalculate.
    ///
    /// This mutates the context — use [`Self::predict`] to simulate an action
    /// against a clone without touching the real state.
    pub fn apply_action(&mut self, action: &Action) -> RecalculationReport {
        for link in &action.remove {
            self.links.remove_link(&link.from, &link.to);
        }
        for link in &action.add {
            self.links.insert_link(&link.from, &link.to);
        }
        self.recalculate()
    }

    /// Predict the consequences of an action **without mutating** this context.
    ///
    /// Clones the context, applies the action to the clone, recalculates, and
    /// diffs the clone against the original — so the returned [`Prediction`]
    /// reports exactly what *would* change (state links added/removed plus every
    /// statement whose probability moves) while the real world model is left
    /// untouched. This is issue-649's headline capability.
    #[must_use]
    pub fn predict(&self, action: &Action) -> Prediction {
        let mut probe = self.clone();
        probe.apply_action(action);
        let difference = self.difference(&probe);
        let statement_changes = probe.statement_changes_against(self);
        Prediction {
            action_id: action.id(),
            action_name: action.name.clone(),
            added: difference.to_add,
            removed: difference.to_remove,
            statement_changes,
            result: probe,
        }
    }

    /// The STRIPS-style delta from `self` (current) to `target` (goal).
    ///
    /// `to_add` is what `target` has that `self` lacks, `to_remove` is what
    /// `self` has that `target` lacks, and `conflicting` pairs links that share
    /// a `from` but disagree on `to` (a functional conflict the sync must
    /// resolve). Only world-state atoms are diffed; statement-layer bookkeeping
    /// links are ignored.
    #[must_use]
    pub fn difference(&self, target: &Self) -> ContextDiff {
        let current: BTreeSet<SubstitutionLink> = self.state_links();
        let goal: BTreeSet<SubstitutionLink> = target.state_links();

        let to_add: Vec<SubstitutionLink> = goal.difference(&current).cloned().collect();
        let to_remove: Vec<SubstitutionLink> = current.difference(&goal).cloned().collect();

        let mut conflicting = Vec::new();
        for removed in &to_remove {
            for added in &to_add {
                if removed.from == added.from && removed.to != added.to {
                    conflicting.push(LinkConflict {
                        current: removed.clone(),
                        target: added.clone(),
                    });
                }
            }
        }

        ContextDiff {
            to_add,
            to_remove,
            conflicting,
        }
    }

    /// Merge another context into this one (ATMS context combination).
    ///
    /// World-state links are unioned; statements are unioned by id with the
    /// incoming context winning ties (last-writer-wins, matching
    /// [`crate::memory_sync::merge_union_by_id`]). The result is recalculated so
    /// dependencies that now cross the merged contexts settle.
    pub fn merge_from(&mut self, other: &Self) -> RecalculationReport {
        for link in other.state_links() {
            self.links.insert_link(&link.from, &link.to);
        }
        for (id, statement) in &other.statements {
            self.statements.insert(id.clone(), statement.clone());
        }
        self.recalculate()
    }

    /// Split a child context off this one (ATMS context separation).
    ///
    /// The child gets copies of the named statements plus every world-state link
    /// that references one of them (by id) — the sub-network relevant to those
    /// statements — leaving `self` unchanged. Shared links are copied, so the
    /// two contexts can then diverge independently. Unknown ids are ignored.
    #[must_use]
    pub fn split_off(&self, child_id: impl Into<String>, statement_ids: &[String]) -> Self {
        let mut child = Self::new(child_id);
        let selected: BTreeSet<&String> = statement_ids
            .iter()
            .filter(|id| self.statements.contains_key(*id))
            .collect();
        for id in &selected {
            if let Some(statement) = self.statements.get(*id) {
                child.statements.insert((*id).clone(), statement.clone());
            }
        }
        for link in self.state_links() {
            if selected.contains(&link.from) || selected.contains(&link.to) {
                child.links.insert_link(&link.from, &link.to);
            }
        }
        let _ = child.recalculate();
        child
    }

    /// Render the whole context — world-state atoms and statement layer — as
    /// Links Notation for exact, glass-box explanation.
    #[must_use]
    pub fn links_notation(&self) -> String {
        self.links.links_notation()
    }

    /// The world-state atoms only (statement-layer bookkeeping links removed).
    fn state_links(&self) -> BTreeSet<SubstitutionLink> {
        self.links
            .links()
            .into_iter()
            .filter(|link| !is_statement_layer_link(link))
            .collect()
    }

    /// The statements whose truth differs from their value in `baseline`.
    fn statement_changes_against(&self, baseline: &Self) -> Vec<StatementChange> {
        self.statements
            .values()
            .filter_map(|statement| {
                let before = baseline
                    .statements
                    .get(&statement.id)
                    .map_or(statement.truth, |previous| previous.truth);
                (before != statement.truth).then(|| StatementChange {
                    statement_id: statement.id.clone(),
                    text: statement.text.clone(),
                    before,
                    after: statement.truth,
                })
            })
            .collect()
    }
}

/// Whether a link belongs to the statement bookkeeping layer rather than the
/// world state, so diffs and merges can operate on state atoms alone.
fn is_statement_layer_link(link: &SubstitutionLink) -> bool {
    link.to == "world:statement"
        || link.to.starts_with("truth:")
        || link.to.starts_with("supports:")
        || link.to.starts_with("contradicts:")
        || link.to.starts_with("neutral:")
}

/// The result of [`Context::recalculate`].
#[derive(Debug, Clone, PartialEq)]
pub struct RecalculationReport {
    /// How many relaxation passes ran.
    pub iterations: usize,
    /// Whether the cascade reached a fixpoint before the guard tripped.
    pub converged: bool,
    /// The statements whose truth value moved during this recalculation.
    pub updated: Vec<StatementChange>,
}

/// One statement's probability moving from `before` to `after`.
#[derive(Debug, Clone, PartialEq)]
pub struct StatementChange {
    /// The statement id.
    pub statement_id: String,
    /// The statement text.
    pub text: String,
    /// The probability before the change.
    pub before: TruthValue,
    /// The probability after the change.
    pub after: TruthValue,
}

/// Two links that share a `from` but disagree on `to` — a functional conflict
/// the current→target sync must resolve.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkConflict {
    /// The link as it stands in the current context.
    pub current: SubstitutionLink,
    /// The link as the target context wants it.
    pub target: SubstitutionLink,
}

/// The difference between two contexts' world states (STRIPS goal − state).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ContextDiff {
    /// Links present in the target but missing from the current context.
    pub to_add: Vec<SubstitutionLink>,
    /// Links present in the current context but missing from the target.
    pub to_remove: Vec<SubstitutionLink>,
    /// Same-`from`, different-`to` conflicts between the two contexts.
    pub conflicting: Vec<LinkConflict>,
}

impl ContextDiff {
    /// Whether the two contexts already agree (nothing to add, remove, or
    /// resolve).
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.to_add.is_empty() && self.to_remove.is_empty() && self.conflicting.is_empty()
    }

    /// Render the delta as Links Notation for exact explanation.
    #[must_use]
    pub fn links_notation(&self) -> String {
        let mut out = String::from("context_diff\n");
        for link in &self.to_add {
            let _ = writeln!(out, "  to_add \"{}\"", link.pattern_text());
        }
        for link in &self.to_remove {
            let _ = writeln!(out, "  to_remove \"{}\"", link.pattern_text());
        }
        for conflict in &self.conflicting {
            let _ = writeln!(
                out,
                "  conflict \"{} vs {}\"",
                conflict.current.pattern_text(),
                conflict.target.pattern_text()
            );
        }
        out.trim_end().to_owned()
    }
}

/// The consequences of an action predicted against a context, without mutating
/// it. Returned by [`Context::predict`].
#[derive(Debug, Clone, PartialEq)]
pub struct Prediction {
    /// The simulated action's content-addressed id.
    pub action_id: String,
    /// The simulated action's human-readable name.
    pub action_name: String,
    /// State links the action would add.
    pub added: Vec<SubstitutionLink>,
    /// State links the action would remove.
    pub removed: Vec<SubstitutionLink>,
    /// Statements whose probability the action would move.
    pub statement_changes: Vec<StatementChange>,
    /// The full post-action context (a copy; the original is untouched).
    pub result: Context,
}

impl Prediction {
    /// Whether the action would change nothing (no state edit lands and no
    /// statement moves).
    #[must_use]
    pub const fn is_noop(&self) -> bool {
        self.added.is_empty() && self.removed.is_empty() && self.statement_changes.is_empty()
    }
}

/// The per-dialogue world model: a `current` context, a `target` context, and a
/// shared `general` context that per-dialogue contexts merge into.
///
/// This is the top-level holder issue-649 describes — *"at any stage of the
/// dialogue we have the current representation of the world … and also the
/// target representation the user wants"* — with `general` as the *"entire world
/// model / general context"* a dialogue context can be merged into or split
/// from.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct WorldModel {
    /// The current state of the (partial) world discussed in the dialogue.
    pub current: Context,
    /// The target state the user wants.
    pub target: Context,
    /// The shared world model per-dialogue contexts merge into.
    pub general: Context,
}

impl WorldModel {
    /// A fresh world model with three empty, distinctly-named contexts.
    #[must_use]
    pub fn new() -> Self {
        Self {
            current: Context::new("current"),
            target: Context::new("target"),
            general: Context::new("general"),
        }
    }

    /// The difference from the current state to the target state (what the agent
    /// must achieve). Directly exposes issue-649's *"difference from the current
    /// state"* at any dialogue stage.
    #[must_use]
    pub fn difference(&self) -> ContextDiff {
        self.current.difference(&self.target)
    }

    /// Predict an action's consequences against the current state without
    /// mutating it.
    #[must_use]
    pub fn predict(&self, action: &Action) -> Prediction {
        self.current.predict(action)
    }

    /// Whether the current state already satisfies the target (the difference is
    /// empty) — i.e. the dialogue's goal is reached.
    #[must_use]
    pub fn target_reached(&self) -> bool {
        self.difference().is_empty()
    }

    /// Merge the current dialogue context into the shared general world model
    /// (ATMS context combination), returning the recalculation report.
    pub fn commit_current_to_general(&mut self) -> RecalculationReport {
        self.general.merge_from(&self.current)
    }
}
