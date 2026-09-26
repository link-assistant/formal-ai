//! Probing: find out whether a toolchain is actually here (#1138 B6, plan 06 L1).
//!
//! Today every catalogue row states availability as a constant. A probe is the
//! only thing that may set it, and "we have not looked" is a third verdict, not
//! a synonym for "unavailable".
//!
//! The probe resolves the program the way a shell would, with one addition: any
//! workspace-scoped toolchain installed under `root` (plan 06 L7's
//! `.formal-ai/toolchains/<program>/<content-id>/bin`) is placed ahead of the
//! ambient `PATH`, so a recovery that installed into the workspace is observed
//! by the same probe that observed its absence.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::seed::parser::parse_lino;

/// The seed document declaring one probe per catalogued program.
const TOOLCHAINS_LINO: &str = include_str!("../../embedded/data/seed/toolchains.lino");

/// Record type of one probe row in `data/seed/toolchains.lino`.
const RECORD_PROBE: &str = "toolchain_probe";

/// The exit status a POSIX shell reports when a command cannot be resolved.
///
/// Recorded as the observed code when the operating system reports the
/// executable is absent, because that is the code every caller in this
/// repository would have seen had the call gone through a shell.
pub const COMMAND_NOT_FOUND_EXIT: i32 = 127;

/// The exit status a POSIX shell reports for a resolvable file it may not run.
pub const PERMISSION_DENIED_EXIT: i32 = 126;

/// The shell's own wording for an unresolvable command, kept as a constant so
/// the classifier and the probe recognise the same observation.
pub const NOT_FOUND_MARKER: &str = "command not found";

/// The directory, beneath a workspace root, that holds workspace-scoped
/// toolchains. Mirrors `install::WORKSPACE_TOOLCHAIN_DIR`.
const WORKSPACE_TOOLCHAINS: &str = ".formal-ai/toolchains";

/// One executable the system may need, and how to find out whether it is here.
///
/// Read from `data/seed/toolchains.lino`; never written in Rust.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolchainProbe {
    /// The program name as a shell would resolve it (`kotlinc`, `scalac`, `docker`).
    pub program: String,
    /// The argv that proves it works, e.g. `["-version"]`. Not a compile.
    pub argv: Vec<String>,
    /// Substring the successful output must contain, when the publisher documents one.
    pub expect: Option<String>,
    /// Toolchains this one needs first (`kotlinc` → `java`).
    pub requires: Vec<String>,
}

impl ToolchainProbe {
    /// A probe for `program` with the argv a version check uses.
    #[must_use]
    pub fn new(program: &str, argv: &[&str]) -> Self {
        Self {
            program: program.to_owned(),
            argv: argv.iter().map(|argument| (*argument).to_owned()).collect(),
            expect: None,
            requires: Vec::new(),
        }
    }
}

/// What a probe observed. There are three states, not two.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProbeVerdict {
    /// Ran, exit 0, `expect` satisfied. Carries the version line observed.
    Present {
        /// The version line exactly as printed.
        version: String,
    },
    /// The program is not on the path, or exited 127.
    Missing {
        /// The exit status observed, when one was reported.
        exit_code: Option<i32>,
        /// Standard error exactly as observed.
        stderr: String,
    },
    /// Present but unusable (wrong version, broken install, permission denied).
    Unusable {
        /// The exit status observed, when one was reported.
        exit_code: Option<i32>,
        /// Standard error exactly as observed.
        stderr: String,
    },
    /// Not probed in this environment, and the reason why.
    NotProbed {
        /// Why the probe was not run.
        reason: String,
    },
}

impl ProbeVerdict {
    /// Stable slug used in the Links Notation trace and in the ledger row.
    #[must_use]
    pub const fn slug(&self) -> &'static str {
        match self {
            Self::Present { .. } => "present",
            Self::Missing { .. } => "missing",
            Self::Unusable { .. } => "unusable",
            Self::NotProbed { .. } => "not_probed",
        }
    }

    /// Whether the program was observed working. Only this discharges a
    /// prerequisite need.
    #[must_use]
    pub const fn is_present(&self) -> bool {
        matches!(self, Self::Present { .. })
    }
}

/// Every `<root>/.formal-ai/toolchains/*/*/bin` directory, in sorted order.
fn workspace_bin_directories(root: &Path) -> Vec<PathBuf> {
    let mut directories = Vec::new();
    let toolchains = root.join(WORKSPACE_TOOLCHAINS);
    let Ok(programs) = std::fs::read_dir(&toolchains) else {
        return directories;
    };
    for program in programs.flatten() {
        let Ok(revisions) = std::fs::read_dir(program.path()) else {
            continue;
        };
        for revision in revisions.flatten() {
            let bin = revision.path().join("bin");
            if bin.is_dir() {
                directories.push(bin);
            }
            if revision.path().is_dir() {
                directories.push(revision.path());
            }
        }
    }
    directories.sort();
    directories
}

/// The `PATH` a probe runs under: workspace-scoped toolchains first, the
/// ambient path after. Returned rather than exported, because shell state does
/// not persist between tool calls.
#[must_use]
pub fn workspace_path(root: &Path) -> String {
    let mut parts: Vec<String> = workspace_bin_directories(root)
        .into_iter()
        .map(|path| path.display().to_string())
        .collect();
    if let Ok(ambient) = std::env::var("PATH") {
        parts.push(ambient);
    }
    parts.join(":")
}

/// The environment bindings a workspace-scoped toolchain needs, explicit and
/// replayable rather than exported into a shell that will not survive.
#[must_use]
pub fn workspace_environment(root: &Path) -> BTreeMap<String, String> {
    let mut environment = BTreeMap::new();
    environment.insert(String::from("PATH"), workspace_path(root));
    environment
}

/// Run `probe` in `root` and report exactly what happened.
///
/// Deterministic in its verdict shape; the observed version string is recorded,
/// never asserted against a hard-coded value.
#[must_use]
pub fn probe_command(probe: &ToolchainProbe, root: &Path) -> ProbeVerdict {
    if probe.program.trim().is_empty() {
        return ProbeVerdict::NotProbed {
            reason: String::from("no program declared"),
        };
    }

    let mut command = Command::new(&probe.program);
    command.args(&probe.argv);
    command.env("PATH", workspace_path(root));
    if root.is_dir() {
        command.current_dir(root);
    }

    let observed = match command.output() {
        Ok(observed) => observed,
        Err(error) => {
            if error.kind() == std::io::ErrorKind::NotFound {
                return ProbeVerdict::Missing {
                    exit_code: Some(COMMAND_NOT_FOUND_EXIT),
                    stderr: [probe.program.as_str(), NOT_FOUND_MARKER, &error.to_string()]
                        .join(": "),
                };
            }
            return ProbeVerdict::Unusable {
                exit_code: None,
                stderr: error.to_string(),
            };
        }
    };

    let stdout = String::from_utf8_lossy(&observed.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&observed.stderr).into_owned();
    let exit_code = observed.status.code();

    if exit_code == Some(COMMAND_NOT_FOUND_EXIT) || stderr.contains(NOT_FOUND_MARKER) {
        return ProbeVerdict::Missing { exit_code, stderr };
    }
    if !observed.status.success() {
        return ProbeVerdict::Unusable { exit_code, stderr };
    }

    let printed = if stdout.trim().is_empty() {
        &stderr
    } else {
        &stdout
    };
    if let Some(expected) = probe.expect.as_ref()
        && !printed.contains(expected.as_str())
    {
        return ProbeVerdict::Unusable { exit_code, stderr };
    }

    ProbeVerdict::Present {
        version: printed.lines().next().unwrap_or_default().trim().to_owned(),
    }
}

/// Every probe declared in `data/seed/toolchains.lino`, in file order.
#[must_use]
pub fn seed_probes() -> Vec<ToolchainProbe> {
    let root = parse_lino(TOOLCHAINS_LINO);
    let mut probes = Vec::new();
    for node in &root.children {
        if node.find_child_value("record_type") != RECORD_PROBE {
            continue;
        }
        probes.push(ToolchainProbe {
            program: node.find_child_value("program").to_owned(),
            argv: split_tuple(node.find_child_value("argv")),
            expect: {
                let expect = node.find_child_value("expect");
                if expect.trim().is_empty() {
                    None
                } else {
                    Some(expect.to_owned())
                }
            },
            requires: split_tuple(node.find_child_value("requires")),
        });
    }
    probes
}

/// The probe declared for one catalogue language slug, when seed declares one.
#[must_use]
pub fn seed_probe_for_language(language: &str) -> Option<ToolchainProbe> {
    let root = parse_lino(TOOLCHAINS_LINO);
    root.children
        .iter()
        .find(|node| {
            node.find_child_value("record_type") == RECORD_PROBE
                && node.find_child_value("language") == language
        })
        .map(|node| ToolchainProbe {
            program: node.find_child_value("program").to_owned(),
            argv: split_tuple(node.find_child_value("argv")),
            expect: None,
            requires: split_tuple(node.find_child_value("requires")),
        })
}

/// The install hint seed carries for one catalogue language slug.
///
/// Issue #1138 plan 06 leaf L3: these were fourteen `&'static str` constants in
/// `src/coding/catalog/languages.rs`, unreachable from the install path that
/// needed them. They are rows now, beside the probe for the same program.
#[must_use]
pub fn seed_setup_hint(language: &str) -> String {
    seed_field(language, "setup_hint")
}

/// The environment description seed carries for one catalogue language slug.
#[must_use]
pub fn seed_environment(language: &str) -> String {
    seed_field(language, "environment")
}

/// The execution status a recorded harness run observed for one catalogue
/// language slug: `verified`, `unavailable`, or absent, which is `not_probed`.
#[must_use]
pub fn seed_execution_status(language: &str) -> String {
    seed_field(language, "execution_status")
}

/// One field of the seed row for a catalogue language slug.
fn seed_field(language: &str, field: &str) -> String {
    parse_lino(TOOLCHAINS_LINO)
        .children
        .iter()
        .find(|node| {
            node.find_child_value("record_type") == RECORD_PROBE
                && node.find_child_value("language") == language
        })
        .map(|node| node.find_child_value(field).to_owned())
        .unwrap_or_default()
}

/// The probe declared for one program, when seed declares one.
#[must_use]
pub fn seed_probe_for_program(program: &str) -> Option<ToolchainProbe> {
    seed_probes()
        .into_iter()
        .find(|probe| probe.program == program)
}

/// Split a Links Notation tuple value `("a" "b")` into its members.
///
/// Shared with plan 03's command allowlist, which reads argument shapes out of
/// the same tuple form.
#[must_use]
pub fn split_tuple(value: &str) -> Vec<String> {
    value
        .split_whitespace()
        .map(|element| element.trim_matches(['(', ')', '"']).to_owned())
        .filter(|element| !element.is_empty())
        .collect()
}
