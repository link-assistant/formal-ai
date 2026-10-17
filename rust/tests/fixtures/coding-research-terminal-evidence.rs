//! Actual native production planner, driven by explicitly synthetic captured observations.
use super::super::final_result::{
    FinalDisposition, ResearchDiscoveryContext, ResearchDiscoveryEvidence, ResearchProvenance,
    ResolvedPlan,
};
use super::super::planner::{AgenticPlan, plan_chat_step_resolved};
use crate::protocol::ChatMessage;
use serde_json::{Value, json};

impl ResearchProvenance {
    pub(super) fn javascript_witness(&self) -> serde_json::Value {
        serde_json::to_value(self).expect("ResearchWitnessSerializationInvariant")
    }
}

impl ResearchDiscoveryEvidence {
    /// This projects the existing JavaScript witness shape without changing native public plans.
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

#[test]
fn selected_native_terminal_metadata_retains_observed_js_witnesses_and_receipts() {
    let rows = [
        include_str!("coding-research/terminal-evidence-1.json"),
        include_str!("coding-research/terminal-evidence-2.json"),
        include_str!("coding-research/terminal-evidence-3.json"),
    ]
    .iter()
    .flat_map(|source| {
        serde_json::from_str::<Value>(source).unwrap()["rows"]
            .as_array()
            .unwrap()
            .clone()
    })
    .collect::<Vec<_>>();
    let fixture = json!({"rows": rows});
    for row in fixture["rows"].as_array().unwrap() {
        let messages: Vec<ChatMessage> = serde_json::from_value(row["messages"].clone()).unwrap();
        let tools: Vec<&str> = row["tools"]
            .as_array()
            .unwrap()
            .iter()
            .map(|item| item.as_str().unwrap())
            .collect();
        let resolved = plan_chat_step_resolved(&messages, &tools).expect("actual production route");
        if row["phase"] == "tool" {
            assert!(matches!(resolved.plan, AgenticPlan::ToolCalls(_)));
            assert!(
                resolved.result.is_none(),
                "tool research side channel remains Pending"
            );
            continue;
        }
        let AgenticPlan::Final(text) = &resolved.plan else {
            panic!("expected terminal plan: {row}");
        };
        let result = resolved.result.as_ref().expect("typed terminal evidence");
        assert_eq!(
            &result.text, text,
            "metadata binds the actual selected public text"
        );
        assert_eq!(
            result.origin,
            row["expected"]["result"]["origin"].as_str().unwrap()
        );
        assert_eq!(
            format!("{:?}", result.disposition).to_lowercase(),
            row["expected"]["result"]["disposition"]
        );
        let diagnostic = result
            .terminal_evidence_diagnostic()
            .expect("retained terminal diagnostic");
        let encoded = diagnostic.to_string();
        assert!(encoded.len() < 4096, "bounded emitted diagnostic");
        assert_eq!(diagnostic["evidence"]["sha256"].as_str().unwrap().len(), 64);
        assert_eq!(
            diagnostic["moduleRead"]["observed"],
            row["observations"]["moduleRead"]["observed"]
        );
        assert_eq!(
            diagnostic["testRead"]["complete"],
            row["observations"]["testRead"]["complete"]
        );
        assert!(!encoded.contains("https://"), "no source URL emitted");
        assert!(!encoded.contains("# tests"), "no TAP receipt body emitted");
        if let Some(receipt) = row["observations"]["receipt"].as_str() {
            let serialized = receipt.as_bytes();
            assert_eq!(
                diagnostic["verification"]["receipt"]["sha256"],
                crate::source_fetch::sha256_hex(serialized)
            );
            assert_eq!(
                diagnostic["verification"]["receipt"]["bytes"],
                serialized.len()
            );
            assert!(!encoded.contains(receipt), "no raw tool receipt emitted");
        }
        let observations = result.research.as_ref().map_or_else(
            || {
                let discovery = result.discovery.as_ref().expect("honest discovery gap");
                assert_eq!(
                    without_source_observation(discovery.javascript_witness()),
                    row["expected"]["result"]["discovery"],
                    "{row}"
                );
                assert!(!discovery.authored && !discovery.verified);
                &discovery.observations
            },
            |research| {
                assert_eq!(
                    without_source_observation(research.provenance.javascript_witness()),
                    row["expected"]["research"],
                    "{row}"
                );
                &research.observations
            },
        );
        assert_eq!(
            serde_json::to_value(&observations.module_read).unwrap(),
            row["observations"]["moduleRead"]
        );
        assert_eq!(
            serde_json::to_value(&observations.test_read).unwrap(),
            row["observations"]["testRead"]
        );
        assert_eq!(
            json!(observations.successful_write_sha256),
            row["observations"]["successfulWriteSha256"]
        );
        if row["scenario"] == "module-incomplete" {
            assert!(observations.verification.is_none());
            assert_eq!(observations.module_read.complete, Some(false));
        } else {
            let verification = observations
                .verification
                .as_ref()
                .expect("actual observed verification attempt");
            assert_eq!(json!(verification.receipt), row["observations"]["receipt"]);
            assert_eq!(
                verification.tool_succeeded,
                Some(row["scenario"] != "failed")
            );
            if row["scenario"] == "incomplete" {
                assert!(verification.receipt_gap.is_some());
                assert_eq!(result.disposition, FinalDisposition::Gap);
            }
        }
        let mismatch = ResolvedPlan::new(
            AgenticPlan::Final("different selected final bytes".to_owned()),
            Some(result.clone()),
        );
        assert!(!mismatch.can_deliver());
        assert!(
            mismatch
                .result
                .as_ref()
                .unwrap()
                .terminal_evidence_diagnostic()
                .is_none()
        );
        assert!(mismatch.result.as_ref().unwrap().research.is_none());
        assert!(mismatch.result.as_ref().unwrap().discovery.is_none());
        let mut refusal = result.clone();
        refusal.disposition = FinalDisposition::Gap;
        assert!(!ResolvedPlan::new(resolved.plan.clone(), Some(refusal)).can_deliver());
        assert!(
            ResolvedPlan::new(AgenticPlan::ToolCalls(Vec::new()), Some(result.clone()))
                .result
                .is_none()
        );
    }
}

#[test]
fn malformed_producer_metadata_is_not_defaulted_to_a_valid_research_witness() {
    use super::super::final_result::ResearchProvenance;
    assert!(
        ResearchProvenance::from_observed(&json!({"url":"observed", "verified":true})).is_none()
    );
}

// Compare every prior witness field unchanged after checking the additive observation.
fn without_source_observation(mut value: Value) -> Value {
    if let Some(observation) = value
        .as_object_mut()
        .and_then(|fields| fields.remove("sourceObservation"))
    {
        assert_eq!(observation["schema"], "formal-ai-source-observation-v1");
        assert_eq!(observation["url"], value["url"]);
        assert_eq!(observation["sha256"], value["sha256"]);
        assert_eq!(observation["authority"], "Unknown");
        assert_eq!(observation["binding"]["status"], "TranscriptBound");
        assert_eq!(observation["binding"]["tool"], "web_fetch");
        assert!(
            observation["binding"]["callId"]
                .as_str()
                .is_some_and(|id| !id.is_empty())
        );
        for field in ["fetchedAt", "cached", "trust", "licence"] {
            assert_eq!(observation[field], json!({"status":"Unknown"}));
        }
        assert_eq!(observation.as_object().unwrap().len(), 9);
        assert_eq!(observation["binding"].as_object().unwrap().len(), 3);
    }
    value
}

#[test]
fn terminal_research_retains_call_bound_provider_reports_without_granting_authority() {
    let rows: Value =
        serde_json::from_str(include_str!("coding-research/terminal-evidence-1.json")).unwrap();
    let row = rows["rows"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["phase"] != "tool" && row["expected"]["research"]["verified"] == true)
        .unwrap();
    let mut messages: Vec<ChatMessage> = serde_json::from_value(row["messages"].clone()).unwrap();
    let index = messages
        .iter()
        .position(|message| message.role == "tool" && message.name.as_deref() == Some("web_fetch"))
        .unwrap();
    let id = messages[index].tool_call_id.clone().unwrap();
    let call = messages
        .iter()
        .flat_map(|message| &message.tool_calls)
        .find(|call| call.id == id)
        .unwrap();
    let args: Value = serde_json::from_str(&call.function.arguments).unwrap();
    messages[index].source_capture = Some(
        json!({"schema":"formal-ai-source-capture-v1", "callId":id,
        "url":args["url"], "sha256":crate::source_fetch::sha256_hex(messages[index].content.plain_text().as_bytes()),
        "fetchedAt":"1730000000", "cached":false, "trust":{"score":0,"policy":"fixture-policy"},
        "licence":{"name":"fixture-declaration","url":"https://licence.invalid/terms"}}),
    );
    let tools: Vec<&str> = row["tools"]
        .as_array()
        .unwrap()
        .iter()
        .map(|tool| tool.as_str().unwrap())
        .collect();
    let resolved = plan_chat_step_resolved(&messages, &tools).unwrap();
    let mut witness = resolved
        .result
        .unwrap()
        .research
        .unwrap()
        .provenance
        .javascript_witness();
    let observation = witness
        .as_object_mut()
        .unwrap()
        .remove("sourceObservation")
        .unwrap();
    assert_eq!(observation["binding"]["callId"], id);
    assert_eq!(observation["authority"], "Unknown");
    assert_eq!(
        observation["fetchedAt"],
        json!({"status":"ProviderReported","value":"1730000000"})
    );
    assert_eq!(
        observation["cached"],
        json!({"status":"ProviderReported","value":false})
    );
    assert_eq!(
        observation["trust"],
        json!({"status":"ProviderReported","value":{"score":0,"policy":"fixture-policy"}})
    );
    assert_eq!(
        observation["licence"],
        json!({"status":"ProviderReported","value":{"name":"fixture-declaration","url":"https://licence.invalid/terms"}})
    );
    assert_eq!(witness, row["expected"]["research"]);
}
