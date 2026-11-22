# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

Older releases are archived under `docs/changelog/` so that no file
exceeds the repository's 1500-line cap (newest first):

- [0.353.0](docs/changelog/releases-from-0.353.0.md)
- [0.352.0 to 0.352.1](docs/changelog/releases-from-0.352.0.md)
- [0.346.0 to 0.351.0](docs/changelog/releases-from-0.346.0.md)
- [0.307.0 to 0.345.0](docs/changelog/releases-from-0.307.0.md)
- [0.242.0 to 0.306.1](docs/changelog/releases-from-0.242.0.md)
- [0.183.0 to 0.241.0](docs/changelog/releases-from-0.183.0.md)
- [0.177.0 to 0.182.0](docs/changelog/releases-from-0.177.0.md)
- [0.105.0 to 0.175.0](docs/changelog/releases-from-0.105.0.md)
- [0.1.0 to 0.104.0](docs/changelog/releases-from-0.1.0.md)

<!-- changelog-insert-here -->

## [0.356.8] - 2026-11-22

### Fixed

- Allow OpenAI-compatible embeddings to use 1 to 4096 dimensions and return a `no_concepts` error with the input index when the lexicon finds no match.
- Recognize more multilingual occupation titles and preserve the B2B qualifier in job-title embeddings.

## [0.356.7] - 2026-11-22

### Fixed

- Match structured output evidence for camelCase, PascalCase, snake_case, and kebab-case properties, including acronyms, in Chat Completions and Responses.
- Use a property's description as a fallback source label when its name yields no evidence.

## [0.356.6] - 2026-11-22

### Fixed

- Keep the original subject of a how-to request and discover publisher titles from bounded source search. The browser and native solver check the topic of returned titles before using their instructions.
- Request and verify dated currency quotes for the requested pair in chat and compound-interest calculations. Keep recorded quotes in test fixtures.
- Build Google Trends catalogs and learning reports from the regional publisher feed. Preserve the capture source and digest in review artifacts and keep historical feeds in explicit replays.
- Build associative learning reports from caller-supplied observations.
- Run computer-use plans from explicit operations or a supplied teaching corpus. Verify HTTP responses against the authorized origin and keep recorded tasks and responses in test fixtures.
- Build contradiction worksheets from the demands in the request and keep reference contradictions in explicit tests.
- Bind print operands and standard input at request or execution time. Report a program run only when a local execution or publication check was observed.
- Bind caller-supplied ranges, numbers, and text in the program catalog, and derive expected output from those inputs.
- Build self-repair reports from submitted traces and owned source. Read approvals and learned lessons from operator-provided records.
- Honor the browser's network opt-out during every procedural source lookup and recovery.
- Derive budget, itinerary, portfolio, and source-metric programs from their inputs. Compose advice, travel plans, and rhymed verse from supplied cited observations.
- Resolve summary subjects through the shared knowledge registries. Keep demonstration dialogues and benchmark cases in explicit replays.
- Derive the system self-explanation from its cited source modules so revisions to the source are reflected in the answer.

## [0.356.5] - 2026-11-22

### Fixed

- Start the full Docker image with a private memory directory writable by its `box` runtime user. Docker Compose and Desktop mount shared memory at that path.
- Assign Docker release tags only after the full and slim images pass their runtime checks.

### Security

- Update the Rust `url` library to 2.5.9 for RUSTSEC-2026-0592.

## [0.356.4] - 2026-11-22

### Fixed

- Preserve Formal AI report-session metadata through Gemini CLI shell redaction and verify both conversation exports before publishing a release, including reports with a trimmed issue preview.

## [0.356.3] - 2026-11-22

### Added

- Answer bilingual label classification, record questions, field extraction, and language identification from supplied text with supporting quotations.
- Correct supported English and Russian grammar forms with Wiktionary evidence and publish bilingual task-parity measurements for the 0.356.0 release.

## [0.356.2] - 2026-11-21

### Added

- Serve deterministic, multilingual concept-link vectors through the OpenAI embeddings API and advertise their fixed dimensions in the model listing.

## [0.356.1] - 2026-11-21

### Fixed

- Restore greetings, conversation titles, and GitHub report confirmations in external CLI clients.
- Retain wrapper diagnostics in the background and associate report exports with the active client session.
- Require greeting and report smoke tests against the selected binary before an automatic or manual release.

## [0.356.0] - 2026-11-21

Record what blocks the vision (issue #1548): keep the issue verbatim as an architect note, correct the status sentences of README, ROADMAP, GOALS, VISION and the meta-algorithm page to what was measured, document the solver switches and the issue planning guideline, add the case study with both inventories of failed attempts on real issues, and add requirements R1548-1 to R1548-19 with the test that pins the published issue graph. No source file of Formal AI changes; the 12 capability rows stay Partial and are owned by the parent issue #1561.

Answer an English issue in English (issue #1557): language markers of a script several languages share (Latin for Spanish and English) now match whole words only in the Rust detector, its JS twin and the browser worker fallback, so "formalization", "described" and "response" no longer read as the Spanish markers "formaliza", "escribe" and "responde". A repository work item's answer language is now decided by the user's own turn, then the proportion of prose in the issue body, then its title, with code, quotations, URLs and identifiers excluded, so a quoted Russian sentence no longer flips an English issue and English identifiers do not flip a Russian one. The 19 attempted issue bodies are kept as a regression fixture checked in both runtimes.

### Fixed
- Resolve file names named in a work item before reading them (issue #1558). Each name is tried as an exact path, then as a unique file-name match found with glob or `find`, then as at most three candidates ranked by path similarity to the words around the reference. A link to a file on GitHub resolves to its repository path, and a link to another repository becomes one unresolved item. Product names such as `Node.js` are recorded as terms. A `File not found` read now counts as a failure. A failed or completed read is never planned again: the planned path is compared with the read bound to the workspace, and the relative and absolute forms count as the same path. The remaining evidence is still read, and the report lists `unresolved` and `terms` next to the observations. JS and Rust share the behaviour and are tested with the five recorded issue texts and the hive-mind 4033 case.

### Fixed
- A work-item session no longer answers with a raw JSON record (issue #1560). It answers with a localized report. The report says what stopped the session, which files it read, and which requirement a refusal concerns. The record stays in the trace.
- The read policy is evaluated over the user's own turn, not over fetched issue text. A Hive Mind session on issues 1486, 1489 and 1501 now reads repository files instead of refusing every read.
- A fetched GitHub issue page keeps only its title and body. A page without an issue body, such as the sign-in page, is rejected and the `gh issue view`, REST and `gh api` reads run instead.
- A Markdown link label that names a file is read at the path of the work item's own blob link, not at the repository root.

### Fixed
- The Rust crate compiles again on main. `ProofOutcome::confidence` (added for #1487) returned `f64`, but its only caller passes it to `finalize_simple`, which takes `f32`. Every Rust gate on main failed with `E0308 mismatched types` at `rust/src/solver_handlers/user_intent.rs:141`. It now returns `f32`, the type the replaced `match` used to infer.
- `negated_theorem_request_is_refuted_never_proven` (`rust/tests/unit/proof_request.rs`) now shows the exact refutation it expects, as the tests-as-docs gate (R234-2) requires.
- The JavaScript worker answers Euclid proof requests exactly as the native engine does (#1487). Before, the refutation of "Prove there are not infinitely many primes." embedded the positive theorem's "Proof (method: contradiction)." block, which the native test forbids. The affirmed proof was a hand-written per-language copy without step labels. Both are now rendered from the seeded theorem with the proof presenter helpers. The refutation is a `Disproof` with labelled steps and the native clarifying-question footer. `rust/tests/web/issue-1487-proposition-identity.test.mjs` pins both exact answers, and no refutation may carry a `Proof (` heading.

### Added
- `experiments/issue-1558/rust-module-harness/` compiles the unchanged `rust/src/agentic_coding/work_item_references.rs` and its unit tests on their own, without building the whole crate (#1558).
- `experiments/plan-event-append-race/stress.sh` runs the generated plan-event append command as two simultaneous processes many times and counts duplicate records. It is used to investigate the intermittent failure of `simultaneous retries take the bounded lock and retain exactly one event` on main.

### Fixed
- Hive Mind issue drafts (#1484) now admit one attempt identity that binds the selected Formal AI source, the built binary's sha256 and the selected Hive Mind source and exact image. A retry keeps the identity and a new source gives a new one. Before any task effect the draft job checks that the downloaded binary and checkouts are the admitted ones; a missing artifact is `deferred-infrastructure` and a stale or same-version-but-different binary is `identity-mismatch`, both with zero capability credit. The workflow hands `FORMAL_AI_DRAFT_SOURCE_SHA` and the `live-current` mode to Hive Mind, then compares Hive Mind's serving record (executable digest, image digest, versions) with the admission. Receipts distinguish live-current, published-release and historical-replay attempts, and only a verified live-current attempt counts as current-source success.

### Fixed
- A system prompt of one agent session is no longer stored as a learned standing requirement and replayed in another repository's session (issue #1559). A prompt that configures a session is recorded with role `instruction` and never learned. A learned rule is one sentence of at most 240 characters (`rule-maximum-characters`) that names no single task. Its topic is the statement's subject, never a pronoun or function word in en/ru/hi/zh. Each amendment carries the scope it was learned in (user, or repository and session for agent sessions) and applies only there, unless it was stated in several scopes and generalized after replay verifies it. Agentic final answers list applied amendments in their trace (`thinking_steps`, `reasoning`, `evidence_links`) instead of their text. Agent sessions sharing one memory store, as on a Hive Mind sidecar, now see only their own requirements unless `FORMAL_AI_SHARE_STANDING_REQUIREMENTS=1` is set.

### Security
- Update `rustix` from 1.1.5 to 1.1.6 in `rust/Cargo.lock` for RUSTSEC-2026-0486 (unsound xattr functions with a zero-length buffer), which made the denied-warning `cargo audit` fail on `main` and on every pull request. Only the version and checksum change; every other locked dependency stays as it was.

### Fixed
- Hive original delivery no longer aborts unstarted clients when another CI run is still pending (#1485). An unclaimed client leg now rechecks a bounded number of times. If a prerequisite is still pending, the leg writes a `deferred-before-effects` receipt (`formal-ai-hive-client-outcome/v1`) with its identity, the blocking run IDs and statuses, a timestamp, proof that nothing was claimed, and zero execution and authorship credit, then exits without claiming anything.
- Verification, ledger and acceptance jobs read that typed outcome instead of looking for claim or result artifacts that were never produced. A deferred-only run is reported as deferred and incomplete, never as accepted. A failed, cancelled, skipped or timed-out prerequisite, or a changed source, is a refusal. A claim held by this run is an interrupted leg, handled by the existing cleanup and never retried.
- A published candidate resumes on the blocking run's completion event through a `hive-wakeup-<run>-<role>` artifact. The resumed run re-admits only the same immutable candidate and allows at most eight continuations. The existing claim ledger still decides which clients run, so a claimed leg is never executed again.

### Added
- Hive draft and three-client delivery runs are measured per phase (#1486): queue, checkout, install, compile, image preparation, transfer and load, solver, verification and publication, with bytes and useful-attempt counts. `ci-phase-timing.yml` checks them against numeric objectives in `data/benchmarks/ci-phase-timing/slo.json` and fails with `::error` when an objective is breached in every one of the latest runs.
- Captured Hive draft failures replay offline. `extract-draft-replay.mjs` turns a session artifact into a fixture that runs with no Docker, release or Rust build, and `select-focused-tests.mjs` runs the tests that `focused-test-map.json` maps to the changed components or to the captured failure.
- Hive drafts build the Formal AI binary at most once per source, toolchain, target, features and lockfile identity. Other drafts reuse the product only after checking its producer, identity and binary digest. Failed, stale or mismatched products are never reused.

### Changed
- The delivery task image travels as a delta over its digest-pinned runtime base. Each leg pulls the base by digest, verifies it and still requires the exact producer image ID. The full archive ships, with the reason recorded, whenever the delta cannot be proven.

### Fixed
- Statement identity keeps the operand order, ordered comparisons (`<` versus `<=`), the and/or shape, quantifier and negation scope, and explicit counts (`twice`) in both the JavaScript and native readers (issue #1487). `2 is less than 1.` no longer restates `1 is less than 2.`, `a - b` and `b - a` and `true or false` and `true and false` keep distinct identities and statement-merge signatures, and summarization drops a sentence only when a checked relation (equivalent or entailed) certifies it. Shared terms alone no longer certify a restatement. Checked paraphrases (`2 is greater than 1.`, `1 < 2`) and repeated mentions still unify.
- A proof request binds to the exact theorem proposition. `Prove there are not infinitely many primes.` is refuted (`proof_binding:negated`, `proof_outcome:disproven`) instead of being answered with the positive proof, and a negation whose scope is not read gets an honest partial plan.
- The intent-formalization cache answers only the request it was formalized from, so `a < b` and `a <= b` no longer share a cached formalization.
- Program synthesis withholds a candidate when the requirement holds structure that no discovered coding part reads (`twice`, `not`, `or`, an operator). The requirement keeps an open `structure:` obligation (`synthesis:obligation:unresolved_structure=…`), so `Return twice the number of vowels` no longer receives the program for `Return the number of vowels`.

### Fixed
- A work-item session no longer stops after reading evidence (#1483). With a Run tool, a boundary repair or regression test is checked after its edit by running samples derived from the requested bound against the original signature, never against the candidate. A pass verifies the requirement; a source that contradicts the bound fails the check (`IndependentCheckFailed`). Running the same task again is a verified no-op.
- A work item that names no repository file collects its leads: attachments become explicit unresolved evidence and are never described, linked documents are fetched as evidence only, and code identifiers in backticks are searched and the best match is read. A work item with no file and no lead is still left to the general planner.
- The localized report states which files changed, every check outcome and every unresolved lead. A verified record that leaves requirements unverified is reported as partly done, never as done.
- A documentation edit is accepted independently of the derivation that wrote it: claims in the task are judged against the observed manifests (a contradicted version is rejected, `ContradictedIntegrationClaim`), every derived fact must be anchored in its raw source and rendered in the artifact, the original guide must be preserved, and each requirement gets its own status. Consumer-search coverage records are not claims, and an accepted requirement set leaves only the work it does not cover (integration semantics, consumer coverage).
- A completed evidence collection that still makes no change names the narrowest gap it leaves: `RequestedInterfaceAbsent` (a function the task asks for exists nowhere, `create-interface:<name>`), `NoChangeRecipe` (the evidence is read but no supported change recipe applies, `change-recipe-unavailable:<path>`), or `RelatedEvidenceUnread` (the task only links related pages). The ten planned-not-executed fixtures of #1483 each end with an edit and a check or with one of these named gaps; the replay is in `docs/case-studies/issue-1483/fixture-replay/`.
- Through the Claude adapter, a Read listing that carries no completeness proof is read again through the registered file-IO program when a Run tool is offered, and an Anthropic Bash result without an exit status is read from the program's receipt, so the Claude and Codex adapters both reach an edit, an independent check and a verified no-op repeat. Without a Run tool the uncertified read stays an explicit gap (`IncompleteSourceObservation`) and nothing is written.
- The qualification oracle classifies every evidence terminal (`NoRepositoryReference`, `AttachmentEvidenceUnavailable`, `ExternalEvidenceUnavailable`, `RelatedEvidenceUnread`) as a gap and carries its named gap, and the focused offline test map selects the evidence-gap and edit-and-check tests for these components.
- The work-item report is written in the language the plan answers in: the body's prose, then the title's, then the whole text (the #1557 rule, `work_item_language` / `workItemLanguage` in `general_planner`). The reports of issues 1330 and 1396 were Russian although their bodies are English; they are now English.
- A regression-test task over a source that already meets the requested bound writes a test that passes when plain `node --test` runs it, and a repeat is a verified no-op.

### Fixed
- Main pipelines no longer fail on Rust 1.100 test executables (#1569). Cargo now writes them under `target/release/build/<package>/<hash>/out/`. `collect-build-artifacts.sh` takes each executable from Cargo's JSON report and searches both layouts when there is no report. The native fixture runner accepts the new path, and the memory store recognises those binaries as tests.
- `rust/src/agentic_coding/modules.rs` is in the canonical order the ordered-lists gate expects. The generated-version-literal gate treats the captured CI run records under `data/benchmarks/ci-phase-timing/baseline-runs/` as evidence.
- The summarization ratchet, the bounded-subprocess anchors and the no-memorization source captures match the merged sources again.
- The committed browser worker `js/formal_ai_worker.wasm` is rebuilt from the current sources with rustc 1.100.0. Three merged commits had edited its inputs without a rebuild, and the runners had moved from rustc 1.99.1, so `check-wasm-worker-current` failed on main.
- The daily dependencies-latest update rebuilds the worker and stages it with its input record. It used to rebuild without staging, so its own wasm gate failed every day and the update never landed.
- The hive draft-attempt and pre-effect deferral tests no longer print `::error`/`::notice` workflow commands. Those commands showed up as red annotations on passing jobs.
- External Benchmarks runs every suite even when one fails. Each failure gets its own annotation and the step still fails. Before, a failing humaneval stopped the nine suites after it.
- The agent ladder's composite legs start only after `leaf-evidence` succeeds. When it failed, all 16 legs used to fail too, on an artifact that had never been uploaded.
- A failed sccache download no longer fails a job, in either the install step or its post step. The job builds with the real compiler and gets a warning. Two ladder leaves had failed on HTTP 502/503 from the sccache release download.
- The agent ladder installs Bun through `.github/actions/setup-bun-verified`. If `oven-sh/setup-bun` reports success but installs nothing, the composite retries once, then checks that the pinned `bun` runs. Without this, a later step failed with "bun: command not found".
- The Hive Mind draft no longer reports an attempt that ran solve as `deferred-infrastructure`, "Hive Mind ended the attempt before solve". It used to read the capture wrapper's own `attempt-outcome.json` as Hive Mind's verdict. Hive Mind's typed outcome is now read from `attempt-hive-outcome.json`, and a capture record is never taken for one.

### Added
- `data/meta/wasm-worker-inputs.lino` records the worker's compiler, its dependency closure and the digest of each input. It is generated by `node scripts/wasm-worker-inputs.mjs --write`.
- `--check` needs no compiler and runs on pull requests, so an edited worker input that was not rebuilt now fails there instead of on main.
- When the main gate fails, `--explain` reports whether the inputs or the compiler changed.

### Changed
- Several regressions failed only on main, after their pull requests had passed. Pull requests now run them in the fast JavaScript inventory:
  - an ordered-declaration check;
  - a JavaScript twin of the generated-version-literal gate;
  - the build-artifact collection tests.
- The fast pull-request JavaScript inventory runs in two longest-first groups instead of one. A single group had outgrown its 360-second budget and was being terminated.

### Changed
- Pull requests that change a Rust compile input now run one focused native check: main's clippy profile on the library and binaries (#1191). PR #1497 passed every PR check and then broke main with E0308 and, after that fix, with `clippy::manual_contains`, because no PR job ran the Rust compiler. The check builds and publishes nothing and is never native or release evidence. Docs-only, JavaScript-only, test-only and script-only pull requests skip it, and an unknown path selects it.
- The tests-as-docs gate has a JavaScript twin, and it runs in the fast pull-request lint. Main run 43710854562 had failed on a loose-only test that its green pull request never checked.
- Hive finalization skips its prepare step when the whole delivery was skipped, so a skipped run no longer fans out extra events.
- Every workflow pins its runner label (`ubuntu-24.04` and the like) instead of a moving `-latest` alias, and `scripts/check-runner-labels.mjs` rejects `-latest` labels in the fast pull-request JavaScript tests (`runner-label-pinning.test.mjs`) and as a web-stage gate; a new test keeps the dependency-free fast lint job free of scripts that import installed packages (#1191).
- The fast pull-request lane now replays every failure that reached main after a green pull request. `data/meta/main-only-regressions.lino` lists each one with the pre-merge check that now catches it: E0308 and `clippy::manual_contains` (the native check), the loose-only test, the module map, inline unit tests, the issue-1488 payload drift, the issue-698 planner expectation, the top-level example, the uutils `mkdir` lock race, the moving runner label and the two Self-development debt-ratchet failures (runs 41317019358 and 41804771645), 12 regressions from 5 main runs.
- The tests-as-docs twin files are spelled in full words.
- `AGENTS.md` and `CLAUDE.md` state one push gate, the same as CONTRIBUTING.md: drafting sub-agents never push, one agent pushes once after collecting every failure of the latest run, nobody pushes while a run is in progress, JavaScript checks run locally first, and native builds stay in CI. `scripts/prune-build-cache.sh` follows a shared `CARGO_TARGET_DIR`.
- The weekly status job reports open pull requests with no update for 30 days, and on main it fails with the exact `gh api` command while the `main` ruleset does not require the JavaScript stage, the Rust stage and `Pipeline Status`.
- The project-structure tree moved from CONTRIBUTING.md to `docs/development/project-structure.md`, so CONTRIBUTING.md stays under the 1,500-line limit.
- `docs/case-studies/issue-1191` records the timeline, root causes and before/after numbers of #1188 against recent pull requests.

### Fixed
- Main's clippy gate passes again: two `iter().any(|x| *x == y)` loops from PR #1497 use `contains`.
- The plan-event lock is claimed with an `O_EXCL` redirection instead of `mkdir`. On ubuntu-26.04 runners uutils `mkdir` could let two writers both claim the lock.

### Fixed
- Main's CI/CD Pipeline is green again after the regressions merged since 58a229dda (#1571):
  - two `clippy::manual_contains` lints are cleared;
  - `docs/architecture/module-map.md` now has the `standing_scope` row;
  - the `work_item_leads` inline tests have moved to `rust/tests/unit/agentic-coding/`;
  - the top-level `examples/` script has moved under `rust/examples/`.
- The Rust `gh` work-item read now takes the body as an observed payload, as JS does, so a body quoting `exit 5` or `HTTP 403` (issue 1488) is no longer dropped (#1571).
- The issue 698 driver test now expects the `read_file` step that #1560 made possible; the JS planner plans the same steps (#1571).
- Layered CI js-tier suite groups are now planned from the per-file seconds CI actually observed (`ci-durations.mjs --javascript-ci-logs`). Group 7 no longer reaches its 360 s budget because of 0.1 s defaults for unrecorded files (#1571).
- `ci-durations.mjs` reads job logs with `gh api --allow-escape-sequences` and reports jobs it skips. Newer `gh` refused the ANSI-coloured logs, and `--tests` and `--playwright-files` silently measured nothing (#1571).
- `check-dependencies-latest.rs --apply` now writes Cargo and npm requirements without SemVer build metadata (`toml_edit = "0.25.19"`, not `"0.25.19+spec-1.1.0"`). Cargo had warned that the metadata "will be ignored" in every build of the daily dependency run (#1571).
- The scheduled learning cycle now caches the `rust/` workspace and prints its captured stderr on every exit. Previously rust-cache ran `cargo metadata` at the root without a `Cargo.toml` and cached nothing, and a failed `cargo run` ended in a bare exit 101 that hid the error (#1571).
- The Issue 1028 Agent CLI ladder now runs each depth-5 leaf as its own leg (32 legs, not 16 pairs), and keeps its 1200 s tree budget and 30-minute job cap. The weekly step-budget audit had failed because a two-leaf leg used 1196 s of the 1200 s budget: its first leaf compiled the crate cold while sccache rejected 706 of 877 writes (#1571).
- The Hive Mind draft capture no longer writes `*.log` files into the mounted session directory (`attempt.capture.jsonl`, `attempt.stdout.txt`, `attempt.stderr.txt`). The shared runner explains a red draft from the newest `*.log` there, and the live capture was always newest, so run 43730396084 reported "docker run exited 5 without a session log error" instead of "Finished without changes".
- Every workflow job now runs on a pinned runner image (`ubuntu-24.04`, `windows-2025`) instead of `ubuntu-latest` or `windows-latest`. GitHub had annotated every job of main run 7837d50 (95 notices) that `ubuntu-latest` will move to Ubuntu 26 from October 19, 2026 (actions/runner-images#14748). The release test matrix keeps `os: ubuntu-latest` as the leg's name for job names, duration data and `unit-tests-<os>` checks, and maps it to `ubuntu-24.04` in `runs-on` (#1571).
- The daily dependency run no longer moves `@link-assistant/web-capture` to 2.x in the desktop and VS Code manifests. Since 2.0.0 it declares `engines.node >=26.10.0`, but Electron 44 embeds Node 24 and VS Code runs extensions on Node 24, so npm printed `EBADENGINE` twice and the move was only a warning. Both manifests now carry a blocked note naming link-assistant/web-capture#191, and a test fails when any runtime package in either lock requires a newer Node than Electron 44 embeds (#1571).
- The Cargo audit now covers every committed `Cargo.lock`, not only `rust/Cargo.lock`. The issue #1017 reproducer's lock had rustls 0.23.43 (four Dependabot alerts), the yanked chacha20 0.10.1 and five more advisories (quinn-proto, quinn-udp, rustix, webpki-root-certs) while the Security workflow stayed green. Those crates are updated. Unshipped locks fail on vulnerabilities, unsound crates and yanked releases. An `unmaintained` notice with no fixed release (unsafe-libyaml under the pinned meta-language 0.58.1) is reported without failing (#1571).

### Added
- Every answer in both runtimes goes through one recursive procedure (#1550): `solve_recursively` with an in-process executor in Rust, and `solveRecursively` in the worker. At every level the procedure:
  - splits the request by its task tree, by obligations and relations, not by English separators;
  - derives a test for each task and runs it;
  - drafts candidates from the methods;
  - selects by least action and records the real reason;
  - composes by the structure of the split;
  - recurses within depth, breadth and budget bounds. The bounds are seed data, can be overridden through `RunOptions`, and are the first event of the trace.
- The trace shows `meta_exec:*` events (bounds, task, test, draft, compose, select, unresolved, result). Rust and JavaScript emit the same sequence, checked by `rust/tests/fixtures/meta-executor-parity.json`. When a sub-task has no test, the procedure returns an explicit unresolved result instead of an answer.
- A census fails when a specification-corpus answer skips the procedure or a new early return appears. It runs in `rust/tests/unit/meta_executor_census.rs`, `rust/tests/web/issue-1550-procedure-census.test.mjs` and `scripts/check-specification-in-javascript.mjs`.
- README.md and `docs/meta-algorithm.md` carry the same stage and entry tables. `rust/tests/web/issue-1550-procedure-documentation.test.mjs` fails when the tables diverge from the code.

### Changed
- The candidate, validation and selection records hold only values derived from the turn. The fixed composition priors 1.15, 1.05 and 0.9 are removed.
- Russian, Hindi and Chinese requests are split the same way as their English form.
- The meta reasoner's declared ground-depth bound decides how deep glosses are grounded, in both runtimes.

### Removed
- The dead `meta_core::record_meta_core_execution`.

### Fixed
- Hello World issue-to-PR delivery (#1162) ends in one clean, ready pull request in both runtimes:
  - The workflow takes the file name the request suggests ("a meaningful name like `test-hello-world.yml`"), in English and Russian. A literal path such as `ci/test-hello-world.yml` (#1450) still wins, and a suggestion that names a directory or sits in a quote falls back to `Main.yml`.
  - A resumed delivery retires an earlier session's workflow for the same program, so it never ends with both `run.yml` and `Main.yml`. It never retires a workflow with uncommitted user edits, an untracked or symlinked copy, or a workflow for another program.
  - The pull request completion reads the review feedback through a bounded `jq` filter. It sets the title from the issue and the description with `Resolves <issue>`, and edits only when they differ, so a second run is a no-op. It then marks the pull request ready.
  - Every orchestrator status notice seen on the 2026-11 test pull requests (14 patterns) counts as the session's own bookkeeping, not as change requests. A real `CHANGES_REQUESTED` review or reviewer comment still counts as feedback.
  - The exact successful stdout of a Codex `exec` envelope or a JSON tool envelope keeps the leading porcelain column, so ` M main.rs` is not read as `M main.rs`.
  - Version provenance notes are written only for the pins the rendered workflow uses.

### Added
- Independent task acceptance (#1166), in Rust (`rust/src/agentic_coding/task_acceptance.rs`, `task_condition.rs`), JavaScript and generated TypeScript. It reads only the task text and the candidate's files and observed output:
  - every clause gets a verdict (discharged, contradicted, unresolved, inapplicable) with a justification and its byte span;
  - judgment clauses stay unresolved, and nothing is reported as a generic "style" pass;
  - the workflow path (including #1450's `ci/` convention), triggers, file name, runtime and observed status are checked against the issue;
  - conditional outputs in the supported subset become a truth table, so reversed comparisons, `and`/`or` and negation stay distinct, and unsupported grammar is a gap;
  - each task gets a canonical meaning and an exact identity.
- A finished recipe's answer lists the actionable verdicts and says the task is not complete while a required clause is contradicted or unresolved.
- One shared fixture (`rust/tests/fixtures/issue-1166/acceptance.json`) for the native and JS tests. It includes executed Python candidates; one shares its generated test's misunderstanding and is still rejected.

### Fixed
- An expected output mentioned twice is printed once; an explicit `twice` prints it twice. Conditional outputs are no longer bound as unconditional literals.

### Fixed
- A work item that only cites a Hello World repository (a `test-hello-world-…` address) or a source file name no longer gets a stored Hello World program written, run, committed and reported as "Created and verified" (#1562). The native work-item route solved the whole fetched text, and `normalize_prompt` read `…/test-hello-world-019f…` as "test hello world". Now a recipe is kept only if the work item's obligation text selects the same file in the same language. The obligation text is the work item with every address, code span, link, table row, block quotation and fenced block blanked byte for byte (`rust/src/agentic_coding/work_item_citations.rs`, `js/agentic/work_item_citations.mjs`). A code span that is the operand of an obligation clause, such as the output in "print exactly: `Hello, World!`", is kept for this binding. So a work item whose only output literal is a code span keeps its program. An unbound recipe is dropped before the command reroute, so nothing is written, run, committed or marked ready. The evidence record names it (`unboundWrite`, remaining work `unbound-write:<path>`), and the summary says the file was not written because no obligation names it.
- Requirement rows that only cite evidence (a table row, a fenced block) are marked `citation` in the work-item evidence record. They are no longer listed or counted as unverified requirements.

### Added
- The texts of hive-mind issues 4031, 4034 and 4035 are regression fixtures (`data/benchmarks/fixtures/issue-1562/`). Their recorded production requests are draft replays of family `work-item-cited-program`. Tests: `rust/tests/unit/agentic-coding/issue_1562_cited_programs.rs`, `rust/tests/web/issue-1562-cited-programs.test.mjs`. The case study is `docs/case-studies/issue-1562/`.

### Fixed
- An output literal is printed once (#1156): "print `Hello, World!` and the program outputs it" is one line, not two. A count ("twice", "3 times") or "again" still repeats it, and "outputs" now counts as an output verb.

### Fixed
- A repeated or resumed delivery is a success (#1157): when the verified files already match the last commit, the answer says so in a sentence ("Nothing to commit: … a repeated or resumed run is a success, not a failure") and the run makes no new commit and no pull request edit. The commit stages only the verified files by literal pathspec. `rust/tests/unit/issue_1157_idempotent_commit.rs` drives the Rust planner through two sessions against a real temporary repository.

### Fixed
- Hive Mind's formal-ai prompt now binds the delivery to the pull request (#1158): "Update the pull request at <url>." joins "Your prepared Pull Request: <url>" in `prepared-pull-request-patterns`, so the run reads the pull request feedback, sets the title and a description that names the verified artifact, the verification commands and `Resolves <issue>`, and marks the pull request ready. Before this the 2026-09-27 runs left a `[WIP]` draft. The final answer reports each of these steps.

### Changed
- A missing compiler without a workspace grant now stops with a "grant required" answer that names the program and the pinned publisher procedure a grant would run, the re-probe and the one retry (#1159), for example `scalac` and `https://api.github.com/repos/scala/scala3/releases/latest`.
- A recovered missing compiler is now reported in the answer: the program, the pinned publisher procedure that ran, the re-probe and the retried command (#1159). A failed transaction is never reported as a recovery.
- The delivered Scala workflow installs `scalac` from the same release feed as the recovery publisher record, pinned by a Rust test (#1159).

### Added
- Hive Mind prompt fixtures in `rust/tests/fixtures/hive-mind/` (#1160): the verbatim formal-ai prompt of the 2026-09-27 Kotlin run, the regular prompt and the auto-restart prompt after `?? Main.jar`, with the shared fake `gh` used by the offline delivery tests in both runtimes. With the regular or restart prompt, a stray `Main.jar` after a Kotlin delivery is ignored, committed and pushed, and the pull request stays ready.

### Added
- Configurable controlled natural language (CNL) output in English, Russian, Hindi, Chinese and Spanish. Independent output-standard (cnl-default, asd-ste100, ace, gellish, sbvr, plain-language, minimal-english, e-prime, basic-english, special-english, globish) and lexical-density (standard, primes, top-N, max-precision) settings are available in the CLI, OpenAI-compatible API, browser worker and desktop preferences, and can be switched during a dialogue. One declarative grammar per language (`data/seed/cnl-language-<id>.lino`) drives realization, parsing and validation, in both the JavaScript and Rust engines.
- Whole-prompt controlled dialogue. Declared fact questions are answered in the selected standard. An ambiguous question gets a numbered CNL clarification, and a numbered reply picks one reading. A task paraphrase that the solver did not understand is solved through its canonical controlled prompt.
- An exhaustive matrix over languages, standards, densities and meaning graphs, with a reviewed infeasibility ledger and per-language JavaScript/Rust parity fixtures.

### Fixed

- Keep duration facts in job descriptions from triggering unit conversions or calendar events (#1588). Conversion intent, number and source unit now belong to the same request span in JavaScript and Rust.
- Recognize English and Russian year forms, render short exact decimals and reduced fractions, and agree Russian time units with the numeral.
- Answer stated day offsets from validated Gregorian dates, including leap days and year rollover: 10 days after 25 December 2026 is 4 January 2027.

### Fixed

- Enable read-only source retrieval by default, preserve explicit offline/network denies, and explain harness permission controls in five languages. Recognize direct permission paraphrases without taking over quoted translation or programming tasks (#1582).
- Keep routine wrapper/server diagnostics in log files by default and offer explicit terminal and nested formatting options (#1582).
- Supply served context limits to Claude, Codex, Qwen, Aider and Grok configurations. Gemini's fixed input-context fallback is documented as an upstream limitation (#1582).

### Fixed

- Apply system and developer output constraints within the current Chat Completions or Responses request, including language, JSON format, allowed labels and word counts. Unsupported constraints are named explicitly in a refusal and recorded in `thinking_steps` (#1587).
- Recall facts stated in earlier user turns without treating assistant text as user evidence or sharing request instructions across conversations.

Fix English and Russian calendar-duration sums and completed months/years between stated dates. Preserve exact day offsets across year boundaries. Currency conversions disclose a captured historical rate, its official source and date, and round once to two decimal places in both runtimes.

Use the repository's local picomatch security backport for desktop and VS Code development dependencies until upstream releases a fix for GHSA-fc5q-39xc-vjpf.

### Fixed

- Make full server dialog capture opt-in and keep default request diagnostics to metadata (#1592).

### Added

- Add `serve --ephemeral` and `FORMAL_AI_EPHEMERAL=1` for read-only memory with no server diagnostics, dialog capture, or stored derivations. Report the active memory and capture settings in `/health`.

### Fixed

- Remove standalone greetings, thanks, and sign-offs from free-text summaries. Keep the main clause and its reasons together with their original sentence case. (#1594)

### Fixed

- Honor forced client function calls in OpenAI-compatible Chat Completions and Responses, including streaming. Validate arguments before returning the call and expose underivable arguments as a structured refusal. Server tool permissions still apply to shell, file and web tools.

### Added

- Support structured JSON answers and machine-readable refusals on the OpenAI Chat Completions and Responses endpoints. Both runtimes validate answers against the requested schema before returning them and record the schema and result in the answer's derivation.

### Fixed

- Use localized wikiHow sources for general how-to requests. The browser can recover relevant steps from a matching archive or the web capture service. Ranked search excerpts no longer count as instructions.
- Continue source-backed how-to answers on follow-up requests in the native solver.

### Added

- Store and run dynamic meta recipes and associative plugins, trace doublet substitutions across CRUD-triggered cycles, and review operator-controlled compilation and rule activation.

### Fixed

- Route procedural questions from Codex through Formal AI's source-backed solver when the client advertises web search.
- Ask for issue-report destinations and log scope in chat when Codex advertises a Plan-only prompt tool. Keep confirmation before creating an issue.
- Run the staged release after credential, secret and crate checks. Keep the broader diagnostic matrix visible in the pipeline status.

## [0.355.34] - 2026-11-05

Give the single public Pages smoke a bounded cold-CDN worker allowance while preserving real WASM and answer assertions, keeping local waits and the original full suite unchanged. Reduce its overall cap and record startup duration plus bounded pending-request diagnostics so a slow or failed worker has an actionable cause.

## [0.355.33] - 2026-11-04

Fix post-publication self-authoring when GitHub adds descendant jobs during pagination: discard the changing inventory and restart from page one at most three times, retaining the shared deadline, request limit, and exact publisher/source/attempt checks. Replay the observed 59-to-60 job-count race offline and refuse persistent churn or invalid job identities.

Fix release freshness checks when GitHub reports a queued or waiting parent pipeline while its staged writer is running. Share nonterminal aggregate validation across all staged entry points while preserving exact current-job, runner, source, attempt and writer-group membership proof. Replay the observed failure and stale/foreign/missing-lease refusals in essential offline PR tests.

## [0.355.32] - 2026-11-04

Formalize explicitly requested sums and inclusive bounds through the existing JavaScript and Rust calculators, preserving the original quantities and source spans. Decline unsupported conditions and mixed units, and keep aggregate concept questions outside the comparison handler. Preserve computed quantity evidence through the server projection, and keep the legacy worker within its existing budget through a separate quantity module. Add captured regression and ownership checks to the essential offline PR inventory; retain complete responses for reasoning acceptance.

Complete quantity seed metadata, role declarations and localized dictionaries; preserve backend input scope, strengthen exact QA answers, correct debug source ownership and move cheap governance checks into the fast PR preflight.

Fix quantity routing source provenance and grammar scope without weakening formalization quality checks.

## [0.355.31] - 2026-11-04

### Tests

- Verify dependency versions, native target constraints, literal feature options and imported-call excerpts in the guide actually written by the maintained planner.

### Documentation

- Record bounded original-task and held-out documentation probes, including an independently checked useful partial edit through the actual Agent client and the maintained JavaScript backend.
- Keep ordinary Hive execution, remaining clients, production orchestration and full integration acceptance explicitly pending. Preserve incomplete attempts and distinguish supplied scaffolding from source-derived work.

### Performance

- Bound quote-predecessor character iteration to the final two UTF-16 units instead of materializing each complete prefix. Preserve Unicode, index coercion and quote-span behavior, with generated TypeScript and browser-worker counterparts.
- A single captured terminal request replay retained identical response content while taking 5.55 seconds before the change and 2.49 seconds afterward. Deterministic work-bound and semantic tests protect the optimization without a wall-clock CI gate.

Fix stale requirement verdict projections and check their freshness in the fast JavaScript PR gate. Bound live work-item runner and broker commands by the authenticated job deadline, allow progressing image acquisition within that window, and reserve time for cleanup and evidence upload. Preserve historical cleanup ownership when the current policy gains a new helper. Retain the failed acquisition logs and truthful test-issue/protected-branch outcomes; full client acceptance remains pending. Verified protected-only branch residue is explicitly deferred after issue/PR closure; it remains incomplete and grants no delivery credit, while uncertain cleanup still fails.

## [0.355.30] - 2026-11-04

### Fixed
- Add a trusted JavaScript-only server grammar setting, verify declared literal creates against fresh target reads, preserve failed-command output streams, and report missing edit capabilities explicitly.
- Bound source-authority metadata read retries with one shared deadline and preserve exact resolver dependency custody in detached release protocols.

- Authenticate each signed artifact against its actual successful producer attempt after partial reruns, preserving exact digest, source, workflow and ref checks.

### Fixed
- Verify literal file changes across dedicated Read and shell clients, preserve exact content, and reconcile all artifact obligations.
- Reuse qualified observations safely across dependent edits and retain specialized editing capability routing.
- Prevent source payloads and overwrite permission from authorizing unintended whole-file changes; bind journal effects to the current plan before execution.
- Run inexpensive JavaScript source-policy checks on pull requests while retaining full production qualification.

### Fixed
- Preserve an owned empty-string fallback in the native statement-aware write guard so the coding runtime can compile.
- Keep unsupported edit tails explicit, and admit typed source-edit evidence only through its maintained guarded plan.
- Share request-authority checks between live execution and fresh replay, with strict loaded-source comparison and bounded mismatch diagnostics.
- Use hyphenated names for new verification messages without increasing notation debt.

# Fix independent evidence receipt verification

- Record the actual child status, stdout and reported receipt for fixture shell actions.
- Accept fresh exact successful ordinary cat readbacks for the unchanged L21 effect and proof records, with negative controls for stale, failed, truncated and mismatched observations.
- Record the initial 5% independent reasoning and fact-checking delegation target separately from tool execution and code authorship.
- Route JavaScript task-decomposition questions through the existing native algorithm and shared seeded wording, with exact TypeScript mirrors.
- Assess finite JSON scalar claims against fresh complete Read evidence as supported, contradicted or unknown, retaining claim/source identities and spans.
- Add an explicit hosted source-generation mode that regenerates the affected translation projections without starting native builds or publication.
- Keep measured full-solver/filesystem scenarios on main and retain fast ownership, operand, and header contracts on pull requests; preserve all integration bodies and the existing execution budget.
- Report actual per-file completion and duration from Node 22+ test summaries.

Fix exact literal content markers and preserve their final newline through Agent creation. Bind scoped AddFile creation to actual qualified preflight, mutation and fresh readback evidence; report unavailable capabilities without claiming unexecuted effects. Keep JavaScript, TypeScript and Rust contracts paired.

Refresh the generated progressive requirements plan and include its lightweight regression in fast PR checks after the stale projection failed on main.

Restore checks-only bot dispatch when a workflow declares additional explicit modes, and align native CI contracts with the isolated source-generation mode. Keep publication and source-generation opt-ins independent from ordinary checks.

Preserve separate proofs for disjoint literal files explicitly declared by the same standing request, and retain original failed Write receipts so transport/precondition failures remain terminal failures. Unknown, pending and same-target effects still invalidate proof.

Retire the unused legacy post-write command receipt adapter after creation proof moved to scoped mutation/readback evidence. Remove the matching JavaScript wrapper and regenerate TypeScript. Preserve the active receipt type and read-only verification helpers, restoring warning-clean native compilation without suppressing the lint.

Verify declared literal-file Agent scenarios through actual bound writes, fresh reads, terminal completion and exact sandbox bytes. Retain minimum-round checks for other scenarios and compilation/execution requirements for generated programs. Add offline protocol regressions for shorter successful creation and false-completion refusals.

Move native path regressions into the registered test tree and execute scoped creation library fixtures from the existing compiler output, with exact Cargo target and test-inventory checks.

Restore complete source edit clauses, initialize replay authority inside its source session, and recognize bound native literal-file observations while retaining exact program verification. Add bounded child failure diagnostics and regression coverage for truthful write failure reporting.

Preserve provider append contracts during replay and update current proof fixtures while keeping historical captures unchanged. Verify supported source edits with fresh Read evidence, retain matched edit failures and repeated-objective context, and correct literal payload boundaries. Keep translations and status projections current and cover captured failures in fast PR checks.

Reacquire a fresh jobs and concurrency-group observation pair after a bounded
release transport failure. Keep the shared deadline, exact caller ownership and
freshness checks, and refuse semantic errors before any retry.

# CI source acquisition

Acquire the complete documentation-policy input closure without materializing unrelated source and evidence. Preserve all existing checks and finite deadlines, and guard imported modules, the full workflow census and locked parser inputs with a focused JavaScript regression.

Layered JavaScript also avoids historical development-log hydration while retaining the real learning-report fixture and every existing suite. Test actual Git sparse selection without increasing execution budgets.

Give the main-only native producer a finite 25-minute allocation after observed source acquisition and successful compilation consumed its old twenty-minute clock before artifact uploads finished. Preserve complete tracked-source witnesses, all producer operations and PR budgets.

Keep version-base and changelog admission metadata-only, preserve every measured source file in fast checks, and bound original-Hive cleanup preparation checkout without changing trusted revisions or cleanup permissions. Capture the observed acquisition failures in focused offline regressions.

Reserve realistic acquisition phases inside unchanged fast-check and cleanup job clocks while retaining reviewed source inputs and all validation time.

Read the base manifest through source-verified GitData objects under one fifteen-second deadline using the existing Actions read token. Narrow Cargo-audit checkout while preserving Rust target discovery, ignore proofs and advisory warnings.

Retain the full live-link audit as a main-only opt-in and keep its unchanged
parser regressions mandatory with a small checkout. Record the policy in vision,
contributor guidance and requirements; scope the native budget test to its named
job rather than the first timeout in the workflow.

Retain full-history and full-tree secret scanning while filtering unrelated
historical blob downloads. Reserve a bounded scan allowance after checkout
so source acquisition cannot consume the entire secrets job budget.

Keep full physical native test-input verification while reading files in
bounded chunks and reporting aggregate phase progress. Reserve a finite
artifact-binding allowance without dropping any tracked input or smoke check.

Acquire only reviewed VS Code resolver and corpus-aggregate inputs, preserve
publication/aggregation policy, and include the corrected producer-budget and
physical-identity regressions in fast pull-request coverage.

Preserve the full real Coding Ladder behind explicit main-only opt-in while
keeping default consistency and mandatory native coding/reporting gates.
Narrow only the public-download controller checkout; all actual public
asset verification jobs remain unchanged.

Bound JavaScript and primary server-parity source acquisition separately from
their unchanged test workloads. Keep finite checkout and job deadlines, complete
parity inputs and every release operation. Replace the stale link-parser command
count with structural trigger and job checks, retaining the native regression.

Remove the obsolete parent-module import after extracting the structural link
parser policy test, so native test targets compile under warnings-as-errors.
Keep the child module's used import and every policy assertion unchanged.

Share source-classification initialization between live capture and strict replay without changing pinned rule data. Restore complete-source evidence in coding test providers and their Rust twins, retain exact timeout-answer and HTTP plan-journal coverage, and include the captured coding regressions in fast PR checks. Keep the physical two-capture Create integration on main while retaining its unit cases in PR qualification.

Regenerate runtime feature coverage from its authoritative sources and check the committed projection in fast PR tests. Add bounded phase timing diagnostics to native artifact binding while preserving physical source checks, smoke tests and the existing deadline.

Preserve raw Edit receipts for exact preimage matching and consume instruction-header colon separators without changing authored payload punctuation. Align native coding fixtures with complete-source Read evidence and accept actual fresh post-write Read proofs. Retain the original Read-only Set verification regression as explicit release-first debt, with active honest-gap and separate-Read coverage. Execute JavaScript CI suites in the duration planner's actual order instead of allowing Node CLI discovery to sort them alphabetically, retaining all tests and the existing budget. Avoid redundant identical-block research analysis while preserving history guards; retain the full solver integration on main and use explicit routing emulation in fast PRs. Report DRY language-coverage gaps and existing scanner debt in CI, and require periodic review for shared invariants.

Bind relative creation requirements to exact absolute tool paths under the host process workspace, preserving raw arguments, receipts and fresh verification requirements. Keep foreign roots, ambiguous aliases, traversal and unavailable host bindings refused. Interpret complete GitHub expression wrappers in layered workflow assertions without weakening path predicates or dependency checks.

## [0.355.29] - 2026-11-02

### Fixed
- Keep issue-specific Hive draft attempts outside release CI admission, while retaining strict source, publisher and commit-check requirements.

## [0.355.28] - 2026-11-01

Fix agent file-command routing for directory typos, literal search, bounded counts and line-limited reads; reject unsupported deletion and compound shell input. Preserve dedicated tool priority and source-repair ownership. Restore native build and captured main-CI regressions, and add a guarded source-derived named-import edit.

### Fixed
- Connect bounded repository observations to source-derived documentation and concrete-syntax interval edits, with guarded effects, fresh readback and explicit pending independent acceptance.
- Preserve literal replacement metacharacters in source-derived documentation, matching native string replacement behavior.
- Add opt-in default-branch work-item qualification with authenticated released runtimes, isolated GitHub transport, complete bounded process logs and truthful cleanup of owned test issues. General work-item coding and original multi-file integration documentation acceptance remain pending.

Restore CI gate citations and fast metadata validation; preserve local path discovery and client Read aliases. Retain bounded server parity failure diagnostics and support CI-only replay from an existing main binary. Move native regressions outside production source without deleting their coverage. Keep parity request sockets open until asynchronous HTTP responses finish.

Retain validated consumer-search diagnostics and report a terminal verification gap when an edit host lacks a Run tool, preventing an unrelated unchanged-file claim.

# Fixed

- Restore the relocated native test fixture path, align CI dependency and disk-policy checks with their actual shared owners, and keep compiler-only explanations out of user-facing prose detection.

# Fixed

- Keep quoted edit payloads from becoming documentation instructions, and align current native and CI policy regressions with the maintained controller and shared lint profile.

## [0.355.27] - 2026-10-31

### Fixed
- Keep the shared npm alias fixture compatible with strict main Clippy checks without changing its test cases or dependency-policy behavior.

### Fixed
- Apply the exact main rustfmt output to the shared npm alias fixture while preserving its strict Clippy-compatible behavior.

### Fixed
- Regenerate complete governed specification objects after package-manifest changes and check complete source parity during fast PR validation.

### Changed
- Restrict production compilation and deployments to main and authorized release consumers; use JavaScript for lightweight PR admission and one group of selected unit tests.
- Avoid redundant Cargo package compilation and make independent Docker cold builds, browser E2E and box-language integration explicit main-only qualifications.

### Fixed
- Derive the PR timeout bound without a stale manual counter and correctly check npm alias publishers and local package dependencies against the registry snapshot.
- Update the isolated matcher compatibility package and consumer locks to the verified upstream minimatch 11.0.3 release.

### Fixed
- Resolve patched matcher dependencies for desktop, extension and browser-test tooling while preserving legacy CommonJS and modern ESM APIs.
- Update main-only coverage, sparse-checkout and staged-release fixtures for the fast PR policy, preserve the vision motto placement, and run the captured regressions in fast PR checks.

## [0.355.26] - 2026-10-30

### Fixed

- Retry transient GitHub JSON reads within finite time, response and attempt
  bounds, and observe publication jobs in smaller exact-attempt pages.
- Reconcile ambiguous release writes for the same observed GitHub 502/503/504
  error class before another creation attempt, preserving exact tag/source checks.
- Defer the optional post-publication authoring canary after exhausted GitHub
  server errors, with explicit diagnostics and no authoring or self-development
  credit. Source, authorization and code failures remain failures.
- Request the selected author's required App permissions and prevent checkout's
  default credential from shadowing that token. Keep live authoring out of PR
  checks; replay captured publication and HTTP failures in fast JavaScript tests.

### Fixed

- Configure the selected author credential explicitly for Git before source
  fetches and writes. Verify that path in the Rust credential policy instead
  of requiring checkout's obsolete default-token header.

### Fixed

- Reuse CI's prepared browser assets instead of rebuilding them inside
  Playwright's server-start wait. Keep standalone preparation, bound startup
  to two minutes, and retain both server log streams for diagnosing failures.
- Recover transient installer connection failures with three bounded attempts
  before download, preserving certificate, grant and archive-integrity checks.

### Fixed

- Verify the seed-mirror policy against evaluated browser configuration and
  include it in fast PR checks, preserving both standalone preparation and
  CI's explicitly prepared asset path.

## [0.355.25] - 2026-10-30

### Fixed

- Request workflow-file permission for the dependency updater's scoped App
  token. With only the default token, retain complete workflow-changing updates
  as review patches, and dispatch checks for ordinary dependency PRs it can create.
- Preserve repository branch protections with immutable per-run update branches;
  retain review data rather than rewriting an existing branch or duplicating a PR.
- Mint optional write credentials after validation so long jobs do not exhaust
  their lifetime before publication; use the default token for registry reads.
- Bound recovery from a truncated GitHub release-create response. Authenticate
  the exact release tag and source before crediting an uncertain write, and
  require an authenticated absence before retrying, at most three times.

## [0.355.24] - 2026-10-30

### Fixed

- Skip public-download verification for an authenticated Desktop workflow that
  intentionally produced no release, without falling back to another release
  or treating skipped publication as successful verification.

## [0.355.23] - 2026-10-30

### Fixed

- Prevent the exact decorative crates.io version badge's transient HTTP 408
  from blocking CI, while continuing to check its crate destination and other
  links. Run real-HTTP timeout and missing-page regressions in the link workflow.

## [0.355.22] - 2026-10-30

### Fixed
- Avoid a redundant full worktree rewrite during native offline source import while retaining exact commit, tree, lock, bundle and native-input verification.
- Use four checkout workers and finite import budgets to accommodate Windows ARM source materialization without dropping source files or history.

## [0.355.21] - 2026-10-29

### Fixed
- Recognize pending, waiting and requested GitHub scheduling states in post-publication self-authoring, while preserving exact publisher, source, attempt and operation authentication.
- Bound self-authoring checkout duration and filter historical blobs while preserving history, current source and bot-branch credentials.

## [0.355.20] - 2026-10-29

### Fixed
- Resolve current Hive Mind source for live issue drafting, reuse the selected commit across jobs, and require its exact versioned runtime image without a stale-image fallback.
- Qualify shared runner diagnostics structurally while preserving solve inputs, record the actual source identity, and retain an issue response on source discovery failures.
- Use the latest published Hive Mind CLI in maintained integration checks; document runtime freshness and distinguish historical replay evidence.

### Fixed
- Keep Hive Mind's latest-version install and version logging on separate command lines, so the CI package-version guard does not misread a shell operator as an unversioned package.

## [0.355.19] - 2026-10-29

### Fixed
- Move remaining Mac ARM release and public-download jobs to supported macOS 15
  runners so the macOS 14 brownouts and November 2 retirement cannot stall releases.

## [0.355.18] - 2026-10-29

### Fixed
- Preserve exact documentation exclusions when projecting publication trigger
  paths, without granting negated patterns source-declaration authority.
- Keep original fetched-work-item safety assertions aligned with structured gap
  observations and expose the shared native path parser for parity verification.

### Fixed
- Treat unknown Markdown runtime inputs as code after documented paths are
  excluded, and test native change admission before CI uses its tier outputs.
- Cancel superseded read-only documentation checks on PRs while protecting main.
- Drop checkout push credentials and preserve native unsupported-work-item
  no-effect assertions with the structured gap protocol.
- Bind evidence reports to the caller's work-item URL and keep native regression
  tests in the integration-test tree, outside production source.

### Fixed
- Remove the redundant task clone exposed by the structured-gap regression update.
- Use native Unicode whitespace semantics when collecting source references in
  JavaScript, keeping the generated TypeScript and native parser aligned.
- Decode Git filenames as NUL-delimited literal paths so Unicode documentation
  filenames cannot accidentally admit an automatic release.

### Fixed
- Document actual command-stream, web-capture and web-search integration boundaries.
- Skip heavy automatic CI and automatic publication for documentation-only changes,
  including bot-dispatched checks and pending-changelog release admission.
- Collect bounded repository evidence and expose unfinished implementation and
  verification steps before stopping on unsupported repository work items.

## [0.355.17] - 2026-10-29

Fix desktop installers recursively including the private parent source tree and previous build outputs. Exclude the repository package from runtime dependencies and audit ASAR contents before installer creation. Add captured Windows NSIS failure regressions and hosted packager filter verification.

Refresh the reviewed desktop source fixtures after the installer-boundary fix. Add a cheap PR check for every governed fixture and native-source digest so stale embedded source is caught before main, while retaining complete-object and reader parity checks.

## [0.355.16] - 2026-10-28

Reuse the verified same-run source history for native, desktop and extension
release consumers, and advertise detached source history during tag authorization
so Git does not download it again. Preserve all byte, tag and ancestry checks.
Only stage packaging console evidence after packaging actually starts.

Keep native release workflow guards aligned with authenticated offline source
restoration, and run those complete custody checks in fast pull-request CI.
Read packaging evidence declarations structurally while preserving full sanitized
console content and hash checks.

## [0.355.15] - 2026-10-28

Prioritize Linux coding and question-answering checks on pull requests while retaining heavy production and integration suites for main or explicit opt-in. Disable live GitHub test effects by default, add captured-data offline regressions and guarded stale test-issue retirement, and prevent authenticated scheduled observations from invalidating release admission.

Repair full-profile workflow expectations after the fast pull-request transition, spell new filenames in full, and discover unique portable archives from authenticated publisher release catalogs while retaining checksum, permission and actual host-probe requirements. Detect the installed JVM runner's supported command grammar before exercising its original Hello World oracle.

Synchronize the packaged prerequisite installer with its canonical source and check all demanded mirrors in fast pull-request lint. Give fast lint explicit concurrency ownership. Retain visual replay export integrations behind an explicit opt-in with bounded test deadlines, while keeping ordinary readiness/PTY checks and adding cheap captured-startup unit coverage.

## [0.355.14] - 2026-10-28

### Fixed

- Store shared GitHub action release metadata once in refreshed dependency snapshots, preserving every sub-action and its immutable release identity while satisfying the existing notation duplication gate. Reject missing, chained, cyclic, conflicting and cross-repository release references.
- Temporarily make duplicate instrumented coverage opt-in while retaining all ordinary critical coding, reporting and release checks; stop artifact consumers after a failed producer and skip unnecessary hosted disk cleanup.
- Add read-only live dependency validation and avoid redundant feature-branch admission runs while retaining manual candidate checks and late main readiness checks.

- Fetch only the exact source-event recorder for the existing three-minute provenance job, avoiding full-repository checkout timeouts while preserving the original source coordinates and minimal payload.
- Point the Mistral model-card citation to its official model repository.
- Resolve symlinked recorder entry points so successful invocation always writes the minimal provenance artifact.

## Fixed

- Recognize Node spec-reporter test evidence in coding research using the same
  strict JavaScript/Rust acceptance rules and shared negative cases.
- Reuse verified same-run Git history for CLI packaging and bounded APT recovery
  for slim runtime and source-builder images, retaining integrity failures.
- Share the pinned official Node/npm payload between full and slim images,
  avoiding the slow distro npm install that left dpkg interrupted at its deadline.
- Copy sparse test fixtures across filesystems and require the new offline CLI
  source import in relocated-stage controls.

- Avoid fetching unrelated release history, keep generated native and TypeScript
  projections current, and use small committed source fixtures in workspace tests.
- Enable Rust panic backtraces and retain large failed-job logs; fix Node 24
  parser loading and test-report compatibility without weakening behavioral checks.

- Retain every desktop packaging attempt and process/log-pipe outcome even on
  failure; move ARM packaging off deprecated macOS 14 while retaining signing
  and both DMG/ZIP runtime checks.
- Keep Hive integration temporary work on the runner workspace so its unchanged
  disk admission measures the filesystem with capacity available.
- Retain complete standalone build attempts and runtime stderr before post-job
  processing; preserve build/log-pipe outcomes and upload diagnostics on failure.
- Delegate Buildx builder disposal to the declared disposable hosted VMs,
  avoiding an implicit post-hook that outlasted the ARM job after runtime passed.
- Retain complete redacted draft console, dialog and native-response evidence,
  explicit process/deadline outcomes, and read-only post-cancellation job logs.
- Refuse evidence-only draft diffs before dispatching the check matrix; useful
  source, configuration and documentation changes still require original
  behavioral acceptance before the task is considered solved.

## [0.355.13] - 2026-10-27

### Fixed

- Use Hive Mind's supported no-change exit compatibility only for verified
  repeat coding tasks, while retaining strict first-delivery failures, every
  original task assertion, and independent no-write/no-commit checks.
- Dispatch authenticated generated-task cleanup with the default Actions token
  beyond GitHub's workflow chain limit, with a finite terminal-owner wait and
  the existing immutable claims and protected-branch safeguards.

### Fixed

- Keep attachment metadata outside requested web targets and isolate OCR export tests from public-network availability.

- Use an explicit bounded issue-report submission, retaining complete reports after uncertain creation (#1438).
- Preserve contextual source edits without copying provider line numbers into files, and align the JavaScript agent-info reader with Rust (#1438).
- Answer practical needs and urgent follow-ups with relevant trusted-source guidance in English, Russian, Hindi, Chinese and Spanish, bounded cold discovery and targeted clarification when evidence is unavailable (#1395, #1396).

## [0.355.12] - 2026-10-26

Prioritize instrumented cold-research tests from observed runtime lower bounds,
keeping slow cases out of shared default-weight batches while retaining every
selected case, original assertion and coverage deadline.

Reuse the exact pinned translator's existing combined output/signature result
to avoid repeating fresh translation work, while retaining full projection,
compilation and snapshot checks under the original deadline.
Include its static import companion in independently captured source-network
release tooling, with historical-checkout and tamper regressions.

Run native coverage shards after a successful manually dispatched producer even
when its change detector is intentionally skipped, keeping missing-profile
refusals and failure/cancellation handling intact.

Audit original client tool histories from actual wire requests, recognize endpoint
query parameters, and bind emitted call identities including text custom tools.
Allow verified no-op continuations past the current viewer's complete Hive session
notices while retaining genuine review feedback and incomplete-receipt refusals.
Close owned fixture issues and pull requests without weakening repository-wide
branch deletion protection; explicitly verify preserved protected branch heads
and retain bounded operation diagnostics for failures.
Keep offline transcript replay within its original job deadline by checking out
only its harness and recorded data, with a separate bounded checkout step.

## [0.355.11] - 2026-10-26

### Fixed

- Cache immutable native tool-capability metadata and reuse it during tool classification. Add opt-in native planning phase timings and captured-request latency regressions.
- Match JavaScript's prediction-hit short circuit in native memory recording, avoiding a second full solve when no predictions exist. Trace dispatch, memory recording and dialog logging boundaries when explicitly enabled.
- Remove verified Hive bootstrap placeholders only after checking the exact issue, branch, replacement sources and observed content again before deletion.
- Read and authenticate complete task pull-request details before cleanup, preserving merged, changed or uncertain fixtures.

## [0.355.10] - 2026-10-25

### Fixed

- Ship pinned Claude, Codex and Playwright MCP in both container variants, initialize the official Claude native linker, and verify local MCP registration and npm availability.
- Check the actual Hive client prerequisites before transferring a task image or creating test issues.
- Keep proposal-only learning events outside release CI gating while retaining mandatory checks and publication verification.
- Report the blocking CI run and bounded, redacted stdout and stderr when a replay command fails.
- Preserve generated-task cleanup refusal details, recognize failed pre-effect admission without phantom tasks, and support a read-only default-token ownership audit.
- Capture ANSI-bearing job logs with modern and legacy GitHub CLIs without relaxing bounded reads, and fail incomplete audit pipelines.
- Let exact, complete source-probe callbacks advance coding recipes when clients omit shell status; preserve explicit failures and require fresh source verification after updates in Rust and JavaScript.
- Separate ready staged-workflow pull requests from hosted CI completion so default-token task delivery can reach independent workflow execution without claiming nonexistent green checks.

## [0.355.9] - 2026-10-25

### Fixed

- Verify released-server fallback recording using the actual linked assistant task inputs and outputs, with exact answer preservation across restart. Retain bounded failing-pass memory diagnostics and cover the verifier with the production JavaScript memory writer.
- Factor shared release-ledger fields into explicit defaults, preserving measurements, row overrides, legacy epochs and archive context without raising the notation limits.
- Check default-token push access with one bounded read-only Git probe when repository role metadata is not positive, and avoid publishing nonexistent solve evidence before a client claim. No additional automation credentials are required.

## [0.355.8] - 2026-10-24

### Fixed

- Run the original Hive three-client acceptance with the default GitHub Actions token using isolated repository branches and staged workflows; additional automation credentials remain optional.
- Finalize owned test issues, pull requests and branches after verification, including cancelled runs, with bounded writes and source/run ownership checks.

### Fixed

- Authenticate older Hive cleanup and delivery records using the policy paths declared in their original source revision, preserving ownership checks across policy updates.
- Cache failed immutable policy lookups so multiple historical records cannot repeat the same missing-source request.
- Transfer only the selected native self-coding executable and its original build receipt, preserving source, checksum and exact-test checks without downloading unrelated test binaries.
- Preserve actual HTTP status and redacted transport diagnostics for governed release reads, with one bounded recovery attempt inside the existing observation deadline and strict live lease checks.

### Fixed

- Wait for Codex's selected-model banner and rendered draft, then send Enter separately after quiet output, preventing loading-screen and paste-burst handling from leaving a test prompt unsubmitted.
- Replay the captured loading/ready terminal sequence with a mocked interactive client during ordinary PR validation.

## [0.355.7] - 2026-10-24

### Fixed
- Check external Hive task credential configuration before allocating builds and images, while retaining the exact repository access preflight before effects.
- Distinguish confirmed skipped effect gates from unresolved generated fixtures during owned cleanup. Missing or ambiguous custody and uncertain writes remain quarantined; a no-op grants no solve or delivery credit.
- Print a safe cleanup state summary even when the full outcome is redirected to an artifact.

## [0.355.6] - 2026-10-24

### Fixed

- Bound search-adapter installation in coverage jobs and preserve signed Ubuntu mirror recovery.
- Recheck queued generated-task cleanup claims after acquiring owner serialization.
- Resume cleanup from bounded authenticated history windows and avoid scans triggered by cancelled admissions.
- Refresh the governed Cargo source witness after the dependency update.
- Rotate release-ledger history into bounded, lossless parts while preserving source-size limits.
- Keep the new ledger archive root consistent with the existing notation naming policy.
- Bound desktop producer and consumer checkouts so a stalled network step fails promptly.

- Reuse evidence-based, bounded npm publication recovery in Agent CLI tests and container runtime installation.
- Preserve installer execution and exit status through symlinked temporary directories.
- Give public macOS app transfers bounded copy deadlines distinct from short metadata probes, with command timing diagnostics.

- Keep historical ledger assertions complete by reading every immutable archive part.

- Resolve every literal matrix job cap in deadline contracts and align Docker assertions with the shipped recovery layout.

- Refresh reviewed governed source inputs after ledger and manifest changes while preserving every assertion.

- Accept equivalent quoted workflow references in the native checkout contract while retaining parsed protocol-pinning checks.

- Fetch only protocol scripts during initial native/desktop/CLI/VS Code protocol capture and bound stalled Git transfers before the unchanged full source checkout.

- Run the complete JavaScript dependency audit before native compilation while retaining its later registered gate.

- Isolate sparse protocol Git checkouts from full source checkouts so missing promisor blobs cannot trigger an unprotected fetch during sparse-index removal.

### Security

- Update both committed Tokio resolutions to 1.53.3 for RUSTSEC-2026-0386.

- Update the compatible desktop and VS Code browser-tool chain to patched Zod 4.6.7 for GHSA-mrhx-c7xc-m3w5.

- Update body-parser to 2.4.0 with its required qs dependency and Electron to 44.9.3 for GHSA-27r3-jxmh-p828 and GHSA-jr5w-w6x2-84p9.

- Refresh the corresponding root Bun body-parser, qs and content-type resolutions and validate all six committed JavaScript lockfiles with the maintained audit gate.

- Patch the existing fast-uri and undici major lines in desktop, VS Code and browser-test locks for the additional URI and HTTP-client advisories.

- Keep the canonical staged workflow and generated caller synchronized with the early dependency audit.

- Temporarily select the compatible older VS Code publisher authentication chain and scope the UUID security backport to its MSAL dependency, clearing the MSAL and UUID advisories without changing production packages.

- Make the UUID override compatible with npm 10 and install the frozen VSIX dependency graph without suppressing deprecation diagnostics.

## [0.355.5] - 2026-10-23

fix: Recover transient browser worker asset imports with a shared two-attempt limit, report persistent startup failures instead of leaving chat loading indefinitely, and retain worker/network diagnostics when the deployed Pages smoke fails.

## [0.355.4] - 2026-10-23

fix: Include the matching bundled npm and npx in the full Node runtime image so derived Hive images can install their locked dependencies. Verify both commands as the ordinary image user and retain an offline regression for the observed missing-npm failure.

## [0.355.3] - 2026-10-23

### Fixed
- Keep browser release, download verification and end-to-end dependency installation independent of unrelated runner APT repositories. Use the unchanged signed Ubuntu source definitions for that command, preserve other repositories and signature verification, and keep browser downloads under the original user.

## [0.355.2] - 2026-10-22

Temporarily make real Lean/Rocq CI integration opt-in through
`FORMAL_AI_REAL_PROVER_INTEGRATION=true` after both official Ubuntu mirrors
exceeded its bounded cold-install budget on main. Preserve the original
compiler tests and deadlines, avoid optional APT recommendations when enabled,
and record restoration conditions as release debt. Production, coding and
released-client checks remain required.

Bound recovery from a clean, empty GitHub REST read failure to one unchanged
retry under the existing admission deadline and request budget. Preserve
fail-closed source authentication, refuse retries for known denials, partial
responses, process uncertainty and GraphQL, and report private-safe request
identities, attempts and failure digests.

Recognize parenthetical GitHub CLI HTTP statuses so known access denials remain
unretried. Reuse the validated official Ubuntu HTTPS mirror transform before
Playwright installs browser system dependencies, retaining signed catalogs,
the existing setup deadline and every browser smoke oracle. Missing browser
publication evidence continues to fail installer checksum publication.

Follow-up: [PR 1394](https://github.com/link-assistant/formal-ai/pull/1394).

## [0.355.1] - 2026-10-22

### Fixed

- Bound repeated agent actions across client retries and server restarts, including unknown errors, missing results, repeated call IDs and equivalent JSON arguments.
- Preserve Codex tool namespaces and structured MCP failures; bind search queries to advertised input fields and refuse invalid or unbound arguments before execution.
- Recover local Git refspec errors from the checked-out branch, stop failed search repair phases, and bound persistent polling, dreaming, interrupted reads, TCP acceptance/dispatch failures and builder download retry configuration.
- Share a bounded exact-input normalization cache across native requests; expose real log-sink IO failures once through diagnostics and health, and preserve original memory storage errors.
- Prevent quoted issue reproductions from becoming unrelated command-output tasks and false source-delivery claims.
- Collect and deduplicate bounded, redacted CI failures across Formal AI and Hive Mind, distinguishing recovered upload errors from no-change outcomes.
- Repair dispatched evidence base refs, authenticate native projection source selection, and bound artifact upload recovery at the observed failing fixture step.
- Keep large planner diagnostics readable under existing record limits, and preserve strict nested schema projection and request-grounded command descriptions.
- Update patched TLS trust dependencies for RUSTSEC-2026-0382 and RUSTSEC-2026-0381 without suppressing the security audit.
- Supervise remaining native and JavaScript subprocesses through shared deadlines, bounded binary capture and finite cleanup; keep original statuses and byte evidence, including Formal AI's own grep, shell and exclusive-create adapters.
- Recognize both maintained Babel dynamic-import AST representations in subprocess boundary checks, retaining alias detection and the strict finite startup policy.
- Derive canonical named-property callback factories from source syntax, enabling a maintained JavaScript self-edit with preserved comments, unchanged consumer tests and complete readable before/after CST evidence.
- Share Docker CLI compatibility pins and finite installation checks across full and slim images, avoiding unpublished transitive SDK versions while retaining the original runtime probes.
- Cache immutable read-mode grammar compilation and bound exact Unicode lookaround, retaining independent fresh-request quality checks and hosted cold/repeated large-request timing.
- Reuse matching installed Node headers for macOS host dependencies without overriding configured Electron targets, and repair the exact observed lint and publisher-snapshot findings.

## [0.355.0] - 2026-10-19

### Added

- Connect host-owned JavaScript and TypeScript syntax validation to retrieved-source agentic repairs, requiring an exact successful repair write before retry and reporting rejected rendering without unchanged retries.

- Formalize typed questions, answers and exact grouped quantities; preserve applicant questions without routing them to assistant identity or capabilities.
- Generate shared shell quoting and invitation composition through Links IR, with reproducible Rust and TypeScript projections.
- Audit source duplication with verified generation identities, retaining unresolved independent clones and coverage gaps as unmet acceptance.
- Resolve immutable action release commits from typed official metadata and refresh current unblocked web dependencies.

### Added

- Continue failed module synthesis through bounded source research, formalize captured pages, and rebind supported pure declaration identities into syntax-validated proposals whose independent verification still uses the existing execution and repair gates.

### Added

- Add readable recursive JavaScript, TypeScript and Rust source-content operations through the pinned shared meta-language grammar, with exact source reconstruction, structural edits, inverse checks and independently reparsed CI fixtures.
- Add source-owned documentary comment propositions with compositional English naturalization and independently checked whole-and-part round trips. Unsupported meanings remain explicit capability needs; complete workspace migration, general semantic translation and multilingual documentation verification remain unfinished.

### Fixed

- Preserve soft-wrapped Markdown paragraphs and list items during requirement extraction, recognize standalone emphasized section headings, and keep emphasized obligations as requirements. Shared JavaScript/native controls retain the original corpus and quality floors.

### Added

- Add bounded native provenance source snapshots and inclusion proofs using the same canonical JavaScript policy compiled through checked meta-language IR into Rust and TypeScript. CPU/automatic backends use native BLAKE3; unavailable GPU requests fail explicitly. This is source-integrity evidence, with native hosted acceptance still required.

### Fixed

- Continue missing-command recovery before the generic tool-result fallback when the current request has an exact bound failure receipt and an explicit caller workspace grant. Preserve existing recipe ownership and refuse stale, unrelated or altered recovery transactions.

### Added

- Add native lossless readable recursive source models for JavaScript, TypeScript and Rust, using shared grammar rules, source-free reconstruction and independent reparse validation. Registered native controls require hosted CI acceptance.

### Fixed

- Preserve observed tool exit status for authorized prerequisite recovery and keep explicitly requested workflow paths in fetched coding tasks.
- Reduce canonical Links codec allocation and scanning overhead without changing serialized bytes; add optional provenance verification phases without request, environment, path or source payloads.
- Retain source ownership as readable metadata around complete recursive source content.

### Fixed

- Preserve independent coding obligations when repeated output literals merge, including requested CI workflows in fetched work items. Shared terminal vocabulary keeps prose continuations bound to the observed command.

Use full readable recursive CST content as the primary JavaScript, TypeScript and Rust source-distribution artifact. Preserve comments, punctuation and whitespace, restore from persisted content alone, verify actual grammar acceptance and reject missing or substituted release members. Provision the small frozen grammar dependency closure only in hosted CI.

Resolve GitHub-hosted toolchains through their declared publisher metadata, including the actual Scala 3 compiler required by the discovered source-run command. Preserve verified progress for an identical standing request and open a fresh evidence window for a changed request. Keep unfinished issue acceptance explicitly open instead of making false automatic-closing claims.

Recognize the real Agent client's canonical absolute workspace paths in the independent output-project verifier while retaining confinement, original argument and byte identities, symlink refusals and exact independent program-output checks.

Include operational scripts, package code and the Rust build recipe in complete readable CST bundles. Preserve CommonJS and CTS syntax, normalize the Rust-script Cargo manifest to its documented crate-comment form, and remove unused SDK checkout/installation from source packet production. Keep complete tracked-source and dirty-source verification under the same unchanged shard budgets.

Use the complete readable CST grammar for ordinary JavaScript self-inspection without an optional SDK. Keep explicitly configured SDK verification and source ownership refusals, and qualify real chat-planner write/read effects and exact source reconstruction. Preserve the separate generic driver’s refusal of unowned generated-document creation.

Resolve explicit source fields through their declared owner context before weak lexical surface guesses. Preserve ambiguous, quoted, wrong-context and undeclared-field refusals across JavaScript, TypeScript and Rust. Re-enable the original native unscoped owner regression while preserving the historical scoped conflict.
Share canonical bundled source inputs and lexicon priority, retain observed declaration conflicts through generated fallback, and separate issue-disposition examples from status claims.

Use optional read credentials only for HTTPS GitHub release metadata during workspace prerequisite discovery. Do not forward credentials through redirects or attach them to documentation and archive downloads. Give hosted live recovery the existing read-only runner token so it avoids the shared unauthenticated API limit.

### Fixed
Keep read credentials scoped to trusted compiler metadata discovery during independent Hive delivery verification, exclude them from delivered program execution, and preserve both bounded diagnostic streams on failure.

### Fixed
Preserve anchored publication trigger filters and automatically project every explicitly declared source path without changing unrelated workflow content or granting filters source authority.

### Fixed
Write canonical self-cst.lino artifacts across Rust, JavaScript and TypeScript, verify complete source content and fresh grammar independently, and recognize self-CST intent without relying on a census phrase. Preserve comments, whitespace, source ownership and original transport limits; keep historical census APIs diagnostic.

The JavaScript dependency audit regressions now derive their expected inventory from every committed lockfile, so new source-runtime packages retain the same mandatory retry, advisory and fail-closed checks without manual totals. Self-CST ownership uses whole words to reject unrelated source requests, and the literal-predicate debt ceiling is lowered to the actual improved measurement.

New explicit source reads after a final answer require fresh bound observations in JavaScript, TypeScript and Rust, while unfinished identical reads retain their existing observation. They cannot replay a previous turn's source contents; standing coding objectives still retain verified write and commit progress. The original native determinism oracle remains unchanged.
The native self-inspection routing regression now requires canonical CST wording and independently verifies full source ownership, content and grammar instead of the superseded CST/AST prose label.

The browser-engine and source-network release resolvers now check out only their standalone resolver module and bound stalled Git HTTP transfers. This avoids fetching unrelated repository blobs and lets checkout retries finish within the unchanged five-minute job limit. Standalone CLI controls retain nonpublishing behavior for pull-request and dispatch checks.

Bound stalled change-detection checkout transfers and materialize only the classifier, installer and changelog inputs, while preserving complete Git history, native classification and existing CI deadlines.

Declare the three CommonJS provenance launchers in the explicit unmeasured-source inventory. Existing custody tests execute their byte-identical private `.cjs` copies; canonical-path browser coverage is not claimed. Preserve all coverage thresholds and tests. Coverage debt remains to aggregate verified copy identities into canonical source coverage.

Reuse the complete progress window before classifying a source-read intent. Ordinary coding steps avoid repeated read/write classification; fresh reads after final answers and standing coding progress retain their existing evidence rules. Preserve all original safety properties, native/coverage deadlines and test inventories.

Keep compiler-cache storage outages from blocking native releases. Probe sccache with compiler metadata before building and select the real compiler when the cache is unavailable; Desktop reuses this shared setup without reinstalling its pinned cache. Preserve actual compiler exit codes, target/version pins, build verification and job deadlines. Bound diagnostics and redact signed cache URLs.

## [0.354.1] - 2026-10-17

### Fixed

- Verify fresh anonymous public release downloads with exact byte counts, SHA-256 digests and complete advertised checksum coverage; include the independently published browser engine in the signed consolidated manifest.
- Cold-install the publicly downloaded browser package and verify real worker behavior, and test downloaded macOS DMG and ZIP packages using the existing account-free self-signing and actual renderer startup.
- Preserve bounded CI-only verification and retain text diagnostics without downloading or executing release binaries locally.
- Build the generated renderer once from the selected Linux source and authenticate its exact handoff to Desktop and VSIX packaging before resource preparation.

## [0.354.0] - 2026-10-17

### Fixed

- Store default derivation records outside the caller repository and let explain select an immutable trace with answer-id@record-digest. Explicit-root storage APIs remain available. JavaScript qualification passed; hosted native qualification and other cache purposes remain pending. Record hashes provide integrity without authorship or full provenance credit.

### Added

- Emit typed Links Notation diagnostics from the JavaScript server with redaction and a private generated debugger-credential handoff. POSIX host compatibility is measured; native logging and packaged/Windows qualification remain pending.
- Add tested CPU unkeyed BLAKE3/XOF and caller-key Ed25519 session-integrity primitives. Engine hooks, durable provenance, replay, key identity and autonomous authorship are not claimed.
- Make prospective self-development presence policy explicitly Report or Enforce. Report preserves actual counts and unmet-floor reporting; strict attribution validation, historical measurements and independent correctness checks remain in force.

### Fixed

- Regenerate the TypeScript final-result worker through the maintained JavaScript-to-TypeScript translator and verify the complete translation tree.
- Create independent client and case evidence directories, keep browser-worker factories current within reviewed budgets, and exercise localized interactive reporting consent.
- Decode typed report traces, legacy and typed TUI dialogs, and coding-ladder refusal evidence while preserving their original assertions and registering the focused reader controls in CI.
- Refresh genuine native census data and preserve source-owned test registration, typed client evidence and portable issue-draft deadlines through the coordinated CI repair batch.
- Preserve large and deeply nested typed dialog records, read existing conversations while silent, and keep machine control summaries independent of diagnostic filters.
- Retain structured issue receipts and truthful Gap outcomes, share one capacity probe per setup batch, and bound issue drafting while preserving failure evidence.
- Preserve original authored question-policy bytes and align typed-log consumers and agentic reporting expectations with their original semantic assertions.
- Supply a target-native official Node 22 runtime in the prepared Docker image so agent dependencies using Unicode-set regular expressions install successfully.
- Preserve fetched issue ownership across ordinary client prompts and keep source-cache captures out of caller repositories.

- Correct native diagnostic digest formatting and serialization error propagation, preserve the literal-predicate ceiling through declarative rules and initialize issue-workflow logs in an allowed GitHub context.
- Retain failed native-producer artifacts and expose complete captured compiler diagnostics in CI logs.

### Added

- Typed Links Notation dialog persistence with legacy readers, shared redaction, semantic terminal records and JavaScript log conversion.
- Bounded per-language source snapshots and driver-scoped provenance/evidence verification, explicitly separate from full replay or autonomous authorship.
- Human-opened-issue drafting through pinned Hive Mind scripts with current-main CI builds and complete failure-evidence links.
- Exact project motto surfaces and a mutation-tested documentation gate.

Full native, real-client and production acceptance remains tracked in PR #1221; source checks do not establish those outcomes.

## [0.353.8] - 2026-10-16

### Fixed

- Consolidate shared Electron updater metadata checksums from the actual published bytes, rejecting conflicting ordinary artifacts and normalizing Windows binary-mode checksum records. Extension transfer now checks its own unique checksum independently while preserving exact signed package, source and producer identity checks.

Move optional Docker build cache export after immutable pull-back, version, executable hash, privileged runtime, and anonymous registry verification. Give cache-only export a hard 90-second deadline so a slow cache service cannot consume the release verification budget. Required publication failures still refuse release receipts.

## [0.353.7] - 2026-10-16

### Fixed

- Use GitHub API media negotiation for Actions artifact ZIP downloads while retaining octet-stream downloads for release assets. Extension publication now reports bounded, redacted GitHub HTTP errors instead of an unexplained status assertion.

Pull request: https://github.com/link-assistant/formal-ai/pull/1228

## [0.353.6] - 2026-10-16

### Fixed

- Complete shallow protocol checkout history before authorizing an older published release, retaining exact fetched tag and Git ancestry validation.

## [0.353.5] - 2026-10-16

### Fixed

- Independently authorize stable container release sources in a preceding workflow step so imported sources receive the verified commits, preserving strict tag, source, and ancestry checks.

### Fixed

- Bound the default live Pages check to a serial native-engine smoke to avoid public CDN rate limits, retaining the full functional suite and recording restoration debt.

### Fixed

- Preserve hidden release advertisement protocol files and distinct macOS signing observations across single or multiple artifact downloads, retaining strict source and signing validation.

## [0.353.4] - 2026-10-15

### Fixed

- Use the canonical repository API endpoint and accept queued aggregate workflow status while retaining exact source and completed publisher checks.
- Declare the worker WASM call provider before solver startup through generated prerequisites.

### Changed

- Keep unmet prospective self-development aspirations visible in report mode while enforcing strict attribution validation and measured zero credit.

## [0.353.3] - 2026-10-15

Restore only the generated image publication stage to its retained logical source allowance of 90 minutes. The former blanket 30-minute stage cap cancelled Docker Hub mirroring after successful full and slim GHCR checks, leaving the immutable image transfer unsealed. Keep every publication operation, source/digest/runtime/anonymous check and per-operation limit; all other physical stages retain 30-minute caps. Hosted completion still requires a fresh production run. Supplied orchestration receives zero Formal AI authorship credit.

## [0.353.2] - 2026-10-13

Publish prepared Docker images directly to the registry instead of exporting a full local image before pushing. Stream plain BuildKit progress and bounded operation timings while retaining source, executable, digest, runtime and anonymous-access verification. Existing cache settings and execution budgets remain unchanged.

The previously published v0.353.1 tag and crate remain unchanged; its Docker and GitHub release targets were blocked by a publication timeout. This patch selects a new release version, normally 0.353.2. Zero Formal AI authorship is credited for supplied orchestration.

Scoped temporary authoring debt: only exact open PR1200 on fix/release-docker-export receives DeferredForReleaseRecovery_PR1200 with qualifyingAuthorship=0 and floorSatisfied=false. Strict evidence validation and ordinary main/schedule/dispatch/closed-context gates remain unchanged; restore the original positive authoring obligations under PR1200-release-recovery-authoring.

Use lossless long-distance compression for the complete instrumented executable archive, with bounded worker jobs and matching decoder history limits. Keep every test harness, runtime executable, coverage report object, test selection, assertion, job deadline and coverage ratchet.

CI transfer delays of 15–18 minutes exhausted the 30-minute shard job clock; one cancelled attempt had already passed all 395 selected tests in 699 seconds of its 1,200-second execution budget. Archive size and hosted timing improvements still require CI measurement. Supplied orchestration receives zero Formal AI authorship credit.

Refresh instrumented scheduling priorities only from uniquely matched long-running warnings and successful completions. Keep these values as observed lower bounds, with per-row source/run/job/log provenance; preserve native timings and never divide pooled batch time among its cases. Print exact batch names when dispatch starts so deadline-interrupted work is identifiable even when no completion trace is emitted. Every test, assertion, execution deadline, job deadline and coverage floor remains intact. Hosted scheduling completion still requires fresh CI.

Distribute the complete native instrumented inventory across 32 coverage shards after a fresh run again exhausted its unchanged 1,200-second execution budget while an integration batch remained incomplete. Keep the 30-minute job cap, all test assertions, coverage floors, archive members, browser shard count and observed scheduling weights. More runner jobs and archive downloads are the tradeoff; hosted completion and transfer costs require fresh CI. Supplied orchestration receives zero Formal AI authorship credit.

## [0.353.1] - 2026-10-13

The layered JavaScript-to-Rust check now overlaps full retranslation with compilation of the committed projections. Both checks must succeed, freshly translated projections must remain byte-equal, and failures cancel the remaining process groups. The original module set, equivalence assertions, and 540-second execution budget are preserved.

### Changed
- Temporarily defer only PR1196's positive authorship requirements for release recovery, with explicit zero credit and named restoration debt. Strict evidence validation, measurement, debt checks and the standing main/daily floor remain active.

### Security

- Upgrade meta-language to 1.0.0 and its maintained Lindera dependency, removing the unmaintained unsafe-libyaml dependency while preserving the default language engine and CJK segmentation.

Keep exact Cargo manifest and lockfile bytes when validating the release version child, so trailing LF or CRLF line endings match the governed source witness. Retain strict source and dependency checks and cover the complete physical Git transition.

Temporarily defer only the two positive-authorship gates for exact open release-repair PR1197, with zero credit and explicit restoration debt; retain strict evidence measurement and all standing main gates.
