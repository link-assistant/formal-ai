use super::*;
use crate::agentic_coding::general_execution::owned_creation_observation;
use crate::agentic_coding::general_planner::compose_general_change_plan;
use crate::agentic_coding::tool_result::complete_owned_source_read_frame;
use crate::protocol::ChatMessage;
use serde_json::json;

fn exchange(
    messages: &mut Vec<ChatMessage>,
    id: &str,
    tool: &str,
    arguments: String,
    receipt: Option<Value>,
) {
    messages.push(
        serde_json::from_value(json!({
            "role":"assistant", "content":"", "tool_calls":[{
                "id":id, "type":"function", "function":{"name":tool,"arguments":arguments}
            }]
        }))
        .unwrap(),
    );
    if let Some(mut receipt) = receipt {
        receipt["role"] = json!("tool");
        receipt["tool_call_id"] = json!(id);
        receipt["name"] = json!(tool);
        messages.push(serde_json::from_value(receipt).unwrap());
    }
}
fn setup() -> (
    crate::agentic_coding::general_planner::GeneralChangePlan,
    Vec<ChatMessage>,
    String,
) {
    let payload = "α🙂\nError: authored data\n";
    let task = format!("Create candidate.txt with the following exact content: «{payload}».");
    let plan = compose_general_change_plan(&task).unwrap();
    let raw = crate::protocol_responses::apply_patch_input(
        &json!({"path":"candidate.txt","content":payload}).to_string(),
    )
    .unwrap();
    (plan, vec![ChatMessage::user(task)], raw)
}
fn readback(messages: &mut Vec<ChatMessage>, content: &str) {
    exchange(
        messages,
        "readback",
        "read",
        json!({"path":"candidate.txt"}).to_string(),
        Some(json!({"content":complete_owned_source_read_frame(content)})),
    );
}
#[test]
fn canonical_creation_retains_edit_capability_and_actual_read_witness() {
    let (plan, mut messages, raw) = setup();
    exchange(
        &mut messages,
        "patch",
        "apply_patch",
        raw,
        Some(json!({"content":"Success"})),
    );
    readback(&mut messages, &plan.content);
    let progress = Progress::scan(&messages);
    let mutation = progress.creation_mutation_for(&plan.target).unwrap();
    assert_eq!(mutation.frame.capability(), Capability::Edit);
    assert!(mutation.qualified && mutation.succeeded);
    assert_eq!(mutation.content, plan.content);
    assert!(
        progress
            .successful_write_content_for(&plan.target)
            .is_none()
    );
    let proof = owned_creation_observation(&plan, &progress).unwrap();
    assert_eq!(proof.method, "read-tool");
    assert_eq!(proof.source, plan.content);
    assert_eq!(proof.command, None);
    assert_eq!(proof.exit_code, None);
}
#[test]
fn pending_failed_wrong_id_and_duplicate_patch_receipts_never_certify_creation() {
    for variant in ["pending", "failed", "wrong-id", "duplicate"] {
        let (plan, mut messages, raw) = setup();
        exchange(
            &mut messages,
            "patch",
            "apply_patch",
            raw,
            (variant != "pending").then(|| json!({"content":"Success"})),
        );
        if variant == "failed" {
            messages.last_mut().unwrap().is_error = true;
        }
        if variant == "wrong-id" {
            messages.last_mut().unwrap().tool_call_id = Some("foreign".to_owned());
        }
        if variant == "duplicate" {
            messages.push(messages.last().unwrap().clone());
        }
        readback(&mut messages, &plan.content);
        let progress = Progress::scan(&messages);
        assert!(
            !progress
                .creation_mutation_for(&plan.target)
                .unwrap()
                .succeeded,
            "{variant}"
        );
        assert!(
            owned_creation_observation(&plan, &progress).is_none(),
            "{variant}"
        );
    }
}
#[test]
fn incomplete_failed_mismatched_and_stale_readbacks_cannot_reuse_an_earlier_proof() {
    for variant in [
        "incomplete",
        "failed",
        "mismatch",
        "foreign",
        "pending-mutation",
    ] {
        let (plan, mut messages, raw) = setup();
        exchange(
            &mut messages,
            "patch",
            "apply_patch",
            raw,
            Some(json!({"content":"Success"})),
        );
        readback(&mut messages, &plan.content);
        if variant == "incomplete" || variant == "failed" {
            exchange(
                &mut messages,
                "latest-read",
                "read",
                json!({"path":plan.target}).to_string(),
                Some(
                    json!({"content":plan.content, "is_error":variant == "failed",
                    "source_read":{"path":plan.target,"format":"raw","success":variant != "failed","complete":false}}),
                ),
            );
        } else if variant == "mismatch" {
            exchange(
                &mut messages,
                "latest-read",
                "read",
                json!({"path":plan.target}).to_string(),
                Some(json!({"content":complete_owned_source_read_frame("foreign")})),
            );
        } else {
            exchange(
                &mut messages,
                "foreign-mutation",
                "edit",
                json!({"path":"other.txt","old_string":"a","new_string":"b"}).to_string(),
                (variant == "foreign").then(|| json!({"content":"Success"})),
            );
        }
        let progress = Progress::scan(&messages);
        assert!(
            owned_creation_observation(&plan, &progress).is_none(),
            "{variant}"
        );
    }
}
#[test]
fn exclusive_patch_preflight_requires_actual_absence_and_never_infers_it_from_text() {
    for variant in ["absent", "present", "opaque", "truncated"] {
        let (_, mut messages, _) = setup();
        let receipt = match variant {
            "absent" => json!({"content":"Error: missing", "is_error":true,
                "source_read":{"path":"candidate.txt","format":"raw","success":false,"complete":false,"error_code":"ENOENT"}}),
            "present" => json!({"content":complete_owned_source_read_frame("existing")}),
            "truncated" => json!({"content":"Error: missing", "is_error":true,
                "source_read":{"path":"candidate.txt","format":"raw","success":false,"complete":false,"error_code":"ENOENT","truncated":true}}),
            _ => json!({"content":"Error: ENOENT candidate.txt","is_error":true}),
        };
        exchange(
            &mut messages,
            "preflight",
            "read",
            json!({"path":"candidate.txt"}).to_string(),
            Some(receipt),
        );
        let progress = Progress::scan(&messages);
        let preflight = progress.creation_preflight_for("candidate.txt");
        assert!(preflight.attempted);
        assert_eq!(
            preflight
                .observation
                .is_some_and(|observed| observed.absent),
            variant == "absent"
        );
    }
}
#[test]
fn typed_error_looking_source_is_data_and_generic_edits_are_not_creation() {
    let (plan, mut messages, raw) = setup();
    exchange(
        &mut messages,
        "patch",
        "apply_patch",
        raw,
        Some(json!({"content":"Success"})),
    );
    readback(&mut messages, &plan.content);
    assert!(owned_creation_observation(&plan, &Progress::scan(&messages)).is_some());
    let (_, mut messages, _) = setup();
    exchange(
        &mut messages,
        "edit",
        "edit",
        json!({
            "path":"candidate.txt","old_string":"old","new_string":plan.content
        })
        .to_string(),
        Some(json!({"content":"Success"})),
    );
    readback(&mut messages, &plan.content);
    assert!(
        Progress::scan(&messages)
            .creation_mutation_for("candidate.txt")
            .is_none()
    );
}

#[test]
fn completed_invalidation_allows_one_observation_but_never_restores_creation_authority() {
    for variant in ["completed", "pending", "wrong-id"] {
        let (plan, mut messages, raw) = setup();
        exchange(
            &mut messages,
            "patch",
            "apply_patch",
            raw,
            Some(json!({"content":"Success"})),
        );
        exchange(
            &mut messages,
            "foreign-run",
            "bash",
            json!({"command":"cat other.txt"}).to_string(),
            (variant != "pending").then(|| json!({"content":"Output: unrelated\nExit Code: 0"})),
        );
        if variant == "wrong-id" {
            messages.last_mut().unwrap().tool_call_id = Some("unbound".to_owned());
        }
        {
            let progress = Progress::scan(&messages);
            let mutation = progress.creation_mutation_for(&plan.target).unwrap();
            let state = progress.creation_readback_for(
                &mutation,
                &super::super::source_freshness::owned_creation_readback_command(&plan.target),
            );
            assert!(!state.current, "{variant}");
            assert_eq!(state.needs_fresh_read, variant == "completed");
            assert!(owned_creation_observation(&plan, &progress).is_none());
        }
        if variant == "completed" {
            readback(&mut messages, &plan.content);
            let progress = Progress::scan(&messages);
            let mutation = progress.creation_mutation_for(&plan.target).unwrap();
            let state = progress.creation_readback_for(
                &mutation,
                &super::super::source_freshness::owned_creation_readback_command(&plan.target),
            );
            assert!(!state.current);
            assert!(!state.needs_fresh_read);
            assert!(owned_creation_observation(&plan, &progress).is_none());
        }
    }
}

#[test]
fn original_failed_write_receipts_remain_qualified_failures_with_the_actual_message() {
    use crate::agentic_coding::final_result::FinalDisposition;
    use crate::agentic_coding::general_execution::plan_general_change_step;
    use crate::agentic_coding::planner::AgenticPlan;
    for reason in [
        "Read the file before writing",
        "Write transport unavailable",
    ] {
        let (plan, mut messages, _) = setup();
        let raw = json!({"is_error":true,"error":reason}).to_string();
        exchange(
            &mut messages,
            "failed-write",
            "write",
            json!({"path":plan.target,"content":plan.content}).to_string(),
            Some(json!({"content":raw})),
        );
        let progress = Progress::scan(&messages);
        let mutation = progress.creation_mutation_for(&plan.target).unwrap();
        assert!(mutation.qualified);
        assert!(!mutation.succeeded);
        assert_eq!(
            mutation.attempt.unwrap().raw_receipt.as_deref(),
            Some(raw.as_str())
        );
        let mut result = None;
        let next = plan_general_change_step(&messages, &["write"], &plan, &mut result);
        assert_eq!(result.unwrap().disposition, FinalDisposition::Failure);
        let AgenticPlan::Final(answer) = next else {
            panic!("failed Write is terminal");
        };
        assert!(answer.contains(reason), "{answer}");
        assert!(!answer.contains("Completed the general change request"));
    }
}
#[test]
fn scoped_creation_proofs_cross_only_current_declared_disjoint_literal_creations() {
    fn assert_thread_safe<T: Send + Sync>() {}
    assert_thread_safe::<Progress>();
    let first_task = "Create a.txt containing «one\n».";
    let plan = compose_general_change_plan(first_task).unwrap();
    for variant in [
        "declared",
        "foreign",
        "wrong-content",
        "failed",
        "pending",
        "duplicate",
        "new-objective",
    ] {
        let request = if variant == "foreign" {
            first_task.to_owned()
        } else if variant == "duplicate" {
            format!("First {first_task} Then {first_task} Finally create b.txt containing «two\n».")
        } else {
            format!("First {first_task} Then create b.txt containing «two\n».")
        };
        let mut messages = vec![ChatMessage::user(request)];
        exchange(
            &mut messages,
            "a-write",
            "write",
            json!({"path":"a.txt","content":"one\n"}).to_string(),
            Some(json!({"content":"Success"})),
        );
        exchange(
            &mut messages,
            "a-read",
            "read",
            json!({"path":"a.txt"}).to_string(),
            Some(json!({"content":complete_owned_source_read_frame("one\n")})),
        );
        if variant == "new-objective" {
            messages.push(ChatMessage::user(
                "Create unrelated.txt containing «other».",
            ));
        }
        exchange(&mut messages, "b-write", "write",
            json!({"path":"b.txt","content":if variant == "wrong-content" {"foreign\n"} else {"two\n"}}).to_string(),
            (variant != "pending").then(|| json!({"content":"Success","is_error":variant == "failed"})));
        let progress = Progress::scan(&messages);
        assert_eq!(
            owned_creation_observation(&plan, &progress).is_some(),
            variant == "declared",
            "{variant}"
        );
        assert!(
            progress.fresh_source_read_for("a.txt").is_none(),
            "generic freshness stays conservative"
        );
    }
}
#[test]
fn immutable_standing_context_and_disjoint_path_guards_do_not_invent_alias_resolution() {
    let task = "First create a.txt containing «one\n». Then create b.txt containing «two\n».";
    let mut messages = vec![ChatMessage::user(task)];
    exchange(
        &mut messages,
        "a-write",
        "write",
        json!({"path":"a.txt","content":"one\n"}).to_string(),
        Some(json!({"content":"Success"})),
    );
    exchange(
        &mut messages,
        "a-read",
        "read",
        json!({"path":"a.txt"}).to_string(),
        Some(json!({"content":complete_owned_source_read_frame("one\n")})),
    );
    exchange(
        &mut messages,
        "b-write",
        "write",
        json!({"path":"b.txt","content":"two\n"}).to_string(),
        Some(json!({"content":"Success"})),
    );
    let progress = Progress::scan(&messages);
    messages[0] = ChatMessage::user("Create other.txt containing «different».");
    let plan = compose_general_change_plan("Create a.txt containing «one\n».").unwrap();
    assert!(owned_creation_observation(&plan, &progress).is_some());
    assert!(disjoint_creation_targets("a.txt", "b.txt"));
    for (left, right) in [
        ("a.txt", "a.txt"),
        ("a.txt", "A.txt"),
        ("a.txt", "./b.txt"),
        ("a.txt", "../b.txt"),
        ("a.txt", "/b.txt"),
        ("C:a.txt", "C:b.txt"),
        ("c:/a.txt", "c:/b.txt"),
        ("dir/a.txt", "dir\\a.txt"),
    ] {
        assert!(!disjoint_creation_targets(left, right), "{left} / {right}");
    }
    assert!(
        owned_creation_declarations(
            "First create a.txt containing «one\n». Then create a.txt containing «one\n»."
        )
        .is_empty()
    );
}
