# Repository task generalization and durable memory

Status: active; analysis and acceptance criteria recorded before test/code edits.
Latest implementation/evidence checkpoint: [Plan 07](07-prerequisite-discovery-bridge.md),
including source-span integrity, bound recovery replay, genuine authorship checks,
and the distinction between a failed open-ended refactor and a verified rename.
The [2026-09-16 issue delta](../raw-data/requirements-delta-2026-09-16.md)
also reopens the resource/outcome dimensions of #491 in the requirements map.
Baseline: `2f7a381a28bdaad01016f1a265201a696c9dd869`, 2026-09-15.
All implementation belongs to PR #888, remote `issue-710-14da90b08a12`.

## Why the earlier completion claim is insufficient

Green checks establish the existing contracts. They do not establish the whole
vision. The first-20 Python benchmark slices, a single-file CLI task, and a
successful source-file write do not prove complete repository implementation.
Plans 01–05 remain historical evidence; their completion labels must not be
interpreted as universal capability or permission to omit new acceptance cases.

The maintainer requests plan first, tests second, implementation third, more
use of Formal AI through Agent CLI, recursive discovery over trusted sources,
general algorithms in the associative/meta-language stack, and preservation of
chat history and personal experience unless the user changes/deletes them.
Tests may change when a more general implementation improves the behavior, but
must retain the original semantic requirements and add stronger evidence.

## External evidence captured before implementation

| Case | Latest observed state | Contract violated / next diagnosis |
| --- | --- | --- |
| [Kotlin PR 2](https://github.com/konard/test-hello-world-019fb330-fa49-7c9d-a664-b7ea33bb698a/pull/2) | head `4ff1d9af8684014081fb75713ba991c1bb447691`; `Main.java` prints `Main.kt`; `Main.class` committed; no checks | Issue 1 explicitly requires Kotlin, exact `Hello, World!`, comments, run instructions, and CI that installs the compiler and asserts output. Wrong language, wrong payload, missing CI and source hygiene are actual failures. |
| [Scala PR 2](https://github.com/konard/test-hello-world-019fb330-00e1-73b9-955e-f357a1600d5b/pull/2) | head `a4f344c5f5a18031d4dbbdbf2a274127a26aac27`; `Main.scala` and workflow; two failed checks | Agent sessions end at `scalac: not found`; inspect workflow and logs for missing setup, output assertion, failed-step recovery, and uncommitted-artifact accounting. Hive Mind correctly reports repeated no progress; do not label that detection a bug. |
| [Rust PR 2](https://github.com/konard/test-hello-world-019fb331-c107-78c7-8ff6-9f127a3c593c/pull/2) | head `9d74fb354ad3ee68ea64167543be59b6604d0357`; placeholder `.gitkeep` only; no checks | Latest session fails before a model turn: `Error loading config.toml: invalid transport in mcp_servers.playwright`. Log says the wrapper removed MCP entries then supplied `mcp_servers.playwright.enabled=false`. Inspect current Hive Mind code and existing reports before reporting the configuration defect upstream. |

PR #888 baseline: 77 checks, 68 success, 9 intended skips; mergeable and clean.
Test deployments report Formal AI 0.350.0, which alone does not identify the
branch commit; reproduce against this worktree binary before attributing fixes.

## General model to implement and prove

1. Acquire the complete issue and relevant comments as attributed task evidence.
   Keep harness instructions, issue requirements, source excerpts, file paths,
   output literals, and programming-language constraints in distinct roles.
   Resolve conflicts explicitly; a filename is not the program's output.
2. Formalize requirements as an obligation graph in Links Notation. Each node
   has its origin, expected observable result, prerequisites, implementation
   artifact(s), verification, and state. Enumerated prose, headings, and bullets
   must survive; an unsupported clause remains a named open requirement.
3. Discover missing knowledge recursively: compiler/runtime -> official install
   instructions -> build/run commands -> official CI setup -> exact-output
   verification. Resolve unknown concepts through the existing trusted registry,
   source cache, and formalization machinery. Record provenance and dependency
   edges. Detect cycles and exhausted searches without inventing knowledge.
4. Compose reusable actions from formal operations and discovered contracts.
   Bind language, path, and output independently. Use existing project evidence
   to choose the build system and necessary project files. Do not add files by
   filename-count heuristics or embed solutions to these three issue IDs.
5. Execute a dependency-ready leaf, interpret its actual exit status/output,
   and repair from evidence. A missing compiler becomes a prerequisite problem
   to solve. A written file, read-back plan, or empty set of CI checks never
   satisfies an unverified program/CI requirement.
6. Verify semantic completion: requested language and exact stdout; comments
   and usable run instructions; required project files; CI triggers, runtime
   setup and assertions; staging excludes compiler outputs and unrelated files.
   Re-evaluate all remaining obligations after every tool result.
7. Cache source-derived procedures with provenance and a replay/rediscovery
   recipe. Eviction is allowed only for explicitly reproducible knowledge.
   User/assistant turns, observations of own actions, and unknown legacy memory
   are retained by default even when infrequently accessed or summarized.
   Explicit user deletion/modification remains supported and tested.

## Test-first leaves and durable progress

Use `[ ]` until evidence exists; record failing command/output before fixing.
Every leaf records files, validation, residual gaps, and next step here.

- [x] P0: inspect branch, disk, and latest test PR summaries; record this plan.
- [x] P1: read all three source issues/comments, patches and failure logs;
  classify Formal AI vs client vs orchestration faults with primary evidence.
  Refresh the all-issue requirements audit since its prior cutoff and reconcile
  each new coding/memory requirement with an executable acceptance criterion.
  Evidence: checkpoints below, R710-R1–R10, the September 15/16 requirement
  deltas, and the #491 continuation shard. This completes reconciliation, not
  the implementation of every accepted requirement.
- [ ] P2: add failing generalized language/output-binding and issue-envelope
  tests. Include unseen filenames, output that resembles a filename, multiple
  languages and Unicode paraphrases; retain existing working behavior.
- [ ] P3: add failing requirement-graph/project-completion tests. Include
  bullets without enumerators, missing compiler, missing workflow setup,
  nonzero verification, generated build artifacts, and a second unseen project.
- [ ] P4: implement the smallest shared formalization/planning changes that
  satisfy P2/P3 using existing source discovery and associative representations.
- [ ] P5: add retention/eviction tests before changes: cold replayable source
  cache can be evicted; original conversations and action experience survive
  pressure, generalization, export/import and restart; explicit deletion works.
  Then implement any missing provenance-based retention policy across runtimes.
- [ ] P6: use the rebuilt Formal AI via external Agent CLI for a meaningful
  requirement-analysis task, a regression-test/source modification, and complete
  project tasks from fresh state. Persist session IDs, model/build identity,
  observed tool results, independently verified artifacts, and failures. Fix
  generalized blockers and rerun; do not count narrated success as authorship.
- [x] P7: report proven current Hive Mind/Agent CLI defects in their own
  repositories, after duplicate/template checks. Link reports here. Do not
  publish full private traces; publish minimal reviewed reproduction evidence.
  Hive Mind #2259 is filed and remains open without comments on September 16.
  No distinct current Agent CLI defect is proven by the compiler/authoring
  failures; those are retained as Formal AI gaps, not duplicate wrapper reports.
- [ ] P8: run focused regressions, existing coding and memory suites, held-out
  discovery/forget/rebuild checks, self-AST/seed/requirements generation, all
  required local gates, and release preflight. Verify benchmark semantics remain
  intact; update tests only when stronger behavior preserves old obligations.
- [ ] P9: review the complete diff; commit completed leaves, then one push for
  this batch. Update PR description with verified results and remaining limits.
  Observe new CI while doing useful work; fix actual failures. A release path
  can be validated, but external service availability/credentials cannot be
  guaranteed by a PR check. Do not merge without the maintainer requesting it.

### 2026-09-15 implementation checkpoint

- All three source issues explicitly require comments, run instructions, exact
  output, CI triggers, compiler setup, execution, and an output assertion. The
  Scala patch has neither setup nor assertions and uses the wrong output case.
- External Agent CLI baseline session `ses_f5a4c1c06ffeFKPhb1kpSZw46W` against
  the branch binary wrote a reporting menu into `Main.kt`; the harness failed.
  Local private logs: `/tmp/agent-out-8916.log`,
  `/tmp/formal-ai-serve-8916.log`. These are not published or committed.
- Three new tests in `tests/unit/issue_1133_hive_mind_three_runs.rs` failed on
  baseline: exact output binding, language/output independence from page chrome,
  and source destination vs report destination. Existing 18 tests passed.
- First shared change composes `entry(print_stdout(literal))` from attributed
  operation records rather than treating arbitrary quotes as program output.
  The agent destination regression passes; the next test run checks integration
  before the ordinary solver decomposes the requirement clauses.
- A new memory pressure test protects original user/assistant/tool observations
  even when their event kind resembles a cache or summary. It is not fixed yet.
- Hive Mind defect reported with current-source evidence as
  [#2259](https://github.com/link-assistant/hive-mind/issues/2259). The separate
  Agent CLI no-progress report is consistent with the observed compiler failure;
  no current Agent CLI defect has been established.
- Disk checkpoint after the first build: 31 GiB available. No cleanup performed.

The first implementation increment passes all 23 issue-1133 tests, including
the five new semantic checks. The ordinary output composer covers literal
stdout in Rust/Kotlin/Scala/Python; this is a reusable primitive, not a claim
of arbitrary project synthesis. Its declaration, entry point, literal escaping,
and CI setup are source-attributed link records. Generated verification is
executed in the test: wrong case, an extra newline and exit 7 each fail.

The project suite previously asserted `git add -A` verbatim. Those assertions
now require the safer explicit-artifact commit, preserve branch/commit evidence,
and reject broad staging. This is an intentional strengthening of behavior.

Still open in this increment: automatic compiler prerequisite discovery and
recovery; full obligation-graph execution; ordinary prose specifying additional
behavior beyond a literal; complete source-cache provenance/legacy retention;
cross-client and full-suite regression review; a rebuilt external Agent run.
Do not mark P2–P6 complete solely because the first focused suite passes.

### Next safety leaf: reconstruction evidence (tests before implementation)

Legacy records have no reliable origin merely because `kind` says `summary` or
`source:http`. New acceptance tests retain unknown/cache-shaped records and
modified imported seeds. A disposable public-source copy must explicitly have
cache origin and a `rediscover:https://...` evidence edge; the observed tool
event stays durable. Embedded seed eviction requires exact identity with this
binary's seed. Derived task caches require successful current replay, not only
the word `derived`. Apply must recheck eligibility and account only bytes
actually removed. No original experience is sacrificed to satisfy a disk target.

Execution-evidence leaf: add wrong-path/wrong-content write, out-of-order run,
duplicate-result and orphaned-result tests before modifying recipe progress.
The #908 fixture previously omitted assistant tool calls; retain the actual
protocol calls while keeping its silent-success/nonzero-failure assertions.
The product must bind file bytes and command order, not count tool messages.

### Broad regression checkpoint

First full unit run: 3,503 passed, 11 failed, 4 ignored. All new coding,
evidence-binding and memory behavior tests passed. Failures name stale generated
self-AST/planner fixtures, moved-function/test traceability, a solver warning-band
limit, the redundant handler insertion's debt count (removed), sandbox-denied
`ps`, and one temporary Git-fixture initialization collision. These are tracked
work, not a green result. Regenerate projections and rerun with necessary process
inspection permission; do not weaken the gates. Public-source compiler versions
and action commits were checked against their upstream repositories; Scala
2.13.18 has an upstream release, rather than being an inferred version.

Live rebuilt-binary evidence: Python session `ses_f5a2aada0ffe8fjZ6GomOOk1u5`
completed six external Agent CLI rounds, wrote source, verifier and workflow,
and produced exactly `Aster 73!` plus LF. Independent rerun of its verifier
passed. Reviewed artifacts are in `/private/tmp/formal-ai-888-python-8917`;
raw logs remain private. Kotlin session `ses_f5a282912ffeOLFh21bbx9Dqsl`
now writes correct Kotlin and both supporting files, but ends honestly at
`kotlinc: command not found` after five rounds. This is still incomplete, and
not a successful Kotlin end-to-end result. Compiler recovery is the next
capability leaf, not an upstream client blame.

The Scala CI job log confirms `scalac: command not found`, exit 127; no issue
newer than #1137 appeared in the refreshed issue list. The second full test
run passed 3,509, failed five, ignored four: three short-deadline tests under
load, plus two historical snapshots whose expected catalog intent changes to
an attributed source-example intent when network/cache discovery succeeds.
Resolve test determinism or strengthen semantic assertions; do not disable
discovery or falsely claim the suite passed.

Next test-first leaf records explicit renamed source destinations and multiple
ordered stdout clauses. Reuse the existing cued write-target parser and compose
literal output operands; do not treat a default path or first quoted value as
the entire requirement. These tests are added before their implementation.

Disk checkpoint: 29 GiB free, shared target 9.7 GiB. One gate launch omitted
the shared-target variable and started a duplicate documentation build; it was
interrupted and rerun with the full environment. Worktree target is 939 MiB
and predates this run, so it was not blindly deleted. No user data was removed.

The historical catalog snapshots are being made explicitly offline in isolated
child processes with fresh source-cache directories. Their original exact
assertions remain unchanged. Live/fixture-backed source-example discovery keeps
its separate tests, and production routing is not restricted to make an old
snapshot pass. Child-only environment overrides avoid parallel-test races.

## Formal AI experiment discipline

### Remaining reconstruction edge, before the next test change

The third full unit run passed 3,514, failed two, ignored four. The historical
offline replays and deadline tests passed. Move the typed destination helper
beside the existing write-request parser to keep the planner under its original
900-line warning limit. Diagnose the isolated fetched Rust execution failure
from command evidence, without replacing its execution assertion with a skip.

The public-cache test currently preserves a historical observation but loses
the URL held only in the evicted cache record. Strengthen it before changing
eviction: retain a compact reconstruction record with the original cache ID,
source URL/recipe, and provenance; never carry its disposable payload. Preserve
that record through export/import and later pressure, and account for retained
metadata when estimating reclaimed bytes. A URL supports reacquisition, not an
assertion that today's page reproduces a historical observation byte-for-byte.
Embedded-seed reconstruction remains tied to exact current seed identity.

The strengthened reconstruction test failed before implementation and now
passes along with all 104 memory-filtered tests. The apply operation moved to
`src/dreaming/apply.rs` to preserve module-size limits. A further test first
checks that retaining metadata cannot turn a tiny-cache eviction into storage
growth; only positive net savings may be selected for pressure relief.

Agent session `ses_f5a131ed8ffe4pk0sDxINN0ZSn` completed a three-round
requirement-formalization run. Its reviewed knowledge base is under
`agent-cli-evidence/memory-contract/`; all five statements survive, but concept
and procedure counts are zero. The generic `pred:states` fallback is source
preservation, not resolved semantics. Its final report incorrectly says all
nine primitives are realized. Add a failing report-coverage regression before
fixing that claim, and rerun through the external Agent after rebuilding.

Persist the successful ordinary Python project shape as an always-run Agent
CLI CI replay with a fresh held-out literal. Keep its six-round, source,
workflow, verifier, and actual-output assertions. This adds continuous coverage
without claiming Kotlin/Scala prerequisite recovery is solved.

### 2026-09-16 all-feature checkpoint

All-feature unit run: 3,507 passed, 11 failed, 4 ignored. The isolated Rust
execution and the three deadline regressions passed without concurrent builds.
Four failures still expected the old spelled-out coverage claim; their exact
complete-case assertion is now `9 of 9`, while the new incomplete-case test
requires `2 of 9`. Four seed failures identified response definitions needing
distinct language facets and references to real meaning/role definitions.
Three memory fixtures used payloads smaller than their retained reconstruction
metadata. Increase those ordering/deduplication fixture payloads, retaining every
original assertion; the separate tiny-cache test requires no eviction when the
net saving is non-positive. None of these failures is being skipped.

Current disk availability is 28 GiB. No data/cache pruning has been performed.
The shared target and pinned Bun are still used; the rebuilt browser bundles
are byte-identical, with only the generated seed-file inventory changed.

The corrected all-feature unit run is green: **3,518 passed, 0 failed, 4
intentionally ignored**, 197.50 seconds, two test threads. Instruction checks
are conjunctive, tiny caches are retained when metadata would erase the saving,
and complete/partial formalization reports show measured primitive counts.
Actionlint, total seed closure, seed metadata, file limits, behavioral test
documentation and debt checks pass. The prose-debt ceiling is reduced from
1,287 to 1,286, with no other ceiling raised. Full Rust-stage gates, remaining
test targets and the rebuilt external Agent replay are still pending.

GitHub refresh on 2026-09-16: PR #888 remains open at the unchanged green
`2f7a381a2` baseline. The three test PR heads and their failed/missing checks
are unchanged. No non-PR issue newer than #1137 appeared in the latest page;
Hive Mind #2259 is open with no comments. This local batch has not been pushed.

Use `/Users/konard/.bun/bin/agent` and this branch's rebuilt binary through its
OpenAI-compatible endpoint. Isolate workspace and memory under `/private/tmp`.
First submit useful ordinary requests without a precomputed answer. If a task
fails, retain the exact failure, decompose the unmet obligation, add a regression,
and repeat after repair. A narrower supported subtask is useful evidence but
does not replace the original task's still-open acceptance criterion.

## Disk, restart, and commands

### 2026-09-16 rebuilt external Agent checkpoint

Session `ses_f59f13589fferBVYTqnpiCxeG2` completed the always-run CI task
through actual Agent CLI in six rounds. Its Python source, exact-output shell
verifier and GitHub Actions workflow are in the private evidence directory
`/private/tmp/formal-ai-888-python-8920`. Independently running the verifier
printed `Saffron 61!` and exited zero; actionlint accepted the workflow.
The rebuilt memory-contract replay on port 8921 also passed: three rounds,
retained source assertions, and the honest `2 of 9 protocol primitives`
report. This does not claim semantic procedures have been discovered.

Browser seed copies were synchronized with the existing generator, including
the new report definitions. Free disk space is 27 GiB; no cleanup performed.
All 32 registered Rust-stage gates are running with the shared target.

Next memory-accounting leaf (test first): the apply result currently subtracts
reconstruction records but not newly retained amendments, patterns, candidates,
failures or trials. Add a pressure test that learns from original requirements
and also evicts a public cache. Compare pre/post event-byte estimates and assert
that the reported saving is the actual nonnegative net decrease, while all
original records remain. Then calculate outcome bytes over the final retained
store, not a hand-maintained subset of generated record kinds. Agent CLI on
port 8922 has also been given an ordinary regression-authoring request for this
requirement; its success must be judged from executable code, not file presence.

The test failed before implementation: reported 853 bytes versus the actual
520-byte net decrease. Compute the apply outcome from pre/post store estimates
so future retained record types are included automatically. External Agent
session `ses_f59efab43ffe4dx4GACXmYBj7C` did not author the requested test:
it read the nonexistent target and ended with a file-not-found report.
Agent exited zero but the harness correctly failed. Raw logs remain private
under `/tmp/*8922.log`. This is a Formal AI creation/semantic-synthesis gap,
not evidence of an Agent CLI defect. Reproduce with workspace-backed context
before designing a shared creation/research route; do not add a canned test.

Validation checkpoint: all 32 Rust-stage and all 12 web-stage gates passed;
the five JavaScript lockfile audits reported zero vulnerabilities. Standalone
browser tests: 83 passed, none failed. Memory-filtered tests after the net-byte
fix: 106 passed, none failed. Strengthen the new accounting test with both a
net-growing learning pass (reported saving zero) and a positive-saving pass.
Self-AST and planner projections were regenerated without deleting documents.

Remaining-target run exposed two integration failures (375 passed, 2 failed,
1 ignored). The common fake-client fixture clears some provider environment
variables but leaves the developer's `ANTHROPIC_AUTH_TOKEN`; the wrapper
correctly honors that configured value instead of the fixture's expected
default. Do not print the value or publish the temporary capture. Make fixture
isolation derive credential/command/config environment keys from the same
integration catalog, preserving production credential precedence. Add a test
with synthetic inherited credentials and command overrides before the helper.
The standalone-wrapper invocation test also contacts hardcoded port 18080,
despite needing only to verify client arguments. Its health response caused
JSON `trailing characters`. Disable server startup in that invocation-only
fixture, like the shared wrapper helper; retain independent lifecycle/version
tests. Do not stop whichever local service owns that port.

The added fixture-isolation test failed on the old list at
`FORMAL_AI_OPENCODE_DESKTOP_BIN`, proving the problem extends beyond one
provider. The helper now derives API-key, command and invocation/config keys
from the integration seed. Only fake-client tests use it; production handling
is unchanged. A header-only check confirmed localhost:18080 returns a chunked
404, not the health response this invocation-only fixture assumed.

Final unit rerun after the net-accounting change: **3,519 passed, 0 failed,
4 intentionally ignored**, 216.81 seconds. The small-cache learning case now
proves a net-growing pass reports zero reclaimed bytes; the large case proves
positive net savings. Source-contract target: **494 passed, 0 failed**. The
fixture-isolated full integration rerun is pending. Secretlint scanned all 83
then-changed files successfully; recheck the newly added fixture helper before
committing it. Local disk is now 26 GiB free, shared target 9.8 GiB at the last
size check. No caches, containers or user files have been pruned.

Integration rerun after fixture isolation: **378 passed, 0 failed, 1 ignored**,
115.61 seconds; this includes the new synthetic-environment regression. Final
Clippy, formatting, diff-whitespace, actionlint and changed-file secret scans
pass. Evidence artifact is checkpointed separately in commit `543671431` with
its actual Formal AI session/model attribution; it contains no raw private log.
The source/test implementation checkpoint follows. No push has been performed;
Plan 07's structural requirement-preservation leaf is the next implementation.

Worktree `/private/tmp/wt888`; main clone's untracked continuation transcript is
user data. Initial free space 32 GiB. Reuse the shared 9.7 GiB target; avoid new
container images, duplicate release builds, and broad cleanup. No known build
process remained from the previous completed turn; sandbox denies `ps`.

Build commands must set `RUSTUP_TOOLCHAIN=1.98.1`,
`LINDERA_DICTIONARIES_PATH=/tmp/formal-ai-lindera-cache`, and
`CARGO_TARGET_DIR=/Users/konard/Code/Archive/link-assistant/formal-ai/target`.
Pinned Bun: `/private/tmp/bun-1.4.0/extracted/bun-darwin-aarch64/bun`.
Read `.githooks/pre-commit` before committing and disable its unrelated pruning
with `CARGO_TEST_NO_PRUNE=1 DOCKER_NO_PRUNE=1`. Check `df -h /private/tmp`
before broad builds and at handoff. Preserve unrelated Cargo/Docker/user data.

Resume by reading this file, `git status --short`, and the latest unchecked
leaf's evidence. Update this document whenever a diagnosis or design changes.
Do not infer a successful test or push from an unchecked plan entry.
