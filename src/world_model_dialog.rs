//! The dialogue-driven symbolic world model — issue #702.
//!
//! [`crate::world_model`] (issue #649) supplies the substrate: a [`Context`] is
//! a links network plus its dependent statements, a [`ContextDiff`] is the
//! STRIPS goal − state delta, and [`Context::predict`] simulates an action
//! against a clone. What it does *not* do is watch a conversation. Issue #702
//! adds that missing half:
//!
//! 1. **Current-state context** — every formalized declarative statement in the
//!    dialogue lands in `current` with provenance back to the turn that said it.
//! 2. **Target-state context** — "I want …" and imperative requests route into
//!    `target` instead.
//! 3. **Difference** — [`DialogueWorldModel::remaining`] is the queryable
//!    "what is left to do?" answer, and it is itself links.
//! 4. **Synchronization loop** — the agent *proposes* a target edit, the user
//!    confirms or corrects it, and both sides converge. Every step of that loop
//!    is an **append-only** [`SyncEvent`] in a hash-chained log: events are only
//!    ever pushed, and each one commits to its predecessor's id.
//! 5. **Merge and split** — [`DialogueWorldModel::merge_from`] and
//!    [`DialogueWorldModel::split_current`] are first-class operations that
//!    reuse the context substrate and record their own events.
//! 6. **Dependent statements** — a causal utterance ("X because Y") records a
//!    relative-meta-logic dependency, so revising the premise recalculates every
//!    dependent statement and the event names each recalculated link.
//! 7. **Action-consequence prediction** — [`DialogueWorldModel::forecast`] runs
//!    [`Context::predict`] and reads the result *against the target*: which
//!    needs the action satisfies, which it violates, and whether the gap to the
//!    goal shrinks. A destructive action is flagged before it is executed.
//! 8. **Everything is links** — no embeddings and no graph/edge/vertex
//!    vocabulary; the recognition vocabulary is link data
//!    (`data/meta/cue-lexicon.lino`), all four supported languages are covered
//!    by that data, the behaviour is off until [`WorldModelMode`] is opted in,
//!    and the whole dialogue model renders as Links Notation.
//!
//! Nothing here reads a clock, a random number, or the network: replaying the
//! same turns always rebuilds the same contexts, the same event ids, and the
//! same answer.

use std::fmt::Write as _;

use crate::engine::stable_id;
use crate::relative_meta_logic::{RelativeEvidence, SourceTier, Stance, TruthValue};
use crate::solver::{ConversationRole, ConversationTurn};
use crate::substitution::SubstitutionLink;
use crate::world_model::{
    Action, Context, ContextDiff, Dependency, Prediction, RecalculationReport, Statement,
    WorldModel,
};
use crate::world_model_atoms::{classify, premise_split, state_atom, UtteranceKind};

/// Whether the dialogue world model is maintained and traced.
///
/// Default [`Off`](WorldModelMode::Off): the solver behaves exactly as it did
/// before issue #702 — no world-model events are appended and the state-query
/// handler declines. [`Track`](WorldModelMode::Track) opts the dialogue in; the
/// model is then rebuilt from the conversation, written to the trace, and the
/// "what is left to do?" question is answered from it. This mirrors the
/// trace-only-until-opted-in shape of [`crate::selection::SelectionMode`].
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum WorldModelMode {
    /// Do not maintain or trace a dialogue world model (the default).
    #[default]
    Off,
    /// Maintain the model and record it as a trace artifact.
    Track,
}

impl WorldModelMode {
    /// Whether this mode maintains the model and emits its trace artifact.
    #[must_use]
    pub const fn emits_artifact(self) -> bool {
        matches!(self, Self::Track)
    }

    /// The stable slug used in configuration and traces.
    #[must_use]
    pub const fn slug(self) -> &'static str {
        match self {
            Self::Off => "off",
            Self::Track => "track",
        }
    }

    /// Parse a slug; unknown input yields `None` so callers keep their default.
    #[must_use]
    pub fn from_slug(slug: &str) -> Option<Self> {
        match slug.trim().to_ascii_lowercase().as_str() {
            "off" => Some(Self::Off),
            "track" => Some(Self::Track),
            _ => None,
        }
    }
}

/// What one step of the current⇄target synchronization loop did.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncEventKind {
    /// A declarative statement entered the current-state context.
    CurrentAsserted,
    /// A wish or request entered the target-state context.
    TargetAsserted,
    /// The agent proposed a target edit and is waiting for the user.
    TargetProposed,
    /// The user accepted the pending proposal.
    TargetConfirmed,
    /// The user rejected the pending proposal and supplied a replacement.
    TargetCorrected,
    /// The user asked what is left to reach the target.
    StateQueried,
    /// A statement was revised, cascading into its dependents.
    StatementRevised,
    /// An action was forecast against the target.
    ActionForecast,
    /// Another dialogue's contexts were merged into this one.
    ContextMerged,
    /// A sub-context was split off the current context.
    ContextSplit,
}

impl SyncEventKind {
    /// The stable slug used in the event log and Links Notation.
    #[must_use]
    pub const fn slug(self) -> &'static str {
        match self {
            Self::CurrentAsserted => "current_asserted",
            Self::TargetAsserted => "target_asserted",
            Self::TargetProposed => "target_proposed",
            Self::TargetConfirmed => "target_confirmed",
            Self::TargetCorrected => "target_corrected",
            Self::StateQueried => "state_queried",
            Self::StatementRevised => "statement_revised",
            Self::ActionForecast => "action_forecast",
            Self::ContextMerged => "context_merged",
            Self::ContextSplit => "context_split",
        }
    }
}

/// One append-only step of the synchronization loop.
///
/// `id` is content-addressed over the event *and its parent's id*, so the log is
/// a hash chain: no recorded step can be edited or dropped without every later
/// id changing. [`DialogueWorldModel::chain_is_intact`] re-derives the chain to
/// check exactly that.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyncEvent {
    /// Zero-based position in the append-only log.
    pub sequence: usize,
    /// Content-addressed id over `(parent, sequence, kind, actor, detail)`.
    pub id: String,
    /// The previous event's id, or `world_sync_root` for the first event.
    pub parent: String,
    /// What this step did.
    pub kind: SyncEventKind,
    /// Who caused it: `user`, `assistant`, or `system`.
    pub actor: String,
    /// The step's payload, rendered as a links-notation-safe string.
    pub detail: String,
}

/// The id the first event of a chain commits to.
pub const SYNC_CHAIN_ROOT: &str = "world_sync_root";

impl SyncEvent {
    /// Re-derive this event's content address from its own fields.
    #[must_use]
    pub fn derived_id(&self) -> String {
        stable_id(
            "world_sync",
            &format!(
                "{}|{}|{}|{}|{}",
                self.parent,
                self.sequence,
                self.kind.slug(),
                self.actor,
                self.detail
            ),
        )
    }

    /// Render the event as Links Notation.
    #[must_use]
    pub fn links_notation(&self) -> String {
        let mut out = format!("sync_event {}\n", self.id);
        let _ = writeln!(out, "  sequence \"{}\"", self.sequence);
        let _ = writeln!(out, "  parent \"{}\"", self.parent);
        let _ = writeln!(out, "  kind \"{}\"", self.kind.slug());
        let _ = writeln!(out, "  actor \"{}\"", self.actor);
        let _ = write!(out, "  detail \"{}\"", self.detail);
        out
    }
}

/// An action's consequences read against the target state (issue #702, req. 7).
///
/// The underlying [`Prediction`] says what *would* change; this says what that
/// change *means for the goal*: the target needs it satisfies, the ones it
/// violates, and whether the remaining work shrinks. It is derived entirely from
/// link set arithmetic, so it is inspectable and reproducible.
#[derive(Debug, Clone, PartialEq)]
pub struct ActionForecast {
    /// The simulated action's content-addressed id.
    pub action_id: String,
    /// The simulated action's name.
    pub action_name: String,
    /// The raw prediction against the current context.
    pub prediction: Prediction,
    /// Target links the action would newly satisfy.
    pub satisfied: Vec<SubstitutionLink>,
    /// Target links the action would destroy — the needs it violates.
    pub violated: Vec<SubstitutionLink>,
    /// How many target links were missing before the action.
    pub remaining_before: usize,
    /// How many would be missing after it.
    pub remaining_after: usize,
}

impl ActionForecast {
    /// Whether the action moves the dialogue closer to the target.
    #[must_use]
    pub const fn shrinks_gap(&self) -> bool {
        self.remaining_after < self.remaining_before
    }

    /// Whether the action destroys something the target needs. A destructive
    /// action is flagged here *before* it is executed.
    #[must_use]
    pub const fn violates_target(&self) -> bool {
        !self.violated.is_empty()
    }

    /// Render the forecast as Links Notation.
    #[must_use]
    pub fn links_notation(&self) -> String {
        let mut out = format!("action_forecast {}\n", self.action_id);
        let _ = writeln!(out, "  action \"{}\"", self.action_name);
        for link in &self.satisfied {
            let _ = writeln!(out, "  satisfies \"{}\"", link.pattern_text());
        }
        for link in &self.violated {
            let _ = writeln!(out, "  violates \"{}\"", link.pattern_text());
        }
        for change in &self.prediction.statement_changes {
            let _ = writeln!(
                out,
                "  statement_change \"{} {} -> {}\"",
                change.statement_id, change.before, change.after
            );
        }
        let _ = writeln!(out, "  remaining_before \"{}\"", self.remaining_before);
        let _ = writeln!(out, "  remaining_after \"{}\"", self.remaining_after);
        let _ = write!(
            out,
            "  verdict \"{}\"",
            if self.violates_target() {
                "violates_target"
            } else if self.shrinks_gap() {
                "shrinks_gap"
            } else {
                "no_progress"
            }
        );
        out
    }
}

/// A symbolic world model maintained *from a dialogue*.
///
/// Built by replaying conversation turns: user utterances are classified against
/// the `world_state_*` cue sets and turned into links-network atoms in either the
/// current or the target context, while every synchronization step is appended
/// to the hash-chained [`SyncEvent`] log.
#[derive(Debug, Clone, PartialEq)]
pub struct DialogueWorldModel {
    /// The current/target/general contexts (issue #649's holder).
    pub model: WorldModel,
    events: Vec<SyncEvent>,
    pending: Option<SubstitutionLink>,
    turns: usize,
}

impl Default for DialogueWorldModel {
    fn default() -> Self {
        Self::new()
    }
}

impl DialogueWorldModel {
    /// An empty dialogue world model.
    #[must_use]
    pub fn new() -> Self {
        Self {
            model: WorldModel::new(),
            events: Vec::new(),
            pending: None,
            turns: 0,
        }
    }

    /// Replay a whole conversation into a fresh model.
    #[must_use]
    pub fn from_turns(turns: &[ConversationTurn]) -> Self {
        let mut model = Self::new();
        for turn in turns {
            model.observe(turn.role, &turn.content);
        }
        model
    }

    /// Feed one turn into the model, returning how it was classified.
    ///
    /// Assistant turns never write state on their own: the agent *proposes*
    /// (see [`Self::propose_target`]) and only the user's confirmation commits
    /// the proposal, which is what makes the loop a synchronization rather than
    /// the agent overwriting the user's goal.
    pub fn observe(&mut self, role: ConversationRole, text: &str) -> UtteranceKind {
        self.turns += 1;
        let kind = classify(text);
        if role == ConversationRole::Assistant {
            return kind;
        }
        match kind {
            UtteranceKind::CurrentState => self.assert_current(text),
            UtteranceKind::TargetState => self.assert_target(text),
            UtteranceKind::Confirmation => self.confirm_pending(text),
            UtteranceKind::Correction => self.correct_pending(text),
            UtteranceKind::RemainingQuery => {
                let remaining = self.remaining().len();
                self.append(
                    SyncEventKind::StateQueried,
                    "user",
                    format!("remaining:{remaining}"),
                );
            }
            UtteranceKind::Unrelated => {}
        }
        kind
    }

    /// Convenience wrapper for a user turn.
    pub fn observe_user(&mut self, text: &str) -> UtteranceKind {
        self.observe(ConversationRole::User, text)
    }

    /// The agent's half of the synchronization loop: propose a target edit and
    /// wait. The proposal is recorded but *not* applied — [`Self::observe_user`]
    /// applies it when the user confirms, or replaces it when they correct.
    pub fn propose_target(&mut self, from: &str, to: &str) -> &SyncEvent {
        let link = SubstitutionLink::new(from, to);
        let detail = link.pattern_text();
        self.pending = Some(link);
        self.append(SyncEventKind::TargetProposed, "assistant", detail)
    }

    /// The pending, unconfirmed target proposal, if any.
    #[must_use]
    pub const fn pending_proposal(&self) -> Option<&SubstitutionLink> {
        self.pending.as_ref()
    }

    /// The append-only synchronization log.
    #[must_use]
    pub fn events(&self) -> &[SyncEvent] {
        &self.events
    }

    /// Whether the hash chain over the append-only log still re-derives, i.e.
    /// no recorded step was edited, reordered, or dropped.
    #[must_use]
    pub fn chain_is_intact(&self) -> bool {
        let mut parent = SYNC_CHAIN_ROOT.to_owned();
        for (index, event) in self.events.iter().enumerate() {
            if event.sequence != index || event.parent != parent || event.id != event.derived_id() {
                return false;
            }
            parent.clone_from(&event.id);
        }
        true
    }

    /// The current→target difference: the whole "what is left to do?" answer as
    /// links.
    #[must_use]
    pub fn difference(&self) -> ContextDiff {
        self.model.difference()
    }

    /// The target links the current state still lacks, sorted and stable.
    #[must_use]
    pub fn remaining(&self) -> Vec<SubstitutionLink> {
        self.model.difference().to_add
    }

    /// Whether the dialogue's goal is reached.
    #[must_use]
    pub fn target_reached(&self) -> bool {
        self.remaining().is_empty()
    }

    /// Record a causal dependency between two statement texts, so revising the
    /// premise recalculates the consequent (relative-meta-logic, req. 6).
    pub fn depends_on(&mut self, consequent: &str, premise: &str) -> String {
        let premise_statement = Statement::new(premise);
        let premise_id = self.model.current.add_statement(premise_statement);
        let dependent =
            Statement::new(consequent).with_dependency(Dependency::supports(premise_id.clone()));
        self.model.current.add_statement(dependent);
        premise_id
    }

    /// Revise a statement with new evidence and cascade the change through every
    /// statement that depends on it.
    ///
    /// The returned report — and the appended [`SyncEventKind::StatementRevised`]
    /// event — name every recalculated link, so the trace shows exactly which
    /// posteriors moved and to what.
    pub fn revise_statement(
        &mut self,
        text: &str,
        stance: Stance,
        strength: f64,
    ) -> RecalculationReport {
        let id = stable_id("world_statement", text);
        let mut statement = self
            .model
            .current
            .statement(&id)
            .cloned()
            .unwrap_or_else(|| Statement::new(text));
        statement.evidence.push(RelativeEvidence::new(
            format!("revision:{id}"),
            SourceTier::OriginalFirstParty,
            stance,
            TruthValue::new(strength),
        ));
        // `extend_statements` inserts and recalculates in one step, so the
        // report it returns is the cascade this revision caused — the values
        // that moved, not the ones a second pass would find already settled.
        let report = self.model.current.extend_statements([statement]);
        let mut detail = format!("{id} recalculated:{}", report.updated.len());
        for change in &report.updated {
            let _ = write!(
                detail,
                "; {} {} -> {}",
                change.statement_id, change.before, change.after
            );
        }
        self.append(SyncEventKind::StatementRevised, "system", detail);
        report
    }

    /// Forecast an action's consequences against the target state without
    /// touching the real world model (req. 7).
    pub fn forecast(&mut self, action: &Action) -> ActionForecast {
        let prediction = self.model.predict(action);
        let before = self.model.current.difference(&self.model.target);
        let after = prediction.result.difference(&self.model.target);
        let target_links = self
            .model
            .target
            .difference(&Context::new("empty"))
            .to_remove;

        let satisfied: Vec<SubstitutionLink> = before
            .to_add
            .iter()
            .filter(|link| !after.to_add.contains(link))
            .cloned()
            .collect();
        let violated: Vec<SubstitutionLink> = after
            .to_add
            .iter()
            .filter(|link| !before.to_add.contains(link) && target_links.contains(link))
            .cloned()
            .collect();

        let forecast = ActionForecast {
            action_id: prediction.action_id.clone(),
            action_name: prediction.action_name.clone(),
            prediction,
            satisfied,
            violated,
            remaining_before: before.to_add.len(),
            remaining_after: after.to_add.len(),
        };
        let detail = format!(
            "{} satisfies:{} violates:{} remaining:{}->{}",
            forecast.action_name,
            forecast.satisfied.len(),
            forecast.violated.len(),
            forecast.remaining_before,
            forecast.remaining_after
        );
        self.append(SyncEventKind::ActionForecast, "system", detail);
        forecast
    }

    /// Merge another dialogue's contexts into this one (union with conflict
    /// detection), recording the conflicts the merge exposed (req. 5).
    pub fn merge_from(&mut self, other: &Self) -> Vec<SubstitutionLink> {
        let conflicts: Vec<SubstitutionLink> = self
            .model
            .current
            .difference(&other.model.current)
            .conflicting
            .iter()
            .map(|conflict| conflict.current.clone())
            .collect();
        self.model.current.merge_from(&other.model.current);
        self.model.target.merge_from(&other.model.target);
        let detail = format!("{} conflicts:{}", other.model.current.id, conflicts.len());
        self.append(SyncEventKind::ContextMerged, "system", detail);
        conflicts
    }

    /// Split a sub-context off the current context by statement id (req. 5).
    pub fn split_current(&mut self, child_id: &str, statement_ids: &[String]) -> Context {
        let child = self.model.current.split_off(child_id, statement_ids);
        let detail = format!("{child_id} statements:{}", statement_ids.len());
        self.append(SyncEventKind::ContextSplit, "system", detail);
        child
    }

    /// Render the whole dialogue world model — both contexts, the difference,
    /// and the append-only log — as Links Notation.
    #[must_use]
    pub fn links_notation(&self) -> String {
        let mut out = String::from("dialogue_world_model\n");
        let _ = writeln!(out, "  record_type \"dialogue_world_model\"");
        let _ = writeln!(out, "  turns \"{}\"", self.turns);
        let _ = writeln!(out, "  events \"{}\"", self.events.len());
        let _ = writeln!(out, "  current_state");
        for link in self.model.current.links() {
            let _ = writeln!(out, "    link \"{}\"", link.pattern_text());
        }
        let _ = writeln!(out, "  target_state");
        for link in self.model.target.links() {
            let _ = writeln!(out, "    link \"{}\"", link.pattern_text());
        }
        let _ = writeln!(out, "  state_diff");
        for line in self.difference().links_notation().lines().skip(1) {
            let _ = writeln!(out, "  {}", line.trim_end());
        }
        for event in &self.events {
            for line in event.links_notation().lines() {
                let _ = writeln!(out, "  {line}");
            }
        }
        out.trim_end().to_owned()
    }

    // -- internals ---------------------------------------------------------

    /// Assert a declarative utterance into the current-state context, with a
    /// statement carrying provenance back to the turn that said it. A causal
    /// utterance also records the dependency it states.
    fn assert_current(&mut self, text: &str) {
        let Some(link) = state_atom(text) else {
            return;
        };
        let turn = self.turns;
        self.model.current.assert_link(&link.from, &link.to);
        record_provenance(&mut self.model.current, text, &link, turn);
        if let Some((consequent, premise)) = premise_split(text) {
            self.depends_on(&consequent, &premise);
        }
        self.append(SyncEventKind::CurrentAsserted, "user", link.pattern_text());
    }

    /// Assert a wish or request into the target-state context.
    fn assert_target(&mut self, text: &str) {
        let Some(link) = state_atom(text) else {
            return;
        };
        let turn = self.turns;
        self.model.target.assert_link(&link.from, &link.to);
        record_provenance(&mut self.model.target, text, &link, turn);
        self.append(SyncEventKind::TargetAsserted, "user", link.pattern_text());
    }

    /// Apply the pending proposal: the user accepted the agent's reading.
    fn confirm_pending(&mut self, text: &str) {
        let detail = match self.pending.take() {
            Some(link) => {
                self.model.target.assert_link(&link.from, &link.to);
                let turn = self.turns;
                record_provenance(&mut self.model.target, text, &link, turn);
                link.pattern_text()
            }
            None => String::from("nothing_pending"),
        };
        self.append(SyncEventKind::TargetConfirmed, "user", detail);
    }

    /// Replace the pending proposal: the user rejected the agent's reading and
    /// supplied the right atom, which supersedes any target link on the same
    /// subject.
    fn correct_pending(&mut self, text: &str) {
        let rejected = self.pending.take().map_or_else(
            || String::from("nothing_pending"),
            |link| link.pattern_text(),
        );
        let detail = match state_atom(text) {
            Some(link) => {
                for existing in self.model.target.links() {
                    if existing.from == link.from && existing.to != link.to {
                        self.model.target.retract_link(&existing.from, &existing.to);
                    }
                }
                self.model.target.assert_link(&link.from, &link.to);
                let turn = self.turns;
                record_provenance(&mut self.model.target, text, &link, turn);
                format!("{rejected} => {}", link.pattern_text())
            }
            None => format!("{rejected} => retracted"),
        };
        self.append(SyncEventKind::TargetCorrected, "user", detail);
    }

    /// Push one step onto the append-only, hash-chained log.
    fn append(&mut self, kind: SyncEventKind, actor: &str, detail: String) -> &SyncEvent {
        let parent = self
            .events
            .last()
            .map_or_else(|| SYNC_CHAIN_ROOT.to_owned(), |event| event.id.clone());
        let sequence = self.events.len();
        let mut event = SyncEvent {
            sequence,
            id: String::new(),
            parent,
            kind,
            actor: actor.to_owned(),
            detail,
        };
        event.id = event.derived_id();
        self.events.push(event);
        self.events
            .last()
            .unwrap_or_else(|| unreachable!("an event was just pushed"))
    }
}

/// Attach the utterance that produced an atom to the context as a statement,
/// linked to the turn it came from and to the atom it asserts. Both links live
/// in the statement bookkeeping layer, so they never pollute a state difference.
fn record_provenance(context: &mut Context, text: &str, link: &SubstitutionLink, turn: usize) {
    let statement = Statement::new(text.trim());
    let id = context.add_statement(statement);
    context.assert_link(&id, &format!("provenance:turn:{turn}"));
    context.assert_link(&id, &format!("asserts:{}", link.pattern_text()));
}

/// Record the dialogue world model as a trace artifact, honouring the mode.
///
/// Returns the rendered Links Notation when the mode opts in, `None` otherwise —
/// the same trace-only-until-opted-in shape as `selection::record_selection`.
pub fn record_world_model(
    log: &mut crate::event_log::EventLog,
    model: &DialogueWorldModel,
    mode: WorldModelMode,
) -> Option<String> {
    if !mode.emits_artifact() {
        return None;
    }
    let notation = model.links_notation();
    log.append("world_model", notation.clone());
    Some(notation)
}
