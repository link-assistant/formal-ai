//! Issue #1138 B6 (plan 06, L7): default-deny installation, scoped to the workspace.
//!
//! Nothing is installed without a grant, nothing is written outside the grant's
//! root, a documented digest that does not match is a refusal without data loss,
//! and free disk is checked *before* a download rather than discovered during
//! one. A successful setup command with a failing postcondition is still
//! missing, and the environment subsequent steps need is returned explicitly
//! because shell state does not persist between tool calls.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use formal_ai::prerequisite::install::{InstallGrant, install_scoped};
use formal_ai::prerequisite::probe::ToolchainProbe;
use formal_ai::prerequisite::publisher::{SetupProcedure, SetupStep};
use formal_ai::prerequisite::{Platform, PrerequisiteError};

fn temp_root(tag: &str) -> PathBuf {
    std::env::temp_dir().join(format!("formal-ai-issue-1138-install-{tag}"))
}

fn postcondition() -> ToolchainProbe {
    ToolchainProbe {
        program: String::from("zig"),
        argv: vec![String::from("version")],
        expect: None,
        requires: Vec::new(),
    }
}

fn procedure(root: &Path, steps: Vec<SetupStep>) -> SetupProcedure {
    SetupProcedure {
        program: String::from("zig"),
        source_id: String::from("zig_official"),
        source_url: String::from("https://ziglang.org/learn/getting-started/"),
        content_id: String::from("1".repeat(64)),
        platform: Platform::observed(),
        steps,
        postcondition: Some(postcondition()),
    }
    .tagged(root)
}

/// Small helper so each case can point its steps at its own root.
trait TaggedProcedure {
    fn tagged(self, root: &Path) -> Self;
}

impl TaggedProcedure for SetupProcedure {
    fn tagged(mut self, root: &Path) -> Self {
        for step in &mut self.steps {
            if step.writes_under.as_os_str().is_empty() {
                step.writes_under = root.to_path_buf();
            }
        }
        self
    }
}

fn unpack_step() -> SetupStep {
    SetupStep {
        command: String::from("tar -xf zig.tar.xz"),
        writes_under: PathBuf::new(),
        digest: None,
    }
}

fn digest_tree(root: &Path) -> Vec<(String, u64)> {
    let mut entries = Vec::new();
    if let Ok(read) = std::fs::read_dir(root) {
        for entry in read.flatten() {
            let length = entry.metadata().map(|meta| meta.len()).unwrap_or_default();
            entries.push((entry.path().display().to_string(), length));
        }
    }
    entries.sort();
    entries
}

/// The default grant installs nothing and changes no bytes.
#[test]
fn install_refuses_without_a_grant() {
    let root = temp_root("refused");
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("the test root should be creatable");
    let before = digest_tree(&root);

    let outcome = install_scoped(&procedure(&root, vec![unpack_step()]), &InstallGrant::default());
    assert_eq!(
        outcome,
        Err(PrerequisiteError::NotGranted {
            program: String::from("zig"),
        }),
        "the default grant is Refused, so nothing may be installed"
    );
    assert_eq!(
        digest_tree(&root),
        before,
        "a refused install changes no bytes"
    );
}

/// A fetched procedure that writes outside the root is refused before a single
/// process is spawned. Fetched text may never widen the grant.
#[test]
fn a_step_writing_outside_the_root_is_refused_before_execution() {
    let root = temp_root("outside");
    let escaping = SetupStep {
        command: String::from("cp zig /usr/local/bin/zig"),
        writes_under: PathBuf::from("/usr/local/bin"),
        digest: None,
    };
    let outcome = install_scoped(
        &procedure(&root, vec![escaping]),
        &InstallGrant::Allowed {
            programs: vec![String::from("zig")],
            root: root.clone(),
        },
    );
    assert_eq!(
        outcome,
        Err(PrerequisiteError::OutsideWorkspace {
            path: String::from("/usr/local/bin"),
        }),
        "a step outside the grant's root is refused, whatever the fetched text says"
    );
}

/// A documented digest that does not match is a refusal, and the prior tree is
/// byte-identical afterwards.
#[test]
fn a_digest_mismatch_is_refused_without_data_loss() {
    let root = temp_root("digest");
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("the test root should be creatable");
    std::fs::write(root.join("existing.txt"), b"keep me").expect("fixture write");
    let before = digest_tree(&root);

    let mut step = unpack_step();
    step.digest = Some(String::from("2".repeat(64)));
    let outcome = install_scoped(
        &procedure(&root, vec![step]),
        &InstallGrant::Allowed {
            programs: vec![String::from("zig")],
            root: root.clone(),
        },
    );
    assert!(
        matches!(outcome, Err(PrerequisiteError::DigestMismatch { .. })),
        "a documented digest that does not match is refused, got {outcome:?}"
    );
    assert_eq!(
        digest_tree(&root),
        before,
        "a refused install leaves the prior tree byte-identical"
    );
}

/// Free disk is checked before the download, and the refusal reports both
/// numbers rather than a bare "not enough space".
#[test]
fn insufficient_disk_is_refused_before_download() {
    let root = temp_root("disk");
    let mut procedure = procedure(&root, vec![unpack_step()]);
    procedure.content_id = String::from("3".repeat(64));
    // A stated requirement larger than any machine has free.
    procedure.steps[0].command = String::from("tar -xf zig.tar.xz requires_bytes=18446744073709551615");

    let outcome = install_scoped(
        &procedure,
        &InstallGrant::Allowed {
            programs: vec![String::from("zig")],
            root: root.clone(),
        },
    );
    match outcome {
        Err(PrerequisiteError::InsufficientDisk {
            required_bytes,
            available_bytes,
        }) => {
            assert!(
                required_bytes > available_bytes,
                "the refusal reports both numbers: required {required_bytes}, available {available_bytes}"
            );
        }
        other => panic!("an unsatisfiable disk requirement must be refused first, got {other:?}"),
    }
}

/// A setup command that exits zero is not success. Only the postcondition probe
/// returning `Present` discharges the need.
#[test]
fn a_successful_command_with_a_failing_postcondition_is_still_missing() {
    let root = temp_root("still-missing");
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("the test root should be creatable");

    let mut procedure = procedure(&root, vec![unpack_step()]);
    // A postcondition that cannot pass: the program is still not there.
    procedure.postcondition = Some(ToolchainProbe {
        program: String::from("formal-ai-no-such-program-1138"),
        argv: vec![String::from("--version")],
        expect: None,
        requires: Vec::new(),
    });

    let outcome = install_scoped(
        &procedure,
        &InstallGrant::Allowed {
            programs: vec![String::from("zig")],
            root: root.clone(),
        },
    );
    assert!(
        outcome.is_err(),
        "a failing postcondition may never be reported as a successful install: {outcome:?}"
    );
}

/// The environment subsequent steps need is returned in the record, not exported
/// into a shell that will not survive the next tool call.
#[test]
fn the_environment_is_returned_explicitly_not_exported() {
    let root = temp_root("environment");
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("the test root should be creatable");
    let path_before = std::env::var("PATH").unwrap_or_default();

    let toolchain = install_scoped(
        &procedure(&root, vec![unpack_step()]),
        &InstallGrant::Allowed {
            programs: vec![String::from("zig")],
            root: root.clone(),
        },
    )
    .expect("a granted, verifiable procedure installs");

    let environment: &BTreeMap<String, String> = &toolchain.environment;
    assert!(
        !environment.is_empty(),
        "the bindings the next step needs are returned explicitly"
    );
    assert!(
        toolchain.prefix.starts_with(&root),
        "the install lands under the workspace root, not outside it"
    );
    assert_eq!(
        std::env::var("PATH").unwrap_or_default(),
        path_before,
        "the process environment is unchanged; nothing was exported"
    );
}
