# Issue 121 Case Study: crates.io Release Failure

Issue: https://github.com/link-assistant/formal-ai/issues/121

Pull request: https://github.com/link-assistant/formal-ai/pull/122

Failing run: https://github.com/link-assistant/formal-ai/actions/runs/26076375801/job/76668537953

## Summary

The `Auto Release` job failed while publishing `formal-ai@0.61.0` to crates.io. The release script reached `cargo publish`, Cargo packaged the crate successfully, and crates.io rejected the upload with HTTP 413 because the generated `.crate` archive exceeded the registry upload limit.

The root cause was package selection, not compilation. `Cargo.toml` did not define a publish allowlist or exclude list, so Cargo's default packaging behavior included repository artifacts that are useful for development and review but not needed by crate consumers. The local package list before the fix contained 798 files, including 559 files under `docs`, 35 files under `tests`, 20 scripts, 18 experiments, all changelog fragments, and the 2.0 MiB generated `CHANGELOG.md`.

The fix makes the package contents explicit, adds a CI guard that checks the generated archive size before release publication, and documents the investigation evidence.

## Data Collected

- `ci-logs/run-26076375801.log`: full failed workflow log.
- `ci-logs/job-76668537953.log`: failed `Auto Release` job log.
- `github-data/issue-121.json`: issue metadata.
- `github-data/pr-122.json`: pull request metadata.
- `github-data/run-26076375801.json`: failed run metadata and job list.
- `github-data/recent-runs-issue-branch.json`: recent branch run list with timestamps and SHAs.
- `github-data/local-cargo-package-before.log`: local reproduction before the package allowlist.
- `github-data/local-cargo-package-list-before.txt`: file list before the package allowlist.
- `github-data/local-cargo-package-list-after.txt`: file list after the package allowlist.
- `github-data/local-crate-package-size-after.log`: final size guard output.
- `template-snapshots/*`: tree and diff snapshots for the requested JS, Rust, Python, and C# templates.

No screenshots or image attachments were present in the issue, PR, or CI logs.

## Failure Timeline

- `2026-05-19T04:37:30Z`: `Auto Release` started in run `26076375801`.
- `2026-05-19T04:40:07Z`: `Publish to Crates.io` started and detected `formal-ai@0.61.0`.
- `2026-05-19T04:40:32Z`: Cargo reported `Packaged 792 files, 25.0MiB (16.1MiB compressed)`.
- `2026-05-19T04:40:32Z`: crates.io rejected the publish: `status 413 Payload Too Large`, `max upload size is: 10485760`.

Key local log references:

- `ci-logs/job-76668537953.log:980`: package was `formal-ai@0.61.0`.
- `ci-logs/job-76668537953.log:991`: CI packaged 792 files, 16.1 MiB compressed.
- `ci-logs/run-26076375801.log:9180`: Cargo failed to publish to crates.io.
- `ci-logs/run-26076375801.log:9183`: crates.io returned the 10,485,760 byte limit.

## Local Reproduction

Before the fix:

- Command: `cargo package --allow-dirty --no-verify`
- Result: `Packaged 805 files, 25.1MiB (16.1MiB compressed)`.
- Generated archive: `target/package/formal-ai-0.61.0.crate`, 16,884,863 bytes.
- Package list categories: `docs` 559 files, `changelog.d` 81, `src` 43, `tests` 35, `scripts` 20, `data` 20, `experiments` 18.

After the fix:

- Command: `rust-script scripts/check-crate-package-size.rs`
- Result: `Packaged 69 files, 854.3KiB (208.6KiB compressed)`.
- Generated archive: 213,569 bytes.
- Package list categories: `src` 43 files, `data` 20, plus Cargo metadata, `Cargo.toml`, `Cargo.lock`, `LICENSE`, and `README.md`.

The post-fix package compiles during `cargo publish --dry-run --allow-dirty`, ending with Cargo's expected dry-run upload abort warning.

## Root Cause

Cargo includes all package-root files by default, except VCS-ignored files and a small set of always-excluded paths. The Cargo manifest supports `include` and `exclude` fields, and `cargo package --list` shows exactly which files will be published.

`formal-ai` had grown a large repository-local documentation and case-study corpus, but `Cargo.toml` did not constrain the crates.io package. The build job listed package contents, but listing alone did not fail the workflow when the generated archive crossed the crates.io upload limit. The failure was therefore delayed until the release job attempted the real `cargo publish`.

External references:

- Cargo manifest `include` and `exclude`: https://doc.rust-lang.org/cargo/reference/manifest.html#the-exclude-and-include-fields
- Cargo publishing and 10 MB `.crate` limit: https://doc.rust-lang.org/cargo/reference/publishing.html#packaging-a-crate

## Template Comparison

Requested templates:

- `link-foundation/js-ai-driven-development-pipeline-template`
- `link-foundation/rust-ai-driven-development-pipeline-template`
- `link-foundation/python-ai-driven-development-pipeline-template`
- `link-foundation/csharp-ai-driven-development-pipeline-template`

Findings:

- JS template: `package.json` uses npm's `files` allowlist for publish contents.
- Python template: `pyproject.toml` publishes only the configured package source tree.
- C# template: tests are marked non-packable and the NuGet project explicitly packs only project output plus README metadata.
- Rust template: current package is small (`132 files, 1.2MiB, 697.6KiB compressed`), but the template has the same missing guard: it runs `cargo package --list` and has no `.crate` size check before publish. It also lacks a package `include` allowlist.

The Rust template gap was reported upstream:

- https://github.com/link-foundation/rust-ai-driven-development-pipeline-template/issues/58

No upstream issue was opened for the JS, Python, or C# templates because their default packaging model is already narrow enough to avoid this exact failure mode.

## Fix

- Added a `Cargo.toml` package `include` allowlist for runtime/source package inputs:
  - `Cargo.lock`
  - `Cargo.toml`
  - `LICENSE`
  - `README.md`
  - `data/**`
  - `src/**`
- Added `scripts/check-crate-package-size.rs`.
  - Runs `cargo package --allow-dirty --no-verify`.
  - Inspects `target/package/<name>-<version>.crate`.
  - Warns above 8 MiB.
  - Fails above 10 MiB / 10,485,760 bytes.
- Added the size guard to the build job after `cargo package --list` and before release jobs can publish.
- Added unit/workflow tests that assert the package allowlist exists and the build job installs `rust-script` before running the size guard.
- Added a changelog fragment for the CI/CD fix.

## Verification

| Check | Result | Log |
| --- | --- | --- |
| `cargo fmt --check` | Pass | `github-data/local-cargo-fmt-after.log` |
| `git diff --check` | Pass | `github-data/local-git-diff-check-after.log` |
| `rust-script scripts/check-crate-package-size.rs` | Pass, archive is 213,569 bytes | `github-data/local-crate-package-size-after.log` |
| `cargo publish --dry-run --allow-dirty` | Pass, dry-run stops before upload | `github-data/local-cargo-publish-dry-run-after.log` |
| `cargo test --all-features --verbose` | Pass, 273 passed, 69 ignored | `github-data/local-cargo-test-after.log` |
| `cargo test --all-features workflow_release` | Pass, 14 workflow tests passed after final YAML cleanup | `github-data/local-cargo-workflow-release-targeted-after.log` |
| `cargo clippy --all-targets --all-features` | Pass | `github-data/local-cargo-clippy-after.log` |
| `cargo build --release --verbose` | Pass | `github-data/local-cargo-build-release-after.log` |
| `rust-script scripts/check-file-size.rs` | Pass with existing near-limit warnings | `github-data/local-check-file-size-after.log` |
| `npm ci --prefix tests/e2e` | Pass, 0 vulnerabilities | `github-data/local-npm-ci-after.log` |
| `npm run --prefix tests/e2e check:i18n` | Pass, 4 locales and 104 keys | `github-data/local-npm-check-i18n-after.log` |

## Residual Notes

`cargo package` now warns that examples and integration/unit test targets are not included in the published package. This is expected for the narrowed crates.io archive and does not block `cargo publish --dry-run`; examples and tests remain in the repository and CI, while the published crate contains the source and runtime data needed by consumers.
