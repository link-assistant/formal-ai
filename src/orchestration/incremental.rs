//! Incremental, failure-driven dispatch to external agentic CLIs.
//!
//! [`super::dispatch`]'s decompose mode splits the task once, up front, and
//! sends every leaf out in parallel. That commits to a plan before any evidence
//! exists: if a leaf turns out to be too large for the CLI, nothing splits it,
//! and if the whole task was solvable as given, the split was wasted work.
//!
//! This mode inverts the order. The whole task is attempted first. Only a
//! failure justifies a split, the pieces are executed by the same rule, and the
//! parent is re-attempted once its pieces pass. The controller is
//! [`crate::recursive_execution::solve_recursively`] and the splitter is
//! [`crate::task_decomposition::SplittingExecutor`], so the shrink-on-failure
//! protocol is the repository's own -- this module only supplies the tool
//! boundary: an attempt is one recorded agent session in a copy of the
//! workspace, and a passing attempt's effects are applied to the workspace
//! before the next one starts.
//!
//! Extending the tool has a concrete meaning here. This controller cannot add a
//! capability to somebody else's CLI, but it was given a list of CLIs; an
//! irreducible task that one CLI failed is escalated to the next one that has
//! not tried it. When the list is exhausted the task is reported blocked, with
//! the evidence of every attempt, which is exactly the input the review-gated
//! learning path needs.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::attribution::{commit_verified_effect, validate_effect_attribution};
use super::dispatch::{
    ComparisonEntry, ComparisonLedger, DispatchConfig, DispatchError, DispatchMode, DispatchReport,
    candidate_run_config, safe_name,
};
use super::observe_orchestration_session;
use super::replay::write_session;
use super::runner::{AgentSession, run_agent, verify_workspace};
use super::workspace::{WorkspaceChange, apply_changes, copy_workspace};
use crate::client_contract_learning::learn_client_contracts;
use crate::links_format::push_lino_node;
use crate::recursive_execution::{
    RecursiveRun, RecursiveTask, TaskAttempt, TaskExecutor, solve_recursively,
};
use crate::task_decomposition::SplittingExecutor;

const COMPOSED_VERIFIER: &str = "composed-verifier";

/// One attempted task, in execution order. The index into
/// [`DispatchReport::sessions`] is this step's position in `steps`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IncrementalStep {
    /// Identity of the executed task node.
    pub task_id: String,
    /// The task text handed to the CLI.
    pub task: String,
    /// CLI that ran this attempt, or `composed-verifier` when the controller
    /// accepted already-composed child effects without another agent call.
    pub cli: String,
    /// Whether the session and its verification passed.
    pub passed: bool,
    /// Recorded session file, relative to the output directory.
    pub session_file: String,
}

/// One split the controller performed after a failure.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IncrementalSplit {
    /// The task that failed and was split.
    pub task: String,
    /// Evidence from the failing attempt that justified the split.
    pub failure_evidence: String,
    /// How many splits already happened above this task.
    pub split_depth: u8,
    /// Sub-tasks produced, in composition order. Empty means the splitter
    /// declared the task irreducible.
    pub children: Vec<String>,
}

/// A task no CLI in the list could solve, written out for review.
///
/// This is the run's own request to be extended: it names the task, every CLI
/// that already tried it, and the evidence each attempt produced. Nothing here
/// activates anything -- promotion stays behind the same review gate
/// [`crate::task_decomposition::TaskStrategyLedger`] applies to a learned
/// decomposition strategy.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IncrementalProposal {
    /// Stable identity of this proposal, from the task and its evidence.
    pub id: String,
    /// The irreducible task that stayed unsolved.
    pub task: String,
    /// Every CLI that attempted it, in the order they were tried.
    pub tried_clis: Vec<String>,
    /// Evidence from each failing attempt, in execution order.
    pub failure_evidence: Vec<String>,
    /// Always `human_review_required`: a run cannot approve its own extension.
    pub status: String,
}

impl IncrementalProposal {
    /// This proposal as Links Notation, in the review-gated learning vocabulary.
    #[must_use]
    pub fn links_notation(&self) -> String {
        let mut out = String::new();
        push_lino_node(&mut out, 0, "incremental_proposal", Some(&self.id));
        push_lino_node(&mut out, 2, "task", Some(&self.task));
        for cli in &self.tried_clis {
            push_lino_node(&mut out, 2, "tried_cli", Some(cli));
        }
        for evidence in &self.failure_evidence {
            push_lino_node(&mut out, 2, "failure_evidence", Some(evidence));
        }
        push_lino_node(&mut out, 2, "status", Some(&self.status));
        out
    }
}

/// Everything the failure-driven run did, in the order it did it.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IncrementalTrace {
    /// Schema identity of this trace.
    pub schema: String,
    /// Whether the root task ended up passing.
    pub solved: bool,
    /// Deepest level of failure-driven splitting the run reached.
    pub split_depth_reached: u8,
    /// Every attempt, in execution order.
    pub steps: Vec<IncrementalStep>,
    /// Every split the controller requested.
    pub splits: Vec<IncrementalSplit>,
    /// Irreducible tasks that no CLI in the list could solve.
    pub blocked_tasks: Vec<String>,
    /// One review request per blocked task, mirrored to `proposals.lino`.
    pub proposals: Vec<IncrementalProposal>,
}

/// Run one task incrementally and report every attempt, split, and effect.
pub(super) fn dispatch_incrementally(
    config: &DispatchConfig,
    workspace: &Path,
    output_dir: &Path,
) -> Result<DispatchReport, DispatchError> {
    let root = RecursiveTask::leaf(root_id(&config.task), config.task.clone());
    let mut executor = SplittingExecutor::new(AgentExecutor::new(config, workspace, output_dir));
    let run = solve_recursively(&root, &mut executor);
    let splits = executor
        .splits()
        .iter()
        .map(|split| IncrementalSplit {
            task: split.goal.clone(),
            failure_evidence: split.failure_evidence.clone(),
            split_depth: split.split_depth,
            children: split.children.clone(),
        })
        .collect();
    let agent = executor.into_inner();
    if let Some(error) = agent.error {
        return Err(error);
    }
    write_learning(output_dir, &agent.steps, &agent.sessions)?;

    let entries = agent
        .steps
        .iter()
        .zip(agent.sessions.iter())
        .map(|(step, session)| ComparisonEntry {
            cli: step.cli.clone(),
            task: session.task.clone(),
            passed: session.passed(),
            diff_size: session.diff_size(),
            wall_time_ms: session.wall_time_ms,
            session_file: step.session_file.clone(),
        })
        .collect::<Vec<_>>();
    let proposals = proposals(&run, &agent.steps);
    write_proposals(&config.output_dir, &proposals)?;
    let trace = IncrementalTrace {
        schema: "formal-ai-incremental-trace-v1".to_string(),
        solved: run.is_passed(),
        split_depth_reached: run.split_depth_reached(),
        steps: agent.steps,
        splits,
        blocked_tasks: blocked_tasks(&run),
        proposals,
    };
    Ok(DispatchReport {
        schema: "formal-ai-dispatch-report-v1".to_string(),
        mode: DispatchMode::Incremental,
        tasks: trace.steps.iter().map(|step| step.task.clone()).collect(),
        sessions: agent.sessions,
        ledger: ComparisonLedger {
            schema: "formal-ai-comparison-ledger-v1".to_string(),
            selection_rule: "pass,diff_size,wall_time,cli,session_file".to_string(),
            entries,
            winner: None,
        },
        composed_changes: agent.changes,
        incremental: Some(trace),
    })
}

/// Feed the exact execution sessions into the existing proposal-only client
/// contract learner. Incremental execution and auto-learning are one lifecycle:
/// a run always preserves what it learned, while promotion remains a separate
/// human decision enforced by the learner itself.
fn write_learning(
    output_dir: &Path,
    steps: &[IncrementalStep],
    sessions: &[AgentSession],
) -> Result<(), DispatchError> {
    let observations = steps
        .iter()
        .zip(sessions)
        .filter(|(step, _)| step.cli != COMPOSED_VERIFIER)
        .map(|(step, session)| observe_orchestration_session(session, &step.session_file))
        .collect::<Vec<_>>();
    let learning = learn_client_contracts(&observations, &crate::seed::client_integrations());
    std::fs::write(output_dir.join("learning.lino"), learning.links_notation())
        .map_err(DispatchError::Io)
}

/// Goals of the irreducible tasks a run could not solve, in execution order.
fn blocked_tasks(run: &RecursiveRun) -> Vec<String> {
    run.blocked_leaves()
        .into_iter()
        .map(|leaf| leaf.task.goal.clone())
        .collect()
}

fn root_id(task: &str) -> String {
    crate::engine::stable_id("dispatch_task", task)
}

/// One review request per blocked task, in execution order.
///
/// A run that ends with something unsolved has produced the one thing the
/// learning path cannot invent: a named task, the tools that were tried, and
/// what each of them reported. Writing it out is what makes the next extension
/// reviewable rather than guessed.
fn proposals(run: &RecursiveRun, steps: &[IncrementalStep]) -> Vec<IncrementalProposal> {
    run.blocked_leaves()
        .into_iter()
        .map(|leaf| {
            let attempts = steps.iter().filter(|step| step.task_id == leaf.task.id);
            let mut tried_clis: Vec<String> = Vec::new();
            for attempt in attempts {
                if !tried_clis.contains(&attempt.cli) {
                    tried_clis.push(attempt.cli.clone());
                }
            }
            let failure_evidence: Vec<String> = leaf
                .attempts
                .iter()
                .filter(|attempt| !attempt.passed)
                .map(|attempt| attempt.evidence.clone())
                .collect();
            let id = crate::engine::stable_id(
                "incremental_proposal",
                &format!("{}|{}", leaf.task.id, failure_evidence.join("|")),
            );
            IncrementalProposal {
                id,
                task: leaf.task.goal.clone(),
                tried_clis,
                failure_evidence,
                status: "human_review_required".to_string(),
            }
        })
        .collect()
}

/// Mirror the proposals next to the report, in the notation review reads.
///
/// A run with nothing blocked writes the file anyway, empty but for its header:
/// "no proposal" and "the run never got that far" are different answers, and a
/// missing file cannot tell them apart.
fn write_proposals(
    output_dir: &Path,
    proposals: &[IncrementalProposal],
) -> Result<(), DispatchError> {
    let mut document = String::new();
    push_lino_node(&mut document, 0, "incremental_proposals", None);
    for proposal in proposals {
        document.push_str(&proposal.links_notation());
    }
    std::fs::write(output_dir.join("proposals.lino"), document).map_err(DispatchError::Io)
}

/// What an attempt observed, as the splitter and the reviewer read it.
///
/// Machine evidence, not prose: each field is one whitespace-free `key:value`
/// token, so the record stays greppable and the split decision is made from
/// facts a reader can check against the recorded session.
fn evidence_of(cli: &str, session: &AgentSession) -> String {
    let verification = session
        .verification
        .iter()
        .map(|result| format!("{}={}", result.program, result.passed))
        .collect::<Vec<_>>()
        .join(",");
    let exit = session
        .exit_code
        .map_or_else(|| "none".to_string(), |code| code.to_string());
    [
        format!("cli:{cli}"),
        format!("status:{:?}", session.status),
        format!("exit:{exit}"),
        format!("verification:{verification}"),
    ]
    .join(" ")
}

/// The tool boundary: one attempt is one recorded agent session.
struct AgentExecutor<'config> {
    config: &'config DispatchConfig,
    workspace: PathBuf,
    output_dir: PathBuf,
    sessions: Vec<AgentSession>,
    steps: Vec<IncrementalStep>,
    changes: Vec<WorkspaceChange>,
    /// How many CLIs each task has already consumed, keyed by task identity.
    escalations: BTreeMap<String, usize>,
    /// The first controller error, kept because the executor trait has no way
    /// to report one. A run that hit it is reported as an error, never as a
    /// failed task: a controller fault is not evidence about the task.
    error: Option<DispatchError>,
}

impl<'config> AgentExecutor<'config> {
    fn new(config: &'config DispatchConfig, workspace: &Path, output_dir: &Path) -> Self {
        Self {
            config,
            workspace: workspace.to_path_buf(),
            output_dir: output_dir.to_path_buf(),
            sessions: Vec::new(),
            steps: Vec::new(),
            changes: Vec::new(),
            escalations: BTreeMap::new(),
            error: None,
        }
    }

    /// Which CLI runs this task next: the first one, then one further along the
    /// list for each escalation this task has been granted.
    fn cli_for(&self, task: &RecursiveTask) -> String {
        let index = self.escalations.get(&task.id).copied().unwrap_or(0);
        self.config.clis[index.min(self.config.clis.len() - 1)].clone()
    }

    fn record(&mut self, error: DispatchError) {
        if self.error.is_none() {
            self.error = Some(error);
        }
    }

    fn preserve_session(
        &mut self,
        task: &RecursiveTask,
        cli: String,
        session: AgentSession,
    ) -> TaskAttempt {
        let index = self.sessions.len();
        let session_file = format!("sessions/{index:03}-{}.json", safe_name(&cli));
        if let Err(error) = write_session(&self.output_dir.join(&session_file), &session) {
            self.record(DispatchError::Replay(error));
            return TaskAttempt::failed("controller_aborted");
        }
        self.record_preserved_session(task, cli, session, session_file)
    }

    fn record_preserved_session(
        &mut self,
        task: &RecursiveTask,
        cli: String,
        session: AgentSession,
        session_file: String,
    ) -> TaskAttempt {
        let passed = session.passed();
        let evidence = evidence_of(&cli, &session);
        self.steps.push(IncrementalStep {
            task_id: task.id.clone(),
            task: task.goal.clone(),
            cli,
            passed,
            session_file,
        });
        self.sessions.push(session);
        if passed {
            TaskAttempt::passed(evidence)
        } else {
            TaskAttempt::failed(evidence)
        }
    }
}

impl TaskExecutor for AgentExecutor<'_> {
    fn attempt(&mut self, task: &RecursiveTask) -> TaskAttempt {
        if self.error.is_some() {
            return TaskAttempt::failed("controller_aborted");
        }
        let index = self.sessions.len();
        let cli = self.cli_for(task);
        let candidate = self
            .config
            .output_dir
            .join("candidates")
            .join(format!("{index:03}-{}", safe_name(&cli)));
        if let Err(error) = copy_workspace(&self.workspace, &candidate, &self.output_dir) {
            self.record(DispatchError::Io(error));
            return TaskAttempt::failed("controller_aborted");
        }
        let run = candidate_run_config(self.config, &cli, task.goal.clone(), &candidate);
        let session = match run_agent(&run) {
            Ok(session) => session,
            Err(error) => {
                self.record(DispatchError::Run(error));
                return TaskAttempt::failed("controller_aborted");
            }
        };
        let passed = session.passed();
        let session_file = format!("sessions/{index:03}-{}.json", safe_name(&cli));
        let session_path = self.output_dir.join(&session_file);
        if let Err(error) = write_session(&session_path, &session) {
            self.record(DispatchError::Replay(error));
            return TaskAttempt::failed("controller_aborted");
        }
        if passed
            && self.config.pull_request.is_some()
            && let Err(error) =
                validate_effect_attribution(&self.workspace, &session_path, &session)
        {
            self.record(error);
            return TaskAttempt::failed("controller_aborted");
        }
        // A passing attempt's effects become the starting point of the next
        // one, so a later sub-task -- and the parent's own retry -- sees the
        // work its predecessors did instead of a workspace that forgot it.
        if passed {
            if let Err(error) = apply_changes(&self.workspace, &candidate, &session.changes) {
                self.record(DispatchError::Io(error));
                return TaskAttempt::failed("controller_aborted");
            }
            if let Some(pull_request) = &self.config.pull_request
                && let Err(error) =
                    commit_verified_effect(&self.workspace, &session_path, &session, pull_request)
            {
                self.record(error);
                return TaskAttempt::failed("controller_aborted");
            }
            for change in &session.changes {
                self.changes.retain(|prior| prior.path != change.path);
                self.changes.push(change.clone());
            }
        }
        self.record_preserved_session(task, cli, session, session_file)
    }

    fn extend_for(&mut self, task: &RecursiveTask, _failure: &TaskAttempt) -> bool {
        if self.error.is_some() {
            return false;
        }
        let used = self.escalations.get(&task.id).copied().unwrap_or(0);
        let next = used + 1;
        if next >= self.config.clis.len() {
            return false;
        }
        self.escalations.insert(task.id.clone(), next);
        true
    }

    fn retry_after_children(&mut self, task: &RecursiveTask) -> TaskAttempt {
        if self.error.is_some() {
            return TaskAttempt::failed("controller_aborted");
        }
        if self.config.verification.is_empty() {
            return self.attempt(task);
        }
        let cli = self.cli_for(task);
        let run = candidate_run_config(self.config, &cli, task.goal.clone(), &self.workspace);
        match verify_workspace(&run) {
            Ok(session) if session.passed() => {
                self.preserve_session(task, COMPOSED_VERIFIER.to_string(), session)
            }
            Ok(_) => self.attempt(task),
            Err(error) => {
                self.record(DispatchError::Run(error));
                TaskAttempt::failed("controller_aborted")
            }
        }
    }
}
