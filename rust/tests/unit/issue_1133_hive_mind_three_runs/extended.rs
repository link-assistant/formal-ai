use super::*;

#[test]
fn the_claude_code_tool_set_reads_the_issue_with_gh_not_a_nested_model() {
    let messages = vec![ChatMessage::user(prompt())];
    let calls = calls(plan_chat_step(&messages, CLAUDE_TOOLS.as_slice()));
    assert_eq!(calls[0].tool, "Bash", "{calls:?}");
    assert!(
        command_of(&calls[0]).starts_with(&format!("gh issue view '{ISSUE}'")),
        "{calls:?}"
    );
}

#[test]
fn a_fetch_only_client_gets_an_extraction_prompt_not_the_solve_request() {
    let messages = vec![ChatMessage::user(prompt())];
    let calls = calls(plan_chat_step(&messages, &["Write", "WebFetch"]));
    assert_eq!(calls[0].tool, "WebFetch", "{calls:?}");
    let arguments: serde_json::Value = serde_json::from_str(&calls[0].arguments).unwrap();
    assert_eq!(arguments["url"], ISSUE);
    let fetch_prompt = arguments["prompt"].as_str().unwrap();
    assert!(
        fetch_prompt.contains("work-item title and body"),
        "{fetch_prompt}"
    );
    assert!(fetch_prompt.contains("Do not solve"), "{fetch_prompt}");
    assert_ne!(fetch_prompt, prompt());
}

#[test]
fn a_non_shell_run_alias_does_not_displace_the_fetch_tool() {
    for run_alias in ["computer_use", "code_interpreter"] {
        let messages = vec![ChatMessage::user(prompt())];
        let calls = calls(plan_chat_step(&messages, &["Write", run_alias, "WebFetch"]));
        assert_eq!(calls[0].tool, "WebFetch", "{run_alias}: {calls:?}");
    }
}

#[test]
fn a_failed_or_empty_gh_read_falls_back_to_fetch_once() {
    for result in ["Error: gh: repository unavailable", ""] {
        let mut messages = vec![ChatMessage::user(prompt())];
        let gh = calls(plan_chat_step(&messages, CLAUDE_TOOLS.as_slice()))
            .into_iter()
            .next()
            .expect("gh read");
        record(&mut messages, "gh", &gh, &gh.arguments, result);
        let fallback = calls(plan_chat_step(&messages, CLAUDE_TOOLS.as_slice()));
        assert_eq!(fallback[0].tool, "WebFetch", "{result:?}: {fallback:?}");
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&fallback[0].arguments).unwrap()["url"],
            ISSUE
        );
    }
}

#[test]
fn a_fetch_only_client_progresses_past_the_read_without_refetching() {
    let mut messages = vec![ChatMessage::user(prompt())];
    let kotlin = page("Kotlin");
    let (planned, answer) = drive(
        &["Write", "WebFetch"],
        &mut messages,
        |tool, _| {
            if tool == "WebFetch" {
                kotlin.clone()
            } else {
                String::new()
            }
        },
        12,
    );
    assert_eq!(
        planned
            .iter()
            .filter(|call| call.tool == "WebFetch")
            .count(),
        1,
        "{planned:?}"
    );
    assert!(
        planned.iter().any(|call| {
            call.tool == "Write" && path_of(call) == ".formal-ai/general-change-plan.lino"
        }),
        "{planned:?}"
    );
    assert!(answer.is_some(), "fetch-only planning must terminate");
}

#[test]
fn the_full_claude_code_drive_reads_builds_verifies_and_pushes_kotlin() {
    let mut messages = vec![ChatMessage::user(prompt())];
    let kotlin = page("Kotlin");
    let (planned, answer) = drive(
        CLAUDE_TOOLS.as_slice(),
        &mut messages,
        |tool, arguments| {
            if is_work_item_read(tool, arguments) {
                kotlin.clone()
            } else if tool == "Bash" && arguments.contains("git push") {
                "feed1234\n".to_owned()
            } else if tool == "Bash" {
                "Hello, World!\n".to_owned()
            } else {
                String::new()
            }
        },
        24,
    );
    assert_eq!(
        planned
            .iter()
            .filter(|call| is_work_item_read(&call.tool, &call.arguments))
            .count(),
        1,
        "{planned:?}"
    );
    for path in [
        "Main.kt",
        "tests/verify-output.sh",
        ".github/workflows/run.yml",
    ] {
        assert!(
            planned.iter().any(|call| path_of(call) == path),
            "{path}: {planned:?}"
        );
    }
    assert!(
        planned
            .iter()
            .map(command_of)
            .any(|command| command.contains("git push")),
        "{planned:?}"
    );
    let answer = answer.expect("full Claude drive terminates");
    assert!(answer.contains("Committed and pushed"), "{answer}");
}

#[test]
fn a_gh_read_does_not_count_as_literal_file_verification() {
    let mut messages = vec![ChatMessage::user(prompt())];
    let objective = "Create a file note.txt containing exactly: hello";
    let (planned, answer) = drive(
        &["Bash", "Write"],
        &mut messages,
        |tool, arguments| {
            if is_work_item_read(tool, arguments) {
                objective.to_owned()
            } else if tool == "Bash" && arguments.contains("cat note.txt") {
                "hello\n".to_owned()
            } else {
                String::new()
            }
        },
        8,
    );
    let commands: Vec<String> = planned.iter().map(command_of).collect();
    assert!(
        commands
            .iter()
            .any(|command| command.contains("gh issue view")),
        "{commands:?}"
    );
    assert!(
        commands.iter().any(|command| command == "cat note.txt"),
        "{commands:?}"
    );
    assert!(
        answer
            .expect("literal file drive terminates")
            .contains("Created and verified"),
        "{planned:?}"
    );
}

#[test]
fn pull_request_fetch_prompts_for_the_description_not_an_issue_body() {
    let request = format!(
        "Resolve the GitHub pull request at {PR} in this repository.\nImplement and verify the solution before reporting completion."
    );
    let messages = vec![ChatMessage::user(request)];
    let calls = calls(plan_chat_step(&messages, &["Write", "WebFetch"]));
    assert_eq!(calls[0].tool, "WebFetch", "{calls:?}");
    let arguments: serde_json::Value = serde_json::from_str(&calls[0].arguments).unwrap();
    assert_eq!(arguments["url"], PR);
    assert!(
        arguments["prompt"]
            .as_str()
            .unwrap()
            .contains("pull request, the body is its description"),
        "{arguments}"
    );
}

#[test]
fn a_fetch_the_harness_echoed_without_its_url_is_not_planned_again() {
    // Before the fix the harness echoed `{"target":""}` for the click and the
    // planner, finding no `url` in it, planned the same fetch forever.
    let mut messages = vec![ChatMessage::user(prompt())];
    let click = PlannedToolCall {
        tool: "mcp__playwright__browser_click".to_owned(),
        arguments: String::new(),
    };
    let error =
        "Error: browserBackend.callTool: Unexpected token \"\" while parsing css selector \"\".";
    record(&mut messages, "c0", &click, "{\"target\":\"\"}", error);
    record(&mut messages, "c1", &click, "{\"target\":\"\"}", error);
    let tools = [
        "Bash",
        "Write",
        "mcp__playwright__browser_click",
        "mcp__playwright__browser_navigate",
    ];
    match plan_chat_step(&messages, tools.as_slice()) {
        Some(AgenticPlan::ToolCalls(calls)) => {
            assert!(
                calls
                    .iter()
                    .all(|call| !call.tool.starts_with("mcp__playwright__")),
                "a browser tool is not a fetch tool: {calls:?}"
            );
        }
        Some(AgenticPlan::Final(_)) | None => {}
    }
}

#[test]
fn a_tool_that_failed_twice_with_the_same_report_is_not_called_a_third_time() {
    let mut messages = vec![ChatMessage::user(prompt())];
    let fetch = PlannedToolCall {
        tool: "WebFetch".to_owned(),
        arguments: format!("{{\"url\":\"{ISSUE}\",\"format\":\"text\"}}"),
    };
    let error = "Error: fetch failed: connect ECONNREFUSED";
    record(&mut messages, "c0", &fetch, &fetch.arguments, error);
    record(&mut messages, "c1", &fetch, &fetch.arguments, error);
    match plan_chat_step(&messages, CLAUDE_TOOLS.as_slice()) {
        Some(AgenticPlan::ToolCalls(calls)) => {
            assert!(
                calls.iter().all(|call| call.tool != "WebFetch"),
                "{calls:?}"
            );
        }
        Some(AgenticPlan::Final(answer)) => {
            assert!(!answer.contains("Created and verified"), "{answer}");
        }
        None => panic!("the work item still has a plan after a failed read"),
    }
}

// ---- F2 + F3: the Scala run ----------------------------------------------

#[test]
fn a_failing_verification_command_is_reported_as_the_failed_step_not_as_verified() {
    let mut messages = vec![ChatMessage::user(prompt())];
    let scala = page("Scala");
    let (planned, answer) = drive(
        AGENT_TOOLS.as_slice(),
        &mut messages,
        |tool, arguments| match tool {
            _ if is_work_item_read(tool, arguments) => scala.clone(),
            // The Agent CLI hands the model the process text only; `exit: 127`
            // stays in metadata it never sends.
            "bash" => "/bin/sh: 1: scalac: not found\n".to_owned(),
            _ => String::new(),
        },
        8,
    );
    let answer = answer.expect("the run must end in a final answer");
    assert!(
        planned
            .iter()
            .any(|call| call.tool == "write" && path_of(call) == "Main.scala"),
        "{planned:?}"
    );
    assert!(!answer.contains("Created and verified"), "{answer}");
    assert!(
        answer.contains("scalac Main.scala") && answer.contains("not found"),
        "{answer}"
    );
}

#[test]
fn a_verified_work_item_is_committed_and_pushed_to_the_named_branch() {
    let mut messages = vec![ChatMessage::user(prompt())];
    let scala = page("Scala");
    let (planned, answer) = drive(
        AGENT_TOOLS.as_slice(),
        &mut messages,
        |tool, arguments| match tool {
            _ if is_work_item_read(tool, arguments) => scala.clone(),
            "bash" if arguments.contains("git push") => "abc123def\n".to_owned(),
            "bash" if arguments.contains("scala Main") => "Hello, World!\n".to_owned(),
            _ => String::new(),
        },
        10,
    );
    let commit = planned
        .iter()
        .map(command_of)
        .find(|command| command.starts_with("git add --") && command.contains("git commit --only"))
        .expect("the recipe ends with a commit step");
    assert!(
        commit.contains(&format!("git push -q origin '{BRANCH}'")),
        "{commit}"
    );
    assert!(
        commit.contains("feat: add Main.scala") && commit.contains(ISSUE),
        "{commit}"
    );
    let answer = answer.expect("final answer");
    assert!(
        answer.contains("Committed and pushed") && answer.contains("abc123def"),
        "{answer}"
    );
    // The verification commands come before the commit, never after it.
    let commands: Vec<String> = planned
        .iter()
        .map(command_of)
        .filter(|c| !c.is_empty())
        .collect();
    assert_eq!(
        commands
            .last()
            .map(String::as_str)
            .map(|c| c.starts_with("git add")),
        Some(true),
        "{commands:?}"
    );
}

#[test]
fn a_work_item_that_asks_for_a_workflow_writes_one_with_the_verified_commands() {
    let mut messages = vec![ChatMessage::user(prompt())];
    let scala = page("Scala");
    let (planned, _) = drive(
        AGENT_TOOLS.as_slice(),
        &mut messages,
        |tool, arguments| {
            if is_work_item_read(tool, arguments) {
                scala.clone()
            } else {
                "Hello, World!\n".to_owned()
            }
        },
        10,
    );
    let workflow = planned
        .iter()
        .find(|call| call.tool == "write" && path_of(call) == ".github/workflows/run.yml")
        .expect("the workflow is written beside the program");
    assert!(
        workflow.arguments.contains("pull_request")
            && workflow.arguments.contains("scalac Main.scala"),
        "{workflow:?}"
    );
}

#[test]
fn a_request_to_commit_the_tree_is_a_git_step_not_a_web_search() {
    let restart = "🔄 Auto-restart: resume the previous session and handle its uncommitted changes.\n\nUncommitted files (1):\n?? Main.scala\n\nChanges summary:\nNo tracked-file diff summary available.\n\nPlease review these changes and commit them with an appropriate commit message.\nFollow the repository's commit message conventions from previous commits.";
    let messages = vec![ChatMessage::user(restart)];
    let calls = calls(plan_chat_step(&messages, AGENT_TOOLS.as_slice()));
    assert_eq!(calls[0].tool, "bash", "{calls:?}");
    let command = command_of(&calls[0]);
    assert!(
        command.starts_with("git add -A && git commit -q -m 'feat: add Main.scala'"),
        "{command}"
    );
    assert!(command.contains("git push -q origin HEAD"), "{command}");
}

// ---- F4: the Rust run -----------------------------------------------------

#[test]
fn codex_reads_the_issue_through_gh_when_the_only_fetch_tool_is_a_remote_connector() {
    let messages = vec![ChatMessage::user(prompt())];
    let calls = calls(plan_chat_step(&messages, CODEX_TOOLS.as_slice()));
    assert_eq!(calls[0].tool, "shell", "{calls:?}");
    let command = command_of(&calls[0]);
    assert!(
        command.starts_with(&format!("gh issue view '{ISSUE}' --json title")),
        "{command}"
    );
}

#[test]
fn codex_plans_the_work_item_and_never_narrates_the_connector_placeholder() {
    let mut messages = vec![ChatMessage::user(prompt().replace("Scala", "Rust"))];
    let rust = page("Rust");
    let (planned, answer) = drive(
        CODEX_TOOLS.as_slice(),
        &mut messages,
        |tool, arguments| match tool {
            "shell" if arguments.contains("gh issue view") => rust.clone(),
            "shell" if arguments.contains("git push") => "0123abcd\n".to_owned(),
            "shell" => "Hello, World!\n".to_owned(),
            _ => String::new(),
        },
        10,
    );
    assert!(
        planned.iter().any(|call| call.tool == "apply_patch"),
        "{planned:?}"
    );
    let answer = answer.expect("final answer");
    assert!(!answer.contains("command completed. Output"), "{answer}");
    assert!(answer.contains("Committed and pushed"), "{answer}");
}

#[test]
fn a_connector_result_keeps_its_structured_content() {
    // The connector answered `Action completed.` in `content` and the issue in
    // `structuredContent`; the issue is what the planner reads.
    let mut messages = vec![ChatMessage::user(prompt().replace("Scala", "Rust"))];
    let fetch = PlannedToolCall {
        tool: "mcp__codex_apps__github_fetch".to_owned(),
        arguments: format!("{{\"url\":\"{ISSUE}\"}}"),
    };
    let envelope = "{\"content\":[{\"type\":\"text\",\"text\":\"Action completed.\"}],\"structuredContent\":{\"content\":\"{\\\"number\\\":1,\\\"title\\\":\\\"Implement Hello World in Rust\\\",\\\"body\\\":\\\"## Task\\\\nPlease implement a Hello World program in Rust.\\\"}\"}}";
    record(&mut messages, "c0", &fetch, &fetch.arguments, envelope);
    let tools = ["write", "bash", "mcp__codex_apps__github_fetch"];
    let calls = calls(plan_chat_step(&messages, tools.as_slice()));
    assert_eq!(calls[0].tool, "write", "{calls:?}");
    assert!(calls[0].arguments.contains("fn main()"), "{calls:?}");
}

// ---- F5: the harness summary envelope ------------------------------------

#[test]
fn a_text_to_summarize_envelope_is_summarized_never_executed() {
    let envelope = format!(
        "\n              The following is the text to summarize:\n              <text>\n              {}              </text>\n            ",
        prompt()
    );
    let messages = vec![ChatMessage::user(envelope)];
    let summary = final_text(plan_chat_step(&messages, AGENT_TOOLS.as_slice()));
    assert_eq!(
        summary,
        format!("Resolve the GitHub issue at {ISSUE} in this repository.")
    );
}

// ---- F6: #1115 / #1116 ------------------------------------------------------

#[test]
fn an_additive_edit_composes_to_an_edit_of_its_anchor_line() {
    let (target, old, new) = compose_edit_request(
        "Edit the tracked file self-development-status.yml: add a line \"  pull_request:\" directly after the line \"on:\"",
    )
    .expect("an additive edit composes");
    assert_eq!(target, "self-development-status.yml");
    assert_eq!(old, "on:");
    assert_eq!(new, "on:\n  pull_request:");
}

#[test]
fn an_escaped_newline_in_edit_prose_is_a_newline() {
    let (_, old, new) = compose_edit_request(
        "In the file self-development-status.yml replace \"  schedule:\" with \"  pull_request:\\n  schedule:\"",
    )
    .expect("a replacement composes");
    assert_eq!(old, "  schedule:");
    assert_eq!(new, "  pull_request:\n  schedule:");
}

#[test]
fn an_edit_instruction_is_never_routed_to_web_search() {
    let messages = vec![ChatMessage::user(
        "Edit the tracked file self-hosting-attribution.rs: add \"Gemfile.lock\" to the LOCKFILE_NAMES list.",
    )];
    if let Some(AgenticPlan::ToolCalls(calls)) = plan_chat_step(&messages, AGENT_TOOLS.as_slice()) {
        assert!(
            calls
                .iter()
                .all(|call| call.tool != "websearch" && call.tool != "webfetch"),
            "{calls:?}"
        );
    }
}

// ---- The seeded words, in every registered language ------------------------

/// A request to commit, in each language: English, Russian, Hindi, Chinese,
/// Spanish. Each is a git step.
#[test]
fn a_commit_request_is_a_git_step_in_every_language() {
    let requests = [
        (
            "english",
            "Please review these changes and commit them.\n?? Main.scala",
        ),
        (
            "russian",
            "Проверь изменения и закоммить изменения.\n?? Main.scala",
        ),
        (
            "hindi",
            "इन परिवर्तनों की समीक्षा करें और बदलाव कमिट करें।\n?? Main.scala",
        ),
        ("chinese", "请检查并提交这些更改。\n?? Main.scala"),
        ("spanish", "Revisa los cambios y haz commit.\n?? Main.scala"),
    ];
    for (language, request) in requests {
        let messages = vec![ChatMessage::user(request)];
        let calls = calls(plan_chat_step(&messages, AGENT_TOOLS.as_slice()));
        assert_eq!(calls[0].tool, "bash", "{language}: {calls:?}");
        assert!(
            command_of(&calls[0]).contains("git commit"),
            "{language}: {calls:?}"
        );
    }
}

/// The branch cue in each language names where the work item is pushed.
#[test]
fn the_named_branch_is_pushed_to_in_every_language() {
    let prompts = [
        (
            "english",
            format!(
                "Resolve the GitHub issue at {ISSUE} in this repository.\nKeep the solution on branch {BRANCH}."
            ),
        ),
        (
            "russian",
            format!(
                "Реши задачу GitHub {ISSUE} в этом репозитории.\nДержи решение в ветке {BRANCH}."
            ),
        ),
        (
            "hindi",
            format!("इस रिपॉज़िटरी में GitHub इश्यू {ISSUE} को हल करें।\nसमाधान को शाखा {BRANCH} पर रखें।"),
        ),
        (
            "chinese",
            format!("在此仓库中解决 GitHub 问题 {ISSUE}。\n将解决方案保留在分支 {BRANCH} 上。"),
        ),
        (
            "spanish",
            format!(
                "Resuelve el issue de GitHub {ISSUE} en este repositorio.\nMantén la solución en la rama {BRANCH}."
            ),
        ),
    ];
    let scala = page("Scala");
    for (language, request) in prompts {
        let mut messages = vec![ChatMessage::user(request)];
        let (planned, _) = drive(
            AGENT_TOOLS.as_slice(),
            &mut messages,
            |tool, arguments| {
                if is_work_item_read(tool, arguments) {
                    scala.clone()
                } else {
                    "Hello, World!\n".to_owned()
                }
            },
            10,
        );
        let commit = planned
            .iter()
            .map(command_of)
            .find(|command| {
                command.starts_with("git add --") && command.contains("git commit --only")
            })
            .unwrap_or_else(|| panic!("{language}: no commit step in {planned:?}"));
        assert!(
            commit.contains(&format!("git push -q origin '{BRANCH}'")),
            "{language}: {commit}"
        );
    }
}

/// A workflow requirement in each language adds the workflow file.
#[test]
fn a_workflow_requirement_adds_the_workflow_in_every_language() {
    let bodies = [
        (
            "english",
            "Please implement a Hello World program in Scala. Create a GitHub Actions workflow that runs it.",
        ),
        (
            "russian",
            "Реализуй программу Hello World на Scala. Создай рабочий процесс GitHub Actions, который её запускает.",
        ),
        (
            "hindi",
            "Scala में Hello World प्रोग्राम लागू करें। एक GitHub Actions वर्कफ़्लो बनाएँ जो उसे चलाए।",
        ),
        (
            "chinese",
            "请用 Scala 实现 Hello World 程序。创建一个运行它的 GitHub Actions 工作流。",
        ),
        (
            "spanish",
            "Implementa un programa Hello World en Scala. Crea un flujo de trabajo de GitHub Actions que lo ejecute.",
        ),
    ];
    for (language, body) in bodies {
        let mut messages = vec![ChatMessage::user(prompt())];
        let (planned, _) = drive(
            AGENT_TOOLS.as_slice(),
            &mut messages,
            |tool, arguments| {
                if is_work_item_read(tool, arguments) {
                    body.to_owned()
                } else {
                    "Hello, World!\n".to_owned()
                }
            },
            10,
        );
        assert!(
            planned
                .iter()
                .any(|call| call.tool == "write" && path_of(call) == ".github/workflows/run.yml"),
            "{language}: {planned:?}"
        );
    }
}

/// The position cues in each language place the new line after or before its
/// anchor.
#[test]
fn positional_inserts_compose_in_every_language() {
    let after = [
        (
            "english",
            "In config.yml add the line \"  pull_request:\" directly after the line \"on:\"",
        ),
        (
            "russian",
            "В файле config.yml добавь строку \"  pull_request:\" сразу после строки \"on:\"",
        ),
        (
            "hindi",
            "config.yml में पंक्ति \"on:\" के बाद \"  pull_request:\" जोड़ें",
        ),
        (
            "chinese",
            "在 config.yml 中，在 \"on:\" 行后添加 \"  pull_request:\"",
        ),
        (
            "spanish",
            "En config.yml añade la línea \"  pull_request:\" justo después de la línea \"on:\"",
        ),
    ];
    for (language, request) in after {
        let (target, old, new) =
            compose_edit_request(request).unwrap_or_else(|| panic!("{language}: {request}"));
        assert_eq!(target, "config.yml", "{language}");
        assert_eq!(old, "on:", "{language}");
        assert!(
            new == "on:\n  pull_request:" || new == "  pull_request:\non:",
            "{language}: {new:?}"
        );
    }
    let before = [
        (
            "english",
            "In config.yml insert \"# generated\" right before the line \"on:\"",
        ),
        (
            "russian",
            "В файле config.yml вставь \"# generated\" прямо перед строкой \"on:\"",
        ),
        (
            "spanish",
            "En config.yml inserta \"# generated\" justo antes de la línea \"on:\"",
        ),
    ];
    for (language, request) in before {
        let (_, old, new) =
            compose_edit_request(request).unwrap_or_else(|| panic!("{language}: {request}"));
        assert_eq!(old, "on:", "{language}");
        assert_eq!(new, "# generated\non:", "{language}");
    }
}
