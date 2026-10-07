//! Issue #1187 (E151): GitHub credentials are optional in every workflow.
//!
//! The branch's drafted state is asserted directly (the resolver action,
//! the dispatcher action, the isolation workflow, the CONTRIBUTING
//! section). The edits to *existing* workflow files are held by the
//! integration GUARD until the #1168/#1169 commits land, so they are
//! tracked in `docs/integration-manifest.md`; this test pins the manifest
//! shape so a deferred edit can never be forgotten silently — every
//! `pending:` row must name a file and an issue, and the deferred files
//! must not silently regress the naming rule once they migrate (the
//! manifest row is the reminder).
//!
//! Run: `RUSTUP_TOOLCHAIN=1.98.1 cargo test --test unit issue_1187_`

use std::fs;
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    // Unit tests run with CWD at the crate root (rust/); the repository
    // root is one directory up.
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crate sits inside the repository")
        .to_path_buf()
}

fn read(path: &str) -> String {
    let full = repo_root().join(path);
    fs::read_to_string(&full).unwrap_or_else(|error| panic!("{path} should be readable: {error}"))
}

#[test]
fn the_resolver_action_implements_the_three_layers() {
    let action = read(".github/actions/automation-token/action.yml");
    assert!(
        action.contains("AUTOMATION_APP_ID") && action.contains("AUTOMATION_APP_PRIVATE_KEY"),
        "layer 1 (GitHub App) must be resolved from the AUTOMATION_* secrets"
    );
    assert!(
        action.contains("AUTOMATION_TOKEN"),
        "layer 2 must be the single secret name AUTOMATION_TOKEN"
    );
    assert!(
        action.contains("github.token"),
        "layer 3 must fall back to the built-in github.token"
    );
    assert!(
        action.contains("GITHUB_STEP_SUMMARY"),
        "R1: the resolved layer is printed to the job summary"
    );
    assert!(
        action.contains("can-create-repositories"),
        "R4: repository creation is decided by the resolver output"
    );
}

#[test]
fn the_dispatcher_dispatches_pull_request_workflows() {
    let action = read(".github/actions/dispatch-checks/action.yml");
    assert!(
        action.contains("gh workflow run"),
        "R2: checks start through workflow dispatch, which needs no approval"
    );
    assert!(
        action.contains("workflow_dispatch"),
        "workflows without a dispatch trigger are reported, not skipped silently"
    );
}

#[test]
fn orphan_branch_isolation_needs_no_credential() {
    let workflow = read(".github/workflows/e2e-isolation.yml");
    assert!(
        workflow.contains("git checkout --orphan"),
        "R4: isolation is an orphan branch with its own tree"
    );
    assert!(
        workflow.contains("e2e-task"),
        "R4: the run tracks itself with an e2e-task labelled issue"
    );
    assert!(
        workflow.contains("automation-token"),
        "R1: even the isolation workflow takes its token from the resolver"
    );
    assert!(
        workflow.contains("e2e/${PURPOSE}/${run_id}/${name}"),
        "R4: the branch layout is e2e/<purpose>/<run_id>/<name>"
    );
}

#[test]
fn contributing_states_the_credential_rules() {
    let contributing = read("CONTRIBUTING.md");
    let section = contributing
        .split("## GitHub credentials")
        .nth(1)
        .expect("CONTRIBUTING.md should carry the GitHub credentials section (R6)");
    assert!(section.contains("AUTOMATION_TOKEN"), "the one token name");
    assert!(
        section.contains("optional in all cases"),
        "tokens are optional, never mandatory"
    );
    assert!(
        section.contains("dispatch"),
        "checks-without-approval via dispatch is documented"
    );
}

#[test]
fn no_workflow_reads_a_foreign_token_secret() {
    // R1's naming rule, enforced on the workflows this branch adds. The
    // pre-existing workflows that still read FORMAL_AI_BOT_TOKEN are the
    // GUARD-tracked migrations; every one of them must appear in the
    // integration manifest as pending, which is asserted below so the
    // naming rule extends to them exactly when they migrate.
    let mut violators = Vec::new();
    let workflows = fs::read_dir(repo_root().join(".github/workflows"))
        .expect("the workflows directory should list");
    for entry in workflows.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if !std::path::Path::new(&name)
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("yml"))
        {
            continue;
        }
        let body = fs::read_to_string(entry.path()).unwrap_or_default();
        if body.contains("secrets.FORMAL_AI_BOT_TOKEN") {
            violators.push(name);
        }
    }
    let manifest = read("docs/integration-manifest.md");
    for violator in &violators {
        assert!(
            manifest.contains(violator),
            "{violator} still reads secrets.FORMAL_AI_BOT_TOKEN and is not recorded as a pending migration in docs/integration-manifest.md"
        );
    }
}

#[test]
fn every_pending_manifest_row_names_a_file_and_an_issue() {
    let manifest = read("docs/integration-manifest.md");
    for line in manifest
        .lines()
        .filter(|line| line.starts_with("- `pending:`"))
    {
        let rest = line.trim_start_matches("- `pending:` ").trim();
        assert!(
            rest.starts_with('`') && rest.contains(".yml`") || rest.contains(".toml`"),
            "a pending row names the file it defers: {line}"
        );
        assert!(
            rest.contains('#'),
            "a pending row names the issue that owns the edit: {line}"
        );
    }
}
