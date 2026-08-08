use regex::Regex;
use std::{fs, path::PathBuf};

fn release_workflow() -> String {
    fs::read_to_string(format!(
        "{}/.github/workflows/release.yml",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap()
    .replace("\r\n", "\n")
}

fn workflow_files() -> Vec<PathBuf> {
    let workflow_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(".github/workflows");
    let mut paths = fs::read_dir(workflow_dir)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| {
            matches!(
                path.extension().and_then(|ext| ext.to_str()),
                Some("yml" | "yaml")
            )
        })
        .collect::<Vec<_>>();
    paths.sort();
    paths
}

fn run_blocks(workflow: &str) -> Vec<String> {
    let lines = workflow.lines().collect::<Vec<_>>();
    let mut blocks = Vec::new();
    let mut index = 0;

    while index < lines.len() {
        let line = lines[index];
        let indentation = line.len() - line.trim_start().len();
        if !line.trim_start().starts_with("run:") {
            index += 1;
            continue;
        }

        let mut block = vec![line];
        index += 1;
        while index < lines.len() {
            let next = lines[index];
            let next_indentation = next.len() - next.trim_start().len();
            if !next.trim().is_empty() && next_indentation <= indentation {
                break;
            }
            block.push(next);
            index += 1;
        }
        blocks.push(block.join("\n"));
    }

    blocks
}

fn job_block<'a>(workflow: &'a str, job_name: &str) -> &'a str {
    let marker = format!("  {job_name}:\n");
    let start = workflow.find(&marker).unwrap();
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

/// Regression test for issue #111:
/// <https://github.com/link-foundation/rust-ai-driven-development-pipeline-template/issues/111>
///
/// GitHub expands expressions in `run:` into a temporary shell script before executing it.
/// User-controlled contexts must cross that boundary through an environment variable instead.
#[test]
fn workflow_run_blocks_do_not_interpolate_untrusted_contexts() {
    let untrusted_expression =
        Regex::new(r"\$\{\{\s*(?:inputs\.|github\.event\.inputs\.|github\.head_ref)[^}]*\}\}")
            .unwrap();

    for path in workflow_files() {
        let workflow = fs::read_to_string(&path).unwrap().replace("\r\n", "\n");
        for run_block in run_blocks(&workflow) {
            assert!(
                !untrusted_expression.is_match(&run_block),
                "{} interpolates an untrusted context directly into a run block:\n{run_block}",
                path.display()
            );
        }
    }
}

/// Write-capable jobs share a non-cancelling concurrency group, while superseded
/// read-only checks are cancelled independently. This keeps a started publication
/// intact without making newer checks wait behind the whole older workflow run.
///
/// Regression test for issue #113:
/// <https://github.com/link-foundation/rust-ai-driven-development-pipeline-template/issues/113>
///
/// GitHub Actions concurrency accepts only `group` and `cancel-in-progress`.
#[test]
fn release_workflow_separates_check_and_write_concurrency() {
    let workflow = release_workflow();
    let workflow_header = workflow.split("\njobs:\n").next().unwrap();
    assert!(
        !workflow_header.contains("\nconcurrency:\n"),
        "workflow-level concurrency would couple cancellable checks to non-cancellable writes"
    );

    for job_name in [
        "detect-changes",
        "changelog",
        "version-check",
        "secrets-scan",
        "fresh-merge",
        "docker-build",
        "cargo-lock",
        "lint",
        "coverage",
        "build",
    ] {
        let job = job_block(&workflow, job_name);
        let expected_group =
            format!("group: ${{{{ github.workflow }}}}-${{{{ github.ref }}}}-{job_name}");
        assert!(
            job.contains(&expected_group) && job.contains("cancel-in-progress: true"),
            "read-only job {job_name} should cancel its superseded instance"
        );
    }

    let test = job_block(&workflow, "test");
    assert!(
        test.contains("group: ${{ github.workflow }}-${{ github.ref }}-test-${{ matrix.os }}")
            && test.contains("cancel-in-progress: true"),
        "each test matrix lane should cancel only its own superseded instance"
    );

    let write_concurrency = concat!(
        "    concurrency:\n",
        "      group: ${{ github.workflow }}-main-write\n",
        "      cancel-in-progress: false\n",
    );
    for job_name in [
        "auto-release",
        "manual-release",
        "changelog-pr",
        "deploy-docs",
    ] {
        assert!(
            job_block(&workflow, job_name).contains(write_concurrency),
            "write-capable job {job_name} should use the shared non-cancelling concurrency group"
        );
    }

    assert!(
        !workflow.contains("\n      queue:"),
        "GitHub Actions concurrency does not support a queue key"
    );
}
