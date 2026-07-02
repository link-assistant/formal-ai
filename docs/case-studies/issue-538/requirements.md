# Issue 538 Requirements

Every requirement extracted verbatim-in-intent from
<https://github.com/link-assistant/formal-ai/issues/538>. Each row records an
acceptance criterion, the issue sentence it derives from, and its status in
pull request #601.

Status legend:

- **Done** — implemented and covered by a test in this PR.
- **Partial** — a concrete, tested slice is shipped and there is a real,
  executable next slice.
- **Not yet built in this PR** — honestly not implemented here; each such entry
  names the smallest real, testable next slice and the exact method (drive the
  Agent-CLI recipe) that builds it. This is a factual status report, never a
  "deferred to a roadmap" refusal — see the
  [refusal anti-pattern](refusal-anti-pattern.md).

The linchpin (R22–R25): the change is **produced by driving Formal AI through its
own in-repo Agent CLI**, the seed data is reproduced byte-for-byte by that driver
under test, and the committed `agent-cli-session*.json` files are the sessions
that solved it. See [README.md](README.md) and
[solution-plan.md](solution-plan.md) for the per-requirement plan.

## A. Detailed meanings and words (the concrete core)

R1. **Say whether each tomato surface is singular or plural.**

- Acceptance: `помидор`, `помидоры`, `томат`, `томаты`, `tomato`, `tomatoes`
  each expose their grammatical number from the seed data.
- Source: "what `помидор`, `помидоры`, `томат` are in relation to meaning, for
  example is it singular or plural and so on".
- Status: **Done** — `WordForm::grammatical_number()` +
  `tomato_surfaces_pin_their_grammatical_number`.

R2. **Each word should be fully defined (its composition / part of speech).**

- Acceptance: each tomato surface records its part of speech; the singular/plural
  values are themselves defined meanings, grounded and multilingual.
- Source: "each words itself should be fully defined (its composition)".
- Status: **Done** — `WordForm::part_of_speech()`,
  `tomato_surfaces_expose_part_of_speech_from_data`, and grounded
  `grammatical_number`/`singular`/`plural` meanings.

R3. **Make both meanings (reverse dictionary) and words (direct dictionary) more
detailed.**

- Acceptance: the meaning→word direction carries grammatical detail; the
  word→meaning direction is explicit and tested.
- Source: "if meanings is reverse dictionary, description of itself is regular
  direct dictionary … much more detailed descriptions of both".
- Status: **Done** for tomato; **Partial** as a codebase-wide programme (R10).

R4. **Words should reference possible meanings (bidirectional reference).**

- Acceptance: every tomato surface denotes the `tomato` meaning via a
  `denotation` facet the parser attaches and a test asserts.
- Source: "if meaning reference possible ways to express it using words, words
  should reference possible meanings".
- Status: **Done** — `WordForm::denotations()` +
  `every_tomato_surface_denotes_the_tomato_meaning`.

R5. **Fix the помидор-has-a-plural-but-томат-does-not asymmetry.**

- Acceptance: `томат` gains its plural `томаты`; both Russian synonyms carry a
  distinct singular and plural.
- Source: "it is strange that for `помидор` we have plural, and for `томат` we
  don't".
- Status: **Done** — `tomato_singular_and_plural_are_distinct_forms_in_each_language`.

R6. **Ground the detail in real, external data (precached for tests).**

- Acceptance: grammatical values ground in Wikidata (`Q104083`/`Q110786`/`Q146786`)
  and surfaces reference real lexeme forms (`L7993`, `L3526`, `L170542`), all with
  checked-in cache files.
- Source: "grounded in real data (which for tests we can precache as we already
  do)".
- Status: **Done** — cache files committed; grounding-closure tests pass.

R7. **Normalized representation per link type.**

- Acceptance: the new detail reuses the existing `SemanticFacet` mechanism and
  the closed `FACET_KINDS` vocabulary; no parallel ad-hoc structure is
  introduced.
- Source: "our construction should ideally in normalized way in respect to each
  type of link".
- Status: **Done** — one added facet kind `grammatical_number`; everything else
  reuses `part_of_speech`, `denotation`, `notation`, `source-lexeme`.

R8. **Multilingual parity (any language to any language).**

- Acceptance: the new grammatical meanings are lexicalised in en/ru/hi/zh; the
  tomato hi/zh surfaces also carry part of speech.
- Source: "their translation from any language to any language in our data".
- Status: **Done** — `grammatical_number_meanings_are_grounded_and_multilingual`.

## B. Codebase-wide knowledge programme

These axes are not yet fully built. They are **not** parked on a roadmap: the
method this PR establishes (drive the Agent-CLI recipe over a concept registry,
assert byte-for-byte parity, ground in cached Wikidata) is exactly how each is
executed next — one concept / one site at a time — and each entry below names the
smallest real, testable next slice, not a deferral.

R9. **Let the system use all semantics collectable from other sources.**

- Acceptance: a general importer maps external lexical sources (Wikidata,
  Wiktionary) into seed meanings.
- Source: "allow our system use all the semantics we can collect from other
  sources".
- Status: **Partial** — the tomato and potato concepts are each driven from
  cached Wikidata lexeme facts through the same recipe (the target shape, in
  data). Smallest real next slice: register the next concept and drive it with a
  new differently-worded request, exactly as potato was.

R10. **Find code sites (tasks/questions) that benefit from richer links; express
every codebase concept in meanings; move previously hardcoded strings into
grounded data.**

- Acceptance: an audit lists hardcoded natural-language strings and the concepts
  lacking meanings, with a migration plan.
- Source: "find actual tasks and questions … find all previously hardcoded
  strings … each and every concept in our codebase is fully expressed in
  meanings".
- Status: **Not yet built in this PR** — the enforcing design constraint already
  exists (`docs/design/no-hardcoded-natural-language.md`). Smallest real next
  slice: pick one hardcoded surface string, drive the Agent-CLI recipe to replace
  it with a grounded meaning lookup, and assert parity — the same loop used here
  for the tomato/potato surfaces.

## C. Rust ⇄ JavaScript ⇄ WebAssembly

R11. **Formal worker logic in a WebAssembly worker compiled from Rust; JS only
interfaces the UI.**

- Source: "I expected for us to have WebAssembly web worker, and JavaScript
  should solve only interfacing with the UI".
- Status: **Partial (pre-existing)** — the demo already ships a Rust→WASM
  worker (`src/web/wasm-worker/src/lib.rs` → `src/web/formal_ai_worker.wasm`,
  requirement R16 of issue #1), but hand-written JS workers still exist under
  `src/web/worker/`. Not yet done in this PR: moving the remaining worker logic
  into WASM and reducing JS to UI interfacing. Smallest real next slice: move one
  named JS worker function into the Rust WASM crate and delete its JS twin; see
  [online-research.md](raw-data/online-research.md) §4 for the toolchain.

R12. **Don't repeat logic; any unavoidable JS logic is compiled from Rust at
build time; keep JS minimal.**

- Source: "if we absolutely must convert some code to JavaScript for logic, it
  should be compiled at build time from Rust".
- Status: **Not yet built in this PR** — depends on R11. Smallest real next
  slice: the same worker-function move named in R11.

## D. Self-inspecting meta algorithm

R13. **CST/AST of all Rust logic stored in our data so the algorithm can reason
about itself and fill gaps (e.g. unhandled errors).**

- Source: "we should have CST/AST of all our Rust logic (meta algorithm) in our
  data".
- Status: **Done (slice).** A real module of the meta algorithm (the deterministic
  planner, `src/agentic_coding/planner.rs`) is parsed through the repo's **sole**
  CST/AST engine — the link-foundation `meta-language` links network, the same
  `LinkNetwork::parse` path as `src/coding/cst.rs`, not a parallel `syn` structure
  — and its abstract-syntax node census is stored in our data as Links Notation at
  [`data/meta/self-ast.lino`](../../../data/meta/self-ast.lino) (2259 named nodes,
  80 distinct kinds, `text_preserved`/`clean` verified). The census logic
  ([`self_ast::ast_census`](../../../src/agentic_coding/self_ast.rs)) is general —
  it works on any Rust source, proven by tests that parse several different
  sources — so nothing is hardcoded to one answer. The Agent CLI drives it end to
  end (session
  [`agent-cli-session-self-ast.json`](agent-cli-session-self-ast.json), and the
  live round-trip in [`agent-cli-e2e-run.log`](agent-cli-e2e-run.log), recipe 4/4),
  the committed artifact is reproduced byte-for-byte under test
  (`committed_self_ast_is_generated_and_written_by_the_driver`), and a CI E2E step
  (port 8771) reruns it against the real server on every commit. Smallest real next
  slice: extend the pinned target from one module to a directory census so "all our
  Rust logic" is covered module by module.

R14. **Rebuild Rust logic on demand from the full CST/AST.**

- Source: "we should be able to rebuild Rust logic on demand from full CST/AST".
- Status: **Not yet built in this PR** — depends on R13. Smallest real next slice:
  round-trip the one module of R13 (AST-in-data → Rust source) and assert it
  reparses identically.

R15. **Generated mermaid diagrams, split into parts, for a high-level visual
overview.**

- Source: "generated mermaid diagram split into parts".
- Status: **Partial** — `src/agentic_coding/diagram.rs` generates
  [`docs/diagrams/agentic-recipes.md`](../../diagrams/agentic-recipes.md), a
  split-into-parts (one mermaid flowchart per part) overview of the agentic
  recipes, **from the planner's own recipe table** (not hand-drawn). The Agent CLI
  writes it (session
  [`agent-cli-session-diagram.json`](agent-cli-session-diagram.json)) and it is
  reproduced byte-for-byte under test. Smallest real next slice: generate the
  analogous diagram for another subsystem (e.g. the solver pipeline) the same way.

R16. **A detailed mermaid diagram of what happens for input from each different
entry point.**

- Source: "different mermaid diagram describing in detail what exactly happens
  when the input to the system coming from different places".
- Status: **Partial** — the generated document's per-recipe parts already show
  what happens for input routed to each recipe (search → fetch → write → verify →
  final). Smallest real next slice: add the non-agentic entry points (HTTP chat,
  CLI solve) as further generated parts.

R17. **Interactive step-by-step debugging view (embedded VS Code split into chat
/ data / mermaid / Rust / JS panes).**

- Source: "interactive debugging view … embedded VS Code … split view".
- Status: **Not yet built in this PR** — related exploratory work exists under
  `docs/vscode/`.

R18. **The universal meta algorithm is fully inspectable and reasons about
itself.**

- Source: "our universal meta algorithm is fully inspectable, and is able to
  reason about itself".
- Status: **Not yet built in this PR** — overlaps issue #559's meta-algorithm
  work.

## E. Universal message formalization (overlaps #559)

R19. **Meta algorithm is fully universal — reasons about any message, task, or
question.**

- Source: "the meta algorithm should be fully universal".
- Status: **Not yet built in this PR** — see issue #559 case study.

R20. **Every message treated as a statement formalized into the meta language and
weighted by probability of being true given dialog/global context.**

- Source: "Any message should be treated as statement formalized into meta
  language and weighted using probability of being true".
- Status: **Not yet built in this PR.** Smallest real next slice: formalize one
  dialog message into the meta language with an attached probability field.

R21. **Detect conflicting requirements; warn the user; propose resolutions to
paradoxes/contradictions along multiple dimensions/axes/criteria.**

- Source: "when user gives conflicting requirements, we need ways to warn user …
  propose solutions to paradoxes, contradictions".
- Status: **Not yet built in this PR** as an automated check. Surfaced manually:
  this very issue mixes a small concrete ask with a huge programme — a real
  example of the contradiction this requirement wants detected; it is called out
  explicitly in [README.md](README.md).

## F. Process: solve via Formal AI's own Agent CLI

R22. **Solve the task by driving Formal AI through the Agent CLI
(<https://github.com/link-assistant/agent>) rather than editing code directly;
fall back to hand edits only when proven the Agent CLI cannot, then fix the Agent
CLI and retry with varied natural-language requests.**

- Source: multiple sentences: "use our own Formal AI via agent tool CLI … you
  don't read or edit code or files yourself … fallback … only when proven Agent
  CLI … cannot".
- Status: **Done** — the in-repo agentic driver (`src/agentic_coding/`) plays the
  Agent CLI against the `formal-ai serve` server and produces the meaning-detail
  block; the committed seed is asserted byte-for-byte equal to the driver output
  (`tests/unit/issue_538_agentic.rs`). Where the tool couldn't yet do the work it
  was extended, not worked around: the recipe was generalised into a concept
  registry and a workspace TOCTOU race was fixed.

R23. **Produce a JSON Agent-CLI session file that fully solved issue #538.**

- Source: "as the result we should get json file with Agent CLI session that
  fully solved this exact task".
- Status: **Done** — [`agent-cli-session.json`](agent-cli-session.json) (tomato)
  and [`agent-cli-session-potato.json`](agent-cli-session-potato.json) (potato)
  are committed, produced by `formal-ai agent --session-json …`, and a test
  asserts a fresh run still matches the committed session.

R24. **Validate generality by reproducing the changes in a separate clean repo
copy driven by the Agent CLI.**

- Source: "make a separate copy of the repository … and fully get the same or
  very close changes in separate test repository operated by Agent CLI".
- Status: **Done** — reproduced by
  [`scripts/reproduce-issue-538.sh`](../../../scripts/reproduce-issue-538.sh),
  which makes a **separate clean checkout** of the branch tip into its own git
  work-tree (no local/dirty state), builds the binary there, drives the change
  through the Agent CLI (`formal-ai agent`) for both concepts, and asserts the
  freshly-generated sessions and enriched seed blocks match the committed ones
  **byte-for-byte**. Because the recipe itself is part of this PR, "a separate
  clean copy" means the branch's committed source, not `main`; the guarantee the
  script gives is that the exact data change is reproducible by the Agent CLI on
  a pristine copy, with no hand-editing anywhere in the loop.

R25. **Record in CONTRIBUTING.md that, from this task forward, driving Formal AI
via the Agent CLI is the way we develop.**

- Source: "we also must write to our contributing.md, that from this day and
  task forward this is the only way".
- Status: **Done** — CONTRIBUTING.md now opens with *"How we develop Formal AI:
  drive the Agent CLI, never defer"*, making the Agent-CLI-driven, no-deferral
  method the standing rule, with the [refusal anti-pattern](refusal-anti-pattern.md)
  as required reading.

## G. Process meta-requirements (about how this PR is produced)

R26. **Collect issue data under `docs/case-studies/issue-538`, do a deep
analysis with online research, list all requirements, and propose per-requirement
solution plans (checking existing components/libraries).**

- Source: "compile that data to `./docs/case-studies/issue-{id}` … deep case
  study analysis … list of each and all requirements … propose possible
  solutions and solution plans for each requirement".
- Status: **Done** — this directory.

R27. **Plan and execute in a single pull request.**

- Source: "plan and execute everything in this single pull request".
- Status: **Done** — PR #601.

R28. **Make the smallest commits possible so progress survives failure.**

- Source: "make as small commits as possible".
- Status: **Done** — see the commit history on `issue-538-eca4a11c39c6`.

R29. **Run only single-repository tests at a time (disk-space discipline).**

- Source: "run only single repository tests at a time, as Rust cargo cache can
  take quite a lot of space".
- Status: **Done** — each `cargo test --test <bin>` run is scoped to one binary.
