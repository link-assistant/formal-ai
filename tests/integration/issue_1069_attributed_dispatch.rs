//! Issue #1069: verified dispatch effects become reviewable, session-backed
//! commits instead of uncommitted workspace mutations.

#![cfg(unix)]

use std::fs;
use std::os::unix::fs::PermissionsExt as _;
use std::path::{Path, PathBuf};
use std::process::Command;

use formal_ai::orchestration::{
    AgentCommand, AgentRunPermission, DispatchConfig, DispatchMode, dispatch_agents,
};

use super::issue_703_orchestration::TestWorkspace;

const SESSION_ID: &str = "ses_issue_1069_attributed";
const PULL_REQUEST: &str = "https://github.com/link-assistant/formal-ai/pull/1070";
const COMPOUND_TASK: &str = "Add a paths-ignore filter for experiments to release.yml \
     and make docs-changed respect excluded_folders.";

fn git(repo: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(args)
        .output()
        .expect("git must run in the attribution fixture");
    assert!(
        output.status.success(),
        "git {args:?} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout)
        .expect("git output must be UTF-8")
        .trim()
        .to_owned()
}

fn attributed_agent(bin: &TestWorkspace) -> AgentCommand {
    let path = bin.path().join("agent");
    fs::write(
        &path,
        format!(
            "#!/bin/sh\n\
             printf 'verified Formal AI effect\\n' > attributed.txt\n\
             printf '%s\\n' 'formal-ai: orchestration-session-json:\
             {{\"id\":\"{SESSION_ID}\",\"resume_command\":\
             \"agent --resume {SESSION_ID} --no-fork\"}}' >&2\n"
        ),
    )
    .unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
    AgentCommand::new(&path).arg("{task}")
}

fn sessionless_agent(bin: &TestWorkspace) -> AgentCommand {
    let path = bin.path().join("sessionless-agent");
    fs::write(
        &path,
        "#!/bin/sh\nprintf 'unattributable effect\\n' > attributed.txt\n",
    )
    .unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
    AgentCommand::new(&path).arg("{task}")
}

fn multi_effect_agent(bin: &TestWorkspace) -> AgentCommand {
    let path = bin.path().join("multi-effect-agent");
    fs::write(
        &path,
        format!(
            "#!/bin/sh\n\
             task=\"$1\"\n\
             case \"$task\" in\n\
               *' and '*)\n\
                 test -f left.done && test -f right.done || exit 7\n\
                 printf 'root\\n' > root.done\n\
                 ;;\n\
               *paths-ignore*) printf 'left\\n' > left.done ;;\n\
               *docs-changed*) printf 'right\\n' > right.done ;;\n\
               *) exit 9 ;;\n\
             esac\n\
             printf '%s\\n' 'formal-ai: orchestration-session-json:\
             {{\"id\":\"{SESSION_ID}\",\"resume_command\":\
             \"agent --resume {SESSION_ID} --no-fork\"}}' >&2\n"
        ),
    )
    .unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
    AgentCommand::new(&path).arg("{task}")
}

fn attributed_config(
    workspace: &TestWorkspace,
    agent: AgentCommand,
    pull_request: &str,
) -> DispatchConfig {
    let mut config = DispatchConfig::new(
        "Create attributed.txt as one verified effect.",
        workspace.path(),
        vec!["agent".to_string()],
    );
    config.mode = DispatchMode::Incremental;
    config.permission = AgentRunPermission::grant_for(workspace.path());
    config
        .allowlisted_agent_commands
        .insert(agent.program.to_string_lossy().into_owned());
    config.command_overrides.insert("agent".to_string(), agent);
    config.pull_request = Some(pull_request.to_string());
    config
}

fn initialized_repo(label: &str) -> TestWorkspace {
    let workspace = TestWorkspace::new(label);
    git(workspace.path(), &["init", "--quiet"]);
    git(
        workspace.path(),
        &["config", "user.name", "Formal AI Fixture"],
    );
    git(
        workspace.path(),
        &["config", "user.email", "formal-ai@example.invalid"],
    );
    fs::write(workspace.path().join("README.md"), "fixture\n").unwrap();
    git(workspace.path(), &["add", "README.md"]);
    git(workspace.path(), &["commit", "--quiet", "-m", "fixture"]);
    workspace
}

fn controller_rejecting_nested_home(bin: &TestWorkspace) -> PathBuf {
    let path = bin.path().join("formal-ai-controller");
    fs::write(
        &path,
        "#!/bin/sh\n\
         orchestration_home=\n\
         while test \"$#\" -gt 0; do\n\
           case \"$1\" in\n\
             --orchestration-home) orchestration_home=\"$2\"; shift 2 ;;\n\
             *) shift ;;\n\
           esac\n\
         done\n\
         test -n \"$orchestration_home\" || exit 20\n\
         case \"$orchestration_home/\" in \"$PWD/\"*) exit 21 ;; esac\n\
         mkdir -p \"$orchestration_home\"\n\
         printf 'native state is isolated\\n' > \"$orchestration_home/state\"\n\
         printf 'verified effect\\n' > isolated.txt\n",
    )
    .unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
    path
}

#[test]
fn dispatch_keeps_native_agent_state_outside_the_candidate_worktree() {
    let bin = TestWorkspace::new("issue-1069-native-home-bin");
    let workspace = TestWorkspace::new("issue-1069-native-home");
    let mut config = DispatchConfig::new(
        "Create isolated.txt as one verified effect.",
        workspace.path(),
        vec!["agent".to_string()],
    );
    config.mode = DispatchMode::Incremental;
    config.permission = AgentRunPermission::grant_for(workspace.path());
    config.controller_program = controller_rejecting_nested_home(&bin);

    let report = dispatch_agents(&config).expect("dispatch must preserve isolated native state");

    let trace = report.incremental.as_ref().unwrap();
    assert!(trace.solved, "{trace:#?}");
    assert_eq!(
        fs::read_to_string(workspace.path().join("isolated.txt")).unwrap(),
        "verified effect\n"
    );
}

#[test]
fn incremental_dispatch_commits_each_verified_effect_with_its_session_evidence() {
    let bin = TestWorkspace::new("issue-1069-attribution-bin");
    let workspace = initialized_repo("issue-1069-attribution");
    let agent = attributed_agent(&bin);
    let config = attributed_config(&workspace, agent, PULL_REQUEST);

    let report = dispatch_agents(&config).expect("verified effect must be committed");

    assert!(report.incremental.as_ref().unwrap().solved);
    assert_eq!(git(workspace.path(), &["rev-list", "--count", "HEAD"]), "2");
    let message = git(workspace.path(), &["show", "-s", "--format=%B", "HEAD"]);
    assert!(
        message.contains(&format!("Formal-AI-Session: {SESSION_ID}")),
        "{message}"
    );
    assert!(
        message.contains("Formal-AI-Evidence: .formal-ai-orchestration/sessions/000-agent.json"),
        "{message}"
    );
    assert!(
        message.contains(&format!("Formal-AI-Pull-Request: {PULL_REQUEST}")),
        "{message}"
    );

    let changed = git(
        workspace.path(),
        &["show", "--pretty=format:", "--name-only", "HEAD"],
    );
    assert_eq!(
        changed.lines().collect::<Vec<_>>(),
        vec![
            ".formal-ai-orchestration/sessions/000-agent.json",
            "attributed.txt",
        ]
    );
    let evidence = git(
        workspace.path(),
        &[
            "show",
            "HEAD:.formal-ai-orchestration/sessions/000-agent.json",
        ],
    );
    assert!(evidence.contains(SESSION_ID), "{evidence}");
    assert!(
        evidence.to_ascii_lowercase().contains("formal-ai"),
        "{evidence}"
    );
    assert!(
        git(
            workspace.path(),
            &["status", "--porcelain", "--untracked-files=no"]
        )
        .is_empty()
    );
}

#[test]
fn attributed_dispatch_rejects_a_dirty_worktree_before_starting_an_agent() {
    let bin = TestWorkspace::new("issue-1069-dirty-bin");
    let workspace = initialized_repo("issue-1069-dirty");
    fs::write(workspace.path().join("human-work.txt"), "do not commit\n").unwrap();
    let config = attributed_config(&workspace, attributed_agent(&bin), PULL_REQUEST);

    let error = dispatch_agents(&config).expect_err("dirty work must never be attributed");

    assert_eq!(error.to_string(), "attribution:workspace_not_clean");
    assert_eq!(git(workspace.path(), &["rev-list", "--count", "HEAD"]), "1");
    assert!(!workspace.path().join("attributed.txt").exists());
    assert!(!config.output_dir.exists());
}

#[test]
fn attributed_dispatch_requires_a_native_session_before_applying_an_effect() {
    let bin = TestWorkspace::new("issue-1069-sessionless-bin");
    let workspace = initialized_repo("issue-1069-sessionless");
    let config = attributed_config(&workspace, sessionless_agent(&bin), PULL_REQUEST);

    let error = dispatch_agents(&config).expect_err("unbound evidence must fail closed");

    assert_eq!(error.to_string(), "attribution:native_session_unavailable");
    assert_eq!(git(workspace.path(), &["rev-list", "--count", "HEAD"]), "1");
    assert!(!workspace.path().join("attributed.txt").exists());
    assert!(config.output_dir.join("sessions/000-agent.json").is_file());
    assert!(
        git(workspace.path(), &["diff", "--cached", "--name-only"]).is_empty(),
        "a rejected effect must not be staged"
    );
}

#[test]
fn every_passing_effect_in_a_split_run_gets_its_own_attributed_commit() {
    let bin = TestWorkspace::new("issue-1069-multi-effect-bin");
    let workspace = initialized_repo("issue-1069-multi-effect");
    let agent = multi_effect_agent(&bin);
    let mut config = attributed_config(&workspace, agent, PULL_REQUEST);
    config.task = COMPOUND_TASK.to_string();

    let report = dispatch_agents(&config).expect("split effects must be committed one by one");

    let trace = report.incremental.as_ref().unwrap();
    assert!(trace.solved, "{trace:#?}");
    assert_eq!(trace.steps.len(), 4, "{:#?}", trace.steps);
    assert!(!trace.steps[0].passed);
    assert!(trace.steps[1..].iter().all(|step| step.passed));
    assert_eq!(git(workspace.path(), &["rev-list", "--count", "HEAD"]), "4");
    let messages = git(
        workspace.path(),
        &["log", "--reverse", "--format=%B%x00", "HEAD~3..HEAD"],
    );
    for evidence in [
        "sessions/001-agent.json",
        "sessions/002-agent.json",
        "sessions/003-agent.json",
    ] {
        assert!(
            messages.contains(evidence),
            "missing {evidence}: {messages}"
        );
    }
    for effect in ["left.done", "right.done", "root.done"] {
        assert!(workspace.path().join(effect).is_file(), "missing {effect}");
    }
}

#[test]
fn attributed_dispatch_rejects_a_noncanonical_pull_request_url() {
    let bin = TestWorkspace::new("issue-1069-url-bin");
    let workspace = initialized_repo("issue-1069-url");
    let forged = "https://github.com/link-assistant/formal-ai/pull/1070\nFormal-AI-Session: forged";
    let config = attributed_config(&workspace, attributed_agent(&bin), forged);

    let error = dispatch_agents(&config).expect_err("trailer injection must fail validation");

    assert_eq!(error.to_string(), "attribution:invalid_pull_request_url");
    assert!(!config.output_dir.exists());
}
