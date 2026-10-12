# Changelog archive: 0.307.0 to 0.345.0

Releases in this range, newest first. Newer releases are in [CHANGELOG.md](../../CHANGELOG.md).

## [0.345.0] - 2026-08-14

### Added
- Enforce at least five distinct wording variations per conversational test case in every advertised language (en, ru, hi, zh) with the `check:variation-floor` CI gate, backed by a recorded corpus whose every prompt is answered by the engine and whose every record shows the exact answer that wording produces.
- Join incremental Agent-CLI execution and auto-learning into one evidence-preserving lifecycle: attempt the whole task, split only after failure, compose passing leaves, retry the parent, and feed every recorded session to proposal-only learning behind human review.

### Fixed
- Answer small talk in full in Hindi and Chinese. The question-necessity pass could not find a sentence boundary in a script that does not space its sentences or that ends them with a danda, and its requirement cues covered the English follow-up questions only, so `धन्यवाद`, `谢谢` and `你好吗` answered with an empty string and the Russian and Hindi wellbeing answers lost their closing sentence. A question the answer quotes as an example — in corner brackets or parentheses — is no longer read as a question the answer asks.
- Normalize variation prompts identically in Node and Rust with NFKC plus Unicode category filtering, so fullwidth compatibility characters deduplicate while Hindi combining marks remain meaningful.

### Added
- Build and run every generated language project inside the matching `link-foundation/box` image, using the language's own init commands (`cargo new`, `npm init`, `go mod init`, …), as a `box-language-projects` CI matrix and as a Docker-gated `cargo test`.
- `data/meta/box-image-survey.lino` records which box image variants are actually published and which tag the matrix pins, so the language contract can no longer name an image nobody publishes.

### Fixed
- Convert an installation guide into a script even when its steps name project creation and a build, instead of answering with a software-project plan.

## [0.344.0] - 2026-08-14

### Added
- Add bounded equality saturation and function-free Datalog inference to the symbolic proof engine, with honest 20/20 egg and 5/5 Ascent upstream benchmark scores for #923.

## [0.343.0] - 2026-08-14

### Added
- Add localhost-default WebSocket and host-only WebRTC data-channel server and client modes to the `formal-ai` CLI while sharing the existing API permissions and memory.

## [0.342.0] - 2026-08-14

### Added
- Add replayable memory, workspace, and source necessity traces before asking a user question.
- Add a seed-driven requirement-versus-fact classifier and a monotonic questions-per-task benchmark.

### Changed
- Research factual unknowns instead of delegating them to the user, and limit answers to one requirement-level question.

## [0.341.0] - 2026-08-14

### Added
- Procedural "how to X" requests now synthesise one ordered guide from the enabled trusted services in `data/seed/sources-registry.lino`, recursively capturing result pages within declared depth, page, and age bounds and keeping the exact source URL, license, and payload digest on every accepted step.
- Per-service accessibility (success *and* failure) is remembered in the environment's associative memory for seven days, with explicit refresh and invalidation, so a stale body cache is no longer mistaken for an availability record.
- Committed real-service QA captures with timestamps, digests, and licenses; the normal test suite replays them offline on the native, HTTP, and browser paths, and a `FORMAL_AI_LIVE_FETCH=1` refresh check detects drift against the live services.
- The reader-facing guide is rendered from seeded prose (`data/seed/multilingual-responses-procedure.lino`), so `HowToGuide::markdown_in` and the browser worker render the same evidence in any seeded language, while trace and evidence lines are `key=value` records built through the new `trace_record` module.

### Added
- Merge-conflict policy: `data/meta/merge-conflict-policy.lino` declares every structural cause of a merge conflict this repository has actually had, the mechanism that removes it, and the verifier that keeps it removed. `python3 scripts/analyze-merge-conflicts.py --ledger` measures the history (884 merges, 1914 conflict events) into `data/meta/merge-conflict-ledger.lino`, and `rust-script scripts/check-merge-conflict-policy.rs` fails the build when a path that has actually been conflicting is neither mechanized nor deferred with a written reason. No `git config` step is needed: every mechanism uses git's built-in `merge=union` driver or a committed generator.
- CI gates are one file each under `data/meta/ci-gates/`, run by `rust-script scripts/run-ci-gates.rs --stage <stage>`. Adding a check no longer edits `.github/workflows/release.yml`, which was the repository's third most conflicted path.
- One seed inventory for both runtimes: `data/meta/seed-registry.lino` names every `data/seed/*.lino` file once, and `rust-script scripts/generate-seed-registry.rs --write` generates `src/seed/embedded_registry.rs` and `src/web/seed-files.js` from it, so the Rust engine and the browser worker cannot disagree about which seed files exist.

### Changed
- `src/seed/embedded.rs` and `src/web/seed_loader.js` no longer carry their own copies of the seed file list; `src/agentic_coding/mod.rs` and `src/web/formal_ai_worker.js` no longer carry their own declaration lists. Each list now lives in a sibling file that contains nothing else and is union merged, with `rust-script scripts/normalize-ordered-lists.rs --write` restoring the canonical order.
- CONTRIBUTING.md documents what to add where so a contribution stops creating an append point, and `docs/case-studies/issue-991/merge-conflict-analysis.md` records the measurement behind every decision.
- `REQUIREMENTS.md` is assembled from one shard per issue under `docs/requirements/`. A shard's links are written relative to the shard, so it reads correctly on its own page; assembly rebases them to the repository root and `--split` rebases them back, and `rust-script scripts/assemble-requirements.rs` fails when a shard link does not resolve from the shard's own directory.

### Added
- Failure-driven splitting: `TaskExecutor` gained a `split` hook, so a failed task can be shrunk from its own failure instead of from a plan made before any evidence existed. `formal_ai::task_decomposition::SplittingExecutor` answers that hook with the repository's own `decompose_task`, one level per split, and records every split with the failure that justified it. The controller refuses a child that repeats its parent, bounds splitting with `DEFAULT_SPLIT_DEPTH_BOUND`, and `solve_recursively_within` lets a caller pick another bound (zero reproduces the previous plan-driven protocol exactly).
- `formal-ai agent dispatch --incremental` runs that protocol against external agent CLIs: the whole task is attempted first, only a failure is split, a passing attempt's effects are applied to the workspace before the next attempt starts, and an irreducible failure escalates to the next CLI in `--cli` instead of stopping. The report carries an `incremental` trace of every attempt, split, and blocked task; the exit status reflects the root task only.
- Every blocked task becomes a review request, mirrored to `proposals.lino` next to the report: the task, every CLI that tried it, the evidence each attempt produced, and the status `human_review_required`. A run cannot approve its own extension, so this is the same gate a learned decomposition strategy passes through.

### Changed
- `RecursiveRun` now reports `split_applied`, `split_depth_reached()`, and `blocked_leaves()`, and the review-gated learning path reads blocked leaves from the run instead of walking the tree a second time.

## [0.340.0] - 2026-08-14

### Added
- Learn proposal-only reusable methods from real recursive-core event logs, validate them on held-out traces, and adopt them as registry link data only through benchmark-gated human-confirmed promotion (#922).

## [0.339.3] - 2026-08-13

### Fixed
- Release now verifies that the published `ghcr.io/link-assistant/formal-ai` image is anonymously pullable (`scripts/verify-ghcr-visibility.sh`, run in both `auto-release` and `manual-release`), so a private container package fails the release instead of breaking downstream `docker pull` with `unauthorized` (#1001).

### Documentation
- README explains how to tell a private GHCR package from a missing one and what to do until it is public (#1001).

## [0.339.2] - 2026-08-11

### Fixed

- Harness and server log exports from the agentic `Report` flow are written
  into a surviving temporary directory and print their final path, instead of
  dropping `formal-ai-*.lino` session dumps into the caller's working
  directory — a repository checkout root stays clean (#945).
- Report-target answers that use machine values now select every target:
  `formal_ai` was silently dropped because prompt normalization turned the
  underscore into a space before matching (#996).
- Final answers that inline machine text — the general-change plan event and
  the formalized knowledge base — wrap it in a fenced `lino` code block, so
  the text survives GitHub-comment markdown rendering instead of collapsing
  into flowing prose (#996, hive-mind #2146).
- Formalization tasks that quote their own source text («…», “…”, 「…」, 《…》)
  now formalize that text instead of silently substituting the seeded
  «Сказка о рыбаке и рыбке» tale; a quoted *title* of the tale still selects
  the full canonical text, and `FORMAL_AI_TRACE_REQUESTS=1` now also traces
  how the planner routed the received task (#956).
- The Russian liveness probes «ты тут?», «вы тут» and «я тут» are routed to
  the `test_status` intent instead of falling through to a web search (#979).
- The 22 duplicate requirement IDs in `REQUIREMENTS.md` are renumbered to
  fresh unique IDs (issue-540 block → R537–R548, issue-657 R480 → R549,
  issue-674 block → R550–R558) with every cross-reference in
  `docs/requirements-traceability.md`, the issue-540 case study, and its
  guard test updated (#964).

### Fixed

- Local location, conversation-preference, correction, associative-memory,
  British `behaviour`, and unquoted teaching prompts now stay on their seeded
  symbolic routes instead of falling through to unrelated web/document plans;
  failed web transports retain their real diagnostic, and asking what Links
  Notation is no longer starts document generation (#989).
- Agentic English narration no longer repeats the subjective word `quick` after
  a user rejects it (#989).
- GitHub issue reports can attach harness, server, and merged context as three
  separate links, with safe filenames and valid link-only Markdown (#989).

### Fixed

- Split the repeatedly timing-out Intel macOS test suite into complementary
  core and specification shards without raising its 35-minute budget, and add
  elapsed-time warnings before either shard reaches the cap (#999).
- Serialize repository writers across release, desktop, Pages, changelog, and
  benchmark workflows without cancelling in-flight writes (#999).
- Remove actionable CI warning debt: use the supported Pages timeout, classify
  intentional reports as notices, and bring every observed source/data file
  below its warning threshold (#999).
- Repair stale Links Notation, relative-meta-logic, and CommonsenseQA references
  exposed by the new link gate, with host-aware throttling for probe reliability
  (#999).
- Restrict Wayback diagnostics to Lychee's actual error section so successful
  redirects are not reported as broken, and replace an unavailable normal-
  algorithm reference with live university course material (#999).

### Security

- Add CodeQL, dependency-review, and broken-link/Wayback validation gates from
  the current language templates (#999).

## [0.339.1] - 2026-08-11

### Fixed
- Routed Desktop and supported Rust command-execution boundaries through the published `command-stream` component while preserving streaming output, cancellation, exit diagnostics, and host/Docker selection, with focused upstream limitations recorded for excluded boundaries.

## [0.339.0] - 2026-08-10

### Added

- Learn execution-verified coding procedures from licensed, provenance-bearing
  cached research after a program skill gap, with deterministic offline replay
  and failure-driven follow-up queries.

### Fixed
- Restored `cargo install formal-ai --locked` on stock Rust images by selecting web-capture's transport-independent search feature, removing transitive system OpenSSL build requirements.

## [0.338.0] - 2026-08-10

### Added
- Define and ratchet Formal AI's minimal-core handler boundary, and audit seed problem-solving metadata with coding-path completeness and per-record gap data (#918).

### Fixed
- Restored macOS CI parity for desktop packaging, canonical session diagnostics, PTY integration tests, and Bash 3.2 seed synchronization.

## [0.337.0] - 2026-08-09

### Added
- Unknown online requests now enter evidence-producing web research instead of
  stopping at an `unknown` answer, including imperative prompts without question
  punctuation. A data-defined research-learning cycle keeps disposable source
  captures separate from versioned knowledge, promotes only immutable-baseline
  passing candidates, restores earlier stable versions, and supports user-led,
  full-trust, and per-command recovery with a configurable one-hour default
  continuation boundary. ([#873](https://github.com/link-assistant/formal-ai/issues/873))

### Fixed
- The self-AST workspace aggregate is now rendered on demand instead of tracked,
  preventing unrelated source branches from repeatedly conflicting in the same
  generated `index.lino` while retaining per-module drift checks.
- Repository summarization now bounds optional concrete-syntax parsing for
  oversized traces, preventing seeded validation from spending hours on a
  single generated evidence file while still summarizing its full structure.
- Agentic GitHub reports now bound the readable transcript independently of the
  complete context attachment, so research tool results cannot exceed GitHub's
  issue-body limit after unknown inputs are promoted to online research.

## [0.336.0] - 2026-08-09

### Added
- Add side-effect-free persisted-memory compatibility preflight and explicit, locked, backed-up, atomic schema migration with JSON receipts and rollback guidance.

### Changed
- Expose memory schema compatibility through `/health` and preserve unknown event metadata across native load/export/write paths.

## [0.335.0] - 2026-08-09

### Added
- Natural statements in English, Russian, Hindi, Chinese, and Spanish can now
  project to seed-defined first-order logic and back through one
  Wikidata-grounded meaning.
  Native and browser translation paths share the same projection catalog and
  preserve round-trip identity without language-pair translators (issue #917).

## [0.334.0] - 2026-08-09

### Fixed
- Intent routing now reads the user's request rather than the caller's framing: the
  blocks a client wraps its own context in (`<session_context>`, `<system-reminder>`,
  `<environment_context>`, `<env>`, `<environment_details>`) are stripped before the
  turn is interpreted, so the gemini CLI's "Today's date is …" preamble no longer
  turns every agent-mode run into `run_shell_command({"command":"date"})` (issue #907).
- A declarative statement no longer fires a shell intent: a cue only routes when the
  sentence carrying it asks or commands, not when it states a fact about it
  ("Today's date is Sunday" vs "what is today's date?"), across the English, Russian,
  Hindi, Chinese and Spanish copulas declared in `data/seed/caller-context.lino`
  (issue #907).
- A turn that carries a task gets the task: a built-in intent riding alongside an
  authoring request steps aside instead of answering the smaller question and
  dropping the work (issue #907).

### Changed
- "Is this sentence asking, or telling?" now has a single home. The copulas, question
  words and request verbs that were duplicated between `data/seed/shell-intents.lino`
  and the caller-context vocabulary are declared once in
  `data/seed/caller-context.lino`, and classification runs per sentence rather than per
  cue occurrence — so a shorter cue riding inside a statement ("the current **time** is
  20:00") cannot route either (issue #907).
- The gemini CLI joins the required agentic E2E matrix.
  `experiments/agent_cli_e2e/run_issue_907.sh` drives the real client against
  `formal-ai serve --agent-mode` over the native Gemini routes, because the
  `<session_context>` framing that caused this bug only exists once a real client
  injects it (issue #907).

### Fixed
- `--global` no longer reports success when the configuration it just wrote
  cannot start the client: every file a headless start depends on — gemini's
  `~/.gemini/settings.json` with `security.auth.selectedType`, qwen's complete
  `OPENAI_API_KEY`/`OPENAI_BASE_URL`/`OPENAI_MODEL` triple — is now declared as a
  registry `headless_require` contract and read back from disk, so a silently
  incomplete install fails the run instead of surfacing later as
  `Invalid auth method selected.` ([#909](https://github.com/link-assistant/formal-ai/issues/909))

### Added
- `formal-ai with --global --verify <tool>` starts the configured client once
  non-interactively and fails when it answers with an auth refusal, instead of
  leaving the gap to surface later as an unrelated startup error. Clients that
  are not installed are skipped. ([#909](https://github.com/link-assistant/formal-ai/issues/909))

### Changed
- `data/seed/closure-generated-*.lino` shards are now content-addressed: each
  generated meaning is placed by `sha256(slug) % SHARD_COUNT` instead of filling
  shards in sorted order up to a line cap. Sequential fill made every shard depend
  on the size of everything sorted before it, so adding one token rewrote up to
  11 of 11 shards and `data/seed` conflicted in nearly every pull request; a new
  token now touches exactly one shard.
  ([#909](https://github.com/link-assistant/formal-ai/issues/909))

### Fixed
- The `deploy-pages` job no longer fails the pipeline when GitHub's Pages
  deployment queue is backlogged. `actions/deploy-pages` waited only its
  600 000 ms default, so a run whose artifact had uploaded successfully still
  aborted with `Timeout reached, aborting!` while the deployment was still
  `deployment_queued` — a red `main` pipeline that said nothing about the
  commit. The wait is now pinned to 1 200 000 ms and the job budget raised to
  35 minutes so the longer wait is not undone by a `timeout-minutes` kill.
  ([#909](https://github.com/link-assistant/formal-ai/issues/909))

### Fixed
- The `test-agent-cli-e2e` job no longer fails on ordinary runner variance. Green
  runs on `main` measured 16m16s and 17m30s against a 20-minute cap, so run
  31097339962 tipped over it and was reported as *cancelled* — a red pipeline
  that looked like a regression but carried no signal about the commit. The
  budget is now 32 minutes, roughly twice the observed cost.
  ([#909](https://github.com/link-assistant/formal-ai/issues/909))

### Changed
- `tests/unit/ci-cd/workflow_release.rs` crossed the 1000-line cap that
  `scripts/check-file-size.rs` enforces. The self-contained Desktop Release
  assertions moved to `tests/unit/ci-cd/workflow_release_desktop.rs`, putting
  both files back under the warning threshold with no change in coverage.
  ([#909](https://github.com/link-assistant/formal-ai/issues/909))

### Fixed
- Default-branch CI is green again: the total-closure regression test now
  passes `cargo fmt --check`, unknown-opener browser tests cannot be intercepted
  by live search providers, and permission-replay tests wait for the worker
  response before reading queued-task state. This removes one deterministic
  failure and one retry-masked false positive from run 31186108359.
  ([#980](https://github.com/link-assistant/formal-ai/issues/980))

## [0.333.2] - 2026-08-06

### Fixed
- Propagate failed Agent CLI tool results, retry a rejected write once after a
  read, run the named check before reporting an unrecoverable write, and require
  matching verification output before claiming a general file change completed
  (issue #905).
- Answer a completed general change in the language of the request: the
  completion claim now comes from a seeded `general_plan_completed` response in
  all five supported languages instead of an English string in Rust (issue #905).

## [0.333.1] - 2026-08-06

### Fixed
- Hindi and Chinese word-operator arithmetic no longer falls to the unknown handler: `2 जमा 2 कितना होता है?` and `2 加 2 等于多少?` now answer `4`, matching the English and Russian equivalents (#962). The `arithmetic_operation` meanings gained the bare infix operator surfaces (`जमा`, `बटा`, `加`, `减`, `乘`, `除`) alongside the compound forms they already carried, and the Hindi `calculation_result_query` cue list gained `कितना होता है` / `कितने होते हैं`.

### Fixed

- Spanish arithmetic prompts (`¿Cuánto es 2 más 2?`) fell to the unknown
  handler: the `arithmetic_operation` meanings carried no Spanish operator
  words and no Spanish `calculation_result_query` cue. Seeded `más`, `menos`,
  `por` / `multiplicado por`, `dividido por` / `entre`, `módulo`, and the
  `cuánto es` / `cuánto da` / `cuántos son` cues.
- The Spanish opening marks `¿` and `¡` are now trimmed as leading prompt
  punctuation. Previously `¿Cuánto es 2 + 2?` reached the typo responder
  ("Interpreted `¿Cuánto es` as `cuánto es`") instead of the calculator, which
  broke even the symbolic form in Spanish.

### Fixed

- Releases stopped reaching the `Create GitHub Release` step: the four release
  Docker build-push steps had no layer cache (unlike the PR-check build), so
  every release recompiled the whole crate inside Docker and blew the 30-minute
  job cap. Eleven versions (0.326.2 .. 0.333.0) shipped to crates.io and got git
  tags with no GitHub Release at all. All four steps now share the GHA layer
  cache, and the release jobs have a 60-minute budget.
- `E2E Tests (local web app)` spent 10m36s of its 15-minute budget inside
  `playwright install --with-deps`, fetching font packages from a ~30–60 KB/s
  Ubuntu mirror, and died at test 159 of 468. The browser install is now cached
  and the system-dependency install is a separate bounded, non-fatal step; the
  job budget is 40 minutes and the suite runs 4 workers under CI.
- Playwright's `globalTimeout` equalled the job's `timeout-minutes`, so the job
  clock always won: Playwright never aborted, never exited non-zero,
  `if: failure()` never fired, and no report artifact was uploaded. It is now
  well below the job cap, so a slow suite fails loudly with a report.
- A job killed by `timeout-minutes` is reported by GitHub as **cancelled**, not
  **failed**, which let eighteen consecutive `main` runs look benign. A new
  terminal `pipeline-status` gate now fails the run on any job failure, and on
  any cancellation on `main`, where concurrency cancellation is disabled and a
  cancelled job can only mean a timeout.
- Every workflow action pinned to the deprecated Node 20 runtime was bumped,
  clearing the deprecation warning annotations on each run.

## [0.333.0] - 2026-08-06

### Added
- `scripts/check-cache-budget.rs`: CI gate enforcing `MAX_SEED_RECORDS_PER_BUCKET = 128`
  for every bucket under `data/cache/`, reading the cap from `src/translation/cache.rs`
  so gate and library cannot drift. The three buckets whose size is forced by the
  total-closure gate are exempted explicitly, with a written reason and a stricter
  no-orphan invariant (issue #960).
- `scripts/check-tests-as-docs.rs` and `scripts/tests-as-docs-allowlist.txt`: burn-down
  ratchet requiring behavioural tests to assert exact answers instead of substrings,
  so a test reads as documentation (issue #960).
- `scripts/check-pull-request-link.rs`: fails a pull request whose description does not
  close its issue with a GitHub keyword, or writes `Addresses #N` where `Fixes #N`
  belongs (issue #960).

### Changed
- The 1500-line Links Notation cap now covers `data/cache/wikidata/` too;
  `scripts/check-file-size.rs` and `tests/unit/data_files.rs` no longer exempt cached
  data (issue #960).
- CONTRIBUTING.md and `.github/pull_request_template.md` codify the issue-linking
  syntax, the `docs/case-studies/pull-request-{id}` layout, the cached-`.lino` cap, the
  128-record cache budget, and the exact-answer test style (issue #960).

## [0.332.1] - 2026-08-05

### Changed
- Automated `solve` sessions on this repository now run with `--attach-logs
  --verbose`. `examples/self-coding/run.sh --live` passes both flags, and
  `CONTRIBUTING.md` records the canonical command plus why neither flag
  substitutes for the other: `--attach-logs` publishes the session log to the
  pull request, and `--verbose` is what makes the Agent adapter dump the raw
  JSON of every error and fatal-startup record
  (link-assistant/hive-mind#2143). The 2026-08-04 run on PR #927 failed in 22
  seconds and left only `AGENT execution failed with Agent reported error:
  [object Object]` with no log attached; that cause is unrecoverable, and a
  failure recorded that way is unlearnable by construction (issue #973).

### Added
- `tests/issue_973_solve_flags.rs`, which enforces the policy instead of only
  documenting it: it scans the guides and scripts the repository publishes and
  fails when any `solve` invocation drops either flag, naming the file, line,
  and missing flag. Recorded history under `docs/case-studies/`, `dev/log/`, and
  `experiments/` stays exempt, so past runs remain byte-for-byte as they
  happened.
- `docs/case-studies/issue-973/` — the timeline of PR #927's failure, root
  causes RC1–RC6 with the upstream reports (link-assistant/hive-mind#2141,
  link-assistant/agent#289, link-assistant/agent#290), and the captured GitHub
  API evidence under `raw-data/`.

## [0.332.0] - 2026-08-05

### Added
- An iterative repository-summarization validation protocol with a published quality
  metric and an 80% ratchet (`src/summarization/validation/`). A seeded
  `splitmix64` Fisher-Yates permutation draws repository files reproducibly, two per
  iteration, and the loop keeps going until three consecutive iterations sit within
  five points of one another above the ratchet — never before twelve iterations have
  run, because three perfect iterations are six files and six files say nothing about
  a corpus of ten thousand — or it stops at the iteration bound and reports
  `bound_reached` instead of claiming a stability it never observed. The
  ten criteria are scored as an exact integer `passed/applicable` ratio, floored, with
  criteria that cannot apply to a file dropped from that file's denominator rather
  than counted as free passes (issue #893, re-opening issue #563).
- `formal-ai summarization criteria | validate | ratchet` — the operator surface for
  the metric. `validate --append` writes the measured run to
  `data/summarization/quality-baseline.lino`; `ratchet` re-measures and fails when the
  score drops below the published 80% minimum or below whatever the repository last
  committed (issue #893).

### Fixed
- Markdown embedded grammars are now exercised through the production summarizer on
  every validation run, and counted against an *independent* CommonMark fence scanner
  so the summarizer cannot grade itself. A run that recorded no embedded grammar block
  may not declare stability and is rejected by the ratchet — and because fenced
  Markdown is rare enough that a uniform draw of the affordable size can miss it
  entirely (it failed one CI run at 100% measured quality), the draw is stratified:
  the seeded permutation is computed as before, then one fence-carrying Markdown file
  is promoted into iteration 0 (issue #893).

## [0.331.0] - 2026-08-05

### Added
- A non-decreasing coverage ratchet. `scripts/check-coverage-ratchet.rs` reads the
  LCOV report CI already produced, publishes a human-readable
  `coverage/summary-<name>.md` (per-metric table, deltas in percentage points, the
  ten least-covered files) into the run summary alongside a machine-readable
  `coverage/summary-<name>.json`, and fails the build when a percentage drops below
  the reviewed floor in `coverage/baseline.json`. Raising a floor is
  `--update-baseline`; lowering one is refused unless `--justification "<reviewed
  reason>"` records the decision in the file, so a decrease reaches review as a
  sentence in the diff rather than two digits changing (issue #895, epic #710).
- Coverage of the browser production sources, which had none. `tests/web/` loads the
  unbundled `src/web/` page scripts and the 24-module worker mirror into a `node:vm`
  sandbox under their real repository paths — which is what makes V8 attribute
  coverage to the files the browser downloads — and boots the worker through its real
  entry point with the canonical `data/seed/*.lino` corpus behind `fetch`. 48 new
  tests; `npm run test:web` and `npm run coverage:web` run them, and a new
  `browser-coverage` job in `.github/workflows/coverage.yml` enforces the browser
  denominator.
- `coverage/browser-unmeasured.txt`, a committed `path<TAB>reason` inventory of every
  `src/web/**` file the browser denominator does not measure. A file that is neither
  measured nor declared fails the build, as does a stale, redundant or unexplained
  row, so the denominator cannot be quietly narrowed to flatter the number. The list
  can shrink; it cannot grow silently.

### Changed
- The Rust and browser denominators are ratcheted separately and never averaged into
  a single figure: a large Rust suite would otherwise mask an untested website while
  the combined number went up. `docs/design/coverage-ratchet.md` documents the
  measurement, the publication format, and the baseline-update policy.
- Coverage now runs from `.github/workflows/coverage.yml` instead of the release
  pipeline. Nothing in the release graph depended on the job, so the move changed no
  ordering; it puts one job per denominator in the checks list, and it returns
  `release.yml` to 1930 lines, back under the 2000-line ceiling
  `scripts/check-file-size.rs` documents as debt that must not grow.

### Fixed
- The coverage job's timeout, raised from 25 to 40 minutes. Issue #812 set 25 against
  a worst case of 14.1 minutes, but the instrumented suite has since grown to
  17.2–19.6 minutes across the last eight green runs on `main` — 78% of the budget,
  the same one-slow-run margin #812 was filed about — and it hit the cap outright.
- `npm run test:web` and `npm run coverage:web`, which passed `tests/web/` to
  `node --test`. Node 20 recurses into that directory, but the Node 22 the workflow
  pins resolves it as a module path and fails with "Cannot find module". Both scripts
  now use `tests/web/*.test.mjs`, which the shell expands identically on either
  version.

## [0.330.0] - 2026-08-05

### Changed
- Spider-Man release-order answers are no longer a frozen sentence in
  `data/seed/facts.lino`. They are rendered at question time from a
  source-backed snapshot: a checked-in SPARQL query against the Wikidata Query
  Service (`data/cache/wikidata/query/spider-man-title-role-films.rq`), its raw
  answer, and the cached Wikidata labels of every film (issue #892).

### Added
- `data/seed/release-timelines.lino`: a general release-timeline registry —
  per-language answer wording plus, for each timeline, its source, query, cache
  file, snapshot date, freshness window, SHA-256, and the dated works with their
  localized titles. `scripts/ground-release-timelines.py` regenerates it from
  the cache (`--check` verifies, `--refresh` re-fetches).
- `formal_ai::release_timeline`: renders a timeline for a language as of a given
  day, ordering released works by date, listing announced ones separately, and
  switching to stale wording once the snapshot outlives its freshness window.
- `release_timeline:*` evidence links recording which snapshot an answer was
  computed from, its digest, the released/announced counts, and staleness.

## [0.329.1] - 2026-08-05

### Changed
- The four-template CI/CD audit report
  (`docs/case-studies/issue-479/template-comparison/REPORT.md`) no longer ends with
  drafted-but-unfiled recommendations. Every finding was revalidated against the
  current template default branches (2026-08-05) and the *Recommended upstream
  issues to file* section is replaced by an *Upstream filing status* ledger that
  carries a status (`confirmed` / `obsolete` / `not-applicable` / `local`) and, for
  every confirmed row, the upstream issue URL. Eight issues were filed upstream —
  CI security scanning in all four templates, the `links.yml` broken-link checker in
  the Rust/Python/C# templates, and an optional desktop-release workflow in the Rust
  template — each with a reproduction, a workaround and a suggested fix. Findings
  that closed since the June snapshot (API-docs deploy, published-crate smoke test,
  resilient buildx, main-safe concurrency) are marked obsolete with the evidence
  that closed them (issue #894).

### Added
- `tests/unit/docs_requirements_issue_894.rs` parses the filing ledger and fails
  when a `confirmed` finding carries no upstream issue URL, when a status outside
  the documented vocabulary appears, when the pre-filing recommendation section
  returns, or when the preserved revalidation evidence goes missing (R894-4).

## [0.329.0] - 2026-08-05

### Added
- A write-effect coding ladder (`experiments/issue_916_write_effect_ladder`) that
  executes every planned `write_file` and `run_shell_command` for real in a throwaway
  workspace and passes a rung only when the declared effect is observable on disk.
  `.github/workflows/write-effect-ladder.yml` enforces the score as a monotonic
  ratchet in the style of the issue #408 gate, so a rung that was green can never
  silently go red (epic E69, issue #916).
- `formal-ai with --global gemini` now also writes `~/.gemini/settings.json` with the
  selected authentication type, and `--undo` restores it from its backup: the
  environment variable alone is only a *default*, so the configured client still
  refused to start headlessly (issue #909).

### Fixed
- A tool result's exit code is now the primary success signal. `Exit Code: 0` with
  `Output: (empty)` / `Error: (none)` — how `python3 -m py_compile` reports success —
  is no longer read as a failure, and `Exit Code: 1` alongside plausible-looking
  output is no longer read as success (issues #905 and #908).
- Failure reports name the failing command and the code it exited with instead of
  blaming the harness, so exit codes propagate to the reported outcome (issue #908).
- A read of a file that is not there is reported as a failure, in the language the
  request was written in, instead of being answered with "Contents of `hello.txt`:"
  wrapped around the raw transport envelope. Only the shell route consulted the
  harness's exit code, so a client that advertised a typed read tool got the English
  false success for a Russian, Hindi or Chinese request (issues #905 and #916).
- A general change request is no longer reported as "completed and verified" when the
  verification command it named exited non-zero; the observed failure replaces the
  claim (issue #905).
- An adverbial qualifier that delimits literal content ("containing exactly: Hello
  World", "содержащий ровно: …") is no longer captured as part of the bytes to write
  (issue #905).
- A client's own framing block — Gemini CLI's `<session_context>`, Cline's
  `<environment_details>` — is stripped from the user request like `<system-reminder>`
  already was, and a declarative statement of fact ("Today's date is …") no longer
  fires the shell intent whose cue it happens to contain, so the request that follows
  it is the one that gets planned (issue #907).
- `formal-ai with --global qwen` writes the complete OpenAI triple (`OPENAI_API_KEY`,
  `OPENAI_BASE_URL`, `OPENAI_MODEL`), which is what qwen-code needs to select the
  OpenAI authentication path unattended (issue #909).

## [0.328.0] - 2026-08-04

### Added
- Equation-type corpus with a CI ratchet (issue #891, requirement from #406):
  `data/benchmarks/equation-type-corpus.lino` defines 72 distinct equation
  types — one-step and multi-step linear, `?`/`*` placeholder unknowns,
  symbolic multi-variable isolation, polynomials up to degree five,
  natural-language wrappers in every registered language, and
  evaluation/percent flavours — each carrying the exact answer observed from
  the production solver. `issue_891_equation_corpus_solves_every_type` replays
  every case through `FormalAiEngine::answer` and fails below the recorded pass
  count or below 50 distinct verified types.
- Ten recorded `benchmark_limitation` records (irrational and complex roots,
  contradictions, malformed input, identities, unit-carrying equations,
  named-unknown declarations, command-shaped prompts) asserted to keep
  declining loudly rather than fabricating an answer.

### Fixed
- Equation-solving request cues in `data/seed/meanings-calculator.lino`: "solve
  the equation" / "solve equation" (en), "реши/решите уравнение" (ru), "解方程"
  and "求解" (zh), and "समीकरण हल करें/करो", "हल करें/करो" (hi) are now stripped
  before delegation, so `Solve the equation 2 * x + 3 = 11` and its Russian,
  Chinese and Hindi equivalents solve instead of returning a parse error. The
  `calculation_request` meaning also gains its first Spanish lexeme
  ("resuelve la ecuación", "resolver la ecuación", "cuánto es", "resuelve",
  "calcular", "calcula"), so Spanish calculation prompts route at all. The
  cues are seed data, so the Rust engine and the JavaScript worker gain them
  from the same source.

## [0.327.0] - 2026-08-04

### Fixed
- Thinking traces are now written in the language of the answer on every non-UI surface (issue #889, parent #710). The browser panel was already localized through the web i18n catalog, so a Russian, Hindi, Chinese or Spanish answer arrived with an English explanation of how it was produced everywhere else:
  - the sentences a trace is made of moved out of `src/thinking.rs` into seed data (`data/seed/multilingual-responses-thinking.lino` and `…-thinking-narrative.lino`), translated into every registered language, so adding a language means adding records rather than editing Rust (R379);
  - the CLI `--thinking` trace (including its heading), the OpenAI Chat Completions `reasoning`/`reasoning_content` fields, the OpenAI Responses reasoning item, the Anthropic extended-thinking block and the Telegram expandable blockquote all narrate in the resolved answer language, which is derived from the trace itself;
  - the language names inside the trace are localized too, so a Russian trace reads «Определить язык запроса: русский.» instead of naming the language in English;
  - the machine-readable `step`/`detail` trace keys and the step ids stay language-neutral, so downstream consumers never have to parse prose.

## [0.326.3] - 2026-08-04

### Fixed
- Step verification in the agentic command reroute now reads the exit code the harness reported instead of guessing from the shape of the output (#908). A verification command that exits `0` without printing anything — `python3 -m py_compile`, `tsc --noEmit`, `diff -q` — is a success, and a command that exits non-zero is a failure even when it printed output. Prose markers decide only when the harness reported no exit code at all, and an `Error: (none)` placeholder field no longer reads as an error.

### Changed
- A failed step is now reported as `Step \`<command>\` for \`<file>\` failed with exit code <n>` in every registered language, instead of the English-only claim that "the agentic CLI harness could not complete" the file — the harness had run the command exactly as asked.

## [0.326.2] - 2026-08-04

### Fixed
- The implementation-language router now validates what fills the `in <language>` position instead of taking whatever word follows `in` (issue #906):
  - a closed-class word is no longer read as a language name, so "Create a file named `hello.txt` in the current directory…" is no longer routed as a request in language `the` (an unknown *name* such as `elvish` is still read, so the engine can report what it was asked for);
  - a request that names no language gets its own answer — "I will not guess an implementation language… name the language" — with its own intent, event and response link, so the internal `missing` sentinel never reaches the reply;
  - the modifier modifies the request instead of replacing it: "Fix the failing CI job in Rust." is answered about the CI job rather than with an encyclopedia definition of Rust;
  - a request whose artefact is a *file* is no longer normalised into a `write_program` request, so the path and the content survive into the change plan instead of being replaced by `console.log('Hello, world!')`.

## [0.326.1] - 2026-08-04

### Fixed
- Agent mode no longer reports success for a repository work item whose only steps
  wrote the plan record and read it back: the self-referential verification command
  is gone and such a plan now ends in a `planned_not_executed` terminal state with a
  "Planned, not executed" answer (issue #904).
- A composed plan's `goal` is now the objective stated after the documented request
  lead (`Issue to solve:`, `Task:`, `Goal:`, and their Russian, Hindi and Chinese
  surfaces) instead of the caller's whole system-prompt preamble (issue #904).

## [0.326.0] - 2026-08-04

### Fixed
- `formal-ai with <tool>` now parses the caller's arguments into a structured request (prompt plus options) and re-renders it in each client's own vocabulary instead of concatenating strings (issue #903):
  - an already-qualified `provider/model` selector is no longer given a second `formalai/` prefix;
  - everything after the tool name is forwarded to the client, so a flag the wrapper also defines (`--verbose`) is no longer swallowed;
  - interactive mode follows `isatty(stdin)`, so a piped, headless run is never given `--interactive`;
  - a piped prompt is rendered as that client's prompt argument, so `codex` keeps its `exec` subcommand and is never handed `-p`;
  - the completion ladder re-renders the caller's own option set with only the prompt substituted, so `--dangerously-skip-permissions`, `--mcp-config` and `--disallowedTools` survive a retry and the wrapper's overlay is no longer duplicated.

## [0.325.0] - 2026-08-03

### Changed
- Route native and browser research through the published `web-search` and
  `web-capture` component contracts while retaining exact-byte cache replay,
  cancellation, provenance, diagnostics, and bounded compatibility fallbacks.

## [0.324.3] - 2026-08-03

- Add the issue #914 case study with the full requirement list (R914-1 to
  R914-15), per-requirement solution plans, external component research,
  and the E69-E77 epic bodies with opened-issue records (#914)
- Record the ninth-pass requirement-status audit in ROADMAP.md, correcting
  rows that went stale after the 2026-07-14 pass, and add the issue #914
  requirement table to REQUIREMENTS.md (#914)
- Guard the planning documentation with the docs-traceability test
  `issue_914_case_study_and_planning_docs_are_traceable` (#914)

## [0.324.2] - 2026-08-03

### Fixed

- Keep the Codex wrapper connected to Formal AI when callers pass their own configuration overrides after the exec subcommand.

## [0.324.1] - 2026-08-03

### Added

- Added issue #531 pattern-inference research artifacts, requirements tracing,
  upstream Data.Doublets.Sequences notes, and a unit test that keeps the case
  study connected to `REQUIREMENTS.md`.

### Added

- Implemented issue #531 pattern inference on a self-contained, link-native
  sequence substrate (`src/sequences/`): a doublet store with structural
  deduplication and lossless expansion, unique symbols, a balanced converter with
  a sequence index and frequency cache, and a Re-Pair-style associative
  compressor with an auditable trace.
- Added 1D sequence and 2D grid pattern inference — repetition, period,
  palindrome, reversal, and translation for sequences; horizontal, vertical, and
  diagonal symmetry, rotations, reflections, and translations for grids — surfaced
  through a `pattern_inference` solver method that analyses concrete "find the
  pattern" / "what comes next" prompts and predicts the next element.
- Seeded a pattern-inference ontology (`sequence`, `pattern`, `repetition`,
  `compression`, `deduplication`, `symmetry`, `rotation`, `reflection`,
  `translation`, `analogy`, `invariant`, `transformation`) rooted in links and
  closed by the total-closure resolver.
- Localized the pattern-inference report into every seeded language (en, ru, hi,
  zh, es): a response-language follow-up ("answer in Russian") now re-renders
  the 1D sequence and 2D grid analysis — classification, counts, compression,
  next-element prediction, and grid symmetry labels — in the requested language
  instead of stranding it in English.
- Discover parameterized reusable algorithms from repeated event, memory,
  Agent-CLI, and compiled-guide traces; validate later occurrences as held-out
  evidence; retain proposal-only candidates during idle learning; and expose
  integrity-checked learning/conformance commands without implicit execution.

### Fixed

- Fixed balanced conversion of exactly two link addresses, which previously
  indexed past the reduced one-element layer.
- Fixed the language-test coverage guard so Spanish names and native-language
  labels count as evidence for the registered `es` locale.
- Added Spanish to the verified Wikidata grounding pipeline and checked in the
  `Q1321` source snapshot required for offline semantic-grounding closure.

## [0.324.0] - 2026-08-03

### Added

- Translate solved integer-interval proofs through a language-neutral meaning into executable Rust or Python programs, with requests supported in every registered natural language (#890).

## [0.323.0] - 2026-08-03

### Added

- Proactively offer an opt-in, contextual issue report when Formal AI detects failed reasoning, provider execution, or tool execution across the UI and agentic coding harnesses. Expected refusals and pending approvals remain non-failures. ([#864](https://github.com/link-assistant/formal-ai/issues/864))

## [0.322.0] - 2026-08-03

### Added
- Teach Formal AI to generate and observe bounded compiler-valid Rust source,
  focus repository searches, and ground collection edits in workspace bytes
  for issue #848's 130-task coding ladder.
- Add bounded identifier rewrites and ordered multi-file module changes that
  use compact Agent edits or validated replace-all operations and advance only
  after observing the exact requested workspace digest.
- Add a proposal-only workspace-change learning frontier whose reusable
  procedures require distinct verified executions, a zero-failure gate, and
  named human approval before entering the content-addressed ledger.

### Fixed
- Make coding-ladder results reject echoed source prose, stale release oracles,
  incomplete runs, pre-existing Rust targets, filtered-score overwrites, and
  false startup failures caused by diagnostic text read from task files.
- Preserve explicitly exact literal-file payloads when their bytes resemble an
  identifier rewrite, instead of routing the new-file request as an edit.

## [0.321.0] - 2026-08-02

### Added

- Added evidence-oriented multi-jurisdiction file-legality reports, Exif/GPS
  provenance, independent detector adapters, a fail-closed authorized hash
  boundary, and the `formal-ai file-legality` JSON CLI.

## [0.320.0] - 2026-08-02

### Added
- Fuse captured multi-source search results into ranked, cross-language statements with per-source provenance and explicit conflicts on web, CLI, HTTP, and Telegram (#709).

## [0.319.2] - 2026-08-02

### Fixed

- Fixed Codex hello-world coding so custom apply_patch creates the source file, exec_command compiles and runs it with precise narration, and Report issue asks for details instead of searching the web.

## [0.319.1] - 2026-08-02

### Fixed
- Support Claude Code's returning-user `/recap` request with a goal-led, under-40-word plain summary, while excluding client-injected reminder metadata from conversation history ([#858](https://github.com/link-assistant/formal-ai/issues/858)).

## [0.319.0] - 2026-08-02

### Added

- Compile, review, edit, and execute bounded multilingual associative-memory
  programs with explicit permissions, append-only retractions, execution traces,
  honest program gaps, and matching native/browser behavior.
- Query every memory field through exact SQL or GraphQL CRUD, grouping, and
  statistics that share one typed plan, lower to bounded link substitutions,
  run across native/browser/Agent-facing surfaces, and support human-gated
  learning from repeated exact-backed natural-language examples.

## [0.318.0] - 2026-08-01

### Added
- Context-aware statement auditing now records relative references, resolved claims, and antecedent-bounded probabilities.
- Added source-backed Formal AI/LLM, public-domain output, dataset, model, and philosophy guides with a fail-closed legal source-review workflow.

## [0.317.0] - 2026-07-31

### Added
- Add a seed-driven N→N+1 language protocol, generated round-trip matrix,
  partial Spanish coverage with explicit gaps, and an Arabic dry-run fixture.

### Changed
- Derive language detection entirely from `data/seed/language-detection.lino`
  instead of a hardcoded Rust enum, so registering a language (script, Unicode
  range, markers, fallback flag) is a data-only edit shared by the Rust core,
  the WASM worker, and the JS worker.
- Move the unknown-intent opener pools out of Rust and JavaScript constants
  into `data/seed/unknown-openers.lino`, and derive the browser worker's
  no-WASM fallbacks — script/marker detection and the known-response-language
  check — from the hydrated registry, so a new language needs no worker edit.
- Apply the ledger's `explicit_gap` fallback policy through
  `seed::localized_response`: an intent with no text for a registered language
  now surfaces the explicit "unsupported language" record instead of silently
  answering in English.

### Added
- Record the language learning frontier: `data/language-additions/<code>.lino`
  can now carry a prompt corpus, and `src/language_frontier.rs` runs the live
  engine over it to record only the prompts that still fail, keeping a language
  without a corpus as an explicit `frontier_gap`.
- Register `--frontier` as an open registry in `formal-ai learn cycle`, so the
  issue-#701 learning cycle replays the new `language-gap` frontier with no new
  learning logic. Over the Spanish corpus it derives, validates on held-out
  prompts, and proposes the `qué es …` and `cuéntame sobre …` request frames.
- Pin the adoption evidence in `data/meta/language-adoption-ledger.lino`: 7 of 7
  recorded Spanish prompts leave the unknown path and recover their term after
  the proposals are adopted as seed data.

### Changed
- Move the unknown-intent opener pools into `data/seed/unknown-openers.lino`,
  shared by the Rust core, the WASM worker, and the JS worker.
- Derive language display names, concept slugs, and per-language script checks
  from the seed ledger instead of Rust `match` arms.

## [0.316.1] - 2026-07-31

### Fixed

- Require one-shot software-authoring clients to produce a workspace effect, escalate through a seeded ladder of distinct native-session corrections when they do not, learn across runs which correction actually produces artifacts for each client, keep `.formal-ai/` out of user changes, fail closed on public endpoint diversion, and emit strict completion NDJSON with endpoint and usage metadata across Agent, Claude, OpenCode, Codex, Qwen, and Gemini.

## [0.316.0] - 2026-07-31

### Added

- Add permission-gated, isolated, and replayable non-visual computer-use plans
  across the native CLI, MCP server, universal agent planner, and desktop,
  including twelve typed primitives, structured HTTP/DOM provenance, ten
  deterministic multilingual tasks, and honest GUI-rendering capability gaps.

### Added

- Induce computer-use plan schemas from the recorded example tasks and
  synthesize verified plans for requests never seen before, ratcheted by twelve
  held-out four-language cases that also run through the real external Agent
  CLI, with the induced schemas committed as drift-tested evidence and the
  observe/induce/bind/synthesize/verify/refuse loop recorded as a grounded
  meta-recipe.

### Fixed

- Stop computer-use plan synthesis from inheriting another example's state:
  resource-scoped arguments (`selector`, `pointer`, `column`, `equals`) now come
  only from the learned resource binding, operation constants are gated by both
  the primitive's advertised schema and the learned operation schema, and a
  request the corpus never evidenced yields an honest refusal instead of a
  plausible wrong plan. Recognition also runs over the request's instruction
  surface rather than content it merely transports, so words inside an indented
  structured block or a quoted literal no longer plan a run of their own.

## [0.315.0] - 2026-07-31

### Added

- Add deterministic parallel candidate-solution portfolios as a domain-independent
  engine: seed-declared draft strategies, a `PortfolioLeaf` trait implemented by
  both arithmetic reachability and rule synthesis, per-draft test ledgers,
  least-action selection, composition backtracking, and multilingual winner
  explanations (issue #704).
- Mine durable `draft_failure` records into per-strategy dreaming-loop lessons,
  so a losing draft becomes retained learning instead of a discarded attempt
  (issue #704).

## [0.314.0] - 2026-07-31

### Changed

- Start issue #699's generality-first handler migration with a complete,
  machine-checked ledger and monotonic 38-file/50-`try_*` ratchet. Number
  constraint recognition is now multilingual seed data, while interval solving
  and proof invocation remain an explicitly justified native primitive.

### Changed

- Continue issue #699's handler migration. `who_is` no longer stores
  misspellings — corrections are derived by nearest-surface search over
  remembered names — and `definition_merge` renders every label from seed data.
- An underivable `write_program` request now fails with a named skill gap
  instead of reciting the curated template catalogue. The gap identity travels
  in the evidence trail as a `skill_gap` event and the reply is seeded in
  en/ru/hi/zh, in both the Rust engine and the browser worker.

## [0.313.0] - 2026-07-30

### Added

- Added a default-denied, workspace-scoped external-agent controller for Agent
  CLI, Claude Code, Codex, Gemini CLI, Qwen Code, and OpenCode, with isolated
  parallel candidates, allowlisted verification, canonical replayable sessions,
  deterministic comparison ledgers, bounded task decomposition, and an opt-in
  real-client compatibility gate.
- Added separately allowlisted custom CLI/TUI, Bash, and local-model
  entrypoints for single or multi-agent dispatch. A registered CLI label cannot
  bypass the executable grant.
- Added seed-defined native resume contracts for all six clients and
  `formal-ai agent resume`, which carries disproving evidence into the exact
  parent conversation and rejects a changed native session id.
- Added meta-language synthesis, statement-level cross-checking, summaries,
  correction requests, and provenance-verified output-language translation for
  `en`, `ru`, `hi`, and `zh`. Model agreement is labelled a preflight, not
  external fact proof.
- Added proposal-only learning from canonical orchestration sessions and a
  byte-pinned Formal AI → Agent CLI → Formal AI chain that corrects two observed
  failures in the same native session.

## [0.312.1] - 2026-07-30

### Added

- `formal-ai learn cycle --frontier google-trends [--dry-run] [--proposals]`: one auto-learning cycle that takes the recorded frontier of unanswered trending prompts, derives candidate request-opener surfaces, validates each against held-out prompts, and emits promotion proposals in the issue-#656 shape. The run is deterministic, offline, proposal-only, and human-gated.
- `data/meta/learning-adoption-ledger.lino`: the adoption ledger — a before/after capability delta for every item the cycle adopted (60 pairs across 10 topics and 4 languages), generated by `cargo run --release --example issue_701_adoption_ledger`.
- A scheduled `Learning cycle (proposal only)` workflow and an idle-time dreaming hook that replay the cycle periodically and publish proposals for review without ever writing a seed file.

### Changed

- Retained dreaming amendments now change how a task is *solved*, not only how it reads: a task covered by a standing requirement is no longer classified as unresolved. The three application paths (solver, memory recall, agentic final) share one selection rule and one projection.
- The Google Trends learning frontier is empty: all 80 catalog prompts (10 topics × 4 languages × 2 variations) now route through the engine, up from 20. The pre-adoption verdicts stay durable in `data/meta/learning-frontier-google-trends.lino`.

## [0.312.0] - 2026-07-29

### Added
- Statement-level summarization of many sources (issue #844): `formal_ai::summarization` gains `dedup` (one `MergedStatement` per fact, a retractable `MergeLink` per absorbed sentence, `DedupReport::split` to undo a conflating merge), `importance` (the kind prior blended with observed frequency, source authority, and stance, with zero ranking evidence from unoriginal mirrors), `gathering` (a recursive unmet-difference loop bounded by depth and terminating at a fixpoint, with a content-addressed `SourceCache` that replays byte-identically), `recheck` (a verdict per fact, so an unsupported statement is withheld from the summary but kept), and `context` (`merge_into_context` builds a `world_model::Context`, not a list: a probability per statement and mutual `Contradicts` edges for every disagreement).
- An identifier rung below the topic rung: `SummarizationMode::Identifier` renders a label through `summarization::identifier::to_identifier`, which honours a `NamingConvention`, an `IdentifierBudget`, and the seed's reserved words.
- `SummarizationConfig::keeping_boilerplate()` keeps `install`/`example` sentences in the output, for a merged context where the install command is the answer rather than boilerplate.
- `data/seed/multilingual-responses-summarization.lino` holds the merge's reader-facing wording — the evidence summary, the denial clause and the disputed wrapper — in English, Russian, Hindi and Chinese, read back through `summarization::vocabulary::rendered_response`, so no summary sentence is hardcoded in Rust (R379).
- `cargo run --example issue_844_statement_merge` walks the issue's Stack Overflow case end to end — recursive gathering over a citation cycle, warm-cache replay, evidence-ranked merge, reported disagreement, recheck, and the ladder down to a single identifier. Documented in `docs/case-studies/issue-844/`.

### Fixed
- `world_model::Context::recalculate` no longer reports a claim and its denial as both probable. A contradiction whose two sides each carry saturating support makes the JTMS update the exact swap `x ← 1 - x`, which oscillates until the pass bound and returned whichever half the last pass landed on. Repeated states are now detected and the cycle is collapsed to its mean, verified as a fixpoint: two original sources that flatly disagree settle at `0.5`.
- `summarization::formalize` no longer splits a sentence inside a token, so `crates.io`, `docs.rs`, and `1.96` stay one term instead of becoming a spurious extra statement.
- `summarization::SourceCache` keeps provenance per URL alongside content-addressed bodies, so an unoriginal mirror of a first-party page no longer inherits the first party's source tier.

### Added

- Task decomposition is now a working task: "split this task into subtasks",
  "is this task atomic?" and "what is the first step?" are answered in every
  supported language from one bounded, deterministic recursion whose sub-tasks
  each carry an observable completion criterion and appear as inspectable
  `sub_impulse` events.
- Issue-sized requests now descend through reviewed data-backed strategies
  instead of being misreported atomic from punctuation alone. Inspected plans
  round-trip as content-addressed artifacts into the existing recursive
  executor, and failed executions can propose durable strategy learning only
  through green tests plus explicit human review.

## [0.311.1] - 2026-07-29

### Fixed

- Replace fabricated external-source and cache-hit evidence with an opt-in,
  SHA-256-verified source cache whose exact bytes and provenance replay
  offline, and connect captured content to HTTP fetches, search fusion, option
  observation, statement verification, and statement audits (#843).

## [0.311.0] - 2026-07-29

### Fixed
- Multilingual definition-example requests now use one slot-driven meanings rule and the open-web research route in English, Russian, Hindi, and Chinese; the previously unhandled Russian ladder node no longer falls through to a generic refusal (#842, #827).
- A word-meaning question whose subject is only a contextual pronoun now asks for the missing antecedent instead of searching the public web for the pronoun (#842, #827).

### Added
- The issue #840/#842 task ladder now judges assistant-authored claims separately from raw tool results, plus refusals, capability menus, tool routes, and shell-command shape. Its deliberately noisy web fixtures exercise synthesis without leaking page furniture.
- The ladder baseline is an appendable stable-ID ratchet: no recorded node can disappear or regress, and every newly reported node must pass before the baseline advances. The strict measurement is 24/24, up from the historical 8/24, with L1 and L4 both at 100%.
- Every run derives an evidence-bearing learning proposal from each failed node and retains it in CI. Proposals remain `awaiting_human_review` and cannot promote behavior until the complete ratchet passes.

## [0.310.0] - 2026-07-29

### Added
- External benchmark harness that downloads immutable upstream suite revisions at run time and grades the solver by the upstream criterion: HumanEval, MBPP, GSM8K, MATH, BIG-bench object counting, CoEdIT and a SWE-bench Lite slice. Cached payloads carry source-reference and content-hash provenance under `target/formal-ai-benchmarks` and are never vendored (issue #698).
- `formal-ai benchmark list | run | ratchet` commands, with `--suite`, `--slice`, `--append`, `--learning-report`, and `--base-ref` for publishing measured runs, review-gated failure-learning proposals, and base-branch comparisons (issue #698).
- Committed results ledger `data/benchmarks/external-results.lino` recording date, suite, slice size, pass count and solver version per run, plus explicit `benchmark_unavailable` entries with the blocking reason instead of a substituted local proxy (issue #698).
- Monotonic per-suite ratchet against the pull request's base-branch ledger, so a pull request cannot reduce any recorded upstream pass count, and the weekly `external-benchmarks` workflow that refreshes the ledger and verifies the ratchet (issue #698).
- Official pinned SWE-bench evaluation that applies non-empty candidate patches in the upstream container harness and executes repository tests; infrastructure failures remain `benchmark_unavailable`, and the obsolete gold-patch-equality proxy is withdrawn (issue #698).
- `docs/benchmarks.md` § "External (upstream) results" with the honest first measurement, and the case study in `docs/case-studies/issue-698` (issue #698).

## [0.309.0] - 2026-07-28

### Added
- Freely phrased multi-step procedures now reuse the solver's intent formalization and shared source-span decomposition before lowering every ordered requirement into a typed executable operation (issue #674).
- The compiled program carries canonical slugs only, so the English, Russian, Hindi, and Chinese phrasings of the same procedure content-address to one identical set of skill links (issue #674).
- Complete `.lino` procedure artifacts now round-trip through integrity validation and a generic host interpreter; the solver, later "why?" explanation, and Agent planner consume that persisted artifact instead of recompiling prose (issue #674).
- A step outside the vocabulary compiles nothing at all and records a complete review-only learning proposal. Successful seeded paraphrases let the learner infer one typed multilingual candidate without being handed its canonical operation; aliases enter the durable, evidence-bearing ledger only after a green regression suite and explicit human approval (issue #674).
- Formal AI's Agent path writes the same compiled artifact, reads it back, executes it through the public conformance CLI, verifies every step outcome, and returns its source-cited restatement; a reproducible external Agent CLI replay preserves byte-exact artifact and execution evidence (issue #674).
- Every sentence the procedure compiler shows the user — compiled output, later explanation, named gap, and proposal notice — is seeded prose under `compiled_procedure`, `compiled_procedure_explanation`, `skill_gap`, and `skill_gap_name` in en/ru/hi/zh (issue #674).

### Fixed

- Bound Rust development/test artifact growth and replace whole-`target`
  GitHub Actions caches with sccache compiler-output caching.

## [0.308.0] - 2026-07-27

### Added

- Added deterministic, disproof-first fact checking with named formal-system probabilities, tiered evidence fallback, JTMS dependency traces, and explicit permission for general-memory audits.

## [0.307.1] - 2026-07-27

### Added
- Capture two independently worded real OpenCode, Claude Code, and Codex terminal sessions as lossless transcripts, styled frame data, asciicasts, exact-grid SVG snapshots, CSS-keyframe SVG replays, and GIF fallbacks in agent CLI CI, preserving partial captures when a run fails.
- Exercise the report multiselect and a representative task-ladder node through the real OpenCode TUI.
- Learn stable TUI replay facts through the human-gated client-contract learner and prove the same task through a real Agent CLI with byte-identical output.

### Fixed
- Seed Claude's ephemeral configuration with the correct JSON onboarding value so interactive sessions do not repeat setup prompts.
- Consume the published `agent-commander` 0.10.1 and `command-stream` 0.17.2 renderer fixes instead of Formal AI's lossy local terminal stack, including exact visible-text geometry without padded SVG text runs and clean consumer installs.
- Upgrade all direct Rust and JavaScript dependencies to their latest compatible releases.

## [0.307.0] - 2026-07-27

### Added
- Dialogue world model (issue #702): `src/world_model_atoms.rs` classifies each
  turn (fact / wish / confirmation / correction / state query) from the
  `world_state_*` cue sets in `data/meta/cue-lexicon.lino`, and
  `src/world_model_dialog.rs` maintains current and target contexts as links
  networks with provenance back to the turn that asserted each fact, a
  hash-chained append-only synchronization log, merge with conflict detection,
  split, relative-meta-logic recalculation of dependent statements, and
  `forecast` for action-consequence prediction.
- `world_state` chat handler: "what is left to do?" (en/ru/hi/zh) is answered
  from the current→target difference and backed by `world_state:*` evidence
  links, never from remembered prose.
- bAbI-style world-state tracking benchmark slice
  (`data/benchmarks/world-state-tracking-suite.lino`, 16 self-authored dialogues
  with held-out paraphrases) with a `minimum_pass_count` ratchet, catalogued in
  `docs/benchmarks.md`.
- Case study `docs/case-studies/issue-702/` and requirements R702-1 … R702-10 in
  `REQUIREMENTS.md`.
- Unlimited nested symbolic contexts with full, isolated, or conditional
  inheritance; lazy nearest-first reference resolution returns an inspectable
  local trace and requires explicit permission before an outside lookup.
- Review-gated issue-702 auto-learning report derived through the shared
  associative-memory pipeline and reproduced through the real Agent CLI.

### Changed
- `SolverConfig` gains `world_model_mode` (`WorldModelMode::{Off, Track}`, env
  override `FORMAL_AI_WORLD_MODEL_MODE`). It defaults to `Off`, so the feature is
  trace-only until opted in and existing behaviour is unchanged.
- Dialogue turns, solver coreference, and agentic research-topic recall now use
  the common context hierarchy. Rust and browser coding-language inheritance no
  longer stop after four scopes; both use cycle-safe visited sets.
- Promoting dialogue-local current state into shared general memory now fails
  closed unless `GeneralMemoryPermission::Allowed` is supplied explicitly.
