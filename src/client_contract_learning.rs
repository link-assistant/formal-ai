//! Human-gated learning for real-client verification contracts (issue #671).
//!
//! A matrix run records what a client actually advertised and invoked. Two
//! independently worded observations of the same capability are enough to
//! propose a stable contract amendment; they are never enough to apply it.
//! Proposals point back to their transcript evidence and remain
//! `awaiting_human_review`, matching the repository-wide learning guardrail.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::engine::stable_id;
use crate::seed::{self, ClientIntegration};

const PATH_PLACEHOLDER: &str = "{path}";
const LINE_PLACEHOLDER: &str = "{line}";
const ERROR_PLACEHOLDER: &str = "{error}";
const EXPECTED_MARKER_PLACEHOLDER: &str = "{expected_marker}";

/// How fixture bytes reached a client response.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeliveryMode {
    /// The client invoked one of its tools.
    ToolCall,
    /// The client supplied file bytes inside its model request.
    InBand,
}

/// One normalized fact derived from a real proxy transcript.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClientContractObservation {
    pub client_id: String,
    pub capability: String,
    pub task_wording: String,
    pub delivery: DeliveryMode,
    #[serde(default)]
    pub advertised_tools: Vec<String>,
    #[serde(default)]
    pub invoked_tools: Vec<String>,
    /// Additional observed contract fields whose stable intersection may be
    /// proposed for human review.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub observed_contract: BTreeMap<String, Vec<String>>,
    pub evidence: String,
}

impl ClientContractObservation {
    /// Construct an observation for tests and programmatic callers.
    #[must_use]
    pub fn new<I, S>(
        client_id: impl Into<String>,
        capability: impl Into<String>,
        task_wording: impl Into<String>,
        delivery: DeliveryMode,
        invoked_tools: I,
        evidence: impl Into<String>,
    ) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self {
            client_id: client_id.into(),
            capability: capability.into(),
            task_wording: task_wording.into(),
            delivery,
            advertised_tools: Vec::new(),
            invoked_tools: invoked_tools.into_iter().map(Into::into).collect(),
            observed_contract: BTreeMap::new(),
            evidence: evidence.into(),
        }
    }

    /// Render one observation as a compact JSON line.
    #[must_use]
    pub fn json_line(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| String::from("{}"))
    }
}

/// A contract amendment inferred from repeated observations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientContractProposal {
    pub id: String,
    pub client_id: String,
    pub capability: String,
    pub field: String,
    pub value: String,
    pub evidence: Vec<String>,
}

/// Comparison between repeated observations and the currently seeded contract.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientContractFinding {
    pub client_id: String,
    pub capability: String,
    pub wording_count: usize,
    pub observed_delivery: Option<DeliveryMode>,
    pub seeded_delivery: String,
    pub status: String,
    pub evidence: Vec<String>,
}

/// Auditable output of one contract-learning pass.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientContractLearningReport {
    pub observation_count: usize,
    pub independently_worded_groups: usize,
    pub findings: Vec<ClientContractFinding>,
    pub proposals: Vec<ClientContractProposal>,
    pub awaiting_human_review: bool,
}

impl ClientContractLearningReport {
    /// Render the review artifact in Links Notation.
    #[must_use]
    pub fn links_notation(&self) -> String {
        let mut out = String::from("client_contract_learning\n");
        field(&mut out, 1, "issue", "671");
        field(&mut out, 1, "human_gated", "true");
        field(
            &mut out,
            1,
            "decision",
            if self.awaiting_human_review {
                "awaiting_human_review"
            } else {
                "no_reviewable_change"
            },
        );
        let _ = writeln!(out, "  observation_count \"{}\"", self.observation_count);
        let _ = writeln!(
            out,
            "  independently_worded_groups \"{}\"",
            self.independently_worded_groups
        );
        let _ = writeln!(out, "  proposal_count \"{}\"", self.proposals.len());
        for finding in &self.findings {
            out.push_str("  finding\n");
            field(&mut out, 2, "client", &finding.client_id);
            field(&mut out, 2, "capability", &finding.capability);
            let _ = writeln!(out, "    wording_count \"{}\"", finding.wording_count);
            field(
                &mut out,
                2,
                "observed_delivery",
                finding
                    .observed_delivery
                    .map_or("inconsistent", DeliveryMode::slug),
            );
            field(&mut out, 2, "seeded_delivery", &finding.seeded_delivery);
            field(&mut out, 2, "status", &finding.status);
            for evidence in &finding.evidence {
                field(&mut out, 2, "evidence", evidence);
            }
        }
        for proposal in &self.proposals {
            out.push_str("  proposal\n");
            field(&mut out, 2, "id", &proposal.id);
            field(&mut out, 2, "client", &proposal.client_id);
            field(&mut out, 2, "capability", &proposal.capability);
            field(&mut out, 2, "field", &proposal.field);
            field(&mut out, 2, "value", &proposal.value);
            field(&mut out, 2, "target", "data/seed/client-integrations.lino");
            field(&mut out, 2, "decision", "awaiting_human_review");
            for evidence in &proposal.evidence {
                field(&mut out, 2, "evidence", evidence);
            }
        }
        out.trim_end().to_owned()
    }
}

/// Learn reusable constraints from independently worded real-client runs.
///
/// A tool becomes stable only when it was invoked in every observation in a
/// `(client, capability)` group and that group contains at least two distinct
/// task wordings. Already-seeded requirements are confirmations, not duplicate
/// proposals.
#[must_use]
pub fn learn_client_contracts(
    observations: &[ClientContractObservation],
    integrations: &[ClientIntegration],
) -> ClientContractLearningReport {
    let contracts: BTreeMap<&str, &ClientIntegration> = integrations
        .iter()
        .map(|integration| (integration.id.as_str(), integration))
        .collect();
    let mut groups: BTreeMap<(&str, &str), Vec<&ClientContractObservation>> = BTreeMap::new();
    for observation in observations {
        groups
            .entry((&observation.client_id, &observation.capability))
            .or_default()
            .push(observation);
    }

    let mut independently_worded_groups = 0;
    let mut findings = Vec::new();
    let mut proposals = Vec::new();
    for ((client_id, capability), group) in groups {
        let wordings: BTreeSet<String> = group
            .iter()
            .map(|observation| normalize(&observation.task_wording))
            .collect();
        if wordings.len() < 2 {
            continue;
        }
        independently_worded_groups += 1;

        let stable_tools = stable_invoked_tools(&group);
        let stable_contract = stable_observed_contract(&group);
        let accepted: BTreeSet<&str> = contracts
            .get(client_id)
            .map(|integration| {
                integration
                    .verification
                    .required_response_tools
                    .iter()
                    .map(String::as_str)
                    .collect()
            })
            .unwrap_or_default();
        let evidence = group
            .iter()
            .map(|observation| observation.evidence.clone())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        let observed_delivery = consistent_delivery(&group);
        let seeded_delivery = contracts
            .get(client_id)
            .map_or("", |integration| {
                integration.verification.file_delivery.as_str()
            })
            .to_owned();
        let status = match observed_delivery {
            Some(delivery) if seeded_delivery == delivery.slug() => "confirmed",
            Some(_) if !seeded_delivery.is_empty() => "contract_drift",
            _ => "unseeded",
        };
        findings.push(ClientContractFinding {
            client_id: client_id.to_owned(),
            capability: capability.to_owned(),
            wording_count: wordings.len(),
            observed_delivery,
            seeded_delivery: seeded_delivery.clone(),
            status: status.to_owned(),
            evidence: evidence.clone(),
        });
        if status == "contract_drift" {
            let delivery = observed_delivery.expect("contract drift has one stable delivery");
            let id = stable_id(
                "client_contract_proposal",
                &format!(
                    "{client_id}\0{capability}\0file_delivery\0{}",
                    delivery.slug()
                ),
            );
            proposals.push(ClientContractProposal {
                id,
                client_id: client_id.to_owned(),
                capability: capability.to_owned(),
                field: String::from("file_delivery"),
                value: delivery.slug().to_owned(),
                evidence: evidence.clone(),
            });
        }
        for tool in stable_tools {
            if accepted.contains(tool.as_str()) {
                continue;
            }
            let id = stable_id(
                "client_contract_proposal",
                &format!("{client_id}\0{capability}\0required_response_tool\0{tool}"),
            );
            proposals.push(ClientContractProposal {
                id,
                client_id: client_id.to_owned(),
                capability: capability.to_owned(),
                field: String::from("required_response_tool"),
                value: tool,
                evidence: evidence.clone(),
            });
        }
        for (contract_field, values) in stable_contract {
            for value in values {
                let id = stable_id(
                    "client_contract_proposal",
                    &format!("{client_id}\0{capability}\0{contract_field}\0{value}"),
                );
                proposals.push(ClientContractProposal {
                    id,
                    client_id: client_id.to_owned(),
                    capability: capability.to_owned(),
                    field: contract_field.clone(),
                    value,
                    evidence: evidence.clone(),
                });
            }
        }
    }

    ClientContractLearningReport {
        observation_count: observations.len(),
        independently_worded_groups,
        findings,
        awaiting_human_review: !proposals.is_empty(),
        proposals,
    }
}

impl DeliveryMode {
    const fn slug(self) -> &'static str {
        match self {
            Self::ToolCall => "tool_call",
            Self::InBand => "in_band",
        }
    }
}

fn consistent_delivery(group: &[&ClientContractObservation]) -> Option<DeliveryMode> {
    let first = group.first()?.delivery;
    group
        .iter()
        .all(|observation| observation.delivery == first)
        .then_some(first)
}

fn stable_invoked_tools(group: &[&ClientContractObservation]) -> BTreeSet<String> {
    let mut observations = group.iter();
    let Some(first) = observations.next() else {
        return BTreeSet::new();
    };
    let mut stable: BTreeSet<String> = first.invoked_tools.iter().cloned().collect();
    for observation in observations {
        let tools: BTreeSet<&str> = observation
            .invoked_tools
            .iter()
            .map(String::as_str)
            .collect();
        stable.retain(|tool| tools.contains(tool.as_str()));
    }
    stable
}

fn stable_observed_contract(
    group: &[&ClientContractObservation],
) -> BTreeMap<String, BTreeSet<String>> {
    let mut observations = group.iter();
    let Some(first) = observations.next() else {
        return BTreeMap::new();
    };
    let mut stable = normalized_contract(&first.observed_contract);
    for observation in observations {
        let contract = normalized_contract(&observation.observed_contract);
        stable.retain(|field, values| {
            let Some(observed_values) = contract.get(field) else {
                return false;
            };
            values.retain(|value| observed_values.contains(value));
            !values.is_empty()
        });
    }
    stable
}

fn normalized_contract(
    contract: &BTreeMap<String, Vec<String>>,
) -> BTreeMap<String, BTreeSet<String>> {
    contract
        .iter()
        .filter(|(field, _)| is_observed_contract_field(field))
        .filter_map(|(field, values)| {
            let values = values
                .iter()
                .map(|value| value.trim())
                .filter(|value| !value.is_empty())
                .map(ToOwned::to_owned)
                .collect::<BTreeSet<_>>();
            (!values.is_empty()).then(|| (field.clone(), values))
        })
        .collect()
}

fn is_observed_contract_field(field: &str) -> bool {
    let mut characters = field.chars();
    matches!(characters.next(), Some('a'..='z'))
        && characters.all(|character| {
            character.is_ascii_lowercase() || character.is_ascii_digit() || character == '_'
        })
        && !matches!(field, "file_delivery" | "required_response_tool")
}

/// Load observation JSONL documents.
pub fn load_observations(
    paths: &[impl AsRef<Path>],
) -> Result<Vec<ClientContractObservation>, String> {
    let mut observations = Vec::new();
    for path in paths {
        let path = path.as_ref();
        let text = fs::read_to_string(path).map_err(|error| {
            learning_message(
                "client_contract_read_observations_error",
                &[
                    (PATH_PLACEHOLDER, path.display().to_string()),
                    (ERROR_PLACEHOLDER, error.to_string()),
                ],
            )
        })?;
        for (index, line) in text.lines().enumerate() {
            if line.trim().is_empty() {
                continue;
            }
            observations.push(serde_json::from_str(line).map_err(|error| {
                learning_message(
                    "client_contract_parse_observation_error",
                    &[
                        (PATH_PLACEHOLDER, path.display().to_string()),
                        (LINE_PLACEHOLDER, (index + 1).to_string()),
                        (ERROR_PLACEHOLDER, error.to_string()),
                    ],
                )
            })?);
        }
    }
    Ok(observations)
}

#[derive(Deserialize)]
struct ProxyRow {
    #[serde(default)]
    request_tools: Vec<String>,
    #[serde(default)]
    response_tool_calls: Vec<ProxyToolCall>,
    #[serde(default)]
    response_content_preview: String,
}

#[derive(Deserialize)]
struct ProxyToolCall {
    name: String,
}

/// Derive one normalized observation from a recorded proxy JSONL transcript.
pub fn observe_proxy_transcript(
    transcript: &Path,
    client_id: &str,
    capability: &str,
    task_wording: &str,
    expected_marker: &str,
) -> Result<ClientContractObservation, String> {
    let text = fs::read_to_string(transcript).map_err(|error| {
        learning_message(
            "client_contract_read_transcript_error",
            &[
                (PATH_PLACEHOLDER, transcript.display().to_string()),
                (ERROR_PLACEHOLDER, error.to_string()),
            ],
        )
    })?;
    let mut advertised_tools = BTreeSet::new();
    let mut invoked_tools = BTreeSet::new();
    let mut marker_seen = expected_marker.is_empty();
    for (index, line) in text.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let row: ProxyRow = serde_json::from_str(line).map_err(|error| {
            learning_message(
                "client_contract_parse_transcript_error",
                &[
                    (PATH_PLACEHOLDER, transcript.display().to_string()),
                    (LINE_PLACEHOLDER, (index + 1).to_string()),
                    (ERROR_PLACEHOLDER, error.to_string()),
                ],
            )
        })?;
        advertised_tools.extend(row.request_tools);
        invoked_tools.extend(row.response_tool_calls.into_iter().map(|call| call.name));
        marker_seen |= row.response_content_preview.contains(expected_marker);
    }
    if invoked_tools.is_empty() && !marker_seen {
        return Err(learning_message(
            "client_contract_missing_evidence_error",
            &[
                (PATH_PLACEHOLDER, transcript.display().to_string()),
                (EXPECTED_MARKER_PLACEHOLDER, expected_marker.to_owned()),
            ],
        ));
    }
    let delivery = if invoked_tools.is_empty() {
        DeliveryMode::InBand
    } else {
        DeliveryMode::ToolCall
    };
    Ok(ClientContractObservation {
        client_id: client_id.to_owned(),
        capability: capability.to_owned(),
        task_wording: task_wording.to_owned(),
        delivery,
        advertised_tools: advertised_tools.into_iter().collect(),
        invoked_tools: invoked_tools.into_iter().collect(),
        observed_contract: BTreeMap::new(),
        evidence: transcript.to_string_lossy().replace('\\', "/"),
    })
}

fn learning_message(intent: &str, substitutions: &[(&str, String)]) -> String {
    let mut message = seed::response_for(intent, "en").unwrap_or_else(|| intent.replace('_', " "));
    for (placeholder, value) in substitutions {
        message = message.replace(placeholder, value);
    }
    message
}

fn normalize(value: &str) -> String {
    value
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

fn field(out: &mut String, depth: usize, key: &str, value: &str) {
    let _ = writeln!(out, "{}{key} \"{}\"", "  ".repeat(depth), quote(value));
}

fn quote(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "'")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
}
