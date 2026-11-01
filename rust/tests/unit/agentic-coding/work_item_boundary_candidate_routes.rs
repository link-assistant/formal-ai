//! CI-only native controller route over real client protocol receipts and files.
//! Loaded as a cfg(test) child of the helper to keep its production API private.
use super::*;
use crate::protocol::ToolCall;
use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

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
    let mut messages = vec![ChatMessage::user(task)];
    let mut calls = Vec::new();
    for turn in 0..8 {
        let mut final_result = None;
        let plan = plan_work_item_boundary_step(
            task,
            &messages,
            &["read", "edit"],
            Some("https://github.com/example/repository/issues/37"),
            paths,
            &mut final_result,
        )
        .expect("owned declared subset");
        match plan {
            AgenticPlan::Final(answer) => {
                assert_eq!(final_result.unwrap().disposition, FinalDisposition::Gap);
                return (serde_json::from_str(&answer).unwrap(), calls);
            }
            AgenticPlan::ToolCalls(planned) => {
                assert_eq!(planned.len(), 1);
                for call in planned {
                    let id = format!("native-boundary-{turn}");
                    let arguments: Value = serde_json::from_str(&call.arguments).unwrap();
                    let path = arguments["filePath"]
                        .as_str()
                        .or_else(|| arguments["path"].as_str())
                        .unwrap();
                    assert_eq!(path, "src/ceiling.mjs");
                    messages.push(ChatMessage::assistant_tool_calls(vec![ToolCall::function(
                        &id,
                        &call.tool,
                        &call.arguments,
                    )]));
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
