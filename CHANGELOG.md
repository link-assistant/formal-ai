# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

Older releases are archived under `docs/changelog/` so that no file
exceeds the repository's 1500-line cap (newest first):

- [0.355.0 to 0.356.0](docs/changelog/releases-from-0.355.0.md)
- [0.353.0 to 0.354.1](docs/changelog/releases-from-0.353.0.md)
- [0.352.0 to 0.352.1](docs/changelog/releases-from-0.352.0.md)
- [0.346.0 to 0.351.0](docs/changelog/releases-from-0.346.0.md)
- [0.307.0 to 0.345.0](docs/changelog/releases-from-0.307.0.md)
- [0.242.0 to 0.306.1](docs/changelog/releases-from-0.242.0.md)
- [0.183.0 to 0.241.0](docs/changelog/releases-from-0.183.0.md)
- [0.177.0 to 0.182.0](docs/changelog/releases-from-0.177.0.md)
- [0.105.0 to 0.175.0](docs/changelog/releases-from-0.105.0.md)
- [0.1.0 to 0.104.0](docs/changelog/releases-from-0.1.0.md)

<!-- changelog-insert-here -->

## [0.357.5] - 2026-11-28

### Added

- Extract verbatim source quotes for structured OpenAI fields and per-criterion evidence arrays in English and Russian. Text requests can also quote the sentence that mentions a target.

## [0.357.4] - 2026-11-28

### Added

- Derive English and Russian skill and minimum-duration verdicts in OpenAI structured responses, with the rule and quoted source evidence in the thinking step.

## [0.357.3] - 2026-11-28

### Added

- Extract English and Russian string lists for OpenAI structured output, including bullet lists and item enums. Mark empty arrays inferred from missing evidence in the structured derivation. (#1619)

## [0.357.2] - 2026-11-25

### Fixed
- CI/CD false negatives, false positives, warnings and errors found in the issue #1639 audit:
  - a rustfmt diff and a `large_enum_variant` clippy error reached `main`, because formatting ran only on `main`; the pull request native lane now runs `cargo fmt --check` before clippy;
  - release preflight no longer reports the crates.io token as "unknown" on every `main` push; it verifies the token and crate scope with a no-archive publish probe that cannot publish anything;
  - the Agent CLI E2E harnesses set `FORMAL_AI_TRACE_REQUESTS=1` without `FORMAL_AI_LOG_LEVEL=trace`, so after the request trace moved to the trace level every evidence projection counted zero requests; every launch now raises the level, and a pull request test scans every harness;
  - the issue #707 generalization reference compared dotted tool names with serde variant names, so every held-out case failed on `main`; the comparison moved into a tested module, and a failing run now keeps its logs and plans;
  - a native capture case re-executed its body inside a replay child, so `main` accepted 27 of 28 capture cases; it was removed, and a pull request test now rejects such cases;
  - the Aider issue report matched menu choices only at the start of a line, and the one-line menu broke that;
  - a download-artifact pattern that matches exactly one artifact was extracted flat, so a one-leg Issue 1028 ladder joined as "0 wrote a result"; the shared download action now keeps the artifact folder for every pattern selection;
  - three Benchmark Corpus Gate shards ran out of memory on the `near_pair` prompt: the composition search applied an amplifying fragment to its own output up to depth 8; it no longer does, and draft verification is capped at 512 MiB, so a runaway fails with a named `MemoryError`;
  - "how many days are in 2 years?" was routed to a concept lookup instead of a unit conversion; a bare duration now stays a unit conversion, and catalog construction records its trace;
  - several main-only Rust tests read live publishers or a shared source cache; they now use isolated caches and deny the network where they assert an answer;
  - the prebuilt full and specification test shards ran default solvers against live publishers (read access is on by default since #1582), which held shards past their budget; they now run with `FORMAL_AI_NETWORK=denied`, and the shard durations are refreshed;
  - Russian duration prompts such as "сколько дней в 2 годах" fell through to the relative-period web search, and "2 years in days" was answered by the calculator with a 365-day year; a request the unit handler derives now stays with the unit handler, and a conversion through the Julian year is no longer claimed by arithmetic;
  - the intent-span and minimal-core tests now also run in the pull request native lane;
  - each of these main-only failures is listed in `data/meta/main-only-regressions.lino` with the pull request test that now catches it;
  - the evidence of every failing scheduled, dispatched and `main` run is kept under `dev/log/issues/1639/pulls/1640`, with a comparison against the eight pipeline templates.

## [0.357.1] - 2026-11-25

### Fixed
- CI/CD false negatives, false positives, warnings and errors found in the issue #1637 audit:
  - the `docker-build` and `docker-slim` jobs could never run, because their condition both refused and required `pull_request`; they now run on `main` after `build-artifacts`, and a contract test rejects any job guard that contradicts itself;
  - the scheduled `dependencies-latest` gate now passes: setup-node and setup-python move to v7, secretlint, the issue-1609 client CLIs and `@link-foundation/ui-links` are bumped, and the registry snapshot gains the records it was missing;
  - the semantic grounding graph is closed (missing Wikidata cache records were added and two dangling relations repointed);
  - Rust 1.100 clippy lint errors are cleared, along with unused rust-script dependencies;
  - ordered-list checks now treat Cargo `[[test]]` and `[[bin]]` targets as module roots;
  - native capture cases compiled into two Cargo targets are qualified with `@target`, so each case binds to exactly one caller;
  - GitHub Pages deployment sends the source commit SHA as `pages_build_version` (as `actions/deploy-pages@v5` does) instead of a synthetic digest the API rejected with 404, and a retry no longer adopts a finished deployment of another artifact;
  - the release client-report check expects `--permission-mode auto` for Claude and frees the report ports between clients;
  - the main run 45549993600 regressions are recorded in the main-only regression ledger with their pull-request replays;
  - corpus shards can trace each prompt and its duration (`FORMAL_AI_CORPUS_TRACE`, off by default); the CI shard job enables it and logs runner memory and disk, so a killed shard leaves evidence.
  - Rust/JS server parity (R1013) is restored: output projections keep one key order, JS reports carry `violations` and `source_origin`, and the method registry hash matches;
  - CI/CD contract tests that drifted on main (workflow, dependency-check, core-boundary ledger and traceability documents) agree with their contracts again;
  - procedural specification tests that build an offline solver replay in JavaScript, because the reader and worker host now carry an offline binding;
  - solver, routing and handler regressions from main run 45549993600 are fixed in Rust and in the JS/TS twins, with the seed mirrors kept byte-equal;
  - network-dependent integration cases (issue 1138 family behaviour, issue 712 Hindi routing) run with read access denied, so their answers no longer change with the runner's network;
  - the issue 1157 delivery test prints the failed answer and the commands it ran.
  - the secrets scan, its scripts and its tests pin the same locked secretlint version (14.1.0).

## [0.357.0] - 2026-11-25

### Added

- Claude Code runs in `--permission-mode auto` by default, and the server answers Claude Code's auto-mode security-monitor requests at once from the seed policy in `data/seed/auto-mode-classifier.lino`, so Bash calls are no longer refused with "temporarily unavailable (timed out)".
- Claude Code's `AskUserQuestion` receives `multiSelect: true` for the multi-choice report-destination question.

### Fixed

- "How to X?" questions always probe wikiHow, the priority how-to source, even after a cached transient failure.
- How-to citations name the human article page (for example `ru.wikihow.com/хорошо-покакать`) instead of the MediaWiki `api.php` capture URL.
- The web-search-unavailable reply no longer lists unused sources.
- Russian "Отправь отчёт об ошибке" and similar phrasings are recognised as a request to report an issue.

## [0.356.12] - 2026-11-24

Update every workflow artifact consumer to verified raw transport and streaming extraction, preserve caller Python and empty pattern selections, and retain transfer timing for local adapters.

### Fixed
- Download native and selected-source renderer artifacts with strict digest verification and streaming extraction, replacing the deprecated Node ZIP parser while preserving cached regular-file refreshes and refusing unsafe archive paths.

bump: patch

Honor Cargo feature activation when constructing standalone Rust CodeQL contexts, including renamed optional dependencies and weak feature forwarding.

bump: patch

Allow explicit read-only CodeQL producer verification on a branch while retaining the automatic main-only analysis and report upload.

### Fixed

- Reject unresolved CodeQL findings, invalid or missing SARIF reports, and extraction diagnostics instead of treating a successful upload as a clean analysis. Preserve the reports for review and refuse incomplete Rust sysroot prerequisites.

### Fixed

- Preserve consumed child evidence and complete route traces in native compound answers.
- Run the consumed-child provenance library regressions in the existing focused native CI step.

### Fixed

- Connect the calendar and currency HTTP regression to its captured-publisher worker through the server's supported injection argument. Assert the actual worker identity so rate checks cannot silently depend on live publisher availability. Existing answer and derivation checks remain unchanged.

### Fixed

- Run the complete JavaScript suite in twelve groups selected by fifteen hosted timing cohorts. Retain every test owner, timings from failed files, prior timing floors, the 270-second warning and the 360-second deadline.

### Fixed

- Require an explicit successful checker exit output before clearing the terminal link gate, so missing outputs cannot be coerced to numeric zero.

### Added

- Verify bounded link recovery against real early TLS and post-request connection resets, including recovery only after a later HTTP success.

bump: patch

Select maintained main directly during checkout and bind the actual commit before running Hive or Formal AI source, preserving immutable attempt and delivery identities.

bump: patch

Run the native Git-change detector regressions in the focused pull-request workflow target.

### Fixed

- Run the focused native PR check when its isolated regression tests, embedded
  fixtures, or HTTP replay inputs change. Replay the existing synthesis contracts
  in the same lane without compiling the complete unit test suite.

### Fixed

- Resolve the package-size archive from Cargo metadata so custom target directories and workspace members inspect the newly generated package, with errors preserved when metadata is unavailable.

Deploy the exact immutable Pages artifact with bounded REST retries and cancellation, without the deprecated bundled punycode dependency. Bind retry adoption to both artifact and stamped source commit.

Verify the pinned PHP template's automatic-release guard against failed,
cancelled and skipped quality checks, with successful-release and manual-release
controls in the maintained template observation suite.

### Fixed

- Locate repository definition answers from current Rust sources, so moved helpers and changed line spans cannot quote an unrelated documentation comment.
- Restore the native Link Store formatting import and avoid duplicate CodeQL boolean options in opt-in macro investigations.

bump: patch

Match CodeQL 2.27.4's Rust compiler ABI and preserve separate declared dependency contexts for standalone scripts and the direct-rustc WASM worker during security analysis. Keep branch producer verification read-only while allowing its status API reads.

Upgrade the search-only web-capture component to remove fxhash, retire its audit exception, and execute the existing native search component regressions in pull requests.

### Fixed

- Preserve concrete output contracts when the recursive procedure considers joining child answers, and retain the intent of validated prime-number candidates.
- Check only consumed child answers when evaluating source operands, and recognize full-width terminal punctuation in concept queries across runtimes.
- Correct CI regression fixtures for source observations, validation events, generated workflow retirement and supported pull-request handoffs.
- Bound CI downloads and duration collection, avoid overlapping Hive drafts, and distinguish image transfer time from solver time.
- Report every required pull-request context for docs-only changes while skipping unchanged Rust compilation.
- Report compiler-cache write errors without asserting unobserved future cache misses, including valid statistics JSON with whitespace.
- Keep required PR context reporters admitted by the documentation filter policy, with behavioral checks that documentation changes select no build or publication.
- Refresh reviewed governed source fingerprints without changing their cases or assertions, and remove the unused native candidate binding rejected by clippy.
- Fail change detection when Git cannot establish the selected complete comparison or write its outputs; preserve legitimate initial trees and successful empty differences.
- Retain complete template workflow documents and add executable controls for failed Git observations and Go manual-release result integrity; guard the exact-version Chrono upstream exception without claiming migration.

- Load workflow inventory YAML consistently across CommonJS environments, preserve large immutable evidence as compressed artifacts, and reject empty or unknown local gate selections.
- Record the complete Go template source comparison and reproduce its combined stale-version/pin-integrity bypass with independent failure and warning controls.
- Retain bounded opt-in compiler-cache backend diagnostics in hosted logs, including missing statistics, while preserving command outcomes and redacting signed URLs.
- Keep calendar meaning identifiers unique and expand calendar/currency aliases into scalar seed rows, preserving every original spelling and the existing duration-sum response intent.
- Replay supported health access modes and offline source-cache fixtures in the fast PR lane, with explicit source-network policy and current TypeScript calendar mirrors.

- Reduce fast source checkout hydration by excluding unmeasured historical archive payloads while preserving every measured source and existing checks.

- Retain seeded counting membership proofs alongside retrieved matches, and give the native connector regression an actual workspace source observation.
- Trace agent-mode documentation through the current CLI/configuration owners and freeze live network input in the interpretation determinism fixture.

- Preserve the complete localized unknown response when its language has no variation pool, while retaining the pool-query fallback API.
- Synchronize improved core source-size limits, generated status and the reviewed governed source witness without changing any complete case object.

- Keep localized opener preservation in the native formatter without invalidating the committed browser worker, and register its native regressions outside production source.
- Use the declared answer-language policy for unresolved JavaScript requests, retaining the full localized failure report and actual detected-language trace.
- Regenerate the maintained TypeScript opener and unresolved-language mirrors to restore projection checks.

- Preserve ASCII quoted connectives during recursive task decomposition, including escaped quotes, contractions and possessives, with shared JavaScript/native quotation fixtures.
- Preserve decoded quote characters in native structural grammar and vocabulary values, preventing the ASCII quotation pair from becoming empty during task solving.

- Keep nonempty unverified drafts unresolved in task leaf tests and parent composition, including cached drafts; retain verified program answers.
- Replay all shared executor fixtures, including unverified draft statuses, in the isolated native executor parity target on pull requests.
- Reduce JavaScript test checkout hydration while retaining consumed historical transcript, native-session and language-verification fixtures and the existing test budgets.
- Retain the selected children's evidence when composing answers, reusing cached drafts and adopting a composition; exclude rejected roots and unused alternatives.

- Keep seeded quoted-passage cues intact during task decomposition in both runtimes, while preserving surrounding independent tasks.
- Reuse existing calculation-question parsing for algebra substitution and replay original and renumbered source-operand compositions before merge.

### Fixed

- Reject failed or partial Git lockfile inventories before starting JavaScript dependency audits, preserving advisory failures, retry bounds and complete lockfile coverage.

### Fixed

- Preserve clock punctuation and minutes when recognizing scheduling requests and generating calendar event payloads in JavaScript and Rust.

### Fixed

- Keep execution budgets active after a command exits while its children still run. Preserve successful and failed command statuses once the process group finishes; expire lingering children, including TERM-resistant workers, and fail explicitly when group inspection is unavailable.

### Fixed

- Decode appended HTTP capture records individually and verify the transport and conversation schemas while preserving capture, privacy and ephemeral-memory assertions.
- Partition the live Hive regression inventory into four measured, deterministic groups so every original case can run within the existing job clocks. Keep exact test names and terminal failures, and admit changes to the planner and its inputs.

### Fixed

- Preserve the full execution budget and cooperative cleanup interval when a whole-second shell clock crosses a wall-clock boundary. Zero budgets and grace periods still expire immediately.

### Fixed

- Keep command deadlines active until every live process in the command group finishes, preserving root exit status after successful cleanup and failing explicitly if process-group inspection is unavailable.

### Fixed

- Check package versions on every pull request, including branches with release prefixes; manual changelog release branches continue to pass when their package version is unchanged.

### Fixed

- Replay Agent CLI workflow delivery against a real temporary Git workspace, retaining explicit compiler fixtures and reporting the language and planned calls when delivery fails.

Restore the full Claude Kotlin work-item regression, including prepared pull request delivery backed by observed Git state.

Reject colliding manual changelog fragment names atomically so an existing release entry cannot be overwritten.

Fail release helpers when a configured GitHub Actions output cannot be opened or written, preserving successful local invocation without an output file.

### Fixed

- Execute the full PR title and description update in the offline Hive delivery
  harness when its compound command also reads the issue title. Keep the original
  issue-closing assertion and exercise real Git delivery with simulated hosted CI.

### Fixed

- Read Rust release identity from the parsed Cargo package table. Reject missing
  or inherited versions, select workspace members from their actual tables, and
  update only the package version token while preserving unrelated manifest text.

### Fixed

- Retry a failed native-input checkout once within its existing total time
  allowance, and fail explicitly when neither checkout attempt succeeds.

Read the changelog release version from the parsed Cargo package table. Refuse missing or inherited package versions instead of using an earlier dependency version, with a physical manifest regression in the focused native workflow target.

Check native compile admission before evaluating checkout retry outcomes. Preserve the documentation-only notice path and verify that neither checkout attempt, its terminal guard nor compilation can enter that path.

Stop changelog collection on fragment inventory, UTF-8, existing changelog, write or cleanup errors. Bind deletion to the successfully read fragment snapshot, preserving later or changed fragments and refusing false completion.

Refuse version-bump selection when changelog directory discovery or a fragment read fails. Reuse the fallible shared inventory and admit the actual bump-selection filesystem regression into focused native CI.

Propagate changelog discovery, fragment, existing changelog and cleanup errors from version-and-commit. Share its actual collection operation with focused native CI while retaining canonical changelog formatting and exact fragment ownership.

Require successful registry evidence before release detection or version selection. Retry transient registry failures within bounded deadlines, reject malformed success responses, and replay both actual release CLIs against loopback registry failures in hosted native CI.

## CI correctness and compressed evidence

- Inspect bounded gzip/tar contents and metadata with the pinned secret scanner, retaining masked output, exact file selection and sequential resource use. Reject malformed archives and incomplete Git inventories; disable unnecessary API profiling as the upstream CLI does.
- Acquire the two tracked component-test logs in the fast JavaScript lane, refresh only reviewed transport/manifest witnesses, and retain native Pages artifact/source and timeout assertions against the owned deployment action.
- Preserve credential-removal assertions with a fixed diagnostic that does not interpolate credential-related identifiers.

Use Secretlint 14's built-in recommended preset without installing the deprecated
external preset package. Preserve masked plain and compressed secret findings.

Start execution-budget warnings, errors and completion messages on fresh lines
after partial child output, including NUL-terminated stdout.

### Fixed

- Reject stale worker inputs in local JavaScript gates without compiling Rust;
  remove an unused worker import that stopped warnings-as-errors compilation.
- Preserve consumed immutable fixtures in sparse CI checkouts and use the strict
  artifact extraction adapter for native contracts.
- Retain first-wave scheduling for measured heavy suites and transport both
  receipt-bound native companion binaries required by existing integration tests.
- Schedule all four complete Hive workflow sessions independently under the
  existing execution budgets, and retain real failures in the BuildKit outage
  control without false failure annotations after successful recovery.
- Select the maintained SQLite runtime explicitly and remove the experimental
  timer mock while preserving warning reporting and deadline assertions.

Partition the live canonical release workflow into two byte-bound source parts. Preserve all 52 operations and 104 bindings, authenticate each physical input, and keep legacy single-file fixtures compatible. Replay JavaScript budget checks against each complete hosted cohort instead of treating maxima from different runs as one observed workload.

Fix release receipt recognition after shell extraction and keep sparse-checkout
regression fixtures, moved Rust documentation owners and seed VM prerequisites
aligned with their maintained consumers (issue #1580).

### Changed

- Keep conversation list hydration and deletion in the existing hook module while preserving sidebar behavior and source headroom.

### Changed

- Extract desktop release checksum and evidence shell bodies into workflow-selected protocol helpers, retaining release-source isolation and producer receipts.

### Changed

- Extract complete native runtime, source-test and grammar-generator units into focused modules while preserving ordered rules and every carried specification case.

### Fixed

- Verify the governed native-test source witness during the issue replay, preserving every test identity and carried program after helper extraction.

Refresh JavaScript shard weights from complete hosted observations while retaining higher historical costs and the existing warning threshold.

### Fixed

- Preserve consumed transcripts in Layered sparse checkouts, refresh hosted
  JavaScript scheduling costs without losing higher historical measurements,
  and record queued files before synchronous runner events.
- Follow the compiled capability classification owner in native acceptance,
  split coding contracts using complete compiled listings and retained timing
  evidence, and bound native execution within the existing job clock.
- Preserve native test coverage while extracting five warning-sized owners;
  allow bounded signal delivery after process-monitor inspection failure.

### Changed

- Split complete whole-line operation implementations into a focused child module while preserving method bodies, comments, visibility and behavior.

Archive eight immutable historical source captures losslessly, preserving their
original paths and digests in a consumed conservation fixture.

### Fixed

- Keep nine native acceptance modules below the unchanged source-size warning threshold while retaining every original test name and assertion body.

### Fixed

- Keep rename metadata and release-ledger writes below the existing size-warning threshold without losing rules, historical records, defaults or legacy archive compatibility.

Compile the three focused native contract executables once and verify their same-run artifact before each existing contract group, preserving case coverage and budgets.

### Changed

- Extract complete runtime and source-test functions into focused modules, retaining comments, visibility and all carried specification cases while reducing source-size warnings.

### Changed

- Share complete seed record readers across browser, worker and Node hosts, preserving seed fields and failing missing prerequisites.

Keep every requirement traceability row while splitting the large source table
into indexed pages, with matching readers and complete evidence bindings. Register
the conserved conversation-list hook in the existing UI boundary inventory.

Allow explicit hosted browser worker receipt regeneration, verifying the actual dispatched source and compiler output before preserving the worker and input manifest. Ordinary pull request checks retain their build restrictions.

### Fixed
- Keep the changelog reconstruction regression independent of developer partial-clone history, using a four-commit local fixture that verifies imported, historically consumed and pending release fragments without remote access.

### Fixed

- Reuse one resolved version snapshot when a recipe attaches and validates its generated CI workflow. A live-to-cache transition within the same planner call no longer disables the owned compiler-failure CI fallback. Keep original provenance and exact source ownership checks; cross-request attempt retention remains unfinished.

Fix isolated native contract imports, shared publisher transport ownership and
the practical-help capture limit regression. Keep extractor tests under the test
tree, retain the consumed browser fixtures in checkout assertions, and isolate
Hive fixture source caches from concurrent test activity while preserving source
freshness and delivery checks.

### Fixed
- Preserve complete inline JavaScript bodies, distinguish program counts from numeric operands, and retain arithmetic event origins in native and browser traces.
- Preserve publisher routing and historical AST census evidence while repairing current CI source paths and stale generated inventories.

### Fixed
- Require the requested crate version in successful registry responses, reject incomplete release-fragment inventories before emitting job decisions, and preserve configured output paths across release tools.

Require successful JavaScript source checks on the exact clean event commit before allocating the focused pull request native jobs.

Keep native request phase tracing limited to metadata unless explicit payload custody is enabled, while retaining complete private dialog capture and wrapped-terminal isolation.

Retain targeted opt-in compiler-cache write failures before compiler argument logs can displace them, including failed native-check jobs.

Refuse unreadable workspace member manifests before choosing a release package.

File-size checks now fail when maintained text cannot be read or the scan cannot
complete. JavaScript and Rust preserve the same errors and existing exclusions,
and evaluate exclusions relative to the checkout so a parent named `target`
cannot hide oversized source files.

### Fixed

- Keep recovered artifact lookup attempts observable without a stale terminal error, reject missing requested artifact IDs, and refresh JavaScript shard planning from complete hosted observations without reducing measured weights or execution budgets.

### Fixed

- Select GitHub release notes with the same parser exercised by native tests, ignore version headings inside fenced examples, and fail on unreadable changelog sources instead of publishing fallback notes.

### Fixed

- Normalize Windows action paths before constructing Node preload options so verified artifact downloads can run on Windows while preserving spaces and retry diagnostics.

### Fixed

- Reject non-breaking spaces and vertical whitespace as closing Markdown fences so required documentation headings inside code examples cannot satisfy the documentation gate.

### Fixed

- List changelog decision paths without rename similarity fetches after sparse checkout credentials are removed, and recognize scripts in root and nested Rust repository layouts.

Fix integration cleanup by stopping Codex plugin-clone descendants before deleting their temporary home.

### Fixed

- Isolate historical recipe protocol fixtures from live and caller-owned version caches, and retain optional tool-call tracing for stale-source failures.

### Fixed

- Remove unused graphics dependencies from the unavailable provenance GPU compatibility feature and its dependency resolver. Keep the feature name and current runtime behavior.

### Fixed

- Require a newly added changelog fragment rather than accepting edits, deletions, unchanged moves or nested archive entries. Preserve exact path names through NUL-delimited Git inventories without partial-clone similarity fetches.

Preserve every tracked Cargo lockfile path during dependency audits, collect findings from all lockfiles, and reject incomplete inventories before invoking Cargo.

### Fixed

- Rebalance JavaScript CI shards using complete hosted file timings and an explicit scheduling reserve while preserving prior duration floors and the existing warning and hard limits.

### Fixed

- Use the existing bounded and version-verified Bun installer for native transport replays so a failed first download receives one retry while final installation failures remain fatal.

### Fixed

- Preserve receipt-bound JavaScript timing records and their source metadata when importing a larger ordinary cohort observation.

### Fixed

- Avoid copying complete source bodies while trimming short Unicode boundaries in the JavaScript planner; preserve character matching and predicate order.

### Fixed

- Refresh JavaScript CI scheduling from all 914 complete hosted file timings, retain prior floors and receipt identity, and preserve the existing warning, budget and provenance controls.

### Fixed

- Cover all active Cargo projects and tracked npm manifests in dependency maintenance. Reject incomplete inventories and unclassified manifests, preserve historical reproducers explicitly, and resolve each active Rust lockfile with its owning manifest.

### Fixed

- Retain all active dependency manifests in native admission checkouts. Parse npm dependency objects across compact and multiline JSON, and update only the selected dependency value while preserving surrounding content.

### Fixed

- Remove reported source-size warnings by extracting Rust helpers, report rendering and tests and moving worker helpers to existing modules. Preserve evidence index values, regenerate TypeScript and worker source identities, and run the source-size regression in fast PR CI.

bump: patch

Fix child module paths in focused native reasoning tests and retain the dependency finding renderer without an unused re-export. Preserve all extracted test cases and verify renderer output through the native workflow contracts.

Fix the governed specification witness after explicit extracted-module paths change source hashes. Preserve all 223 supported cases and 970 unsupported records; review the complete objects before refreshing the eight affected hashes.

### Fixed

- Keep historical CI reruns separate from current pull request checks, while allowing a newer commit to cancel superseded initial checks.

### Fixed
- Recover pending package configuration after a timed-out search-adapter install, and recognize official EC2 Ubuntu mirrors during the existing bounded fallback.

### Fixed
- Preserve complete Formal AI command output when source bytes contain the driver's answer marker, and qualify the exact command's terminal process receipt instead of printed exit-code claims.

fix: Pass publishing credentials to Cargo through its environment and preserve configured credential-provider fallback without a premature warning.

Probe Cargo sparse-index version identity before the crates.io API and retain unknown availability outcomes without expanding request retries.

### Fixed
- Prefer the official Ubuntu HTTPS archive before search-tool provisioning spends its first deadline on a regional HTTP mirror. Existing retry budgets, explicit opt-outs and signed source verification are preserved.

### Fixed
- Install the default secret scanner from a committed isolated dependency lock, with lifecycle scripts disabled, engine checks and temporary cleanup. Failed installation cannot certify a clean scan; literal filenames and explicit custom-version overrides are preserved.

### Fixed
- Preserve literal Git filenames and rename destinations when selecting size warnings. Incomplete growth observations report every warning, with optional diagnostics; existing limits and escaped annotation ownership remain.

### Fixed
- Validate the scanner's inconsistent-lock negative control with an empty cache instead of relying on cached registry metadata.

### Fixed
- Reject non-UTF-8 tracked dependency paths before resolving any lockfile, while preserving valid Unicode filenames.

### Fixed

- Refresh JavaScript shard scheduling from all 922 completed hosted file observations while preserving earlier scheduling floors and unchanged worker and execution budgets.

Reject malformed UTF-8 Git path inventories before CI admission emits outputs. Preserve valid BOM, newline and split multibyte paths with a fatal streaming decoder and real-Git regressions.

### Fixed

- Preserve an explicit unknown-test fallback when importing only known test durations or an empty observation set.

### Fixed

- Bound dialogue-owner scheduling allowances by existing provenance ordering, queue budgets and complete historical replays, preserving every prior cost floor.

### Fixed

- Run the meta-reasoner ladder and learning cases in two bounded isolated processes, preserving every original assertion and capture check.
- Extract historical plan text and scheduling comments to keep active files below the existing warning threshold.

### Fixed

- Detect Changes jobs in `release.yml`, `coverage.yml` and `layered-ci.yml` sparse-check out the `#[path]` modules of `detect-code-changes.rs`, so downstream CI is no longer silently skipped; the staged release generator emits the same list.
- A new `scripts/check-sparse-checkout-rust-modules.mjs` guard, run by the Workflows check, fails when a sparse-checked-out Rust script misses one of its `#[path]` modules (`FORMAL_AI_SPARSE_CHECK_VERBOSE=1` prints every checked block).
- The Hive Mind draft adapter inserts diagnostics at the closing brace of `setEnv`, so it accepts upstream keys appended after the shared spread.
- The Rust unit and integration test targets compile again: private test-fixture imports, a stale `SORT_PYTHON` assertion, the private `concepts` module (its test now lives in `rust/tests/source/concept_query_tests.rs`, keeping `lib.rs` and the governed specification witness stable) and the undeclared `tempfile` crate are fixed.

Fix Windows and other artifact consumers failing on a lookup immediately after a successful upload. Retry the pinned raw download at most three times under existing caller deadlines, discard partial archives before each retry, and preserve strict digest validation, cancellation and extraction failures.

Retain review-gated benchmark failure reports when a score fails the ledger
ratchet.

Check the packed browser engine's existing 128 MiB archive budget before
artifact upload or npm publication, and report the largest files when the
archive exceeds the limit.

Pin every BuildKit bootstrap to one verified multi-platform image digest, recover registry outages by retrying actual builder setup through the mirror, and verify real daemon and image identity in the hosted workflow checks.

Regenerate the cache-request cue role registry from its meaning source and add the matching JavaScript local gate and committed-projection regression.

Fix canonical seed parsing for Gemini global token overrides and CNL reference lists. Preserve grammar forms, template styles and output policy behavior in both runtimes.

Keep capability routing below the existing source warning threshold by extracting advertised-tool classification into an explicitly located module, preserving all original function bodies and parent entry points with scanner and module-path regression coverage.

Correct five JavaScript twin citations after capability classification extraction and verify their actual Rust targets with a failing-before, passing-after regression.

Keep the native unknown-source positive control independent of transient runtime caches. Replay both bare-term and question requests from the existing committed isogram capture, preserving the complete definition, citation and digest checks.

Split the controlled-dialogue language cases across three maintained JavaScript
test owners, preserving every original assertion and fixture while preventing
one sequential five-language file from dominating a CI shard.

Resolve the read-only CodeQL macro observer's memory policy against its actual
Rust dataset before execution. Keep the 4 GiB total budget while assigning heap
through the child evaluator's resolver, and fail visibly on runtime diagnostics
that otherwise survive a successful query exit.

### Fixed

- Refuse a newly staged primary Rust lockfile that is absent from the HEAD
  commit before invoking Cargo; retain independent lock audits and strict
  advisory proofs. Exercise the actual wrapper with finite temporary Git
  repositories in the full hosted JavaScript suite and local feedback inventory.

Fixed native fallback cache lookups treating an arbitrary “about” clause as a definition request. Cached definitions now cover the complete requested subject, and link-memory focus keeps operative requests intact. Explicit cache requests and bare known terms retain their attributed answers.

Keep the computer-use executor below its source-size warning threshold by moving its unchanged CSV/HTML fixture helpers into a private module. Preserve exact source reconstruction evidence and existing permission, workspace and verification behavior.

Move unchanged unknown-reasoning text and memory-fact helpers into a private module, preserving the original native test module path and source reconstruction.

Keep lexeme importing below its source-size warning threshold by separating unchanged shard installation and stale-file removal, with the existing public function reexported at its original path.

Separate the unchanged Telegram Markdown renderer into a private module, preserving the existing public entry point and HTML escaping while removing the source-size warning.

Separate unchanged quotation framing from tool-result normalization to remove its source-size warning while preserving status and citation interpretation.

Separate unchanged file-end insertion and compact anchor selection into a private module to remove the computed workspace change source-size warning.

Separate unchanged search follow-up boundaries and affine arithmetic presentation/comparison helpers into private modules to remove both source-size warnings.

Extract unchanged rule-surface matching, unknown-word span readers and HTML text helpers into private modules; retain public concept-lookup paths.

Regenerate native constant witnesses after the lexeme module extraction; preserve every generated program value and algorithm.

Extract unchanged repair-loop notes and text replacement ranges into private modules; preserve existing repair-note public paths.

Separate unchanged controlled-language realization helpers from grammar parsing while retaining their existing public paths.

Separate unchanged CLI solver invocation helpers; regenerate literal-owner paths after source moves without adding allowlisted text.

Separate unchanged solver configuration and repository command admission into private modules while preserving their public paths.

Move unchanged research adoption and method execution helpers into private modules, preserving approval gates and public paths.

Validate workflow and job concurrency queue values and reject max with cancellation enabled beside the pinned actionlint schema exception. Keep expression outcomes explicitly unresolved and preserve the Docker ShellCheck canary.

Keep conversation references ahead of researched explanations in the native fact lookup. A request such as “Explain the program?” now reaches the existing coreference handler when prior user context supplies the program, rather than receiving a cached general explanation. The unchanged multilingual coreference regression and seven existing fixture-based fact and explanation tests run in the focused native pull request target.

Extract complete merge-layout and changelog contributor guides, preserving
instructions, relative links and existing entry-point anchors.

Refresh JavaScript shard scheduling floors from the complete current hosted timing cohort, preserving prior costs and receipt identities while retaining the original execution and headroom limits.

### Fixed
- Retain completion receipts for assertion-only JavaScript test files and refresh shard weights from all current hosted observations without changing execution budgets.
- Forward the opt-in compiler-cache diagnostic switch into reusable native checks.

Admit the existing native data-file contracts and their consumed Links Notation and cached JSON sources to the focused pull-request check. Replay the original meaning-identity, pipe-list and canonical-notation failures without compiling the full unit target.

### Fixed

- Keep documentation extraction available in the docs.rs build profile and correct Rust test inputs used to validate automatic dependency updates.

fix: Keep Google Trends snapshot answers offline, include Spanish prompt variants, and stage fresh native catalog/session projections for byte-for-byte fixture replay.

### Fixed
- Read complete pull request file pages with supported GitHub CLI flags before selecting documentation-only checks. Keep source-receipt observations in the documentation workflow regression fixture.

Keep documentation requirement assertions in their native owner while moving
tracked-text scanning helpers below the existing file-size warning threshold.

Preserve dynamic meta gateway qualification when its host, frozen dependencies or fixtures change. Retain both checkout attempts and recursive fixture inclusion after merging the latest main. Accept distinct existing test suites beyond the obsolete 256-file inventory cap while preserving source validation, measured shards and job deadlines.

Keep separately requested evidence destinations out of literal source edits.
A request that names its source after an evidence instruction can still apply
the edit to that source.

Spell thirteen newly published evidence archive filenames in full and update their physical manifest references. Preserve original source names, compressed bytes, digests and the existing abbreviation ceiling.

Allow the focused test selector to read evidence-heavy Git diffs within an explicit 16 MiB bound, preserve exact filenames with NUL delimiters, and keep Git failures terminal.

Preserve the shared public plan-observation helper API in focused native regression targets, and regenerate the localized source endpoint TypeScript projection. Warning enforcement and generated-source parity stay enabled.

### Fixed
- Encode operator-controlled fragment content and paths in logs so Actions cannot interpret them as commands.

Run the same complete default JavaScript suite before merge and on main,
preserving shard budgets and reporting every failing test file.

Refresh the native Hive shard weights from all 37 completed hosted cases, keeping the maximum prior duration per case. The previously unmeasured eleven-minute workflow replay now receives its own shard; every test remains covered once under the unchanged twenty-minute job deadline.

Separate Hive commit expectation fixture helpers while preserving all original
native regression names, assertions and recorded timing identities.

### Fixed

- Run the existing Hive Mind commit, push, pull request preparation and restart
  regressions in the focused native lane, including their complete fixture
  inputs. Require the harness toolchains so an unavailable compiler cannot
  silently skip delivery verification.

Replay the intact Hive workflow and tool-workspace contracts in focused native CI, including multilingual workflow requests, remote connector reads, failed verification and file-bound commit behavior. Track every consumed fixture in pull request admission.

Resolve scheduled ladder tests for CLI-only modules through their declared
Cargo binary target and record the exact test run in the verifier receipt.

Require observed execution before a verifiable-task response can populate an Agent CLI evidence record. Preserve multiline literal write ownership when a separate delivery destination appears in the request.

Keep the agent change, verification report and Cargo logs available for upload
when a scheduled ladder job reaches its deadline.
Run the exact projected tests once per Cargo target and save the outcome of
each selected case.

Read large Git change inventories completely with bounded decoding and private temporary spooling, preserving failed-Git errors and complete NUL-delimited path admission with a real 24,004-path regression.

Give the scheduled Google Trends learning cycle separate build and capture
jobs with phase deadlines and retained failure diagnostics.

Fix native regression admission for localized how-to test and fixture changes.

Add an optional read-only observation of original and expanded source macro contexts in the existing hosted CodeQL Rust database, retaining query/pack metadata and raw diagnostic outputs without replacing the security suite.

Set the default-off hosted macro observer's Java loading heap explicitly,
separately from its unchanged evaluator RAM budget. Preserve the original
security suites, strict SARIF gate and bounded disk, thread and step settings.
The preceding 4 GiB run still warned at a 1,800 MiB loading heap; the new
allocation and warning outcome require hosted verification.

Require a complete successful lockfile inventory before dependency maintenance resolves any package, and keep transient-link request timers and retry waits within their shared budget.

### Fixed

- Hydrate registered meaning seed filenames in the browser worker and reset derived search markers when seed text is replaced, preserving the full Spanish unresolved-answer sibling contract.

### Fixed

- Give focused native contract artifacts unique workflow-attempt names and select their immutable producer upload ID, preserving exact receipt checks when failed consumers rerun.

fix: Overlap the independently serialized native benchmark group with other isolated contracts after one shared compilation, retaining both exit statuses and the existing 20-minute CI limit.

Share external lookup request logging in the event log, retaining its distinction from captured source evidence. Remove the redundant explanation forwarder and lower the reviewed handler baseline by three lines.

Run the existing benchmark derivation and synthesis contracts in the focused native pull request lane, and admit changes to their license and research inputs.

Remove the two-job throttle from the ten independent native contract consumers,
so available hosted runners can execute the complete matrix concurrently after
its shared compile artifact is ready.

### Fixed

- Split isolated native coding, reasoning, Hive Mind, benchmark and interface contracts into independent jobs after serial execution exhausted the 20-minute job budget. Preserve the full target union, single Rust test threads, failure propagation, read-only permissions and existing job limits. Keep the default complete native runner available.

Replay all seven existing contextual follow-up tests and nine debug-session tests in the focused native reasoning target. Changes to either included module select the native lane, and every existing test body remains intact.

### Fixed

- Keep Hindi and Chinese wikiHow candidate titles consistent between native and
  browser execution, including Latin words and Devanagari combining marks,
  while retaining Russian and Spanish lowercase slugs.
- Run focused native regressions with the repository's optimized test profile,
  retaining debug assertions, overflow checks and the existing CI deadline.

Collect independent native regression failures in one CI run while retaining failed job status.

Extract procedural follow-up and global CLI configuration assertions while
preserving their original native test names, timing records and complete bodies.

Replay complete planner, job-budget and pinned-image regression modules in
the existing focused native CI targets, retaining their original fixtures.

### Fixed

- Preserve canonical IPv4 web operands and keep attachment metadata outside requested URLs.
- Isolate native dialog, Hive and wrapper diagnostics fixtures while preserving their success and no-leak assertions.
- Retain actual rejected synthesis IR attempts and seed bounded closed-program sequence search from ranked fragment types.
- Admit complete composition and search regressions to native CI, and wait for every isolated regression group before reporting failure.

Replay the complete native CLI, PTY isolation, cross-surface memory and retired-handler regression modules in the focused pull-request lane. Changes to every shared harness and captured input now select that lane.

Extract release policy, data integrity and benchmark evaluation helpers into
ordinary native test child modules, retaining every assertion and the existing
size limits. Check standalone Cargo crate-root helper resolution before CI.

Replay the existing candidate-validation and interpretation-selection tests in the focused native CI lane. Select that lane when either consumed module changes, preserving all 28 assertions and the original reasoning-loop source.

Refresh the generated native constant registry after adding optional thinking-step origins. Preserve all 158 program objects and the complete governed inventory, with only the reviewed thinking source hash updated.

Record external fact and web lookup requests in the universal reasoning trace, including honest offline misses, and replay routing contracts with explicit offline configuration and the current default-on retrieval message.

Run the focused native behavior regressions with explicit offline configuration so routing assertions replay committed sources without waiting for live providers. Fixture transports still exercise fetched responses, and the native CLI transport replays retain their default configuration.

Fail the pipeline status when a required job is cancelled on a pull request, preserving timed-out native checks as incomplete instead of reporting success.

Correct the native standalone-script coverage observer to include helpers compiled by native test owners as well as registered gates, preserving the standalone selection and its direct native regression.

Retain nested experiment shell scripts in the focused native sparse checkout so the unchanged network-download contracts can read and sweep their inputs. Keep experiment payloads excluded and exercise the actual checkout patterns in a finite Git fixture.

### Fixed

- Align the newly isolated native synthesis specification with the existing whole-sentence summarization contract, and identify the parity case when an exact answer differs.

### Fixed

- Refresh the governed synthesis source identity after the verified complete-sentence assertion correction, preserving all 1,185 specification records and complete case objects.

Extract native command-capture, script-suite discovery and image-policy regression groups into ordinary child modules. Preserve every assertion and existing test module prefix while removing three source-size warnings under unchanged limits. Keep the scanner regression in the maintained JavaScript checks.

Replay the complete native release, Pages and workflow contracts in the focused pull-request lane, retaining their fixture inputs and installing only the pinned YAML parser they consume.

Retry the existing ShellCheck download with finite connection and transfer deadlines, register its exact invocation count in the native download sweep, and replay the same repository inventory in JavaScript before the next native run.

Fix offline unresolved action requests so the seeded procedure vocabulary can hand them to the cache-only source client, while nominal unknown prompts retain the teaching guide. Refresh the Google Trends catalog and complete driver session from the actual hosted native artifact, and replay the complete existing catalog and unknown-reasoning contracts in the focused native lane.

Run the existing shell documentation and network download retry contracts in the focused native regression lane. Changes to their documentation inputs and every discovered shell script now select that lane.

Build validated main Pages sources after a package publisher fails, preserving successful release child identities and immutable fallback checkout.

### Fixed
- Add bounded real-Git controls for PHP template release push failures, retaining source identities and successful-publication controls.

Fix the single-branch repeated-extglob denial of service in desktop and VS Code development dependencies with a reviewed local picomatch backport for GHSA-fc5q-39xc-vjpf. Preserve the upstream MIT license and source hashes, block nested wildcard repetitions through the existing literal fallback, and add bounded regression and installed-consumer checks.

### Fixed
- Keep fenced examples and existing line endings intact when collecting release notes without an insertion marker.

Keep procedure assertion bodies visible to JavaScript specification and
documentation checks while extracting their source-backed fixtures.

Isolate finite process-lifetime and JavaScript planning fixtures from remote Bash startup files, preserving all production clocks and retaining child diagnostics when a fixture cannot write its PID.

### Fixed

- Stream public release assets with bounded concurrency so verification checks every download within the existing deadline and reports which asset failed when a transfer is incomplete.

Preserve exact JSON tokens while compacting readable primitive leaves, keep feasibility producer formatting reproducible, and retain full fixture and JavaScript gate verification.

### Fixed

- Refresh the four governed Cargo fixture hashes after adding the existing reasoning test target, preserving all 1,185 specification records and every complete case object.

Split the native reasoning contract listing into two measured, exact test shards
so compilation and cleanup retain room inside the existing 20-minute job limit.
Keep every case, failure status, single test thread and complete runner entry point.

### Fixed

- Keep crate availability uncertain when a registry endpoint returns a malformed success response and its peer reports the version absent.
- Require valid SemVer in every sparse index record and reject API responses that identify a different crate.

Keep release-publishing assertions below the existing source warning band while
preserving native test identities and qualifying the complete owner on PRs.

### Fixed

- Bound release credential requests so a stalled registry still leaves time to report the other credential checks.

Replay the existing procedure and concept-retrieval contracts in focused native pull-request CI, and quote the Git tree selector in native fixture identity capture.

Reuse conservative Go-template recovery for confirmed hosted-runner acquisition failures, with trusted default-branch code, complete paginated observations, source freshness checks, bounded attempts and CLI-level negative controls.

### Fixed

- Fail the Rust dependency audit when Git cannot complete the committed lockfile
  inventory, including failures after partial output. Preserve strict primary
  and experiment-lock audit policies and reject missing sparse-checkout inputs.

Retain a pinned six-case upstream runner-recovery pagination replay, including
negative controls and exclusions for actual code failures.

Replay the existing place, calendar, spoken-hour and capability-routing contracts in the focused native pull-request check, and select that check for their sources and fixtures.

Split three CI Rust scripts below the existing source-size warning threshold while preserving all tests and behavior; update the secret-scan production contract to require complete, error-checked file inventories.

Compute the search installer's mirror fallback before passing its environment to the bounded APT helper, preserving existing defaults and explicit overrides while satisfying ShellCheck SC2097 and SC2098.

Split complete meaning records and multilingual synthesis families while
preserving lexicon priority, localized lookups and all original consumers.

Generate the complete native seed bundle inventory in a verified include while
preserving declaration ownership and meaning priority.

Use shared scheduling-act and time-reference meanings for multilingual calendar requests while keeping request spans separate from source text.

Parse calendar clock phrases through shared numeral spans and clock markers in five languages. Preserve requested minutes, AM/PM, explicit 24-hour times and the existing source-admission controls; exclude unrelated years and counts from clock selection. Recognize the additional Hindi spelling of Tbilisi through the existing entity registry.

### Fixed

- Run the original standalone requirements, ordered-list and self-hosting metric
  sources in the focused native script job, including their split modules. Admit
  every consumed helper to that lane without rebuilding the application.

List insertions now require a complete read of the exact target before replacing its contents. Reads of other files and partial source responses can no longer supply the replacement bytes.

Install the locked SWE-bench Lite dataset reader before scheduled and sharded
external benchmark evaluations.

Cancel only synchronized pull-request checks and give independently requested CI observations distinct run groups, with enforcement and actual event-matrix controls while preserving shared writer queues and all 52 release operations.

Correct the Rust template review commit identifier against its Git tree, verify all twelve recorded file identities, and retain maintained regressions for complete source pins and the upstream timeout documentation mismatch.

Repair extracted native test module paths and isolate offline fixture source caches. Update stale release, TypeScript and notation assertions with negative controls; preserve response ordering while splitting the remaining primary response seed and wrapping generated registry entries.

### Fixed

- Regenerate TypeScript companions for the workflow generation and bounded text changes. Preserve both hosted shard failures and their actual file results in the timing evidence.

Correct the unknown-term regression to use uncaptured terms in four languages, preserve the complete attributed cursor definition as a positive control, and execute all nine unknown-source contracts in the focused native pull-request check.

### Fixed

- Keep retrieved word glosses from answering unparsed questions in languages without declared answer support, retain supported definitions and bare-term lookups, and execute the captured-source regressions in the focused native CI check.
- Bind retrieved definitions to the requested term before source ranking and require cached concept answers to cover the complete request focus; preserve matching attributed definitions and explicit cache requests.

Encode contributor-controlled scanner diagnostics so filenames and prose cannot create GitHub Actions commands; preserve intentional annotations and exact ordinary twin output.

Update both audited Rust lockfiles to url 2.5.9 and its required form_urlencoded 1.2.3 dependency, repairing RUSTSEC-2026-0592. Keep the strict audit and replay bounded nested-blob origin controls in the focused native interface contracts.

Fix debugger attribution for validation events by recording the actual append-site identity in Rust and JavaScript. Preserve validation kinds, payloads, evidence and stable step ids, keep legacy traces readable and leave unknown origins unavailable. Add focused regressions for generic answers, candidate validation and metadata custody, plus hosted native replay controls.

## Fixed

- Preserve every verified edit in a request sequence when the result is written
  to a structured effect and its supporting proof.
- Accept complete source reads using the host workspace's absolute paths when
  verifying list edits.
- Resolve numbered source requirements to declared Rust lists and canonical
  seed files, and verify each edit before delivering its evidence.

### Fixed

- Compare pull request versions against the exact fetched base commit despite custom Git refmaps or concurrent fetches.
- Keep complete-suite and release inventory regression fixtures aligned with the maintained checked planner contracts.

### Fixed

- Remove remaining seed and generated release-workflow size warnings while preserving all seed records, consumers and parsed workflow behavior.
- Preserve client description enum choices when adding shell narration and compare sparse-checkout coverage without imposing directory order.

- Keep deliberate BuildKit outage annotations accurate, disable unnecessary RML install scripts, and split provenance verification into independently scheduled suites without dropping controls.

Retry only failed GitHub advisory transport in both complete Zizmor audits, retain fatal security findings, expose both CodeQL producers to audit, and declare the macOS verification shell.

Run the intact web-search, research-follow-up and fetched-page regression modules in the focused native pull-request lane. Admit every included source and preserve the existing fixture contracts.

The browser worker now registers calendar creation directly and promotes it with the shared sentence-local scheduling evidence, so an ambiguous memory-write verb reaches the calendar. Explicit memory writes keep their existing behavior.

Calendar handlers share one named argument list. Refresh the generated runtime inventory and solver diagram, and record the lower measured closure gap.

Extract worker notation and code-refactoring sections into named modules to remove four source-size warnings without raising size limits or combined worker ceilings. Regenerate module inventories, TypeScript twins and status counts; preserve notation bindings and refactoring behavior with maintained regressions.

## [0.356.11] - 2026-11-23

### Fixed

- Recover verified wikiHow instructions through the publisher’s public search when the structured search API fails. Present the relevant instructions and their citation in chat, or say when verified steps are unavailable.

## [0.356.10] - 2026-11-23

### Fixed
- Restore the GitHub Pages release build by correcting Rust API documentation markup. Add pre-merge checks for both documentation errors that stopped the site build after the client reporting fixes shipped.

## [0.356.9] - 2026-11-22

### Fixed

- Accept draft-07 and 2019-09 schema declarations and common annotations, including `format`, in OpenAI structured output. Refusals for unsupported constraints identify the keyword and its JSON Pointer.

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
