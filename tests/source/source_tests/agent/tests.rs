use super::*;

fn config(name: &str) -> AgentWorkspaceConfig {
    config_with_budget(name, Duration::from_secs(1))
}

fn config_with_budget(name: &str, time_budget: Duration) -> AgentWorkspaceConfig {
    AgentWorkspaceConfig {
        base_dir: std::env::temp_dir()
            .join("formal-ai-agent-tests")
            .join(name),
        time_budget,
    }
}

#[test]
fn python_path_resolution_prefers_interpreter_names_before_launcher() {
    let root = std::env::temp_dir().join("formal-ai-agent-tests-python-path");
    let early = root.join("early");
    let late = root.join("late");
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&early).unwrap();
    fs::create_dir_all(&late).unwrap();
    fs::write(early.join("py.exe"), "").unwrap();
    fs::write(late.join("python.exe"), "").unwrap();

    let path = std::env::join_paths([early.as_path(), late.as_path()]).unwrap();
    let resolved =
        resolve_program_from_path_names(&["python3.exe", "python.exe", "py.exe"], Some(path))
            .unwrap();

    assert_eq!(resolved, late.join("python.exe"));
}

#[test]
fn windows_python_commands_have_platform_budget_floor() {
    let configured = Duration::from_secs(5);
    let effective = effective_command_time_budget("python3", configured);

    if cfg!(windows) {
        assert_eq!(effective, WINDOWS_PYTHON_TIME_BUDGET_FLOOR);
    } else {
        assert_eq!(effective, configured);
    }
    assert_eq!(effective_command_time_budget("cat", configured), configured);
}

#[test]
fn planned_prompt_mutates_only_workspace_files() {
    let prompt = "[agent] create file report.txt with `alpha`, modify report.txt to `beta`, \
                      create file scratch.tmp with `remove`, delete scratch.tmp, and run command \
                      `cat report.txt`";
    let run = run_agent_plan(prompt, &config("planned_prompt")).unwrap();

    assert_eq!(run.status, AgentRunStatus::Completed);
    assert_eq!(
        fs::read_to_string(run.workspace.join("report.txt")).unwrap(),
        "beta"
    );
    assert!(!run.workspace.join("scratch.tmp").exists());
    assert_eq!(run.command_results[0].stdout, "beta");
    assert_eq!(run.actions.len(), 5);
}

#[test]
fn run_terminal_command_is_planned_once() {
    let plan = parse_agent_plan("[agent] run terminal command `ls`");

    assert_eq!(
        plan,
        vec![PlannedAgentAction::RunCommand {
            command: "ls".to_owned()
        }]
    );
}

#[test]
fn parent_paths_are_rejected() {
    let prompt = "[agent] create file ../escape.txt with `x`";
    let run = run_agent_plan(prompt, &config("parent_paths")).unwrap();

    assert_eq!(run.status, AgentRunStatus::Failed);
    assert_eq!(run.actions[0].status, AgentActionStatus::Failed);
    assert!(!run.workspace.join("../escape.txt").exists());
}

#[test]
fn command_environment_is_cleared() {
    std::env::set_var("FORMAL_AI_TEST_SECRET", "do-not-leak");
    let prompt = "[agent] run command `env`";
    let run = run_agent_plan(prompt, &config("empty_env")).unwrap();
    std::env::remove_var("FORMAL_AI_TEST_SECRET");

    assert_eq!(run.status, AgentRunStatus::Completed);
    assert!(!run.command_results[0].stdout.contains("do-not-leak"));
}

#[test]
fn python3_command_runs_from_allowlisted_resolved_path() {
    let prompt = "[agent] create file script.py with `print(\"agent_python_ok\")`, \
                      and run command `python3 script.py`";
    let run = run_agent_plan(
        prompt,
        &config_with_budget("python3_command", Duration::from_secs(5)),
    )
    .unwrap();

    assert_eq!(run.status, AgentRunStatus::Completed, "{run:#?}");
    assert_eq!(run.command_results[0].stdout.trim(), "agent_python_ok");
}

#[test]
fn unsupported_commands_are_rejected() {
    let prompt = "[agent] run command `sh -c echo blocked`";
    let run = run_agent_plan(prompt, &config("unsupported_command")).unwrap();

    assert_eq!(run.status, AgentRunStatus::Failed);
    assert!(run.command_results.is_empty());
    assert_eq!(run.actions[0].status, AgentActionStatus::Failed);
    assert!(run.actions[0]
        .detail
        .contains("unsupported sandbox command"));
}
