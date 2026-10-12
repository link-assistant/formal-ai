# Architecture: Universal Problem Solver, Nested Steps, and Rules

Part of the [architecture overview](../../ARCHITECTURE.md) (§7–§9). Section
numbers match the overview's table of contents; paths are relative to the
repository root.

## 7. Universal Problem Solver

The solver follows the universal loop documented in `VISION.md` (Section
"Universal Problem-Solving Algorithm"). The implementation is in
`rust/src/solver.rs`:

1. **Impulse** — an `impulse` event is appended through `EventLog::append`.
2. **Formalization** — alias resolution plus P/Q-id lookup with fallbacks.
3. **Context and domain data** — language detection, surface, mode flags.
4. **History lookup** — search local doublets first; record `cache_hit`
   on success.
5. **Decomposition** — split conjunctions ("and", "with tests", "with
   benchmarks") into sub-impulses.
6. **TDD-style test generation** — emit at least one `test:` event per
   candidate.
7. **Solution synthesis** — reuse known parts → reason from rules →
   randomized / evolutionary search if the structure allows.
8. **Combination** — recombine partial solutions.
9. **Verification** — run candidate against generated tests; surface
   `trace:execution_failure` on failure.
10. **Simplification** — apply meaning-preserving transformation rules to
    shrink the answer.
11. **Presentation** — produce the user-facing reply + Links Notation trace
    + evidence links.

Every numbered step writes its own event before the next one starts.

### Minimal Compiled Core (Issue #918)

The implementation boundary has exactly four categories: **Meta algorithm**
for bounded selection, proof, and learning; **Link store** for associative
memory; **Generic interpreters** for data-defined rules, recipes, queries, and
programs; and **Host surfaces** for safe CLI, server, browser, filesystem,
process, and network I/O. Domain vocabulary, routing policy, prose, examples,
units, preconditions, and effects belong in data.

The complete rationale and promotion test live in
`docs/design/minimal-core-boundary.md`. The source-level ledger recursively
covers `rust/src/solver_handlers/**/*.rs`; `scripts/check-minimal-core-boundary.rs`
rejects unreviewed files, expanded migration debt, or stale lowered baselines.
This is a burn-down boundary, not a claim that mixed handlers are already core.

### 7.1 Project lookups and summarization

"What is `<project>`?" prompts about projects and repository URLs go through a
generic `project_lookup` path. When associative project promotion is enabled
(the default), repositories from `link-assistant`, `link-foundation`, and
`linksplatform` are listed first when they match the prompt; turning promotion
off keeps the same prompt on the generic GitHub/GitLab/Bitbucket lookup path.
The pipeline has three pieces:

1. **Curated registry.** `data/seed/projects.lino` records the canonical
   repository, primary language, weighted statements, English/Russian
   localisations, topic label, and aliases for each project. The seed file is
   embedded at compile time and parsed once per process via
   `rust/src/seed/projects.rs::projects_registry()`.
2. **Formalize → summarize → deformalize pipeline.** `rust/src/summarization/mod.rs`
   exposes a deterministic three-stage pipeline. `formalize` (or
   `Statement::from_seed`) turns free-form prose or curated statements into a
   homogeneous `Vec<Statement>` with a `StatementKind` (identity, purpose,
   language, stars, feature, use_case, install, example, misc) and a numeric
   weight. `summarize` then applies a `SummarizationConfig` whose
   `SummarizationMode` selects the target size — `Topic` (1–5 words),
   `Short` (~20%), `Standard` (~50%), `Full` (100%), or `Expand` (~200%) — and
   the optional explicit `max_statements` cap. Boilerplate kinds (`install`,
   `example`) are dropped from compressed answers; `Expand` mode appends
   Natural Semantic Metalanguage paraphrases. `deformalize` joins the
   surviving statements back into a single block of prose.
3. **Handler integration.** The solver dispatch table in `rust/src/solver.rs`
   still lets `concept_lookup` answer seed concepts such as Links Notation
   first. Immediately after a concept miss, `project_lookup` handles promoted
   project aliases such as Hive Mind or link-cli, explicit GitHub/GitLab/
   Bitbucket repository URLs, and the promotion-off fallback. Promoted answers
   log `project:promoted`, `summarization:mode`, `summarization:language`, the
   repository URL, and the web-search providers consulted alongside the local
   answer so the trace explains both *what* was matched and *how* the text was
   compressed.

The compression knobs are configurable from one struct (`SummarizationConfig`)
so callers can dial topic labels, chat titles, project descriptions, or
expanded explanations from the same pipeline.

The same pipeline also drives four additional surfaces:

- **README ingestion.** `strip_markdown_noise` removes badges, fenced code
  blocks, HTML comments, heading markers, and blockquote chevrons. The
  cleaned prose is fed through `formalize_markdown` (a thin wrapper around
  `formalize`) and `describe_readme(repo_slug, markdown, &config)`. In
  `Topic` mode the helper returns the repository slug so the same call can
  serve as a chat-title source for a fetched repository.
- **Repository-file summaries.** `formalize_repository_file(path, content)`
  detects common repository file formats, records path/format/line/byte
  metadata, converts file content into ranked statements, and renders the
  result as link-native `repository_file` notation. Supported source and data
  grammars also carry `MetaLanguageFormalization` evidence from
  `meta_language::LinkNetwork` (parser label, syntax-link count, total-link
  count, parse-error state, and text-preservation state). Markdown files are
  formalized recursively: prose is summarized through `formalize_markdown`, and
  each fenced code block becomes an `EmbeddedGrammarFormalization` with its own
  normalized language label and optional parser evidence. `summarize_repository_file`
  then reuses `SummarizationConfig`, `summarize`, and `deformalize` so file
  summaries follow the same modes and caps as project, README, and dialog
  summaries.
- **Repository-resource summaries (files and folders).**
  `rust/src/summarization/resource.rs` generalizes file summarization to any
  repository resource so the solution is not file-specialized. A caller builds a
  filesystem-free `RepositoryEntry` tree (`RepositoryEntry::file` /
  `RepositoryEntry::directory`); `formalize_repository_resource` then dispatches
  on kind, reusing `formalize_repository_file` for files and recursing through
  `formalize_repository_directory` for folders. A directory is summarized by the
  meta algorithm's decompose -> summarize -> compose loop: it is split into its
  children, each child is summarized on its own (files via `file.rs`,
  subdirectories by recursion), and the child summaries are composed behind an
  aggregate identity sentence carrying recursive file/subdirectory counts and
  total lines/bytes. Recursion depth is bounded by the *mode ladder*
  (`SummarizationMode::one_step_shorter`): a `Full` folder describes its direct
  children in `Standard`, theirs in `Short`, and everything deeper as a `Topic`
  label, so arbitrarily deep trees stay bounded while the most important
  structure surfaces first. `RepositoryDirectoryFormalization::links_notation`
  renders a link-native `repository_directory` block (path, counts, per-child
  kind) for inspectable evidence, and `summarize_repository_resource` is the
  general entry point that subsumes `summarize_repository_file` for file inputs.
- **Summarization quality protocol (seeded sampling + 80% ratchet).**
  `rust/src/summarization/validation/` answers the part of issue #563 that a
  summarizer alone cannot: *is the summarizer any good on files nobody
  optimized for?* `SamplingProtocol` fixes a seed, two files per iteration, an
  iteration bound and a stability window, and permutes the corpus with a seeded
  `splitmix64` Fisher-Yates shuffle, so the same seed over the same corpus draws
  the same files in the same order and no file is drawn twice in one run. Each
  sampled file goes through the *production* summarizer — `evaluate_file` calls
  `formalize_repository_file` and `RepositoryFileFormalization::summary`, never a
  test-only reimplementation — and is scored against the published `CRITERIA`
  (identity, format, size, retained content, grounded content, compression,
  embedded-grammar recursion, meta-language evidence, determinism, mode ladder).
  A criterion that cannot apply to a file is excluded from that file's
  denominator rather than scored as a free pass, and scores are exact integer
  `passed/applicable` ratios micro-averaged across files and floored when
  rendered, so 79.6% gates as 79%. `validate_repository_summarization` iterates
  until `stability_window` consecutive iterations all clear the ratchet within
  `stability_tolerance_percent` of one another, at least `minimum_iterations`
  iterations have run (three perfect iterations are six files, which say nothing
  about a corpus of thousands) *and* at least one Markdown embedded grammar block
  has been exercised; otherwise it stops at the bound and reports
  `bound_reached` rather than claiming stability. Because that last condition is
  fatal and fenced Markdown is a small minority of the corpus, the draw is
  stratified: `stratified_sampling_order` promotes the first fence-carrying
  Markdown file of the seeded permutation into iteration 0 and leaves every other
  file where the seed put it. `QUALITY_RATCHET_PERCENT
  = 80` is the published floor, `ratchet_violations` enforces it together with
  monotonicity against the committed baseline
  `data/summarization/quality-baseline.lino`, and `formal-ai summarization
  criteria | validate | ratchet` (`rust/src/cli_summarization.rs`) is the operator
  surface. The embedded-grammar criterion grades against its own independent
  CommonMark fence scanner, so the summarizer never grades itself.
- **Dialog summarization.** `DialogTurn { role, text }` and
  `formalize_dialog` weight user turns +20 and assistant turns -10 so a
  short summary keeps the user's questions even when both sides talk a
  lot. `summarize_dialog(turns, &config)` runs the result through
  `summarize` / `deformalize`, and `generate_chat_title(turns, language)`
  wraps it in `SummarizationMode::Topic`. `try_summarize_conversation` in
  `rust/src/solver_handlers/conversation_memory/conversation_summary.rs` now collects `prior_turn:user` and
  `prior_turn:assistant` events into `DialogTurn`s, calls `summarize_dialog`
  in `Standard` mode, and logs `summarization:mode`,
  `summarization:language`, and `chat_title` evidence alongside the
  per-turn list.
- **HTTP fetch for curated GitHub URLs.** When `try_http_fetch` recognises
  a `github.com/<org>/<name>` URL whose `<org>/<name>` matches the curated
  registry (`match_curated_github_url`), the handler runs `describe_project`
  in `Standard` mode and embeds the result in the response. The trace
  records `http_fetch:curated_project`, `summarization:mode`, and
  `summarization:language` so the path from URL → curated record → summary
  is fully visible.

The shared `DEFAULT_MAX_STATEMENTS = 30` constant in `rust/src/summarization/mod.rs`
documents the default cap on retained statements; any caller can raise or
lower it with `SummarizationConfig::with_max_statements`.

---

## 8. Nested Reasoning Steps

Tools called during synthesis can produce their own reasoning steps. The
calculator (`link-calculator`) is the current canonical example: when the
universal solver decides to delegate a calculation, the calculator's own
`StepRecord` items are appended to the parent trace as a **nested** sub-trace
under the parent `candidate:` event.

```text
candidate_007
  intent calculation
  expression "8% of $50"
  delegate "link-calculator"
  nested
    step "extract percentage 8"
    step "extract amount 50 USD"
    step "compute 4 USD"
  result "4 USD"
```

The nested-trace contract holds for every future tool integration (HTTP
fetch, Wikipedia summary, JS execution, etc.). Each tool returns a
`Vec<NestedStep>` instead of an opaque value, so the user can ask "why?" and
get a step-by-step explanation at any depth.

---

## 9. Transformation and Substitution Rules

The associative store supports five kinds of rules. They are listed in
order from "lowest privilege" to "highest privilege":

1. **Pure data rules** — `when LHS then RHS` doublet patterns. No code is
   executed; the rule rewrites links in place. Stored as Links Notation,
   reviewable by a human.
2. **Rust handlers** — compiled-in Rust functions registered against a rule
   id. Today this is how the specialized handlers in `solver_handlers/`,
   `solver_handler_units.rs`, and `solver_handler_how.rs` are wired up.
3. **JavaScript handlers** — JS functions registered for the browser worker
   and the upcoming `try_javascript_execution` solver step. They run in a
   sandboxed Worker.
4. **Dynamically compiled Rust/JS** — code snippets stored *inside* the
   associative store as text and compiled (Rust) or interpreted (JS) on
   demand. The compiled output is cached by the snippet's content hash.
5. **Natural-language skills** — prose instructions like "When the user asks
   for hello world in language X, look up the seed for X and print the
   verified output." These are compiled on demand into one of the above
   four representations: a data rule, a Rust handler stub, a JS handler
   stub, or an interpreted sequence of solver steps.

Issue #936 adds the executable path for pure substitution rules. The
Rust-owned compiler first lowers `SubstitutionRuleSet` into one serializable,
target-neutral `SubstitutionProgramIr`: ordered rules, `when` conditions,
`replace` actions, literal/whole-node/prefix patterns, and the interpreter's
bounded application limit. Emitters may only consume that IR. The canonical
Rust target embeds the generated runtime directly; WebAssembly compiles the
same generated Rust runtime; the JavaScript target is only an ES-module bridge
to that generated WASM and contains no rule matching or rewrite logic. Every
artifact also carries the JSON IR and an auditable compilation trace.

`ProgramPlan::compile` is the proof boundary. It refuses an unchanged plan or
a plan that reached its termination guard. The solver's seeded
`export_substitution_rule` route first reuses rule synthesis's semantic fixture,
then crosses that boundary and returns the named source/support files plus an
`ExecutionRecipe`. Thus natural-language export cannot bypass the existing
verified plan, while direct library callers can compile any parsed rule set via
`compile_substitution_rules` and compare its output with the interpreter.

`rust/src/skill_compiler.rs` implements the deterministic compiler subset. The
legacy `When ... answer ...` form still lowers into a `CompiledSkillPackage`
with a trigger rule, a deterministic compiled handler, an E1-style
`LinkRecord` projection, and a Links Notation export. The structured subset
adds reviewable `Skill`, typed `Input`, `Precondition`, ordered `Step`,
`Effect`, `Expected test`, `Permission`, `Tool`, and `Target` records. Expected
tests become deterministic replay fixtures, and `Target` records produce
inspectable Rust/JavaScript/native handler stubs rather than executable code.
The compiler refuses unsupported or nondeterministic instructions and requires
explicit `Permission` records for package/tool capabilities such as
`tool:local_shell`. The solver scans dialog history for compiled packages
before falling back to behavior-rule re-derivation; a replay appends
`compiled_skill:replay` and `cache_hit:<compiled_skill_id>` to the trace.

`rust/src/skill_procedure.rs` covers the prose that falls outside that typed shape
(E55, issue #674). It reuses `intent_formalization::ordered_requirement_spans`
to decompose a request such as "when I paste a link, fetch its title, translate
it to Russian, save both, and reply with the translation", then maps each
source-grounded requirement onto a step verb seeded in
`data/seed/meanings-skill-procedure.lino`. Two guards keep ordinary prompts out:
the request needs a seeded trigger lead and at least two recognized steps. The
canonical program contains meaning slugs only, so English, Russian, Hindi, and
Chinese phrasings content-address to the same id and `LinkRecord`s.

The durable artifact adds the formalized impulse id, ordered requirements,
source spans, trigger, and typed steps. Parsing it recomputes its canonical
program and ids and validates its provenance before the generic
`ProcedureHost` interpreter may walk it. The solver publishes this artifact,
the later *"why did you do that?"* handler restores it without recompiling
conversation prose, and the Agent planner writes and reads back the same bytes
before claiming success.

A clause with no vocabulary entry compiles nothing at all:
`rust/src/solver_handlers/procedure_rules.rs` answers with the named gap, appends a
`skill_gap` event, and emits a review-only learning proposal rather than
dropping the step. A proposal can add multilingual aliases to the durable
capability ledger only for an existing typed operation, after a green
regression gate and explicit human approval; it cannot silently authorize a new
side effect.

`rust/src/associative_package.rs` is the R65 package boundary. It models
Deep.Foundation-inspired packages in the local doublet architecture with
package metadata, dependency links, handler records, trigger records, and
explicit permission grants. A package can be exported/imported as Links
Notation, installed only after dependencies validate, replayed through its
trigger/handler links, and queried by the tool-call gate for capabilities such
as `tool:calculator`. Compiled skills can be wrapped as packages and imported
back without hand-editing Rust code; structured expected tests become package
triggers/handlers and structured permissions become package permission grants.
The `/api/formal-ai/v1/network` projection (the deprecated `/api/formal-ai/v1/graph`
alias still resolves) includes the package, handler, trigger, and permission
links so the permission path is inspectable alongside ordinary rules.

The compilation chain (NL → code → binary) is the long-term path. The
runtime never *requires* compilation: a natural-language skill can be
interpreted one step at a time without ever being lowered to Rust/JS.
