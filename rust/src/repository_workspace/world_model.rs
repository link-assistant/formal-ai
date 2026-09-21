//! Evidence-backed current-to-goal deltas for repository work items.
//!
//! The repository protocol observes effects. This layer compares those effects
//! with independently formalized requirements before completion can be claimed.
//! Case-specific paths, predicates and values arrive as Links Notation data.

use crate::engine::stable_id;
use crate::execution_evidence::{Evidence, EvidenceSource, ObservationKind};
use crate::links_format::format_lino_record;
use crate::needs::{Need, NeedKind, NeedState};
use crate::obligation_ledger::{
    ObligationExpectation, ObligationLedger, ObligationNode, ObligationOutcome,
};
use crate::seed::parser::{LinoNode, parse_lino};

const CASE: &str = "repository_case";
const REQUIREMENT: &str = "repository_requirement";
const OBSERVATION: &str = "repository_observation";
const ATTRIBUTION: &str = "repository_attribution";

/// One expected fact in the repository goal state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepositoryRequirement {
    /// Stable requirement identifier.
    pub requirement_id: String,
    /// Source of the requirement.
    pub origin: String,
    /// Verbatim requirement text.
    pub text: String,
    /// Entity whose state is constrained.
    pub subject: String,
    /// Property constrained on the subject.
    pub predicate: String,
    /// Finite accepted value set.
    pub accepted: Vec<String>,
    /// Existing universal need record for the required evidence.
    pub need: Need,
}

/// One observed fact in the repository current state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepositoryObservation {
    /// Stable observation identifier.
    pub observation_id: String,
    /// Entity whose state was observed.
    pub subject: String,
    /// Property observed on the subject.
    pub predicate: String,
    /// Normalized observed value.
    pub value: String,
    /// Evidence supporting the observation.
    pub evidence: Option<Evidence>,
}

/// Why a goal remains outside the observed current state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RepositoryGapKind {
    /// No fact was observed for the subject and predicate.
    MissingObservation,
    /// A fact exists but has no execution evidence.
    MissingEvidence,
    /// Evidence supports a value outside the accepted set.
    Mismatched,
}

impl RepositoryGapKind {
    const fn slug(self) -> &'static str {
        match self {
            Self::MissingObservation => "missing_observation",
            Self::MissingEvidence => "missing_evidence",
            Self::Mismatched => "mismatched",
        }
    }
}

/// One open current-to-goal delta.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepositoryGap {
    /// Requirement that remains open.
    pub requirement_id: String,
    /// Need that remains open.
    pub need_id: String,
    /// Why it remains open.
    pub kind: RepositoryGapKind,
    /// Accepted goal values.
    pub expected: Vec<String>,
    /// Current-state values, possibly empty.
    pub observed: Vec<String>,
}

/// Verdict about which external component boundary was reached.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttributionVerdict {
    /// The component owns an observed defect.
    Defect,
    /// The component produced positive evidence.
    PositiveEvidence,
    /// Execution stopped before reaching the component.
    NotReached,
    /// Corrupt downstream state was propagated without proving root ownership.
    DownstreamCorruption,
    /// No known verdict was declared.
    Unknown,
}

impl AttributionVerdict {
    fn parse(value: &str) -> Self {
        match value {
            "defect" => Self::Defect,
            "positive_evidence" => Self::PositiveEvidence,
            "not_reached" => Self::NotReached,
            "downstream_corruption" => Self::DownstreamCorruption,
            _ => Self::Unknown,
        }
    }

    const fn slug(self) -> &'static str {
        match self {
            Self::Defect => "defect",
            Self::PositiveEvidence => "positive_evidence",
            Self::NotReached => "not_reached",
            Self::DownstreamCorruption => "downstream_corruption",
            Self::Unknown => "unknown",
        }
    }
}

/// Evidence-based ownership statement for an external component.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepositoryAttribution {
    /// Component whose boundary was observed.
    pub component: String,
    /// Verdict supported by the observation.
    pub verdict: AttributionVerdict,
    /// Exact supporting detail.
    pub detail: String,
    /// Public external issue when one exists.
    pub external_issue: Option<String>,
}

/// Observed state, formalized goal state and the open delta between them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepositoryWorldModel {
    case_id: String,
    head: String,
    requirements: Vec<RepositoryRequirement>,
    observations: Vec<RepositoryObservation>,
    attributions: Vec<RepositoryAttribution>,
    gaps: Vec<RepositoryGap>,
    obligations: ObligationLedger,
}

impl RepositoryWorldModel {
    /// Build a world model from caller-supplied Links Notation.
    ///
    /// Requirements and observations join only by subject and predicate. Case
    /// names and accepted values never enter control flow.
    ///
    /// # Errors
    /// Returns a stable diagnostic slug for missing records or required fields.
    pub fn from_lino(document: &str, case_id: &str) -> Result<Self, String> {
        let root = parse_lino(document);
        let case = root
            .children
            .iter()
            .find(|node| record_type(node) == CASE && identifier(node) == case_id)
            .ok_or_else(|| format!("repository_case_missing:{case_id}"))?;
        let head = required(case, "head", case_id)?;
        let language = required(case, "language", case_id)?;
        let mut requirements = root
            .children
            .iter()
            .filter(|node| {
                record_type(node) == REQUIREMENT && node.find_child_value("case") == case_id
            })
            .map(|node| parse_requirement(node, case_id, &language))
            .collect::<Result<Vec<_>, _>>()?;
        if requirements.is_empty() {
            return Err(format!("repository_requirements_missing:{case_id}"));
        }
        let observations = root
            .children
            .iter()
            .filter(|node| {
                record_type(node) == OBSERVATION && node.find_child_value("case") == case_id
            })
            .map(|node| parse_observation(node, case_id, &head))
            .collect::<Result<Vec<_>, _>>()?;
        let attributions = root
            .children
            .iter()
            .filter(|node| {
                record_type(node) == ATTRIBUTION && node.find_child_value("case") == case_id
            })
            .map(|node| parse_attribution(node, case_id))
            .collect::<Result<Vec<_>, _>>()?;
        let frame_id = stable_id("repository_world_model", &format!("{case_id}:{head}"));
        let root_id = stable_id("repository_world_model_root", &frame_id);
        let mut children = Vec::with_capacity(requirements.len());
        let mut gaps = Vec::new();
        let mut cursor = 0_usize;

        for requirement in &mut requirements {
            let matching = observations
                .iter()
                .filter(|fact| {
                    fact.subject == requirement.subject && fact.predicate == requirement.predicate
                })
                .collect::<Vec<_>>();
            let proved = matching
                .iter()
                .find(|fact| requirement.accepted.contains(&fact.value) && fact.evidence.is_some());
            let observed = distinct_values(&matching);
            let outcome = if let Some(fact) = proved {
                let mut evidence = fact
                    .evidence
                    .clone()
                    .expect("the selected fact was evidence-bearing");
                evidence.for_need.clone_from(&requirement.need.need_id);
                evidence.produced_by = String::from("repository_world_model");
                requirement.need.state = NeedState::Satisfied;
                requirement.need.satisfied_by = Some(evidence.evidence_id.clone());
                ObligationOutcome::Satisfied { record: evidence }
            } else {
                let accepted_without_evidence = matching.iter().any(|fact| {
                    requirement.accepted.contains(&fact.value) && fact.evidence.is_none()
                });
                let kind = if matching.is_empty() {
                    RepositoryGapKind::MissingObservation
                } else if accepted_without_evidence
                    || matching.iter().all(|fact| fact.evidence.is_none())
                {
                    RepositoryGapKind::MissingEvidence
                } else {
                    RepositoryGapKind::Mismatched
                };
                gaps.push(RepositoryGap {
                    requirement_id: requirement.requirement_id.clone(),
                    need_id: requirement.need.need_id.clone(),
                    kind,
                    expected: requirement.accepted.clone(),
                    observed,
                });
                if kind == RepositoryGapKind::Mismatched {
                    let mut evidence = matching
                        .iter()
                        .find_map(|fact| fact.evidence.clone())
                        .expect("a witnessed mismatch has evidence");
                    evidence.for_need.clone_from(&requirement.need.need_id);
                    evidence.produced_by = String::from("repository_world_model");
                    ObligationOutcome::Refuted {
                        record: evidence,
                        mismatch: format!(
                            "{}!={}",
                            requirement.accepted.join("|"),
                            matching
                                .iter()
                                .map(|fact| fact.value.as_str())
                                .collect::<Vec<_>>()
                                .join("|")
                        ),
                    }
                } else {
                    ObligationOutcome::Unattempted
                }
            };
            let start = cursor;
            cursor = cursor.saturating_add(requirement.text.len());
            children.push(ObligationNode {
                node_id: stable_id(
                    "repository_requirement_obligation",
                    &format!("{frame_id}:{}", requirement.requirement_id),
                ),
                parent: Some(root_id.clone()),
                clause: requirement.text.clone(),
                span: (start, cursor),
                need_id: Some(requirement.need.need_id.clone()),
                depth: 1,
                expectation: ObligationExpectation::SymbolicCheck {
                    check_id: requirement.requirement_id.clone(),
                },
                outcome,
                children: Vec::new(),
            });
            cursor = cursor.saturating_add(1);
        }
        let obligations = ObligationLedger {
            frame_id,
            root: ObligationNode {
                node_id: root_id,
                parent: None,
                clause: case_id.to_owned(),
                span: (0, cursor.saturating_sub(1)),
                need_id: None,
                depth: 0,
                expectation: ObligationExpectation::Underivable {
                    reason: String::from("aggregate_goal"),
                },
                outcome: ObligationOutcome::Unattempted,
                children,
            },
        };
        Ok(Self {
            case_id: case_id.to_owned(),
            head,
            requirements,
            observations,
            attributions,
            gaps,
            obligations,
        })
    }

    /// Immutable repository head described by the observations.
    #[must_use]
    pub fn head(&self) -> &str {
        &self.head
    }

    /// Goal-state requirements in declaration order.
    #[must_use]
    pub fn requirements(&self) -> &[RepositoryRequirement] {
        &self.requirements
    }

    /// Current-state facts in declaration order.
    #[must_use]
    pub fn observations(&self) -> &[RepositoryObservation] {
        &self.observations
    }

    /// Component-boundary attribution records.
    #[must_use]
    pub fn attributions(&self) -> &[RepositoryAttribution] {
        &self.attributions
    }

    /// Missing, unevidenced or mismatched goals in requirement order.
    #[must_use]
    pub fn gaps(&self) -> &[RepositoryGap] {
        &self.gaps
    }

    /// Existing obligation ledger projected from the comparison.
    #[must_use]
    pub const fn obligation_ledger(&self) -> &ObligationLedger {
        &self.obligations
    }

    /// Requirement ids that reached evidence-backed satisfaction.
    #[must_use]
    pub fn satisfied_requirement_ids(&self) -> Vec<&str> {
        self.requirements
            .iter()
            .zip(&self.obligations.root.children)
            .filter(|(_, node)| matches!(node.outcome, ObligationOutcome::Satisfied { .. }))
            .map(|(requirement, _)| requirement.requirement_id.as_str())
            .collect()
    }

    /// Whether every goal is supported by matching evidence.
    #[must_use]
    pub fn completion_ready(&self) -> bool {
        !self.requirements.is_empty()
            && self.gaps.is_empty()
            && self.obligations.every_obligation_discharged()
            && self.obligations.satisfied_count() == self.requirements.len()
            && self.obligations.refuted_count() == 0
            && self.obligations.unattempted_count() == 0
            && self.obligations.unsatisfiable_count() == 0
    }

    /// Replayable projection of goal, current state, delta and proof.
    #[must_use]
    pub fn to_links_notation(&self) -> String {
        let mut out = format_lino_record(
            &stable_id(
                "repository_world_model",
                &format!("{}:{}", self.case_id, self.head),
            ),
            &[
                ("record_type", String::from("repository_world_model")),
                ("case", self.case_id.clone()),
                ("head", self.head.clone()),
                ("requirements", self.requirements.len().to_string()),
                ("observations", self.observations.len().to_string()),
                ("gaps", self.gaps.len().to_string()),
                ("completion_ready", self.completion_ready().to_string()),
            ],
        );
        for requirement in &self.requirements {
            out.push('\n');
            let mut pairs = vec![
                ("record_type", String::from("repository_goal_fact")),
                ("requirement_id", requirement.requirement_id.clone()),
                ("origin", requirement.origin.clone()),
                ("subject", requirement.subject.clone()),
                ("predicate", requirement.predicate.clone()),
                ("need_id", requirement.need.need_id.clone()),
            ];
            pairs.extend(
                requirement
                    .accepted
                    .iter()
                    .map(|value| ("expected", value.clone())),
            );
            out.push_str(&format_lino_record(&requirement.requirement_id, &pairs));
            out.push('\n');
            out.push_str(&requirement.need.to_links_notation());
        }
        for fact in &self.observations {
            out.push('\n');
            let mut pairs = vec![
                ("record_type", String::from("repository_current_fact")),
                ("observation_id", fact.observation_id.clone()),
                ("subject", fact.subject.clone()),
                ("predicate", fact.predicate.clone()),
                ("observed", fact.value.clone()),
            ];
            if let Some(evidence) = &fact.evidence {
                pairs.push(("evidence_id", evidence.evidence_id.clone()));
            }
            out.push_str(&format_lino_record(&fact.observation_id, &pairs));
            if let Some(evidence) = &fact.evidence {
                out.push('\n');
                out.push_str(&evidence.to_links_notation());
            }
        }
        for gap in &self.gaps {
            out.push('\n');
            let mut pairs = vec![
                ("record_type", String::from("repository_gap")),
                ("requirement_id", gap.requirement_id.clone()),
                ("need_id", gap.need_id.clone()),
                ("kind", gap.kind.slug().to_owned()),
            ];
            pairs.extend(gap.expected.iter().map(|value| ("expected", value.clone())));
            pairs.extend(gap.observed.iter().map(|value| ("observed", value.clone())));
            out.push_str(&format_lino_record(
                &stable_id(
                    "repository_gap",
                    &format!("{}:{}", self.case_id, gap.requirement_id),
                ),
                &pairs,
            ));
        }
        for attribution in &self.attributions {
            out.push('\n');
            let mut pairs = vec![
                ("record_type", String::from("repository_attribution")),
                ("component", attribution.component.clone()),
                ("verdict", attribution.verdict.slug().to_owned()),
                ("detail", attribution.detail.clone()),
            ];
            if let Some(issue) = &attribution.external_issue {
                pairs.push(("external_issue", issue.clone()));
            }
            out.push_str(&format_lino_record(
                &stable_id(
                    "repository_attribution",
                    &format!("{}:{}", self.case_id, attribution.component),
                ),
                &pairs,
            ));
        }
        out.push('\n');
        out.push_str(&self.obligations.to_links_notation());
        out
    }
}

fn parse_requirement(
    node: &LinoNode,
    case_id: &str,
    language: &str,
) -> Result<RepositoryRequirement, String> {
    let requirement_id = required_identifier(node, case_id)?;
    let origin = required(node, "origin", &requirement_id)?;
    let text = required(node, "text", &requirement_id)?;
    let subject = required(node, "subject", &requirement_id)?;
    let predicate = required(node, "predicate", &requirement_id)?;
    let accepted = node
        .children
        .iter()
        .filter(|child| child.name == "accepted")
        .map(|child| child.id.clone())
        .collect::<Vec<_>>();
    if accepted.is_empty() {
        return Err(format!(
            "repository_requirement_expected_missing:{requirement_id}"
        ));
    }
    let mut need = Need::raised(NeedKind::Evidence, &text, language, case_id);
    need.source_span = format!("{case_id}:{requirement_id}");
    Ok(RepositoryRequirement {
        requirement_id,
        origin,
        text,
        subject,
        predicate,
        accepted,
        need,
    })
}

fn parse_observation(
    node: &LinoNode,
    case_id: &str,
    head: &str,
) -> Result<RepositoryObservation, String> {
    let observation_id = required_identifier(node, case_id)?;
    let subject = required(node, "subject", &observation_id)?;
    let predicate = required(node, "predicate", &observation_id)?;
    let value = required(node, "value", &observation_id)?;
    let command = node.find_child_value("evidence_command");
    let source_url = node.find_child_value("evidence_url");
    let evidence = if command.is_empty() || source_url.is_empty() {
        None
    } else {
        let kind = match node.find_child_value("evidence_kind") {
            "command_exit" => ObservationKind::CommandExit,
            "file_bytes" => ObservationKind::FileBytes,
            "tool_result" => ObservationKind::ToolResult,
            _ => ObservationKind::SymbolicCheck,
        };
        let source = match node.find_child_value("evidence_source") {
            "local_process" => EvidenceSource::LocalProcess,
            "engine" => EvidenceSource::Engine,
            _ => EvidenceSource::Harness,
        };
        let exit_code = node.find_child_value("evidence_exit").parse::<i64>().ok();
        let observed = format_lino_record(
            &observation_id,
            &[
                ("subject", subject.clone()),
                ("predicate", predicate.clone()),
                ("value", value.clone()),
            ],
        );
        let mut evidence = Evidence::observed(
            command,
            vec![source_url.to_owned(), head.to_owned()],
            exit_code,
            observed.as_bytes(),
            kind,
            source,
        );
        evidence.source_ids = vec![stable_id(
            "repository_observation_source",
            &format!("{source_url}@{head}"),
        )];
        Some(evidence)
    };
    Ok(RepositoryObservation {
        observation_id,
        subject,
        predicate,
        value,
        evidence,
    })
}

fn parse_attribution(node: &LinoNode, case_id: &str) -> Result<RepositoryAttribution, String> {
    let component = required(node, "component", case_id)?;
    let verdict = AttributionVerdict::parse(node.find_child_value("verdict"));
    let detail = required(node, "detail", &component)?;
    Ok(RepositoryAttribution {
        component,
        verdict,
        detail,
        external_issue: optional(node.find_child_value("external_issue")),
    })
}

fn distinct_values(observations: &[&RepositoryObservation]) -> Vec<String> {
    let mut values = Vec::new();
    for observation in observations {
        if !values.contains(&observation.value) {
            values.push(observation.value.clone());
        }
    }
    values
}

fn record_type(node: &LinoNode) -> &str {
    node.find_child_value("record_type")
}

fn identifier(node: &LinoNode) -> &str {
    let field = node.find_child_value("id");
    if field.is_empty() { &node.id } else { field }
}

fn required_identifier(node: &LinoNode, owner: &str) -> Result<String, String> {
    let value = identifier(node);
    if value.is_empty() {
        Err(format!("repository_record_id_missing:{owner}"))
    } else {
        Ok(value.to_owned())
    }
}

fn required(node: &LinoNode, field: &str, owner: &str) -> Result<String, String> {
    let value = node.find_child_value(field);
    if value.is_empty() {
        Err(format!("repository_field_missing:{owner}:{field}"))
    } else {
        Ok(value.to_owned())
    }
}

fn optional(value: &str) -> Option<String> {
    (!value.trim().is_empty()).then(|| value.to_owned())
}
