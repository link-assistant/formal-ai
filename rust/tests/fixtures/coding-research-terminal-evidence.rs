//! Actual native production planner, driven by explicitly synthetic captured observations.
use super::super::final_result::{FinalDisposition, ResolvedPlan};
use super::super::planner::{AgenticPlan, plan_chat_step_resolved};
use crate::protocol::ChatMessage;
use serde_json::{Value, json};

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
        let observations = if let Some(research) = &result.research {
            assert_eq!(
                research.provenance.javascript_witness(),
                row["expected"]["research"],
                "{row}"
            );
            &research.observations
        } else {
            let discovery = result.discovery.as_ref().expect("honest discovery gap");
            assert_eq!(
                discovery.javascript_witness(),
                row["expected"]["result"]["discovery"],
                "{row}"
            );
            assert!(!discovery.authored && !discovery.verified);
            &discovery.observations
        };
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
