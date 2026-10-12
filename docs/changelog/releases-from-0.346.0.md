# Changelog archive: 0.346.0

Releases in this range, newest first. Newer releases are in [CHANGELOG.md](../../CHANGELOG.md).

## [0.346.0] - 2026-09-04

### Added

- Added the replayable Hive Mind full-circle integration gate in both
  directions, with committed workspace effects and honest failure propagation
  for #921.

### Added

- Added the #924 Formal AI self-development release loop: every cycle now
  requires a merged, session-backed pull request and a non-decreasing
  self-hosting target recorded in the release ledger.

### Fixed

- Run installation conversion, program synthesis, coding catalog, numeric-list,
  and rule synthesis through one shared seven-stage meta-algorithm builder in
  both Rust and the browser worker.

### Fixed

- Partition long macOS tests, remove false error and warning output from CI,
  reuse one Box-language binary, and retain opt-in cache diagnostics for future
  backend failures.

### Fixed

- Fix issue #1014 in pull request #1015: defer ineligible automatic releases
  without weakening manual release evidence, reuse one macOS nextest archive,
  keep its test binaries relocatable, audit every JavaScript lock, package the
  browser runtime safely, and remove misleading Gemini, dependency-graph, and
  lifecycle diagnostics with tests-first evidence.

### Added

- Compile verified substitution-rule programs to standalone Rust,
  Rust-to-WebAssembly, and JavaScript interoperability artifacts through one
  target-neutral IR.

### Fixed

- Fix issue #1017 in pull request #1018: make the step execution budget own the
  deadline instead of `timeout-minutes`, so an overrun reports `failure` with an
  `::error` naming the budget rather than degrading into a `cancelled` run and a
  skipped release. Every budget is now checked against the job cap it sits
  under, which surfaced two further at-risk jobs; the macOS core lane runs
  sixteen duration-skew-tolerant slices; `cargo audit` runs on the default
  branch and on a schedule with its one false positive ignored behind a proof
  line CI re-derives; the CodeQL Rust extractor is pinned to a `std` it can
  parse so live code stops being extracted with errors; the link check tests its
  report parser and no longer reports links it never checked; every read-only
  job belongs to a concurrency group that never cancels the default branch; and
  nested CI evidence is no longer silently excluded by `.gitignore`.
- Answer the first request in a process without round-tripping a whole module's
  CST/AST. Rule recall built the canonical learning ledger — and therefore parsed
  the pinned planner module — before checking whether the ledger could answer the
  prompt at all, which cost over ten seconds inside the *first* HTTP response and
  timed out two macOS integration tests at the harness's thirty-second limit.
  The lookup now proves a miss from the canonical failure trace before building
  anything, and the pinned round-trip is computed once per process. Recall
  behaviour is unchanged and no promotion gate is relaxed. Set
  `FORMAL_AI_TRACE_SLOW_INIT=1` (off by default) to report each whole-source
  parse with its size and duration.
- Stop a `python3` agent command's *start-up* from deciding whether it succeeded.
  Commands run with a cleared environment, which on macOS also removed `TMPDIR` —
  where `/usr/bin/python3`'s `xcrun` stub keeps the resolution cache — so every
  invocation paid a full re-resolution and a loaded runner exceeded the
  fifteen-second floor while the command itself was fine. The child now receives
  one constructed `TMPDIR` and nothing else, the floor is a sixty-second backstop
  documented against measurements rather than a frozen literal, and
  `FORMAL_AI_TRACE_COMMANDS=1` (off by default) reports the executed path, the
  budget and the elapsed time.
- Stop a *successful* macOS desktop package from being reported as a failure.
  electron-builder downloads its toolsets with a single request whose only
  deadline is ten minutes, and a stalled one is recorded in an append-only error
  list that `awaitTasks()` rethrows even after the DMG, the ZIP and both
  blockmaps have been written. Packaging now seeds the checksum-validated
  toolset cache before every build on every platform — every prefetch failure
  degrades to a warning, so it can never be the reason a build fails — and the
  retry wrapper treats the stall as transient while refusing any attempt the
  job clock cannot finish, so the backstop cannot manufacture a `cancelled` run
  of its own. `FORMAL_AI_PREFETCH_VERBOSE=1` (off by default) reports each
  toolset's cache decision and every download attempt.
- Record the reviewed npm install scripts of the `desktop` and `vscode` projects
  by package name in `allowScripts`. npm 11 warns about install scripts that are
  not recorded and documents that a future release will block them, which would
  have failed every desktop and `.vsix` build on the next runner-image bump and,
  later, silently stopped `node-pty`, `keytar` and `esbuild` building their
  native halves. An unreviewed install script still fails the install, but the
  report now names each one and the exact `npm approve-scripts
  --no-allow-scripts-pin` command that clears it.
- Stop a push to the base branch from failing macOS core slices that have
  nothing wrong with them. The archive job and each of the sixteen slices ran
  the fresh-merge simulation separately, and each resolved the base branch tip
  *at its own start time*; because the runner pool serializes the slices across
  roughly forty minutes, one commit landing on `main` mid-run gave the archive
  one merged tree and the later slices another, so every slice that started
  after the push failed its archive tree check. The archive now records the base
  commit it merged and every slice merges that same commit, which is the
  property the tree check was always asserting.
- Stop the desktop release from shipping installers built from different source
  trees. `release.yml` and `desktop-release.yml` also merge the base branch in
  more than one job, but neither compares trees across jobs, so the same
  divergence the macOS lane reports was silent there: in one run the `linux-x64`
  and `macos-arm64` installers were built against one base commit and
  `windows-arm64`, starting an hour later, against another, and all six were
  published as one release set. A reusable `pin-base-commit.yml` now resolves the
  base branch tip once per workflow and every merge — the six packaging legs, the
  `.vsix` job, `lint`, `test`, and the macOS archive and its sixteen slices
  through a new `base-commit` input — merges that one commit.
- Stop the language test coverage gate from demanding evidence in five
  languages for a change that cannot regress any of them. Any edit under
  `src/solver_handlers/` counted as language-facing, so rewording one
  English-only diagnostic string — in a handler whose meanings live in seed data
  and which has no localized counterpart — blocked the pull request. Changes
  under the language-independent code prefixes are now judged per changed line,
  while seed and translation data stay file-level and a line naming a locale or
  carrying non-Latin script still counts.

### Added

- Support Scala and Kotlin in the coding catalog, with the full task-template
  set, multilingual language aliases, and a browser-worker mirror. Both are
  marked as having no verified execution profile, because no `scalac` or
  `kotlinc` compiled them.

### Fixed

- Read the referenced work item before concluding a repository task cannot be
  executed, so `planned_not_executed` is reserved for a genuinely unavailable
  capability rather than being every repository run's terminal state (#904).
- Route the objective a caller states after an explicit delimiter instead of the
  unmarked harness preamble before it, and stop a caller policy sentence that
  merely mentions a privileged command from selecting it (#907).

### Fixed

- Fix the red `CI/CD Pipeline` on `main` in pull request #1019: give the three
  research E2E harness `run_issue_781.sh` the MCP `tool_call_timeout` the other
  harnesses already carry, plus `mcp_defaults` for the Agent CLI only --
  OpenCode reads the same file and its schema rejects that key. Without them the Agent CLI computes its per-tool
  deadline as `NaN`, so a call the local mock answers in milliseconds aborts
  with `timed out after NaN seconds`; the issue #781 turn then ended after one
  fetch and tripped its own `[ "$fetches" -ge 3 ]` assertion. A unit test pins
  both values, because `experiments/` is excluded from change detection and a
  fix confined to it gates no test job.
- Guard every unchecked `cd` in the Agent CLI E2E harnesses (`capture_all.sh`,
  `run_agent_cli.sh`, `run_issue_687.sh`, `run_issue_758.sh`,
  `run_issue_771.sh`, `run_issue_907.sh`). An unguarded `cd` in a script
  without `set -e` runs everything after it in the wrong directory, so a
  missing workspace surfaces as a confusing assertion failure elsewhere -- or
  as a pass against the wrong tree. `experiments/agent_cli_e2e/` is now
  shellcheck-clean.

### Fixed

- Answer the reported behaviour range of issue #1021 — a bare `ls`, `Execute ls command`, `List me files here`, a copy-stdin-to-stdout request, a Rosetta Code URL, a filesystem move, and a Laravel request in Russian — by fixing the rule that was wrong rather than the prompt that exposed it.

### Added

- Catalogue PHP: the eleven verified task templates every other catalogued language carries, so a PHP or Laravel request is answered with code and its result instead of the uncatalogued-language fallback.

### Added

- Compose the process artifacts a contribution carries — a changelog fragment and a pull-request body that closes its issue — and put the commands that publish them on a mutating-action ladder that is refused by default.

### Fixed

- Pin the third-party agent CLIs the end-to-end job installs. `@openai/codex@0.148.0` shipped overnight and drops the ENTER that answers its first-run trust dialog ([openai/codex#39487](https://github.com/openai/codex/issues/39487)), turning the Codex terminal leg red before any request reached the server under test; a test now holds the pinning rule `experiments/agentic_cli_matrix/clients.lock` already stated, for every CLI the project does not publish itself.
- Commit the case-study evidence `.gitignore` was silently dropping. Git never descends into an excluded directory, so the `!docs/case-studies/**/*.log` re-include could not rescue files under a `logs/` directory: `git add` reported success and committed nothing, and a test asserting the cited probe output exists passed locally while failing in CI. The directories are re-included beside the files.

### Fixed

- Enter the contribution write path's opt-in through one locked helper in its tests. The opt-in is a process-wide environment variable and the test harness runs tests as threads, so the assertion that publishing is refused by default could read the value a sibling test had set for its own opted-in case and report a permitted publication. Measured before and after with `experiments/issue_1021_opt_in_race/run.sh`: 33 failures in 200 rounds, then 0.

### Fixed

- Survive the transient package-mirror stalls that fail the agentic CLI matrix. Issue #1017 gave the matrix's Xvfb install a 300s budget so a hung mirror would report `failure` instead of a benign-looking `cancelled`; in run 32272689026 that deadline fired for real and turned a green pipeline red, while the sibling GUI legs of the same run installed the same package in 52s. `scripts/apt-install-with-retry.sh` now bounds each *attempt* as well: a stalled attempt is killed while the budget still has room for another, the wrapper refuses to start when its attempts cannot fit the budget above it, and a test checks that arithmetic for every budgeted retry a workflow composes.

### Fixed

- Ask the kernel, not the filesystem, whether an agent's timeout terminated a descendant process. `timeout_terminates_descendant_processes` inferred termination from the absence of a file its descendant writes after a delay, which is also what an alive-but-sleeping descendant looks like, so a loaded macOS runner failed it (run 32272689475, job 96137354605) on a branch that touches neither `run_agent` nor its fixture. The fixture now records the descendant's pid, the test polls its process state, and the three ways this can go wrong -- never spawned, still running, terminated only after outliving its own delay -- are reported as three different failures instead of one ambiguous one.

### Fixed

- Bound a CI command with a deadline both runner families have. The apt retry
  wrapper used GNU `timeout`, which macOS does not ship, so the macOS core
  slices failed the tests that drive it while its own Linux job was green;
  `scripts/run-with-deadline.sh` keeps `timeout`'s 124-on-expiry contract on
  every runner, and a new gate holds the rule for every tracked script and
  workflow. The replacement is held to the promise it replaces: it never expires
  a deadline early, which measurement — not assertion — is what caught.

### Added

- Answer the rest of the reported coding range: a catalogued copy-stdin-to-stdout task in every catalogued language, a framework named beside its language answered in that framework, and a request that names code and no language read as a coding request.

### Added

- Added versioned recoverable memory (#946): a candidate version is written
  against a byte-for-byte snapshot and a digest-pinned baseline, and a version
  that fails to compile, fails a baseline specification, or edits the baseline
  it is judged against is rolled back to the last one that passed.
- Added bounded autonomy with a stuck-recovery limit (#947): the recovery loop
  reads an injected clock, stops after its limit -- one hour by default -- and
  asks with the plan it accumulated, and keeps per-command permission and full
  trust as separate opt-ins so delegating commands is not delegating choices.

### Added

- Added verified mutating filesystem actions (#824, #944): a request to move or
  copy a file is carried out as the ordered recipe its seed intent declares --
  the source exists, the destination is free, the destination's parent is
  created, the action runs, and the result is checked -- with each step observed
  before the next is planned, so a deep target path works and a destination that
  is already taken stops the recipe before anything changes.
- Added the mutating rungs `824.L1`-`824.L5` to the issue #916 write-effect
  ladder together with the sandbox-reset semantics #944 asks for: every rung
  declares the filesystem state it starts from, that state is materialized and
  read back off disk before the rung runs, and a rung may now require the steps
  that carried its action out and not only the effect they left behind.

### Changed

- A blocked action reports the check that stopped it and the status it exited
  with instead of claiming a completion the workspace would contradict.

### Changed

- Test-suite environment overrides (`FORMAL_AI_MEMORY_PATH`, `HOME`,
  `FORMAL_AI_DIALOG_LOG_DIR`, the write-path opt-in, and the rest) are now
  scoped to the closure that needs them, through `temp-env`, instead of being
  assigned to the process and put back by hand.

### Fixed

- A test that failed while it held an environment override no longer leaks that
  override into the rest of its binary: the previous value is restored on
  unwind, not only on the success path that ran the restore statements.

### Changed

- The crate is built on Rust edition 2024. The already-declared
  `rust-version = "1.96"` covers it, and the edition needs no nightly
  toolchain: every `dtolnay/rust-toolchain@stable` invocation in CI stays
  exactly as it was.
- Let-chains, which edition 2024 makes available on stable, replace the nested
  `if let` ladders they were standing in for throughout `src/`, `tests/`,
  `examples/`, `build.rs`, and `scripts/`. Clippy's `collapsible_if` asks for
  this the moment the edition moves, so the change is the lint's, not a
  matter of taste.
- The tree is formatted by the 2024 style edition.
- Every `scripts/*.rs` `rust-script` file now declares `edition = "2024"` in
  its embedded manifest, so the scripts read the same dialect as the crate
  that ships beside them. `scripts/rust-paths.rs` gained the `regex`
  dependency it had only ever borrowed from the scripts that include it, which
  makes it runnable on its own for the first time.
- Every `rustc` this repository *spawns* now names edition 2024 too. Three of
  those compile Rust the system wrote: `memory_revision::rustc_verdict` builds
  the crate's own next version, the issue-#847 ladder runner builds the sources
  a solve authored, and the substitution compiler's export commands build the
  program it just emitted. Left at 2021 they would answer "does not compile" to
  a let-chain and roll back, or fail, work that `cargo build` accepts.
- `rustc_verdict` reads that edition from `Cargo.toml` rather than from a
  constant, via a new `FORMAL_AI_CRATE_EDITION` that `build.rs` exports, so the
  manifest stays the only place the crate's edition is written down.
- The Rust→WASM worker is built at edition 2024. It `#[path]`-includes the
  crate's own modules, so once those used let-chains `sh
  src/web/wasm-worker/build.sh` stopped compiling; the shipped
  `src/web/formal_ai_worker.wasm` is rebuilt from it. Its `#[no_mangle]`
  exports -- and those of the WebAssembly programs the substitution compiler
  writes -- are spelled `#[unsafe(no_mangle)]`, which edition 2024 requires and
  every edition has accepted since Rust 1.82.

### Changed

- Every direct Rust dependency is on its newest release that builds on stable.
  Six needed more than a version number: `lino-objects-codec` 0.2.1 → 0.4.1
  (library and dev-dependency both), `links-notation` 0.13.0 → 0.14.0,
  `meta-language` 0.54.0 → 0.58.2, `sha2` 0.10 → 0.11, `which` 7 → 8, and
  `web-capture` 0.3.36 → 0.3.37. `command-stream` stays pinned at `=0.16.0`,
  which issue #1014 pinned deliberately and `tests/unit/ci-cd/issue_1014.rs`
  asserts.
- `sha2` 0.11 returns its digest as a `hybrid_array::Array` rather than a
  `GenericArray`, and that type does not implement `LowerHex` -- so the nine
  places that rendered a digest with `format!("{:x}", ..)` all stopped
  compiling at once. They now go through `source_fetch::sha256_hex`, and the
  encoding itself is written once, in `source_fetch::hex_lower`. Adopting the
  new major rather than pinning back to the old one is what this costs, and it
  leaves one implementation of "digest bytes as text" where there were nine.
- `browser-commander`, the browser runtime both the desktop app and the VS Code
  extension override inside `@link-assistant/web-capture`, goes 0.15.0 → 0.16.1.
  The 0.16 line adds a native `better-sqlite3` addon and twenty-five more
  transitive packages, which grows the bundled `web-tools.cjs` the VSIX ships
  from 9.3 MB to 11.8 MB. It is taken rather than held: the addon backs
  browser-commander's cookie database, `web-capture`'s `src/browser.js` never
  reaches it, the esbuild bundle the VSIX is built from still builds, and both
  lockfiles audit clean.

### Added

- Stable Rust is now a gate rather than a convention.
  `nothing_in_the_tree_reaches_for_a_nightly_toolchain` refuses a toolchain
  file, a toolchain action asking for anything but stable, a per-invocation
  toolchain override, a bootstrap environment variable, and an unstable feature
  attribute -- across every tracked source, script and workflow, so a future
  dependency that only builds on nightly is caught at review time instead of
  quietly moving the toolchain.
  `the_crate_is_on_edition_2024_and_the_judge_compiles_the_same_edition` holds
  the manifest and the `rustc` that judges a self-authored version to the same
  edition.

### Fixed

- The issue-#1021 traceability gate counted its requirements with a literal
  `(1..=31)` and went stale the moment R1021-32 was written: it kept passing
  while checking one fewer requirement than the branch had. The IDs are now read
  from the shard that assigns them and asserted to run contiguously from 1, so a
  gap, a duplicate, or a row nobody wired up is a failure rather than a shorter
  loop.

### Fixed

- One timed-out link no longer makes the Broken Link Checker report every
  healthy redirect in the repository as broken. `extractBrokenUrls` narrowed its
  permissive bullet parser to the failure section by searching for a single
  hard-coded `## Errors per input` heading; lychee writes only the sections a
  run actually has links for, so a report whose sole failure was a timeout
  matched nothing and fell through to parsing the whole document -- including
  `## Redirects per input`. Every `## ... per input` section is now sliced out by
  heading and counts as failing unless it is one of the outcomes known to be
  healthy, so a category this parser has not heard of is reported rather than
  silently dropped. Four new tests cover the report shapes the old lookup got
  wrong; all four fail against the previous parser, and the real report from run
  32454084765 is kept as a fixture in
  `experiments/issue-1021-link-checker-false-positive/`.

### Fixed

- The server answers Anthropic's `/api/hello` reachability probe, so a Claude
  Code session no longer opens with a `404`. `@anthropic-ai/claude-code`
  2.1.238 added `HEAD <base-url>/api/hello` to the `HEAD <base-url>` probe it
  already made, which against the `/api/anthropic` base URL our wrapper writes
  arrives as `/api/anthropic/api/hello`. The doubled `/api` belongs to neither
  side: `https://api.anthropic.com/api/hello` is Anthropic's own endpoint and
  answers `200 {"message": "hello"}`, so an Anthropic-compatible surface answers
  it too -- `GET` with that payload, `HEAD` with an empty body.

### Changed

- `t3code`'s recorded launch contract in `data/seed/client-integrations.lino`
  now lists the `pair` and `service` subcommands that t3 0.0.33 added. The
  matrix leg asserts t3's subcommand list verbatim so that a new *prompt* path
  makes the leg fail instead of silently going unexercised; both additions were
  read from the shipped CLI and neither is one -- `pair` mints a pairing token
  and prints it as a QR code, and `service` installs, updates or reports on the
  same server as a background service.

### Changed

- A step terminated by `scripts/run-with-budget-warning.sh` now reports the
  compiler cache counters alongside the seconds it spent. A budgeted Rust step
  that runs long has two causes with the same shape in the log -- work that
  grew, and a compiler cache that stopped answering -- because cargo prints
  ``Running `sccache rustc ...` `` on a cache hit exactly as it does on a miss.
  The counters are asked for at the 70% warning and at the termination, and
  only when `RUSTC_WRAPPER` names sccache, so a budgeted step that compiles
  nothing is exactly as quiet as it was before.

### Changed

- The deterministic sampling seed in `translation::selection` is called a seed,
  which is what it is. Its parameter was named `salt`, and CodeQL's
  `rust/hard-coded-cryptographic-value` treats *any* argument reaching a
  parameter literally named `salt` as a cryptographic salt: every configuration
  literal that flowed into `sample_index` — `0.0`, `1.0`, `0.7`, the
  `SolverConfig` defaults — was reported as a hard-coded salt, 98 critical
  alerts across 24 files. Nothing on that path is cryptography; `fnv1a64` is a
  non-cryptographic hash and the seed only makes a draw reproducible.
- `PromotionApplyOutcome::agent_session_ids` is now
  `PromotionApplyOutcome::agent_session_digests`. The values were already
  content-addressed FNV-1a digests of the recorded session JSON, as the field's
  own documentation said, and are committed as evidence under
  `docs/case-studies/`; the `session_id` spelling made CodeQL's
  `rust/cleartext-logging` heuristic read `formal-ai improve`'s evidence line as
  a session token written to a log.
  The field is `pub` on a re-exported type, so this is a breaking rename and the
  bump is `minor` rather than `patch` -- on a 0.x crate that is where an
  incompatible change goes.
- `tests/unit/ci-cd/codeql_sink_heuristics.rs` now holds both heuristics over
  every Rust file the CodeQL configuration analyses, so a name that a static
  analyser will read as a credential fails here first, at the site that
  introduces it, rather than as a critical alert on a pull request.

### Fixed

- Enforce the issue #534 disk policy across every workflow instead of three
  hand-listed files. `agentic-cli-matrix.yml` and `external-benchmarks.yml` were
  caching the multi-GiB `target/` tree — exactly what the policy forbids — because
  the guard never read them. Both stop, and the guard now sweeps
  `.github/workflows` so a new workflow cannot reintroduce it.

### Fixed

- Start the sccache server explicitly and report its counters between steps.
  `Test (ubuntu-latest / full)` compiled 514 crates while sccache reported one
  compile request, zero misses and zero write errors — a wrapper that is never
  asked cannot miss, so the counter described neither a cold cache nor a broken
  one. The server now starts before any cargo step, and the counters are read
  straight after the step that compiles rather than only post-job.

### Fixed

- Build the Docker image's dependencies from the manifests alone, before the
  sources are copied. `COPY . .` preceded `cargo build`, so every file in the
  tree was part of the build layer's cache key and editing one `.rs` rebuilt all
  ~500 dependency crates — a 24-minute image build that gated the pipeline's
  finish by itself. Measured locally: the manifest-only layer builds in 1m48s,
  and the source layer then compiles `formal-ai` alone.

### Fixed

- Accept link-checker timeouts as host-side, the way 429 and 5xx already are.
  `eur-lex.europa.eu` stopped answering on 2026-08-21 — 45 seconds with no
  response — and reddened every open pull request over three
  `LEGAL-COMPLIANCE.md` citations none of them touches. A timeout carries no
  status code, so no accept range could match it. 404, 403 and 410 still fail.

### Changed

- Stop running the specification shard twice per pipeline. The `full` test lane
  skipped only `data_files::` and `self_ast_census`, so it also ran the 1034
  `specification::` tests that the parallel `specification` lane was running at
  the same moment — a lane that needs 689 seconds on its own to do exactly
  that. Measured on run 32555911181: the `full` lane held 700.17 seconds in a
  single test binary, 87% of the job that set the pipeline's critical path.
  Compilation was not the cost — sccache reported a 79.57% hit rate with zero
  errors in that same job, and the test step logged no `Compiling` line at all.
  The lane that owns those tests still runs them, and a test pins that so the
  skip cannot quietly become a coverage hole.

- Sweep the build cache on every commit, not only Rust ones. Cargo never
  removes anything, so `target/` accumulates artifacts from every branch and
  dependency version until the disk fills. A docs-only commit used to leave the
  previous build's artifacts behind just the same.

- Prune with cargo-sweep when it is installed. It asks cargo which artifacts the
  current build actually references, so a dependency the next build still needs
  survives even when it was compiled weeks ago; the previous mtime comparison
  could not tell a stale artifact from a current one that simply did not need
  rebuilding, and deleted live dependencies the next build then recompiled. The
  mtime path remains as a fallback. `CARGO_TARGET_MAX_SIZE_MB` caps the tree
  locally (4GB by default) and is unset on CI, where the runner is billed for
  the rebuild rather than the disk.

- Run the `cargo-test` commit hook through `scripts/cargo-test.sh`. A bare
  `cargo test` starts one compile job and one test thread per core, pinning the
  whole machine for the length of a commit, and prunes nothing afterwards.

### Fixed

- Retry the macOS test-archive download instead of reddening the pipeline on a
  transient storage failure. Run 32555911181 failed `main` with `Artifact
  download failed after 5 retries` on one of sixteen slices; the other fifteen
  downloaded the same artifact from the same run and passed, no test ran, and
  the blob URL named GitHub's own storage backend. `actions/download-artifact`
  spends its five internal retries back-to-back, so a backend having a bad
  minute exhausts them; the wrapper pauses between attempts so a later one meets
  a different minute. Exhausting the attempts still fails the step, with an
  `::error` annotation naming the cause.

- Let a single macOS slice be reran on its own. The archive is uploaded as
  `macos-core-tests-<run_id>-<run_attempt>`, but `gh run rerun --failed` puts
  the reran slice on attempt 2 while the archive job — which succeeded, so it is
  not rerun — left its artifact named `...-1`. The slice looked for a name that
  does not exist, so every partial rerun of a macOS slice failed with "artifact
  not found" and forced a full rerun of the whole pipeline. The download now
  resolves the artifact by name prefix.

- Raise the macOS slice job cap from 15 to 18 minutes so the retry fits beneath
  it. A retry that cannot finish inside its cap converts a transient failure
  into a *terminated* step, which GitHub reports as `cancelled` rather than
  `failure` — the issue #977 and #1017 failure this must not reintroduce. Worst
  case is now 140s download + 600s slice budget + 133s setup + 15s grace = 888s
  against a 1080s cap, and a test pins that arithmetic.

### Fixed

- Install the build-cache sweep from `build.rs` instead of waiting for someone
  to install the `pre-commit` framework. The hook has been described in
  `.pre-commit-config.yaml` since the previous release and never ran: that
  config takes effect only after `pre-commit` is installed *and*
  `pre-commit install` has been run, and on a fresh clone neither is true. The
  machine it was written for reached 205MiB free of 460GiB with the config
  committed and inert. `build.rs` now points `core.hooksPath` at a tracked
  `.githooks/`, so an ordinary `cargo build` arms it — the one step every
  contributor takes without being told. Installation is best-effort and skipped
  on CI: a tarball with no `.git`, a sandbox with no `git`, or a read-only
  checkout must all still build, and an existing `core.hooksPath` is never
  overwritten.

### Changed

- Build the E2E harness binary without full LTO. `Build formal-ai (release)`
  took 536 seconds and compiled 510 crates from scratch, twice per pipeline, to
  produce a binary the agent-CLI harnesses only *run* — it ships nowhere, so the
  runtime speed LTO buys is measured by nothing in those jobs. Full LTO also
  defeats the compilation cache those jobs restore, since it defers optimisation
  into a single link-time unit that sccache has little to reuse. Measured
  locally on the same one-line source change: 198s with LTO against 42s without.
  The override goes through the environment rather than a named profile so the
  output stays at `target/release/formal-ai`, which seventy harness scripts
  hardcode. Everything that ships still builds with the unmodified `--release`
  profile, and a test pins that boundary.

### Fixed

- Retry past a connection reset instead of reporting it as a broken link. Run
  32586546161 failed `main` on four links with `Network error: Connection reset
  by peer (os error 104)`; all three distinct hosts answer 200 from a
  workstation. A reset happens below HTTP and carries no status code, so neither
  the accept list nor `--cache-exclude-status` could name it — and one of the
  four came back as `Error (cached)`, the same reset replayed from cache.
  Retries go from three to six with a growing wait, so a later attempt meets a
  different moment. 404, 403 and 410 still fail the build.

- Stop link-checking a recorded lychee report. The fixture under
  `experiments/issue-1021-link-checker-false-positive/` is captured evidence of
  a past failure whose URLs are *supposed* to be broken, so checking them could
  only ever produce a false positive.

### Changed

- Schedule the longest tests first when splitting work across parallel runners.
  `cargo nextest --partition slice:N/D` splits by test *index*, which is
  uncorrelated with duration: measured across run 32591020809 (2895 tests, 4704
  seconds of work over eight partitions), index order gave an 870-second worst
  partition against a 588-second ideal, so the critical path waited on one
  machine while the other seven idled. The same pattern appeared inside a
  partition — the quarter of tests finishing last averaged 4.31 seconds against
  0.70 for the quarter finishing first. `scripts/plan-test-partition.rs` now
  assigns each recorded test longest-first onto the emptiest partition, reaching
  588 seconds with a 0.2% spread, and the `check_test_partition_balance` gate
  fails if the plan drifts back out of balance. Tests with no recorded duration
  still go through nextest's own index split, so a new test runs exactly once
  without a re-recording.

- Record the rule in `CONTRIBUTING.md` for every future fan-out: start the
  longest work first, and do not throttle CI parallelism. A long task started
  last runs alone on a machine everything else has already finished waiting for.

### Fixed

- Stop linking 116 example binaries on every Rust commit. The pre-commit hook
  ran `cargo clippy --all-targets`, which links every example into a ~190MB
  binary — and cargo keeps both a hashed and an unhashed copy, so a single run
  left about 27GB in `target/debug/examples`. The `run_clippy` CI gate already
  used the cheaper split, `cargo clippy --lib --bins --tests` plus
  `cargo check --examples`, which type-checks examples without linking them; the
  hook now matches it, and a test keeps the two from drifting apart.

- Remove linked example binaries when pruning. `cargo sweep` reasons about what
  the current build references, and those binaries *are* current — so neither
  `--installed` nor `--maxsize` touched them, and the ceiling added for issue
  #1037 reported `applied 4096MB ceiling` over a 28GB tree. Measured on this
  repository: 28GB before, 672MB after.

### Changed

- Compile the release binary once per pipeline instead of four times. Measured
  on run 32598625222, a single pull request ran `cargo build --release --bin
  formal-ai` in two jobs with byte-identical commands (10.8 and 4.2 minutes),
  built the project again inside Docker (33.0 minutes), and once more for the
  package (5.7). The E2E, box-language and Docker jobs now download the one
  artifact the build job uploads.

- Stop recompiling the project inside the Docker image on pull requests. That
  build could not use sccache — `RUSTC_WRAPPER` and the Actions cache token live
  on the runner, and BuildKit cannot reach them — so it compiled 510 crates cold
  for 33 of the pull request's 42.8 minutes, gating the whole run. The image now
  takes `--build-arg BINARY_SOURCE=prebuilt` and copies the binary the pipeline
  already built and tested. Published images keep the default `compile`, so what
  ships is still built from source and the Dockerfile stays provably able to
  build the project unaided.

### Changed

- Compile the test suite with optimization. `[profile.test]` never set
  `opt-level`, so it inherited Cargo's default of 0 — a good default for
  projects whose tests are I/O-bound, and the wrong one here: the seven tests
  over 60 seconds spawn no subprocesses at all and are pure in-process
  computation. Measured over the same 1945 tests, the unit suite runs in 28.25
  seconds at `opt-level = 2` against 104.64 unoptimized, a 3.8× difference, for
  about 40 seconds of extra compilation per job. On the macOS lane that cost is
  paid once in the archive job while all eight slices run the faster binaries.
  `debug-assertions` and `overflow-checks` are now stated explicitly so the
  speedup cannot quietly turn them off: `debug_assert!` appears throughout
  `src/`, and an arithmetic overflow must keep panicking rather than wrapping.

### Changed

- Turn off link-time optimization. LTO is the one stage of a Rust build that
  does not parallelize: it merges every crate into a single optimization unit
  and links it on one thread, so unlike compilation it does not shrink when the
  runner has more cores. Measured with `cargo test --release --no-run --bins
  --tests` from a touched `lib.rs`: 867 seconds with `lto = true` and
  `codegen-units = 1`, against 162 without — 705 seconds sitting on the critical
  path of every downstream job. `codegen-units` returns to its default so the
  compiler uses every core available.

- Compile once per platform and reuse the result. One job now runs `cargo test
  --release --no-run --bins --tests`, producing the binary and all three test
  executables together; the test lane, Docker check, agent-CLI E2E and packaging
  all download them instead of compiling again. Packaging no longer runs
  `cargo build --release --verbose` at all — `cargo package` needs the manifest
  and sources, not a fresh compile.

- Order the pipeline so nothing waits without reason: `lint` and `secrets-scan`
  compile nothing and start immediately, tests begin as soon as the build
  artifacts exist, and packaging and release run last behind every check.

### Fixed

- Stop Docker layers evicting the compiler cache. The GitHub Actions cache is
  one 10GB pool shared by every workflow, and it reached 10.01GB — buildkit
  blobs holding 5.26GB against sccache's 2.44GB. GitHub then evicted
  compilation entries to make room for layers: the macOS specification lane's
  Rust hit rate fell from 48% to 27% between consecutive runs and the lane was
  killed at its 1400-second budget with no test having started, because those
  budgets were sized for a cache that hits. Two writers were paying for layers
  nobody reads — the pull-request image check, which since the previous release
  copies a prebuilt binary and so compiles nothing worth keeping, and the Docker
  Hub publish steps, which export the same layers the GHCR step in the same job
  just exported. Both now read the cache without writing to it; the from-source
  publish still exports, so a release does not rebuild every layer.

### Changed

- Run only platform-sensitive tests on macOS. The lane ran the same 2895 tests
  as Linux against the same `cfg(unix)` code — no conditional in `src/`
  distinguishes the two platforms, so that logic cannot behave differently
  there. Every macOS-only failure this repository has recorded came from the
  environment instead: `timeout` absent, bash 3.2 without `mapfile`, subprocess
  and path handling. The cost was real: each of eight slices downloaded a 916 MB
  archive, 7 GB per run, and two of those downloads failed outright, taking
  `main` red for a reason no commit caused. The lane now runs the 139 tests
  named in `data/meta/macos-platform-tests.lino` — about ten seconds on one
  runner. When something does behave differently on macOS, add its module to
  that file rather than widening the filter back to everything; CONTRIBUTING.md
  states the rule and a test fails if the list empties out.

### Fixed

- Bound how long an automatic release may stay deferred. A policy-ineligible
  cycle still defers rather than turning every push on `main` red, but past
  seven days or twenty pending changelog fragments the same verdict now fails
  the release preflight instead of reporting success. The unbounded deferral
  held 268 commits and 45 fragments behind a green pipeline for 14 days,
  including the fix a downstream consumer was blocked on (#1064).

### Fixed

- Bring the issue #1028 Agent CLI ladder workflow back under the two policies it
  broke when it landed: it now belongs to a concurrency group, so a superseded
  push releases its runner instead of running the ladder twice (#1017), and it
  no longer caches the `target` tree (#534). Both were pinned by tests that
  failed on the commit that introduced the workflow, but the follow-up push
  touched only a shell script, so path filtering skipped the lane that would
  have caught them.
- Commit `experiments/issue_1028_agent_cli_ladder/run.sh` executable. It shipped
  as mode `100644` while every other script a workflow invokes bare is `100755`,
  so a checkout handed CI a non-executable file and the ladder's only step died
  with `Permission denied` on every run the workflow ever had. A new sweep over
  every workflow pins the executable bit for all thirty such scripts.

### Added

- Make task decomposition a recursive binary tree rather than a flat list. The
  contract now carries the `binary` rule — every non-leaf task splits into
  exactly two children — and the Agent-CLI ladder generates the canonical
  63-node depth-five tree at runtime from its 32 atomic leaves, so the structure
  itself is executable and testable. The workflow is manual-only
  (`workflow_dispatch`) with depth and single-node inputs, in its own
  concurrency group so a manual run never disturbs PR CI (#1028).

### Fixed

- Regenerate the self-AST census for `src/task_decomposition/strategy.rs`, which
  the new contract field moved.

### Changed

- Remove the release deferral budget entirely. Work in this repository is not
  deferred however hard it is, so an ineligible release cycle is now reported as
  blocked from the first push rather than after a seven-day, twenty-fragment
  grace period. `SelfDevelopmentReleaseStatus` has two states — eligible or
  blocked — and the preflight fails on the second instead of writing a notice
  and reporting success (#1066).

Fixed the incremental self-authoring harnesses reporting a failed run for a
dispatch that had solved its task. `experiments/issue_924_self_authoring/run.sh`
and `experiments/issue_933_self_authoring/run.sh` read the UTF-8 dispatch report
with Ruby's `File.read`, which decodes with the locale's default external
encoding, so on a host whose locale is `POSIX`/`C` the first non-ASCII byte
raised `Encoding::InvalidByteSequenceError`. Both harnesses now name the
report's encoding.

### Fixed

- Repaired the #1028 recursive agent ladder so it can run at all: the tree
  generator raised `TypeError` before selecting a single node, leaves claimed
  children they do not have, `run.log` rows were written with literal
  backslash-t instead of tabs, and the per-node instructions reached the agent
  as one line with literal backslash-n in the middle. Covered by
  `tests/unit/issue_1066_agent_ladder.rs` and reproduced end to end by
  `experiments/issue_1066_self_development/reproduce-ladder-tree-generation.sh`
  for #1066.

### Added

- Compose the document a request describes instead of transcribing its
  description. "Produce a final evidence note containing the selected tree
  level, node outcomes, test results, and session id" names the headings a
  finished note must have, not the bytes of a file; a new planner route reads
  the three seed-declared signals that say so (a composition verb, the noun for
  a document whose content is described rather than supplied, and a content lead
  followed by two or more enumerated parts) and returns the composed note. The
  note reports what was asked for and what the session actually observed, and
  says plainly when nothing backs a requested part (#1066).
- Answer a question about the repository by reading the repository. A request
  that says *inspect*, *examine*, *review* or *identify* without saying *search*
  now admits the workspace-search route, provided it names a code-shaped subject
  and no external source — so "check the current exchange rate" still reaches
  the open web (#1066).

### Fixed

- Never write a file that breaks a constraint the same request states about it.
  A literal write is only literal when it satisfies every stated constraint on
  the file it writes, so content recovered from prose is no longer written when
  the request also pins the file's opening line. All three literal-write routes
  share the guard, so the misroute cannot simply move (#1066).
- Stop reading a dotted run of digits as a file name. `1.1.1.1.1`, `2.7.19` and
  `192.168.0.14` all split on their last dot into a file-shaped stem and
  extension; a ladder node addressed by its path in the tree was opened as a
  file, failed with "File not found", and the run ended on a fabricated answer
  (#1066).
- Strip the sentence's full stop from a path token, so "Read the file
  `Cargo.toml`." is a read rather than a web search (#1066).

### Added

- Answer a question about a task's structure by thinking about the task. Nothing
  on the open web knows the caller's own work, so "Break the customer import
  rewrite into sub-tasks" no longer becomes a search for its own words: a new
  planner route puts the question to Formal AI's recursive decomposition and
  returns what it finds (#1066).
- Deliver an answer only the symbolic engine reaches. A request that asks for
  something to be found out *and* recorded at a named path used to deliver only
  what the agentic router produced, so every residual that needs no tool ended
  with nothing written (#1066).
- Read an English phrasal verb with its object in the middle. "Break the
  customer import rewrite into sub-tasks" and "break into sub-tasks" are the
  same verb, but only the contiguous form was in the lexicon — and it is the
  form a caller is least likely to write, because English puts a long object in
  the middle (#1066).
- Say why a decomposition produced nothing, in the caller's language. A task
  that is atomic, that states a single need, or that hit the depth bound now
  reports that reason instead of announcing a list and enumerating none (#1066).

### Fixed

- Never open for reading a file the same request asked to be written. The named
  path was the only file-shaped token in an evidence-record request, so it was
  read, the read failed, and the evidence file was never written (#1066).
- Never write the words that *name* a work product as its body. "Record the
  findings in `report.md`" states where the findings go, not that the file
  should contain the word "findings" (#1066).
- Never read a literal payload across a sentence boundary. A payload marker in
  one sentence and the file clause in another recovered everything in between,
  so a handover memo opened by instructing the reader to leave it somewhere
  (#1066).
- Never deliver a description of a pending web search as a finding, and never
  read an authoring sentence as a delivery destination — either one wrote prose
  about the wrong subject into the caller's file (#1066).
- Keep the text of a sub-task that ends in a question mark. The question-shape
  enforcement rewrote such a task to its bare marker, so a listed sub-task said
  nothing about what to do (#1066).
- Never throw the work away with the sentence that delivers it. "Break the
  customer import rewrite into sub-tasks and record what you work out in
  `import-split.md`" coordinates the work and its delivery into one sentence,
  and consuming the whole sentence as delivery left nothing to answer, so the
  request was answered in the transcript and the named file never appeared
  (#1066).
- Read a task from the colon that introduces it, not from the last colon in the
  prompt. "Break the warehouse restocking rewrite into sub-tasks. Deadline: the
  end of the quarter." made the deadline the task, and a deadline is an
  irreducible single need, so a rewrite that splits four ways was reported as
  unsplittable. The colon now counts only in the sentence that asks the
  question, which is the same sentence scoping already used to tell a command
  that is named from one that is ordered (#1066).
- Read a task from the block that asks it, not from the instructions addressed
  to the solver. A prompt that states its task, leaves a blank line, and then
  says how to work and where to leave evidence had that second block decomposed
  beside the task, so one listed sub-task was the framing sentences pasted
  together -- a numbered line a reader can do nothing with. The blocks that ask
  are the task, decided by the same recogniser that routed the prompt (#1066).
- Never let a calculation cue claim every word written after it. "Solve" is also
  ordinary English, and an embedded cue was read to the end of the prompt, so
  four unrelated sentences became one expression; they carried a digit and an
  `=`, so it looked evaluable, failed to evaluate, and answered anyway --
  displacing the decomposition the prompt actually asked for. A request is
  stated in a sentence, so its cue claims that sentence (#1066).

### Fixed

- Compose a document that a request specifies, instead of writing the
  specification into the file. "Produce a final evidence note containing the
  selected tree level, node outcomes, test results, and session id." names a
  document and its parts; "containing" is also the marker that introduces a
  literal payload, so the words after it were taken for the bytes and the
  request's own wording was written back as the answer. The sentence around the
  marker decides which reading applies, and the recogniser that decides is the
  one the composing route already uses (#1066).
- Answer the work a request states, not a label it carries. A request that
  reads "Atomic task 9: Assemble an intake summary containing the applicant
  name, the referral source, and the interview date." was answered "yes, that is
  atomic" -- true, and a reply to the heading rather than to the sentence after
  the colon, because the heading alone carries the atomicity predicate and the
  task noun. Naming something to produce states work to do, so the
  task-structure route stands aside for it (#1066).
- Stop a calculation cue at the end of its sentence whether a blank line follows
  or not. The bound was whichever the search found first, and it looked for the
  blank line first, so a cue in a paragraph with another paragraph after it
  still claimed every sentence up to the break (#1066).

### Fixed

- Report what a tool found instead of answering the same request a second time
  without it. The route that answers a question about how a task decomposes
  plans no tool call, so its answer is the same on every turn, and it sat ahead
  of the route that reports a tool result: a request to look at the repository
  was correctly planned as a search, and the turn that existed to report the
  search reported a decomposition of the instructions instead. An answer reached
  without looking has no standing to overrule one reached by looking, so the
  route now stands aside once a tool has run — the same rule the routes on
  either side of it already follow (#1066).

### Fixed

- Keep the whole payload when a written file's text contains a semicolon. The
  bound on a literal write was read with the sentence splitter written for shell
  routing, where `build; deploy` is two commands to judge one at a time. Prose
  does not read a semicolon that way, so a file whose text was
  "… or a host surface; domain knowledge and policy belong in data." was written
  ending at *host surface;*, with the clause saying where domain knowledge goes
  thrown away. The two readings are now two named splitters over one
  implementation, and the payload keeps its second half (#1066).

### Fixed

- Answer a Spanish-speaking client's decomposition questions in Spanish.
  `data/seed/multilingual-responses-decomposition.lino` carried English,
  Russian, Hindi and Chinese for all thirteen of its intents and Spanish for
  none, so every reply on the decomposition path — the sub-task list, the
  atomicity verdict, the first step, the depth-bound note, and the two honest
  refusals to enumerate — fell back out of the asked-in language. All thirteen
  now have their Spanish record, pinned per language by an end-to-end test
  (#1066).

### Fixed

- Read a request's subject from the block that states the work rather than from
  the note that places the worker. A prompt handed to a worker often ends with a
  paragraph saying where they are and how to report; scored as part of the
  request, the longest code-shaped word in that paragraph won, so twenty of the
  twenty-nine searches the #1066 ladder planned looked for `binary_tree` -- a
  word out of the framing -- instead of the data model, atomicity check or
  execution adapter the node was asked about.
- Stop reading a permission to use the web as a statement that the answer is on
  it. "Use web research when it materially improves factual accuracy" sits in
  that same paragraph, and matching it across the whole prompt disqualified
  every ladder node from looking at the repository it had just been handed: six
  proof files recorded an open-web query assembled out of the framing, ending in
  "the tool returned no content", as their evidence. The workspace admission and
  the external-source veto that guards it are now both read at the scope of one
  block (#1066).
- Judge a proof whose body reports that a tool returned no content as hollow.
  An empty tool result is a step that did not happen and proves as much as an
  empty file, so `experiments/issue_1066_ladder_offline/judge-proof.py` now
  refuses it. A search that ran and matched nothing is an observation about the
  workspace and still passes (#1066).

### Fixed

- Stop every former of an open-web query at the end of the request. Three
  functions still read a whole prompt when they picked what to search for --
  the stated research subject, an explicit search request, and the planner's
  last-resort route for a request nothing else understood. A prompt whose second
  paragraph only places the worker was therefore sent to a search engine with
  that paragraph attached: "a two node decomposition at depth one this is
  recursive binary tree node 1 2 1 2 1 at depth 5 solve only this node s task in
  this fresh temporary repository ...", a query no source answers. Each now
  reads the block that states the subject, the same scope the search and
  workspace routes already use (#1066).
- Never report a research round that was not run. When the last completed tool
  call belonged to another route -- a workspace search, a file read -- the
  research route took it for a round of its own and composed "Research completed
  for ..., but the tool returned no content." over the result the agent was
  already holding. Five of the #1066 ladder's thirty-two leaves recorded exactly
  that as their evidence, with the matching `grep` output unused in the same
  transcript. The route now plans a further round or stands aside, so the search
  that actually ran is what gets reported (#1066).

bump: patch

### Fixed

- A line a search *quoted* is no longer read as the search's own diagnosis. The
  failure lexicon is now asked only about a result's own words — the part before
  it starts naming the places it is quoting — so a `grep` that matched fifty
  lines no longer reports itself as the command that failed because one of the
  files it matched is an installer that prints *not found* when a program is
  missing. The cut is made wherever a line number stands as its own word before a
  colon, so it holds for a plain `<path>:<line>:<text>` listing and equally for a
  search that announces `Found 100 matches` and then quotes each hit under a
  path heading. A step that really did fail still says so: one citation is not a
  quotation, and a harness announcing its own refusal cites no place at all
  (#1066).

### Fixed

- Keep an explicitly named repository-search subject exact instead of widening
  it with surrounding prose, and store terse source observations through the
  multilingual response seed rather than hardcoded English (#1069).
- Keep ladder search facts canonical Links Notation and within the seed's
  single-value action model (#1069).
- Refresh stable Rust, Cargo, JavaScript, agentic-client, and GitHub Actions
  dependencies to their current compatible releases. The desktop and VS Code
  graphs hold `@kreuzberg/html-to-markdown-node` at 3.5.5 because every later
  stable release declares unpublished Linux musl packages; clean npm 11
  installs cover the complete 3.5.5 graph (#1069).
- Route native and server link storage through link-cli's persistent,
  recovery-logged transactions and atomically publish complete server
  projections without the superseded local doublets/platform-mem stack (#1069).
- Reclaim stopped containers, dangling images, and oversized Docker caches from
  pre-commit and non-cancelled CI cleanup paths without blocking work when the
  daemon is unavailable (#1069).

### Security

- Clear the newly published `qs`, `fast-uri`, and `@xmldom/xmldom` advisories
  from every committed JavaScript lockfile, overriding the `qs` release that
  Express 4's pinned range excludes (#1069).

### Fixed

- Discover structural member insertions from the target file's own bytes
  instead of one hardcoded request shape. The route previously required the
  word "array", exactly one quoted value, and a `snake_case` identifier, then
  inserted at the first `[` after it — so a `matches!` alternation produced no
  tool calls at all, and `const NAMES: &[&str] = &["a"];` was corrupted into
  `&[&str, "b"]` by writing into the type instead of the value. The anchor is
  now the delimiter pair that already holds the quoted members (or, when none
  are present yet, the literal-bearing pair inside the named declaration), and
  the separator and spacing are copied from the members already there (#1069).

### Fixed
- The agentic planner no longer mistakes a dotted run of digits for a file name, so a request that states its own identifier (`1.1.2.2.1`) is not sent to open it. No tracked file in the repository has an all-digit extension, while 12.77% of the dotted tokens the predicate accepted in committed prose end in one — every one of them an IP address, a licence identifier or a version.
- When a request names several files and changes only one, the planner now picks the operand by shape and position — a workspace-relative path outranks a bare basename, and among equals the earliest wins — instead of preferring an undelimited token over the path the request marked up. Ranked against this repository's own commit messages paired with the files each commit touched, that raises the share naming a changed file from 68.42% to 84.67%.

### Fixed
- A request that asks for a change *and* for records of that change now produces all of them. The agentic planner's change routes matched such a request whole and answered as soon as the edit landed, so a ladder node made its edit correctly and still failed verification with `missing_proof`, having never written the effect and proof files the same prompt asked for. The route that peels "do this, and leave the answer in FILE" into a delivery plus a residual now runs ahead of the change routes, and the residual carries the change on to them.
- A request that names the artifact of a registered recipe still reaches that recipe. The delivery route peels the named destination off and re-plans the remainder, which is the wrong reading whenever a route below it already writes that very file from the request as it stands: an auto-learning report is *identified* by its artifact path, so peeling "and write self-hosting-learning-report.lino" off left a residual that reached the self-healing recipe instead, and the run wrote a repair case nobody had asked for before writing the report that was asked for. Delivery now plans the whole request through the routes below it first, and stands down when one of them writes the same destination.
- Delivery no longer claims a file the request asked to *edit* as somewhere to put its own answer. The same cues introduce a delivery destination and the file work happens *in*, so "In the file `src/x.rs`, add \"toward \" to the list" used to be read as a place to put an answer -- and the planner wrote its status line over the source it was asked to edit. What separates them is order, the adjacency rule that already binds a cue to a path: a destination is named *after* the write action, an operand before it. Across the 1 118 recorded request sentences that name a file and carry a cue, that reads 21.56% as destinations rather than all of them.
- A route that changes a file now states the change it made instead of reporting that a file was written. Inserting members names the values that went in; replacing a literal and renaming an identifier name what became what, told apart by the scope the edit already carries; registering a module names the registration. A caller that asked for a record of the change -- a ladder node's `result=` field must state it in at least four words -- no longer records "Created or updated and observed `path`", which names the file and nothing else.
- A path written as a double-quoted value is read as a value, not as a place to put an answer. "In the file `x.rs`, add \"bun.lock\" to the `IGNORE_FILES` list" names a file-shaped literal *after* the write action, where a delivery destination would be; quoting tells them apart, because this repository's prose mentions paths bare or in backticks and hands values over in quotes. Of the 241 recorded sentences the order rule admits, 2 quote their path, and both are insertions of exactly that shape.
- A replacement clause no longer runs past the sentence that opened it. The route took everything from the new-value cue to the end of the request as the replacement, which is right for a one-sentence order and wrong for every request that says anything afterwards: a ladder node's five further sentences of harness contract became part of the text it wrote into the source.
- Deciding whether an edit's two operands were quoted no longer counts the request's quotation marks. Requiring exactly two quoted segments in the whole prompt disqualified every request that quoted anything else -- including this repository's own habit of backticking the file it names. Each clause is now asked whether it *is* quoted, which keeps the guarantee that both operands are literals the request supplied.

### Changed
- The self-hosting release target can now be lowered, in the ledger and nowhere else. The ratchet is unchanged -- the next release's floor is still the greater of the previous floor and the previous comparable trailing share -- but a ratchet that only ever climbs can strand a cycle that cannot answer it, which is where issue #1069 ended up: the bar sat at 12.77% and was reachable only by out-measuring it, with no review able to bring it back down. The newest comparable row may now carry a reviewed `target_override_basis_points` that replaces the floor and carries forward until another commit changes or removes it. The v0.345.0 row records **0.50%**, along with the share it replaces, the decision that authorised it ([PR #1070](https://github.com/link-assistant/formal-ai/pull/1070#issuecomment-5535449300)), and why -- so a release with a real Docker image can be cut and iterated on. No flag, environment variable, or workflow input moves the number: lowering the bar is allowed, lowering it quietly is not, and the release notes name the override that let a release out. A falling trailing share is still reported at release time, and the attribution floor is untouched -- a cycle still needs a merged, session-backed Formal AI pull request, which a lowered percentage does not substitute for.

### Fixed
- `check_javascript_dependencies` no longer reports a registry outage as a dependency finding, and can no longer spend the job's budget failing to find out. Two runs showed both halves: in run 100928011479 `bun audit` exited with `error: POST https://registry.npmjs.org/-/npm/v1/security/advisories/bulk - 503`, so npmjs.org had said nothing at all about `bun.lock` and the branch went red for it; in run 100948708530 the retry then reached a clean answer on attempt 2 in 4.35s, but attempt 1 had hung for five minutes first and the 15-minute job was cancelled two lockfiles later. Each attempt now runs under its own deadline (`FORMAL_AI_AUDIT_ATTEMPT_SECONDS`, 180s), and the time spent on attempts that never answered is charged to one budget shared by every lockfile (`FORMAL_AI_AUDIT_BUDGET_SECONDS`, 300s), a worst case near six minutes that leaves the 15-minute job time to print why it failed. An outage is now also recognised in `npm`'s wording, not only `bun`'s: replayed against a degraded registry, `npm audit` reported `npm warn audit 503 Service Unavailable`, `npm warn audit network timeout at:` and `npm error audit endpoint returned an error`, none of which the first draft's list matched, so a genuine npm outage would still have failed the gate outright. The deadline is sized from a measurement rather than a guess: `npm audit --package-lock-only` over `desktop/package-lock.json`, the largest lockfile here, answered `found 0 vulnerabilities` in 2m01s on one run and hung past 300s on the next, so a 120s deadline -- the first draft's -- killed a healthy audit twice and then reported the outage it had itself caused. Only failures are charged, so five slow-but-healthy audits can never exhaust a budget meant for an outage -- and because that leaves slowness unbounded, the five lockfiles are now audited concurrently rather than one after another. Run 100973301529 is why: every audit *succeeded* there and the job was cancelled anyway, having spent 92s, 97s, 155s and 203s-and-counting on them in turn. Neither the deadline nor the budget can help with a registry that answers slowly, and nothing is relaxed to fix it, because the five waits were never competing for anything. Serially the gate costs their sum; concurrently it costs the longest one, and each lockfile carries its own budget. A registry that answers -- with an advisory, or with anything not recognisable as a transport fault -- still ends the gate on its first attempt, and retrying is still not passing: an outage that outlasts the attempts leaves the lockfiles unaudited, which is exactly what this gate refuses to wave through, so it stays closed and annotates the job with which limit it hit.

### Fixed
- A delivery no longer records its own status line over a file the command it just ran had written. The guard that declines a delivery when a later route already produces the requested file only recognised a write call naming that file, never a command naming it with `--output`, so `formal-ai statement-audit --root . --output statement-audit.lino` was followed by a write of the agent's narration to the same path. The guard now reads a planned command's destination the way `driver.rs` already reads it, and the agent-CLI statement-audit run reports the audit instead of overwriting it.

### Added
- `scripts/author-change-with-formal-ai.sh` lets Formal AI author a repository change through the real Agent CLI. It drives the same live loop the `experiments/agent_cli_e2e/` harnesses drive — `formal-ai serve` plus `@link-assistant/agent` — and then does the part those harnesses drop: it lands the file the CLI wrote, keeps the run's raw traces as evidence, and commits both with the `Formal-AI-Session`, `Formal-AI-Evidence` and `Formal-AI-Pull-Request` trailers the self-hosting metric reads. It opens no pull request and pushes nothing, so the work rides inside an ordinary pull request instead of needing one of its own (issue #1069).

### Changed
- The self-development release gate now counts a merged pull request for the work Formal AI did in it, instead of demanding that *every* commit it introduced carry the attribution trailers. The old rule measured the composition of a pull request rather than the authorship of the work, and it had one practical consequence: a self-authored change could never ride along inside ordinary review, because a single human commit beside it erased it. Every claim the trailers make is still enforced — valid session evidence, an evidence path present in the commit, and no attributed commit naming a pull request other than the one that introduced it — and the measured share is unchanged, because it is computed per commit: an unattributed commit stays in the denominator and out of the numerator either way (issue #1069).

### Fixed
- The computer-use end-to-end steps now own their deadline instead of waiting for the runner. Each of the twenty (and, for the held-out set, twenty-four) sessions was bounded by `AGENT_TIMEOUT_SECONDS` and the run was bounded by nothing, so the sessions were entitled to 2400s under a 600s step; on a slow day the runner ended the job and reported only `The action ... has timed out after 10 minutes`, naming the step and not the scenario. The script now clamps every session to what is left of `TEST_BUDGET_SECONDS`, `scripts/run-with-budget-warning.sh` enforces the same budget one level up, and `timeout-minutes` is left as the backstop it is meant to be (issues #977, #1017).
