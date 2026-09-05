---
bump: patch
---

### Added
- Issue #1076: a scheduled headroom audit (`.github/workflows/job-headroom.yml`, `scripts/check-job-headroom.rs`, `scripts/collect-job-durations.sh`) that reads real job durations from the Actions API and fails when a job spends more than 85% of its declared `timeout-minutes` — the repository previously enforced only that a *declared* budget stays under 70% of its cap, never that the *measured* runtime does.
- A workflow security audit: `zizmor` now runs over `.github/workflows` and `.github/actions` with `.github/zizmor.yml`, matching all four `link-foundation/*-ai-driven-development-pipeline-template` repositories, which the previous `actionlint`-only lint did not cover.
- `FORMAL_AI_CI_VERBOSE` runner telemetry (`scripts/report-runner-capacity.sh`) on the coverage job, default off, so the 7.4x runtime variance on identical tests can be attributed on the next occurrence rather than guessed at.
- A Links Notation parse failure now names the line that caused it. `links-notation` reports the unconsumed remainder of the file and no position, so one stray `:` in a `#` prose paragraph of `data/meta/ci-gates/check-job-headroom.lino` failed the whole test suite with a wall of quoted text and no line number; `tests/unit/lino_location.rs` locates it and holds the gate registry against a repeat. Reported upstream as link-foundation/links-notation#301 (the notation has no comment syntax, so `#` prose is structural) and #302 (the Rust errors carry no line or column, while the JavaScript port of the same version reports both).
- `.github/actions/cache-cargo-registry` gained a `restore-only:` input and a step-summary line per invocation, so a cache miss is visible in the run summary instead of only in a folded log group.

### Fixed
- The `Coverage / Code Coverage` job was killed by its `timeout-minutes` and reported `cancelled` rather than `failure`, so an overrun was invisible to branch protection. The `cargo llvm-cov` run now carries a `TEST_BUDGET_SECONDS` deadline through `scripts/run-with-budget-warning.sh`, which fails the step before the cap cancels the job.
- `actionlint` ran as a bare pinned binary. It delegates every `run:` block to ShellCheck and, when ShellCheck is absent, skips those checks and exits 0 — a green check that had verified nothing. It now runs as `docker://rhysd/actionlint:1.7.12`, which bundles ShellCheck, and a second step aims that same image at `tests/fixtures/actionlint/shellcheck-canary.yml` — a fixture whose only defect is inside a `run:` block — and fails if the fixture *passes*, so the gate cannot silently stop being one again.
- Four workflow `name:` scalars were unquoted and contained ` #`, which YAML reads as a comment: `Task Ladder (issue #840 dataset)` was stored as `Task Ladder (issue`. Valid YAML, so no linter reported it.
- Job caps measured against 400 `main` runs: `lint` (12.7 min against 15), `build` (11.6 against 15) and both release jobs (50.6 against 60) had turned their backstop into their deadline, and the 45-minute publish budget inside a 60-minute release job could never have fired. Caps raised to 25, 20 and 90.
- Docker layer caches were saved with `mode=min`, which stores nothing for a multi-stage compiled build, and were unscoped; they are now `mode=max` with `scope=docker-image`.
- The browser coverage baseline was ~12 points stale (functions 45.54% committed against 57.23% measured), so a real regression to ~46% would have passed the ratchet.
- The remaining five inline cargo-registry cache blocks now route through the shared composite action, so one registry no longer occupies six key prefixes in a shared quota.
