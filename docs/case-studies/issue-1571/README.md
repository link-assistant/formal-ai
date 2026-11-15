# Issue #1571: false positives, false negatives, warnings and errors in CI/CD

Issue: <https://github.com/link-assistant/formal-ai/issues/1571>

Pull request: <https://github.com/link-assistant/formal-ai/pull/1572>

Guideline followed: [hive-mind `docs/CI-CD-BEST-PRACTICES.md`](https://github.com/link-assistant/hive-mind/blob/main/docs/CI-CD-BEST-PRACTICES.md)

## 1. Requirements

| # | Requirement (issue text and comment) | Where it is met |
| --- | --- | --- |
| R1 | Find every false positive, false negative, warning and error in CI/CD | §3, §4 and §5. The inventory covers every failed workflow on `main` and every annotation on `main` at `7837d5073`. |
| R2 | Fix them all | §4 lists one commit per root cause. Each has a regression test, except where the fix *is* a check. |
| R3 | Compare with the eight `link-foundation/*-ai-driven-development-pipeline-template` repositories and report shared problems | §6, and link-foundation/rust-ai-driven-development-pipeline-template#250 |
| R4 | Follow the hive-mind CI/CD best practices | §7 |
| R5 | Keep a deep case study: timeline, requirements, root causes, solutions, existing libraries | This document |
| R6 | Report problems in other projects, with reproductions, workarounds and fixes | §8 |
| R7 | The issue comment lists red runs on `main` at `7837d50`: Hive Mind draft, Layered CI and CI/CD Pipeline | §4.3, §4.4 and §4.8. All three causes are fixed. |

## 2. Data

| Path | Content |
| --- | --- |
| `raw-data/timeline-cicd-pipeline.md` | Every *CI/CD Pipeline* run on `main` from the last green run (`aa7a8005e`, run 42062109916, 2026-11-05) to `7837d5073` (run 43730270231), with the failed jobs of each |
| `raw-data/timeline-layered-ci.md` | The same for *Layered CI* |
| `raw-data/main-7837d50-annotation-classes.txt` | The 150 check-run annotations of `main` at `7837d5073`, grouped into classes, with digits replaced by `N` |
| `../../../experiments/issue-1571/` | Reproduction scripts: `lock-engines.mjs` lists lock entries whose `engines.node` floor is above a runtime, `osv-cargo-lock.sh` runs an OSV batch scan over a `Cargo.lock`, and `a6-driver.mjs` checks the JS planner's plan for issue 698 |

The full job logs stay in the git-ignored `dev/log/issues/1571/pulls/1572/`.

## 3. Timeline

| When (UTC) | Event |
| --- | --- |
| 2026-11-05 | Last green *CI/CD Pipeline* on `main` (`aa7a8005e`, run 42062109916) |
| 2026-11-13 | *Dependencies Latest* fails because rustc 1.100 cannot build wasm. #1569 fixed this. |
| 2026-11-14 20:42 | `58a229dda` (#1559) makes *CI/CD Pipeline* red: clippy `manual_contains`, a missing module-map row, and an inline test module plus a top-level `examples/` file. Main stays red through `7837d5073`. |
| 2026-11-15 05:23 | The daily *Dependencies Latest* run on `686367d20` fails seven Rust-stage gates. Six fail on one E0308 (`user_intent.rs:141`, f64 passed as f32) and one on the tests-as-docs check. `92ca9ea76` and `10edee05b` had already fixed both on `main`. The cause is the systemic false negative in §5.1. |
| 2026-11-15 | Layered CI js-tier suite group 7 is killed at its 360 s budget (run 43730269695). The Hive Mind draft (run 43730396084) says "docker run exited 5 without a session log error". The learning cycle exits 101 with no message (run 43706083454). |
| 2026-11-15 15:30 | `7837d5073`: every job carries the GitHub notice "The ubuntu-latest label will migrate to Ubuntu 26" (95 notices). Issue #1571 is opened. |
| 2026-11-15 | Fixes land on this branch (§4). Upstream reports are filed (§8). |

## 4. Root causes and fixes

### 4.1 Main's CI/CD Pipeline: lint, module map and layout (`dbea32ce9`, `978803615`)

#1559 merged four problems that fail on `main` only:

- two `clippy::manual_contains` lints;
- no `standing_scope` row in `docs/architecture/module-map.md`;
- an inline `#[cfg(test)]` module in `work_item_leads`, which fails `src_has_no_inline_unit_tests`;
- a top-level `examples/` script, which fails `the_crate_lives_under_rust`.

**Fix:**

- clear the lints;
- regenerate the map;
- move the tests to `rust/tests/unit/agentic-coding/`;
- move the example under `rust/examples/`.

The existing gates are the regression tests. They pass in CI on this branch.

### 4.2 Rust tests that disagreed with JS (`67558abc1`)

- **A5:** the Rust `gh` work-item read used `normalized_payload`. That function drops a body quoting `exit 5` or `HTTP 403` (issue 1488) as if the command had failed. JS (`js/agentic/progress.mjs`) uses `observed_payload`, and Rust now does too.
- **A6:** since #1560, a fetched issue body no longer refuses reads, so the issue 698 driver plans `read_file`. The Rust assertion still expected the old plan. The JS planner plans the same three steps (`experiments/issue-1571/a6-driver.mjs`), so the assertion was stale.

### 4.3 Layered CI js-tier group overrun (`f83f2eee1`)

Run 43730269695 recorded durations for only 305 of the 853 suite files. The planner gave unrecorded files 0.1 s, so several 70–150 s files landed in group 7, which then hit its 360 s budget.

There were two causes:

1. `data/meta/javascript-test-durations.lino` was never refreshed from CI.
2. Newer `gh` refuses ANSI-coloured job logs unless `--allow-escape-sequences` is passed. Without it, `ci-durations.mjs --tests` and `--playwright-files` measured nothing and said nothing (a false negative of the measuring tool).

**Fix:** `ci-durations.mjs --javascript-ci-logs` folds the `[test-file] duration_ms` lines of a run into the table. Every log read now passes `--allow-escape-sequences`, and skipped jobs are reported.

### 4.4 Hive Mind draft explains a red run with the wrong log (`1711797eb`)

The shared runner explains a failed draft with the newest `*.log` in the session directory. `capture-draft-command.mjs` rewrote `attempt.log` and `attempt.{stdout,stderr}.log` on every chunk, so the capture itself was always the newest file. Run 43730396084 reported "docker run exited 5 without a session log error" instead of "Finished without changes".

**Fix:** the capture is now named `attempt.capture.jsonl` plus `attempt.{stdout,stderr}.txt`. A regression test replays the runner's newest-`*.log` selection. The runner-side contract is reported as hive-mind#4065 (§8).

### 4.5 Learning cycle exits 101 with no message (`a1309e282`)

The workflow wrote stderr to `run.log`, and the `cat` that would print it never ran under `set -e`. `Swatinem/rust-cache` looked for a root `Cargo.toml`, `cargo metadata` failed, and a cache of about 0 MB was saved.

**Fix:** set `workspaces: rust`, and print stderr in a trap that runs on every exit.

### 4.6 Build metadata written into `Cargo.toml` (`bcf1792c5`)

crates.io reports `toml_edit` as `0.25.19+spec-1.1.0`, and `--apply` wrote that string verbatim. As a result, every cargo invocation of the daily run warned that the metadata "will be ignored" (SemVer 2.0.0 §10).

**Fix:** strip `+…` before writing a Cargo or npm requirement, with a parser test.

### 4.7 Issue 1028 ladder budget (`0e5a9f5cd`)

The weekly step-budget audit failed. A two-leaf ladder leg used 1196 s of its 1200 s budget because the first leaf compiled the crate cold while sccache rejected 706 of 877 writes (run 42472083551).

**Fix:** run one leg per depth-5 leaf (32 legs). Each leg keeps the 1200 s budget, which is 67% of the 30-minute job cap.

### 4.8 Floating runner labels (`4e0805a73`)

Every job on `main` at `7837d50` carried the notice that `ubuntu-latest` moves to Ubuntu 26 from 2026-10-19 (actions/runner-images#14748), 95 in all. A floating label lets the image change without a commit, so a toolchain difference shows up as an unexplained red run.

**Fix:** pin `ubuntu-24.04` and `windows-2025`. The release matrix keeps `os: ubuntu-latest` only as the leg's *name*, because job names, duration data and the `unit-tests-<os>` checks depend on it, and maps it to the pinned image in `runs-on`.

Three things followed from the pin:

- The canonical release original `original-release-workflow.yml` was exactly at its 1500-line cap. A three-line comment pushed it to 1503 lines, and the fast lint on `ec8562ef5` failed (run 43741781733). The comment is now a trailing comment on the `runs-on` line. The sha in `stage-source-coverage.json` is refreshed, and the projections are regenerated with `generate-staged-release.mjs --write --deployed`. `--check-deployed` passes.
- Three tests pin reviewed digests or structures of jobs whose only change is the runner: `ci-fast-critical-profile`, `verify-public-release-downloads` and `browser-engine-release-integration`. They keep their reviewed baselines. Each now asserts that the job runs on `ubuntu-24.04`, then checks the baseline with only that field restored. Restoring it reproduces each baseline hash exactly, so the pin is the only difference.
- The pull-request CI did not run these JavaScript tests (§5.1). They were run locally, one file at a time.

### 4.9 `@link-assistant/web-capture` 2.x on Node 24 runtimes (`ec8562ef5`)

The daily dependency run moved web-capture to 2.x in the desktop and VS Code manifests. web-capture 2.0.0–2.0.2 declares `engines.node >=26.10.0`, set in its commit `b055a3058` "refresh locked tooling". The runtimes that load it are older:

- Electron 44 embeds Node 24: 44.0.0 has 24.18.1, the locked 44.9.3 has 24.22.0, and the newest 44.11.3 has 24.22.2 (releases.electronjs.org);
- VS Code runs extensions on Node 24;
- the desktop build uses Node 22.

npm printed `EBADENGINE` twice. That is only a warning, so the run would have shipped an incompatible pin (a false positive).

**Fix:**

- Both manifests carry the repository's blocked-pin convention (`"@link-assistant/web-capture//": "blocked: …/web-capture/issues/191 (…)"`), which `scripts/check-dependencies-latest-parsers.rs` already honours.
- `rust/tests/web/issue-1571-workflow-diagnostics.test.mjs` parses every runtime entry of both lockfiles and fails when an `engines.node` floor is above Node 24.21.0, which is what Electron 44.7.0 embeds (the `^44.7.0` floor in `desktop/package.json`).
- I verified that the test fails when the lock's web-capture engine is set to `>=26.10.0`.

### 4.10 Cargo audit read one lock of two (`be46692e3`)

`scripts/check-rust-dependencies.sh` audited only `rust/Cargo.lock`. The committed issue #1017 reproducer `experiments/issue-1017-meta-language-quadratic/Cargo.lock` had:

- rustls 0.23.43 (Dependabot alerts 289–292);
- the yanked chacha20 0.10.1;
- advisories in quinn-proto, quinn-udp, rustix and webpki-root-certs.

The Security workflow stayed green (a false negative), and its sparse checkout (`/Cargo.lock`) did not even fetch the file.

**Fix:**

- The script discovers locks with `git ls-files -- 'Cargo.lock' '*/Cargo.lock'`.
- It fails when a committed lock was not checked out.
- The shipped lock keeps `--deny warnings`. Unshipped locks fail on `vulnerabilities`, `unsound` and `yanked`.
- An `unmaintained` notice with no fixed release (`unsafe-libyaml`, pulled in by the pinned meta-language 0.58.1) is printed but does not fail. Ignoring it would need a blocked-upstream proof, and a reproducer pinned to a historical version is not shipped.
- The reproducer's crates are updated (`cargo update -p`).
- The sparse-checkout pattern is now `Cargo.lock`, which matches at any depth. `fast-pull-request-production-profile.test.mjs` asserts with real `git sparse-checkout check-rules` that every committed `Cargo.lock` is selected.

## 5. Systemic findings that need no new code

### 5.1 Rust regressions surface only on `main`

Pull requests do not compile Rust: the repository builds and tests Rust only in CI, and its fast pull-request profile skips the Rust stage. So the E0308 in §3 merged green and failed first on `main` and in the daily dependency run.

`92ca9ea76` and `10edee05b` had already fixed both failures before this branch started. This is the reason the case study keeps §4.1 and §4.2 as separate commits.

Changing the pull-request profile is a cost decision (CPU minutes on every pull request), so it is recorded here and not changed.

### 5.2 Annotations that are correct as they are

`raw-data/main-7837d50-annotation-classes.txt`:

| Annotation | Verdict |
| --- | --- |
| sccache `N% - N hits, N misses, N errors` (26) | Informational notice |
| Closure-driven cache bucket exempt from the hard cap (3) | Informational, by design |
| Shard 3/8 at 70% of its 1080 s budget | A true early warning. Shards are balanced from the durations that §4.3 now refreshes. |
| `attempt-identity.json` absent in the shared Hive Mind checkout | True. It is tracked upstream as hive-mind#3909. |
| `delivery-*` exceeded N in N of N runs | Hive phase thresholds report real slow phases |
| VS Code extension "consists of N files … you should bundle" | True. Bundling is a separate feature, not a CI defect. |
| crates.io publish token `UNKNOWN` | By design: crates.io offers no read endpoint that accepts a token, so preflight cannot confirm it and says so |
| Job headroom rows named `Agent CLI ladder ${{ matrix.leg.id }}` (scheduled run, 2026-11-09) | By design: the report keys jobs by their workflow `name:` template, so all matrix legs share one row ("worst of 31 runs"). Its 77.2%-of-30-minutes warning and 1196 s of 1200 s overrun were real; §4.7 fixes them. |
| File-size "approaching limit" warnings (in job logs: 89 Rust files over 900 of 1000 lines; 13 maintained, 12 Links Notation, 2 workflow and 2 worker files over 1400 of 1500) | True early warnings below the hard limit. Splitting those files is refactoring outside this issue. |

### 5.3 HumanEval regression

The HumanEval regression found while reading the layered runs is a solver-quality problem, not a CI defect. It is filed as #1574 and kept under `experiments/`.

## 6. Template comparison

| Template | Floating `*-latest` runners | Cargo/npm lock audit coverage |
| --- | --- | --- |
| js, python, php, csharp, java, cpp | none | Each template commits one lockfile per ecosystem |
| go | none. The single match is the guard in `workflows.yml:44` that forbids them. | n/a |
| rust (HEAD `309387f761f9`) | none | `security.yml` runs `cargo audit --file Cargo.lock --deny warnings` but also commits `examples/javascript-first/Cargo.lock`. The gap is latent: that lock has one package, and an OSV scan finds nothing. Reported as link-foundation/rust-ai-driven-development-pipeline-template#250. |

The templates already pin runner images, which is the practice §4.8 adopts. I found none of the other root causes in §4 in the templates, because those causes live in formal-ai-specific scripts.

## 7. Best-practices mapping

| Practice (hive-mind `CI-CD-BEST-PRACTICES.md`) | Applied in |
| --- | --- |
| Pin tool and runner versions so a change has a commit | §4.8 |
| Every check must fail loudly, never silently succeed | §4.3 (`gh` log reads), §4.5 (stderr on exit), §4.10 (lock not checked out) |
| Timeouts below the job cap, budgets from observed durations | §4.3, §4.7 |
| Security audit over every shipped and committed artifact | §4.10 |
| A regression test for every CI fix | `rust/tests/web/issue-1571-workflow-diagnostics.test.mjs` and the tests named in §4 |
| Debug output off by default | The new diagnostics are failure-only: the stderr trap and the skipped-job report. Nothing new prints on a green run. |

## 8. Reports to other projects

| Report | Problem | Workaround here |
| --- | --- | --- |
| link-assistant/web-capture#191 | 2.x declares `engines.node >=26.10.0`, so it cannot load in Electron 44 or VS Code (both Node 24) | Hold 1.x with a blocked note (§4.9) |
| link-assistant/hive-mind#4065 | The shared runner picks the newest `*.log` as the session log, so a capture written in the same directory hides the real reason | Do not write `*.log` captures (§4.4) |
| link-foundation/rust-ai-driven-development-pipeline-template#250 | The Cargo audit reads only the root `Cargo.lock` | None needed. The issue proposes the `git ls-files` loop from §4.10. |
| #1574 (this repository) | HumanEval regression | Tracked separately |

## 9. Existing tools used

- **cargo-audit 0.22**: repeated `--deny` values (`unsound`, `yanked`, `warnings`) give per-lock severity without an ignore list.
- **OSV `querybatch` API**: an independent check of a `Cargo.lock` where cargo-audit is not installed (`experiments/issue-1571/osv-cargo-lock.sh`).
- **`git sparse-checkout check-rules`**: tests sparse patterns against the real file list instead of a copy of git's matching rules.
- **`gh api --allow-escape-sequences`**: required by newer `gh` to read coloured job logs.
