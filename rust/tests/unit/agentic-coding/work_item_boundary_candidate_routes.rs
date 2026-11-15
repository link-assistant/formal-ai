//! CI-only native controller route over real client protocol receipts and files.
//! Loaded as a cfg(test) child of the helper to keep its production API private.
use super::*;
use crate::protocol::ToolCall;
use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

/// Issue #1560: a work-item final is a localized report, never a JSON object;
/// its record stays in the trace.
fn recorded(answer: &str) -> Value {
    assert!(
        !crate::agentic_coding::work_item_report::parses_as_json_object(answer),
        "{answer}"
    );
    crate::agentic_coding::work_item_report::reported_work_item_record(answer)
        .expect("a work-item final keeps its record")
}

struct Directory(PathBuf);
impl Drop for Directory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn directory() -> Directory {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let directory = std::env::temp_dir().join(format!(
        "formal-ai-native-boundary-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&directory).unwrap();
    Directory(directory)
}
fn paths(clause: &str) -> Vec<String> {
    let mut paths = Vec::new();
    for word in clause.split_whitespace() {
        let path = word.trim_matches(['`', '\'', '"', '(', ')', ',', '.', ';', ':']);
        if path.ends_with(".mjs") && !paths.iter().any(|value| value == path) {
            paths.push(path.to_owned());
        }
    }
    paths
}
fn session(directory: &Path, task: &str) -> (Value, Vec<String>) {
    session_with(directory, task, &["read", "edit"])
}
/// Drive the controller to its final; a `bash` call runs in `directory` through `sh -c`.
fn session_with(directory: &Path, task: &str, tools: &[&str]) -> (Value, Vec<String>) {
    let mut messages = vec![ChatMessage::user(task)];
    let mut calls = Vec::new();
    for turn in 0..8 {
        let mut final_result = None;
        let plan = plan_work_item_boundary_step(
            task,
            &messages,
            tools,
            Some("https://github.com/example/repository/issues/37"),
            paths,
            &mut final_result,
        )
        .expect("owned declared subset");
        match plan {
            AgenticPlan::Final(answer) => {
                let report = recorded(&answer);
                // Only a fully verified ledger is a finding; anything else stays a gap.
                let expected = if report["complete"] == true {
                    FinalDisposition::Finding
                } else {
                    FinalDisposition::Gap
                };
                assert_eq!(final_result.unwrap().disposition, expected);
                return (report, calls);
            }
            AgenticPlan::ToolCalls(planned) => {
                assert_eq!(planned.len(), 1);
                for call in planned {
                    let id = format!("native-boundary-{turn}");
                    let arguments: Value = serde_json::from_str(&call.arguments).unwrap();
                    messages.push(ChatMessage::assistant_tool_calls(vec![ToolCall::function(
                        &id,
                        &call.tool,
                        &call.arguments,
                    )]));
                    if call.tool == "bash" {
                        let output = std::process::Command::new("sh")
                            .arg("-c")
                            .arg(arguments["command"].as_str().unwrap())
                            .current_dir(directory)
                            .output()
                            .unwrap();
                        let mut text = String::from_utf8_lossy(&output.stdout).into_owned();
                        text.push_str(&String::from_utf8_lossy(&output.stderr));
                        let mut receipt = ChatMessage::tool_result(&id, &call.tool, text);
                        receipt.is_error = false;
                        messages.push(receipt);
                        calls.push(call.tool);
                        continue;
                    }
                    let path = arguments["filePath"]
                        .as_str()
                        .or_else(|| arguments["path"].as_str())
                        .unwrap();
                    assert_eq!(path, "src/ceiling.mjs");
                    let mut receipt = match call.tool.as_str() {
                        "read" => {
                            let source = fs::read_to_string(directory.join(path)).unwrap();
                            let mut receipt = ChatMessage::tool_result(&id, &call.tool, source);
                            receipt.source_read = Some(
                                json!({"path":path,"success":true,"complete":true,"format":"raw"}),
                            );
                            receipt
                        }
                        "edit" => {
                            let before = fs::read_to_string(directory.join(path)).unwrap();
                            assert_eq!(arguments["oldString"], before);
                            fs::write(
                                directory.join(path),
                                arguments["newString"].as_str().unwrap(),
                            )
                            .unwrap();
                            ChatMessage::tool_result(&id, &call.tool, "File edited successfully")
                        }
                        other => panic!("undeclared tool {other}"),
                    };
                    receipt.is_error = false;
                    messages.push(receipt);
                    calls.push(call.tool);
                }
            }
        }
    }
    panic!("finite controller budget exhausted");
}
#[test]
fn native_route_retains_gap_provenance_after_real_edit_and_verified_no_op() {
    let directory = directory();
    fs::create_dir(directory.0.join("src")).unwrap();
    let source = "// 雪: preserve this explanation.\nexport function accepts(value,minimum,maximum) {\n  return value >= minimum && value < maximum;\n}\nexport function untouched(value) { return value * 3; }\n";
    fs::write(directory.0.join("src/ceiling.mjs"), source).unwrap();
    let task = "Repair the upper-bound check in src/ceiling.mjs so the maximum is included. Preserve the lower bound, comments and other functions.";
    let (report, calls) = session(&directory.0, task);
    assert_eq!(calls, ["read", "edit", "read"]);
    assert_eq!(
        report["workItem"],
        "https://github.com/example/repository/issues/37"
    );
    assert_eq!(report["taskIdentity"], digest(task));
    assert_eq!(report["physicalReadbackVerified"], true);
    assert_eq!(report["authored"], true);
    assert_eq!(report["verified"], false);
    assert_eq!(report["complete"], false);
    assert!(!report["remainingWork"].as_array().unwrap().is_empty());
    for requirement in report["requirements"].as_array().unwrap() {
        let start = usize::try_from(requirement["span"][0].as_u64().unwrap())
            .expect("span start fits usize");
        let end =
            usize::try_from(requirement["span"][1].as_u64().unwrap()).expect("span end fits usize");
        assert_eq!(&task[start..end], requirement["text"].as_str().unwrap());
    }
    // Private oracle runs after the protocol interaction ends; no expected
    // source, final diff or grader result was supplied to the planner.
    let after = fs::read_to_string(directory.0.join("src/ceiling.mjs")).unwrap();
    assert_eq!(after, source.replace("value < maximum", "value <= maximum"));
    let (replay, replay_calls) = session(&directory.0, task);
    assert_eq!(replay_calls, ["read"]);
    assert_eq!(replay["noOp"], true);
    assert_eq!(replay["authored"], false);
    assert_eq!(
        fs::read_to_string(directory.0.join("src/ceiling.mjs")).unwrap(),
        after
    );
}

/// Issue #1483: with a Run tool the readback-verified artifact is executed
/// against samples derived from the requested bound, never from the candidate;
/// only that executed check verifies the requirement, and a replay is a
/// verified no-op.
#[test]
fn native_route_executes_the_contract_derived_check_and_replays_as_verified_no_op() {
    let directory = directory();
    fs::create_dir(directory.0.join("src")).unwrap();
    let source = "// Keep this comment.\nexport function accepts(value,minimum,maximum) {\n  return value >= minimum && value < maximum;\n}\nexport function untouched(value) { return value * 3; }\n";
    fs::write(directory.0.join("src/ceiling.mjs"), source).unwrap();
    let task = "Repair the upper-bound check in src/ceiling.mjs so the maximum is included. Preserve the lower bound, comments and other functions.";
    let tools = ["read", "edit", "bash"];
    let (report, calls) = session_with(&directory.0, task, &tools);
    assert_eq!(calls, ["read", "edit", "read", "bash"]);
    assert_eq!(report["reason"], "UsefulChangeVerified", "{report}");
    assert_eq!(report["verified"], true);
    assert_eq!(report["authored"], true);
    assert_eq!(report["complete"], true);
    let check = report["checks"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["kind"] == "contract-derived-execution")
        .expect("an executed contract check");
    assert_eq!(check["status"], "passed");
    assert_eq!(check["declaration"], "accepts");
    assert!(
        report["requirements"]
            .as_array()
            .unwrap()
            .iter()
            .all(|row| row["status"] == "verified")
    );
    assert_eq!(report["remainingWork"], json!([]));
    let (replay, replay_calls) = session_with(&directory.0, task, &tools);
    assert_eq!(replay_calls, ["read", "bash"]);
    assert_eq!(replay["reason"], "VerifiedNoOp");
    assert_eq!(replay["noOp"], true);
    assert_eq!(replay["verified"], true);
    assert_eq!(replay["authored"], false);
}

/// A regression test over a source that contradicts the requested bound fails its executed check.
#[test]
fn native_regression_check_fails_when_the_source_contradicts_the_bound() {
    let directory = directory();
    fs::create_dir(directory.0.join("src")).unwrap();
    fs::create_dir(directory.0.join("test")).unwrap();
    let source = "export function accepts(value,minimum,maximum) {\n  return value >= minimum && value < maximum;\n}\n";
    fs::write(directory.0.join("src/ceiling.mjs"), source).unwrap();
    let task = "Add a regression test in test/ceiling.test.mjs for the upper bound of function accepts in src/ceiling.mjs: the maximum is inclusive.";
    let mut messages = vec![ChatMessage::user(task)];
    let tools = ["read", "edit", "write", "bash"];
    for turn in 0..10 {
        let mut final_result = None;
        let plan =
            plan_work_item_boundary_step(task, &messages, &tools, None, paths, &mut final_result)
                .expect("owned declared subset");
        let AgenticPlan::ToolCalls(planned) = plan else {
            let AgenticPlan::Final(answer) = plan else {
                unreachable!()
            };
            let report = recorded(&answer);
            assert_eq!(report["reason"], "IndependentCheckFailed", "{report}");
            assert_eq!(report["verified"], false);
            assert_eq!(report["complete"], false);
            assert_eq!(final_result.unwrap().disposition, FinalDisposition::Gap);
            return;
        };
        for call in planned {
            let id = format!("native-regression-{turn}");
            let arguments: Value = serde_json::from_str(&call.arguments).unwrap();
            messages.push(ChatMessage::assistant_tool_calls(vec![ToolCall::function(
                &id,
                &call.tool,
                &call.arguments,
            )]));
            let path = arguments["filePath"]
                .as_str()
                .or_else(|| arguments["path"].as_str());
            let mut receipt = match (call.tool.as_str(), path) {
                ("bash", _) => {
                    let output = std::process::Command::new("sh")
                        .arg("-c")
                        .arg(arguments["command"].as_str().unwrap())
                        .current_dir(&directory.0)
                        .output()
                        .unwrap();
                    ChatMessage::tool_result(
                        &id,
                        &call.tool,
                        String::from_utf8_lossy(&output.stdout).into_owned(),
                    )
                }
                ("read", Some(path)) => match fs::read_to_string(directory.0.join(path)) {
                    Ok(text) => {
                        let mut receipt = ChatMessage::tool_result(&id, &call.tool, text);
                        receipt.source_read = Some(
                            json!({"path":path,"success":true,"complete":true,"format":"raw"}),
                        );
                        receipt
                    }
                    Err(error) => {
                        // The filesystem driver's shape: a transport failure plus ENOENT read metadata.
                        let mut receipt = ChatMessage::tool_result(
                            &id,
                            &call.tool,
                            json!({"is_error":true,"error":error.to_string()}).to_string(),
                        );
                        receipt.source_read = Some(json!({"path":path,"success":false,
                            "complete":false,"format":"raw","error_code":"ENOENT"}));
                        receipt.is_error = true;
                        messages.push(receipt);
                        continue;
                    }
                },
                ("write", Some(path)) => {
                    fs::write(
                        directory.0.join(path),
                        arguments["content"].as_str().unwrap(),
                    )
                    .unwrap();
                    ChatMessage::tool_result(&id, &call.tool, "")
                }
                (other, _) => panic!("undeclared tool {other}"),
            };
            receipt.is_error = false;
            messages.push(receipt);
        }
    }
    panic!("finite controller budget exhausted");
}

// Deliberately inconsistent receipts qualify terminal custody checks only; no
// physical mutation, independent task success or production repair is claimed.
fn guard_receipt(
    messages: &mut Vec<ChatMessage>,
    tool: &str,
    arguments_value: Value,
    content: &str,
    source_read: Option<Value>,
) {
    let id = format!("boundary-guard-{}", messages.len());
    messages.push(ChatMessage::assistant_tool_calls(vec![ToolCall::function(
        &id,
        tool,
        arguments_value.to_string(),
    )]));
    let mut receipt = ChatMessage::tool_result(&id, tool, content);
    receipt.source_read = source_read;
    messages.push(receipt);
}
#[test]
fn native_boundary_refuses_ambiguous_reads_and_all_unowned_mutations_before_filtering() {
    let task = "Repair the upper-bound check in src/ceiling.mjs so the maximum is included. Preserve the lower bound, comments and other functions.";
    let source = "export function accepts(value,minimum,maximum) { return value >= minimum && value < maximum; }";
    for (variant, expected) in [
        ("ambiguousRead", "BoundaryAmbiguousReadPathAttempt"),
        (
            "ambiguousEdit",
            "BoundaryUnrelatedOrAmbiguousMutationAttempt",
        ),
        (
            "unrelatedEdit",
            "BoundaryUnrelatedOrAmbiguousMutationAttempt",
        ),
        ("totalBudget", "BoundaryEffectAttemptBudgetExceeded"),
    ] {
        let mut messages = vec![ChatMessage::user(task)];
        guard_receipt(
            &mut messages,
            "read",
            json!({"filePath":"src/ceiling.mjs"}),
            source,
            Some(json!({"path":"src/ceiling.mjs","success":true,"complete":true,"format":"raw"})),
        );
        if variant == "ambiguousRead" {
            guard_receipt(
                &mut messages,
                "read",
                json!({"filePath":"src/ceiling.mjs","path":"other.mjs"}),
                source,
                Some(
                    json!({"path":"src/ceiling.mjs","success":true,"complete":true,"format":"raw"}),
                ),
            );
        } else {
            let mut arguments_value = json!({"filePath":if matches!(variant,"unrelatedEdit"|"totalBudget"){"other.mjs"}else{"src/ceiling.mjs"},"oldString":source,"newString":source});
            if variant == "ambiguousEdit" {
                arguments_value["path"] = json!("other.mjs");
            }
            guard_receipt(
                &mut messages,
                "edit",
                arguments_value,
                "File edited successfully",
                None,
            );
            if variant == "totalBudget" {
                guard_receipt(
                    &mut messages,
                    "edit",
                    json!({"filePath":"src/ceiling.mjs","oldString":source,"newString":source}),
                    "File edited successfully",
                    None,
                );
            }
        }
        let mut final_result = None;
        let plan = plan_work_item_boundary_step(
            task,
            &messages,
            &["read", "edit", "write"],
            None,
            paths,
            &mut final_result,
        )
        .unwrap();
        let AgenticPlan::Final(answer) = plan else {
            panic!("{variant}: another effect was planned");
        };
        let report = recorded(&answer);
        assert_eq!(report["reason"], expected, "{variant}");
        assert_eq!(report["authored"], false);
        assert_eq!(report["physicalReadbackVerified"], false);
        assert_eq!(report["complete"], false);
        assert_eq!(final_result.unwrap().disposition, FinalDisposition::Gap);
    }
}
