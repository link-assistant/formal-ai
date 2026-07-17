# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

<!-- changelog-insert-here -->

## [0.296.2] - 2026-07-17

- Make agentic code generation and contextual follow-up changes use real client workspace tools across every catalog language. Follow-ups now execute auditable, bounded normal-algorithm programs with ordered/leftmost/restart/terminal semantics, empty-string creation and deletion, multi-rule and arbitrary-path support, no partial write on exhaustion, structural multilingual literal slots, and review-gated associative learning verified through built-in and OpenCode Agent CLI replays.
- Add `links_substitution_query`: the [link-cli](https://github.com/link-foundation/link-cli) substitution query language, as the meta-language representation between a natural-language request and a harness's read/write tools. `(matching pattern) (substitution pattern)` carries link-cli's CRUD-by-substitution shorthands — `() (("new"))` creates, `(("old")) ()` deletes, `(("old")) (("new"))` updates, `(("x")) (("x"))` reads — lowers to a bounded, Turing-complete normal algorithm, round-trips through a canonical renderer, and is published in every mutation trace alongside each rule's CRUD effect. Queries are also accepted directly as requests and lower identically on every harness vocabulary.
- Substitute over links as well as text sequences. The substitution model is the operand-independent part, so the query language is written once and its operands are read two ways: quoted character sequences, or `(source target)`/`(index: source target)` doublets with link-cli's `$i`/`$s`/`$t` variables binding across slots. `parse_link_substitution_query` reads link-cli's documented queries verbatim — `() ((1 1))` creates, `((1 1)) ()` deletes, `((1: 1 1)) ((1: 1 2))` updates, `(($i: $s $t)) (($i: $s $t))` reads all links without modifying them — and executes them under the same ordered, restart-at-rule-zero, step-bounded Markov control model, which is what carries Turing completeness across to the associative store.
- Fix the published mutation trace not being readable as the Links Notation it is written in, so a trace for a change to real code — anything carrying both a quote and a paren, such as `println!("Hello, world!");` — now parses instead of failing on an unclosed group. Links Notation escapes a quote by doubling it and picks a delimiter the value does not already contain, so a value is now carried as `pattern 'println!("Hello, world!");'` rather than backslash-escaped. The trace renderer delegates to the codec's own escaper instead of keeping a private copy of the rules, and `tests/issue_715_notation.rs` holds it there by parsing every published trace with the same codec the library encodes with.
- Fix text-manipulation and document requests losing their operands after an ASCII apostrophe, so prompts such as `It doesn't matter, replace "cat" with "dog" in this text: "cat naps"` no longer fall through to the unknown intent. Both handlers kept private copies of the literal-slot reader that predated the general one in `normal_markov`; the copies are deleted and `quoted_segments`/`quoted_segment_spans` is now the single implementation behind every caller.
- Answer link substitution queries from `formal-ai memory query`, so the link half of the meta language is reachable from the surface that owns the links. `parse_link_substitution_query` and `matched_links` were public API with no caller in the product, which left link-cli's own syntax unable to reach link-cli's own operand domain; a turn that *is* a query now routes to it ahead of the natural-language recognisers, and `(($i: $s $t)) (($i: $s $t))` reads every link of the memory projection back in the notation it was asked in. Link-level writes are refused with a message naming the reason rather than silently doing nothing: the doublet view is a one-way projection of memory events, so an edited link has no inverse back to the event it came from. Prose that merely opens with a parenthesis still falls through to natural language.
- Read Links Notation's own quote escaping in the seed parser, fixing a live corpus corruption that predated this issue. The notation escapes a delimiter by *doubling* it, and `strip_comment` already read it that way, but the value decoder never learned the rule: a value carrying its own delimiter had no closing quote on its line, failed to decode, and fell back to raw text with the quotes still in it. Five files under `data/` already write that form — `the subject''s name` in `data/cache/wikidata/property/P138.lino` read back as `'the subject''s name'` — and now decode. The change is additive, not a migration: doubled delimiters had no valid meaning before, so the backslash dialect the rest of the corpus uses is still read alongside it.
- Route every Links Notation renderer through one encoder, so a record carrying code is written in the notation it claims to be. Seven files had each grown a private `escape_lino_value` implementing the same C-style escape with subtle variations, and several value slots were interpolated with no escaping at all; substitution rule sets, associative packages, skill packages, behavior rules, self-facts, intent formalizations, and formalization candidates now share `format_lino_value`. It borrows the rule rather than restating it — the codec's always-quoting encoder is private, so a one-field record is formatted with the public `format_indented_ordered` and the field taken back off it — which means it cannot drift from the notation, because it is the notation's encoder. `tests/unit/issue_715_renderer_artifacts.rs` checks every renderer against both readers of the same document: the real grammar and the repository's own parser.
- Carry the above through the `tests/source/` mirror, the hand-copied second library that exists to reach private functions. Nothing enforces that it matches `src/`, and it had rotted where nobody looked: 51 of 143 mirrored modules differ from their source by more than two lines. Every module this change touches whose copy *can* compile is brought back into line, including `normal_markov`, whose copy was left behind by an earlier commit on this branch. One cannot, and saying so is the point: `intent_formalization`'s copy still holds the private `escape_lino_value` deleted above, because bringing it forward needs `cue_lexicon`, which reads its data through `include_str!("../data/meta/cue-lexicon.lino")` — a path that resolves under `tests/` from the mirror and cannot resolve at all. A faithful copy is not merely absent there, it is unrepresentable, so the mirror can only ever cover the part of `src/` that neither reads a file relative to itself nor depends on something that does. Nothing tests that copy — there is no `source_tests/intent_formalization/` — so no test asserts the old escaper's behaviour and the rot is dead weight rather than a false pass; but dead weight is what the next reader copies from, and the only thing that would have caught it is the `--check` this mirror has never had.
- Execute issue #715's own auto-learning task through two real external Agent CLIs, closing the one evidence row that cited only the in-process harness. The derived report names its promotion gate `normal_algorithm_laws_multilingual_slots_and_agent_cli_e2e_pass`, but no external Agent CLI had ever run the task — the gate asserted a pass that did not exist, and the in-process harness is precisely the one that cannot show capability routing surviving the wire. `experiments/agent_cli_e2e/run_issue_715_learning.sh` now drives `@link-assistant/agent` and `opencode` against the same task and diffs the two derived reports byte for byte, which turns "all harnesses supported in the similar way" into an assertion: a harness is supported only if it derives the *same* artifact. All three harnesses produce an identical 3961-byte report, so a harness contributes its tool vocabulary and nothing semantic. The script also asserts the report never promotes itself, and is wired into the `E2E Tests (agent CLI ↔ formal-ai)` CI job so the parity is enforced continuously rather than captured once.
- Fix eleven CI step names silently losing the issue number they exist to carry. An unquoted `#` opens a YAML comment, so `- name: Run agent CLI E2E — declarative new-file phrasing (issue #712)` reached the runner as `… (issue` — every E2E step advertised a dangling open paren where its issue reference should be, which is precisely the link a reader follows when the step goes red. The names are now quoted.
- Fix the writer and the reader of the same document disagreeing about what an escape means, so a rewrite of real code survives being written down. `sanitize_lino_value` escapes `\r`, `\n` and `\t` for the line-based `seed::parser`, but the decoder only ever learned `\n`, and neither escaped the backslash itself — so the pair was not invertible in either direction. A tab was written and read back as the two characters `\` and `t`, and a value carrying a backslash was read as an escape it never wrote: rewriting `println!("\n")` returned a *real newline* where the Rust source holds two characters. Neither is exotic for the subject of this issue — a Makefile recipe line is required to begin with a tab, Go is tab-indented, and a substitution rule round-tripped this way silently stops matching the code it was derived from. `SubstitutionRuleSet::from_links_notation` reads rules back through exactly this path. The backslash is now escaped first (it must be: sanitizing *introduces* backslashes), the decoder learned `t` and `r`, and its catch-all stays so LaTeX like `\ldots` still passes through. `tests/source/source_tests/links_format/tests.rs` holds the two functions to being inverses over the cases that motivated them rather than asserting the escape table by eye.
- Fix `scripts/build-views.py` emitting a backslash it never escaped, which the above would otherwise have turned into corruption. The generator interpolates each gloss straight into a quoted slot, so `\rightarrow` in `data/view/en/graph.lino` reached a decoder that now has an `r` arm and would have read a carriage return. Doubling it at the emit site — not in `_gloss_clean`, whose output the merge and keyword logic compare against — makes the decoder's `\\` arm return the one backslash the gloss holds; the regeneration moves exactly three lines in two files. The escaper is deliberately minimal and documented as such: a faithful one would have to reproduce the codec's delimiter choice, and doing that in a second language is precisely what the generator already gets wrong elsewhere — it rewrites Wiktionary's own quotation marks into apostrophes because it cannot escape them, so *hello* is defined as `'Hello!' or an equivalent greeting.` where the source says `"Hello!"`. Fixing that means writing these values through the codec rather than adding a tenth hand-rolled escaper, and is left as its own change.
- Fix the auto-learning report being unreadable whenever it reported on real code. An eighth private escaper survived the unification above, in the one renderer whose document the seed parser never reads back — so the grammar was its only reader, and the grammar is exactly the reader a backslash escape defeats. A `text` field carrying a quote made the whole report fail to parse, which is to say the auto-learning loop could not report on the subject of this issue. Separating the two jobs the encoder had been doing is what makes the fix free: `format_lino_value_verbatim` quotes and nothing else, while `format_lino_value` keeps sanitizing newlines for the documents `seed::parser` reads back a line at a time. The report takes the verbatim path, so its values keep their newlines and the committed issue-686 Agent CLI session stays byte-reproducible. `tests/unit/issue_715_renderer_artifacts.rs` asserts the field *survives* the grammar rather than that the document merely parses — a distinction the probes in `experiments/` had to establish, because the same escape elsewhere parses fine while the field silently disappears from the tree.

## [0.296.1] - 2026-07-16

### Fixed
- Stop the auto-release job from failing with "cannot rebase: Your index contains
  uncommitted changes" when a concurrent release lands on `origin/main` mid-job.
  The release now rebases onto the remote while the tree is still clean, before
  the version bump is written and staged.
- Only rebase when `origin/<branch>` actually has commits the release job lacks.
  Being ahead of the remote no longer reports "Local branch is behind remote".
- Create the release tag only after the release commit reaches the remote, so a
  `pull --rebase` retry can no longer leave the tag on an orphaned commit.

### Fixed
- Stop the release jobs from crashing the runner with "No space left on device"
  during the Docker build, which published the crate to crates.io but produced
  no image and no GitHub Release. The disk reclaim that issue #523 added to the
  Pages job now also runs in the auto-release and manual-release jobs, and lives
  in the shared `scripts/free-runner-disk.sh`.
- Warn when a runner is still nearly full after the reclaim, so the next
  occurrence leaves a diagnosable annotation instead of a job that dies with no
  failed step and no downloadable log.

### Fixed
- Stop the desktop release from reporting success when only some targets built.
  `BUILD-PROVENANCE.txt` listed all six builders unconditionally, so a run where
  most of the matrix failed still published a green, authoritative-looking
  `SHA256SUMS.txt` claiming builds that never happened. The manifest now lists
  only the builders that produced a fragment, names the missing targets, and the
  run fails after publishing the partial manifest.
- Verify the Linux and Windows artifacts before uploading them. Only the macOS
  artifacts were smoke tested, so the other four targets shipped with nothing
  checking that they were produced under the expected names and non-empty.
- Attach the SLSA build provenance before publishing assets to the release,
  rather than after, so assets are never downloadable without an attestation.
- Deduplicate concurrent desktop releases on the automatic (`workflow_run`) path.
  The concurrency group read `release.tag_name`/`inputs.tag`, neither of which
  that event carries, so it fell through to the always-unique `run_id` and
  concurrent runs for the same tag raced on `gh release upload --clobber` and on
  the consolidated `SHA256SUMS.txt`.

### Fixed
- Write `CHANGELOG.md` during a release in the exact shape the reconstruction
  check expects. The release spliced each new section in before the first
  `## [` line, but the preceding lines already ended with the blank line that
  follows the insert marker and the entry opened with another newline, so every
  release left a doubled blank line; `lines()` also dropped the file's trailing
  newline, which `join` never restored. Both defects went unnoticed because the
  check only runs when the lint job's path filter fires, which release commits
  do not trigger, so `main` turned red on the next unrelated pull request and
  the artifacts were refreshed by hand instead. Applied to both the automatic
  (`version-and-commit.rs`) and manual (`collect-changelog.rs`) release paths.

### Fixed
- Stop the issue #656 traceability test from asserting that a changelog fragment
  exists forever. Fragments are consumed by the release that ships them, so the
  test began failing on every run once the v0.296.0 release deleted the fragment
  it pinned. It now follows the entry across its lifecycle: a fragment before
  release, a `CHANGELOG.md` section after one.

## [0.296.0] - 2026-07-16

### Added
- Add a benchmark-gated promotion protocol (issue #656): `formal-ai improve --promote`
  executes canonical coding-modification, industry, and promotion-unit gates from
  fresh process output and, under `--apply --confirm`, creates a clean local review
  branch and materializes accepted `.lino` seed edits through Formal AI's Agent
  task path. Proposal-supplied runners/results, unsafe paths, malformed evidence,
  and failed commands are rejected; no push occurs. The promotion event chain and
  rejected changes round-trip through bundle export/import.

### Fixed
- Attest desktop and VS Code release artifacts directly so LF checksum manifests cannot break Windows provenance.
- Enforce rustdoc warnings, least-privilege workflow permissions, bounded desktop jobs, and fail-closed classification of known dependency diagnostics.
- Keep file-authoring Agent CLI requests from being misrouted into duplicate GitHub issue creation.

## [0.295.2] - 2026-07-16

### Fixed
- Route typed generated-source artifacts and ordered compiler/run commands through the write and shell tools advertised by an agentic CLI instead of scraping rendered answer labels or describing execution performed in a server-private fixture. Follow-up output edits now update the source before it is written, failures stop the command sequence, and Chat Completions, Responses, Anthropic Messages, and Gemini use the same routing behavior.
- Prevent HTTP API requests from executing agent actions in Formal AI's embedded temporary workspace; the client harness remains the auditable execution boundary.
- Persist issue #716 observations and evidence-linked architectural amendments in the associative auto-learning substrate, and produce a human-review-gated client-execution report through Formal AI and the real Agent CLI.

### Tests
- Add issue #716 presentation-independence, all-catalog-language, API-surface, auto-learning, and real Agent CLI E2E coverage that verifies `main.rs` is written and the harness receives both Rust compile and execution commands.

## [0.295.1] - 2026-07-15

### Fixed
- Route URL-navigation wording to advertised fetch tools, broader web-research wording to search tools, common file-update verbs to edit tools, and declarative `new file: …, contents: …` requests to write rather than read (issue #712).

## [0.295.0] - 2026-07-15

### Fixed
- Consume and stage changelog fragments after a successful release collection, preventing later releases from republishing stale notes.
- Reconstruct `CHANGELOG.md` from Git release history so each of the 391 released fragments appears exactly once.

## [0.294.0] - 2026-07-15

### Fixed
- Route agentic “Report issue” actions through an advertised shell tool to `gh issue create`, and enable OpenCode's documented Exa-backed `websearch` tool in ephemeral Formal AI sessions.
- Preserve client-executed tool inputs and outputs as durable memory evidence after the final API turn, including unnamed OpenAI tool results and Anthropic/Responses translations, so the associative and dreaming loops can learn from work performed by an Agent CLI.
- Parse Gemini `functionCall`/`functionResponse` history, retain call ids, and continue the shared multi-turn planner after a Gemini client executes a tool.

## [0.293.0] - 2026-07-15

### Fixed

- Keep subcommand-only and value-taking prompt flags out of empty interactive `with-formal-ai` launches, with PTY launch coverage for every supported CLI.

## [0.292.0] - 2026-07-15

### Added

- **Agentic mode now acts on simple natural-language requests** instead of
  falling to the "I could not determine…" blurb (issue #687). When Formal AI is
  driven as an agentic backend (e.g. OpenCode over the OpenAI-compatible server),
  the deterministic planner now recognises three new request classes and emits
  the appropriate tool calls for the harness to run:
  - **Factual / research questions** the symbolic engine cannot answer locally
    ("When are the next elections in the USA?", "What is the current population of
    Japan?", "Learn about it.") are routed to the client's **web-search** tool,
    then the surfaced source is **fetched** and the answer read from it
    (`src/agentic_coding/web_research.rs`). Whether a prompt warrants web research
    is decided by *asking the engine* — we search precisely what it cannot resolve
    from its own knowledge base — so it generalises across phrasings rather than
    matching fixed strings.
  - **"Report [this] on GitHub"** in natural language is turned into a real
    `gh issue create` shell tool call against the Formal AI repository, and the
    created issue URL is surfaced back to the user
    (`src/agentic_coding/report_issue.rs`). Agentic mode has no Formal AI web UI,
    so the top-bar "Report issue" button was previously unreachable.
  - **Conversational / meta questions** ("What we were talking about?") are
    answered from the message history with no tool call
    (`src/agentic_coding/conversation_recall.rs`).

### Changed

- The agentic `Progress` scan now also captures **web-search output**
  (`Progress::search_output`) so the research recipe can pick the source URL the
  search surfaced and fetch it before answering.

## [0.291.0] - 2026-07-15

- Fix Windows desktop provenance attestation by using the LF-safe current attestation action, update deprecated CI actions, remove recurring false-positive workflow warnings, and prevent docs-only final commits from hiding earlier code changes from CI.
- Make file-edit plans read their target before editing so read-before-write Agent CLIs can execute Formal AI's requested patch.

## [0.290.0] - 2026-07-15

### Added
- Usage-weighted associative persistence for issue #686
  (`src/associative_persistence.rs`): an `AssociativeMemory` that keeps a
  persistent version of meta-language expressions saved in an associative links
  network. Each expression is a content-addressed node (`stable_id`, so one meaning
  is one node) in an embedded `SubstitutionGraph`; the store counts usages (reads)
  and changes (writes) per expression and derives an independent usage signal from
  each node's incoming and outgoing link degree. A single `retention_score` (reads
  + writes + in-degree + out-degree, under configurable `RetentionWeights`) drives
  an LFU-style policy so the most used, most changed, and most connected knowledge
  persists longest; `eviction_order` / `evict_least_used` / `retain_most_used`
  forget the lowest-scored first, and `forget` removes an expression together with
  its incident links. Everything serializes to Links Notation, `from_context`
  ingests an issue #649 world-model `Context` preserving statement ids, and the
  whole policy is deterministic (no clocks, no randomness). Durable
  `MemoryEvent::write_count` now round-trips through native serialization, sync,
  substitutions, link projection, and the browser mirror; automatic dreaming
  rebuilds this associative view and uses the complete score for real eviction.
  Event ingestion also preserves qualifiers and validation warnings, normalizes
  evidence aliases, and supports bounded multi-hop recall. A derived persisted
  memory scenario executes through Formal AI and the real external Agent CLI.
  Covered by the issue-686 persistence, dreaming, and agentic regression suites.
- Design case study for issue #686 under `docs/case-studies/issue-686/`: a deep
  analysis mapping persistence, read/write counting, incoming/outgoing-link-degree
  usage, and links-only retention onto the associative stack, with cited online
  research (the Wikontic paper's full transferable symbolic pipeline,
  AriGraph, LFU/LRU cache replacement, reference counting, degree centrality), a
  per-requirement solution plan and prior-art survey, requirement rows R445–R458 in
  `REQUIREMENTS.md`, and the `tests/unit/docs_requirements_issue_686.rs`
  traceability test.

## [0.289.0] - 2026-07-14

### Fixed
- OpenAI Chat Completions: accept an assistant tool-call turn with an explicit `"content": null` (the standard OpenAI shape emitted by Qwen Code) instead of returning `400 invalid chat request: data did not match any variant of untagged enum MessageContent`. `#[serde(default)]` only covered an absent `content` key; a small deserializer now maps an explicit `null` to the default empty content. (#682)

## [0.288.0] - 2026-07-14

### Fixed
- Route natural-language file-creation requests ("create/write/save/generate a file …") to the `write` tool instead of emitting a `read` on the not-yet-existing target (issue #681). Write intent now beats read intent across every supported language via a general `has_file_write_intent` gate, and the write planner recognises the `named …` + `with the content …` phrasing.

## [0.287.0] - 2026-07-14

### Fixed
- Corrected documentation that had drifted from the codebase: `ARCHITECTURE.md` no longer claims a nonexistent `Event::Impulse` enum variant or `parent_id`/`language`/`surface` event fields, lists all 18 `SolverConfig` knobs instead of 9, drops four event kinds that no longer exist, renumbers a duplicated section 4.4, counts five rule shapes instead of four, documents the VS Code surface, and states honestly that ~26,700 lines of solver logic still live in `src/web/worker/*.js` (issue #658) rather than implying the JavaScript boundary is already narrow.
- `CONTRIBUTING.md` no longer carries template boilerplate: the title and clone URL name `formal-ai` instead of `rust-ai-driven-development-pipeline-template`, the project-structure tree reflects the real repository, and the line-limit rule distinguishes the 1000-line Rust cap from the 1500-line `.lino`/worker-JS caps.
- `docs/meta-algorithm.md` records the previously undocumented recursive-core recipe (issue #559) and corrects the procedural how-to record counts (11 roles, 8 functions, 6 stages, 4 parity pairs) to match the grounding suite.
- `docs/ci-cd/troubleshooting.md` invokes `rust-script scripts/publish-crate.rs` instead of a `node scripts/publish-crate.mjs` file that does not exist.
- `docs/testing/agentic-cli-tools.md` and the generated `docs/diagrams/agentic-recipes.md` now state their real scope instead of implying the multi-CLI CI matrix (issues #625/#671) and the full planner router set are already covered.
- Fixed the stale `SolverConfig::selection_mode` doc comment, which described `Legacy`/`Registry`/`Compare` variants that R344 replaced with `Off`/`Record`.

## [0.286.0] - 2026-07-14

### Changed
- Synchronized `VISION.md`, `GOALS.md`, `NON-GOALS.md`, `ROADMAP.md`, `ARCHITECTURE.md`, and `docs/USER-JOURNEYS.md` with the 2026-07-14 full-history requirement audit (issue #651): the roadmap now tracks requirement-level status (done / partial / not done), stale "current PR" headings reference their merged PRs, and the vision records the self-evolution frontier (world models #702, orchestration #703, portfolios #704, prediction #705, any-language #706, computer-use #707).
- Updated the doc-traceability pins in `tests/unit/docs_requirements*` to match the corrected wording.

### Added
- `docs/case-studies/issue-710/` preserving the raw audit reports over all 329 closed issues, 317 merged PRs, 31 open issues, and 17 open PRs, plus the konard/problem-solving methodology digest.

## [0.285.0] - 2026-07-14

### Added
- Tool-call emission in `formal-ai serve` is now **intent-based** rather than
  phrasing-gated (issue #680). When a client advertises a web-search, web-fetch, or
  write/edit tool, a request expressing that intent in *any* phrasing — across en, ru,
  hi, and zh — routes to the matching `tool_call` instead of a prose description. The
  routing holds over all three wire surfaces the target CLIs use (OpenAI Chat
  Completions, OpenAI Responses, and Gemini `generateContent`), and only fires when the
  matching capability tool is actually advertised, so a request that cannot be honoured
  still falls through to the prose answer.
- A file-creation intent that names a relative target file and literal content now
  routes to the advertised write tool in any phrasing/language. The write intent is
  recognised entirely from the seed lexicon (the new `file_write_*` roles in
  `data/seed/meanings-file-write.lino`) rather than from hardcoded English or Russian
  phrasings (CONTRIBUTING §2), and is probed before the file-read router so
  "create file X containing Y" is a *write*, not a read of X.
- A file-modification intent that names a target file plus an old→new replacement
  ("In greeting.txt, change hello to goodbye", "Replace foo with bar in notes.txt",
  «замени привет на пока в файле заметки.txt») now routes to the advertised edit tool,
  whatever the CLI calls it (`edit`, `replace`, `apply_patch`, `str_replace`). The
  new `Capability::Edit` recovers the `(target, old, new)` triple entirely from the
  seed lexicon (the new `file_edit_*` roles in `data/seed/meanings-file-edit.lino`),
  emits every common argument-key alias so one plan drives any CLI's edit tool, and is
  probed after the create-file write router and before the file-read router so an edit
  is never mistaken for a write or a read.
- A semantic shell request that never names the command — expressing an *intent* such as
  "Print the current working directory", "How much disk space is free?", or "What is my
  username?" — now routes to the advertised run tool carrying the concrete command
  (`pwd`, `df -h`, `whoami`) instead of a prose answer. The intent→command table,
  including multilingual cue phrases and per-intent argument recovery (`wc -l Cargo.toml`,
  `mkdir build`), lives in the new `data/seed/shell-intents.lino`, so coverage is retuned
  by editing seed data rather than the planner (CONTRIBUTING §2). It runs as a fallback
  after the named-command (#676) and directory-listing routers, so existing shell
  behaviour is unchanged, and only fires when a run/shell tool is advertised.

### Fixed
- The Russian navigation verb "загрузи" (load) is no longer misclassified as an
  `http_fetch`; it stays with `url_navigate`, while "скачай" (download the bytes)
  remains the fetch verb, so bare-domain navigation prompts resolve to an HTTPS link
  without fetch advice (issue #680).
- The general write router no longer mistakes a sentence-ending word for a target file:
  a token whose only dot is a terminal `.`/`!`/`?` ("… add the plural to томат.") is no
  longer treated as a dotted filename, so stored recipe requests are not hijacked
  (issue #680).

## [0.284.0] - 2026-07-13

### Added
- Add a replayable Hive Mind → Agent CLI → Formal AI self-coding scenario and CI-pinned evidence.

## [0.283.0] - 2026-07-13

### Added

- Added an issue #482 Nemotron 3 Ultra training-data sample suite: a
  no-full-download Hugging Face row sampler, a compact 10-row CC-BY-4.0 legal
  training-data fixture, benchmark/catalog provenance, and unit ratchets that
  verify sampler output, row provenance, digests, and `length=1` ingestion.

## [0.282.0] - 2026-07-13

### Added
- The assistant now honours being named in conversation. After "Now your name is
  Ineffa" (or "I'll call you Ada", "you are called …") it acknowledges the name and
  recalls it when later asked "what is your name", using dialog-local memory with no
  server state — mirrored in the browser worker (issue #676).
- Reasoning traces now open with a human, first-person narrative of what the
  assistant understood and decided ("You asked how I'm doing, so I told you and
  offered to help.") instead of an identical per-intent category template. The
  concrete steps remain beneath it as an expandable, recursive "robotic detail"
  layer. Applied to the API/CLI reasoning field (what agentic clients such as
  OpenCode render) and mirrored in the web thinking preview across en/ru/zh/hi
  (issue #676).

### Fixed
- Agentic planner now runs any seed shell token (`pwd`, `git`, `cargo`, …) named in a
  prompt, not just `ls`. `execute pwd`, `run git status`, and their many phrasings map
  to the real command (issue #676).
- Natural-language file-listing requests such as "give me a list of files in current
  folder" resolve to `ls` across many more phrasings (issue #676).
- Self-healing now triggers on natural self-directed repair requests such as "Can you
  fix it yourself?", "debug yourself", or "heal yourself", while ordinary "fix this
  file" requests still stay out of the repair loop (issue #676).
- "How are you?" small talk now gets its own warm wellbeing reply instead of the
  generic greeting. A dedicated `wellbeing` intent is matched before `greeting`
  (first-match-wins), so "how are you", "как дела", "आप कैसे हैं", and "你好吗" reply
  with an actual answer across en/ru/hi/zh — mirrored in the browser worker (issue
  #676).

## [0.281.0] - 2026-07-13

### Added
- Compose deterministic, capability-tagged Agent CLI plans for safe file-oriented change requests that are not encoded as pinned recipes.

## [0.280.0] - 2026-07-13

### Added
- Symbolic world models & contexts for issue #649 (`src/world_model.rs`): a
  first-class `Context` (a links network plus dependent statements), a
  `WorldModel` holding the per-dialogue `current`, `target`, and shared `general`
  contexts, and an `Action` modeled as STRIPS-style add/delete link edits. The
  module exposes the current→target `difference` (add / remove / conflicting
  links), predicts an action's consequences without mutating the model
  (`Context::predict` = apply-to-a-clone + recalculate + diff), recalculates every
  dependent statement's relative-meta-logic probability to a bounded fixpoint when
  the world changes (JTMS-style cascade over `Dependency` justification edges), and
  merges/splits contexts (ATMS-style). Reuses the existing `SubstitutionGraph`,
  the `relative_meta_logic` kernel, and `stable_id` content addressing; covered by
  `tests/unit/issue_649_world_model.rs`.
- Design case study for issue #649 under `docs/case-studies/issue-649/`: a deep
  analysis mapping the current-state / target-state world models, context
  merge/split, dependent statements, and action-consequence prediction onto the
  associative stack (links networks, `SubstitutionGraph`, the relative-meta-logic
  kernel, symbolic probability), with cited online research (STRIPS/PDDL,
  JTMS/ATMS, AGM belief revision, the JEPA world-model literature), a
  per-requirement solution plan and prior-art survey, requirement rows R428–R434
  in `REQUIREMENTS.md`, and the `tests/unit/docs_requirements_issue_649.rs`
  traceability test.

## [0.279.0] - 2026-07-13

### Fixed

- Keep Responses API instructions separate from the latest user request, make
  `formal-ai with` interactive/headless mode selection uniform across all
  supported tools, handle inline compaction prompts, and accept `--globally`.

## [0.278.0] - 2026-07-12

### Added
- Make `formal-ai with` auto-start a temporary agent-mode server, disable supported client summarization by default, isolate one-shot configuration, and support Claude Code, Qwen Code, Grok Build, and Aider.

## [0.277.0] - 2026-07-12

Issue #540 adds default-on dreaming maintenance planning for memory. The new
`formal-ai memory dream` command reports recomputable duplicate cleanup,
low-use cache/intermediate eviction under a 20% free-space target, and
storage-migration needs without mutating memory unless `--apply --confirm` is
used. The desktop shell now schedules the plan-only task in the background at
low priority.

Issue #540 dreaming now learns and generalizes, not just garbage-collects. While
idle it recalculates which topics the user interacts with most, remembers the
durable requirements the user has stated on them so he never has to repeat
himself, and generalizes each requirement into a meta-algorithm amendment baked
into memory as retained, never-forgotten learning (`meta_algorithm_amendment`).
Because an amendment can reproduce the specific task/test-run records it covers,
those specifics are forgotten first under storage pressure (the new
`ForgetCoveredSpecific` action) while the generalization is kept forever. The
dreaming meta-algorithm is now recorded as grounded data in
`data/meta/dreaming-recipe.lino`, pinned to the live source by
`tests/unit/specification/dreaming_meta_algorithm.rs`.

The follow-up completes that loop: structured amendments are now read by future
chat and Responses requests; coverage requires exact replay; repeated task
structures and multilingual data cues feed learning; real filesystem pressure,
incoming bytes, and persisted consent govern minimal cleanup; and core plus
desktop workers run only while idle and yield to foreground work. A complete
Formal AI Agent CLI gap-audit session is preserved with the issue case study.

# Issue #540: verified organic learning and runtime regression tests

- Amendments now form for any topic with stated requirements, so organic
  chat-only memory stores (raw messages plus durable task events) learn rules
  even before reproducible specifics exist.
- Refinement folds back only explicit `Learned standing requirement (...)`
  projection marker lines; free-form prose that merely quotes a requirement
  (such as solver fallback text) no longer pollutes rules.
- New regression tests: coverage revocation on rule change, eviction fallback
  for unverifiable records, the full organic record→dream→apply loop through
  the production chat path, refinement resurrection, durable failure records,
  numeric-pattern trial synthesis, multilingual task-kind gating, and the core
  dreaming runtime (idle gate, mid-run yield, `FORMAL_AI_DREAMING` opt-out,
  serve() wiring, locked atomic writes, desktop `PRIORITY_LOW`).

## [0.276.0] - 2026-07-09

### Added
- Recognize a language-agnostic "learn from this data source" directive so the
  reported issue #499 prompt is routed to a new `learn_from_source` intent instead
  of `intent: unknown`. Recognition is data-driven from a seed-declared
  learnable-source registry (`data/seed/learning-sources.lino`) shared by the chat
  handler and the Agent CLI planner, and the same teaching directive drives
  Formal AI's own Agent CLI learning recipe end-to-end (pinned session plus a live
  external-CLI E2E step in CI).

## [0.275.0] - 2026-07-09

### Added

- Added the issue #498 Google Trends catalog pipeline: parse a Trends RSS snapshot, expand the top 10 searches into multilingual prompt variants, answer every prompt through `FormalAiEngine`, and render the reviewable catalog at `data/meta/google-trends-catalog.lino`.
- Added a `google_trends_catalog` Agent CLI recipe with a pinned session under `docs/case-studies/issue-498`, plus raw Trends/GitHub evidence and tests that keep the seed, generated catalog, recipe routing, and documentation traceable.

### Added

- Closed the issue #498 auto-learning loop: `trending_learning_report()` re-answers every Google Trends catalog prompt, separates the ones the engine already routes from the *learning frontier* it cannot yet resolve, and hands that frontier to the human-gated issue #558 self-improvement learner. Because trending searches are open-domain questions, the learner honestly adopts nothing; the proposal-only result is rendered at `data/meta/google-trends-learning.lino`.
- Added a `google_trends_learning` Agent CLI recipe (`GOOGLE_TRENDS_LEARNING_TASK`) with a pinned session under `docs/case-studies/issue-498`, plus tests that keep the frontier split, proposal-only run, recipe routing, and documentation traceable byte-for-byte.

### Fixed

- Made the live Agent-CLI ↔ formal-ai E2E harness (`experiments/agent_cli_e2e/run_agent_cli.sh`) resilient to the third-party `@link-assistant/agent` CLI's non-deterministic early exit: the deterministic server plans the same next step every time, but the external CLI occasionally stops after the first tool round without writing the file, so the harness now retries the whole invocation up to `ATTEMPTS` (default 5) times and still enforces every hard assertion on a genuine, complete round-trip.

## [0.274.0] - 2026-07-07

### Documentation

- Added the issue #558 auto-learning case study, PR #601 gap analysis,
  requirements matrix, online research notes, and phased self-learning solution
  plan.
- Corrected the root issue #538 requirement status so delivered Agent CLI,
  diagram, and self-AST slices are no longer described as missing follow-ups.

### Added
- Issue #558 auto-learning: a closed, human-gated self-healing loop (`src/self_healing.rs`) that composes a failure trace, a verified source↔links round-trip, a benchmark-gated candidate lesson, and a terminal human-review outcome into one auditable `RepairCase`.
- `SourceRoundTrip::for_pinned_target` proves a real module survives a byte-for-byte `source → links → source` round-trip (the first genuine Links-to-source direction, not just a census).
- Fifth agentic recipe (`src/agentic_coding/self_heal.rs`): the self-healing loop is reachable through the agentic interface (Codex / OpenCode / Gemini / Agent CLI or the in-repo driver), emitting the repair case as `data/meta/self-healing-case.lino`. Adoption stays a human decision — nothing is auto-written.

### Added
- Issue #558 auto-learning (R558-04/R558-05): the **entire** source code of Formal AI is now translatable to the links / meta language and back. `build.rs` embeds every owned `src/*.rs` file (`OWNED_SOURCE_FILES`) so the whole tree is present in our data, and `src/self_source_graph.rs` content-addresses all of it and proves every owned module round-trips byte-for-byte through the sole CST/AST engine (`SourceGraph::owned`, exhaustive lossless proof).
- Sixth agentic recipe (`src/agentic_coding/source_graph.rs`): the whole-repository source↔links projection is reachable through the agentic interface, emitting a read-only Links Notation projection document (`self-source-graph.lino`). Nothing writes source back — the recompile-itself guardrail stays human-gated.
- `project_source_graph` example prints the exhaustive whole-repository projection for review.

### Added
- Issue #558 auto-learning (R558-03): `src/learning_ledger.rs` is the single, human-gated promotion protocol that terminates the self-healing loop. `LearningLedger::promote` records a `RepairCase` as a durable *approved learning record* only when **both** the benchmark gate is green **and** a human approves, and refuses every other case with a specific reason (`TestsNotGreen`, `NoReviewableProposal`, `SourceNotFaithful`, `HumanDeclined`, `AlreadyPromoted`). A repeated failure is then answered from the ledger instead of re-derived — the concrete payoff of "auto learning".
- Seventh agentic recipe (`src/agentic_coding/ledger.rs`): the promotion ledger is reachable through the agentic interface, emitting the approved learning record as a Links Notation document (`learning-ledger.lino`). The document records an already-approved decision, so nothing new is adopted and the recompile-and-reattach guardrail stays human-gated.
- `dump_learning_ledger` example prints the canonical approved ledger; `data/meta/learning-ledger.lino` is the generated, byte-for-byte-pinned artifact.

### Added
- Issue #558 auto-learning (R558-08): `src/self_explanation.rs` answers "how does Formal AI work?" grounded in the system's *own* source, data, and tests rather than prose docs. Each topic cites real artifacts; every `CitationKind::Source` citation resolves its `content_id` from the owned manifest and *panics* if the path is not an owned source file, so a fabricated citation cannot be constructed. The rendered Links Notation is anchored to the whole-source manifest content id that the source-to-links round-trip proves lossless.
- Eighth agentic recipe (`src/agentic_coding/explain.rs`): the grounded self-explanation is reachable through the agentic interface, emitting `how-formal-ai-works.lino`. Like the source-graph recipe it commits no byte-pinned artifact because the citation ids track the whole source tree.
- `explain_formal_ai` example prints the canonical grounded explanation.

### Added
- Issue #558 auto-learning (R558-07): `src/change_request.rs` turns a natural-language "change Formal AI itself" request into a reviewable pull request through the *same* human-gated repair loop the ledger uses. A request plus a target module becomes a `ChangeRequest` — a normalised requirement, a proposed test name, and an ordered patch plan whose target is grounded against the owned manifest (`ChangeRequest::for_module` *panics* on any path the repository does not ship, so a request can never target fabricated source). `ChangeRequest::review` merges the change only when a `BenchmarkGateReport` is green *and* an explicit `HumanApproval` is granted, refusing every other case (`TestsNotGreen` / `HumanDeclined`); neural inference stays a NON-GOAL, and the patch is a deterministic plan a human or Agent CLI executes, not generated code.
- Ninth agentic recipe (`src/agentic_coding/change_request.rs`): the user-driven self-change is reachable through the agentic interface, emitting `requested-change.lino`. Like the source-graph and explain recipes it commits no byte-pinned artifact because the target's manifest content id tracks the whole source tree.
- `request_change` example prints the canonical change request and demonstrates the accept/decline review gate.

### Added
- Issue #558 auto-learning (R558-02): `src/repair_strategy.rs` is the *general* front of the failure-to-repair loop. The self-healing slice repairs a single canonical failure by synthesising a solver method; this generalises it. `RepairStrategy::classify` reads an arbitrary `UnknownTrace` — the same trace the self-healing loop reasons about — and, purely deterministically from the trace's own prompt and event signals, maps it onto exactly one of the three targets issue #558 names (`RepairTarget::SolverMethod` / `DataRecord` / `Test`), so the loop is *total* — every failure is classified. For each it composes the grounded repair plan (rationale, proposed change scoped to the target class, and the automated verification that must be green before human promotion). It stays proposal-only and human-gated; neural inference stays a NON-GOAL — the classification and plan are deterministic functions of the trace, and the "change" is a plan a human or Agent CLI executes, never generated code applied automatically.
- Tenth agentic recipe (`src/agentic_coding/repair_strategy.rs`): the general classifier is reachable through the agentic interface, emitting the three canonical strategies (one per target class) as `repair-strategies.lino`. Unlike the source-graph, explain, and change-request recipes it commits a byte-pinned artifact (`data/meta/repair-strategies.lino`), because the document depends only on self-contained canonical traces, not the whole source tree — asserted byte-for-byte against a fresh render like the self-healing repair case.
- `classify_repair` example prints the grounded, human-gated repair strategy the classifier composes for each of the three failure classes.

### Added
- Issue #558 auto-learning (R558-06): `src/rebuild_plan.rs` closes the final loop — *"recompile and reattach the improved code to the UI."* `RebuildPlan::for_accepted_change` derives a plan purely from an already-accepted change (a green benchmark gate AND a human approval), so a rebuild can never precede acceptance. It grounds every reattached UI artifact against the real repository bytes and the owned manifest (`Cargo.toml`, `src/main.rs`, `src/web/formal_ai_worker.js`, `src/web/index.html`), and emits a strictly ordered, observable, reversible five-step pipeline (recompile → regenerate worker → reattach → hot-swap → verify). The regenerated `formal_ai_worker.wasm` is deliberately absent from the grounded inputs — it is the pipeline's *output*, referenced by the steps. The plan stays proposal-only and human-gated; nothing rebuilds or hot-swaps automatically, and neural inference stays a NON-GOAL — the plan is a deterministic, content-addressed function of the accepted change that a human or Agent CLI executes.
- Eleventh agentic recipe (`src/agentic_coding/rebuild_plan.rs`): the rebuild-and-reattach plan is reachable through the agentic interface as `rebuild-and-reattach.lino`. Like the source-graph, explain, and change-request recipes it asserts a *live* document (never a byte-pinned artifact), because the plan depends on the whole owned source tree; it keys on `reattach` so it stays disjoint from the source-graph recipe, which owns `recompile`.
- `rebuild_and_reattach` example prints the grounded, human-gated recompile-and-reattach pipeline the plan composes for the accepted canonical change.

### Added

- Added a lazy, configurable issue #527 question-generation API with grammar and meaning classification plus an answer stream through `FormalAiEngine`. The generator is language-agnostic: `QuestionGenerationConfig::for_language` and `question_lexicon_summary_for_language` drive the same enumeration, frequency-tiering, and classification over English, Russian, Hindi, and Chinese vocabulary seeded in `data/seed/question-generation-lexicon.lino`, so no language-specific code path exists.
- Added the `question_catalog` agentic recipe (the eleventh) that drives Formal AI through its own Agent CLI to enumerate questions smallest-first, classify them, answer the meaningful ones, and record the reviewable catalog in Links Notation (`data/meta/question-catalog.lino`), grounded in the `data/seed/question-generation-lexicon.lino` frequency-tier vocabulary. Answered questions form a case/whitespace-insensitive recall table (`QuestionCatalog::answer_for`) that never mutates the human-gated learning ledger.

## [0.273.0] - 2026-07-04

### Added
- Added issue #526 round-trip translation quality requirements, natural/code translation regression coverage, and case-study documentation.

### Changed
- Reworked code translation (`translate_program`) to route through a language-neutral code meta language (`CodeMeaning` / `formalize_code_meaning` / `render_code_meaning`) instead of direct `(source, target)` pairs, so it stays at `N` formalizers + `N` renderers and pairs like Python → JavaScript or Rust → Go translate through one shared meaning.

## [0.272.0] - 2026-07-04

### Fixed

- Aligned one-shot `with-formal-ai codex` invocations and documentation with the
  direct Codex examples by applying `--sandbox read-only` alongside
  `--skip-git-repo-check`.

## [0.271.0] - 2026-07-04

### Documentation

- Added an agentic CLI testing guide with fixture markers, logging proxy
  provenance checks, phrasing matrices, and CI e2e assertions for `codex`,
  `opencode`, `gemini`, and `agent`.

## [0.270.0] - 2026-07-04

### Fixed

- Agent-mode OpenAI-compatible tool planning now routes local file-reading
  prompts to `read`/`bash` tool calls instead of treating filenames such as
  `beta.md` as URLs or falling through to non-agentic answers.

## [0.269.0] - 2026-07-04

### Fixed
- Routed natural-language current-directory listing prompts, such as "what files are in this folder?", to agent-mode shell tool calls with `{"command":"ls"}` instead of falling through to the unknown-answer response.

## [0.268.0] - 2026-07-04

### Added
- Added `formal-ai proxy`, a built-in logging reverse proxy that forwards HTTP traffic to a Formal AI server and appends JSONL provenance/routing summaries.

## [0.267.0] - 2026-07-03

### Fixed
- Fixed Codex Responses compatibility by matching shell tool-call arguments to the advertised `cmd` schema, returning `slug` in OpenAI-compatible model metadata, and allowing `with-formal-ai codex` to start outside Git worktrees.

## [0.266.0] - 2026-07-03

### Added
- Added `with-formal-ai agent` support with inline Agent CLI config and
  persistent `~/.config/link-assistant-agent/opencode.json` setup.

## [0.265.0] - 2026-07-03

### Fixed

- Isolated one-shot `with-formal-ai gemini` invocations from cached Gemini CLI
  OAuth settings by selecting API-key auth in a temporary Gemini home and
  enabling workspace trust.

## [0.264.0] - 2026-07-03

### Added
- Added deterministic OCR/text market-price claim extraction for the document verification path, including ETH aliases across supported languages and source-backed relative-meta-logic assessments.
- Mirrored market-price contradiction checks in the browser worker and added an e2e regression for the image/OCR flow.
- Preserved issue #493 evidence under `docs/case-studies/issue-493`, including the screenshot, OCR output, market-data captures, and before/after regression logs.

### Fixed
- Preserved full multi-line OCR/text samples during document verification so factual claims after the first line are checked instead of being dropped from the statement plan.
- Flagged `ETH in 2024: $1,700` as contradicted using captured Binance ETHUSDT 2024 daily klines.

## [0.263.0] - 2026-07-03

### Added

- Route the whole class of externally verifiable questions to web research by
  reasoning about the referent instead of memorising topic vocabulary: any
  interrogative that names a referential external entity — a brand written with
  interior capitalisation such as `ChatGPT`, `OpenAI`, `iPhone`, or `TypeScript`
  — and that the solver cannot resolve from local memory now routes to the
  source-gathering research plan. The structural rule fires identically for
  English, Russian, Hindi, and Chinese prompts and across any topic (pricing,
  release dates, hardware specs, features), so the Russian annual-discount prompt
  for Claude Max and ChatGPT Pro is handled as one instance of the general class
  rather than by a product-specific answer or a stored word list.

## [0.262.0] - 2026-07-03

### Fixed

- Answer previous-user-question recalls such as Russian `что я спрашивал` from the user's earlier request instead of the assistant's previous reply.

## [0.261.0] - 2026-07-03

### Added
- Expose solver thinking traces through OpenAI Chat `reasoning_content`, OpenAI Responses reasoning summary output/events, and Anthropic thinking documentation.

### Added
- Added `formal-ai with` and the standalone `with-formal-ai` wrapper for running or permanently configuring Codex, OpenCode, and Gemini against a local Formal AI server from seed-backed client integration templates.

## [0.260.0] - 2026-07-03

### Fixed

- Covered the legacy `/v1/responses` route with a real loopback HTTP streaming
  regression test so Codex-style Responses SSE clients must receive
  `response.completed`.

### Documentation

- Documented a copy-paste Codex 0.142+ configuration and `codex exec "hi"`
  command for driving Formal AI through the Responses wire API.

## [0.259.0] - 2026-07-03

### Added

- Added `/api/<protocol>/...` gateway routes for OpenAI, Anthropic, Gemini,
  Vertex, and formal-ai native APIs, with per-protocol model discovery.

### Fixed

- Added named OpenAI Responses SSE events for streaming Responses clients,
  including the final `response.completed` event.

## [0.258.0] - 2026-07-02

### Fixed
- Renamed the advertised model id to `formal-ai` and accepted seed-backed aliases such as `@link-assistant/formal-ai`.

## [0.257.0] - 2026-07-02

### Fixed
- Added real loopback HTTP regression coverage for OpenAI Chat Completions `stream:true` responses so the SSE stream must use `chat.completion.chunk` frames with `choices[].delta.content`, and documented a verified OpenCode `hi` setup.

## [0.256.0] - 2026-07-02

### Fixed

- Agentic Chat Completions now emits `bash` / `shell` / `run_command` tool calls
  for natural-language `ls` directory-listing requests when agent mode is enabled,
  so the Link Assistant Agent CLI can execute and return the listing.

### Added

- `formal-ai serve --agent-mode` as the documented command-line opt-in for
  OpenAI-compatible agent clients, alongside the existing `FORMAL_AI_AGENT_MODE=1`
  environment variable.

## [0.255.0] - 2026-07-02

### Added

- Grammatical detail for the tomato **and potato** meanings (issue #538): every
  surface (`tomato`/`tomatoes`, `помидор`/`помидоры`, `томат`/`томаты`,
  `potato`/`potatoes`) now pins its part of speech and grammatical number
  (singular/plural) in the seed data, and the previously missing plurals `томаты`
  and `potatoes` were added.
- New `grammatical_number` semantic facet kind plus `WordForm::grammatical_number()`,
  `WordForm::part_of_speech()`, and `WordForm::denotations()` accessors.
- Grounded, multilingual `grammatical_number` / `singular` / `plural` meanings
  (Wikidata `Q104083` / `Q110786` / `Q146786`) lexicalised in en/ru/hi/zh, with
  cached Wikidata data for offline grounding-closure tests.
- The meaning-detail change is produced by **driving Formal AI through its own
  in-repo Agent CLI** (`src/agentic_coding/`), with the committed seed asserted
  byte-for-byte equal to the driver output. A concept registry generalises the
  recipe, proven by driving tomato and potato with two *differently worded*
  requests. The Agent-CLI sessions that solved the task are committed
  (`docs/case-studies/issue-538/agent-cli-session*.json`), and
  `scripts/reproduce-issue-538.sh` regenerates the change on a clean checkout.
- Generated agentic-recipe **mermaid diagrams**, split into parts
  (`docs/diagrams/agentic-recipes.md`), rendered from the planner's own recipe
  table by `src/agentic_coding/diagram.rs` — a non-lexeme axis (issue #538
  R15/R16) proving the Agent-CLI method generalises beyond meaning data. The Agent
  CLI writes the document from a *third* differently worded request; the document
  and its session JSON are reproduced byte-for-byte under test.
- Self-inspection **CST/AST census** recipe (`src/agentic_coding/self_ast.rs`,
  issue #538 R13): the meta algorithm parses one of its own Rust modules (the
  deterministic planner) through the repo's sole CST/AST engine — the
  link-foundation `meta-language` links network — and stores the abstract-syntax
  node census in our data as Links Notation (`data/meta/self-ast.lino`). The
  census logic is general (works on any Rust source, proven by tests over several
  sources), the Agent CLI drives it from a *fourth* differently worded request
  (`docs/case-studies/issue-538/agent-cli-session-self-ast.json`), and the
  committed artifact is reproduced byte-for-byte under test.
- `formal-ai agent --session-json <path>` to capture a replayable Agent-CLI
  session as JSON.
- Case study `docs/case-studies/issue-538` with a requirements decomposition,
  per-requirement solution plan, online research, and a `refusal-anti-pattern.md`
  recording the rejected "ship a slice, defer the rest" reasoning.
- Real Agent CLI ↔ formal-ai E2E round-trip test
  (`experiments/agent_cli_e2e/run_agent_cli.sh`) that boots `formal-ai serve`
  and drives it with the **external** `@link-assistant/agent` CLI over the
  OpenAI-compatible endpoint — no mocks. Wired as the new `test-agent-cli-e2e`
  CI job in `.github/workflows/release.yml` (running all four recipe axes —
  tomato, potato, diagrams, and the self-AST census — against the real server),
  and the real captured console log is committed at
  `docs/case-studies/issue-538/agent-cli-e2e-run.log` so the round-trip evidence
  is inspectable, not synthesised.

### Changed

- Made the Agent-CLI-driven, no-deferral development workflow the **standing
  rule** in `CONTRIBUTING.md`: from this task forward Formal AI changes are
  produced by driving the Agent CLI (never hand-editing, never deferring to
  follow-ups), with the tool extended when it cannot yet do the work. Added
  four further standing rules covering the real Agent-CLI E2E requirement,
  hardcoded cases only in tests, real captured logs in case studies, and small
  atomic commits.
- Fixed a TOCTOU race in `AgentWorkspace::for_prompt` (parallel runs with the
  same prompt shared a deterministic temp dir) via a per-instance unique
  workspace id.
- Split the meaning-lexicon seed parser into `src/seed/meanings/parse.rs` so both
  `meanings.rs` and the new module stay under the Rust file-size guard after the
  grammatical-detail additions (mirrors the existing `roles.rs` split).
- Reworked the meaning-detail recipe (`src/agentic_coding/meaning_detail.rs`) to
  **derive** every enriched surface from real, checked-in Wikidata lexeme JSON
  (parsed by a general serde_json algorithm) instead of hardcoded answer tables:
  the singular form is anchored to the lexeme's lemma and the plural is paired by
  matching non-number grammatical features, so the logic is general for any case
  paradigm and references no hardcoded case id. Hardcoded strings now live only in
  tests; the four seed blocks are reproduced byte-for-byte from the source JSON.

## [0.254.0] - 2026-07-01

### Added
- Generalized the document-originality handler into a full verification class: authenticity, factual-accuracy, and veracity requests (not only plagiarism/uniqueness) now route to the same grounded workflow across English, Russian, Hindi, and Chinese.
- Weighed every extracted statement with relative-meta-logic (github.com/link-foundation/relative-meta-logic): statements start from an assumed-true prior, are raised by trusted original-first sources, lowered by contradicting originals, and unoriginal reposts are ignored — recorded deterministically in the append-only event log.
- Grounded each statement with a dedicated fact-check web-search query, mirrored byte-for-byte into the Web app worker so the browser matches the Rust engine.

### Fixed
- Routed multilingual text-attachment originality and plagiarism checks through a grounded attachment workflow instead of falling back to unknown.
- Included sampled text/plain attachment content in Web app solver context so browser uploads can be inspected by deterministic handlers.
- Folded Telegram document attachments into the shared attachment-context builder so forwarded files reach the same originality/verification handler.
- Classified `.lino` seed and language-resource changes as code in `detect-code-changes.rs` so editing files the language-change-parity guard watches (e.g. `src/web/i18n-catalog.lino`) now triggers lint/test instead of silently skipping them.

## [0.253.0] - 2026-07-01

### Added
- Issue #556: generalized the response-language follow-up beyond repository lookups to the whole class of "re-answer the previous request in another language" turns. A bare follow-up such as "I do not understand English, write in Russian" now replays the previous request through the entire solver with the target language forced at a single detection seam, so capabilities, identity, project lookups, and other localizable answers all re-render in any seeded language (English, Russian, Hindi, Chinese) — and reverse back to English on request.
- Grounded the follow-up in a machine-readable meta-algorithm recipe (`data/meta/response-language-followup-recipe.lino`), pinned by `tests/unit/specification/response_language_meta_algorithm.rs`, so the eight recursive-reasoning steps, seed roles, Wikidata groundings, handler functions, forced-language seam, and Rust↔JS parity targets can never silently drift from the live source. Documented in `docs/meta-algorithm.md`.
- Added round-trip translation tests (issue #526) proving English↔Russian/Hindi/Chinese vocabulary survives a source→meta-language→target→source cycle with both meaning and surface preserved.

### Fixed
- Issue #556: repository lookup language-change follow-ups now rerender the previous GitHub lookup in the requested seeded response language instead of falling through to unknown.

## [0.252.0] - 2026-06-30

### Added
- Natural-language access to the entire associative memory (issue #529). Queries now read across all stored memory events and projected memory links, and `formal-ai memory query` performs Turing-complete read+write control: appending new memory and applying substitutions that rewrite every matching stored value in place (not just recording intent). The WASM/browser app reaches parity: the JS worker recognizes the same multilingual append and substitution directives, and the browser persists them by appending memory events and rewriting matching stored values in IndexedDB. All paths are driven by the multilingual seed lexicon across English, Russian, Hindi, and Chinese.

### Fixed
- Asking "what was written in the previous message?" (and its Russian, Hindi, and Chinese equivalents) now recalls the previous message instead of returning an unknown intent, in both the Rust runtime and the browser JS worker (issue #529).

## [0.251.0] - 2026-06-30

### Fixed
- Recognize Russian calendar event prompts that use spoken-hour wording such as "на 10 часов" and route them to calendar event creation.

## [0.250.0] - 2026-06-30

### Fixed

- Guard the reported Russian hackathon dialog so `Где посмотреть актуальные хакатоны?` and the follow-up `Найди мне хакатоны` stay on the `web_search` route instead of the unknown fallback.

## [0.249.0] - 2026-06-30

### Fixed
- Route current public-event questions such as `Какие хакатоны сейчас проходят?` to web search instead of the unknown-intent fallback.

## [0.248.0] - 2026-06-30

### Added
- The Rust solver can now answer natural-language queries over prior dialog turns and persisted `.lino` memory, such as "When did I mention Rust?" or "Find Rust in another conversation", through the `conversation_recall` intent.
- The local HTTP surfaces (`/v1/chat/completions`, `/v1/responses`, and `/v1/messages`) now scan `FORMAL_AI_MEMORY_PATH` when a natural-language recall query asks about memory outside the current request history.
- The CLI now includes `formal-ai memory query --prompt ...` for direct natural-language recall over a saved `demo_memory` or `formal_ai_bundle` file.

## [0.247.0] - 2026-06-29

### Fixed

- Route unresolved bare term prompts such as `cursor` to web search after local knowledge sources miss instead of returning the unknown fallback.

## [0.245.0] - 2026-06-29

### Fixed
- Route multilingual event-listing prompts such as "Найди мне хакатоны" to web search instead of the unknown-intent fallback.

## [0.244.0] - 2026-06-29

### Fixed
- Routed telegraphic install how-to prompts such as `how install cursor` through official-documentation-first procedural discovery instead of the unknown fallback.

## [0.243.0] - 2026-06-28

### Fixed
- GitHub repository traffic questions now answer from official GitHub traffic documentation instead of falling to `intent: unknown`, including the reported Russian prompt.

## [0.242.0] - 2026-06-28

### Fixed
- Route multilingual topic-interest prompts to web search instead of the unknown-intent fallback.

## [0.241.0] - 2026-06-28

### Fixed
- Answer supported-language behavior-rule count questions such as `Сколько всего правил?` with the built-in, dialog-local, and total rule counts instead of falling through to the unknown fallback.

## [0.240.0] - 2026-06-28

### Fixed
- Behavior-rule follow-up prompts can now ask for the rule count or a brief localized recap after listing the rules, including the reported Russian prompts.

## [0.239.0] - 2026-06-28

- Concept lookup now honors data-driven response-language markers, so prompts
  such as "Tell me about Telegram Ads in Russian" render the known concept in
  the requested language instead of treating the language phrase as context.

## [0.238.0] - 2026-06-28

### Fixed
- **Issue #485 - multilingual elided `how <action> ...` prompts now route to procedural how-to.** The Rust solver and browser worker recognize seeded weak leads such as Russian `как ...` when the following action is approved by the procedural action lexicon, preserving greeting-prefixed compound answers instead of falling through to unknown.

### Fixed

- Replaced live crates.io/docs.rs status badges in GitHub release notes with
  static release-version artifact badges that still link to the exact published
  crate and documentation pages.
- Restored the README project badge block and added regression coverage plus
  issue #492 case-study evidence.

## [0.237.0] - 2026-06-28

### Fixed
- **Issue #481 - telegraphic `how order ...` prompts now route to procedural how-to.** The Rust solver and browser worker accept the weak `how ...` lead only when the following action is approved by the seed lexicon, so `how order 3d print in nan chang vietnam?` produces the normal source-backed discovery plan without broadly claiming arbitrary `how <word>` prompts.

## [0.236.0] - 2026-06-28

### Fixed

- Added a multilingual seeded concept for neural-network inference so `что такое нейросетевой инференс?` resolves through concept lookup instead of the unknown fallback.

## [0.235.0] - 2026-06-28

### Fixed
- Resolve unseeded English and Russian authorship prompts, such as War and Peace questions, through Wikidata `P50` instead of returning `unknown`.

## [0.234.0] - 2026-06-28

### Fixed

- Added a seed-backed fact answer for Russian Spider-Man film release-order prompts so they resolve to `fact_lookup` instead of `unknown`.

### Fixed

- Added a seed-backed Air India infant stroller baggage allowance answer so the reported Russian prompt resolves to `fact_lookup` instead of `unknown`.
- Corrected the Spider-Man film fact's Wikidata anchor and checked in the required Wikidata cache records for the new and corrected facts.

### Fixed
- Issue #477: Russian prompts like `Что такое кубаторит?` now resolve through the seeded concept dictionary entry for `кубаторить / кубатурить` instead of falling through to `unknown`, with Academic.ru source evidence.
- Corrected the Spider-Man film release-order fact's Wikidata anchor from stale `Q79054` to `Q2307877` and checked in the matching cache snapshot.

## [0.233.0] - 2026-06-27

### Fixed
- Resolve Rust creator follow-up questions by rewriting seeded pronoun references back to the Rust fact lookup path.

## [0.232.0] - 2026-06-26

### Fixed

- Solve train meeting relative-speed prompts with verification-tagged reasoning in Rust and the browser worker instead of falling through to `unknown` or `calculation_error` (issue #460).

## [0.231.0] - 2026-06-26

### Fixed
- **Issue #464 - clock-time duration prompts now route to the calculator.** The web worker and Rust solver now handle `17:30 - 14:00` and elapsed-time wording such as `If a train leaves at 14:00 and arrives at 17:30, how long is the trip?`, returning `3 hours, 30 minutes` instead of `unknown`.

## [0.230.0] - 2026-06-26

### Fixed
- Route composite crypto portfolio tracker prompts to a Python blueprint with mocked prices, alert logic, and a Markdown dashboard instead of generic search or missing-template fallbacks.

## [0.229.0] - 2026-06-26

Fixed
- Route class-based program requests such as "Write a Python class" through `write_program`, and answer the smart travel planner prompt with a Python `TravelPlanner` blueprint instead of falling through to search or an unsupported template.

## [0.228.0] - 2026-06-26

### Fixed
- Fixed Russian follow-up code requests like `На php не получится написать?` so they inherit the advertised Hello World task and return the cached PHP example instead of `unknown`.

## [0.227.0] - 2026-06-26

### Fixed

- Issue #457: route Rust self-source metrics and response-comparison prompts to
  a curated `write_program` blueprint instead of the missing-template fallback.

## [0.226.0] - 2026-06-26

### Fixed

- Kept terse research result follow-up prompts tied to the prior search attempt instead of defining the word "result" after a failed browser search.

## [0.225.0] - 2026-06-25

### Added
- Added repository-file formalization and summarization helpers that record file
  metadata, meta-language parser evidence, and recursive Markdown embedded
  grammar summaries.
- Generalized summarization from files to any repository resource, including
  folders: `RepositoryEntry`, `formalize_repository_resource`, and
  `summarize_repository_resource` summarize a directory tree by the recursive
  decompose → summarize → compose meta-algorithm loop, with recursion depth
  bounded by the summarization mode ladder and link-native `repository_directory`
  evidence.

## [0.224.0] - 2026-06-25

### Fixed
- Route verbless "records/financials/statistics about a subject" prompts such as `Financial records for boeing after crisis with icas system` to web search instead of the unknown-prompt report, with multilingual coverage (en/ru/hi/zh).

## [0.223.0] - 2026-06-25

### Fixed

- Issue #441: Russian definition prompts that start in Cyrillic and ask about
  Latin technical terms, such as `Что такое vulkan layer`, now keep
  `language:ru` instead of being misclassified as English and falling through to
  the unknown-intent answer. The browser worker mirror now follows the same
  detection rule.

## [0.222.0] - 2026-06-24

### Fixed
- Browser worker and Rust seed coverage now recognize length-vs-mass unit questions such as `Сколько метров в килограмме?` as `unit_incompatibility` instead of falling back to `unknown`.

## [0.221.0] - 2026-06-24

### Fixed
- **Issue #446 — large integer exponents in the web calculator were truncated.** Arithmetic fallback evaluation now keeps integer exponentiation exact, so prompts such as `10^100` render the full integer instead of `1e+1`.

## [0.220.0] - 2026-06-24

### Fixed
- **Issue #445 — compound courtesy/question prompts were treated as one unknown.** The solver now decomposes unresolved independent prompt parts, responds to greetings first, and then answers the following question segment while preserving existing specialized decomposition for algebra and list-style synthesis.

## [0.219.0] - 2026-06-24

### Fixed
- Deploy GitHub Pages from the resolved release commit so the website and API docs advertise the same version as the latest release.
- Retry `rust-script` installation in CI so transient crates.io HTTP failures do not fail unrelated workflow jobs.

## [0.218.0] - 2026-06-24

### Added
- Added the issue 559 planning case study for generalizing the meta algorithm architecture.
- Expanded the case study with a recursive link-native solver plan, Voyager design mapping, and upstream dependency audit.
- Deepened the case study into a spine plus companion documents (alignment, critical review, options comparison, recursive core, evidence pipeline), with a critical check (CR1–CR12), strategic re-check resolving conflicts C1–C7, a canonical vocabulary mapping onto existing VISION/REQUIREMENTS terms, option comparisons with `SolverConfig`-knob comparison harnesses, and proposed requirement rows R330–R335.

### Added
- Issue #559 (Phase 1A): an explicit, link-serializable problem frame. Every prompt now produces a `ProblemFrame` (`src/meta_frame.rs`) that wraps the formalized intent and enumerates every detected `Need` (questions, requirements, tasks) found across sentences and coordinating clauses. The frame is emitted as a trace-only `problem_frame` loop event and serialized to Links Notation via `format_lino_record`, making the meaning record first-class without changing routing or answers. Tracked by REQUIREMENTS.md R330.

### Added
- Issue #559 (Phase 1B): the recursive, bounded downward pass of the general meta algorithm. Every problem frame is now decomposed into a `WorkUnit` tree (`src/meta_frame.rs`) — each unit is either a direct-method leaf, an irreducible single need, or split into children, always stopping at `SolverConfig::max_decomposition_depth` so the recursion is terminating. The tree is serialized to Links Notation and emitted as trace-only `work_unit` / `work_unit:enter` / `work_unit:exit` loop events, so the recursive core is observable without changing routing or answers. The reasoning-loop guarantee is widened beyond arithmetic to prove each handler family still emits candidate/validation events when reached as a recursion leaf. Tracked by REQUIREMENTS.md R332.

### Added
- Issue #559 (Phase 2): the need-satisfaction ledger. Every problem frame now produces a `NeedLedger` (`src/meta_frame.rs`) with exactly one row per detected need, each carrying an explicit status derived from the recursive work-unit tree — a need that maps to a known method is satisfiable, while a need with no recognized method is recorded as `blocked` rather than silently dropped. This makes "address every detected need" structural rather than prose (R8). The ledger is serialized to Links Notation and emitted as trace-only `need_ledger` / `need:status` loop events, changing neither routing nor answers. Tracked by REQUIREMENTS.md R333.

### Added
- Issue #559 (Phase 3): the method registry as first-class link data. The catalogue of handlers each atomic work-unit leaf can route to — the ordered `SPECIALIZED_HANDLERS` table plus the five contextual overrides — is now derived from the live dispatch code (`src/method_registry.rs`, `MethodRegistry::from_dispatch`) and serialized to Links Notation, so the meta algorithm can read and reason about its own methods rather than having them locked away in Rust. The registry is recorded as a trace-only `method_registry` loop event, changing neither routing nor answers, and a grounding test pins every derived method name against `src/solver_dispatch.rs` so the data can never drift from the handlers that actually run. Tracked by REQUIREMENTS.md R331.

### Added
- Issue #559 (R335): the recursive meta core now describes *itself* as grounded link data. `data/meta/recursive-core-recipe.lino` enumerates the eight ordered steps that turn any message into a solved, link-native knowledge base — formalize the impulse, build the problem frame, decompose recursively into a bounded work-unit tree, account for every need in a ledger, catalogue the resolving methods, resolve each atomic leaf through the single ordered dispatch, record evidence, and project the answer — and pins each step to the live function that implements it. A grounding test (`tests/unit/specification/recursive_core_recipe.rs`) asserts the source still defines every named function, so the core's self-description can never drift from the code that runs. This is the concrete sense in which the meta algorithm can reason about itself: its own algorithm exists as data the engine can read. Tracked by REQUIREMENTS.md R335.

### Added
- Issue #559 (R334): the evidence pipeline. The meta core now joins its separate link artifacts — the problem frame, the recursive work-unit tree, the need-satisfaction ledger, and the method registry — into one end-to-end `SolutionEvidence` record (`src/solution_evidence.rs`). For every detected need it traces the full chain `frame need → work-unit leaf → ledger status → catalogued method`, with `accounted_for` (every need has a connected, non-pending status) and `fully_resolved` (every need is satisfied) flags, so "ensure every detected need is addressed in the response" is a single auditable fact rather than four projections a reader must reconcile by hand. The ledger rows gained additive `unit_id`/`route` links to support the join. The evidence is serialized to Links Notation and emitted as a trace-only `solution_evidence` loop event, changing neither routing nor answers. Tracked by REQUIREMENTS.md R334.

### Added
- Issue #559: a runnable example (`cargo run --example issue_559_meta_core`) that emits the meta core's Links Notation artifacts — problem frame (R330), recursive work-unit tree (R332), need-satisfaction ledger (R333), method registry (R331), and solution evidence (R334) — for a single routed need, a conjunction, and an unroutable need, offline with no network or neural inference.
- Issue #559: a deep, data-grounded case-study analysis (`docs/case-studies/issue-559/implementation-results.md`) recording what shipped against the plan, walking the real emitted artifacts, and surfacing a genuine route↔method vocabulary gap the unified evidence projection exposes (the routing label `write_program` does not match any registered handler name, so `resolved_to_method` honestly trails `trail_count`). The verbatim run is captured at `docs/case-studies/issue-559/raw-data/meta-core-artifacts.txt`.

### Added
- Added route→method aliases as first-class link data (`data/meta/route-method-aliases.lino`, `src/route_method_alias.rs`) so the meta core can resolve a meta-language intent slug that is coarser or finer than the handler serving it — for example `write_program` → `write_script` — to a catalogued method (issue #559, R336).

### Changed
- The solution evidence join now resolves each need's route through `MethodRegistry::method_for_route` (direct match, then alias), recording `method_via_alias` provenance, so the program-writing need in a request like "translate apple to Russian and write a hello world program in Python" reports a resolving method instead of appearing unaddressed.

### Added
- Added white-box recursive reasoning to the meta core (`src/meta_reasoning.rs`, R337): every work unit now carries a human-readable thought in both directions — the downward thought (what span was observed, why it was decomposed or judged atomic, and which method an atomic leaf resolves to) and the upward thought (how the unit's answer is composed from its solved children). The reasoning is a parallel tree to the work-unit tree, serialized to Links Notation and emitted as the trace-only `work_unit_reasoning` / `work_unit_reasoning:steps` events, so the box is inspectable by users and developers — the reasoning, not just the predicate, is visible (issue #559).

### Changed
- The self-describing recipe (`data/meta/recursive-core-recipe.lino`) now lists nine ordered steps, adding the white-box reasoning step and pinning `WorkUnitReasoning::for_unit` and `record_work_unit_reasoning` to their source.

### Added
- Added the upward construction pass to the meta core (`src/meta_construction.rs`, R338): the construction half of the recursion. A post-order (bottom-up) walk of the work-unit tree composes each answer from leaf to root — every leaf is a base case constructed directly from the method that resolves its route (via the same `method_for_route` bridge the evidence join uses), and every parent is a recursive case composing its already-constructed children in source order. Serialized to Links Notation and emitted as the trace-only `upward_construction` / `upward_construction:steps` events, so both directions of the recursion — decompose and compose — are inspectable link data (issue #559).
- Added the `RecursionMode` knob (`Down` | `Up` | `Both`), surfaced as `SolverConfig::recursion_mode` and the `FORMAL_AI_RECURSION_MODE` env override. The default `Down` reproduces the pre-knob trace exactly, so the upward pass is always an explicit opt-in and the default solver behavior is unchanged (R13).

### Changed
- The self-describing recipe (`data/meta/recursive-core-recipe.lino`) now lists ten ordered steps, adding the upward construction step and pinning `UpwardConstruction::for_unit` and `record_upward_construction` to their source.

### Added
- Added the method-selection trace to the meta core (`src/selection.rs`, R339): for every atomic work-unit leaf, `MethodSelection::for_unit` names the method the single data-driven registry authority resolves (`MethodRegistry::method_for_route`, alias-aware — e.g. `write_program` resolves to `write_script` through its route→method alias), or marks the leaf `unresolved` when no method serves it, and counts resolved vs. unresolved leaves. Serialized to Links Notation and emitted as the trace-only `selection` event, this makes the dispatch the registry performs auditable per request.
- Added the `SelectionMode` knob (`Off` | `Record`), surfaced as `SolverConfig::selection_mode` and the `FORMAL_AI_SELECTION_MODE` env override. The default `Off` records nothing and leaves both routing and the answer unchanged, so the trace is an explicit opt-in (R13).

### Changed
- The self-describing recipe (`data/meta/recursive-core-recipe.lino`) lists the method-selection step, pinning `MethodSelection::for_unit` and `record_selection` to their source.

### Added
- Added the gated meta self-improvement loop (`src/meta_self_improvement.rs`, R340): the meta algorithm now reads *itself* — the recursive-core recipe (the algorithm encoded as Links Notation) against the live `record_meta_core` pipeline (the algorithm as code) — and emits the *updated* algorithm as link-encoded output. It detects drift between the recipe's `meta_function` citations and the `record_*` stages the pipeline actually runs, proposing the additions and stale-citation removals that reconcile them (`MetaSelfImprovement::from_repo().propose()` → `MetaRecipeProposal::to_links_notation`). It is gated and proposal-only: the default `SelfImprovementMode::Off` proposes nothing and it never writes the recipe back, so adoption stays a human review step and behaviour is unchanged (issue #559).

### Changed
- Adopted the loop's first real finding: the self-describing recipe (`data/meta/recursive-core-recipe.lino`) now cites `record_solution_evidence` (and lists the `solution_evidence` event), so it describes every stage the pipeline runs and the loop reports the live sources as self-consistent.

### Added
- Lifted the meta core's hardcoded natural-language recognition cues into reviewable link data (`data/meta/cue-lexicon.lino`, `src/cue_lexicon.rs`, R341). The arithmetic operators, web-search verbs, the fourteen text-manipulation operations, the calendar fallback verbs, and the other intent cues that used to live as inline Rust string literals in `src/intent_formalization.rs` are now `cue_set` records, each declaring how it is matched (`token` whitespace-bounded word / CJK substring, `substring` raw contains, or `prefix` starts-with). The Rust call sites (`append_prompt_relevants`, `looks_arithmetic`, `looks_like_text_manipulation`) read the cue strings from the data and keep only the structural glue (digit presence, AND/OR composition, which input each set is tested against). A grounding test pins every consulted set to the data with its expected mode and proves routing is unchanged, so adding a trigger word for an existing handler family is now a data edit rather than a Rust change. Behaviour is identical: the data reproduces exactly the lists it replaced (issue #559).

### Added
- Added a proposal-only skill-accumulation ledger as the twelfth step of the recursive meta core (`src/skill_ledger.rs`, R342). Each request's solution evidence is distilled into learning the next request can reuse — the deterministic analog of an agent that grows a skill library and a curriculum: every detected need that was satisfied by a catalogued method becomes a proposed, reusable `CandidateSkill` (a named capability the solver demonstrably has, captured with the span that demonstrated it), and every blocked need becomes a `CurriculumItem` recording the gap to close rather than a silent failure. Accumulation is proposal-only and gated: a candidate skill is born `proposed` and may only be promoted to `stable` once its `PromotionGate` is satisfied — tests *and* a benchmark delta — so the promotable count is always zero at trace time and no skill is ever auto-promoted without review (C3). The default `off` mode (env `FORMAL_AI_SKILL_MODE`) records nothing, so the ledger changes neither routing nor the answer (R13); `accumulate` emits it as the trace-only `skill_ledger` event. The recipe (`data/meta/recursive-core-recipe.lino`) describes the new stage, keeping the meta self-improvement loop self-consistent (issue #559).

### Added
- Made the recursive-core recipe *executable as data*, not just a checked description (`src/recipe_interpreter.rs`, R343). Each trace-recorded step in `data/meta/recursive-core-recipe.lino` now binds to the recorder primitive it drives via a `records` field, and `RecipeProgram` parses the recipe into an ordered program and runs it — invoking those primitives in the order the data declares, threading the intermediate artifacts (problem frame, work-unit tree, need ledger, method registry, solution evidence) exactly as the hand-written pipeline does, and mirroring its mode gates. The headline guarantee is parity: `RecipeProgram::reproduces_pipeline` proves the event log produced by executing the recipe is identical, event-for-event, to the one `meta_core::record_meta_core` produces for the same input across every recursion/selection/skill mode combination, and the recipe's recorder order equals the live pipeline's actual stage order. A misordered dependency or unknown binding surfaces as an error rather than silent divergence. This makes the algorithm-as-data and the algorithm-as-code provably the same algorithm — the foundation for eventually driving the pipeline from the recipe itself — while staying trace-only: it changes neither routing nor the answer (issue #559).

### Changed

- Issue #559 (R344): completed the **total migration to the data-driven method
  registry as the sole dispatch authority**. An interim corpus-wide parity
  certificate first proved the registry resolved the *entire route vocabulary the
  system can ever emit* — every registered method name (each a self-resolving
  route), every route→method alias (R336, including the `write_program` intent),
  and every classifier route slug — as a behavior-preserving replacement for the
  legacy hardcoded mapper, with **zero contradictions**. With that proof in hand,
  the legacy authority and the parity scaffolding were **removed outright**:
  `src/dispatch_parity.rs` and `intent_formalization::specialized_handler_name`
  are gone, leaving `MethodRegistry::method_for_route` (alias-aware) as the only
  route→method resolver and the only live dispatch path
  (`src/meta_method_dispatch.rs`). The closure invariant the certificate
  guaranteed now lives directly against the live registry — grounded in
  `MethodRegistry::from_dispatch`, `route_method_alias::aliases`, and
  `seed::intent_routing` — pinned by
  `tests/unit/specification/method_registry.rs::the_registry_is_the_sole_authority_that_closes_over_the_route_corpus`:
  no route resolves to an unregistered method, and every method-name and alias
  route resolves.

### Changed

- Issue #559: replace the solver-local specialized-handler loop with the live
  registry-backed meta method dispatcher. `MethodRegistry` now supplies the
  prelude, specialized, and contextual method ordering used by
  `meta_method_dispatch::try_dispatch`, which is now the sole dispatch authority
  (the legacy route mapper and its parity scaffolding were removed outright once
  the registry was proven a behavior-preserving replacement — see R344). Selected
  handler answers now re-project their
  evidence and Links Notation after the `method` event is recorded, so responses
  expose the selected registry method directly.

## [0.217.0] - 2026-06-21

### Added

- **Dedicated install landing pages for every interface (issue #554).** The site
  chooser now links to three new pages rendered by the shared
  `src/web/site-chrome.js`: `/vscode/` for the VS Code extension, `/cli/` for the
  command-line tool, and `/telegram/` for the Telegram bot. Each page carries
  copy-paste install commands (with one-click copy buttons), ordered manual
  steps, and direct links to the raw installer and the latest release.
- **A universal one-line installer** — [`scripts/install.sh`](scripts/install.sh)
  (POSIX `sh`) and [`scripts/install.ps1`](scripts/install.ps1) (PowerShell) —
  with a target for every interface: `desktop`, `vscode`, `cli`, `telegram`
  (installs the CLI that powers the bot), and `all`, from a single command
  (`curl -fsSL …/install.sh | sh -s -- <target>`). The VS Code page documents the
  manual-only ".vsix" flow ("VS Code Extension only" mode) while the extension is
  still off the Marketplace.
- **One-click VS Code extension install from the desktop app.** Settings now
  offers *Install VS Code extension*: the Electron shell
  (`desktop/lib/vscode-install.cjs`) detects an installed `code`/`code-insiders`/
  `codium`/`cursor`/`windsurf` CLI, downloads the published `.vsix` from the
  latest GitHub release, and runs `code --install-extension … --force` — all
  exposed through the `formalAiDesktop:installVsCodeExtension` IPC bridge.
- The desktop release CI now builds and uploads the `formal-ai-vscode-*.vsix`
  asset so the installers and the one-click flow have a release artifact to fetch.

### Changed

- The shared chooser (`src/web/site-chrome.js`) gained a sectioned-content
  renderer (`section-<id>`, `command-<testid>`, `copy-<testid>`) used by the new
  install pages; the existing landing/docs/download pages are unchanged.
- The root landing chooser now surfaces six destinations (web app, docs,
  download, VS Code, CLI, Telegram) instead of three.

## [0.216.0] - 2026-06-21

### Added
- Add shared-dialog conversion for ChatGPT shared-page HTML and Markdown transcripts, plus CLI replay export to `demo_memory`.

### Fixed
- Preserve multi-line memory event content through Links Notation export/import and answer captured shell-loop prompts with readable one-line commands.

## [0.215.0] - 2026-06-21

### Fixed
- Thinking preview now fades the **whole** collapsed reasoning stack with a single
  container-level scroll gradient instead of masking each step line separately, so
  two stacked steps read as one continuously-scrolling surface (issue
  link-assistant/formal-ai#550, problem 1).
- Naturalized thinking detail is no longer clipped at 120 characters; the cap was
  raised to 600 in both `truncate_thinking_detail` (Rust core) and the
  `thinkingDetailText` browser helper, so realistic single-step detail renders in
  full while staying bounded (issue link-assistant/formal-ai#550, problem 2).
- A pending assistant message that only shows thinking steps now keeps the full
  message-body width instead of collapsing to a fixed 116px box, removing the
  sudden width jump when the answer body starts streaming (issue
  link-assistant/formal-ai#550, problem 3).
- The desktop **services** and **update** panels (and the services error text) now
  have complete dark-theme rules, so every surface, token input, action button and
  status line is readable in dark mode instead of falling back to light styling
  (issue link-assistant/formal-ai#550, problem 4).
- Every top-bar control — source/download/report/memory buttons, the sidebar and
  mobile-menu toggles, and the mode/diagnostics toggles — now shares one hover
  treatment and one keyboard focus ring, so hover/focus feedback is consistent
  across the whole header rather than partial (issue link-assistant/formal-ai#550,
  problem 5).

### Changed
- The web stylesheet now defines a single semantic design-token palette (`--fa-*`)
  per theme instead of hand-duplicating every colour across the light base and both
  dark layers. New surfaces and controls inherit correct theming by consuming a
  token, which removes the duplication that was the shared root cause of the dark
  `services` panel (problem 4) and the partial top-bar hover (problem 5). The change
  is value-preserving — every token equals the previous colour — so no rendered
  output changed (issue link-assistant/formal-ai#550).
- The eleven top-bar controls now render through a single reusable `ToolbarButton`
  React component, so their markup, classes, accessibility attributes and overflow
  priority stay uniform by construction rather than being copied per button (issue
  link-assistant/formal-ai#550). Together with the `--fa-*` tokens this delivers the
  reusable-component and design-system substance of the requested Chakra UI
  direction; Chakra's Emotion CSS-in-JS runtime remains intentionally unadopted
  because its runtime `<style>` injection is incompatible with the app's strict
  `style-src 'self'` Content-Security-Policy (issue link-assistant/formal-ai#479).

## [0.214.0] - 2026-06-21

### Added
- Added packaged desktop auto-update checks, in-app update notifications, and user-triggered update installation.

### Fixed
- Fixed the desktop shell version badge so packaged apps show the Electron app version instead of `vdev`.

## [0.213.0] - 2026-06-20

### Fixed
- Run granted desktop and VS Code `shell` tool calls on the host machine by default, while keeping Docker isolation available for explicit sandboxed shell requests and code execution.

## [0.212.0] - 2026-06-20

### Fixed
- Dark theme now reaches every primary widget. The topbar mode-status badge, the collapsed-sidebar toggle, the mobile drawer section headings, and the per-step "tool"/"agent" mode badges in the reasoning trace had hard-coded light colors with no `[data-theme="dark"]` or `@media (prefers-color-scheme: dark)` counterparts, so their light palette bled through when the rest of the UI went dark. Each missing override is now in place using the codebase's existing dark palette, so theme switching looks consistent end-to-end (issue #541).
- Desktop conversations now survive app upgrades. The userData directory is pinned to a stable, productName-independent name (`formal-ai`) so a rebrand or package rename can no longer orphan a profile, and on first launch any legacy profile (e.g. `formal-ai Desktop`) is **non-destructively** migrated forward — the renderer's IndexedDB conversation store and `localStorage` preferences are copied into the pinned directory without ever deleting the originals, then version-stamped for future schema migrations (issue #541).
- Desktop app no longer reports "Docker unavailable" when Docker Desktop is installed and running. The `docker` binary is now resolved across well-known install locations (`/usr/local/bin`, `/opt/homebrew/bin`, `/Applications/Docker.app/...`, Windows `Program Files`, NixOS), fixing the GUI-launch PATH gap, and availability is re-probed on a short TTL so a daemon started after the app opened is detected without a restart (issue #541).
- Collapsed reasoning preview now shows the current step in full (at least one whole paragraph) instead of clipping it to a single ellipsised line, so the thinking trace is actually readable while collapsed (issue #541).
- Demo mode no longer touches user conversations. Switching demo on from inside a real conversation now spawns a dedicated, sidebar-invisible demo conversation that holds the scripted turns; switching demo off restores the user's conversation exactly as they left it; and clicking any conversation in the sidebar auto-disables demo mode so the user is never left looking at a demo thread they did not choose. The dedicated demo conversation is reused within a session, so the "last example" survives an off/on toggle without leaking into the user's threads (issue #541).
- The desktop permission panel now offers a single primary action — **Grant all and switch to Agent mode** — directly above the per-tool rows, so users who asked the assistant to run a terminal command (e.g. "run \`ls ~\` in terminal") can replay the request in one click instead of being asked to re-type it. When a command is waiting for permission, the button copy upgrades to "Grant all, switch to Agent mode, and run pending task" and clicking it grants all six desktop tools, flips the mode toggle to Agent, and replays the queued shell command through `executeTerminalCommand`. Without a pending task, the same button still grants and flips mode in one step (issue #541).

### Changed
- Reasoning steps now read as plain human language at every detail level. The "Thinking detail" setting defaults to **Standard** (the 50% midpoint), which shows only the high-level reasoning phases and folds the mechanical sub-steps out of view so newcomers are not overwhelmed. Even at maximum **Detailed** granularity the trace no longer leaks the internal symbolic Links tuple (`(@USER OP:… ?term)`) or jargon like "symbolic form" — the formalization step is projected to a plain task noun (e.g. "greeting", "calculation", "search") in all four UI languages. The raw symbolic formalization remains available in Diagnostics mode for maintainers (issue #541).

### Added
- Minimum message animation time setting (Settings → "Minimum thinking animation"). Reasoning steps now reveal one-by-one and the answer body fades in only after the trace has played out, so the deterministic engine's instant answers still feel considered. Defaults to 2 seconds; set it to 0 for immediate display. Honours `prefers-reduced-motion` (issue #541).
- `FORMAL_AI_DESKTOP_DEBUG` environment variable enables verbose desktop diagnostics (Docker binary resolution and probe results) to help diagnose environment-specific problems.
- `FORMAL_AI_DOCKER_BIN` environment variable overrides the resolved `docker` binary path.

## [0.211.0] - 2026-06-20

### Added
- Case study for issue #511 under `docs/case-studies/issue-511/`: deep analysis of
  the `unknown` answer for terminal-command prompts (e.g. ``Выполни `ls ~` в
  терминале``), a full requirement inventory (R1–R20), per-requirement solution
  plans that reuse the existing permission-gated tool router, Docker sandbox,
  OpenAI-compatible local server, and `src/agentic_coding/` loop, and a sequenced
  implementation epic (E1–E8) for agent/full-auto mode driven by
  `link-assistant/agent` through `link-assistant/agent-commander`.
- The epic milestones are filed as live GitHub issues (via `gh`) and linked as
  sub-issues of #511: E1–E8 (#513–#520). The case-study docs reference the created
  issue numbers so the plan stays in sync with the tracker.
- Re-verified the integration against the latest upstream versions
  (`@link-assistant/agent` v0.24.0, `agent-commander` js_0.8.0 / rust_0.2.6): the
  Agent-CLI permission gap (`link-assistant/agent#271`) is **resolved** by
  `link-assistant/agent#272` (v0.24.0), which adds a native, enforceable
  `--permission-mode auto|plan|readonly|ask`, an OpenCode-compatible `--permission`
  JSON policy, and a per-command JSON approval protocol (JS + Rust). Both residual
  `agent-commander` gaps filed last round are now **closed**:
  `link-assistant/agent-commander#39` (map `--read-only`/`--plan-only` for the
  `agent` tool onto its native `--permission-mode`, shipped js_0.7.0 / rust_0.2.5)
  and `link-assistant/agent-commander#40` (uniform per-command approve-each relay,
  `--approve-each` / `--permission-mode ask`, shipped js_0.8.0 / rust_0.2.6). No open
  `agent-commander` issues remain, so the #511 plan is fully implementable today.
- Documented the **default-backend decision**: the desktop agent path defaults to
  `@link-assistant/agent` through `agent-commander`, because per the agent-commander
  approve-each parity (`docs/common-concepts.md`) only `agent` (scope `session`) and
  `claude` (scope `tool-input`) can relay per-command JSON approvals — and `agent` is
  the only org-owned backend with a clean session-wide `once|always|reject` grant;
  `codex`/`gemini`/`qwen`/`opencode` lack a relayable headless approval handshake
  (documented upstream-CLI limitation, not a bug). The case-study docs and raw-data
  snapshots (incl. a new `external-agent-commander-common-concepts.md`) were refreshed
  to these latest versions.
- Reconciled the issue #511 plan with the merged E1 work from PR #525 and the latest
  `main` service-control changes: E1 is now recorded as implemented, the remaining
  sequence starts at E2, and E3/E5 explicitly reuse the prepared GHCR image,
  `compose.yaml`, and desktop one-click service-control stack instead of duplicating
  server/container lifecycle code.

### Added
- Terminal-command intent (`tryTerminalCommand`) in both the Rust solver
  (`src/solver_terminal.rs`) and the JS worker (`src/web/formal_ai_worker.js`).
  Prompts that ask to run a shell command — fenced/backtick commands, "run … in
  terminal" / «выполни … в терминале» phrasings, or an explicit leading shell
  token like `ls`/`git status` — now resolve to an `agent_suggestion` response
  that names the detected command, explains Agent mode, and offers to switch and
  grant the `shell` capability, instead of falling through to `unknown`
  (visible fix for #511, issue #513). Localized for en/ru/hi/zh.
- Three-way `Chat` / `Agent` / `Full Auto` mode radio group in the web toolbar
  and drawer, replacing the binary agent toggle. A new `mode` preference is
  persisted and the legacy `agentMode` boolean is derived from it
  (`mode !== "chat"`) for back-compat. The topbar status label now reflects the
  active mode.

### Changed
- Toolbar/drawer mode controls expose `data-testid="mode-radio"` /
  `mode-option-<mode>` and a `mode-status` label; existing e2e specs were
  updated from the old `agent-toggle` selector accordingly.
- The terminal-command response prose is no longer hardcoded in either engine.
  The four-language bodies now live in `data/seed/multilingual-responses.lino`
  under the `agent_suggestion` (Chat mode) and `agent_suggestion_active` (Agent
  mode on) intents, with a `{command}` placeholder. Both `src/solver_terminal.rs`
  (via `seed::response_for`) and the JS worker (via `answerFor`) look the
  template up and fill in the detected command, so the natural-language wording
  is sourced from seed data rather than living in code (addresses #513 review
  feedback).
- The terminal-command *trigger* vocabulary is no longer hardcoded either. The
  terminal/shell phrases, run verbs, Chinese run verbs, and leading shell tokens
  now live in the new `data/seed/terminal-commands.lino`. The Rust solver parses
  it via `src/seed/terminal_commands.rs`
  (`seed::terminal_command_vocabulary`), and the JS worker embeds a
  byte-identical inline mirror kept in lockstep by
  `experiments/issue-513-sync-worker-terminal.mjs` (the same convention as the
  operation vocabulary, #386). A `--check` mode guards the parity in CI.
- Every new terminal-command vocabulary token (shell tokens, `command-line`,
  the `agent_suggestion*` intents and their `response_*` templates) is grounded
  as a first-class meaning so the total reference-closure audit
  (`scripts/audit-total-closure.py`) stays at zero. The
  `data/seed/closure-generated-*.lino` shards were regenerated via
  `scripts/close-total.py`; the generation is idempotent.
- E2E Playwright configs now set reasonable per-test, whole-suite
  (`globalTimeout`), assertion (`expect.timeout`), and navigation/action caps in
  both `tests/e2e/playwright.local.config.js` and `playwright.pages.config.js`
  so a hung worker, server, or deployment aborts promptly instead of wedging CI
  (addresses #513 review feedback on iterating faster).

### Added
- Added per-tool desktop permission grants, first-run Agent/Full Auto onboarding,
  and Agent-mode per-command shell approval prompts so desktop tools can be
  granted or declined independently while preserving default-deny behavior.

### Changed
- Routed the desktop permission, command-approval, and one-click services UI
  strings in `src/web/app.js` through the i18n catalog (`t(key, params)`) so
  they translate with the active UI language (en/ru/zh/hi) instead of rendering
  hardcoded English introduced in PR #528.

### Added
- Added a strict CI guard, `tests/e2e/scripts/check-web-hardcoded-ui-strings.mjs`
  (`check:web-hardcoded-ui`), that fails the build when a user-facing prose
  string literal is passed as a child of an `h(...)` render call in
  `src/web/app.js`, plus catalog keys and `check:i18n` coverage for the new
  permission/command/services strings. Documented the rule in CONTRIBUTING.md and
  `docs/design/no-hardcoded-natural-language.md` so the regression cannot recur.

### Changed
- Split the web UI translation catalog so each source file stays under the
  Links Notation line limit enforced by `scripts/check-file-size.rs`. The
  desktop tool-permission and Services strings now live in
  `src/web/i18n-catalog-permissions.lino`, while the core UI strings remain in
  `src/web/i18n-catalog.lino`. The loader (`src/web/i18n.js`) fetches both files
  and merges their per-locale keys, and `check:i18n` plus the language-parity
  guards validate the merged catalog.

### Added
- Auto-start the desktop local OpenAI-compatible server when Agent or Full Auto mode is entered, reusing a healthy running server and exposing its `apiBase` for provider configuration.

### Added
- Added the desktop AgentProvider seam with an in-process default provider, an
  opt-in agent-commander provider for the `agent` backend, and tests guarding
  read-only execution plus direct host `agent`/`claude`/`codex` spawns.

### Added
- Added the installable Formal-AI Agent environment container flow: the prepared image now bundles `@link-assistant/agent` and `agent-commander`, the desktop Services panel can install and health-check `formal-ai-agent`, and Compose exposes the matching `agent` profile.

### Added
- Added an Agent CLI NDJSON adapter that maps assistant text, tool start/result,
  and error events onto the existing chat answer and diagnostics rendering path.

### Added
- Added cold-start desktop e2e coverage for the issue #511 `ls ~` journey,
  including first-run onboarding, three-way mode switching, per-command denial
  and approval, the hermetic in-process provider path in CI, and a
  `FORMAL_AI_E2E_AGENT_COMMANDER=1` gated real commander-provider variant.

### Fixed
- Routed read-only commander-provider requests for the default `agent` backend
  through `agent-commander --read-only`, using the shipped upstream mapping
  instead of the old `--approve-each` workaround.

### Documentation
- Finalized the issue #511 Agent CLI + agent-commander best-practices write-up
  and upstream closeout status for issue #520.

### Documentation
- Updated the issue #511 case study and PR #512 description to reflect that all
  eight implementation milestones (E1–E8, #513–#520) are merged into the parent
  branch: rewrote the "why a plan, not the whole feature" and "acceptance
  criteria" sections to state the feature ships in this PR, with every acceptance
  criterion met and pinned by tests.

## [0.210.0] - 2026-06-17

### Added

- Publish the Telegram Docker-in-Docker image to GHCR on release and document the one-line `docker run` / `docker compose up` startup path.
- Add root `compose.yaml` for the prebuilt Telegram bot image with `TELEGRAM_BOT_TOKEN` as the only required setting.
- One-click start/stop of both prepared services — the Telegram bot and the OpenAI-compatible API server — from the desktop app, with live Docker status polling (`desktop/lib/service-control.cjs` over IPC).
- Opt-in `server` profile in `compose.yaml` so a server reproduces the identical containers with one line (`docker compose --profile all up -d`); each Docker-in-Docker service gets its own inner-Docker volume so the bot and server can run together.
- New `docs/desktop/service-control.md` documenting both the one-click desktop and one-line server paths in detail.

## [0.209.0] - 2026-06-17

### Fixed

- CI: the **Deploy Demo to GitHub Pages** job no longer crashes with
  `No space left on device`. The job stopped restoring the multi-gigabyte
  `target/` cache shared with the `lint`/`test` jobs (it now caches only the
  Cargo registry under a dedicated `*-cargo-docs-*` key) and proactively frees
  unused pre-installed SDKs from the runner before building the API docs. Disk
  usage is now logged with `df -h` around the cleanup for future diagnosis.
  (#523)

## [0.208.0] - 2026-06-17

### Added
- Added a default assistant thinking preview that shows a collapsed current step, a faded previous step, an expandable localized summary list, and a configurable thinking-detail setting while preserving raw reasoning diagnostics behind the diagnostics toggle.
- Added first-class solver thinking metadata derived from the append-only event log and exposed it through Links Notation, Chat Completions, Responses, and the desktop HTTP chat path.
- Made thinking steps concrete by default: a shared naturalizer turns each reasoning event into a human-readable sentence that names the real content (the prompt, the detected language, the chosen route, the computed `expr = result`, the looked-up entity, the composed answer) instead of a generic label, and surfaces the same concrete reasoning on the CLI `--thinking` output, the OpenAI-compatible and Anthropic APIs, the browser, and the Telegram bot via a native collapsed-by-default expandable blockquote.

### Changed
- Promoted thinking to a first-class concern in a dedicated `thinking` module (step model plus naturalizer) so it is shared across every surface rather than embedded in the engine internals.

## [0.206.0] - 2026-06-16

Fix macOS desktop release signing by re-sealing ad-hoc `.app` bundles with
`codesign` before DMG upload, and document the `v0.205.0` CI failure that left
Linux/Windows assets present but macOS assets absent.

## [0.204.0] - 2026-06-15

### Fixed

- **Desktop apps are actually built and available on `/download` (issue #479).**
  The automated release tags a *child* `chore: release vX.Y.Z` commit whose
  CI run carries the *parent* SHA, so the desktop-release resolve step (which
  required a tag pointing at `workflow_run.head_sha`) never matched and zero
  desktop assets were uploaded — every release since the path went live showed
  "Not available in latest release". The resolve script now targets the latest
  published release with a defensive exact-SHA tier and an idempotency guard,
  and emits grouped verbose diagnostics (`[desktop-release-resolve]` logs) so
  the resolution decision is auditable for future triage; the
  `desktop-release` workflow no longer gates on full-pipeline
  `conclusion == 'success'` (the release is published early, so a later job
  failure used to suppress the whole desktop build); and
  `scripts/wait-for-pages-deployment.sh` is now marker-authoritative
  (`deployment.json`'s SHA proves the matching stamped build is live, since
  GitHub Pages deploys atomically) so the E2E Pages probe stops timing out and
  failing the pipeline. Landing/docs assets are cache-busted with
  `?v=__FORMAL_AI_ASSET_VERSION__` like `/app/`.

- **macOS install screenshots are real captures, not synthetic renders
  (issue #479).** The `/download` macOS Gatekeeper figures are now genuine
  macOS 15 (Sequoia) captures from the sibling app `konard/vk-bot-desktop`,
  which ships the identical `electron-builder` ad-hoc signing flow, replacing
  the previously generated images the maintainer rejected as fake. The
  synthetic generator and HTML fixture are removed; provenance is documented
  in `src/web/download/assets/screenshots/README.md`.

### Added

- **Source code is a big hero button on the landing page (issue #479).** The
  landing surfaces the source repository as a prominent `.source-cta` call to
  action (translated for every supported locale) instead of a small footer
  link.

## [0.203.0] - 2026-06-15

### Changed
- Raised the Rust toolchain MSRV to 1.96 (latest stable) and updated the Docker builder image to `rust:1.96-slim`, matching the `web-search` and `web-capture` crate MSRVs.
- Updated all Rust workspace dependencies to their latest versions (`clap` 4.6, `doublets` 0.4.0, `link-calculator` 0.19.0, `meta-language` 0.45, plus transitive updates).
- Updated web bundle dependencies to the latest versions (`react`/`react-dom` 19.2.7, `marked` 18.0.5, `dompurify` 3.4.10) and rebuilt `src/web/vendor.bundle.js`; pinned Bun to 1.3.14.
- Updated desktop (`electron` 42, `electron-builder` 26) and VS Code (`@vscode/test-web` 0.0.80, `@vscode/vsce` 3.9.2) dependencies to the latest versions.
- Refreshed the issue #410 case study to reflect that the upstream `web-search`/`web-capture` readiness blockers are resolved (`web-search` published at npm 0.10.3 / crates.io 0.3.1 with full provider parity; `web-capture` at npm 1.10.9 / crates.io 0.3.31).

### Fixed
- Resolved Clippy lints newly reported by the latest stable toolchain (1.96) so `cargo clippy --all-targets --all-features` stays clean under `-Dwarnings`: added `const fn` where derivable, switched to `Option::is_none_or`, `std::iter::repeat_n`, `f64::midpoint`, and `u*::is_multiple_of`.
- Preserved code-block enhancements (highlighting and copy buttons) under React 19 by memoizing rendered markdown by message content; React 19 compares `dangerouslySetInnerHTML` by object identity, which otherwise re-assigned `innerHTML` and wiped the out-of-band DOM enhancements on unrelated re-renders.

## [0.202.0] - 2026-06-15

### Added

- A **deterministic agentic planner** (`src/agentic_coding/planner.rs`) — the
  server's "brain" for issue #468's *"solve such tasks in agentic mode"*
  framing. It is a pure function of the conversation so far and the tool names an
  agentic CLI advertised, driving a small state-machine recipe
  (`web_search → web_fetch → write_file → run_command → final`) that formalizes
  «Сказка о рыбаке и рыбке» into a Links Notation knowledge base. Steps whose
  tool the CLI did not advertise are skipped, and tool *errors* are observed (an
  errored fetch is ignored and the formalizer falls back to the canonical
  synopsis), so the loop always completes with a stable, all-nine-primitive
  document. No sampling, no hidden state — the same history always yields the
  same plan, keeping neural inference a NON-GOAL.

### Changed

- The OpenAI-compatible chat endpoint (`create_chat_completion_with_solver`) now
  **emits `tool_calls`** with `finish_reason: "tool_calls"` when agent mode is on
  and a formalization task is in flight, closing the core gap that the server
  could never *request* a tool — it previously hard-coded `finish_reason: "stop"`
  on every turn. A `tool`-role result feeds back into the planner on the next
  request until the recipe is exhausted, at which point the server answers with
  the knowledge base inline (`finish_reason: "stop"`). Unrecognised requests
  still fall through to the ordinary symbolic solver, so non-agentic behaviour is
  byte-for-byte unchanged. `ChatMessage` gained OpenAI `tool_calls` /
  `tool_call_id` / `name` fields and `ToolCall` / `FunctionCall` types so tool
  requests and results round-trip through the wire format (issue #468).

### Added

- An **in-repo agentic driver** (`src/agentic_coding/driver.rs`) that plays the
  role of an external agentic CLI against our own OpenAI-compatible server,
  closing issue #468's *"our Formal AI system should have enough skills … to
  actually call all the tools from any agentic CLI, understand errors from
  tools, … do web fetch and web search, to actually complete the task"*. It
  advertises the four-tool set (`web_search`, `web_fetch`, `write_file`,
  `run_command`), and on every `tool_calls` turn the server emits it **executes**
  each call — search/fetch against an offline corpus, file writes and commands in
  a single reused, sandboxed [`AgentWorkspace`] — feeds each result back as a
  `tool` message, and loops until the server returns the finished knowledge base.
  The loop is bounded by a hard turn cap, so unbounded reasoning stays a NON-GOAL
  and no network or neural inference is ever involved. Exposed as
  `run_agentic_task` / `run_agentic_task_in` returning a `DriverOutcome` with the
  full tool-call transcript.
- An **offline, deterministic web corpus** (`src/agentic_coding/corpus.rs`) that
  resolves `web_search` / `web_fetch` tool calls against a fixed page set: a
  search that surfaces the canonical Викитека page for «Сказка о рыбаке и рыбке»
  and a fetch that returns the canonical synopsis (the formalizer's fallback
  text), plus a 404 path for unknown URLs so the driver exercises the
  *"understand errors from tools"* requirement with no live network.
- A new **`agent` CLI subcommand** (`formal-ai agent [--task …] [--transcript]`)
  that drives the whole offline loop and prints the resulting Links Notation
  knowledge base, with `--transcript` showing every executed tool call.
- An `issue_468_agentic_loop` example that runs the driver end to end and prints
  the transcript plus the final knowledge base.

### Changed

- `AgentWorkspace` gained a `last_command_result()` accessor so a long-lived
  workspace reused across a tool-call loop can observe each command's output
  between steps, before `finish` consumes it.
- The default associative packages now include a permission-only
  `pkg_agentic_coding` package granting the client-executed `web_fetch`,
  `write_file`, and `run_command` capabilities, so the full agentic loop passes
  the server's tool-permission gate. `agent_mode` remains the real guard (every
  tool is still refused unless it is explicitly enabled), and capabilities the
  package does not name (e.g. `local_shell`) stay denied.

### Added

- A **grounded meta-algorithm recipe for the agentic-coding loop**
  (`data/meta/agentic-coding-recipe.lino`), following the issue #444 pattern. It
  names every part the deterministic loop is made of — the plan constants
  (`SEARCH_QUERY`, `CANONICAL_SOURCE_URL`, `KB_PATH`), the four advertised tools
  and their capabilities and permissions, the `search → fetch → write → run →
  final` state-machine stages, the fourteen handler functions, the nine protocol
  primitives the product realises, the `MAX_TURNS` cap, and the CLI/example/test
  exposure surfaces — plus the eight ordered steps that generalise to a new task.
- A **grounding test** (`tests/unit/specification/agentic_meta_algorithm.rs`)
  that loads the recipe and asserts the live source still matches every entry, so
  the recipe can never silently drift from the code (CI fails if it does).

### Changed

- `docs/meta-algorithm.md` now documents the agentic-coding meta-algorithm as a
  second grounded recipe alongside the procedural how-to one, including the state
  machine, the eight ordered steps, and the grounded-record table.

### Added

- The deterministic **agentic-coding loop now drives the Anthropic Messages
  (`/v1/messages`) and OpenAI Responses (`/v1/responses`) surfaces**, not just
  Chat Completions. The maintainer's framing for issue #468 was that the system
  must "call all the tools from any agentic CLI"; `claude` speaks Anthropic
  Messages and `codex` speaks OpenAI Responses, so both now emit native tool
  requests (`tool_use` content blocks / `function_call` output items) and
  *understand* fed-back tool results delivered in each protocol's own idiom (an
  Anthropic `tool_result` block carried on a `user` message, an OpenAI
  `function_call_output` item) so the loop advances rather than restarting.
- `AnthropicMessagesRequest` and `ResponsesRequest` now accept `tools` and
  `tool_choice`, translated into the shared OpenAI tool shape so a single
  deterministic planner backs all three surfaces.
- New public types for the Responses tool mirror: `ResponseFunctionToolCall` and
  the `ResponseOutputItem` enum (`Message` | `FunctionCall`), with
  `ResponseObject::output_messages()` and `ResponseObject::function_calls()`
  accessors.
- A focused test module (`tests/unit/agentic_surfaces.rs`, 9 tests) pins the
  mirror: tool emission in agent mode, tool-result feed-back advancing the loop,
  the final knowledge-base answer once the recipe is exhausted, refusal without
  agent mode, SSE `input_json_delta` streaming for `tool_use`, and symbolic
  fall-through for non-agentic tasks.

### Changed

- The chat, Anthropic, and Responses surfaces now share one `agentic_outcome`
  decision (refuse / plan / fall-through) so the agent-mode gate, per-tool
  permission gate, and planner behave identically everywhere; the symbolic
  fall-through still preserves `evidence_links`.
- `AnthropicMessage.content` is now a list of typed content blocks
  (`AnthropicContentBlock::Text` | `ToolUse`) instead of a single text block, and
  `ResponseObject.output` is now a list of `ResponseOutputItem`s. Wire-format JSON
  is unchanged for the text-only case.

### Changed
- Rewrote the issue #468 case study (`docs/case-studies/issue-468/README.md`) and
  `REQUIREMENTS.md` rows **R306–R319** to describe the shipped `src/agentic_coding/`
  agentic-coding loop — the deterministic planner (server brain), the in-repo driver
  and offline corpus (client), the two permission gates, and the
  nine-primitives-as-links formalizer — replacing the earlier text that described a
  removed typed-struct draft.

### Added
- `docs/desktop/server-api.md` §4e documents the multi-surface agentic tool-calling
  loop: how each agentic CLI (`codex` via Responses, `opencode` + `agent` via Chat
  Completions, `claude` via Anthropic Messages) drives `formal-ai serve`, how the
  server emits the next tool call and consumes the fed-back result, and the
  `agent_mode` + `pkg_agentic_coding` gating — with external CLIs as front-ends,
  never embedded in the engine.
- A traceability test (`issue_468_agentic_coding_case_study_is_traceable` in
  `tests/unit/docs_requirements_issue_468.rs`) pins `REQUIREMENTS.md` rows
  R306–R319, the case study `README.md`, `formal-protocol-mapping.md`, the
  `server-api.md` §4e agentic-loop section, and the two worked examples
  (`examples/issue_468_agentic_loop.rs`, `examples/issue_468_formalize_text.rs`)
  to the live implementation.

### Fixed
- Desktop app downloads are available again on `/download` for every platform. The desktop-release workflow resolved the parent commit SHA, but the auto-release tag is created on the child "chore: release" commit, so the exact-SHA match never succeeded and v0.187.0–v0.201.0 shipped zero desktop assets. Resolution is now two-tier (exact SHA, then the latest published release / auto-release child commit), with verbose logging for future diagnosis (issue #479).
- Refreshed the obsolete `/download` desktop app-preview and page screenshots.
- Replaced a manual `(len + 1) / 2` ceiling in the lexicon matcher with `div_ceil`, satisfying clippy's `manual_div_ceil` lint under newer stable toolchains.

### Added
- macOS Gatekeeper install screenshots on the `/download` page, mirroring the vk-bot-desktop walkthrough.
- A landing-page chooser at `/` that links to the web app (`/app/`), the documentation hub (`/docs/`), and the desktop download (`/download/`), wired to the shared theme + UI-language preference store and localized into en/ru/zh/hi.
- A documentation hub at `/docs/` and a generated Rust API reference at `/docs/api/`, built with `cargo doc` during the GitHub Pages deploy.
- End-to-end coverage for the new landing and documentation pages, plus CI guards for the new static and deploy invariants.

### Changed
- The interactive web app moved from `/` to `/app/`, served with `<base href="../">` so its shared site-root assets still resolve under both the GitHub Pages path prefix and the desktop static server; the desktop wrapper and in-site back-links now target `/app/`.
- The release pipeline's concurrency is now main-safe, so a release run on `main` is never cancelled mid-flight.

## [0.201.0] - 2026-06-14

### Added
- Recognize document-generation requests ("Сделай мне пдф файл …", "make me a
  PDF/document/report with …") and answer with the universal algorithm's formal
  plan — scope, gather, classify, assemble, export — localized to the prompt
  language, instead of falling through to the unknown response.
- Updated `meta-language` to 0.45.0 (raising the crate `rust-version` to 1.77)
  and exposed its document-format concept layer through the natural-language
  document workflow: TXT, Markdown, HTML, PDF, and DOCX conversion now routes
  through `LinkNetwork::reconstruct_text_as`, reports target fidelity fallbacks,
  and records DOCX package-layer evidence when the upstream OPC profile is
  available.

### Fixed

- List-files program answers now render language-aware sample output, so Python examples no longer show Rust fixture files like `Cargo.toml` or `main.rs` (issue #440). Browser responses also separate the "not run" status from the copy instruction and use a light code-block palette when the app is in light mode.

## [0.200.0] - 2026-06-14

### Fixed

- Issue #444: a bare elaboration follow-up after a "how to …" answer (e.g.
  "Can you give me specific instructions?") no longer dead-ends at the
  unknown-intent opener. It now rebinds to the procedure recovered from the
  prior turn and answers as `procedural_how_to` in the original language.

### Added

- New `procedural_elaboration` seed meaning (en/ru/hi/zh) and
  `try_procedural_how_to_followup` handler, mirrored in the browser worker
  (`tryProceduralHowToFollowup`), keeping Rust ↔ JS parity.

### Added

- **External trusted services are available and opt-out-able (issue #444).**
  The procedural how-to handler may now consult wikiHow, the Stack Exchange
  network, the MediaWiki family (Wikibooks, Wikiversity, Wikivoyage), and
  GitHub READMEs/docs in addition to Wikipedia and Wikidata. Every external
  source is declared in `data/seed/sources-registry.lino` under an
  `external_trusted` group with its own `settings_key` and `default_enabled true`
  (opt-out model), and the web settings UI exposes a section to toggle each one.
- **Procedural how-to / instruction-following benchmark slice (issue #444).**
  `data/benchmarks/procedural-howto-suite.lino` adds self-authored representative
  cases in the style of six widely-used instruction-following benchmarks
  (IFEval, Super-NaturalInstructions, Self-Instruct, OASST1, BIG-bench, MMLU),
  each with a paraphrased held-out variant for anti-memorization, ratcheted by
  `tests/unit/specification/procedural_howto_benchmarks.rs`. Topics span apology
  letters, meal planning, gardening, bicycle repair, pour-over coffee, and
  nutrition labels so the routing is exercised across diverse domains.
- **Central benchmark catalog (issue #444).** `docs/benchmarks.md` indexes every
  benchmark suite the repository has ever touched (issues #103, #304/#317, #362,
  #408, #444) with their fixtures, ratchet tests, sources, and licenses; guarded
  by `tests/unit/docs_requirements.rs`.
- **Grounded meta-algorithm that reproduces topic handlers on demand (issue #444).**
  `data/meta/procedural-howto-recipe.lino` is a machine-readable recipe naming
  every seed role, handler function, evidence stage, Rust↔JS parity target,
  external-service toggle, and benchmark that make up the procedural how-to
  topic, plus eight ordered steps that generalise to any topic.
  `tests/unit/specification/meta_algorithm.rs` keeps the recipe grounded by
  asserting the live source still matches every entry, and
  `docs/meta-algorithm.md` explains how to run and generalise it — so we learn
  from our own source code how to produce changes on the topic rather than only
  emitting one-off code changes.

### Fixed

- **Procedural elaboration follow-ups rebind to the prior how-to (issue #444).**
  After a "how to X" turn, a bare elaboration follow-up such as "Can you give me
  specific instructions?" now rebinds to the established procedure and restates
  the task in both the Rust solver (`src/solver_handler_how.rs`) and the browser
  worker mirror (`src/web/formal_ai_worker.js`), instead of falling through to
  the unknown opener.

## [0.199.0] - 2026-06-13

### Added
- Symbolic AI reference and best-practice audit (issue #451). `README.md`,
  `VISION.md`, and `ARCHITECTURE.md` now cite the Wikipedia
  [*Symbolic artificial intelligence*](https://en.wikipedia.org/wiki/Symbolic_artificial_intelligence)
  article (plus *Semantic network*, *Physical symbol system*, and *Neuro-symbolic
  AI*), making the project's GOFAI lineage explicit: the associative network of
  links is a semantic network in the classical sense.
- A case study under `docs/case-studies/issue-451/` with deep analysis, collected
  issue/PR data, and cited online research (`raw-data/online-research.md`,
  including 2024–2026 neuro-symbolic surveys).
- `docs/case-studies/issue-451/symbolic-ai-best-practices.md` — a 20-row audit
  mapping every technique family the article names to the associative-stack
  component that realizes it (`solver.rs`, `proof_engine/`, `probability.rs`,
  `substitution.rs`, `rule_synthesis.rs`, `knowledge.rs`, `event_log.rs`), with an
  honest applied/partial/proposed status and named reuse targets for each gap.
- Requirements **R298–R305** in `REQUIREMENTS.md` and the regression test
  `tests/unit/docs_requirements.rs::issue_451_symbolic_ai_reference_documents_are_present_and_traceable`,
  which pins the reference, the audit, and the requirement list so they cannot
  silently regress.

The documentation half of this work is reference and tests only; the
accompanying engine change (the DPLL satisfiability backend that closes the
audit's single proposed gap, R305) is described in its own changelog entry.

### Added

- A deterministic, dependency-free **DPLL satisfiability backend** at
  `src/proof_engine/decision/sat.rs` (issue #451, R305), closing the
  best-practice audit's single proposed gap (§3.2 SAT / constraint solving). The
  solver works over CNF (`CnfFormula`, `Literal`, `SatOutcome`) with unit
  propagation, pure-literal elimination, and chronological backtracking, and is
  byte-reproducible and WebAssembly-safe (lowest-index variable, `false`-before-
  `true` branch order). The Rust crates `splr` / `varisat` were evaluated as
  reuse targets but set aside to keep the engine free of native dependencies;
  they remain the documented upgrade path for CDCL/CSP-scale workloads.

### Changed

- The propositional decision procedure (`src/proof_engine/decision/boolean.rs`)
  now generalizes past the eight-variable truth-table limit: claims with more
  variables are Tseitin-encoded to CNF and handed to the new DPLL backend behind
  the existing "formalize → delegate → trace" seam. An unsatisfiable negation
  yields a tautology proof; a satisfiable one yields a concrete countermodel
  disproof. Claims of eight or fewer variables keep the exhaustive truth-table
  witness unchanged, so every prior proof and test is byte-for-byte unaffected.
- Doubled the propositional-decision test surface: new
  `tests/source/source_tests/proof_engine/decision/{sat,boolean}/tests.rs` unit
  suites plus `tests/unit/proof_request.rs` integration cases exercise the SAT
  path (wide tautology proven via DPLL, wide non-tautology disproven with a
  countermodel, the truth-table boundary, Tseitin-encoding fidelity, and the
  over-width decline), keeping coverage close to 100%.

## [0.198.0] - 2026-06-13

### Fixed
- **Bare "invert the sort" follow-up no longer answers `unknown` (issue #427).**
  After a numeric-list sorting conversation, the bare follow-up
  "Сделай инверсию сортировки." (make the inversion of the sort) fell through to
  `unknown`: the operation vocabulary did not recognize the *invert* phrasing as
  a descending sort, and even when an operation was named the handler had no
  numbers to act on because the follow-up lists none of its own. The
  numeric-list handler now inherits the list from the most recent operation turn
  that carried a concrete list — while the language and code request keep coming
  from the most recent turn that named a language (issue #412) — so a
  number-less invert-sort continues the established coding context and emits the
  descending code plus result.

### Added
- `reverse_sort` operation vocabulary now matches *invert*-style phrasings across
  every supported language: English `invert the sort` / `invert the sorting` /
  `invert sort` and `combo invert+sort`; Russian `combo инверс+сортиров` /
  `combo инверт+сортиров`; Hindi `combo उलट+क्रम`; Chinese `combo 反转+排序` /
  `combo 颠倒+排序`.
- Implemented identically in the Rust solver
  (`src/solver_handlers/numeric_list/mod.rs`) and the browser worker mirror
  (`src/web/formal_ai_worker.js`) so both runtimes inherit the prior list and
  reverse it; covered by `tests/integration/issue_427_invert_sort.rs`, the
  `operation_vocabulary_reverse_sort_matches_invert_phrasings` source test, and
  the `experiments/issue-427-worker-invert-sort-parity.mjs` cross-runtime check.

## [0.197.0] - 2026-06-13

### Added
- Symbolic evidence count `C` tracked separately from accumulated utility `U` in
  `src/probability.rs` via `ProbabilityStore::target_evidence_count`, porting the
  interpretable transition model from Kolonin's arXiv:2605.00940 onto the
  associative stack (issue #449).
- Counted-utility decision policy and under-evidenced gating in
  `ProbabilityRankingConfig`: `counted_utility` (rank by `U·C`),
  `min_transition_utility`, and `min_transition_count`. Defaults preserve the
  prior additive behavior.
- `RankedProbabilityCandidate::evidence_count`, surfacing the evidence count next
  to the evidence weight so each ranked option stays locally interpretable.
- Case study `docs/case-studies/issue-449/` with compiled raw data, online
  research, deep analysis, requirement enumeration, and per-requirement plans.

### Changed
- Documented the evidence-count / counted-utility / transition-threshold
  mechanisms in `ARCHITECTURE.md` section 6.1.

### Added

- Ported the remaining interpretable, non-neural mechanisms from Kolonin's
  "Interpretable Experiential Learning" (arXiv:2605.00940) onto the symbolic
  probability layer (issue #449):
  - `symbolic_cosine_similarity` plus `ProbabilityStore::nearest_similar_evidence`
    implement the paper's `SS` inexact-state fallback — a candidate with no exact
    evidence borrows the nearest stored target's utility, scaled by a
    deterministic bag-of-words cosine, gated by a similarity floor.
  - `ProbabilityStore::reinforce_transition_path` implements the paper's
    episode-wide global feedback — one append-only `markov_transition`
    observation per adjacent state pair, replayable through the event log and
    link-store projection.
  - `ProbabilityDecisionPolicy` groups the `CU`/`TU`/`TC`/`SS` knobs into one
    `Copy` policy, threaded through `SolverConfig::probability_policy` into every
    selection use case via `ProbabilityRankingConfig::with_decision_policy`.
  - `RankedProbabilityCandidate` now exposes a `similarity` field so a
    fallback-driven decision stays locally interpretable.
- Added the `examples/issue_449_interpretable_learning.rs` worked tour of all
  four mechanisms (Bayesian utility, counted utility, thresholds, similarity
  fallback, and episode reinforcement).

### Changed

- Doubled the probabilistic-reasoning specification suite to lock the new
  behaviour and keep coverage close to 100%; existing callers are byte-for-byte
  unaffected because the default policy reproduces the paper's recommended
  baseline (`CU=False`, `TU=0`, `TC=1`, no similarity fallback).

## [0.196.0] - 2026-06-13

### Added
- Added `docs/USER-JOURNEYS.md` (issue #454): a dedicated document that states the pain Formal AI closes, the personas it is for, and the concrete user journeys it supports today (transparent "why did you answer that?", multilingual chat, code generation with honest execution notes, math/units/currency, sourced fact and definition lookup, single-file memory export and cross-surface migration, OpenAI-compatible endpoint, Telegram, edit-the-data reconfiguration, bounded agent tasks, and follow-up edits) plus the journeys it could support next (visual graph, compiled skills, cloud memory sync, search-based solving, WebVM, shared associative packages). Each journey is mapped to the implemented surfaces and the relevant `VISION.md` principles, with a journey-to-surface coverage matrix and a worked end-to-end example.

### Changed
- `VISION.md` now opens with a "Who This Is For And What Pain It Closes" section and a concrete example user journey, linking to `docs/USER-JOURNEYS.md` so the vision makes a concrete promise rather than only describing the machine. `README.md` links the new document from its project-direction paragraph.

## [0.195.0] - 2026-06-13

### Added
- Natural-language calendar event creation that exports a **real, importable
  calendar event** in every supported environment (issue #404). The prompt
  "Забей мне 18 число в 17:00 по грузии на встречу с Леваном" (and its English,
  Hindi, and Chinese equivalents) now resolves to a `calendar_create_event`
  intent instead of `unknown`, and the confirmation proposal carries two
  login-free, portable artifacts:
  - a universal **RFC 5545 `.ics` VEVENT** document (CRLF line endings,
    `DTSTART;TZID=`/`DTEND;TZID=`, escaped `SUMMARY`, stable content-derived
    `UID`) that imports cleanly into Apple Calendar, Outlook, Google Calendar,
    Thunderbird, and any other iCalendar client — the simplest method available
    in the CLI and HTTP environments, where the user can save/import a file; and
  - a **Google Calendar "render" template URL**
    (`calendar.google.com/calendar/render?action=TEMPLATE&text=…&dates=START/END&ctz=…`)
    that pre-fills a new event in the user's browser with no API token or server
    — the simplest method in a browser environment.
- Full multilingual support (en, ru, hi, zh). Surface words — schedule verbs,
  "meeting"/"встреча"/"मीटिंग"/"会议", clock times, and timezone aliases such as
  "по грузии" → `Asia/Tbilisi` — live as self-describing meanings in
  `data/seed/meanings-calendar.lino`; the code knows only roles and English
  slugs. Hindi (verb-final) and Chinese (no word spaces) titles are trimmed of
  trailing/leading schedule-action fragments so the `.ics` SUMMARY keeps only
  the event and its participant.
- Byte-for-byte Rust ↔ WASM parity: the `.ics` builder, Google Calendar URL
  builder, and title tidying are mirrored in the browser worker
  (`src/web/formal_ai_worker.js`), verified to produce identical artifacts
  across all four languages.

### Changed
- Extracted the calendar export logic (the `ScheduledEvent` model, RFC 5545
  `.ics` builder, Google Calendar URL builder, and date/duration helpers) into a
  new `src/solver_handlers/calendar_ics.rs` module, and split the docs-method /
  how-to procedure reasoning-path tests into
  `tests/unit/specification/reasoning_paths_procedures.rs`, keeping every file
  under the repository's 1000-line limit.

The core remains purely symbolic and deterministic: the solver *proposes* the
event and invites confirmation rather than silently mutating a remote calendar.
No existing weekday-relation or "today" calendar behaviour was changed.

Example (ru, Asia/Tbilisi):
  U: Забей мне 18 число в 17:00 по грузии на встречу с Леваном
  A: (calendar_create_event) "Создать событие «Встречу с Леваном» на 18 число
     (2026-06-18). Время: 17:00, часовой пояс: Asia/Tbilisi. … BEGIN:VCALENDAR …
     https://calendar.google.com/calendar/render?action=TEMPLATE… Ответьте «да»…"

### Added
- Relative-date calendar scheduling (issue #435). The prompt
  "Можешь поставить мне созвон в кальндарь на завтра?" — which carries no day
  number and no clock time, only a relative-date word ("на завтра") and an event
  noun ("созвон") — now resolves to a `calendar_create_event` instead of
  `unknown`. The solver recognizes relative-date words as a date anchor, resolves
  "завтра"/"tomorrow"/"कल"/"明天" to **tomorrow** (and "послезавтра"/"day after
  tomorrow"/"परसों"/"后天" to the day after), and titles the draft from the matched
  event noun when no explicit subject is given.
- New `calendar_relative_date` role with the `calendar_tomorrow` and
  `calendar_day_after_tomorrow` meanings in `data/seed/meanings-calendar.lino`,
  grounded in Wikidata and surfaced in en/ru/hi/zh. As with the rest of the
  lexicon-driven design, the code knows only the role and English slugs; adding a
  language never touches code.
- A new `calendar:parsed_relative_offset` evidence link records the resolved
  day offset, alongside the existing `calendar:parsed_*` trace, in both the Rust
  engine and the byte-for-byte browser worker mirror (`src/web/formal_ai_worker.js`).

The core remains purely symbolic and deterministic: the solver *proposes* the
tomorrow event with an importable RFC 5545 `.ics` VEVENT and a login-free Google
Calendar render URL, and invites confirmation rather than silently writing the
calendar. A bare relative-date mention with no schedule verb or event noun is
not hijacked into a create request.

Example (ru):
  U: Можешь поставить мне созвон в кальндарь на завтра?
  A: (calendar_create_event) "Создать событие «Созвон» на 14 число (2026-06-14).
     … BEGIN:VCALENDAR … calendar.google.com/calendar/render… Ответьте «да»…"

## [0.194.0] - 2026-06-13

### Fixed

- CI no longer runs the Rust test suite on changes that touch no code (issue #442). The `test` job in `release.yml` previously ran whenever the `changelog` job was *skipped* — but `changelog` is skipped precisely when there are no code changes, so docs-only / `.gitkeep` / changelog-fragment-only commits triggered the full `cargo test` matrix. The `test` job now gates on the `detect-changes` outputs (`any-code-changed` / `rs-changed` / `toml-changed` / `workflow-changed`), the same way `lint` and `coverage` already do.

## [0.193.0] - 2026-06-13

### Changed

- Generalized the installation-conversion command recognizer
  (`installation_conversion::looks_like_command`) from a fixed tool-prefix
  whitelist to a provenance-aware structural rule: any well-formed command line
  is accepted regardless of which tool it invokes, while prose lines are rejected
  even when they mention a tool (issue #433).
- Replaced the install-step description table (`describe_command`) with verb/object
  intent inference, so unseen but recognizable tools (`bun install`, `pdm install`,
  `just build`, `zig build`) get accurate step descriptions without extending a
  table.
- Mirrored the structural recognizer and verb/object describer into the browser
  worker (`src/web/formal_ai_worker.js`) for cross-runtime parity.

### Added

- Case study `docs/case-studies/issue-433/` with (1) an audit classifying every
  specialized handler recognizer as fixed-enumeration vs compositional, (2) the
  installation-conversion generalization, and (3) a documented reconstruction of
  the `numeric_list` coding handler from the meta-algorithm rule primitives.
- False-positive (prose) and unlisted-tool regression coverage for the
  installation-conversion recognizer across the Rust unit suite, the
  source-mirror private-function suite, and the browser-worker experiment.

## [0.192.0] - 2026-06-13

### Added
- Seeded a `metatheory` concept so prompts like `theory of theory`,
  `metatheory`, `теория теории`, and `元理论` resolve to a verified
  `concept_lookup` answer instead of the unknown fallback (issue #436). The
  record carries en/ru/zh summaries grounded in Wikipedia, keeps the existing
  Link Foundation `Links meta-theory` (`теория связей`) routing intact, and is
  grounded for total reference-closure via a new `metatheory` meaning and
  `proof_concept_metatheory` role.

## [0.191.0] - 2026-06-13

### Added
- Added deterministic installation conversion support for README.md
  install/deploy guides, Bash/sh scripts, and PowerShell scripts. The new
  `installation_conversion` handler extracts ordered install commands into a
  shared IR, renders scripts or README guides from that IR, and is mirrored in
  the browser worker so conversion prompts no longer fall through to `unknown`
  or generic script generation.

- Added issue #423 regression coverage, including README-to-Bash/PowerShell,
  script-to-README, nested fenced README content, PowerShell-to-README,
  meta-algorithm trace assertions, and a 100-project GitHub repository corpus
  captured from the most-starred repository snapshot.

- Added an algorithm-construction trace for installation conversion responses,
  connecting the problem-class -> shared-IR -> renderer -> verification pattern
  to the existing coding catalog, program synthesis, program blueprint,
  numeric-list, and rule-synthesis surfaces.

## [0.190.0] - 2026-06-12

### Changed
- Updated the `meta-language` dependency to 0.40.0 and documented the issue
  #428 upstream research, compatibility audit, and follow-up integration plan.

## [0.189.0] - 2026-06-12

### Fixed
- Apply user-requested text replacements to generated code answers, including follow-up replacement requests that refer to the previous assistant response.
- Accept broader replacement prompt shapes, including input-first phrasing, smart quotes, corner quotes, and punctuation-tolerant multi-word matches.
- Add deterministic remove, append, prepend, trim-whitespace, normalize-whitespace, case-conversion, extraction, counting, punctuation, and line-shape text/code edit operations with multilingual operation vocabulary triggers.
- Cover 61 benchmark-family prompt-answer examples across CoEdIT, EditEval, InstrEditBench, CodeEditorBench, CanItEdit, EDIT-Bench, HumanEvalFix, and SWE-bench style edit tasks.
- Add a manifest-backed issue #408 benchmark profile with 48 researched sources, 30 local variations per source, a per-source 3-check 10% floor, and a 1,440/1,440 pass-count ratchet.
- Document the issue #408 benchmark-source audit and keep the roadmap, vision, requirements, architecture, and case-study benchmark contract in sync.

## [0.188.0] - 2026-06-12

### Fixed
- Delegated `?` and `*` placeholder equations to `link-calculator` 0.18.2, with expanded coverage for symbolic multi-variable and polynomial equation categories.
- Kept the web worker aligned for placeholder, symbolic, and polynomial equation prompts, including Markdown-safe rendering of spaced `*` placeholders.

## [0.187.0] - 2026-06-12

### Fixed
- Recognize short behavior-rule list prompts such as `Покажи правила`, `Show rules`, `नियम दिखाओ`, and `显示规则` instead of falling through to the unknown fallback.

### Fixed
- Recognize calculation requests embedded inside longer user statements, including Russian prompts like `хочу понять сколько будет 2+2`.

### Changed
- Replaced emoji toolbar glyphs with accessible local icons and a persisted toolbar icon-pack setting.

## [0.186.0] - 2026-06-11

### Fixed
- Answer Russian hidden-number interval riddles by formalizing the bounds as a
  linear constraint and showing the proof-engine verification instead of
  returning the unknown fallback.

### Fixed

- Answer Russian “Что делаешь в свободное время?” small talk with localized assistant free-time responses instead of the unknown fallback.

## [0.185.0] - 2026-06-11

### Fixed
- A bare numeric-list follow-up no longer answers `unknown` (issue #412). After
  a turn establishes a coding context — e.g. "…отсортируй их в JavaScript, дай
  мне код и результат" — a follow-up that names no language and does not ask for
  code, such as `Отсортируй 4, 3, 1, 17, 8, 9, 15`, now recovers the target
  language (and the code request) from the conversation and continues the coding
  context: idiomatic code in the established language plus the deterministically
  computed result.

### Added
- Conversational coreference for the numeric-list coding path. A new
  `numeric_list_history_context` inherits the language / code request from a
  prior turn **only** when that turn was itself a genuine numeric-list coding
  request (a recognised operation, a supported program language, and ≥2 numbers),
  so unrelated chatter never leaks a language. A `numeric_list_coreference`
  trace event records what was inherited. Implemented identically in the Rust
  solver (`src/solver_handlers/numeric_list/mod.rs`) and the browser worker
  mirror (`src/web/formal_ai_worker.js`); the 170-cell cross-runtime parity
  matrix stays byte-identical.

### Added
- Coding oracle backed by external knowledge sources (issue #412, R6). The
  solver now treats Rosetta Code, Wikifunctions, the Hello World Collection, and
  Stack Overflow as cached external APIs (even though they expose no machine
  API) and generalises `write_program` beyond the verified catalogue: a request
  for a language the catalogue does not template — Kotlin, Swift, PHP, Bash, Lua,
  Haskell — now returns a reviewed snippet, its deterministic output, and its
  source attribution instead of dead-ending on the unsupported answer.
  Catalogued languages keep their verified "compiled and ran" route untouched;
  the oracle only ever supplies an answer the solver would otherwise lack. New
  module `src/knowledge.rs` (sources + `CodingOracle`) and handler
  `src/solver_handler_oracle.rs`, mirrored byte-for-byte in the browser worker
  (`src/web/formal_ai_worker.js`).
- Bounded-cache policy (issue #412, R8). `cache_capacity` /
  `within_cache_capacity` / `KNOWLEDGE_CACHE_FLOOR` enforce "never cache more
  than 1% of a source, or 512 items when 1% is smaller", clamped to the source
  size, for every per-source / per-topic cache. A ratchet test fails CI if the
  committed snapshot set ever exceeds the cap, so a cache can never silently grow
  into a mirror.

## [0.184.0] - 2026-06-10

### Added
- Issue #395: a concrete "sort these numbers in <language>, give me the code and the result" request now routes to `write_program` instead of `unknown`. Rather than a narrow sorting handler, this ships a universal, data-oriented list coding engine (`src/solver_handlers/numeric_list/`): it reads the operation, the given values, and the target language from meanings, builds a semantic `NumericProgram` syntax tree, renders idiomatic code in the requested programming language (JavaScript, TypeScript, Python, Rust, Go, Ruby, Java, C#, C, C++), and computes and shows the deterministic result in the solver itself.
- Added CST/AST validation for generated programs in native Rust using [link-foundation/meta-language](https://github.com/link-foundation/meta-language) 0.39 (`meta_language::LinkNetwork`) as the sole, mutable CST/AST engine. `src/coding/cst.rs` parses rendered source through the links network and accepts it only when the parse produces real `LinkType::Syntax` links, round-trips the text, and has no errors. meta-language 0.39 ships grammars for every target (JavaScript, Python, Rust, Java, C, C++, C#, TypeScript, Go, Ruby), so all of them validate through the same links network and the direct `tree-sitter` dependency is gone. The TypeScript/Go/Ruby grammar gaps were reported upstream and resolved in meta-language — [#41](https://github.com/link-foundation/meta-language/issues/41), [#42](https://github.com/link-foundation/meta-language/issues/42), [#43](https://github.com/link-foundation/meta-language/issues/43). The remaining missing feature — rendering target-language source from a programmatically constructed syntax network, so generated code would be valid by construction instead of validated after composition — is reported as [meta-language#64](https://github.com/link-foundation/meta-language/issues/64). The trace now logs `synthesis:cst_engine` (`meta_language`) alongside `synthesis:cst_tree`.
- The list engine covers seven operations out of one shared algorithm — `sort`, descending sort (`reverse_sort`), `reverse`, `sum`, `product`, `minimum`, `maximum` — driven by `data/seed/numeric-list-operations.lino`. Transformations now support both numeric lists and quoted string lists; numeric reductions remain gated behind a `code_request` signal to avoid over-matching prose. Localized rendering covers all four supported UI languages (English, Russian, Hindi, Chinese).

- Code generation is now seed data, not code: `data/seed/coding-idioms.lino` declares per-language scaffolds and idioms (code fragments with cases selected by operation and value class, inherited through `extends`), and both runtimes discover the composition at execution time by walking the language's inheritance chain and recursively expanding idiom slots (`src/solver_handlers/numeric_list/codegen.rs` and the matching composer in `src/web/formal_ai_worker.js`). The per-language renderer functions are gone, so covering a new language or coding task is a seed-data change. Composition failures are explicit (`None`/`null`), never silent fallbacks.

### Changed
- Mirrored the list engine in the web runtime (`src/web/formal_ai_worker.js`, `tryNumericList`) so the browser and Rust paths build equivalent syntax trees and produce equivalent code/results, including quoted string list transforms, verified by `experiments/issue-395-js-numeric-list.mjs` and exhaustively by `experiments/issue-395-cross-runtime-codegen-parity.mjs`, which byte-compares all 170 (operation × language × value class) answers between the Rust engine (`examples/numeric_list_matrix.rs`) and the worker.
- Updated the related Python program-synthesis handler in Rust and the web worker to store a `PythonFunctionTree` with semantic statement nodes and render source from that tree. Native Rust now also validates the rendered Python source through meta-language and exposes `synthesis:cst_engine` and `synthesis:cst_tree` evidence before the code fragment.
- Added `examples/numeric_list_execution.rs`, an execution-verification harness that compiles and runs every generated program across the available toolchains and asserts the program's stdout equals the solver's computed result, closing the loop between the "verified by construction" claim and real execution.

## [0.183.0] - 2026-06-10

### Added
- Added meaning-level semantic facets so seed meanings can link notation, annotation, denotation, and connotation to other meanings.
- Added word-form semantic facets plus derived notation/denotation links and lexical meta meanings for word surfaces, lexical forms, lexical senses, and part-of-speech links.
- Added a compact Links-Theory semantic root seed with self-equations, defined connectives, quantity primitives, and one-symbol-one-meaning sense splits.
- Added semantic grounding checks and source cache records so Links-root definitions resolve recursively through checked-in Wikidata and Wiktionary data.
- Added issue #398 case-study documentation and semantic meta-language seed vocabulary.

### Fixed
- Replaced the codepoint byte-dump encoding of seed text (e.g. `answer codepoints 72 105 ...`) with human-readable quoted scalars, so `data/seed/*.lino` stays legible while every runtime parser decodes the same values.

### Changed
- Taught the Rust, web, and e2e LiNo seed parsers (plus the worker's embedded fallback) to decode single-quote, double-quote, and backtick scalars with a non-escaping delimiter.
- Removed the 4,677 synthetic `seed-surface-<hash>` ids from `data/seed/*.lino`: a surface is now the text (and facets) recorded under a language, not an opaque minted id. Added `scripts/clean-seed-readability.rs` to perform the lossless migration and regenerate the browser worker fallback.
- Stripped keyword-restating noise comments (`# language`, `# definition-link`, `# semantic-role`, `# facet`, `# seed lexical surface`, `# source-id`, `# action`) from the seed while keeping comments that carry the human meaning of an opaque id.

### Added
- Added a CI guard that bans codepoint byte-dumps in seed data and a guard that bans inline `#[test]`/`#[cfg(test)]` scaffolding under `src/`.
- Added CI guards that ban reintroducing synthetic `seed-surface-<hash>` ids and keyword-restating noise comments in seed data.

### Fixed
- Collapsed every `facet <kind>` wrapper whose child was an empty-bodied colon redefinition (`word_surface:`, `lexical_sense:`, ...) into native Links Notation `subject predicate` lines (`notation word_surface`, `denotation lexical_sense`). This removes the valueless `concept:` shape the review banned, across the whole `data/seed` tree.

### Changed
- Taught the Rust seed consumer (`parse_semantic_facets`) to read the direct `<kind> <target>` subject-predicate form in addition to the legacy `facet <kind>` wrapper, de-duplicating targets so both forms project identical facets.
- Added `scripts/migrate-empty-facet-fields.rs`, a std-only re-runnable migration that performs the collapse tree-wide and regenerates the embedded browser worker fallback (`src/web/formal_ai_worker.js`).

### Added
- Added a tree-walking CI guard (`seed_lino_files_have_no_empty_redefinition_fields`) that fails when any `data/seed/**/*.lino` line is an empty-bodied colon field with no deeper-indented child, with no hard-coded filename.

### Added
- Added the `data/overrides/` grounding override layer beside `data/cache/` with the same per-id structure. Resolution is `(cache or live API) then overrides`: `formal_ai::seed::resolve` decorates a cached external-source record with an override's facts, and every override records why it exists in a `reason` line.
- Added a tree-walking CI suite (`tests/unit/overrides.rs`) that fails when an override references an id with no checked-in cache record, omits its reason, carries no facts, or is redundant (repeats a value the cache already holds), so the layer self-prunes once upstream catches up.

### Changed
- Recorded the issue #398 PR review data-quality standards in `REQUIREMENTS.md` (R278-R283) under the governance rule "latest requirement overrides any earlier one", mapping each CI check to its requirement.

### Changed
- Made the JSON ↔ Links Notation cache codec (`formal_ai::json_lino`) losslessly round-trip the *entire* Wikidata/Wiktionary snapshot — `forms`, `senses`, `claims`, and every metadata key — instead of the previous lexeme-only projection. `data/cache/**/*.lino` were regenerated as full native snapshots (e.g. `L3412` 6 → 195 lines), empty arrays/objects/nulls are never emitted, and the Wiktionary source JSON is pretty-printed multi-line.
- Replaced the circular round-trip test (which compared the lino to the converter's own lossy output) with one that rebuilds the full original JSON from the lino and asserts key-for-key equality with the raw `.json` (`wikidata_lino_cache_rebuilds_full_json_losslessly`, `wiktionary_cache_is_pretty_printed_and_rebuilds_full_json`, and the `verify_cache_roundtrip` example).
- Migrated every meaning header in `data/seed/**/*.lino` from the YAML-style trailing-colon form (`monday:`) to native Links Notation nodes (`monday`), removing all 428 empty colon redefinition fields tree-wide. The transform is parse-equivalent (`parse_colon_definition` already mapped `monday:` to `(name = "monday", id = "")`) and regenerates the embedded browser-worker seed. `seed_lino_files_have_no_empty_redefinition_fields` now enforces the reviewer's exact `^\s*[\w-]+:\s*$` regex.

### Added
- Added `scripts/migrate-empty-redefinition-fields.rs`, a re-runnable whole-tree migration that strips trailing-colon redefinition headers and refreshes the `src/web/formal_ai_worker.js` embed.

### Added
- Added `scripts/ground-meanings.rs`, a re-runnable, self-verifying Wikidata grounding pipeline (issue #398, defect #3). For each curated `(slug, id, expected-label-token)` it fetches `Special:EntityData/<id>.json`, trims it to the cache convention (`type`/`id`/`labels`/`descriptions`/`aliases` in en/ru/hi/zh, wrapped in `{entities:{…}, success:1}`), **verifies** the entity's labels actually contain the expected concept token before grounding — refusing wrong ids such as `Q206` ("Stephen Harper", not "seven") — writes the lossless `.lino` snapshot, and inserts `grounded-in <id>` into the meaning block idempotently. Ids are sharded by kind, so items land under `entity/`, properties under `property/`, and lexemes under `lexeme/`.
- Grounded 114 common-vocabulary meanings to verified Wikidata items, raising grounded-meaning coverage from 18 to 131 `grounded-in` anchors (30.6% of the 428 seed meanings). Coverage now spans calendar weekdays, days, dates and weeks; arithmetic operations and mathematical functions (including `cosine`, `tangent`, `modulo`); cardinal numbers 0–10; currencies and exchange rate; length/mass/time/temperature/data-size units and their physical dimensions; unit conversion; the 11 catalogued programming languages; the four supported natural languages; the concrete-noun translation vocabulary (`apple`, `bread`, `water`, …) plus `translate` and `synonym`; the lexical-meta concepts (`noun`, `part_of_speech`, `noun_phrase`, `grammatical form`, `word sense`); finance concepts (`investment`, `interest`, `compound interest`, `year`); and core quantities and the `physical constant`. Fact relations ground to Wikidata **properties** — `capital` → `P36`, `population` → `P1082`, `continent` → `P30`, `currency` → `P38`, `official_language` → `P37`, `author_of_book` → `P50`, `painter_of_painting` → `P170`, `built_year` → `P571` — rather than to items. Every id's source snapshot is checked in under `data/cache/wikidata/`.
- Added the `grounded_meaning_coverage_does_not_regress` data test: a monotonic ratchet that records the grounded-meaning floor (131) so grounding is append-only and progress toward full grounding can only increase.

### Added
- Added `scripts/ground-lexemes.py`, a re-runnable, self-verifying Wikidata **lexeme** grounding pipeline (issue #398, defect #6 / CI check 6). For each curated `(slug, lexeme-id, expected-lemma, sense-id)` it fetches the full `Special:EntityData/<L-id>.json` (lexemes are cached untrimmed so claims, forms and senses survive), caches it pretty-printed under `data/cache/wikidata/lexeme/`, generates the lossless `.lino` snapshot, **verifies** the entry really is the expected English noun (lemma match, `language` Q1860, `lexicalCategory` Q1084, named sense present) — refusing on any mismatch — and rewrites the meaning's plain `lexeme en` surface into the rich `source-lexeme` notation, sourcing the part of speech (`lexical-category`), every form (`form` + grammatical `feature`) and the grounded `sense` directly from the lexeme rather than hand-authoring them.
- Enriched the grounded concrete-noun vocabulary (`apple`, `water`, `bread`, `potato`, `tomato`) with full lexical detail sourced from Wikidata lexemes (`L3257`, `L3302`, `L3865`, `L3784`, `L7993`): each now records its part of speech and its singular/plural forms with grammatical features (`Q110786` singular, `Q146786` plural), and its English surface references a real lexeme form instead of a hand-typed string.
- Added the `lexical_completeness_does_not_regress` data test: a monotonic ratchet (floor 6) that records how many meanings expose a `source-lexeme` with its part of speech and at least one form + feature, and asserts every referenced lexeme resolves to a checked-in cache file. Lexical grounding is append-only, so coverage toward the defect #6 goal — every grounded word carrying its parts of speech and forms from the source — can only increase.

### Changed
- Replaced every pipe-packed multi-value in `data/seed/*.lino` with the canonical
  reference-list form `keyword ("a" "b c" d)`, so multi-values are real links
  instead of in-string separators (issue #398, defect #4). Covers
  `supported_languages`, `tasks`, `languages`, `inputs`, `outputs`, aliases, and
  every other former `"a|b|c"` field; `code` listings remain the sole field that
  may legitimately contain `|`.
- The LiNo reference-list tokenizer now decodes quoted scalars (which may contain
  spaces) across all four parsers (Rust `seed::parser`, `src/web/seed_loader.js`,
  the e2e `lino-seed-parser.mjs`, and migration tooling).

### Added
- `formal_ai::supported_languages()` accessor that reads the declared languages
  from the `agent-info.lino` reference list, replacing ad-hoc `split('|')` parsing
  scattered across the test suite.
- A comprehensive CI guard (`seed_lino_values_never_pipe_pack_multi_values`) that
  fails on *any* `|` in a seed value except the exempt `code` field, so pipe
  packing can never silently return.

### Added
- Grounded five more conversational/discourse meanings to verified Wikidata
  items (issue #398, defect #3): `greeting_hello` → `Q98815142` (the English
  salutation "hello"), `gratitude_thank_you` → `Q2728730` (gratitude),
  `affirmation_yes` → `Q6452715` (the affirmative particle "yes"), `example`
  → `Q14944328`, and `conjunction_or` → `Q1651704` (logical disjunction). Each
  id was confirmed by the `scripts/ground-meanings.rs` label-token verifier
  before grounding, and its trimmed source snapshot is checked in under
  `data/cache/wikidata/entity/`. The `grounded_meaning_coverage_does_not_regress`
  ratchet floor rises from 131 to 136 (31.8% of the 428 seed meanings).

### Added
- Grounded three core programming-artifact meanings to verified Wikidata items
  (issue #398, defect #3): `program` → `Q40056` (computer program),
  `code` → `Q128751` (source code), and `sort` → `Q2303697` (sorting, the
  action of arranging objects into order). Each id was confirmed by the
  `scripts/ground-meanings.rs` label-token verifier, with its trimmed source
  snapshot checked in under `data/cache/wikidata/entity/`. The
  `grounded_meaning_coverage_does_not_regress` ratchet floor rises from 136 to
  139 (32.5% of the 428 seed meanings).

### Added
- Grounded two more meanings to verified Wikidata items (issue #398, defect #3):
  `politeness` → `Q281287` (politeness, the application of good manners) and
  `calendar_today` → `Q3151690` (today, the current day). Each id was confirmed
  by the `scripts/ground-meanings.rs` label-token verifier, with its trimmed
  source snapshot checked in under `data/cache/wikidata/entity/`. The
  `grounded_meaning_coverage_does_not_regress` ratchet floor rises from 139 to
  141 (32.9% of the 428 seed meanings).

### Added
- Wiktionary grounding pipeline `scripts/ground-wiktionary.py` (issue #398, open
  item #1 of the `92a29b0` review): it **discovers** candidate lemmas from the
  data — every single-word English surface of a `grounded-in` meaning — fetches
  each from the Wiktionary-backed Free Dictionary API (CC BY-SA 3.0, the same
  source and schema as the existing `en/reference.json`), **verifies** the
  response actually describes the requested lemma, and caches it as pretty
  multi-line JSON plus the lossless `.lino` snapshot via the
  `wikidata_json_to_lino` codec. Idempotent and re-runnable.
- 155 verified Wiktionary entries under `data/cache/wikidata`'s sibling
  `data/cache/wiktionary/en/`, raising the cache from a single placeholder entry
  to 156. Each entry round-trips its full JSON through
  `wiktionary_cache_is_pretty_printed_and_rebuilds_full_json`.
- `wiktionary_cache_breadth_does_not_regress` ratchet (floor 156) so Wiktionary
  coverage is append-only and can only grow as more grounded surfaces are cached.

### Added
- **Total reference-closure at zero (issue #398, PR #399 review 4668929105).**
  Widened the closure gate from the `defined-by`/facet/role backbone to *every*
  non-keyword, non-quoted value token in `data/seed/**.lino`. `scripts/close-total.py`
  is an idempotent migration that defines each previously-dangling token as a
  first-class meaning — 17 parent category concepts (intent, task,
  prompt_pattern, source_kind, programming_language, …) rooted at `concept`,
  plus 508 member meanings parented under the category their predicate implies.
  `scripts/audit-total-closure.py` now reports **0** unresolved tokens.
- **Open English WordNet 2024 source.** `scripts/ground-wordnet.py` imports OEWN
  2024 offline (one download, no per-word network calls) and caches 312 English
  lemmas as `.json` + lossless `.lino` under `data/cache/wordnet/en/`, recorded
  under CC BY 4.0.
- **Multi-source `data/view/` merge layer.** `scripts/build-views.py` merges the
  WordNet and Wiktionary lexical caches into 536 per-lemma view entities, each
  with a deterministic `M-<sha1[:12]>` id, a `sources` list, and per-sense
  provenance. Senses sharing part-of-speech with gloss Jaccard ≥ 0.5 merge and
  keep both sources; others stay separate. `--check` verifies no drift, id
  determinism, and merge-threshold correctness.
- **`data/seed/sources-registry.lino`** enumerating every ingested source
  (Wikidata, Wiktionary, WordNet, Wikipedia) with its API endpoint, permissive
  license, and cache path.
- **`tests/unit/total_closure.rs` CI gates** that fail immediately if: any seed
  value token is unresolved (naming offenders); the seed collapses below
  hundreds of meanings; the WordNet cache is absent; `sources-registry.lino`
  omits an ingested source, API, or license; `data/view/` is missing, drifted,
  non-deterministic, or has a provenance-less field; or no view entity is
  genuinely multi-source.

### Added

- `meaning_definition_references_resolve_to_defined_meanings` CI gate proving
  the meaning graph is fully reference-closed: every `defined-by` target and
  every semantic-facet reference (notation/annotation/denotation/connotation)
  across all 35 `data/seed/meanings*.lino` files resolves to a meaning defined
  exactly once (issue #398, PR #399 review point #1).
- `data/seed/roles.lino` canonical reserved-role registry declaring all 207
  distinct `role` values exactly once, each classified as `kind meaning` (also
  a defined meaning slug) or `kind predicate` (role-only identifier).
- `scripts/generate-role-registry.py` to regenerate the role registry
  deterministically and idempotently from the meaning seed.
- `every_role_value_is_declared_in_the_registry` and
  `role_registry_is_in_lockstep_with_usage` tests keeping the registry and its
  usage in lockstep (PR #399 review point #2).
- `experiments/closure_audit.py` documenting the meaning-layer closure
  measurement.

## [0.182.0] - 2026-06-08

### Fixed
- Route Russian latest-news prompts such as `последние новости` to web search and include Wikinews as a CORS-readable news source.

## [0.181.0] - 2026-06-04

### Fixed

- Keep Russian unknown-response and capability rule-configuration examples in Russian, and describe the behavior as local links rules rather than Links Notation rules.

## [0.180.0] - 2026-06-04

### Fixed

- Fixed conversation-list "Copy" so conversation Markdown is written to the
  clipboard during the user's click, and added e2e coverage for every chat copy
  action without relying on pre-granted clipboard permission.

## [0.179.0] - 2026-06-04

### Fixed
- Kept oversized changelog entries from breaking GitHub release creation by shortening release notes and linking to the full tagged changelog.
- Made GitHub release creation fail on unexpected validation errors instead of treating every `Validation Failed` response as an existing release.
- Prevented automatic desktop release builds from targeting a stale latest release when the completed CI run has no matching GitHub release.
- Removed the invalid `electron-builder --config package.json` desktop packaging flag so electron-builder reads the top-level `build` configuration normally.

## [0.178.0] - 2026-06-04

### Fixed
- Made the web topbar collapse action labels with a container query before desktop widths clip the action row.
- Added dark-theme coverage for diagnostics detail cards, diagnostic payload blocks, conversation copy controls, and settings reset controls.

### Added
- Added Playwright coverage for issue #388 topbar fit and dark-theme surface parity.

## [0.177.0] - 2026-06-04

### Added
- Settings panel can reset each setting to its default individually, or all of
  them at once (issue #386).
- Conversations list can copy the whole dialog as Markdown; with diagnostics
  mode on, reasoning steps are folded in after each AI message (issue #386).

### Changed
- Prompt recognition references *meanings*, not hardcoded word lists. A new
  canonical lexicon (`data/seed/meanings.lino`) defines language-independent,
  self-describing meanings — each `defined_by` other meanings (a closed graph in
  the spirit of relative-meta-logic), grounded in real lexical data
  (`wiktionary`), tagged with the semantic `role`s it plays, and lexicalised in
  every supported language. The program-artifact follow-up gate
  (`src/program_coreference.rs` and its `formal_ai_worker.js` mirror) no longer
  enumerates ~100 per-language words; it asks the lexicon which surface words
  evidence a `program_artifact` and a `program_modification`, so the words live
  once in data while the code understands the concepts (issue #386).
- Unit-incompatibility detection (`src/solver_handler_units.rs`) is now
  data-driven too. The units, the physical dimensions they measure, and their
  surface words in every supported language live in the lexicon
  (`data/seed/meanings-units.lino`), where each unit meaning is `defined_by` the
  dimension it measures. The handler walks every `measurement_unit` meaning and
  resolves its dimension label through the `defined_by` graph, so the code knows
  only the concepts "measurement unit" and "physical dimension" — no hardcoded
  unit arrays remain. The lexicon is split across `meanings*.lino` files (listed
  by `MEANING_FILES`) so no single seed file breaches the file-size guard; the
  Rust loader and the `formal_ai_worker.js` mirror both walk every `meanings`
  container (issue #386).
- Calendar weekday reasoning (`src/solver_handlers/calendar.rs` and its
  `formal_ai_worker.js` mirror) is data-driven too. The seven weekdays, the
  "day after"/"day before" relations, "today", the day/date/week references, and
  the interrogatives that ask "which day" now live as self-describing meanings
  in `data/seed/meanings-calendar.lino` — each `defined_by` the calendar
  concepts it builds on and lexicalised in every supported language. The handler
  detects the operation and weekday by querying the lexicon for the
  `calendar_direction_next`/`calendar_direction_previous`/`calendar_weekday`/…
  roles instead of matching hardcoded alias and marker arrays. Because the words
  now exist in every language, weekday-relation answers work in Hindi and
  Chinese as well as English and Russian — not only the originally supported
  cases (issue #386).
- Knowledge-base fact-relation detection
  (`src/solver_handlers/benchmark_prompts.rs`) is data-driven too. The nine
  relations a fact query can ask about (capital, population, currency, official
  language, continent, book author, painting painter, build year, physical
  constant) and the surface words that evidence each one in every supported
  language now live as self-describing meanings in
  `data/seed/meanings-facts.lino` — each `defined_by` a `knowledge_relation`
  concept that is in turn `defined_by` `knowledge_subject` and `knowledge_value`
  (a closed cycle in the spirit of relative-meta-logic). `detect_relation` walks
  every meaning carrying the `fact_relation` role in declaration order instead
  of the former hardcoded per-language keyword table, so the code knows only the
  concept "a relation maps a subject to a value" while the words live once in
  data. Declaration order is preserved so the shared "написал" verb still
  resolves to the book author before the painting painter, and the relation
  slugs (hence the `fact_query:relation:*` reasoning trace) stay identical to the
  browser worker (issue #386).
- Software-project request recognition (`src/solver_handlers/software_project.rs`
  and its `formal_ai_worker.js` mirror) is data-driven too. The authoring verbs
  (write/build/create/implement/develop/design/scaffold) and the 19 artifact
  kinds a request can ask for (web app, CLI tool, browser extension, library, …)
  now live as self-describing meanings in
  `data/seed/meanings-software-project.lino` — each artifact kind `defined_by`
  the `software_artifact` genus and lexicalised in every supported language. The
  handler builds its recognition tables by querying the lexicon for the
  `software_authoring_action` and `software_artifact_kind` roles, resolving a
  matched lexeme back to its stable slug; a small in-code resolver maps the slug
  to its canonical English label (the calendar `from_slug` precedent), so
  recognition vocabulary lives in data while the canonical output stays in code.
  The word-boundary scan is now CJK-aware: CJK surfaces match as substrings while
  Latin/Cyrillic/Devanagari keep whole-token boundaries, so a short surface like
  `апи` (API) never matches inside the Cyrillic verb `напиши` ("write") — fixing
  a regression that mislabelled a plain "write a program" request as a software
  project. Because the artifact words now exist in every language, "create a
  library"/"создай библиотеку"/"एक डैशबोर्ड बनाओ"/"开发一个网站" all resolve to
  the same canonical artifact. Feature-requirement detection and subtask
  categorization are data-driven the same way: the seven requirement categories
  (state tracking, data exchange, automation, validation, integration, user
  interface, and a catch-all project behavior) are self-describing meanings
  `defined_by` the `software_feature` genus and lexicalised in every supported
  language. A clause is a requirement when it contains any
  `software_requirement_category` word, and the first category (in declaration
  order) whose word it contains classifies the resulting subtask, so the former
  hardcoded `FEATURE_MARKERS` list and the seven-branch classifier are gone — the
  code knows only the concept "a requirement has a category" (issue #386).
- The remaining software-project request signals are lexicon-driven too. The
  delivery mode (manual instructions, immediate execution, script generation, or
  the default generated code), the implementation language (python, rust,
  javascript, or the default typescript), the game-unit tracker (a request is one
  only when it pairs a `game_tracker_domain` with a `game_tracker_mechanic`), the
  step-granularity and shell/command approval gates, and the whole-prompt approval
  trigger (approve/yes/proceed/…) are now self-describing meanings in
  `data/seed/meanings-software-project.lino`, each `defined_by` the concepts it
  builds on and lexicalised in every supported language. The detectors walk the
  matching `software_delivery_mode`/`software_implementation_language`/
  `game_tracker_*`/`software_step_granularity`/`software_bash_command`/
  `software_approval_trigger` roles (delivery modes and languages in declaration
  order, so the order encodes priority) and resolve a matched slug back to its
  stable label, so the former hardcoded `contains_any`/`contains_word` keyword
  lists are gone — the code knows only the concepts and the words live once in
  data. The boundary-aware, now phrase-capable matcher (`surface_present`)
  replaces raw substring scans, so a short surface like `hp` never matches inside
  `php` and a multi-word go-ahead matches only on word boundaries; the
  `formal_ai_worker.js` mirror walks the same embedded meanings, and all 22
  dialogue examples classify identically in the Rust solver and the browser
  worker (issue #386).
- Python program-synthesis recognition
  (`src/solver_handlers/program_synthesis.rs`, the
  `looks_like_program_synthesis` router in `src/intent_formalization.rs`, and the
  `formal_ai_worker.js` mirror) is data-driven too. The request *subject* (the
  function asked for), its *domain* signals (Python, or a data kind it works over
  — tuple/numbers/vowels), the request *action* verbs (implement/write/return),
  the per-task distinguishing *signals* (distinct numbers/differ/threshold/
  similar elements/count vowels), and the synthesis *tasks* themselves
  (`has_close_elements`, `similar_elements`, `count_vowels` — each slug *is* the
  canonical Python function name) now live as self-describing meanings in
  `data/seed/meanings-program-synthesis.lino`, each `defined_by` the concepts it
  builds on and lexicalised in every supported language. The gate asks the
  lexicon for the `program_synthesis_subject`/`_domain`/`_action` roles; task
  selection walks every `program_synthesis_task` meaning in declaration order and
  picks the first whose `defined_by` `program_synthesis_signal`s are all
  evidenced, using its slug directly as the function name — so the former
  hardcoded English-substring gate and the per-task phrase checks
  (`"similar elements"`, `"count vowels"`, `"distinct numbers" && "differ" &&
  "threshold"`) are gone. The browser worker additionally embeds the full
  multilingual operation vocabulary inline (byte-identical to
  `data/seed/operation-vocabulary.lino`) and runs `canonicalizedPrompt()` — the
  JS mirror of `OperationVocabulary::canonicalized_prompt` — before gating,
  replacing the hand-maintained three-operation `PROGRAM_MODIFIER_OPERATIONS`
  subset it carried before; native operation verbs glued to sentence punctuation
  (a Hindi `लिखें।`, a Chinese `编写`, a Russian `напиши`) now canonicalize to
  their English tokens so the boundary-aware gate accepts them. Because the
  vocabulary now exists in every language, multilingual `count_vowels` and
  `similar_elements` requests in Russian, Hindi, and Chinese synthesize correctly
  in both the Rust solver and the browser worker — not only English (issue #386).
- Write-a-script recognition (`is_write_script_request` in
  `src/solver_helpers.rs`, its single call site in
  `src/solver_handlers/mod.rs`, and the `formal_ai_worker.js` mirror) is
  data-driven too. Four new semantic roles in `src/seed/roles.rs` name the
  concepts the recogniser reasons over — `program_genus` (the broad "program"
  noun), `script_authoring_verb` (the write / напиши / написать / लिखो / 编写
  author verb, a strict subset of `program_request` that omits
  show/create/generate), `script_or_code_artifact` (the script / code / скрипт /
  код / स्क्रिप्ट / कोड / 脚本 / 代码 noun, a strict subset of `program_kind`
  that excludes the program genus and the function noun), and
  `hello_world_reference` (the canonical hello-world archetype) — carried by the
  `program`, `write`, `script`/`code` meanings and a new self-describing
  `hello_world` meaning in `data/seed/meanings.lino`, each lexicalised in every
  supported language. The recogniser steps aside for the broad program genus and
  the hello-world archetype (which the parametric write-program and
  program-synthesis routes own) and otherwise fires when a `script_authoring_verb`
  meets a `script_or_code_artifact`, so the former hardcoded per-language
  verb/noun substring lists are gone — the code knows only the concept "author a
  script" and the two routes it defers to. Adding the `написать` infinitive to the
  `write` meaning also makes it evidence `program_request` like its imperative
  sibling `напиши`, so "написать код" now routes to the program path consistently
  rather than falling through to `unknown`. The implementation file's unit tests
  moved to a sibling `src/solver_helpers_tests.rs` mounted with `#[path]` (the
  `blueprint_tests.rs` precedent) so the recogniser file stays under the
  1000-line file-size guard, and the worker's embedded `MEANINGS_LINO`
  regenerates byte-identically (issue #386).
- Conversational-intent recognition is data-driven too. A closed sub-graph of
  conversational meanings (`data/seed/meanings-intent.lino`) defines the
  assistant, user, inquiry, and answer plus the concepts they build on —
  capability, knowledge, fact, introduction, clarification, understanding — each
  `defined_by` the others (a closed graph in the spirit of relative-meta-logic)
  and lexicalised in every supported language. Five role-bearing meanings carry
  the surface words the handlers used to hardcode: `clarification_request`
  ("I don't understand", "не понял", "समझ नहीं आया", "我不明白"),
  `capability_query` ("what can you do", "что ты умеешь", the "что за дичь"
  slang, "你能做什么"), its follow-up `capability_query_more` ("what else can you
  do", "что ещё ты умеешь", "और क्या कर सकते", "你还能做什么"), `self_fact_query`,
  and `self_introduction_request`. The clarification and capability gates
  (`src/solver_handlers/user_intent.rs`) and the self-fact / self-introduction
  gates (`src/solver_handlers/self_awareness.rs`) now ask the lexicon which role
  a prompt evidences instead of matching per-language phrase arrays; each
  re-normalises the prompt first so trailing punctuation ("what can you do?") and
  apostrophes ("I don't understand") collapse to the canonical spacing the seed
  stores. Recognition is language-agnostic — the surface words are
  script-specific — while the per-language response bodies stay in code, so the
  Chinese/Hindi "what else can you do" follow-ups ("你还能做什么", "और क्या कर
  सकते") now reach the capabilities answer even though the former
  Russian/English-only "more" check missed them. The `formal_ai_worker.js` mirror
  queries the same embedded meanings, and a parity harness
  (`experiments/issue-386-js-intent-lexicon.mjs`) proves the worker's role →
  word-sets and its recognizers agree with the seed and the Rust handlers across
  all four languages (issue #386).
- The "how does X work" / "how to X" handler (`src/solver_handler_how.rs`) is
  data-driven too. Two self-describing meanings in `data/seed/meanings-how.lino`
  carry every surface the handler used to hardcode: `mechanism_inquiry`
  ("how does X work", "как устроен X", "X कैसे काम करता है", "X 如何工作") and
  `procedural_request` ("how to X", "как сделать X", "कैसे करें X", "如何做 X"),
  each `defined_by` the `inquiry` and `action` concepts and lexicalised in every
  supported language. Rather than carry per-language prefix/circumfix/suffix
  arrays, each surface word encodes the position of the subject (or task) slot
  with an ellipsis marker `…` (U+2026): no marker is a bare phrase, a trailing
  `…` is a prefix surface, a leading `…` is a suffix surface, and a `…` in the
  middle is a circumfix surface. The handler derives its affix-matching strategy
  by bucketing the forms by `WordForm::slot()` (a `Slot` computed from the
  marker) and matching each against the prompt — so the code knows only the
  concepts "an inquiry into a mechanism" and "a request for a procedure", never a
  surface word. A procedural surface may name its canonical operation in an
  `action` child (do/perform/implement/create/write); when it does not, the
  operation is taken from the task's first word. Declaration and bucket order are
  preserved so behaviour is identical to the former inline arrays, and the
  existing multilingual reasoning-path tests still pin "how it works", "как
  устроен AUR", "AUR कैसे काम करता है", "AUR 如何工作", and the procedural
  "how to" cases. The `formal_ai_worker.js` mirror drives its
  `extractHowItWorksSubject` / `extractProceduralHowToTask` recognisers from the
  same embedded meanings — bucketing the slot-marked surfaces by position with a
  shared `makeWordForm` helper exactly as the Rust handler does — instead of the
  inline per-language prefix/circumfix/suffix arrays it carried before. A parity
  harness (`experiments/issue-386-js-how-cluster.mjs`) proves the worker
  reproduces the canonical surface set with the expected per-slot bucket counts
  and returns byte-identical results to the pre-conversion logic across a
  multilingual prompt battery (issue #386).
- The web-intent handlers (`src/solver_handlers/web_requests.rs` and their
  `formal_ai_worker.js` mirror) are data-driven too. Three self-describing
  meanings in `data/seed/meanings-web-navigation.lino` carry every surface the
  two handlers used to hardcode in four inline arrays: `web_resource` (the
  URL-identified thing both intents act on — url/site/page, `defined_by`
  `entity`), `http_fetch` ("fetch …", "сделай запрос к …", "अनुरोध भेजें",
  "发送请求"), and `url_navigate` ("go to …", "открой …", "पर जाएं", "打开"), the
  two verbs each `defined_by` `inquiry` + `action` + `web_resource` and
  lexicalised in every supported language. As in the how-cluster, each surface
  marks its URL slot with the ellipsis marker `…` (U+2026): a trailing `…` is a
  prefix surface ("fetch …" begins "fetch google.com") and no marker is a bare
  phrase matched anywhere ("запрос к" appears inside "сделать запрос к
  google.com"). A shared `role_evidences_web_intent` helper buckets a role's
  forms by `WordForm::slot()` and matches each against the prompt, so
  `is_http_fetch_prompt`/`is_url_navigate_prompt` ask the lexicon for the
  `http_fetch`/`url_navigate` roles instead of carrying
  `HTTP_FETCH_PREFIXES`/`HTTP_FETCH_MARKERS`/`URL_NAVIGATE_PREFIXES`/
  `URL_NAVIGATE_MARKERS` — the code knows only the concepts "fetch a web
  resource" and "navigate to a web resource". The protective URL gate
  (`first_url_candidate`, which rejects `@`-bearing tokens so emails never
  trigger) and the bare-URL navigation early-return are unchanged. Because the
  verbs now exist in every language, Hindi and Chinese fetch/navigate requests
  ("打开 https://…", "获取 https://…", "पर जाएं …") route correctly where the
  former English/Russian-only arrays recognised nothing, with the fetch and
  navigate verb sets staying disjoint. A parity harness
  (`experiments/issue-386-js-web-navigation.mjs`) proves the worker reproduces
  the canonical surface set (16 prefix + 25 bare http_fetch forms, 45 prefix + 27
  bare url_navigate forms), routes 83 English/Russian probes byte-identically to
  the pre-conversion logic through the real URL gate and fetch-before-navigate
  precedence, and adds the Hindi/Chinese coverage the old arrays lacked (issue
  #386).
- Web-search request recognition (`src/solver_handlers/web_search_intent.rs` and
  its `formal_ai_worker.js` mirror) is data-driven too — the deepest of the web
  clusters. Four self-describing seed files carry every surface the recogniser
  used to hardcode in seventeen inline arrays: `meanings-web-search.lino` (the
  `web_search_concept` backbone plus the `web_search_action`/`_strong_action`/
  `_signal`/`_source_only`/`_imperative_lead` roles),
  `meanings-web-search-query.lino` (`web_search_explicit_prefix`, the
  `web_search_topic_marker` whose prefix and suffix forms split into the
  before/after topic markers, and the leading/trailing query-noise roles),
  `meanings-web-research.lino` (the `research_question_opener`/
  `_superlative_modifier`/`research_evidence_domain`/`research_evaluation_domain`
  and `enumeration_request_opener`/`enumeration_constraint` roles), and
  `meanings-web-followup.lino` (`followup_instruction_verb`,
  `clause_continuation_marker`) — each `defined_by` the concepts it builds on and
  lexicalised in every supported language. As in the how- and navigation-clusters
  each surface marks its query slot with the ellipsis marker `…` (U+2026), so the
  recogniser buckets a role's forms by `WordForm::slot()` and matches prefixes,
  suffixes, and bare phrases by position. A single `WebSearchMarkers` projection
  (an 18-field struct on the Rust side, `webSearchMarkers()` memoised on the
  worker) gathers the seventeen roles once — `web_search_topic_marker` feeding two
  fields — and every detector (explicit-prefix stripping, semantic-action
  extraction, enumeration-research and implicit-research-question gating,
  source-only removal, and follow-up-clause truncation) reads from it instead of a
  hardcoded array, so the code knows only the concepts "a web search", "its
  query", "a research question", and "a follow-up instruction". Follow-up
  truncation is now a single universal-boundary routine
  (`truncate_search_instruction_tail`) that cuts the query at the first
  `followup_instruction_verb` lying on a token or sentence boundary in any
  language, replacing the per-language tail heuristics. A new
  `is_personal_fact_filter_request` guard suppresses web search when the prompt
  asks about the user's own contributed facts ("facts I have contributed", "my
  facts"), fixing a leak where the pre-conversion worker returned a bogus
  `{query:"my"}` search for "search my facts". Because the markers now exist in
  every language, source-marker queries ("Find apple on the internet" / "Найди
  яблоко в интернете" / "सेब के बारे में इंटरनेट पर खोजो" / "查找苹果网上信息"),
  enumeration-research, and implicit research questions resolve in Hindi and
  Chinese as well as English and Russian. A parity harness
  (`experiments/issue-386-js-web-search.mjs`) proves the worker reproduces all
  seventeen role word-sets from the seed, exposes the eighteen-field marker
  projection memoised, reproduces a frozen 33-prompt golden of pre-conversion
  behaviour byte-identically, and matches the Rust handler's multilingual
  source-marker, enumeration, implicit-research, and follow-up-drop cases — 78
  assertions, all green (issue #386).
- Every meaning now descends from a single ontology root, so the lexicon is one
  connected graph rather than disjoint clusters. A new backbone
  (`data/seed/meanings-ontology.lino`) defines `link` as the self-rooted root of
  the merged ontology (the relative-meta-logic "everything is a link" stance),
  `type` as a type-system sub-root directly under it, and
  `entity`/`concept`/`relation`/`action`/`property` as the top-level categories
  every domain genus roots in. Each existing cluster gains a `defined_by` edge up
  into one of these categories (`program` → `entity`, `sort`/`modify` →
  `action`, `quantity` → `property`, `calendar_day` → `concept`,
  `knowledge_relation` → `relation`, the software-project genera → their
  categories, …), so following `defined_by` from any meaning reaches
  `link`. A public ontology-reasoning API (`Lexicon::ontology_root`,
  `Lexicon::reaches_root`) and two invariants
  (`the_ontology_has_a_single_link_root`, `every_meaning_reaches_the_link_root`)
  enforce it; the `formal_ai_worker.js` mirror carries the same backbone and the
  parity harness proves the worker forms one connected ontology under the single
  `link` root (issue #386).
- Self-awareness known-facts recognition (`src/solver_handlers/self_awareness.rs`
  and its `formal_ai_worker.js` mirror) is data-driven too. The "facts" noun, the
  enumerating interrogatives (what/which/list/show), the second-person attribution
  of knowing (you know / you have / тебе известно / 你知道 / …), and the complete
  standalone phrasings that ask what the assistant knows now live as
  self-describing meanings in `data/seed/meanings-intent.lino` — the shared `fact`
  noun (reused through its `knowledge` definition rather than duplicated) plus the
  new `knowledge_inventory_probe`, `assistant_knowing`, and
  `knowledge_inventory_query` meanings, each `defined_by` the
  `knowledge`/`inquiry`/`fact` concepts and lexicalised in every supported
  language. `is_known_fact_query` now composes four semantic roles —
  `knowledge_inventory_noun` ∧ `knowledge_inventory_interrogative` ∧
  `knowledge_possession`, or the standalone `knowledge_inventory_phrase` — with one
  universal algorithm for every language instead of four per-language word
  conjunctions. Two deliberate consistency refinements follow: Chinese now also
  requires an explicit second-person marker (你知道/您知道/你有/您有), so a bare
  noun-only "哪些事实" falls through exactly as the English "which facts" does; and
  the Russian noun matches clean citation forms (факт/факты) at token boundaries
  like every other lexicon noun, rather than the former stem-fragment
  `.contains("факт")`. `self_awareness_language` now detects the language purely by
  Unicode script range (the Cyrillic range subsumes the former hardcoded
  second-person pronoun list), and the now-unused `contains_any` helper was
  removed (issue #386).
- Conversation-summary recognition (`try_summarize_conversation` in
  `src/solver_handlers/mod.rs` and its `formal_ai_worker.js` mirror) is
  data-driven too. Four self-describing meanings in
  `data/seed/meanings-intent.lino` carry every surface the recogniser used to
  hardcode in an English exact-set, a fifteen-entry prefix set, and three
  per-language anchored regexes: `conversation_summary_directive` (the summarize
  / суммируй / резюме / सारांश / 总结 verb), `conversation_reference` (the
  conversation / беседа / बातचीत / 对话 noun the directive can take as object),
  `conversation_summary_phrase` (complete standalone phrasings such as "summarize
  so far", "what have we talked about", "о чём мы разговаривали"), and
  `conversation_summary_courtesy` (objectless courtesy frames such as "can you
  summarize", "подведи итог", "सार दो"), each `defined_by` the `inquiry` concept
  and the summary concepts it builds on, and lexicalised in every supported
  language. `asks_for_conversation_summary` now composes those roles with one
  universal algorithm for every language — a standalone phrase, a courtesy frame,
  a directive together with a conversation reference, or a bare directive (the
  whole prompt for whitespace-delimited scripts, a leading directive for CJK) —
  instead of the former English exact-set / prefix lists and the
  Russian/Hindi/Chinese anchored regexes. Two refinements follow from reasoning
  over the concept rather than the raw words: the CJK bare directive now anchors
  at the start (`总结…`) so a compound like "工作总结" (a *work* summary) no longer
  mis-triggers — fixing a Rust `.contains("总结")` bug that the worker's `^总结`
  regex never had — and the directive-plus-reference conjunction recognises any
  conversation reference ("summarize our discussion", "резюме разговора"), not
  only the handful of "the/this/our conversation/chat" prefixes the worker
  enumerated. The generic `words_for_role` accessor the bare-directive check uses
  is now named identically on both sides (the worker's misnamed-but-generic
  `calendarWordsForRole`, already used for non-calendar roles, was renamed
  `wordsForRole`). The `formal_ai_worker.js` mirror queries the same embedded
  meanings, and a parity harness
  (`experiments/issue-386-worker-summarize-parity.mjs`) proves the recogniser
  fires on nineteen multilingual phrasings across all four composition arms,
  rejects content-summary and unrelated prompts, routes the four pinned
  with-history cases to `summarize_conversation`, and honours the empty-history
  turn gate (issue #386).
- The remaining user-intent recognisers (`src/solver_handlers/user_intent.rs`
  and its `formal_ai_worker.js` mirror) are data-driven too — proof requests,
  who-is questions, and the prior-turn web-search signal. A new self-describing
  seed file (`data/seed/meanings-proof.lino`) defines five meanings: `prove`
  (carrying both the clause-initial `proof_directive` bare verbs — prove / proof
  / докажи / доказать / … — and the `proof_claim_scaffold` prefixes that strip
  the claim out of "prove that …" / "докажи что …" / "साबित करो कि …" / "证明…",
  separated by slot within the one meaning), `proof_request_frame` (the English
  `proof_request_lead` frames that need no *that* clause — "can you prove …",
  "give me a proof of …"), `proof_assertion` (the mid-prompt `proof_marker`
  substrings in every language), and the `godel` / `determinism` proof concepts
  (`proof_concept_godel` / `proof_concept_determinism`); the who-is surfaces move
  into a `who_is_question` meaning in `data/seed/meanings-intent.lino` (the
  head-initial `who_question_lead` prefix — "who is …", "кто такой …" — and the
  head-final `who_question_tail` suffix — "… कौन है", "…是谁"); and the
  prior-turn signal becomes a `web_search_mention` meaning in
  `data/seed/meanings-web-search.lino` carrying the raw `web_search_history_signal`
  substrings. `is_proof_request`, `extract_claim_from_prompt`, `is_who_question`,
  the Goedel/determinism guards, and `prior_history_mentions_web_search` now ask
  the lexicon for those roles — bucketing each role's forms by `WordForm::slot()`
  so the clause-initial verb-boundary check, the first-matching-prefix claim
  extraction, and the head-initial/head-final who-is split are all derived from
  the data — instead of the former hardcoded per-language word arrays; the four
  generic affix helpers shared with the web-search cluster
  (`search{Prefix,Suffix,Bare,Source}Literals`) are renamed to the
  universal `{prefix,suffix,bare,source}Literals` now that proof and who-is reuse
  them. Reasoning over the concept also unified the Rust proof-marker behaviour
  with the worker's (it gained three Russian mid-sentence markers it had lacked),
  with no test regressing. A parity harness
  (`experiments/issue-386-worker-user-intent-parity.mjs`) loads the committed
  baseline and the working-tree worker into separate sandboxes and proves the
  four recognisers return byte-identical results across a 50-prompt multilingual
  matrix — including the prover/proven/improve/approve boundary negatives and
  claim extraction with leading noise — 221 assertions, all green (issue #386).
- The prefilled "Report issue" body omits settings already at their shipped
  default (Mode, Status, Diagnostics, Theme, Guess/Follow-up probability,
  Temperature, inference-only Location), folds the worker into the version line
  (`<version> (wasm)`), shortens the attach-memory section to a docs pointer, and
  drops the Reasoning Trace when the dialog was trimmed to fit GitHub's URL cap
  (issue #386).
- Documented the issue #386 case study (`docs/case-studies/issue-386/`) with raw
  data, a reconstructed timeline, the full requirements list, a corrected
  root-cause analysis of the "Отмени сортировку" refusal, and the implemented
  inverse-derivation fix.
- Every meaning in the lexicon now lexicalises *all* supported languages
  (en/ru/hi/zh), enforced unconditionally by the
  `every_meaning_covers_all_supported_languages` invariant. The two remaining
  English-/Russian-only meanings were backfilled with genuine surfaces: the
  broad proof request-frame (`proof_request_frame`, role `proof_request_lead`)
  gained Russian, Hindi and Chinese leads — each embedding an existing
  `proof_marker` substring (доказать / साबित / 证明 …) so recognition stays
  behaviour-neutral while the request-frame concept is complete in every
  language — and the prior-turn web-search signal (`web_search_mention`, role
  `web_search_history_signal`) gained Hindi and Chinese surfaces. A
  language-coverage audit (`experiments/issue-386-audit-language-coverage.mjs`)
  and the 221-assertion parity harness confirm the backfill leaves every
  recogniser byte-identical to its pre-backfill behaviour (issue #386).
- The policy and edge-case handlers (`src/solver_handlers_policy.rs`, the
  `is_inappropriate_content` screen in `src/solver_helpers.rs`, and the
  `formal_ai_worker.js` mirror) are data-driven too. A new seed file
  (`data/seed/meanings-policy.lino`) defines three self-describing meanings, each
  rooted in the `link` ontology and lexicalised in every supported language:
  `physical_action_query` (role `physical_action_trigger` — the crude "did you
  …" taunt the assistant answers factually because it has no physical body),
  `circular_joke_idiom` (role `circular_joke_phrase` — «купи слона» and its
  buy-an-elephant calque), and `vulgar_content` (role `vulgar_content_marker` —
  the English profanity and Russian mat migrated verbatim from the old hardcoded
  refusal lists, plus Hindi and Chinese equivalents). `try_physical_action_question`,
  `try_kupi_slona`, and `is_inappropriate_content` now ask the lexicon for those
  roles as raw substrings instead of carrying inline word arrays, so the code
  knows only the concepts while the surfaces live once in data; the physical-
  action and buy-elephant replies localise through `seed::response_for`. Because
  the idiom is now lexicalised everywhere, the buy-an-elephant calque routes to
  the same handler in every language, and the content screen generalises to
  Hindi and Chinese obscenities it never covered before. A vm parity harness
  (`experiments/issue-386-js-policy.mjs`) proves the worker's buy-elephant
  recogniser and its embedded policy lexicon — including the
  Rust-only `vulgar_content_marker` and `physical_action_trigger` roles — stay on
  par across all four languages (issue #386).
- The currency rate-basis handler (`src/solver_handlers/calculator_rate.rs` and
  its `formal_ai_worker.js` mirror) is data-driven too. A new seed file
  (`data/seed/meanings-calculator.lino`) defines four self-describing meanings,
  each rooted in the `link` ontology and lexicalised in every supported language:
  a `money` genus (`defined_by` `concept`, role `monetary_concept` — structural
  only, no handler queries it) that groups the currency meanings so they build
  from a shared concept, the `exchange_rate` between currencies (`defined_by`
  `money` + `relation`, role `exchange_rate_reference`), the `us_dollar` currency
  (`defined_by` `money`, role `currency_usd_reference` — including the two common
  Russian misspellings долар/долор), and the `calculation_basis` question frame
  (`defined_by` `action` + `inquiry`, role `calculation_basis_reference` — the
  "do you use … for calculations" / "у тебя … при расчётах" side of the prompt).
  `asks_for_usd_rate_basis` now composes the three queried roles as raw substrings
  via `Lexicon::mentions_role_raw` — an `exchange_rate_reference` *and* a
  `currency_usd_reference` *and* a `calculation_basis_reference` — instead of the
  former three hardcoded per-language `contains` disjunctions, so the code knows
  only the concepts while every surface lives once in data. The migration is
  byte-faithful: the role surface sets equal the original recognizer lists exactly
  (the worker even gains the "calculations" plural the Rust list always carried), so
  the USD/RUB delegation is behaviour-neutral. A vm parity harness
  (`experiments/issue-386-js-calculator-rate.mjs`) proves the worker routes the
  five spec prompts to the calculator in all four languages, falls through on
  currency prompts that miss one of the three concepts, and reproduces every role's
  surface set byte-for-byte across en/ru/hi/zh (issue #386).
- Natural-language skill recognition (`src/skill_compiler.rs`, its
  `structured.rs` permission/determinism screens, and the
  `formal_ai_worker.js` mirror) is data-driven too. The trigger leads
  ("when i say", "when the user says/asks", "if i ask"), the response verbs
  (answer/reply/respond, the Russian stem "ответ", …), the standalone
  behaviour-rule edit directives, and the conditional when-then frames now live
  as self-describing meanings in `data/seed/meanings-skill-compiler.lino` — each
  `defined_by` the concepts it builds on (a trigger lead `defined_by` `relation`
  + `inquiry`, a when-then frame `defined_by` `relation` + `concept`, …) and
  lexicalised in every supported language. The when-then frames are stored as
  `Slot::Circumfix` word forms whose literal before the ellipsis … (U+2026) is
  the head clause and whose literal after it is the link clause, so the head/link
  keyword pairs that were hardcoded in both runtimes now live once in data.
  `looks_like_skill_description` and `explicit_teaching_form` query the
  `skill_teaching_trigger_lead`, `skill_teaching_response_verb`,
  `behavior_rule_edit_directive`, and `skill_when_then_pair` roles via
  `Lexicon::mentions_role_raw`/`role_word_forms` instead of the former inline
  string lists and the `WHEN_THEN_KEYWORD_PAIRS` table. The structured-skill
  determinism screen and the implicit-capability inference likewise read the
  `nondeterministic_marker`, `shell_capability_cue`, and `network_capability_cue`
  roles (shell checked before network so a step touching both is attributed to
  the shell), while the formal `tool:local_shell`/`tool:web_fetch` identifiers
  stay in code as a tool-namespace bridge. Because the surfaces now cover every
  language uniformly, the browser worker gains the trigger leads it used to miss
  ("when the user says"/"when the user asks") and the "respond" verb. A 28-case
  truth table is shared, case for case, between the Rust inline test
  (`skill_description_recogniser_reads_every_language_from_the_lexicon`) and a vm
  parity harness (`experiments/issue-386-worker-skill-trigger-parity.mjs`) so the
  two runtimes are proven to agree across en/ru/hi/zh (issue #386).
- Natural-language tool/API recognition
  (`src/solver_handlers/natural_language_tools.rs`) is data-driven too. A new seed
  file (`data/seed/meanings-tool-access.lino`) defines five self-describing
  meanings, each rooted in the `link` ontology and lexicalised in every supported
  language: `tool_invocation_cue` (role `tool_invocation_cue` — the call / invoke /
  run / api / tool surfaces, `defined_by` `action`), `calculator_tool` (role
  `calculator_tool_name`, `defined_by` `entity`), `web_search_tool` (role
  `web_search_tool_name`, including the `web_search`/`web search`/`web-search`
  spellings, `defined_by` `entity`), `local_shell_tool` (role
  `local_shell_request_cue` — the whole request phrases such as "local shell tool"
  and "invoke the shell tool", which bundle verb and tool name so the cue is
  decisive on its own, `defined_by` `entity`), and `tool_argument_marker` (role
  `tool_argument_marker` — the "with query" / "query" / "with" / "for" argument
  introducers, `defined_by` `relation`). `is_explicit_tool_api_request` now asks
  the lexicon whether a prompt evidences a named tool together with a
  `tool_invocation_cue`, `is_explicit_local_shell_request` asks for the
  `local_shell_request_cue` alone, and the fallback argument extractor walks the
  English `tool_argument_marker` forms in declaration (priority) order, so the
  former hardcoded alias slices, the space-padded cue substrings (`" api"`,
  `"call "`, …), the local-shell phrase list, and the four `after_marker` calls
  are gone — the code knows only the concepts "an explicit tool call", "the named
  calculator/web-search/shell tool", and "the phrase that introduces a tool
  argument". Matching is token-bounded (the CJK-substring / whole-token contract),
  so a cue like "tool" no longer matches inside a larger word; the English forms
  drive the argument heuristic while the other languages stay in the seed for
  self-description. The handler is Rust-only — the browser worker has no
  natural-language-tool route — but its embedded `MEANINGS_LINO` mirrors the new
  file byte-identically so the shared knowledge base stays complete (issue #386).
- Feature-capability recognition (`src/solver_handlers/feature_capability.rs` and
  its `formal_ai_worker.js` mirror) is data-driven too. A new seed file
  (`data/seed/meanings-feature-capability.lino`) defines nineteen self-describing
  meanings, each rooted in the `link` ontology and lexicalised in every supported
  language: sixteen `feature_capability_*` alias meanings (role
  `feature_capability_alias`, `defined_by` `concept` — the surface words that name
  each of the sixteen advertised features: web search, diagnostics, agent mode,
  definition fusion, configuration, memory actions, greeting, write program,
  concept lookup, arithmetic, translation, memory, demo mode, http url, javascript
  execution, planning), the `feature_capability_question` frame (role
  `feature_capability_question`, `defined_by` `action`, grounded in *can* — the
  "can you …" / "do you support …" / "умеешь ли …" / "你能…" availability question
  the recogniser keys on), and the two action-request gates
  `feature_action_arithmetic` and `feature_action_planning` (roles
  `feature_action_arithmetic`/`feature_action_planning`, `defined_by` `action` —
  the imperative "can you calculate …" / "can you summarize …" frames that must
  route to the live arithmetic and planning handlers, not the capability answer).
  `detect_feature_capability`, `is_feature_capability_question`, and
  `is_feature_action_request` now ask the lexicon for those roles — resolving a
  matched `feature_capability_alias` meaning back to its stable slug, gating on the
  `feature_capability_question` frame (with the pre-existing English
  "is/are … enabled/available" availability shape kept as a structural fallback),
  and stepping aside when an arithmetic/planning *action* frame leads the prompt —
  instead of the former per-feature alias arrays and the hardcoded
  `WEB_SEARCH_CAPABILITY_PHRASES` / `featureAliases` lists, so the code knows only
  the concepts "a feature", "a question about whether a feature is available", and
  "an imperative that should run the feature instead". The migration is
  byte-faithful to the alias/action split that origin/main relied on: the
  arithmetic *alias* set stays `arithmetic` / `calculate` / `math` / `2 + 2`
  (Russian `арифмет` / `считать` / `посчитать`, …) while `compute` lives only in
  the `feature_action_arithmetic` *action* role, so a bare "Can you compute 7 * 6?"
  detects no capability and falls through to the calculation handler exactly as
  before. Because the alias words now exist in every language, feature-availability
  questions resolve uniformly across en/ru/hi/zh. A vm parity harness
  (`experiments/issue-386-js-feature-capability.mjs`) replays the sixty
  feature×language rows and twenty web-search probes from the Rust battery in
  `tests/unit/specification/capabilities.rs`, the arithmetic/planning action gates,
  the alias/action role separation, and the availability-frame fallback — 95
  assertions, all green — proving the worker's recogniser agrees with the Rust
  solver in every language (issue #386).
- The Playwright starter-script recogniser
  (`src/solver_handlers/playwright_script.rs` and its `formal_ai_worker.js`
  mirror) is data-driven too. A new seed file (`data/seed/meanings-playwright.lino`)
  defines two self-describing meanings rooted in the `link` ontology and
  lexicalised in every supported language: `playwright` (role
  `playwright_tool_name`, `defined_by` `entity` — the tool name plus its common
  misspelling, whose `playright` word form carries `action "playwright"` naming
  the canonical spelling) and `playwright_script_request_cue` (role
  `playwright_script_cue`, `defined_by` `concept` — the script-authoring cues
  *script* / *test* / *spec* / *code* / *write* / *create* / *generate* / *make* /
  *build* / "can you" / "could you" and their ru/hi/zh equivalents).
  `is_playwright_script_request` now gates on both roles via `mentions_role_raw`
  (raw substring, byte-faithful to the former two `contains` pairs), and
  `mentions_playwright_misspelling` resolves the misspelled form by its `action`,
  so the handler still reports the `Playright -> Playwright` correction without
  naming either spelling in code — the code knows only "the Playwright tool" and
  "a request to author a script for it".
- Research comparison-table recognition (`src/solver_handlers/research_table.rs`
  and its `formal_ai_worker.js` mirror) is data-driven too. A new seed file
  (`data/seed/meanings-research-table.lino`) defines eight self-describing
  meanings rooted in the `link` ontology and lexicalised in every supported
  language: the comparison trigger `compare` (role `comparison_table_trigger`),
  the weak pair `table` (role `comparison_table_noun`) and `differences` (role
  `comparison_difference_cue`), the `research_prompt_signal` meaning (role
  `research_prompt_signal` — bare markers like 'web search' / 'research' plus
  prefix surfaces 'search …' / 'find information …' whose `…` slot the code reads
  as a `before_slot` opener), and four `research_criterion` meanings declared in
  column order (`key_differences` / `use_cases` / `advantages` / `disadvantages`).
  `is_comparison_table_request`, `looks_like_research_prompt`, and
  `append_criteria_from_text` now ask the lexicon for those roles, and a new
  `Criterion::from_slug` keys each column off the matched meaning's slug — so the
  code names only the language-independent slug, never a surface word. Declaration
  order fixes the K/U/A/D column order, and the space-guarded criterion stems
  `pro ` and ` con ` keep their surrounding spaces (the criterion match stays a
  raw `contains`) so they never match inside *process* / *control*. Because the
  comparison gate is now token-bounded across en/ru/hi/zh, the apple-on-the-internet
  web-search prompts (which name no comparison/table/difference surface in any
  language) still route to web search rather than tripping the table follow-up.
  The Rust lib and unit suites stay green — including the Playwright
  clarification/starter and research comparison-table specifications — and the
  worker's inline `MEANINGS_LINO` was regenerated so the mirror verifier
  (`experiments/issue-386-meanings-mirror.mjs`) reports byte-identical parity
  across all twenty-eight meaning files (issue #386).
- Conversational-opener topic extraction (`conversation_topic` in
  `src/solver_handlers/benchmark_prompts.rs` and its `formal_ai_worker.js`
  mirror) is data-driven too. A new seed file
  (`data/seed/meanings-conversation.lino`) defines the
  `conversation_topic_opener` meaning (role `conversation_topic_opener`,
  `defined_by` the `inquiry` and `action` concepts) rooted in the `link`
  ontology and lexicalised in every supported language — the let-us-talk-about-X
  phrasings ("let's talk about …", "давай поговорим о …", "चलो बात करें …",
  "聊聊…"). Each surface marks the topic position with the ellipsis `…` (U+2026)
  slot marker, so the recogniser walks the role's prefix forms in declaration
  order and strips the opener via `before_slot()` instead of the former
  fifteen-entry per-language prefix array. The single surface whose `action` is
  `scan` ("поговорим о …") is additionally matched anywhere in the prompt,
  preserving the old `split_once` fallback that catches an opener following a
  greeting — so the code knows only the concept "an opener that proposes a
  topic" while the surfaces live once in data. The `formal_ai_worker.js` mirror
  queries the same embedded meanings and the mirror verifier
  (`experiments/issue-386-meanings-mirror.mjs`) reports byte-identical parity
  across all twenty-nine meaning files (issue #386).
- Software-project follow-up output extraction (`extract_expected_output` in
  `src/solver_handlers/software_project_followup.rs`) is data-driven too. A new
  `output_display_request` meaning in `data/seed/meanings-software-project.lino`
  (role `output_display_request`, `defined_by` the `software_followup` and
  `action` concepts) carries the show-me/print/display openers that name what
  the user wants surfaced ("show me …", "show …", "print …", "display …", plus
  ru/hi/zh surfaces such as "покажи мне …" / "मुझे दिखाओ …" / "给我看…"). Each
  surface marks the output position with the ellipsis `…` (U+2026) slot marker,
  so the handler walks the role's forms in declaration order — the longer "show
  me " tried before the bare "show " — strips the opener via `before_slot()`,
  and reads the clause that follows from the original-case prompt (stopped at
  the first sentence-ending punctuation, capped at twelve words) instead of the
  former hardcoded four-marker array. The opener is still matched anywhere in
  the prompt, so "test it and show me the result" keeps capturing "the result".
  Because the openers now exist in every language, a Hindi or Chinese follow-up
  records its expected output where the English-only array recognised nothing.
  The extractor is Rust-only — the browser worker has no follow-up output route
  — but its embedded `MEANINGS_LINO` mirrors the new meaning byte-identically so
  the shared knowledge base stays complete, with the mirror verifier reporting
  parity across all twenty-nine meaning files (issue #386).
- The mechanism-inquiry subject cleanup (`strip_mechanism_tail` /
  `clean_mechanism_subject` in `src/solver_handler_how.rs` and the
  `formal_ai_worker.js` mirror) is data-driven too — completing the how-cluster
  conversion. Three new self-describing meanings in `data/seed/meanings-how.lino`
  carry the surfaces these helpers used to hardcode in three inline arrays:
  `mechanism_predicate` (role `mechanism_predicate`, `defined_by` `action` +
  `mechanism_inquiry` — the "… work" / "… works" / "… structured" / … predicate
  tails a prefix match leaves behind), `detail_modifier` (role `detail_modifier`,
  `defined_by` `property` + `mechanism_inquiry` — the "… in detail" / "…
  internally" / "… please" / … thoroughness-or-politeness tails), and
  `non_referential_subject` (role `non_referential_subject`, `defined_by`
  `entity` + `mechanism_inquiry` — the pronouns and dangling function words "it" /
  "this" / "does …" / "to …" / … that name no real topic). As in the rest of the
  cluster each surface marks its slot with the ellipsis `…` (U+2026): the predicate
  and detail tails are suffix surfaces whose text after the slot is the literal to
  strip (tried in declaration order — the predicate by return-on-first-match, the
  modifiers stripped together in one re-trimming pass), while the reject set mixes
  bare surfaces matched against the whole candidate with prefix surfaces matched
  against its start. `strip_mechanism_tail` walks the `mechanism_predicate` role
  and `clean_mechanism_subject` walks `detail_modifier` then
  `non_referential_subject`, so the former `[" work", …]` / `[" in detail", …]`
  arrays and the nineteen-entry `PRONOUN_SUBJECTS` set with its four `starts_with`
  checks are gone — the code knows only the concepts "the predicate that completes
  a how-it-works clause", "an optional detail modifier", and "a subject that names
  no real topic". Because the surfaces now cover every language, the ru/hi/zh
  predicate tails and the hi/zh detail modifiers are stripped where the
  English-/Russian-only arrays left them intact, while every English and Russian
  case stays byte-identical. The `formal_ai_worker.js` mirror drives
  `cleanMechanismSubject` / `stripMechanismTail` from the same embedded meanings,
  and a differential parity harness
  (`experiments/issue-386-js-mechanism-subject.mjs`) reconstructs the
  pre-conversion arrays and proves the two functions are byte-identical to them
  across forty-nine English/Russian probes, documents the ten intended
  all-language generalizations, and confirms the issue-#386 reasoning paths — the
  mirror verifier reporting parity across all twenty-nine meaning files (issue
  #386).
- The procedural-task cleanup (`clean_procedural_fragment` /
  `correct_common_procedural_typos` in `src/solver_handler_how.rs` and the
  `formal_ai_worker.js` mirror) is data-driven too — finishing the how-cluster
  conversion. Two new self-describing meanings in `data/seed/meanings-how.lino`
  carry the surfaces these helpers used to hardcode in a seventeen-entry suffix
  array and a single-entry typo table: `procedural_task_modifier` (role
  `procedural_task_modifier`, `defined_by` `property` + `procedural_request` —
  the trailing "step by step" / "in steps" / "for me" / "please" / … and their
  ru/hi/zh equivalents that a procedural extractor strips from the end of the
  task) and `common_typo` (role `common_typo`, `defined_by` `relation` — a
  misspelling paired with its correction, the canonical case being the
  transposed "dirven" -> "driven"). Each modifier surface is a `Slot::Suffix`
  whose text after the ellipsis `…` (U+2026) is the literal tail to strip, walked
  in declaration order with the first match winning, so the longer Russian
  "напиши по шагам" is still tried before its "по шагам" tail; each typo surface
  is a `Slot::Bare` whose `action` child names the correct spelling, so a task
  token is repaired by data rather than a hardcoded `token == "dirven"` check.
  The former inline suffix array and the one-branch typo table are gone — the
  code knows only the concepts "a trailing step-by-step or politeness modifier"
  and "a misspelling and its correction". Because the surfaces now cover every
  language, the genuine ru/hi/zh typos (руский -> русский, वेबसाईट -> वेबसाइट,
  登陆 -> 登录) are repaired where the English-only table did nothing, while every
  pre-existing case stays byte-identical. A differential parity harness
  (`experiments/issue-386-js-procedural-cluster.mjs`) reconstructs the
  pre-conversion suffix array and typo table, proves the two functions are
  byte-identical to them across thirty-two cleanup probes and six typo probes
  (all four languages, order-sensitivity, punctuation and whitespace controls),
  documents the three intended all-language typo generalizations, and confirms
  the four issue-#343 spec-driven prompts still reduce to "spec driven
  development" with a recorded dirven->driven fix — the mirror verifier reporting
  parity across all twenty-nine meaning files (issue #386).
- The prior-reply topic scan (`extract_topic_from_prior_reply` in
  `src/solver_handler_how.rs`) is data-driven too — closing out the how-cluster
  conversion. When a "how does it work?" follow-up names no subject and the prior
  assistant reply has no "Term (category):" header, the handler falls back to the
  first capitalised token that is not a function word. That skip list was a
  hardcoded English title-case array
  (`["I", "The", "A", "An", "In", "To", "For", "Of", "And", "Or", "Source"]`); it
  now lives as a self-describing `topic_scan_stop_word` meaning in
  `data/seed/meanings-how.lino` (role `topic_scan_stop_word`, `defined_by` the
  `concept` category — closed-class articles, prepositions, conjunctions and
  pronouns plus the 'source' citation heading, lexicalised in every supported
  language). The handler walks the role's `Slot::Bare` forms and compares them
  case-insensitively, so the code knows only the concept "a function word that
  names no topic". The case-insensitive match is a strict superset of the former
  case-sensitive comparison: it reproduces the old behaviour for ordinary
  title-case prose and additionally skips all-caps English function words and
  capitalised Cyrillic ones that the English-only array left to be mis-read as the
  topic. The extractor is Rust-only — the browser worker has no prior-reply topic
  route — but its embedded `MEANINGS_LINO` mirrors the new meaning byte-identically
  so the shared knowledge base stays complete, and a regression test
  (`how_it_works_prior_reply_fallback_skips_function_words_case_insensitively`)
  pins the generalization while the mirror verifier reports parity across all
  twenty-nine meaning files (issue #386).
- Counting numbers are a self-describing ontology too, so the code reads counts
  from data instead of hardcoded number words. A new cardinal-number sub-graph
  in `data/seed/meanings-units.lino` defines the `cardinal_number` genus
  (`defined_by` the `quantity` property) and the leaves `zero`…`ten` (role
  `cardinal_number_word`, each `defined_by` `cardinal_number`), lexicalised in
  every supported language; each leaf's English lexeme carries both the spelled
  word ("ten") and the script-independent numeral surface ("10"), from which the
  cardinal's integer value is read. `contains_spelled_arithmetic`
  (`src/calculation.rs`) now asks the lexicon for the `cardinal_number_word`
  forms — skipping the pure-numeral surfaces the numeric parser already handles —
  instead of the former twenty-six-entry English/Russian number-word table, and
  the brainstorm-count recogniser (`requested_brainstorm_count` in
  `src/solver_handlers/benchmark_prompts.rs` and the worker's
  `requestedBrainstormCount`) derives the requested count from the `ten`
  cardinal's own numeral surface via a new `cardinal_value` / `cardinalValue`
  helper, replacing the hardcoded `TEN_HINTS` / `tenHints` literal. Matching is
  the boundary-aware lexicon contract, so the spelled "ten" no longer false-
  matches inside "often" and the Chinese 十 matches as a substring; because the
  cardinals now exist in every language, "придумай десять идей" / "दस नाम सुझाओ" /
  "给我十个想法" all resolve to ten where the former English-leaning table missed
  them. A new role `ROLE_CARDINAL_NUMBER_WORD` (`src/seed/roles.rs`) names the
  concept, and a vm parity harness
  (`experiments/issue-386-worker-brainstorm-count-parity.mjs`) proves the worker
  reads the count from the seed across the pinned English prompts, the
  multilingual cardinal cases, and the "often" substring negative (issue #386).
- The browser worker's `currencyCodeFromWord` no longer carries hand-written
  Russian declension tables for the dollar and ruble. It now walks the
  `currency_usd_reference`, `currency_eur_reference` and `currency_rub_reference`
  roles (`data/seed/meanings-calculator.lino`) and returns the ISO 4217 code of
  the first role a surface matches, with the canonical code mapped in a tiny
  `currencyCodeForRole` resolver (the output stays in code; only the recognition
  vocabulary lives in the seed). Matching follows each surface's script the same
  way `surfacePresent` already splits CJK from the rest: Latin and CJK/Devanagari
  surfaces match the whole token exactly — so unrelated words such as "rubbish"
  or "european" are rejected just as the original exact-match list rejected them
  — while Cyrillic surfaces are treated as stems and matched by prefix, so every
  Russian declension (доллар…, руб…) is recognised from the доллар / руб stems
  without enumerating each inflected form. A vm parity harness
  (`experiments/issue-386-worker-currency-code-parity.mjs`) proves the seed-driven
  walk returns byte-identical codes to the former tables across all 13 USD, 4 EUR
  and 14 RUB inputs, rejects the unrelated words, and still resolves the
  "1000 рублей в долларах" → USD capture pinned by the calculator delegation and
  multilingual e2e tests (issue #386).
- The calculator's spelled-operator detector (`contains_word_operator` in
  `src/calculation.rs`) no longer carries a 14-element array of operator words.
  The spelled operators now live as their own ontology: an `arithmetic_operation`
  genus (`defined_by "action"`) with five operations beneath it — `addition`,
  `subtraction`, `multiplication`, `division`, `modulo` — each `defined_by` that
  genus and carrying its operator surfaces in English, Russian, Hindi and Chinese
  (`data/seed/meanings-calculator.lino`). A new role
  `ROLE_ARITHMETIC_OPERATOR_WORD` (`src/seed/roles.rs`) marks those surfaces, and
  the detector reads them through `Lexicon::mentions_role`, which matches each
  operator as a whole token (and CJK surfaces as a substring) — the same boundary
  contract the former space-padded `.contains` checks enforced for the English and
  Russian operators, now extended to every language the meanings lexicalise. The
  pinned spelled-operator delegations ("two plus two", "nine multiplied by nine",
  "шесть умножить на семь", "the fifth Fibonacci number multiplied by 10", …) keep
  resolving through the existing Rust unit suite (issue #386).
- The calculator's request-cue stripper (`strip_calculation_wrappers` in
  `src/calculation.rs`) no longer carries a 28-element array of leading prompt
  cues. Those cues — imperatives like "calculate" / "посчитай" / "गणना करें" and
  question openers like "what is" / "сколько будет" / "请计算" — now live in a new
  `calculation_request` meaning (`defined_by "action"` and `defined_by "inquiry"`)
  with their surfaces in English, Russian, Hindi and Chinese
  (`data/seed/meanings-calculator.lino`). A new role
  `ROLE_CALCULATION_REQUEST_CUE` (`src/seed/roles.rs`) marks them, and the
  stripper reads them through `Lexicon::words_for_role`, rebuilding each surface
  into a strip prefix that follows its script: space-delimited scripts gain a
  trailing space so a cue strips only on a word boundary ("calculate" never eats
  the start of "calculated"), while CJK surfaces strip as-is because those scripts
  have no inter-word spaces. The Chinese cues are stored longest first, and
  `words_for_role` preserves declaration order, so a more specific cue strips
  before a shorter one it contains. The pinned prompt-stripping delegations keep
  resolving through the existing Rust unit suite (issue #386).
- The browser worker's calculation-signal recognizers (`src/web/formal_ai_worker.js`)
  now read the same meanings as the Rust solver instead of carrying their own
  literal arrays. `hasArithmeticWordOperator` reads `ROLE_ARITHMETIC_OPERATOR_WORD`
  through `lexiconMentionsRole` (dropping the 14-element `ARITHMETIC_WORD_OPERATORS`),
  `hasSpelledArithmetic` reads `ROLE_CARDINAL_NUMBER_WORD` through `roleWordForms`,
  skipping pure-numeral surfaces (dropping the 26-element `ARITHMETIC_NUMBER_WORDS`),
  and `extractArithmeticExpression` rebuilds its leading-cue prefixes from
  `ROLE_CALCULATION_REQUEST_CUE` via `wordsForRole` (dropping the 28-element prefix
  array). Each conversion is byte-faithful to the former arrays for every English
  and Russian case and additionally recognises the Hindi and Chinese surfaces the
  seed lexicalises, exactly mirroring the Rust `contains_word_operator` /
  `contains_spelled_arithmetic` / `strip_calculation_wrappers` changes. A new
  parity harness `experiments/issue-386-worker-calc-signal-parity.mjs` pins the
  equivalence (issue #386).
- The calculator's *trailing*-cue stripper no longer carries a hardcoded suffix
  list either — completing `strip_calculation_wrappers` (`src/calculation.rs`)
  and its worker twin `extractArithmeticExpression`
  (`src/web/formal_ai_worker.js`). The trailing cues a calculation prompt may
  carry split into two self-describing meanings in
  `data/seed/meanings-calculator.lino`: `calculation_result_query`
  (`defined_by` `action` + `inquiry`, role `calculation_result_query_cue` — the
  equals word or sign, the how-much-is-it question, and the head-final
  do-the-calculation imperative: equal / equals / = / равно / 是多少 / 等于多少 /
  等于几 / कितना है / क्या है / की गणना करें) and `politeness` (`defined_by`
  `property`, role `politeness_cue` — the courtesy tail that carries no task
  content: please / for me / пожалуйста / कृपया / 请). A new
  `calculation_wrapper_suffixes` helper walks the two roles via
  `Lexicon::words_for_role` and rebuilds each surface into a strip suffix
  following its script — CJK surfaces strip as-is (no inter-word spaces), a
  pure-symbol surface like the equals sign strips both bare and on a word
  boundary (so a compact `2*2+2=` is recognised), and every other surface gains
  a leading space so the cue strips only on a word boundary — replacing the
  former thirteen-element Rust array and the eleven-regex worker array (two new
  roles `ROLE_CALCULATION_RESULT_QUERY_CUE` / `ROLE_POLITENESS_CUE` in
  `src/seed/roles.rs` name the concepts). The conversion is byte-faithful to the
  former arrays for every English and Russian case and adds the bare-`=` strip
  to the worker so the two engines now agree on `2*2+2=` (the old worker left the
  sign in place); because the cues now exist in every language, the new ru
  `равно`, hi `कृपया` and zh `请` surfaces — needed for the every-language
  invariant — strip where the old arrays left them, while the Hindi cues gain a
  required leading space so they too strip only on a boundary. A vm parity
  harness (`experiments/issue-386-worker-calc-suffix-parity.mjs`) reconstructs
  the pre-conversion regexes and proves the worker is byte-identical to them
  across the English/Russian/Chinese cases, applies the bare-`=` consistency fix
  and the three multilingual generalizations, and composes prefix-plus-suffix
  stripping — all green (issue #386).
- The calculator router's currency-conversion exemption
  (`has_calculation_signal` in `src/calculation.rs`) no longer hardcodes a
  to/into/convert/exchange list. A prompt that pairs a currency symbol with
  letters but is not an explicit `calculate` command is otherwise treated as
  prose and rejected; a conversion is itself a calculation, so a conversion cue
  must exempt it. Those cues now live in a new self-describing
  `quantity_conversion` meaning (`defined_by` `action` + `relation`, role
  `quantity_conversion_cue`) in `data/seed/meanings-calculator.lino`,
  lexicalised in every supported language — the bare target markers to / into,
  the verbs convert / exchange, and their ru/hi/zh equivalents (конвертировать /
  обмен, बदलें / परिवर्तित, 转换 / 兑换). The guard reads them through
  `Lexicon::mentions_role` (a new role `ROLE_QUANTITY_CONVERSION_CUE` in
  `src/seed/roles.rs` names the concept), which matches each surface
  whole-token in space-delimited scripts — byte-faithful to the former
  `lower.contains(" to ")` so the markers to/into still count only on a word
  boundary, never inside another word — and as a substring in Chinese. This is a
  dedicated meaning rather than a reuse of `conversion_action` (the
  money-specific verb the compound-interest handler matches as a raw substring):
  adding the bare markers to/into there would match them everywhere, whereas the
  router's general conversion signal must stay whole-token. The exemption is
  strictly more permissive (it can only flip the prose-rejection guard from
  reject to accept), so adding the multilingual surfaces leaves every existing
  case byte-identical. The browser worker has no twin — its conversion rescue is
  the separate `evaluateCurrencyConversionExpression` — so only its embedded
  `MEANINGS_LINO` was re-synced byte-identically; a regression test
  (`calculator_currency_conversion_is_exempt_from_prose_rejection`) pins the
  English behaviour and the no-cue contrast (issue #386).
- The calculator router's known-domain-word gate (`has_calculation_signal` in
  `src/calculation.rs` and the `extractArithmeticExpression` gate in
  `src/web/formal_ai_worker.js`) no longer carries a hardcoded signal array — the
  62-entry Rust list and the worker's 39-entry list are both gone. The surfaces
  whose presence beside a number marks a prompt as a calculation now live as
  self-describing meanings read through three roles: `math_function_name` (a new
  `mathematical_function` genus in `data/seed/meanings-calculator.lino` with
  `square_root` / `sine` / `cosine` / `tangent` / `logarithm` /
  `natural_logarithm` beneath it, each lexicalised in every supported language),
  `calculation_domain_term` (carried by the currency meanings `us_dollar` /
  `euro` / `ruble` and the calculator-relevant measurement units `kilobyte` /
  `megabyte` / `kilogram` / `gram` / `ton` / `second` / `minute` / `hour` /
  `millisecond` / `day` / `month` in `data/seed/meanings-units.lino`, each still
  `defined_by` the dimension it measures), and the CJK members of the existing
  `quantity_conversion_cue`. A shared `calculator_domain_signals` helper (its
  worker twin `calculatorDomainSignals`, using a new `isAsciiText` mirror of
  Rust's `str::is_ascii`) shapes each surface by script: a `math_function_name`
  gains only a leading space so it still fires when glued to a parenthesis
  ("sqrt(16)"); a `calculation_domain_term` is matched whole-token (leading and
  trailing space) for ASCII so a short code never fires inside a longer word;
  non-ASCII surfaces in both roles match as raw substrings so every inflected
  form is caught; and only the CJK conversion verbs (转换 / 兑换 / 换成) become
  signals — the Latin to/into are far too common to mark a calculation on their
  own. Two new roles `ROLE_MATH_FUNCTION_NAME` / `ROLE_CALCULATION_DOMAIN_TERM`
  (`src/seed/roles.rs`) name the concepts, so the code knows only "a mathematical
  function" and "a calculator-domain term" while the words live once in data. The
  conversion is byte-faithful to the former arrays for the whole-token currency
  codes and units, and tightens the latent substring false positives the old
  leading-space-only word forms allowed (for example "euro" inside "european" and
  "dollar" inside "dollarized") now that ASCII domain terms match whole-token.
  Because the surfaces now cover every language uniformly, the worker gains the
  sin/cos/tan/log/ln math functions and the CJK/Devanagari unit surfaces it
  lacked, and both engines drop the Russian and Hindi month *names* (феврал /
  январ, फरवरी / जनवरी) the old arrays carried: those name calendar months, not
  durations, and a genuine date-difference calculation still carries a duration
  unit (месяцев / दिन / months) the domain-term role covers — while the Chinese
  month names stay recognised because 二月 / 一月 embed the 月 month-unit ideograph
  the role matches as a substring. A vm parity harness
  (`experiments/issue-386-worker-calc-signal-parity.mjs`) reconstructs the
  pre-conversion worker array and proves the new `calculatorDomainSignals` gate is
  byte-identical across the agreement cases, documents the eight intended
  differences (the month-name drops, the whole-token tightening, the
  math-function adds, and the Chinese unit adds), and confirms
  `extractArithmeticExpression` still routes "convert 10 tons to kg" and
  "300000 ms in seconds" to the calculator while rejecting digit-free CJK prose
  (issue #386).
- The arithmetic evaluator's spelled→symbolic rewrite (`normalize_expression` in
  `src/arithmetic.rs` and its worker twin `normalizeArithmeticWords` in
  `src/web/formal_ai_worker.js`) no longer carries a hardcoded word→value map —
  the Rust `ARITHMETIC_WORD_TOKENS`/phrase pairs and the worker's five phrase
  regexes plus its `ARITHMETIC_WORD_TOKENS` map are all gone. A spelled
  expression ("two plus three", "пять умножить на два", "पाँच गुणा दो") is
  rewritten into its symbolic form ("2 + 3", "5 * 2") from the seed: every
  `cardinal_number_word` and `arithmetic_operator_word` meaning carries its
  script-independent *value surface* — the word form with no alphabetic
  character, the numeral "2" for the cardinal two, the symbol "+" for addition —
  and each spelled surface maps onto it. The five operator meanings
  (`data/seed/meanings-calculator.lino`) gained that symbol word form so an
  operator is value-carrying exactly as a cardinal already was. A new
  `Lexicon::arithmetic_normalization_tables` (`src/seed/meanings.rs`) derives the
  `(tokens, phrases)` mapping — single words applied after tokenization, longest-
  first multi-word phrases applied before it so "разделить на" rewrites before
  the shorter "делить на" it contains — and `normalize_expression` folds the
  phrases then maps the tokens. Because `arithmetic.rs` is compiled into the wasm
  worker (`#![no_std]`, no `build.rs`) and cannot reach the seed at runtime, the
  table is materialized at author time into the `no_std` static
  `src/arithmetic_word_tables.rs` by `examples/issue_386_gen_arith_table.rs` and
  pinned to the live seed by the `arithmetic_word_tables_match_seed` test
  (`src/calculation.rs`), so a stale table fails CI. The worker's
  `arithmeticNormalizationTables()` derives the same mapping from its inline
  `MEANINGS_LINO`. The new symbol word forms are detection-neutral: a new
  `Lexicon::mentions_role_spelled` (worker `lexiconMentionsRoleSpelled`) skips
  value surfaces, so the spelled-operator gate (`contains_word_operator` /
  `hasArithmeticWordOperator`) still keys only on the alphabetic operator words
  and never treats a bare "+" as one. Because the mapping now spans every
  language the meanings lexicalise, the worker gains the Hindi space-separated
  ("पाँच गुणा दो" → "5 * 2") and the CJK/"по модулю" arithmetic the former
  English/Russian-only map lacked. A vm parity harness
  (`experiments/issue-386-worker-arith-normalize-parity.mjs`) proves the worker's
  derived tables equal the Rust-generated static entry-for-entry (67 tokens, 6
  phrases, order included) and that `normalizeArithmeticWords`/`evaluateArithmetic`
  agree on en/ru/hi golden cases — so all three representations (the two language
  builders and the materialized table) are proven identical (issue #386).
- The unknown-implementation-language extractor (`requested_program_language`
  in `src/intent_formalization.rs` and the `programLanguageFromPrompt` mirror in
  `formal_ai_worker.js`) no longer hardcodes the function words that introduce a
  language name. Two new self-describing meanings —
  `implementation_language_preposition` ("in"/"на") and
  `implementation_language_noun` ("language"/"языке"), in
  `data/seed/meanings-software-project.lino` — carry those surfaces in every
  supported language; the positional scan reads the head-initial English/Russian
  markers from the lexicon and returns the bare language name trailing them, so
  an unknown target such as "write a program in Brainfuck" still resolves with no
  literals in the parser. The catalog-driven resolution of *known* languages is
  unchanged; worker parity with the Rust extractor is proven entry-for-entry by
  `experiments/issue-386-worker-program-language-parity.mjs` (issue #386).
- The translation handler's define-in-Links-Notation gate (`try_translation`
  in `src/solver_handlers/mod.rs`) no longer keys on the literal verb `define `
  or the format phrases ` links notation` / ` в links`. Two new meanings —
  `definition_command` (the imperative verb) and `links_notation_format` (the
  target-format name), in `data/seed/meanings-translation.lino` — carry those
  surfaces in every supported language; the gate composes the head-initial
  English verb with a quoted or backticked phrase and an English/Russian format
  marker sourced from the lexicon, preserving the original recogniser exactly.
  The scanned surface set is locked by the lib test
  `define_in_links_roles_expose_the_scanned_surfaces` (`src/seed/meanings.rs`),
  and a dispatch-level test documents that `concept_lookup` answers these prompts
  first so the refactor changes nothing observable (issue #386).
- The worker's `N% of M <currency>` recognizer (`formal_ai_worker.js`) no
  longer hardcodes the `usd|eur|rub|dollars?|euros?|rubles?` alternation. A new
  cached `percentOfExpressionRegex()` builds the trailing-currency alternation
  from the same three currency-reference roles `currencyCodeFromWord` resolves,
  so the recognizer captures exactly the ISO codes and the English/Cyrillic/CJK
  /Devanagari names the resolver already understands — longest-first and
  regex-escaped. Parity with the resolver is proven by
  `experiments/issue-386-worker-percent-of-currency-parity.mjs` (issue #386).
- The intent classifier's question/statement split
  (`starts_with_question_word` in `src/intent_formalization.rs`) no longer
  hardcodes the fronted wh-words. A new self-describing meaning
  `interrogative_opener` (`data/seed/meanings-intent.lino`, rooted through
  `inquiry` to the `link` ontology root) carries the opener surfaces in every
  supported language, each word form described; the classifier reads the
  English and Russian openers from the lexicon and prefix-matches them with a
  trailing space, exactly reproducing the previous recogniser. English and
  Russian are head-initial so the opener fronts the prompt and is consulted
  positionally; the head-final Hindi and Chinese surfaces are carried for
  coverage but not matched at the front. The scanned surface set is locked by
  the lib test `interrogative_opener_role_exposes_head_initial_question_words`
  (`src/seed/meanings.rs`) and the behaviour by
  `fronted_interrogative_opener_classifies_prompt_as_question`
  (`tests/unit/specification/intent_formalization.rs`); the worker
  `MEANINGS_LINO` mirror was re-synced byte-identical (issue #386).
- The ru→en compositional translator (`src/translation/pipeline.rs` and its
  `formal_ai_worker.js` mirror) is data-driven too. Its lexicon was four
  hardcoded dictionaries — phrase fallbacks, per-word lemma fallbacks,
  genitive-governing relation heads, and the single genitive-tagged complement
  (`RU_EN_PHRASE_FALLBACKS` / `RU_EN_WORD_FALLBACKS` /
  `RU_EN_GENITIVE_RELATION_HEADS` / `RU_EN_GENITIVE_NOUN_FALLBACKS` on the worker
  side). Those surfaces now live as ten self-describing meanings in
  `data/seed/meanings-translation.lino`: seven compositional lemmas (apple, good,
  find, synonym, example, agreement, conjunction *or*) and three fixed phrases
  (who-are-you, what-is-this, how-are-you), each `defined_by` its natural genus
  (`entity` / `property` / `action` / `concept` / `relation`) so every one roots
  to `link`, each lexicalised in every supported language with a per-form
  description. Three new roles in `src/seed/roles.rs` name the concepts the
  translator reasons over — `compositional_lemma` (a single word it maps
  word-for-word), `compositional_phrase` (a fixed multi-word phrase it maps
  whole), and `compositional_genitive_head` (a noun that can govern a Russian
  genitive-of complement, carried by `synonym` and `example`); agreement's
  genitive-singular form `согласования` carries `action "genitive"` so the
  genitive-of construction is recognised from the form's grammatical tag rather
  than by naming the word in code. Three new `Lexicon` query methods in
  `src/seed/meanings.rs` — `role_surface_translation` (translate a source surface
  to the target language through the role's meaning that lists it),
  `role_lists_surface` (a structural test by role), and
  `role_action_surface_translation` (the same translation but the source form must
  also carry a given grammatical `action` tag) — back the four pipeline functions
  (`russian_phrase_to_english`, `russian_word_to_english`,
  `russian_genitive_relation_head`, `russian_genitive_noun`), which now delegate to
  `crate::seed::lexicon()` instead of matching hardcoded arms; the word-sequence
  walker and the capitalizer are unchanged. The worker drops the four dictionaries
  for the same three role consts and four mirror helpers (`wordIn`,
  `roleSurfaceTranslation`, `roleListsSurface`, `roleActionSurfaceTranslation`),
  and its embedded `MEANINGS_LINO` was re-synced byte-identically (11323 lines).
  The conversion is behaviour-neutral on the pinned specs — "доброе яблоко" →
  "Good apple", "что это такое?" → "What is this?", and issue #230's "Найти
  синонимы или примеры согласования" → "Find synonyms or examples of agreement"
  all stay green in both engines — and converges the worker with the Rust pipeline
  on an explicit "как дела" → "how are you" translation request (a bare greeting
  still routes to the greeting detector first, unchanged). Because the surfaces now
  exist in every language, the compositional lemmas and phrases are complete in
  Hindi and Chinese as well as English and Russian (issue #386).
- The README/prose summarizer's sentence classifier (`classify_sentence` in
  `src/summarization/mod.rs`) is data-driven too. Its seven cue categories were
  seven hardcoded mixed English/Russian substring arrays scanned in a fixed
  priority order. Those surfaces now live as self-describing meanings in a new
  seed file (`data/seed/meanings-summary.lino`): one structural
  `summary_statement_kind` genus (`defined_by` `concept`, role
  `summary_statement_kind` — structural only, no handler queries it) groups the
  seven `summary_kind_*` leaf meanings (install, example, language, stars,
  purpose, use case, feature), each `defined_by` that genus, carrying the
  `summary_classification_cue` role, and lexicalised in every supported language
  with a per-form description. `classify_sentence` now walks the meanings
  carrying `ROLE_SUMMARY_CLASSIFICATION_CUE` in declaration order — which encodes
  the original priority order — and returns the kind of the first meaning whose
  surface fragments occur in the lowercased sentence as a raw substring, mapping
  the matched slug to a `StatementKind` through a new `StatementKind::from_slug`
  resolver, so the former seven `contains_any`-over-hardcoded-arrays blocks (and
  the now-unused `contains_any` helper) are gone — the code knows only the
  concept "a kind of summary statement" while every cue lives once in data. The
  `language` kind keeps its length guard structurally: a sentence that contains a
  language cue but runs past twelve whitespace words is not an identity line, so
  the scan continues to a later kind exactly as the original `&& word_count <= 12`
  arm did. The migration is byte-faithful — the per-kind surface sets equal the
  original arrays exactly, including the significant leading/trailing spaces the
  lino parser preserves (`" supports "`, `"$ "`, `"npm install"`) — every one of
  the 48 original English/Russian cue surfaces survives in its original kind. And
  because the cues now exist in every language — Hindi and Chinese throughout,
  plus Russian for the install and use-case kinds the original arrays left
  English-only — sentences classify where the former arrays recognised nothing. The summarizer is Rust-only — the browser worker has no
  prose-classification route — but its embedded `MEANINGS_LINO` mirrors the new
  file byte-identically (30 meaning files, verified by
  `experiments/issue-386-meanings-mirror.mjs`) so the shared knowledge base stays
  complete, and three locking tests pin the declaration-order scan, one surface
  per kind, and the language length-guard fall-through (issue #386).
- The coding catalog (`src/coding/catalog/`) is data-driven too. Its two tables —
  the ten supported languages (`languages.rs`) and the eleven coding tasks
  (`tasks.rs`) — no longer carry inline `aliases` arrays naming the request
  surfaces in raw words; the `aliases` field is gone from both `ProgramLanguage`
  and `ProgramTask`. Those surfaces now live as self-describing meanings in a new
  seed file (`data/seed/meanings-coding-catalog.lino`): a `program_language`
  genus and a `program_task` genus (each `defined_by` `concept`, rooted through
  it to the `link` ontology), with one `program_language_<slug>` leaf per
  language and one `program_task_<slug>` leaf per task `defined_by` its genus and
  lexicalised in every supported language — each leaf's slug naming the catalog
  entry it lexicalises, so the table names only the language-independent concept
  while the words live once in data. Two new roles in `src/seed/roles.rs` —
  `program_language_alias` and `program_task_alias` — mark the surface-bearing
  leaves; `program_language_by_alias` / `program_task_by_alias`
  (`src/coding/catalog/mod.rs`) read their surfaces from the lexicon by slug
  (`alias_surfaces("program_language_", slug)` / `"program_task_"`) instead of
  the deleted inline arrays, so the code knows only "a programming language" and
  "a coding task". Several leaves additionally `defined_by` the deeper meaning
  they specialise — `program_language_rust`/`_python`/`_javascript` point at the
  `language_*` meanings the formalizer already defines, `program_task_hello_world`
  at the canonical `hello_world` archetype — so the catalog rejoins the rest of
  the lexicon rather than standing apart. The seed surface set is a strict
  superset of both engines' former arrays, so the migration loses no recognition:
  every original Rust alias and every original worker alias still resolves
  (pinned by `experiments/issue-386-worker-catalog-alias-parity.mjs` across all
  ~29 legacy surfaces). Because the surfaces now cover every language uniformly,
  the browser worker's `programLanguageFromPrompt` / `programTaskFromPrompt`
  (rewired to read each slug's surfaces through a new `wordsForMeaning` helper,
  with their inline `aliases` likewise deleted) converge onto the Rust catalog —
  the worker gains the symbol spellings `c++` / `c#` the Rust table always carried
  and the Hindi/Chinese surfaces (रस्ट, जावा, गो, …) neither engine had before.
  Six lib tests (`seed_alias_coverage` in `src/coding/catalog/mod.rs`) enforce the
  bidirectional drift guard — every catalog slug owns a role-bearing alias meaning
  that lexicalises at least one surface, every alias meaning names a real catalog
  slug, and every language/task resolves through its seed surfaces — and the
  worker's embedded `MEANINGS_LINO` was re-synced byte-identically (31 meaning
  files, 12222 lines, verified by `experiments/issue-386-meanings-mirror.mjs`)
  (issue #386).
- The translation formalizer no longer hardcodes its Wikidata vocabulary — the
  last raw-word tables in the codebase. The `ITEM_LABELS` and `PROPERTY_PATTERNS`
  arrays in `src/translation/formalization.rs` — the entity labels (apple/Q89,
  fruit/Q3314483, sorting algorithm/Q181593, water, bread, carrot, …) and the
  binary-relation and translation properties (instance-of/P31, subclass-of/P279,
  part-of/P361, …) the formalizer matched against — are gone. They are now
  projected once (cached in a `std::sync::OnceLock`, allowed here because
  `formalization.rs` is not compiled into the wasm worker) from a new seed file
  `data/seed/meanings-wikidata.lino`: nine `wikidata_item_*` meanings carry the
  new `ROLE_WIKIDATA_ENTITY_ANCHOR`, seven `wikidata_property_*` meanings carry
  `ROLE_BINARY_RELATION_PROPERTY`, and one carries `ROLE_TRANSLATION_PROPERTY`
  (three new roles in `src/seed/roles.rs`). Each meaning records its
  language-independent Q-id/P-id in a `wikidata` field, its canonical English
  label as the first English word, and every multilingual surface as a described
  word form — so the formalizer references the concept by role and
  language-independent id, never by raw words in one language. The old hardcoded
  `P31`→`P279` copular ambiguity (an `is a` form that can read as instance-of or
  subclass-of) is data-driven too: the ambiguous form's `action` records the
  alternative property's slug, which the parser follows. The seed surface set is
  a strict superset of both former tables, so the migration loses no recognition;
  the worker's embedded `MEANINGS_LINO` re-synced byte-identically to 32 meaning
  files, 12625 lines (`experiments/issue-386-meanings-mirror.mjs`), and the full
  `--lib` and `--test unit` suites stay green (issue #386).
- Three source files that had grown past the 1000-line file-size guard
  (`scripts/check-file-size.rs`) were split into focused modules with no
  behaviour change. The seed loader `src/seed/meanings.rs` keeps only its parsing
  and querying code, with its invariant tests moved to `src/seed/meanings/tests.rs`.
  The role registry `src/seed/roles.rs` became a thin parent that re-exports five
  topic submodules (`roles/{program,intent,language,reasoning,tooling}.rs`), so
  every `ROLE_*` constant stays reachable at its existing
  `crate::seed::roles::ROLE_*` path. The code-generation spec
  `tests/unit/specification/code_generation.rs` became a directory module
  (`code_generation/{mod,single_turn,follow_up,task_catalog}.rs`) sharing one
  `POPULAR_LANGUAGES` table. All three pass the guard again, and the `--lib`
  (378) and `--test unit` (760) suites are unchanged (issue #386).

### Fixed
- The follow-up "Отмени сортировку" ("cancel the sorting") no longer returns
  `intent: unknown`. Operations now declare their inverse in the seed
  (`cancel_reverse_sort` carries `inverse "reverse_sort"`), and the subtractive
  substitution rules are *derived at runtime* by mirroring the additive ones, so
  a "cancel X" follow-up lowers the accumulated program back through "X" —
  restoring the ascending sort while keeping earlier edits such as the path
  argument. Adding a new cancellable operation is now pure seed data with no new
  control flow, and the behavior is covered across English, Russian, Hindi, and
  Chinese in both the Rust solver and the web worker (issue #386).
- "Можешь написать мне Playwright скрипт?" (and its English counterpart) again
  route to the Playwright starter-script handler instead of the generic
  write-program clarification. The issue #386 generalisation of
  `writeProgramParameters` made "написать … скрипт" look like a bare
  write-program request, and the browser worker dispatched `tryWriteProgram`
  ahead of `tryPlaywrightScript` — the reverse of the canonical Rust order where
  `try_playwright_script` runs before the specialized-handler group. The worker
  dispatch was reordered to mirror `src/solver.rs`, with a vm regression harness
  (`experiments/issue-386-worker-playwright-dispatch.mjs`) asserting the
  Playwright handler wins for both languages while a bare "напиши программу"
  still reaches write-program (issue #386).
- CJK prose no longer triggers a phantom unit-incompatibility refusal. The
  unit-word boundary check (`contains_unit_word` in `src/solver_handler_units.rs`)
  previously took the permissive substring path for every non-ASCII unit, so the
  day unit "天" matched inside "天气" (weather) and the gram unit "克" inside the
  transliteration "弗拉克斯", turning a units-free Chinese prompt into a bogus
  time-vs-mass incompatibility answer. Because CJK ideographs are alphabetic to
  `char::is_alphabetic` and the scripts have no inter-word spaces, the same
  word-boundary rule already used for ASCII units now also applies to CJK units —
  a unit glued inside a larger compound is rejected, while one next to a digit
  ("7天", "5千克") or at a token edge still matches. Inflected alphabetic scripts
  (Russian "килобайт" → "килобайте", Hindi "किलोबाइट") keep the permissive
  substring path, since they attach suffixes directly to the unit (issue #386).

## [0.175.0] - 2026-06-01

### Fixed
- Fixed GitHub repository extraction prompts so repository URLs without schemes route correctly, avoid false configuration capability answers, and keep JSON formatting directives attached to the extraction task.

## [0.174.0] - 2026-06-01

### Fixed
- Resolved relational apple-box arithmetic word problems by reducing box facts into calculator expressions with step-by-step reasoning in Rust and the browser worker.

## [0.173.0] - 2026-06-01

### Fixed
- Answer compound-interest prompts that ask for step-by-step calculation and a
  follow-up EUR conversion instead of falling through to the unknown fallback.

## [0.172.0] - 2026-06-01

### Fixed
- Agent-mode Wikipedia research prompts now split quoted patent comparisons into focused searches and strip follow-up instructions from web-search queries.

### Fixed
- Routed the Russian `SPEC dirven development` how-to prompt through procedural planning with a `dirven` to `driven` typo correction instead of the unknown fallback.

## [0.171.0] - 2026-06-01

### Fixed

- Kept agent-mode research table follow-up prompts tied to the prior web-search step instead of falling through to the unknown response.

## [0.170.0] - 2026-05-31

### Fixed
- Route the budget-calculator composite prompt to a Python `write_program` blueprint instead of reducing it to web search or an unsupported template.

## [0.169.0] - 2026-05-31

### Changed
- Documented local-server setup for Codex, Claude Code, OpenCode, and Link Assistant Agent in the README and server API guide.

## [0.167.0] - 2026-05-31

### Added
- Add a white-box self-improvement loop that proposes learned Links Notation seed rules from accumulated unknown traces and gates adoption behind the coding-modification benchmark ratchet.

## [0.166.0] - 2026-05-30

### Fixed
- Reduced response-level "Report issue" prompts to unresolved unknown turns and included the focused reasoning trace in missing-rule report URLs.

### Changed
- Softened multilingual unknown fallback copy so reporting is framed as a last-resort seed-extension path.

## [0.165.0] - 2026-05-30

### Added

- Added an issue #362 multilingual multi-turn coding-modification benchmark with
  a deterministic `minimum_pass_count` ratchet and download-on-test provenance
  for CanItEdit, HumanEvalFix, and EDIT-Bench.

## [0.164.0] - 2026-05-30

### Fixed
- Added an issue #361 cross-runtime parity harness that verifies the browser
  worker mirrors the Rust core for the issue #349 reverse-sort follow-up,
  including the unknown-path rule synthesis trace and the no-active-program
  guard.

## [0.163.0] - 2026-05-30

### Fixed
- Added full diagnostic traces for synthesized write-program follow-ups, including route attempts, coreference binding, modifier detection, rule construction, verification, and program-plan lowering.

## [0.162.0] - 2026-05-30

### Added
- Added unknown-path rule construction and verification traces for resolvable program-modification follow-ups.

## [0.161.0] - 2026-05-30

### Added
- Generalized `write_program` modifiers so operation-vocabulary slugs referenced
  by program-plan rules are discovered as modifiers instead of being hard-coded.
- Added reverse-sorted file-listing program variants, including the composed
  path-argument plus reverse-sort variant across supported template languages.

### Fixed
- Reverse-sort follow-ups to file-listing programs now lower to reverse-sorted
  program output instead of reusing the ascending file-listing variant.

## [0.160.0] - 2026-05-30

Fixed
- Route bare program-result follow-ups back to the active generated program artifact across English, Russian, Hindi, and Chinese.

## [0.159.0] - 2026-05-30

### Added

- Added the issue #356 rule-synthesis design for constructing verified
  substitution rules over Links Notation, plus a docs traceability test that
  pins the core contract for #357, #358, and #359.

## [0.158.0] - 2026-05-30

### Added
- VS Code extension (`vscode/`) that embeds the committed `src/web/` chat UI inside a Webview around the same HTTP/web boundary as the browser, the HTTP server, and the Electron desktop shell — no forked UI (issue #353).
- Dual-host packaging from one manifest: a Node host (`src/extension.node.cjs`, `shell: "VS Code"`) that starts an opt-in loopback `formal-ai serve` process, routes chat through `POST /v1/chat/completions`, and can drive Docker-sandboxed code execution; and a Web Worker host (`src/extension.web.cjs`, `shell: "VS Code Web"`) for `vscode.dev` / `github.dev` that stays on the in-process WebAssembly engine and imports no `node:*` builtins.
- Reusable pure extension libraries (`vscode/src/lib/`): `config.cjs` (settings → `desktopStatus` mapping), `bridge.cjs` (host-agnostic, default-deny `FormalAiDesktop` dispatcher), `webview-html.cjs` (Webview sandbox reconciliation — `<base href>`, strict nonce CSP, same-origin blob Worker bootstrap, main-thread/worker `fetch` and `importScripts` seed rebasing, and the `postMessage` bridge), `chat-view.cjs` (shared `WebviewView` provider), and `server-process.cjs` (Node-only `formal-ai serve` discovery / health-wait / spawn). Each takes its effectful dependencies by injection so it is unit-testable without a live VS Code host.
- Six `formal-ai.*` settings (server enabled/host/port, docker image, default tool grants, default agent mode) and four commands (Open Chat, Toggle Local Server, Sync Memory, Open Network View); the extension declares `virtualWorkspaces` and `untrustedWorkspaces` support because the in-process agent is safe everywhere while the server/Docker features only run in trusted desktop windows.
- `vscode` environment declared in the canonical seed (`data/seed/environments.lino`) with `browser_to_vscode` and `vscode_local_sync` flows, plus a strengthened `environment_directory_declares_every_supported_surface` unit test.
- VS Code spec test (`tests/unit/specification/vscode_surface.rs`, 13 cases) that pins the dual-host file contracts and exercises the shared engine endpoints (`/v1/chat/completions`, `/v1/graph`, full-bundle memory round-trip) to prove "all the same features", and a Playwright e2e spec (`tests/e2e/tests/issue-353.spec.js`) asserting the VS Code surface labelling for both hosts.
- `npm run vscode:dev` / `vscode:package` / `vscode:smoke` / `vscode:test` root scripts, with the VS Code node test suite wired into the CI lint job; `.cjs` files now count as code changes in `detect-code-changes.rs` so extension-host edits trigger lint/test/changelog.
- Architecture docs (`docs/vscode/extension.md`), a Marketplace README (`vscode/README.md`), a README VS Code section, and an issue-353 case study (`docs/case-studies/issue-353/`).

### Changed
- The web app's desktop status label is now surface-aware: `desktopSurfaceLabel(status)` returns "VS Code" when the host shell matches `/code/i` (so both `"VS Code"` and `"VS Code Web"` read as *VS Code*), otherwise "Desktop". The Electron shell is unaffected.

## [0.157.0] - 2026-05-30

### Added
- Added an ignored regression test and runnable example that reproduce issue #349's Russian reverse-sort follow-up for issue #355.

## [0.156.0] - 2026-05-30

### Fixed
- Answer prompts asking which dollar exchange rate is used for calculations by delegating the USD/RUB lookup to `link-calculator`.

## [0.155.0] - 2026-05-30

### Fixed
- Recognize Russian "Привет давай знакомиться!" and equivalent get-acquainted prompts as identity self-introduction requests instead of unknown prompts.

## [0.154.0] - 2026-05-30

### Added
- `/download` landing page for formal-ai Desktop (`src/web/download/`), modelled on
  vk-bot-desktop: OS auto-detection with macOS/Windows/Linux tabs, a release grid
  fed from the GitHub Releases API, in-browser SHA-256 checksum verification
  against `SHA256SUMS.txt`, build-provenance guidance, and macOS Gatekeeper notes.
  It honours the existing theme and locale switching (en/ru/zh/hi) and ships a CSP
  (issue #347, R1/R2/R7).
- Cross-platform desktop release pipeline (`.github/workflows/desktop-release.yml`)
  with explicit electron-builder `artifactName` templates, `SHA256SUMS.txt`,
  `BUILD-PROVENANCE.txt`, SLSA build-provenance attestation, and release-asset
  upload for macOS, Windows, and Linux (R1).
- Playwright e2e coverage for `/download` (`tests/e2e/tests/issue-347.spec.js`) and
  CI-generated theme/locale screenshots committed under `docs/screenshots/issue-347/`
  (R2).
- `docs/desktop/server-api.md`: how to enable the opt-in local OpenAI-compatible
  server (`formal-ai serve`) and point the `codex`, `agent`, and `claude` CLIs at
  it, with bearer-token auth and the in-process-by-default contract (R3/R4).
- `docs/case-studies/issue-347/` case study (requirements, prior-art survey,
  CI/CD-template comparison) plus a `ROADMAP.md` documenting the R5c/R5d/R6
  implementation (R8/R9/R10).
- Local-database sync (R5c): `src/memory_sync.rs` (`SyncStore` + union-by-id
  merge) and `GET /v1/memory`, `GET /v1/memory/since`, `POST /v1/memory/import`
  endpoints, with `desktop/lib/memory-sync.cjs` reconciling the browser
  (IndexedDB) log with the native store while server mode is on.
- Local-execution routing (R5d): `desktop/lib/tool-router.cjs`, a default-deny,
  permission-gated tool dispatcher. `http_fetch` / `url_navigate` /
  `read_local_file` are served by the local process; `eval_js` / `code_exec` /
  `shell` run inside the `konard/box-dind:2.1.1` Docker sandbox with logs
  captured. Denied calls (the default) return a structured refusal and nothing
  executes; Docker absence refuses rather than running unsandboxed.
- Links-Notation REST envelopes + LinksQL (R6): `GET /v1/bundle`, `GET /v1/links`
  (the knowledge graph as a `knowledge_graph` document), and `POST /v1/links/query`
  returning a `links_query_result` envelope, backed by the read-only LinksQL
  evaluator in `src/links_query.rs` (`MATCH (a)-[r]->(b) WHERE … RETURN …`).
- First-party Anthropic→OpenAI adapter (R4): `POST /v1/messages` (`src/anthropic.rs`)
  translates the Anthropic Messages protocol to the existing solver and back,
  including SSE streaming, so `claude` targets the local server via
  `ANTHROPIC_BASE_URL` without a third-party proxy.

### Fixed
- Web app (`src/web/app.js`): declared the R5c `syncDesktopMemoryNow` callback
  before the effect that lists it as a dependency. React evaluates a hook's
  dependency array during render, so referencing the later `const … =
  useCallback(…)` hit its temporal dead zone and threw
  `ReferenceError: Cannot access 'syncDesktopMemoryNow' before initialization`,
  crashing the whole component before it could mount. Added a static guard
  (`tests/e2e/scripts/check-web-tdz.mjs`, wired into the lint job as
  `check:web-tdz`) that fails CI if any hook dependency array references a
  `useCallback`/`useMemo` const declared later in the same component.

### Changed
- The desktop shell (`desktop/main.cjs`) now runs the **in-process** reasoning
  agent by default and only starts the local OpenAI-compatible server when
  `FORMAL_AI_DESKTOP_SERVER` is set — matching the `/download` page copy and the
  in-process-by-default requirement. The web app routes chat to the local server
  only when the Electron bridge reports it is ready, otherwise it stays in-process
  (R3/R4/R5a/R5b).

## [0.153.0] - 2026-05-29

### Added
- Composite `write_program` **blueprints**: a request the verified template
  catalog cannot resolve to a single alias (e.g. "make an HTTP GET request,
  parse the JSON, compute the mean and median, and output the results with error
  handling and comments") no longer dead-ends on `write_program_unsupported`.
  The blueprint synthesizer (`src/coding/blueprint.rs`) decomposes the prompt
  into capabilities (http_request, json_parse, statistics, output_results,
  error_handling, comments — each matched in English, Russian, Hindi, and
  Chinese), matches a recipe (`http_json_stats`), and returns a real, idiomatic
  program for Rust, Python, or JavaScript together with a numbered decomposition
  plan, the required libraries, and how-to-run instructions.
- Honest execution contract for blueprints: because composite programs need
  external libraries and network access the offline sandbox cannot provide, the
  blueprint is always reported as **"not run"** and never claims it "compiled and
  ran". The decomposition is recorded as `program_blueprint:` trace links and a
  `response:write_program:blueprint:<recipe>:<language>` evidence link.
- Case study `docs/case-studies/issue-340/` with timeline, requirements,
  root-cause analysis, solution plans, and an existing-components review.
- Two independent compositional axes: a blueprint program is now a *projection*
  of its decomposed capabilities rather than a single frozen string.
  - `comments` axis — when the request asks for comments the documented program
    is emitted; otherwise whole-line documentation (and a leading Python
    docstring) is stripped.
  - `error_handling` axis — optional defensive blocks are wrapped in
    `// region:error_handling … // endregion:error_handling` markers (`#` for
    Python/Ruby): the Rust empty-input guard, the Python `raise_for_status` +
    empty-list guard, and the JavaScript `!response.ok` + empty-array guard. The
    marker lines are always stripped from output; the region body is kept only
    when the request asks for error handling.
  The axes are orthogonal, so one recipe yields the full cross-product of four
  distinct, still-compilable programs (`documented`, `comments_only`,
  `errors_only`, `stripped`) — reasoning from the decomposition instead of
  memoizing one answer (`NON-GOALS.md`). Verified by unit tests in
  `src/coding/blueprint_tests.rs`, mirrored in the JS worker, and compile-checked
  offline via `examples/issue_340_emit_variants.rs` (each emitted Python/JS
  variant passes `py_compile` / `node --check`).
- `BlueprintComposition` setting ("Program composition"): switches the synthesis
  strategy between `Composed` (default — project the program from the decomposed
  capabilities) and `Documented` (always emit the fully annotated program with
  every region and comment). Exposed as a dropdown in the demo UI, toggleable by
  natural language ("documented programs", "полная документация", …), persisted
  in preferences, forwarded to the worker, localized across all four lino-i18n
  locales (en/ru/hi/zh), and reported in the self-facts inventory as
  `relation "blueprint_composition"` across the Rust core, JS worker, and app.js
  local fallback.

### Changed
- Browser worker parity (R7): `src/web/formal_ai_worker.js` mirrors the
  blueprint synthesizer byte-for-byte, so the GitHub Pages WASM/JS demo answers
  composite program requests identically to the Rust core. A `vm`-sandboxed
  parity experiment (`experiments/issue-340-worker-parity.mjs`) asserts both
  engines agree across English/Russian Rust, Python, and JavaScript variants,
  that both the `comments` and `error_handling` axes compose identically in both
  engines, that the `Documented` strategy keeps every region/comment, that the
  active composition is reported in the self-facts, and that partial requests
  (no statistics) stay honestly unsupported.

## [0.152.0] - 2026-05-29

### Added
- Recursive `fibonacci` coding task in the coding catalog (Rust catalog, `.lino`
  seed, and the WASM/JS worker), so prompts like "Write a Python function that
  calculates the Fibonacci sequence recursively" generate a verified program
  that prints F(10) = 55 (issue #334).
- Natural-language "word problem" normalizer that resolves "(the) N-th Fibonacci
  number" references, rewrites spelled-out operators ("and multiply it by" →
  `*`), and drops trailing instruction sentences, so "calculate the 10th
  Fibonacci number and multiply it by 8% of 500" reduces to `55 * 8% of 500`
  = 2200 (issue #334).

### Fixed
- The shared `no_std` arithmetic evaluator (used by the CLI fallback, the
  compiled WASM worker, and the JS worker fallback) now understands "N% of M"
  percentage-of phrases, rewriting `8% of 500` to `( 8 * 500 / 100 )` so the
  GitHub Pages WASM demo evaluates `55 * 8% of 500` to 2200 instead of returning
  "unparseable". A bare `%` not followed by "of" still parses as modulo
  (issue #334).
- Coding prompts containing "number" or "program" are no longer misread as a
  unit-incompatibility conversion: unit tokens such as "mb" and "gram" now match
  only on word boundaries instead of as substrings of "nu**mb**er" /
  "pro**gram**" (issue #334).

### Added
- Software-project follow-up handler so a decomposed agent step such as "test it
  by scraping wikipedia.org and show me the top 10 most frequent words" stays
  bound to the active project dialogue. It formalizes a `software_project_followup`
  meaning (parent request, follow-up kind, target site, expected output) with
  `generated_code`, `test_execution`, and `network_access` approval gates instead
  of running the test. Verification/execution/demonstration verbs are recognized
  across all supported languages (en, ru, hi, zh), and the handler is mirrored in
  both the Rust solver and the browser worker (issue #341).

### Fixed
- A software-project test/run/verify follow-up no longer misroutes to a
  `wikipedia` concept lookup (online) or the unknown-intent opener (offline)
  after the first plan turn (issue #341).

## [0.151.0] - 2026-05-29

### Added
- Syntax highlighting for chat code blocks via a dependency-free, highlight.js-compatible tokenizer (`src/web/syntax-highlight.js`) covering rust, python, javascript/typescript, go, c, cpp, java, csharp, ruby, bash, and json (issue #330).
- A copy button on every rendered code block that copies the raw source to the clipboard with "Copied!" feedback.
- A "Copy as Markdown" button on each chat message that copies the whole message content (Markdown fences preserved).
- Localized strings for the new copy buttons in all four locales (en/ru/zh/hi).
- End-to-end Playwright tests proving highlighting renders and both copy buttons work against a freshly built `src/web`.
- Runnable example (`examples/issue-330-code-highlighting/`) with run/test instructions and a deep case study in `docs/case-studies/issue-330/`.
- Code answers now teach a novice: every generated program is followed by a localized "How it works" explanation and step-by-step "How to test it yourself" instructions (install the toolchain, save the file, compile, run, compare the output) in en/ru/hi/zh (issue #330).
- When the dialog already walked the user through running code, a follow-up code edit omits the verbose setup steps and shows a concise "test it the same way" note instead, detected from prior assistant turns in the conversation history.
- Four new deterministic coding tasks broaden the catalog beyond hello-world and list-files — FizzBuzz, factorial of 5, string reversal, and the sum from 1 to 10 — each with a verified fixed output and templates for all ten supported languages, reachable in en/ru/hi/zh (issue #330).
- The JavaScript demo worker (`src/web/formal_ai_worker.js`) now mirrors the full Rust catalog (the four new tasks, all ten languages with their setup/run/check metadata) and the novice "How it works"/"How to test" guidance, keeping the in-browser engine in lockstep with the Rust engine.

### Changed
- Reorganized the coding-task support into a cohesive `src/coding/` module — a `catalog/` submodule (`types.rs` for the records, `languages.rs`/`tasks.rs` for the catalog tables, `templates_core.rs`/`templates_extended.rs` for the per-language templates, and `mod.rs` for the lookups) plus `guidance.rs` for the novice "How it works"/"How to test" guidance — replacing the misleadingly named `src/engine_hello_world.rs` and `src/engine_program_guidance.rs`. The module covers general coding tasks across all ten supported languages, not only hello-world, and every file stays well under the repository's per-file line limit (issue #330).

## [0.150.0] - 2026-05-29

### Added
- Added `ROADMAP.md`, an implementation-progress tracker that maps every `VISION.md` pillar to its real `src/` status, the closed planning batches, and the planning epic that closes each remaining gap (issue #244).
- Added the issue #244 case study under `docs/case-studies/issue-244/`: a deep analysis (`README.md`), a structured code audit (`raw-data/code-audit.md`), summarized online prior-art research (`raw-data/online-research.md`), the full body and acceptance criteria of every planning epic (`proposed-issues.md`), and the raw issue/PR/CI snapshots.

- Opened the 14 vision-implementation planning issues (E1–E14, [#246](https://github.com/link-assistant/formal-ai/issues/246)–[#259](https://github.com/link-assistant/formal-ai/issues/259)), each linked to #244 and labeled `enhancement`, and recorded their numbers in `ROADMAP.md` and the case-study "Created Planning Issues" table.
- Added the 2026-05-26 post-implementation audit after E1-E14 were merged, preserving closed-issue/merged-PR/deferred-marker snapshots under `docs/case-studies/issue-244/raw-data/`.
- Opened the remaining follow-up batch E15-E20 ([#278](https://github.com/link-assistant/formal-ai/issues/278)–[#283](https://github.com/link-assistant/formal-ai/issues/283)) for native doublets storage, symbolic probabilistic ranking, desktop packaging, associative packages/permissions, Rust/WASM parity, and generalized skill compilation.
- Opened the reasoning-focused batch E21-E27 ([#298](https://github.com/link-assistant/formal-ai/issues/298)–[#304](https://github.com/link-assistant/formal-ai/issues/304)) after a third-pass audit on issue #244 feedback: reasoning under unknowns instead of a canned fallback, intent formalization as Links Notation (dropping the fixed catalogue), parametric `write a program` intents, `link-cli`-style `replace x y` / `when n do m` substitution rules over link CRUD, natural-language access to memory/APIs/code execution, a general code-modifying/executing agent, and permissively-licensed industry benchmark datasets.
- Opened the synthesis-focused batch E28-E32 ([#313](https://github.com/link-assistant/formal-ai/issues/313)–[#317](https://github.com/link-assistant/formal-ai/issues/317)) after a fourth-pass audit on issue #244 feedback: the universal 11-step loop is the verified single main path, but the synthesis step still resolves seeded answers instead of deriving them by composing decomposed sub-results over the links network (the imported industry benchmark suite passed 0/5 at the time; it now passes 10/10 after E28-E32 merged — see the 2026-05-29 entry). The batch adds a general link-native synthesis substrate, derived math/word-problem and counting answers, general program synthesis from spec + tests, general text manipulation over link structure, and a ratcheting benchmark suite — each bound by an anti-memorization rule (pass counts must rise via derivation, with paraphrased/renumbered held-out variants passing only when composed, never recalled).
- Embedded the hand-drawn universal problem-solving algorithm diagram in `README.md` with a stage→11-step-loop mapping table that points at `src/solver.rs`, `src/solver_unknown_reasoning.rs`, and `src/intent_formalization.rs`.

### Changed
- Reconciled stale documentation with the real state of the code: `ARCHITECTURE.md` §17 now references the `REQUIREMENTS.md` matrix as R1 … R251 (was R1 … R149) and links `ROADMAP.md`, and `REQUIREMENTS.md` gains an Issue #244 vision-planning section (R246–R251).
- Refreshed `ROADMAP.md`, `VISION.md`, `ARCHITECTURE.md`, `REQUIREMENTS.md`, and the issue #244 case study so they record E1-E27 as closed/merged (PRs #305-#311) and scope the then-remaining gap — generality of the synthesis step — to the E28-E32 batch, instead of describing the original 69-test planning backlog. (The 2026-05-29 entry records E28-E32 as merged and the benchmark suite at 10/10.)

### Added
- Implemented E33 ([#326](https://github.com/link-assistant/formal-ai/issues/326)): a single shared, data-driven multilingual operation vocabulary (`data/seed/operation-vocabulary.lino`, loaded by both the Rust core via `seed::operation_vocabulary()` and the browser worker via `seed_loader.js`). The text-manipulation handler now canonicalises every transform (uppercase, lowercase, reverse words, extract email, count occurrences, count unique words, deduplicate lines, sort lines, replace) against this table instead of matching hardcoded English literals, so a request triggers equally from native `en|ru|hi|zh` phrasing. Adding a new surface form or language is now a seed-data edit, never a code change. Covered by cross-language specs in `tests/unit/specification/text_manipulation.rs` and unit tests in `src/seed/operation_vocabulary.rs`.
- Opened the parity batch E33-E34 ([#326](https://github.com/link-assistant/formal-ai/issues/326)-[#327](https://github.com/link-assistant/formal-ai/issues/327)) after a fifth-pass audit on the issue #244 PR feedback ("all Rust and JavaScript logic are in sync", "all languages are supported equally"); E34 tracks porting the E28-E31 derivation paths into the JavaScript browser worker so it mirrors the Rust core.

### Changed
- Synced the issue #244 tracking docs to the post-E32 state: `ROADMAP.md`, `VISION.md`, `ARCHITECTURE.md` §16, `REQUIREMENTS.md`, and the case study now record E28-E32 ([#313](https://github.com/link-assistant/formal-ai/issues/313)-[#317](https://github.com/link-assistant/formal-ai/issues/317)) as closed/merged (PRs #319-#323), the synthesis step as deriving rather than seeding answers, and the industry benchmark suite as passing **10/10** with a `minimum_pass_count` ratchet (was the stale "0/5"). Vision pillars 24-26 move to **Built**.
- Recorded the fifth-pass parity audit in `docs/case-studies/issue-244/README.md`, scoping the remaining vision gap to cross-language and cross-runtime parity.

## [0.149.0] - 2026-05-29

### Added
- Mirror Rust synthesis, program synthesis, and text manipulation parity cases in the browser worker with a shared Rust/JS parity fixture.

### Fixed
- Prevent browser-worker synthesis prompts from falling through to the unknown or legacy template paths when the Rust core can derive the answer.

### Added
- Added a shared multilingual operation vocabulary so text manipulation and program synthesis recognize operation verbs across English, Russian, Hindi, and Chinese.

## [0.148.0] - 2026-05-29

### Added
- Response-language preference (`last message language` default, `preferred selected language`, or `UI language`) in the web app, with new `settings.responseLanguage` / `settings.preferredLanguage` i18n entries for all four locales.
- `list_files_arg` `write_program` task (list files at a path supplied on argv) with templates for all ten catalog languages.
- Conversation-context recovery for follow-up program modifications: a follow-up such as "make the program accept a path as an argument" now reuses the language and task from the prior turn instead of failing with `missing`/`missing`.
- Data-driven program-modification pipeline (`src/program_plan.rs`, mirrored in the browser worker) that represents the request as a Links Notation plan and lowers it through the substitution engine using rules defined as data in `data/seed/program-plan-rules.lino` (e.g. `path_argument` rewrites `list_files` → `list_files_arg`). Adding a new `(modifier → task-variant)` rewrite is pure rule data, proven by data-driven tests in both the Rust core and the JS worker. The lowered plan is surfaced as a `write_program_plan:` evidence link (Issue #324 R4/R6).
- Case study `docs/case-studies/issue-324/` with timeline, root-cause analysis, solution plans, and a universal dynamic problem-solving roadmap.

### Fixed
- `write_program` answers (intro, unsupported message, and execution report) are now rendered in the detected response language for Russian, Hindi, and Chinese instead of always English. Applied in both the Rust engine and the browser worker so the GitHub Pages demo stays in parity.

## [0.147.0] - 2026-05-28

### Added
- Expanded the industry benchmark fixture with held-out variants and a recorded pass-count ratchet.

## [0.146.0] - 2026-05-28

### Added
- Added formalized text-manipulation routing backed by composed substitution rules for transforms, rewrites, extraction, counting, line operations, and multi-step text workflows.

## [0.145.0] - 2026-05-28

### Added
- Added Python function synthesis with isolated verification for HumanEval/MBPP-style write-program prompts.

## [0.144.0] - 2026-05-28

### Added
- Added deterministic synthesis traces and anti-memorization coverage for GSM8K-style word problems, algebra substitution, and category-filtered object counting.

## [0.143.0] - 2026-05-28

### Added
- Added link-native synthesis over solved sub-impulse links for algebra substitution, remainder-sale word problems, and object counting.

## [0.142.0] - 2026-05-27

### Added
- `write_program` now answers "list the files in the current directory" requests
  for every catalog language (Rust, Python, JavaScript, TypeScript, Go, C, C++,
  Java, C#, Ruby). The Rust template uses `std::fs::read_dir`, matching what
  general assistants return for issue #312. The task is recognized in English,
  Russian, Hindi, and Chinese prompts.
- CJK-aware token and phrase matching in the program intent detectors
  (`engine_hello_world.rs`, `intent_formalization.rs`, and the web worker), since
  Chinese has no inter-word spaces and could not be matched by whitespace-split
  tokenization.
- Case study `docs/case-studies/issue-312/` documenting the timeline, the full
  list of requirements, root-cause analysis, the solution plan, and online
  research into how `read_dir`-based file listing is idiomatically written.

### Fixed
- A concrete `write_program` request (recognized task + language with a matching
  template) now takes precedence over the specialized handlers. Previously a
  prompt naming a language could be intercepted by `concept_lookup` and answered
  as an encyclopedia definition ("Rust") instead of returning the requested
  program.
- The JS-fallback `normalizePrompt` in the web worker now preserves the
  Devanagari block (U+0900–U+097F, including combining marks), restoring parity
  with the Rust `normalize_prompt` so Hindi prompts are matched identically on
  both code paths.

## [0.141.0] - 2026-05-26

### Added
- Add a curated permissive benchmark slice for HumanEval, MBPP, GSM8K, MATH, and BIG-bench with provenance notes and a deterministic pass/fail reporting test harness.

## [0.140.0] - 2026-05-26

### Added
- Add a bounded agent workspace runtime that can create, modify, delete, and inspect files through logged sandbox actions.

## [0.139.0] - 2026-05-26

### Added
- Natural-language API and code execution requests now go through agent-mode and associative-package permission gates, with auditable tool parameters, results, and execution status links.

## [0.138.0] - 2026-05-26

- Added the parameterized `write_program(language, task)` intent for seeded program generation, replacing per-language hello-world routing with language/task template parameters.
- Extended the catalog with `count_to_three` templates and unsupported-parameter responses so missing languages or missing language/task combinations fail explicitly.
- Updated Rust and browser-demo behavior-rule/tool metadata to advertise the single `write_program` path.

### Added
- Added a data-driven substitution-rule engine for link-pattern `replace x y` rules, conditional `when ... do ...` composition, CRUD event triggers, and trace-link records.

## [0.137.0] - 2026-05-26

### Added
- Added Links-Notation intent formalization with an impulse-id cache and routed the Rust solver from the formalized intent record.

## [0.136.0] - 2026-05-26

### Added

- Added traced unknown-prompt reasoning that records knowns, unknowns, candidate sources, and gather attempts before using the legacy fallback.

## [0.135.0] - 2026-05-26

### Fixed
- Localized behavior-rule list, detail, and dialog rule-update responses for supported UI languages.
- Added multilingual coverage for the reported "list your rules" phrasing and tightened chat markdown rendering for rule lists.

## [0.134.0] - 2026-05-26

Issue #288: add a seeded concept entry for `ложная тотальность` / `false
totality` so Russian manual-mode prompts such as `Что такое ложная
тотальность?` resolve through local concept lookup instead of the
unknown-intent fallback.

## [0.133.0] - 2026-05-26

### Changed
- Moved browser-worker stable id generation, unknown-answer opener selection, and intent-route matching semantics behind the Rust/WASM core so multilingual browser answers stay aligned with the native solver.

### Added
- Extended the natural-language skill compiler with a deterministic structured subset for typed inputs, procedure steps, generated tests, handler stubs, and explicit package/tool permissions.

### Fixed
- Recognize the Russian assistant-name command `Теперь тебя зовут ...` in the web demo and persist the configured name instead of falling through to the unknown-rule answer.

Issue #286: add a seeded concept entry for `антирежим` / `antiregime` so
Russian manual-mode prompts such as `Что такое антирежим?` resolve through
local concept lookup instead of the unknown-intent fallback.

## [0.132.0] - 2026-05-26

### Added
- Added reusable associative packages with dependency validation, Links Notation import/export, trigger replay, package permission checks for tool calls, and graph visibility for package handler/trigger/permission links.

## [0.131.0] - 2026-05-26

### Added

- Added an Electron desktop wrapper that starts the local Rust HTTP API, reuses the existing web chat, and exposes desktop API, graph, memory, and permission status.

## [0.130.0] - 2026-05-26

### Added
- Added link-native symbolic probability evidence with deterministic Bayesian-style and Markov-style ranking over formalization and answer candidates.
- Added probability evidence replay into traces and link-store memory, including cached-source provenance and offline-mode handling.

## [0.129.0] - 2026-05-26

### Changed
- Make `doublets-rs` the default native link-store backend while preserving Links Notation import/export as the recovery and migration projection.

## [0.128.0] - 2026-05-26

### Fixed
- Issue #272: recognize Russian prompts such as `А в чём ты можешь быть полезен` as capability questions instead of returning the unknown-intent teaching fallback.

## [0.127.0] - 2026-05-26

### Fixed
- Issue #262: recognize the Russian acknowledgement `ого, чето начал соображать:)` as a courtesy response instead of returning the unknown-intent teaching fallback.

## [0.126.0] - 2026-05-26

### Added
- Added deterministic natural-language skill compilation into reusable Links Notation packages with trigger-rule replay and `cache_hit` evidence.

## [0.125.0] - 2026-05-26

### Added
- Graduated the issue #258 trace-surface coverage for non-blocking graph-adjacent chat, Telegram trace links, code-answer execution status, and default-off diagnostics prose.

## [0.124.0] - 2026-05-26

### Added
- Graduated the OpenAI compatibility checks for configured bearer-token authentication and tool/function-call refusal unless agent mode is enabled.

## [0.123.0] - 2026-05-26

### Added
- Graduated agent-mode isolation and chat bounded-autonomy checks for explicit opt-in, sandbox disclosure, visible action logs, surfaced failures, destructive-action confirmation, time budgets, secret hygiene, and privilege revocation.

## [0.122.0] - 2026-05-26

### Added
- Graduated the links-network invariants for dynamic Type/SubType chains, source-backed facts, answer trace links, and ordered reasoning steps.

## [0.121.0] - 2026-05-26

### Added
- Graduated transparent-state chat queries for network snapshots, concept links, diagnostics, why explanations, retraction policy, Links Notation export, and user fact filtering.

### Fixed
- Kept personal fact-list prompts on the append-only memory query path instead of routing them as generic web-search requests.

## [0.120.0] - 2026-05-25

### Added
- Added a delegated relative-meta-logic / SMT-style decision procedure for propositional tautologies and linear arithmetic constraints.

## [0.119.0] - 2026-05-25

### Changed
- Graduated the issue #252 code-generation specification tests for top-10 hello-world seeds, execution Links Notation, isolation disclosure, sorting algorithms with tests, semantic code translation, and execution-failure traces.

## [0.118.0] - 2026-05-25

### Fixed
- Graduated the E6 translation-via-Links checks by preserving canonical meaning links and translated surface events in translation traces.

## [0.117.0] - 2026-05-25

### Added
- Graduated the public-knowledge source-cache provenance specification so source URL, fetched_at, content hash, refresh, cache-hit, conflict, flush, and offline-policy behavior are active tests.

### Fixed
- Offline external lookup attempts now emit an auditable `policy:offline` evidence link instead of only recording a skipped search.

## [0.116.0] - 2026-05-25

### Added
- Added temperature-based formalization selection with deterministic softmax
  guessing, ambiguity policy events, and clarifying-question handling.

## [0.115.0] - 2026-05-25

### Added
- Added a Wikidata P/Q-id prompt formalization engine with scored anchors,
  Wiktionary/Wikipedia/raw fallbacks, and solver evidence links for unresolved
  terms.

## [0.114.0] - 2026-05-25

### Changed
- Graduate the universal reasoning-loop acceptance tests and record candidate, validation, simplification, and trace events for finalized handler answers.

## [0.113.0] - 2026-05-25

### Added

- Introduced a `LinkStore` abstraction for `.lino` memory, event-log replay, and optional native doublets-rs mirroring.
- Graduated the issue 246 links-network specification tests for doublet reducibility, stable IDs, schema versioning, append-only history, concept uniqueness, and malformed Links Notation rejection.

## [0.112.0] - 2026-05-25

Issue #242: recover malformed English meaning questions such as
`what i digress mean?`, route them through the existing concept/Wikipedia/
Wikidata/Wiktionary lookup chain, and list dictionary page sources in the
source registry and connectivity diagnostics. Extend meaning prompt coverage
across the supported `en`, `ru`, `hi`, and `zh` language patterns.

## [0.111.0] - 2026-05-25

### Added
- Added confirmed, backup-aware memory purge/reset operations across the Rust library, CLI, and browser demo.
- Added multilingual browser controls and reset phrases for permanent deletion and full memory reset.
- Added issue #196 case-study evidence and regression coverage for destructive memory maintenance.

## [0.110.0] - 2026-05-24

### Fixed
- Russian proof requests such as `привет. докажи что простых бесконечно` now
  resolve to the formal Euclid infinitude-of-primes proof instead of the
  generic proof-plan fallback.
- English, Russian, Hindi, and Chinese prime-infinitude prompts now share a
  coverage-checked proof test matrix so localized phrasing cannot regress to a
  generic plan or capability response.

## [0.109.0] - 2026-05-24

### Fixed

- Russian requests for a Playwright script, including the common `Playright` typo, now return a Playwright starter example or ask for the target URL/actions when guess probability is low instead of falling through to `unknown` (issue #135).

## [0.108.0] - 2026-05-24

### Fixed
- Recognize general "what facts do you know?" prompts, including the reported Russian phrasing, and answer with local, internet, memory, and self-fact sources instead of the unknown fallback.
- Recognize LLM/OpenAI/API architecture follow-ups and explain the deterministic Links Notation runtime instead of falling through to unknown.
- Extend the same self-awareness coverage to reported Russian self-introduction, world-model, working-principle, project-purpose, and conversation-topic prompts from issues #137, #139, #141, #142, #147, #148, #155, and #237, including assistant-name configuration status in self-facts.

### Added
- Issue #223: pandas `DataFrame.join` method questions now return a scoped official-docs summary instead of the unknown fallback, with diagnostics linking the pandas docs source.

## [0.107.0] - 2026-05-23

### Fixed
- Answer Russian creator prompts like `кто тебя создал?` from the built-in Formal AI origin fact instead of falling through to the unknown fallback.

## [0.106.0] - 2026-05-23

### Added

- Answer assistant-name prompts in supported languages and add a configurable assistant name setting for the web demo.

## [0.105.0] - 2026-05-22

### Fixed
- Translate the Russian phrase `Найти синонимы или примеры согласования` as `Find synonyms or examples of agreement` instead of returning an English placeholder.
- Report unknown translation gaps explicitly with `translation_gap` evidence instead of rendering bracketed language placeholders such as `[en] ...` or `[ru] ...`.

### Changed
- Enforce PR language-facing test coverage for every supported language: English, Russian, Hindi, and Chinese.

### Fixed
- Issue #232: Answer Russian definition-style Wikipedia disambiguation pages such as `Существо` with their listed meanings instead of falling through to the Wikidata `Animalia` alias.
- Extend the Issue #232 regression to English, Russian, Hindi, and Chinese, with a CI coverage guard that fails if the definition-style disambiguation matrix loses a supported language.

## [0.104.0] - 2026-05-22

### Fixed
- Route enumeration-style research prompts such as "list all Genshin characters
  with off-field DMG" to deterministic web search instead of the unknown
  fallback.
- Cover enumeration-style web-search prompts across English, Russian, Hindi,
  and Chinese, and add a CI guard for language-resource changes that omit
  Hindi or Chinese updates.

## [0.103.0] - 2026-05-22

### Fixed
- Answer Wikipedia article-existence questions such as `есть такая статья в википедии?` with sourced exact or closest-match results instead of falling through to the unknown fallback.
- Avoid treating quoted Russian-language prose as an implicit UI language command.
- Extend the Issue #226 regression to English, Russian, Hindi, and Chinese, with a CI coverage guard that fails if the Wikipedia article-question or UI-language command matrices lose a supported language.

## [0.102.0] - 2026-05-22

### Fixed
- Stop faking translation for common nouns: the demo and the Rust pipeline
  now return real Wiktionary/Wikidata-backed translations for every noun
  reachable through the seeded raw-API-response cache, in both directions,
  replacing the `[ru]` / `[en]` placeholders (issue #221).
  `Переведи "помидор" на английский.` returns `tomato`,
  `translate "carrot" to russian` returns `морковь`, and the unquoted
  variants resolve through the same path.
- Wikidata SPARQL lexeme joins now restrict P5137 matches by
  `wikibase:lexicalCategory` so polysemous surfaces like `water` no
  longer cross noun ↔ verb boundaries (`water` resolves to `вода`,
  not `поливать`).

### Added
- `data/seed/api-cache/*.lino` — verbatim Wikidata and Wiktionary API
  response bodies, stored in indented Links Notation with base64-encoded
  payloads (RFC 4648, 76-character chunks). Bundle is capped at 128
  records per semantic bucket (entities, properties, search, sparql,
  pages-per-language) and every file stays under 1500 lines; bodies that
  exceed that cap are split into deterministic `<bucket>-partN.lino`
  parts and re-joined at load time by URL. No pre-extracted dictionary
  lives in the repo — only raw API responses that the formalization
  pipeline replays.
- `build.rs` enumerates `data/seed/api-cache/*.lino` at compile time and
  emits `OUT_DIR/seed_bundle_files.rs`. `src/translation/cache.rs`
  pulls the generated list with
  `include!(concat!(env!("OUT_DIR"), "/seed_bundle_files.rs"))` so new
  part files ship automatically without per-file `include_str!` edits.
- `FORMAL_AI_TRANSLATION_DEBUG=1` enables stage-by-stage stderr tracing
  through the translation pipeline (closes the "Future work" item from
  issue #218).
- Live Wiktionary fallback in `src/web/formal_ai_worker.js`
  (`liveWiktionaryTranslate`): the browser worker now hits
  `*.wiktionary.org/w/api.php?action=parse&...&origin=*` directly for any
  surface that is not already covered by the seed bundle, follows
  `{{see translation subpage|...}}` to the `/translations` subpage when
  present, and extracts the `{{tt+|<lang>|...}}` / `{{t|...}}` template
  payload. Mobile-friendly: no offline dictionary is bundled into the
  worker, the seed bundle stays small, and the MediaWiki action API is
  CORS-friendly through `origin=*`.

### Changed
- `src/translation/cache.rs` reorganises the on-disk accelerator and
  seed bundle by **semantic identity** rather than URL hash:
  - `data/wikidata-cache/{search,entities,query,sparql}/` for Wikidata
    `wbsearchentities` / `wbgetentities` / `action=query` / SPARQL.
  - `data/wiktionary-cache/<lang>/` keyed by page title.
  - `data/http-cache/misc/` (URL-hash) for anything else.
  The `data/wikidata-cache/`, `data/wiktionary-cache/` and
  `data/http-cache/` trees are gitignored — they are local accelerators
  written by `FORMAL_AI_LIVE_API=1` runs. The committed offline source of
  truth is the seed bundle under `data/seed/api-cache/`.
  `CachedHttpClient::get` consults seed bundle → on-disk accelerator →
  live transport in that order, so a clean checkout reproduces every
  test deterministically.
- `examples/refresh_translation_cache.rs` drives the full pipeline
  against a curated 128-noun seed list, populates the on-disk
  accelerator, then re-bundles it into the committed `.lino` seed files,
  splitting oversize records into `<bucket>-partN.lino` parts and
  removing stale parts that no longer back any record.
- `src/translation/wikidata.rs` adds the `wikibase:lexicalCategory`
  filter to the lexeme-join SPARQL so the polysemy edge case described
  in `docs/case-studies/issue-221/online-research.md` no longer crosses
  part-of-speech boundaries.
- `src/web/formal_ai_worker.js` no longer ships an offline translation
  dictionary. Translation flows through the existing meaning registry
  first, then falls back to the live Wiktionary fetch above. Removed:
  the `TRANSLATION_DICTIONARY` and `lookupDictionary` plumbing plus the
  `extractTranslations` parser in `src/web/seed_loader.js`.
- `tests/e2e/tests/issue-221.spec.js` exercises the live worker path
  end-to-end (quoted RU→EN, quoted EN→RU, unquoted prompts, Russian
  inflected forms via MediaWiki redirect, round-trip stability).
- `tests/unit/docs_requirements.rs` exempts the new seed-bundle and
  cache roots when scanning for deferred labels — the bundled wikitext
  bodies contain ISO 639-3 language codes that would otherwise trip the
  scanner.

Case study, raw reproductions, and external references live in
`docs/case-studies/issue-221/`.

## [0.101.0] - 2026-05-22

### Added
- Issue #224: implicit open-ended research questions such as `What is the most popular dataset for translation quality validation?` now route to the `web_search` pipeline instead of the unknown fallback. The matcher records `web_search:query_kind:implicit_research_question` so diagnostics show why a question without an explicit search verb used external-source gathering.

## [0.100.0] - 2026-05-21

### Changed
- The root Docker image now uses `konard/box-dind:2.1.1` as the supported runtime, keeps the Box DinD entrypoint, and starts `formal-ai telegram --mode polling` by default.

### Added
- Installed `start-command` in the image and documented the `$ --isolated docker --auto-remove-docker-container --` runner contract for tracked coding-task commands.
- Added `verify-formal-ai-dind`, Docker runtime regression tests, and issue #195 case-study evidence under `docs/case-studies/issue-195/`.

## [0.99.0] - 2026-05-21

### Fixed
- Translate single nouns and unquoted prompts correctly: `translate apple to russian` now returns `яблоко` and `переведи «яблоко» на английский` returns `apple` instead of the `[ru]` / `[en]` placeholders (issues #216, #217, umbrella #218). Adds an unquoted-surface fallback to the translation handler, mirrors it in the browser worker, seeds the Wiktionary/Wikidata cache for both nouns, and extends the browser offline registry with the apple meaning so the GitHub Pages demo answers offline.

## [0.98.0] - 2026-05-21

### Fixed
- Resolve the reported `OpenStreerMap` typo through the general Wikipedia fuzzy-search fallback instead of adding a one-off concept seed; the regression covers English, Russian, Hindi, and Chinese wrappers with mocked Wikimedia responses.

## [0.97.0] - 2026-05-21

### Fixed
- Resolved BSD ports prompts in every supported language from the local concept base instead of falling through to unrelated OpenBSD Wikipedia hits.

### Fixed

- Route current-day questions such as `Какой сегодня день?` through calendar reasoning instead of the unknown fallback.
- Cover current-day prompts across every supported language (`en`, `ru`, `hi`, and `zh`) in Rust and browser tests.
- Mirror the browser worker behavior with local time-zone evidence for current date and weekday answers, and add a CI coverage guard for multilingual feature matrices.

## [0.96.0] - 2026-05-21

### Fixed
- Recognize possessive behavior-rule list requests like `Покажи список своих правил` across supported languages instead of falling through to the unknown-intent fallback, with CI coverage for the multilingual prompt matrix.

## [0.95.0] - 2026-05-21

### Fixed
- Recognize Russian web-search prompts like `Найди яблоко в интернете` as `web_search` instead of falling through to the unknown fallback.

## [0.94.0] - 2026-05-21

### Fixed
- Route broader information-search prompts to web search instead of the unknown fallback, including English/Russian synonym forms plus Hindi and Chinese "search for information about ..." phrasings.

## [0.93.0] - 2026-05-21

### Fixed
- Added a seeded graph concept so prompts like `что такое граф`, `ग्राफ क्या है`, and `图是什么` answer through the Links Notation/meta-theory framing instead of the unknown fallback.
- Added a CI guard requiring localized concept records to cover every supported language.

### Fixed
- Fixed Russian translation prompts so quoted requests like `кто ты такой`, `что это такое?`, and `доброе яблоко` stay on the translation path and return English surfaces instead of identity/capability responses or `[en]` placeholders.

## [0.92.0] - 2026-05-21

### Fixed
- Fuzzily match close calculator command typos such as `Calcualte` and `Calcuate`, and show an interpretation statement before evaluating prompts like `Calcualte 2+5050`.

## [0.91.0] - 2026-05-21

### Added
- Dedicated `proof_request` intent and a universal proof / disproof engine
  (`src/proof_engine/`) for prompts like "Prove …", "Show that …",
  "Докажи …", "साबित कर …", and "证明 …". The engine never refuses: it
  evaluates arithmetic equalities with the exact arbitrary-precision
  calculator, looks up classical theorems (Pythagoras, Euclid's infinitude
  of primes, irrationality of √2, Fermat's little theorem, Gödel's first
  incompleteness theorem, Laplacian determinism reduced to Picard–Lindelöf)
  in a multilingual library (en/ru/hi/zh), and otherwise returns a
  structured `PartialPlan` that walks through the axiom-set reduction and
  enumerates the missing inputs. Closes
  [#185](https://github.com/link-assistant/formal-ai/issues/185).
- Configuration-aware proof rendering. The proof engine now consults two
  sliders on `SolverConfig`: `guess_probability` (default `0.8`) and the
  new `follow_up_probability` (default `0.75`). High guess probability
  prepends an "Interpretation" header that names the formal system and
  expands `PartialPlan` outcomes with explicit deep-reasoning steps — a
  closed-sentence translation `⟦φ⟧ = ∀x. P(x) → Q(x)` plus a
  `relative-meta-logic` verification step. High follow-up probability
  appends a localized "Clarifying questions" footer (en/ru/hi/zh) that
  enumerates what the user still needs to confirm before execution. Both
  sliders are exposed through the web preferences UI, the `SolverConfig`
  builder API, and the new `FORMAL_AI_GUESS_PROBABILITY` /
  `FORMAL_AI_FOLLOW_UP_PROBABILITY` environment overrides, and every
  decision is recorded as a `policy:*` / `proof_render:*` event in the
  append-only evidence log.

### Fixed
- Issue [#185](https://github.com/link-assistant/formal-ai/issues/185): the
  prompt "Prove determinism the way logic can handle paradoxes like Godel's
  math incompleteness" no longer returns the generic "I cannot answer that
  from local Links Notation rules yet" fallback. It now returns a real
  Laplacian-determinism proof inside the Newtonian axiom set, references
  Picard–Lindelöf for existence/uniqueness and Gödel's first incompleteness
  theorem for the limit, and asks the user to pick a concrete axiom set so
  the claim becomes a checkable proposition.

## [0.90.0] - 2026-05-21

### Changed
- Translation answers now read like natural conversation: the response body is just the deformalized target surface (still quoted when the user quoted the source) instead of the `meaning: … / surface (…): …` template. The meaning ID, source language, and target language remain in the Links Notation trace via `evidence_links`.

### Added
- Generalized `formalize → meaning → deformalize` pipeline under `src/translation/` (`http`, `cache`, `wiktionary`, `wikidata`, `meaning`, `pipeline`, `formatting`) routes Rust translations through real Wiktionary translation tables and Wikidata lexeme/sense joins, so any surface pair resolves through public data rather than a hand-written list.
- `CachedHttpClient` persists raw API responses under `data/translation-cache/` (FNV-1a-keyed `.body` + `.url` files); tests run deterministically offline against the committed cache, and contributors can refresh it with `FORMAL_AI_LIVE_API=1` via `examples/refresh_translation_cache.rs`.
- `match_source_formatting` / `matchSourceFormatting` helpers preserve the source fragment's leading capitalization and terminal punctuation, so `как у тебя дела?` round-trips to lowercase `how are you?` and an unterminated source stays unterminated in the target.

### Fixed
- Translations no longer rely on a hardcoded set of meanings: any surface routed through `TranslationPipeline::translate` can now resolve via Wiktionary + Wikidata, and the lowercase / uppercase source distinction is preserved in the target.

## [0.89.0] - 2026-05-21

Fixed multilingual "how X works" prompts, including Russian "как устроен AUR",
so they extract the explicit subject, route known subjects through concept
lookup, and use source-backed Wikipedia/Wikidata/web-search discovery for
unknown subjects instead of returning the unknown fallback.

## [0.88.0] - 2026-05-21

### Added

- Issue #205: optional experimental OCR image attachments using
  `tesseract.js@7.0.0`. The setting is off by default, warns that first use
  downloads about 6 MB, lazy-loads a separate `ocr.bundle.js`, and stores
  enabled image attachments as base64 data URLs in the exported Links memory
  event log.

## [0.87.0] - 2026-05-20

### Fixed
- Prebundle the browser demo's JavaScript dependencies into a local Bun-built vendor bundle so GitHub Pages no longer depends on external React, markdown, sanitizer, or `lino-i18n` CDN scripts.

## [0.86.0] - 2026-05-20

### Fixed
- Added browser-worker and Rust-core handling for `Переведи "как у тебя дела?" на английский.` so it returns an English translation instead of the unknown fallback.
- Made the Russian `Что ещё ты умеешь?` follow-up use conversation history and avoid repeating already discussed web-search details.
- Made the left `MENU` sidebar action group collapsible and persistent like the other sidebar sections.
- Improved prefilled issue reports by renaming the dialog section, removing reproduction boilerplate, and preserving more earlier dialog context within the GitHub URL budget.

## [0.85.0] - 2026-05-20

### Fixed
- Updated `link-calculator` to v0.17.2 so Russian currency conversions, binary modulo expressions, and simple equations use the latest upstream parser fixes.

## [0.84.0] - 2026-05-20

### Added
- Added symbolic calendar reasoning for weekday successor and predecessor prompts, including the reported Russian "после вторника" case.

## [0.83.0] - 2026-05-20

### Added
- Added a `summarization` module (`src/summarization.rs`) implementing a
  deterministic formalize → summarize → deformalize pipeline with configurable
  `SummarizationMode` (`Topic` 1–5 words, `Short` ~20%, `Standard` ~50%, `Full`
  100%, `Expand` ~200%), explicit statement caps, NSM-style semantic-prime
  expansion, compound-word shortening, and a boilerplate filter that drops
  install / example sentences from compressed answers.
- Added a curated project registry (`data/seed/projects.lino`,
  `src/seed/projects.rs`) covering Link Assistant, Link Foundation, and
  LinksPlatform projects with weighted statements, English/Russian localized
  variants, repository URLs, topic labels, and aliases.
- Added a `project_lookup` handler that runs after `concept_lookup` and answers
  "What is <project>?" prompts using the curated registry plus the summarization
  pipeline, logging `summarization:mode`, `summarization:language`, repository
  evidence, and the web-search providers consulted alongside the local answer.
- Added `scripts/decode-github-issue-url.rs` to decode prefilled GitHub issue
  URLs into readable Markdown for future overlong report-link investigations.

### Fixed
- Routed "What is Hive Mind?" / "Что такое Hive Mind?" prompts through
  `project_lookup` as a promoted registry match for `link-assistant/hive-mind`
  before showing other web-search results, preventing the Wikipedia
  closest-match fallback from answering with unrelated pages such as LOIC.
  Hive Mind now shares the same generic project path as other repository
  records.

### Added
- Extended the `summarization` module (`src/summarization.rs`) with a README
  ingestion path: `strip_markdown_noise` removes badges, fenced code blocks,
  HTML comments, heading markers, and blockquote chevrons; `formalize_markdown`
  feeds the cleaned prose through the existing classifier; `describe_readme`
  drives the whole `formalize → summarize → deformalize` pipeline with any
  `SummarizationConfig` (returns the repository slug in `Topic` mode).
- Added `DialogTurn`, `formalize_dialog`, `summarize_dialog`, and
  `generate_chat_title` so multi-turn conversations and chat titles are
  produced by the same pipeline (user turns weigh +20, assistant turns -10).
  The conversation-summary intent in `src/solver_handlers/mod.rs` now uses
  this pipeline and logs `summarization:mode`, `summarization:language`, and
  `chat_title` evidence in addition to the per-turn list.
- Wired `try_http_fetch` to recognise curated GitHub repositories
  (`match_curated_github_url`): when the requested URL points to one of the
  projects in `data/seed/projects.lino`, the response embeds a `Standard`-mode
  project description and the trace records `http_fetch:curated_project`,
  `summarization:mode`, and `summarization:language` so the URL → curated
  record → summary path is fully visible.
- Generalized project lookup so Hive Mind is treated as a promoted project
  record instead of a special handler. Project promotion defaults on for
  `link-assistant`, `link-foundation`, and `linksplatform`, can be switched
  off through solver/browser configuration, and explicit GitHub/GitLab/
  Bitbucket repository URLs route through the same `project_lookup` intent.
- Documented the default 30-statement cap as a `DEFAULT_MAX_STATEMENTS`
  constant in `src/summarization.rs` and re-exported it from the crate root.
- Re-exported the new helpers (`describe_readme`, `formalize_markdown`,
  `strip_markdown_noise`, `DialogTurn`, `formalize_dialog`, `summarize_dialog`,
  `generate_chat_title`, `DEFAULT_MAX_STATEMENTS`) from the crate root so
  downstream callers and the integration tests can use them directly.
- Added 11 specification tests in
  `tests/unit/specification/summarization_pipeline.rs` covering the new
  README, dialog, and chat-title flows plus the documented size targets and
  default cap; added two new tests to
  `tests/unit/specification/project_lookups.rs` pinning the HTTP-fetch
  curated-URL evidence.
- Documented the new surfaces in `ARCHITECTURE.md` § 7.1 and added rows
  R202-R207 to `REQUIREMENTS.md`.

## [0.82.0] - 2026-05-20

### Added

- Added chat commands to list and read behavior rules, list self facts, and teach dialog-local behavior overrides.
- Surfaced behavior rules as `When X then Y` (or `When X do Y`) statements grouped by topic in both the catalog listing and per-rule detail; the same grammar — and its Russian, Hindi, and Chinese translations — now records dialog-local overrides.

### Changed

- Expanded unknown-intent fallback text with self-contained Links Notation teaching guidance.
- Behavior-rule listing now groups entries by topic (Greetings, Farewells, Identity, Capabilities, Hello-world programs, Unknown fallback) and renders each row as a `When X then Y` statement; runtime rules appear in a dedicated `Dialog-local rules` section.

## [0.81.0] - 2026-05-20

Added procedural "how to X Y" handling that decomposes action/object requests,
checks Wikimedia/wikiHow sources first, and records web-search plus recursive
fetch fallback evidence instead of returning the unknown fallback.

## [0.80.0] - 2026-05-20

### Fixed
- For URL navigation prompts, check CORS-readable frame-policy metadata before
  rendering an iframe preview. Pages that send blocking `X-Frame-Options` or
  CSP `frame-ancestors` headers now get a polite direct new-tab link instead of
  a broken embedded preview. Markdown links in chat messages now open in a new
  tab and show an external-link indicator.

## [0.79.0] - 2026-05-20

### Fixed

- Recognize test and liveness prompts such as `Test`, `test passed`, and `I'm here` as `test_status` instead of unknown.

## [0.78.0] - 2026-05-20

### Fixed
- Issue #145: Feature capability questions such as "Can you search the internet?", "Can you do arithmetic?", and "Ты можешь искать в интернете?" now resolve to localized capabilities answers instead of the unknown fallback, with runtime-aware availability for web search, diagnostics, agent mode, and definition fusion.

### Added
- Issue #145: The demo chat now accepts message-driven settings/actions for existing controls such as diagnostics, theme, UI language/style, agent/demo mode, attachments, issue reports, and memory visibility.

## [0.77.0] - 2026-05-20

### Added

- Issue #180: every `solve()` turn now ends with a `deformalize` reasoning
  step that projects the resolved formalization back to natural language. A
  new `formalization-context` payload is threaded through every handler so
  the worker can emit `formalize → <handler> → formalize_resolved →
  deformalize` for the fact-style and `web_search` flows, and `formalize →
  <handler> → deformalize` for greeting / unknown / agent / memory flows. The
  diagnostics row carries the worker-emitted `projection.summary` (with the
  `⇒` glyph) so the symbolic-to-natural-language hand-off is visible in the
  UI, not only in the underlying step payload.
- Google-style rendering for `web_search` results: each hit is now formatted
  as `url + title (≥ domain) + fragment containing query + "Read more"` with
  the source priority order **DuckDuckGo, Internet Archive, Wikipedia,
  Wikidata, Wiktionary, then everything else**. Duplicate Wikipedia /
  Wikidata entries for the same canonical Q-id collapse into a single bullet
  with the alternate URLs surfaced under a localized `"Другие
  источники:" / "Other sources:"` sub-line.
- Per-session CORS availability cache. Each provider is probed once per tab
  and the result is kept in RAM until the tab is closed, so unreachable
  providers no longer add latency to subsequent searches in the same session.
- Diagnostics mode: raw HTTP request / response panels per provider call and
  a unified Links Notation block per reasoning step. Every diagnostics row
  also exposes a stable `data-step` attribute so automation can assert raw
  step kinds (`impulse`, `formalize`, `formalize_resolved`, `deformalize`, …)
  without depending on the i18n-localised display label.
- Six new unit tests in `src/web_search_core.rs` pinning the issue-180
  contract: provider priority order, language-line trimming, Internet Archive
  CORS readability, the Cormack / Clarke / Buettcher RRF formula
  (`score = 1 / (k + rank)`, `k = 60`), human-readable provider labels in the
  default plan, and the default plan being a subset of the provider
  registry. All 16 `web_search_core` tests pass (10 existing + 6 new).
- Playwright spec `tests/e2e/tests/issue-180.spec.js` with three scenarios
  (greeting prompt ends with `deformalize`, unknown prompt ends with
  `deformalize`, `web_search` emits `formalize` → `formalize_resolved` →
  `deformalize` with the `⇒` projection summary). Registered in
  `tests/e2e/playwright.local.config.js`. All 136 local Playwright tests
  pass.
- Node-side smoke test `experiments/issue-180-deformalize-trace.mjs` boots
  the Web Worker inside a `vm.createContext` shim and asserts the full step
  list across the greeting / unknown / fact-style / web-search flows (24
  assertions). Useful for fast local regressions without booting Playwright.
- Issue #180 case study under `docs/case-studies/issue-180/`: the issue and
  comment payloads, the three screenshots referenced in the issue, and a
  deep dive into requirements R210–R220 (Google-style rendering, dedupe,
  priority order, session-CORS cache, dark theme, single-column menu,
  diagnostics badges, raw HTTP panels, always-on deformalize, 2× test
  coverage, case study).

### Fixed

- Dark theme parity in the new UI: topbar collapse / expand affordance and
  the source-code button now honor the active palette; broader audit of the
  remaining surfaces.
- Left menu now renders as a single column on both mobile and desktop and
  stays collapsible.
- Diagnostics badges: sizing and markup are now consistent across providers
  so long provider labels no longer break the row layout.

## [0.76.0] - 2026-05-19

### Fixed
- **Issue #160 — polite follow-up returned unknown intent.** Added a `courtesy_response` intent for phrases such as "I am fine, thank you", "thanks", "спасибо", "धन्यवाद", and "谢谢", with localized responses across the Rust solver and browser worker so small-talk acknowledgements stay in normal chat flow instead of showing the missing-rule fallback.
- Added browser courtesy-response variation that composes the acknowledgement and optional next-action question separately, with a settings slider for how often the assistant should ask/propose the next step.

## [0.75.0] - 2026-05-19

### Fixed

- Recognize "how are you?" small-talk prompts as greeting intent across English, Russian, Hindi, and Chinese instead of returning the unknown fallback.
- Add a CI coverage check that guards supported-language greeting phrases and localized conversational responses.

## [0.74.0] - 2026-05-19

### Fixed
- Browser demo percentage-of-currency prompts such as `What is 8% of $50?` now resolve as calculations before the Wikipedia fallback.

### Fixed
- Avoid accepting unrelated fuzzy Wikipedia search results for short term lookups; fall back to exact Wikidata and Wiktionary matches instead.

## [0.73.0] - 2026-05-19

### Added

- Issue #153: every reasoning step is now formalized as a deterministic
  `(Subject Verb Object)` tuple using `@USER`, `OP:<verb>`, `Q<n>`, `WP:<key>`,
  and `WT:<word>` ids regardless of the prompt's source language. Diagnostics
  mode shows the raw message, the SVO tuple, and a numbered S / V / O slot
  list through a new `FormalizationView` React component. A second
  `formalize_resolved` step folds the matched Wikidata Q-id (when one is
  found) back into the tuple as `(@USER OP:search Q89)` so the trace records
  the symbolic mapping end-to-end.
- Cross-provider deduplication for web search results. `searchWikidataEntities`
  now requests `props=sitelinks/urls`, and a new
  `canonicalEntityKey` / `buildItemMetadataIndex` / `dedupeFusedEntries`
  pipeline collapses entries returned by multiple providers (Wikipedia +
  Wikidata for the same `Q89`) into a single bullet with the other URLs
  surfaced under an `"Other sources:"` sub-line in the user's language. Each
  merge is appended to memory as
  `web_search:dedupe:<key>:absorbed:<url>` so the trace stays replayable.
- Localized search results template covering `en`, `ru`, `zh`, `hi`. Header
  (`Search results for / Результаты поиска для / 搜索结果 / खोज परिणाम`), the
  empty-state line, and the "Other sources:" sub-line all render in the UI
  language picked up from `navigator.language` / saved preferences.
- "Source code" link in the top menu, pointing to
  `https://github.com/link-assistant/formal-ai`, with i18n labels for
  `buttons.sourceCode` / `titles.sourceCode` in all four locales.
- Collapsible left sidebar for desktop, persisted through a new
  `sidebarCollapsed` preference. The mobile drawer is unchanged. A
  `[data-testid="sidebar-toggle"]` button with i18n labels
  (`buttons.collapseSidebar` / `buttons.expandSidebar`) flips the state, and
  `.workspace.sidebar-collapsed` styles the collapsed layout.
- Playwright spec `tests/e2e/tests/issue-153.spec.js` with eight scenarios
  (lab emoji, source-code link, disabled `New conversation`, sidebar collapse,
  SVO formalization view, cross-source dedupe, DuckDuckGo signature
  regression, localized search header) registered in
  `tests/e2e/playwright.local.config.js`. All 127 local Playwright tests pass.
- Issue #153 case study under `docs/case-studies/issue-153/`, including raw
  issue JSON, the three screenshots from the issue description, and a deep
  analysis of requirements R195–R209.
- Priority-based topbar overflow: every `.topbar-actions` button now carries
  a `data-menu-priority` attribute (1=highest = Bug reporting, last to drop;
  7=lowest = dynamic status indicators). New media queries at 720px and 560px
  drop priorities 7→5 then 4 out of the topbar, while the hamburger drawer
  (already wired to the same React state) remains the source of truth for
  every action. Bug reporting (1), Diagnostics (2) and Demo (3) survive to
  the narrowest viewports.
- Internet Archive (`archive.org/advancedsearch.php`) web search provider
  added to `WEB_SEARCH_PROVIDERS`. The CORS-enabled JSON endpoint returns
  ranked results across the entire collection (web captures, books, audio,
  software, …) and complements the DuckDuckGo Instant Answer fallback. Each
  hit surfaces with `sourceKind: "internet-archive"` and a deterministic
  `IA:<identifier>` virtual id so RRF and dedupe handle it like any other
  provider.
- `composer.sending` i18n key for the in-flight Send button label, plus a
  `.send-spinner` rotating ring and `send-spinner-rotate` keyframes that
  replace the ASCII `...` placeholder while a request is pending.

### Fixed

- Top bar wrapping at narrow viewports: `.brand strong` and `.demo-status`
  now use `white-space: nowrap` (and `flex: 0 0 auto` for the status pill)
  so the title and "Demo will start in …" pill never wrap awkwardly.
- Send button no longer renders as raw `... ...` while pending — it shows a
  rotating spinner and a localized `Sending…` label instead.

### Fixed

- DuckDuckGo provider was silently returning zero results because
  `searchDuckDuckGo(query, limit)` was declared with two parameters while the
  dispatcher passed three (`(query, language, providerLimit)`). The new
  signature `(query, language, limit)` coerces `limit` to a numeric cap with
  `Math.floor`, defaults to 5 when missing, and forwards a `kl=<lang>-<lang>`
  region hint when the UI language is not English. A new regression test
  proves the fix.
- The diagnostics toggle's magnifying-glass icon (🔍) was replaced with a lab
  emoji (🧪) to match the issue's request for a "diagnostics" affordance.
- `New conversation` is now disabled when the chat is empty so the click is
  no longer a no-op.

### Removed

- Stripped the `Providers (default first): duckduckgo, wikipedia, wikidata.`
  footer from search responses. Providers still appear inline next to each
  bullet (`via wikipedia#2, wikidata#1`), so no information is lost.

## [0.72.0] - 2026-05-19

### Fixed
- Kept prefilled GitHub "Report issue" URLs from the web demo within
  GitHub's 8 KB request-line cap by progressively omitting earlier dialog
  messages (keeping the last user/assistant pair) and truncating very
  long messages while preserving their start and end.

### Changed
- Compacted the "User Context" block in reported issues: UI languages
  are emitted on a single line with the active language emphasized,
  viewport/screen/user-agent/platform are folded into one `**UI**` line,
  locale/time zone into one `**Locale**` line, and the user agent line
  is dropped from "Environment". Unset preferences (UI Skin, Chat Style,
  Composer Style, Composer Action, Online, empty Preferred Location)
  are no longer emitted.
- Simplified the `**Location**` line to `inferred from <source>` form
  instead of a separate `Location Inference` paragraph.

### Added
- Added `docs/case-studies/issue-140/` with root-cause analysis, raw
  GitHub evidence, a reproduction script, and the verification log.
- Added `experiments/issue-140-prefilled-url-budget.mjs`, a standalone
  Node smoke harness that mirrors the web bundle's URL fitter so the
  8 KB cap and omission markers can be checked without a browser.

## [0.71.0] - 2026-05-19

### Added

- Issue #133: DuckDuckGo Instant Answer is now the default web search engine
  across the CLI, server, and the browser-only GitHub Pages app. The shared
  provider list `WEB_SEARCH_PROVIDERS = ["duckduckgo", "wikipedia", "wikidata"]`
  is owned by `src/solver_handlers/web_requests.rs` and mirrored by the JS
  worker so every surface dispatches the same plan.
- Combined ranking via Reciprocal Rank Fusion (Cormack, Clarke, Buettcher
  2009) with `k = 60`. Each provider returns up to its top-10 results and the
  worker merges them with `score(d) = Σ 1 / (k + rank_i(d))`, so URLs returned
  by more than one engine bubble up. The fused order is appended to memory as
  `web_search:fused:<rank>:<providers>:<url>` events.
- Per-category concurrency cap of 5 with `runWithConcurrencyLimit` in the
  worker and `CATEGORY_CONCURRENCY = 5` in the browser diagnostics page so the
  per-origin socket budget is never starved.
- Session-scoped CORS auto-disable: when a provider fetch throws a CORS or
  network error, the worker's `WEB_SEARCH_DISABLED` map (and the dashboard's
  `state.disabled` map) record the failure and skip the provider for the rest
  of the session. The decision is appended to memory as
  `web_search:disabled:<provider>`.
- Expanded browser-only diagnostics matrix at `/formal-ai/tests`: now 26
  providers across four categories.
  - `search`: DuckDuckGo (default), Google, Bing, Brave, Yahoo, Yandex,
    Ecosia, Mojeek, Startpage.
  - `knowledge`: Wikipedia, Wikidata, Wiktionary, DBpedia, Open Library,
    OpenAlex, Crossref, Semantic Scholar.
  - `papers`: arXiv, Europe PMC, DOAJ.
  - `code`: GitHub, GitLab, Codeberg, Gitee (China), Bitbucket Cloud,
    GitFlic (Russia).
- Structured `web_search:*` evidence kinds in both the Rust solver and the JS
  worker, with matching formatter branches in
  `src/event_log.rs::build_evidence_links` so the reasoning trace can be
  replayed offline.
- Issue #133 case study under `docs/case-studies/issue-133/`, including raw
  issue/PR JSON, branch log, online research notes, and a deep analysis of
  requirements R181–R194.

### Changed

- The Playwright explicit web-search regression now mocks all three default
  providers (`api.duckduckgo.com`, Wikipedia REST, `wikidata.org/w/api.php`)
  and asserts the new `web_search:provider:duckduckgo`,
  `web_search:provider:wikipedia`, and `web_search:combined:rrf:k=60` evidence
  shape.

## [0.70.0] - 2026-05-19

### Fixed
- Kept the desktop tools registry inside the side panel without horizontal overflow
  and added a draggable sidebar/chat separator for non-mobile layouts.

## [0.69.0] - 2026-05-19

### Added
- Issue #127: structured fact-query reasoning pipeline. Multilingual prompts
  about a country's capital, population, currency, official language,
  continent, area, head of state, and head of government are parsed into
  `(relation, subject, language)` triples, routed against a 1-week TTL cache
  pre-warmed from `data/seed/facts.lino`, and resolved live against Wikidata
  (`wbsearchentities` + `wbgetentities`) for any uncovered country. Each step
  is recorded in the append-only memory log as a `fact_query:*` event so the
  reasoning trace is fully inspectable.
- Pre-warmed capital cache for Russia, Japan, France, Germany, China, India,
  the United States, the United Kingdom, and Brazil — every entry carries
  multilingual `subject_aliases`, localized labels, and Wikidata Q-IDs so the
  same prompt resolves consistently across English, Russian, Hindi, and
  Chinese.
- Force-fresh markers in every supported language (e.g. "refresh", "не из
  кэша", "ताज़ा", "刷新") let users bypass the cache and force a live
  Wikidata fetch when they explicitly ask for fresh data.

### Changed
- The Rust solver now emits structured `fact_query:request`,
  `fact_query:relation`, `fact_query:subject`, `fact_query:cache:hit`,
  `fact_query:subject_qid`, and `fact_query:value_qid` evidence links
  alongside the legacy `fact_lookup:*` events so the Rust and browser stacks
  agree on the reasoning shape and on Q-ID anchoring.

## [0.67.0] - 2026-05-19

### Added

- Added the GitHub Pages `/tests` connectivity diagnostics page with direct browser fetch checks, iframe expansion, and configurable `web-capture` proxy mode.
- Added issue #129 case-study evidence and Playwright coverage for the diagnostics route in both local and Pages e2e suites.

### Changed

- Updated the Pages artifact stamping script to replace version and asset placeholders in nested HTML files, not only the root demo page.

## [0.66.0] - 2026-05-19

### Fixed
- Issue #127: Russian fact prompts like `столица россии` now resolve to the seeded Russia/Moscow capital answer instead of the unknown fallback.

### Fixed
- Recognize URL navigation prompts such as `Navigate to github.com` and show them as direct HTTPS links with iframe preview controls instead of treating them as unknown prompts.

## [0.65.0] - 2026-05-19

### Fixed
- Browser demo prompt examples now handle the reported `What you can do?`, `Search online for Genshin Impact`, `Pretend you are Albert Einstein...`, and `Купи слона` prompts without falling through to `intent: unknown`.
- Added regression coverage for the reported browser examples and the `search online for ...` web-search wording.

## [0.64.0] - 2026-05-19

### Added
- Open-ended software artifact requests now route to a `software_project_plan` answer instead of `intent: unknown`, first rendering a Links Notation meaning record with a requirement graph, subtasks, delivery mode, approval gates, reasoning, and plan steps, then returning language-aware starter code after the user approves the plan (issue #80).

## [0.63.0] - 2026-05-19

### Added
- Added deterministic cross-language definition fusion for prompts like `Merge Wikipedia definitions of IIR`, combining localized seed/Wikipedia definition blocks for the same concept anchor with source-language and citation evidence. Fixes issue #63.
- Added `SolverConfig::definition_fusion_by_default`, `FORMAL_AI_DEFINITION_FUSION`, `formal-ai chat --definition-fusion auto`, and a persisted browser Settings control so plain prompts like `What is IIR?` can opt into the same fusion path.
- Expanded definition-fusion coverage with 15 self-explanatory prompt examples across IIR, color, KISS, Links theory, and Telegram Ads, plus a negative unknown-term case.

## [0.62.0] - 2026-05-19

### Added
- Added `scripts/mine-hive-mind-dataset.rs` plus `formal-ai github-logs plan|collect` for reproducible GitHub issue, PR, review, diff, workflow-run, and run-log evidence collection into case-study directories.
- Added the issue #115 case study with formal-ai and hive-mind raw evidence captures.

### Fixed
- Keep the crates.io archive below the upload limit by publishing only source/runtime inputs and checking the generated `.crate` size in CI.

## [0.61.0] - 2026-05-19

### Changed
- Issue #117: browser UI translations now live in a nested Links Notation catalog loaded through `lino-i18n@0.1.1` instead of a flat JavaScript translation object.

### Added
- Added a CI catalog check that parses `src/web/i18n-catalog.lino` with `lino-i18n` and enforces complete English, Russian, Chinese, and Hindi key coverage.

## [0.60.0] - 2026-05-18

### Added
- Issue #107: Russian URL-request prompts such as `Сделай запрос к google.com` now route to the `http_fetch` intent instead of the unknown fallback.
- Explicit web-search prompts now route to `web_search`; the browser demo queries the CORS-enabled Wikipedia search endpoint and renders ranked result links.

### Documented
- Added an issue #107 case study with raw GitHub data, browser provider probe results, and web-capture capability notes.

## [0.59.0] - 2026-05-18

### Added
- Issue #112: Added mobile drawer menu actions, conversation soft-delete visibility, localized tool registry descriptions, and broader example/tool seed coverage.
- Issue #112: Added case-study evidence under `docs/case-studies/issue-112`.

### Fixed
- Issue #112: Fixed the mobile composer to auto-resize without clipping text, cap itself to half the visible chat space, and use a centered CSS menu icon.

## [0.58.0] - 2026-05-18

### Added
- Issue #110: Added whole-UI skin settings (`flat`, `glass`, `contrast`) and chat message style settings (`cards`, `compact`, `bubbles`) alongside the existing input style controls.
- Issue #110: Added focused mobile keyboard viewport coverage that simulates `visualViewport.offsetTop`.

### Fixed
- Issue #110: Anchored the web app shell to the visual viewport offset so the mobile topbar and chat remain reachable when the on-screen keyboard pans the viewport.

## [0.57.0] - 2026-05-18

### Added
- Issue #108: Added configurable composer input styles (`flat`, `glass-soft`, `glass-clear`, `bubble`) and configurable composer action icons (`attach`, `plus`) to the web demo settings.
- Issue #108: Added a composer-adjacent action menu for attachments, memory export/import, and report issue actions.
- Issue #108: Added a mobile UI case study with source issue data, screenshots, research notes, and verification evidence.

### Fixed
- Issue #108: Reworked the mobile topbar and composer so the menu stays reachable, the logo/title/version live inside the drawer, and the input remains a compact one-row control above mobile browser UI.
- Issue #108: Show the app version next to the logo/title on desktop.

## [0.56.0] - 2026-05-17

### Fixed
- Recognize Russian word-number arithmetic such as "Сколько будет два плюс два?" as a calculation instead of falling through to the unknown-intent response.

## [0.55.0] - 2026-05-17

### Added
- Added the formal-ai deterministic symbolic implementation library, CLI, OpenAI-compatible JSON API server, Docker packaging, and GitHub Pages demo.
- Added a markdown-capable chat demo with randomized dialog mode, greeting-first examples, and multi-language hello-world prompts.
- Added Rust-script dataset generation into human-readable Links Notation `.lino` files under `data/`.
- Added Playwright e2e test suite under `tests/e2e/` covering page load, initial messages, quick prompts, chat interactions, demo mode toggle, trace panel, and preview mode.
- Added `test-e2e-local` CI job that serves the demo with a local HTTP server and runs Playwright tests on every PR.
- Added `test-e2e-pages` CI job that runs Playwright tests against the live GitHub Pages URL after each deployment.

### Changed
- Expanded deterministic hello-world responses from Rust-only to Rust, Python, JavaScript, TypeScript, Go, and C.
- Extended repository file-size checks to enforce the 1500-line limit for `.lino` dataset chunks.
- Fixed GitHub Pages deployment to publish the React chat demo from `docs/demo/` instead of Rust API documentation.

### Added
- Added a TDD-style full-scope test suite under `tests/unit/specification/` that pins down the chat surface, code generation, multilingual chat, OpenAI compatibility, Telegram surface, links network, reasoning loop, source cache, agent isolation, translation-via-Links, network visualization, and transparent-state requirements drawn from `VISION.md`, `GOALS.md`, `NON-GOALS.md`, and `docs/REQUIREMENTS.md`. Tests describing not-yet-implemented full-scope behavior are marked `#[ignore = "tracked requirement: ..."]` so they document expectations without blocking CI; run them locally with `cargo test -- --include-ignored`.

### Added
- Issue #103: New `tests/unit/specification/prompt_variations.rs` test module with 5-10 input variations per category (greetings, farewells, identity, clarification, concept lookups, capabilities, hello-world, basic math, refusal, idioms) translated across English, Russian, Hindi, and Chinese. The module ships generalized helpers (`assert_intent_for_each`, `assert_language_for_each`, `assert_answer_contains_for_each`, etc.) so per-language matrices stay declarative.
- Issue #103: New `ARCHITECTURE.md` describing the evolving architecture — context assembly, Links Notation translation, Wikidata P-id/Q-id formalization, temperature-driven interpretation selection, doublets-rs/doublets-web persistence, internet-as-public-database with local cache, and the five transformation-rule shapes (data rules, Rust handlers, JS handlers, dynamic compilation, natural-language skills).
- Issue #103: New `docs/case-studies/issue-103/` case study folder with collected raw data, competitor-test research, and the holistic plan that ties R129-R136 together.
- Issue #103: Added deterministic solver handlers for summarization, brainstorming, factual Q&A with Wikidata anchors, multi-turn coreference, and roleplay so the competitor-derived prompt categories run as active regression tests.

### Changed
- Issue #103: Updated `VISION.md` with a new "Formalization And Temperature" section, expanded "Computation Model" coverage of the five transformation-rule shapes, and meaning-and-identity language tying together natural-language ↔ programming-language translation via Links Notation.
- Issue #103: Updated `REQUIREMENTS.md` with the new R129-R136 entries and the "Issue #103 Test-Matrix And Architecture" matrix that traces each requirement to its enforcing test or document.
- Issue #103: Updated `tests/unit/docs_requirements.rs` to pin the documentation surface for issue #103 alongside the existing issue #12 and issue #16 traceability tests.

## [0.54.0] - 2026-05-17

### Added
- Added an Assistant behavior settings section to the web sidebar with controls for ambiguity handling, temperature, UI language, theme, location, and greeting variations.
- OpenAI-compatible Chat Completions and Responses requests now accept an optional `temperature` parameter, and `SolverConfig` can read `FORMAL_AI_TEMPERATURE`.

### Fixed
- Russian typo prompts such as `что такое граматика` now resolve through Wikipedia search as the closest match by default, while low-guessing settings ask the user to clarify before using that fuzzy match.

## [0.53.0] - 2026-05-17

### Fixed
- Issue #64: Russian prompts like «Расскажи о теории связей» now resolve to the Link Foundation `meta-theory` concept instead of the generic unknown-intent fallback. The seeded answer cites `https://github.com/link-foundation/meta-theory` and notes that similarly named theories may refer to different domains.

## [0.52.0] - 2026-05-17

### Fixed
- Treat compact calculation prompts like `2*2+2=?` as calculator requests.

### Fixed

- Fixed Russian combined prompts such as "Привет. ты кто?" so the browser worker preserves Unicode words during intent routing and answers with the identity rule instead of `unknown`.

### Fixed
- Solved simple single-variable linear equations such as `x*2 = 123` instead of returning the unknown-intent fallback.

## [0.51.0] - 2026-05-17

### Added
- Delegate calculator-parsable expressions to `link-calculator`, including unit, currency, percentage, datetime, and function-style calculator prompts.
- Add multilingual calculator delegation tests, calculator tool seed metadata, demo simulator prompts, and issue #96 case-study documentation.

### Fixed
- Preserve the local arithmetic fallback for English word operators and binary `%` remainder expressions until those cases are supported by `link-calculator`.

## [0.50.0] - 2026-05-16

## Added

- Added automatic UI language detection for English, Russian, Chinese, and Hindi in the browser demo.
- Added `lino-i18n@0.0.1` as the browser demo's translation runtime with a local fallback catalog.
- Added user UI/browser context to agent requests, issue reports, and full-memory exports.
- Added a case study for Issue #94 with issue assets, research notes, and implementation tradeoffs.

## Changed

- The browser demo now follows `prefers-color-scheme` for dark mode.
- The browser demo now localizes action tooltips, message metadata, fetched-page controls, memory action responses, and tool mode labels.
- Topbar action labels now switch to icon-only at wider breakpoints before the controls wrap.

## [0.49.0] - 2026-05-16

### Fixed
- **Report issue** dialog annotation now marks the reported message with `intent: <intent>, reported` for any intent, not only `unknown` (issue #73). Previously, clicking "Report issue" on a TypeScript hello-world response (intent: `hello_world_typescript`) produced a prefilled GitHub issue body with no annotation on the reported message — a maintainer could not tell which turn was considered problematic. The `appendDialogBlock` function now always adds `intent: <intent>` and `reported` to the focused message regardless of its intent value.
- Updated E2E Playwright coverage in `tests/e2e/tests/demo.spec.js`: the known-dialog report test now asserts `A (intent: …, reported):` appears in the body, and a new test verifies that a TypeScript hello-world dialog report includes `A (intent: hello_world_typescript, reported):`.

## [0.48.0] - 2026-05-16

### Changed
- **Report issue** prefilled body is now short enough to fit GitHub's `/issues/new?body=…` URL-length limit (issue #78). The verbose memory-upload instructions (`.zip` walkthroughs per OS, redaction reminders, full-memory explainer) moved out of the prefilled body and into a single repository doc, [`docs/upload-memory.md`](docs/upload-memory.md). The body now references that page with a one-line link instead of repeating the workflow each click (R112, R115, R117).
- Dialog transcripts inside the prefilled body now render as a single fenced code block with `U:` / `A:` line prefixes (issue example: `U: 1+2` / `A: 3`) instead of one Markdown subsection per message. Known-intent turns stay as plain `A: …`; only the `unknown` intent keeps the inline annotation (`A (intent: unknown, reported): …`) where the marker is needed to identify the missing rule, so the encoded `body=` parameter stays comfortably below GitHub's request-line cap (R116).

### Added
- [`docs/upload-memory.md`](docs/upload-memory.md): single canonical guide that explains what *full memory* means, walks through **Export memory**, redaction, and the two upload paths (GitHub Gist with no extension restrictions, or `.zip` for issue attachments), and documents why `.lino` is not yet a native attachment type (R117, R118).
- `docs/case-studies/issue-78/` with raw GitHub data, mirrored issue screenshots, root-cause analysis (the encoded `?body=` query string overflows the 8192-byte URL cap once the transcript reaches ~5 turns), the R115–R119 requirement matrix, and verification notes.
- Playwright e2e coverage in `tests/e2e/tests/multilingual.spec.js` and `tests/e2e/tests/demo.spec.js`: the Report-issue test now asserts the body links to `docs/upload-memory.md` and no longer contains the long per-OS Compress / Send-to instructions; the dialog-shape tests assert the `Legend: \`U\` = user, \`A\` = agent.` block, `U: …` / `A: …` line prefixes, and the absence of the old `### 1. …` / `- **Role**: …` subsections (R119).
- `experiments/issue-78-dialog-format.mjs` smoke script that prints the compact transcript for a hand-crafted set of cases (empty, greeting, unknown prompt, fenced code block, arithmetic dialogue).

## [0.47.0] - 2026-05-16

### Fixed
- **Issue #67 — "пока" not recognised as a valid prompt.** Added a new `farewell` intent to `intent-routing.lino` with keywords for "bye", "goodbye", "пока", "ciao", "再见", "अलविदा" and phrases "до свидания" / "досвидания". Added multilingual farewell responses for English, Russian, Hindi, and Chinese in `multilingual-responses.lino`, farewell examples in `greetings.lino`, and keyword/phrase patterns in `prompt-patterns.lino`. Wired the `Farewell` variant into the Rust engine (`SelectedRule`, `select_rule_for`, `language_aware_answer_for`) and the browser worker (`isFarewellPrompt`, `solve`). The agent now responds with a language-appropriate goodbye instead of the unknown-intent fallback.

### Added
- Issue #71: `fetch <url>` prompts are now recognised as `http_fetch` intent instead of falling through to `unknown`; the assistant returns a structured response that triggers a browser `fetch()` request and falls back to an embedded iframe when CORS blocks the direct request

## [0.46.0] - 2026-05-16

### Fixed
- Recognize English `who is X` variants such as `Tell me, who is Trump` and `Who Trump is` as concept/Wikipedia lookup prompts instead of falling back to `intent: unknown`.

### Fixed
- Issue #65: punctuation-only prompts such as `.` now ask a clarification question instead of returning the generic unknown-intent fallback.

## [0.44.0] - 2026-05-16

### Fixed
- Issue #84: `scripts/publish-crate.rs` now classifies crates.io HTTP 429 rate-limit responses ("You have published too many versions of this crate in the last 24 hours") as `publish_result=rate_limited` with a dedicated banner explaining that `scripts/check-release-needed.rs` will automatically retry on the next push to `main` once the 24-hour throttle window has rolled over, instead of surfacing the failure as the misleading "Failed to publish for unknown reason"

### Added
- `FailureKind` enum and `classify_failure` helper in `scripts/publish-crate.rs` with unit tests covering rate-limit, already-uploaded, already-exists, missing-token, unauthorized, and unknown-failure responses
- `docs/case-studies/issue-84/` case study capturing the failing CI runs, root cause, upstream template comparison and reproduction commands

## [0.43.0] - 2026-05-16

### Added
- `pattern_concept_prefix_rasskazhi_za` and `pattern_concept_prefix_rasskazhi_mne_za` prompt patterns in `data/seed/prompt-patterns.lino` so the colloquial Russian prefix «расскажи за» triggers `concept_lookup` instead of falling through to unknown-intent
- `concept_telegram_ads` entry in `data/seed/concepts.lino` with English and Russian localisations, aliases, and an official source so queries about Telegram Ads are answered from the knowledge base

### Fixed
- Issue #66: «Расскажи за Telegram Ads» now resolves to `concept_lookup` and returns a factual summary about Telegram Ads instead of the generic «я пока не знаю символьного правила» fallback

## [0.42.0] - 2026-05-16

### Fixed
- Issue #72: GitHub Pages demo no longer advertises a stale hardcoded version. `src/web/index.html` now uses a `__FORMAL_AI_VERSION__` placeholder that `scripts/stamp-pages-artifact.sh` substitutes from `Cargo.toml` during the Pages deploy.

### Added
- CLI `--version` flag prints `formal-ai <CARGO_PKG_VERSION>` via clap's `version` attribute.
- Telegram `/version` (and `/version@formal_ai_bot`) command replies with `formal-ai <CARGO_PKG_VERSION>`.

## [0.41.0] - 2026-05-16

### Fixed
- Issue #39: Queries asking whether formal-ai performed a physical action (e.g. Russian «Сосал?») are now answered factually ("No. I have no physical body.") via a new `try_physical_action_question` handler instead of being refused as inappropriate content
- Removed «сосал»/«сосёшь»/«соси»/«сосать» from the vulgar-content word list since these words describe physical actions and deserve a factual response, not a policy refusal

## [0.40.0] - 2026-05-16

### Fixed
- Handle "who is X" prompts with typo correction: queries like "who is elon mask" now suggest "Elon Musk" instead of returning an unknown intent error

## [0.39.0] - 2026-05-16

### Fixed
- Issue #70: Prompts like "what is tesla" that match a Wikipedia disambiguation page now fall back to the full-text search endpoint to find the top-ranked article (e.g. "Tesla, Inc.") instead of returning an unknown-intent error

## [0.37.0] - 2026-05-16

### Fixed
- Russian capability phrases (e.g. "что ты умеешь?") now correctly map to the capabilities intent and return a Russian-language answer
- Russian confusion phrases (e.g. "я не понимаю") now correctly map to the confusion intent instead of being misidentified

### Added
- `try_kupi_slona` handler in `src/solver_handlers.rs` that recognises the Russian circular-joke idiom «Купи слона» and returns the traditional reply
- `kupi_slona` intent wired into `handle_specialized_pattern` in `src/solver.rs`
- 3 new unit tests in `tests/unit/mvp/multilingual.rs` covering the idiom intent, answer content, and Russian language tag

### Fixed
- Issue #41: «Купи слона» no longer falls through to the generic unknown-intent fallback; it is handled with a culturally appropriate Russian explanation of the folk game

## [0.36.1] - 2026-05-16

### Added
- Generic "write a script in \<language>" requests now route to the matching code block instead of returning `intent: unknown`. Supports English ("write a script in Python"), Russian with inflected language names ("Напиши скрипт на питоне", "расте", "джаваскрипт"), Hindi, and Chinese phrasing (issue #35).

## [0.36.0] - 2026-05-16

### Added
- `try_incompatible_units` handler: queries that mix dimensionally incompatible units
  (e.g. meters vs kilobytes) now return `intent:unit_incompatibility` with a clear
  symbolic explanation instead of falling through to `intent:unknown` (fixes #43).
- Five new `reasoning_paths` tests covering the Russian prompt from the bug report
  (`"Сколько метров в килобайте?"`), the English equivalent, evidence-link emission,
  and regression guards for greetings and arithmetic.

### Fixed
- **Issue #50 — "шабат шалом!" not recognised as a greeting.** Added `шалом` as a greeting keyword and `шабат шалом` as a greeting phrase to `intent-routing.lino`, `greetings.lino`, and `prompt-patterns.lino`. The agent now routes these Hebrew-origin greetings (common in Russian-speaking communities) to the `greeting` intent and responds in Russian instead of returning the unknown-intent fallback.

### Fixed
- Russian prompts such as "покажи как ты работаешь?" now correctly resolve to `intent: meta_explanation` instead of falling back to `intent: unknown` (#51)

### Added
- Multilingual responses for the `meta_explanation` intent (English, Russian, Hindi, Chinese) so the agent explains how it works in the user's language
- Pattern recognition for "how do you work" / "show me how you work" style queries in English, Russian, Hindi, and Chinese
- Prompt patterns for `meta_explanation` intent in the routing seed

### Added
- Opinion question intent (`opinion_question`) that handles prompts like "Do you think space is continuous or discrete?" with a deterministic explanation instead of the generic unknown-intent error
- `try_opinion_question` handler in `solver_handlers.rs` detecting opinion/belief phrasings across multiple patterns
- Tests pinning the opinion question intent for the exact prompt from issue #42 and five related phrasings

### Fixed
- Issue #42: Opinion-style questions such as "Do you think space is continuous or discrete?" now return a helpful deterministic explanation instead of the confusing "I do not have a learned symbolic rule" fallback

## [0.35.0] - 2026-05-16

### Fixed

- Inappropriate or vulgar prompts (e.g. Russian mat) now receive a polite policy refusal (`intent: policy_inappropriate_content`) with a language-matched response instead of the generic "intent: unknown" fallback. Applies to Russian, Hindi, Chinese, and English content. Fixes issue #39.

### Added
- Russian "назови " prefix recognized as a `concept_lookup` intent trigger (issue #30). The prompt "назови цвет" previously returned `intent: unknown`; it now resolves to `concept_lookup` and returns a definition of the color concept.
- `concept_color` seed record in `data/seed/concepts.lino` with full multilingual support (English, Russian, Hindi, Chinese), Wikidata anchor Q1075, and per-language localized blocks citing Wikipedia in each language.
- Two regression tests in `tests/unit/mvp/multilingual.rs` pinning down the reporter's exact prompt: `russian_nazovi_prefix_routes_to_concept_lookup` and `russian_nazovi_tsvet_answer_references_color`.
- `DEFAULT_CONCEPT_PREFIXES` fallback in `src/web/formal_ai_worker.js` updated to include "назови " so the browser worker mirrors the Rust pipeline when the seed has not yet been loaded.

### Fixed
- **Issue #44 — Topbar "Report issue" generates misleading title when session contains unknown-intent responses.** `createIssueTitle` and `createIssueReportBody` now fall back to the last `intent: unknown` assistant message as the effective focus when the user clicks the topbar button (no per-message `focusMessage`). This ensures the generated GitHub issue title reads `Unknown prompt: <prompt>` and the dialog body marks the relevant message as `(reported message)`, matching the behaviour already seen when clicking the per-message "Report missing rule" link.

## [0.34.0] - 2026-05-15

### Fixed
- Russian phonetic transliterations "хелло" and "ворлд" are now recognized as valid hello/world tokens, and Russian language names "питоне" (Python), "расте" (Rust), and "джаваскрипт" (JavaScript) are now matched as language aliases. Previously, prompts like "Напиши хелло ворлд на питоне" fell through to `intent: unknown` (issue #53).

## [0.33.0] - 2026-05-15

### Fixed
- `scripts/version-and-commit.rs` now updates the workspace-package entry in `Cargo.lock` in the same release commit that bumps `Cargo.toml`. Previously every release left `Cargo.lock` stale, forcing follow-up "sync Cargo.lock" commits and producing avoidable merge conflicts on every open PR.

### Fixed
- Follow-up prompts such as "how it works?" or "how does it work?" after a concept lookup no longer return `intent: unknown`. A new `try_how_it_works` handler recognises these elaboration patterns, extracts the topic from an inline subject ("how does Wikipedia work?") or from the prior assistant reply, and either re-runs a concept lookup or returns a meaningful fallback. Five new regression tests cover the bare form, the explicit-subject form, the multi-turn history form, and the evidence-link audit requirement (issue #52).

## [0.32.0] - 2026-05-15

### Fixed
- Large integer multiplication no longer returns an overflow error; integer-only expressions now use arbitrary-precision arithmetic so results like `123123980921093128 * 2348023048230429324 * …` are exact (issue #55).

## [0.31.0] - 2026-05-15

### Fixed
- Added a visible hint message in the composer when demo mode is active, explaining how to stop the demo and type a custom message. Fixes confusion on Android and other mobile browsers where the demo-status text and button labels are hidden.

## [0.30.0] - 2026-05-15

### Added
- Clarification intent that handles "не понял" and similar phrases across Russian, English, Hindi, and Chinese
- Multilingual clarification responses in `data/seed/multilingual-responses.lino`
- `try_clarification` handler in `solver_handlers.rs` with language-aware response selection
- Tests pinning the clarification intent for Russian ("не понял") and English ("I don't understand", "I didn't understand")

### Fixed
- Issue #29: "не понял" (Russian: "I didn't understand") now returns a helpful explanation of what formal-ai can do instead of the generic unknown-intent fallback

## [0.29.0] - 2026-05-15

### Fixed
- KISS queries in a programming context (e.g., "что такое Kiss в рамках програмирования") now return the KISS software design principle instead of the rock band. Fixed by adding the " в рамках " Russian context delimiter, extending `context_programming` aliases with genitive/locative misspellings, adding an offline `concept_kiss_principle` corpus entry, and adding a context-aware Wikipedia search fallback in the browser worker.

## [0.28.0] - 2026-05-15

### Fixed
- **Issue #37 — Russian and Chinese conversation-summary phrases not recognised.** The `try_summarize_conversation` handler now matches Russian phrasings (`о чём мы разговаривали`, `о чём мы говорили`, `резюме беседы`, `резюмируй разговор`, bare `резюме`) and the Chinese shorthand (`总结`), as well as the English `summarize this conversation` and bare `summarize`, so users asking "О чём мы разговаривали?" receive the conversation summary instead of the unknown-intent response.

## [0.27.0] - 2026-05-15

### Added
- **Issue #27 — Mobile-responsive topbar and slide-out drawer.** Topbar buttons (Report issue, Export memory, Import memory, Diagnostics, Chat/Agent, Demo) now render an emoji icon next to their label; under the 820 px breakpoint the label collapses and only the emoji is shown while the full action name stays available via `aria-label` and `title`. A hamburger toggle (`☰` / `✕`) surfaces below the same breakpoint and slides the conversations / prompts / tools sidebar in over the chat as a drawer with a tappable backdrop, so the chat surface keeps the full viewport on phones. Three new Playwright tests under `Issue #27: mobile layout` pin the behavior at a 390×780 viewport.
- **Issue #27 — Chat / Agent mode toggle.** A new topbar toggle decomposes a single user message into sequential steps (`;` / `then` / Russian and Chinese equivalents) and executes each step through the existing deterministic solver, rendering the aggregated plan + per-step result as Markdown. Single-step prompts keep the plain Q&A surface. Covered by three e2e tests.
- **Issue #27 — Deterministic summarize skill.** A logical (no neural-network) summarizer projects the live conversation onto turn counts, languages, intents, concepts, calculations, hello-world programs, and unanswered questions, then renders a Markdown report. Triggered by EN / RU / HI / ZH phrasings (`summarize`, `резюме беседы`, `सारांश`, `总结`, …). The trigger now runs before normalization bails on non-Latin scripts, and `conversationHistory()` carries intent / evidence so per-turn classification survives into the summary. Four new e2e tests.
- **Issue #27 — Conversations sidebar.** Every appended event is now tagged with `conversationId` / `conversationTitle`, so the append-only log can be grouped per chat thread on read. The sidebar lists known threads (most-recent first), click-to-switch hydrates the transcript from IndexedDB, the `+ New conversation` button mints a fresh thread, and a new `currentConversationId` preference restores the last active conversation on reload. Three e2e tests under `Issue #27: conversations sidebar`.
- **Issue #27 — Random greeting variations.** Variant entries for the four greeting languages live alongside the canonical `text` in `data/seed/multilingual-responses.lino`; the JS seed loader returns `{ text, variants }` per response while the Rust parser ignores `variant` siblings so the shared seed stays compatible. A preference toggle exposes the random-variant selection and pins the canonical greeting when disabled.
- **Issue #27 — Natural-language Export/Import memory.** Typing `Export memory` / `Export your memory` (and ru / hi / zh translations) into the chat now triggers the Export memory toolbar action and produces an assistant acknowledgement; same for `Import memory` → Import memory dialog. The phrase recognizer normalizes casing / punctuation / whitespace before matching against a curated seed list so the chat surface and the sidebar stay in lock-step.
- **Issue #27 — Natural-language cross-conversation recall.** Typing recall prompts like `When did I ask about Rust?`, `search my conversations for Wikipedia`, `find Donald Trump in another conversation` (and ru / zh equivalents — `Когда я спрашивал про Илона Маска?`, `найди Илон Маск в другой беседе`, `我什么时候问过 Rust`, `在对话中搜索 Donald Trump`) now scans the IndexedDB event log, groups matching turns by `conversationId`, and renders a Markdown report with timestamps and excerpts. A scope suffix (`in another conversation` / `в другой беседе` / `在其他对话中`) restricts the search to threads other than the current one; otherwise all conversations are scanned. Two new Example prompts surface the skill in the sidebar, and four new e2e tests under `Issue #27: cross-conversation recall` pin the EN/RU phrasings, the cross-conversation scope filter, and the no-match path.
- `tests/e2e/playwright.adhoc.config.js` — Playwright config that listens on port 3499 to sidestep stale dev servers occupying 3456.
- `docs/case-studies/issue-27/` — raw issue + PR snapshots used as input to the case study.
- `docs/screenshots/desktop-topbar.png`, `docs/screenshots/mobile-topbar-closed.png`, `docs/screenshots/mobile-drawer-open.png` — visual regression baselines for the new responsive topbar.

### Fixed
- **Issue #27 — `Кто такой Илон Маск?` Wikipedia lookup.** Russian Wikipedia biographies use the `Surname, Given names` slug form (e.g. ru.wikipedia.org redirects `Илон_Маск` to `Маск,_Илон` and the REST summary endpoint 404s on the former). `wikipediaTermVariants` now appends the swapped two-word form so the lookup hits the canonical biography slug. Covered by a new e2e test that returns 404 for every variant except `Маск,_Илон` and asserts the chat renders the biography.
- **Issue #27 — Sidebar accordion regression coverage.** The VS Code-style sidebar (expanded sections share height via `flex: 1 1 0` and each body has `overflow: auto`) is now pinned by three Playwright tests under `Issue #27: sidebar accordion`: equal-height expanded sections, independent scroll on each body, and collapsing a section grows its siblings.

### Changed
- **Issue #27 — Demo cycle iterates the Example prompts list.** `createDemoTurns` now reads directly from `EXAMPLE_PROMPTS` and advances persistent `demoGreetingCursor` / `demoFeatureCursor` indices, so each demo cycle visits a new greeting + feature prompt in deterministic rotation (instead of repeatedly picking from a tiny hard-coded `DEMO_LANGUAGES` / `DEMO_GREETINGS` subset). Export / Import are filtered out since they would trigger file downloads. Each demo user message now exposes the source label via `data-demo-label` for regression testing.
- Replaced the `Download bundle` button with an alias of `Export memory` so the action stays available under both names while collapsing the duplicate toolbar entry, per the issue body.
- `tests/e2e/tests/demo.spec.js`: the `quick prompts sidebar` test now selects `Rust` by label instead of by absolute index because the example-prompts list now leads with multilingual greetings; the `unknown prompts …` test now uses a nonsense prompt that cannot resolve so the unknown-intent path is actually exercised.
- The Chat/Agent toggle test asserts on the `.btn-label` span because the topbar buttons now render `btn-icon` + `btn-label` so the label can collapse on the mobile breakpoint.

## [0.26.0] - 2026-05-15

### Added
- Context-aware concept lookup for "what is X in Y" style prompts in English, Russian, Hindi, and Chinese: the solver now models a definition query as `(concept, optional context)` and ranks records whose `contexts` list matches the user-supplied context phrase (issue #20). The trigger prompt `что такое iir в ml`, which previously returned `intent: unknown`, now resolves to `concept_lookup_in_context` with a localised response template per language.
- New `context_delimiter` pattern kind in `data/seed/prompt-patterns.lino` describing how to split "concept" from "context" per language (` in `, ` for ` for English; ` в `, ` для ` for Russian; ` में `, ` के लिए ` for Hindi; `中`, `中的`, `领域的` for Chinese). Hindi and Chinese also place context before the concept, so the ranker retries with swapped halves.
- New `contexts` field on `ConceptRecord` (Links Notation `context "..."` line) listing applicable domains in every supported language; the IIR seed entry covers signal processing and machine-learning aliases (`ml`, `машинное обучение`, `मशीन लर्निंग`, `机器学习`, `digital signal processing`, etc.).
- New `concept_lookup_in_context` intent with localised response templates in `data/seed/multilingual-responses.lino` (`В контексте «{context}»…`, `In the context of {context}…`, `{context} के संदर्भ में…`, `在「{context}」的语境下…`).
- Append-only event-log debug surface for the concept handler: every lookup emits `concept_lookup:request`, `concept_lookup:context` (when a context is parsed), and one of `concept_lookup:hit` / `concept_lookup:miss` / `concept_lookup:context-match` / `concept_lookup:context-mismatch`. The events are exposed through `evidence_links` in `--format chat` and `--format responses`, satisfying the verbose-trace requirement from the issue without a new flag.
- `docs/case-studies/issue-20/` with raw GitHub data, timeline, root-cause analysis, comparison against Wikipedia/Wikidata/schema.org disambiguation, and per-requirement solution plan.
- New `data/seed/concept-contexts.lino` registry of disambiguating contexts (`context_machine_learning`, `context_signal_processing`, `context_neural_network`, `context_programming`) anchored by Wikidata Q-IDs (Q2539, Q1058710, Q192776, Q80006) with per-language `label "en|ru|hi|zh"` blocks and multilingual alias lists. The IIR record references the registry via a new `context_links "..."` field so contexts are wired as links rather than restated inline (issue #20 follow-up R8/R13).
- Per-language `localized "en|ru|hi|zh"` blocks on `ConceptRecord` carrying the native term, native aliases, native-Wikipedia summary, and source URL (e.g. `Фильтр с бесконечной импульсной характеристикой … или IIR-фильтр` from ru.wikipedia.org). The solver and JS worker prefer the prevailing-language block when rendering the response, so the original maintainer prompt `что такое iir в ml` now returns a Russian-language definition citing the Russian Wikipedia article (issue #20 follow-up R9/R10/R11).
- New `concept_lookup_in_context_no_alias` response template variant: when the user already typed the localized context label, the body renders as `В контексте Машинное обучение IIR …` instead of `«Машинное обучение» (Машинное обучение)`. The variant is picked by template id from `multilingual-responses.lino` so future languages add a row without Rust changes.
- `wikidata "Q…"` field on every concept and new `wikidata:Q…` evidence link so the cross-language join key used by `link-assistant/human-language` and `link-assistant/meta-expression` is part of the public trace (issue #20 follow-up R13).
- Additional Rust integration tests in `tests/unit/mvp/multilingual.rs` pinning down native-language body content (Russian, English, Hindi, Chinese), the `«ml» (Машинное обучение)` rendering, the ru.wikipedia.org source citation, the no-alias-duplication template, and the Wikidata anchor in `evidence_links`.

### Changed
- `concepts.rs` now exposes `ConceptQuery` / `ConceptLookup` and the context-aware `extract_concept_query` + `lookup_concept_query`; the prior single-string `extract_concept_term` / `lookup_concept` shims have been removed. The ranker resolves context phrases through the new registry: a record can declare `context_links "context_machine_learning|…"` and `record_has_context` will follow the registry rather than require every alias inline.
- `src/web/formal_ai_worker.js` mirrors the Rust pipeline: it parses the context delimiter, emits the same `concept_lookup:*` and `wikidata:*` evidence labels, picks the `_no_alias` template when needed, and renders both plain and in-context bodies from the localized block selected by `detectLanguage(prompt)`.
- `tests/unit/mvp/multilingual.rs` pins down the four-language variants of the trigger prompt and the evidence-link debug trail; `src/solver_helpers.rs` gained unit tests for context delimiter parsing across the four languages.

## [0.25.0] - 2026-05-15

### Fixed
- Wikipedia and concept-lookup source URLs now render in human-readable IRI form (e.g. `https://ru.wikipedia.org/wiki/Изумруд`) across every formal-ai surface — web demo, CLI, HTTP server, Telegram bot, and library — while the underlying link target stays the canonical percent-encoded URI so the link still resolves (issue #21).

### Added
- `formal_ai::humanize_url` — public helper that decodes percent-encoded UTF-8 sequences in a URL while preserving reserved URI delimiters (`; / ? : @ & = + $ , #`), with `String::from_utf8` fallback for malformed input. Mirrored by `humanizeUrl` in `src/web/formal_ai_worker.js`. Covered by Cyrillic, Devanagari, CJK, Japanese, Arabic, query-string-preservation, malformed-escape, lowercase-hex, and invalid-UTF-8 unit tests.
- Playwright e2e regression test that stubs the Wikipedia REST endpoint with the exact percent-encoded URL pattern from the bug report and asserts both the readable display text and the canonical `href` are present on the rendered assistant message.
- `docs/case-studies/issue-21/` — full root-cause analysis (URI vs IRI), reproducible curl example, library survey, and upstream considerations.

## [0.23.0] - 2026-05-15

### Fixed
- Hardened the GitHub Pages demo release gate so live e2e tests wait for the exact deployed commit and load cache-busted static assets.

## [0.22.0] - 2026-05-15

### Changed
- **Export memory** now produces the full, self-contained agent state by default on every interface (issue #18). The browser **Export memory** topbar button writes `formal-ai-memory.lino` as a complete `formal_ai_bundle` — seed files (rules, concepts, tools, multilingual responses), UI preferences, environment metadata, version, and the full append-only event log — instead of only the in-session events. `formal-ai memory export` matches: it defaults to the full bundle and accepts `--events-only` to opt back into the legacy `demo_memory` shape (R109, R113).

### Added
- `FormalAiMemory.exportFullMemory({ seed, events, preferences, info })`, `FormalAiMemory.importFullMemory(text)`, and `FormalAiMemory.suggestMigrations({ imported, current })` in `src/web/memory.js`, mirrored on the Rust side by `memory::export_full_memory`, `memory::import_full_memory`, `memory::suggest_migrations`, `BundleInfo`, and `ParsedBundle` (re-exported from `formal_ai`'s crate root) so embedders writing their own surface get the same defaults (R109, R110, R111, R113).
- Header-agnostic imports: both `formal-ai memory import` / `formal-ai bundle import` (CLI) and `handleImportMemory` (browser) auto-detect `formal_ai_bundle` and legacy `demo_memory` documents so older exports keep round-tripping (R110).
- Seed-version migration suggestions on import. When the imported bundle's `agent_info.version` differs from the running app, the browser memory-status indicator and the CLI (`Migration: <message>` via `eprintln!`) tell the user what changed in `data/seed/` so they can take action without reading code (R111).
- Rewritten **Attach full memory** block in the prefilled "Report issue" body: explicitly tells users that the export is the full memory of the agent, walks them through wrapping it in a `.zip` on macOS / Windows / Linux (GitHub does not yet accept `.lino` attachments), and reminds them to redact sensitive content (R112).
- `docs/case-studies/issue-18/` with raw GitHub data, root-cause analysis, requirement-to-implementation mapping (R109-R114), and verification notes.
- Playwright e2e coverage in `tests/e2e/tests/multilingual.spec.js`: `Export memory downloads a full formal_ai_bundle by default` asserts `formal_ai_bundle` / `seed_files` / `preferences` in the downloaded file; `Import memory accepts a formal_ai_bundle and reports seed migrations` exercises the auto-detect path and the migration-suggestion surface; the report-issue test now also asserts the `.zip` and redaction guidance (R114).
- Rust unit coverage in `src/memory.rs`: `full_memory_round_trip_preserves_seed_preferences_and_events`, `import_full_memory_accepts_legacy_demo_memory`, `suggest_migrations_flags_seed_version_drift`, `suggest_migrations_flags_legacy_only_import`, `suggest_migrations_is_quiet_when_versions_match`.

## [0.21.0] - 2026-05-15

### Added
- Multilingual chat across Rust, CLI, HTTP server, Telegram bot, and the web demo: greetings, identity, unknown-answer, and concept-lookup handlers reply in Russian, Hindi, and Chinese in addition to English; the input language is detected from Unicode blocks (Cyrillic, Devanagari, CJK) (issue #16).
- Browser Wikipedia REST fallback for `What is X?` prompts that miss the offline `CONCEPTS` table; the offline corpus still wins and the network answer carries `source:` evidence so the citation can be audited.
- `data/seed/` as the canonical knowledge surface for every interface: `src/seed.rs` `include_str!`-embeds every `.lino` file and exposes typed accessors (`multilingual_responses`, `concepts`, `tools`, `language_rules`, `prompt_patterns`, `intent_routing`) that the Rust library, CLI binary, HTTP server, and Telegram webhook all read; the browser worker fetches the same files through `src/web/seed_loader.js`.
- Data-driven intent routing via `data/seed/intent-routing.lino` with explicit `keyword` / `phrase` / `token` / `combo` match semantics, replacing hardcoded match arms in the dispatcher.
- `seed::merged_bundle()` / `seed::parse_bundle()` plus their JS mirrors `FormalAiSeed.parseBundle` / `FormalAiSeed.loadFromBundle` give the seed a single-file Links Notation round-trip.
- Append-only memory log in the web demo (`src/web/memory.js`) with Export memory, Import memory, and Download bundle buttons; reasoning steps and tool invocations are recorded alongside chat turns, and no delete/forget/clear API is exposed.
- Tool registry panel surfacing seeded tools (`http_fetch`, `web_search`, `wikipedia_lookup`, `eval_js`, `read_local_file`, `append_memory`, `export_memory`) with a `thinking` vs `agent` mode badge.
- Playwright e2e suite (`tests/e2e/tests/multilingual.spec.js`) covering multilingual greetings/identity, offline + Wikipedia `What is X?` resolution, export/import round-trip, append-only enforcement, bundle download, tool registry, and reasoning-event logging; runs both against `npx serve src/web` (PR) and against `link-assistant.github.io/formal-ai` (post-deploy).
- `docs/case-studies/issue-16/` with raw GitHub data, online research, requirements-to-implementation mapping, and a Follow-Up section covering the universal seed loader and bundle round-trip.

### Changed
- Moved the deployable demo from `docs/demo/` to `src/web/` so it sits next to the other library/CLI/web sources; the GitHub Pages deploy job already targets `src/web`.
- Relocated `REQUIREMENTS.md` to the repository root next to `VISION.md` and expanded it with R90–R104 (move to `src/web`, multilingual surface, Wikipedia fallback, e2e coverage, export/import, append-only constraint, seed externalization, tool registry UI, reasoning/tool-call events, single-file bundle, universal seed loader, intent routing as data, bundle round-trip).
- Rewrote `src/web/formal_ai_worker.js` to initialise mutable `MULTILINGUAL_ANSWERS`, `CONCEPTS`, and `TOOLS` tables from the seed instead of carrying hardcoded literals; `solve()` now returns structured `steps[]` and `toolCalls[]` for the append-only log.

## [0.20.0] - 2026-05-15

### Added
- Added arithmetic, concept-lookup, conversation-recall, and explicit-JavaScript-execution handlers to the universal solver so the same loop now answers "what is 2 + 2?", "what is Wikipedia?", "what is my name?", and "please run this javascript: ..." prompts (issue #14).
- Added `solve_with_history`, `ConversationRole`, and `ConversationTurn` to the library API so every surface can carry conversation memory through the same solver loop.
- Added `data/seed/concepts.lino` with offline records for Wikipedia, Wikidata, Wiktionary, Links Notation, doublet links, the universal solver, the event log, WebAssembly, and Rust; each record cites its source for auditability.
- Added `tests/unit/mvp/reasoning_paths.rs` (R85–R88) with 24 tests pinning the new handlers and proving every answer is a projection of the append-only event log rather than a memoized constant.
- Added `examples/universal_solver_tour.rs` which walks every specialized handler through the same `FormalAiEngine::answer` entry point used by the library, CLI, HTTP server, Telegram bot, and demo.

### Changed
- Re-implemented `docs/demo/formal_ai_worker.js` as a universal-solver port: the hardcoded prompt→answer table is gone and every reply now walks greeting, identity, arithmetic, concept-lookup, recall, JavaScript-execution, hello-world, and unknown-fallback handlers that mirror `src/solver.rs`. JavaScript snippets requested with "please run this javascript" are actually executed in the worker sandbox; output, return value, and errors are reported with execution-status evidence.
- Persisted the demo's "Demo on/off" and "Diagnostics" toggles in `localStorage` using a Links Notation encoding (`docs/demo/preferences.js`) so the UI state survives reloads in the same format that grounds the solver's knowledge.

## [0.19.0] - 2026-05-14

### Added
- Added the issue #12 holistic case study, root vision/goals/non-goals documents, and a unit test that keeps the documentation set present and traceable.

### Added
- Added a TDD-style MVP test suite under `tests/unit/mvp/` that pins down the chat surface, code generation, multilingual chat, OpenAI compatibility, Telegram surface, links network, reasoning loop, source cache, agent isolation, translation-via-Links, network visualization, and transparent-state requirements drawn from `VISION.md`, `GOALS.md`, `NON-GOALS.md`, and `docs/REQUIREMENTS.md`. Tests describing not-yet-implemented MVP behavior are marked `#[ignore = "MVP-target: ..."]` so they document expectations without blocking CI; run them locally with `cargo test -- --include-ignored`.

### Added
- Added the universal 11-step solver loop (impulse → formalization → context → history → decomposition → TDD → synthesis → combination → verification → simplification → documentation) and wired it into `FormalAiEngine.answer` so every reply walks the same reasoning pipeline regardless of intent.
- Added a deterministic in-process append-only event log (`EventLog`) that records every step of the loop with content-addressed ids and projects to `evidence_links` plus a Links Notation `steps:` trace block.
- Added lightweight Unicode-block based language detection (`Language::{English, Russian, Hindi, Chinese, Unknown}`) so every impulse is tagged with `language:<slug>` evidence.
- Added in-process knowledge graph projection with `GraphNode`/`GraphEdge`/`KnowledgeGraph` and a `/v1/graph` endpoint that returns either Links Notation, JSON, or Graphviz DOT.
- Added `rate_limit` metadata on `/v1/models` responses, SSE streaming for `/v1/chat/completions` when `stream: true` is requested, and a configurable `SolverConfig` (offline mode, autonomy bounds) driven by environment variables.
- Added Telegram trace pointers: every chat reply now ends with a `/trace <id>` footer so users can request the inline event log for the message they just received.

### Changed
- Re-implemented Telegram, OpenAI, and library code paths to consume the universal solver instead of bespoke per-intent helpers, removing the previous hard-coded greeting/identity/hello-world branches in favor of pattern-matched specialized handlers (diagnostic, conversation memory, translation, algorithm synthesis, source-cache refresh, conflict surfacing, policy bounded-autonomy).
- Split `src/solver.rs` into `solver.rs` + `solver_helpers.rs` to keep both modules under the 1000-line hard limit and re-organized the imports so each helper is explicit.

## [0.18.0] - 2026-05-12

### Added
- Added a Telegram webhook endpoint backed by the symbolic engine, plus execution metadata for hello-world code answers and verification coverage for private and public chat updates.

### Added
- Added a `formal-ai telegram` CLI subcommand that defaults to Telegram long polling (`getUpdates`) and keeps the existing webhook server as `--mode=webhook`, configured through `lino-arguments` so flags, environment variables, and `.lenv`/`.env` files all feed the same parser.
- Introduced `TelegramPollingConfig`, `parse_get_updates_response`, the `TelegramTransport` trait, and a curl-backed default transport so the polling loop is fully unit-tested without a network.

## [0.17.0] - 2026-05-12

### Added
- Added demo chat issue-report links that prefill GitHub issues with dialog history and browser metadata.
- Added a symbolic identity response for "Who are you?" style prompts.

### Removed
- Removed the unused demo composer preview control.

## [0.16.0] - 2026-05-12

### Changed

- Start the web demo in interactive demo mode by default, add a live next-dialog countdown, and hide diagnostic trace details unless diagnostics mode is enabled.

## [0.15.0] - 2026-05-12

### Fixed
- Deploy the demo through the GitHub Pages workflow artifact path so workflow-sourced Pages deployments fail accurately and expose the deployed URL to Pages e2e tests.

## [0.13.0] - 2026-05-12

### Added
- Added the formal-ai proof-of-concept library, CLI, OpenAI-compatible JSON API server, Docker packaging, and GitHub Pages demo.
- Added a markdown-capable chat demo with randomized dialog mode, greeting-first examples, and multi-language hello-world prompts.
- Added Rust-script dataset generation into human-readable Links Notation `.lino` files under `data/`.

### Changed
- Expanded deterministic hello-world responses from Rust-only to Rust, Python, JavaScript, TypeScript, Go, and C.
- Extended repository file-size checks to enforce the 1500-line limit for `.lino` dataset chunks.

## [0.11.0] - 2026-05-09

### Added
- Added optional Docker Hub image publishing tied to Rust crate releases, including crates.io visibility waiting, version/latest image tags, and Docker Hub badges in GitHub release notes.

### Changed
- Release completeness checks now self-heal when crates.io exists but configured Docker Hub or GitHub release artifacts are missing.

## [0.10.0] - 2026-05-09

### Fixed
- Made `create-github-release.rs` build GitHub release titles as `[Language] X.Y.Z` instead of reusing the tag prefix.

## [0.9.0] - 2026-05-03

### Changed
- Added explicit GitHub Actions job timeouts and documented Rust test timeout guidance.

### Fixed
- Added a non-blocking warning threshold to the Rust file-size check so near-limit files are surfaced before concurrent PR merges can exceed the hard limit.

## [0.8.0] - 2026-05-01

### Fixed
- Make release scripts resolve the publishable crate manifest when the repository root uses a Cargo workspace manifest.

### Fixed
- Decoupled GitHub Pages documentation deployment from package release publication and fixed release-script warning failures under `RUSTFLAGS=-Dwarnings`.

## [0.7.0] - 2026-04-14

### Fixed

- Change detection script now uses per-commit diff instead of full PR diff, so commits touching only non-code files correctly skip CI jobs even when earlier commits in the same PR changed code files

## [0.6.0] - 2026-04-13

### Fixed

- Fix publish steps overriding workflow-level CARGO_TOKEN fallback, breaking CARGO_REGISTRY_TOKEN-only configurations (#32)
- Fix non-fast-forward push failures in multi-workflow repos by adding fetch/rebase and push retry logic (#31)
- Add mono-repo path support to check-changelog-fragment.rs, check-version-modification.rs, and create-changelog-fragment.rs
- Add `!cancelled()` guard to test job condition to respect workflow cancellation

## [0.5.0] - 2026-04-13

### Fixed
- Fixed unsupported look-ahead regex in `create-github-release.rs` that caused a panic when parsing CHANGELOG.md. Replaced with a two-step approach using only features supported by Rust's `regex` crate.

### Changed
- Restructured example application as a simple CLI sum calculator using `lino-arguments`
- Renamed default package to `example-sum-package-name` with Unlicense license
- Reorganized test structure: `tests/unit/sum.rs`, `tests/integration/sum.rs`, `tests/unit/ci-cd/`
- Converted experiment scripts into proper unit tests in `tests/unit/ci-cd/changelog_parsing.rs`
- Added CI/CD skip logic for template default package name `example-sum-package-name`
- Updated README.md badges and documentation

## [0.4.0] - 2026-04-13

### Added

- Cache `restore-keys` for partial cache hits across all workflow jobs
- Explicit `token` parameter in checkout for release jobs
- Code coverage job with `cargo-llvm-cov` and Codecov integration
- Codecov badge in README.md
- Pre-release version support (e.g., `0.1.0-beta.1`) in version parsing
- `--release-label` parameter for multi-language release disambiguation
- `ensure_version_exceeds_published()` logic to prevent publishing duplicate versions
- `get_max_published_version()` to query highest non-yanked version from crates.io
- `max_published_version` output from check-release-needed for downstream use
- Version fallback logic in auto-release Create GitHub Release step

### Changed

- Updated `actions/checkout` from v4 to v6
- Updated `actions/cache` from v4 to v5
- Updated `peter-evans/create-pull-request` from v7 to v8
- Made `publish-crate.rs` fail (exit 1) when version already exists on crates.io
- Improved `create-github-release.rs` to check combined stdout+stderr and detect "Validation Failed"

## [0.3.0] - 2026-04-13

### Fixed

- Fixed `version-and-commit.rs` to check crates.io instead of git tags for determining if a version is already released
- This prevents the release pipeline from getting stuck when git tags exist without corresponding crates.io publication

### Added

- Added `--tag-prefix` support to `version-and-commit.rs` for multi-language repository compatibility
- Added crates.io and docs.rs badges to README.md
- Added automatic crates.io and docs.rs badge injection in GitHub release notes
- Added documentation deployment job to CI/CD pipeline (deploys to GitHub Pages after release)
- Added case study documentation for issue #25

## [0.2.0] - 2026-03-11

### Added
- Changeset-style fragment format with frontmatter for specifying version bump type
- New `get-bump-type.mjs` script to automatically determine version bump from fragments
- Automatic version bumping on merge to main based on changelog fragments
- Detailed documentation for the changelog fragment system in `changelog.d/README.md`

### Changed
- Updated `collect-changelog.mjs` to strip frontmatter when collecting fragments
- Updated `version-and-commit.mjs` to handle frontmatter in fragments
- Enhanced release workflow to automatically determine bump type from changesets

### Changed
- Add `detect-changes` job with cross-platform `detect-code-changes.mjs` script
- Make lint job independent of changelog check (runs based on file changes only)
- Allow docs-only PRs without changelog fragment requirement
- Handle changelog check 'skipped' state in dependent jobs
- Exclude `changelog.d/`, `docs/`, `experiments/`, `examples/` folders and markdown files from code changes detection

### Fixed
- Fixed README.md to correctly reference Node.js scripts (`.mjs`) instead of Python scripts (`.py`)
- Updated project structure in README.md to match actual script files in `scripts/` directory
- Fixed example code in README.md that had invalid Rust with two `main` functions

### Added

- Added crates.io publishing support to CI/CD workflow
- Added `release_mode` input with "instant" and "changelog-pr" options for manual releases
- Added `--tag-prefix` and `--crates-io-url` options to create-github-release.mjs script
- Added comprehensive case study documentation for Issue #11 in docs/case-studies/issue-11/

### Changed

- Changed changelog fragment check from warning to error (exit 1) to enforce changelog requirements
- Updated job conditions with `always() && !cancelled()` to fix workflow_dispatch job skipping issue
- Renamed manual-release job to "Instant Release" for clarity

### Fixed

- Fixed deprecated `::set-output` GitHub Actions command in version-and-commit.mjs
- Fixed workflow_dispatch triggering issues where lint/build/release jobs were incorrectly skipped

### Fixed

- Fixed changelog fragment check to validate that a fragment is **added in the PR diff** rather than just checking if any fragments exist in the directory. This prevents the check from incorrectly passing when there are leftover fragments from previous PRs that haven't been released yet.

### Changed

- Converted shell scripts in `release.yml` to cross-platform `.mjs` scripts for improved portability and performance:
  - `check-changelog-fragment.mjs` - validates changelog fragment is added in PR diff
  - `git-config.mjs` - configures git user for CI/CD
  - `check-release-needed.mjs` - checks if release is needed
  - `publish-crate.mjs` - publishes package to crates.io
  - `create-changelog-fragment.mjs` - creates changelog fragments for manual releases
  - `get-version.mjs` - gets current version from Cargo.toml

### Added

- Added `check-version-modification.mjs` script to detect manual version changes in Cargo.toml
- Added `version-check` job to CI/CD workflow that runs on pull requests
- Added skip logic for automated release branches (changelog-manual-release-*, changeset-release/*, release/*, automated-release/*)

### Changed

- Version modifications in Cargo.toml are now blocked in pull requests to enforce automated release pipeline

### Added

- Added support for `CARGO_REGISTRY_TOKEN` as alternative to `CARGO_TOKEN` for crates.io publishing
- Added case study documentation for Issue #17 (yargs reserved word and dual token support)

### Changed

- Updated workflow to use fallback logic: `${{ secrets.CARGO_REGISTRY_TOKEN || secrets.CARGO_TOKEN }}`
- Improved publish-crate.mjs to check both `CARGO_REGISTRY_TOKEN` and `CARGO_TOKEN` environment variables
- Added warning message when neither token is set

### Added
- New `scripts/rust-paths.mjs` utility for automatic Rust package root detection
- Support for both single-language and multi-language repository structures in all CI/CD scripts
- Configuration options via `--rust-root` CLI argument and `RUST_ROOT` environment variable
- Comprehensive case study documentation in `docs/case-studies/issue-19/`

### Changed
- Updated all release scripts to use the new path detection utility:
  - `scripts/bump-version.mjs`
  - `scripts/check-release-needed.mjs`
  - `scripts/collect-changelog.mjs`
  - `scripts/get-bump-type.mjs`
  - `scripts/get-version.mjs`
  - `scripts/publish-crate.mjs`
  - `scripts/version-and-commit.mjs`

### Changed

- **check-release-needed.mjs**: Now checks crates.io API directly instead of git tags to determine if a version is already released. This prevents false positives where git tags exist but the package was never actually published to crates.io.

### Added

- **CI/CD Troubleshooting Guide**: New documentation at `docs/ci-cd/troubleshooting.md` covering common issues like skipped jobs, false positive version checks, publishing failures, and secret configuration.

- **Enhanced Error Handling in publish-crate.mjs**: Added specific detection and helpful error messages for authentication failures, including guidance on secret configuration and workflow setup.

- **Case Study Documentation**: Added comprehensive case study at `docs/case-studies/issue-21/` analyzing CI/CD failures from browser-commander repository (issues #27, #29, #31, #33) with timeline, root causes, and lessons learned.

### Fixed

- **Prevent False Positive Version Checks**: The release workflow now correctly identifies unpublished versions by checking crates.io instead of relying on git tags, which can exist without the package being published.

### Changed

- Translated all CI/CD scripts from JavaScript (.mjs) to Rust (.rs) using rust-script
- Scripts now use native Rust with rust-script for execution in shell
- Removed Node.js dependency from CI/CD pipeline
- Updated GitHub Actions workflow to use rust-script instead of node
- Updated README and CONTRIBUTING documentation with new script references

## [0.1.0] - 2025-01-XX

### Added

- Initial project structure
- Basic example functions (add, multiply, delay)
- Comprehensive test suite
- Code quality tools (rustfmt, clippy)
- Pre-commit hooks configuration
- GitHub Actions CI/CD pipeline
- Changelog fragment system (similar to Changesets/Scriv)
- Release automation (GitHub releases)
- Template structure for AI-driven Rust development
