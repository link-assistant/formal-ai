//! One repository protocol for SWE-bench, the #848 ladder and self-coding (#1138 B7).
//!
//! Plan 03 owns plan 00 §4.4's `Workspace` contract and implements it exactly
//! once, as [`RepositoryWorkspace`], so a fix in one caller is a fix in all. The
//! ordered protocol itself is data
//! (`data/meta/repository-workspace-protocol.lino`): adding "run the linter
//! before the tests" is a `.lino` edit, not a Rust edit.
//!
//! Wave T lands the shapes only; wave I7 leaves 03-L2 through 03-L9 fill the
//! bodies in.

pub mod clone;
pub mod diff;
pub mod edit;
pub mod locate;
pub mod outcome;
pub mod verify;
pub mod world_model;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::execution_evidence::{Evidence, EvidenceDetail, EvidenceSource, ObservationKind};
use crate::meta_frame::{AtomicityReason, LedgerRow, NeedLedger, NeedStatus};
use crate::obligation_ledger::{
    ObligationExpectation, ObligationLedger, ObligationNode, ObligationOutcome,
    need_ledger_with_execution,
};
use crate::seed::parser::parse_lino;
use clone::WorkspaceSpec;
use locate::Location;
use verify::RunCommand;

/// The default-deny allowlist, as reviewable data.
///
/// A program with no row is refused, and so is a listed program used with a
/// subcommand no row names: the grant is a program *and* a subcommand *and* an
/// argument shape, never a bare program. Widening the set is an edit to that
/// file, which is reviewable, rather than an edit to a list in Rust, which is
/// not.
///
/// Wave F recorded why the scope has to be this narrow. A Russian prompt —
/// *"Запусти это и скажи точно, что оно печатает: print(sum(range(1, 11)))"* —
/// had its leading verb stripped and the remainder handed to `/bin/sh -c`; the
/// shell's syntax error came back labelled as a completed command. Under this
/// table the first word of that remainder has no row, so the sentence is
/// refused before any shell sees it.
///
/// The document itself carries no prose header: `scripts/audit-total-closure.py`
/// reads every word of a `data/seed/**.lino` comment as a value token that must
/// resolve to a grounded meaning, so the rationale for a seed table lives in the
/// module that reads it. No committed seed file carries `#` comments.
const ALLOWLIST_LINO: &str = include_str!("../../data/seed/repository-command-allowlist.lino");

/// Record type of one allowlist row.
const RECORD_ALLOWED: &str = "allowed_command";

/// The ordered protocol, as data. Adding "run the linter before the tests" is a
/// `.lino` edit, not a Rust edit.
const PROTOCOL_LINO: &str = include_str!("../../data/meta/repository-workspace-protocol.lino");

/// Record type of one protocol step.
const RECORD_STEP: &str = "meta_step";

/// The header the regenerated document carries, so a deleted document is
/// rediscovered to the same content id rather than to a near-miss.
const PROTOCOL_HEADER: &str = include_str!("protocol-header.txt");

/// Everything that can stop the protocol, stated rather than swallowed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkspaceError {
    /// `base_commit` is not a 40-character object name; a branch is not a commit.
    NotACommit {
        /// What the spec carried instead.
        given: String,
    },
    /// A command the allowlist does not permit.
    UnsupportedCommand {
        /// The program that was refused.
        program: String,
    },
    /// The named tests could not run because a program is missing. This is
    /// neither a pass nor a test failure (plan 06 turns it into a requirement).
    MissingPrerequisite {
        /// The program that is not available.
        program: String,
        /// The exit status observed, when one was reported.
        exit_code: Option<i32>,
        /// Standard error exactly as observed.
        stderr: String,
    },
    /// The run exceeded its deadline; both numbers are reported.
    TimedOut {
        /// The deadline in seconds.
        deadline_seconds: u64,
        /// How long the run actually took, in seconds.
        elapsed_seconds: u64,
    },
    /// Something the workspace observed and is reporting verbatim.
    Observed {
        /// What was observed.
        detail: String,
    },
}

impl std::fmt::Display for WorkspaceError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotACommit { given } => write!(formatter, "not an exact commit: {given}"),
            Self::UnsupportedCommand { program } => {
                write!(formatter, "repository command is not allowed: {program}")
            }
            Self::MissingPrerequisite {
                program,
                exit_code,
                stderr,
            } => write!(
                formatter,
                "missing prerequisite {program} (exit {exit_code:?}): {stderr}"
            ),
            Self::TimedOut {
                deadline_seconds,
                elapsed_seconds,
            } => write!(
                formatter,
                "repository command timed out after {elapsed_seconds}s (deadline {deadline_seconds}s)"
            ),
            Self::Observed { detail } => formatter.write_str(detail),
        }
    }
}

impl std::error::Error for WorkspaceError {}

/// A checked-out tree a repository task may read, edit, test and diff.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepositoryWorkspace {
    root: PathBuf,
    spec: WorkspaceSpec,
    /// Byte snapshot of every file the protocol has read or written, so a diff
    /// is computed from observed bytes rather than from a second `git` call.
    baseline: BTreeMap<String, Vec<u8>>,
}

impl RepositoryWorkspace {
    /// Clone `spec` into a fresh directory under `base_dir`.
    ///
    /// # Errors
    /// Propagates [`clone::clone_at_base`]'s refusals.
    pub fn open(spec: &WorkspaceSpec, base_dir: &Path) -> Result<Self, WorkspaceError> {
        let root = clone::clone_at_base(spec, base_dir)?;
        Ok(Self {
            root,
            spec: spec.clone(),
            baseline: BTreeMap::new(),
        })
    }

    /// Adopt an existing directory (the ambient checkout, a `git worktree`)
    /// without cloning.
    ///
    /// # Errors
    /// Propagates the observed `git` failure.
    pub fn adopt(root: &Path) -> Result<Self, WorkspaceError> {
        let base_commit = clone::observed_head(root).unwrap_or_default();
        Ok(Self {
            root: root.to_path_buf(),
            spec: WorkspaceSpec {
                origin: root.display().to_string(),
                base_commit,
                sparse_paths: Vec::new(),
            },
            baseline: BTreeMap::new(),
        })
    }

    /// The workspace root.
    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// The spec this workspace was materialised from.
    #[must_use]
    pub const fn spec(&self) -> &WorkspaceSpec {
        &self.spec
    }

    /// The commit the tree is checked out at.
    #[must_use]
    pub fn base_commit(&self) -> &str {
        &self.spec.base_commit
    }

    /// Every source file in the tree as `(relative_path, contents)`, in path order.
    ///
    /// # Errors
    /// Propagates the observed read failure.
    pub fn source_files(&self) -> Result<Vec<(String, String)>, WorkspaceError> {
        let mut files = Vec::new();
        collect_files(&self.root, &self.root, &mut files);
        files.sort_by(|left, right| left.0.cmp(&right.0));
        Ok(files)
    }

    /// Read one file, relative to the root.
    ///
    /// # Errors
    /// Propagates the observed read failure.
    pub fn read(&self, relative: &str) -> Result<String, WorkspaceError> {
        std::fs::read_to_string(self.root.join(relative)).map_err(|error| {
            WorkspaceError::Observed {
                detail: error.to_string(),
            }
        })
    }

    /// Write one file, relative to the root, recording its prior bytes.
    ///
    /// # Errors
    /// Propagates the observed write failure.
    pub fn write(&mut self, relative: &str, contents: &str) -> Result<(), WorkspaceError> {
        let path = self.root.join(relative);
        if !path.starts_with(&self.root) {
            return Err(WorkspaceError::Observed {
                detail: path.display().to_string(),
            });
        }
        let before = std::fs::read(&path).unwrap_or_default();
        self.baseline.insert(relative.to_owned(), before);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|error| WorkspaceError::Observed {
                detail: error.to_string(),
            })?;
        }
        std::fs::write(&path, contents).map_err(|error| WorkspaceError::Observed {
            detail: error.to_string(),
        })
    }

    /// The unified diff between the base commit and the current tree.
    ///
    /// # Errors
    /// Propagates the observed failure.
    pub fn diff(&self) -> Result<String, WorkspaceError> {
        // A file the protocol created is untracked, and an untracked file is
        // invisible to `git diff`. Recording the intent to add it makes the
        // deliverable complete without staging any bytes.
        clone::run_git(&self.root, &["add", "--intent-to-add", "--all"])?;
        clone::run_git(&self.root, &["diff"])
    }

    /// The bytes this workspace observed before it wrote over them, so a diff
    /// is attributable to what the protocol actually did.
    #[must_use]
    pub const fn baseline(&self) -> &BTreeMap<String, Vec<u8>> {
        &self.baseline
    }
}

/// One step of the protocol: what it does, what it must observe before the next
/// step runs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProtocolStep {
    /// Position in the protocol, 1-based and contiguous.
    pub order: usize,
    /// `clone | locate | read | edit | verify | diff`.
    pub id: String,
    /// What must hold before the step runs.
    pub precondition: Vec<String>,
    /// What must be observed after it.
    pub postcondition: Vec<String>,
}

/// What a repository task is, independent of where it came from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepositoryTask {
    /// The requirement text, verbatim, in whatever language it arrived in.
    pub requirement: String,
    /// Where the tree comes from.
    pub clone: WorkspaceSpec,
    /// Named tests, when the source supplies them.
    pub tests: Option<RunCommand>,
}

/// What one protocol run observed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProtocolOutcome {
    /// Where the requirement resolved to.
    pub located: Vec<Location>,
    /// Which files were edited.
    pub edited: Vec<String>,
    /// Every observation the run made.
    pub observations: Vec<Evidence>,
    /// The unified diff, whatever it is.
    pub diff: String,
    /// The first step whose postcondition was not observed, if any.
    pub stopped_at: Option<ProtocolStep>,
    /// Requirements the protocol could not satisfy, stated plainly.
    pub open: Vec<String>,
    /// One planned need per declared step, upgraded only by linked execution
    /// evidence produced during this run.
    pub need_ledger: NeedLedger,
}

/// The ordered repository protocol, read from
/// `data/meta/repository-workspace-protocol.lino`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceProtocol {
    steps: Vec<ProtocolStep>,
}

/// Render one named repository-protocol template from the same data document
/// that declares the protocol steps.
#[must_use]
pub fn render_protocol_template(id: &str, values: &[(&str, &str)]) -> Option<String> {
    render_protocol_template_from(PROTOCOL_LINO, id, values)
}

/// Render a protocol template from caller-supplied data, used by replay and by
/// tests proving that changing the meta document changes behavior.
#[must_use]
pub fn render_protocol_template_from(
    document: &str,
    id: &str,
    values: &[(&str, &str)],
) -> Option<String> {
    let mut rendered = parse_lino(document)
        .children
        .into_iter()
        .find(|node| node.name == "repository_template" && node.id == id)?
        .find_child_value("text")
        .to_owned();
    for (name, value) in values {
        rendered = rendered.replace(&format!("{{{name}}}"), value);
    }
    Some(rendered)
}

impl WorkspaceProtocol {
    /// Parse the committed protocol document.
    #[must_use]
    pub fn load() -> Self {
        Self::parse(PROTOCOL_LINO)
    }

    /// Parse a protocol document.
    #[must_use]
    pub fn parse(document: &str) -> Self {
        let mut steps: Vec<ProtocolStep> = parse_lino(document)
            .children
            .iter()
            .filter(|node| node.find_child_value("record_type") == RECORD_STEP)
            .map(|node| ProtocolStep {
                order: node.find_child_value("order").parse().unwrap_or_default(),
                id: node.find_child_value("id").to_owned(),
                precondition: vec![node.find_child_value("precondition").to_owned()],
                postcondition: vec![node.find_child_value("postcondition").to_owned()],
            })
            .collect();
        steps.sort_by_key(|step| step.order);
        Self { steps }
    }

    /// The file each step is implemented in, in protocol order.
    #[must_use]
    pub fn source_files() -> Vec<String> {
        parse_lino(PROTOCOL_LINO)
            .children
            .iter()
            .filter(|node| node.find_child_value("record_type") == RECORD_STEP)
            .map(|node| node.find_child_value("source_file").to_owned())
            .collect()
    }

    /// The ordered steps.
    #[must_use]
    pub fn steps(&self) -> &[ProtocolStep] {
        &self.steps
    }

    /// Execute the protocol for `task` against `workspace`.
    ///
    /// Every step records an execution record into the `NeedLedger` row for its
    /// obligation before the next step is planned.
    #[must_use]
    pub fn execute(
        &self,
        workspace: &mut RepositoryWorkspace,
        task: &RepositoryTask,
    ) -> ProtocolOutcome {
        let mut outcome = ProtocolOutcome {
            located: Vec::new(),
            edited: Vec::new(),
            observations: Vec::new(),
            diff: String::new(),
            stopped_at: None,
            open: Vec::new(),
            need_ledger: protocol_ledger(&self.steps, task),
        };

        for step in &self.steps {
            match step.id.as_str() {
                "clone" => match clone::observed_head(workspace.root()) {
                    Some(head) if head == task.clone.base_commit => {
                        let evidence = Evidence::observed(
                            "git rev-parse HEAD",
                            vec![String::from("HEAD")],
                            Some(0),
                            head.as_bytes(),
                            ObservationKind::CommandExit,
                            EvidenceSource::LocalProcess,
                        );
                        record_step_observation(&mut outcome, step, evidence);
                    }
                    observed => {
                        outcome.stopped_at = Some(step.clone());
                        outcome.open.push(
                            render_protocol_template(
                                "workspace_base_commit_mismatch",
                                &[
                                    ("expected", &task.clone.base_commit),
                                    ("observed", observed.as_deref().unwrap_or("unavailable")),
                                ],
                            )
                            .unwrap_or_else(|| String::from("workspace_base_commit_mismatch")),
                        );
                        return outcome;
                    }
                },
                "locate" => {
                    let need = crate::needs::Need::raised(
                        crate::needs::NeedKind::Part,
                        &task.requirement,
                        "",
                        "repository_workspace_protocol",
                    );
                    match locate::locate_targets(workspace, &need) {
                        Ok(found) if found.is_empty() => {
                            // Nothing resolved. The protocol stops here and says
                            // so, rather than editing a file it guessed at.
                            outcome.stopped_at = Some(step.clone());
                            outcome.open.push(task.requirement.clone());
                            return outcome;
                        }
                        Ok(found) => {
                            let observation = found
                                .iter()
                                .map(|location| {
                                    format!(
                                        "{}:{}:{:?}",
                                        location.relative_path,
                                        location.symbol.as_deref().unwrap_or_default(),
                                        location.how
                                    )
                                })
                                .collect::<Vec<_>>()
                                .join("\n");
                            outcome.located = found;
                            let evidence = Evidence::observed(
                                "repository locate",
                                Vec::new(),
                                None,
                                observation.as_bytes(),
                                ObservationKind::SymbolicCheck,
                                EvidenceSource::Engine,
                            );
                            record_step_observation(&mut outcome, step, evidence);
                        }
                        Err(error) => {
                            outcome.stopped_at = Some(step.clone());
                            outcome.open.push(format!("{error:?}"));
                            return outcome;
                        }
                    }
                }
                "read" => {
                    let mut observed = Vec::new();
                    let mut seen = std::collections::BTreeSet::new();
                    for location in &outcome.located {
                        if !seen.insert(location.relative_path.clone()) {
                            continue;
                        }
                        match workspace.read(&location.relative_path) {
                            Ok(source) => observed.push((location.relative_path.clone(), source)),
                            Err(error) => {
                                outcome.stopped_at = Some(step.clone());
                                outcome.open.push(format!("{error:?}"));
                                return outcome;
                            }
                        }
                    }
                    let mut evidence = Evidence::observed(
                        "repository read",
                        observed.iter().map(|(path, _)| path.clone()).collect(),
                        None,
                        observed
                            .iter()
                            .flat_map(|(_, source)| source.as_bytes())
                            .copied()
                            .collect::<Vec<_>>()
                            .as_slice(),
                        ObservationKind::FileBytes,
                        EvidenceSource::LocalProcess,
                    );
                    evidence.source_ids = observed
                        .iter()
                        .map(|(path, source)| {
                            crate::engine::stable_id(
                                "repository_source",
                                &format!("{path}:{source}"),
                            )
                        })
                        .collect();
                    record_step_observation(&mut outcome, step, evidence);
                }
                "edit" => {
                    let mut changes = Vec::new();
                    let mut seen = std::collections::BTreeSet::new();
                    for location in &outcome.located {
                        if !seen.insert(location.relative_path.clone()) {
                            continue;
                        }
                        match edit::derive_change(workspace, location, &task.requirement) {
                            Ok(Some(change)) => changes.push(change),
                            Ok(None) => {}
                            Err(error) => {
                                outcome.stopped_at = Some(step.clone());
                                outcome.open.push(format!("{error:?}"));
                                return outcome;
                            }
                        }
                    }
                    if changes.is_empty() {
                        outcome.stopped_at = Some(step.clone());
                        outcome.open.push(String::from(
                            "no registry-grounded structural edit was derivable",
                        ));
                        return outcome;
                    }
                    for change in changes {
                        match edit::apply_change(workspace, &change) {
                            Ok(evidence) => {
                                if !outcome.edited.contains(&change.relative_path) {
                                    outcome.edited.push(change.relative_path);
                                }
                                record_step_observation(&mut outcome, step, evidence);
                            }
                            Err(error) => {
                                outcome.stopped_at = Some(step.clone());
                                outcome.open.push(format!("{error:?}"));
                                return outcome;
                            }
                        }
                    }
                }
                "verify" => {
                    if let Some(tests) = task.tests.as_ref() {
                        match verify::run_named_tests(
                            workspace,
                            tests,
                            &crate::execution_box::ExecutionBackend::HostSandbox,
                        ) {
                            Ok(evidence) => {
                                let failed = matches!(
                                    &evidence.detail,
                                    EvidenceDetail::Tests { failed, .. } if !failed.is_empty()
                                );
                                record_step_observation(&mut outcome, step, evidence);
                                if failed {
                                    outcome.stopped_at = Some(step.clone());
                                    outcome.open.push(String::from("named tests did not pass"));
                                    return outcome;
                                }
                            }
                            Err(error) => {
                                outcome.stopped_at = Some(step.clone());
                                outcome.open.push(format!("{error:?}"));
                                return outcome;
                            }
                        }
                    }
                }
                "diff" => match workspace.diff() {
                    Ok(patch) => {
                        let evidence = Evidence::observed(
                            "git diff",
                            Vec::new(),
                            Some(0),
                            patch.as_bytes(),
                            ObservationKind::CommandExit,
                            EvidenceSource::LocalProcess,
                        );
                        outcome.diff = patch;
                        record_step_observation(&mut outcome, step, evidence);
                    }
                    Err(error) => {
                        outcome.stopped_at = Some(step.clone());
                        outcome.open.push(format!("{error:?}"));
                        return outcome;
                    }
                },
                _ => {}
            }
        }
        outcome
    }

    /// Regenerate the committed protocol document from the live source, so a
    /// deleted document is rediscovered to the same content id.
    #[must_use]
    pub fn regenerate_document() -> String {
        use std::fmt::Write as _;

        let mut out = String::from(PROTOCOL_HEADER);
        let sources = Self::source_files();
        let declared = parse_lino(PROTOCOL_LINO);
        let records: Vec<&crate::seed::parser::LinoNode> = declared
            .children
            .iter()
            .filter(|node| node.find_child_value("record_type") == RECORD_STEP)
            .collect();
        for (step, node) in Self::load().steps().iter().zip(records) {
            let _ = writeln!(out, "repository_step_{}", step.id);
            let _ = writeln!(out, "  record_type \"{RECORD_STEP}\"");
            let _ = writeln!(out, "  order \"{}\"", step.order);
            let _ = writeln!(out, "  id \"{}\"", step.id);
            let _ = writeln!(out, "  detail \"{}\"", node.find_child_value("detail"));
            let _ = writeln!(out, "  precondition \"{}\"", step.precondition.join(" "));
            let _ = writeln!(out, "  postcondition \"{}\"", step.postcondition.join(" "));
            let _ = writeln!(
                out,
                "  source_file \"{}\"",
                sources
                    .get(step.order.saturating_sub(1))
                    .map_or("", String::as_str)
            );
        }
        // Runtime messages are data owned by the same protocol document. They
        // are not executable steps and therefore do not participate in the
        // source/order reconstruction above, but forgetting them would change
        // behaviour as surely as forgetting a step. Preserve the complete
        // suffix from the first template record, so adding another template is
        // a data edit and requires no matching Rust arm.
        if let Some(offset) = PROTOCOL_LINO.find("\nrepository_template ") {
            out.push_str(&PROTOCOL_LINO[offset + 1..]);
        }
        out
    }
}

fn protocol_ledger(steps: &[ProtocolStep], task: &RepositoryTask) -> NeedLedger {
    let frame_id = crate::engine::stable_id(
        "repository_protocol",
        &format!("{}:{}", task.clone.base_commit, task.requirement),
    );
    NeedLedger {
        frame_id: frame_id.clone(),
        rows: steps
            .iter()
            .map(|step| LedgerRow {
                need_id: crate::engine::stable_id(
                    "repository_protocol_step",
                    &format!("{frame_id}:{}", step.id),
                ),
                source_span: task.requirement.clone(),
                status: NeedStatus::Planned,
                leaf_reason: Some(AtomicityReason::DirectMethod),
                unit_id: None,
                route: Some(step.id.clone()),
            })
            .collect(),
    }
}

fn record_step_observation(
    outcome: &mut ProtocolOutcome,
    step: &ProtocolStep,
    mut evidence: Evidence,
) {
    let Some(need_id) = outcome
        .need_ledger
        .rows
        .iter()
        .find(|row| row.route.as_deref() == Some(step.id.as_str()))
        .map(|row| row.need_id.clone())
    else {
        return;
    };
    evidence.for_need.clone_from(&need_id);
    if evidence.produced_by.is_empty() {
        evidence.produced_by = format!("repository_protocol_{}", step.id);
    }
    let obligations = ObligationLedger {
        frame_id: outcome.need_ledger.frame_id.clone(),
        root: ObligationNode {
            node_id: crate::engine::stable_id("repository_step_obligation", &need_id),
            parent: None,
            clause: step.id.clone(),
            span: (0, step.id.len()),
            need_id: Some(need_id),
            depth: 0,
            expectation: ObligationExpectation::SymbolicCheck {
                check_id: step.id.clone(),
            },
            outcome: ObligationOutcome::Satisfied {
                record: evidence.clone(),
            },
            children: Vec::new(),
        },
    };
    outcome.need_ledger = need_ledger_with_execution(&outcome.need_ledger, &obligations);
    outcome.observations.push(evidence);
}

/// One row of `data/seed/repository-command-allowlist.lino`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AllowedCommand {
    /// The row's stable id.
    pub id: String,
    /// The program a shell would resolve.
    pub program: String,
    /// The subcommand, when the row names one.
    pub subcommand: Option<String>,
    /// The argument shape, with `{placeholders}`.
    pub arguments: Vec<String>,
    /// Whether the command changes the tree.
    pub mutating: bool,
    /// What must hold before it runs.
    pub precondition: Option<String>,
    /// What must be observed after it.
    pub postcondition: Option<String>,
}

/// The seed allowlist, in file order. Default-deny: a program with no row is
/// refused, and so is a listed program with an unlisted subcommand.
#[must_use]
pub fn command_allowlist() -> Vec<AllowedCommand> {
    parse_lino(ALLOWLIST_LINO)
        .children
        .iter()
        .filter(|node| node.find_child_value("record_type") == RECORD_ALLOWED)
        .map(|node| AllowedCommand {
            id: node.find_child_value("id").to_owned(),
            program: node.find_child_value("program").to_owned(),
            subcommand: optional(node.find_child_value("subcommand")),
            arguments: crate::prerequisite::probe::split_tuple(node.find_child_value("arguments")),
            mutating: node.find_child_value("mutating") == "true",
            precondition: optional(node.find_child_value("precondition")),
            postcondition: optional(node.find_child_value("postcondition")),
        })
        .collect()
}

/// The bound used when no row declares one.
pub const DEFAULT_DEADLINE_SECONDS: u64 = 300;

/// The bound a permitted command shape is measured against, in seconds.
///
/// Declared per shape rather than globally, because "it did not finish" means
/// something different for a one-line script and for a whole suite, and one
/// number for both would make the shorter one an allowance or the longer one a
/// guillotine.
#[must_use]
pub fn deadline_seconds(program: &str, argv: &[&str]) -> u64 {
    let tail = subcommand_of(program, argv);
    let subcommand = tail.first().copied().unwrap_or_default();
    parse_lino(ALLOWLIST_LINO)
        .children
        .iter()
        .find(|node| {
            node.find_child_value("record_type") == RECORD_ALLOWED
                && node.find_child_value("program") == program
                && node.find_child_value("subcommand") == subcommand
        })
        .and_then(|node| node.find_child_value("deadline_seconds").parse().ok())
        .unwrap_or(DEFAULT_DEADLINE_SECONDS)
}

/// A field that is present only when it is non-empty.
fn optional(value: &str) -> Option<String> {
    if value.trim().is_empty() {
        None
    } else {
        Some(value.to_owned())
    }
}

/// `git`'s global options, which precede the subcommand and are not one.
///
/// Scoped to `git` deliberately. `-c` is a global option there and the whole
/// subcommand elsewhere — `python3 -c` is the inline-script form — so stripping
/// it unconditionally would leave an interpreter with no subcommand at all, and
/// a default-deny table would then refuse a shape it names.
const GIT_GLOBAL_FLAGS: &[&str] = &["-C", "-c"];

/// The program whose leading options are global rather than subcommands.
const GLOBAL_FLAG_PROGRAM: &str = "git";

/// Strip the leading global options so the subcommand is found where a row
/// names it, and never mistaken for one.
fn subcommand_of<'a>(program: &str, argv: &'a [&'a str]) -> &'a [&'a str] {
    if program != GLOBAL_FLAG_PROGRAM {
        return argv;
    }
    let mut index = 0;
    while index + 1 < argv.len() && GIT_GLOBAL_FLAGS.contains(&argv[index]) {
        index += 2;
    }
    &argv[index..]
}

/// Whether `program` with `argv` is permitted by the seed allowlist.
///
/// Default-deny in three places at once: a program with no row is refused, a
/// listed program with an unlisted subcommand is refused, and a listed
/// subcommand whose arguments do not fit the row's shape is refused. That is
/// what makes a sentence unrunnable — its first word names no program, so it
/// never reaches a shell to fail there.
#[must_use]
pub fn allows(program: &str, argv: &[&str]) -> bool {
    let tail = subcommand_of(program, argv);
    let Some((subcommand, arguments)) = tail.split_first() else {
        return false;
    };
    command_allowlist().into_iter().any(|row| {
        row.program == program
            && row.subcommand.as_deref() == Some(*subcommand)
            && shape_matches(&row.arguments, arguments)
    })
}

/// Whether `arguments` fit `shape`. A `{placeholder}` matches exactly one
/// argument; `{rest}` as the last token matches any remainder; a literal must
/// match exactly.
fn shape_matches(shape: &[String], arguments: &[&str]) -> bool {
    let mut index = 0;
    for (position, token) in shape.iter().enumerate() {
        if token == "{rest}" {
            return position + 1 == shape.len();
        }
        let Some(argument) = arguments.get(index) else {
            return false;
        };
        if !token.starts_with('{') && token != argument {
            return false;
        }
        index += 1;
    }
    index == arguments.len()
}

/// Every file under `directory`, relative to `root`, excluding the git database.
fn collect_files(root: &Path, directory: &Path, into: &mut Vec<(String, String)>) {
    let Ok(entries) = std::fs::read_dir(directory) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if matches!(entry.file_name().to_str(), Some(".git" | "target")) {
            continue;
        }
        if path.is_dir() {
            collect_files(root, &path, into);
        } else if let Ok(text) = std::fs::read_to_string(&path) {
            let relative = path
                .strip_prefix(root)
                .unwrap_or(&path)
                .display()
                .to_string();
            into.push((relative, text));
        }
    }
}
