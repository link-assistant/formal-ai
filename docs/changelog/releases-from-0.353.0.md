# Changelog archive: 0.353.0

Releases in this range, newest first. Newer releases are in [CHANGELOG.md](../../CHANGELOG.md).

## [0.353.0] - 2026-10-12

Shard browser coverage into six source-bound producers with complete unchanged-ratchet collection. Preserve planner preflight states and original route ceiling, fix equivalent Clippy expressions and unused native fixture import. Observe an active release ancestor independently of the pending contender. Restrict staged releases to four consumed optional credentials while preserving all operations, transfers and publication obligations. Migrate container publication assertions through checked physical source views and preserve all original controls.

Preserve source-bound progressive work drafts, unfinished acceptance and complete failed/partial observations for restart. Fresh hosted CI, native runtime and production release authority remain required.

Activated complete staged release workflows while preserving all original operations, transfers, credential checks, compiler and publication dependencies. Native workflow fixtures now inspect checked operation views and retain physical permission scans and all eight writer roles. Isolated source mutation controls prevent concurrent tests from changing shared workflows. Removed obsolete inline deadline exceptions and selected deployed source inventory in the configured fixture.

Imported the genuine native census only after exact source, Git blob, ZIP and producer identity checks.43source guards,20repository checks and105portable release controls pass, with one existing optional skip. Fresh native CI and actual production release authority remain required.

Refresh the unused staged release projection from the reviewed canonical cache and caller concurrency changes. All 52 original operations and 104 output transfers remain checked, including real BuildKit cache settings. Production release callers, budgets and operations are unchanged; native/publication timing and U9 remain Open.

Constrain the dormant staged release workflow token to the operations that require write access, retaining its checked canonical projection. This workflow remains noninvoked; U9 timing remains open.

Add an uninvoked staged release workflow, readable immutable source/executable/package/image transfer contracts, and independent byte/inventory/permission refusal controls. Preserve the original release callers, package verification, cold registry smoke, published stable authority and timing floors. U9 remains Open pending actual GitHub lease/artifact behavior and complete cold release observations.

Add a read-only PR CI fixture for controlled text-tar artifact-ID transfer across reusable jobs, actual active/inactive outputs, and a private per-run caller lease with independent queued contender observations. It never invokes production release authority, native builds, compiler, package publishing, version/tag/main writes or registry actions. Actual GitHub service observations arrive only from CI; U9 remains Open and this fixture provides no production release or cold timing proof.

Fixed the native planner enum wrapper and strict-warning condition identified by genuine PR CI. Source-member move and copy requests now require source-aware changes; whole-file path operations keep their existing behavior. Preserved the original memory byte roundtrip tests through a physical test module and corrected source-capture meanings to use the declared plural words field.

All 34 closest JavaScript controls and twenty source and projection checks pass. Fresh native CI and production release validation remain required.

### Fixed
- Keep CI release version commits on the exact tested main source; refuse stale-main rebases and changed retry children before tagging or publishing. Preserve general command-line rebasing and downstream release source authority.

Release controls now bind the metadata operation to the prepared image factory and retain every original desktop and updater asset obligation. Independent fixture checks cover version/latest tag ordering, complete release evidence, each missing original asset and unknown API/authentication responses. Production release gates and budgets are unchanged; U9 remains Open.

Source controls track the relocated native executable and coverage producers, require declared stage caps at most thirty minutes, and retain original feature, artifact, smoke and reduction obligations. All four current attestation producers are checked. Production workflows, quality floors and operations are unchanged; actual cold timing and U9 remain Open.

Keep authored literal payload actions inside their source-owned spans, bind ambiguous authoring verbs to their actual operation object, and preserve independent creation/edit goals through request-local replay. Unsupported later goals stay explicit gaps; exact Unicode bytes and existing failure/verification safeguards remain required.

### Fixed

- The Coverage lane survives a contended host: main run 36269286808
  burned the full 4200s budget with every test passing and no hanger,
  while the identical tree on branch run 36266423193 finished the same
  lane green in 2574s the same day — so `TEST_BUDGET_SECONDS` rose to
  5400 together with the job timeout to 135 minutes, keeping the 66.7%
  budget-to-cap ratio issue #1017 enforces (issue #1149).

### Fixed

- The unknown-reasoning fallback now executes its web search for the focus
  phrase instead of answering with the canned "Web search requested for `…`"
  description of the browser demo's machinery. The description arms (default,
  latest-news, open-research, in English and Russian) are deleted as an
  answer surface everywhere: an executed search answers from its captured
  statements with citations and records `web_search:executed`, and an
  offline or cache-miss run degrades to the localized
  `web_search_unavailable` response that names the query, with the live
  transport still behind the `FORMAL_AI_LIVE_FETCH` opt-in (issue #1173).

### Fixed

- The client registry now declares *how* a client is pointed at its config
  instead of implying an `XDG_CONFIG_HOME` relocation that also moves `gh`'s
  and git's config and drops the session's GitHub authentication (the
  2026-09-27 Hive Mind incident, link-assistant/hive-mind#2314). Every
  `global_configs` entry in `data/seed/client-integrations.lino` may carry
  `config_env "VAR"` declarations — parsed into
  `ClientIntegrationGlobalConfig::config_env`, surfaced additively as
  `global_configs[].config_env` in `formal-ai clients --format json` and as
  `via VAR` on each `global[...]` line of the text output. Filled only where
  in-repo evidence exists: `agent` → `LINK_ASSISTANT_AGENT_CONFIG` /
  `LINK_ASSISTANT_AGENT_CONFIG_DIR`, `codex` and the `t3code` OpenAI entry →
  `CODEX_HOME`, the opencode family → `OPENCODE_CONFIG` /
  `OPENCODE_CONFIG_DIR`; profile-writing and fixed-path entries declare none.
  A documented host integration contract (`docs/configuration/agentic-clis.md`)
  requires hosts to use the declared variables and never relocate
  `XDG_CONFIG_HOME`/`HOME` (issue #1161).

### Fixed

- Factual Q&A no longer answers for the wrong subject: "What is the capital
  of Australia?" answered "The capital of the United States is Washington,
  D.C." because the United States alias "us" matched as a raw substring
  inside "a**us**tralia" (issue #1172). `FactRecord::matches_normalized`
  (`rust/src/seed/facts.rs`) now matches subject aliases and question
  keywords only at word boundaries through the new
  `FactRecord::contains_word_sequence` — the phrase is tokenized with the
  same normalizer the prompt was and must appear as a consecutive token run,
  while CJK surfaces (no inter-word spaces) keep substring matching, the
  contract `seed::meanings::surface_present` established (issue #386). The
  browser worker's `tryFactLookup`, the coreference keyword prefilter, and
  the antecedent fact-alias match (`js/worker/formal_ai_worker_05.js`) get
  the same boundary treatment via the shared `surfacePresent` helper.
  `fact_store_resolves` therefore returns `false` for subjects with no
  seeded record, unblocking the open-web planner's fallback instead of
  discarding it for a false hit. Possessive aliases ("japan's") match again
  through the phrase-side normalization, and the contradicted
  "Implemented" rows R173/R174/R177 of the issue-0127 requirements shard now
  record the defect and its correction. Pinned by
  `rust/tests/unit/issue_1172_factual_qa_subject_match.rs`.

### Added

- Every release now publishes standalone `formal-ai` CLI archives:
  `formal-ai-cli-<target>.tar.gz` for `x86_64-unknown-linux-musl`,
  `aarch64-unknown-linux-musl`, `x86_64-apple-darwin` and
  `aarch64-apple-darwin`, and
  `formal-ai-cli-x86_64-pc-windows-msvc.zip` — built by a new `cli` job in
  the Desktop Release workflow, each holding the binary plus `LICENSE` and
  `README.md`, sha256-verified through the same consolidated
  `SHA256SUMS.txt`/`BUILD-PROVENANCE.txt` as the desktop assets, with SLSA
  attestations and a per-leg smoke test that extracts the archive and runs
  `formal-ai --version` on its own runner. `cargo binstall formal-ai`
  resolves the archives through new `[package.metadata.binstall]` metadata,
  and `scripts/install.sh` / `scripts/install.ps1` now install the CLI from
  the prebuilt archive first, falling back to `cargo install formal-ai`
  only when no archive matches the host (issue #1181).

### Fixed

- Boolean `FORMAL_AI_*` environment switches share one spelling table —
  `1/0/true/false/yes/no/on/off`, case-insensitive — through the new
  `cli_env` module, so `FORMAL_AI_SILENT=1 formal-ai chat …` no longer fails
  with `invalid value '1' for '--silent'` (the same fix covers
  `FORMAL_AI_TRACE_*`, `FORMAL_AI_MEMORY_DEBUG`, `FORMAL_AI_DREAMING*`,
  `FORMAL_AI_RECORD_CHAT`, `FORMAL_AI_LIVE_FETCH`/`_LIVE_API`,
  `FORMAL_AI_LINK_CLI_DEBUG` and `FORMAL_AI_SEED_LINKS_MIRROR`); an
  unrecognised value keeps each switch's documented default instead of being
  silently coerced (issue #1181).

### Fixed

- Prompts whose first word merely happens to be a shell-token word ("Find the
  bug: …", "Make a 3-day itinerary …") are no longer answered as terminal
  commands: the leading-token detection path now requires the rest of the
  prompt to parse as command arguments (no question mark, no colon-space
  outside quotes, no sentence-final punctuation on bare tokens, no bare
  function words; quoted spans are masked first), while real commands such as
  `find . -name '*.log' -size +10M` and `git commit -m 'fix the parser bug'`
  stay detected. The software-project detector likewise no longer picks its
  artifact kind from an artifact word anywhere in the prompt — only from the
  head noun of the authoring verb's object phrase — so "Write a regular
  expression that matches a US ZIP code with an optional 4-digit extension."
  no longer becomes an implementation plan for an extension, while
  artifact-headed requests in English, Russian, Hindi and Chinese keep their
  frame (issue #1175).

### Added

- Quantity questions are now computed exactly instead of answered with the
  canned search paragraph: statistics over a stated list ("what are the mean
  and the median of 4, 8, 15, 16, 23, 42?" → mean 108 / 6 = 18, median
  (15 + 16) / 2 = 15.5, variance from the exact fraction with an ≈ marker on
  the rendered digits), unit conversion from seed factors and formulas
  ("26.2 miles" → 42.1648128 km with the multiplication shown; °C → °F shows
  25 × 9/5 + 32 = 77), and the price×count word-problem pattern ("buys 4
  pens at 3 dollars each and pays with 20" → the change is 8: 20 - 4 × 3 =
  8). Operation names, markers and answer prose are seed data in five
  languages (issue #1176).

### Fixed

- "What day of the week will it be 100 days after Monday?" no longer drops
  the stated offset: the calendar handler reads the numeral and unit the
  prompt states (days or weeks, digits or spelled cardinals, unspaced CJK
  "100天" included) and shifts by it — 100 days after Monday is Wednesday,
  derived as 100 days = 14 weeks + 2 days; a whole number of weeks states
  that the weekday is unchanged; a bare "the day after Monday" keeps its
  original ±1 reading (issue #1176).

### Added

- Nine code-task request families are now answered with derived artifacts
  instead of the canned search paragraph: debugging names the quotient-shift
  defect ("what's wrong with `def average(xs): return sum(xs) / len(xs) -
  1`?" quotes the line, names `- 1` as the shift, shows both fixes and the
  property the name promises with its source), regular expressions are
  composed from seed vocabulary and verified structurally ("five digits
  optionally followed by a hyphen and four digits" → `^\d{5}(-\d{4})?$`),
  SQL is composed clause by clause with the request words mapped back
  ("select all users older than 30" → `SELECT * FROM users WHERE age > 30;`),
  file-finding requests compose a shell command with the manual cited
  (".log files larger than 10 MB under /var" → `find /var -name '*.log'
  -size +10M`), code is explained line by line against a grounded construct
  table, code review reports the five seed rules it matches with upstream
  sources, test generation emits pytest suites for recognized problem
  shapes with a `normalize()` helper on cue, promise chains are refactored
  to `async`/`await` with handler bodies verbatim, and JSON↔YAML conversion
  round-trips the document through the same subset parser with a value
  check. Every answer states honestly that no code, query, command, regex
  or generated test was executed; unrelated prompts are claimed by none of
  the nine handlers (issue #1177).

### Added

- Five text-transform request families are now answered with the computed
  text instead of the canned search paragraph: a summarization request that
  carries its own paragraph returns a weight-ranked selection of its
  statements ("Summarize this paragraph: The Halley research station, opened
  in 1956, is used to study the Antarctic ice shelf. It provides year-round
  measurements…" keeps the purpose and feature statements and drops the
  rest), register rewriting substitutes every seeded informal token and
  splices politeness phrase rules ("can u send me the report asap thx" →
  "could you please send me the report as soon as possible thank you", each
  substitution logged, already-formal texts reported honestly), grammar
  correction re-agrees subjects and pluralizes numerals' nouns with each
  fired rule named ("She don't like apples and he have two cat." →
  "doesn't"/"has"/"cats" under third_person_do_not, third_person_have and
  plural_noun_after_numeral), genre writing composes a Conventional Commits
  line or a business email from the seeded frames ("correct an off-by-one
  error in the pagination helper" → "fix(pagination): correct off-by-one
  error"; a day-off email fills greeting, intent, reason, weekday and
  closing; missing slots are named, not fabricated), and a translation
  request whose sentence carries no quotes still translates word by word
  ("Translate to Russian: The weather is nice today, let's go for a walk."
  → every content word resolved from its seeded lemma, function words
  dropped and logged, unknown words named rather than hidden). All
  recognition vocabulary and response prose live in seed data with
  five-language response parity (issue #1174).

bump: minor
---

### Added

- Generated CI workflows now resolve every third-party pin at generation
  time instead of shipping memorized strings (issue #1168): the action refs
  (`actions/checkout`, `actions/setup-java`, `fwilhe2/setup-kotlin`,
  `actions/setup-python`), the Kotlin and CPython versions, and the Java LTS
  come from the publishers' own APIs through the existing
  `CachedSourceClient`, are recorded in the derivation as `SourceCapture`s,
  and degrade loudly — cache first, then the shipped
  `data/seed/toolchains.lino` baseline measured 2026-09-30 — with the origin
  written as a comment on the generated workflow. A new gate
  (`scripts/check-generated-version-literals.rs`) fails on any literal
  action SHA or bare toolchain version under `data/`.

### Added

- The executor gains an error-driven repair loop for failed build and run
  steps (issue #1185): instead of stopping at the first failed step, the
  raw output is formalized into a structured diagnostic — file, line, error
  code, message — through one generic `{slot}` template matcher over the
  new `diagnostic_code_shapes` seed (14 languages, slots validated by
  shape, no per-language regexes in Rust), the diagnostic becomes a search
  query carrying the exact error code, each fetched source is read and a
  fragment retained only when it addresses that exact code or message, the
  retained fix is written as a Links Notation `repair_edit` record beside
  the artifact (`<stem>.repair.lino`, never a raw text patch), and the
  failed command is retried on a ladder bounded at three rungs whose limit
  is reported honestly when reached. Every attempt is reconstructable as an
  evidence chain (`repair_attempts` document) for the answer's derivation;
  when no fetched source matches, the loop reports the unresolved need by
  name and falls back to today's honest failure report rather than
  fabricating a fix. Delivered inert pending the main session's wiring:
  module registration, seed-registry entries, and the one-line consult in
  `command_reroute`'s failure branch (see
  `docs/requirements/issue-1185-error-driven-repair-loop.md`).

### Added

- A coding procedure is now a deletable, rediscoverable cache entry instead of
  memorized knowledge (issue #1165, E130). The new
  `rust/src/discovery_production.rs` fronts the production miss path —
  `cached_or_research` answers a `(language, task)` pair from
  `data/cache/coding-procedure-cache.lino` and, on a miss, runs
  `research_coding_skill_gap` and stores the verified procedure with its
  rediscovery query and documented source. The policy is data, not code:
  `data/seed/program-cache-policy.lino` declares which fields a row must
  carry, the FNV-1a content address recomputed on every load and store, and
  whether the embedded `ORACLE_SNAPSHOTS` bootstrap in `rust/src/knowledge.rs`
  still answers — retiring it is a seed edit, not a code change.

### Changed

- `CodingOracle::lookup` and `knows_language` serve the embedded snapshots
  only while the policy seed's bootstrap record is active; the snapshots are
  the record of discovery that ran before the cache file existed, and the
  production authority becomes "a grammar exists and discovery found a
  procedure".

### Added

- The request is now formalized into obligations instead of matched phrases
  (issue #1166, E131). `rust/src/intent_formalization/obligations.rs` parses
  an issue body into an `ObligationGraph`: one ledger `ObligationNode` per
  enumerated clause, each classified against the seed lexicon (`print_stdout`,
  `ci_workflow_request`, the program-request roles, plus documented
  multilingual fallback tables for style, naming, and badge vocabulary), with
  quoted output literals anchored to their value and mentions of the same
  literal coreferenced to exactly one node.

### Fixed

- The Kotlin doubled-output root cause of #1156: the literal to print was
  collected once per mentioning clause, so an issue body naming
  `Hello, World!` twice (in "print exactly" and in the Expected Output block)
  printed it twice. `unique_output_literal` now returns the value once no
  matter how many clauses carry it.
- A prompt starting with a shell token is no longer classified as a terminal
  command when it formalizes into work obligations — a quoted output literal
  plus an authoring clause is a sentence about building something, including
  in languages the English-only marker words of the #1175 guard do not cover;
  `git status` and `echo "Hello, World!"` stay commands.

### Added

- `formal-ai explain <answer-id>` prints the white-box derivation of a
  previously returned answer: the search queries issued, every fetched URL
  with its SHA-256 hash and fetch timestamp, the formalized fragments, the
  decomposed parts, the recomposition, the rendering, and the verification
  output — read from a durable Links Notation record persisted under
  `data/cache/derivations/<answer_id>.lino`, so explaining a past answer
  never replays the request (`--format links` prints the canonical record).
  The record is a projection of the same append-only event log the
  `--thinking` trace narrates, so the two surfaces cannot disagree, and a
  stage the route never populated is reported as "not recorded", never
  fabricated. The subcommand is disjoint from the in-chat "explain how
  Formal AI works" self-explanation recipe (issue #1184).

### Notes

- The derivation stage vocabulary (`formalize:fragment`, `decompose:part`,
  `recompose:bind`, `render:emit`, `verify:evidence`) is declared as seed
  data in `data/seed/derivation-schema.lino`; the E128/E129/E131/E132
  landings append their stages to it.

---
bump: patch
issue: 1154
---

### Fixed

- Codex's `exec_command` sends the shell command under the `cmd` argument
  key; the agentic progress reader recognized only `command`, so a successful
  work-item read was never recorded and the planner re-planned the identical
  `gh issue view` 78 times in one session (6.2M input tokens). One shared
  command-argument reading now serves the whole `agentic_coding` module —
  `command`, `cmd`, `script`, joined array forms, and a schema-driven key
  lookup for clients that name the property differently — and a new
  parsing-independent invariant replaces any planned call that repeats one
  this session already completed with a report of the stuck step and its
  last result.

---
bump: patch
issue: 1155
---

### Fixed

- An unauthenticated `gh issue view` prints its how-to-authenticate banner,
  and the Agent CLI echoes that output with the exit status dropped — the
  banner was then accepted as the issue body, the run wrote its plan record,
  and answered `planned_not_executed` without ever trying the `webfetch`
  tool the client offered (2026-09-27 Scala run). Every planned read now
  prints its own exit status as a sentinel line, the output must still have
  the title-and-body shape of a work item, and a failed read walks the
  fallbacks in order — the client's fetch tool, the credential-free REST
  read, `gh api`, then the prepared pull request. An exhausted retrieval
  closes by listing every read tried with its result, and the only write is
  no longer spent on a plan record for an issue the session never read.

bump: minor
---

### Added

- Every dependency now sits at its latest release, enforced by a gate
  (issue #1169): `scripts/check-dependencies-latest.rs` reads
  `Cargo.toml`/`Cargo.lock`, every `package.json` and its lockfile, the
  workflow `uses:` refs, the Dockerfile base image and the `rust-version`
  floor, and compares each against the publisher's own registry (crates.io,
  npm, GitHub releases, Docker Hub, the rust stable channel) — never a
  memorized list. A dependency may be held back only with a same-line
  `# blocked: <issue-url>` annotation (`CONTRIBUTING.md` documents the
  rule); drift without one fails the gate. A daily
  `dependencies-latest` workflow refreshes the registry snapshot the
  offline mode reads, applies every unblocked bump, re-resolves the
  lockfiles, re-runs the CI gates and opens one PR with what moved.

### Changed

- The tree itself moved to latest: `cargo update` re-resolved the Rust
  lockfile (command-stream 1.2.0, webrtc 0.21.0, toml_edit 0.25.15,
  clap 4.6.7, …; `links-notation` stays 0.16.1 behind an annotated block on
  lino-objects-codec 0.7.0), the npm manifests and lockfiles were
  hand-bumped with real registry integrity (react/react-dom 19.3.0,
  dompurify 3.4.16, marked 18.0.14, command-stream 1.2.0, electron 44.4.5,
  the electron-builder 26.17.0 ring, playwright 1.63.0 everywhere), and the
  image/action pins moved (`konrad/box-dind` 2.10.2, `actions/cache` v6,
  `zizmor-action` v0.6.4).

bump: minor
---

### Added

- Shareable package library and example artifact (#668): preserve provenance and knowledge links, review permissions against a local handler catalog, reject missing handlers, and record successful imports. CLI and browser integration remains pending.

bump: minor
---

### Added

- Opt-in append-only memory sync library (#669): immutable SHA-256 event objects, atomic user-owned directory transport, validation before local merge, idempotent duplicate suppression and convergence tests. CLI, browser and network backend integration remains pending.

bump: minor
---

### Added

- Optional anonymous diagnostic library (#861): disabled by default, single-attempt or revocable automatic consent, fixed diagnostic allowlist, validated Sentry HTTPS envelope transport, reporting choices, and consent tests. Configuration surfaces and proactive reporting UI integration remain pending.

bump: minor
---

### Added

- Duplicate-function gate and weekly cross-organization candidate scan (#1182), with shared literal-aware extraction, explicit baseline maintenance, fixture tests and an experimental seed-tree adapter for installed links-notation 0.16.1. Exact baseline measurement, full corpus conformance and consolidation/adoption remain pending.

### Fixed

- Deduplicate repeated stdout requirements and name generated workflows from their artifacts.
- Preserve recipe progress across continuation turns and make artifact commits idempotent.
- Add guarded PR completion, prepared-work restart routing, and bounded typed-publisher prerequisite recovery.
- Consult the error repair ladder before finalizing failed recipe commands.

### Fixed

- Align CLI release output directories with archive packaging on every platform.
- Correct code-task role loading and preserve all code-explanation constructs.
- Consolidate duplicate meanings and restore missing grounded cache fields.
- Update desktop and coverage workflow assertions for the added CLI release leg.

### Fixed
- Page formalization and source trust read their seed records under the file's wrapper, so mime hints, sniff rules, code-fence language tags and trust weights apply again (issue #1163).
- `git log` history import passes its record format as `--format=`, not as a revision (issue #1180).
- A comma inside a date ("July 4, 1776") no longer splits the question into two sub-requests.
- A clarifying question about an unfilled slot (`*_unspecified`) and an arithmetic error naming what failed are kept rather than replaced by an open meta reply.
- The word-internal punctuation table no longer lists ASCII characters for spaced scripts, so names like `Node.js` stay whole inside an object phrase.
- The HTML page walker steps into container tags (`html`, `body`, `table`) instead of jumping past their close tag, skips comments and opaque tags, and no longer lets `</p` match `</pre>`, so captured pages yield their headings, rows and code blocks again (issues #1163, #1164).
- A folder named on a personal location ("on my desktop") stays the honest `list_dir` gap; only a file-type filter (`.lino files`) hands the request to the shell-command composer.
- "Make an add-on for ..." is read as an extension project: `add-on` is a seeded surface of the extension artifact, grounded in WordNet.
- The role registry is generated from exactly the files the meaning lexicon loads, so roles declared in `software-project-phrases.lino` are registered.
- Lean and Rocq renderings of a "No ..." statement keep the negation (`¬∃` in Lean, `~ (exists ...)` in Rocq).
- Word problems are read as quantity relations: gain, loss, group and share words from the seed join the stated numbers ("Tom has 5 apples and buys 3 more" → 5 + 3 = 8), in Rust and the browser worker.
- The no-memorization gate also scans the nine code-task handlers' answer surfaces (issue #1177).

### Changed
- The coding-ladder gate accepts a lower floor only with a `coding_ladder_correction` note that says "corrected overcount" and names the previous floor.
- Derivation records under `data/cache/derivations/` are registered in the sources registry and git-ignored as run-time output.
- Unit conversion factors are checked against Wikidata: a cached P2370 table for the grounded units pins every SI factor of the length, mass and time units (#1176 R1).
- A stopped error-driven repair loop now says why it stopped (ladder spent, or no fetched source addressed the error) and attaches the attempt chain to the failure report, in both the Rust and JS roots (#1185 R4-R6).
- A no-memorization ratchet now counts the verbatim Hello World programs stored under data/ (14, the catalog templates) and fails if any more are added (#1165 R8).
- `formal-ai explain` now lists the rules a text-transform answer applied (register substitutions, grammar fixes, genre frames, summarization bounds, translation steps): the derivation record gains a schema-declared rule stage (#1174 R9).
- The SQL composer groups an aggregate by the column a seeded grouping cue names ("count users per country" → `GROUP BY country`), and reads a threshold after the grouping as HAVING on the aggregate ("having at least 3 orders" → `HAVING COUNT(*) >= 3`) instead of a bogus WHERE filter, in Rust and the browser worker (#1177 R3).
- A manual dispatch of the CI/CD pipeline now runs the checks only unless a release mode is chosen, and every release, publish and tag job also requires the main branch (#1187 R3).
- The i18n catalog check registers the glass-opacity and material-skin settings keys added with the configurable skins.
- The JavaScript coding oracle reads the bootstrap gate of the cache policy correctly (it treated every language as unknown), and the Rust procedure cache reads its required fields from the policy seed, so both roots require `verified_output` (#1165 R10).
- The code debugger reports a second defect class: a loop bound that lets an index reach the length of a collection the code indexes ("range(len(xs) + 1)", "i <= arr.length"), with the bound that stops one position earlier (#1177 R1).
- The shell composer answers a substitution request ("replace 1.0 with 2.0 in version.txt", also in Russian) with a literal `sed -i 's/1\.0/2.0/g' version.txt`, its flags explained from the GNU sed manual; before, such requests went unanswered (#1177 R4).
- The code debugger reports a third defect class, an assignment inside a parenthesized `if`/`while` condition (`if (x = 5)` → `if (x == 5)`), while a deliberate nested assignment and an `=` inside quoted text are left alone. The no-intent note is now separated from the sentence before it (#1177 R1).
- A verified procedure-cache row now answers a concrete `write_program` request in the solver and in the work-item executor. Generated Scala CI pins `coursier/setup-action`, the JVM and the Scala version resolved at generation time instead of memorized literals, and the `data/` version-literal gate is now a CI stage. The JavaScript root resolves the version pins through an injected fetch (live, then cache, then baseline). The CI-workflow demand is read from the formalized request in both roots (#1165 R1/R2/R10, #1168 R6–R8, #1166 R3, #1169 R4).
- The SQL composer joins on a stated shared column ("joined with the orders table on user_id" → `JOIN orders USING (user_id)`, also in Russian) and never guesses a join key. Both runtimes now read SQL identifiers from the lowercased prompt, so `user_id` keeps its underscore in the native engine too (#1177 R3).
- Repository history: issue and pull-request bodies are formalized into `requirement:` statements and dated `state:` lifecycle transitions; a new `repository_lineage` route answers "which issue introduced <path>?" from the path's git history, naming the issue and pull request (en/ru/hi/zh/es cues). Every answer surface carries its `derivation_id`: the browser worker result and the `/v1/responses` object of both servers. The repair loop's diagnostic table is pinned for all fourteen languages in both roots, and PHP diagnostics with an inline location are no longer dropped. `scripts/translate-es.mjs` now renders `js/agentic/**/*.mjs` into a `ts/agentic/**/*.mts` TS root (#1180, #1184, #1185).
- Fetched pages now do work in the solver (issue #1163): a need no registered source covers is researched through web search and answered from the formalized page, research reuses a page already in working memory instead of fetching it again, trust scores are computed (official site from Wikidata P856, registry primacy, open-license markers, cross-page agreement), HTML and Markdown documents can be read through the page formalizer, and a prompt that supplies a page answers "the command in the paragraph that mentions …" from that page in both runtimes. The native engine also gains a live Wikidata fact resolver (issue #1172 R3), not yet wired into fact lookup.
- Held-out Pascal is learned from its documentation alone (#1164): Pascal gains a grammar row (meta-language ships tree-sitter-pascal) but no stored vocabulary or program; the Free Pascal Quick Start example plus the RTL `WriteLn` page's prose (a new `print` prose relation) recompose `Hello, Formal AI!`, and recomposed Rust, Go and Pascal programs are compiled and run with their own toolchains where installed, with a gated live test over the real pages. A `write_program` cache miss now names the research input the solve lacks (#1165), the program-contract executor records every unreadable request clause as an `obligation_gap` event (#1166), the JavaScript `forGeneration` walks live → cache → baseline over an injected fetch (#1168), and a shared cross-runtime case pins the same derivation id in both runtimes (#1184).
- Fact questions no seeded record answers are now answered live from Wikidata in the native `fact_lookup` row (issue #1172 R3), with the subject read from seed cues and the claim's reference URL cited through the new `fact_live_answer` template (the browser's live answer uses the same template); "Explain X" falls back to the statements of retrieved, formalized pages, each cited with URL and SHA-256 (R8). The page formalizer reads kotlinlang's `data-lang` blocks inside list items and inherits a container's language class, verified on byte-for-byte captures of kotlinlang, the Rust Book, Scala, go.dev and Swift pages (#1163 R14, #1164 R12), with a `FORMAL_AI_LIVE_FETCH`-gated live kotlinlang test (#1163 R11) and case-study evidence from real runs (#1163 R15, #1164 R14).
- Routing, repository questions, repair and formal export (#1175 R3, #1180 R10/R11, #1185 R3, #1186 R4/R6, #1169 R4): a claim router now consults the `claim` rows of `data/seed/capability-routing.lino` before `software_project`, `terminal_command` and `repository_lineage` run, so they claim only on structural evidence (Rust and the browser worker). `formal-ai chat` explains why the self-development status fails, using the ledger floor and target and the tag range, and answers "what does `fn` X do?" from the self-AST census, the doc comment and `git log -S`, with a JS twin. The repair loop renders the `repair_edit` record back into source and applies it only when the meta-language CST accepts the result. Formalization runs `rml export lean` from link-foundation/relative-meta-logic when it is present, and records its exit status and output path for `formal-ai explain`; CI runs it at a pinned commit. The desktop and VS Code sandbox image moves to `konard/box-dind:2.10.2`.

### Added
- Web search pages become formal knowledge (issue #1163): source research stores every captured page's formalized network in the solver's working memory under its URL and SHA-256 and reuses it on a second pass, and its learning proposal carries one `formalized_page_statement` record per statement.
- The memory query language resolves "code blocks on `<domain>` whose text contains `<term>`" and "the command in the paragraph that mentions `<phrase>`" from `page_query` templates in `data/seed/page-formalization-rules.lino`; the second answers only for paragraphs that themselves mention the phrase, with the command each one introduces.
- A need whose kind no registered source declares can be researched through web search (`research_unmatched_need`), and the sources registry gains a `web_page` row whose `generic_page_v1` extractor reads any fetched page through the generic formalizer in the concept-lookup dispatcher.
- The browser worker gets the generic page formalizer as a JavaScript twin (`js/worker/formal_ai_worker_web_formalize.js`): mime hints and sniffing, the HTML walker, Markdown, JSON and plain-text blocks, seed-driven language tags, the URL-and-hash page store, both page queries and the trust score, with node tests over the same fixtures as the Rust tests.

### Added
- Recomposition builds a program from the aligned decomposed parts (issue #1164): each decomposed example keeps its own body with the output literal slotted, the generalized procedure carries one body per contributing language beside the `output_call` parameter, and the bound literal is escaped for its quote; a language no example contributed is refused instead of filled from a stored template.
- `adopt_decomposed_procedure` promotes a generalized procedure through the existing execution-and-approval gate of `adopt_extracted_procedure`, filling the capture provenance and license the decomposer leaves empty.
- Parts read from a page's code carry that page's URL (`decompose_code_node_from`), and the R language has a part vocabulary.
- The browser worker gets the decomposer as a JavaScript twin (`js/worker/formal_ai_worker_code_examples.js`): decompose, generalize, recompose, step records and Links Notation, with node tests over the captured Hello World pages.

### Changed
- `data/seed/code-example-parts.lino` stores no program templates and no Pascal vocabulary, so Pascal stays held out until a decomposed Free Pascal example supplies it.

### Added
- Repository history import records `Co-Authored-By:` trailers as `coauthor:<name>` evidence, and names the top-level items a commit added, removed or modified (`symbol:<path>#fn name`, `#impl Display for Answer`, `#function load`) from seeded item keywords in `data/seed/history-formalization.lino` (issue #1180).
- `formal-ai repository-history import` and `formal-ai repository-history query <SQL>` run the incremental importer and a read-only ANSI-SQL memory query over its store.
- `js/agentic/crate/history_context.mjs` is the JavaScript twin of the importer's pure core (rules seed, `git log` parser, trailer and co-author scans, named-item diff, commit formalization), with node tests in `rust/tests/web/agentic-history-context.test.mjs`.

### Fixed
- The history rules seed rows were never read: `HistoryRules::from_seed_text` did not descend under the `history_formalization` root, so the built-in defaults always applied.

### Added
- Deformalization reads Lean 4, Rocq and Links Notation, not only first-order logic: the target is matched against the same seeded templates it was rendered from, re-rendered and compared by structure, in both the Rust handler and the browser worker (issue #1186).
- A formalization answer records its parsed clause, each rendered target with its template and the prover lookup as derivation fragments, so `formal-ai explain <answer-id>` shows them.
- A 44-prompt formalization probe set in `data/benchmarks/formalization/{en,ru,hi,zh}.lino`, run by a Rust test and a browser-worker node test; a cross-runtime parity case `e1186_formalization_first_order`; and a no-memorization gate over the formalization seed and runtime.

### Fixed
- Hindi object-verb sentences formalize as a two-place relation through a seeded `verb_final` grammar flag, and the Chinese conjunctive template separates the head from the predicate.

### Changed
- `self-authored-pull-request.yml` takes its token from the automation-token resolver instead of `FORMAL_AI_BOT_TOKEN`, and at the `default` layer dispatches the bot pull request's checks so they start without approval (issue #1187).
- `layered-ci.yml`, `evidence-check.yml`, `workflows.yml` and `web-ui-boundary.yml` accept a checks-mode `workflow_dispatch`; the dispatcher dispatches on the head branch and only workflows that declare the checks mode.
- Every `pull_request` workflow ignores `e2e/**` base branches, so isolated end-to-end pull requests never trigger this repository's pipeline.
- The credentials test now rejects any GitHub-token secret outside the `AUTOMATION_*` names, a `pull_request` workflow without a dispatch trigger or the `e2e/**` ignore, and an unguarded release dispatch not listed as pending.
- The GitHub credential resolver now takes the `AUTOMATION_*` secrets as inputs. A composite action cannot read `secrets`, so the self-authored pull request workflow had failed before doing anything.
- Every pull-request workflow can be dispatched in checks mode, except four with named reasons. When the App layer allows it, an isolated end-to-end run gets its own private repository. At the default layer, cross-repository findings go to one tracking issue in this repository.
- The full test lane now runs as four parallel shards over the prebuilt test executables. The runner reports every failing target instead of stopping at the first one, and a failed data or census gate no longer hides the rest of the suite.
- The agentic CLI matrix starts every CLI that declares a config relocation variable from a relocated config with an empty HOME. The session must still reach the server through the recording proxy (#1161 R7).
- The browser worker logs the native `procedure_cache` miss (with `research_missing` from the cache-policy seed) for an unmodified catalog program request, and the JS derivation module gains `persist`, `load` and `explainAnswer` over an injected file io (#1165 R10, #1184 R9).

### Security
- The VS Code extension now packages with `@vscode/vsce` 4.0.0. The 3.x line pulled in `braces` (GHSA-vfj7-8cjw-p6xm), which has no patched release (issue #1169).
- Desktop packaging overrides `global-agent` to ^4.1.3, so `electron-builder` 26.17.0 no longer pulls `roarr` > `sprintf-js` (GHSA-hp3w-g68c-fv3c, no patched release).
- Overrides `proxy-addr` to ^2.0.8 in the root `bun.lock` (GHSA-jqcg-44mw-7w3h) and `compression` to ^1.8.2 in the end-to-end harness. Every lockfile now audits clean at `--audit-level=moderate`.

### Added
- `formal-ai benchmark run --offset N` and the `shard_size` input of `external-benchmarks.yml` run one upstream suite as concurrent graded windows whose counts add up. A shard cannot append to the ledger (issue #1177, R1177-12).

### Added
- Repository history (#1180 R11): the git subprocess, events store, cursor and incremental import have a JavaScript twin, `js/agentic/crate/history_store.mjs`, over an injected io.
- Obligation gaps (#1166 R4): every executor reports the obligations it cannot discharge, including output literals the request demands but the program does not print. This covers the catalog `write_program` arm in Rust, the JS agentic root and the browser worker.
- Generated versions (#1168 R8): `ci_workflow::render_with` / `renderWith` take an injected version set. A shared fixture pins the same rendered workflow in both runtimes.
- Code examples (#1164 R11): a supplied page whose query asks for the parts of its code examples is decomposed by the solver in both runtimes (parity case `e1164_supplied_page_code_example_parts`).
- Derivations (#1184 R9): the JS server persists each answer's derivation and gains `explain <id>`, mirroring `finalize_answer` and `formal-ai explain`.
- Prompt-text questions (#1172 R7): the answer projects the covering sentence into subject, predicate, object, time and place slots, and answers the asked slot first.
- Explanation research (#1172 R8) has a browser twin. The JS server reads the procedure cache and calls `cachedWriteProgram` (#1165 R10).
- Capability routing (#1175 R3): six more claim rows (supplied page, JavaScript program, incompatible units, fetch URL, navigation URL, calendar date signal).

### Fixed
- The shared `formal-ai-binary` CI action now keys its cached release binary by `rust/embedded/` and `data/seed/api-cache/` as well. A seed-only push could previously restore a binary that embedded the previous seed, so server parity compared the JavaScript server against stale Rust answers.

### Added
- The browser worker records a white-box derivation for every answer. It also answers `explain <answer id>` from the memory event log, matching the server byte for byte (issue #1184, R1184-9).
- Code debugging, explanation and review now claim a prompt only when it carries code. Summarization and rewriting claim one only when it carries the text to transform. Both runtimes enforce this (issue #1175, R1175-3).
- Statistics, word problems, arithmetic, compound interest, number constraints, unit conversion and calendar reasoning now claim a prompt only when it states the operand their own reader parses (issue #1175, R1175-3).

### Changed
- Every manifest moves to its publisher's latest release (issue #1169). Rust: links-notation 0.16.1 → 0.23.0 (the lino-objects-codec#60 hold-back is lifted), lino-objects-codec 0.7.0 → 0.8.0, lino-arguments =0.3.0 → 0.4.0, link-calculator 0.20.3 → 0.22.0, link-cli 0.2.11 → 1.0.0, web-search 0.5.0 → 0.6.0, command-stream 1.2.0 → 1.5.3, plus a full `cargo update`. npm: `@link-assistant/web-search` 0.11.1, `lino-i18n` 0.3.0, `marked` 18.1.0, electron ^44.6.0, desktop command-stream 1.5.0, puppeteer/puppeteer-core overrides ^25.12.0, and the `browser-commander` override 0.16.1 → 0.20.0.
- links-notation 0.23 reads `#` comments and reports syntax errors with their line and column. The seed adapter now uses the crate's `comments::strip_comments` in place of its own whole-line filter and returns the crate's located errors. The line-by-line locator the tests used for issue #1076 is retired. `data/meta/dreaming-lexicon.lino` no longer writes `#540` after an open parenthesis, where it would now start a comment.
- The browser bundle takes `lino-i18n/browser`. In 0.3, the package root wires Node file loaders. `js/i18n.js` loads its three catalogs through the library's `loadCatalogs`, which replaces its own fetch-and-merge code.
- `web-search` is taken with its recommended merge-only `merge` feature.
- The `has_bare_dot` guard in `rust/src/calculation.rs` is removed, along with the test that pinned it. link-calculator has rejected `2. 3` as a recoverable error since 0.18.0 (link-assistant/calculator#168).
- Agent orchestration runs commands through command-stream on Windows too. command-stream now keeps exact argv there (command-stream#190) and, since 1.5.1, stops the whole process tree on cancel, so the std-process fallback is removed. The desktop installs the VS Code CLI through the shared adapter's shell `{ file, args }` form (command-stream#191) instead of spawning `code.cmd` itself.

### Security
- Some releases are held back, each with a `"<name>//"` note in its manifest. Desktop command-stream stays at 1.5.0: 1.6.0+ and 2.0.0 depend on shelljs, which pulls in `braces` (GHSA-vfj7-8cjw-p6xm, no patched release). The `browser-commander` override stays at 0.20.0, because 0.21+ launches the installed Chrome by default (web-capture#160). The `@kreuzberg/html-to-markdown-node` override stays at 3.5.5, because later releases declare musl packages that npm does not have.
- The `proxy-addr`, `compression`, `global-agent` and `qs` overrides are still required. bun.lock and all three package-locks audit clean at `--audit-level=moderate`.

### Fixed
- The JavaScript repository-history importer now records the same ES `token_count` change and census-histogram ordering as the native importer, through a shared tokenizer module (`js/agentic/crate/es_tokenizer.mjs`). `formal-ai translate --from js --to ts --write` now also renders the agentic `.mts` twins (issue #1180, R1180-11).

### Added

- A `write_program` cache miss now rediscovers its program from documentation
  instead of answering from a stored template (issue #1165, R1165-1/R1165-2).
  `data/seed/coding-documentation-captures.lino` is pre-cached source data:
  the code blocks the page formalizer reads from byte-for-byte captures of the
  Rust Book, go.dev, kotlinlang.org and the Scala book, each pinned by SHA-256
  and re-derived by `scripts/generate-coding-documentation-captures.mjs`. The
  #1164 decomposer recomposes the page's example with the task's expected
  output bound into its literal, verifies it by decomposing it again and
  against the language's run contract, logs `procedure_cache
  outcome=discovered` with the page and content id, and stores the row in a
  configured runtime cache (`FORMAL_AI_PROCEDURE_CACHE`) so the next request
  is a hit. Both roots: `rediscover_from_documentation` in Rust and
  `rediscoverDocumentedProgram` in the browser worker.

### Changed

- The page formalizer keeps a code block's indentation (R1165-1): only the
  blank lines around a block and the indentation all its lines share are
  dropped, so programs recomposed from captured pages keep their layout.
- The Rust, Go and Kotlin Hello World programs are no longer stored in
  `data/seed/hello-world-programs.lino` or the browser worker's template table
  (R1165-4); they are rediscovered from the documentation captures, and the
  no-memorization ratchet falls from 14 to 11.

### Added
- Generated, split Mermaid system diagrams (overview, solver handlers, CLI subcommands, HTTP routes) from live data, with a drift gate (R382).
- `serve --debug-session`: a loopback-only, token-guarded protocol that pauses a turn between solver stages and advances it step by step from the VS Code debugger view, in both servers (R383, partial).
- The proof engine, number-constraint answers and coding guidance read their prose from seed data in every registered language (R379: 250 allowlist rows retired).
- Two more handlers migrated to seed rule sets (`algorithm`, `document_generation_plan`), with `operation`, `table` and `response` interpreter primitives in both runtimes (R918-2, R914-6, R1085-2).

### Fixed
- Formal AI (the repository's own agent) learned the edit shapes it failed while doing this work: a positional insert widens a fragment anchor to its whole line, reads position cues and target cues outside quoted payloads, and takes a colon-introduced block as the inserted lines; "set K to V" assigns a setting instead of overwriting its key; a quoted literal led by a description ("the heading '# Title'") stands for the literal; a sentence's closing mark is not file content; a path inside a quoted payload never outranks the file the request names; functions named by identifier can be removed whole.
- Fourteen native and eight browser routing probes now answer as intended (R1173-3): seeded English contractions, lock-picking legality, HTTP `GET`, `find` calculations, one-item change problems.

### Added
- `experiments/formal_ai_subagent/`: the coding agents' shared preamble, claims, gap list, task prompts, planner probe (`probe.mjs`) and git-ignored sandboxes now live in the repository, so Formal AI can read, edit and run them as a subagent on its own requirements (R1026).
- Formal AI line operations in both runtimes: numbered lines and ranges ("delete lines 2-3", "insert … after line 4"), a quoted line together with the line directly above or below it, moving a line (top, bottom, after or before an anchor), and swapping two lines.
- Formal AI replace and setting semantics: "change K from A to B" (including declaration lines such as `const K: T = v;`), "bump the version to …", whole-line replacement for "replace the line …", several lines or several insert clauses in one request, and "after the line Y that follows Z".
- The debug session records a Mermaid flowchart and the Rust and JavaScript source location of each stage's handler. The VS Code view renders the diagram (sanitized) and shows both source panes, and a held turn is neither persisted nor answered until it is released (R383, partial).

### Changed
- Two more handlers (`calendar_reasoning`, `calendar_create_event`) read their weekday names, parser vocabulary and sentences from seed data (R918-2: 48 migrated, 23 pending).

### Fixed
- Destructive Formal AI edits: an "Add … to <file>" description, a routed write in a removal request, an append of a block, and a replace scoped by "the line it follows" no longer overwrite the whole file. A computed change that would drop most of a file is refused.
- Diagram routing words inside a quoted payload no longer send an edit to the diagram recipe.

### Added
- A browser end-to-end test steps a held turn through the debugger view in the built app: the question goes through the composer, the SVG diagram and both handler panes are checked, the answer arrives only after the last stage, and the view is saved as the live session capture (R383).
- `experiments/formal_ai_subagent/local-gates.mjs` runs every CI gate that needs no Rust build, one at a time. It reads the gate registry (`data/meta/ci-gates/`) and the checker steps of the standalone workflows, so coding agents and Formal AI share one gate list with CI.
- `experiments/formal_ai_subagent/apply-census.sh` applies the self-AST census that CI regenerated for a commit or a run. It reads the census paths from the workflow's upload step.

- Formal AI searches file contents ("Where is add used in this project?", "Find TODO in the repository"), runs a named test file with the runner its extension needs, checks a reported bug against its stated expectation before changing anything, lists a file's exported functions, summarizes a named file, and lists the directory a request names.
- Formal AI edits: indented blocks keep their indentation relative to the anchor; ordinal lines ("the second line", "the last line"); "with these lines:" block replacements; several quoted lines in one insert or append; "the contents of a.txt" as the text to insert; "the last 2 lines of f" and "lines 3-5 of f" answer only those lines; inserts at the end or start of a Markdown section; empty-line inserts next to an anchor; quoted file renames.
- Formal AI answers a Spanish request in Spanish when the script detector would fall back to English.
- The debugger names the code that emitted each stage in both runtimes (`data/meta/debug-stage-sources.lino`, generated from the source with a drift gate), and a turn is held before it is solved (R383 delivered).
- Self-authoring, SWE-bench and the coding ladder record the same nine-stage trace of `data/meta/repository-workspace-protocol.lino`; the authoring loop runs the document's stages (R1138-3-5, Rust; the JS runners are still missing).
- Routing: a call to an undefined function is answered as an execution failure, JSON-to-JSON conversion re-emits the parsed document, a product request with no marketplace names the catalogued ones, and the browser learns from a named source (R1173-3: browser misroutes 9 → 2).

### Changed
- Three more handlers read their vocabulary and sentences from seed data: `text_manipulation`, `document_originality_check` and `software_project_followup` (R918-2: 51 migrated, 20 pending).
- `scripts/author-change-with-formal-ai.sh` refuses to commit unless it is given `--commit`, like `formal-ai solve` (R1138-3-7). The self-authoring action passes it, and `--no-commit` still means the default.

### Fixed
- Destructive Formal AI edits: "Delete the line 'x'" removes only the line that is `x` (all lines containing it only when the request says so), a nested quote inside a quoted line no longer turns an append into a move, a list of quoted lines is appended line by line instead of overwriting the file, and a different existing file is kept unless the request consents to overwrite it.
- Shell intent cues match whole words: the `move` cue inside "removed" no longer plans `mv`.
- A quoted line in an edit request no longer chooses a shell command: "Add the line '- run tests' …" used to run the workspace tests.
- Diagnostics mode no longer crashes the web app and the VS Code debugger view: the debugger view binds the JSX factory, and a test now fails on any app file that writes JSX without it.
- The debugger view gets its own row in the chat panel, so the message list no longer covers its buttons.
- An answer that echoes a literal holding backticks now fences it with a longer CommonMark code span, so the Markdown shows it verbatim.
- The file-size gate measures only the files a commit can carry, so a git-ignored local file no longer fails it. Claude Code session exports at the repository root are ignored.

### Changed

- PR #1188 round 15, Formal AI as a subagent working on its own requirements:
  - **No local Rust builds.** The gate runner `experiments/formal_ai_subagent/local-gates.mjs` now runs a script's JavaScript twin (`scripts/<name>.mjs`) in place of `rust-script scripts/<name>.rs`, and leaves scripts without a twin to CI. Twins with CI parity gates now cover:
    - the requirement pipeline: assemble, generate status, render, check;
    - file size, the debt ratchet, hardcoded language, the minimal-core boundary, worker line budgets and the changelog fragment check.
  - **The response-language debt is measured over every `data/seed/multilingual-responses*.lino`,** not one file. Moving an intent between files can no longer hide it. The honest ceiling is 436, and `scripts/generate-response-parity-debt.mjs` writes the ledger. Code templates (`language rust`) owe no translation.
  - **Edit composer (TEACH-F), fixed in JS and the Rust twin:**
    - unpaired quotes are declined before any route reads the payload (G71, unsafe);
    - contractions are expanded in both roots (G70);
    - an unquoted addition asks a question instead of writing (G69);
    - a whole-line anchor is widened until it is unique (G73), and a repeated anchor is reported with its count (G72);
    - "followed by an empty line" writes that line (G74);
    - a listing is detected by its command word (G75);
    - a Replace whose new text contains the old text is applied in one pass (G78);
    - a scoped Replace edits only the named part, or declines (G79);
    - a list of lines is replaced as one block (G80);
    - unquoted anchors (G51), assertions in the file's own style (G13) and stated-expectation tests (G25) work;
    - a backticked command followed by prose runs only the command (G76, unsafe);
    - the timeout report is honest (G77);
    - line moves work (G81);
    - multi-sentence copy-and-replace requests are planned sentence by sentence (G82);
    - words inside file names are not request words (G83).

    The planner arms are back to 58.
  - **Routing (ROUTE3):**
    - "a program that prints <text>" is answered by the documented hello_world procedure with the operand. No program text is stored.
    - Native recurrence requests read the Wikifunctions recurrence cache offline.
    - GitHub traffic is asked before promotion.
    - Both misroute ceilings are 0.
  - **Handlers (MIGRATE4):** `diagnostic`, `coreference`, `verifiable_task`, `nl_tool`, `installation_conversion` and `translation` are migrated to seed data. 57 are migrated and 14 pending. The browser gains honest twins in place of "unknown".
  - **Discovery (DISCOVER):** check and run commands come from captured documentation pages (R1165-6 implemented). No language row restates a command that a page states.
  - **CI fixes (CIFIX2):**
    - file-creation requests write without reading first;
    - an explicit grep runs as written;
    - Spanish articles are never read as directories;
    - `solver.rs` is split below its warning band;
    - the debug-session test helper is fixed.
  - **Upstream:** link-foundation/meta-language PR #216 (issues #202, #203) translates sibling items and relative imports. Measured on our tree, it raises the JS -> Rust translated items from 254 to 657.
  - Formal AI did the ledger rows, gap marks and most single-line edits. Its new failures are logged as gaps G73–G86.
- Round 15, second push:
  - **CI speed:**
    - `scripts/check-ci-speed.mjs` (gate `check-ci-speed`) enforces the 30-minute job limit, with 15 listed exceptions, and rejects sharding by list position.
    - `scripts/plan-test-shards.mjs` splits tests longest-first from recorded durations. The JS tier runs `node --test` with full concurrency.
  - **Requirement audit:**
    - 106 distinct requirements were mapped (`docs/case-studies/pull-request-1188/requirement-coverage.md`), and every owner message was collected (`user-messages.md`).
    - New rows R1188-U1 to U26 in `docs/requirements/issue-1188-user-requirements.md`: architecture, naming, notation, CI speed, text understanding (web formalization, round-trip translation, requirement extraction, dependency summarization), readable multi-line code, minimal local tests, disk care.
  - **Seed registry:** `scripts/generate-seed-registry.mjs` is the JS twin of the registry generator, with a parity gate. `meanings-coding-assertions` is now registered. The JS planner reads its response files from the registry instead of a hand-kept list.
  - **Responses:**
    - 54 Spanish agentic responses were added, applied by `experiments/formal_ai_subagent/add-translations.mjs`.
    - The `local_search_*` responses now live in `multilingual-responses-local-search.lino`.
    - Response debt is 382.
  - **TEACH-F round 4:**
    - nested «» pairs are part of the payload, and an unnamed write target is refused (G86, unsafe);
    - "everywhere" replaces every occurrence (G84);
    - a repeated Replace answers not-found or already-replaced (G87);
    - a line-range move between files is a move of lines, never `mv` (G85).
  - **CI fixes:** a non-exact-size iterator in `test_assertion.rs`, plus two clippy errors.

### Added

- PR #1188 round 16:
  - **Requirement extraction (R1188-U20):** `js/agentic/crate/requirement_extraction.mjs` and its Rust twin list the requirements an issue states: units with a seeded obligation cue, units opening with a directive verb, task-list checkboxes, and items under a heading naming requirements, deduplicated by content-word overlap. The vocabulary (`data/seed/meanings-requirement-extraction.lino`) covers five languages. `scripts/measure-requirement-extraction.mjs` scores it on 138 cached issues against 1350 reviewed shard rows: recall 0.411, precision 0.406. The floors in `data/meta/text-capability-ratchet.lino` may only rise; gate `check_requirement_extraction`.
  - **Readable code (R1188-U23):** `scripts/check-readable-code.mjs` (gate `check_readable_code`) fails packed files and lines over 300 characters of code under `js/`, `ts/`, `rust/`, `scripts/` and `packages/`. Exemptions need a reason and may only shrink. `translate-es` now carries the JavaScript layout through the pivot, so every `ts/` twin keeps its source's lines, indentation and comments. Long JSX lines are wrapped. The meta-language projections stay exempt until link-foundation/meta-language#217.
  - **Response languages:** 1,272 ru/hi/zh/es response rows were added. The response-language debt falls from 382 to 0: the 16 machine-text templates (code, log lines) are marked `audience machine`, which the ledger generator and its Rust twin test exempt. `multilingual-responses-synthesis.lino` is split into web, procedural-how-to, installation and documents files.

  - **Architecture principles (R1188-U2):** `data/meta/architecture-principles.lino` maps the 51 universal principles of link-foundation/code-architecture-principles to how the repository applies each, the gate that enforces it (37 have one) and the requirement row tracking any gap. `scripts/render-architecture-principles.mjs` renders `docs/architecture/principles.md`; gate `check_architecture_principles`.
  - **The 2026-10-08 vision, verbatim (R1188-U14):** a new architect note quotes the owner's messages byte for byte from the collected `user-messages.md`. `VISION.md` folds them in, and a test proves the quotes are exact.

### Changed

- The JS meaning lexicon reads its file list from `data/meta/seed-registry.lino`. The hand-kept list had missed `meanings-response-intents-handlers`.
- The web UI boundary ratchet counts non-whitespace characters, so wrapping a line no longer changes the measure.
- JS -> Rust translated items: 254 -> 263.
- CONTRIBUTING's pull request process asks for the tests next to a change and the JavaScript gate twins locally, not every test; CI runs the rest.
- The local web E2E suite runs as three Playwright `--shard` legs of a 30-minute job instead of one 40-minute job, so CI speed carries 14 exceptions.

### Changed

- PR #1188 round 17:
  - **Formal AI never takes an action it was not asked for (R1188-U17),** in both roots:
    - a replace list reads its target from any clause (G89), and compares quoted segments after unescaping prose newlines (G88);
    - an edit that names two or more target files outside quotes is declined with the seeded `request_several_edit_targets` answer (G91);
    - the reply language is detected from the text outside quotes (G92);
    - a cue word quoted whole is data, not a request word;
    - a routed Write that carries an unquoted edit cue is not a whole-file write;
    - an odd `"` after a letter counts as unpaired.

    A property test of 240 generated requests per root (seed 1188) pins it.
  - **Requirement listing chat route (R1188-U20):** "List the requirements of this issue: <text>" lists what the extractor finds, in five languages. It is a `requirement_listing` rule set with two generic rule-language additions: the `command_head` subject and the `value <name> transform <transform>` source. A pasted phrase never routes.
  - **Text capabilities (R1188-U18, U19, U21):**
    - formalization of whole web pages;
    - round-trip translation over all 20 ordered language pairs;
    - dependency-based summarization.

    Each has a measured ratchet that only rises.
  - **Planner twins (R1188-U16):** `scripts/check-planner-twins.mjs` (gate `check_planner_twins`) counts the functions `js/agentic/` exports with no Rust twin, by snake_case name, a `Mirrors` citation, or a declared Rust built-in or dependency. All 88 that had none now have one, so the ceiling in `data/meta/planner-twin-ratchet.lino` is 0:
    - 14 gained a citation;
    - 11 were extracted from inline Rust into named functions, which also removed duplicated code;
    - 27 were declared as standing for a Rust built-in or a dependency crate;
    - one Rust twin was written: `module_from_document`;
    - one unused export was deleted.

    Formal AI made the 55 doc-comment and ledger edits. It failed twice, and both were fixed in both roots: G96, an anchor holding a literal `\n`; and G97, a Spanish marker matched inside a file name.
  - **Long tests first and parallel at test level in the browser suite (R1188-U10, U11):**
    - the local web E2E legs are planned longest-first by `scripts/plan-test-shards.mjs` from `data/meta/playwright-test-durations.lino`, recorded from CI by `ci-durations.mjs --playwright-files`;
    - the config runs its long specs first in a leading `chromium-long` project;
    - the speed gate now rejects Playwright's count-based `--shard=i/n`.
    - the suite runs `fullyParallel` (R1188-U11). The two specs whose tests share state made in `beforeAll` keep their tests in one worker.
  - **Formal AI tally (R1188-U13):** `scripts/tally-formal-ai-dogfood.mjs` generates `formal-ai-tally.md` from the dogfood ladder: tasks, outcomes and resolutions, per agent. Gate `check_formal_ai_tally` fails on an unresolved failure, and on a fixed one whose regression test is neither named in its row nor cites its id.
  - **Generated files are checked in CI (R1188-U8):** `scripts/check-generated-files.mjs` (gate `check_generated_files`) fails on a file whose header names a generator that no gate or workflow runs, so a hand edit of a generated file cannot be merged. New gates run three checks that no gate ran before: `check_llm_task_parity`, `check_fact_captures` and `check_requirement_coverage`.
  - **Generalize, don't specialize (R1188-U1):** `scripts/check-prompt-specialization.mjs` (gate `check_prompt_specialization`) counts the prompts that tests send to Formal AI held verbatim in code. These are branches or canned answers keyed to one prompt. The ratchet holds the count at 23 and it only falls.
  - **Requirement audit:**
    - the coverage map now includes the owner's 2026-10-08 messages (R1188-U18 to U26), 115 requirements in all;
    - R1188-U15, U22, U24, U25 and U26 are implemented: the working rules are pinned to their record and to the tools that keep them by `rust/tests/web/pull-request-1188-working-rules.test.mjs`;
    - the closure audit grounds the `machine` response audience.

### Changed

- PR #1188 round 18:
  - **Generalize, don't specialize (R1188-U1).** None of the test prompts in the solver and worker code remain there verbatim (23 → 0). Their words and answers moved to seed data:
    - the behavior-rule catalog: 49 `behavior_rule_*` intents in five languages (`multilingual-responses-behavior-rules.lino`), which both roots read by intent. The browser's list and detail answers now match the native ones byte for byte.
    - the rule-detail openings: a seeded `rule_detail_request` role.
    - the browser capability listings: the seeded `capabilities` responses, which fixes hi, zh and es.
    - the knowledge-export example prompts: `example` lines in `intent-routing.lino`.
    - the browser translation table: `translation_phrase` meanings (`meanings-translation-phrases.lino`).

    The specialization gate now also scans the app's `.jsx` files. It finds 17 pairs there, which are the ceiling now. Other ratchets fell:
    - hardcoded-language rows: 788 → 700;
    - JS worker literals: 645 → 445.

    `behavior_rules` is migrated to seed (R344: 58 migrated, 13 pending).
  - **G98 (destructive) fixed in both roots.** "Change 'a' to 'b' in the file f.txt" wrote `'b' in the`. The new text now ends before any article or function word that comes ahead of the target cue.
  - **Meaning language parity (LEXEMES):** the language-parity debt falls from 611 to 43 rows (from 910 at the start of the batch).
    - Spanish lexemes were added across the meaning files, plus ru, hi and zh where English stood alone.
    - The lexicon importer now takes the registry's partial languages (es) from the cached Wikidata labels as optional surfaces. It packs shards by line budget, and its offline reproduction regenerates the five import shards by rule.
    - Files that passed 1500 lines are split by category: number words, conversation summary, software requirements, language protocol and decomposition ladder.
    - Meanings that share a surface are ranked by languages, then by fewer surfaces, then by declaration order.
    - The round-trip measure divides by all content terms, so learning a word can no longer lower it. Term survival is 0.436.
    - The remaining gaps are deliberate, each with a reason: relational words that Spanish says with function words, and structural single-language records.

Ground source/report artifact roles and new lexical surfaces in loaded meanings. Keep notation checks consistent with variable-backed Rust template keys, migrate coding-ladder caller labels, and synchronize embedded project response records. Preserve actual Formal AI failures and checked integration evidence.

Verify complete source network serialization in eight JavaScript CI shards and eight native Rust shards. Export full compressed Links Notation networks with checked source/network/packet identities; preserve exact source bytes and distinguish working-tree inputs from a committed head.

Fix closure reference scanning for escaped quotes and backslashes inside captured source literals. Preserve exact captured source identities as quoted scalars and ground the three source-qualified response meanings in the shared five-language seed. Three parser regressions preserve real bare-reference discovery; closure stays at the existing1035 ceiling.

Provision actual ripgrep before JavaScript suites and browser coverage so agentic search observations use a working external adapter.

### Changed

- PR #1188 round 19:
  - **No job over 30 minutes (R1188-U9).** Two of the listed exceptions are gone:
    - the agent CLI e2e workflow runs its computer-use record and generalization phases as their own matrix legs, each under a 30-minute cap;
    - the GitHub Pages build (web bundle, `cargo doc`, stamp, upload) moved to the reusable `.github/workflows/pages-artifact.yml` under 30 minutes. The release workflow's `deploy-pages` job now only deploys that artifact, capped at 15 minutes.
  - **Meaningful file names (R1188-U5).** The 25 numbered browser worker modules are named by subject (`formal_ai_worker_00.js` → `formal_ai_worker_seed_responses_and_language.js`, …), together with their `ts/` twins and line-budget shards. The renames are recorded in `data/meta/rename-map.lino` and applied by rule (`experiments/formal_ai_subagent/rename-by-rule.mjs`, gate `check-rename-map`). The modules now load in any order.
  - **Readable links notation (R1188-U6, R1188-U7).** `scripts/measure-notation.mjs` measures abbreviated names, `_` against `-`, and duplicated lines across the `.lino` files. The counts are held by `data/meta/notation-ratchet.lino` (gate `check-notation`), the rules live in `data/meta/notation-rules.lino`, and the style is described in `docs/links-notation-style.md`.
  - **A valid, testable release (R1188-U27)** is recorded as a requirement of its own.
  - **Spanish capability questions (R1188-U1).** "¿Puedes traducir?" used to fall through to the unknown answer in both roots. A capability question is now read in its own language whenever the seed carries cues for it, and only otherwise as English, instead of through an `ru`/`zh`/`hi` switch. The features gained Spanish labels, and the Spanish answers quote the label (`La función «traducción» está disponible…`). Pinned by `rust/tests/web/feature-capability-languages.test.mjs` and the native case table.
  - **One answerer in the browser (R1188-U1).** The four `js/app/local-*.jsx` modules (about 1450 lines) were a second, smaller answerer the page kept for when the worker was missing. The page always creates the worker, so they are deleted, and a request made before the worker exists waits for it. When the desktop server returns an empty answer, the worker answers instead of a canned text. The sidebar example prompts are data (`js/app/example-prompts.lino`). Test prompts held verbatim in code: 17 → 0.
  - **Dependencies point inward (R1188-U2).** The new gate `check-dependency-direction` holds two counts to falling ceilings in `data/meta/dependency-direction.lino`: solver code under `rust/src/` that names a surface module (CLI, server, Telegram, client integrations; 38), and `js/agentic/` modules that import `node:` built-ins or the Node server outside the listed host adapters (19). 39 of the 51 architecture principles now have a gate.
  - **Requirement outputs named by area (R1188-U5, R1188-U7).** The requirement-status ledger is one file per requirement shard. Each file states its shard, its issue and the values most of its records share once, and a record lists only what differs. The ledger falls from 461K to 224K characters. The 88 CI gate names now use `-`.
  - **The progressive JPEG method (R1188-U28 to R1188-U30).** The owner's direction is recorded in a new architect note, in `docs/progressive-delivery.md`, in `VISION.md` and in `CONTRIBUTING.md`: every problem and every pull request are complete at every moment at some resolution, and each pass raises the lowest level first.
    - `scripts/render-progressive-plan.mjs` (gate `check-progressive-plan`) generates `docs/progressive-plan.md` from the requirement ledger. It shows every requirement on its level (recorded, measured, partial, implemented) and which root's test pins it (JavaScript 230, Rust only 1116, none 23).
    - JavaScript goes first, and translation between JavaScript, TypeScript, Rust and the meta language is automated, with recorded temporary workarounds where the meta language cannot translate yet.
    - The requirement-status generator now honours a status that opens with its verdict: "Partial: … delivered …" is partial.
    - The Formal AI tally sets each agent's edits by hand beside its delegated edits.

# PR #1188: CI documentation, response templates and Pages trust

- Repair Rust documentation, formatting, ordered module declarations and Clippy diagnostics reported by the first CI compile of the current changes.
- Carry renamed handler captures through localized response placeholders and browser bootstrap templates. Repository traffic answers now render both official documentation links in all five languages, for named and unnamed repositories. Preserve Rust formatting variables and unrelated template fields.
- Build Pages only from the trusted triggering ref, accept a full commit SHA, and verify that the checkout matches the selected release commit before executing repository code. Fail explicitly if the release branch moved.

Validation: 17 nearest planner regression tests and 13 notation/Pages tests pass; standalone Rust formatting and the notation gate pass. All source edits were executed by Formal AI, or by the existing notation and TypeScript generators. Rust compilation and CodeQL verification run in CI.

## Fixed

- Repair quoted edit payloads through the JavaScript Formal AI planner: scalar code spans preserve comma-separated backticks, copy/move clauses precede quoted edits, file paths inside literal content do not become targets, and exact quoted content retains indentation and its final newline.
- Keep objective labels inside quoted source as data, and preserve new-only source newline escapes. Both JavaScript and Rust roots share these general repairs, with focused JavaScript and native regression tests.
- Record the limits of the Formal AI only development experiment: feature-level repair synthesis still needs a parent-supplied implementation; the earlier multi-line answer truncation was not reproduced. The JavaScript source-effect replay reaches 31 of 32 Issue 1028 leaves; native compilation and the full CLI ladder remain CI checks.

Reuse the base image account that owns UID1000 as the slim runtime box account, preserving its UID and group ownership while moving its home to /home/box. Create the account only when that UID is unallocated, and verify the final account identity during the image build. This repairs the actual Ubuntu26.10 useradd collision without running the service as root.

Define report and document through the existing entity, knowledge, and observation concepts so evidence artifact definition edges have real semantic parents. Generate the role registry from actual declared meaning roles. Keep existing five-language artifact vocabulary and unchanged translation ratchets; the actual semantic reference graph closes and the original round-trip suite passes.

Native seed metadata coverage now reads comment and blank headers using the same substantive root rule as the production audit. The coding metadata floor and gap assertions remain intact.

The browser worker now ships a checked response projection from canonical seed data for offline startup. Greeting variants and all baseline languages come from seed rather than copied source strings; seed synchronization verifies and regenerates that projection.

The Pages artifact caller releases superseded builds using its own concurrency group. Debug-stage coverage pins the observed absence of a browser raw emitter for native rule verification. Routing fixtures identify the complete verified copy/move recipe rather than a preparatory directory recipe; all requested-command comparisons and destination checks remain intact.

The migration ledger follows current production rank order with identical records and statuses. Native census pins now include the already-delivered seed rule requirement_listing:59 migrated,26 justified native,13 pending; debt ceilings stay unchanged. Historical shell replay uses its captured tool schema.

### Fixed

- Render removed lines and declarations as explicit lists while preserving backticks and placeholder text inside each value.
- Substitute seeded editing summaries once so literal scalar and list content cannot become another template.

Generated Rust source requires an observed destination preimage, preserves existing bytes without replacement consent, and refuses unreadable or unobserved targets. Identifier digits cannot become requested numeric values. Shell parsing correctly treats quoted program backticks as data, preserves opaque commands, and keeps semantic web searches out of the shell.

### Fixed

- Repair Rust test lints and pin complete scalar editing summaries.
- Run macOS archived tests from the Cargo workspace source root and keep platform selection assertions current.
- Keep prebuilt test execution on stable interfaces while preserving existing shard duration inputs.
- Preserve variable-backed template keys during notation migration and fully render procedural install answers.
- Render explicitly listed replacement lines separately and route multiple-file edits through advertised capabilities.
- Match workflow jobs at their real indentation even when dispatch inputs share their names.
- Preserve native line-plus-blank insertion, classify bare seeded shell commands, and validate formal rewrite escapes with their own grammar.
- Bind declared filesystem options and shell glob collections to verified operands, preserve force consent, and report attempted mutation failures honestly.

Historical file-creation and failure fixtures now observe the named auxiliary plan event before testing the requested target. Target bytes, bounded retry behavior, nonzero verification and wrong-output checks remain enforced. Write-only clients retain an explicit missing-receipt outcome; shell fixtures execute the unchanged append command and read its real bytes.

## Fixed

- Keep quoted whole-document edits ahead of skill-teaching cues in their payloads, while preserving genuine teaching and malformed-quote guards (G125).
- Restore concise-lexeme parsing in the frozen source-test root, disambiguate requirement metadata identities, and align held-out family identifiers with the runtime.
- Require whole-word calendar-day evidence outside CJK so words such as Wikipedia and India cannot become calendar requests.

These changes were applied through JavaScript Formal AI. The original G125 request now succeeds unchanged in a selected-file sandbox; broad feature repair synthesis (G112) remains open. Closest JavaScript checks pass; native compilation and full suites remain CI observations.

## Fixed

- Exclude complete seeded commit cue spans when checking for independent source authoring, so existing-tree commits route correctly in every seeded language. Independent authoring, quoted commit payloads and larger-token boundaries retain their guards.
- Seed contracted avoidance forms, and expose named text-transform metadata alongside its original content-addressed provenance link in both event-log roots.
- Promote proof directives only from leading seeded directive forms, preserving genuine proof requests in five languages while embedded Spanish test wording retains its workspace capability reading.
- Verify 17 closest JavaScript checks, exact package seed mirrors, owned Rust formatting and diff whitespace. Native retries remain CI; the worker solver's general workspace-capability fallback and broad Formal AI feature repair planning remain open.

Observed-source callable discovery now reuses the JavaScript/Rust lexer to retain complete declaration spans, UTF-8 byte spans, module/declaration hashes, import/export aliases and unresolved contract evidence. It does not certify unknown types or effects, or synthesize the selected-summary wrapper.

Internal final outcomes distinguish audit reports from complete source artifacts. Source operands require a matching successful syntax receipt; ordinary observed findings can still be delivered to reports. Explicit successful shell stdout observations preserve compact JSON and whitespace while friendly result rendering remains unchanged.

Validation: 50 focused JavaScript checks pass; native sources pass standalone formatting. The unchanged composition fixture remains 3/10 passing with no wrapper authored, so the broad synthesis capability gap stays open.

### Changed

- PR #1188 round 20:
  - **The release test job meets 30 minutes (R1188-U9, R1188-U10).**
    - The specification shards no longer compile their own test binary (220-362s a shard, 1201s cold). They run the unit executable that `build-artifacts` compiles once, through `scripts/run-prebuilt-tests.sh` (`TEST_SUITE=specification`).
    - The full lane runs five shards and the specification lane four, planned longest-first from per-test durations re-recorded from CI (the longest test takes 520s).
    - The job's cap fell from 90 to 30 minutes: 83 of 94 jobs now meet the limit.
    - The unsharded macOS specification leg is gone: macOS runs only the tests whose behaviour can differ from Linux (issue #1059).
  - **JavaScript first (R1188-U29).**
    - When a requirement row names tests in both roots, the requirement ledger now cites the JavaScript one, so the requirements pinned only by a Rust test fall from 1116 to 921.
    - `js/agentic/crate/least_action.mjs` is the JavaScript twin of the least-action scoring (issue #491). R918-2 (the handler inventory) is delivered and pinned by a JavaScript test.
  - **The Rust specification suite asked of the browser worker (R1188-U29, R1188-U30).**
    - `scripts/lib/rust-specification-cases.mjs` reads every specification test of the simple shape (a prompt, then assertions on the answer, the intent or the evidence links) into a JavaScript case, which is the rust → js direction of the automated translation.
    - The gate `check-specification-in-javascript` asks the worker each case: 136 of 1169 are carried and 102 pass.
    - The 34 failing cases are listed in `data/meta/specification-javascript-gaps.lino` as the worker's parity debt. The number that pass never falls.
  - **Recorded translation workarounds (R1188-U30, R994).**
    - The js → rust leg runs meta-language at eb0574d5, which translates imports and sibling calls.
    - Where it cannot translate yet, a recorded workaround in `data/meta/translation-workarounds.lino` does, each linked to the upstream issue that retires it.
    - The CI compile builds one crate per root.
  - **The progressive plan's ratchet (R1188-U28).** A recorded, measured or partial level may grow only by newly recorded requirements and by rows that rose from below. A pass that refines a high level while a lower one grows now fails `check-progressive-plan`.
  - **Readable names (R1188-U4, R1188-U5).**
    - The abbreviation rule has a measure and a falling ratchet (gate `check-abbreviations`).
    - Twelve file names are spelled in full words.
    - The lexicon-import shards and the changelog archives are named by what they hold (`meanings-lexicon-import-actor-to-diamond.lino`, `releases-from-0.1.0.md`).
  - **Text capabilities in chat (R1188-U18, R1188-U19, R1188-U21).**
    - "Summarize …" answers through the dependency summarizer in both roots. The browser worker runs the JavaScript crate module itself, through generated `js/worker/formal_ai_worker_crate_*.js` copies kept current by the gate `check-worker-crate-modules`.
    - "Formalize <url>" and "formalize this page: <url>" fetch the page and answer its statements, in five languages, from a seeded `page_formalization` meaning. Offline, the answer says so and formalizes nothing.
    - Chat translation chooses, among the surfaces a dictionary offers, the one whose meaning comes back through the seed lexicon (`round_trip_choice` / `roundTripChoice`); the primary surface wins ties.
  - **Requirement extraction (R1188-U20)** reads a definition-of-done label ("Fixed means:", "done when", and the same in Russian, Hindi, Chinese and Spanish) as an obligation: recall 0.411 → 0.419 and precision 0.406 → 0.407 on the 138-issue benchmark.
  - **The last three test files named only by an issue number** are named for what they pin (R1188-U5).
  - **CI.**
    - Clippy 1.99's new `doc_markdown` and `significant_drop_tightening` findings are fixed.
    - `audit-seed-metadata` finds the `meanings` root after a comment header, and has a JavaScript twin.
    - The browser app registers its offline service worker only after the engine is ready, so the precache no longer delays the first answer past the end-to-end tests' wait.
  - **The browser worker passes 122 of the 136 carried Rust specification cases (was 102, R1188-U29).**
    - Its answers carry the native evidence links, through a JavaScript twin of `build_evidence_links` (`js/agentic/crate/event_log.mjs`).
    - The policy gates (bounded autonomy, destructive actions, agent time budget, cache flush, add-only history, inappropriate content) read seeded trigger words and responses in five languages in both roots.
    - Project lookup follows the two phases `data/seed/method-execution.lino` declares.
  - **Formal AI edit gaps fixed in both roots:**
    - nested same-mark quotes are declined (G90);
    - "then" sequences of edits are applied step by step (G99);
    - a create request that describes the file is declined safely (G102);
    - every file of a coordinated list is counted (G104);
    - an edit clause that cannot be planned is named (G106);
    - a backticked payload runs to its own close (G107).
    The several-files guard reads only the clause that holds the edit, which restores the Issue 1028 ladder's proof leaves.
  - **The `check-architecture-contents` gate** keeps ARCHITECTURE.md a table of contents whose links and anchors name the topic files (R1188-U2).
  - **Handoff.** `docs/case-studies/pull-request-1188/handoff.md` records the owner's requirements, the methodology and what is left, for the next agent.
  - **Formal AI.** 31 dogfood rows whose gaps were fixed or not reproduced later now say so. 160 of 165 recorded failures are fixed.
  - The Formal AI tally counts "Not fixed …" as open.

### Fixed

- Build native release targets once from one selected source and compiler contract, then verify executable and extracted CLI archive bytes before desktop and CLI packaging.
- Capture coherent release helpers from the workflow commit so older selected tags remain buildable; heal missing version-specific VSIX and checksum/provenance manifests.
- Prepare the default-feature test-profile library before the unchanged timed doctest step; bind full Docker release metadata to the checked-out release commit.
- Correct native lint diagnostics and preserve exact gate identity and answer-field contracts. Native CI and publication verification remain required.

### Fixed

- Verify the immutable pushed slim image digest against the actual release source commit, version and executable SHA-256 before runtime smoke checks. Attach the verified image receipt and checksum to the stable GitHub release.

# Observed callable request discovery

Requests declaring a callable and a source destination now read the destination, immutable source operands and named acceptance file before reporting unsupported contracts. The internal outcome retains request-local content identities and typed read failures; it cannot be delivered as executable source. File names and declaration parameters cannot supply authoring cues. Existing arithmetic module/test synthesis is preserved.

The original composition remains unimplemented; this change stops at an honest MissingContract outcome and does not certify callable types, effects, optional guards or rollback.

Validation:21 closest JavaScript checks and5 existing arithmetic route tests pass; both Rust modules pass standalone formatting. Native regression pins await CI compilation.

Verify published stable release tags against the repository default branch before native compilation, and independently recheck event-bound source authority before consumers use downloaded native source. Preserve read-only pull-request merge previews, actual head and pinned base prerequisites.

Publish byte-bound source, protocol, eight native target receipts and actual macOS signing observations with release checksums and workflow attestations. Automatic repair validates complete published evidence and package digests before deciding a release is already complete.

Keep the macOS archive producer and consumers bound to an explicit matching test profile with optimizer level1 and all runtime debug/overflow checks. Retain unit, integration and source targets including installation conversion tests, and omit empty lib/bin harnesses. Cold native compilation remains verified by CI under the original budgets.

Keep declared free text as the owner of summarization requests across languages. Conversation vocabulary inside supplied text cannot redirect the request to conversation history; bare and explicit conversation requests retain their existing behavior. Extract the conversation request classifier from the write-program/research worker into its named module without increasing existing module ceilings.

Record compact-input coding and self-coding requirements and source-bound work measurements, retaining failures and supplied-patch attribution. Correctness remains mandatory; no token savings or autonomous coding baseline is claimed.

Observe complete callable declaration spans, module/declaration content identities and actual export/import bindings before attempting composition. Supported scalar and Optional return bodies produce typed guarded graphs; unsupported structural schemas, calls, input writes and module initialization/import effects remain explicit contract gaps. The request-local discovery outcome exposes its actual typed Gap/Failure witness, while the public final-plan projection stays unchanged.

The original selected-summary composition remains unimplemented (real acceptance3/10). Closest JavaScript probes pass41/41, native fixtures register from the integration harness, and native compilation remains a CI check.

Formal AI now keeps declared source payloads separate from instructions when creating or replacing whole file contents, preserves their exact bytes, and reads longer Markdown fences around embedded examples. Both JavaScript and Rust planners retain ordinary replacement edits and quoted source data.

The generated browser crate factories preserve named re-exports and load response lexicons alongside meaning lexicons. The browser dispatches the existing seeded procedure compiler and records real capability gaps and review-only learning proposals, raising carried native specification parity from 122 to 124 passing cases out of 136; the twelve remaining failures stay recorded.

Literal verification also treats exact authored failure narratives as data while retaining explicit harness errors and nonzero exit failures. The previously failed evidence-writing request completes on retry.

Closest JavaScript checks: 31 passed. Native source formatting and scoped worker budgets pass; Rust compilation and native regression checks run in CI. Feature-level source repair synthesis remains an open capability.

Formal AI now recognizes the shell listing command ls as a standalone token. Coding requests that mention calls, details, signals or goals keep their explicitly named file read instead of listing the workspace. Genuine list-then-read requests retain their order in both JavaScript and Rust.

The original multi-turn specification-reader implementation request now reads its named source on retry, but feature-level source synthesis remains open: the request still makes no source change. Two focused JavaScript regressions pass, and the native twin is formatting-clean for CI.

Preserve exact captured source bodies, digests and times while fixing quoted reference syntax. Expose verified registry decoding through the real capture boundary and keep its tamper/duplicate/time tests outside production source. Native canonical parser validation remains pending CI.

Formal AI now strips complete seeded content leads and their word-bounded qualified suffixes only before a clause separator. Exact-content Set requests write the declared quoted bytes, including multi-paragraph source, instead of writing instruction words and quote markers. Both JavaScript and Rust read the same existing multilingual seed leads.

Eight focused authored-literal tests pass, covering eight lead forms across the five supported languages, preserved quoted payload text, ordinary leading prose and real verification failures. The original CI task-document request completes unchanged on retry with byte-equal output. Its pre-repair transcript was not saved; the observed malformed artifact and explicitly attributed failure summary are retained alongside the complete retry evidence.

Literal-file parsing now requires the write action and target to belong to the same statement, preventing read instructions and unrelated prose from authorizing destructive writes. Write-only clients can receive their requested file when auxiliary event persistence is unavailable; the final result retains a typed verification gap. Closest JavaScript checks pass 36/36; native execution awaits CI.

Multiple-file progress now judges exact verification payload bytes against each declared file, keeping raw transcript evidence unchanged. Write acknowledgements, incidental path mentions, nonzero statuses and stale successes cannot certify file contents. The unchanged original two-file request now writes and reads back both artifacts; it still reports unsupported clauses honestly. Closest JavaScript checks pass 34/34; native execution awaits CI.

Closed quoted operands and seeded explicit content qualifiers now preserve punctuation-only file payloads. Sentence boundaries ignore closed literal spans while retaining original positions and exact bytes in both runtimes; malformed and empty operands remain unlicensed.

Retire CLI and desktop build timeout exceptions after all five CLI and six desktop jobs passed with independently verified same-run native binaries. All package jobs now enforce thirty-minute caps; historical timings and source/startup/byte checks remain intact.

Explicit literal requests with multiple declared artifacts now share the existing obligation transaction before semantic recipes inspect payloads. Completion requires independent successful readbacks for every artifact; unavailable tools and underivable mandatory clauses remain honest gaps. Both runtimes preserve single-file routing and public final projections.

Build and verify full and slim containers on native AMD64 and ARM64 runners with exact same-run source/binary receipts, separate independent full compilation, real runtime/size checks, and anonymous immutable multiarch manifest verification. Call publication after actual stable release creation. Explicit source event/purpose contracts preserve PR and manual validation while retaining independent stable release authorization. Actual cold/runtime/publication evidence remains pending CI.

Require independent seed-role evidence outside the named canonical surface. Reuse one normalized bounded seed pronoun/antecedent matcher so real-history coreference reaches its handler before general meta reasoning and quoted payloads or embedded aliases do not invent context. Preserve original whole-program assertions; native execution remains pending CI.

Decode real ordered answer fields with the same links-notation parser used by the production formatter. Preserve all six positions, whole answers, event order and trace contents without requiring a particular legal scalar quote delimiter. Native execution remains pending CI.

Record expansion, compression and unrestricted size expectations before task execution. Bind independent acceptance to exact code or response bytes and preserve complete declared cohorts, failures and source attribution. Track a 95% objective over at least20 tasks per category without claiming an observed autonomous baseline or making output length a correctness condition.

Witness each declared and used native string lifetime before interpreting unchanged test records. Preserve every original ordered assertion, source identity and effect/profile refusal; unknown or malformed generic declarations reject the whole case. Actual seven nearest controls and full existing reader suites pass; no native compilation claim.

### Changed

- Browser engine and VS Code packages follow automated releases, resolve only the matching trusted main commit, and attach installable archives to GitHub releases. Pull requests verify packaging without publishing.
- Repeated npm publication verifies the immutable tarball bytes; registry authentication and transport errors fail visibly.
- Automatic and manual releases publish the slim Docker image and verify its live chat and file-write tool responses before creating a GitHub release. Pull requests exercise the same runtime contract and image size limit.
- Platform CI is enabled for this pull request, including production desktop and CLI build checks.

Dispatch instrumented tests in actual weight-priority batches over a bounded two-process pool and16 exactly partitioned legs. Preserve1200-second deadlines, all native duration records, failure continuation and complete shard/profile guards. Require every selected test completion and matching harness counts; ignored cases remain separately reported. Instrumented warning-derived weights are honest lower bounds; actual cold timing remains pending CI.

Preserve actual Bash process exit and captured-channel completeness through both pinned AgentCLI adapters using a checked0.26.0 producer repair. Keep timeout, abort, signal and truncation explicit; unknown process status stays unknown. Native driver emits its observed status instead of bare success text. Old ladder receipts remain historical unknowns; native execution and a fresh ladder require CI.

Render conversation-summary history details from actual preceding UI records, showing recorded roles and intents in the selected UI language. Preserve canonical solver answers and explicit plain recap, escape source text, and exclude future/system turns. Original multilingual Browser scenarios pass on guarded scratch overrides; final-source CI remains required.

Source Read observations now preserve authored file bytes, including JSON and failure vocabulary, separately from optional provider-owned metadata. Exact current-call, tool-name and path binding controls auxiliary persistence; bare reads retain unknown status/completeness and allow authorized target delivery with an honest gap. Explicit errors and partial or interrupted process receipts still veto verification.

### Added

- JavaScript repository protocol runners execute declared structural and authoring stages through bounded injected ports, retain observed evidence, and keep commit gates closed until prerequisites pass.

### Changed

- Derive Rust unit-test areas from imports, public module reexports and documentation, moving 156 tests into ten named areas while preserving registrations and relative fixtures.
- Load named rename-map shards and resolve chained historical moves. Spell out eight JavaScript source file names and update their tracked references and TypeScript companion.

Add independently source-bound whole-answer examples to the original Russian prior-question and Rust creator-history specifications, preserving all existing assertions. Both original and extended native programs replay completely in JavaScript; final native CI remains required. Prune one legitimately stale structured-learning allowance. Other undocumented native answers still require actual source-bound execution evidence.

### Added

- Live JavaScript solve, SWE-bench, coding-ladder and authoring command-line surfaces consume the shared repository protocol through Node file/process ports. Clone exact named bases, retain framed Formal AI session evidence, validate and land byte-preserving declared artifacts, and commit only through the four-trailer gate.

### Fixed

- Match native partial-census evidence ranking and report equal candidates without guessing.

Literal-write action binding now uses the target’s own prose statement and checked source ranges, preventing cross-statement native slicing panics. External regressions preserve multilingual, Unicode, declarative, postposed, read-scope and exact quoted-payload behavior.

Keep browser-engine publication and lossless source-network recovery tooling at the independently selected workflow revision when building older stable tags. Bind source serialization to the selected checkout, retain the declared Cargo-derived engine package version, and refuse absent selected package layouts or changed protocol/source bytes.

### Fixed

- File-operation intent comes from the instruction rather than names in path operands. Copying files named for rename, word or line keeps the source and uses the same verified copy recipe for quoted, relative and absolute operands.
- Seeded bare deletion verbs resolve file operands in five languages, while requests to remove text within a file keep their existing edit guards.
- Both roots carry the same general instruction boundary and preserve scalar and list output separately.

Whole-file grounded rewrites and structured member insertions now verify a fresh digest associated with the current exact Write. Explicit failures, wrong paths, stale receipts and later failed writes cannot certify the source effect. Retained Agent CLI fixtures and69 closest JavaScript checks pass; native execution remains CI-only.

- Repair the slim Docker release build by copying Bun 1.4.2 from its published official image instead of the nonexistent 1.2.28 tag.
- Keep the CI Bun pin at the same verified official release; the image manifest contains both Linux amd64 and arm64.
- Leave Rust compilation and the complete Docker runtime smoke test to CI using the prebuilt release binary.

Callable observations now separate deferred body effects, module initialization and syntax obligations. Exact import/initialization and structural-access witnesses preserve source identities without certifying unknown schemas or effects. Forty-four closest JavaScript checks pass; original source-callable composition remains a typed gap.

Fix repository solve to report source-bound test locations and actual command outcomes before mutation, preserving patch transport and refusing incomplete compound goals.

Keep the release checks aligned with the reusable Pages and slim Docker workflows. The slim build has a 25-minute job cap with a 15-minute step deadline, and superseded PR builds release their runners. Preserve the measured full Docker publishing budgets and include the slim sidecar in their contract.

Attach the installable browser engine tarball before optional npm publication. Report missing registry configuration explicitly while configured publishing errors still fail. Record observed CLI, desktop, Docker and package checks with final-head and post-merge verification still required.

Repair the canonical semantic-routing case-study paths, adopted-method invariant wording and exact four-file complete-source metadata contract.

Give a validated explicit shell request priority over research and workspace words inside its quoted program data. Keep caller policy guards, shell file fallbacks and declared mutation preconditions. Both JavaScript and Rust planners follow the same rule.

The unchanged failed Node evidence command now executes in one shell call and writes its exact canonical artifact. Focused JavaScript and native routing regressions preserve complete command bytes and policy refusal; seven nearest JavaScript checks pass. Broad source-change synthesis remains an open capability gap.

Literal content, objective and pinned-line cues now map lowercase matches back to original UTF16/UTF8 character boundaries. Expanding or shrinking Unicode case mappings preserve exact payload bytes and circumfix closers; invalid interior boundaries refuse. Forty-four focused JavaScript controls pass; native execution remains for CI.

### Fixed

- Route container-scoped enumeration to workspace listing when the request names no program artefact or implementation language; retain explicit program requests and named local searches. A later existential clause follows the earlier listing action instead of supplying a fabricated filename.
- Keep quoted first-line constraints and content cues inside authored payloads, while honoring actual constraints outside the literal and preserving fenced source bytes.
- Derive ladder leaf L07 from the canonical greeting seed after its former Rust example list was removed. The current node performs a real seed edit and writes separate effect and proof evidence; the changed greeting reaches the production caller.

State a common rename reason once on its tree and inherit it for each record. Per-record reasons override the shared value, and a new tree resets it. This preserves rename provenance while removing eight duplicate metadata fields. Twelve focused rename-rule checks pass; the metadata repeated-field ratchet returns from 81 to 80.

Fallback Write routing now retains the original request block and requires the shared statement-bound literal contract. Classification text cannot join completed statement cues into writable operands. Fifty-two focused JavaScript controls pass, including full-planner negative ownership and exact positive bytes; native execution remains for CI.

General change plans now retain earlier plan events instead of overwriting the event stream. Known shell clients append under a bounded lock, check the stable event identity for retries, and read back the actual event. Clients with file reads preserve the observed stream and verify it after writing; clients that expose only an overwrite tool cannot claim an append.

The native driver exposes protected workspace file reads. Structurally valid numbered file-read frames preserve authored error narratives as data while explicit tool failures still block completion. The repository-root JavaScript dogfood driver maps append execution into the same ignored plan sandbox as file tools, preserving the committed historical log.

Closest JavaScript checks passed: eighteen append and authored-content tests, plus the two selected G15 tests. They cover actual shell execution, simultaneous retries, denied reads, lost-history readback, bounded lock failure and sandbox mapping. Native sources and closest tests pass standalone Rust formatting checks; native execution remains a CI check. Broad source-change synthesis still needs further capability work: the unchanged implementation request read the source without implementing it.

### Fixed

- Explicit shell requests pair POSIX outer quotes and escapes before prose literal guards, preserving adjacent apostrophe fragments as opaque command data in JavaScript and Rust. Malformed command quotes still decline.
- Retain real-shell regression checks for apostrophe, dollar and backtick data, and the unchanged original evidence-collection retry.

All repository mutations used Formal AI or checked generators. Native execution remains a CI check; standalone formatting and four closest JavaScript checks pass. Broad source-feature synthesis remains open.

Explicit module paths now scope requirement declaration resolution before ranking. File-qualified scalar literal edits can bind a unique initializer from identity-checked source; ambiguous unqualified declarations still refuse. Closest JS controls pass; the independent L09 native census freshness blocker remains.

Closed literal operands now keep internal enumeration cues as payload during obligation splitting in both JavaScript and Rust. Genuine outer obligations still split, with exact UTF-8 spans.

The unchanged long authored-document requests now write all expected bytes and receive real readback receipts. Focused JavaScript checks pass 41/41, covering all 34 seeded cues, Unicode and separate artifacts; native sources and external tests pass standalone formatting. Broad source-repair synthesis remains open.

Source edit verification now checks successful, target-bound stdout bytes instead of comparing whole shell receipts with file contents. The same JS/native mechanism covers whole rewrites, structured insertion, generated source and independent source/registration readbacks.

Exact Unicode, JSON and terminal newlines are retained; failed, unrelated, stale or structured presentation receipts cannot certify source. Forty focused JavaScript checks and the unchanged original failed edit pass. Native receipt helpers were extracted within the existing file budget and pass standalone formatting; native execution remains a CI check. Broad repair synthesis remains open.

Mac package checks now start the actual mounted sidecar and retain its complete startup and post-sign executable bytes alongside the independently validated build receipt. Durable signing evidence cross-checks the package, source, target, compiler and original receipt without assuming signing preserves executable bytes. Real native package execution and publication remain subject to CI.

### Fixed

- Count source callable input coverage from the authoritative recurrence expression body, so declaration-only names cannot outrank a program that reads its inputs. Keep the existing cost and candidate comparator unchanged.
- Retain whether a callable signature was provisional, explicitly declared or observed in assertions. Preserve source identity for provisional requests, structurally bind explicit parameters and recursive calls, and reject incompatible or shadowed self-call bindings.
- Verify that each recurrence body contract matches the callable and source being evaluated; preserve explicit signatures and imported annotations in the rendered program.

Share browser/server formalization evidence selection and preserve raw events for idempotent projection. The browser now carries 125 of 136 selected native specification cases successfully, with 11 gaps and the remaining native cases still explicit debt; full browser meta-core parity remains pending.

Handle local named export clauses and validate generated worker factory syntax before writing files. Preserve exact append-record readback while excluding inter-record separators. Move repository diagnostics into canonical message data and preserve immutable evidence names in the established abbreviation policy. Closest checks: 7 shared-projection/export cases, 8 append cases, 6 abbreviation cases and 20 repository caller cases passed; native behavior is verified in CI.

Composed document findings now retain producer-owned audit provenance, allowing the planner to write the complete honest note to its requested destination. The colon-ended source-condition fixture executes real bounded workspace writes and readbacks instead of answering every tool with an acknowledgement. Original note bytes, exact source condition, proof and effect assertions, and failure certification guards remain intact. Closest JavaScript replay: nine passing assertions; native execution remains CI-only.

### Fixed

- Keep clarification, gap, failure and uncertified final outcomes out of evidence files. Bind a finding certificate to its selected answer bytes while preserving the public text-only final protocol.
- Bind test-authoring questions to their own instruction and distinguish call-shaped code artifacts from evidence destinations. Preserve full marker-led multiline payloads with embedded quoted fields and existing whole-content consent, including longer fenced source.
- Exercise real plan-event shell receipts in literal creation and workspace fixtures instead of reporting empty fake success. Retain the planner route and worker module ceilings by reusing an existing dispatch branch and extracting final-result helpers.

The JavaScript worker now reuses the shared weighted summarization pipeline for curated project descriptions, renders the native promoted-project narrative from shared multilingual seed records, and projects curated GitHub HTTP requests before network access. Project weights follow the native complete unsigned-byte parser, including zero. Live promoted searches retain actual captured snippets, fusion evidence and diagnostics even when statement formalization is unavailable.

The closest production-worker tests carry all 22 unchanged assertions from five native specification cases and cover arbitrary modes, weights, localization, URL boundaries, unavailable network, uncurated fetches and genuine provider extraction/fusion. Native expectations and ranking remain unchanged; native compilation is left to CI.

Browser replies now require explicit agent mode for JavaScript execution, identify the documentation behind unexecuted programs, validate prime candidates before selecting them, retain the native unsupported-language failure response, and execute bounded file actions in fresh in-memory workspaces with visible outcomes. Formal AI tool adapters preserve exact successful stdout and report missing adapters and failed searches as typed failures. The checked browser specification record improves from125/136 to135/136; genuine external citation retrieval remains open.

The native ladder fixture helpers now execute actual bounded workspace writes, reads and append shell commands, retaining real process status and separate source-read metadata. Planned content assertions inspect physically written files across the original bounded run. Literal continuation, terse source evidence and original note targets retain their exact requests and assertions. Desktop Release fixtures use consistent forty-hex source identities, route release metadata independently from tag queries, and inspect executable packaging YAML. The closest 37 JavaScript and real shell controls pass with no skips; standalone Rust formatting passes. Native compilation and execution remain CI-only; no production trust or verification guard changes.

Source-qualified definition requests now bind the subject and named provider through shared multilingual frames before the existing dictionary lookup. Both runtimes replay genuine registry captures only after verifying their content address and capture time. Wikipedia endpoints preserve the literal subject, and disambiguation responses retain every captured list entry with URL, full SHA-256, capture time, cache status and license.

The unchanged associative-memory citation passes with all six genuine meanings. English and Russian entropy entries, arbitrary synthetic providers, source controls, offline misses, corrupt captures and fifteen-entry disambiguation are covered. The focused JavaScript suite passes 13/13; all 136 carried specification cases pass out of 1,171 native cases. Native formatting passed; native compilation and execution remain CI checks.

Formal AI applied every repository edit. Its broad implementation request and unchanged retry both declined before tool calls, so autonomous synthesis remains open. Evidence preserves those failures, the fixture/cache failures and resolutions, and an overly broad formatter invocation with five exact restorations.

Fetch observations now separate actual provider failure from document bytes in both native and JavaScript planners. Failed responses retain their requested URL and cannot become an authored work-item goal; successful prose and JSON retain exact source bytes. The offline corpus exposes source-owned lookup results, and the native driver reports genuine missing-source failures without inferring status from body text. Source-aware fixtures distinguish unavailable source, an unpersisted capability boundary, and a genuinely fetched non-artifact goal with verified append-only history. The nested local-readiness fixture now executes actual workspace tool receipts. Closest validation reports 113 passing tests, zero failures or skips, and one existing TODO; native formatting passes, while native execution remains CI-only.

# Native receipts and release checks

Validate original native Agent task and target bytes against complete command-bound process receipts, preserving immutable historical sessions and exact actual CI operands. The full-session replay tests retain byte equality with the observed protocol.

Declare safe container checks dispatch inputs and require explicit stable publication; scope the known artifact adapter deprecation to its own steps. Source-level release checks follow current verified producer reuse, imported version constants and measured seed metadata. Fresh CI and actual release publication remain required.

The source-bound native whole-answer capture runner now dispatches original tests longest-first using the existing recorded-duration lookup and bounded process scheduler. It retains singleton isolated processes, one thread and one concurrent process, exact qualified caller binding, original assertions, source/build receipts, raw streams and all failures. Reports distinguish recorded weights from declared fallback estimates and retain the actual process launch order; estimates are not fresh timing measurements.

The relevant JavaScript controls pass34/34, including six new actual Node-process ordering/isolation/failure controls and28 unchanged observation and queue controls. Scoped readability and diff checks pass. Original native test execution and final-source launch-order acceptance remain pending CI; no local Rust compilation or published-release claim is made. Broad original and unchanged implementation requests remain Open G112; reviewed patch replay is not autonomous synthesis.

Move the seeded source-qualified definition interpreter under formalization, preserving its capture validation, source binding, disambiguation and native fixture assertions. Route the dictionary hook through that module and keep the dictionary parent under its existing line limit without raising boundary ceilings.

Validated with unchanged JavaScript citation/dictionary tests, exact source preservation checks and targeted standalone native formatting. Native compilation remains a CI check.

Native whole-answer capture now appends each observed stdout and stderr byte chunk to its isolated case files before waiting for process completion. Interrupted capture controllers retain already-observed bytes; the unchanged completion, qualified caller and independent source/build checks still govern whole-answer certification.

The unchanged 34 portable capture/scheduling controls and two new binary-stream/interrupted-controller controls pass. No native compilation or original native tests ran locally; actual native capture remains a CI acceptance step. Formal AI applied reviewed source; the broad implementation request still has no independently verified autonomous implementation.

The original native-response capture runner's miniature repository now declares its scheduling-data precondition before creating the producer identity. Its tracked duration record has only the declared fallback and no invented measured rows. All original test programs and assertions across 17 controls remain unchanged.

The initial unchanged runner command exposed six missing-file failures. The exact command retry passes all 17 controls; the related original 34 scheduling/capture controls plus two stream interruption controls also pass. Actual native execution and publication remain CI acceptance steps.

The issue982 immutable self-hosting fixtures retain every original task, leaf byte and whole-session field while requiring actual auxiliary plan reads and complete command-bound process receipts. The first contract additionally compares the full actual failed-CI operand losslessly retained with official job/run and raw-log provenance. The rollback was not executed after the original assertion failure; its whole original fixture and fresh native comparison remain required, with no invented capture. Closest native-protocol and provenance controls pass; fresh native execution remains pending CI.

The supplied-file reader imports the existing same_path predicate from its extracted record owner, restoring compilation without changing the predicate body. Actual Formal AI packet raw logs are included explicitly despite the global log ignore rule, preserving exact original failure and unchanged retry evidence for local links. New-head native acceptance remains pending.

Restore report-action round-trip translation through Chinese by adding the unambiguous seeded action surface 上报. The plural report noun keeps 报告, and the existing meaning-preserving surface chooser retains both identities.

All 20 existing translation ratchets hold again, including English/Chinese0.459, Russian/Chinese0.352 and Hindi/Chinese0.392 term survival. Eight JavaScript tests pass with unchanged corpus/threshold assertions and five new multilingual action/noun controls. Added equivalent native pins; targeted formatting passes, native execution remains CI-only.

### Fixed

- Bind complete multilingual source requests to exact technical types and canonical seed meanings.
- Authenticate completed release sources and publish the exact Desktop-attested VS Code package, with helper changes covered by consumer CI.
- Preserve complete Unicode specification identities and original failed execution evidence with reviewed inventory witnesses.

Auto and instant releases now record the existing native build against independently fetched tagged-main source, Cargo version, compiler, flags and executable bytes. Full image publication copies that verified binary with the existing BuildKit cache; optional Docker Hub publication mirrors the observed immutable image without another compilation. Both paths retain slim verification before GitHub release creation and attach source/build/image receipts with checksums.

The 37 focused JavaScript controls pass, including genuine Node-process failure, changed-source/bytes, immutable runtime/version/hash, anonymous manifest and mirror negatives. Actionlint and the two native fixture formatting checks pass. Native compilation, real registry publication and cold job timing remain CI acceptance checks; the original release budget floor and six recorded CI timing exceptions remain.

JavaScript symbolic answers now serialize the native six-field answer record from the actual append-ordered solver log, preserving prior turns and their trace IDs. Record thinking retains response-link provenance; display thinking retains the composed answer. Memory finalization uses the same serializer, and independent worker and persisted derivations remain intact.

Source-qualified definitions project verified capture observations into typed solver events in native order, preserving real URL, capture time, complete SHA-256, cache status and unresolved outcomes through the API and derivation store. The pure native slot parser is public, and its original fixture assertions now run from the integration harness rather than a production-source test module.

Validation: ten production answer-record contracts and twenty-five existing trace, citation, derivation and memory tests pass. Targeted standalone native formatting passes; native execution remains a CI check. Original broad Formal AI synthesis still only reads source, so its failure remains open independently of the reviewed changes applied through Formal AI.

### Fixed

- Verify consumed source declarations, pinned lines and module registration before coding effects.
- Correct production regex classification, signing failure propagation and quoted-program dependency scanning.
- Ground multilingual authoring diagnostics and conserve full worker seed inventory within maintained budgets.
- Preserve original CI failures, delegated source evidence and current release verification boundaries.

The source-test lexicon registry now includes the canonical number-word seed, preserving every existing digit, token and phrase table assertion. New checks parse the actual registry and compare both unchanged native arithmetic tables with the real multilingual worker.

Leading courtesies now bind to one substantive directive through semantic intent records. Proof parsing reads that directive while retaining the full original impulse. Explicit question lists and greeting-prefixed procedural questions keep their composition. The source snapshot mirrors the same mechanism and existing question guard.

Validation: six focused JavaScript checks pass, including the four unchanged native prime-proof prompts, an additional greeting-prefixed proof, actual impulse preservation and CJK/punctuation boundaries. Seven touched native/source files pass standalone rustfmt checks. Native behavioral execution remains assigned to CI. The specialization gate reports zero held prompts. The global worker budget check currently fails only on the separately owned bootstrap module; the two touched worker modules retain their existing ceilings. Broad autonomous implementation requests remain unresolved and their unchanged retries are preserved.

Canonical seed requirements can now bind through a real surface and semantic role to their declared registry constant. Existing named-declaration priority is preserved. Missing declarations, unknown surfaces and equally supported distinct owners remain unresolved; a longer actual surface supplies stronger evidence. The explicit-source lexicon parser is shared with the existing cached parser.

The Node planner host now supplies actual committed census documents, checking their content identifier and byte length against the real source before exposing them. A changed source makes the old document unavailable.

Validation: the unchanged L07 greeting requirement resolves to the canonical conversation seed declaration, four focused binding/ambiguity/identity checks pass, and the existing two real greeting-edit/caller checks pass. The full unchanged 32-leaf JavaScript matrix currently reaches L17 and correctly refuses its stale source census; it remains open until a genuine native census refresh. Original native assertions are preserved and native behavioral execution remains assigned to CI. Targeted native formatting and the zero-specialization gate pass. Broad original implementation ask and unchanged retry only read source and remain open.

### Fixed

- Bind released WebAssembly to selected source and compiler, and verify cold browser engine installs within a shared deadline.
- Retry Marketplace propagation and preserve exact version acceptance.
- Support typed source authoring, truthful discovery status and advertised physical native edits.
- Conserve seeded diagnostics, planner routing and release operation budgets while preserving complete original CI evidence.

Scope authored artifact kinds to their action and destination header, excluding filename and content bytes. Preserve Unicode offsets and genuine callable guards. The original L21 task now writes its separate effect and proof records; independent record readback remains pending.

Verify evidence and effect record bytes after their current Write before delivering a Finding. Preserve exact-path and raw-byte receipts, bare Read compatibility, and typed failures or gaps when observation fails or is unavailable. Both unchanged original L21 requests now independently verify their mandatory records.

Correct source-owned Read forms, preserve unavailable line selection refusals, and add500 multilingual/renamed/negative controls. Complete new authored seed metadata without raising existing debt or closure ceilings. Retain explicit claimed/no-step planning states and scoped Desktop WASM warning handling.

Add source-bound staged release authority, isolated read-only Actions token observations and stronger conserved WASM build obligations. Canonical release activation and genuine new native, package, Docker and production publication validation remain pending; historical CI evidence is retained without promoting it to current-source proof.

Share raw literal parsing and semantic-content ownership between the JavaScript composer and worker, scope source-links routing to positive unquoted instructions, and preserve a typed MissingContract for unsupported semantic implementations. Exact authored bytes and genuine source-links requests remain supported; this safety repair does not implement general source synthesis.

Bind actual complete filesystem Read observations and caught Read failures to outer provider metadata in the JavaScript dogfood driver and auxiliary event providers. Preserve the legacy execute text projection, raw source bytes, event history safeguards and unknown bare-source status.

# Automate source repair and preserve qualification boundaries

PR1188 now uses guarded source compilers for214 canonical JavaScript cases, locked CI fixture dependencies, lossless response-family/role regeneration, source-owned Read/append controls, measured browser receipts, independent artifact profiles and hosted original-session production. Every installed effect runs through JavaScript Formal AI; complete failures, source inverses and native data receipts are preserved.

The finite batch passes ordinary local qualification and required gates. Fresh hosted native CI, remaining968 parity cases, genuine autonomous coding and production publication remain required. No scanner, threshold, deadline, raw denominator or authority boundary was weakened.

Source readers and callable discovery now use request-local provider observations instead of treating JSON file keys as transport failures. Full reads preserve exact source bytes; unknown or partial observations remain uncertified, and genuine provider errors retain Failure. Existing record matching is factored into focused helpers.

bump: patch

Decode successful command receipts before algorithm artifact and conformance checks. Preserve command binding, exact source bytes, failure vetoes and all independent verification calls; align native driver regression assertions with its typed receipt schema.

Keep synchronous algorithm library success separate from OS process exits, and attach invalid structured-source controls to the actual fresh digest observation.

Preserve original fetched-issue safety and structured list byte contracts with actual auxiliary execution and fresh digest observations.

Declare JSON import attributes as explicit Rust no-form with canonical grammar-key binding, preserve source bytes and punctuation identities, and validate Unicode source changes through fresh digest receipts.

The issue1099 two-file fixture now executes real bounded workspace tools for auxiliary append and source readback. It preserves the original request and eight-turn limit, both target paths and complete artifact bytes, and the assertion that a final answer cannot precede the second write.

### Fixed

- Preserve quoted command operands, use truthful failed-command observations in fixtures, and isolate CI evidence paths so source-bound checks do not consume unrelated files.
