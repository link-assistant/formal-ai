# Pull request #1018 — making the deadline structural for #1017

Issue: <https://github.com/link-assistant/formal-ai/issues/1017>

Pull request: <https://github.com/link-assistant/formal-ai/pull/1018>

## Initial state and discussion audit

The prepared draft contained only its bootstrap commit and a placeholder
description. Issue #1017, PR #1018 and all three PR discussion surfaces
(conversation comments, inline review comments, and reviews) were downloaded
before any code was changed; there were no comments or reviewer decisions to
reconcile. The complete immutable inputs are indexed in
`dev/log/issues/1017/pulls/1018/README.md`.

Collecting that evidence immediately exposed its own defect: `.gitignore`
negated `!dev/log/**/ci-logs/*.log`, which reaches only files directly inside
`ci-logs/`. This archive groups logs one level deeper, per head SHA, so `git
add` reported success while committing nothing. Found with `git check-ignore
-v`. Every analysis in this pull request would otherwise have cited files that
were never in the repository.

## Decisions made in this pull request

- **Move the deadline into the step instead of raising the cap.** Raising
  `timeout-minutes` buys time without changing the failure *mode* — the next
  overrun still reports `cancelled`. It also has no ceiling: the overrun that
  produced this issue was itself preceded by a timeout increase.
- **Make the budget/cap relationship a checked invariant rather than a per-job
  accident.** `MAX_BUDGET_SHARE_PERCENT = 70` is what found the *next* two
  instances, neither of which was involved in the incident.
- **Sixteen slices rather than a hand-maintained list of slow tests.** A curated
  list drifts silently; the next slow test lands wherever the round-robin puts
  it.
- **Keep the security false positive visible rather than silenced.** The
  RUSTSEC-2026-0235 ignore carries a proof line that CI re-derives from the
  dependency graph on every run, so it fails the moment the crate becomes
  reachable. A bare ignore would become a permanent blind spot.
- **Treat the CodeQL macro failures as an analysis-coverage loss, not noise.**
  Excluding the affected paths, or accepting 20,725 warnings, would both have
  hidden the fact that 1,023 files of live code were being extracted with
  errors. The extractor's sysroot is pinned instead, and losing that pin warns
  loudly rather than failing the scan — it mitigates someone else's defect.
- **Per-job concurrency groups that never cancel `main`.** Workflow-level
  `cancel-in-progress` is forbidden here: these workflows contain write jobs,
  and cancelling `main` would restore the exact blind spot
  `scripts/check-pipeline-status.sh` exists to close.
- **Document the single-architecture container deviation instead of rewriting
  the publish path.** The Rust template's multi-arch `docker-publish` /
  `docker-merge-manifest` pair cannot be exercised outside a real release, so a
  blind port would be verified for the first time by a production release. The
  deviation, and a concrete follow-up plan, are stated explicitly in the
  evidence README rather than left silent.

## Upstream reports

Four exact report bodies are retained verbatim under
`dev/log/issues/1017/pulls/1018/upstream-reports/`:

- the missing step-execution budget in each of the three `link-foundation`
  pipeline templates — each with a one-minute reproduction, the real-world
  instance with its 1.3-second margin, a workaround, and a code-level fix
  including why `timeout(1)`, `bun test --timeout` and `pytest-timeout` are not
  substitutes;
- a repository-scale data point for the open `github/codeql#19982`, with the
  20,725-diagnostic breakdown across 1,023 files, the per-macro histogram
  showing every failing macro is external, the extractor configuration dump,
  the confirmed workaround on CLI 2.26.3, and three suggestions — chiefly that
  the extractor should emit one diagnostic naming the `std` version it cannot
  parse instead of one warning per call site.

## Verification

`cargo fmt --check`, `cargo clippy --lib --bins --tests --all-features`,
`cargo check --examples --all-features`, `rust-script scripts/check-file-size.rs`,
`rust-script scripts/check-hardcoded-language.rs`,
`bash scripts/lint-shell-scripts.sh`, and the full `ci_cd::` unit suite
including the thirteen tests in `tests/unit/ci-cd/issue_1017.rs`.
