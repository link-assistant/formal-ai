//! Incremental dispatch to real agent CLIs: split what failed, escalate what
//! cannot be split.
//!
//! The unit coverage in `tests/unit/issue_991_incremental_decomposition.rs`
//! pins the protocol against an in-process tool. These tests drive the same
//! protocol through the process boundary that matters in practice -- an
//! external agent CLI receiving `{task}` on its command line, editing a copy of
//! the workspace, and having its effects composed only when it passed.

#![cfg(unix)]

use std::fs;
use std::os::unix::fs::PermissionsExt as _;
use std::path::Path;

use formal_ai::orchestration::{
    dispatch_agents, AgentCommand, AgentRunPermission, DispatchConfig, DispatchMode,
    VerificationCommand,
};

use super::issue_703_orchestration::TestWorkspace;

/// A task the shipped splitter turns into two independently checkable pieces.
const COMPOUND_TASK: &str = "Add a paths-ignore filter for experiments to release.yml \
     and make docs-changed respect excluded_folders.";

/// A task with nothing to split off, so only extension can rescue it.
const ATOMIC_TASK: &str = "Add dev/log/ to the excluded_folders array.";

/// A CLI that can only carry a task up to `limit` characters.
///
/// This is what "the task was too big" looks like from outside: nothing is
/// unsupported, there is simply more of it than one session can do. Work
/// already recorded in `done.txt` counts, so the same CLI can finish the whole
/// task once its pieces are done -- which is exactly the climb back up.
fn size_limited_cli(directory: &Path, name: &str, limit: usize) -> AgentCommand {
    script(
        directory,
        name,
        &format!(
            concat!(
                "#!/bin/sh\n",
                "task=\"$1\"\n",
                "if [ ${{#task}} -gt {limit} ] && [ ! -s done.txt ]; then\n",
                "  echo \"task of ${{#task}} characters exceeds {limit}\" >&2\n",
                "  exit 7\n",
                "fi\n",
                "printf '%s\\n' \"$task\" >> done.txt\n",
            ),
            limit = limit
        ),
    )
}

/// A CLI that refuses every task it is given.
fn refusing_cli(directory: &Path, name: &str) -> AgentCommand {
    script(
        directory,
        name,
        "#!/bin/sh\necho 'this cli cannot do that' >&2\nexit 7\n",
    )
}

/// A CLI that solves anything.
fn capable_cli(directory: &Path, name: &str) -> AgentCommand {
    script(
        directory,
        name,
        "#!/bin/sh\nprintf '%s\\n' \"$1\" >> done.txt\n",
    )
}

/// A CLI whose parent retry destroys effects that its children composed.
fn composition_regressing_cli(directory: &Path, name: &str) -> AgentCommand {
    script(
        directory,
        name,
        concat!(
            "#!/bin/sh\n",
            "task=\"$1\"\n",
            "printf '%s\\n' \"$task\" > .current-task\n",
            "case \"$task\" in\n",
            "  *' and '*)\n",
            "    if [ -f left.done ] && [ -f right.done ]; then\n",
            "      rm -f left.done\n",
            "      printf 'parent retry regressed composed effects\\n' > regressed.txt\n",
            "    fi\n",
            "    ;;\n",
            "  *paths-ignore*) printf 'left\\n' > left.done ;;\n",
            "  *docs-changed*) printf 'right\\n' > right.done ;;\n",
            "  *) exit 9 ;;\n",
            "esac\n",
        ),
    )
}

fn script(directory: &Path, name: &str, body: &str) -> AgentCommand {
    let path = directory.join(name);
    fs::write(&path, body).unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
    AgentCommand::new(&path).arg("{task}")
}

fn incremental_config(
    task: &str,
    workspace: &TestWorkspace,
    clis: &[(&str, AgentCommand)],
) -> DispatchConfig {
    let mut config = DispatchConfig::new(
        task,
        workspace.path(),
        clis.iter().map(|(cli, _)| (*cli).to_string()).collect(),
    );
    config.mode = DispatchMode::Incremental;
    config.permission = AgentRunPermission::grant_for(workspace.path());
    for (cli, command) in clis {
        config
            .allowlisted_agent_commands
            .insert(command.program.to_string_lossy().into_owned());
        config
            .command_overrides
            .insert((*cli).to_string(), command.clone());
    }
    config
}

#[test]
fn a_task_too_big_for_the_cli_is_split_from_its_failure_and_composed_back_up() {
    let bin = TestWorkspace::new("incremental-bin");
    let workspace = TestWorkspace::new("incremental-split");
    let config = incremental_config(
        COMPOUND_TASK,
        &workspace,
        &[("codex", size_limited_cli(bin.path(), "codex", 70))],
    );

    let report = dispatch_agents(&config).expect("incremental dispatch");

    assert_eq!(report.mode, DispatchMode::Incremental);
    let trace = report.incremental.as_ref().expect("an incremental trace");
    assert!(trace.solved, "{trace:?}");
    assert_eq!(trace.split_depth_reached, 1);
    assert!(trace.blocked_tasks.is_empty());

    assert_eq!(trace.splits.len(), 1, "{:?}", trace.splits);
    assert_eq!(trace.splits[0].task, COMPOUND_TASK);
    assert_eq!(trace.splits[0].children.len(), 2);
    assert!(
        trace.splits[0].failure_evidence.contains("exit:7"),
        "the split must be justified by the failure that caused it: {}",
        trace.splits[0].failure_evidence
    );

    assert_eq!(trace.steps.len(), 4, "{:?}", trace.steps);
    assert_eq!(trace.steps[0].task, COMPOUND_TASK);
    assert!(
        !trace.steps[0].passed,
        "the whole task must be attempted before anything is split"
    );
    assert!(trace.steps[1..].iter().all(|step| step.passed));
    assert_eq!(
        trace.steps.last().map(|step| step.task.as_str()),
        Some(COMPOUND_TASK),
        "the parent is retried once its pieces pass"
    );

    // The pieces' work reached the real workspace, in composition order, and
    // the retried parent saw it there.
    let done = fs::read_to_string(workspace.path().join("done.txt")).unwrap();
    let lines = done.lines().collect::<Vec<_>>();
    assert_eq!(lines.len(), 3, "{done}");
    assert_eq!(lines[2], COMPOUND_TASK);
    assert!(report
        .composed_changes
        .iter()
        .any(|change| change.path == "done.txt"));
    for step in &trace.steps {
        assert!(
            config.output_dir.join(&step.session_file).is_file(),
            "every attempt must leave replayable evidence: {}",
            step.session_file
        );
    }

    // The same sessions are learning input, not dead-end execution logs. The
    // learner may only propose a contract amendment; the run cannot approve
    // its own observation or silently mutate the client registry.
    let learning = fs::read_to_string(config.output_dir.join("learning.lino"))
        .expect("incremental execution must emit its proposal-only learning artifact");
    assert!(learning.contains("human_gated \"true\""), "{learning}");
    assert!(
        learning.contains(&format!("observation_count \"{}\"", trace.steps.len())),
        "{learning}"
    );
    assert!(
        learning.contains("decision \"awaiting_human_review\"")
            || learning.contains("decision \"no_reviewable_change\""),
        "{learning}"
    );
}

#[test]
fn verified_child_composition_is_not_regressed_by_a_redundant_parent_retry() {
    let bin = TestWorkspace::new("incremental-composition-bin");
    let workspace = TestWorkspace::new("incremental-composition");
    fs::write(
        workspace.path().join("verify.sh"),
        concat!(
            "#!/bin/sh\n",
            "task=${FORMAL_AI_VERIFICATION_TASK:-$(cat .current-task)}\n",
            "case \"$task\" in\n",
            "  *' and '*) test -f left.done && test -f right.done ;;\n",
            "  *paths-ignore*) test -f left.done ;;\n",
            "  *docs-changed*) test -f right.done ;;\n",
            "  *) exit 9 ;;\n",
            "esac\n",
        ),
    )
    .unwrap();
    let mut config = incremental_config(
        COMPOUND_TASK,
        &workspace,
        &[("codex", composition_regressing_cli(bin.path(), "codex"))],
    );
    config.allowlisted_commands.insert("sh".to_string());
    config
        .verification
        .push(VerificationCommand::new("sh", ["verify.sh"]));

    let report = dispatch_agents(&config).expect("incremental dispatch");

    let trace = report.incremental.as_ref().expect("an incremental trace");
    assert!(trace.solved, "{trace:#?}");
    assert_eq!(trace.steps.len(), 4, "{:?}", trace.steps);
    assert_eq!(trace.steps.last().unwrap().task, COMPOUND_TASK);
    assert_eq!(trace.steps.last().unwrap().cli, "composed-verifier");
    assert!(trace.steps.last().unwrap().passed);
    assert!(workspace.path().join("left.done").is_file());
    assert!(workspace.path().join("right.done").is_file());
    assert!(!workspace.path().join("regressed.txt").exists());

    let replay: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(
            config
                .output_dir
                .join(&trace.steps.last().unwrap().session_file),
        )
        .unwrap(),
    )
    .unwrap();
    assert!(replay["native_session"].is_null(), "{replay:#?}");
    assert_eq!(replay["program"], "verification-only");

    let learning = fs::read_to_string(config.output_dir.join("learning.lino")).unwrap();
    assert!(learning.contains("observation_count \"3\""), "{learning}");
}

#[test]
fn an_irreducible_failure_escalates_to_the_next_cli_instead_of_stopping() {
    let bin = TestWorkspace::new("incremental-escalation-bin");
    let workspace = TestWorkspace::new("incremental-escalation");
    let config = incremental_config(
        ATOMIC_TASK,
        &workspace,
        &[
            ("codex", refusing_cli(bin.path(), "codex")),
            ("claude", capable_cli(bin.path(), "claude")),
        ],
    );

    let report = dispatch_agents(&config).expect("incremental dispatch");

    let trace = report.incremental.as_ref().expect("an incremental trace");
    assert!(trace.solved, "{trace:?}");
    assert_eq!(trace.split_depth_reached, 0, "an atomic task cannot split");
    assert_eq!(trace.splits.len(), 1);
    assert!(
        trace.splits[0].children.is_empty(),
        "the recorded split says the task is irreducible, which is what \
         justifies reaching for another tool"
    );
    assert_eq!(trace.steps.len(), 2, "{:?}", trace.steps);
    assert_eq!(trace.steps[0].cli, "codex");
    assert!(!trace.steps[0].passed);
    assert_eq!(trace.steps[1].cli, "claude");
    assert!(trace.steps[1].passed);
    assert_eq!(
        fs::read_to_string(workspace.path().join("done.txt")).unwrap(),
        format!("{ATOMIC_TASK}\n"),
        "only the passing attempt may touch the workspace"
    );
}

#[test]
fn a_task_no_cli_can_solve_is_reported_blocked_with_all_of_its_evidence() {
    let bin = TestWorkspace::new("incremental-blocked-bin");
    let workspace = TestWorkspace::new("incremental-blocked");
    let config = incremental_config(
        ATOMIC_TASK,
        &workspace,
        &[
            ("codex", refusing_cli(bin.path(), "codex")),
            ("claude", refusing_cli(bin.path(), "claude")),
        ],
    );

    let report = dispatch_agents(&config).expect("a blocked task is a result, not an error");

    let trace = report.incremental.as_ref().expect("an incremental trace");
    assert!(!trace.solved);
    assert_eq!(trace.blocked_tasks, vec![ATOMIC_TASK.to_string()]);
    assert_eq!(trace.steps.len(), 2, "{:?}", trace.steps);
    assert!(trace.steps.iter().all(|step| !step.passed));
    assert!(
        report.composed_changes.is_empty(),
        "nothing passed, so nothing may be composed"
    );
    assert!(!workspace.path().join("done.txt").exists());

    // The run cannot extend somebody else's CLI, so what it owes review is the
    // evidence: which task, which tools, and what each of them reported.
    let proposal = trace
        .proposals
        .first()
        .expect("a proposal per blocked task");
    assert_eq!(trace.proposals.len(), 1, "{:?}", trace.proposals);
    assert_eq!(proposal.task, ATOMIC_TASK);
    assert_eq!(proposal.tried_clis, vec!["codex", "claude"]);
    assert_eq!(proposal.status, "human_review_required");
    assert_eq!(proposal.failure_evidence.len(), 2);
    assert!(
        proposal
            .failure_evidence
            .iter()
            .all(|evidence| evidence.contains("exit:7")),
        "{:?}",
        proposal.failure_evidence
    );

    let document = fs::read_to_string(config.output_dir.join("proposals.lino"))
        .expect("the proposals are mirrored next to the report");
    assert!(document.contains(&proposal.id), "{document}");
    assert!(
        document.contains("status \"human_review_required\""),
        "{document}"
    );
}

/// A run that solved everything owes review nothing, and says so explicitly.
#[test]
fn a_solved_run_writes_an_empty_proposal_document_rather_than_none_at_all() {
    let bin = TestWorkspace::new("incremental-solved-bin");
    let workspace = TestWorkspace::new("incremental-solved");
    let config = incremental_config(
        ATOMIC_TASK,
        &workspace,
        &[("codex", capable_cli(bin.path(), "codex"))],
    );

    let report = dispatch_agents(&config).expect("a solvable task dispatches");

    let trace = report.incremental.as_ref().expect("an incremental trace");
    assert!(trace.solved);
    assert!(trace.proposals.is_empty());
    let document = fs::read_to_string(config.output_dir.join("proposals.lino"))
        .expect("the document is written even when there is nothing to propose");
    assert_eq!(document.trim(), "incremental_proposals");
}
