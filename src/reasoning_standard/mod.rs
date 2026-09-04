//! The reasoning standard (issue #1073), as ordered gates over a reasoning episode.
//!
//! The maintainer asked for a reference dialog's depth to become the floor for
//! every request, for reasoning to be refutation-first, for gathered internet
//! instructions to be formalized, and for source trust to be *computed* rather
//! than assumed — and then, in requirement 6, for all of that to be "expressed
//! as formal, checkable procedures rather than stylistic guidance", such that
//! "with the language model removed, the same conclusions must remain reachable
//! by following the formal procedure".
//!
//! So the standard is not prose in a prompt. It is
//! [`data/meta/reasoning-standard.lino`](../../data/meta/reasoning-standard.lino):
//! an ordered list of gates, each with a trigger, a requirement, and the slug it
//! fails with, plus the numeric thresholds the gates compare against. This
//! module loads that file and evaluates every gate as a pure predicate over a
//! [`ReasoningEpisode`]. No gate consults a model, a clock, or the network.
//!
//! Two properties matter and are pinned by tests:
//!
//! * **The depth floor is unconditional.** [`audit`] enumerates *every* declared
//!   gate on every episode, trivial or hard. A gate whose trigger did not fire is
//!   reported [`GateStatus::NotTriggered`] together with the trigger that was
//!   false — never omitted, so depth cannot quietly depend on how hard the task
//!   looked or on how the request was phrased.
//! * **The default verdict is honest.** Unless every triggered gate is satisfied
//!   *and* the refutation ledger settles, the verdict is
//!   [`Verdict::NotConfirmedNotRefuted`] carrying the blockers by name.

pub mod episode;
pub mod instructions;
pub mod refutation;
pub mod trust;

use std::collections::BTreeMap;
use std::fmt;

use crate::event_log::EventLog;
use crate::intent_formalization::IntentFormalization;
use crate::links_format::push_lino_node;
use crate::seed::parser::{LinoNode, parse_lino};

use episode::{ActionOutcome, ReasoningEpisode};
use instructions::InstructionSet;
use refutation::{LedgerState, RefutationLedger};

/// The formal standard, embedded so the audit ships with the library.
const REASONING_STANDARD: &str = include_str!("../../data/meta/reasoning-standard.lino");

/// The reference episode distilled from the dialog issue #1073 points at.
const REFERENCE_EPISODE: &str =
    include_str!("../../data/meta/reasoning-standard-reference-episode.lino");

/// A structural problem in the standard's data file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReasoningStandardError {
    /// Stable slug naming what was wrong.
    pub reason: String,
}

impl fmt::Display for ReasoningStandardError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.reason)
    }
}

impl std::error::Error for ReasoningStandardError {}

fn error(reason: impl Into<String>) -> ReasoningStandardError {
    ReasoningStandardError {
        reason: reason.into(),
    }
}

/// What has to be present in an episode for a gate to apply.
///
/// Triggers exist so a gate that does not apply says *which* predicate was false
/// rather than disappearing from the ledger.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GateTrigger {
    /// The episode claims something about the world.
    WorldClaim,
    /// The episode consulted at least one source.
    SourceConsultation,
    /// The episode declares a task class instructions were gathered for.
    InstructionNeed,
    /// The episode weighs at least one source's trust.
    SourceAssessment,
    /// The episode would conclude something.
    Conclusion,
    /// The episode changed the world.
    MutatingAction,
    /// Something the episode did failed or only partly succeeded.
    ActionFailure,
}

impl GateTrigger {
    /// Stable slug used in the standard's data file.
    #[must_use]
    pub const fn slug(self) -> &'static str {
        match self {
            Self::WorldClaim => "world_claim_present",
            Self::SourceConsultation => "source_consultation_present",
            Self::InstructionNeed => "instruction_need_present",
            Self::SourceAssessment => "source_assessment_present",
            Self::Conclusion => "conclusion_present",
            Self::MutatingAction => "mutating_action_present",
            Self::ActionFailure => "action_failure_present",
        }
    }

    fn from_slug(slug: &str) -> Option<Self> {
        [
            Self::WorldClaim,
            Self::SourceConsultation,
            Self::InstructionNeed,
            Self::SourceAssessment,
            Self::Conclusion,
            Self::MutatingAction,
            Self::ActionFailure,
        ]
        .into_iter()
        .find(|trigger| trigger.slug() == slug)
    }

    /// Whether this trigger fires for `episode`.
    #[must_use]
    pub fn fires(self, episode: &ReasoningEpisode) -> bool {
        match self {
            Self::WorldClaim => episode.claims.iter().any(|claim| claim.about_world),
            Self::SourceConsultation | Self::SourceAssessment => !episode.sources.is_empty(),
            Self::InstructionNeed => !episode.task_class.trim().is_empty(),
            Self::Conclusion => !episode.conclusions.is_empty(),
            Self::MutatingAction => episode.actions.iter().any(|action| action.mutating),
            Self::ActionFailure => episode
                .actions
                .iter()
                .any(|action| !matches!(action.outcome, ActionOutcome::Succeeded)),
        }
    }
}

/// One declared gate of the standard.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Gate {
    /// Stable slug (`evidence_before_claim`, `refutation_variety`, …).
    pub slug: String,
    /// Position in the pipeline, starting at 1.
    pub order: usize,
    /// What must be present for the gate to apply.
    pub trigger: GateTrigger,
    /// What the gate demands, in the standard's own words.
    pub requirement: String,
    /// The slug this gate fails with.
    pub failure_slug: String,
}

/// The loaded standard: the gates, the thresholds, and the depth-floor promise.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReasoningStandard {
    /// Title from the data file.
    pub title: String,
    /// The depth floor the standard promises (`unconditional`).
    pub depth_floor: String,
    /// Gates in pipeline order.
    pub gates: Vec<Gate>,
    /// Numeric thresholds by slug.
    pub thresholds: BTreeMap<String, usize>,
}

impl ReasoningStandard {
    /// A threshold by slug, or `0` when the standard does not declare it.
    #[must_use]
    pub fn threshold(&self, slug: &str) -> usize {
        self.thresholds.get(slug).copied().unwrap_or_default()
    }

    /// Whether the standard's depth floor applies to every request.
    #[must_use]
    pub fn is_unconditional(&self) -> bool {
        self.depth_floor == "unconditional"
    }
}

fn root_named<'a>(tree: &'a LinoNode, name: &str) -> Result<&'a LinoNode, ReasoningStandardError> {
    tree.children
        .iter()
        .find(|node| node.name == name)
        .ok_or_else(|| error(format!("standard_missing_root_{name}")))
}

/// Load and validate the formal standard.
pub fn standard() -> Result<ReasoningStandard, ReasoningStandardError> {
    let tree = parse_lino(REASONING_STANDARD);
    let root = root_named(&tree, "reasoning_standard")?;
    let mut thresholds = BTreeMap::new();
    for node in root.children.iter().filter(|node| node.name == "threshold") {
        let value = node
            .find_child_value("value")
            .parse::<usize>()
            .map_err(|_| error(format!("standard_threshold_invalid_{}", node.id)))?;
        thresholds.insert(node.id.clone(), value);
    }
    if thresholds.is_empty() {
        return Err(error("standard_thresholds_missing"));
    }
    let mut gates = Vec::new();
    for node in root.children.iter().filter(|node| node.name == "gate") {
        let order = node
            .find_child_value("order")
            .parse::<usize>()
            .map_err(|_| error(format!("standard_gate_order_invalid_{}", node.id)))?;
        let trigger = GateTrigger::from_slug(node.find_child_value("trigger"))
            .ok_or_else(|| error(format!("standard_gate_trigger_unknown_{}", node.id)))?;
        let requirement = node.find_child_value("requirement").to_owned();
        if requirement.trim().is_empty() {
            return Err(error(format!(
                "standard_gate_requirement_empty_{}",
                node.id
            )));
        }
        let failure_slug = node.find_child_value("failure_slug").to_owned();
        if failure_slug.trim().is_empty() {
            return Err(error(format!("standard_gate_failure_empty_{}", node.id)));
        }
        gates.push(Gate {
            slug: node.id.clone(),
            order,
            trigger,
            requirement,
            failure_slug,
        });
    }
    if gates.is_empty() {
        return Err(error("standard_gates_missing"));
    }
    gates.sort_by_key(|gate| gate.order);
    for (index, gate) in gates.iter().enumerate() {
        if gate.order != index + 1 {
            return Err(error(format!("standard_gate_order_gap_{}", gate.slug)));
        }
    }
    let depth_floor = root.find_child_value("depth_floor").to_owned();
    if depth_floor != "unconditional" {
        return Err(error("standard_depth_floor_conditional"));
    }
    Ok(ReasoningStandard {
        title: root.find_child_value("title").to_owned(),
        depth_floor,
        gates,
        thresholds,
    })
}

/// The reference episode distilled from the dialog issue #1073 points at.
///
/// It is the harness case the issue asks for: the audit must pass every gate on
/// it, and mutating it — dropping the after-measurement, smoothing the partial
/// failure into a success, narrowing the refutations to one axis — must fail the
/// matching gate.
#[must_use]
pub fn reference_episode() -> ReasoningEpisode {
    ReasoningEpisode::from_lino(REFERENCE_EPISODE)
}

/// Whether a gate applied, and if it did, whether it held.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GateStatus {
    /// The gate applied and held.
    Satisfied,
    /// The gate applied and was broken.
    Violated,
    /// The gate's trigger did not fire for this episode.
    NotTriggered,
}

impl GateStatus {
    /// Stable slug for the trace.
    #[must_use]
    pub const fn slug(self) -> &'static str {
        match self {
            Self::Satisfied => "satisfied",
            Self::Violated => "violated",
            Self::NotTriggered => "not_triggered",
        }
    }
}

/// One gate's result on one episode.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GateOutcome {
    /// The gate that ran.
    pub gate: String,
    /// Its position in the pipeline.
    pub order: usize,
    /// The trigger that decided whether it applied.
    pub trigger: GateTrigger,
    /// Whether it applied and held.
    pub status: GateStatus,
    /// What broke, by name. Empty when the gate held or did not apply.
    pub findings: Vec<String>,
}

/// The verdict of one audit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Verdict {
    /// Every triggered gate held and every refutation was itself refuted.
    Confirmed,
    /// Every triggered gate held and a refutation survived on evidence.
    Refuted,
    /// The honest default: something is unsettled, and the blockers say what.
    NotConfirmedNotRefuted {
        /// What stopped the check, by name.
        blockers: Vec<String>,
    },
}

impl Verdict {
    /// Stable slug for the trace.
    #[must_use]
    pub const fn slug(&self) -> &'static str {
        match self {
            Self::Confirmed => "confirmed",
            Self::Refuted => "refuted",
            Self::NotConfirmedNotRefuted { .. } => "not_confirmed_not_refuted",
        }
    }

    /// The blockers, empty unless the verdict is the honest default.
    #[must_use]
    pub fn blockers(&self) -> &[String] {
        match self {
            Self::NotConfirmedNotRefuted { blockers } => blockers,
            Self::Confirmed | Self::Refuted => &[],
        }
    }
}

/// The full result of auditing one episode against the standard.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReasoningAudit {
    /// The episode audited.
    pub episode_id: String,
    /// One outcome per declared gate, in pipeline order — always the full set.
    pub outcomes: Vec<GateOutcome>,
    /// The verdict the gates and the refutation ledgers reach together.
    pub verdict: Verdict,
    /// The instruction set compiled from whatever the episode gathered.
    pub instruction_set: InstructionSet,
}

impl ReasoningAudit {
    /// Whether every gate that applied held.
    #[must_use]
    pub fn all_triggered_gates_satisfied(&self) -> bool {
        !self
            .outcomes
            .iter()
            .any(|outcome| matches!(outcome.status, GateStatus::Violated))
    }

    /// One gate's outcome by slug.
    #[must_use]
    pub fn outcome(&self, gate: &str) -> Option<&GateOutcome> {
        self.outcomes.iter().find(|outcome| outcome.gate == gate)
    }

    /// Render the audit as Links Notation for the event log.
    #[must_use]
    pub fn to_links_notation(&self) -> String {
        let mut out = String::new();
        push_lino_node(
            &mut out,
            0,
            "reasoning_standard_audit",
            Some(&self.episode_id),
        );
        push_lino_node(&mut out, 2, "record_type", Some("reasoning_standard_audit"));
        push_lino_node(&mut out, 2, "verdict", Some(self.verdict.slug()));
        for outcome in &self.outcomes {
            push_lino_node(&mut out, 2, "gate", Some(&outcome.gate));
            push_lino_node(&mut out, 4, "order", Some(&outcome.order.to_string()));
            push_lino_node(&mut out, 4, "trigger", Some(outcome.trigger.slug()));
            push_lino_node(&mut out, 4, "status", Some(outcome.status.slug()));
            for finding in &outcome.findings {
                push_lino_node(&mut out, 4, "finding", Some(finding));
            }
        }
        for blocker in self.verdict.blockers() {
            push_lino_node(&mut out, 2, "blocker", Some(blocker));
        }
        out
    }
}

fn evaluate_gate(
    gate: &Gate,
    standard: &ReasoningStandard,
    episode: &ReasoningEpisode,
    set: &InstructionSet,
) -> Vec<String> {
    match gate.slug.as_str() {
        "evidence_before_claim" => check_evidence_before_claim(episode),
        "documentation_default" => check_documentation_default(standard, episode),
        "instruction_formalization" => check_instruction_formalization(standard, set),
        "computed_source_trust" => check_computed_source_trust(standard, episode),
        "refutation_variety" => check_refutation_variety(standard, episode),
        "verify_after_act" => check_verify_after_act(episode),
        "honest_failure_report" => check_honest_failure_report(episode),
        unknown => vec![format!("{unknown}:no_evaluator")],
    }
}

fn check_evidence_before_claim(episode: &ReasoningEpisode) -> Vec<String> {
    let mut findings = Vec::new();
    for claim in episode.claims.iter().filter(|claim| claim.about_world) {
        if claim.support.is_empty() {
            findings.push(format!("{}:no_supporting_observation", claim.id));
            continue;
        }
        for support in &claim.support {
            match episode.observation(support) {
                None => findings.push(format!("{}:unknown_observation:{support}", claim.id)),
                Some(observation) if observation.ordinal >= claim.ordinal => {
                    findings.push(format!(
                        "{}:observation_not_before_claim:{support}",
                        claim.id
                    ));
                }
                Some(observation) if observation.output.trim().is_empty() => {
                    findings.push(format!("{}:observation_without_output:{support}", claim.id));
                }
                Some(_) => {}
            }
        }
    }
    findings
}

fn check_documentation_default(
    standard: &ReasoningStandard,
    episode: &ReasoningEpisode,
) -> Vec<String> {
    let required = standard.threshold("minimum_documentation_sources");
    let primary = episode
        .sources
        .iter()
        .filter(|source| source.is_primary_for_subject())
        .count();
    if primary >= required {
        Vec::new()
    } else {
        vec![format!(
            "primary_sources_consulted:{primary}:required:{required}"
        )]
    }
}

fn check_instruction_formalization(
    standard: &ReasoningStandard,
    set: &InstructionSet,
) -> Vec<String> {
    let mut findings = Vec::new();
    let required = standard.threshold("minimum_instruction_sources");
    if set.steps.is_empty() {
        findings.push(format!("{}:no_instructions_gathered", set.task_class));
    }
    if set.source_count() < required {
        findings.push(format!(
            "instruction_sources:{}:required:{required}",
            set.source_count()
        ));
    }
    for step in set.unverifiable_steps() {
        findings.push(format!("{}:step_without_check", step.action));
    }
    findings
}

fn check_computed_source_trust(
    standard: &ReasoningStandard,
    episode: &ReasoningEpisode,
) -> Vec<String> {
    let minimum_steps = standard.threshold("minimum_primacy_steps");
    let mut findings = Vec::new();
    for source in &episode.sources {
        if source.chain.steps.len() < minimum_steps {
            findings.push(format!("{}:trust_asserted_without_chain", source.id));
            continue;
        }
        for (index, step) in source.chain.steps.iter().enumerate() {
            if !step.is_well_founded() {
                findings.push(format!("{}:primacy_step_{index}_unfounded", source.id));
            }
        }
        if source.assertion_disagrees() {
            findings.push(format!(
                "{}:asserted_tier_disagrees_with_derived:{}",
                source.id,
                source.derive_trust().tier.slug()
            ));
        }
    }
    findings
}

fn check_refutation_variety(
    standard: &ReasoningStandard,
    episode: &ReasoningEpisode,
) -> Vec<String> {
    let minimum_attempts = standard.threshold("minimum_refutation_attempts");
    let minimum_kinds = standard.threshold("minimum_refutation_axis_kinds");
    let mut findings = Vec::new();
    for conclusion in &episode.conclusions {
        let ledger = RefutationLedger::for_conclusion(&conclusion.id, &episode.probes);
        if !ledger.has_sufficient_variety(minimum_attempts, minimum_kinds) {
            findings.push(format!(
                "{}:variety:{}:kinds:{}:required:{minimum_attempts}:{minimum_kinds}",
                conclusion.id,
                ledger.variety(),
                ledger.axis_kinds()
            ));
        }
        if let LedgerState::Open { blockers } = ledger.state() {
            findings.extend(blockers);
        }
    }
    findings
}

fn check_verify_after_act(episode: &ReasoningEpisode) -> Vec<String> {
    let mut findings = Vec::new();
    for action in episode.actions.iter().filter(|action| action.mutating) {
        match &action.after {
            None => findings.push(format!("{}:no_measurement_after_action", action.id)),
            Some(measurement) => {
                if episode.observation(&measurement.observation).is_none() {
                    findings.push(format!(
                        "{}:measurement_without_observation:{}",
                        action.id, measurement.observation
                    ));
                }
                if measurement.value.trim().is_empty() {
                    findings.push(format!("{}:measurement_without_value", action.id));
                }
            }
        }
    }
    findings
}

fn check_honest_failure_report(episode: &ReasoningEpisode) -> Vec<String> {
    episode
        .actions
        .iter()
        .filter(|action| !matches!(action.outcome, ActionOutcome::Succeeded))
        .filter(|action| !action.is_honestly_reported())
        .map(|action| {
            format!(
                "{}:reported_{}_but_{}",
                action.id,
                action.reported_as.slug(),
                action.outcome.slug()
            )
        })
        .collect()
}

fn settle_verdict(
    standard: &ReasoningStandard,
    episode: &ReasoningEpisode,
    outcomes: &[GateOutcome],
) -> Verdict {
    let mut blockers: Vec<String> = outcomes
        .iter()
        .filter(|outcome| matches!(outcome.status, GateStatus::Violated))
        .flat_map(|outcome| {
            outcome
                .findings
                .iter()
                .map(move |finding| format!("{}:{finding}", outcome.gate))
        })
        .collect();
    if episode.conclusions.is_empty() {
        blockers.push(format!("{}:no_conclusion_recorded", episode.id));
        return Verdict::NotConfirmedNotRefuted { blockers };
    }
    let minimum_attempts = standard.threshold("minimum_refutation_attempts");
    let minimum_kinds = standard.threshold("minimum_refutation_axis_kinds");
    let mut alternative_proven = false;
    for conclusion in &episode.conclusions {
        let ledger = RefutationLedger::for_conclusion(&conclusion.id, &episode.probes);
        if !ledger.has_sufficient_variety(minimum_attempts, minimum_kinds) {
            blockers.push(format!("{}:refutation_variety_insufficient", conclusion.id));
        }
        match ledger.state() {
            LedgerState::Open { blockers: open } => blockers.extend(open),
            LedgerState::AlternativeProven { probe } => {
                alternative_proven = true;
                blockers.retain(|blocker| blocker != &probe);
            }
            LedgerState::Discharged => {}
        }
    }
    if !blockers.is_empty() {
        return Verdict::NotConfirmedNotRefuted { blockers };
    }
    if alternative_proven {
        Verdict::Refuted
    } else {
        Verdict::Confirmed
    }
}

/// Audit one episode against the standard.
///
/// Every declared gate is evaluated and reported, in pipeline order, whatever
/// the episode looks like: that enumeration *is* the unconditional depth floor.
#[must_use]
pub fn audit(standard: &ReasoningStandard, episode: &ReasoningEpisode) -> ReasoningAudit {
    let set = instructions::formalize(
        &episode.task_class,
        &episode.excerpts,
        standard.threshold("minimum_instruction_sources"),
    );
    let outcomes = standard
        .gates
        .iter()
        .map(|gate| {
            if gate.trigger.fires(episode) {
                let findings = evaluate_gate(gate, standard, episode, &set);
                let status = if findings.is_empty() {
                    GateStatus::Satisfied
                } else {
                    GateStatus::Violated
                };
                GateOutcome {
                    gate: gate.slug.clone(),
                    order: gate.order,
                    trigger: gate.trigger,
                    status,
                    findings,
                }
            } else {
                GateOutcome {
                    gate: gate.slug.clone(),
                    order: gate.order,
                    trigger: gate.trigger,
                    status: GateStatus::NotTriggered,
                    findings: Vec::new(),
                }
            }
        })
        .collect::<Vec<_>>();
    let verdict = settle_verdict(standard, episode, &outcomes);
    ReasoningAudit {
        episode_id: episode.id.clone(),
        outcomes,
        verdict,
        instruction_set: set,
    }
}

/// Open a reasoning episode for one request, before anything has been observed.
///
/// The meta core audits *every* request, so it needs an episode even at the seam
/// where no command has been run and no source has been read yet. The episode
/// opened here carries only the request's identity and task class; every gate
/// therefore reports [`GateStatus::NotTriggered`] naming the trigger that was
/// false. That empty-but-complete checklist is the point: the obligations are
/// enumerated on the trivial request exactly as they are on the hard one.
#[must_use]
pub fn open_episode(formalization: &IntentFormalization) -> ReasoningEpisode {
    ReasoningEpisode {
        id: formalization.impulse_id.clone(),
        task_class: formalization.kind.slug().to_owned(),
        ..ReasoningEpisode::default()
    }
}

/// Audit `episode` and append the result to the event log.
///
/// Trace-only (R13): the audit records what the standard found; it never changes
/// routing or the answer. A standard that fails to load is itself recorded rather
/// than swallowed, because a missing audit is exactly the silent skip the depth
/// floor forbids.
pub fn record_reasoning_standard(
    log: &mut EventLog,
    episode: &ReasoningEpisode,
) -> Option<ReasoningAudit> {
    match standard() {
        Err(problem) => {
            log.append("reasoning_standard:error", problem.reason);
            None
        }
        Ok(standard) => {
            let audit = audit(&standard, episode);
            log.append("reasoning_standard", audit.to_links_notation());
            log.append("reasoning_standard:gates", audit.outcomes.len().to_string());
            log.append("reasoning_standard:verdict", audit.verdict.slug());
            Some(audit)
        }
    }
}
