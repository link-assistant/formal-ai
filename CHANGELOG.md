# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

Older releases are archived under `docs/changelog/` so that no file
exceeds the repository's 1500-line cap (newest first):

- [0.346.0](docs/changelog/archive-07.md)
- [0.307.0 to 0.345.0](docs/changelog/archive-06.md)
- [0.242.0 to 0.306.1](docs/changelog/archive-05.md)
- [0.183.0 to 0.241.0](docs/changelog/archive-04.md)
- [0.177.0 to 0.182.0](docs/changelog/archive-03.md)
- [0.105.0 to 0.175.0](docs/changelog/archive-02.md)
- [0.1.0 to 0.104.0](docs/changelog/archive-01.md)

<!-- changelog-insert-here -->

## [0.352.1] - 2026-09-27

### Fixed

- The Auto Release job can publish the Docker image again: the final stage
  copies the binary from `/app/target/release/formal-ai`, a contract the
  prebuilt stage already satisfied but the compile stage never did after the
  Plan 16 L1 move put cargo's output in `rust/target/release` — run
  36278582429 (v0.352.0) failed at the GHCR push with
  `"/app/target/release/formal-ai": not found`, after the crate itself had
  already published to crates.io and passed its smoke test. The compile stage
  now stages its binary at the contract path, the contract is stated where
  both stages can see it, and no pull request could have caught this: the
  docker-build check pins `BINARY_SOURCE=prebuilt` and never runs on main
  (issue #1151). The same PR un-broke the next release in a second way the
  v0.352.0 run revealed: the release commit bumps the crate version and
  appends the self-hosting ledger row but never re-rendered the status
  surfaces that project them (README.md, docs/status.md), and because the
  bot's push triggers no workflow, nothing noticed until the next tree ran
  the `check_status_render` gate — the release script now regenerates and
  stages those surfaces as part of the release commit.

## [0.352.0] - 2026-09-26

### Added

- Add deterministic idle next-request prediction, offline variant probes, a
  proposal-only adoption frontier, consent-gated source prelearning, TTL-aware
  offline recall, and an honest prediction-hit ledger.

### Added

- Issue #1138: the plan set and implementation of a general, self-coding
  meta algorithm. One `Need` record connects every step; the universal loop
  performs live concept lookup over the trusted sources registry instead of
  logging that it cannot fetch; a content-addressed sense ledger forgets and
  rediscovers; every obligation needs an execution `Evidence` record before
  it is satisfied; prerequisites such as a missing compiler become needs
  solved through trusted publishers with workspace-scoped installs; one
  repository workspace protocol serves SWE-bench, the coding ladder and
  self-coding with a seed-driven default-deny command allowlist; capability
  routing is decided by object type, act and locus from seed with a 420-case
  held-out suite in five languages; selection heuristics (least action, TRIZ
  contradictions, refutation-first search, balanced splitting) are registry
  methods; learned methods must change a held-out answer before adoption.

### Fixed

- Prose could reach `/bin/sh -c` on the agent path and a non-zero exit could
  be reported as completed; the seed allowlist and two-valued command
  outcomes close both.
- GitHub work items are read as structured source through `gh` when a real
  shell is available; fetch-only clients receive a data-declared extraction
  prompt instead of the solve request. This prevents model-backed fetch tools
  from recursively solving a task and returning generated code as issue text,
  while preserving a bounded fetch fallback when the CLI read is unavailable.
- The closure audit no longer counts its own generator's output; the sixteen
  generated closure shards are deleted and the debt ratchet is strict in
  both directions.
- The tests-as-documentation gate parses Rust assertions structurally, so an
  answer mentioned only in diagnostic formatting no longer masquerades as an
  exact behavioral contract.
- Browser synchronous-handler membership, invocation metadata and precedence
  now come from a registered Links Notation seed instead of a worker-local
  array. A web-stage gate rejects duplicate inventories, missing bindings,
  stale generated seed lists and fixed-path debt scans that overlook the real
  dispatcher.
- Repository completion now compares evidence-backed current facts with an
  independently formalized goal state. Missing, unevidenced and mismatched
  requirements remain typed Need/obligation gaps, so mergeability, a diff or a
  partial green check cannot by itself declare a repository task complete.
- Selection is now a registry capability: source-linked TRIZ contradictions
  are exercised by a 20-task corpus in five languages, refutation-first search
  precedes sampling, binary splitting reports underivable tasks honestly, and
  approach deduplication retains the first historical source.
- Release publication is now convergent and observable: crates.io throttling
  fails the job instead of yielding a green partial release, automatic reruns
  resume the prepared current-main version without double-bumping or
  double-tagging, publication state is rechecked after synchronization, and
  Cargo verifies the generated `.crate` before any downstream artifact is
  published.

### Added

- A runtime obligation ledger. An obligation reaches `satisfied` only while
  carrying an execution record — the command, its exit status or an explicit
  none, and a SHA-256 of the observed bytes — and that variant has exactly one
  field, so a satisfied obligation nobody observed cannot be constructed at all.
- A fourteenth step in the recursive meta core, `verify_obligations`, described
  in `data/meta/recursive-core-recipe.lino` and executable from it. Executing
  the recipe still reproduces the native trace event for event under every mode
  combination.
- `data/meta/obligation-evidence-contract.lino`, which states how a clause's
  shape becomes the observation that would settle it, so a new expectation shape
  is a data edit rather than a new branch.

### Changed

- A need row reaches `satisfied` in exactly one place: the join that reads a
  discharged obligation. Route selection no longer implies satisfaction
  anywhere.
- An observation discharges only the obligation whose expectation names its
  path, command or check. An unrelated successful result now clears nothing.
- A clause no composer can read an artifact out of is kept as a node with its
  byte span and split, instead of being dropped.

### Fixed

- Derivations record their composed structure instead of a content hash:
  search drafts now log `typed_search(reduce_count(...))`-style notation
  with meaning fragments inline, scope fillers omitted, and runtime
  materialization templates rendered as what they wrap (issue #326).
- Multilingual synthesis benchmarks assert the same verified artifact
  behind a wrapper rendered in the prompt's own language; only the
  English lead case pins the full rendering byte for byte.
- Elliptical clock-hour prompts create calendar events without a web
  search detour (issue #595), shell-process prompts keep their terminal
  suggestion across languages (issue #870), and commutative idioms emit
  operands in the order the task names them (issue #315).
- Offline concept misses record their consulted-source trail instead of
  skipping it; Spanish fallback-script markers no longer vote inside
  English words.

### Fixed

- The three regressions batch a5abd1ab2 introduced on the CI Test job. The
  Hindi personal statement of non-understanding ("मुझे समझ नहीं आया") is again
  a clarification: the bare "समझ नहीं" stem no longer sits in the
  prior-turn-reference class, whose other surfaces are re-render idioms ("दूसरे
  शब्दों", "फिर से कहिए") — Russian keeps the same split (impersonal
  "непонятно" re-renders, personal "Я не понимаю" asks for clarification), and
  English never carried the personal form. The embedded rule document's rule
  count pin follows the `clarification_inflected_stem` rule the batch added
  (15 → 16), and the Spanish typed-write pin includes the request-anchors line
  the HonestGap now appends.

### Fixed

- Indexed the seed-network children read in `LinkStoreSource::from_store`, so
  booting the condition backend no longer scans every projected link per
  meaning child. A cold first completion on a 400-event memory store drops
  from ~17.7 s to under a second, and the projection-rebuild budget test
  passes again (issue #1138).
- Meaning interrogations ("what does X mean", "meaning of X") now surface the
  consulted-source record: the subject is read by the same seeded extractor
  the concept handler routes with, instead of the whole prompt becoming an
  unspecific focus that buries the record under the generic unknown guide.
- Restored the issue-report invitation for prompts the definition router
  claimed and still could not resolve ("explain X"), closing the issue 864
  regression. Raw unmatched prompts keep the plain teaching guide without the
  invitation, as the chat-surface specification pins it.
- HonestGap agent-mode requests no longer decline on the HTTP server surface,
  and dotted filenames (`alpha.txt`, `main.rs`) are recognized as request
  anchors, so the aider-style request reaches the planner again.
- Removed the document-generation action cues (给我/帮我) from the handler
  promotion table, restoring correct routing for Chinese sentences that
  mention rather than request those actions.

### Changed

- The ordered-list gate now honours an `inert` module list per declaration
  file (registry: `data/meta/merge-conflict-policy.lino`), so a plan drafted
  as tests before its leaf lands no longer blocks derived-artifact
  regeneration. The mirror stays exact for every compiled module, and a
  second unregistered module is still reported as drift.

### Fixed

- The two upstream prompt shapes of issue #1085's transfer slice. An MBPP
  assertion's argument tuples no longer type the derived parameters: a bare
  `(3, 4, 5, 6)` is ambiguous between a fixed pair and an ordered sequence,
  so `example_parameter_type` leaves the parameter open and the signature
  stays generic by arity, exactly as the assertion-is-not-a-signature rule
  demands (previously the catch-all read the tuples as text and annotated
  the parameters `str`). A signature the prompt did declare is now echoed
  verbatim into the composition draft: the search still unifies types to
  find the composition, but the declared spellings, return annotation, and
  import block travel with the artifact, so an upstream `List[float]` keeps
  `from typing import List` and its own spelling instead of a reconstructed
  `list[float]` that dropped the import and with it executability
  (HumanEval/0).
- The held-out capability-routing corpus recovers its hi non-understanding
  case: removing the bare "समझ नहीं" stem from the prior-turn class left
  "मैं समझ नहीं पा रहा।" unrouted, and the walk fell through to a
  prohibited web search. The continuous forms ("समझ नहीं पा रहा/रही/रहे")
  join the prior-turn surfaces as full phrases — the same class that
  carries en "lost me" and ru "Я потерял нить" — while the exact personal
  statement "मुझे समझ नहीं आया" stays with the clarification handler.

Fixed: Russian compose verbs are the compose signal, not the removed nouns.
0bc8d42e2 stopped retrieval theft by deleting the "материал"/"подробн" surfaces
from act_compose, which also silenced two held-out compose paraphrases
(ru_compose_03/07) whose only signal had been the noun. The lexeme now carries
the imperative verbs "наброса" and "составь" — exact counterparts of the
existing en surfaces "draft" and "put together" — so composition prompts route
by their act verb while "Найди подробные сведения…"-style retrieval prompts
keep reaching web_search through their own verbs (420/420 held-out cases).

Fixed: the tests-as-docs allowlist covers the fifteen loose-only behavioural
tests that the closure-audit step had masked (place_timezone, issue_435,
issue_595, translation_via_links, prompt_variations, intent_phrase_migration,
extended definition), regenerated with the gate's own `--write` migration.

### Fixed

- A learning directive that names a URL ("learn from … at
  trends.google.com/…", "обратясь сюда ты узнаешь …", "यहाँ से सीख
  सकते हो …", "在这里了解…") now routes to `learn_from_source` instead of the
  generic web fetch (issue #499). The act vocabulary gains a ninth act,
  *learn* — narrower than *retrieve* because the request asks the engine to
  adopt knowledge from a declared source rather than fetch the URL once — with
  surfaces in `data/seed/meanings-acts.lino` and a `(url, learn, web)` row in
  the capability table. The handler stays gated on the seed-declared
  learnable-source registry, so a directive the registry declines falls
  through to the specialized walk unchanged, and cue-less prompts ("Open
  a bare URL") keep the fetch route.

### Fixed

- A response-language follow-up no longer strands a pattern-inference report
  in English (issue #531's #556 generalization, red in the wave-F tail). The
  pattern handler's report language was a hard-coded `en` default, so the
  replay forced Russian onto `detect` while the one handler that does not
  detect kept answering in English. `language::forced_response_language_slug`
  now exposes the forced slug to such handlers, `try_pattern_inference`
  renders in it, and an explicit switch *to* English still records
  `language_to:en` (the variant treats `en` as its no-op default and would
  not log it). Also covers a language the conversation already established
  (issue #724), since both force the same slot.
- The issue #724 binding test asserted the wrong answer family: it expected
  the unknown-reasoning trace where plan 01's pinned behaviour for a
  definition question is the consulted-source record (batch a5abd1ab2 kept
  the record for "what does X mean"). The test now pins the seed-grounded
  Russian record prefix up to the environment-dependent consulted list, so
  the leaf's actual claim — the demonstrated language binds — is what fails
  if the binding regresses.

### Fixed
- An explicit shell passthrough ("execute cp a.txt b.txt") is no longer re-read
  by the capability table into `cat` of both paths; dictated commands reach the
  shell cascade verbatim (issue #749).
- A protocol-hosted fetch tool now performs the work-item read itself instead of
  planning `gh` first, so server-side clients stop wasting a turn on a command
  their sandbox refuses (issue #904).
- Hindi calculator prompts that spell the operator as a word route to arithmetic
  (the arithmetic promotion now recognizes the `arithmetic_operator_word` role),
  and Hindi verb-final reachability prompts no longer read as web-search
  imperatives.
- Russian compose stems ("материал", "подробн") no longer steal retrieval
  prompts into long-form composition, and the bare English noun "machine" no
  longer turns "Machine learning" into a path-scope request.
- The unknown-reasoning web-search handoff no longer demands a "specific" focus,
  so general research prompts reach their sources instead of the legacy
  fallback.

### Added
- `experiments/issue_1138_feedback_recovery/` — reusable collector that
  recovers genuine user directives from session `.jsonl` transcripts
  (harness wrappers, agent reports, and cron echoes filtered out), with
  the 2026-09-23 recovered snapshot for issue #1138.

### Changed
- Requirement-status ledger regenerated with the post-L1 test path fix
  (shards name `rust/tests/`, older rows `tests/`); all 68 R1138
  requirements now read `implemented` with named automated tests,
  including R1138-B2-6 (external-benchmark floors pinned by
  `rust/tests/unit/specification/external_benchmarks.rs`) and R1138-B2-8
  (OEIS/python_docs replay pins).
- CONTRIBUTING.md records three standing directives: Opus-only
  sub-agents, classify CI failures before fixing, and the
  feedback-recovery audit protocol.

### Changed
- An unresolved *instruction* is a research trigger again (issue #873,
  R873-1): the wave-F suppression of the web-search handoff now applies
  only to question-shaped prompts (`?` in either width or a seed-carried
  interrogative opener), so an imperative like "Calibrate the snorflax
  …" keeps its `web_search` intent in every registered language while
  the wave-F question pins keep their consulted-source compound
  response.
- Issue #932 box image survey re-run against the live registry
  (2026-09-23T08:37:08Z): `konard/box-kotlin` is now published, so it
  moved from the missing list to the published list and the canonical
  survey was regenerated through the real agent-CLI recipe; kotlin stays
  a deferred project with a truthful reason (publication postdates the
  pinned contract, promotion is planned follow-up), and
  `rust/tests/unit/issue_1138_execution_box.rs` pins the new split.
- `rust/tests/unit/issue_1138_universal_loop_lookup.rs` policy scan
  re-armed after plan 16 L1: it scans `rust/src` (the live tree) instead
  of the removed root `src`, so it can no longer pass vacuously.
- Issue #932 case study refreshed: README survey conclusions match the
  2026-09-23 log (eight published repositories, five missing, 2.4.0
  pinned as the verified-against tag rather than the newest), and the
  self-hosting decomposition artifacts carry post-L1 `rust/` paths.

### Changed
- The promotion-gate runners execute `cargo test` with
  `--manifest-path rust/Cargo.toml`, and `improve --promote` anchors gate
  replay to the compiled checkout root instead of the invocation
  directory. Plan 16 L1 moved the crate one level below the repository
  root, so every canonical gate command was failing to find a manifest
  and reporting `blocked:0/1`, which kept the issue #922 agent-CLI E2E
  lane red; the suite manifests, the fixed unit-specification command,
  and their pins now agree with the form `docs/benchmarks.md` already
  documented.
- The committed `data/seed/learned-methods.lino` is now the byte-exact
  product of the promotion protocol. The 2026-09-20 re-derive wrote the
  file with a trailing newline, but seed-edit generators trim their
  payloads (`adopted_seed_lino` ends in `trim_end`) and the agent that
  authors the materialized file echoes the task text without its final
  newline, so the protocol can only ever produce a file without one;
  the stray byte made the issue #922 provenance `cmp` fail by a single
  EOF.
- `rust/examples/issue-922-method-learning/run.sh` resolves
  repository-root-relative fixtures through a `REPO_ROOT` indirection
  (comparison target, stderr classifier), matching the post-L1 layout
  where the script's `ROOT` is the crate root.
- The external replay in that harness now passes
  `--no-summarize-session --compaction-models "(same)"`. The Agent CLI
  summarizes sessions by default through a hosted provider that rejects
  calls from outside its own client, so the run failed at teardown with
  an `UnhandledRejection` even though the file write through the local
  formal-ai server had succeeded; the flags keep every model call on the
  provider under test, matching the canonical
  `experiments/agent_cli_e2e/run_agent_cli.sh` invocation.

### Fixed
- `rust/examples/self-coding/run.sh` resolves
  `scripts/classify-agent-cli-stderr.sh` through `REPO_ROOT` and invokes the
  Agent CLI with `--no-summarize-session --compaction-models "(same)"`. Plan
  16 L1 left the script's `ROOT` at the crate root, so the classify step
  exited 127 on a path that no longer exists, and a local replay without the
  job-wide `LINK_ASSISTANT_AGENT_SUMMARIZE_SESSION=false` would crash at
  teardown on the hosted Console provider (the same issue #922 failure the
  promotion harness fixed).
- `data/meta/self-healing-case.lino` is regenerated with
  `cargo run --example dump_self_healing_case`: the committed copy still
  carried the pre-L1 bare `cargo test --test unit ...` benchmark runner that
  the self-improvement fallback stopped emitting, which the issue #558
  byte-equality check compares against. The retained issue #905 evidence is
  refreshed the honest way -- session
  `ses_f31609a51ffeU6RSdVVZNdp0XX` re-ran the canonical self-healing recipe
  through Formal AI and the real Agent CLI via
  `experiments/issue_905_self_healing_refresh/run.sh` (planner count pin
  16074 -> 19974), producing a byte-identical artifact; the README and the
  issue #905 session pin follow the new trace.

### Changed
- The lint `timeout-minutes` pins in `issue_1076` and `workflow_release`
  tests follow the workflow to 45 minutes, citing run 35806556194, where the
  gate set consumed the full 25-minute cap and the kill surfaced as
  `cancelled` (issue #1138). `check-job-headroom.rs` already audits 45;
  the two unit pins predated the raise and were masked because every
  intervening Test job failed at an earlier step.

### Changed
- The `Run tests` step of the full matrix leg gets `TEST_BUDGET_SECONDS:
  2400`, up from 1440 (issue #1138). Both attempts of run 35877276920
  measured the demand the old budget hid: the unit phase alone needs
  1251-1315s, and the integration phase ~200s more (389 of its 397 tests
  finished inside the 164s window the first attempt reached before its
  kill), so the step demands ~1520s. 253687e0f only looked complete
  because its three real failures ended the step at 1344s first. 2400
  holds the measured worst at 63%, and the full lane's budget sum
  reaches 3600s -- 66.7% of the job's 90-minute cap, level with the
  specification lane. The cap pins in `workflow_release`, the
  `issue_1076` lint-pin rationale, and the `issue_1081` job-sum comment
  (7200s declared, 3600s spendable, naive summing would demand a
  171-minute cap) follow the raise.

### Changed
- The benchmark corpus gate (`no_benchmark_prompt_reaches_the_unknown_opener`,
  issue #1138 plan 10 leaf 10) moves from the Test job's shared `Run tests`
  step into its own workflow, `benchmark-corpus-gate.yml`. The gate answers
  all ~1355 committed benchmark prompts through the full engine -- 842s and
  873s measured locally, and run 35893508679 proved the CI pace exceeds
  1154s without finishing -- so it cannot share a 2400s budget with a unit
  phase that alone measures 1220-1315s; the lane died at exit 124 on both
  attempts. The new workflow budgets 1500s for the single-target build and
  4200s for the gate (holding the worst CI/local ratio the specification
  lane measured, 4.4x) under a 140-minute cap, at 68% of the share issue
  #1081 caps. `run-prebuilt-tests.sh` skips the test in the shared lane the
  same way `data_files`, `self_ast_census`, and `specification` are skipped,
  and the `Run tests` comment in `release.yml` records the move.

### Fixed
- `scripts/run-prebuilt-tests.sh` handed the corpus-gate skip to the test
  binaries as one quoted argument, so libtest saw the single option
  `skip issue_1138_no_silent_unknown` and refused to start the suite:
  run 35912188200 (job 107358064235) died with exit 101 twenty-six seconds
  into the `Run tests` step, before any test ran. The skip is now a bash
  array expanded as `"${CORPUS_GATE_SKIP[@]}"`, giving libtest the two words
  `--skip` and `issue_1138_no_silent_unknown` the way the three skip flags
  beside it already do. Verified by executing the script against arg-echo
  stubs for all three suites -- the earlier check was `bash -n` only, which
  cannot see word-splitting.

### Fixed
- `issue_749_shell_routing::whole_shell_task_matrix_routes_without_web_search`
  failed only in CI (run 35919283963, job on tip 02a810b80): its "run the
  tests" row resolves through `formal-ai:workspace-test`, which picks the
  command from the first marker file in the process's working directory.
  The prebuilt binaries issue #1055 ships run from the repository root,
  where plan 16 L1's bun umbrella (`bun.lock`, `package.json`) wins and the
  answer is legitimately `bun test`; a local `cargo test` starts the same
  binary in `rust/`, where `Cargo.toml` wins and the answer is `cargo
  test`. The engine's cwd anchoring is the design; the matrix was the part
  making an implicit assumption. It now holds in a cargo workspace the test
  controls (a temp dir with a `Cargo.toml` marker, restored on drop), the
  same pattern its unit twin established when L1 landed. Verified by
  running the compiled integration binary from the repository root, the
  exact condition that failed.

### Fixed
- The wave-T corpus gate (`issue_1138_no_silent_unknown`) failed assertion (a)
  for 18 of 1355 benchmark prompts: every elliptical news ask (`Anything big I
  missed today?`, `Что важного было сегодня?`, …) routes through the capability
  table to `Routed { web_search }`, but the web-search family's query
  extractors derive no query for them, so nothing answered and the
  comprehension-gap terminal emitted the seeded unknown opener. That terminal
  (`answer_with_legacy_fallback`) now consults the table before giving up
  (plan 10 leaf 10-10): a `Routed`, `Lowered` or `HonestGap` placement renders
  the capability-gap answer naming the capability the chat surface lacks —
  the same honesty the accepted `relative_period` + `web_search` arm already
  answers with — and an agent client keeps the fall-through because it
  advertises the tool itself. An `Ask` outcome is the table declining to place
  the prompt, so the unknown-reasoning ladder's own answer stands for it.
  Silent UNKNOWN is now unreachable for every table-placed benchmark prompt.

### Fixed
- The wave-T corpus gate (`issue_1138_no_silent_unknown`) still failed assertion
  (a) for the six quoted-example lipogram paraphrases (ru, hi, zh, each carried
  by both held-out corpora). The consult walk recorded attributed misses for
  every variant, but the record arm rejected ru/hi/zh on focus specificity
  while en passed only through the implementation-language modifier scan
  stripping ` in e` — a letter it mistook for a language; `на букву e` is
  correctly rejected and head-final hi/zh carry no preposition to scan. A
  quoted-example question (quoted span plus question shape) now closes on the
  consulted-source record in every language: the predicate it turns on lives
  outside the quotes, `unknown_surfaces` skips quoted spans for exactly that
  reason, and every alternative terminal embedded a seeded unknown opener. A
  question that quotes nothing keeps the issue-#44 teaching ladder. The Spanish
  pin moved from the localized unresolved body to the same record (still
  Spanish, still never the unsupported-language fallback); en is byte-identical
  to its pin.

### Added
- `formal-ai translate --from rust --to meta --input PATH` (plus `--list` for
  every direction): any-direction translation through the meta language as the
  single pivot — `translate(X → Y)` is always `extract(X → meta)` then
  `render(meta → Y)`, and the dispatcher holds no per-pair code. The first
  materialized leg is the Rust extractor into the pivot; every leg not built
  yet answers with the exact plan-16 leaf that owes it (js/ts extractors and
  the `meta → rust` inverse by L5, the js → ts dogfood by L3), the same
  stated-gap honesty the capability table practices.

### Added
- The three-roots doctrine is now standing requirements: full parity between
  the Rust, JavaScript and TypeScript implementations of the client and the
  entire backend server and all other logic, each translatable into the others
  through the meta language (REQUIREMENTS.md assembled from the new
  docs/requirements shard, the architect note
  docs/architect-notes/2026-09-24-three-roots-full-parity-via-the-meta-language.md,
  docs/meta-algorithm.md, docs/source-roots.md, VISION.md and the
  requirements-traceability index). Rationale recorded with it: JavaScript
  executes faster than Rust compiles, so js/ts can carry the iteration cycle
  once parity holds, and `formal-ai translate` is the vehicle for moving code
  in any direction without hand porting.

### Fixed
- The release pipeline reads the formal-ai version through
  `rust-script scripts/get-version.rs` (rust-paths root discovery)
  everywhere it needs it for GitHub Pages deploys: the two
  "Resolve Pages deploy ref" steps (auto-release and changelog-pr) and
  the Pages deploy "Read formal-ai version" step previously ran inline
  `sed` reads against the repository-root `Cargo.toml`, which plan 16 L1
  (`70df98cf6`) moved to `rust/Cargo.toml`; on the first release-relevant
  main push after that merge the `if: always()` ref-resolution step
  failed with "Could not read formal-ai version from Cargo.toml" and
  took the whole Auto Release job red. `scripts/stamp-pages-artifact.sh`
  mirrors the same fallback ordering (root manifest first, then
  `rust/Cargo.toml`).
- The Pages deploy job installs rust-script through
  `scripts/install-rust-script.sh` (retry wrapper) before reading the
  version, and the `rust_script_install_steps_use_retry_wrapper`
  tripwire test counts the new occurrence (8 → 9) so the install step
  cannot silently regress to a bare `cargo install`.

bump: patch
---

### Fixed

- The projection rebuild-cost ratio pin (`issue_1106_projection_reuse`) no
  longer flakes on loaded CI runners: each leg (append and rebuild) is now the
  minimum of three sampled timings instead of a single wall-clock sample, so
  one contended sample cannot speak for the leg. Scheduling noise only ever
  adds time, so the minimum estimates the intrinsic cost, and the fsync-per-
  doublet regime the pin guards against (issue #710, PR #888) stays two orders
  of magnitude past the bound under any sample count. The failure message now
  names the binding budget explicitly. Main run 36263664670 failed the merge of
  PR #1144 with a 2.27 s rebuild against the 2 s generosity floor — 13 % over
  on a runner loaded by the rest of the suite — while the identical commit had
  passed the same test on its branch run hours earlier.

### Fixed

- The Auto Release job can compile its scripts again: the unused
  `PreparedRelease` import that `RUSTFLAGS=-Dwarnings` turned into a hard
  error in `scripts/version-and-commit.rs` (run 36269287140, latent since
  the PR #1139 bulk state) is gone, and `scripts/create-changelog-fragment.rs`
  now declares the `regex` dependency its `rust-paths.rs` include has needed
  since manifest parsing moved there — every other rust-paths includer
  already declared it. Both scripts run only in push-to-main release jobs,
  so no PR-side job had ever compiled them; both are now verified locally
  under `-Dwarnings` via side-effect-free early exits. The import had a
  second consumer: the PR-side `check_release_changelog_collection` gate
  runs `rust-script --test` over the script, whose tests module constructs
  `super::PreparedRelease` — so the name is now imported under
  `#[cfg(test)]`, present exactly when the tests compile it and absent
  from the release build that must stay warning-free (issue #1147).

### Fixed
- Changelog fragments and the collected CHANGELOG.md are discovered where
  they live instead of being derived from the rust root: the shared
  `rust-paths.rs` helpers (and `get-bump-type.rs`, `version-and-commit.rs`,
  `create-changelog-fragment.rs`, `collect-changelog.rs`, which now delegate
  to them) resolve repository-root `changelog.d/` first, then
  `{rust-root}/changelog.d/`, defaulting to the root. Plan 16 L1
  (`70df98cf6`) moved `Cargo.toml` to `rust/` while the fragments stayed at
  the root, so the bump-type step reported `fragment_count=0` /
  `has_fragments=false` and `check-release-needed.rs` skipped the cut with
  ~24 pending fragments (issue #1143). `workspace_manifest_resolution`
  pins the discovery contract hermetically instead of the old derivation.
- The Pages deploy job generates the Rust API docs through
  `scripts/build-rust-api-docs.sh`, which now discovers the crate manifest
  (root `Cargo.toml`, else `rust/Cargo.toml`) and pins `--target-dir
  target`, so the build works from either layout and the artifact
  assembly keeps copying repository-root `target/doc/`. The bare
  `cargo doc` invocation failed with exit 101 "could not find Cargo.toml"
  on main run 36255580824 after the same manifest move (issue #1143).

## [0.351.0] - 2026-09-15

### Fixed

- Recovered multi-question composition, multilingual assistant naming,
  target-less modification clarification, and deterministic free-time response
  variants; re-verified all 32 dropped-requirement audit rows with honest
  evidence. A follow-up audit on v0.337.0 confirms 29 working requirements,
  one superseded folder-routing requirement, and two focused open gaps.
- Keep the issue #961 authorship regression valid after release automation
  promotes its generated fragment from `changelog.d` into `CHANGELOG.md`.

### Fixed
- Rebuilding the native projection of a memory store no longer pays one `fsync` per doublet. The replacement database is scratch until a `rename` publishes it, so its transitions log is now staged without per-append syncing and flushed once before publishing; what reaches the served path is exactly as durable as before. A 112-event rebuild fell from 15.8 s to 0.2 s and the cost is flat instead of superlinear, which is what pushed the held-out computer-use generalization suite past its 600 s budget in one run while the same code finished in 85 s in another (issues #1106, #710).

### Added
- Coding synthesis can now recognize HumanEval, MBPP, and multilingual conversational task shapes, discover licensed operations from Python documentation and Wikifunctions, compose verified Python programs, and remember provenance-bearing procedures without benchmark-specific bodies. Upstream benchmark runs opt into live discovery with `--online`, while offline replay remains the default.
- Coding synthesis now distinguishes callable functions from runnable programs, binds requested standard output through multilingual semantic slots, executes whole-program candidates in the bounded workspace, and carries every verified artifact through the shared agent write-tool contract.
- Abstract Wikifunctions recurrences are now discovered as typed expression trees, grounded through fetched operator descriptions, proven to descend toward a boundary, replayed against source testers, and rendered without embedding benchmark-specific function bodies. A rebuildable non-seed web cache carries the same formalization and multilingual Wikidata aliases to the browser worker.
- A generalized structural composer now covers arithmetic, collection, scan, ordering, pattern, predicate, symmetry, and weighted-grid families through source-grounded meanings and bounded example execution. The first 20 HumanEval tasks pass without a source cache, and the first 20 MBPP tasks pass when live discovery is enabled.
- Named integer sequences and source-defined recurrences can now be discovered through the official OEIS JSON API. The bounded catalog follows cross-references, accepts only a strict arithmetic or linear-recurrence grammar, derives index mappings from examples, preserves CC-BY-SA provenance, and replays through the shared content-addressed source cache.
- Rosetta Code example requests now return attributed GFDL examples, and explicit Rust execution requests run only in the bounded agent workspace.

### Fixed
- OEIS discovery now isolates the requested content-object noun between the tile and board dimensions, normalizes ordinary English plurals, and builds a canonical tiling query. Explanatory bridge prose can therefore no longer turn a source-backed recurrence request into an unrelated search phrase.
- Structural coding meanings now use canonical nested surface facts and carry distinct official documentation groundings, preserving semantic-seed integrity without prompt-specific definitions.
- Conversational coding recognition now considers only the outer instruction after benchmark signatures and assertions are parsed, so programming words inside a fenced document cannot steal an unrelated document-conversion request.
- Pull requests that change the agentic routing subsystem now run the complete Agent, OpenCode, Claude, and Codex research replay before merge, closing the post-merge-only coverage gap from issue #1137 while unrelated branches retain the cheaper held-out gate.
- CLI repository roots are resolved to a stable absolute path before benchmark or summarization child workspaces run, so `--repository-root .` no longer turns a valid grader script into a duplicated nested path after `current_dir` changes.
- Python task recognition now selects the requested target definition after completed helpers, treats blank doctest output as `None`, and preserves periods inside quoted literals while splitting requirements.
- Coding-structure lookup now normalizes punctuation and tolerates one edit in long single-token concepts, so hyphenation and ordinary misspellings do not create artificial capability gaps.
- Agentic authoring now keeps read-and-author obligations together: it can derive caller-declared Links Notation fields from an inspected source record, write the result, and verify a decorated client read-back instead of ending after the input read.
- Agentic structured-document authoring now preserves repeated source-record cardinality, scopes values to each record, retains repeated exact fields, rejects ambiguous partial field-name matches, and accepts schema lists spanning semicolons.
- Agentic evidence authoring now distinguishes a requested schema or opening line from a literal payload, derives the remaining file content from observed tool results, and keeps repository statement audits ahead of generic file writing.
- The general agentic planner now treats bare `with` as a literal file-content lead, allowing the real Formal AI Agent CLI self-authoring flow to preserve exact backticked multiline payloads.
- The self-authoring harness now waits explicitly for its local Formal AI server to bind instead of relying on platform-dependent curl retry behavior.
- The Rust lockfile now uses `rustls` 0.23.45, resolving RUSTSEC-2026-0285, and the dependency-audit proof parser uses extended `sed` expressions that work on both GNU/Linux and macOS.
- Live-link checking now excludes the byte-for-byte Python documentation captures used for coding-discovery replay, so expired links inside upstream fixtures do not fail repository documentation checks.
- Derived-artifact regeneration now formats generated Rust before producing byte-sensitive self-AST and planner fixtures, so a successful regeneration cannot make its own outputs stale.

### Fixed

- Compose literal-output program requirements with independent language, path and output operands; include source-backed CI runtime setup and executable exact-output verification.
- Bind recipe progress to actual write bytes and ordered command results, and commit only the recipe's source artifacts.
- Protect original and legacy memory under storage pressure, revalidate eviction at application time, and retain reconstruction provenance for disposable public-source caches.

### Fixed

- Preserve inline formalization sources across shared quote forms and in source
  order. Domain words no longer replace user text with an unrelated cached work;
  only an exact normalized catalogue title selects the supported reference.
- Clarify that the current lexicon's shallow extraction is an implementation
  limitation, not a claim that open-domain formalization requires neural inference.

### Fixed

- Separate selected methods (`planned`) from validated results (`satisfied`) in
  the shared need ledger. Native and data-driven planning traces no longer
  report complete resolution or demonstrated skills before execution.
- Keep planned and blocked needs in the learning curriculum. Skill promotion
  still requires its existing test and benchmark evidence gates.

Resume recipe execution after an observed, correctly bound successful retry,
without discarding the failed attempt or accepting unrelated setup as proof.
Reject unchanged seeded artifacts as self-authored changes before publishing
them, and clarify that the authoring helper's no-commit mode does not stage files.
Reuse the shared source digest helper in the recurrence-cache generator.

Preserve literal output, source URLs, file paths and numbered/unnumbered requirements during shared structural decomposition, including exact Unicode source offsets.

## [0.350.0] - 2026-09-14

### Fixed
- Agentic planner: a namespaced MCP research tool now outranks the client's own research alias. Claude Code advertises `WebSearch` beside a wired-up `mcp__issue781__websearch` but grants permission only for the MCP one, so planning its built-in alias ended the four-client research run at "Claude requested permissions to use WebSearch, but you haven't granted it yet" with no search recorded (issue #781, regression from the issue #1133 ordering).

### Fixed
- Agentic planner: browser-automation tools (`mcp__playwright__browser_click` and siblings) are no longer classified as fetch tools, client-executed research tools rank above namespaced guesses, a fetch whose `url` the harness projected away still counts as attempted, and a tool call that has failed twice with the same report is replaced by the failure report instead of a third attempt (issue #1133; 547 identical calls in Hive Mind's Kotlin run).
- Execution recipes judge an exit-less shell result by the seed failure lexicon, so `/bin/sh: 1: scala: not found` is reported as the failed step instead of "Created and verified" (issue #1133).
- Repository work items are read through the client's own `gh issue view` when the only fetch tool is a remote connector, MCP results keep their `structuredContent`, `codex_apps` connectors are remote-scoped, and file-creation routes accept `apply_patch` as a creation tool, so Codex plans the work item instead of narrating the connector's placeholder (issue #1133).
- A harness "text to summarize" envelope is answered with a summary of the quoted text, never by executing it (issue #1133).
- An additive edit ("add a line X directly after the line Y in F") composes to an edit, `\n` in edit prose means a newline, and an instruction that edits a named file is never routed to web search (issues #1115, #1116, #1133).
- Self-authoring: task-contract values keep no author quotes (issue #1117), and a task that targets `.github/workflows/` without `FORMAL_AI_BOT_TOKEN` fails before authoring with the reason (issue #1118).

### Added
- A work item that names a pull request or branch ends with `git add`, `git commit`, `git push` and reports the commit hash; a request to "commit them" is a stage-commit-push step; a work item that asks for a GitHub Actions workflow gets one running the verified commands (issue #1133).
- `examples/replay_hive_mind_1133.rs` replays the three 2026-09-13 Hive Mind runs offline; `docs/case-studies/hive-mind-hello-world/2026-09-13-three-runs.md` holds the root-cause analysis.

## [0.349.2] - 2026-09-13

### Fixed

- Docker Hub publishing now runs. `DOCKERHUB_IMAGE` and `DOCKERHUB_USERNAME`
  were read from repository configuration that was never set, so every release
  since the feature was added disabled Docker Hub and still reported success --
  `hub.docker.com/r/linkassistant/formal-ai` had never received a push. Both
  now default in `release.yml` (`konard/formal-ai`, `konard`), the way hive-mind
  names its image, and `DOCKERHUB_TOKEN` becomes what opts in: a fork without
  the secret skips Docker Hub and still gets a green release (#1131).
- A whole-file rewrite is no longer mistaken for a member insertion. A request
  to rewrite a shell script was routed into the structural-edit path because its
  prose contained the bare word `set` -- from `set -euo pipefail` -- and quoted
  two values while explaining the change. Both were spliced into the script's
  nearest bracket, producing a shell condition that parses and is always true.
  Growing a member list now requires a verb that asks for it (#1131).

## [0.349.1] - 2026-09-12

### Fixed

- A release too small to measure no longer sets the self-hosting bar for every release after it. `v0.348.1` changed exactly one line; that line was Formal AI's, so the cycle measured 100%, and weighted into the trailing window that single line carried the ratchet to 2.91%. The next cycle -- thousands of reviewed lines with a document Formal AI authored inside it -- projected 0.05% and was blocked, so the bar set by a one-line release punished the following cycle for containing real work. A share measured over fewer than 100 changed lines is now treated as noise and may not raise the ratchet; it is still recorded and still reported. The identical share measured over a real cycle ratchets exactly as before, so this is a floor on evidence rather than a way out of the ratchet.

- A multi-line literal write is no longer truncated to its first line. A content marker alone on its line already introduced the whole block below it, but the same request with the payload starting on the marker's own line -- `with exactly this content: <document>` -- was cut at the end of the first prose sentence: a 1478-byte document was written as its 58-byte title, and `alpha\nbeta\ngamma` was written as `alpha`. Nothing in the request said the rest would be dropped and the write reported success, so self-authoring produced one-line stubs of the documents it was handed and looked broken for a reason absent from its own logs. Both spellings now deliver the same bytes; a payload that stays on one line keeps the bound it always had.

- The bar a degenerate cycle manufactured is no longer preserved by the rows that merely carried it forward. `target_percentage_basis_points` caches the ratchet walk, so refusing a one-line cycle as a *source* while still reading its cached result fixed nothing: `v0.349.0` is a legitimate 4050-line cycle and still carried the 2.91% that the one-line `v0.348.1` created two releases earlier, which would have left `main` red immediately after the fix merged. The bar is now recomputed from the rows that were entitled to raise it. A reviewed `target_override_basis_points` still replaces the ratchet outright, and the ratchet resumes from the level it set.

## [0.349.0] - 2026-09-12

### Added

- Hello World ladder: 20-language table, seeds, task generator and tests, plus the architect's standing vision guideline

### Fixed

- Test binaries no longer resolve the developer's own `~/.formal-ai` store. Sharing one store across a test binary let the parallel `seed_links::mirror()` rebuilds corrupt the size-balanced tree until `fix_size` overflowed and aborted the process, and left a 64 MB `memory.links` behind on the machine that ran the suite. An explicit `FORMAL_AI_MEMORY_PATH`, or a `HOME` a test has relocated itself, still wins.

- Self-authoring works again. `--summarize-session` and `--generate-title` default to true in the Agent CLI, and both make a second model call that ignores `--model` and goes to the CLI's own default provider; with no credentials that provider answers "OpenCode's free tier can only be used in OpenCode" and aborts the run *after* Formal AI has already written the artifact. The authoring script now disables both, so the loop that had looked broken produces a commit.
- The self-development floor relaxes after a day on a pull request, the same window `check-formal-ai-contribution.rs` already applied, so a release is not held by a requirement the architect said should yield after a day. A relaxed cycle reports as relaxed, never as satisfied; on a push to `main` it stays unrelaxed.
- The self-development floor answers the question the commit can answer. On a `pull_request` the cycle cannot contain a *merged* Formal AI pull request -- the branch under test is that pull request, and it merges after the check runs -- so the floor was unsatisfiable by construction there. It now counts the branch's own attributed commits, validated by the same walk the merged reading uses, and says the result is prospective. A push to `main` keeps the literal reading.

## [0.348.2] - 2026-09-11

### Fixed
- Issue #1123: fix(metric): go.sum is a lockfile the self-hosting share never counts. Authored by Formal AI through the Agent CLI.

### Fixed
- The self-development status now runs on pull requests, not only on pushes to `main` and the daily schedule. Both `7f3d61fee` and `5b0973f65` merged green and left `main` red, because no pull request could see the check that would have caught it (issue #1113).
- Lowered the `non_kernel_rust_lines` ceiling from 112805 to its measured value 112713, the shrink since `v0.348.0` that the kernel ratchet requires of a release (issue #1085 D1.4).

## [0.348.1] - 2026-09-10

### Fixed
- Issue #1120: fix(metric): composer.lock is a lockfile the self-hosting share never counts. Authored by Formal AI through the Agent CLI.

## [0.348.0] - 2026-09-10

### Added
- Issue #1076: a scheduled headroom audit (`.github/workflows/job-headroom.yml`, `scripts/check-job-headroom.rs`, `scripts/collect-job-durations.sh`) that reads real job durations from the Actions API and fails when a job spends more than 85% of its declared `timeout-minutes` — the repository previously enforced only that a *declared* budget stays under 70% of its cap, never that the *measured* runtime does.
- A workflow security audit: `zizmor` now runs over `.github/workflows` and `.github/actions` with `.github/zizmor.yml`, matching all four `link-foundation/*-ai-driven-development-pipeline-template` repositories, which the previous `actionlint`-only lint did not cover.
- `FORMAL_AI_CI_VERBOSE` runner telemetry (`scripts/report-runner-capacity.sh`) on the coverage job, default off, so the 7.4x runtime variance on identical tests can be attributed on the next occurrence rather than guessed at.
- A Links Notation parse failure now names the line that caused it. `links-notation` reports the unconsumed remainder of the file and no position, so one stray `:` in a `#` prose paragraph of `data/meta/ci-gates/check-job-headroom.lino` failed the whole test suite with a wall of quoted text and no line number; `tests/unit/lino_location.rs` locates it and holds the gate registry against a repeat. Reported upstream as link-foundation/links-notation#301 (the notation has no comment syntax, so `#` prose is structural) and #302 (the Rust errors carry no line or column, while the JavaScript port of the same version reports both).
- `.github/actions/cache-cargo-registry` gained a `restore-only:` input and a step-summary line per invocation, so a cache miss is visible in the run summary instead of only in a folded log group.

### Fixed
- The `Coverage / Code Coverage` job was killed by its `timeout-minutes` and reported `cancelled` rather than `failure`, so an overrun was invisible to branch protection. The `cargo llvm-cov` run now carries a `TEST_BUDGET_SECONDS` deadline through `scripts/run-with-budget-warning.sh`, which fails the step before the cap cancels the job.
- `actionlint` ran as a bare pinned binary. It delegates every `run:` block to ShellCheck and, when ShellCheck is absent, skips those checks and exits 0 — a green check that had verified nothing. It now runs as `docker://rhysd/actionlint:1.7.12`, which bundles ShellCheck, and a second step aims that same image at `tests/fixtures/actionlint/shellcheck-canary.yml` — a fixture whose only defect is inside a `run:` block — and fails if the fixture *passes*, so the gate cannot silently stop being one again.
- Four workflow `name:` scalars were unquoted and contained ` #`, which YAML reads as a comment: `Task Ladder (issue #840 dataset)` was stored as `Task Ladder (issue`. Valid YAML, so no linter reported it.
- Job caps measured against 400 `main` runs: `lint` (12.7 min against 15), `build` (11.6 against 15) and both release jobs (50.6 against 60) had turned their backstop into their deadline, and the 45-minute publish budget inside a 60-minute release job could never have fired. Caps raised to 25, 20 and 90.
- Docker layer caches were saved with `mode=min`, which stores nothing for a multi-stage compiled build, and were unscoped; they are now `mode=max` with `scope=docker-image`.
- The browser coverage baseline was ~12 points stale (functions 45.54% committed against 57.23% measured), so a real regression to ~46% would have passed the ratchet.
- The remaining five inline cargo-registry cache blocks now route through the shared composite action, so one registry no longer occupies six key prefixes in a shared quota.
- A dropped connection is no longer a build failure: `Agentic CLI Matrix` went red on a commit that changed no shell script when a 345 MB VS Code tarball stopped arriving mid-transfer (`curl: (18)`). All six network downloads in the repository -- three in `experiments/agentic_cli_matrix/install_client.sh`, two in the published `scripts/install.sh` and the composer bootstrap in `experiments/issue-1021-laravel/run.sh` -- now carry `--retry 3 --retry-delay 2 --retry-all-errors`; `--retry` alone does not cover curl exit 18, as `experiments/issue-1076/repro-curl-truncated-download.sh` measures. Two tests in `tests/unit/ci-cd/network_download_retry.rs` sweep every `*.sh` for a seventh one.
- `issue_896_component_boundaries` pinned the `Build Package` job cap with `contains("timeout-minutes: 15")`, so raising the cap to 20 -- more headroom than issue #896 asked for -- failed the test that exists to protect that headroom. The assertion now parses the cap and enforces a named floor.
- `Check Links` rejected the European Commission's general-purpose AI guidance with `403`. The page answers `200` to GET and HEAD from a workstation under lychee's own user agent, so the refusal follows the runner's address range, not the request; the host is now a documented `.lycheeignore` entry, since the workflow deliberately does not blanket-accept 403.
- `scripts/simulate-fresh-merge.sh` and `scripts/pin-base-commit.sh` fetched the base branch once, unretried, so a runner that lost name resolution for a moment (`Could not resolve host: github.com`, run 33973154494) failed a required check 30 seconds in and skipped every later step. Both now retry five times with a growing delay and still fail when the fetch never succeeds; `tests/unit/ci-cd/fresh_merge_fetch_retry.rs` pins both halves. Reported against the three templates carrying the same code as rust#157, js#169 and python#70.

### Fixed
- Issue #1075: a tool call is now grounded in *where* its effect lands before it is emitted. Codex advertises `codex_apps__github.create_file` beside its own `apply_patch`, and `classify_tool` read the substring `create_file` as a plain write capability, so a request to create a file in the checkout was routed to a GitHub connector; `response_arguments_for_tool` then filled the connector's required `repository_full_name` and `message` with `""` because they were missing, and the call answered 404 against no repository at all while the workspace stayed untouched. Scope is now read from what the client advertises -- required arguments first (`repository_full_name`, `project_id`, `session_id` and the rest), then whole address segments of the name -- through `src/tool_scope.rs` and the new `data/seed/tool-resource-scopes.lino` vocabulary, and `tool_for` refuses a remote-service or process-input tool for any capability that acts on the workspace.
- An argument that *names* a resource is no longer invented. A required identity is grounded from a repository URL the request actually carries, or the call is not made; a required enum with several options and no schema default is left to the client rather than silently resolved to its first option. An explicit `default` in the schema is still a statement by the client and is still honoured.
- A relative path in a write no longer resolves against the server's own directory. The Scala session in the issue wrote `/home/box/.formal-ai/general-change-plan.lino` while the task workspace was `/tmp/gh-issue-solver-1788563504540`, because `absolute_path` fell back to `std::path::absolute`. A workspace the client declares is now honoured even when the server cannot stat it (the transcript may have been recorded on another machine), a workspace the client never declares can be observed from an absolute path the client itself echoed back, and a byte-carrying write with no observed workspace keeps the requested spelling instead of landing in the server's directory. Reads keep the previous fallback.

### Fixed
- Issue #1079: `Security / Rust dependency audit` reported success while printing two findings. `cargo audit` classifies `unmaintained`, `unsound` and `yanked` as warnings, and a warning does not move its exit status — the run ended `warning: 2 allowed warnings found` and exited 0, with `Cargo.lock` pinning `chacha20 0.10.1`, a release its own authors had yanked. `scripts/check-rust-dependencies.sh` now passes `--deny warnings`, and both findings are answered rather than suppressed: `chacha20` is upgraded past the yank, and `fxhash` — genuinely compiled in through `web-capture → scraper 0.21 → selectors 0.26`, with `patched = []`, so no lockfile edit clears it — is ignored under a new proof form. `blocked-upstream` expires in the opposite direction from `unreachable`: it fails the moment the crate *leaves* the build graph, which is exactly when the upstream fix lands, and it must name the report it is waiting on. Filed as link-assistant/web-capture#155 with a build proving `scraper 0.25` needs no source changes.
- `.github/zizmor.yml` has declared `'*': hash-pin` since issue #1076 and it had never once been applied to a container image. That policy configures the `unpinned-uses` audit, which reads *action* references; images belong to the separate `unpinned-images` audit, which zizmor classifies as Pedantic while the job ran `--persona regular`. So `docker://rhysd/actionlint:1.7.12` — a mutable third-party tag executing with the repository checked out — sat inside the workflow whose purpose is auditing the pipeline. Both actionlint references are now digest-pinned, and a second zizmor pass runs `--persona pedantic --min-severity high --min-confidence high`: 0 of the 164 pedantic findings on the clean tree, 2 with the protections reverted. There is no narrower expression available — zizmor 1.29 and 1.30 both reject `rules.<audit>.persona`, and `remap` rewrites only severity, which is not what the persona filter reads.
- The zizmor step told a maintainer to reproduce with `zizmor==1.30.0` while CI could not install it. `zizmor-action` resolves versions from a static 37-row table shipped inside the action and `die`s on anything absent; v0.6.2's `latest` row carries the same digest as its `1.29.0` row, so the default does not float, it freezes one minor release behind. Both passes now set `version: 1.29.0` explicitly, making the next bump a visible line in the diff rather than a side effect of bumping the action.
- 46 of this repository's 48 `actions/checkout` steps persisted the job token into `.git/config` as an `http.extraheader`, where every later step — and every tool any of them shells out to — could read it. All five templates set `persist-credentials: false`; zizmor reports this as `artipacked`, at Low confidence, which is below both of the repository's gates (the default pass floors confidence at medium, the pedantic pass at high), so no configured check was ever going to report it. 42 of those 46 now drop it too, bringing the repository to 44 of 48; the four that keep the credential are the jobs that push with it — `external-benchmarks.yml` records scheduled upstream results with `git push`, `release.yml`'s `auto-release` and `manual-release` publish the version bump and its tag through `scripts/version-and-commit.rs`, and its `changelog-pr` job pushes a branch through `peter-evans/create-pull-request` — and each says so in a comment above the step. Verified that the eight jobs whose scripts `git fetch` the base branch still resolve the same head anonymously; measured the sweep as 46 findings → 4, the four being exactly those sites, with every other audit count unchanged. It is checked in both directions: `every_job_that_pushes_still_has_a_credential_to_push_with` reads each job body for a remote git write and fails if such a job drops its credential, because the first pass of this sweep removed the credential `auto-release` and `manual-release` push with, and both run only on `main` after a merge, where no pull request would have caught it. The `release.yml` cost moves it from 1521 to 1565 lines (1576 once the failure-time evidence dump below is added), so the warning band in `issue_999` and `issue_1012` moves with it: `persist-credentials` is an input to the action that *performs* the checkout, and a local composite action cannot wrap it, because a local composite action does not exist until the checkout has run.
- `tests/unit/ci-cd/issue_1017.rs` accepted only the `unreachable` proof form for an ignored advisory. That form cannot be written honestly for a crate that *is* compiled in, so an advisory whose only fix is upstream had no way to be ignored except by an unconditional suppression; it now accepts either form, and `issue_1079` rejects an entry carrying both.
- `Self-Hosting Evidence Check` reported a correct finding whose only remedy did not exist. It fails a pull request on a commit recording one of `Formal-AI-Session`/`Formal-AI-Evidence` without the other, on the premise that a commit still in review can still be amended — but ruleset 21300712 applies `deletion` and `non_fast_forward` to `~ALL` branches with an empty `bypass_actors`, so no pushed commit message in this repository can ever be rewritten (a `filter-branch` history that passed the metric locally was rejected with `GH013 … Cannot force-push to this branch`). That is the deadlock shape #796/#810/#812 already removed from the release path, surviving on the pull-request path only because nobody had mis-trailered a commit there yet. A later commit in the same measured range now withdraws an earlier claim with `Formal-AI-Retract: <full 40-character sha>`. The trailer is one-directional by construction — it moves a commit out of the numerator and there is no trailer that moves one in without the session evidence that was always required — so no retraction can raise the measured share; the target must be a full sha inside the range being measured and may not be the commit carrying it, and a malformed retraction follows the existing policy split, erroring under `Strict` and warning under `Lenient` so it cannot deadlock a release either — the lenient reader drops only the trailer it cannot resolve, because a release range that begins after a retraction's target makes that trailer permanently stale and discarding its siblings with it would return the commits they withdraw to the numerator. Applied on both walks, in `scripts/self-hosting-metric.rs` and `scripts/self-development-loop.rs`, because applying it on one only would let a commit leave the measured share and still count toward the release floor.
- The same gate read an *indented* `Formal-AI-*:` line as a declared trailer. `trailer_values` scans the whole commit body rather than using git's `%(trailers)` placeholder — issue #796, where a blank line between two trailers hid one of them — and it trimmed each line before matching the key, so a commit message that documented a trailer's format in an indented example thereby declared one. Reproduced by the commit that introduced the retraction trailer: it showed the format in its own message and the check answered `must name a full 40-character sha, found <full 40-character sha>`. `git interpret-trailers --parse` returns nothing for an indented line; so does this parser now, with the blank-line tolerance #796 needs untouched.
- Both Agent CLI end-to-end jobs failed on a *session title*. `@link-assistant/agent` summarizes every session by default, and `--compaction-model same` — the flag every harness here passed to keep that summary on the session's own model — is silently ignored: `src/cli/model-config.js` resolves the plural `--compaction-models` first and falls back to a non-empty default, so the single-model branch is unreachable. The summarizer therefore reached the head of the hosted cascade, `opencode/big-pickle`, which answered `Request is missing x-opencode-session`; that rejection is neither awaited nor caught at either call site, so `process.on('unhandledRejection')` exited 1 and aborted the turn that was still streaming. The plural spelling `--compaction-models "(same)"` and `--no-summarize-session` now go to all 25 harnesses CI runs, `LINK_ASSISTANT_AGENT_SUMMARIZE_SESSION: "false"` binds the two workflow jobs that reach the client outside a harness command line, and the same pair is pinned in `data/seed/client-integrations.lino` — the product path, so every `formal-ai with agent …` invocation stops sending a flag the client ignores. Filed upstream as link-assistant/agent#303 and #304, each with an offline reproduction that needs no account and no network beyond loopback.
- A failing Agent CLI job printed its exit status and nothing else: run 34061511110's `agent-cli-failure-report` step is two seconds of log ending `##[error]Process completed with exit code 1.`, with no message, no stack and no exit path. The harnesses redirect the client's streams to files and classify them afterwards with `scripts/classify-agent-cli-stderr.sh`, which does print an unexpected diagnostic and refuses to hide it — but `set -e` ends the harness on the client's own non-zero exit, one line before that classification runs, so the component built to speak is unreachable exactly when it has something to say. The cause was in the uploaded artifact and only there. `scripts/dump-agent-cli-evidence.sh` reads those same files back into the job log from an `if: failure()` step beside each job's existing artifact upload, in `release.yml`, `proactive-failure-report-e2e.yml` and `issue-1028-agent-ladder.yml`: `*stderr*` files first, because that is where the cause is; a missing path reported rather than passed over; and always exit 0, since a diagnostic that fails would turn one failure into two and bury the first. A green run never reaches it. This is the repository's own practice applied evenly — `experiments/agentic_cli_matrix/lib.sh` has tailed its serve, proxy and client logs from `matrix_fail` since the matrix legs were written.

### Added
- `tests/unit/ci-cd/issue_1079.rs`: fourteen tests pinning the defects above as invariants that fail without Docker, network or a CI run — each was a gate whose unenforced state was byte-identical to its enforced one.
- `tests/unit/specification/self_hosting_metric/retraction.rs`: six tests pinning the retraction trailer — that it unblocks a branch whose history cannot be rewritten, that it can only lower the measured share, that it withdraws the commit from the release floor as well as from the metric, that a short sha or a sha outside the measured range is an error, that a stale retraction does not restore the claims its siblings withdraw, and that an indented `Formal-AI-*` line is prose rather than a declaration.
- `dev/log/issues/1079/pulls/1080/`: the evidence the above is derived from — all ten workflow runs at `f971b8205` with logs and 23 annotations, the cargo-audit exit-status matrix, the zizmor persona inventory and live-gate proof, `zizmor-action@v0.6.2`'s version table, and immutable snapshots of all five `link-foundation/*-ai-driven-development-pipeline-template` trees. `README.md` reconstructs the timeline and registers all twelve defects, including the two that are warnings working as designed and the one true positive that must stay red.
- `dev/log/issues/1079/pulls/1080/upstream-reports/`: the eight reports filed against other repositories for defects that reproduce there, indexed against the fifteen issue URLs they were filed as — rust#164/#165/#166, js#177/#178, python#75/#76, php#5/#6/#7, csharp#52/#53, web-capture#155 and agent#303/#304 — each with a reproduction that runs at the snapshotted commit, a workaround, and the code-level fix.
- `experiments/issue_1079_agent_compaction_flag/`: an offline reproduction of both upstream Agent CLI defects — a stdlib-only OpenAI-compatible mock and a runner that measures which model the summarizer reaches for and, with `MOCK_FAIL_NON_STREAMING`, that a failed summary aborts a healthy turn. Both switches default off. It exits non-zero when either defect stops reproducing, so the workarounds above can be dropped the day upstream fixes them.

### Fixed
- Issue #1081: the `CI/CD Pipeline` run at `main`'s tip failed, and the step that failed was a step this repository had itself been growing past its own deadline. `Test (macos-15-intel / specification)` is wrapped in a 1400-second execution budget; the specification suite took 424 seconds when the budget was written and 1401 seconds on the failing run, at which point `timeout(1)` killed it with exit 124 and `Pipeline Status` reported `Pipeline failed. Failing jobs: test`. Every warning the repository had built for exactly this — `run-with-budget-warning.sh` emitted its 70% notice at 980 seconds — fired into a job log nobody reads until the job is already red. The budget now covers only what it was measured against: the dependency build moves into its own preceding step, so the budget times the suite rather than the suite plus however long the toolchain took that morning.
- The budget invariant compared in one direction only. `check-execution-budgets.rs` asserted every budget expires before its job cap and never asked whether a budget still bounds the work inside it, so a step could approach its own limit indefinitely and stay green until the run it killed. `scripts/check-step-budget-headroom.rs` reads the measured durations back out of the run history and fails a budget the observed maximum is within 15% of, which is the check that would have reported the defect above eleven days before it landed.
- The measurement feeding that comparison silently dropped its own evidence. The duration sweep filtered runs to `conclusion == success`, so the runs where a step ran *longest* — the ones it was terminated on — were the runs excluded from the maximum: measured over successes the specification step peaked at 84.4% of its budget, and including terminated runs it peaked at 100.1%. Survivorship is now an error rather than a default; job-level utilisation moves 62.9% → 74.7% with the filter removed.
- `Desktop Release` reported `skipped` on 102 of its last 107 runs and `success` on 5. A workflow whose steady state is grey cannot be read as a signal in either direction; its `on: push` trigger now names `branches: [main]`, so it runs when it has something to do and is absent when it does not.
- `release.yml`'s 312-line, 51-step Agent CLI end-to-end job is extracted to `.github/workflows/agent-cli-e2e.yml` behind `on: workflow_call`, taking the file from 1576 to 1407 lines. The `env:` block is copied rather than referenced, because a reusable workflow inherits nothing from its caller — the failure mode that makes this extraction worth a test of its own.
- Three `cargo test` steps ran with no execution budget at all, bounded only by their job cap, which GitHub reports as **cancelled** rather than failed when it fires. Each is now wrapped, and the check that finds unwrapped long-running steps no longer trusts a hand-maintained exemption list.
- `sccache` recorded 843 successful writes against 868 write errors in a single run — a cache reporting a hit rate while more than half of its stores failed. `scripts/check-sccache-write-health.sh` reads `sccache --show-stats` at the end of a job and fails when the write error share crosses a threshold, so a cache that has quietly stopped caching says so.
- Nothing verified that `main`'s tip had a pipeline run. A push made with `GITHUB_TOKEN` does not start a workflow run — documented GitHub behaviour, and the reason the benchmark-ledger commit at `dda02efb` sat at the tip of `main` having never been built. `scripts/check-head-pipeline-coverage.rs` reports the gap instead of leaving it invisible; it is a report and not a trigger, because a trigger here would re-enter the same suppression.
- The shared-branch writer pushed without a rebase-retry, so two scheduled jobs writing the same branch raced. `scripts/push-to-shared-branch.sh` retries against a fresh base, with `PUSH_MAX_ATTEMPTS` and `PUSH_RETRY_DELAY_SECONDS` overridable and defaulted.
- 23 test cases in four `scripts/*.rs` files were executed by nothing. `cargo test` does not build rust-script programs, and the gate registry ran three of those four *without* `--test`, which runs the check and not its tests. `scripts/test-scripts.sh` derives the selection — every `scripts/*.rs` carrying a `cfg(test)` suite that neither the unit test crate nor a registered gate already covers — so a script added later is picked up without anyone remembering the file exists.
- The scan written to find unenforced gates had two false negatives of its own: `run_by_a_gate` missed a multi-job reusable workflow and `job_needs` returned empty for a `needs:` written as a wrapped list.
- A budget-headroom `::notice` fired on 6 of the 12 steps that were correctly configured. The marker it is derived from stays; the annotation does not, because a warning that is wrong half the time trains a reader to skip the half that is right.
- `Broken Link Checker` reddened `main` on a URL that answers 200, and the failure arrived 1.5 seconds into a step configured with `--max-retries 6 --retry-wait-time 2` — six retries with a growing wait do not fit in 1.5 seconds, because for this failure no retry ran. Issue #1045 had already met the same reset and answered it by raising `--max-retries`; the setting cannot cover this class at any value. lychee decides retryability by the phase an error happened in before it looks at what the error was (`retry.rs` answers `is_connect()` with a flat `false`, above the `should_retry_io` that lists `ConnectionReset` as retryable), so a reset during connect or the TLS handshake is never retried, while the identical reset one byte later is retried six times — measured, 1 connection attempt against 6, in `experiments/issue-1081-lychee-connect-retry/`. `scripts/recheck-broken-links.mjs` re-asks only the links **no host answered**; a failure carrying a status code is an answer and is final, so a `404` is never re-checked and nothing real is hidden. The script exits 0 in every case: it can downgrade a failure, never raise one.
- Three of the fixes above were themselves put through the pipeline they change, and it found the defect this issue is about in the tests written for it. `release_preflight.rs` set `CARGO_TOKEN` and inherited `CARGO_REGISTRY_TOKEN` from the job, which `preflight-credentials.sh` prefers, so on a runner the probe authenticated with the real credential while the assertion looked for the fixture's — green on every developer machine, red in CI, the same verdict disagreeing with itself depending on where it ran. The suite now clears all twenty variables the script reads before setting the ones a case provides, and a test scans the script for expansions the list has fallen behind on.
- The gate that requires the release pipeline to drive issue #707 through the real Agent CLI asserted on a path in `release.yml`, so the extraction above read to it as a deletion. It now follows the call into `agent-cli-e2e.yml` and checks both ends — a caller that stops calling and a callee that stops running the harness are the same false negative, and a gate that checked only one end would have missed one of them.
- The same class again, twice, from the link-checker fix: two gates written in earlier rounds quoted `links.yml` verbatim — one line of the `node --test` step, one line of the `Fail if broken links were found` condition — and the fix above reflowed both across more lines without removing anything from either, so both gates failed. An assertion that quotes a workflow tests the typography of a file YAML lets you write several ways; it fails on every legitimate edit, and it trains its reader to re-paste the literal without reading what changed. Each now asks its property instead — `node --test` runs before `lycheeverse/lychee-action` with the parser's suite named between them; the condition carries both `!cancelled()` and `steps.lychee.outputs.exit_code != 0` and has not widened back to `always()` — checked against a whitespace-normalised copy of the workflow.

### Added
- `scripts/preflight-credentials.sh`: principle 16 of `CI-CD-BEST-PRACTICES.md`, implemented. Every publishing credential is probed before the build rather than by the step that first needs it, and the probe is a **write** — an OCI blob upload session, opened and immediately deleted — because a login is not evidence of push access: docker.io answers a `pull,push` scope request with **200** and a pull-only `access` claim, while ghcr.io answers **403 DENIED** to the same probe done wrong. It reports every failing credential rather than stopping at the first, and reports `unknown` where it cannot tell rather than guessing. The tokens travel to `curl` in a `-K -` config document on stdin rather than in `-H` arguments, because an argument list is world-readable for as long as the process lives (`/proc/<pid>/cmdline`, `ps`) — a probe that leaked the credential it was verifying would be a poor trade for the minute it saves.
- `tests/unit/ci-cd/issue_1081.rs`: the defects above pinned as invariants that need no CI run to fail.
- `data/meta/ci-gates/test-script-suites.lino`: the gate that runs the standalone script suites in the `rust` stage.
- `dev/log/issues/1081/pulls/1082/`: the evidence — every run at both of `main`'s recent heads with logs and annotations, the step-duration tables the survivorship finding is measured from, the sixteen-principle audit of `CI-CD-BEST-PRACTICES.md`, and immutable snapshots of all five `link-foundation/*-ai-driven-development-pipeline-template` trees. `README.md` reconstructs the timeline and registers all twenty-two defects, including the one that remains open and why.
- `dev/log/issues/1081/pulls/1082/upstream-reports/`: seven reports filed as nineteen issues — `persist-credentials` coverage (js#179, php#8, csharp#54), a missing terminal `pipeline-status` job (php#9, csharp#55), missing execution budgets (php#10, csharp#56), a budget invariant returning both kinds of wrong answer (js#180, with a verified patch), and no release preflight in any of the five (rust#167, js#181, python#77, php#11, csharp#57). The last two went in the other direction — a false positive that reddened `main` here, root-caused into lychee (lycheeverse/lychee#2297, with a patch that was built and measured rather than proposed) and then found in all five templates, which run the same `--max-retries` and have no re-check (rust#168, js#182, python#78, php#12, csharp#58). Each carries a reproduction at the snapshotted commit, a workaround and the code-level fix. It also corrects a claim from the #1079 round: `persist-credentials: false` is present in all five trees but its coverage is 26/26, 2/27, 18/18, 2/11 and 1/13.
- Two gaps too large to land here are filed rather than left implicit: #1083 (JavaScript lint coverage) and #1084 (published container images are amd64-only while the base image is multi-arch, with the `--platform linux/amd64` workaround and the digest-list build that fixes it).
- Verbose switches for the new diagnostics, all default off: `FORMAL_AI_CI_VERBOSE`, `PREFLIGHT_VERBOSE`, `SCCACHE_LOG` and `RECHECK_VERBOSE`.

### Fixed
- Issue #1085 / #1081: the release preflight no longer reports the crates.io publish token as "revoked, expired or misscoped". `GET /api/v1/me` is cookie-only in crates.io (`AuthCheck::only_cookie()`), so every API token is answered 403 there; run 34149311523 blocked a release with the token that had published v0.347.0 two days earlier. The probe now records the token as `unknown` with the reason, never sends it anywhere, and leaves `cargo publish` as the step that proves it.
- Issue #1085 / #1081: the macOS test-archive build budget is sized from the measured runner spread (11 to 26 minutes over ten `main` runs at a 93% compiler-cache hit rate): 1800 s under a 55 m cap, 66.7% with the doc-test budget beside it.

- Issue #1085 (D2.3): the self-authored pull-request workflow authored its task again on every re-run (six duplicate commits on #1093, two on #1094) because its guard piped `git log` into `grep -q` under `pipefail`, where a present trailer makes the pipeline fail. The decision is now `scripts/self-authored-commit-count.sh` (jq over the pull request's commits), taken again right before the push; pushes go through `scripts/push-to-shared-branch.sh`.
- Issue #1085 (D4): the Agent CLI ladder records how many of its 32 leaves Formal AI actually changed (15 in the first run under the compile-and-test criteria) and the workflow fails a full-width run that passes fewer; the previous comparison was inverted and errored when the measured level was deeper than the record. The seventeen failing leaves are three mechanisms, filed as #1095 and #1096 with the per-leaf evidence.
- Issue #1085 (D1.3): the 54 tokens the rule and response seed files introduced are defined in the total closure (`scripts/close-total.py`), the self-AST census and method-registry tests resolve a rule-backed handler to the interpreter's `run_handler`, and the recursive handler-source count is 45.

### Changed
- Issue #1085 (D3): the self-hosting metric is at version 3. A commit counts as self-authored only when its `Formal-AI-Model` trailer names formal-ai and the committed evidence names that model; a session id or model naming a hosted model is ordinary work. Only behaviour-changing paths count on either side of the share: `docs/`, `dev/`, `experiments/` and `changelog.d/` are excluded like captured artifacts. Ledger rows name who opened each qualifying pull request, and `--replay-epoch` restates earlier tags under the new definition by appending rows.
- Issue #1085 (D3.5): the self-development floor no longer gates releases. `release.yml` and `scripts/version-and-commit.rs` record the row and cut the release on CI correctness; `.github/workflows/self-development-status.yml` runs the floor red-until-true on every push to `main` and daily, with no budget, window or bypass. This reverses the release-path placement from #924 and #1066 because the floor had become satisfiable by documentation commits carrying trailers and had held a downstream-critical fix back for 268 commits (#1064).

### Fixed (upstream benchmarks)
- Issue #1085 (D5.3): the seeded HumanEval and MBPP tasks scored 0 under the upstream prompt shape for two mechanical reasons the scheduled log named. The HumanEval candidate copied `numbers: List[float]` from the upstream signature without the prompt's `from typing import List`, so its own verification raised `NameError` at definition time and the solver fell back to the unknown opener; the MBPP candidate copied `similar_elements((3, 4, 5, 6), ...)` out of an `assert` as if it were a signature and did not parse. The synthesis handler now carries the prompt's import lines with the candidate and accepts only a parameter list as a declared signature, and the minimal-script route (hoisted by the bare tokens `code`/`script`) stands aside for a prompt that specifies a function to derive, in any language, so the `Write a function to … Reply with the Python code` prompt shape reaches the synthesis handler instead of the hello-world template.

### Added
- Issue #1085 (D2.3): `.github/workflows/self-authored-pull-request.yml` lets Formal AI author a change from an issue labelled `formal-ai-solve` and opens the pull request under the GitHub Actions bot with the self-hosting trailers on the authored commit; #1091 is the first task.
- Issue #1085 (D2.2, D2.3, D4): a requirement that names behaviour resolves to its file through the self-AST census (`resolve_requirement_target`); the 32 ladder leaves are committed link-edit rules; the Agent CLI ladder compiles and unit-tests every leaf, merges both children's diffs at depth 4, verifies requirement-shaped prompts at depth 3 and above, runs on pull requests and weekly, and ratchets the deepest passing level in `data/meta/ladder-ratchet.lino`.
- Issue #1085 (D1.1, D1.2): `src/seed_links.rs` loads every bundled seed document and the routing meta documents as one links network at startup, mirrored into a native link-cli store beside the memory store when the server starts; handler precedence, cue lookup and intent routing read it through link queries instead of each owning a parser.
- Issue #1085 (D5.1, D5.2): `formal-ai benchmark run --frontier-record` rewrites `data/meta/learning-frontier-upstream-benchmarks.lino` from every failed upstream case, the scheduled workflow commits it beside the ledger, and `formal-ai learn cycle --frontier upstream-benchmarks` replays it; `formal-ai benchmark ratchet` prints a warning for a suite whose last three runs scored the same.
- Issue #1085 (D1.3): `src/rule_interpreter.rs` interprets `data/seed/handler-rules.lino`, so a specialized handler can be links instead of Rust; eleven handlers (fourteen rules) migrated and their Rust was deleted, taking the migration ledger from 51 pending to 40. Their English-only wording moved to `data/seed/multilingual-responses-policy.lino` in en, ru, hi and zh. A unit test injects a rule for an intent no Rust knows and routes it in four languages.
- Issue #1085 (D1.4): `data/meta/kernel-ratchet.lino` names the Rust kernel and five measured ceilings for everything outside it; `scripts/check-kernel-ratchet.rs` (gate `check_kernel_ratchet`) lets a ceiling move down and never up, and the status workflow requires the non-kernel line ceiling to be lower than at the previous tag. The raisable `specialized_handler_files_max` / `try_dispatch_entries_max` ceilings, raised twice in August, are retired.
- Issue #1085 (D4): every `.rs` leaf of the Agent CLI ladder is now compiled (`cargo check --lib`) in `verify-node.sh`, with one target directory per run.
- Issue #1085 (D5.4): the upstream benchmark row (HumanEval 0/20, MBPP 0/20, GSM8K 2/20, MATH 0/20, CoEdIT 0/20, SWE-bench Lite 0/1 on 2026-09-07) now stands beside every curated 13/13 citation in `VISION.md` and `ROADMAP.md`.

### Fixed
- Issue #1091: fix(metric): Gemfile.lock is a lockfile the self-hosting share never counts. Authored by Formal AI through the Agent CLI (issue #1085 D2.3).

### Added
- Issue #1085 (D2.3): the self-authored loop is a composite action, `.github/actions/author-with-formal-ai`, that another repository can install. Formal AI comes from the published container by default so a consuming repository spends no compile, the task contract can be relaxed to attempt every new issue rather than only labelled ones, and the shared scripts are fetched from this repository rather than vendored. `self-authored-pull-request.yml` is its first consumer and passes `formal-ai-source: source`, because a change to the meta algorithm must be measured by the branch making it. link-assistant/hive-mind#2233 asks for the first outside installation, on `issues: opened`.
- Issue #1107: `.github/actions/formal-ai-binary` provides `target/release/formal-ai` from a cache keyed by the content of the sources it is built from. Seven workflows compiled the same release binary on every push, two to five minutes each; a push that changed no source now reuses the build and installs no Rust toolchain.

### Changed
- Issue #1107: a workflow file no longer counts as a code change for the pipeline's heavy jobs. `any-code-changed` matched every `.yml`, so editing a scheduled benchmark bought the macOS archive, the Docker check, six box-image legs and a 25-minute agent end-to-end run; lint still runs for a workflow edit, and `.github/workflows/workflows.yml` audits it independently.
- Issue #1107: the pipeline's heavy jobs (the macOS archive, the Docker image check, the six box-image legs and the 25-minute agent CLI end-to-end run) are gated on a new `pipeline-changed` output instead of `workflow-changed`. The old flag was true for any file under `.github/workflows/`, so editing a scheduled benchmark bought the full cost of a code push; the new one is true only for the pipeline's own definition, a composite action it calls, or a script those run.
- Issue #1085: the CI contract tests read `.github/actions/**` as well as `.github/workflows/**`. The shell CI executes is the same shell whichever directory it sits in, so moving a `git push` or a credentialed checkout into a composite action must not move it out of review.
- Issue #1085 (D2.3): `docs/github-action.md` documents installing the action in another repository — the `issues: opened` configuration that attempts every new issue, every input, the task contract, and how to read a draft. The README and the landing page link it.
- Issue #1085 (D2.3): a bot pull request the action resumes catches up with its base by merging it before the branch is judged. #1103 was nine commits behind and red on two tests its own base had already fixed, which measures the base rather than the authored change. It merges rather than rebases because this repository answers a force push with `GH013: Cannot force-push to this branch`; the catch-up merge is made by the bot, carries no attribution trailers, and so is not counted as authored work.
- Issue #1085 (D2.3): the `Self-authored backlog` workflow reports every open `formal-ai/*` pull request daily with its age, check state and how far its base has moved, names which of the three actions each one needs, and fails once one has been open more than three days. Seven were opened on 2026-09-08 and none was merged; nothing in CI said so. A draft is finished when it is merged or closed with its defect filed, never by being left open.

### Security
- `js-yaml` is updated from 4.3.1 to 4.3.2 in the desktop and VS Code lockfiles, closing GHSA-2883-xcg3-v3hh (high: `maxTotalMergeKeys` does not limit CPU use for empty merge sources). It is a transitive dependency of the Electron builder tooling in both projects; the patched release is on the same major line, so nothing else moves.

### Fixed
- Issue #1106: `formal-ai serve` no longer stops answering every request while one is being solved. The accept loop handled each connection inline, so a single slow chat completion blocked unrelated callers — including a `GET /v1/models` that had answered a second earlier — which is why the failure was reported as a permanent wedge. Each connection is served on its own thread now. The underlying slowness is separate: every chat completion opens the memory store and hands its events to the solver, so the cost grows with accumulated history (measured: 21 ms for `formal-ai solve` against a store that takes the server over 60 s). This change stops it from being an outage for every other caller; the slowness itself is fixed separately in this same release.

### Fixed
- Issue #1106: recording a chat exchange no longer rebuilds the entire native link-cli projection. Every completion persists the memory, and persisting replaced the whole graph in one transaction, so a store of 400 events rebuilt all 400 to append three — measured at 51.8 s of a 53 s request, growing with the square of the accumulated history. That was the slowness the previous fix in this release deliberately left open, having only stopped it from blocking unrelated callers. The projection now records how many events it holds, in a marker beside the database, and a completion appends only what is new: the same request costs 0.35 s. The graph is unchanged by this — a store built incrementally and a store rebuilt from scratch were compared byte for byte, and both the 67 MB database and its 1 736 recorded addresses are identical. A prefix that does not describe the database (a `.lino` replaced underneath it, a marker left by an interrupted write) is detected and rebuilds instead, so the reduction is an optimization and never a weakened guarantee.

### Fixed
- Issue #1072: the Agent CLI ladder no longer copies the repository once per node. Each node extracted a 966 MB `git archive` (most of it `dev/` and `docs/` evidence) into a fresh temporary directory, `git init`ed and committed it, then compiled there — a path cargo had never seen, so every leaf rebuilt the crate from scratch. Measured on run 34326451343: 110 minutes for 32 leaves, 25 of them Agent CLI work. One sparse worktree is created once and reset between nodes, so the second leaf's `cargo check` is incremental and the agent's snapshot store is created once instead of per node (115 stores, 31 GB, in six hours). Each node's Agent CLI turn is bounded (`LADDER_NODE_BUDGET`, default 240 s; leaves that produced no proof had run 236 s unbounded), one line per node reaches the job log while the step runs, and `verify-node.sh` prints its `cargo` timings.
- Issue #1109: opening a link-cli store removes replacement databases left by processes that no longer exist. A rebuild killed partway leaves a 67 MB `.<name>.database.<pid>.<n>.tmp` beside the store; 1.1 GB across seventeen dead process ids was found beside one store. Only files whose process id is dead are removed.

### Changed
- Issue #1107: a check that already passed on identical inputs is not run twice. `detect-changes` compares the whole pull-request range, so a branch that once touched `src/` re-ran every Rust check on every later docs-only push. Eight checks (unit tests, macOS core tests, the Docker image, the six box projects, the local E2E, the agent-CLI E2E, the #1028 ladder and the agentic CLI matrix) now key a green marker on the content of their inputs through the new `green-ledger` action; on a hit the job says so in its log and step summary and finishes without running its body — a reported skip, never a silent one. Markers are saved only by a job that succeeded, and `main` never skips. Two ladders (Write-Effect, Task) run on a branch only when the branch changes them, otherwise on `main` and daily; the agent-CLI E2E runs its held-out generalization gates on a branch and the full replay on `main` (3½ minutes against 25).

### Fixed
- Issue #1095: a turn that is only a continuation cue ("Continue if you have next steps", "continue", "продолжай", "जारी रखें", "继续") resumes the task already established in the agentic session. The only recovery path looked for a compaction envelope, which an ordinary tool loop never has, and matched the English phrase alone; the bare cue became the request and its words went to web search — eight ladder leaves ended on prose from `docs.continue.dev`. The cue is a seed role now, compared as a whole prompt through the rule interpreter's new `whole` mode, and a cue with nothing to resume is answered ("Nothing is in progress to continue") instead of searched for.
- Issue #1096: a replacement in an existing file ("In the file src/x.rs, replace "A" with "B" … keep it valid Rust") is no longer claimed as a request to generate that file. The generation step synthesised its own artifact and verified the agent's correct edit against it, failing seven ladder leaves on "the observed bytes differ". It declines whatever the edit reader recognises, and the edit route claims it.

### Fixed
- Issue #1099: a coding task that names two artifacts is no longer finished after the first. One prompt named two files; Formal AI edited the first, answered `Added "Gemfile.lock" to the list ... and observed the result.`, and ended the session in five seconds with the second file never written. The whole prompt had arrived, so the second clause was read and dropped. A request is now split at its own enumeration cues (a seed role, five languages), each clause is planned exactly as a standalone request would be, and the session cannot answer `Final` while an artifact it named is still missing. A request naming one artifact is unaffected: splitting is refused unless two clauses each name one, because reading one obligation as two would invent work nobody asked for.

### Fixed
- Issue #1105 (RC5): two conversations that open with the same words are no longer one conversation. With no `x-formal-ai-dialog-id` header — which is every opencode session — the dialog id was a content hash of the first user message and nothing else, so in the reported store a single collided dialog had absorbed 52 % of every exchange and exporting it returned another conversation's turns. The fallback id now mixes in a marker that is constant for one conversation and distinct across two, keyed by the log directory the client writes to as well as its opening prompt.
- Issue #1105 (RC7): a report that filed a GitHub issue while an export failed said only that the issue was filed. `report_finished` read the issue URL and nothing else; it now checks every target's command through the same failure classifier the verification path uses, and names the destination that failed. The command order is shared with the pairing, so a failure is attributed to the target that produced it.
- Issue #1105 (RC4): `context learn --session latest` resolves `latest` like every other session subcommand. It passed the word through as a literal conversation id — which never exists — so the one command that teaches Formal AI from a session could not name the session the user was in, while `context export` beside it resolved the same word correctly.
- Issue #1105 (RC6): resolving `latest` to the most recently recorded conversation is a guess, and the caller is now told when that is what answered. The newest dialog file on disk is whichever conversation any client wrote to last, which is how the reported session exported a different conversation than the one it was reporting on.
- Issue #1105 (RC2): the inline context budget falls from 50 000 bytes to 20 000, so a transcript of the size the reported issue carried (45 KB) moves to a gist with an excerpt left in the body instead of being pasted whole. `--max-inline-bytes` still raises it.

### Fixed
- Issue #1110: the rename command Formal AI emits runs on macOS. It was `sed -i 's/\bX\b/Y/g' -- FILE`, which is GNU-only twice over: BSD sed reads the script after `-i` as a backup suffix, and BSD sed has no `\b`. Every rename on a Mac therefore failed with `bad flag in substitute command`, left the file untouched, and was reported by Formal AI as its own verification failure — a confident failure report instead of a rename. The command is `perl -pi -e` now, which means the same thing on both. Verified by running ladder leaf 2.2.2.2.1 on macOS, where it now passes.
- The #1028 ladder's sparse checkout keeps `docs/` and one path under `dev/`: 31 `include_str!` sites compile files from `docs/case-studies/` and one from `dev/log/`, so excluding those trees made `cargo test` fail to compile inside a node — which the harness scored as the leaf's own failing tests rather than as its own breakage.

### Fixed
- Issue #1101: the same documentation question is now answered the same way in every language. `how does pandas DataFrame.join work?` was answered from the documentation rule, while its Russian, Hindi and Chinese translations were answered with the web-search handler's offline-fetch notice. The cause was not the precedence order the issue suspected: the last branch of the web-search cascade — an interrogative naming an engineered brand, carrying no search imperative — claimed all four, and English escaped only by accident. Once the question opener is stripped, the English residual begins with `does`, which the seed lists as a `non_referential_subject` so that "does it …" is rejected; Russian, Hindi and Chinese form the same question without do-support, so nothing rescued them. That branch now asks the rule set whether a documentation rule already answers the prompt, which is language-neutral by construction. An explicit search imperative still reaches web search in all four languages, and a brand question no documentation rule covers is still searched for.

### Changed
- Issue #1111: non-Linux CI is temporarily non-blocking, so the release path is no longer held up by it. `main` produced no release after v0.347.0 (2026-09-05) because the macOS test-archive build kept being killed at its execution budget — most recently on merge commit `7f3d61fee` at a 19.32% compiler-cache hit rate, against ~93% when healthy. That budget had already been raised twice for the same reason (1200 s → 1400 s → 1800 s), so raising it again would have cost another twenty minutes of wall clock per run and bought no confidence. Nothing was deleted: the macOS job, the reusable workflow it calls, and the `macos-15-intel` matrix leg are all still defined, and setting the repository variable `FORMAL_AI_NON_LINUX_CI` to `run` restores them with no code change. A skipped platform annotates the run and writes to the job summary, so it can never be misread as coverage, and a macOS job that genuinely *fails* still blocks the build — only `skipped` is accepted. Linux keeps its full suite.

## [0.347.0] - 2026-09-05

### Added
- Issue #1073: a reasoning standard declared as data (`data/meta/reasoning-standard.lino`) and evaluated as pure predicates (`src/reasoning_standard/`). Seven gates — evidence before claims, documentation by default, formalized instructions, computed source trust, refutation variety, verify-after-act, honest failure reporting — are audited on every request, with no mode in front of the call. A gate that does not fire reports the trigger that was false, so the obligations are enumerated identically on a trivial request and a hard one.
- `data/meta/reasoning-standard-reference-episode.lino` encodes the reference dialog the standard was derived from; every gate is shown to fail under a mutation that removes the behaviour it enforces.
- `data/meta/reasoning-standard-recipe.lino` describes the procedure as data, grounded against the live source by `tests/unit/specification/reasoning_standard_meta_algorithm.rs`.

### Changed
- Source trust is derived rather than declared. Every source in `data/seed/sources-registry.lino` carries a `primacy` chain citing the site's own policy, and `SourceRecord::tier` is now `PrimacyChain::derive_tier()`. The hand-written tier survives only as `asserted_tier` and is checked against the derivation; `tier_from_seed`, with its silent `_ => independent_corroboration` arm, is gone.
- The meta core's depth defaults moved from the quiet setting to the full one: `RecursionMode::Down` → `Both`, `SelectionMode::Off` → `Record`, `SkillMode::Off` → `Accumulate`. The narrow modes remain for deliberately quietening a trace, but reasoning depth is no longer conditional on a caller asking for it.
- The recursive core recipe gains a thirteenth step, the unconditional reasoning-standard audit.

### Fixed
- Two delivery-document tests hard-required a changelog fragment to still be on disk, so they failed for every commit after the release that consumed it — `v0.346.0` deleted the fragments they read. `tests/unit/ci-cd/issue_1014.rs` and `tests/unit/issue_1021_closed_circle.rs` now follow the entry across its lifecycle, reading the fragment before release and the `CHANGELOG.md` section after, the way `tests/unit/docs_requirements_issue_656.rs` already did.
- `examples/regenerate_issue_922_open_proposals.rs` regenerates `examples/issue-922-method-learning/open-proposals.lino` from the live learner instead of leaving its content-addressed candidate id to be hand-edited whenever a pipeline stage is added. It refreshes only the machine-derived fields and keeps the two review decisions the document carries: the single strongest proposal, and the reviewer's own summary sentence.
- `data/seed/learned-methods.lino` is re-derived through the production promotion path instead of hand-edited: the thirteenth pipeline stage lengthens the recurring recursive-core tail from twelve operations to fifteen, so the adopted method is now the 851-byte `learned_recursive_core_e17957243eaaf6db`. The three canonical gates were replayed fresh for it (4/4, 13/13, 12/12) and the decision record is kept in `docs/case-studies/issue-1073/logs/issue-922-promotion-rerun.lino`.
