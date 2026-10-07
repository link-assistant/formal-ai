//! Issue #1187 (E151): GitHub credentials are optional in every workflow.
//!
//! The resolver action, the dispatcher action, the isolation workflow and
//! the CONTRIBUTING section are asserted directly, and so are the issue's
//! automated-test rules (R7) over every workflow and composite action: no
//! GitHub-token secret but the `AUTOMATION_*` names, every `pull_request`
//! workflow dispatchable and blind to `e2e/**` bases, and release dispatch
//! guarded by an explicit mode and the main ref. An edit still deferred
//! (release.yml's R3 guard) must stay a `pending:` row in
//! `docs/integration-manifest.md`, whose shape is pinned too.
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
    assert!(
        workflow.contains("steps.creds.outputs.can-create-repositories")
            && workflow.contains("gh repo create")
            && workflow.contains("gh repo delete"),
        "R4: only when the resolver reports can-create-repositories does the run get a separate repository, deleted at cleanup"
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

/// Every workflow file and every composite action, as `(path, body)`.
fn workflow_and_action_files() -> Vec<(String, String)> {
    let mut files = Vec::new();
    let workflows = fs::read_dir(repo_root().join(".github/workflows"))
        .expect("the workflows directory should list");
    for entry in workflows.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if std::path::Path::new(&name)
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("yml"))
        {
            let body = fs::read_to_string(entry.path()).unwrap_or_default();
            files.push((format!(".github/workflows/{name}"), body));
        }
    }
    let actions = fs::read_dir(repo_root().join(".github/actions"))
        .expect("the actions directory should list");
    for entry in actions.flatten() {
        let path = entry.path().join("action.yml");
        if let Ok(body) = fs::read_to_string(&path) {
            let name = entry.file_name().to_string_lossy().to_string();
            files.push((format!(".github/actions/{name}/action.yml"), body));
        }
    }
    files.sort();
    files
}

/// The `on:` block of a workflow: the trigger lines up to the next
/// top-level key.
fn trigger_block(body: &str) -> Vec<&str> {
    let mut lines = body.lines().skip_while(|line| {
        !(*line == "on:" || line.starts_with("on: #") || line.starts_with("on:  #"))
    });
    if lines.next().is_none() {
        return Vec::new();
    }
    lines
        .take_while(|line| line.is_empty() || line.starts_with(' ') || line.starts_with('#'))
        .collect()
}

/// The child lines of the `pull_request:` trigger, or `None` when the
/// workflow has no `pull_request` trigger.
fn pull_request_trigger<'a>(block: &[&'a str]) -> Option<Vec<&'a str>> {
    let start = block
        .iter()
        .position(|line| line.trim_end() == "  pull_request:")?;
    Some(
        block[start + 1..]
            .iter()
            .take_while(|line| line.is_empty() || line.starts_with("    "))
            .copied()
            .collect(),
    )
}

/// The GitHub-credential secret names R1 allows: the resolver's three, plus
/// the built-in `GITHUB_TOKEN`.
const ALLOWED_GITHUB_SECRETS: [&str; 4] = [
    "AUTOMATION_APP_ID",
    "AUTOMATION_APP_PRIVATE_KEY",
    "AUTOMATION_TOKEN",
    "GITHUB_TOKEN",
];

/// Keys whose value is a GitHub credential wherever they appear.
const GITHUB_CREDENTIAL_KEYS: [&str; 3] = ["GH_TOKEN:", "GITHUB_TOKEN:", "github-token:"];

#[test]
fn no_workflow_or_action_reads_a_github_token_secret_but_the_automation_names() {
    // R1 / R7: a GitHub-token secret is any secret named like one
    // (`*GITHUB*`, `GH_*`, `*BOT_TOKEN*`, `AUTOMATION_*`) or any secret
    // handed to a GitHub-credential key. Registry and marketplace tokens
    // (CARGO_TOKEN, NPM_TOKEN, DOCKERHUB_TOKEN, VSCE_PAT, ...) are not
    // GitHub credentials and stay out of scope.
    let mut violators = Vec::new();
    for (path, body) in workflow_and_action_files() {
        for (number, line) in body.lines().enumerate() {
            let trimmed = line.trim_start();
            if trimmed.starts_with('#') {
                continue;
            }
            for piece in line.split("secrets.").skip(1) {
                let name: String = piece
                    .chars()
                    .take_while(|character| character.is_ascii_alphanumeric() || *character == '_')
                    .collect();
                if name.is_empty() || ALLOWED_GITHUB_SECRETS.contains(&name.as_str()) {
                    continue;
                }
                let named_like_github = name.contains("GITHUB")
                    || name.starts_with("GH_")
                    || name.contains("BOT_TOKEN")
                    || name.starts_with("AUTOMATION_");
                let handed_to_github = GITHUB_CREDENTIAL_KEYS
                    .iter()
                    .any(|key| trimmed.starts_with(key));
                if named_like_github || handed_to_github {
                    violators.push(format!("{path}:{}: secrets.{name}", number + 1));
                }
            }
        }
    }
    assert!(
        violators.is_empty(),
        "only AUTOMATION_APP_ID, AUTOMATION_APP_PRIVATE_KEY, AUTOMATION_TOKEN and \
         github.token may serve as GitHub credentials (issue #1187 R1):\n{}",
        violators.join("\n")
    );
}

#[test]
fn every_pull_request_workflow_is_dispatchable_and_ignores_e2e_bases() {
    // R2: a bot pull request's checks start through dispatch, so every
    // `pull_request` workflow declares `workflow_dispatch`. R4: an isolated
    // orphan-branch pull request (`e2e/**` base) never triggers them.
    let mut missing = Vec::new();
    let mut pull_request_workflows = 0usize;
    for (path, body) in workflow_and_action_files() {
        let block = trigger_block(&body);
        let Some(trigger) = pull_request_trigger(&block) else {
            continue;
        };
        pull_request_workflows += 1;
        if !block
            .iter()
            .any(|line| line.starts_with("  workflow_dispatch"))
        {
            missing.push(format!("{path}: no workflow_dispatch trigger (R2)"));
        }
        if !trigger
            .iter()
            .any(|line| line.trim() == "branches-ignore: ['e2e/**']")
        {
            missing.push(format!(
                "{path}: pull_request lacks branches-ignore e2e/** (R4)"
            ));
        }
    }
    assert!(
        pull_request_workflows >= 4,
        "the scan should find the pull_request workflows, found {pull_request_workflows}"
    );
    assert!(missing.is_empty(), "{}", missing.join("\n"));
}

#[test]
fn the_named_workflows_declare_the_checks_mode_the_dispatcher_passes() {
    // R2 names layered-ci, evidence-check and workflows; each declares the
    // `mode` (options: [checks]) and `pull-request` inputs the dispatcher
    // passes, since GitHub refuses an undeclared input.
    for name in [
        "layered-ci.yml",
        "evidence-check.yml",
        "workflows.yml",
        "web-ui-boundary.yml",
    ] {
        let body = read(&format!(".github/workflows/{name}"));
        let block = trigger_block(&body).join("\n");
        assert!(
            block.contains("workflow_dispatch:")
                && block.contains("options: [checks]")
                && block.contains("pull-request:"),
            "{name} declares the checks-mode dispatch inputs"
        );
    }
    let dispatcher = read(".github/actions/dispatch-checks/action.yml");
    assert!(
        dispatcher.contains("options: \\[checks\\]"),
        "the dispatcher calls only checks-mode workflows, never a release or authoring dispatch"
    );
    assert!(
        dispatcher.contains("headRefName"),
        "the dispatch API takes a branch, so the runs go to the pull request's head branch"
    );
}

#[test]
fn every_pull_request_workflow_declares_the_checks_mode_or_a_reason_not_to() {
    // R2: the dispatcher starts every `pull_request` workflow that declares
    // the checks mode. The only exclusions are workflows whose non-pull-request
    // runs publish or spend credentials, so dispatching them from a bot branch
    // would release or leak; and the authoring workflow itself.
    let excluded = [
        // A dispatch builds and publishes desktop release assets.
        "desktop-release.yml",
        // A dispatch runs the credentialed full-slice benchmark jobs.
        "external-benchmarks.yml",
        // Its dispatch modes are release modes; `checks` is its default.
        "release.yml",
        // The workflow that opens the bot pull request and dispatches.
        "self-authored-pull-request.yml",
    ];
    let mut missing = Vec::new();
    for (path, body) in workflow_and_action_files() {
        let block = trigger_block(&body);
        if pull_request_trigger(&block).is_none()
            || excluded.iter().any(|name| path.ends_with(name))
        {
            continue;
        }
        let joined = block.join("\n");
        if !(joined.contains("options: [checks]") && joined.contains("pull-request:")) {
            missing.push(path);
        }
    }
    assert_eq!(missing, Vec::<String>::new());
}

#[test]
fn cross_repository_work_keeps_one_tracking_issue_at_the_default_layer() {
    // R5: at layer default the scan opens nothing in the owning
    // repositories and keeps one tracking issue in this repository.
    let workflow = read(".github/workflows/cross-org-duplication.yml");
    assert!(
        workflow.contains("--open requested at the default credential layer"),
        "R5: --open degrades at the default layer"
    );
    assert!(
        workflow.contains("steps.token.outputs.layer == 'default'")
            && workflow.contains("gh issue edit")
            && workflow.contains("gh issue create")
            && workflow.contains("issues: write"),
        "R5: the default layer updates one tracking issue, opening it only when none exists"
    );
}

#[test]
fn the_resolver_takes_its_secrets_as_inputs_from_every_caller() {
    // A composite action cannot read `secrets` (the runner refuses to load
    // it), so the resolver declares the three credentials as inputs and each
    // workflow that uses it passes them.
    let action = read(".github/actions/automation-token/action.yml");
    assert!(
        !action.contains("${{ secrets."),
        "a composite action that reads secrets fails to load"
    );
    for (path, body) in workflow_and_action_files() {
        if !body.contains("uses: ./.github/actions/automation-token") {
            continue;
        }
        assert!(
            body.contains("app-id: ${{ secrets.AUTOMATION_APP_ID }}")
                && body.contains("automation-token: ${{ secrets.AUTOMATION_TOKEN }}"),
            "{path} passes the AUTOMATION_* secrets to the resolver"
        );
    }
}

#[test]
fn the_self_authored_workflow_resolves_its_token_and_dispatches_checks() {
    // R1 and R2 applied to the bot pull request's own workflow.
    let workflow = read(".github/workflows/self-authored-pull-request.yml");
    assert!(
        workflow.contains("uses: ./.github/actions/automation-token"),
        "R1: the author job's token comes from the resolver"
    );
    assert!(
        workflow.contains("GH_TOKEN: ${{ steps.automation.outputs.token }}"),
        "R1: the action pushes with the resolved credential"
    );
    assert!(
        workflow.contains("uses: ./.github/actions/dispatch-checks")
            && workflow.contains("steps.automation.outputs.layer == 'default'"),
        "R2: at layer default the bot pull request's checks are dispatched"
    );
    assert!(
        workflow.contains("actions: write"),
        "R2: dispatching a workflow needs the actions scope"
    );
}

#[test]
fn release_dispatch_needs_an_explicit_mode_and_main_or_a_pending_row() {
    // R3 / R7: every release, publish or tag job reachable from
    // `workflow_dispatch` requires an explicit release mode and the main ref.
    // Until release.yml carries that guard, the edit must stay a pending
    // #1187 R3 row in docs/integration-manifest.md, so the gap is never
    // silent.
    let release = read(".github/workflows/release.yml");
    let unguarded: Vec<&str> = release
        .lines()
        .filter(|line| line.starts_with("    if:") && line.contains("workflow_dispatch"))
        .filter(|line| !line.contains("refs/heads/main"))
        .collect();
    let checks_default = trigger_block(&release)
        .iter()
        .any(|line| line.trim() == "default: checks");
    if unguarded.is_empty() && checks_default {
        return;
    }
    let manifest = read("docs/integration-manifest.md");
    let pending = manifest.split("- `pending:`").skip(1).any(|row| {
        row.trim_start()
            .starts_with("`.github/workflows/release.yml`")
            && row.contains("#1187 R3")
    });
    assert!(
        pending,
        "release.yml dispatch jobs run without the main-ref guard ({} unguarded `if:` lines, \
         checks default: {checks_default}) and no pending #1187 R3 row records it",
        unguarded.len()
    );
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
