use formal_ai::agentic_coding::{AgenticPlan, plan_chat_step};
use formal_ai::protocol::{ChatMessage, ToolCall};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
const ORIGINAL: &str = "First, create file a.txt with exactly this content «alpha». Second, create file b.txt with exactly this content «beta».";
static SERIAL: AtomicU64 = AtomicU64::new(0);
struct Run {
    root: PathBuf,
    receipts: Vec<(String, String)>,
    writes: usize,
    answer: Option<String>,
}
impl Drop for Run {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}
fn run(task: &str, tools: &[&str]) -> Run {
    let root = std::env::temp_dir().join(format!(
        "formal-ai-obligation-{}-{}",
        std::process::id(),
        SERIAL.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir_all(&root).expect("fixture directory");
    let mut run = Run {
        root,
        receipts: Vec::new(),
        writes: 0,
        answer: None,
    };
    let mut messages = vec![ChatMessage::user(task)];
    for turn in 0..8 {
        match plan_chat_step(&messages, tools) {
            Some(AgenticPlan::ToolCalls(calls)) => {
                for (index, call) in calls.into_iter().enumerate() {
                    let args: serde_json::Value =
                        serde_json::from_str(&call.arguments).expect("arguments");
                    let result = if call.tool == "write" {
                        fs::write(
                            run.root.join(args["path"].as_str().expect("target")),
                            args["content"].as_str().expect("bytes"),
                        )
                        .expect("target write");
                        run.writes += 1;
                        String::new()
                    } else if call.tool == "bash" {
                        let command = args["command"].as_str().expect("command");
                        let output = Command::new("bash")
                            .args(["-c", command])
                            .current_dir(&run.root)
                            .output()
                            .expect("bounded fixture shell");
                        let stdout = String::from_utf8(output.stdout).expect("UTF8 stdout");
                        let result = format!(
                            "Output: {stdout}\nExit Code: {}",
                            output.status.code().unwrap_or(1)
                        );
                        if command.starts_with("cat ") {
                            run.receipts.push((command.to_owned(), stdout));
                        }
                        result
                    } else {
                        panic!("unsupported fixture primitive {}", call.tool);
                    };
                    let identity = format!("step-{turn}-{index}");
                    messages.push(ChatMessage::assistant_tool_calls(vec![ToolCall::function(
                        &identity,
                        &call.tool,
                        call.arguments,
                    )]));
                    messages.push(ChatMessage::tool_result(identity, call.tool, result));
                }
            }
            Some(AgenticPlan::Final(answer)) => {
                run.answer = Some(answer);
                break;
            }
            None => break,
        }
    }
    run
}
fn exact_pair(run: &Run, first: &str, second: &str) {
    assert_eq!(
        fs::read_to_string(run.root.join("a.txt")).expect("first file"),
        first
    );
    assert_eq!(
        fs::read_to_string(run.root.join("b.txt")).expect("second file"),
        second
    );
    assert!(
        run.receipts
            .contains(&("cat a.txt".to_owned(), first.to_owned()))
    );
    assert!(
        run.receipts
            .contains(&("cat b.txt".to_owned(), second.to_owned()))
    );
    assert_eq!(run.writes, 2);
    assert!(run.answer.is_some());
}
#[test]
fn unchanged_original_delivers_both_bytes_and_independent_receipts() {
    exact_pair(&run(ORIGINAL, &["bash", "write"]), "alpha", "beta");
}
#[test]
fn keywords_inside_every_literal_obligation_remain_bytes() {
    let quote = char::from(96);
    let first = format!("When {quote}input{quote} then {quote}output{quote}");
    let second = "rename a.txt to gone.txt and replace alpha with wrong";
    let request = format!(
        "First, create file a.txt with exactly this content «{first}». Second, create file b.txt with exactly this content «{second}»."
    );
    let outcome = run(&request, &["bash", "write"]);
    exact_pair(&outcome, &first, second);
    assert!(!outcome.root.join("gone.txt").exists());
}
#[test]
fn write_only_target_is_retained_without_fabricating_second_artifact() {
    let outcome = run(ORIGINAL, &["write"]);
    assert_eq!(
        fs::read_to_string(outcome.root.join("a.txt")).expect("first target"),
        "alpha"
    );
    assert!(!outcome.root.join("b.txt").exists());
    assert_eq!(outcome.writes, 1);
    assert!(
        !outcome
            .answer
            .as_deref()
            .expect("honest gap")
            .contains("Completed the general change request")
    );
}
#[test]
fn mandatory_unknown_clause_is_a_gap_after_first_verified_artifact() {
    let outcome = run(
        "First, create file a.txt with exactly this content «alpha». Second, derive the missing compiler contract.",
        &["bash", "write"],
    );
    assert_eq!(
        fs::read_to_string(outcome.root.join("a.txt")).expect("first artifact"),
        "alpha"
    );
    assert!(
        outcome
            .receipts
            .contains(&("cat a.txt".to_owned(), "alpha".to_owned()))
    );
    let answer = outcome.answer.as_deref().expect("mandatory gap");
    assert!(answer.contains("no_artifact_in_clause"));
    assert!(!answer.contains("Completed the general change request"));
}

#[test]
fn closed_fence_enumeration_payload_has_one_real_exact_receipt() {
    let quote = char::from(96).to_string().repeat(12);
    let body = "λ🙂 Alpha.\nNext beta.\nWhen «input» then «output».\nAlso list behavior rules.\n";
    let request =
        format!("Set the contents of a.txt to exactly this content:\n{quote}text\n{body}{quote}");
    let outcome = run(&request, &["bash", "write"]);
    assert_eq!(
        fs::read_to_string(outcome.root.join("a.txt")).expect("literal bytes"),
        body
    );
    assert_eq!(outcome.writes, 1);
    assert_eq!(
        outcome.receipts,
        vec![("cat a.txt".to_owned(), body.to_owned())]
    );
    assert!(outcome.answer.is_some());
}
#[test]
fn outer_literal_artifacts_keep_inner_enumeration_payloads() {
    let first = "λ🙂 Alpha. Next beta.";
    let second = "Gamma. Then delta.";
    let request = format!(
        "First, create a.txt with exactly this content «{first}». Second, create b.txt with exactly this content «{second}»."
    );
    exact_pair(&run(&request, &["bash", "write"]), first, second);
}
