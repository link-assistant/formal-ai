# Repository task generalization and durable memory

Status: active; analysis and acceptance criteria recorded before test/code edits.
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
- [ ] P1: read all three source issues/comments, patches and failure logs;
  classify Formal AI vs client vs orchestration faults with primary evidence.
  Refresh the all-issue requirements audit since its prior cutoff and reconcile
  each new coding/memory requirement with an executable acceptance criterion.
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
- [ ] P7: report proven current Hive Mind/Agent CLI defects in their own
  repositories, after duplicate/template checks. Link reports here. Do not
  publish full private traces; publish minimal reviewed reproduction evidence.
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

## Formal AI experiment discipline

Use `/Users/konard/.bun/bin/agent` and this branch's rebuilt binary through its
OpenAI-compatible endpoint. Isolate workspace and memory under `/private/tmp`.
First submit useful ordinary requests without a precomputed answer. If a task
fails, retain the exact failure, decompose the unmet obligation, add a regression,
and repeat after repair. A narrower supported subtask is useful evidence but
does not replace the original task's still-open acceptance criterion.

## Disk, restart, and commands

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
