# Issue 1580: CI failures, warnings and result integrity

Work in progress in [PR #1581](https://github.com/link-assistant/formal-ai/pull/1581), addressing [issue #1580](https://github.com/link-assistant/formal-ai/issues/1580). This record distinguishes source fixes, executed checks and unresolved acceptance criteria. It is not a completion claim.

## Reproduction and evidence

The issue's workflow table, every issue/PR conversation comment, inline review comments and reviews were read. The interrupted session was recovered from its authenticated Gists; its five commits were preserved, and main was merged forward. The audited main source is `b68d21647e678cce91ec83d5a28ea86b481064bf`; the source checkpoint for this report is `22ffaa73d1b2b0dc800ed2f9093059ac80ddef56`.

`experiments/issue-1580/evidence/ci/all-cited-and-latest.json` indexes 71 cited/latest workflow runs with SHA, creation time, conclusion, log line count and SHA-256. The corresponding `.log.gz` files preserve the complete available logs. Annotation JSONL files contain original log line numbers. `summary.json` adds the failed-job inventory. The graph-update run 31914675335 returned HTTP 410 for its expired log; its `LOG_UNAVAILABLE` record is missing evidence, despite the historical success conclusion.

Important production failures, with original log lines:

| Run | Observation | Current disposition |
| --- | --- | --- |
| [43815999666](https://github.com/link-assistant/formal-ai/actions/runs/43815999666) | 31 failing native tests; full shards 3/5/8 exhausted the unchanged 1,080-second budget; shard 5 termination at line 60051 | Source fixes cover workflow parsing/retry, fixture receipts, composition/constraints and debug-session expectations. Native execution and remaining benchmark/timing defects are outstanding. |
| [43805800653](https://github.com/link-assistant/formal-ai/actions/runs/43805800653) | Browser worker budget and JavaScript regressions; shard 8 consumed 317 of 360 seconds at line 26315 | Recovered worker/runner fixes plus production composition and candidate-validation regressions; remaining shard headroom needs measurement. |
| [43899940611](https://github.com/link-assistant/formal-ai/actions/runs/43899940611) | Draft exited 5 after 8m32s: serving identity unverified, null Hive attempt identity, no useful source changes | Live identity contract remains blocked on [Hive Mind #3909](https://github.com/link-assistant/hive-mind/issues/3909). No identity, delivery or benchmark credit is fabricated. |
| [43878821415](https://github.com/link-assistant/formal-ai/actions/runs/43878821415) | File-size warning inventory, lines 3783–4023 | Solver reduced below its warning band. Other warning-band files remain open; limits were not globally raised. |
| [43872809507](https://github.com/link-assistant/formal-ai/actions/runs/43872809507) | Missing required main-branch contexts, line 784 | PR contexts now report on documentation changes. Repository rule configuration and machine-writer compatibility remain open. |
| [43870207134](https://github.com/link-assistant/formal-ai/actions/runs/43870207134) | HumanEval 12/20, MBPP 11/20 and GSM8K 1/2 below existing floors | Real capability defects reproduced with held-out native probes; floors unchanged. Concrete-output composition fix removes a false success but does not implement the missing algorithms. |
| [43815999458](https://github.com/link-assistant/formal-ai/actions/runs/43815999458) | Compiler cache: 88 attempted writes, 87 errors, one stored, line 1113 | Warning no longer invents a cause or asserts certain recompilation; pretty-printed counters are recognized. Backend error cause still needs retained diagnostics. |
| [41768761745](https://github.com/link-assistant/formal-ai/actions/runs/41768761745) | All 12 work-item qualification containers failed to pull | Public log lacks the underlying Docker stderr; inaccessible evidence prevents a verified root-cause claim. Qualification is not passed. |

Successful scope-gated release consumers and the issue-1028 ladder are not evidence that their heavy paths executed. In particular, the successful ladder runs had `FULL_LADDER=false`. Old failures already repaired by newer main commits are kept as historical observations, not attributed to this branch.

## Implemented changes and focused verification

| Requirement/failure | Implementation | Verification |
| --- | --- | --- |
| Unknown quoted concept and described-file safety | Preserve path authority, terminal punctuation and source order; do not fetch an unsupported quoted concept or write an inferred file | `rust/tests/web/pull-request-1188-fix-gaps.test.mjs`: 31 passing tests, including G102; source punctuation has held-out cross-language cases |
| Prime constraints and concrete output | Preserve a validated candidate's intent, reuse existing prime validation before Spanish conjunction parsing, prohibit joining prose as code/constraint success | `worker-candidate-validation.test.mjs`: four tests across five languages; `issue-1580-production-composition.test.mjs`: three positive/negative controls |
| Parent result validity | Validate child answers only when the selected composition actually consumes them | JavaScript controls and all eight isolated native executor cases pass on `6a9aa198`; broader parent/passage binding remains unfinished |
| Real work-item receipts | Native fixtures obtain source-probe/write receipts from the actual workspace implementation; compiler/network effects remain explicit mocks | `connector-work-item-receipts.test.mjs`: 20 tests; `recipe-resume-policy-notation.test.mjs`: two tests; native fixture execution remains outstanding |
| Required context reporting | Documentation-only PRs report the same native context without compiling; missing/failed change detection conservatively runs the compiler | `default-branch-build-policy.test.mjs`: 21 tests; `stale-pull-request-report.test.mjs`: six tests; canonical staged-release projection verified |
| Download/YAML/runner regressions | Recovered bounded retries, flow-YAML timeout parsing and pinned browser runner fixes | Local JavaScript gates pass; full native regression execution remains outstanding |
| Draft resource/result accounting | Recovered occupancy handling, image pre-pull outside the solving allowance, default-off debug and bounded duration-collector retries | Dedicated recovered workflow tests are retained with their actual results under `evidence/checks`; live drafting still fails its identity acceptance criterion |
| Cache warning accuracy | Parse compact and pretty JSON; report measured counters and causal uncertainty | `sccache-outage-fallback.test.mjs`: twelve passing tests, including actual health-check and budget-wrapper subprocesses |
| Shared template transport defect | Real curl server reproduces truncated HTTP 200 transfer without retry; `--retry-all-errors` retries and succeeds | `template-download-retry.test.mjs`: one passing bounded regression; reported as [C++ template #35](https://github.com/link-foundation/cpp-ai-driven-development-pipeline-template/issues/35) |

At the checkpoint, the full local JavaScript gate run reported **86 passed, 52 skipped, zero failed** (138 total). Skips explicitly include Rust builds/tests and checks requiring hosted context or artifacts. Full transcripts and hashes are under `experiments/issue-1580/evidence/checks/`. Required Rust formatting and maintained source/projection currency checks passed; Rust compilation was intentionally left to adequately provisioned CI in accordance with repository policy. This container's 2.92 GiB limit is not an adequate reason to mislabel skipped native checks as passes.

The six-line increase in the JavaScript quantity worker is recorded in its individual module allowance with a concrete rationale: invoke the already-existing prime validator before arithmetic interpretation, matching the Rust path. It adds no prime algorithm and does not relax the global worker target. The solver module was split along its executor boundary rather than changing its size limit.

## Hosted CI follow-up on 17 November

All 24 runs on `dd334ef6a8d5bbef80795b6721cca9b706ad9447` completed. The three failed workflows and their complete logs are retained under `evidence/ci/dd334ef6a/`; the other runs must not be treated as proof that scope-gated heavy paths executed.

- Documentation policy [44059368525](https://github.com/link-assistant/formal-ai/actions/runs/44059368525), lines 202 and 2544–2610: the old policy still required a PR path filter that suppressed required contexts. The policy now exempts only the release workflow's PR trigger; its heavy main push stays filtered. The 32-test regression executes retained job/step conditions and rejects build admission, forced compilation, restored PR filters and an unguarded compiler step.
- CI/CD Pipeline [44059369287](https://github.com/link-assistant/formal-ai/actions/runs/44059369287), lines 8189–8198: clippy rejected an unused `intent` at `rust/src/solver.rs:605`. The redundant binding is removed; the validated candidate still supplies its own intent. The later pipeline run [44064714244](https://github.com/link-assistant/formal-ai/actions/runs/44064714244) passed its native clippy check. This is type/lint evidence, not native execution evidence.
- Layered CI [44059368522](https://github.com/link-assistant/formal-ai/actions/runs/44059368522): group 1 repeated the documentation-policy mismatch; group 2 detected stale source fingerprints in the governed Unicode witness (lines 3919, 4170 and 4191). The explicit reviewed-baseline refresh changes 20 hash leaves and preserves all 221 complete cases and assertions; the reader is unchanged. The full object diff is retained in `checks/governed-baseline-review.json`. Fixture integrity (2 tests), Unicode specification (34), build policy (21) and stale-PR reporting (6) pass locally.

The two hosted JavaScript groups consumed 259 and 168 seconds of their unchanged 360-second allowances. These failures were policy/source-witness mismatches, not shard timeouts. This does not verify native shard headroom or the unexecuted external benchmarks.

The latest local gate invocation reached its 600-second command-wrapper limit without a reported gate failure. Its uncompleted remainder was then run separately: all 138 registered gates are accounted for, with 86 passes, 52 explicit skips and zero failures. Both logs and a complete outcome map are retained; the interrupted invocation is recorded as incomplete rather than silently relabelled a pass.

## Git observation, template evidence and the next audit blocker

All 24 workflows on `510fe7606b2406de1037eccbb5bab37a6f93b4a5` completed: 23 succeeded and Security [44064713318](https://github.com/link-assistant/formal-ai/actions/runs/44064713318) failed. The three preceding failed workflows are now green. Metadata and the complete Security log are retained under `evidence/ci/510fe7606/`; successful scope-gated workflows still grant no heavy-path execution credit.

The Security log identifies Chrono 0.4.45 and RUSTSEC-2026-0543 at lines 742–748, with the denied warning and exit at lines 741 and 750. This is an unmaintained notice with no patched release, not a claim of an exploitable defect. Both direct dependency paths still require Chrono in their latest published releases (command-stream 1.13.0 and link-calculator 0.24.0); retained registry responses are under `evidence/chrono-upstream/`. The audit configuration records this under the existing exact-version `blocked-upstream` policy and the maintainer's deprecation report. Three subprocess regressions execute the actual audit shell script: reachable exact-version proofs retain `--deny warnings`, removing Chrono invalidates the exception, missing proofs fail, and unrelated audit failures propagate. Removing Chrono remains unfinished; the exception does not implement or verify a migration.

The local Rust change detector had the same failed-Git boundary found in several templates: an unsuccessful command returned an empty string and could emit successful all-false outputs. Git process/status/UTF-8 errors and output-write failures now propagate. Only a genuinely initial commit can use a tree fallback; incomplete PR merge observations fail instead of narrowing to a last-commit comparison. Parent counting stops at the commit header boundary. The embedded native regression uses a disposable real repository for non-repository, initial commit, forged message header, successful empty diff, missing before commit, incomplete PR, and unavailable parent controls. Rust formatting passes; this new native unit has not executed locally and is left for CI under the existing policy.

The maintained template observation entrypoint passes **nine tests** locally: two full-document/hash inventory controls, four Go workflow/detector regressions, and three actual Java/Python/PHP detector subprocess reproductions. All three language runtimes ran without skips. Unmodified fixtures preserve Git blob and SHA-256 identities and original licenses. Go's manual-release finding is an explicitly bounded source-grounded workflow model plus the actual terminal helper, not a hosted publishing run; its first-commit detector case also uses a real Git repository. Java, Python and PHP cases capture actual `GITHUB_OUTPUT` files and distinguish unavailable history from successful docs/empty controls. The five upstream report bodies are retained under `upstream/`; filing status is recorded in `upstream/filed-issues.txt` and the PR description.

The subsequent full invocation accounted for all 138 gates: 85 passed, 52 explicitly skipped, and shell lint failed because ShellCheck was absent from `PATH`. Rechecking that one gate with the verified ShellCheck 0.11.0 binary passed. Combined outcomes are **86 passed, 52 explicit skips, zero unresolved failures**. `checks/all-local-gates-fifth-summary.json` preserves the initial exit 1 and every original outcome, the separate successful recheck, and hashes of the full gate/focused logs. The shared fast-PR inventory now includes the audit-proof and template-observation entrypoints. Skipped checks are not counted as execution passes.

## CommonJS inventory and evidence-size repair

All 24 workflows on `71e1dee442fd74589df20a1d6f33710b6dec0d18` completed: 22 succeeded and two failed. Full logs and metadata are retained under `evidence/ci/71e1dee44/`. Layered CI [44071443786](https://github.com/link-assistant/formal-ai/actions/runs/44071443786), lines 7931–7934, rejects the named ESM `parse` import from CI's CommonJS `yaml` package. The inventory generator now loads that package through `createRequire`, matching existing tooling. A disposable dynamically exported CommonJS fixture reproduces the exact import error before the fix and passes afterward. The maintained template entrypoint now passes ten tests locally without runtime skips.

CI/CD Pipeline [44071444011](https://github.com/link-assistant/formal-ai/actions/runs/44071444011), lines 1136–1143, rejects two saved registry responses at 4,005 and 6,302 lines. They are now compressed evidence with both compressed and original SHA-256 identities; decompression is checked byte-for-byte against the previously committed responses. Both actual file-size gates pass, with global limits unchanged.

The initial size-gate invocation copied a stale documented spelling, `check_file_size`; the runner returned success after selecting zero gates. That invocation receives no verification credit. The runner now rejects unknown gate names, empty selections and unmatched filters before executing any selected gate. Its three new subprocess regressions fail before the fix (two failures, one valid-skip control) and pass afterward; the complete file passes eleven tests. Working-rules and push-gate regressions pass five and six tests. `checks/ci-repair-sixth.json` indexes every full before/after log.

Security [44071443716](https://github.com/link-assistant/formal-ai/actions/runs/44071443716) actually executes the upstream proof and audit: lines 735–741 prove the exact blocked dependencies and scan 555 main-lock dependencies. This verifies the guarded exception, not Chrono removal. The experimental lock still emits its existing allowed unsafe-libyaml unmaintained warning (lines 747–754); that warning remains an unresolved dependency migration rather than a newly hidden result.

## Go source and integrity checkpoint

All 42 pinned Go template files have now been read, compared by role/behavior and recorded in [the per-file review](templates/go-semantic-review.json). Seven complete template semantic reviews remain open. The maintained template entrypoint passes eleven tests without runtime skips. Its added combined stale-version/bad-pin case reproduces an integrity check bypass on non-PR events, with current-pin failure and freshness-only warning controls. A one-line candidate repair is tested only in a disposable fixture copy; no remote template repair or hosted publication is claimed. The report is filed as [Go template #20](https://github.com/link-foundation/go-ai-driven-development-pipeline-template/issues/20) with the published executable regression at `a4053def4bd4b921001fdf84a026ae35deebfc39`; no upstream repair is claimed. Full regression logs and hashes are retained in `checks/go-pin-integrity.json`.

## Retained cache backend diagnostics

An opt-in logging rerun previously wrote the server response to a runner-local file that the budget wrapper's health check never retained in job output. Two behavioral regressions reproduce the missing evidence before the fix (nine passes, two failures). The health check now retains the last 64 KiB and at most 80 lines whenever existing `FORMAL_AI_CI_VERBOSE=true` logging is enabled, including when statistics are unavailable. URLs are redacted; a fresh 128-bit stop/resume token suspends workflow commands while printing backend text. Default mode remains quiet.

Twelve tests pass afterward, including actual wrapper success/failure outcomes, output bounds, missing logs, token renewal and unchanged warning counts. The verified ShellCheck binary and all five touched gates pass. Complete logs and identities are in `checks/cache-log-retention.json`. This repairs evidence retention; no new hosted backend response has been observed and the historical cache failure cause remains unresolved.

## Full template comparison

The [template source manifest](templates/sources.json) pins all eight complete source trees. Every one of their 3,007 blobs was downloaded and verified against its Git blob identity; the 48 workflows have parsed run bodies in the per-language workflow JSONL. Each full-tree TSV records every file's role, byte count, Git blob and same-name local comparison. CI candidates outside `.github`, including scripts, test/configuration files and package/build metadata, are inventoried separately.

This is a complete file inventory, not a claim that same-name matching establishes semantic equivalence. The remaining transitive CI reference review and policy decisions are tracked in [the template review](templates/README.md). Application source and historical evidence are accounted for instead of silently omitted.

Upstream reports are preserved under `experiments/issue-1580/upstream/`: [Hive Mind #4071](https://github.com/link-assistant/hive-mind/issues/4071), [Hive Mind #4072](https://github.com/link-assistant/hive-mind/issues/4072), and [C++ template #35](https://github.com/link-foundation/cpp-ai-driven-development-pipeline-template/issues/35). Reporting them does not satisfy unimplemented local acceptance criteria.

## Provenance and remaining acceptance criteria

Development commands execute through the JavaScript Formal AI driver. The externally supplied orchestration, literal commands, patches and generators receive **zero autonomous-authorship credit**. `evidence/commands/index.jsonl` hashes full compressed request/response/tool transcripts and source identities; the supplied-patch archive preserves the one-off inputs. Recovered commits retain their historical attribution. The frozen list has 20 substantive reasoning units; independently accepted Formal AI reasoning is **0/20**, below the reporting target. Successful shell execution is not counted as independent reasoning.

Unfinished criteria remain in this PR: final-head hosted CI; native behavior and newly added native fixtures; external benchmark floors and underlying search/arithmetic capability; native/JavaScript shard timing; all remaining warning sources, including extension packaging and cache backend errors; work-item qualification; compatible required-status rules; full semantic template/CI-closure review and all applicable best-practice changes; and the live Hive identity/delivery contract. This PR remains draft and unmerged until these criteria are verified. No benchmark floor, file-size target, identity requirement or default-branch-only build policy is weakened to obtain a green result.

### Hosted checkpoint after CommonJS and evidence-size repairs

All 24 workflow runs on `09021cb30ed98851f59f8b55f2cf71470ec613f4` completed successfully, with zero pending or failed runs. [Exact-head run metadata](../../../experiments/issue-1580/evidence/ci/09021cb30/summary.json) records the identities and byte hashes. Successful scope-gated workflows do not execute the main-only native runtime suites, native detector regressions, external benchmark capability checks, or live work-item qualification. These broader criteria remain unfinished; the subsequent Go and cache commits require their own hosted results.

### Java release commit integrity

The active Java source review now covers 62 files and all four workflows, with per-file transfer decisions. Its 145 historical evidence blobs are inventoried but not fully semantically reviewed, so the complete Java and other remaining repository reviews stay open. Five new bounded regressions use actual local Git and bare remotes: a rejected release commit still produces success and publishes an old-source tag in both Java release modes. Removing only the failure fallback in a disposable copy blocks publication, and successful-release controls retain the correct source/version. The maintained PR entrypoint passes 16 tests without skips. Formal AI already checks the release commit before pushing/tagging; no unnecessary duplicate release implementation is added. No hosted Java execution, upstream fix, main-only native execution or broader issue completion is claimed. Full logs/hashes are in `checks/java-release-integrity.json`.

### Hosted checkpoint after Go/cache changes

All 24 workflows on `e6627385d3ef58435df22ec61d4dfdc4551c9d28` completed successfully. [Exact-head metadata](../../../experiments/issue-1580/evidence/ci/e6627385d/summary.json) retains every run identity and source-byte hashes. These PR results verify the scoped JavaScript/CI changes; they do not execute the main-only native runtimes or resolve the outstanding benchmark, qualification, live identity, full semantic review and historical backend-cause criteria. The subsequent Java regression/review changes need their own hosted verification.

Java failed-release-commit evidence is now filed as [upstream #31](https://github.com/link-foundation/java-ai-driven-development-pipeline-template/issues/31). Its immutable real-Git test link and posted body are retained under `experiments/issue-1580/upstream/`; reporting does not establish an upstream repair. All 24 workflows on `078849576b` have now completed successfully; [exact-head metadata](../../../experiments/issue-1580/evidence/ci/078849576/summary.json) is retained. These scoped PR results do not complete the outstanding main-only criteria.

### Self-repository reference inventory

The partial C# review found a gap in this PR's lexical graph traversal: valid GitHub `$/` references lacked explicit edges. The repair follows reusable workflows, nested composite actions and JavaScript action phases, preserves unresolved/escaping paths, and checks every visited blob. Three new cases fail before the repair; all five focused cases and the maintained 21-test entrypoint pass afterward. Regeneration at all eight unchanged pins adds 13 explicit C#/Python edges and zero files because conservative paths already included those targets. [Full evidence](../../../experiments/issue-1580/evidence/checks/self-repository-references.json) retains the intermediate test-control correction as well. This inventory repair does not complete the C# or remaining full semantic reviews.

### C# active review and changeset collision regression

The [C# active review](templates/csharp-partial-semantic-review.json) covers all six workflows and all 158 active files / 18,327 lines, with exact verified identities and per-file decisions. All 179 historical blobs remain semantically unread; runtime/publication acceptance is unverified. A valid-input filename collision in the pinned merger reproduces successful deletion of every pending changeset. The preservation assertion fails before a disposable exclusive-create guard; all four new controls and the maintained 25-test entrypoint pass afterward. The [full evidence index](../../../experiments/issue-1580/evidence/checks/csharp-changeset-integrity.json) retains before/after logs, source identities, positive controls and duplicate/source checks. No .NET/Bun build, hosted template execution, upstream repair or complete C# review is claimed.

All 24 workflows on `e582b270a01443b995d7f36fd2fa685d508fde77` now pass. Layered CI attempt 1 failed before tests during checkout with curl 28, a bad pack header and a missing promisor object (log lines 5634-5636). The same-head failed-job-only rerun passes. Both complete attempt logs and metadata are retained under `experiments/issue-1580/evidence/ci/e582b270a/`; this successful scoped PR result does not qualify the outstanding main-only native, benchmark, publication or live-identity criteria. Head `0f90c0a7a3bdd77b7d31c276a180d5ee297e57f6` completed with 23 passing workflows and one Layered CI failure: the new reporter evidence basename raised the filename abbreviation ratchet from 90 to 91 (full log lines 7069 and 7965-7968). The byte-preserving rename to `initial-specification-reporter.log.gz` restores the existing baseline. All six abbreviation regression cases and the three touched gates pass; the full failed run, local before/after logs and hashes are retained in the [correction receipt](../../../experiments/issue-1580/evidence/checks/csharp-changeset-collision/abbreviation-filename-correction.json). All 24 workflows on `b79dc1e0f147ed57d8e317c1b93bfa6ffc79078d` subsequently passed; the complete successful Layered CI log retains the passing ratchet at line 7074. Exact-head metadata and the full compressed log are retained under `experiments/issue-1580/evidence/ci/b79dc1e0f/`. These scoped results do not verify main-only acceptance.

The C# valid-input changeset-loss finding is filed as [upstream #111](https://github.com/link-foundation/csharp-ai-driven-development-pipeline-template/issues/111), with immutable executable evidence at `0f90c0a7a3bdd77b7d31c276a180d5ee297e57f6`. Its exact posted body and fresh source/duplicate checks are retained locally. Reporting is not an upstream repair or completion of the remaining review and native/runtime criteria.

### Partial Python review and fragment preservation

The [Python checkpoint](templates/python-partial-semantic-review.json) verifies all 329 blobs and records 24 fully read active files plus one partial file (4,064 of 44,284 active lines), including all six workflows and two separately read release helpers. The other active files, generated bundles/locks and all 126 historical blobs remain semantically unread. This is not complete transitive CI review.

The unchanged manual fragment entrypoint silently overwrites the earlier category and description when two valid invocations share the same second, Git user and branch. A frozen-clock regression uses the complete pinned source and real local Git, with bounded Python processes. The preservation case fails before a disposable exclusive-create guard; four focused cases and the maintained 29-test entrypoint pass afterward, with no skips. Distinct-time and invalid-input controls preserve intended behavior. Full before/after logs, source identities, the initial reporter parsing correction, fresh source verification and duplicate checks are indexed in `experiments/issue-1580/evidence/checks/python-fragment-integrity.json`. The upstream source is unchanged; the published evidence supports [upstream report #157](https://github.com/link-foundation/python-ai-driven-development-pipeline-template/issues/157). Formal AI's analogous Rust timestamp/nonexclusive-write path remains an unexecuted source question. All broader criteria remain active.

## Python review, audit coverage and release-note preservation

The Python review now fully reads 199 active files (21,394/44,284 active lines), through corrected pages 1-54 and separately read release/helper tests. The four unread active files include both generated bundles and both lockfiles; all 126 historical blobs remain unread. Hash verification is separate from semantic reading and source reading is separate from execution.

All 24 workflows passed on `1cc2d536ac11e26602cc55a40dffb61716233aa6`. Full Layered CI run 44116797181 and hashes are retained under `experiments/issue-1580/evidence/ci/1cc2d536a/`. Log lines 4954-4956 confirm all three Python npm inventory observations executed; lines 4948-4951 confirm all four fragment cases. The dedicated Formal AI audit tests were missing from the explicit fast inventory; this batch adds their complete ten-case file and a failing-before selection assertion. Main-only native, benchmark, publication and live-identity criteria remain unfinished.

The repaired Formal AI JavaScript dependency helper checks one successful lockfile inventory before auditing; all ten local regressions pass and existing retry/threshold behavior is retained. The matching Python defect is filed as [upstream #158](https://github.com/link-foundation/python-ai-driven-development-pipeline-template/issues/158), against published evidence at `1cc2d536a`. The exact posted body, fresh duplicate inventory and real-Git reproduction are retained under `experiments/issue-1580/evidence/checks/audit-inventory-boundary/`. Reporting does not repair the upstream helper.

The full unchanged Python release-formatting CLI rewrites literal `\n`, `\r` and escaped quotes in code examples and submits the corrupted body with success. A disposable candidate removes those three rewrites; four focused observations preserve escape bytes, Unicode, badge insertion, idempotence and fatal API read/write errors. The preservation case fails when that candidate is disabled; the maintained entrypoint passes 38 cases without skips. Complete logs, source identities and the distinct report draft are under `experiments/issue-1580/evidence/checks/python-release-formatting/` and `experiments/issue-1580/upstream/python-release-formatting-report.md`. The GitHub CLI is an offline stand-in; no hosted release, upstream repair or full Python suite is claimed. Publish immutable evidence before filing. Independent Formal AI reasoning remains 0/20 accepted.

The initial complete local gate run records 83 pass, 52 native/context exclusions and three failures: two supplied 128 MiB Node-heap exhaustion failures and an old-name spelling in a gate-selection negative control. That control now uses a different typo and its focused suite passes; no policy is relaxed. Main advanced to 512ce428029a92687628b4180309ee5ab660fe08 during validation. Merge it without losing either fast-test addition, reconcile the duplicate Chrono advisory documentation, then revalidate the combined shared data under a measured 256 MiB Node cap before the single push. Hosted execution of the ten selected audit cases and formatter observations remains pending.

Fresh main head 512ce428 has two failed workflows, fully archived with current timestamps and exact hashes. Its six JavaScript assertions now pass in four complete focused suites (15/20/11/2 cases, no skips). The browser test also gains a cooperative reaper after a bounded probe proves an exited PID-1 zombie; timeout status and exact ESRCH assertions are unchanged. A new generated-file inventory regression fails with the original full-file retention and passes all eight cases after twelve-line, one-file-at-a-time reading under a 96 MiB heap cap. The 35 native failing tests and full-shard-5 1080-second timeout are inventoried and remain unverified on the branch. Final full local gates, single-push preflight and hosted execution of the newly selected audit/formatter observations remain pending. All supplied work has zero independent Formal AI authorship credit.

The distinct standalone formatter corruption is filed as [Python template #159](https://github.com/link-foundation/python-ai-driven-development-pipeline-template/issues/159), with complete immutable source and four-case regression evidence at da712fadd and the posted report at d0b8b8956. Immediately before filing, all 158 current issue/PR entries are compared with the prior full inventory (zero changed bodies or new items), current main c532c173 source/helper bytes are verified unchanged, and GitHub-hosted evidence bytes are compared with the local published objects. The exact posted body hash and metadata are retained under evidence/checks/python-release-formatting/publication/. This is an upstream report, not an upstream repair or completion of the wider issue.

Calendar clock regression: main run 44117975432 loses meeting participant/time/zone after the normalized representation erases clock punctuation. A JavaScript reproduction confirms seven failing event cases in five languages and five passing negative controls before the fix. The request recognizer and event payload now read the same unnormalized request span before the existing normalized fallback. The initial worker-only rerun passes all 12 cases. A local HTTP boundary control, shared native fixture, fast inventory entry and main-only replay ledger row are drafted. Final focused checks, projection currency and the complete shared-data gate set are pending. Native execution remains unavailable under the default-branch-only build policy; supplied code earns zero independent authorship credit.

The calendar clock regression has seven failing-before event cases across five languages and five negative controls. Parsing the same raw request span preserves clock punctuation and minutes through both worker and local HTTP chat paths: the final file passes 13 cases. The matching Rust fixture remains unexecuted. Source-budget rechecks pass without raising limits. The initial full gate run retains six failures, five subsequently resolved; the remaining picomatch advisory GHSA-fc5q-39xc-vjpf has no published patched version and blocks the next push. See experiments/issue-1580/evidence/checks/calendar-clock-preservation/ for full logs and current limits.

The new picomatch advisory has a local candidate backport, with published-source identity, license and hashes retained. The focused security probes cover single branches, nested wildcards and two independently reproduced bracket bypasses; hostile matches are finite and isolated. Real installed micromatch/tinyglobby/fdir controls pass on both upstream and the candidate. Desktop and VS Code locks now use the local source and both audits return zero vulnerabilities. The complete local gate set and final expanded tests remain pending. This is no upstream repair and earns zero independent authorship credit; all broader acceptance gaps remain open.

The picomatch candidate now has 20 passing focused security/API/provenance/lock
regressions, with retained failing-before and option-specific reproduction logs.
Real installed micromatch/tinyglobby/fdir consumers pass after the final helper
extraction, and the npm package dry run includes all 12 required files. The
complete local gate run passed 86 gates with 52 explicit exclusions before the
last option refinements; all six touched gates and the 15-case fast-profile file
pass again afterward. Two runner selector mistakes are retained separately and
corrected, rather than counted as product failures. Extracting the existing
extglob helpers removes the new parser file-size warning without raising a
threshold. The finite 1,092-comparison probe has 28 reviewed intentional risky
pattern differences; this is not a general equivalence proof. Four current
template-lock audits reproduce the same advisory in JavaScript/Python/C++
tooling, with full responses and current blob identities retained under
evidence/checks/picomatch-template-locks/. Publication of their distinct reports
is pending this evidence checkpoint. Native execution and all broader acceptance
gaps remain unfinished; supplied work receives zero independent authorship credit.

Final picomatch checkpoint: the complete final-source JavaScript gate run passes all 86 gates with 52 explicit native/context exclusions, zero failures, and no local Rust build. The sealed focused runs pass all 20 security regressions and 15 fast-profile cases. Full aggregate and per-gate logs, exact source hashes and the retained earlier failures are under evidence/checks/picomatch-single-branch/. The final run takes 675.184 seconds with a 256 MiB Node heap cap; the cumulative child peak is 1,011,676 KiB. The JavaScript abbreviation ceiling is lowered from 723 to 722; no ceiling is raised. Publish this batch and inspect exact-head hosted checks before filing the three reproduced template advisory reports. Native regression execution, remaining template semantic review and broader issue acceptance remain unfinished. All supplied work receives zero independent Formal AI authorship credit.

The reproduced picomatch dependency-gate failures are filed in https://github.com/link-foundation/js-ai-driven-development-pipeline-template/issues/307, https://github.com/link-foundation/python-ai-driven-development-pipeline-template/issues/160, https://github.com/link-foundation/cpp-ai-driven-development-pipeline-template/issues/36. Immediately before filing, the complete current issue/PR inventories contain 306/159/35 entries with no changed bodies or matching advisory reports; all three default heads and all four lock Git blobs match the audited inputs. Hosted evidence bytes are verified at immutable commit 83882c0ec7c26b28dc8ac274fada2b39d9dc3a67. Exact posted bodies, hashes, metadata and duplicate/source preflight responses are retained under evidence/checks/picomatch-template-locks/publication/. These reports do not repair the templates or establish runtime reachability. Wider acceptance remains unfinished and supplied work earns zero independent authorship credit.

Manifest-witness repair: current head 83882c0ec has 23 successful workflows and two assertions failing in Layered CI 44156399182, because the picomatch desktop/VS Code manifest updates were not reflected in the governed source fixture. The reviewed complete refresh changes exactly 16 fields (eight source strings and eight hashes), preserving all 221 supported/964 unsupported cases and native source hashes. Both existing failing files now pass all 36 tests without skips; evidence and the complete delta are under evidence/checks/picomatch-governed-refresh/. The complete local gate set and next-head hosted verification remain pending. This supplies no native execution or independent authorship credit; broader acceptance remains unfinished.

The complete pre-merge local gate set passes 86 gates with 52 explicit native/context exclusions, zero failures, in 669.796 seconds with a 256 MiB Node heap cap. All 86 full per-gate logs and aggregate hashes are retained. Fresh default branch 826289fc78154dcd894199ed966f366176be0afb advanced during validation; it must be merged and the combined shared data revalidated before the next push. Current branch runs are complete (23 successful workflows, the one archived Layered CI failure). No post-merge or next-head success is claimed.


The [runtime fixture checkpoint](runtime-fixture-checkpoint.md) records the inline OpenCode test relocation, controlled Moon lexeme/count correction, official generated-registry refresh, native Claude argv expectations and lazy private binding diagnostic contract. Full local gates initially pass 86 with 52 explicit exclusions; a later ledger regression catches four missing fast-inventory replays, now added for final verification. Native execution, recursive parent proof and wider acceptance remain unfinished.

The four added fast-inventory replays pass the ledger/profile checks (4/15 tests), and the final pre-merge source passes 86 JavaScript gates with 52 explicit exclusions, no failures and verified full log hashes. Main 015a6831 advances during validation; combined-source checks remain pending. Java historical review now fully covers pages 1–14/254, with 15 complete files and one partial file; wider semantic review remains unfinished.


The combined calendar/currency source checkpoint at `584b5f57` passes all 15 focused checks (163 tests), formatting/projection checks, and the complete gate set (86 pass, 52 explicit exclusions, zero failures). All detailed hashes are verified; next-head hosted execution remains pending. Java historical review reaches pages 1–22/254, 24 complete files, one partial and 1,369 source lines. Full template applicability, recursive context binding, native runtime and all broader issue criteria remain unfinished. See the [runtime checkpoint](runtime-fixture-checkpoint.md).


The [calendar seed checkpoint](calendar-seed-checkpoint.md) records two
main-only data failures, their failing-before replays and lossless scalar-row
repair (43 passing calendar/currency tests, four ledger tests). The published
2ec6ef93 head has checkout failures and a stale TypeScript mirror exposed by
one retry; the official parity check now passes all 717 sources and 12 tests.
The final combined-source gates pass 86 checks with 52 explicit exclusions and
zero failures; all 86 detailed logs and 14 source hashes are verified. All 24
health/cache fixture tests also pass without transport calls. Next-head hosted
checks remain pending. Native runtime, general recursive context binding and
all broader acceptance remain
unfinished; independent reasoning acceptance remains 0/20.


## Fast source archive acquisition checkpoint

The [checkout checkpoint](checkout-archive-checkpoint.md) records all 23 results
on 6b1d75a6, the remaining repeated checkout failure, a small successful remote
probe and a measured 320 MB source-input reduction. Its regression passes
16/16 and all 20 underlying source checks pass across the retained attempts.
The full hosted checkout remains unverified for the next head. Java historical
literal review covers pages 1–38 and 48–55/254; pages 39–47 remain literally
unread despite separate complete structured OSV comparison. Broader acceptance
and independent reasoning acceptance remain unfinished.


The [native evidence checkpoint](native-evidence-checkpoint.md) records all 23
published-head checks passing on 7e89122c after one checkout retry. Direct native
job logs correct the historical main inventory from 39 to 53 distinct panics.
Four narrow candidate changes retain counting seed provenance, observe the actual
connector workspace, trace current agent-mode owners and freeze interpretation
network input. Focused JavaScript/formatting checks pass; native behavior and
the strict Google Trends catalog regeneration remain unverified. All wider
acceptance criteria stay active, the PR remains draft and independent reasoning
acceptance remains 0/20.


The [language source-binding checkpoint](language-source-binding-checkpoint.md)
now records published `8a74006f`: nine captured-source tests, the original
Spanish sibling, 33 isolated native cases and both HTTP modes pass; the sole
workflow failure is three stale TypeScript projections, regenerated locally.
Main `7b7e189d` is merged. Complete request binding adds four native controls
awaiting hosted execution, and test-only structured-output changes now select
native validation. The final combined source passes 104 focused JavaScript
tests and 86 complete-inventory gates, with 52 explicit native/context exclusions.
Eight original native failures have published replays; the remaining 45 and
three historical budget terminations are unfinished. Java historical reading
reaches 122/254 pages. These are scoped checkpoints; the full issue remains open.


The next [source-binding checkpoint](language-source-binding-checkpoint.md)
records published `f6ecab59`: all 13 captured-source controls, the Spanish sibling,
33 isolated native cases and three HTTP replays pass. Its two workflow failures
are the complete Git-diff buffer and stale generated roles, both reproduced and
repaired locally. The repaired source passes 85 focused tests with one explicit
local skip and 87 executable gates with 52 explicit exclusions. Nine original
unknown-source contracts now have isolated hosted admission; their execution
is pending. Java historical review reaches 124/254 pages and 8,069 lines.
The full issue remains unfinished and the PR remains draft.


The data-contract follow-through records `4d105df7`: 22 workflows pass and one
native cache-positive assertion fails because it assumes an uncommitted runtime
capture. The repair uses the existing committed isogram capture for both bare
and question forms, retaining the whole attributed answer. The next hosted batch
also admits 22 byte-identical original data-file tests. Combined local validation
passes 75 focused tests and 11 affected gates; actual native execution is pending.
Java historical review reaches 126/254 pages and 8,122 lines. The remaining 44
original panic contracts, three budget terminations and broader issue criteria
remain unfinished; the PR remains draft.


The [canonical-data and scheduling checkpoint](canonical-data-and-scheduling-checkpoint.md)
records `0f8c178c` (22/23 workflows pass), repairs both native data failures while
preserving complete decoded grammar objects, and admits 18 unchanged native
scheduling cases. Main `53cbe443` is merged. The merged source passes 138 focused
JavaScript tests, TypeScript runtime qualification and all 87 executable gates;
52 native/context gates explicitly skip. Current native execution remains
pending. Java historical review reaches 130/254 pages and 8,240 literal lines.
The remaining 43 original panics, three budget failures and wider criteria stay
unfinished; the PR remains draft with zero independent reasoning credit.

The [native failure collection and scheduling checkpoint](native-failure-collection-and-scheduling-checkpoint.md) records both `2c9d93c5` failures, 60 native passes, sentence-local scheduling repair, independent failure collection and the remaining acceptance limits.

The [calendar dispatch and native success checkpoint](calendar-dispatch-and-native-success-checkpoint.md) records 23 passing workflows and 96 native Rust cases at `86dcf79d`, the locally verified Russian dispatcher repair, and the pending unchanged operational contracts. Full issue acceptance remains unfinished.

The [calendar clock and native checkout checkpoint](calendar-clock-and-native-checkout-checkpoint.md) records the next published CI failure, shared spelled-hour repair, finite checkout regression and pending native validation.

The [native reasoning and download inventory checkpoint](native-reasoning-and-download-inventory-checkpoint.md) records hosted clock success, the complete shell sweep failure and its local repair, and pending execution of the existing 28 reasoning contracts.

The [offline instruction and catalog checkpoint](offline-research-and-catalog-checkpoint.md) records complete `3c52b7f5` execution (202 native passes, one failure and four HTTP passes), the cache-only instruction repair, actual generated catalog/session adoption and pending fresh native replay. Full issue acceptance remains unfinished.

The [Hive replay and native profile checkpoint](hive-replay-and-native-profile-checkpoint.md) records the complete `cf22c611` failures, the assertion-preserving optimization repair, localized title controls and 45 pending native Hive cases. The wider issue remains unfinished.

The [interface replay and compilation checkpoint](interface-replay-and-compile-checkpoint.md) records both `53d8212b` failures, unchanged shared-helper visibility repair, maintained TypeScript regeneration and 77 pending native interface cases. The strict original replay union is 27/53; the wider issue remains unfinished.

### Published native package evidence and next repair batch, 20 November

Published head `5bbbd91e` finished 23 workflows with 21 successes. Its eight executed native jobs pass, including 77 workflow contracts and the actual Cargo package/version consumers; three downstream jobs are explicitly skipped. The two failures are a documentation-context `steps` reference and a 360-second JavaScript shard timeout. Their raw lines and local repairs are recorded in [the exact-head assessment](../../../experiments/issue-1580/evidence/ci/native-families-5bbbd91e/assessment.json). The first assessment helper incorrectly included skipped jobs in its all-success assertion; the failed observation and corrected inventories are preserved. Warning text echoed as colored shell source is separated from emitted diagnostics.

The combined batch guards retry admission, raises stale timing weights from 208 complete file events without lowering conservative history, splits the complete PR inventory into four groups with unchanged clocks, and repairs release fragment I/O propagation and snapshot cleanup in three real consumers. Local final checks pass 35 admission, 95 maintained workflow and 32 documentation cases plus nine touched gates; the measured-timing repair also has a passing complete 139-entry gate inventory with explicit exclusions. Actual new Rust filesystem behavior and repaired hosted shard completion still require CI on the next published head. No local Rust build ran.

Rust template literal review is 132/373 files and 22,468 lines. All remaining 241 files, counterpart/runtime practice comparisons and the other unfinished original acceptance criteria remain open. Strict original acceptance is unchanged at 31/53 and accepted independent Formal AI reasoning at 0/20; this PR remains draft and unmerged.


The [registry/release checkpoint](registry-release-checkpoint.md) records all 23
passing workflows on 14ef1b5f, actual native family counts, 124 unresolved source
size warnings, the unexecuted local registry repair, its failing-before HTTP
fixture correction and Rust literal review at 139/373 files. Independent
reasoning remains 0/20 and strict original-source acceptance remains 31/53.

The [2026-11-20 original CodeQL report and Rust context checkpoint](rust-codeql-context-checkpoint.md)
records all 23 automatic workflows passing on `ad59ff56`, the separate failing
producer's two Actions findings and 511 Rust extraction warnings, and source
repairs whose hosted validation is still pending. It retains all 286 verified
collector streams, the original SARIF bytes, 15 security regression groups and
the complete 87-pass/53-excluded local gate inventory. Strict original replay
acceptance remains 31/53; broader criteria and 0/20 independent reasoning stay
open. Rust literal review is 237/373 files and 34,513 lines, with 136 files and
applicable practice/runtime verification unfinished.


The [Cargo optional-dependency and actual producer checkpoint](codeql-optional-dependency-checkpoint.md) records clean original Actions SARIF on `1fd0c4fd`, the Rust pre-analysis failure and its captured-input regression fix, pending native detector execution, and the precise incomplete review boundary. Rust literal review reaches 243/373 files and 34,777 lines. All broader unfinished criteria remain open.

The [artifact extraction checkpoint](artifact-extraction-checkpoint.md) records all 24 terminal workflows on `1fd0c4fd`, 506 native passes, the bounded real DEP0005 reproduction and the shared consumer repair. Local archive tests pass; the optional Rust context repair, native detector and three hosted artifact transports require the next head. Wider acceptance remains unfinished.
