# Issue #1014 / pull request #1015 evidence archive

This directory is the canonical evidence record for the complete CI/CD audit
requested by [issue #1014](https://github.com/link-assistant/formal-ai/issues/1014)
and implemented in
[pull request #1015](https://github.com/link-assistant/formal-ai/pull/1015).
It preserves the inputs, the exact failing and warning output, the research,
the tests-first proof, the upstream reports, and the final verification.

## 1. Scope and collection method

The issue named twelve runs at default-branch SHA
`98cb3c803a72161a880968647330358c65d9b83f`. All twelve complete logs were
downloaded with `gh run view --log`; the one failed run also has a
failure-only copy. For every run the run, job, and artifact metadata is in
`raw-data/`. The combined status, all 66 check-run annotation responses, issue
and pull-request timelines, comments, reviews, and events were collected
separately so that a green workflow conclusion could not hide annotations.

The five initial #1015 workflows at prepared SHA
`6fd8cdb620a0ddb7f4f8dfc4557f1401df8b8124` were also downloaded after they
completed. Their stored metadata and logs prove the observations predate this
fix. `raw-data/branch-initial-diagnostics.txt` is the corresponding diagnostic
scan.

The archive contains more than 570 files at the time of analysis:

- `ci-logs/run-*.log`: every issue-listed baseline workflow log.
- `ci-logs/branch-initial/`: all initial pull-request workflow logs.
- `ci-logs/pushed-head-3e7d64f6/`: the complete first-push Agentic Matrix
  failure and the Desktop VSIX failure, retained because they exposed two
  regressions before finalization.
- `ci-logs/pushed-head-c5fae9d4/`: all thirteen exact second-candidate workflow
  logs plus focused copies of the lint and three macOS consumer jobs. They
  prove twelve unaffected workflows passed while the remaining run exposed a
  package-only `esbuild` test in the source-only VS Code suite, an
  argv/process-tree race, and an under-provisioned archive fan-out.
- `ci-logs/final-head-1359540e/`: both complete non-passing third-candidate
  workflow logs, all five macOS consumer logs, all six desktop release-lane
  logs, and the focused local red/green reproductions they motivated.
- `raw-data/run-*-jobs.json`, `run-*-artifacts.json`, and annotations: complete
  GitHub execution metadata.
- `raw-data/pushed-head-c5fae9d4/`: exact-head metadata for all thirteen
  second-candidate workflows, their jobs and artifacts, plus all 71 check-run
  annotation responses.
- `raw-data/audited-warning-error-excerpts.txt` and
  `key-diagnostics-with-lines.txt`: indexed candidates, not substitutes for
  the complete logs.
- `raw-data/references/`: current immutable template heads/trees and Hive Mind
  `CI-CD-BEST-PRACTICES.md`.
- `raw-data/related/`: the latest related issues, merged PRs, three GitHub
  comment surfaces per PR, and previous template reports.
- `raw-data/*audit*`, `*versions*`, and Gemini v0.55.1 source snapshots: online
  package/advisory and upstream implementation research.
- `upstream-reports/`: exact report bodies with reproduction, workaround, and
  suggested source fix.
- `local-tests/`: failing-before-fix and passing-after-fix test evidence.

The older template source captures remain byte-for-byte evidence, but their
manifest basenames now end in `.snapshot`. GitHub's Dependency Graph treats
any `Cargo.toml`, `package.json`, or `pyproject.toml` below the repository as a
live project; the suffix prevents archived evidence from creating projects.

## 2. Reconstructed timeline

| Time (UTC, 2026-08-15) | Event | Evidence |
| --- | --- | --- |
| 12:35–12:36 | Main CI starts JavaScript installs, agent CLI E2E, and the three Intel macOS core lanes. | `ci-logs/run-31884932415.log` |
| 12:35:46 | Two npm installs print `2 high severity vulnerabilities`, but do not fail. | `raw-data/key-diagnostics-with-lines.txt` |
| 12:35:54–12:36:11 | Bun refuses three untrusted global package lifecycle scripts and reports that fact as routine output. | Same indexed evidence and full pipeline log |
| 12:36:26–12:36:38 | macOS core-1/2/3 independently begin `cargo nextest run`; each recompiles the same test graph before applying its partition. | `raw-data/macos-core-timing-lines.txt` |
| 12:45:37–12:45:39 | Gemini warns twice that true color is absent and once that its disappearing `projects.json.lock` directory cannot be scanned. | `raw-data/key-diagnostics-with-lines.txt` |
| 12:53:07–12:55:03 | The three macOS lanes pass after 999, 1,075, and 1,117 seconds, crossing the 840-second warning threshold and approaching the 1,200-second execution budget. | `raw-data/macos-core-timing-lines.txt` |
| 13:08:44 | Auto Release rejects cycle `v0.345.0..HEAD` because no merged PR has valid end-to-end Formal AI attribution. Pipeline Status correctly propagates this job failure. | `ci-logs/run-31884932415-failed.log` |
| 13:41:40 | Issue #1014 opens against main SHA `98cb3c80`, listing twelve runs and one failure. | `raw-data/issue.json` |
| 13:43:02–13:43:05 | Draft PR #1015 opens from prepared SHA `6fd8cdb6`; five initial workflows start and later all succeed. | `raw-data/pull-request.json`, `branch-runs-latest.json` |
| Investigation | Minimal issue tests are committed to the worktree and fail 7/7 against the original implementation. Full lock audits reveal advisories outside the two install-time summaries. | `local-tests/regression-red.log`, `raw-data/*audit*` |
| Fix verification | Issue tests, five explicit lock audits, affected desktop/VS Code suites, web build, and actionlint pass. | `local-tests/` |
| 15:28:41 | The first implementation head `3e7d64f6` starts fresh PR workflows. | `raw-data/pushed-head-3e7d64f6/branch-runs-after-first-push.json` |
| 15:29:41 | VSIX packaging fails: esbuild follows Playwright's optional private `chromium-bidi/lib/cjs` imports after the Puppeteer 25 update. | `ci-logs/pushed-head-3e7d64f6/Desktop-Release-31892818161-vsix-job-95031537607.log` |
| 15:35:33–15:37:21 | The Rust policy stage finds an archived `/v1/knowledge-graph` string because the terminology lint scans `dev/log`, then detects that the new requirements shard was not assembled into `REQUIREMENTS.md`. | `ci-logs/pushed-head-3e7d64f6/CI-CD-Pipeline-31892818297-lint-job-95031839325.log` |
| 15:36:52 | The OpenCode matrix leg fails with `Exec format error`: the blanket `--ignore-scripts` policy prevented its pinned package from selecting the Linux binary. | `ci-logs/pushed-head-3e7d64f6/Agentic-CLI-Matrix-31892818188.log` |
| 15:39:38 | The pipeline's second OpenCode installation fails for the same omitted postinstall, proving both install surfaces need the scoped exception. | `ci-logs/pushed-head-3e7d64f6/CI-CD-Pipeline-31892818297-agent-e2e-job-95031839344.log` |
| 15:50:04–15:51:35 | All three archive consumers extract to unique temporary directories, but tests invoke compile-time paths under the build workspace; each slice stops on four `NotFound` failures. | `ci-logs/pushed-head-3e7d64f6/CI-CD-Pipeline-31892818297-macos-slice*-job-*.log` |
| Fresh-CI correction | Playwright is externalized and shipped in the VSIX; browser-commander 0.10 forwards the packaged executable. Lifecycle trust is limited to pinned OpenCode. Historical evidence is excluded from the source-terminology lint, requirements are reassembled, and nextest extracts into the workspace so legacy Cargo binary paths resolve. The extracted-VSIX, OpenCode, and archive-relocation experiments pass. | `local-tests/fresh-ci-regressions/` |
| 16:16–16:51 | The corrected candidate passes exact registered Rust, wasm, and web gates; desktop 140/140; VS Code 52/52; all five JavaScript lock audits; data 17/17; self-AST 10/10; all-feature shards 488/488 and 2,777 passed with four ignored; and doctests. The main suite takes 1,102 of its 1,200-second budget, so the existing 70% performance warning is retained rather than hidden. | `local-tests/fresh-ci-regressions/*-final.log` |
| 17:11:32 | Candidate `c5fae9d4` starts thirteen fresh exact-SHA workflows. Twelve finish successfully: Security, stock-install, all auxiliary benchmarks, the VSIX package, Gemini, and all three OpenCode lanes pass; only CI/CD Pipeline fails. | `raw-data/pushed-head-c5fae9d4/branch-runs.json`, complete workflow logs, and GitHub job/check metadata |
| 17:25:13 | The registered web gate reaches the VS Code suite after installing only root and e2e dependencies. The new bundle test imports package-local `esbuild`, so lint fails with `ERR_MODULE_NOT_FOUND` while the dedicated VSIX job succeeds. | `ci-logs/pushed-head-c5fae9d4/CI-CD-Pipeline-31897604270-lint-job-95043525067.log:4762` |
| Second fresh-CI correction | Keep the documented 51-test source suite dependency-free; move the real esbuild/Playwright graph check to `test:package` and run it after extension dependencies are installed, immediately before VSIX packaging. A workflow regression and no-`node_modules` execution pin both boundaries. | `local-tests/fresh-ci-regressions/vscode-ci-dependency-boundary-*.log` and `vscode-source-without-node-modules-green.log` |
| 17:25:22–17:28:13 | The three archive consumers spend 22–171 seconds downloading the same archive and another 31–38 seconds extracting it; that overhead is inside the 10-minute GitHub job cap but outside the advertised 480-second command budget. | `ci-logs/pushed-head-c5fae9d4/CI-CD-Pipeline-31897604270-macos-job-*.log` |
| 17:29:42 | Slice 2 fails `timeout_terminates_descendant_processes`: the 20 ms timeout reaches command-stream 0.15's shell-string process leader while a fixture descendant is joining/spawning, and `descendant-survived` is written after the one-shot group signal. | `ci-logs/pushed-head-c5fae9d4/CI-CD-Pipeline-31897604270-macos-job-95044719217.log:1486` |
| 17:34:53–17:35:38 | Slices 1 and 3 are canceled by the unchanged 10-minute job cap after only 518/897 and 737/896 tests. The live command warning had already identified slow 60–148 second tests; neither job failed archive relocation. | `ci-logs/pushed-head-c5fae9d4/CI-CD-Pipeline-31897604270-macos-job-95044719212.log` and `...95044719220.log` |
| Third fresh-CI correction | Keep one build and the existing 10-minute consumer cap, but fan the archive to five smaller slices. Upgrade command-stream to 0.16 and use its exact-argv constructor so the allowlisted agent executable, not an added shell, leads the Unix process group. | `local-tests/fresh-ci-regressions/macos-fanout-and-exact-argv-{red,green}.log` |
| 18:08–19:00 | The corrected source-only/package boundary passes without `vscode/node_modules`; the descendant timeout passes ten consecutive 20 ms stress runs; the complete registered Rust stage passes. The first exhaustive run finds only a stale generated runner census after 2,805 passing unit tests. Regenerating that census makes its focused test and the final all-feature run green: 3,755 tests across all harnesses, including 2,806 in the largest unit harness, with four intentional ignores. | `local-tests/fresh-ci-regressions/{vscode-source-without-node-modules-green,descendant-timeout-command-stream-016-stress,registered-rust-stage-after-command-stream-016-final,full-rust-tests-after-command-stream-016,self-ast-census-green,full-rust-tests-final}.log` |
| 19:13:09 | Candidate `1359540e` starts thirteen exact-SHA workflows. Eleven succeed; Desktop Release and CI/CD Pipeline preserve the only failures. | `raw-data/final-review/branch-runs-through-1359540e.json` and `run-31903294{075,195}.json` |
| 19:28–19:52 | All six desktop release lanes fail the same source test: its Cargo-manifest assertion still expects command-stream 0.15 although the reviewed production manifest and lock use 0.16. | `ci-logs/final-head-1359540e/Desktop-Release-31903294075.log` and six focused job logs |
| 19:31:40 | macOS slice 2 fails a Gemini logging-proxy request after 30.424 seconds with `WouldBlock`. All helper children inherited one real home memory file, so nextest-concurrent servers serialized response recording on the same advisory lock. | `ci-logs/final-head-1359540e/CI-CD-Pipeline-macos-core-slice-2-95058579481.log` and `local-reproduction-shared-memory-lock.log` |
| 19:35–19:42 | The exact five-way archive layout remains too large: slices 5, 1, and 3 hit the 10-minute outer cap. Slice 3 alone spends 502 seconds in the budgeted command and the successful slice 4 spends 434 seconds, leaving inadequate setup/skew margin. | `ci-logs/final-head-1359540e/CI-CD-Pipeline-macos-core-slice-{1,3,4,5}-*.log` |
| Fourth fresh-CI correction | Synchronize the desktop assertion; give every helper-owned server private memory/dialog state; add an atomic discriminator to concurrent proxy logs; and fan the one archive to eight consumers. The original 600-second job cap and 480-second command budget remain strict. | `local-tests/fresh-ci-regressions/desktop-command-stream-manifest-green.log`, `private-server-state-green.log`, `logging-proxy-parallel-final.log`, and `macos-eight-way-fanout-{red,green}.log` |
| 21:14:30–21:48:13 | Candidate `743534ed` proves the desktop assertion and helper-state corrections, but exact slice 7/8 is canceled at the unchanged outer cap after 290/336 tests. The command emits its 360-second warning, reaches 471 seconds with 46 tests still queued, and GitHub cancels the job before nextest can summarize. | `ci-logs/final-head-743534ed/ci-cd-pipeline-31908885457-slice-7-job-95072624766.log` |
| Fifth fresh-CI correction | Preserve the single archive and both strict budgets, but repartition the measured outlier to twelve consumers. This reduces the nominal per-consumer test count from 336 to about 224, providing roughly one-third execution headroom without hiding the capacity failure. | `local-tests/fresh-ci-regressions/macos-twelve-way-fanout-{red,green}.log` |

## 3. Requirement ledger

| ID | Requirement reconstructed from the issue and repository policy | Result |
| --- | --- | --- |
| R1014-1 | Preserve and inspect all twelve named runs plus PR checks, jobs, artifacts, annotations, discussion, and exact SHAs. | Complete; see section 1 and `raw-data/`. |
| R1014-2 | Find all false positives, false negatives, warnings, and errors; fix every actionable occurrence across the codebase without suppressing true failures. | Complete; see finding ledger below. |
| R1014-3 | Correct the only failed run at its root while retaining release-integrity enforcement. | Automatic release now defers; manual release remains strict. |
| R1014-4 | Correct near-timeout macOS warnings structurally and prove the archive is relocatable. | One reusable nextest archive replaces three cold builds; twelve consumers extract to the original workspace, and a retained experiment reproduces default-extraction failure before passing with that setting. |
| R1014-5 | Close dependency-audit false negatives for every JS lock, not only the lock that happened to print a warning. | One dynamically discovered fail-closed gate covers both Bun locks plus three npm locks; all advisories are resolved. |
| R1014-6 | Remove harness and evidence artifacts that make successful CI noisy or misleading. | Gemini environment, Bun lifecycle policy, and archived manifest names corrected everywhere found. |
| R1014-7 | Compare the full current Rust, JS, and Python template trees and Hive Mind guidance, reuse proven components, and report shared defects upstream. | Complete immutable comparison; seven upstream issues and eight exact report/comment bodies retained. |
| R1014-8 | Reproduce first and retain automated tests, complete analysis, and verification evidence. | 7/7 initial failures retained; focused and affected green checks retained. |
| R1014-9 | Finish all work in prepared PR #1015, document timeline/root causes/plans, merge current main, and require fresh green CI. | Documentation is here; merge/fresh-CI evidence is appended during finalization. |

## 4. Complete diagnostic and root-cause ledger

| Observed signal | Classification and root cause | Resolution / evidence |
| --- | --- | --- |
| Auto Release error: no eligible Formal AI-authored PR in `v0.345.0..HEAD` | **True policy result, wrong automatic control flow.** PRs #1007, #1011, and #1013 do not put the same canonical PR trailer plus valid session evidence on every introduced non-merge commit. The new issue-#924 check hard-exited before the older report-only release ratchet could run, making immutable main unable to heal itself. | A read-only preflight returns `Deferred` and writes `should_release=false`; automatic publication stops cleanly with a notice. `version-and-commit.rs` still calls the hard gate for manual release. Behavioral and workflow tests pin both halves. |
| Pipeline Status failure | **True positive.** It correctly propagated Auto Release's failure; changing aggregation would hide a real failed dependency. | No suppression or aggregator weakening. Fix the producer only. |
| `Core test slice took ...` warnings at 999/1,075/1,117 seconds, followed by capped three-way, five-way, and eight-way archive consumers | **True capacity warning.** nextest originally partitioned execution only after three independent compilations. Reusing one archive removed that waste, but default temporary extraction broke compile-time `CARGO_BIN_EXE_*` paths. Workspace extraction fixed relocation. Exact five-way evidence measured 434 seconds for a successful command and 502 seconds for a canceled one. Eight-way evidence then isolated the remaining skew: slice 7 reached 290/336 tests at 471 seconds and was canceled at the 600-second outer cap. | One reusable workflow creates and uploads the archive. Twelve consumers extract into `$GITHUB_WORKSPACE`, restoring the original `target` path while nextest remaps runtime metadata. The retained experiment proves default extraction fails and workspace extraction passes; the measured five- and eight-way red runs justify the final 12-way fan-out without increasing either timeout. |
| npm reports two high advisories twice but succeeds | **True vulnerability, false-negative gate.** Install-time npm audit is advisory output, not a required exit status. Separate audits found 2 high advisories in `tests/e2e`, 10 in each desktop lock, 10 in the VS Code lock, and a moderate DOMPurify issue in the root Bun lock; the experiment Bun lock was already clean. | The fail-closed script discovers and audits all five live locks at moderate-or-higher severity. DOMPurify is 3.4.13; npm locks are refreshed; desktop/VS Code use web-capture 1.11.2 plus scoped Puppeteer 25.7 overrides to remove the stale `extract-zip` chain. All five audits return zero. Routine installs use `--no-audit --no-fund` so findings have one authoritative gate. |
| `Blocked 2 postinstalls` / `Blocked 1 postinstall` | **Expected Bun security policy, ambiguous noise, with one required exception.** Most global CLIs do not need dependency lifecycle scripts. OpenCode does: `opencode-ai@1.18.4` uses its own postinstall to replace a Windows placeholder with the platform binary. Applying `--ignore-scripts` indiscriminately caused the fresh matrix's Linux `Exec format error`. | Routine installs use `--ignore-scripts`; only the exact lockfile-pinned OpenCode package uses `--trust`. An isolated ignored install reproduces the failure and an isolated trusted install runs version 1.18.4. The regression enumerates every global install and permits no other trusted package. |
| Gemini true-color warnings | **Harness configuration.** Gemini v0.55.1 checks `COLORTERM` for `truecolor`/`24bit`; `TERM=xterm-256color` alone does not satisfy it. | Export `COLORTERM=truecolor` in the E2E harness. Upstream behavior source is preserved in `gemini-...-compatibility.ts`. |
| Gemini `projects.json.lock` ENOENT | **Upstream race exposed by harness layout.** The isolated home was also the scanned project. `ProjectRegistry` creates/removes a proper-lockfile directory while `getFolderStructure` recursively traverses it; the directory may disappear between enumeration and descent. | Use sibling `$WORKDIR/project` and `$WORKDIR/home`. The upstream race is reported as Gemini #28826 with reproduction and proposed ENOENT handling. |
| Dependency Graph workflow for `/dev/log/.../templates/...` | **False-positive project discovery.** Archived full template trees retained exact manifest names, so Dependabot treated evidence as applications and submitted their dependency graph. A full-tree search found the same hazard in older case-study raw data. | Rename all 21 evidence manifests below `dev/log` and `docs/case-studies` with `.snapshot`; a recursive regression fails if any scanner basename returns. Historical bytes remain available. |
| `rehash: warning: skipping ca-certificates.crt` in Dependency Graph | **Third-party container noise.** The message is emitted by the generated Dependabot action's OS certificate update and the run succeeds. It has no repository workflow source. | No unsafe suppression. Removing false project discovery prevents this generated job for the evidence tree. Search evidence is retained in `dependabot-ca-*.json`. |
| CodeQL sources contain deprecated-command and `::error` strings | **Non-emitted candidates.** These are action source/fixture strings in debug expansion, not check annotations or executed diagnostics. | Classified only; complete annotations and logs prove no matching emitted warning. |
| Cache restore misses, Codecov notices, skipped change-filtered jobs | **Informational/intentional.** A cache miss is valid cold-state behavior; Codecov reported successful processing; skips match no-change predicates. | No suppression and no change to pass/fail semantics. |
| npm 11 refuses a regenerated Kreuzberg lock | **Upstream package metadata defect discovered during repair.** `@kreuzberg/html-to-markdown-node@3.7.2` names two musl packages not published at that version. | Locks were generated and clean-installed with CI's npm 10 on glibc. Reported upstream as html-to-markdown #459; do not fabricate absent platform entries. |
| VSIX esbuild cannot resolve Playwright's `chromium-bidi/lib/cjs/...` imports | **Real fresh-CI packaging regression.** Puppeteer 25 supplies chromium-bidi 17, which removed those private paths; Playwright still carries optional Bidi-over-CDP imports. Playwright maintainers explicitly do not support bundling this server runtime. A source-only smoke check never exercised the actual dependency graph. | A package-time helper externalizes `playwright` and `playwright-core`; the VSIX allowlists those production packages. A new test bundles and loads the real desktop web-tools graph. The 720-file VSIX packages successfully and its extracted artifact launches the shipped Chromium and renders a local page. Existing Playwright issue #33031 is retained rather than duplicated. |
| Lint cannot import package-local `esbuild` | **Real fresh-CI test-boundary regression.** The dependency-backed bundle test was appended to `npm run vscode:test`, whose documented and CI contract is to read committed source without installing `vscode/node_modules`. Local verification had that directory from packaging, masking the missing prerequisite; the dedicated VSIX job installed dependencies and passed. | Restore the 51-case source-only suite and expose the bundle check as `test:package`. The VSIX job now runs it after `scripts/install-node-dependencies.sh vscode` and before packaging. The regression asserts that ordering, while an explicit run with `vscode/node_modules` moved aside proves the lint suite remains hermetic. Installing the full browser graph in the generic lint job was rejected as duplicated, heavyweight work. |
| macOS descendant survives a 20 ms agent timeout | **Real fresh-CI process-boundary race.** Formal AI reconstructed exact argv as a quoted shell string for command-stream 0.15. The added `/bin/sh` became the process-group leader; on the loaded Intel runner the timeout's one-shot signal raced the fixture executable/descendant joining that group. | Upgrade to command-stream 0.16 and call `StreamingRunner::from_argv`, preserving exact boundaries and making the allowlisted executable the group leader. The existing descendant behavioral test plus a source/API regression pin the correction; Windows retains its existing direct `std::process` path. |
| Full local suite reports stale `src/orchestration/runner.lino` | **True generated-artifact failure.** The exact-argv refactor changed the runner's source census, but the checked-in self-AST document still described the prior bytes and symbol lines. | Run the repository's `regenerate_self_ast_census` example, inspect the one-file mechanical diff, and rerun both the exact census regression and all-feature suite. |
| Packaged browser path ignored | **Upstream dependency-floor defect.** web-capture forwards `executablePath`, but its `browser-commander@^0.8.0` dependency discards it. | Scoped override pins the first forwarding release, 0.10.0. The extracted-VSIX test proves the packaged executable is used. Reported as web-capture #154. |
| Source terminology lint flags `/v1/knowledge-graph` inside archived PR JSON | **False positive caused by inconsistent historical-data scope.** The lint already excludes docs, experiments, caches, and frozen source corpora but omitted `dev/log`, where immutable third-party responses are required evidence rather than authored API surface. | Exclude `dev/log` consistently; the lint's own directory fixture now proves an in-scope source route still fails while the same text in captured evidence is ignored. |
| `REQUIREMENTS.md does not match docs/requirements/` | **True derived-artifact failure.** The issue shard was added without running the repository assembler. | Rebuild with the documented `rust-script scripts/assemble-requirements.rs --write` command and retain the gate check. |
| Desktop release source test expects command-stream 0.15 | **True regression-test drift, not a production dependency mismatch.** The source assertion was not updated with the intentional 0.16 exact-argv upgrade, so every release OS failed identically. | Match the pinned Cargo manifest at 0.16; all four focused command-runner tests pass. |
| Gemini logging-proxy request times out with `WouldBlock` | **True test-isolation defect.** Concurrent helper-owned server processes inherited one `$HOME/.formal-ai/memory.lino`; every response records memory under its advisory lock, so unrelated nextest cases could consume the entire 30-second HTTP budget waiting for shared state. | Allocate private memory and dialog-log paths per helper server, preserve explicit test overrides, delete scratch state on drop, and verify recording occurs only in that private store. |
| Concurrent proxy test loses its JSONL file | **True local test-artifact collision.** PID plus wall-clock nanoseconds did not guarantee unique filenames for same-process parallel tests. One test could remove another's log. | Add a process-local atomic sequence to the filename. All four logging-proxy cases pass concurrently. |

Every observed root cause is identified. The existing `FORMAL_AI_CI_VERBOSE`
backend tracing from #1012 remains disabled by default; no new speculative debug
mode was needed for #1014.

## 5. Current template and best-practice comparison

The comparison uses complete repository trees, not a shortlist of workflow
filenames:

| Source | Frozen revision | Relevant result |
| --- | --- | --- |
| Rust pipeline template | `56aa18ac041398afa037cec0da3cf5cae2553e07` | Dependency Review covers changes, but there is no required/scheduled RustSec audit of the current lock. Reported as template #132. |
| JS pipeline template | `77b8f1b520fde96f9a65a0fd7b5e5a5c9d1046d3` | Dependency Review covers changes, but there is no fail-closed audit of committed current npm locks. Reported as template #134. |
| Python pipeline template | `c3a2eb2eaaa5741c9ece6903c7675626d03e7ea3` | Dependency Review covers changes, but there is no pip-audit/OSV check of the resolved current environment. Reported as template #58. |
| Hive Mind CI/CD guidance | `44372fd6` | Supports pinned/reproducible dependencies, fail-fast security checks, isolated jobs, bounded execution, and avoiding duplicate work. The new lock gate and archive topology apply those practices. |

Dependency Review remains valuable for PR deltas, but it cannot detect an
advisory published later against an unchanged lock. The local gate addresses
that temporal false negative. The same gap was reported to all three templates
with ecosystem-specific commands rather than copied implementation.

## 6. Online research and existing components

- [cargo-nextest partitioning](https://nexte.st/docs/ci-features/partitioning/)
  defines `slice:M/N`; [nextest archiving](https://nexte.st/docs/ci-features/archiving/)
  supports compiling once and running an archive elsewhere with workspace
  remapping. Its relocation guidance requires runtime `NEXTEST_BIN_EXE_*` for
  new tests. The repository's existing compile-time callers remain compatible
  by extracting the archive at their original workspace path, as the retained
  minimal experiment proves. These are the native components used instead of
  a custom test scheduler.
- GitHub Dependency Review is retained for changed dependencies. Native
  `bun audit` and `npm audit --package-lock-only` cover current committed locks
  without adding another scanner.
- npm registry metadata captured in `raw-data/*versions.json` confirms the
  current Puppeteer 25 browser downloader no longer depends on the vulnerable
  `extract-zip` path.
- Playwright [#33031](https://github.com/microsoft/playwright/issues/33031)
  records the maintainers' position that Playwright is not bundle-ready. npm
  source/metadata comparisons show browser-commander 0.10.0 is the first
  release forwarding `channel` and `executablePath`.
- `opencode-ai@1.18.4` registry metadata identifies the package-owned
  `postinstall.mjs`; isolated red/green installs prove why this one pinned
  lifecycle must run.
- Gemini v0.55.1 source snapshots identify the exact color check, project
  registry lock, storage location, and tree walker involved in the warnings.
- Recent Kreuzberg workflow data and package metadata are preserved to
  distinguish the registry publication defect from this repository's lock.
- command-stream 0.16's published `StreamingRunner::from_argv` API is used in
  place of the 0.15 shell-string adapter. Its crates.io metadata is preserved
  in `raw-data/command-stream-0.16.0-crates-io.json`; the exact-argv API avoids
  introducing an extra shell into Unix process-group ownership.

Upstream reports filed from the retained bodies:

- Gemini CLI [#28826](https://github.com/google-gemini/gemini-cli/issues/28826)
- web-capture [#153](https://github.com/link-assistant/web-capture/issues/153)
- web-capture [#154](https://github.com/link-assistant/web-capture/issues/154)
- html-to-markdown [#459](https://github.com/xberg-io/html-to-markdown/issues/459)
- Rust template [#132](https://github.com/link-foundation/rust-ai-driven-development-pipeline-template/issues/132)
- JS template [#134](https://github.com/link-foundation/js-ai-driven-development-pipeline-template/issues/134)
- Python template [#58](https://github.com/link-foundation/python-ai-driven-development-pipeline-template/issues/58)

Each `upstream-reports/*.md` file includes a minimum reproduction, an immediate
workaround, and a suggested source-level correction.

## 7. Alternatives considered

| Requirement | Rejected alternative | Why | Selected plan |
| --- | --- | --- | --- |
| Release deadlock | Treat missing attribution as eligible or ignore Auto Release in Pipeline Status. | Both weaken a true release-integrity property. | Preserve strict manual enforcement; defer only the non-mutating automatic publisher. |
| macOS duration | Increase timeout, split into more cold jobs, or mechanically rewrite every historical binary invocation. | The first two preserve/multiply compilation; the third is broad unrelated test churn when archive extraction can preserve Cargo's established paths. | Compile one nextest archive, extract it into the original workspace, then fan out twelve bounded consumers based on exact five- and eight-way timing evidence. |
| JS advisories | Rely on install summaries, Dependabot, or `npm audit fix --force`. | Summaries do not fail; bots are asynchronous; force can make uncontrolled major upgrades. | Required audit gate plus reviewed direct/scoped updates and affected tests. |
| Gemini lock warning | Filter warning text. | Conceals a real upstream race and remains fragile. | Stop scanning mutable home; report race upstream. |
| Evidence dependency graphs | Disable Dependency Graph globally or delete evidence. | Loses a useful service or audit provenance. | Preserve bytes with non-manifest `.snapshot` basenames. |
| Bun lifecycle warning | Trust all scripts or disable all scripts. | The former expands install-time execution; the latter leaves OpenCode's wrong-platform placeholder installed. | Default to `--ignore-scripts`; trust only exact, lockfile-pinned OpenCode where the lifecycle is required. |
| VSIX bundling | Alias private chromium-bidi paths or mark only those imports external. | Relies on package internals; the resulting bundle still cannot resolve Playwright's package-relative metadata. | Follow Playwright guidance: externalize both runtime packages, ship them, and test the extracted artifact. |

## 8. Tests-first and verification record

`local-tests/regression-red.log` records the seven implementation-facing issue
tests failing 7/7 before the implementation: release semantics, archive reuse,
Gemini isolation, evidence manifests, complete JS audit coverage,
lifecycle/install noise, and durable case studies. `regression-red.status` is
`101`. Evidence-completeness tests were then added for every R1014 requirement
plus the composed whole task.

The green evidence includes:

- `regression-green.log`: the composed issue #1014 suite.
- `release-preflight.log` and `release-preflight-outputs.log`: the real current
  ineligible cycle exits zero and emits `should_release=false`.
- `javascript-dependency-audits.log`: root Bun, experiment Bun, tests/e2e,
  desktop, and VS Code audits all report zero vulnerabilities.
- `desktop-tests-node20.log`: 140/140 desktop tests pass.
- `vscode-tests-after-update.log`: the original 51/51 extension tests pass;
  the package-stage real-graph bundle test passes separately.
- `web-build.log`: the vendored web bundle regenerates successfully.
- `actionlint.log`: both workflow topology and expressions validate.
- `fresh-ci-regressions/nextest-archive-extract-to-workspace.log`: default
  temporary extraction fails with status 100 and workspace extraction passes.
- `fresh-ci-regressions/registered-rust-stage-after-command-stream-016-final.log`,
  `wasm-stage-final.log`, and `web-stage-final.log`: the exact registered CI
  gate stages pass together, including strict Clippy, examples, Rustdoc,
  ShellCheck, generated files, and repository policy checks.
- `fresh-ci-regressions/full-rust-tests-after-command-stream-016.log`: the
  first exhaustive run reproduces the stale runner census only after 2,805
  other unit tests pass; `regenerate-self-ast-census.log` and
  `self-ast-census-green.log` retain the mechanical repair and focused proof.
- `fresh-ci-regressions/full-rust-tests-after-exact-ci-corrections-red.log`:
  the first complete rerun found two older topology contracts still hardcoded
  to five macOS consumers after 3,754 other tests passed. Updating those
  codebase-wide contracts to eight makes the focused topology suite pass
  10/10.
- `fresh-ci-regressions/macos-portability-exact-topology-green.log`: final
  source review found one additional permissive portability helper that only
  required the first five slices and therefore passed an eight-slice workflow.
  It was tightened to require the exact eight-slice intermediate topology.
- `fresh-ci-regressions/full-rust-tests-after-exact-ci-corrections-final.log`:
  all 3,756 tests across the all-feature harnesses pass, including 2,806 in
  the largest unit harness; four tests are intentionally ignored and doctests
  pass.
- `fresh-ci-regressions/desktop-tests-final.log` and
  `vscode-tests-final.log`: the affected suites pass 140/140 and 52/52 before
  the source/package boundary is separated; the final boundary evidence is in
  `vscode-source-without-node-modules-green.log` and
  `vscode-package-graph-test-green.log` (51 source cases plus one real graph).
- `fresh-ci-regressions/javascript-audits-final.log`: dynamically discovered
  audits of all five tracked JavaScript locks pass with zero advisories at the
  configured moderate threshold.
- `fresh-ci-regressions/macos-fanout-and-exact-argv-red.log`: both final
  macOS workflow/process boundaries fail before their implementation; the
  corresponding `-green.log` passes 14/14 issue checks.
- `fresh-ci-regressions/descendant-timeout-command-stream-016-stress.log`:
  the exact 20 ms descendant-termination regression passes ten consecutive
  runs with command-stream 0.16.
- `fresh-ci-regressions/macos-eight-way-fanout-{red,green}.log`: the final
  intermediate topology test fails against five consumers and passes against
  eight, while the workflow's original timeout bounds remain unchanged.
- `fresh-ci-regressions/macos-twelve-way-fanout-{red,green}.log`: the exact
  eight-way GitHub outlier is converted into a failing 12-way contract before
  the workflow is repartitioned, still without changing either timeout.
- `fresh-ci-regressions/desktop-command-stream-manifest-green.log`: all four
  production command-adapter checks pass with the synchronized 0.16 assertion.
- `fresh-ci-regressions/private-server-state-green.log` and
  `logging-proxy-parallel-final.log`: private helper recording passes, and all
  four proxy protocols pass concurrently after log-path isolation.

The repository-wide Rust checks also pass: strict all-feature Clippy, all
examples, 3,756 all-feature tests across the registered binaries, and doc
tests. The final-code evidence is
`registered-clippy-after-exact-ci-corrections.log` and
`examples-check-after-exact-ci-corrections.log`. For completeness,
`clippy-after-exact-ci-corrections.log` preserves a broader ad hoc
`--all-targets` probe: it reports only two `assigning_clones` nursery
diagnostics in the pre-existing issue-933 example, which is outside the
repository's registered Clippy scope; the separately registered Clippy and
all-example compilation commands both pass. The final issue contract passes
14/14 in `issue-1014-final-after-evidence.log`.
Fresh GitHub run IDs are added here only after they execute against the final
#1015 implementation head; stale green checks are never presented as final
evidence.
