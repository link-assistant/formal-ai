//! Shared fixtures for the release-workflow CI/CD unit tests.
//!
//! These helpers read the real workflow YAML from the repo and slice out
//! individual jobs/steps so the assertions in `workflow_release.rs` and
//! `release_publishing.rs` can stay focused and small.

use std::fs;
use std::path::Path;

/// The declared job cap, including matrix expressions.
pub fn job_timeout(job: &str) -> Option<&str> {
    job.lines()
        .find_map(|line| line.trim().strip_prefix("timeout-minutes:"))
        .map(str::trim)
}

/// Resolve a literal cap or every row of the maintained literal include matrix.
/// Unsupported axes, expressions, missing fields and incomplete rows fail closed.
pub fn minimum_job_timeout(job: &str) -> Option<u64> {
    let raw = job_timeout(job)?;
    if let Ok(value) = raw.parse::<u64>() {
        return (value > 0).then_some(value);
    }
    let key = raw.strip_prefix("${{ matrix.")?.strip_suffix(" }}")?;
    if key.is_empty()
        || !key
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
    {
        return None;
    }
    let mut lines = job.lines();
    let matrix = lines.find(|line| line.trim() == "matrix:")?;
    let indent = matrix.len() - matrix.trim_start().len();
    let mut include = false;
    let mut caps = Vec::new();
    for line in lines {
        let text = line.trim();
        if text.is_empty() || text.starts_with('#') {
            continue;
        }
        let depth = line.len() - line.trim_start().len();
        if depth <= indent {
            break;
        }
        if text == "include:" && depth == indent + 2 && !include {
            include = true;
            continue;
        }
        if !include || depth != indent + 4 {
            return None;
        }
        let row = text.strip_prefix("- {")?.strip_suffix('}')?;
        let last = row.rsplit(',').next()?.trim();
        let (name, value) = last.split_once(':')?;
        if name.trim() != key {
            return None;
        }
        let value = value.trim().parse::<u64>().ok()?;
        if value == 0 {
            return None;
        }
        caps.push(value);
    }
    caps.into_iter().min()
}

#[test]
fn matrix_job_deadlines_cover_every_literal_leg_and_refuse_unknown_forms() {
    let job = "  native:\n    timeout-minutes: ${{ matrix.capmin }}\n    strategy:\n      matrix:\n        include:\n          - { os: linux, capmin: 30 }\n          - { os: intel, capmin: 45 }\n    steps:\n      - uses: actions/checkout@v7\n        timeout-minutes: 5\n";
    assert_eq!(minimum_job_timeout(job), Some(30));
    assert_eq!(
        minimum_job_timeout(&job.replace("capmin: 45", "capmin: 6")),
        Some(6)
    );
    for changed in [
        job.replace("capmin: 45", "other: 45"),
        job.replace("capmin: 45", "capmin: 0"),
        job.replace("capmin: 45", "capmin: expression"),
        job.replace("capmin: 45", "note: \"fake, capmin: 45\""),
        job.replace("include:", "exclude:"),
        job.replace("${{ matrix.capmin }}", "${{ inputs.minutes }}"),
        job.replace(
            "          - { os: intel, capmin: 45 }",
            "          - os: intel\n            capmin: 45",
        ),
    ] {
        assert_eq!(minimum_job_timeout(&changed), None, "{changed}");
    }
    assert_eq!(minimum_job_timeout("    timeout-minutes: 20\n"), Some(20));
    assert_eq!(minimum_job_timeout("    timeout-minutes: 0\n"), None);
    assert_eq!(
        minimum_job_timeout("    timeout-minutes: ${{ matrix.capmin }}\n"),
        None
    );
}

/// Everything CI executes for a pull request, workflow and gate registry as one
/// text.
///
/// Issue #991 moved the `lint` job's checks into `data/meta/ci-gates/`, so a
/// test that asks "does CI run this check?" must read both files.
/// `crate::ci_gates` answers that for the whole suite; it is re-exported here
/// because these fixtures are where the workflow tests already look.
pub use crate::ci_gates::{ci_surface, release_workflow};

/// Issue #895: the two coverage denominators live in their own workflow. They
/// are a leaf of the release graph -- nothing `needs:` them -- so moving them
/// out of `release.yml` changed no ordering, and it keeps that file under the
/// 1500-line ceiling `scripts/check-file-size.rs` enforces.
pub fn coverage_workflow() -> String {
    fs::read_to_string(format!(
        "{}/../.github/workflows/coverage.yml",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap()
    .replace("\r\n", "\n")
}

/// PR #1188 (R1188-U9): the Pages build (web bundle, `cargo doc`, stamp,
/// upload) runs in its own reusable workflow under a 30-minute cap, and the
/// release workflow's `deploy-pages` job only deploys its artifact.
pub fn pages_artifact_workflow() -> String {
    fs::read_to_string(format!(
        "{}/../.github/workflows/pages-artifact.yml",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap()
    .replace("\r\n", "\n")
}

pub fn self_development_status_workflow() -> String {
    fs::read_to_string(format!(
        "{}/../.github/workflows/self-development-status.yml",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap()
    .replace("\r\n", "\n")
}

pub fn desktop_release_workflow() -> String {
    fs::read_to_string(format!(
        "{}/../.github/workflows/desktop-release.yml",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap()
    .replace("\r\n", "\n")
}

/// The same workflow with every run of whitespace collapsed to one space.
///
/// Issue #1081: two gates pinned one line-wrapping of `links.yml`'s `if:`
/// expression as a literal, so folding a condition across lines to add a term
/// to it read to them as a deletion -- a true failure for a change that removed
/// nothing. A condition means the same thing however it is wrapped, so the
/// assertions that care about one ask it of this rather than of the raw file.
pub fn unwrapped(workflow: &str) -> String {
    workflow.split_whitespace().collect::<Vec<_>>().join(" ")
}

pub fn job_block<'a>(workflow: &'a str, job_name: &str) -> &'a str {
    let marker = format!("  {job_name}:\n");
    let start = workflow.find(&format!("\n{marker}")).unwrap() + 1;
    let body_start = start + marker.len();
    let rest = &workflow[body_start..];

    let next_job = rest
        .lines()
        .scan(0usize, |offset, line| {
            let current_offset = *offset;
            *offset += line.len() + 1;
            Some((current_offset, line))
        })
        .find_map(|(offset, line)| {
            let starts_at_job_indent = line.starts_with("  ") && !line.starts_with("    ");
            (starts_at_job_indent && line.trim_end().ends_with(':')).then_some(offset)
        });

    next_job.map_or_else(
        || &workflow[start..],
        |end| &workflow[start..body_start + end],
    )
}

#[test]
fn job_lookup_ignores_a_nested_input_with_the_same_name() {
    let workflow = "on:\n  workflow_dispatch:\n    inputs:\n      publish:\n        type: boolean\njobs:\n  publish:\n    timeout-minutes: 30\n  resolve:\n    timeout-minutes: 5\n";
    assert_eq!(
        job_block(workflow, "publish"),
        "  publish:\n    timeout-minutes: 30\n"
    );
}

pub fn workflow_step_block<'a>(job: &'a str, step_name: &str) -> &'a str {
    let marker = format!("      - name: {step_name}\n");
    let start = job.find(&marker).unwrap();
    let body_start = start + marker.len();
    let rest = &job[body_start..];

    let next_step = rest
        .lines()
        .scan(0usize, |offset, line| {
            let current_offset = *offset;
            *offset += line.len() + 1;
            Some((current_offset, line))
        })
        .find_map(|(offset, line)| line.starts_with("      - ").then_some(offset));

    next_step.map_or_else(|| &job[start..], |end| &job[start..body_start + end])
}

pub fn workflow_job_names(workflow: &str) -> Vec<&str> {
    let marker = "jobs:\n";
    let start = workflow.find(marker).unwrap() + marker.len();

    workflow[start..]
        .lines()
        .filter_map(|line| {
            let starts_at_job_indent = line.starts_with("  ") && !line.starts_with("    ");
            // A comment at job indentation is not a job. Issue #1017: a prose
            // line that happened to end in a colon was parsed as one, which
            // made every job-sweeping contract fail against a phantom job whose
            // "name" was the sentence -- an error that points at the wrong
            // thing entirely and takes a CI round trip to see.
            let is_comment = line.trim_start().starts_with('#');
            (starts_at_job_indent && !is_comment && line.trim_end().ends_with(':'))
                .then(|| line.trim().trim_end_matches(':'))
        })
        .collect()
}

/// The workflow files, and only those.
///
/// Most callers parse what they get here as a workflow -- `jobs:`, job caps,
/// concurrency groups -- so a composite action, which has none of those, makes
/// them panic rather than fail with a reason. `ci_shell_files` is the list for
/// contracts about the shell CI runs, wherever it lives.
pub fn workflow_files() -> Vec<(String, String)> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("the repository root sits one level above the crate");
    let mut files = Vec::new();
    collect_ci_files(
        &root.join(".github/workflows"),
        root,
        &mut files,
        &["yml", "yaml"],
    );
    files.sort();
    assert!(!files.is_empty(), "no workflow files found");
    files
}

/// Everything CI executes as shell: the workflows, the composite actions, and
/// the scripts either of them runs.
///
/// Issue #1085 moved the self-authored authoring loop into
/// `.github/actions/author-with-formal-ai/` so another repository can install
/// it. A contract about the shell -- no bare `git push`, no credentialed
/// checkout without a reason -- has to read this list, or moving a line one
/// directory across would move it out of review. The shell CI runs is the same
/// shell either way.
pub fn ci_shell_files() -> Vec<(String, String)> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("the repository root sits one level above the crate");
    let mut files = Vec::new();
    collect_ci_files(
        &root.join(".github/workflows"),
        root,
        &mut files,
        &["yml", "yaml"],
    );
    collect_ci_files(
        &root.join(".github/actions"),
        root,
        &mut files,
        &["yml", "yaml", "sh"],
    );
    files.sort();
    assert!(!files.is_empty(), "no CI shell files found");
    files
}

fn collect_ci_files(
    dir: &Path,
    root: &Path,
    files: &mut Vec<(String, String)>,
    extensions: &[&str],
) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries {
        let path = entry.expect("workflow entry").path();
        if path.is_dir() {
            collect_ci_files(&path, root, files, extensions);
            continue;
        }
        if !path
            .extension()
            .is_some_and(|ext| extensions.iter().any(|allowed| ext == *allowed))
        {
            continue;
        }
        let name = path
            .strip_prefix(root)
            .unwrap_or(&path)
            .to_string_lossy()
            .into_owned();
        let name = name
            .strip_prefix(".github/workflows/")
            .unwrap_or(&name)
            .to_owned();
        files.push((
            name,
            fs::read_to_string(&path).unwrap().replace("\r\n", "\n"),
        ));
    }
}

/// Ordinary job groups retain their existing rule; delegated groups must protect main.
pub fn read_only_group(job: &str, job_name: &str, workflows: &[(String, String)]) -> bool {
    if job
        .lines()
        .any(|line| line.trim_end() == "    concurrency:")
    {
        return true;
    }
    let called = job
        .lines()
        .find_map(|line| line.trim().strip_prefix("uses: ./.github/workflows/"));
    let Some((_, source)) =
        called.and_then(|called| workflows.iter().find(|(name, _)| name.as_str() == called))
    else {
        return false;
    };
    let header = source.split("\njobs:\n").next().unwrap_or_default();
    if !header.contains("workflow_call:") {
        return false;
    }
    let Some(group) = header.split("\nconcurrency:\n").nth(1) else {
        return false;
    };
    let fields: Vec<_> = group
        .lines()
        .take_while(|line| line.starts_with("  "))
        .collect();
    let expected_group =
        format!("  group: ${{{{ github.workflow }}}}-${{{{ github.ref }}}}-{job_name}");
    fields.contains(&expected_group.as_str())
        && fields.contains(&"  cancel-in-progress: ${{ github.ref != 'refs/heads/main' }}")
}

#[test]
fn delegated_read_only_group_requires_real_callee_and_protected_main_policy() {
    let job = "  evidence:\n    uses: ./.github/workflows/evidence.yml\n";
    let source = "name: Evidence\non:\n  workflow_call:\nconcurrency:\n  group: ${{ github.workflow }}-${{ github.ref }}-evidence\n  cancel-in-progress: ${{ github.ref != 'refs/heads/main' }}\njobs:\n  inspect:\n    runs-on: ubuntu-latest\n";
    let catalog = |text: &str| vec![("evidence.yml".to_owned(), text.to_owned())];
    assert!(read_only_group(job, "evidence", &catalog(source)));
    assert!(!read_only_group(job, "evidence", &[]));
    for changed in [
        source.replace("concurrency:", "unowned:"),
        source.replace("workflow_call:", "push:"),
        source.replace("-evidence", "-parent"),
        source.replace("${{ github.ref != 'refs/heads/main' }}", "true"),
        source.replace("${{ github.ref != 'refs/heads/main' }}", "false"),
    ] {
        assert!(!read_only_group(job, "evidence", &catalog(&changed)));
    }
    assert!(read_only_group(
        "  ordinary:\n    concurrency:\n      group: existing\n",
        "ordinary",
        &[]
    ));
}
