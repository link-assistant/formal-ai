//! The reasoning episode: the record one audit runs over.
//!
//! An episode is everything a reasoning pass did that the standard can check —
//! the observations it executed, the claims it made about the world, the sources
//! it weighed, the instructions it gathered, the refutations it attempted, the
//! actions it took and what it measured afterwards. It is deliberately a *record
//! of what happened*, not a plan: every gate in
//! [`crate::reasoning_standard`] is a pure predicate over this structure, so the
//! same verdict is reachable by replaying the episode with no model in the loop.
//!
//! Episodes are Links Notation both ways ([`ReasoningEpisode::from_lino`] /
//! [`ReasoningEpisode::to_links_notation`]), so a captured session is a data
//! file the harness can replay and a runtime episode is a trace the event log
//! can carry.

use crate::links_format::push_lino_node;
use crate::relative_meta_logic::SourceTier;
use crate::seed::parser::{LinoNode, parse_lino};

use super::instructions::{SourceExcerpt, SourceStep};
use super::refutation::{ProbeOutcome, RefutationAxis, RefutationProbe};
use super::trust::{SourceAssessment, chain_from_node};

/// Something that was executed or retrieved, together with what was seen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Observation {
    /// Stable id referenced by claims, probes, and measurements.
    pub id: String,
    /// Position in the episode; claims may only rest on smaller ordinals.
    pub ordinal: u32,
    /// The command run or the document retrieved.
    pub command: String,
    /// What was actually seen back.
    pub output: String,
    /// Where the observation came from (`local_shell`, a host, a registry id).
    pub source: String,
    /// Instruction checks this observation discharges.
    pub checks: Vec<String>,
}

/// An assertion the episode makes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Claim {
    /// Stable id.
    pub id: String,
    /// Position in the episode.
    pub ordinal: u32,
    /// What is asserted.
    pub statement: String,
    /// Whether the claim is about the world (and so needs evidence) rather than
    /// about the episode's own intentions.
    pub about_world: bool,
    /// Observation ids the claim rests on.
    pub support: Vec<String>,
}

/// A number read off the world before or after an action.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Measurement {
    /// What was measured.
    pub metric: String,
    /// The value read.
    pub value: String,
    /// The observation that produced it.
    pub observation: String,
}

/// How an action actually turned out.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionOutcome {
    /// Everything the action set out to do happened.
    Succeeded,
    /// Some of it happened and some did not.
    PartiallySucceeded,
    /// None of it happened.
    Failed,
}

impl ActionOutcome {
    /// Stable slug for the data files and the trace.
    #[must_use]
    pub const fn slug(self) -> &'static str {
        match self {
            Self::Succeeded => "succeeded",
            Self::PartiallySucceeded => "partially_succeeded",
            Self::Failed => "failed",
        }
    }

    /// Parse a slug; anything unrecognized is [`Self::Failed`], so an
    /// unreadable outcome is never mistaken for success.
    #[must_use]
    pub fn from_slug(slug: &str) -> Self {
        match slug {
            "succeeded" => Self::Succeeded,
            "partially_succeeded" => Self::PartiallySucceeded,
            _ => Self::Failed,
        }
    }

    /// The honest report for this outcome.
    #[must_use]
    pub const fn honest_report(self) -> Self {
        self
    }
}

/// One thing the episode did to the world.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Action {
    /// Stable id.
    pub id: String,
    /// Position in the episode.
    pub ordinal: u32,
    /// Whether the action changed the world (as opposed to only reading it).
    pub mutating: bool,
    /// What was done.
    pub description: String,
    /// What actually happened.
    pub outcome: ActionOutcome,
    /// How the episode reported it. Reporting a partial result as a success is
    /// exactly what the honesty gate refuses.
    pub reported_as: ActionOutcome,
    /// Why it failed or only partly succeeded.
    pub reason: String,
    /// The measurement taken before the action, when one was taken.
    pub before: Option<Measurement>,
    /// The measurement taken after the action.
    pub after: Option<Measurement>,
}

impl Action {
    /// Whether the report matches what happened.
    #[must_use]
    pub fn is_honestly_reported(&self) -> bool {
        if self.reported_as != self.outcome {
            return false;
        }
        matches!(self.outcome, ActionOutcome::Succeeded) || !self.reason.trim().is_empty()
    }

    /// Whether the effect was re-measured afterwards.
    #[must_use]
    pub const fn is_verified(&self) -> bool {
        self.after.is_some()
    }
}

/// Something the episode wants to conclude.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Conclusion {
    /// Stable id, referenced by refutation probes.
    pub id: String,
    /// What the episode would conclude.
    pub statement: String,
    /// The claim the conclusion rests on.
    pub claim: String,
}

/// One reasoning pass, as a record the standard can audit.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ReasoningEpisode {
    /// Stable id of the episode.
    pub id: String,
    /// The *class* of task, not the single request: what instructions were
    /// gathered for. Empty when the request needs no procedure.
    pub task_class: String,
    /// Everything executed or retrieved, in record order.
    pub observations: Vec<Observation>,
    /// Everything asserted, in record order.
    pub claims: Vec<Claim>,
    /// Every source weighed, with its primacy chain.
    pub sources: Vec<SourceAssessment>,
    /// Instructions gathered per source for the task class.
    pub excerpts: Vec<SourceExcerpt>,
    /// Every refutation attempted.
    pub probes: Vec<RefutationProbe>,
    /// Everything done to the world.
    pub actions: Vec<Action>,
    /// Everything the episode would conclude.
    pub conclusions: Vec<Conclusion>,
}

impl ReasoningEpisode {
    /// Look one observation up by id.
    #[must_use]
    pub fn observation(&self, id: &str) -> Option<&Observation> {
        self.observations
            .iter()
            .find(|observation| observation.id == id)
    }

    /// Every instruction check any observation discharged.
    #[must_use]
    pub fn discharged_checks(&self) -> Vec<String> {
        let mut checks: Vec<String> = Vec::new();
        for observation in &self.observations {
            for check in &observation.checks {
                if !checks.contains(check) {
                    checks.push(check.clone());
                }
            }
        }
        checks
    }

    /// Parse an episode from Links Notation.
    #[must_use]
    pub fn from_lino(text: &str) -> Self {
        let tree = parse_lino(text);
        tree.children
            .iter()
            .find(|node| node.name == "reasoning_episode")
            .map_or_else(Self::default, Self::from_node)
    }

    fn from_node(node: &LinoNode) -> Self {
        Self {
            id: node.id.clone(),
            task_class: node.find_child_value("task_class").to_owned(),
            observations: collect(node, "observation", observation_from_node),
            claims: collect(node, "claim", claim_from_node),
            sources: collect(node, "source_assessment", source_from_node),
            excerpts: collect(node, "instruction_excerpt", excerpt_from_node),
            probes: collect(node, "refutation", probe_from_node),
            actions: collect(node, "action", action_from_node),
            conclusions: collect(node, "conclusion", conclusion_from_node),
        }
    }

    /// Render the episode back to Links Notation.
    #[must_use]
    pub fn to_links_notation(&self) -> String {
        let mut out = String::new();
        push_lino_node(&mut out, 0, "reasoning_episode", Some(&self.id));
        push_lino_node(&mut out, 2, "record_type", Some("reasoning_episode"));
        push_lino_node(&mut out, 2, "task_class", Some(&self.task_class));
        for observation in &self.observations {
            push_lino_node(&mut out, 2, "observation", Some(&observation.id));
            push_lino_node(
                &mut out,
                4,
                "ordinal",
                Some(&observation.ordinal.to_string()),
            );
            push_lino_node(&mut out, 4, "command", Some(&observation.command));
            push_lino_node(&mut out, 4, "output", Some(&observation.output));
            push_lino_node(&mut out, 4, "source", Some(&observation.source));
            for check in &observation.checks {
                push_lino_node(&mut out, 4, "check", Some(check));
            }
        }
        for claim in &self.claims {
            push_lino_node(&mut out, 2, "claim", Some(&claim.id));
            push_lino_node(&mut out, 4, "ordinal", Some(&claim.ordinal.to_string()));
            push_lino_node(&mut out, 4, "statement", Some(&claim.statement));
            push_lino_node(
                &mut out,
                4,
                "about_world",
                Some(if claim.about_world { "true" } else { "false" }),
            );
            for support in &claim.support {
                push_lino_node(&mut out, 4, "support", Some(support));
            }
        }
        for source in &self.sources {
            push_lino_node(&mut out, 2, "source_assessment", Some(&source.id));
            push_lino_node(&mut out, 4, "label", Some(&source.label));
            push_lino_node(&mut out, 4, "subject", Some(&source.subject));
            if let Some(tier) = source.asserted_tier {
                push_lino_node(&mut out, 4, "asserted_tier", Some(tier.slug()));
            }
            for step in &source.chain.steps {
                push_lino_node(&mut out, 4, "primacy", Some(step.kind.slug()));
                push_lino_node(&mut out, 6, "upstream", Some(&step.upstream));
                push_lino_node(&mut out, 6, "basis", Some(&step.basis));
            }
        }
        for excerpt in &self.excerpts {
            push_lino_node(&mut out, 2, "instruction_excerpt", Some(&excerpt.source_id));
            for step in &excerpt.steps {
                push_lino_node(&mut out, 4, "step", Some(&step.action));
                push_lino_node(&mut out, 6, "check", Some(&step.check));
            }
        }
        for probe in &self.probes {
            push_lino_node(&mut out, 2, "refutation", Some(&probe.id));
            push_lino_node(&mut out, 4, "conclusion", Some(&probe.conclusion));
            push_lino_node(&mut out, 4, "axis", Some(probe.axis.slug()));
            push_lino_node(&mut out, 4, "mechanism", Some(&probe.mechanism));
            push_lino_node(&mut out, 4, "denies", Some(&probe.denies));
            push_lino_node(&mut out, 4, "outcome", Some(probe.outcome.slug()));
            for evidence in &probe.evidence {
                push_lino_node(&mut out, 4, "evidence", Some(evidence));
            }
            push_lino_node(&mut out, 4, "blocker", Some(&probe.blocker));
        }
        for action in &self.actions {
            push_lino_node(&mut out, 2, "action", Some(&action.id));
            push_lino_node(&mut out, 4, "ordinal", Some(&action.ordinal.to_string()));
            push_lino_node(
                &mut out,
                4,
                "mutating",
                Some(if action.mutating { "true" } else { "false" }),
            );
            push_lino_node(&mut out, 4, "description", Some(&action.description));
            push_lino_node(&mut out, 4, "outcome", Some(action.outcome.slug()));
            push_lino_node(&mut out, 4, "reported_as", Some(action.reported_as.slug()));
            push_lino_node(&mut out, 4, "reason", Some(&action.reason));
            push_measurement(&mut out, "before_measurement", action.before.as_ref());
            push_measurement(&mut out, "after_measurement", action.after.as_ref());
        }
        for conclusion in &self.conclusions {
            push_lino_node(&mut out, 2, "conclusion", Some(&conclusion.id));
            push_lino_node(&mut out, 4, "statement", Some(&conclusion.statement));
            push_lino_node(&mut out, 4, "claim", Some(&conclusion.claim));
        }
        out
    }
}

fn push_measurement(out: &mut String, name: &str, measurement: Option<&Measurement>) {
    let Some(measurement) = measurement else {
        return;
    };
    push_lino_node(out, 4, name, None);
    push_lino_node(out, 6, "metric", Some(&measurement.metric));
    push_lino_node(out, 6, "value", Some(&measurement.value));
    push_lino_node(out, 6, "observation", Some(&measurement.observation));
}

fn collect<T>(node: &LinoNode, name: &str, build: fn(&LinoNode) -> T) -> Vec<T> {
    node.children
        .iter()
        .filter(|child| child.name == name)
        .map(build)
        .collect()
}

fn values(node: &LinoNode, name: &str) -> Vec<String> {
    node.children
        .iter()
        .filter(|child| child.name == name && !child.id.is_empty())
        .map(|child| child.id.clone())
        .collect()
}

fn ordinal(node: &LinoNode) -> u32 {
    node.find_child_value("ordinal").parse().unwrap_or(0)
}

fn flag(node: &LinoNode, name: &str, default: bool) -> bool {
    match node.find_child_value(name) {
        "true" => true,
        "false" => false,
        _ => default,
    }
}

fn observation_from_node(node: &LinoNode) -> Observation {
    Observation {
        id: node.id.clone(),
        ordinal: ordinal(node),
        command: node.find_child_value("command").to_owned(),
        output: node.find_child_value("output").to_owned(),
        source: node.find_child_value("source").to_owned(),
        checks: values(node, "check"),
    }
}

fn claim_from_node(node: &LinoNode) -> Claim {
    Claim {
        id: node.id.clone(),
        ordinal: ordinal(node),
        statement: node.find_child_value("statement").to_owned(),
        about_world: flag(node, "about_world", true),
        support: values(node, "support"),
    }
}

fn source_from_node(node: &LinoNode) -> SourceAssessment {
    SourceAssessment {
        id: node.id.clone(),
        label: node.find_child_value("label").to_owned(),
        subject: node.find_child_value("subject").to_owned(),
        chain: chain_from_node(node),
        asserted_tier: tier_from_slug(node.find_child_value("asserted_tier")),
    }
}

/// Parse a tier slug. An unrecognized or absent slug is *no assertion at all*
/// rather than a default tier, so nothing is trusted by omission.
#[must_use]
pub fn tier_from_slug(slug: &str) -> Option<SourceTier> {
    match slug {
        "original_first_party" => Some(SourceTier::OriginalFirstParty),
        "original_journalism" => Some(SourceTier::OriginalJournalism),
        "independent_corroboration" => Some(SourceTier::IndependentCorroboration),
        "unoriginal" => Some(SourceTier::Unoriginal),
        _ => None,
    }
}

fn excerpt_from_node(node: &LinoNode) -> SourceExcerpt {
    SourceExcerpt::new(
        node.id.clone(),
        node.children
            .iter()
            .filter(|child| child.name == "step")
            .map(|child| SourceStep::new(child.id.clone(), child.find_child_value("check")))
            .collect(),
    )
}

fn probe_from_node(node: &LinoNode) -> RefutationProbe {
    RefutationProbe {
        id: node.id.clone(),
        conclusion: node.find_child_value("conclusion").to_owned(),
        axis: RefutationAxis::from_slug(node.find_child_value("axis"))
            .unwrap_or(RefutationAxis::Assumption),
        mechanism: node.find_child_value("mechanism").to_owned(),
        denies: node.find_child_value("denies").to_owned(),
        outcome: ProbeOutcome::from_slug(node.find_child_value("outcome")),
        evidence: values(node, "evidence"),
        blocker: node.find_child_value("blocker").to_owned(),
    }
}

fn measurement_from_node(node: &LinoNode, name: &str) -> Option<Measurement> {
    node.children
        .iter()
        .find(|child| child.name == name)
        .map(|child| Measurement {
            metric: child.find_child_value("metric").to_owned(),
            value: child.find_child_value("value").to_owned(),
            observation: child.find_child_value("observation").to_owned(),
        })
}

fn action_from_node(node: &LinoNode) -> Action {
    let outcome = ActionOutcome::from_slug(node.find_child_value("outcome"));
    let reported = node.find_child_value("reported_as");
    Action {
        id: node.id.clone(),
        ordinal: ordinal(node),
        mutating: flag(node, "mutating", true),
        description: node.find_child_value("description").to_owned(),
        outcome,
        reported_as: if reported.is_empty() {
            outcome
        } else {
            ActionOutcome::from_slug(reported)
        },
        reason: node.find_child_value("reason").to_owned(),
        before: measurement_from_node(node, "before_measurement"),
        after: measurement_from_node(node, "after_measurement"),
    }
}

fn conclusion_from_node(node: &LinoNode) -> Conclusion {
    Conclusion {
        id: node.id.clone(),
        statement: node.find_child_value("statement").to_owned(),
        claim: node.find_child_value("claim").to_owned(),
    }
}
