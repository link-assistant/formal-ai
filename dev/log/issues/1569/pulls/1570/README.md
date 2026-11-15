# Issue #1569 / PR #1570: CI/CD false positives, false negatives, warnings and errors

Issue: <https://github.com/link-assistant/formal-ai/issues/1569>

Pull request: <https://github.com/link-assistant/formal-ai/pull/1570>

## 1. Scope and collection method

The issue was opened on 2026-11-15 against `main` at `57b7a3b6`. While the work was in progress, `main` moved to `0212e13a`. The issue repeats the requirements of #1081 and #1079, with eight templates instead of five:

* **R1**: find and fix every false positive, false negative, warning and error in CI/CD.
* **R2**: download all logs and data to this directory and analyse them here: timeline, requirements, root causes, solutions, and existing libraries.
* **R3**: where the data is too thin to find a root cause, add debug output that is off by default.
* **R4**: compare every file, workflow and CI/CD script with the `link-foundation/<lang>-ai-driven-development-pipeline-template` repositories for rust, js, python, php, csharp, java, cpp and go.
* **R5**: file an issue on every template that shares a defect, with reproduction, workaround and suggested fix.
* **R6**: follow [`hive-mind/docs/CI-CD-BEST-PRACTICES.md`](https://github.com/link-assistant/hive-mind/blob/main/docs/CI-CD-BEST-PRACTICES.md).
* **R7**: fix each defect in every place it occurs, all in this one pull request.

Collection started from the API, not from the issue text, so that a run the issue did not name could not be missed. `runs/latest-main-per-workflow.tsv` records the latest `main` run of **every** workflow file (59 workflows). Each non-green entry there was downloaded.

| Path | Contents |
| --- | --- |
| `issue-1569.json`, `pull-1570.json` | Issue and PR metadata at collection time. |
| `runs/latest-main-per-workflow.tsv`, `runs/workflows.tsv`, `runs/run-list-main.json` | Latest `main` run per workflow, with event, conclusion and head. Nine workflows were red: `CI/CD Pipeline`, `Layered CI`, `Hive Mind draft`, `dependencies-latest`, `External Benchmarks`, `Issue 1028 Agent CLI binary-tree ladder`, `Job Headroom`, `Work-item useful-edit hosted qualification`. |
| `runs/run-<id>.json`, `runs/jobs-<id>.json` | Run and per-step job records for each red run. |
| `annotations/main-0212e13a.tsv` (+ `analysis/collect-annotations.sh`) | Every check-run annotation at `main` `0212e13a`: 111 rows (17 failures, 7 warnings, 87 notices). |
| `ci-logs/job-1325905408xx.log` | Run 43638938948, `main` `58a229dd`: lint `check-wasm-worker-current` and the build. |
| `ci-logs/job-132643050511.log` | Run 43658508394, `5062e4f0`: the wasm gate. |
| `ci-logs/job-13265629xxxx.log`, `job-132656999xxx.log`, `job-132657622233.log` | Runs 43662952908/43662953231 (`a6b3d318`) and 43663095797/43663095973 (`ca458af8`): R1165-8 no-memorization, summarization ratchet, ordered lists, and fixture `::error` noise. |
| `ci-logs/job-1326671xxxxx.log`, `job-1326713xxxxx.log`, `job-132671427967.log` | Runs 43666503542/43666503819 (`1d59e65a`) and 43667851812/43667852158 (`66634fe0`): JS groups 4, 7 and 8 (summarization ratchet; "eight original domain status boundaries remain non-process"; R1165-8), lint ordered lists. |
| `ci-logs/job-13268168xxxx.log`, `job-132684771112.log` | Runs 43671213054 (JS 4/7/8) and 43671213344 (`57b7a3b6`: build, lint lane 1, `Pipeline Status`: "Failing jobs: lint, build-artifacts"). |
| `ci-logs/main-head-0212e13a/` | Runs 43692237186 (Layered CI; groups 4, 7 and 8 red) and 43692237432 (CI/CD Pipeline, lint and build), plus `job-132743905114.log` (Hive Mind draft run 43691120627). |
| `ci-logs/main-scheduled/` (+ `index.txt`) | The scheduled or dispatched red runs: ladder 42472083551 (leaves, leaf evidence, composite, and the comparison job, compressed as `../job-128712025335.log.gz` because it is 18 MB), External Benchmarks (seven weekly runs `ext-*.log`), Job Headroom (`headroom-*.log`, job 129184985116), dependencies-latest job 132444622868, and work-item qualification 41768761745. |
| `ci-logs/dependencies-latest/` | The failed steps of dependencies-latest runs 43431619115 and 43594031456. |
| `ci-logs/main-wasm-history/job-127367016456.log` | Run 42062109916 (`aa7a8005`), the last green wasm gate. It is the 1.99.1 baseline. |
| `analysis/wasm-worker-bytes.tsv` | Local rebuilds of `js/formal_ai_worker.wasm` across source trees and compilers (D1). |
| `references/wasm-artifact-43658508394/` | The worker artifact CI built in run 43658508394. It matches the local 1.100.0 build byte for byte. |
| `ci-logs/pr-head-212105dd/`, `ci-logs/pr-head-37be2669/` | This PR's own CI (section 4, last table). |
| `analysis/template-comparison.md` | R4: defect-by-defect comparison with the eight templates. |
| `references/templates/SOURCES.tsv` | The template commits that were compared. The clones themselves are not committed (60 MB), following the #1082 precedent. |
| `references/CI-CD-BEST-PRACTICES.md` | The guide at collection time. |
| `upstream-reports/*.md`, `upstream-reports/filed.tsv` | The five upstream issue bodies and their URLs (R5). |

`sessions/` holds the solver's own transcripts and is not part of the evidence.

## 2. Reconstructed timeline

| When (UTC) | Event |
| --- | --- |
| 2026-10-16 | `752da53be` commits `js/formal_ai_worker.wasm` (sha256 `4d5d3fd0…`), built with rustc 1.99.1. |
| 2026-11-04 03:42 | Work-item qualification 41768761745 (dispatch): `docker pull` hits its 180 s cap. Already fixed on `main` by `066cffea2` (#1541). |
| 2026-11-05 | Run 42062109916: the wasm gate passes on rustc 1.99.1. This is the last green baseline. |
| 2026-11-08 10:35 | Ladder run 42472083551: the sccache download fails with 503 in depth-1-2 and 502 in depth-1-1, and setup-bun reports success with no Bun in depth-3-1.2.1. `leaf-evidence` and then all 16 composite legs fail. |
| 2026-11-09 12:57 | External Benchmarks 42621251488: humaneval scores 12/20, and the loop skips the nine suites after it. |
| 2026-11-09 13:20 | Job Headroom 42624594054 fails: the ladder used 99.7% of its 1200 s test budget. |
| 2026-11-13, 11-14 | dependencies-latest (43431619115, 43594031456) rebuilds the worker without staging it, so its own wasm gate fails: "dependency currency: FAIL". |
| 2026-11-14 | Commits `c50eedf30`, `0bb8317e0` and `29a8be67e` edit worker inputs without a rebuild, and the runners move to rustc 1.100.0. Main run 43638938948 (`58a229dd`) fails `check-wasm-worker-current`. The build fails too, because Rust 1.100 moved test executables to `target/<profile>/build/<pkg>/<hash>/out/`. |
| 2026-11-14 to 15 | Runs at `a6b3d318`, `ca458af8`, `1d59e65a`, `66634fe0` and `57b7a3b6` each add one more main-only failure: the ordered-list gate after #1563, the summarization ratchet, the domain-status anchors, R1165-8, and the version-literal gate. Every one of their PRs had passed. |
| 2026-11-15 07:42 | Hive Mind draft 43691120627 (issue event) ends "No verified useful candidate: gap-no-unique-candidate". |
| 2026-11-15 07:44 | Issue #1569 opened and PR #1570 started. |
| 2026-11-15 07:57 | `main` `0212e13a`: CI/CD Pipeline 43692237432 and Layered CI 43692237186 are red. The annotation set is captured. |
| 2026-11-15 07:58 to 09:19 | This PR's commits (section 5). PR run at `212105dd` catches two of its own regressions (section 4). |

## 3. Causal chain

Almost every red `main` run has the same shape: **a check that runs on `main` but not on the pull request**. Each feature PR passed its PR checks. Each merge then broke `main` in a gate the PR had never run. Because `main` was already red, the next breakage was not noticed. That is why five independent defects piled up in one week.

1. Rust 1.100 changed the build-dir layout (D2). Release gates that globbed `target/*/deps` found no test executables. The PR profile did not run them.
2. A compiler upgrade and three input edits changed the worker bytes (D1). `check-wasm-worker-current` runs only on `main`, so no PR saw it. dependencies-latest rebuilt the worker but did not stage it, so its own gate stayed red every day.
3. Ordered lists, version literals, the summarization ratchet, anchors located by line number, and R1165-8 (D3–D5) were again checked only on `main`, or only in groups the PR inventory did not include.
4. Moving them to PRs pushed the single fast JS group past its 360 s budget (D6). The group had to be split.
5. Separately, infrastructure flakes (D8, D9) and a fail-fast loop (D7) turned one failure into many red jobs. These false positives in turn hid the true signal (the humaneval drop, D10).

## 4. Defect register

| ID | Kind | Symptom (evidence) | Root cause | Fix in this PR |
| --- | --- | --- | --- | --- |
| D1 | false negative on PRs, error on main | `check-wasm-worker-current` red on `main` since 11-14 (`job-132643050511`, `annotations`). dependencies-latest red daily. | The worker bytes depend on the sources **and** rustc. 1.99.1 → 1.100.0 changed them, and `c50eedf30`, `0bb8317e0` and `29a8be67e` changed them again even on 1.99.1 (`9e1cc04e…`, `analysis/wasm-worker-bytes.tsv`). The gate needs a compiler, so it ran only on `main`. dependencies-latest rebuilt (311231 bytes) without `git add`. | `108db5898`: the worker is rebuilt on 1.100.0 (`85662987…`, 311725 bytes, the same bytes as the CI artifact). `data/meta/wasm-worker-inputs.lino` records the compiler, the dependency closure and the input digests. `scripts/wasm-worker-inputs.mjs --check` runs on PRs with no compiler. `--explain` on `main` says whether the inputs or the compiler moved. dependencies-latest stages the worker and its record. |
| D2 | error on main | Build and memory tests found no test binaries (`job-132590540918`). | Rust 1.100 puts test executables under `target/<profile>/build/<pkg>/<hash>/out/` (build-dir v2). `collect-build-artifacts.sh` searched `deps/`. | `62be90540` takes `executable` from `cargo --message-format=json`, and falls back to searching both layouts. `2c6ec5756`: the native fixture runner accepts the new path. `3a5d802cc` and `7aca024f4`: the memory store recognises those binaries. |
| D3 | false negative | `check-ordered-lists-are-canonical` (`job-132656302385` …): `rust/src/agentic_coding/modules.rs` was out of order after #1563. | A union merge produced an order no PR had ever checked. | `142087d2d` reorders the file. `282a4ff4e` runs the ordered-declaration check in the PR inventory. |
| D4 | false negative | `check-generated-version-literals` red on `0212e13a`. | The gate did not count the captured run records under `data/benchmarks/ci-phase-timing/baseline-runs/` as evidence. | `9421953d3` counts them as evidence, and adds a JS twin that runs on PRs (`issue-1569-generated-version-literals.test.mjs`). |
| D5 | false negative | JS groups 4/7/8 on `main`: the summarization ratchet, "eight original domain status boundaries remain non-process", and R1165-8. | Floors were computed on wrong language labels, the anchors were line numbers, and #1558 added a request capture that was never counted. | `4f8ecd922`, `545c30b04`, `2674b9663`. All three suites now also run in the PR inventory (`282a4ff4e`). |
| D6 | warning heading for a timeout | PR run at `212105dd`: "still running after 270s (75% of its 360s execution budget)". | Moving D3–D5 onto PRs grew the single fast JS group. | `9ab8b9b27` splits the inventory into two longest-first groups. `fast-pull-request-production-profile.test.mjs` asserts the split. |
| D7 | false positive (red annotations on green jobs) | `annotations/main-0212e13a.tsv`, JS group 1: three `failure` annotations ("no attempt identity was admitted…", "…hashes to dddd…"), each printed twice. | The hive draft-attempt and pre-effect deferral tests ran the scripts with `GITHUB_ACTIONS` inherited, so their deliberate fixture errors became workflow commands. | `572c558ee` strips the runner variables from those subprocesses. |
| D8 | masked errors | External Benchmarks: after humaneval failed, the loop never ran the other nine suites (`ext-*.log`). | `set -e` in a `for` loop. | `defe84654` runs every suite, annotates each failure, and still fails the step. |
| D9 | false positive cascade | Ladder 42472083551: three infrastructure failures made `leaf-evidence` fail, then all 16 composite legs failed on an artifact that was never uploaded. | (a) The composites had `if: always()`. (b) The sccache release download answered 503/502 after `@actions/tool-cache` had already retried it 3 times. The action's post step then failed again ("Unable to locate executable file: undefined"), because `SCCACHE_PATH` was never set. (c) `oven-sh/setup-bun@v2` (`0c5077e5`, v2.2.0, the latest) exits only in `.then`/`.catch`. When the download promise never settles, Node drains the loop and exits 0: no output, no error, post step skipped, then "bun: command not found". | `defe84654`: the composites require `leaf-evidence` success. `f378aa882`: the sccache install gets `continue-on-error: true`, and the runner clones that flag into the post stage (actions/runner `ActionManager.cs` copies the step for the post list; `CompositeActionHandler.cs` applies `ContinueOnError`). The existing #1031 probe falls back to the real compiler, now with a "sccache install failed" warning. The same applies to `desktop-release.yml`. A new composite `.github/actions/setup-bun-verified` retries once when `bun-path` is empty and then proves the pinned `bun` runs. |
| D10 | true positive, documented | humaneval: 1/20 (09-14), 20/20 (09-21), 12/20 (10-26 and 11-02). On 09-21 mbpp 18/20 stopped the loop. | A real capability regression between `efb6ddb6` and `d209aac6`. D8 had hidden the other suites. | Not a CI defect. With D8 fixed, the next weekly run reports every suite. Recorded here for the capability owners. |
| D11 | warning, documented | Job Headroom 42624594054: ladder at 99.7% of its 1200 s `TEST_BUDGET_SECONDS` (error); ladder 77.2% of 30 min, CI/CD Test 83.3% of 1080 s, JS tier 73.1% of 360 s (warnings). Earlier runs: CI/CD Test 118.9% and 109.8%, Coverage 93.4%. | Growth of the measured suites. The audit is doing its job. | Documented only. The CI speed policy keeps budgets and asks for repartitioning, not larger timeouts. The JS tier part is addressed by D6. The ladder budget is left for its own issue. |
| D12 | false positive (wrong diagnosis published on the issue) | Hive Mind draft 43691120627. The issue comment said "Attempt outcome: `deferred-infrastructure`", and the job warned "Hive Mind ended the attempt before solve". Yet solve ran from 07:59 to 08:02, the session was posted, and the model ended in `planned_not_executed` (`main-head-0212e13a/job-132743905114.log`; session artifact 14153260749: `attempt-outcome.json` is `{"schema":"draft-command-outcome/v1","state":"finished","exitCode":5,...}`). | `scripts/capture-draft-command.mjs:16` writes `<phase>-outcome.json`, which for the `attempt` phase is `attempt-outcome.json`. #1484 (`0f683de29`, the same morning) named Hive Mind's typed outcome `FILES.hiveOutcome = 'attempt-outcome.json'` in the same directory. `verifyHiveAttempt` read the capture record as a Hive verdict. With no `outcome` field, that became `deferred-infrastructure` with the default "before solve" message on **every** attempt. Hive Mind at `318abfd9` writes none of the contract files yet (hive-mind#3909 is open), so the honest receipt is `serving-identity-unverified`. The job's red status itself comes from `scripts/hive-mind-draft-result.mjs:72` (`gap-no-unique-candidate`, no draft PR). That part is by design: 13 of the last 15 draft runs succeeded. | This PR renames Hive Mind's expected outcome file to `attempt-hive-outcome.json`, so no capture phase can collide with it, and ignores any `draft-command-outcome/v1` record passed as a Hive outcome. Test: `hive-draft-attempt-identity.test.mjs` "the capture record of the attempt phase is never read as a Hive Mind outcome". It fails on the old library: `'attempt-outcome.json'` equals the `attempt` capture file. |
| D13 | error, already fixed | Work-item qualification 41768761745: `docker pull` hit its 180 s cap. | Image pull time. | Fixed on `main` by `066cffea2` (#1541) before this issue. Not repeated. |

### What was checked and found clean

* The Ubuntu 26 migration notices (87 of the notices): an informational runner-image notice.
* Release Preflight "UNKNOWN: crates.io publish token": correct. crates.io has no endpoint that accepts a token for reading, and the warning says so.
* VS Code "bundle your extension" warning from `vsce`: advisory, about packaging size.
* Hive phase-timing warnings for run 40668396305 (delivery-wall 3620 > 3600, and others): true measurements of a historical run.
* "No files were found … native-agentic-protocol-captures": expected when no protocol capture runs.
* The "sccache unavailable" warnings (#1031): the intended fallback.
* Cache-bucket notices.
* Eleven "file has N lines (approaching limit of 1500)" warnings on the final PR head (`298d39e6a`). They come from `scripts/check-file-size.mjs`, whose warning threshold is 1400 and whose hard limit is 1500. They name ten files: seven `.lino` files under `data/seed/` and `data/meta/`, and three `ts/` sources, up to `ts/seed_loader.ts` at 1498. These are true advisory warnings. Every count is identical on `main`, and this PR touches none of the files. Splitting a seed or loader file changes what the seed and worker loaders read, which is a content refactor rather than a CI defect, so it is left to the owners of those files. The warning exists precisely so that this split happens before the hard limit is reached.
* actionlint 1.7 on the touched workflows (ladder, desktop-release, external-benchmarks): clean. zizmor is configured in `.github/zizmor.yml`. Its `unpinned-uses` stance (tags rather than SHAs) is deliberate in this repository and is noted in the template comparison.
* The templates' `rerun-unacquired` workflow was not adopted. Every cancelled run in the sample was a concurrency supersede, with no unacquired runner.
* `check-source-duplication.mjs` exits 1 locally, but it is not wired into any workflow. Noted, and out of scope.

### What this PR's own CI caught

| Run | Head | Finding | Fixed by |
| --- | --- | --- | --- |
| 43693412224 (`pr-head-212105dd/layered-js-132748674911.log:2013`) | `212105dd` | "✖ the repository holds the abbreviation ratchet": the new layout helper used an abbreviation | `7aca024f4` |
| same log, `:3680` | `212105dd` | 75% budget warning on JS group 1 | `9ab8b9b27` (D6) |
| 43693412224, annotations | `212105dd` | the D7 fixture `::error` lines also show up on the PR | `572c558ee` |
| `pr-head-37be2669/jobs.txt` | `37be2669` | all jobs green after the split | — |

## 5. Requirement-by-requirement root cause and plan

**R1.** D1–D9 and D12 are fixed, each with a test that reproduces it:

* `issue-1569-*.test.mjs`
* `rust/tests/web/issue-1569-ladder-leaf-infrastructure.test.mjs` (6 cases: the sccache tolerance, its main and post stages, the fallback warning, the Bun retry conditions, and the verifier on a missing, wrong, right or unpinned Bun)
* `sccache-outage-fallback.test.mjs`
* `fast-pull-request-production-profile.test.mjs`
* `hive-draft-attempt-identity.test.mjs` (D12)
* the twin and inventory tests for D3–D5

D10, D11 and D13 are documented above with the reason for each.

The systemic cause (main-only gates) is now handled by policy:

* the fast PR inventory runs every main-only regression that does not need a compiler;
* the one gate that does need a compiler (D1) gets a compiler-free input check on PRs.

**R2.** This directory.

**R3.** Two cases were too thin to diagnose:

* D1: `node scripts/wasm-worker-inputs.mjs --explain` prints which input or which compiler moved. It runs only when the gate fails.
* sccache: the existing `FORMAL_AI_CI_VERBOSE` diagnostics (off by default) still apply. The install outcome is now named in the warning.

**R4.** See `analysis/template-comparison.md`. In summary:

* The Rust template is not affected by D2. cargo-llvm-cov 0.9.1 handles build-dir v2, and its main run 43566214814 on 1.100.0 is green.
* The class "gate only on main" is present in rust, php and csharp.
* java has no aggregate status job.
* go jobs have no `timeout-minutes`.

Template practices formal-ai still lacks (documented, not adopted):

* SHA-pinned actions (584 tag pins). A repository-wide change; the zizmor policy is deliberate.
* A Pages deploy gated by a variable. Pages is enabled here, so it does not fail today.

**R5.** These were filed without naming this repository:

| Template | Issue | Topic |
| --- | --- | --- |
| rust | <https://github.com/link-foundation/rust-ai-driven-development-pipeline-template/issues/249> | Pages deploy only on push: green PR, red main |
| php | <https://github.com/link-foundation/php-ai-driven-development-pipeline-template/issues/25> | Same, and `configure-pages` inside the build job |
| csharp | <https://github.com/link-foundation/csharp-ai-driven-development-pipeline-template/issues/110> | "Check Codecov token" only on main |
| java | <https://github.com/link-foundation/java-ai-driven-development-pipeline-template/issues/29> | No aggregate pipeline-status job |
| go | <https://github.com/link-foundation/go-ai-driven-development-pipeline-template/issues/17> | No `timeout-minutes` on release jobs |

js, python and cpp had no verified defect beyond the informational X-1 (preflight in `report` mode on same-repo PRs).

**R6.** The guide's relevant rules are followed:

* §7: fresh-merge validation already exists.
* Budget warnings before timeouts: D6.
* Aggregate status: present.
* Never weaken assertions: no test was relaxed. Each ratchet was re-based on corrected inputs, with the reason in its commit.
* Debug output off by default: R3.

**R7.** "Same problem everywhere":

* D2: all three consumers of test-binary paths were fixed.
* D7: both leaking test files were fixed.
* D9 (sccache): both sites, the composite and `desktop-release.yml`.
* D9 (setup-bun): the verified composite is adopted where the failure was observed (the two ladder sites).

The other 32 `oven-sh/setup-bun@v2` uses in 18 workflow files carry the same latent hazard. They are not converted in this PR, for three reasons:

* `browser-coverage-bootstrap.test.mjs` and `relocated-build-stage-controls.test.mjs` pin the raw `uses:` of their workflows;
* `release.yml` and the generated `release-staged.yml` are out of bounds without the costly deployed check;
* the hazard occurred once in all collected logs.

The composite is a drop-in (`uses: ./.github/actions/setup-bun-verified`) when one of those sites shows the symptom.

## 6. Existing components and libraries

* `@actions/tool-cache` `downloadTool` already retries 5xx errors 3 times. A second retry layer for sccache would add little, so the fix makes the cache optional instead.
* actions/runner composite semantics: `continue-on-error` on a composite step covers its post step, so no wrapper action is needed.
* `oven-sh/setup-bun` v2.2.0 is the latest release, so an upgrade would not help. The composite relies on its documented `bun-path` output.
* cargo's `--message-format=json` `compiler-artifact.executable` field is the supported way to locate test binaries, and it works in both build-dir layouts. cargo-llvm-cov uses the same approach.
* The repository's own `scripts/run-with-budget-warning.sh`, the fast-inventory scheduler, and the #1031 sccache probe were reused rather than replaced.

## 7. Verbose output

| Switch | Default | Effect |
| --- | --- | --- |
| `FORMAL_AI_CI_VERBOSE=true` (repository variable) | off | sccache server debug log, as in #1081 D9 |
| `node scripts/wasm-worker-inputs.mjs --explain` | runs only when the main wasm gate fails | names the changed inputs, or the compiler change |
| `scripts/run-test-files.mjs` `[test-file]` lines | always | per-file start and finish times, used to read the D6 budget |

## 8. Local verification

No Rust crate was built or tested locally, per repository policy. The worker was built with the exact `wasmCompilerArguments` flags only to establish D1. JS tests were run one file at a time:

* `issue-1569-ladder-leaf-infrastructure.test.mjs` 6/6
* `sccache-outage-fallback.test.mjs` 5/5
* `fast-pull-request-production-profile.test.mjs` 15/15
* the `issue-1569-*` twins, and the hive annotation tests

Scripts:

* `node scripts/documentation-only-ci-policy.mjs`
* `node scripts/check-readable-code.mjs`
* `node scripts/measure-abbreviations.mjs`
* `node scripts/check-generated-version-literals.mjs`
* `node scripts/wasm-worker-inputs.mjs --check`
* `node scripts/translate-js-rust.mjs --verify`
* actionlint on the changed workflows

All pass. CI on the PR heads is the final evidence. See the PR checks.
