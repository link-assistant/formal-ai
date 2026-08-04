//! Coding-ladder ratchet rungs for issue #916 (epic E69).
//!
//! Every rung below names one open defect from the #902–#909 family and pins the
//! *observed effect* that closes it. The rung identifiers are the same ones the
//! `experiments/issue_916_write_effect_ladder` runner reports and the CI ratchet
//! enforces, so a fix, its regression test, and its ladder rung all carry one name.
//!
//! | Rung | Defect | Guarantee |
//! | --- | --- | --- |
//! | `R916-01` | #905 §2, #908 row 2 | a non-zero `Exit Code:` in a harness envelope is a failure |
//! | `R916-02` | #908 row 3 | `Exit Code: 0` with empty output is a success |
//! | `R916-03` | #908 fix 3 | the failure report names the exit code, not the harness |
//! | `R916-04` | #905 fix 2 | no completion claim without observed verification evidence |
//! | `R916-05` | #905 §3 | literal content after an adverbial qualifier is the content |
//! | `R916-06` | #907 fix 1 | caller context blocks do not hijack intent routing |
//! | `R916-07` | #907 fix 2 | a declarative statement of fact is not a request |
//! | `R916-08` | #909 | `--global` writes a complete headless config |

use formal_ai::agentic_coding::general_planner::compose_general_change_plan;
use formal_ai::agentic_coding::tool_result::{step_outcome, StepOutcome};
use formal_ai::agentic_coding::{plan_chat_step, plan_symbolic_command_reroute, AgenticPlan};
use formal_ai::protocol::{
    create_chat_completion_with_solver, latest_user_request, ChatCompletionRequest, ChatMessage,
    ToolCall,
};
use formal_ai::seed::client_integrations;
use formal_ai::solver::{SolverConfig, UniversalSolver};

/// The exact envelope qwen-code handed back in issue #908 for a command that
/// succeeded silently (`python3 -m py_compile main.py`).
const SILENT_SUCCESS_ENVELOPE: &str = "Command: python3 -m py_compile main.py\n\
     Directory: (root)\n\
     Output: (empty)\n\
     Error: (none)\n\
     Exit Code: 0\n\
     Signal: 0\n\
     Process Group PGID: 685377";

/// The same envelope shape for the `cat hello.txt` failure of issue #905.
const MISSING_FILE_ENVELOPE: &str = "Command: cat hello.txt\n\
     Directory: (root)\n\
     Output: (empty)\n\
     Error: cat: hello.txt: No such file or directory\n\
     Exit Code: 1\n\
     Signal: 0\n\
     Process Group PGID: 685377";

fn tool_turn(prompt: &str, tool: &str, arguments: &str, result: &str) -> Vec<ChatMessage> {
    vec![
        ChatMessage::user(prompt),
        ChatMessage::assistant_tool_calls(vec![ToolCall::function("call_916", tool, arguments)]),
        ChatMessage::tool_result("call_916", tool, result),
    ]
}

fn final_answer(messages: &[ChatMessage], tools: &[&str]) -> String {
    match plan_chat_step(messages, tools) {
        Some(AgenticPlan::Final(answer)) => answer,
        other => panic!("expected a final answer, got {other:?}"),
    }
}

fn planned_command(messages: &[ChatMessage], tools: &[&str]) -> String {
    match plan_chat_step(messages, tools) {
        Some(AgenticPlan::ToolCalls(calls)) => {
            let call = calls.first().expect("at least one tool call");
            let arguments: serde_json::Value =
                serde_json::from_str(&call.arguments).expect("tool arguments");
            format!(
                "{}({})",
                call.tool,
                arguments
                    .get("command")
                    .or_else(|| arguments.get("file_path"))
                    .or_else(|| arguments.get("path"))
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or_default()
            )
        }
        other => panic!("expected a tool call, got {other:?}"),
    }
}

/// The first tool call the server itself plans for `messages`, through the same
/// entry point `/v1/chat/completions` uses: the planner first, then the solver's
/// execution recipe. Routing defects only show up on this whole path — the
/// hijacked `date` command of issue #907 was chosen before the recipe was ever
/// consulted.
fn first_agentic_call(messages: &[ChatMessage], tools: &[&str]) -> String {
    let request = ChatCompletionRequest {
        model: Some(String::from("formal-ai")),
        messages: messages.to_vec(),
        tools: tools
            .iter()
            .map(|name| {
                serde_json::json!({
                    "type": "function",
                    "function": {
                        "name": name,
                        "description": format!("issue-916 ladder tool: {name}"),
                        "parameters": {"type": "object"}
                    }
                })
            })
            .collect(),
        temperature: None,
        stream: false,
        tool_choice: None,
        functions: Vec::new(),
        function_call: None,
        stream_options: None,
    };
    let solver = UniversalSolver::new(SolverConfig {
        agent_mode: true,
        ..SolverConfig::default()
    });
    let completion = create_chat_completion_with_solver(&request, &solver);
    let choice = &completion.choices[0];
    let call = choice.message.tool_calls.first().unwrap_or_else(|| {
        panic!(
            "expected a tool call, got {:?}: {}",
            choice.finish_reason,
            choice.message.content.plain_text()
        )
    });
    let arguments: serde_json::Value =
        serde_json::from_str(&call.function.arguments).expect("tool arguments");
    format!(
        "{}({})",
        call.function.name,
        arguments
            .get("path")
            .or_else(|| arguments.get("file_path"))
            .or_else(|| arguments.get("command"))
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default()
    )
}

// ---------------------------------------------------------------------------
// R916-01 — a non-zero exit code is a failure, whatever the output says
// ---------------------------------------------------------------------------

#[test]
fn r916_01_nonzero_exit_code_in_a_harness_envelope_is_a_failure() {
    assert_eq!(step_outcome(MISSING_FILE_ENVELOPE), StepOutcome::Failed);
    assert_eq!(
        step_outcome(
            &serde_json::json!({
                "output": "cat: hello.txt: No such file or directory",
                "exit_code": 1,
                "status": "failed",
            })
            .to_string()
        ),
        StepOutcome::Failed
    );
}

#[test]
fn r916_01_a_failing_verification_command_is_reported_as_a_failure() {
    let messages = tool_turn(
        "Run cat hello.txt",
        "bash",
        r#"{"command":"cat hello.txt"}"#,
        MISSING_FILE_ENVELOPE,
    );
    let answer = final_answer(&messages, &["bash"]);

    assert!(
        !answer.contains("completed"),
        "a command that exited 1 must not be reported as completed: {answer}"
    );
    assert!(
        answer.contains("No such file or directory"),
        "the failure must quote what the harness observed: {answer}"
    );
}

// ---------------------------------------------------------------------------
// R916-02 — exit 0 with no output is a success, never a failure
// ---------------------------------------------------------------------------

#[test]
fn r916_02_exit_zero_with_empty_output_is_a_success() {
    assert_eq!(
        step_outcome(SILENT_SUCCESS_ENVELOPE),
        StepOutcome::Succeeded
    );
}

#[test]
fn r916_02_a_silently_succeeding_command_is_not_reported_as_a_failure() {
    let messages = tool_turn(
        "Run python3 -m py_compile main.py",
        "bash",
        r#"{"command":"python3 -m py_compile main.py"}"#,
        SILENT_SUCCESS_ENVELOPE,
    );
    let answer = final_answer(&messages, &["bash"]);

    assert!(
        !answer.contains("failed") && !answer.contains("could not complete"),
        "`py_compile` exiting 0 without output is its documented success: {answer}"
    );
    assert!(
        !answer.contains("Exit Code:") && !answer.contains("Process Group PGID"),
        "the transport envelope must not leak into the answer: {answer}"
    );
}

#[test]
fn r916_02_the_failure_lexicon_never_fires_on_a_clean_exit() {
    // `Error: (none)` is the harness saying there was no error. Keying the
    // verdict on the word "error" read it as one (issue #908).
    for raw in [
        SILENT_SUCCESS_ENVELOPE,
        "Command: true\nOutput: (empty)\nError: (none)\nExit Code: 0\nSignal: 0",
        "Output: ok\nError: (none)\nExit Code: 0",
    ] {
        assert_eq!(step_outcome(raw), StepOutcome::Succeeded, "{raw}");
    }
}

/// The #908 end-to-end run: `Write a hello world program in Python.` produces a
/// source artifact whose own check command is `python3 -m py_compile main.py`,
/// which prints nothing when it succeeds.
///
/// The run is driven to its terminal step, feeding each planned command the
/// envelope the harness would hand back: `check_result` for the silent check, and
/// the program's own greeting for the run that follows it.
fn python_hello_world_reroute(check_result: &str) -> AgenticPlan {
    const RUN_RESULT: &str = "Command: python3 main.py\nDirectory: (root)\nOutput: Hello World\n\
         Error: (none)\nExit Code: 0\nSignal: 0\nProcess Group PGID: 685377";
    const MAX_STEPS: usize = 8;

    let prompt = "Write a hello world program in Python.";
    let solver = UniversalSolver::new(SolverConfig {
        agent_mode: true,
        ..SolverConfig::default()
    });
    let answer = solver.solve(prompt);
    let recipe = answer
        .execution_recipe
        .as_ref()
        .expect("python hello world carries an execution recipe");
    assert_eq!(recipe.path, "main.py");
    assert!(
        recipe
            .commands
            .first()
            .is_some_and(|command| command.contains("py_compile")),
        "the check command is the one that succeeds silently: {:?}",
        recipe.commands
    );

    let mut messages = vec![ChatMessage::user(prompt)];
    let tools = ["write_file", "run_shell_command"];
    for index in 0..MAX_STEPS {
        let plan = plan_symbolic_command_reroute(&messages, &tools, &answer)
            .expect("every step of a recipe run is planned");
        let AgenticPlan::ToolCalls(calls) = plan else {
            return plan;
        };
        let call = &calls[0];
        let result = if call.arguments.contains("py_compile") {
            check_result
        } else if call.arguments.contains("python3 main.py") {
            RUN_RESULT
        } else {
            "wrote main.py"
        };
        let id = format!("reroute-{index}");
        messages.push(ChatMessage::assistant_tool_calls(vec![ToolCall::function(
            &id,
            &call.tool,
            call.arguments.clone(),
        )]));
        messages.push(ChatMessage::tool_result(id, &call.tool, result));
    }
    panic!("the run never terminated within {MAX_STEPS} steps");
}

#[test]
fn r916_02_a_silent_check_command_does_not_abandon_the_completed_work() {
    let AgenticPlan::Final(answer) = python_hello_world_reroute(SILENT_SUCCESS_ENVELOPE) else {
        panic!("the run must terminate with an answer, not another tool call");
    };

    assert!(
        !answer.contains("could not complete"),
        "`python3 -m py_compile` exiting 0 with no output is success (issue #908): {answer}"
    );
    assert!(
        answer.contains("main.py"),
        "the completed artifact must still be reported: {answer}"
    );
}

// ---------------------------------------------------------------------------
// R916-03 — the failure report names the exit code
// ---------------------------------------------------------------------------

#[test]
fn r916_03_failure_reports_name_the_exit_code() {
    let messages = tool_turn(
        "Run cat hello.txt",
        "bash",
        r#"{"command":"cat hello.txt"}"#,
        MISSING_FILE_ENVELOPE,
    );
    let answer = final_answer(&messages, &["bash"]);

    assert!(
        answer.contains('1'),
        "the report must state the observed exit code: {answer}"
    );
    assert!(
        !answer.contains("harness could not complete"),
        "the harness reported the exit code correctly; do not blame it: {answer}"
    );
}

#[test]
fn r916_03_a_genuinely_failing_check_command_names_its_exit_code() {
    let failing = "Command: python3 -m py_compile main.py\n\
         Directory: (root)\n\
         Output: (empty)\n\
         Error: SyntaxError: invalid syntax\n\
         Exit Code: 1\n\
         Signal: 0\n\
         Process Group PGID: 685377";
    let AgenticPlan::Final(answer) = python_hello_world_reroute(failing) else {
        panic!("a failing check must terminate the run with a report");
    };

    assert!(
        answer.contains("SyntaxError: invalid syntax"),
        "the report must quote what the tool observed: {answer}"
    );
    assert!(
        answer.contains('1'),
        "the report must state the observed exit code (issue #908 fix 3): {answer}"
    );
    assert!(
        !answer.contains("harness could not complete"),
        "the harness did its job correctly; the command failed: {answer}"
    );
}

// ---------------------------------------------------------------------------
// R916-04 — no completion claim without an observed workspace effect
// ---------------------------------------------------------------------------

#[test]
fn r916_04_completion_is_not_claimed_when_verification_exits_nonzero() {
    let task = "Create a file hello.txt containing exactly: Hello World";
    let tools = ["write_file", "bash"];
    let mut messages = vec![ChatMessage::user(task)];

    // Both writes "succeed" as far as the transport is concerned — this is the
    // `write_stdin failed: Unknown process id 0` run of issue #905, where the
    // effect never reached the workspace.
    for (index, result) in ["wrote the plan", "wrote hello.txt", MISSING_FILE_ENVELOPE]
        .into_iter()
        .enumerate()
    {
        let AgenticPlan::ToolCalls(calls) =
            plan_chat_step(&messages, &tools).expect("planned step")
        else {
            panic!("step {index} must be a tool call");
        };
        let call = &calls[0];
        let id = format!("call-{index}");
        messages.push(ChatMessage::assistant_tool_calls(vec![ToolCall::function(
            &id,
            &call.tool,
            call.arguments.clone(),
        )]));
        messages.push(ChatMessage::tool_result(id, &call.tool, result));
    }

    let answer = final_answer(&messages, &tools);
    assert!(
        !answer.contains("Completed the general change request"),
        "`cat hello.txt` exited 1: completion must not be claimed: {answer}"
    );
    assert!(
        !answer.contains("verified it"),
        "verification must not be claimed against contrary evidence: {answer}"
    );
    assert!(
        answer.contains("hello.txt"),
        "the honest answer still names the requested target: {answer}"
    );
}

#[test]
fn r916_04_completion_is_claimed_when_the_effect_is_observed() {
    let task = "Create a file hello.txt containing exactly: Hello World";
    let tools = ["write_file", "bash"];
    let mut messages = vec![ChatMessage::user(task)];

    let observed = "Command: cat hello.txt\nDirectory: (root)\nOutput: Hello World\n\
         Error: (none)\nExit Code: 0\nSignal: 0\nProcess Group PGID: 685377";
    for (index, result) in ["wrote the plan", "wrote hello.txt", observed]
        .into_iter()
        .enumerate()
    {
        let AgenticPlan::ToolCalls(calls) =
            plan_chat_step(&messages, &tools).expect("planned step")
        else {
            panic!("step {index} must be a tool call");
        };
        let call = &calls[0];
        let id = format!("call-{index}");
        messages.push(ChatMessage::assistant_tool_calls(vec![ToolCall::function(
            &id,
            &call.tool,
            call.arguments.clone(),
        )]));
        messages.push(ChatMessage::tool_result(id, &call.tool, result));
    }

    let answer = final_answer(&messages, &tools);
    assert!(
        answer.contains("Completed the general change request"),
        "an observed, exit-0 verification is exactly what completion means: {answer}"
    );
}

// ---------------------------------------------------------------------------
// R916-05 — an adverbial qualifier is not part of the content
// ---------------------------------------------------------------------------

#[test]
fn r916_05_adverbial_qualifiers_are_stripped_from_literal_content() {
    let cases = [
        "Create a file hello.txt containing exactly: Hello World",
        "Create a file hello.txt containing precisely: Hello World",
        "Create a file hello.txt whose content is exactly: Hello World",
        "Создай файл hello.txt, содержащий ровно: Hello World",
    ];
    for task in cases {
        let plan = compose_general_change_plan(task).unwrap_or_else(|| panic!("plan for {task}"));
        assert_eq!(plan.target, "hello.txt", "{task}");
        assert_eq!(
            plan.content, "Hello World",
            "the adverb qualifies the requirement, not the bytes: {task}"
        );
        assert!(
            plan.steps
                .iter()
                .all(|step| !step.expected_evidence.contains("exactly:")
                    && !step.expected_evidence.contains("ровно:")),
            "{task}: {:?}",
            plan.steps
        );
    }
}

// ---------------------------------------------------------------------------
// R916-06 / R916-07 — caller framing is not the user's request
// ---------------------------------------------------------------------------

/// The `<session_context>` block gemini-cli prepends to every single turn.
const GEMINI_SESSION_CONTEXT: &str = "<session_context>\n\
     Today's date is Sunday, August 2, 2026 (formatted according to the user's locale).\n\
     My operating system is: linux\n\
     I'm currently working in the directory: /tmp/work\n\
     </session_context>\n";

#[test]
fn r916_06_caller_context_does_not_hijack_the_request() {
    let prompt = format!("{GEMINI_SESSION_CONTEXT}Write a hello world program in Python.");
    let messages = vec![ChatMessage::user(&prompt)];

    // The framing is the client describing itself, so it is not the request the
    // planner reads.
    assert_eq!(
        latest_user_request(&messages).as_deref(),
        Some("Write a hello world program in Python."),
        "the caller's own block is not the user speaking"
    );
    assert!(
        plan_chat_step(&messages, &["write_file", "run_shell_command"]).is_none(),
        "`Today's date is …` inside a context block is not a question about the date"
    );

    // …so the run reaches the artifact the user actually asked for.
    let call = first_agentic_call(&messages, &["write_file", "run_shell_command"]);
    assert_eq!(call, "write_file(main.py)", "{prompt}");
}

#[test]
fn r916_07_a_declarative_statement_of_fact_is_not_a_request() {
    // Statements supplying a fact must not fire the `date` intent; genuine
    // questions still must.
    for statement in [
        "Today's date is Sunday, August 2, 2026.",
        "The current time is 20:00.",
    ] {
        let prompt = format!("{statement}\nWrite a hello world program in Python.");
        let messages = vec![ChatMessage::user(&prompt)];
        let call = first_agentic_call(&messages, &["write_file", "run_shell_command"]);
        assert_eq!(
            call, "write_file(main.py)",
            "{statement} states a fact; it does not ask for one"
        );
    }

    for question in ["What is the date?", "What's the date?"] {
        let messages = vec![ChatMessage::user(question)];
        let planned = planned_command(&messages, &["run_shell_command"]);
        assert_eq!(
            planned, "run_shell_command(date)",
            "a real question still asks the shell: {question}"
        );
    }
}

// ---------------------------------------------------------------------------
// R916-08 — `--global` writes a complete headless configuration
// ---------------------------------------------------------------------------

#[test]
fn r916_08_global_configuration_selects_an_auth_type_for_gemini() {
    let integrations = client_integrations();
    let gemini = integrations
        .iter()
        .find(|integration| integration.id == "gemini")
        .expect("gemini integration");

    let companion = gemini
        .global_config
        .companion_files
        .iter()
        .find(|file| file.path.ends_with(".gemini/settings.json"))
        .expect("gemini --global must materialise ~/.gemini/settings.json (issue #909)");
    assert!(
        companion.json_settings.iter().any(
            |(key, value)| key == "security.auth.selectedType" && value == "{google_auth_type}"
        ),
        "gemini-cli treats an auth type as selected only from a settings file: {:?}",
        companion.json_settings
    );
}

#[test]
fn r916_08_global_configuration_completes_the_openai_triple_for_qwen() {
    let integrations = client_integrations();
    let qwen = integrations
        .iter()
        .find(|integration| integration.id == "qwen")
        .expect("qwen integration");

    for key in ["OPENAI_API_KEY", "OPENAI_BASE_URL", "OPENAI_MODEL"] {
        assert!(
            qwen.global_config
                .shell_env
                .iter()
                .any(|env| env.key == key),
            "qwen selects the OpenAI auth path only from the complete triple; {key} is missing"
        );
    }
}
