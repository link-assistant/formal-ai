# Issue #1639: CI/CD false positives, false negatives, warnings and errors

Evidence and analysis for PR #1640. Collected on 2026-11-25. This round follows
issue #1637 / PR #1638 (`dev/log/issues/1637/pulls/1638/`), which merged green as
513e4d954 and then failed on `main`.

## Layout

| Path | Contents |
|---|---|
| `issue-1639.json`, `pull-1640.json` | Issue and PR snapshots |
| `runs/` | `gh run view --json` metadata for the seven failing runs named in the issue |
| `ci-logs/` | Failed-job logs of those runs. Logs over 5 MB stay local and are not committed |
| `ci-logs/cicd-jobs/`, `ci-logs/corpus-jobs/` | Per-job logs of CI/CD Pipeline 45636781871 and Benchmark Corpus Gate 45636781444 |
| `ci-logs/annotations/` | Raw check-run annotations (`<run>-<job>.json`) |
| `ci-logs/round1/` | Logs of this PR's own first CI round (Layered CI, Issue 1028 ladder) |
| `analysis/template-comparison.md` | Comparison with the eight link-foundation pipeline templates and hive-mind `CI-CD-BEST-PRACTICES.md` |
| `analysis/e2e-native-package-pages.md` | E2E generalization, native observations, package verify (Aider), Pages deploy |
| `analysis/ladder-lone-artifact.md` | Issue 1028 ladder joined a passing leg as "0 wrote a result" |
| `analysis/external-benchmarks-regression.md` | External Benchmarks ratchet failure, bisected to PR #1139 |
| `analysis/test-failures-overview.txt`, `panics.txt`, `test-shard-durations.tsv` | Failing Rust tests on main and shard timings |
| `analysis/githubstatus-incident-qn8mfv6q5hlx.json` | GitHub incident covering the Pages HTTP 500s |
| `upstream-reports/` | Upstream issues (filed ones have a `.url` file) and an unfiled draft |

## Timeline

| When | Event |
|---|---|
| 2026-09-19 | PR #1139 deletes seven algorithm-shaped coding templates; the authors record the HumanEval/MBPP fall in plan 02 L15 and leave it open |
| 2026-09-28 onward | External Benchmarks ratchet fails every week (run 45197910485 is the issue's sample) |
| 2026-11-04 | Work-item qualification dispatch 41768761745 fails |
| 2026-11-21 | Merge 8711d61f2 joins the "Report issue" menu onto one line; Aider's scripted turns stop matching |
| 2026-11-22 | Issue 1028 ladder schedule 45018538750 fails |
| 2026-11-23 | Job Headroom 45204085037 fails: the `main` ruleset requires no status checks |
| 2026-11-25 14:41Z | PR #1638 merges as 513e4d954. CI/CD Pipeline 45636781871 fails 23 jobs (rustfmt, clippy, full test shards, E2E, native observations, package verify, Pages) and Benchmark Corpus Gate 45636781444 fails 7 shards |
| 2026-11-25 15:44Z | dependencies-latest 45648404039 fails on the same rustfmt diff and the clippy `large_enum_variant` error |
| 2026-11-25 15:55–16:52Z | GitHub incident `qn8mfv6q5hlx`; all three Pages HTTP 500s fall inside it |
| 2026-11-25 | Issue #1639 opened. This PR |
| PR round 1 (7f664bb80) | Layered CI caught two regressions of this PR (crates.io probe test, governed witness); the dispatched ladder found the lone-pattern join defect |

## Requirements and status

| # | Requirement | Status | Evidence |
|---|---|---|---|
| R1 | Find every false positive, false negative, warning and error in the named runs | Done | `analysis/*`, the tables below |
| R2 | Fix them all, in this PR, everywhere each fix applies | Done except the blocked items below | Commits on the branch |
| R3 | Compare with the eight pipeline templates and report shared defects upstream | Done | `analysis/template-comparison.md`, "Upstream reports" |
| R4 | Download logs and data into this folder | Done | `ci-logs/`, `runs/` |
| R5 | Deep analysis: timeline, requirements, root causes, solutions, existing components | Done | This file and `analysis/` |
| R6 | Add debug or verbose output, off by default, where the root cause is unknown | Done where a cause was unknown | Aider deadline assertion names the awaited line; generalization failure evidence; corpus tracing from #1637 kept |

## Root causes and fixes

| Signal | Run | Kind | Root cause | Fix |
|---|---|---|---|---|
| rustfmt diff in `concept_lookup_meta_algorithm.rs`; clippy `large_enum_variant` on `ResearchDiscoveryContext` | 45636781871, 45648404039 | False negative before merge | `cargo fmt --check` ran only on `main`; the PR native lane went straight to clippy | 903009ca2 (format, box the variant); 1019111d3 (PR lane runs `cargo fmt --check` before clippy); ledgered in bd9be3a38 |
| Release preflight: crates.io token "unknown" on every main push | 45636781871 | False positive (warning noise) | The probe read a crate lookup that cannot prove publish scope | ebbe0cb36 (no-archive `PUT /crates/new` probe, which cannot publish); test updated in 2dbf99eef |
| Agent CLI E2E: every evidence projection counted zero requests | 45636781871 | False negative of the harness | Request trace moved to the trace level; harnesses set `FORMAL_AI_TRACE_REQUESTS=1` only | f2e9e9964 (every launch raises the level; a PR test scans every harness) |
| Issue #707 generalization: every held-out case "differs from local synthesis" | 45636781871 | False negative | Dotted tool names compared with serde variant names | 638ebbcab (comparison moved into a tested module; a failing run keeps its logs and plans in `failure/`) |
| Generalization upload: "No files were found" | 45636781871 | Warning | The EXIT trap deleted the workdir before evidence was copied | 638ebbcab |
| native-response-observations: 27 of 28 accepted | 45636781871 | Error | One capture case re-executed its body in the replay child | 731b6ae33 (case removed; a PR test rejects such cases) |
| Package verify: Aider issue report timed out | 45636781871 | Error | The one-line menu (8711d61f2) broke start-of-line choice matching | a03645116 (match anywhere on the line; deadline kill names the awaited line) |
| Main-only regressions with no pre-merge replay | all of the above | Process gap | Not in the ledger | b6f43c490 (`data/meta/main-only-regressions.lino`, each with its narrowest PR check) |
| Pages deploy HTTP 500 | 45636781871 | Infrastructure | GitHub incident `qn8mfv6q5hlx` (Pages degraded) | None needed; payload matches `actions/deploy-pages@v5`. Re-run the job |
| Issue 1028 ladder: a passing leg joined as "0 wrote a result" | PR round 1, ladder dispatch | False negative | download-artifact v5+ extracts a lone pattern match flat; the join looked for `<name>/` | 342b5f05b (the shared download action keeps the artifact folder for every pattern selection; six consumers covered); filed upstream |
| Failing Rust tests in the full and specification shards | 45636781871 | True positives and stale expectations | See `analysis/test-failures-overview.txt` | Background work, see "Still in progress" |
| Benchmark Corpus Gate shards killed | 45636781444 | Runner shutdown | See "Still in progress" | |
| External Benchmarks ratchet | 45197910485 | True positive (capability) | PR #1139 deleted seven templates while its own L15 gate was unmet | See `analysis/external-benchmarks-regression.md` |
| PR round 1: `release-preflight-checkout.test.mjs` | 7f664bb80 Layered CI | True positive on this PR | The test still expected the retired crate lookup | 2dbf99eef |
| PR round 1: governed witness hash mismatch | 7f664bb80 Layered CI | True positive on this PR | The rustfmt fix moved a governed native source hash | Reviewed witness refresh (cases unchanged), as in #1637's 008402605 |

## Blocked or not fixed in this PR

| Item | Why |
|---|---|
| Job Headroom 45204085037: the `main` ruleset requires no status checks | Needs a repository admin to edit the ruleset |
| Work-item qualification 41768761745 | Its fix (066cffea2, docker pull timeout) is already on main; verifying it needs a manual dispatch |
| Pages deploy | Needs a re-run after the GitHub incident; no code change |
| Ladder verification on `main` | Re-dispatch the Issue 1028 ladder after this PR merges |

## Trade-offs chosen

- A release still publishes from a `main` commit whose lint job failed. Gating release on lint
  was proposed (`template-comparison.md` F2) but contradicts the owner's decision in 4c6d5653b.
- PR clippy still does not add `--tests` (doubles PR build time; F4). Test-target lints run on main.
- `dtolnay/rust-toolchain@stable` stays floating (F8), as in #1637.
- No ready-for-review tier is added (F5). It would move the full suites onto PRs and is a
  repository-policy change for the owner.

## Upstream reports

- https://github.com/link-foundation/rust-ai-driven-development-pipeline-template/issues/274
  (a lone pattern match is extracted flat; repro, workaround and the fix applied here)
- Already reported to GitHub, not refiled: actions/download-artifact#426 and #455, with the
  unmerged fix PR #422.
- `upstream-reports/all-templates-pr-ready-tier.md`: not filed. It is a best-practice
  divergence, and hive-mind already ships per-template patches.

## Existing components used

- `actions/download-artifact` `merge-multiple`/pattern semantics and the upstream issues above.
- `actions/deploy-pages@v5` `api-client.js`: the reference Pages payload.
- crates.io `PUT /api/v1/crates/new`: authenticates and checks scope before reading an archive.
- Node's built-in `node:test`; Python `unittest` and `zipfile` for the artifact fixtures.
