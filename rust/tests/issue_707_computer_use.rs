use std::collections::BTreeSet;
use std::fs;

fn root() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crate root parent")
        .to_path_buf()
}
use std::process::Command;

use formal_ai::agentic_coding::{AgenticPlan, plan_chat_step};
use formal_ai::computer_use::{
    COMPUTER_USE_PRIMITIVES, ComputerPlanStep, ComputerUsePolicy, ComputerUsePrimitive,
    ComputerUseSession, VerificationPhase, capability_gap_for_request, execute_verified_plan,
    parse_benchmark_tasks, plan_for_prompt, plan_request, replay_verified_plan_with_policy,
    run_verified_plan,
};
use formal_ai::protocol::{ChatMessage, ToolCall};
use serde_json::json;

#[path = "fixtures/computer-use/support.rs"]
mod fixtures;

fn request(prompt: &str, steps: &[ComputerPlanStep]) -> String {
    let entries = steps
        .iter()
        .map(|step| {
            json!({
                "primitive": step.primitive.name(), "arguments": step.arguments
            })
        })
        .collect::<Vec<_>>();
    format!("{prompt}\n```computer-use\n{}\n```", json!(entries))
}

fn customer_request() -> String {
    let tasks = parse_benchmark_tasks(fixtures::CORPUS);
    let prompt = "Filter active customers into a report";
    let plan = plan_for_prompt(&tasks, prompt).expect("recorded task");
    request(prompt, &plan.steps)
}

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
    let seed = fs::read_to_string(root().join("data/seed/tools.lino")).expect("tool registry");
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
    let publisher = fixtures::Publisher::start();
    let tasks = parse_benchmark_tasks(&publisher.corpus());
    let policy = ComputerUsePolicy::agent_mode_all()
        .with_http_origin(publisher.origin.parse().expect("fixture publisher origin"));
    assert_eq!(tasks.len(), 10);
    let mut covered = BTreeSet::new();
    let mut workspaces = BTreeSet::new();

    for task in &tasks {
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
        assert!(
            plan_request(prompt).is_none(),
            "cold request selected {}",
            task.id
        );
        let plan = plan_for_prompt(&tasks, prompt).expect("explicit corpus plan");
        let outcome = execute_verified_plan(plan, policy.clone())
            .unwrap_or_else(|error| panic!("{}: {error}", task.id));
        assert!(outcome.verified, "{}: {:?}", task.id, outcome.steps);
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
            replay_verified_plan_with_policy(&outcome, policy.clone())
                .expect("deterministic replay"),
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
    for task in parse_benchmark_tasks(fixtures::CORPUS) {
        let prompt = task.prompts.get("en").expect("English prompt");
        let prompt = request(prompt, &task.steps);
        let expected_plan = plan_request(&prompt).expect("explicit plan");
        let mut messages = vec![ChatMessage::user(prompt)];
        for expected in &expected_plan.steps {
            let plan = plan_chat_step(&messages, &TOOLS).expect("computer-use plan");
            let AgenticPlan::ToolCalls(calls) = plan else {
                panic!("{} finished before {}", task.id, expected.id);
            };
            assert_eq!(calls.len(), 1);
            assert_eq!(calls[0].tool, expected.primitive.name());
            let arguments: serde_json::Value =
                serde_json::from_str(&calls[0].arguments).expect("step arguments");
            assert_eq!(arguments["plan_id"], expected_plan.id);
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
    let prompt = customer_request();
    let expected = plan_request(&prompt).expect("explicit plan");
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
    assert!(answer.contains(&expected.steps[0].id));
}

#[test]
fn planner_resolves_openai_tool_results_through_their_call_id() {
    let mut messages = vec![ChatMessage::user(customer_request())];
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
        assert!(
            record.output["error"]
                .as_str()
                .is_some_and(|error| error.contains("agent_mode_required"))
        );
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
    assert!(
        record.output["error"]
            .as_str()
            .is_some_and(|error| error.contains("confirmation_required"))
    );
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
    assert!(
        record.output["error"]
            .as_str()
            .is_some_and(|error| error.contains("path_escapes_workspace"))
    );
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
        let gap = capability_gap_for_request(cue).expect("capability gap");
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
    let task = "Save and verify the isolated process status";
    let plan = plan_for_prompt(&parse_benchmark_tasks(fixtures::CORPUS), task).expect("test plan");
    let prompt = request(task, &plan.steps);
    let denied = Command::new(env!("CARGO_BIN_EXE_formal-ai"))
        .args(["--silent", "computer-use", "--prompt", &prompt])
        .output()
        .expect("run denied CLI plan");
    assert!(!denied.status.success());
    assert!(String::from_utf8_lossy(&denied.stderr).contains("--agent-mode is required"));

    let completed = Command::new(env!("CARGO_BIN_EXE_formal-ai"))
        .args([
            "--silent",
            "computer-use",
            "--prompt",
            &prompt,
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
    // Issue #1081 lifted these 51 steps out of `release.yml` into a reusable
    // workflow, which put one hop between the release pipeline and the
    // harness. The invariant is that a release run drives issue #707 through
    // the real Agent CLI -- not that one particular file spells the path out
    // -- so follow the hop rather than drop the assertion, and check both
    // ends: a caller that no longer calls, and a callee that no longer runs
    // the harness, are the same false negative.
    let release =
        fs::read_to_string(root().join(".github/workflows/release.yml")).expect("release workflow");
    let called = ".github/workflows/agent-cli-e2e.yml";
    assert!(
        release.contains(&format!("uses: ./{called}")),
        "release CI must reach the Agent CLI E2E job, directly or through {called}"
    );
    assert!(
        release.contains("test-agent-cli-e2e,"),
        "the terminal pipeline-status gate must observe the extracted job"
    );

    let workflow = fs::read_to_string(root().join(called)).expect("agent CLI E2E workflow");
    assert!(
        workflow.contains("experiments/agent_cli_e2e/run_issue_707.sh"),
        "release CI must drive issue #707 through the real Agent CLI"
    );

    let harness = fs::read_to_string(root().join("experiments/agent_cli_e2e/run_issue_707.sh"))
        .expect("issue #707 Agent CLI harness");
    assert!(harness.contains("for phase in record replay"));
    assert!(harness.contains("expected exactly ten seeded task ids"));
    assert!(harness.contains("--output-format stream-json"));
    assert!(harness.contains("verify_issue_707.mjs"));
}

#[test]
fn explicit_plans_validate_the_whole_request_and_stop_on_failed_steps() {
    let invalid = [
        r#"{"primitive":"shell.run","arguments":{"operation":"filter_csv","input":"a","output":"b","confirmed":true}}"#,
        r#"{"primitive":"fs.write","arguments":{"path":"b","content":"extra","confirmed":"true"}}"#,
        r#"{"primitive":"fs.read","arguments":{"path":"a","postcondition":"override"}}"#,
    ];
    let first = r#"{"primitive":"fs.write","arguments":{"path":"first","content":"first","confirmed":true}}"#;
    for step in invalid {
        let prompt = format!("```computer-use\n[{first},{step}]\n```");
        assert!(plan_request(&prompt).is_none());
        assert_eq!(
            capability_gap_for_request(&prompt).unwrap().capability,
            "plan-input"
        );
    }
    let valid = format!("```computer-use\n[{first}]\n```");
    assert!(plan_request(&format!("{valid}\ntrailing payload\n```")).is_none());
    assert!(plan_request(&format!("```text\nexample\n```\n{valid}")).is_none());
    let bad_read = r#"{"primitive":"fs.read","arguments":{"path":"missing"}}"#;
    let prompt = format!("```computer-use\n[{bad_read},{first}]\n```");
    let outcome = run_verified_plan(&prompt).expect("execute valid typed plan");
    assert!(!outcome.verified);
    assert_eq!(outcome.steps.len(), 1);
    assert!(
        !std::path::Path::new(&outcome.workspace)
            .join("first")
            .exists()
    );
}

#[test]
fn http_receipt_requires_a_successful_response_from_the_configured_origin() {
    let publisher = fixtures::Publisher::start();
    let policy = ComputerUsePolicy::agent_mode_all()
        .with_http_origin(publisher.origin.parse().expect("fixture origin"));
    let mut session = ComputerUseSession::new("http-receipt", policy).expect("session");
    let mut receive = |url: &str| {
        session.execute_primitive(
            "receive",
            ComputerUsePrimitive::HttpFetch,
            json!({"url": url, "save_as": "received"}),
            "granted",
            "verified bytes",
        )
    };
    let foreign = receive("https://example.invalid/caller-data");
    assert!(!foreign.verified);
    assert_eq!(foreign.output["error"], "computer_use_http_origin_mismatch");
    let redirect = receive(&format!("{}/redirect", publisher.origin));
    assert!(!redirect.verified);
    assert_eq!(redirect.output["error"], "computer_use_http_status:302");
    assert!(!session.root().join("received").exists());
    let observed = session.execute_primitive(
        "receive",
        ComputerUsePrimitive::HttpFetch,
        json!({"url": format!("{}/caller-data", publisher.origin), "save_as": "received"}),
        "granted",
        "verified bytes",
    );
    assert!(observed.verified, "{:?}", observed.events);
    assert_eq!(observed.output["status"], 200);
    assert_eq!(observed.output["body"], "caller α\n");
    assert_eq!(
        observed.output["headers"]["content-type"],
        "text/plain; charset=utf-8"
    );
    assert_eq!(
        fs::read(session.root().join("received")).expect("observed bytes"),
        "caller α\n".as_bytes()
    );
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
