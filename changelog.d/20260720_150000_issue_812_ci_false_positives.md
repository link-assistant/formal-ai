---
bump: patch
---

### Fixed
- Stopped the self-hosting ratchet from deadlocking every release: the metric no longer counts captured CI transcripts (`*.log`, `*.jsonl`, `*.diff`, `*.patch`, `*.stderr`, `*.stdout`) or dependency lockfiles, which made up 94.20% of the measured range and turned "commit your evidence" into a guaranteed regression, and enforcement moved from `record_release` on `main` — where every contributing commit is already immutable — to a differential pull-request gate where the commits can still be amended (issue #812).
- `cargo clippy` now runs with `-D warnings`. Every lint in `[lints.clippy]` is set to `warn`, so the job printed findings and still exited 0 (issue #812).
- `auto-release` and `manual-release` now gate on `Secrets Scan` and both E2E suites, not just `[lint, test, build]`; a red secrets scan on `main` could previously publish the crate, the Docker image and the GitHub Release anyway (issue #812).
- The desktop `finalize` job runs under `!cancelled()` instead of `always()`, so a cancelled run can no longer clobber a complete `SHA256SUMS.txt` with a partial one via `gh release upload --clobber` (issue #812).
- The desktop packaging and VS Code jobs now simulate the fresh merge on pull requests, the same way `lint` and `test` already did, so packaging is validated against the merge result rather than a stale merge preview (issue #812).
- `scripts/check-file-size.rs` excluded `.github/workflows/**` by accident: its `.git` exclusion was matched as a substring. Directory exclusions are now matched per path component, GitHub Actions workflows are measured (warn at 1500 lines, fail at 2000), and quoted third-party workflows under `docs/case-studies/**` stay exempt (issue #812).
- `node --test $(ls … | grep -v …)` in the desktop library test step fell back to Node's own test discovery if the glob ever stopped matching — a green step running none of the intended tests. The file list is now built explicitly and an empty list fails (issue #812).
- Pinned `secretlint` to an explicit version in `scripts/check-secrets.sh`; `npx --yes -p secretlint` resolved to whatever `latest` was when the job ran (issue #812).
- Quoted `$BASE_REF` at every expansion in `scripts/simulate-fresh-merge.sh`, which upstream leaves bare on two of three lines (issue #812).

### Added
- `actionlint` (with `shellcheck`) now lints every workflow definition, and `shellcheck` lints the shipped `*.sh` scripts. Nothing validated either before, so a mistyped `needs.<job>` reference or a malformed expression — which fails *open*, silently skipping the guarded step under a green check — could only be found by pushing (issue #812).
- `METRIC_VERSION` on self-hosting ledger rows, so a change to how the share is measured starts a new comparison epoch instead of silently invalidating recorded history; rows from different epochs are never compared (issue #812).
