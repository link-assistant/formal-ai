"""Regression tests for GitHub Actions workflow policy."""

from __future__ import annotations

import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
WORKFLOWS = ROOT / ".github" / "workflows"


def read_workflow(name: str) -> str:
    """Read a workflow file by name."""
    return (WORKFLOWS / name).read_text(encoding="utf-8")


def workflow_job_block(workflow: str, job_name: str) -> str:
    """Return the YAML text block for one top-level workflow job."""
    lines = workflow.splitlines()
    start = next(index for index, line in enumerate(lines) if line == f"  {job_name}:")
    end = next(
        (
            index
            for index, line in enumerate(lines[start + 1 :], start + 1)
            if re.match(r"^  [A-Za-z0-9_-]+:$", line)
        ),
        len(lines),
    )
    return "\n".join(lines[start:end])


def workflow_step_block(job_block: str, step_name: str) -> str:
    """Return the YAML text block for one named workflow step."""
    lines = job_block.splitlines()
    start = next(
        index
        for index, line in enumerate(lines)
        if line.strip() == f"- name: {step_name}"
    )
    end = next(
        (
            index
            for index, line in enumerate(lines[start + 1 :], start + 1)
            if re.match(r"^      - ", line)
        ),
        len(lines),
    )
    return "\n".join(lines[start:end])


def workflow_run_blocks(workflow: str) -> list[str]:
    """Return every shell ``run:`` block from a workflow."""
    lines = workflow.splitlines()
    blocks: list[str] = []
    index = 0

    while index < len(lines):
        line = lines[index]
        indentation = len(line) - len(line.lstrip())
        if not line.lstrip().startswith("run:"):
            index += 1
            continue

        block = [line]
        index += 1
        while index < len(lines):
            next_line = lines[index]
            next_indentation = len(next_line) - len(next_line.lstrip())
            if next_line.strip() and next_indentation <= indentation:
                break
            block.append(next_line)
            index += 1
        blocks.append("\n".join(block))

    return blocks


def assert_action_pin_count(
    workflow: str, action: str, version: str, count: int
) -> None:
    """Assert every expected action reference is pinned to the requested version."""
    pattern = rf"uses:\s+{re.escape(action)}@{re.escape(version)}\b"
    assert len(re.findall(pattern, workflow)) == count


def assert_action_pin_absent(workflow: str, action: str, version: str) -> None:
    """Assert an outdated action reference is not used."""
    pattern = rf"uses:\s+{re.escape(action)}@{re.escape(version)}\b"
    assert not re.search(pattern, workflow)


def test_workflow_run_blocks_do_not_interpolate_untrusted_inputs() -> None:
    """Contributor-controlled inputs must reach shell scripts through env vars."""
    unsafe_expression = re.compile(
        r"\$\{\{\s*(?:inputs\.|github\.event\.inputs\.|github\.(?:base|head)_ref)"
        r"[^}]*\}\}"
    )

    for path in sorted(WORKFLOWS.glob("*.y*ml")):
        workflow = path.read_text(encoding="utf-8")
        for run_block in workflow_run_blocks(workflow):
            assert not unsafe_expression.search(run_block), (
                f"{path.name} interpolates an untrusted expression in a run block:\n"
                f"{run_block}"
            )


def test_changelog_check_safely_requires_a_fragment() -> None:
    """Source-changing pull requests must fail safely without a fragment."""
    workflow = read_workflow("release.yml")
    changelog_job = workflow_job_block(workflow, "changelog")
    check_step = workflow_step_block(changelog_job, "Check for changelog fragments")

    assert "GITHUB_BASE_REF: ${{ github.base_ref }}" in check_step
    assert "set -euo pipefail" in check_step
    assert 'git diff --name-only "origin/${GITHUB_BASE_REF}...HEAD"' in check_step
    assert 'grep -cE "$SOURCE_PATTERN" || true' in check_step
    assert "::error::No changelog fragment found." in check_step
    assert "::warning::No changelog fragment found." not in check_step
    assert "exit 1" in check_step
    assert "exit 0" not in check_step


def test_release_workflow_separates_check_and_write_concurrency() -> None:
    """Checks cancel independently while release writes share a FIFO queue."""
    workflow = read_workflow("release.yml")
    workflow_header = workflow.split("\njobs:\n", maxsplit=1)[0]

    assert "\nconcurrency:\n" not in workflow_header

    for job_name in (
        "detect-changes",
        "lint",
        "test",
        "build",
        "changelog",
        "docker-build",
    ):
        block = workflow_job_block(workflow, job_name)
        expected_group = (
            "group: ${{ github.workflow }}-${{ github.ref }}-" f"{job_name}"
        )
        assert expected_group in block
        assert "cancel-in-progress: true" in block

    write_concurrency = "\n".join(
        (
            "    concurrency:",
            "      group: ${{ github.workflow }}-main-write",
            "      cancel-in-progress: false",
            "      queue: max",
        )
    )
    for job_name in ("auto-release", "manual-release"):
        assert write_concurrency in workflow_job_block(workflow, job_name)


def test_release_workflow_uses_least_privilege_permissions() -> None:
    """Only publishing jobs should receive write-capable tokens."""
    workflow = read_workflow("release.yml")

    assert "\npermissions:\n  contents: read\n" in workflow

    for job_name in ("auto-release", "manual-release"):
        block = workflow_job_block(workflow, job_name)
        assert "permissions:\n      contents: write\n      id-token: write" in block


def test_release_workflow_jobs_have_explicit_timeouts() -> None:
    """Release workflow jobs should fail fast instead of using the six-hour default."""
    workflow = read_workflow("release.yml")

    expected_timeouts = {
        "detect-changes": 5,
        "lint": 20,
        "test": 30,
        "build": 20,
        "changelog": 10,
        "docker-build": 60,
        "auto-release": 30,
        "manual-release": 30,
    }

    for job_name, timeout in expected_timeouts.items():
        block = workflow_job_block(workflow, job_name)
        assert f"timeout-minutes: {timeout}" in block


def test_release_workflow_action_versions_are_current() -> None:
    """Release workflow actions should use the current major versions."""
    release_workflow = read_workflow("release.yml")

    assert_action_pin_count(release_workflow, "actions/checkout", "v6", 8)
    assert_action_pin_count(release_workflow, "actions/setup-python", "v6", 7)
    assert_action_pin_count(release_workflow, "actions/upload-artifact", "v7", 1)
    assert_action_pin_count(release_workflow, "actions/download-artifact", "v7", 1)
    assert_action_pin_count(release_workflow, "codecov/codecov-action", "v7", 1)

    assert_action_pin_absent(release_workflow, "actions/setup-python", "v5")
    assert_action_pin_absent(release_workflow, "codecov/codecov-action", "v4")


def test_release_workflow_sets_git_default_branch_before_checkout() -> None:
    """Release workflow should suppress Git's default branch hint during checkout."""
    workflow = read_workflow("release.yml")

    assert "env:\n  GIT_CONFIG_COUNT: '1'" in workflow
    assert "  GIT_CONFIG_KEY_0: init.defaultBranch" in workflow
    assert "  GIT_CONFIG_VALUE_0: main" in workflow

    env_index = workflow.index("env:\n  GIT_CONFIG_COUNT: '1'")
    first_checkout_index = workflow.index("uses: actions/checkout@v6")
    assert env_index < first_checkout_index


def test_release_workflow_gates_codecov_upload_on_token() -> None:
    """Codecov uploads should be skipped without a token and fail loudly with one."""
    workflow = read_workflow("release.yml")
    test_job = workflow_job_block(workflow, "test")
    skip_step = workflow_step_block(test_job, "Report skipped Codecov upload")
    upload_step = workflow_step_block(test_job, "Upload coverage to Codecov")

    assert "CODECOV_TOKEN: ${{ secrets.CODECOV_TOKEN }}" in test_job
    assert "if: env.CODECOV_TOKEN == ''" in skip_step
    assert "::notice::" in skip_step
    assert "if: env.CODECOV_TOKEN != ''" in upload_step
    assert "uses: codecov/codecov-action@v7" in upload_step
    assert "files: ${{ steps.python_layout.outputs.root }}/coverage.xml" in upload_step
    assert "\n          file:" not in upload_step
    assert "token: ${{ env.CODECOV_TOKEN }}" in upload_step
    assert "disable_search: true" in upload_step
    assert "fail_ci_if_error: true" in upload_step
    assert "fail_ci_if_error: false" not in upload_step


def test_release_workflow_auto_detects_python_layout() -> None:
    """Release workflow should support root and python/ package layouts."""
    workflow = read_workflow("release.yml")

    assert "if [ -f pyproject.toml ]; then" in workflow
    assert "elif [ -f python/pyproject.toml ]; then" in workflow
    assert "root=python" in workflow
    assert "multi_language=true" in workflow


def test_release_workflow_namespaces_multi_language_python_tags() -> None:
    """Multi-language releases should use py_v tags and plain root releases keep v."""
    workflow = read_workflow("release.yml")
    auto_release = workflow_job_block(workflow, "auto-release")

    assert 'TAG="py_v$CURRENT_VERSION"' in auto_release
    assert 'TAG="v$CURRENT_VERSION"' in auto_release
    assert 'git rev-parse "$TAG"' in auto_release


def test_release_workflow_runs_python_steps_from_detected_root() -> None:
    """Package build and release commands should run against the detected root."""
    workflow = read_workflow("release.yml")

    assert 'cd "${{ steps.python_layout.outputs.root }}"' in workflow
    assert "path: ${{ steps.python_layout.outputs.dist_dir }}" in workflow
    assert "packages-dir: ${{ steps.python_layout.outputs.dist_dir }}" in workflow
    assert (
        'python "${{ steps.python_layout.outputs.root }}/scripts/create_github_release.py"'
        in workflow
    )


STATUS_CHECK_FUNCTIONS = ("always()", "!cancelled()", "!failure()", "success()")


def job_condition(workflow: str, job_name: str) -> str:
    """Return the ``if:`` condition text for a workflow job."""
    block = workflow_job_block(workflow, job_name)
    match = re.search(r"^    if:(.*?)(?=^    [a-z])", block, re.DOTALL | re.MULTILINE)
    assert match, f"job {job_name!r} has no if condition"
    return match.group(1)


def test_dispatch_dependent_jobs_use_status_check_function() -> None:
    """Jobs that depend on skippable jobs must override the default status gate.

    ``detect-changes`` is skipped for ``workflow_dispatch``. GitHub Actions skips
    a job whose dependency was skipped unless the dependent ``if`` condition
    includes a status-check function (``always()``, ``!cancelled()``, ...).
    Without it, a manual release silently skips lint/test and then the release
    itself even though it appears successful.
    """
    workflow = read_workflow("release.yml")

    for job_name in ("lint", "test", "manual-release"):
        condition = job_condition(workflow, job_name)
        assert any(fn in condition for fn in STATUS_CHECK_FUNCTIONS), (
            f"job {job_name!r} depends on a skippable job but its if condition "
            f"does not start with a status-check function: {condition!r}"
        )


def test_release_workflow_propagates_cancellation() -> None:
    """Dependent jobs must stop when a workflow run is cancelled."""
    workflow = read_workflow("release.yml")

    assert "always() && !cancelled()" not in workflow
    for job_name in ("lint", "test", "build", "manual-release"):
        condition = job_condition(workflow, job_name)
        assert "!cancelled()" in condition
        assert "always()" not in condition


def test_change_gated_jobs_use_detector_for_pull_requests_and_pushes() -> None:
    """Automatic events must use the same authoritative detector output."""
    workflow = read_workflow("release.yml")

    for job_name in ("lint", "test", "build"):
        condition = job_condition(workflow, job_name)
        assert "needs.detect-changes.outputs.any-code-changed == 'true'" in condition
        assert "github.event_name == 'push'" not in condition
        assert "github.event_name == 'workflow_dispatch'" in condition


def test_detect_changes_only_exports_consumed_outputs() -> None:
    """Detector outputs should not drift from the workflow's actual job gates."""
    workflow = read_workflow("release.yml")
    block = workflow_job_block(workflow, "detect-changes")

    assert "any-code-changed:" in block
    for unused_output in (
        "py-changed",
        "tests-changed",
        "package-changed",
        "docs-changed",
        "workflow-changed",
    ):
        assert f"{unused_output}:" not in block
        assert f"outputs.{unused_output}" not in workflow


def test_release_workflow_checks_fresh_merge_and_secrets() -> None:
    """Pull requests must test a fresh base merge and scan for secrets."""
    workflow = read_workflow("release.yml")
    lint = workflow_job_block(workflow, "lint")

    assert "fetch-depth: 0" in lint
    assert "- name: Simulate fresh merge with base branch (PR only)" in lint
    assert "if: github.event_name == 'pull_request'" in lint
    assert "BASE_REF: ${{ github.base_ref }}" in lint
    assert "run: bash scripts/simulate-fresh-merge.sh" in lint
    assert "- name: Check for secrets" in lint
    assert (
        "npx --yes -p secretlint -p "
        '@secretlint/secretlint-rule-preset-recommend secretlint "**/*"' in lint
    )
    secretlint_config = (ROOT / ".secretlintrc.json").read_text(encoding="utf-8")
    assert '"id": "@secretlint/secretlint-rule-preset-recommend"' in secretlint_config


def test_release_workflow_builds_docker_images_on_pull_requests() -> None:
    """Docker regressions must fail before packages are published."""
    workflow = read_workflow("release.yml")
    block = workflow_job_block(workflow, "docker-build")

    assert "name: Docker Image Build Check" in block
    assert "needs: [detect-changes]" in block
    assert (
        "if: github.event_name == 'pull_request' && "
        "needs.detect-changes.outputs.any-code-changed == 'true'" in block
    )
    assert "uses: docker/setup-buildx-action@v4" in block
    assert "uses: docker/build-push-action@v7" in block
    assert "push: false" in block
    assert "load: true" in block
    assert "cache-from: type=gha" in block
    assert "cache-to: type=gha,mode=max" in block


def test_manual_release_requires_required_checks_to_succeed() -> None:
    """Manual release must only run after lint, test, and build succeed."""
    workflow = read_workflow("release.yml")
    condition = job_condition(workflow, "manual-release")

    assert "needs.lint.result == 'success'" in condition
    assert "needs.test.result == 'success'" in condition
    assert "needs.build.result == 'success'" in condition
    assert "github.event_name == 'workflow_dispatch'" in condition


def test_release_jobs_smoke_test_published_package_before_github_release() -> None:
    """Published packages must be installed and exercised before announcing release."""
    workflow = read_workflow("release.yml")

    expected_version_outputs = {
        "auto-release": "steps.version_check.outputs.current_version",
        "manual-release": "steps.version.outputs.new_version",
    }

    for job_name, version_output in expected_version_outputs.items():
        block = workflow_job_block(workflow, job_name)
        assert "- name: Smoke test published package" in block
        assert "python scripts/smoke_test_published_package.py" in block
        assert f'--version "${{{{ {version_output} }}}}"' in block

        publish_index = block.index("- name: Publish to PyPI")
        smoke_index = block.index("- name: Smoke test published package")
        release_index = block.index("- name: Create GitHub Release")
        assert publish_index < smoke_index < release_index


def test_docs_workflow_action_versions_are_current() -> None:
    """Docs workflow actions should stay aligned with the current Pages stack."""
    docs_workflow = read_workflow("docs.yml")

    assert_action_pin_count(docs_workflow, "actions/checkout", "v6", 1)
    assert_action_pin_count(docs_workflow, "actions/setup-python", "v6", 1)
    assert_action_pin_count(docs_workflow, "actions/upload-artifact", "v7", 1)
    assert_action_pin_count(docs_workflow, "actions/configure-pages", "v6", 1)
    assert_action_pin_count(docs_workflow, "actions/upload-pages-artifact", "v5", 1)
    assert_action_pin_count(docs_workflow, "actions/deploy-pages", "v5", 1)

    assert_action_pin_absent(docs_workflow, "actions/checkout", "v4")
    assert_action_pin_absent(docs_workflow, "actions/setup-python", "v5")
    assert_action_pin_absent(docs_workflow, "actions/upload-artifact", "v4")
    assert_action_pin_absent(docs_workflow, "actions/configure-pages", "v5")
    assert_action_pin_absent(docs_workflow, "actions/upload-pages-artifact", "v3")
    assert_action_pin_absent(docs_workflow, "actions/deploy-pages", "v4")


def test_docs_workflow_deploys_pages_only_when_opted_in() -> None:
    """Fresh repositories should build docs without failing Pages deployment."""
    workflow = read_workflow("docs.yml")
    build_job = workflow_job_block(workflow, "build")
    deploy_job = workflow_job_block(workflow, "deploy")
    configure_step = workflow_step_block(build_job, "Configure GitHub Pages")
    upload_step = workflow_step_block(build_job, "Upload GitHub Pages artifact")
    skip_step = workflow_step_block(build_job, "Report skipped GitHub Pages deployment")

    deploy_condition = (
        "github.event_name == 'push' && "
        "github.ref == 'refs/heads/main' && "
        "vars.DEPLOY_GITHUB_PAGES == 'true'"
    )
    skip_condition = (
        "github.event_name == 'push' && "
        "github.ref == 'refs/heads/main' && "
        "vars.DEPLOY_GITHUB_PAGES != 'true'"
    )

    assert f"if: {deploy_condition}" in configure_step
    assert f"if: {deploy_condition}" in upload_step
    assert f"if: {deploy_condition}" in deploy_job
    assert f"if: {skip_condition}" in skip_step
    assert "::notice::" in skip_step
    assert "DEPLOY_GITHUB_PAGES=true" in skip_step
    assert "Settings -> Pages" in skip_step
