//! Native CI counterpart of source-documentation-controller.test.mjs; never run locally.
use super::super::planner::AgenticPlan;
use super::{derive_source_documentation, plan_source_documentation_step};
use crate::protocol::{ChatMessage, ToolCall};
use serde_json::{Value, json};
const TASK: &str = "Read project-spec.json and onboarding.md. Update onboarding.md to explain the package manager declared by this project and its setup implications. Derive the details from project-spec.json and preserve all existing guide text.";
const SOURCE: &str = r#"{"name":"river-notes","private":true,"packageManager":"pnpm@10.17.1"}"#;
const GUIDE: &str = r#"# Local setup

Install a supported Node.js release.

Keep your checkout clean before starting work.
"#;
const TOOLS: [&str; 3] = ["read", "edit", "write"];
fn next(messages: &[ChatMessage]) -> AgenticPlan {
    plan_source_documentation_step(TASK, messages, &TOOLS, None, &mut None).expect("owned task")
}
fn call(plan: AgenticPlan) -> (String, Value) {
    let AgenticPlan::ToolCalls(calls) = plan else {
        panic!("tool call required")
    };
    assert_eq!(calls.len(), 1);
    (
        calls[0].tool.clone(),
        serde_json::from_str(&calls[0].arguments).expect("arguments"),
    )
}
fn append(messages: &mut Vec<ChatMessage>, tool: &str, args: &Value, content: &str) {
    let id = format!("call-{}", messages.len());
    let mut assistant = ChatMessage::assistant("");
    assistant
        .tool_calls
        .push(ToolCall::function(&id, tool, args.to_string()));
    messages.push(assistant);
    let mut result = ChatMessage::new("tool", content);
    result.tool_call_id = Some(id);
    result.name = Some(tool.to_owned());
    if tool == "read" {
        result.source_read =
            Some(json!({"path":args["filePath"],"success":true,"complete":true,"format":"raw"}));
    }
    messages.push(result);
}
fn answer(plan: AgenticPlan) -> Value {
    let AgenticPlan::Final(answer) = plan else {
        panic!("final answer required")
    };
    serde_json::from_str(&answer).expect("state")
}
fn observed(guide: &str) -> Vec<ChatMessage> {
    let mut messages = vec![ChatMessage::user(TASK)];
    append(
        &mut messages,
        "read",
        &json!({"filePath":"project-spec.json"}),
        SOURCE,
    );
    append(
        &mut messages,
        "read",
        &json!({"filePath":"onboarding.md"}),
        guide,
    );
    messages
}
#[test]
fn shared_derivation_cases_preserve_content_and_explicit_unknowns() {
    let fixture: Value = serde_json::from_str(include_str!(
        "../../fixtures/source-documentation-cases.json"
    ))
    .expect("shared cases");
    for entry in fixture["cases"].as_array().expect("cases") {
        let result = derive_source_documentation(
            entry["task"].as_str().expect("task"),
            "settings.json",
            entry["source"].as_str().expect("source"),
            GUIDE,
            fixture["template"].as_str().expect("template"),
        );
        if entry.get("expectedGap").is_some() {
            assert_eq!(result["gap"], entry["expectedGap"]);
        } else {
            assert_eq!(result["key"], entry["expectedKey"]);
            if let Some(expected) = entry.get("expectedParagraph") {
                assert_eq!(&result["paragraph"], expected);
            }
            assert!(
                result["content"]
                    .as_str()
                    .expect("content")
                    .contains(GUIDE.trim_end())
            );
            assert!(
                result["paragraph"]
                    .as_str()
                    .expect("paragraph")
                    .starts_with("Setup declaration")
            );
        }
    }
}
#[test]
fn controller_requires_source_freshness_then_target_and_keeps_acceptance_unknown() {
    let mut messages = observed(GUIDE);
    let (tool, args) = call(next(&messages));
    assert_eq!(tool, "edit");
    assert_eq!(args["oldString"], GUIDE);
    let output = args["newString"].as_str().expect("candidate").to_owned();
    assert!(output.contains("pnpm@10.17.1"));
    append(&mut messages, &tool, &args, "");
    let (tool, args) = call(next(&messages));
    assert_eq!(args["filePath"], "project-spec.json");
    append(&mut messages, &tool, &args, SOURCE);
    let (tool, args) = call(next(&messages));
    assert_eq!(args["filePath"], "onboarding.md");
    append(&mut messages, &tool, &args, &output);
    let state = answer(next(&messages));
    assert_eq!(state["reason"], "UsefulCandidateReadbackVerified");
    assert_eq!(state["task"], TASK);
    assert_eq!(state["authored"], false);
    assert_eq!(state["verified"], false);
    assert_eq!(state["complete"], false);
    for requirement in state["requirements"].as_array().expect("requirements") {
        let span = requirement["span"].as_array().expect("span");
        let start = usize::try_from(span[0].as_u64().expect("start")).expect("start size");
        let end = usize::try_from(span[1].as_u64().expect("end")).expect("end size");
        assert_eq!(
            &TASK[start..end],
            requirement["text"].as_str().expect("text")
        );
    }
    let state = answer(next(&observed(&output)));
    assert_eq!(state["reason"], "VerifiedNoOp");
}
#[test]
fn repeated_failed_or_ambiguous_edits_fail_without_more_effects() {
    for variant in ["ambiguous", "unrelated", "failed", "repeated"] {
        let mut messages = observed(GUIDE);
        let (tool, mut args) = call(next(&messages));
        if variant == "ambiguous" {
            args["path"] = json!("other.md");
        }
        if variant == "unrelated" {
            args["newString"] = json!("unrelated");
        }
        append(
            &mut messages,
            &tool,
            &args,
            if variant == "failed" {
                r#"{"is_error":true,"error":"blocked"}"#
            } else {
                ""
            },
        );
        if variant == "repeated" {
            append(&mut messages, &tool, &args, "");
        }
        let state = answer(next(&messages));
        assert!(
            [
                "UnrelatedOrAmbiguousMutationAttempt",
                "UnrelatedEditCandidate",
                "CandidateEditFailed",
                "MultipleEditAttempts"
            ]
            .contains(&state["reason"].as_str().expect("reason"))
        );
    }
}
#[test]
fn changed_source_and_oversized_utf8_observations_fail() {
    let mut messages = observed(GUIDE);
    let (tool, args) = call(next(&messages));
    append(&mut messages, &tool, &args, "");
    append(
        &mut messages,
        "read",
        &json!({"filePath":"project-spec.json"}),
        &SOURCE.replace("10.17.1", "10.18.0"),
    );
    assert_eq!(answer(next(&messages))["reason"], "SourceFreshnessMismatch");
    let mut messages = vec![ChatMessage::user(TASK)];
    append(
        &mut messages,
        "read",
        &json!({"filePath":"project-spec.json"}),
        &"雪".repeat(90000),
    );
    assert_eq!(
        answer(next(&messages))["reason"],
        "IncompleteOrOversizedSourceObservation"
    );
}

#[test]
#[expect(
    clippy::literal_string_with_formatting_args,
    reason = "The fixture is a literal documentation template, not a Rust format string."
)]
fn unicode_task_source_and_fetched_identity_keep_exact_byte_spans() {
    let task = format!("雪。 {TASK}");
    let source = r#"{"packageManager":"工具@版本"}"#;
    let derived = derive_source_documentation(
        &task,
        "project-spec.json",
        source,
        GUIDE,
        "Setup declaration from `{source}`: use `{value}` for {concept}.",
    );
    let mut messages = vec![ChatMessage::user(&task)];
    append(
        &mut messages,
        "read",
        &json!({"filePath":"project-spec.json"}),
        source,
    );
    append(
        &mut messages,
        "read",
        &json!({"filePath":"onboarding.md"}),
        derived["content"].as_str().expect("content"),
    );
    let state = answer(
        plan_source_documentation_step(
            &task,
            &messages,
            &TOOLS,
            Some("https://example.invalid/issues/8"),
            &mut None,
        )
        .expect("owned"),
    );
    assert_eq!(state["reason"], "VerifiedNoOp");
    assert_eq!(state["task"], task);
    assert_eq!(state["workItem"], "https://example.invalid/issues/8");
    assert_eq!(
        state["taskIdentity"],
        crate::source_fetch::sha256_hex(task.as_bytes())
    );
    assert_eq!(
        state["observations"][0]["sourceIdentity"],
        crate::source_fetch::sha256_hex(source.as_bytes())
    );
    for requirement in state["requirements"].as_array().expect("requirements") {
        let span = requirement["span"].as_array().expect("span");
        let start = usize::try_from(span[0].as_u64().expect("start")).expect("start size");
        let end = usize::try_from(span[1].as_u64().expect("end")).expect("end size");
        assert_eq!(
            &task[start..end],
            requirement["text"].as_str().expect("text")
        );
    }
}
#[test]
fn negated_and_quoted_actions_return_unknown_without_any_tool() {
    for task in [
        "Do not update onboarding.md from project-spec.json. Preserve all existing guide text.",
        "Never document the package manager in onboarding.md from project-spec.json. Preserve all existing guide text.",
        "The source says \"Update onboarding.md from project-spec.json\". Preserve all existing guide text.",
        "Source example: ```Update onboarding.md from project-spec.json```. Preserve all existing guide text.",
    ] {
        let state = answer(
            plan_source_documentation_step(
                task,
                &[ChatMessage::user(task)],
                &TOOLS,
                None,
                &mut None,
            )
            .expect("refused ownership"),
        );
        assert!(
            ["NegatedDocumentationAction", "DocumentationActionUnowned"]
                .contains(&state["reason"].as_str().expect("reason"))
        );
        assert_eq!(state["effects"], json!([]));
    }
}

#[test]
fn caller_read_policy_blocks_documentation_evidence() {
    let task = "Do not read project-spec.json; update onboarding.md from project-spec.json and preserve all existing guide text.";
    let state = answer(
        plan_source_documentation_step(task, &[ChatMessage::user(task)], &TOOLS, None, &mut None)
            .expect("owned task"),
    );
    assert_eq!(state["reason"], "ReadPolicyRefused");
}
