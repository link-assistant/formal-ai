//! A missing executable stated as a requirement rather than as an error (#1138 B6).
//!
//! Plan 06 owns this module tree. A `command not found` is a need of kind
//! `prerequisite` (plan 00 §4.1); it is looked up, formalized, installed under
//! an explicit grant and then **re-probed**, because a successful setup command
//! with a failing postcondition is still missing.
//!
//! Wave T lands the shapes only; wave I6 leaves 06-L4 through 06-L8 fill the
//! bodies in.

pub mod install;
pub mod ledger;
pub mod probe;
pub mod publisher;

use crate::execution_evidence::Evidence;
use crate::source_walk::{LookupBounds, SourceLookup};
use install::{InstallGrant, WorkspaceToolchain};
use ledger::ToolchainLedger;
use probe::ProbeVerdict;
use publisher::SetupProcedure;

/// The recovery sequence, as data. The document is the authority; `recover`
/// walks it and `tests/unit/specification/prerequisite_recipe.rs` grounds it.
const PREREQUISITE_RECIPE_LINO: &str = include_str!("../../../data/meta/prerequisite-recipe.lino");

/// The host platform, as observed — never inferred from the requested language
/// (issue-710 plan 07 recovery step 2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    /// macOS.
    Darwin,
    /// Linux.
    Linux,
    /// Windows.
    Windows,
    /// Observed, but not one this build knows how to name.
    Unknown,
}

impl Platform {
    /// The platform of the machine this process is running on, observed.
    ///
    /// Read from the target this binary runs on, never inferred from the
    /// programming language a task was written in: a Kotlin task on macOS runs
    /// on macOS (issue-710 plan 07 recovery step 2, `observe_platform`).
    #[must_use]
    pub fn observed() -> Self {
        Self::from_os(std::env::consts::OS)
    }

    /// The platform an operating-system name denotes.
    #[must_use]
    pub fn from_os(os: &str) -> Self {
        match os {
            "macos" | "ios" => Self::Darwin,
            "linux" | "android" => Self::Linux,
            "windows" => Self::Windows,
            _ => Self::Unknown,
        }
    }

    /// Stable slug used in the Links Notation trace.
    #[must_use]
    pub const fn slug(self) -> &'static str {
        match self {
            Self::Darwin => "darwin",
            Self::Linux => "linux",
            Self::Windows => "windows",
            Self::Unknown => "unknown",
        }
    }

    /// The platform a recorded slug names.
    #[must_use]
    pub fn from_slug(slug: &str) -> Self {
        match slug.trim() {
            "darwin" => Self::Darwin,
            "linux" => Self::Linux,
            "windows" => Self::Windows,
            _ => Self::Unknown,
        }
    }
}

/// A missing executable, stated as a requirement rather than as an error.
///
/// The in-memory projection of a plan 00 §4.1 `need` record whose `kind` is
/// `prerequisite` and whose `subject` is the missing program.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrerequisiteNeed {
    /// The missing program.
    pub program: String,
    /// The requirement that needed it, verbatim, in its original language.
    pub source_span: String,
    /// The exact call that failed: command, exit code, stderr.
    pub observed: ProbeVerdict,
    /// Host platform as observed.
    pub platform: Platform,
    /// Needs this one depends on, discovered recursively; cycles are detected.
    pub requires: Vec<String>,
}

/// Why a recovery refused before it ran anything.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PrerequisiteError {
    /// The procedure carries no postcondition probe, so nothing could verify it.
    NoPostcondition,
    /// The grant does not name this program.
    NotGranted {
        /// The program the procedure would have installed.
        program: String,
    },
    /// The target install was allowed, but this executable was not separately
    /// granted. Retrieved procedure text cannot widen process capabilities.
    ExecutionCapabilityNotGranted {
        /// Exact executable the typed step requested.
        program: String,
    },
    /// A typed step declared network use but the operator did not grant it.
    NetworkCapabilityNotGranted,
    /// A step writes outside the grant's root.
    OutsideWorkspace {
        /// The step's declared write location.
        path: String,
    },
    /// A documented digest did not match the retrieved bytes.
    DigestMismatch {
        /// The digest the publisher documented.
        expected: String,
        /// The digest observed.
        observed: String,
    },
    /// Free disk is below the procedure's stated requirement.
    InsufficientDisk {
        /// Bytes the procedure states it needs.
        required_bytes: u64,
        /// Bytes observed free.
        available_bytes: u64,
    },
    /// The postcondition probe cannot verify the program the procedure claims
    /// to install, so nothing it observed could discharge the need.
    PostconditionUnverifiable {
        /// The program the procedure claims to install.
        program: String,
        /// The program its postcondition would have probed instead.
        probes: String,
    },
    /// An authorized exact process could not start or returned nonzero.
    SetupStepFailed {
        /// Executable that was observed failing.
        program: String,
        /// Exit code, when the platform supplied one.
        exit_code: Option<i32>,
        /// Captured error text or process-start error.
        stderr: String,
    },
    /// A dependency cycle was detected between prerequisites.
    DependencyCycle {
        /// The programs on the cycle, in the order they were visited.
        cycle: Vec<String>,
    },
}

/// One step of the recovery sequence, read from `data/meta/prerequisite-recipe.lino`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecoveryStep {
    /// Position in the sequence, 1-based and contiguous.
    pub order: usize,
    /// The step's stable id.
    pub id: String,
    /// What must hold before the step runs.
    pub precondition: String,
    /// What must be observed after it.
    pub postcondition: String,
}

/// What a recovery attempt observed. Every variant is a statement the answer can
/// make verbatim; there is no variant meaning "probably fine".
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecoveryOutcome {
    /// Installed and re-probed `Present`; the original step may be retried.
    Recovered {
        /// The workspace-scoped toolchain that is now present.
        toolchain: WorkspaceToolchain,
        /// The command the caller should retry.
        retry: String,
        /// Every observation made along the way.
        evidence: Vec<Evidence>,
    },
    /// A procedure was found but the grant refused it.
    NotPermitted {
        /// The procedure the grant refused.
        procedure: SetupProcedure,
        /// Exact validation or execution failure observed.
        reason: PrerequisiteError,
    },
    /// No trusted publisher procedure was found. Names every source consulted.
    NotFound {
        /// Registry ids consulted, in consultation order.
        consulted: Vec<String>,
    },
    /// Installed but the postcondition still failed. Names both observations.
    StillMissing {
        /// What the probe said before the install.
        before: ProbeVerdict,
        /// What the probe said after it.
        after: ProbeVerdict,
    },
}

/// The recovery sequence as data: every step of `data/meta/prerequisite-recipe.lino`.
///
/// The document is the authority; this function is the live reading of it, so
/// `tests/unit/specification/prerequisite_recipe.rs` can regenerate the
/// committed skeleton from the source and compare content ids.
#[must_use]
pub fn recovery_steps() -> Vec<RecoveryStep> {
    let root = crate::seed::parser::parse_lino(PREREQUISITE_RECIPE_LINO);
    let mut steps: Vec<RecoveryStep> = root
        .children
        .iter()
        .filter(|node| node.find_child_value("record_type") == "meta_step")
        .map(|node| RecoveryStep {
            order: node.find_child_value("order").parse().unwrap_or_default(),
            id: node.find_child_value("id").to_owned(),
            precondition: node.find_child_value("precondition").to_owned(),
            postcondition: node.find_child_value("postcondition").to_owned(),
        })
        .collect();
    steps.sort_by_key(|step| step.order);
    steps
}

/// Recover from `need`, or report precisely why recovery is not possible.
///
/// Every step — the first probe, each setup step, the re-probe — appends a plan
/// 00 §4.3 `evidence` record referencing `need`.
///
/// The eight stages are the ones `data/meta/prerequisite-recipe.lino` declares,
/// in that order: `bind_failure`, `observe_platform`, `look_up_publisher`,
/// `formalize_procedure`, `prefer_workspace_scope`, `lower_to_tools`,
/// `retry_and_recheck`, `retain_experience`.
pub fn recover<L: SourceLookup>(
    need: &PrerequisiteNeed,
    grant: &InstallGrant,
    lookup: &mut L,
    bounds: &LookupBounds,
    ledger: &mut ToolchainLedger,
) -> RecoveryOutcome {
    recover_bounded(need, grant, lookup, bounds, ledger, &mut Vec::new())
}

/// Recover from an already discovered and formalized procedure.
///
/// Callers that own a pinned upstream manifest can enter the same scoped
/// installer, evidence, ledger and re-probe path without manufacturing a
/// second `SourceLookup` implementation. The procedure's publisher provenance
/// is still validated when it is constructed.
pub fn recover_discovered_procedure(
    need: &PrerequisiteNeed,
    procedure: SetupProcedure,
    grant: &InstallGrant,
    ledger: &mut ToolchainLedger,
) -> RecoveryOutcome {
    let root = grant
        .root()
        .unwrap_or_else(|| std::path::PathBuf::from("."));
    let postcondition = procedure
        .postcondition
        .clone()
        .unwrap_or_else(|| probe_for(&need.program));
    let before = probe::probe_command(&postcondition, &root);
    let evidence = vec![observation(need, "probe", &postcondition, &before)];
    finish_recovery(need, procedure, grant, ledger, &root, before, evidence)
}

/// The recovery walk, carrying the chain of programs already being recovered so
/// a dependency cycle terminates instead of recursing.
fn recover_bounded<L: SourceLookup>(
    need: &PrerequisiteNeed,
    grant: &InstallGrant,
    lookup: &mut L,
    bounds: &LookupBounds,
    ledger: &mut ToolchainLedger,
    visiting: &mut Vec<String>,
) -> RecoveryOutcome {
    // Stage 5 `prefer_workspace_scope`: everything happens under the grant's
    // root when there is one, and never anywhere else.
    let root = grant
        .root()
        .unwrap_or_else(|| std::path::PathBuf::from("."));

    // Stage 1/2 `bind_failure` and `observe_platform`: what is required first,
    // and on which machine. A dependency already retained in the ledger is a
    // shared setup and is not discovered a second time.
    if !visiting.contains(&need.program) {
        visiting.push(need.program.clone());
        let requirements = need.requires.clone();
        for requirement in requirements {
            if requirement == need.program || visiting.contains(&requirement) {
                continue;
            }
            if ledger.record_for(&requirement).is_some() {
                continue;
            }
            let dependency = PrerequisiteNeed {
                program: requirement.clone(),
                source_span: need.source_span.clone(),
                observed: probe::probe_command(&probe_for(&requirement), &root),
                platform: need.platform,
                requires: probe::seed_probe_for_program(&requirement)
                    .map(|probe| probe.requires)
                    .unwrap_or_default(),
            };
            let _ = recover_bounded(&dependency, grant, lookup, bounds, ledger, visiting);
        }
    }

    let postcondition = probe_for(&need.program);
    let before = probe::probe_command(&postcondition, &root);
    let evidence = vec![observation(need, "probe", &postcondition, &before)];

    // Already here: retain what was observed and report it rather than
    // installing over a working toolchain.
    if let probe::ProbeVerdict::Present { version } = &before {
        let record = ledger::ToolchainRecord {
            program: need.program.clone(),
            source_id: String::from("observed_on_the_machine"),
            source_url: String::new(),
            content_id: String::new(),
            platform: need.platform,
            postcondition,
            rediscover: String::new(),
            observed_version: version.clone(),
            installed_prefix: root.clone(),
        };
        if ledger.record_for(&need.program).is_none() {
            let _ = ledger.append(&record);
        }
        return RecoveryOutcome::Recovered {
            toolchain: WorkspaceToolchain {
                program: need.program.clone(),
                prefix: root.clone(),
                environment: probe::workspace_environment(&root),
                content_id: String::new(),
            },
            retry: retry_command(need),
            evidence,
        };
    }

    // Stage 3/4 `look_up_publisher` and `formalize_procedure`.
    let search = publisher::search_setup_procedure(need, lookup, bounds);
    let Some(procedure) = search.procedure else {
        return RecoveryOutcome::NotFound {
            consulted: search.consulted,
        };
    };

    finish_recovery(need, procedure, grant, ledger, &root, before, evidence)
}

fn finish_recovery(
    need: &PrerequisiteNeed,
    procedure: SetupProcedure,
    grant: &InstallGrant,
    ledger: &ToolchainLedger,
    root: &std::path::Path,
    before: ProbeVerdict,
    mut evidence: Vec<Evidence>,
) -> RecoveryOutcome {
    let fallback_postcondition = probe_for(&need.program);
    // Stage 8 `retain_experience`: the reconstruction record is durable whether
    // or not the grant lets the bytes land.
    if ledger.record_for(&need.program).is_none() {
        let _ = ledger.append(&ledger::ToolchainRecord {
            program: procedure.program.clone(),
            source_id: procedure.source_id.clone(),
            source_url: procedure.source_url.clone(),
            content_id: procedure.content_id.clone(),
            platform: procedure.platform,
            postcondition: procedure
                .postcondition
                .clone()
                .unwrap_or_else(|| fallback_postcondition.clone()),
            rediscover: procedure.source_url.clone(),
            observed_version: String::new(),
            installed_prefix: root.join(install::WORKSPACE_TOOLCHAIN_DIR),
        });
    }

    // Stage 6 `lower_to_tools`, under the grant.
    let (toolchain, observations) = match install::install_scoped_observed(&procedure, grant) {
        Ok(installed) => installed,
        Err(reason) => {
            return RecoveryOutcome::NotPermitted { procedure, reason };
        }
    };
    for observed in observations {
        evidence.push(setup_observation(need, &procedure, &observed));
    }

    // Stage 7 `retry_and_recheck`: a successful setup command is not success.
    let postcondition = procedure
        .postcondition
        .as_ref()
        .unwrap_or(&fallback_postcondition);
    let after = probe::probe_command(postcondition, root);
    evidence.push(observation(need, "reprobe", postcondition, &after));
    if after.is_present() {
        RecoveryOutcome::Recovered {
            toolchain,
            retry: retry_command(need),
            evidence,
        }
    } else {
        RecoveryOutcome::StillMissing { before, after }
    }
}

fn setup_observation(
    need: &PrerequisiteNeed,
    procedure: &SetupProcedure,
    observed: &install::SetupStepObservation,
) -> Evidence {
    let mut argv = vec![observed.program.clone()];
    argv.extend(observed.arguments.clone());
    let mut bytes = observed.stdout.clone();
    bytes.push_str(&observed.stderr);
    let mut record = Evidence::observed(
        argv.join(" "),
        argv,
        observed.exit_code.map(i64::from),
        bytes.as_bytes(),
        crate::execution_evidence::ObservationKind::CommandExit,
        crate::execution_evidence::EvidenceSource::LocalProcess,
    );
    record.for_need.clone_from(&need.program);
    record.produced_by = [PRODUCED_BY, "setup"].join(":");
    record.source_ids.push(procedure.source_id.clone());
    record
}

/// The probe that verifies one program: the seed row when there is one, a bare
/// version check otherwise.
fn probe_for(program: &str) -> probe::ToolchainProbe {
    probe::seed_probe_for_program(program)
        .unwrap_or_else(|| probe::ToolchainProbe::new(program, &["--version"]))
}

/// The command the caller should retry once the prerequisite is present.
fn retry_command(need: &PrerequisiteNeed) -> String {
    let mut retry = need.program.clone();
    for argument in probe_for(&need.program).argv {
        retry.push(' ');
        retry.push_str(&argument);
    }
    retry
}

/// One observation of a probe, recorded as plan 00 §4.3 evidence.
fn observation(
    need: &PrerequisiteNeed,
    stage: &str,
    probe: &probe::ToolchainProbe,
    verdict: &probe::ProbeVerdict,
) -> Evidence {
    let mut argv = vec![probe.program.clone()];
    argv.extend(probe.argv.clone());
    let mut record = Evidence::observed(
        argv.join(" "),
        argv,
        match verdict {
            probe::ProbeVerdict::Present { .. } => Some(0),
            probe::ProbeVerdict::Missing { exit_code, .. }
            | probe::ProbeVerdict::Unusable { exit_code, .. } => exit_code.map(i64::from),
            probe::ProbeVerdict::NotProbed { .. } => None,
        },
        verdict_bytes(verdict).as_bytes(),
        crate::execution_evidence::ObservationKind::CommandExit,
        crate::execution_evidence::EvidenceSource::LocalProcess,
    );
    record.for_need.clone_from(&need.program);
    record.produced_by = [PRODUCED_BY, stage].join(":");
    record
}

/// The method id every prerequisite observation is produced by.
const PRODUCED_BY: &str = "prerequisite_recover";

/// The bytes a verdict observed, hashed into the evidence record.
fn verdict_bytes(verdict: &probe::ProbeVerdict) -> String {
    match verdict {
        probe::ProbeVerdict::Present { version } => version.clone(),
        probe::ProbeVerdict::Missing { stderr, .. }
        | probe::ProbeVerdict::Unusable { stderr, .. } => stderr.clone(),
        probe::ProbeVerdict::NotProbed { reason } => reason.clone(),
    }
}

/// Classify an observed command failure: a missing prerequisite, an unusable
/// one, or an ordinary failure that is not a prerequisite at all.
///
/// Exit 127 is a need; exit 126 is not installation consent; a compiler
/// diagnostic is not a prerequisite. This is recipe stage 1, `bind_failure`.
#[must_use]
pub fn classify_failure(
    command: &str,
    exit_code: Option<i32>,
    stderr: &str,
    source_span: &str,
) -> Option<PrerequisiteNeed> {
    let program = command.split_whitespace().next().unwrap_or_default();
    if program.is_empty() {
        return None;
    }

    let missing = exit_code == Some(probe::COMMAND_NOT_FOUND_EXIT)
        || stderr.contains(probe::NOT_FOUND_MARKER);
    let denied = exit_code == Some(probe::PERMISSION_DENIED_EXIT)
        || stderr.contains(PERMISSION_DENIED_MARKER);

    if !missing && !denied {
        return None;
    }

    let observed = if missing {
        probe::ProbeVerdict::Missing {
            exit_code,
            stderr: stderr.to_owned(),
        }
    } else {
        probe::ProbeVerdict::Unusable {
            exit_code,
            stderr: stderr.to_owned(),
        }
    };

    Some(PrerequisiteNeed {
        program: program.to_owned(),
        source_span: source_span.to_owned(),
        observed,
        platform: Platform::observed(),
        requires: probe::seed_probe_for_program(program)
            .map(|probe| probe.requires)
            .unwrap_or_default(),
    })
}

/// The operating system's own wording for a file it will not execute. Kept as a
/// constant so a permission denial is never read as a missing program.
const PERMISSION_DENIED_MARKER: &str = "Permission denied";

/// The need-ledger status a prerequisite need carries right now: `Blocked` until
/// a procedure is selected, `Planned` while it runs, `Satisfied` only after a
/// re-probe returns `Present`.
#[must_use]
pub const fn need_status(
    _need: &PrerequisiteNeed,
    reprobe: Option<&ProbeVerdict>,
) -> crate::meta_frame::NeedStatus {
    let Some(verdict) = reprobe else {
        return crate::meta_frame::NeedStatus::Blocked;
    };
    crate::obligation_ledger::need_status_with_observation(
        verdict.is_present(),
        crate::meta_frame::NeedStatus::Planned,
    )
}

impl PrerequisiteNeed {
    /// The plan 00 §4.1 need record this is the in-memory projection of.
    #[must_use]
    pub fn to_need(&self, language: &str) -> crate::needs::Need {
        let mut need = crate::needs::Need::raised(
            crate::needs::NeedKind::Prerequisite,
            &self.program,
            language,
            PRODUCED_BY,
        );
        need.source_span.clone_from(&self.source_span);
        need
    }
}
