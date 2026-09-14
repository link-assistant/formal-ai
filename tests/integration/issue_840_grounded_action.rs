//! Executed whole-journey coverage for issue #840's grounded-action recipe.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

use formal_ai::agentic_coding::{AgenticPlan, plan_chat_step};
use formal_ai::protocol::{ChatMessage, ToolCall};

static FIXTURE_SEQUENCE: AtomicU64 = AtomicU64::new(0);

struct DesktopFixture {
    root: PathBuf,
    target: PathBuf,
    guide: PathBuf,
    decoy: PathBuf,
}

impl DesktopFixture {
    fn new() -> Self {
        let sequence = FIXTURE_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "formal-ai-issue-840-{}-{sequence}",
            std::process::id()
        ));
        let target = root.join("Archive/hive-control-center");
        let guide = target.join("operations-guide.txt");
        let decoy = root.join("Archive/hive-mind-bot.2025-12-26.private-key.pem");
        std::fs::create_dir_all(&target).expect("create nested folder fixture");
        std::fs::write(&guide, "grounded fixture\n").expect("create contents fixture");
        std::fs::write(&decoy, "not the requested folder\n").expect("create PEM decoy");
        Self {
            root,
            target,
            guide,
            decoy,
        }
    }
}

impl Drop for DesktopFixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

#[derive(Debug)]
struct Journey {
    answer: String,
    commands: Vec<String>,
    observations: Vec<String>,
}

fn execute_journey(prompt: &str, desktop: &Path) -> Journey {
    let mut messages = vec![ChatMessage::user(prompt)];
    let mut commands = Vec::new();
    let mut observations = Vec::new();

    for step in 0..6 {
        let plan = plan_chat_step(&messages, &["bash", "websearch"])
            .unwrap_or_else(|| panic!("step {step} did not produce a plan"));
        match plan {
            AgenticPlan::Final(answer) => {
                return Journey {
                    answer,
                    commands,
                    observations,
                };
            }
            AgenticPlan::ToolCalls(calls) => {
                assert_eq!(calls.len(), 1, "one minimal action per step: {calls:?}");
                let call = calls.into_iter().next().unwrap();
                assert_eq!(call.tool, "bash", "local location must dominate: {call:?}");
                let arguments: serde_json::Value =
                    serde_json::from_str(&call.arguments).expect("tool arguments");
                let command = arguments["command"].as_str().expect("shell command");
                assert_simple_command(command);
                let output = Command::new("bash")
                    .args(["-c", command])
                    .env("FORMAL_AI_DESKTOP_DIR", desktop)
                    .output()
                    .expect("execute planned observation");
                assert!(
                    output.status.success(),
                    "{command}\n{}",
                    String::from_utf8_lossy(&output.stderr)
                );
                let observation = String::from_utf8(output.stdout).expect("UTF-8 fixture paths");
                let id = format!("grounded_{step}");
                messages.push(ChatMessage::assistant_tool_calls(vec![ToolCall::function(
                    id.clone(),
                    call.tool,
                    call.arguments,
                )]));
                messages.push(ChatMessage::tool_result(
                    id,
                    "bash",
                    if observation.is_empty() {
                        "(no output)"
                    } else {
                        &observation
                    },
                ));
                commands.push(command.to_owned());
                observations.push(observation);
            }
        }
    }
    panic!("grounded journey exceeded its six-step bound: {commands:?}");
}

fn assert_simple_command(command: &str) {
    assert!(!command.contains(';'), "{command}");
    assert!(!command.contains("&&"), "{command}");
    assert!(!command.contains("-print -quit"), "{command}");
    assert_eq!(command.lines().count(), 1, "{command}");
}

#[test]
fn exact_observation_widens_then_reports_the_verified_directory_and_discrepancy() {
    let fixture = DesktopFixture::new();
    let journey = execute_journey(
        "Find hive-mind-control center folder on my desktop",
        &fixture.root,
    );

    assert_eq!(journey.commands.len(), 2, "{journey:?}");
    assert!(
        journey.commands[0].contains("-iname 'hive-mind-control-center'"),
        "{journey:?}"
    );
    assert!(journey.observations[0].is_empty(), "{journey:?}");
    assert!(
        journey.commands[1].contains("-iname '*hive*'"),
        "{journey:?}"
    );
    assert!(
        journey.observations[1].contains(&fixture.target.to_string_lossy().to_string()),
        "{journey:?}"
    );
    assert!(
        journey
            .commands
            .iter()
            .all(|command| command.contains("-type d")),
        "{journey:?}"
    );
    assert!(journey.answer.contains("hive-mind-control-center"));
    assert!(journey.answer.contains("hive-control-center"));
    assert!(
        journey
            .answer
            .contains(&fixture.target.to_string_lossy().to_string()),
        "{}",
        journey.answer
    );
    assert!(
        !journey
            .answer
            .contains(&fixture.decoy.to_string_lossy().to_string()),
        "{}",
        journey.answer
    );
}

#[test]
fn followup_local_tasks_verify_type_contents_and_scope_listing() {
    let fixture = DesktopFixture::new();

    let type_journey = execute_journey(
        "On my desktop, is hive-control-center a file or folder?",
        &fixture.root,
    );
    assert_eq!(type_journey.commands.len(), 2, "{type_journey:?}");
    assert!(
        type_journey.commands[1].starts_with("ls -ld -- "),
        "{type_journey:?}"
    );
    assert!(
        type_journey.answer.to_lowercase().contains("folder"),
        "{type_journey:?}"
    );

    let contents_journey = execute_journey(
        "What's inside hive-mind-control center on my desktop?",
        &fixture.root,
    );
    assert_eq!(contents_journey.commands.len(), 3, "{contents_journey:?}");
    assert!(
        contents_journey.commands[1].contains("-iname '*hive*'"),
        "{contents_journey:?}"
    );
    assert!(
        contents_journey.commands[2].contains("-mindepth 1 -maxdepth 1"),
        "{contents_journey:?}"
    );
    assert!(
        contents_journey
            .answer
            .contains(&fixture.guide.to_string_lossy().to_string()),
        "{contents_journey:?}"
    );

    let exact_contents = execute_journey(
        "What is inside the Archive folder on my desktop?",
        &fixture.root,
    );
    assert_eq!(exact_contents.commands.len(), 2, "{exact_contents:?}");
    assert!(
        exact_contents.commands[1].contains("-mindepth 1 -maxdepth 1"),
        "{exact_contents:?}"
    );
    assert!(
        exact_contents.answer.contains("hive-control-center"),
        "{exact_contents:?}"
    );

    let listing_journey = execute_journey("List what is on my desktop", &fixture.root);
    assert_eq!(listing_journey.commands.len(), 1, "{listing_journey:?}");
    assert!(
        listing_journey.commands[0].contains("-mindepth 1 -maxdepth 1"),
        "{listing_journey:?}"
    );
    assert!(
        listing_journey.answer.contains("Archive")
            && !listing_journey.answer.contains("private-key.pem"),
        "{listing_journey:?}"
    );
}
