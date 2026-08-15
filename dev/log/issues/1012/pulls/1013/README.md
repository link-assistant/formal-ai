# Issue #1012 / pull request #1013 CI/CD audit

Issue: <https://github.com/link-assistant/formal-ai/issues/1012>

Pull request: <https://github.com/link-assistant/formal-ai/pull/1013>

This is the canonical evidence bundle for the complete CI/CD diagnostic audit.
It preserves more than 760 source and evidence files: every
log from the ten default-branch workflows named by the issue, all five initial
pull-request workflow logs, GitHub run/job/check metadata, all three PR comment
surfaces, full repository trees and source snapshots for the three reference
templates, the Hive Mind guide, upstream issue data, and local red/green tests.

## Evidence map

| Path | Contents |
| --- | --- |
| `ci-logs/main-run-*.log` | Complete logs for all ten issue-listed default-branch runs. |
| `ci-logs/initial-pr/` | Complete logs for the five workflows on prepared PR head `d4e8488`. |
| `raw-data/main-run-*.json` and `raw-data/main-run-jobs/` | Run and per-job timestamps, conclusions, steps, and head SHAs. |
| `raw-data/check-annotations/` | GitHub check annotations, retained separately from textual logs. |
| `raw-data/issue-*` and `raw-data/pr-*` | Issue, PR, timelines, files, commits, comments, reviews, initial diff and status. |
| `references/templates/` | Full source snapshots of the current Rust, JavaScript and Python templates. |
| `raw-data/*-tree.json` | Complete Git trees, including every workflow and CI/CD script path. |
| `raw-data/CI-CD-BEST-PRACTICES.md` | Hive Mind guidance at audited revision `44372fd`. |
| `raw-data/upstream-research/` | Primary-source action, VS Code, Gemini CLI and sccache research. |
| `local-tests/` | The issue regression before the fix, focused green result, and affected suites. |

## Timeline reconstructed from independent timestamps

All times are UTC on 2026-08-15.

1. `07:16`: pull request #1011 merged as `ac6e24d5955a9df6a8ad3b8e24db33eb9c608a4d`.
2. `07:16:55`: the ten default-branch workflow runs were created for that SHA.
3. `07:20:17`: the Intel macOS core test began executing after its cold build.
4. `07:20:55`: seven parallel Box jobs reached the same cache service; the
   Rust leg recorded HTTP 429 cache restoration warnings at log lines
   35762-35763. Every Box leg then compiled the same release binary.
5. `07:54:05`: the macOS core lane was still reporting passing tests when its
   35-minute job limit killed it (`main-run-31871548846.log:30775-30778`).
6. `07:54:38`: Pipeline Status correctly converted the otherwise ambiguous
   cancelled result into a failure and named `test` as the cancelled job
   (`:41622-41625`).
7. `07:54:43`: Desktop Release was triggered from the failed workflow and
   correctly skipped because its upstream precondition was not met.
8. `08:10:11`: issue #1012 was opened with the ten-run snapshot.
9. `08:11:09`: the prepared branch commit `d4e8488` was pushed.
10. `08:11:19`: draft PR #1013 was opened. Its five initial workflows all
    completed successfully, but reproduced the same successful-run warning
    debt, proving that green conclusions alone were not an adequate audit.
11. `08:32`: exact shared `download-artifact@v8` defects were reported to the
    Rust and JavaScript templates as issues 131 and 133.

## Requirements inventory

| ID | Requirement | Evidence and disposition |
| --- | --- | --- |
| R1012-1 | Download and inspect all ten listed runs, including warnings, errors, false positives, false negatives, jobs, timestamps, and matching SHAs. | Complete under `ci-logs/` and `raw-data/`; every run is for `ac6e24d`, and the later Desktop skip is causally linked to run 31871548846. |
| R1012-2 | Fix every actionable CI/CD diagnostic everywhere it occurs, without hiding true failures. | The finding table below maps every class to a narrow fix or an evidence-backed informational classification. Pipeline Status remains fatal. |
| R1012-3 | Compare the complete workflow/CI script tree with current Rust, JS/TS, and Python templates. | Full trees and source snapshots are archived; all 32 Rust, 42 JS and 18 Python workflow/script files were inventoried and searched, not only similarly named files. |
| R1012-4 | Apply relevant template and Hive Mind CI/CD practices. | Preserved status aggregation, permissions, pinned actions, safe concurrency and retry boundaries; added bounded sharding, live budget telemetry, shared immutable build output and focused diagnostic policy. |
| R1012-5 | Report defects that also exist in templates. | Filed Rust template #131 and JS template #133 with reproduction, workaround, and source fix. Python has no affected v8 download step. Existing actions/download-artifact#484 and microsoft/vscode#319867 were reused rather than duplicated. |
| R1012-6 | Reproduce problems with automated tests before fixing them and verify the composition. | `local-tests/regression-red.log` records eight failures before implementation. A ninth invariant pins the later sccache finding; a tenth verifies the required Formal AI / real Agent CLI self-authorship evidence. The final issue suite passes 10/10 and CI/CD passes 207/207. |
| R1012-7 | Preserve research, reconstruct the sequence, identify root causes, alternatives, known components and per-requirement plans. | This document plus the primary-source archive provide the durable analysis. |
| R1012-8 | If evidence cannot establish an exact root cause, add verbose diagnostics disabled by default. | `FORMAL_AI_CI_VERBOSE` defaults to `false`; setting the repository variable to `true` enables millisecond sccache backend debug logs through the shared action. |
| R1012-9 | Apply requirements across the whole codebase and finish in the single prepared PR. | All v8 download steps and all affected agent-matrix consumers are covered by enumerating regressions; PR #1013 is the sole delivery vehicle. |

## Finding-by-finding root cause analysis

| Finding | Classification and root cause | Implemented solution |
| --- | --- | --- |
| CI/CD Pipeline cancellation | **True failure.** The cold Intel macOS `core` lane remained healthy but monolithic until the 35-minute limit. Its `TEST_BUDGET_SECONDS=2100` equalled the complete job budget and its warning was calculated only after `cargo test` returned, so a timeout could never emit it. | Keep a smaller 25-minute job cap; run three complementary cargo-nextest `slice:m/3` core lanes plus the specification lane; run a watchdog that warns live at 70% of a 1,200-second execution budget. |
| Pipeline Status errors | **True positive.** GitHub represented the timeout as `cancelled`; the aggregator deliberately detected this and failed main. | Preserve unchanged and include the new shared Box build in its required-job set. |
| Desktop Release skipped | **Expected consequence**, not a failure or false negative. Its upstream CI requirement was false. | No relaxation. A green parent run will exercise it. |
| Stock Rust `error: package ID ... openssl-sys` | **False error-shaped success.** `cargo tree -i` exits nonzero when the intentionally absent package is queried; shell negation made the job green but retained Cargo's error text. | Capture a quiet full package list and reject an exact `openssl-sys` line. Install into a job-level root already on `PATH`, verify with `command -v`/`ldd`, and invoke bare `formal-ai`. This also removes Cargo's PATH warning. |
| Fourteen `DEP0005 Buffer()` warnings | **Third-party warning.** `actions/download-artifact@v8` bundles the deprecated call; upstream issue actions/download-artifact#484 tracks it. | Put `NODE_OPTIONS=--disable-warning=DEP0005` only on every affected v8 step. Other Node warnings remain visible. |
| VS Code `DEP0169 url.parse()` warning | **Third-party warning.** VS Code CLI installation emits its known diagnostic, tracked by microsoft/vscode#319867. | Scope `--disable-warning=DEP0169` to the single `code --install-extension` process and document the upstream removal condition. |
| Seven `Broken pipe` lines in the agent matrix | **Harness defects masked by success.** `grep -q` exited as soon as it matched and closed the producer's process substitution, so `sed` failed; TUI consumers could also close stdin before scripted keystrokes completed. | Consume the entire search stream into `/dev/null`; treat EPIPE only on best-effort TUI input as an already-closed successful consumer. Test all three search forms and both input writes. |
| Gemini ripgrep, numerical-classifier JSON, and terminal-color errors | **Configuration mismatch with fallback.** The E2E sandbox has no `rg`; the automatic model router requested a structured classifier response from a compatibility endpoint that returns normal model text; the PTY lacked a declared color terminal. Gemini caught these and continued, leaving alarming green logs. | Select the explicit `formal-ai` model, disable `useRipgrep`, and export `TERM=xterm-256color` in this isolated test. Keep its real tool/model behavior covered. |
| Codex temporary-directory PATH-alias warnings | **Product harness warning.** Fake HOME and `.codex` were rooted directly below `/tmp`, which Codex intentionally distrusts for helper aliases. | Allocate ephemeral client homes below the real home cache directory and retain automatic cleanup. An integration regression asserts that Codex HOME is no longer under the OS temp directory. |
| `src/solver.rs` 956-line warning | **True maintainability warning.** Configuration enums and their parsers shared a near-limit implementation file with solving behavior. | Extract those cohesive public types to `solver_config.rs`, re-export the unchanged API from `solver`, and reduce `solver.rs` to 861 lines. |
| Box cache HTTP 429 and 1,131 cache-write errors | **True infrastructure pressure and hidden degradation.** Seven simultaneous jobs restored cache and compiled the identical host binary. The action log proves an HTTP 429; sccache's post step reports counters but not backend responses, so the exact cause of every write error is not provable. | Build and upload the release binary once, then download it in seven language-only legs. This removes six redundant compiles and cache interactions. Add disabled-by-default sccache backend logging for any residual write failures. |
| sccache `Compilation failures` | **Misleading statistic, not failed CI.** These are compiler invocations sccache could not cache (including probes); the commands and jobs succeeded, and the action emits a notice rather than a failure. | Do not suppress or convert the statistic. The opt-in backend log remains available if the counter changes alongside a real failure. |
| CodeQL `expect-error` / deprecated schema strings | **Source/test vocabulary, not emitted diagnostics.** Searches hit analyzer fixtures and schemas in successful CodeQL output. | No change; retain coverage. |
| Coverage action source containing `::error` | **Downloaded script text, not an annotation.** No corresponding check annotation or failed command exists. | No change. |
| Codecov notice | **Informational optional integration.** Coverage generation and artifact upload succeeded; no required token or failed gate was present. | No change to security or coverage enforcement. |

No reviewed real test assertion failed before the timeout. Conversely, the
successful jobs contained reproducible error-shaped output and a masked cache
429, so treating workflow conclusion alone as truth would have been a false
negative.

## Complete template comparison

The comparison used immutable current revisions, with repository metadata and
complete Git trees beside the extracted sources:

| Template | Revision | Workflow + script files | Result |
| --- | --- | ---: | --- |
| Rust | `56aa18ac` | 32 | Same unsuppressed `download-artifact@v8` warning in both desktop finalize downloads; reported as #131. Its release/status, security, links, size, changelog, retry and fresh-merge patterns were already represented in Formal AI. |
| JavaScript/TypeScript | `77b8f1b` | 42 | Same unsuppressed v8 warning in the Docker manifest download; reported as #133. Its release/status, security, links, Docker build, changeset and fresh-merge patterns were reviewed. |
| Python | `c3a2eb2` | 18 | No affected v8 download step and no matching defect. Its docs, release/status, security, links, size, changelog and publication scripts added no applicable missing gate. |

The whole `.github` and `scripts` subtrees were compared. Language-specific
publishing components were not copied into a Rust application merely to make
the trees look alike. The Hive Mind guidance similarly favors explicit
timeouts, least privilege, pinned actions, safe concurrency, deterministic
status aggregation, bounded retries and artifact reuse. The changes here
strengthen those existing boundaries without weakening any gate.

## Online research and reusable components

- [cargo-nextest partitioning](https://nexte.st/docs/ci-features/partitioning/)
  provides stable `slice:m/n` sharding after filters and is purpose-built for
  long CI suites. Its actual 0.9.143 binary accepted this repository's filter
  expression locally; the installer action avoids building it from source on
  macOS.
- [actions/download-artifact#484](https://github.com/actions/download-artifact/issues/484)
  is the canonical action-owned DEP0005 report. A narrow Node diagnostic code
  is preferable to `--no-warnings`.
- [VS Code #319867](https://github.com/microsoft/vscode/issues/319867) tracks
  the exact CLI DEP0169 emission; the workaround is isolated to that child.
- [sccache GHA documentation](https://github.com/mozilla/sccache/blob/main/docs/GHA.md)
  explains that storage may be skipped on service rate limits, and
  [configuration docs](https://github.com/mozilla/sccache/blob/main/docs/Configuration.md)
  define `SCCACHE_LOG` and millisecond logging. This supports the opt-in trace
  rather than guessing about counter-only evidence.
- Gemini CLI's current settings schema, model router implementation, numerical
  classifier, and ripgrep setting are preserved under
  `raw-data/upstream-research/gemini-cli/` from the upstream repository.
- GitHub Actions artifacts are used for the immutable shared Box binary, an
  existing repository convention that avoids adding a new cache/library.

## Alternatives and solution plans

1. **Timeout:** raising 35 minutes again would postpone the symptom; removing
   macOS would lose portability. Selected plan: keep a stricter bound and shard
   the exhaustive core set with a component designed for CI partitioning.
2. **Budget telemetry:** post-process elapsed logging cannot survive a killed
   process. Selected plan: a small watchdog wrapper with cleanup and original
   exit-status propagation.
3. **Node warnings:** workflow-wide `--no-warnings` hides unrelated regressions;
   downgrading actions would discard supported behavior. Selected plan: one
   diagnostic code on one third-party process, with upstream issue links.
4. **Agent pipes:** redirecting all stderr would conceal real CLI failures.
   Selected plan: remove the premature consumer and tolerate EPIPE only where
   a successfully closed TUI input is expected.
5. **Box jobs:** unique cache keys would still compile seven identical binaries;
   serializing jobs would lengthen CI. Selected plan: compile once and fan out
   the immutable artifact while preserving parallel language validation.
6. **sccache:** treating every write counter as fatal would make optional cache
   availability break correctness. Selected plan: reduce backend load, retain
   counters, and expose opt-in details for the next residual occurrence.
7. **Large source:** suppressing or raising the line warning would erase its
   preventive value. Selected plan: extract a cohesive configuration module
   and preserve public re-exports.

## Upstream reports

- Rust template: <https://github.com/link-foundation/rust-ai-driven-development-pipeline-template/issues/131>
- JavaScript template: <https://github.com/link-foundation/js-ai-driven-development-pipeline-template/issues/133>
- Artifact action canonical issue: <https://github.com/actions/download-artifact/issues/484>
- VS Code canonical issue: <https://github.com/microsoft/vscode/issues/319867>

Both new reports contain an exact reproduction, a narrow workaround, and a
suggested workflow regression. No duplicate Python, action, VS Code, Gemini or
sccache report was filed where an exact template occurrence was absent or an
upstream report already existed.

## Tests and verification record

- `local-tests/regression-red.log`: the eight initial invariants fail against
  the prepared branch before implementation.
- `local-tests/regression-focused-green.log`: the first nine final regressions
  pass before the self-hosting invariant was added.
- `local-tests/agent-cli-self-hosting.log` and
  `local-tests/agent-cli-template-self-hosting.log`: two successful live
  Agent CLI runs authored differently phrased requirement leaves. Their
  durable client/server/session bundles are under
  `docs/case-studies/issue-1012/self-hosting-authorship/`.
- `local-tests/affected-green.log`: CI/CD 206/206 before the tenth regression,
  issue #988 4/4, issue #961 12/12, and Codex model-metadata integration 1/1
  pass.
- `local-tests/regression-self-hosting-green.log`: the final issue suite passes
  10/10, including deterministic replay of both self-authored leaves.
- `local-tests/ci-cd-final-green.log`: the complete CI/CD module passes 207/207.
- `local-tests/cargo-test.log`: the first full run correctly found only the two
  stale self-AST census expectations introduced by the new `solver_config.rs`
  module (2,788 other tests passed). The canonical census generator then
  created the missing document and refreshed its four dependent indices.
- `local-tests/cargo-test-green.log`: the corrected full run passes 2,790
  tests, with zero failures and four explicitly ignored network benchmarks;
  doc tests also pass.
- The downloaded cargo-nextest 0.9.143 binary was additionally used to compile
  and validate the exact partition/filter command locally.

Final full-suite, lint, formatting, documentation, actionlint and remote-run
results are appended to this archive after the final clean commit so their
reported SHA cannot become stale.
