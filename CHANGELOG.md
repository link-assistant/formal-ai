# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

Older releases are archived under `docs/changelog/` so that no file
exceeds the repository's 1500-line cap (newest first):

- [0.355.0 to 0.355.32](docs/changelog/releases-from-0.355.0.md)
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
