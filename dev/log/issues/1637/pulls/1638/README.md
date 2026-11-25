# Issue #1637: CI/CD false positives, false negatives, warnings and errors

Evidence and analysis for PR #1638. Collected on 2026-11-25.

## Layout

| Path | Contents |
|---|---|
| `issue-1637.json`, `pull-1638.json` | Issue and PR snapshots |
| `runs/` | `gh run view --json` metadata for every failing run analysed |
| `ci-logs/` | Failed-job logs. Logs over 5 MB stay local and are not committed |
| `analysis/agent-A-parity-corpus.md` | Server parity (R1013) and benchmark corpus gate |
| `analysis/agent-B-other-workflows.md` | Six failing workflows outside the main CI pipeline |
| `analysis/template-comparison.md` | Comparison with the eight link-foundation pipeline templates |
| `analysis/clusters/`, `failed-rust-tests.txt`, `panics.txt` | Failing Rust tests on main, grouped by cause |
| `analysis/rust-fixes-c2.md` | Fix notes for test cluster 2 |
| `upstream-reports/` | Upstream issues filed, each with a `.url` file |
| `local-clippy-oom.log` | Why this PR does not run cargo locally (3 GB cgroup) |

## Timeline

| When | Event |
|---|---|
| 2026-09-28 | External Benchmarks code-generation scores fall below threshold (humaneval 12/20, mbpp 10/20, gsm8k 1/20). Still open |
| 2026-10-31 | The dependency registry snapshot stops being refreshed, so `check-dependencies-latest` starts failing |
| PR #1585 (30ff2d8a8) | Server parity (R1013) between the Rust and JS servers breaks on main |
| PR #1632 (faa55b564) | `formal-ai with claude` starts passing `--permission-mode auto`. The release client-report fixture still expects `default` |
| 6a49eff07..1c109025c | Benchmark corpus shards begin to be killed 6–7 minutes in (runs 45490076540, 45549993299) |
| Run 45275490842 | Last green Pages deploy (`actions/deploy-pages@v5`, version = commit SHA) |
| Runs 45490077033, 45549993600 | Pages deploy returns HTTP 404 for a synthetic 64-hex `pages_build_version` |
| 2026-11-25 | Issue #1637 opened. This PR |

## Requirements and status

| # | Requirement | Status | Evidence |
|---|---|---|---|
| R1 | Find every false positive, false negative, warning and error in CI/CD | Done | `analysis/*`, the tables below |
| R2 | Fix them all, in this PR, everywhere each fix applies | Done except the blocked items below | Commits on the branch |
| R3 | Download logs into this folder | Done | `ci-logs/`, `runs/` |
| R4 | Deep analysis: timeline, requirements, root causes, solutions, existing components | Done | This file and `analysis/` |
| R5 | Add debug or verbose output, off by default, where the root cause is unknown | Done for the corpus gate | `FORMAL_AI_CORPUS_TRACE` |
| R6 | Report upstream issues with repro steps, workarounds and fixes | Done | See "Upstream reports" |

## Root causes and fixes

| Signal | Kind | Root cause | Fix |
|---|---|---|---|
| `docker-build` and `docker-slim` never ran | False negative (dead jobs) | The `if:` both refused and required `pull_request` | c1e7b9ef9. Jobs run on `main` after `build-artifacts`. A contract test rejects self-contradictory guards |
| `dependencies-latest` red | True positive | `@v6` setup-node/setup-python pins, stale CLI versions, an incomplete registry snapshot | 05f67352f |
| Semantic grounding graph not closed | True positive | Missing Wikidata cache records, two dangling relations | 0f0266dc1 |
| Rust 1.100 clippy errors, rust-script unused dependency warnings | Error and warning | New lints on the floating stable toolchain | 836fe921e |
| Ordered-list check false positive | False positive | Cargo `[[test]]` and `[[bin]]` targets were not treated as module roots | 2ba2ae143 |
| A native capture case bound to two callers | Error | One file compiled into two Cargo targets | dec7e0d17 (`@target` qualifier) |
| Pages deploy 404 | Error | `deploy.mjs` sent a synthetic sha256 as `pages_build_version`. The API needs the commit SHA, which is what `actions/deploy-pages@v5` sends. A retry could also adopt a finished deployment of another artifact | 14054acac |
| verify-package client report failure | False negative (stale fixture) | The fixture expected `default` permission mode after #1632. The next client then found ports 19409 and 19410 still bound | 8404d0aa8 |
| Main-only regressions with no PR replay | Process gap | Run 45549993600 regressions were not in the ledger | 1ce9dfd31 |
| Corpus shards killed | Unknown root cause (see below) | Not reproduced locally | 9b68396a3 adds tracing |
| Server parity (R1013) | True positive | Key order in `output_projection` (BTreeMap in Rust, insertion order in JS), JS `report.violations`, missing `source_origin`, method registry hash | ba658ec66 |
| Failing Rust tests on main (clusters c1–c4) | True positives and stale expectations | Listed per cluster in `analysis/clusters/` | c0813fb8a, 2df34f2ae (solver plus JS/TS twins), 15ae736ab (tests) |
| CI/CD contract tests failing on main | Stale contracts | Workflow, dependency-check, ledger and traceability docs drifted from their tests | 4e48b8b54 |
| Procedural specification tests not replayable in JS | False negative | An offline `UniversalSolver` constructor that the JS reader could not parse, so the replay skipped the case | c6000fee9 (offline binding in the reader and worker host) |
| `issue_1138` family behaviour and `issue_712` Hindi routing | Flaky (network-dependent) | Read access is on by default since #1582, so live fetches changed answers or used up the 30 s budget | 15ae736ab (these cases run with the network denied) |
| `issue_1157` idempotent commit | Unknown root cause, fails on main only | Passes in isolation. Main reported only the unchanged `[WIP]` title | 4cc63096a (the assertion now prints the answer and commands; diagnosis continues on the next main failure) |
| PR round 2 (run on 394a1bda7): secrets scan red | True positive introduced by this PR | 05f67352f bumped `.github/secretlint` to 14.1.0, but `check-secrets.sh`, `check-secrets-files.mjs` and their tests pinned 14.0.0 | 1a67d3cf3 (one pin; the lock test reads the manifest) |
| PR round 2: tests-as-docs, notation, abbreviation, R999, translation and inventory gates | True positives on this PR's own changes | New names and worker lines from the regression fixes; twins not regenerated | 6398f7b31, b1d6b6a2f, 6fb036e94, 13614f864 (logs in `ci-logs/round2/`) |
| PR round 3 (run on 0775b5e2e): minimal-core boundary and governed witness | True positives on this PR's own changes | Four ledgered handlers grew with the regression fixes; the hello-world test change moved a governed native source hash | a910ebf3e (comments compressed, ledger re-measured with a note), 008402605 (reviewed witness refresh, cases unchanged). Logs in `ci-logs/round3/` |
| PR round 4 (run on 4bcdb5a35): `docs/status.md` stale | True positive on this PR's own change | The status page counts ledger lines, and the core-boundary note added one | Regenerated with `render-status.mjs --write`. Logs in `ci-logs/round4/` |
| PR round 5 (run on 2271bfebf): native type check, `try_arithmetic` never used | True positive on this PR's own change | 2df34f2ae sent both arithmetic routes through `arithmetic_with_online`, so the public wrapper lost its last caller. The check had not run in earlier rounds because fast-pr-lint failed first | The wrapper is removed and the ledger re-measured. `experiments/issue-1637-unused-imports.py` scanned the changed files for other leftovers (none new). Logs in `ci-logs/round5/` |
| Hive Mind draft | Upstream drift | hive-mind 45c85a4 reshaped `setEnv` | Already fixed on main by 1b538b39d |
| Issue 1028 ladder budget | True positive | Three serial single-test `cargo test` calls on a cold cache | Already fixed on main by 971802a07 |
| Work-item qualification `docker pull` timeout | Harness bug | 180 s timeout for an 8.1 GB image | Already fixed on main by 066cffea2 |

### Corpus gate: what is known

- The PR run of the corpus gate passes trivially, because the heavy shards are main-only (#1191 policy).
- On main, shards were killed by the runner shutdown signal 6–7 minutes in.
- A local replay of shard 1 kept RSS at 120–185 MB, so memory alone does not explain the kills.
- The slowdown appeared in 6a49eff07..1c109025c (PRs #1626, #1614, #1623, #1625, #1622, #1627, #1633, #1581, #1635).
- With the network denied (`FORMAL_AI_OFFLINE=1`), the same shard finished in 424 s (`ci-logs/local-corpus-shard1-offline.log`), against more than 970 s online. Live source fetches are therefore most of the shard's runtime. Offline mode also routes 12 of the 234 prompts to the unknown opener, though, so switching the gate offline would change what it measures. This PR does not do that.
- `FORMAL_AI_CORPUS_TRACE=1` (off by default) logs each prompt and its duration. The shard job enables it and logs `free` and `df` every 30 s, so the next killed shard names the prompt it was answering and the runner's resources.

## Blocked or not fixed in this PR

| Item | Why |
|---|---|
| Job Headroom (c): the `main` ruleset requires no status checks | Needs a repository admin to edit the ruleset |
| External Benchmarks code-generation regression | A long-standing capability gap (since 2026-09-28), not a CI defect. Raising the scores is product work |
| sccache write refusals | Cache-backend infrastructure noise. Builds still succeed |
| Durable SIGTERM forwarding in `formal-ai with` | Follow-up. Needs Rust changes that cannot be compiled locally (3 GB OOM). Tracked here, not done |

## Trade-offs chosen

- `dtolnay/rust-toolchain@stable` stays floating. Pinning it would hide new lints until someone bumps the pin. Instead, new lint errors are fixed as they appear.
- PR clippy does not add `--tests`. Doing so would double PR build time. Test-target lints run on main.

## Upstream reports

- https://github.com/link-foundation/rust-ai-driven-development-pipeline-template/issues/270
- https://github.com/link-foundation/rust-ai-driven-development-pipeline-template/issues/271
- https://github.com/link-foundation/js-ai-driven-development-pipeline-template/issues/317

## Existing components used

- `actions/deploy-pages@v5` `api-client.js`: the reference payload for the Pages fix.
- `fuser` and `lsof`: free the client-report ports.
- Node's built-in `node:test` and the `yaml` package: workflow contract tests.
