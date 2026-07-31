use std::collections::BTreeSet;
use std::fs;
use std::process::Command;

use formal_ai::agentic_coding::{plan_chat_step, AgenticPlan};
use formal_ai::computer_use::{
    benchmark_tasks, capability_gap_for_prompt, replay_verified_plan, run_verified_plan,
    ComputerPlanStep, ComputerUsePolicy, ComputerUsePrimitive, ComputerUseSession,
    VerificationPhase, COMPUTER_USE_PRIMITIVES,
};
use formal_ai::protocol::{ChatMessage, ToolCall};
use serde_json::json;

const TOOLS: [&str; 12] = [
    "fs.read",
    "fs.write",
    "fs.list",
    "fs.move",
    "shell.run",
    "http.fetch",
    "http.post",
    "dom.query",
    "dom.extract",
    "archive.pack",
    "archive.unpack",
    "process.status",
];

#[test]
fn every_seeded_primitive_advertises_verification_isolation_and_permission() {
    let seed = fs::read_to_string("data/seed/tools.lino").expect("tool registry");
    for primitive in TOOLS {
        let record = seed
            .split(&format!("    name {primitive}\n"))
            .nth(1)
            .and_then(|tail| tail.split("\n  tool ").next())
            .unwrap_or_else(|| panic!("missing primitive {primitive}"));
        assert!(
            record
                .lines()
                .find(|line| line.trim_start().starts_with("inputs"))
                .is_some_and(|inputs| {
                    inputs.contains("precondition") && inputs.contains("postcondition")
                }),
            "{primitive} does not advertise its verification context"
        );
        assert!(record.contains("\n    isolation "), "{primitive}");
        assert!(record.contains("\n    permission "), "{primitive}");
    }
}

#[test]
fn ten_seeded_tasks_execute_and_replay_with_every_step_verified() {
    let tasks = benchmark_tasks();
    assert_eq!(tasks.len(), 10);
    let mut covered = BTreeSet::new();
    let mut workspaces = BTreeSet::new();

    for task in tasks {
        assert_eq!(task.prompts.len(), 4, "{}", task.id);
        assert!(task.steps.len() >= 2, "{}", task.id);
        for step in &task.steps {
            assert!(
                step.precondition.contains(&step.primitive.permission_key()),
                "{} {}",
                task.id,
                step.id
            );
            assert!(
                !step.postcondition.trim().is_empty(),
                "{} {}",
                task.id,
                step.id
            );
        }
        let prompt = task.prompts.get("en").expect("English benchmark prompt");
        let outcome =
            run_verified_plan(prompt).unwrap_or_else(|error| panic!("{}: {error}", task.id));
        assert!(outcome.verified, "{}", task.id);
        assert!(
            workspaces.insert(outcome.workspace.clone()),
            "workspace reused"
        );
        assert_eq!(outcome.steps.len(), task.steps.len());
        for record in &outcome.steps {
            covered.insert(record.primitive.clone());
            assert!(record.verified, "{} {}", task.id, record.step_id);
            assert_eq!(record.events.len(), 3);
            assert_eq!(record.events[0].phase, VerificationPhase::Precondition);
            assert_eq!(record.events[1].phase, VerificationPhase::Effect);
            assert_eq!(record.events[2].phase, VerificationPhase::Postcondition);
            assert!(record.events.iter().all(|event| event.passed));
        }
        assert!(
            replay_verified_plan(&outcome).expect("deterministic replay"),
            "{}",
            task.id
        );
    }

    assert_eq!(
        covered,
        COMPUTER_USE_PRIMITIVES
            .into_iter()
            .map(|primitive| primitive.name().to_owned())
            .collect()
    );
}

#[test]
fn planner_emits_the_same_seeded_plan_to_an_external_tool_client() {
    for task in benchmark_tasks() {
        let prompt = task.prompts.get("en").expect("English prompt");
        let mut messages = vec![ChatMessage::user(prompt)];
        for expected in &task.steps {
            let plan = plan_chat_step(&messages, &TOOLS).expect("computer-use plan");
            let AgenticPlan::ToolCalls(calls) = plan else {
                panic!("{} finished before {}", task.id, expected.id);
            };
            assert_eq!(calls.len(), 1);
            assert_eq!(calls[0].tool, expected.primitive.name());
            let arguments: serde_json::Value =
                serde_json::from_str(&calls[0].arguments).expect("step arguments");
            assert_eq!(arguments["plan_id"], task.id);
            assert_eq!(arguments["step_id"], expected.id);
            assert_eq!(arguments["precondition"], expected.precondition);
            assert_eq!(arguments["postcondition"], expected.postcondition);
            messages.push(ChatMessage::tool_result(
                format!("call-{}", expected.id),
                calls[0].tool.clone(),
                r#"{"verified":true}"#,
            ));
        }
        let final_plan = plan_chat_step(&messages, &TOOLS).expect("final plan");
        let AgenticPlan::Final(answer) = final_plan else {
            panic!("{} did not finish", task.id);
        };
        assert!(answer.contains("computer_use_complete"));
        assert!(answer.contains(&task.steps.len().to_string()));
    }
}

#[test]
fn planner_scopes_progress_to_the_latest_user_turn_and_stops_after_failed_verification() {
    let prompt = "Filter active customers into a report";
    let mut messages = vec![
        ChatMessage::user("An earlier unrelated request"),
        ChatMessage::tool_result("old-call", "fs.read", r#"{"verified":true}"#),
        ChatMessage::user(prompt),
    ];

    let first = plan_chat_step(&messages, &TOOLS).expect("first computer-use step");
    let AgenticPlan::ToolCalls(calls) = first else {
        panic!("historical tool results skipped the first step");
    };
    assert_eq!(calls[0].tool, "fs.write");

    messages.push(ChatMessage::tool_result(
        "current-call",
        "fs.write",
        r#"{"verified":false}"#,
    ));
    let halted = plan_chat_step(&messages, &TOOLS).expect("verification failure response");
    let AgenticPlan::Final(answer) = halted else {
        panic!("planner scheduled an effect after failed verification");
    };
    assert!(answer.contains("computer_use_incomplete"));
    assert!(answer.contains("active_customers-01"));
}

#[test]
fn planner_resolves_openai_tool_results_through_their_call_id() {
    let prompt = "Filter active customers into a report";
    let mut messages = vec![ChatMessage::user(prompt)];
    let first = plan_chat_step(&messages, &TOOLS).expect("first computer-use step");
    let AgenticPlan::ToolCalls(calls) = first else {
        panic!("expected a tool call");
    };
    messages.push(ChatMessage::assistant_tool_calls(vec![ToolCall::function(
        "call-01",
        "formal_ai_fs_write",
        calls[0].arguments.clone(),
    )]));
    let mut result =
        ChatMessage::tool_result("call-01", "formal_ai_fs_write", r#"{"verified":true}"#);
    result.name = None;
    messages.push(result);

    let next = plan_chat_step(&messages, &TOOLS).expect("second computer-use step");
    let AgenticPlan::ToolCalls(calls) = next else {
        panic!("name-less OpenAI tool result halted a verified plan");
    };
    assert_eq!(calls[0].tool, "shell.run");
}

#[test]
fn every_primitive_is_agent_mode_permission_and_confirmation_gated() {
    let base = std::env::temp_dir().join("formal-ai-issue-707-policy-tests");
    for primitive in COMPUTER_USE_PRIMITIVES {
        let mut session =
            ComputerUseSession::in_base("denied", ComputerUsePolicy::deny_all(), &base)
                .expect("session");
        let step = ComputerPlanStep {
            id: format!("denied-{}", primitive.name()),
            primitive,
            arguments: minimal_arguments(primitive),
            precondition: "permission".to_owned(),
            postcondition: "verified".to_owned(),
        };
        let record = session.execute_step(&step);
        assert!(!record.verified, "{}", primitive.name());
        assert!(record.output["error"]
            .as_str()
            .is_some_and(|error| error.contains("agent_mode_required")));
        assert!(!record.events[0].passed);
        assert!(!record.events[1].passed);
        assert!(!record.events[2].passed);
    }

    let grant = ComputerUsePrimitive::FsWrite.permission_key();
    let mut session = ComputerUseSession::in_base(
        "confirmation",
        ComputerUsePolicy::with_grants(true, [grant]),
        &base,
    )
    .expect("session");
    let record = session.execute_primitive(
        "confirmation-01",
        ComputerUsePrimitive::FsWrite,
        json!({"path":"result.txt","content":"no confirmation"}),
        "confirmed",
        "written",
    );
    assert!(!record.verified);
    assert!(record.output["error"]
        .as_str()
        .is_some_and(|error| error.contains("confirmation_required")));
    assert!(!session.root().join("result.txt").exists());
}

#[test]
fn workspace_paths_cannot_escape_the_isolation_boundary() {
    let base = std::env::temp_dir().join("formal-ai-issue-707-path-tests");
    let mut session =
        ComputerUseSession::in_base("escape", ComputerUsePolicy::agent_mode_all(), &base)
            .expect("session");
    let outside = base.join("outside.txt");
    let record = session.execute_primitive(
        "escape-01",
        ComputerUsePrimitive::FsWrite,
        json!({"path":"../outside.txt","content":"escape","confirmed":true}),
        "confined",
        "written",
    );
    assert!(!record.verified);
    assert!(record.output["error"]
        .as_str()
        .is_some_and(|error| error.contains("path_escapes_workspace")));
    assert!(!outside.exists());
}

#[test]
fn rendering_capability_gap_is_explicit_in_all_supported_languages() {
    let cues = [
        "Take a screenshot of the rendered page",
        "Сделай снимок отрисованной страницы",
        "rendered page का screenshot लो",
        "截取渲染页面的屏幕截图",
    ];
    let mut locales = BTreeSet::new();
    for cue in cues {
        let gap = capability_gap_for_prompt(cue).expect("seeded capability gap");
        assert_eq!(gap.capability, "gui_rendering");
        assert!(gap.response.contains("capability_gap"));
        assert!(gap.response.contains("gui_rendering"));
        locales.insert(gap.locale);
    }
    assert_eq!(
        locales,
        BTreeSet::from([
            "en".to_owned(),
            "hi".to_owned(),
            "ru".to_owned(),
            "zh".to_owned()
        ])
    );
}

#[test]
fn external_agent_mcp_prefix_resolves_every_primitive() {
    for primitive in COMPUTER_USE_PRIMITIVES {
        let agent_name = format!("formal_ai_{}", primitive.name().replace('.', "_"));
        assert_eq!(
            ComputerUsePrimitive::from_tool_name(&agent_name),
            Some(primitive),
            "{agent_name}"
        );
    }
}

#[test]
fn native_cli_requires_permissions_and_can_replay_a_verified_plan() {
    let denied = Command::new(env!("CARGO_BIN_EXE_formal-ai"))
        .args([
            "--silent",
            "computer-use",
            "--prompt",
            "Save and verify the isolated process status",
        ])
        .output()
        .expect("run denied CLI plan");
    assert!(!denied.status.success());
    assert!(String::from_utf8_lossy(&denied.stderr).contains("--agent-mode is required"));

    let completed = Command::new(env!("CARGO_BIN_EXE_formal-ai"))
        .args([
            "--silent",
            "computer-use",
            "--prompt",
            "Save and verify the isolated process status",
            "--agent-mode",
            "--confirm-effects",
            "--replay",
        ])
        .output()
        .expect("run verified CLI plan");
    assert!(
        completed.status.success(),
        "{}",
        String::from_utf8_lossy(&completed.stderr)
    );
    let json: serde_json::Value =
        serde_json::from_slice(&completed.stdout).expect("CLI outcome JSON");
    assert_eq!(json["outcome"]["verified"], true);
    assert_eq!(json["replay_verified"], true);
}

#[test]
fn real_agent_cli_record_replay_is_a_required_ci_gate() {
    let workflow = fs::read_to_string(".github/workflows/release.yml").expect("release workflow");
    assert!(
        workflow.contains("experiments/agent_cli_e2e/run_issue_707.sh"),
        "release CI must drive issue #707 through the real Agent CLI"
    );

    let harness = fs::read_to_string("experiments/agent_cli_e2e/run_issue_707.sh")
        .expect("issue #707 Agent CLI harness");
    assert!(harness.contains("for phase in record replay"));
    assert!(harness.contains("expected exactly ten seeded task ids"));
    assert!(harness.contains("--output-format stream-json"));
    assert!(harness.contains("verify_issue_707.mjs"));
}

fn minimal_arguments(primitive: ComputerUsePrimitive) -> serde_json::Value {
    match primitive {
        ComputerUsePrimitive::FsRead | ComputerUsePrimitive::FsList => json!({"path":"missing"}),
        ComputerUsePrimitive::FsWrite => json!({"path":"file","content":"x","confirmed":true}),
        ComputerUsePrimitive::FsMove => json!({"from":"missing","to":"moved","confirmed":true}),
        ComputerUsePrimitive::ShellRun => {
            json!({"operation":"count_lines","input":"missing","output":"out","confirmed":true})
        }
        ComputerUsePrimitive::HttpFetch => {
            json!({"url":"fixture://orders.json","save_as":"cache"})
        }
        ComputerUsePrimitive::HttpPost => json!({
            "url":"fixture://submit",
            "body":"token=fixture-token",
            "save_as":"cache",
            "confirmed":true
        }),
        ComputerUsePrimitive::DomQuery => {
            json!({"source":"missing","selector":"title","save_as":"out"})
        }
        ComputerUsePrimitive::DomExtract => {
            json!({"source":"missing","pointer":"/id","save_as":"out"})
        }
        ComputerUsePrimitive::ArchivePack => {
            json!({"paths":["missing"],"archive":"out","confirmed":true})
        }
        ComputerUsePrimitive::ArchiveUnpack => {
            json!({"archive":"missing","destination":"out","confirmed":true})
        }
        ComputerUsePrimitive::ProcessStatus => json!({"save_as":"status"}),
    }
}
