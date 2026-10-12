# Changelog archive: 0.352.0 to 0.352.1

Releases in this range, newest first. Newer releases are in [CHANGELOG.md](../../CHANGELOG.md).

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
