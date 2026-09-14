# Rust AI-Driven Development Pipeline Template — Analysis

Source: `link-foundation/rust-ai-driven-development-pipeline-template` (branch `main`).
Raw copies of the workflow, scripts, experiments, examples and key config files are
saved under `docs/case-studies/issue-1/data/templates/rust/`.

This document describes how the template's CI/CD and "AI-driven development"
conventions work, so an equivalent **PHP/Symfony + Composer** template can be built
from the same principles.

---

## 1. High-Level Architecture

- **Single workflow** `.github/workflows/release.yml` (display name "CI/CD Pipeline")
  does everything: CI checks on PRs, and release/publish on merge to `main` and on
  manual dispatch.
- **All automation logic lives in standalone scripts** under `scripts/`, written as
  `rust-script` files (shebang `#!/usr/bin/env rust-script`, with an inline
  `//! ```cargo` dependency block). The YAML stays thin — it installs `rust-script`
  and calls these scripts. This is the central "AI-driven" idea: logic is in a real,
  testable language, not in brittle shell embedded in YAML.
- **crates.io is the source of truth for "what is already released"** — NOT git tags.
  This avoids false "already released" results when a tag exists but publishing failed.
- **Fragment-based changelog** (`changelog.d/*.md`): each PR adds a fragment with a
  `bump:` level; on release, fragments are collected into `CHANGELOG.md` and removed.
- **Template-safe defaults**: the default package name `example-sum-package-name` is a
  sentinel — publish/release steps detect it and skip real publishing so the template
  itself never tries to push to crates.io.

For a PHP/Symfony equivalent the mapping is:
crates.io → Packagist; `Cargo.toml` → `composer.json`; `cargo publish` →
Packagist webhook / git tag; `rust-script` → a small PHP CLI (or plain PHP scripts run
via `php scripts/*.php`); clippy/rustfmt → PHPStan/Psalm + PHP-CS-Fixer/php-cs.

---

## 2. Release Pipeline

### 2.1 Triggers (`on:`)

```yaml
on:
  push:        { branches: [main] }
  pull_request:
    types: [opened, synchronize, reopened]
  workflow_dispatch:
    inputs:
      release_mode: { type: choice, options: [instant, changelog-pr], default: instant }
      bump_type:    { type: choice, options: [patch, minor, major] }
      description:   # optional free text
```

Three release entry points:
1. **Auto-release** — every push/merge to `main` runs the full pipeline and, if a
   release is needed, bumps + publishes automatically (`auto-release` job).
2. **Instant manual release** — `workflow_dispatch` with `release_mode=instant`
   (`manual-release` job) bumps + publishes immediately using the chosen `bump_type`.
3. **Changelog-PR** — `workflow_dispatch` with `release_mode=changelog-pr`
   (`changelog-pr` job) opens a PR (via `peter-evans/create-pull-request@v8`) that
   contains the version bump + collected changelog, for human review before merge.

### 2.2 Top-level `env`

- `CARGO_TERM_COLOR: always`
- `RUSTFLAGS: -Dwarnings` (warnings fail the build)
- `CARGO_REGISTRY_TOKEN: ${{ secrets.CARGO_REGISTRY_TOKEN || secrets.CARGO_TOKEN }}`
  (accepts either secret name)
- `DOCKERHUB_IMAGE: ${{ vars.DOCKERHUB_IMAGE }}` (a *variable*, not a secret — controls
  whether optional Docker publishing happens)

### 2.3 Jobs (CI side, run on PRs)

- **detect-changes** — runs `detect-code-changes.rs`; sets `any-code-changed`. Skipped
  on `workflow_dispatch`. Used to gate changelog enforcement and release.
- **changelog** ("Changelog Fragment Check") — PR-only; if code changed, requires a new
  fragment in `changelog.d/` (`check-changelog-fragment.rs`). Docs-only / changelog-only
  PRs are exempt.
- **version-check** — PR-only; `check-version-modification.rs` blocks manual edits to the
  version in `Cargo.toml` (versioning is automated; humans must not hand-edit it).
- **lint** — `cargo fmt --check`, `cargo clippy` (pedantic + nursery, warnings = errors),
  and `check-file-size.rs` (1000-line cap). Uses `if: always() && !cancelled()` so it runs
  independently of the changelog gate.
- **test** — matrix over ubuntu/macos/windows + doc tests.
- **coverage** — `cargo-llvm-cov` → Codecov.
- **build** — `needs: [lint, test]`; `cargo build --release`, `cargo package --list`,
  `check-crate-size.rs` (10 MiB crates.io limit, warns at 80%).

### 2.4 Jobs (release side)

`auto-release` (push to main) and `manual-release` (dispatch/instant) share the same
ordered sequence of script calls:

1. `git-config.rs` — set committer to `github-actions[bot]`.
2. `get-bump-type.rs` — determine bump level. From changelog fragments' `bump:`
   frontmatter (highest wins: major > minor > patch, default patch). On manual release
   the dispatched `bump_type` overrides.
3. `check-release-needed.rs` — **decision step**. Checks crates.io (is this version
   already published?), the Docker tag, and the GitHub release. Outputs `should_release`
   and `skip_bump`. Key behavior: if the version is published but the GitHub release or
   Docker image is missing, it **re-creates the missing artifact WITHOUT re-bumping**
   (idempotent recovery).
4. `version-and-commit.rs` — bump `Cargo.toml`, ensure the new version strictly exceeds
   the max published crates.io version (`ensure_version_exceeds_published`), update
   `Cargo.lock` for the named package only, collect changelog (stripping fragment
   frontmatter, inserting at the `<!-- changelog-insert-here -->` marker, deleting
   fragments), commit `chore: release v{ver}`, create annotated tag `v{ver}`, push with
   retry/rebase on contention.
5. `cargo build --release` then `check-crate-size.rs`.
6. `publish-crate.rs` — `cargo publish --allow-dirty -p <name>`. Skips for the template
   sentinel name. Classifies failures (`FailureKind`: AlreadyExists / AuthFailed /
   RateLimited / Unknown). A 429 rate-limit is treated as "deferred" → exit 0 (not a
   hard failure; the next run completes it).
7. `wait-for-crate.rs` — poll crates.io up to 30×10s until the new version is visible.
8. **(optional) Docker Hub** — only if `DOCKERHUB_IMAGE` var is set AND a root
   `Dockerfile` exists AND `DOCKERHUB_USERNAME`/`DOCKERHUB_TOKEN` secrets are present.
9. `create-github-release.rs` — build a GitHub Release from the matching `CHANGELOG.md`
   section; release name `[Rust] {semver}`, tag `v{semver}`; embeds crates.io / docs.rs /
   Docker badges; caps body at 125 000 chars; idempotent if the release already exists.

`deploy-docs` job — `cargo doc` → GitHub Pages (one-time repo setting: Settings → Pages →
source = GitHub Actions).

### 2.5 Why crates.io (not tags) is the source of truth

Documented in `docs/ci-cd/troubleshooting.md`: a prior bug ("Version Already Released —
False Positive") came from checking git tags. A tag could exist while the publish failed,
permanently blocking re-release. The fix: query the registry API. This is the single most
important transferable design decision — the **registry/Packagist is authoritative, not
local tags**.

---

## 3. Versioning & Changelog Fragments

- `Cargo.toml` holds the version; it is **only** changed by automation
  (`check-version-modification.rs` enforces this on PRs).
- Each PR that changes code must add a fragment `changelog.d/<name>.md`:

  ```markdown
  ---
  bump: patch        # patch | minor | major
  ---
  ### Fixed
  - Description of the change.
  ```

  Categories: Added / Changed / Fixed / Removed / Deprecated / Security (Keep a Changelog).
- `get-bump-type.rs` scans all fragments; the **highest** bump wins; default `patch`.
- `collect-changelog.rs` / `version-and-commit.rs` merge fragments into `CHANGELOG.md` at
  the `<!-- changelog-insert-here -->` marker, strip frontmatter, then delete the fragments.
- `create-changelog-fragment.rs` maps bump → default category (major → Breaking Changes,
  minor → Added, patch → Fixed) for convenience.

PHP/Symfony equivalent: keep the exact same `changelog.d/*.md` fragment convention; store
the version in `composer.json`; replace registry checks with Packagist's API.

---

## 4. Linting, Formatting & pre-commit

- **rustfmt**: `cargo fmt --all --check` in CI; `cargo fmt --all --` as a pre-commit hook.
- **Clippy**: `cargo clippy --all-targets --all-features -- -D warnings`; lint levels
  configured in `Cargo.toml`:
  - `[lints.rust] unsafe_code = "forbid"`
  - `[lints.clippy] all/pedantic/nursery = "warn"`, with a small allow-list
    (`module_name_repetitions`, `too_many_lines`, `missing_errors_doc`,
    `missing_panics_doc`).
- **File-size guard**: `check-file-size.rs` — hard cap 1000 lines per `.rs` file, warn at
  900. Enforces small, AI-reviewable files.
- **`.pre-commit-config.yaml`**:
  - `pre-commit-hooks` v5.0.0: trailing-whitespace, end-of-file-fixer, check-yaml,
    check-added-large-files, check-merge-conflict, check-toml, debug-statements.
  - local hooks: `cargo fmt`, `cargo clippy -D warnings`, `cargo test`.

PHP/Symfony equivalent: PHP-CS-Fixer or `php-cs` (rustfmt analogue), PHPStan and/or Psalm
at max level with warnings-as-errors (clippy analogue), the same generic pre-commit hooks,
plus a file-size guard, and `phpunit` in the pre-commit test hook.

---

## 5. `docs/ci-cd/troubleshooting.md` (summary)

Seven sections:
1. **Release Jobs Skipped** — usually no code changes detected, or changelog/version gate.
2. **Version Already Released (False Positive)** — root cause of checking tags instead of
   the registry; fixed by querying crates.io.
3. **Crates.io Publishing Fails** — auth/token issues, rate limits (429 deferred), name
   already taken.
4. **Crate Package Too Large** — crates.io 10 MiB (10 485 760 bytes) limit, HTTP 413;
   fixed via the narrow `include = [...]` in `Cargo.toml` and `check-crate-size.rs`.
5. **Docker Hub Publishing Fails** — missing `DOCKERHUB_IMAGE` var / secrets / Dockerfile.
6. **Secret Configuration** — table of `CARGO_REGISTRY_TOKEN` / `CARGO_TOKEN` /
   `DOCKERHUB_TOKEN` / `GITHUB_TOKEN`.
7. **Multi-Language Repository Issues** + general debugging tips.

Note: a few examples in this doc still reference legacy `.mjs`/node scripts (pre-migration
to rust-script); the active pipeline is all rust-script.

---

## 6. `experiments/` test scripts (what they verify)

These are exploratory/regression harnesses for the trickiest pipeline logic:

- `test-changelog-parsing.rs` — 5 tests of changelog section extraction (pulling the
  correct version section out of `CHANGELOG.md`).
- `test-crates-io-check.rs` — 4 tests that hit the **real** crates.io API to validate the
  "is this version published?" logic.
- `test-detect-code-changes.sh` — builds a synthetic merge commit and verifies the
  per-commit diff range (`HEAD^2^..HEAD^2`) correctly classifies code vs non-code changes.
- `test-version-check.sh` and `test-version-check-dependencies.sh` — version-check
  scenarios (these still call legacy node `.mjs` scripts).

Several core scripts also carry inline `#[cfg(test)]` unit tests
(`version-and-commit.rs`, `publish-crate.rs`, `create-github-release.rs`,
`check-crate-size.rs`).

---

## 7. Overall "AI-Driven Development" Conventions

1. **Logic in a real language, not YAML.** Every non-trivial step is a standalone
   `rust-script` in `scripts/`, unit-testable and runnable locally. The workflow only
   orchestrates. (PHP analogue: small PHP CLI scripts under `scripts/`.)
2. **Registry is the source of truth**, not git tags — makes releases idempotent and
   self-healing (rate-limit deferral, missing-artifact re-creation without re-bump).
3. **Fragment-based changelog** decouples "what changed" (per PR) from "when released",
   and drives automated semver.
4. **Humans never touch the version**; CI enforces it.
5. **Small files** (1000-line cap) and **strict lints** (warnings = errors, pedantic
   clippy, `unsafe forbidden`) keep code AI-reviewable and safe.
6. **Template-safe sentinel defaults** (`example-sum-package-name`) so the template builds
   and runs CI green without ever publishing.
7. **Idempotent, recoverable release steps** — every step checks current state before
   acting and can re-run safely.
8. **Optional, gated integrations** (Docker Hub, Pages) controlled by repo variables /
   presence of files, so the template works with zero extra config and scales up when
   opted in.
9. **One-time setup is documented** (Pages source, secrets table) rather than assumed.
10. **Multi-language-repo aware** (`rust-paths.rs` auto-detects `./Cargo.toml` vs
    `./rust/Cargo.toml`, resolves workspace members, skips `publish = false`).
