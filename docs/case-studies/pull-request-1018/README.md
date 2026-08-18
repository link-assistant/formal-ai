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
  Because it mitigates someone else's defect, it was measured rather than
  assumed: `Security` run 31967180539 took `macro expansion failed` from 20,725
  to zero, and the 358 smaller diagnostics it uncovered are recorded rather than
  silenced.
- **Per-job concurrency groups that never cancel `main`.** Workflow-level
  `cancel-in-progress` is forbidden here: these workflows contain write jobs,
  and cancelling `main` would restore the exact blind spot
  `scripts/check-pipeline-status.sh` exists to close.
- **Fix the timeout the deadline fix exposed, rather than raising the harness's
  limit.** With the deadline structural, two macOS slices failed cleanly instead
  of degrading into `cancelled` — and what they exposed was a runtime defect,
  not a CI one: the *first* request to a fresh server built the canonical
  learning ledger before checking whether the ledger could answer at all, and
  building it round-trips a 39 KB module through a parser that is quadratic in
  input size. Raising `RESPONSE_TIMEOUT` would have hidden a ten-second
  first-response for every user. The lookup now proves a miss before it builds
  anything, which is a property of the lookup rather than a tuned constant.
- **Report the algorithm upstream and work around it here.** The quadratic scan
  lives in `meta-language`; nothing in this repository can fix it, only avoid
  calling it on a request path. Both are done —
  [`meta-language#193`](https://github.com/link-foundation/meta-language/issues/193)
  carries a standalone reproducer and a patch, and this branch stops paying for
  the parse.
- **Grant one constructed variable rather than widen a timeout.** The next run
  reduced to one red job, and it was a `python3` agent command reporting
  `timed_out: true` after 16.2 s while its slice used 83 s of a 600 s budget. The
  sandbox clears the environment on purpose, and on macOS that removes `TMPDIR`,
  where `/usr/bin/python3`'s `xcrun` stub keeps its resolution cache — so every
  invocation paid a full re-resolution. Simply raising the floor would have left
  the seconds-per-call in place and the next loaded runner to find it. The child
  now receives exactly one *constructed* entry, `TMPDIR`, and still inherits
  nothing from the host; the floor becomes a documented backstop rather than the
  fix.
- **Assert the relation, not the literal.** The old contract test froze `15`, so
  the only available repair was to freeze a bigger number — the constant *was*
  the contract. It now asserts that the floor stays at least four times clear of
  the slowest start-up measured here, that a small budget is raised and a
  generous one never lowered, and that the spawned environment has exactly one
  entry. A number that is never explained is a number nobody can be wrong about.
- **Document the single-architecture container deviation instead of rewriting
  the publish path.** The Rust template's multi-arch `docker-publish` /
  `docker-merge-manifest` pair cannot be exercised outside a real release, so a
  blind port would be verified for the first time by a production release. The
  deviation, and a concrete follow-up plan, are stated explicitly in the
  evidence README rather than left silent.

## Upstream reports

Six exact report bodies are retained verbatim under
`dev/log/issues/1017/pulls/1018/upstream-reports/`, each recording the URL it
was filed under:

- the missing step-execution budget in each of the three `link-foundation`
  pipeline templates — each with a one-minute reproduction, the real-world
  instance with its 1.3-second margin, a workaround, and a code-level fix
  including why `timeout(1)`, `bun test --timeout` and `pytest-timeout` are not
  substitutes — filed as
  [rust#135](https://github.com/link-foundation/rust-ai-driven-development-pipeline-template/issues/135),
  [js#137](https://github.com/link-foundation/js-ai-driven-development-pipeline-template/issues/137)
  and
  [python#60](https://github.com/link-foundation/python-ai-driven-development-pipeline-template/issues/60);
- a repository-scale data point for the open `github/codeql#19982`
  ([comment 5309221141](https://github.com/github/codeql/issues/19982#issuecomment-5309221141)),
  and a follow-up carrying the measured before/after counts once the workaround
  had actually been run here
  ([comment 5309264165](https://github.com/github/codeql/issues/19982#issuecomment-5309264165)),
  with the
  20,725-diagnostic breakdown across 1,023 files, the per-macro histogram
  showing every failing macro is external, the extractor configuration dump,
  the confirmed workaround on CLI 2.26.3, and three suggestions — chiefly that
  the extractor should emit one diagnostic naming the `std` version it cannot
  parse instead of one warning per call site;
- the quadratic `point_at_byte` scan in `meta-language`'s tree-sitter adapter —
  filed as
  [meta-language#193](https://github.com/link-foundation/meta-language/issues/193)
  with a standalone reproducer crate whose only dependency is `meta-language`,
  release- and dev-profile scaling tables showing the per-byte cost doubling
  every time the input doubles, `gdb` attribution putting 12 of 12 samples in
  that one function, a line-start-table patch that turns each lookup into
  `O(log lines)`, and the consumer workaround this pull request applies.

## Verification

`cargo fmt --check`, `cargo clippy --lib --bins --tests --all-features`,
`cargo check --examples --all-features`, `rust-script scripts/check-file-size.rs`,
`rust-script scripts/check-hardcoded-language.rs`,
`bash scripts/lint-shell-scripts.sh`, `actionlint`,
`rust-script scripts/run-ci-gates.rs --stage rust`, and `cargo test --tests
--all-features` — the whole suite (2,825 unit, 345 integration, 488 source, the
new `issue_1017_ledger_recall` binary and every other per-issue harness, 0
failures), not only the `ci_cd::` module that holds the thirteen tests in
`tests/unit/ci-cd/issue_1017.rs`.

That last distinction was learned here rather than assumed. The first push
verified only `cargo test --test unit ci_cd::`, and `Coverage` then failed on
`tests/issue_961_macos_portability.rs`, which froze the macOS matrix at twelve
slices and the archive cap at 25 minutes. Raising the slice count is exactly
the kind of change that contract was written to catch, so the contract was
repaired rather than relaxed: it now reads the denominator out of the
`--partition slice:N/M` argument and asserts the matrix lists 1..=N, and it
asserts the archive cap is *at least* the 25 minutes issue #961 measured as
necessary instead of exactly 25. Both forms are strictly stronger than the
literals they replace — a matrix that disagrees with its denominator silently
drops those tests while CI stays green, which is the same class of false
negative this pull request exists to remove.

The runtime half taught the same lesson twice more, locally this time. Editing
four `src/` files made their committed self-AST census documents stale, which
only the full `cargo test` surfaces (`issue_673_self_ast_census`), and the new
regression test asserted on an engine answer without showing it, which only the
gate stage surfaces (`check_tests_as_docs`, R234-2). Both are repaired in the
branch: the census was regenerated by its own example, and the test now pins the
exact offline answer, so it documents what the solver says as well as what it
must not do first.

And a third time on the sandbox fix, which is why the chain is run whole every
time rather than trimmed to what seems relevant: `Duration::from_secs(60)` trips
`clippy::duration_suboptimal_units` where this repository already writes
`Duration::from_mins(1)` (`src/local_transport.rs:38`), and a module doc that
linked `[`command_environment`]` — a private function — failed both
`check_rust_api_documentation` and `check_docs_rs_dependency_profile` under
`-D rustdoc::private-intra-doc-links`. Two gates, one cause, neither visible to a
targeted test run. Editing `src/agent.rs` also made its census document stale
again (`479 documents (1 rewritten, 0 removed)`).
