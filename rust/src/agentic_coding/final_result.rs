//! Internal final-result provenance; public plans retain their text projection.

use super::planner::AgenticPlan;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum FinalDisposition {
    Finding,
    Clarification,
    Gap,
    Failure,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum FinalPayloadRole {
    Finding,
    AuditReport,
    SourceModule,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct SourceArtifact {
    pub(super) complete: bool,
    pub(super) content: String,
    pub(super) content_id: String,
    pub(super) syntax_content: String,
    pub(super) syntax_content_id: String,
    pub(super) syntax_exit_code: i32,
}

/// Observed source-binding witnesses retain their catalog payload without reinterpretation.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct ResearchBinding {
    pub(super) from: String,
    pub(super) to: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub(super) enum ResearchStatus {
    Proposal,
    Verified,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct ResearchProvenance {
    pub(super) url: String,
    pub(super) sha256: String,
    pub(super) query: String,
    pub(super) declaration: serde_json::Value,
    pub(super) contract: serde_json::Value,
    pub(super) bindings: Vec<ResearchBinding>,
    pub(super) status: ResearchStatus,
    pub(super) verified: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) source_authored: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) acceptance_sha256: Option<String>,
    #[serde(default, flatten)]
    pub(super) additional: std::collections::BTreeMap<String, serde_json::Value>,
}

impl ResearchProvenance {
    pub(super) fn from_observed(value: &serde_json::Value) -> Option<Self> {
        serde_json::from_value(value.clone()).ok()
    }

    #[cfg(test)]
    pub(super) fn javascript_witness(&self) -> serde_json::Value {
        serde_json::to_value(self).expect("ResearchWitnessSerializationInvariant")
    }

    /// The shared effect kernel already chose these flags; this only presents its verdict.
    pub(super) fn terminal(
        mut self,
        verified: bool,
        existing: bool,
        acceptance_sha256: String,
    ) -> Self {
        self.status = if verified {
            ResearchStatus::Verified
        } else {
            ResearchStatus::Proposal
        };
        self.verified = verified;
        if existing {
            self.source_authored = Some(false);
        }
        self.acceptance_sha256 = Some(acceptance_sha256);
        self
    }
}

/// Wire view of the existing source-read status, without inferring success.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
pub(super) enum ResearchReadStatus {
    ReportedSuccess,
    ReportedFailure,
    Unknown,
}

/// Missing observations stay optional rather than becoming successful default records.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct ResearchReadObservation {
    pub(super) path: Option<String>,
    pub(super) observed: bool,
    pub(super) status: Option<ResearchReadStatus>,
    pub(super) complete: Option<bool>,
    pub(super) absent: Option<bool>,
    pub(super) source_sha256: Option<String>,
    pub(super) error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub(super) struct ResearchVerificationObservation {
    pub(super) command: Option<String>,
    pub(super) receipt: Option<String>,
    pub(super) tool_succeeded: Option<bool>,
    pub(super) outcome: String,
    pub(super) receipt_gap: Option<String>,
    pub(super) decision: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub(super) struct ResearchObservations {
    pub(super) module_read: ResearchReadObservation,
    pub(super) test_read: ResearchReadObservation,
    pub(super) successful_write_sha256: Option<String>,
    pub(super) verification: Option<ResearchVerificationObservation>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub(super) struct ResearchFinalEvidence {
    pub(super) provenance: ResearchProvenance,
    pub(super) observations: ResearchObservations,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub(super) enum ResearchDiscoveryContext {
    None,
    Search { query: String, attempted: usize },
    Source(ResearchProvenance),
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub(super) struct ResearchDiscoveryEvidence {
    pub(super) reason: String,
    pub(super) authored: bool,
    pub(super) verified: bool,
    pub(super) path: Option<String>,
    pub(super) context: ResearchDiscoveryContext,
    pub(super) observations: ResearchObservations,
}

impl ResearchDiscoveryEvidence {
    /// This projects the existing JavaScript witness shape without changing native public plans.
    #[cfg(test)]
    pub(super) fn javascript_witness(&self) -> serde_json::Value {
        let mut value = serde_json::json!({
            "reason":self.reason, "authored":self.authored, "verified":self.verified,
        });
        let extra = match &self.context {
            ResearchDiscoveryContext::None => serde_json::json!({}),
            ResearchDiscoveryContext::Search { query, attempted } => {
                serde_json::json!({"query":query, "attempted":attempted})
            }
            ResearchDiscoveryContext::Source(provenance) => provenance.javascript_witness(),
        };
        value
            .as_object_mut()
            .expect("ResearchDiscoveryObjectInvariant")
            .extend(
                extra
                    .as_object()
                    .expect("ResearchDiscoveryContextInvariant")
                    .clone(),
            );
        if let Some(path) = &self.path {
            value["path"] = serde_json::json!(path);
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct FinalResult {
    pub(super) text: String,
    pub(super) disposition: FinalDisposition,
    pub(super) origin: String,
    pub(super) payload_role: FinalPayloadRole,
    pub(super) artifact: Option<SourceArtifact>,
    pub(super) research: Option<ResearchFinalEvidence>,
    pub(super) discovery: Option<ResearchDiscoveryEvidence>,
}

/// Fingerprints preserve evidence identity without emitting its source or receipt bodies.
fn evidence_fingerprint(value: &impl serde::Serialize) -> serde_json::Value {
    let observed = serde_json::to_value(value).expect("ResearchEvidenceSerializationInvariant");
    let bytes = observed
        .as_str()
        .map(|text| text.as_bytes().to_vec())
        .unwrap_or_else(|| {
            serde_json::to_vec(&observed).expect("ResearchEvidenceSerializationInvariant")
        });
    serde_json::json!({"sha256":crate::source_fetch::sha256_hex(&bytes),"bytes":bytes.len()})
}
fn read_diagnostic(read: &ResearchReadObservation) -> serde_json::Value {
    serde_json::json!({"observed":read.observed,"status":read.status,"complete":read.complete,
        "absent":read.absent,"sourceSha256":read.source_sha256,
        "path":read.path.as_ref().map(evidence_fingerprint),
        "error":read.error.as_ref().map(evidence_fingerprint)})
}
impl FinalResult {
    /// Opt-in evidence presentation is independent of delivery authority and public answer bytes.
    pub(super) fn terminal_evidence_diagnostic(&self) -> Option<serde_json::Value> {
        let observations = self
            .research
            .as_ref()
            .map(|value| &value.observations)
            .or_else(|| self.discovery.as_ref().map(|value| &value.observations))?;
        let verification = observations.verification.as_ref().map(|value| serde_json::json!({
            "command":value.command.as_ref().map(evidence_fingerprint),
            "receipt":value.receipt.as_ref().map(evidence_fingerprint),
            "toolSucceeded":value.tool_succeeded,
            "outcome":value.outcome.chars().take(32).collect::<String>(),
            "receiptGap":value.receipt_gap.as_ref().map(|gap| gap.chars().take(96).collect::<String>()),
            "decision":value.decision.chars().take(96).collect::<String>()}));
        Some(serde_json::json!({
            "evidence":evidence_fingerprint(&(&self.research,&self.discovery)),
            "research":self.research.as_ref().map(|value| serde_json::json!({
                "status":value.provenance.status,"verified":value.provenance.verified,
                "sourceSha256":value.provenance.sha256,"acceptanceSha256":value.provenance.acceptance_sha256,
                "sourceAuthored":value.provenance.source_authored,
                "provenance":evidence_fingerprint(&value.provenance)})),
            "discovery":self.discovery.as_ref().map(|value| serde_json::json!({
                "reason":value.reason.chars().take(96).collect::<String>(),
                "authored":value.authored,"verified":value.verified,
                "context":evidence_fingerprint(&value.context),"path":value.path.as_ref().map(evidence_fingerprint)})),
            "moduleRead":read_diagnostic(&observations.module_read),"testRead":read_diagnostic(&observations.test_read),
            "successfulWriteSha256":observations.successful_write_sha256,"verification":verification}))
    }
}

pub(super) struct ResolvedPlan {
    pub(super) plan: AgenticPlan,
    pub(super) result: Option<FinalResult>,
}

/// Observe final metadata without granting delivery authority to public text.
pub(super) fn observed_final_result(
    plan: &AgenticPlan,
    result: Option<FinalResult>,
) -> Option<FinalResult> {
    match plan {
        AgenticPlan::ToolCalls(_) => None,
        AgenticPlan::Final(text) => Some(
            result
                .filter(|result| result.text == *text)
                .unwrap_or_else(|| FinalResult {
                    text: text.clone(),
                    disposition: FinalDisposition::Unknown,
                    origin: String::new(),
                    payload_role: FinalPayloadRole::Finding,
                    artifact: None,
                    research: None,
                    discovery: None,
                }),
        ),
    }
}

impl ResolvedPlan {
    pub(super) fn new(plan: AgenticPlan, result: Option<FinalResult>) -> Self {
        let result = observed_final_result(&plan, result);
        if crate::cli_env::flag_enabled("FORMAL_AI_TRACE_REQUESTS")
            && let Some(diagnostic) = result
                .as_ref()
                .and_then(FinalResult::terminal_evidence_diagnostic)
        {
            super::planner::trace_route("coding_research_terminal", &diagnostic.to_string());
        }
        Self { plan, result }
    }

    pub(super) fn can_deliver(&self) -> bool {
        self.can_deliver_as(FinalPayloadRole::Finding)
    }

    pub(super) fn can_deliver_as(&self, required: FinalPayloadRole) -> bool {
        self.result.as_ref().is_some_and(|result| {
            result.disposition == FinalDisposition::Finding
                && !result.origin.is_empty()
                && (required != FinalPayloadRole::SourceModule
                    || (result.payload_role == FinalPayloadRole::SourceModule
                        && result.artifact.as_ref().is_some_and(|artifact| {
                            artifact.complete
                                && artifact.content == result.text
                                && !artifact.content_id.is_empty()
                                && artifact.syntax_exit_code == 0
                                && artifact.syntax_content == artifact.content
                                && artifact.syntax_content_id == artifact.content_id
                        })))
        })
    }

    pub(super) fn into_plan(self, result: &mut Option<FinalResult>) -> AgenticPlan {
        *result = self.result;
        self.plan
    }
}

/// Capture metadata only for the final plan actually selected by its route.
pub(super) fn record(
    plan: AgenticPlan,
    disposition: FinalDisposition,
    origin: &str,
    result: &mut Option<FinalResult>,
) -> AgenticPlan {
    record_with_role(plan, disposition, origin, FinalPayloadRole::Finding, result)
}

pub(super) fn record_with_role(
    plan: AgenticPlan,
    disposition: FinalDisposition,
    origin: &str,
    payload_role: FinalPayloadRole,
    result: &mut Option<FinalResult>,
) -> AgenticPlan {
    if let AgenticPlan::Final(text) = &plan {
        *result = Some(FinalResult {
            text: text.clone(),
            disposition,
            origin: origin.to_owned(),
            payload_role,
            artifact: None,
            research: None,
            discovery: None,
        });
    }
    plan
}
