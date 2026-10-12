# Changelog archive: 0.346.0 to 0.351.0

Releases in this range, newest first. Newer releases are in [CHANGELOG.md](../../CHANGELOG.md).

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
