#![cfg(test)]

use super::*;
#[test]
fn shared_seed_retains_original_subject_and_unconsumed_unicode_tail() {
    for source in [
        contract_text("original-request-1", &[]),
        contract_text("original-request-2", &[]),
        contract_text("original-request-3", &[]),
        contract_text("original-request-4", &[]),
        contract_text("original-request-5", &[]),
    ] {
        let contract = workspace_discovery_contract(&source).expect("shared grammar");
        assert_eq!(contract.source, source);
        assert_eq!(contract.subject, "task model");
        assert_eq!(contract.subject_span, [12, 22]);
        assert_eq!(contract.remaining_span, [22, source.encode_utf16().count()]);
    }
    assert!(utf16_boundary("😀", 1).is_none());
    assert_eq!(utf16_boundary("😀", 4), Some(2));
    assert!(workspace_discovery_contract("Inspect ../task_model").is_none());
    assert!(workspace_discovery_contract("node_path=1.1.1.1.1").is_none());
}
#[test]
fn structural_binding_rejects_foreign_source_and_inconsistent_spans() {
    let source = "Inspect the task model. Unconsumed α😀 tail";
    let fresh = || workspace_discovery_contract(source).expect("source grammar");
    assert!(workspace_discovery_contract_bound(&fresh(), source));
    let foreign_source = contract_text("original-request-1", &[]);
    assert!(!workspace_discovery_contract_bound(
        &fresh(),
        &foreign_source
    ));
    let mut contract = fresh();
    contract.source.push('x');
    assert!(!workspace_discovery_contract_bound(&contract, source));
    let mut contract = fresh();
    contract.subject = "queue model".to_owned();
    assert!(!workspace_discovery_contract_bound(&contract, source));
    let mut contract = fresh();
    contract.subject_span[0] += 1;
    assert!(!workspace_discovery_contract_bound(&contract, source));
    let mut contract = fresh();
    contract.subject_span.reverse();
    assert!(!workspace_discovery_contract_bound(&contract, source));
    let mut contract = fresh();
    contract.subject_span[1] = usize::MAX;
    assert!(!workspace_discovery_contract_bound(&contract, source));
    let mut contract = fresh();
    contract.remaining_span[0] += 1;
    assert!(!workspace_discovery_contract_bound(&contract, source));
    let mut contract = fresh();
    contract.remaining_span[1] -= 1;
    assert!(!workspace_discovery_contract_bound(&contract, source));
    let mut contract = fresh();
    contract.words[0] = "queue".to_owned();
    assert!(!workspace_discovery_contract_bound(&contract, source));
}

#[test]
fn partial_receipts_and_unsafe_candidate_paths_cannot_become_source_evidence() {
    for raw in [
        r#"{"command_output_complete":false}"#,
        r#"{"stdout_capture":{"complete":false}}"#,
        r#"{"truncated":true}"#,
    ] {
        assert!(incomplete(raw));
    }
    for path in [
        "../task_model",
        "task model",
        "a//task_model",
        "./task_model",
        "task_model;true",
    ] {
        assert!(!safe_path(path));
    }
}
