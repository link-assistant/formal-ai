# Python AI-Driven Development Pipeline Template — Analysis

Source: `link-foundation/python-ai-driven-development-pipeline-template` (default branch `main`).

This document analyzes the Python/Hatchling+pip template so an equivalent PHP/Symfony+Composer template can be built. Raw copies of every workflow, script, config, doc, and test live alongside this file under `templates/python/` (paths preserved, e.g. `templates/python/.github/workflows/release.yml`, `templates/python/scripts/bump_version.py`).

The template explicitly models itself on the JavaScript template (`js-ai-driven-development-pipeline-template`) and mirrors patterns from the Rust template. Several scripts are described in their docstrings as "the Python equivalent of `*.mjs` from the JS template." This is useful: the conventions are intentionally language-agnostic and portable to PHP.

---

## 1. Repository Layout

```
.github/workflows/
  docs.yml          # Sphinx build + GitHub Pages deploy
  release.yml       # CI checks (detect/lint/test/build/changelog) + release (auto + manual)
changelog.d/        # Changelog fragments (like JS .changeset/)
  README.md                 # How to add fragments
  fragment_template.md.j2   # scriv new-fragment template (Jinja2)
  <timestamp>_<user>_<branch>.md  # one fragment per issue/PR
docs/               # Sphinx source (conf.py, index.md, api.md, requirements.txt, preview-regeneration.md)
examples/basic_usage.py
scripts/            # 9 Python automation scripts (see §4)
src/my_package/     # src-layout package (__init__.py + py.typed)
tests/              # pytest tests, incl. workflow-policy regression tests
pyproject.toml      # single source of truth: build, deps, ruff, mypy, pytest, coverage, scriv
.ruff.toml          # supplemental ruff config (isort known-first-party)
.pre-commit-config.yaml
CHANGELOG.md  CONTRIBUTING.md  README.md  LICENSE (Unlicense)
```

Key design choices: `src/` layout (prevents accidental imports), `py.typed` marker, `pyproject.toml` as single config source, hatchling build backend (PEP 517), Unlicense (public domain).

---

## 2. The Release Pipeline (end-to-end)

### 2.1 Versioning model

- **Single source of truth**: `version = "X.Y.Z"` in `pyproject.toml` `[project]`. Scripts read/write it with a regex (`^version\s*=\s*["']([^"']+)["']`), not a TOML parser.
- **SemVer**, bumped by `bump_type` ∈ {major, minor, patch} (`scripts/bump_version.py::bump_version`).
- Tags use prefix `v` by default (`v1.2.3`); `create_github_release.py` supports a configurable `--tag-prefix` and `--language` label.

### 2.2 Two release triggers

The whole CI/CD lives in **one workflow** (`release.yml`, named "CI/CD Pipeline") with two distinct release paths:

**A. Auto-release on push to `main`** (`auto-release` job)
- Runs after `lint`, `test`, `build` on `push` to `main`.
- Reads `version` from `pyproject.toml`; checks if tag `v<version>` already exists via `git rev-parse`.
- If the tag does **not** exist → it's a new version → publish. If it exists → skip (idempotent).
- Downloads the `dist` artifact built earlier, publishes to PyPI, then creates the GitHub release.
- This means: **a release happens whenever someone commits a version bump to `main` and the tag doesn't yet exist.** The version-bump commit itself is normally produced by the manual-release path or a contributor editing `pyproject.toml`.

**B. Manual release via `workflow_dispatch`** (`manual-release` job)
- Inputs: `bump_type` (choice: patch/minor/major, required) and `description` (optional string).
- Steps: configure git bot identity → `scriv collect --version <bump_type>` (only if fragments exist) → run `version_and_commit.py` (bumps version, updates CHANGELOG, commits + pushes to `main`) → build → `twine check` → publish to PyPI → `create_github_release.py`.
- This is the canonical "cut a release" button. It bumps version, collects changelog fragments, commits, and publishes in one run.

### 2.3 Changelog fragments (changelog.d) — the core AI-driven convention

Uses **[scriv](https://scriv.readthedocs.io/)** (`scriv[toml]`), explicitly framed as "the Python equivalent of Changesets."

- Each PR/issue adds **one markdown fragment** to `changelog.d/`, named `YYYYMMDD_HHMMSS_<user>_<branch>.md` (e.g. `20260609_000000_issue_18_manual_release_skip.md`). Naming encodes issue/PR provenance.
- A fragment contains one or more category headings — `### Added / Changed / Deprecated / Removed / Fixed / Security` — with bullet points.
- **Why**: avoids `CHANGELOG.md` merge conflicts when many PRs are open; each PR self-documents.
- scriv config lives in `pyproject.toml` `[tool.scriv]`:
  - `format = "md"`, `fragment_directory = "changelog.d"`, `output_file = "CHANGELOG.md"`.
  - `categories = [Removed, Added, Changed, Deprecated, Fixed, Security]`.
  - `entry_title_template = "## [{{ version }}] - {{ date.strftime('%Y-%m-%d') }}"`.
  - `insert_marker = "<!-- scriv-insert-here -->"` — `CHANGELOG.md` contains this marker; collected entries are inserted there.
  - `new_fragment_template = file:changelog.d/fragment_template.md.j2` — the Jinja2 template scriv uses for `scriv create`.
- **During release**, `scriv collect --version <bump_type>` merges all fragments into `CHANGELOG.md` under the new version heading and deletes the fragment files.

### 2.4 GitHub release creation

`scripts/create_github_release.py`:
- Extracts the changelog section for the target version from `CHANGELOG.md` (regex on `^## <version>`), reading until the next `## X.Y.Z` heading.
- Appends a PyPI shields.io badge if none present.
- **Caps release notes to 60,000 UTF-8 bytes** (GitHub limit safety, from issue #16): truncates on byte boundary without splitting a UTF-8 char and appends a notice linking to the full tagged `CHANGELOG.md` (`https://github.com/<repo>/blob/<tag>/CHANGELOG.md`).
- Creates the release via `gh release create <tag> --repo --title "[<language>] <version>" --notes ...`. Title format is `[Python] 1.2.3`; tag is `<prefix><version>`.
- Requires `GH_TOKEN`/`GITHUB_TOKEN` and the `gh` CLI.

### 2.5 PyPI publishing

- Uses **OIDC trusted publishing** via `pypa/gh-action-pypi-publish@release/v1` — **no API tokens** stored. The release jobs grant `permissions: id-token: write` (+ `contents: write`).
- Package is built with `python -m build` (hatchling backend) and validated with `twine check` before upload.
- `scripts/publish_to_pypi.py` exists as a local/standalone alternative (clean → build → `twine check` → `twine upload`, with `--dry-run`), but the workflows use the official PyPA action rather than this script.

### 2.6 Idempotency / re-run safety

`scripts/version_and_commit.py` is built to survive partial reruns:
- Fetches `origin/main`, compares local vs remote HEAD. If remote advanced and remote version differs → rebase and continue. If remote version already equals the bumped version → assume a prior run succeeded and emit `already_released=true` (skip re-bumping).
- Emits GitHub outputs `version_committed`, `already_released`, `new_version`, which gate the subsequent build/publish/release steps (`if: ... version_committed == 'true' || already_released == 'true'`).
- `auto-release` independently guards with the tag-exists check.

---

## 3. Workflows

### 3.1 `release.yml` ("CI/CD Pipeline")

- **Triggers**: `push` to `main`; `pull_request` (opened/synchronize/reopened); `workflow_dispatch` (with `bump_type`, `description` inputs).
- **Concurrency**: group `${{ github.workflow }}-${{ github.ref }}`, `cancel-in-progress: ${{ github.ref != 'refs/heads/main' }}` — i.e. **never cancel in-progress main runs** (protects releases), but cancel superseded PR runs.

Jobs (each has an explicit `timeout-minutes`):

| Job | Trigger condition | Timeout | Purpose |
|-----|-------------------|---------|---------|
| `detect-changes` | not `workflow_dispatch` | 5 | runs `detect_code_changes.py`; outputs `py/tests/package/docs/workflow/any-code-changed` |
| `lint` | push, dispatch, or any relevant change | 20 | `ruff check`, `ruff format --check`, `mypy src`, `check_file_size.py` |
| `test` | push, dispatch, or code/test/pkg/workflow change | 30 | `pytest --cov`, upload to Codecov |
| `build` | push/dispatch, or lint+test success/skipped | 20 | `python -m build`, `twine check`, upload `dist` artifact |
| `changelog` | PR **and** `any-code-changed == true` | 10 | warns (not fails) if source changed without a fragment |
| `auto-release` | `push` to `main` | 30 | publish if `v<version>` tag absent (perms: contents+id-token write) |
| `manual-release` | `workflow_dispatch` + lint/test/build all success | 30 | collect fragments, bump, commit, publish (perms: contents+id-token write) |

**Skippable-dependency gotcha (issue #18)**: `detect-changes` is skipped on `workflow_dispatch`. In GitHub Actions a skipped dependency normally *propagates* and skips dependents. So `lint`, `test`, and `manual-release` use `if: always() && !cancelled() && (...)` status-check functions to evaluate anyway, and `manual-release` additionally requires `needs.lint.result == 'success' && needs.test.result == 'success' && needs.build.result == 'success'`. Without `always()/!cancelled()`, manual releases would silently skip CI and the release itself while appearing green. `build` similarly accepts `result == 'success' || 'skipped'` so docs-only PRs (where lint/test are skipped) still build.

**Changelog gate is intentionally a warning** (`exit 0` with `::warning::`), not a hard failure — comment in the workflow notes "Change `exit 0` to `exit 1` to make it required." The job only runs for PRs touching code (`any-code-changed`), and checks `git diff` for changes under `src/|tests/|scripts/` with zero fragments present.

**Action pins** (asserted by tests): `actions/checkout@v6`, `actions/setup-python@v5`, `actions/upload-artifact@v7`, `actions/download-artifact@v7`, `codecov/codecov-action@v4`, `pypa/gh-action-pypi-publish@release/v1`. Python `3.13` is the CI/build version even though the package supports 3.9–3.13.

### 3.2 `docs.yml` ("Docs")

- **Triggers**: `push` to `main`, `pull_request`, `workflow_dispatch`, all path-filtered to `docs/**`, `src/**`, `pyproject.toml`, `.github/workflows/docs.yml`.
- **Permissions**: `contents: read`, `pages: write`, `id-token: write`.
- **Concurrency**: same pattern as release (don't cancel main).
- `build` job (timeout 10): install package + `docs/requirements.txt`, `sphinx-build -W --keep-going -b html docs _site` (warnings-as-errors), upload `_site` as artifact; on push-to-main additionally `configure-pages@v6` + `upload-pages-artifact@v5`.
- `deploy` job (timeout 10, needs build, only push-to-main): `actions/deploy-pages@v5` to the `github-pages` environment. **PRs build but never deploy** (catches doc regressions before merge).
- **One-time manual setup**: Settings → Pages → Source = "GitHub Actions" (documented in both the workflow header and README; the first deploy fails otherwise and it can't be set from a workflow).

---

## 4. Scripts (`scripts/`)

All scripts use `argparse`, return exit codes, locate project root via `Path(__file__).parent.parent`, and write GitHub Actions outputs to `$GITHUB_OUTPUT`.

| Script | Purpose | Key inputs | Key outputs / behavior |
|--------|---------|-----------|------------------------|
| `bump_version.py` | Bump `version` in `pyproject.toml` and prepend a CHANGELOG entry | positional `major/minor/patch`, `--description` | regex-edits pyproject; inserts `## <v> - <date>` block into CHANGELOG; prints next-step git commands |
| `version_and_commit.py` | CI wrapper: bump + commit + push to main, rerun-safe | `--bump-type`, `--description`; env `GITHUB_OUTPUT` | configures git bot, handles remote-advanced/rebase, calls `bump_version.py`, commits (msg = new version), `git push origin main`; sets `version_committed`/`already_released`/`new_version` |
| `create_github_release.py` | Create GH release from CHANGELOG section | `--version`, `--repository`, `--tag-prefix=v`, `--language=Python`, `--prerelease`; env `GH_TOKEN` | extracts changelog entry, adds PyPI badge, caps notes to 60 KB with truncation notice + tagged-CHANGELOG link, runs `gh release create` |
| `format_release_notes.py` | Post-process an existing release's body (badge + PR link) | `--release-id`, `--version`, `--repository`, `--commit-sha`, `--package-name`, `--dry-run` | gh-API GET body → add PyPI badge + `**Pull Request:** #N` (resolved from commit) → de-escape/clean whitespace → PATCH release. Idempotent (skips if badge present). Python port of `format-release-notes.mjs` |
| `publish_to_pypi.py` | Standalone build+publish (not used by workflows, which use the PyPA action) | `--dry-run` | clean dist/build/egg-info → `python -m build` → `twine check` → `twine upload` |
| `create_manual_changeset.py` | Create a changelog fragment (Python `npx changeset`) | positional `major/minor/patch`, `--description`, `--no-scriv` | uses `scriv create` if available else writes a manual fragment `<ts>_<user>_<branch>.md`; maps bump→category (major→Changed, minor→Added, patch→Fixed) |
| `validate_changeset.py` | Validate fragments have content + a valid category heading | none (scans `changelog.d/`) | exit 0/1; **currently warns-only when zero fragments** (comment shows how to make required); warns if >1 fragment; checks each fragment has a `### <Category>` and non-comment content |
| `detect_code_changes.py` | Classify changed files for conditional CI | env `GITHUB_EVENT_NAME`, `GITHUB_BASE_SHA`, `GITHUB_HEAD_SHA` | diffs (PR: base..head; push: HEAD^..HEAD; first commit: ls-tree); sets outputs `py-changed`, `tests-changed`, `package-changed`, `docs-changed`, `workflow-changed`, `any-code-changed`. **Excludes** `*.md`, `changelog.d/`, `docs/`, `experiments/`, `examples/` from "code changes"; `any-code-changed` matches `.py/.toml/.yml/.yaml` or `.github/workflows/` among non-excluded files |
| `check_file_size.py` | Enforce max **1000 lines** per `.py` file | none (scans cwd) | exit 1 listing violators; excludes venv/build/dist/egg-info/etc. |

---

## 5. Linting / Formatting / Type-checking / Pre-commit

- **Ruff** (lint + format, replaces flake8/black/isort): config in `pyproject.toml` `[tool.ruff]` — line length 88, target `py39`, a broad `select` rule set (E, W, F, I, N, UP, B, C4, DTZ, T10, EM, ISC, ICN, PIE, PT, Q, RSE, RET, SIM, TID, ARG, PTH, ERA, PL, PERF, RUF), ignoring E501/PLR0913/PLR2004; per-file ignores relax tests. `[tool.ruff.format]`: double quotes, space indent, LF line ending. `.ruff.toml` adds `[lint.isort] known-first-party = ["my_package"]`.
- **mypy** (`[tool.mypy]`): effectively strict — `disallow_untyped_defs`, `disallow_incomplete_defs`, `no_implicit_optional`, `warn_unused_ignores`, `strict_equality`, etc. CI runs `mypy src`.
- **pytest** (`[tool.pytest.ini_options]`): `addopts = "-ra -q --strict-markers"`, `testpaths=["tests"]`, `pythonpath=["src"]`. Coverage: branch coverage on `src`, with standard `exclude_lines`.
- **pre-commit** (`.pre-commit-config.yaml`): pre-commit-hooks v5 (trailing-whitespace, end-of-file-fixer, check-yaml/toml, check-added-large-files, check-merge-conflict, debug-statements); ruff-pre-commit v0.8.4 (`ruff --fix --exit-non-zero-on-fix`, `ruff-format`); mirrors-mypy v1.13.0 (`--strict --ignore-missing-imports`).
- CI lint job order: `ruff check .` → `ruff format --check .` → `mypy src` → `python scripts/check_file_size.py`.

---

## 6. Docs Publishing

- **Sphinx** with **Furo** theme, MyST (Markdown), autodoc + autosummary + napoleon (Google-style docstrings) + viewcode + intersphinx. Config: `docs/conf.py` (adds `src/` to `sys.path`, reads version via `importlib.metadata`).
- `docs/index.md` is the MyST landing page with a toctree → `api.md` (autosummary of `my_package`, recursive) and `preview-regeneration.md`.
- Build deps pinned in both `docs/requirements.txt` and `pyproject.toml` `[project.optional-dependencies] docs` (`sphinx>=7.4`, `furo>=2024.8.6`, `myst-parser>=3.0`).
- Published to **GitHub Pages** by `docs.yml` only on push to `main`; PRs build (warnings-as-errors) but don't deploy. Requires one-time Pages "GitHub Actions" source setting.

---

## 7. Tests & How Workflows Are Tested

`tests/` mixes package unit tests with **infrastructure/policy regression tests**:

- `test_my_package.py` — unit tests for the example `add`/`multiply`/`delay` API (incl. an `@pytest.mark.asyncio` test).
- `test_create_github_release.py` — imports the script via `importlib`, monkeypatches `run_command`, asserts: tag-prefix vs `[Python] <v>` title separation; oversized notes capped to `MAX_RELEASE_NOTES_BYTES` with truncation notice + tagged-CHANGELOG link; PyPI badge added when missing and not duplicated when present.
- `test_workflows.py` — **parses the YAML as text** and asserts workflow policy: main runs aren't cancelled (`cancel-in-progress: ${{ github.ref != 'refs/heads/main' }}`); every job has the expected explicit `timeout-minutes`; action versions are current (e.g. `checkout@v6` appears 7×, `upload-artifact@v7` 1×; docs uses pages v5/v6 stack and *not* the old v3/v4); dispatch-dependent jobs (`lint`, `test`, `manual-release`) contain a status-check function (`always()`/`!cancelled()`/...); `manual-release` requires lint+test+build success and `workflow_dispatch`. This is the mechanism that locks in the issue #18 fix and the action-pin policy.
- `test_preview_regeneration_docs.py` — asserts the issue #9 tracking doc keeps required content and stays linked from `docs/index.md` and `README.md`.

So workflows are "tested" via string/regex assertions over the YAML files plus unit tests of the release scripts — there's no full workflow execution harness.

---

## 8. AI-Driven Development Conventions (summary for porting)

1. **One changelog fragment per issue/PR** in `changelog.d/`, named with timestamp + user + branch/issue, categorized by Keep-a-Changelog headings; collected automatically at release. Prevents CHANGELOG merge conflicts. (PHP equivalent: a `changelog.d/` + a tool like `changelog` or a small custom collector; or `jwage/changelog-generator`.)
2. **Change detection drives conditional CI** (`detect_code_changes.py`): docs/markdown/examples/experiments are excluded so docs-only PRs skip lint/test/changelog requirements. Outputs feed `if:` conditions across jobs.
3. **File-size cap** (1000 lines/`.py`) enforced in CI to keep files AI-context-friendly (`check_file_size.py`). CONTRIBUTING also recommends functions < 50 lines.
4. **Changelog-fragment check** on code PRs (currently a soft warning; one-line change to make it blocking).
5. **Idempotent, rerun-safe release scripts** (rebase/already-released detection) so retried CI jobs don't double-release.
6. **OIDC trusted publishing** to the package registry (no stored tokens) + `gh` CLI for releases.
7. **Single combined CI/CD workflow** with strict concurrency that never cancels main, explicit per-job timeouts, and pinned action versions — all guarded by YAML-policy regression tests.
8. **Don't-cancel-main + status-check-function** discipline to avoid silently-skipped releases on `workflow_dispatch`.
9. **Docs build on every PR, deploy only on main**; warnings-as-errors.
10. **Intentional non-goals documented** (e.g. `docs/preview-regeneration.md` records why Playwright screenshot automation is deferred and the exact checklist to add it later) — explicit parity tracking against sibling templates.

### PHP/Symfony+Composer mapping cheatsheet

| Python piece | PHP/Symfony equivalent |
|---|---|
| `pyproject.toml` version + deps | `composer.json` `version`/`require`/`require-dev` |
| hatchling `python -m build` | n/a for libs; for apps build a phar or just tag; `composer validate` as the "twine check" analog |
| PyPI + OIDC trusted publishing | Packagist (auto-updates from GitHub tags/webhook); no artifact upload needed — push a git tag |
| ruff + ruff-format | PHP-CS-Fixer / PHP_CodeSniffer (+ Symfony rules) |
| mypy strict | PHPStan (max level) or Psalm |
| pytest + coverage | PHPUnit (+ `--coverage-clover`) |
| pre-commit | `captainhook` or GrumPHP, or pre-commit with PHP hooks |
| scriv fragments | `changelog.d/` + custom collector or `jwage/changelog-generator` |
| Sphinx + GitHub Pages | phpDocumentor / Doctum → GitHub Pages (same deploy gating) |
| `check_file_size.py` | small PHP/shell script counting lines |
| `detect_code_changes.py` | reuse near-verbatim (language-agnostic; swap `.py`→`.php`, `pyproject.toml`→`composer.json`) |
| `gh release create` | identical (`gh` CLI) |
