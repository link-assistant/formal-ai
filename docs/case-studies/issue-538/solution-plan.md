# Issue 538 Solution Plan

A per-requirement solution plan for the requirements in
[requirements.md](requirements.md). For each, it names the approach, the
existing components/libraries reused (so we don't reinvent), and — for tracked
items — the smallest next concrete step.

## Guiding decisions

1. **Data over code.** Grammatical detail is knowledge, so it lives in `.lino`
   seed data and reuses the existing `SemanticFacet` machinery, not new Rust
   branches. (code-architecture-principles: prefer data; DRY.)
2. **One normalized facet vocabulary.** New detail is expressed as facet kinds in
   the single closed `FACET_KINDS` list, keeping one representation per link
   type (R7).
3. **Ground everything.** Every new meaning and surface references a real
   Wikidata item/lexeme/form with a checked-in cache file, so tests run offline
   and nothing is invented (R6).
4. **Ship the verifiable core; route the programme.** The concrete tomato detail
   is delivered and tested now; the large self-hosting/WASM/AST programmes are
   decomposed and sent to the roadmap rather than half-built.

## A. Detailed meanings and words (shipped)

### R1 singular/plural + R5 fix asymmetry

- **Approach:** add a `grammatical_number` facet kind; tag every tomato surface
  with a grounded `singular`/`plural` target. Add the missing `томаты` plural for
  `томат` (lexeme `L170542`, form `F7`) so both Russian synonyms are symmetric.
- **Reused:** `SemanticFacet` parsing, `semantic_facet_targets`,
  `FACET_KINDS`; Wikidata forms already cached.
- **Where:** `data/seed/meanings-translation.lino` (tomato block),
  `src/seed/meanings.rs` (`FACET_KINDS`, `grammatical_number()` accessor),
  `data/seed/meanings-lexical-meta.lino` (`grammatical_number`/`singular`/`plural`
  meanings).
- **Test:** `tomato_surfaces_pin_their_grammatical_number`,
  `tomato_singular_and_plural_are_distinct_forms_in_each_language`.

### R2 full word definition / part of speech

- **Approach:** every surface carries `part_of_speech noun`; `singular`/`plural`
  and `grammatical_number` are first-class grounded meanings, so the tag is a
  reference to an understood concept, not an opaque string.
- **Reused:** the existing `part_of_speech` facet kind and the `noun` meaning
  (`Q1084`).
- **Test:** `tomato_surfaces_expose_part_of_speech_from_data`,
  `grammatical_number_meanings_are_grounded_and_multilingual`.

### R3/R4 direct dictionary + bidirectional reference

- **Approach:** rely on the parser's auto-attached `denotation` facet (word →
  meaning) and expose it through `WordForm::denotations()`; assert every tomato
  surface denotes `tomato`.
- **Reused:** `parse_word_form` already attaches `notation word_surface` +
  `denotation <parent meaning>`; no new mechanism needed — only a public
  accessor and a test.
- **Test:** `every_tomato_surface_denotes_the_tomato_meaning`.

### R6 grounding + R8 multilingual parity

- **Approach:** ground `grammatical_number`→`Q104083`, `singular`→`Q110786`,
  `plural`→`Q146786`; cache `Q104083` (the values were already cached). Surfaces
  reference real forms of `L7993`/`L3526`/`L170542`. Lexicalise the three new
  meanings in en/ru/hi/zh mirroring the `part_of_speech` template.
- **Reused:** `scripts/ground-meanings.rs` fetch/trim/cache pipeline; the
  grounding-closure test that validates every referenced id has cache files.
- **Test:** the grounding-closure suite in `tests/unit/semantic_grounding.rs`
  plus `grammatical_number_meanings_are_grounded_and_multilingual`.

### R7 normalized per link type

- **Approach:** exactly one new facet kind; no parallel structure. `source-lexeme`
  blocks remain inert documentation whose ids the closure tests validate.
- **Reused:** the whole existing facet grammar.

## B. Codebase-wide programme (tracked)

### R9 import all collectable semantics

- **Existing components:** Wikidata REST + lexeme API (already used by
  `scripts/ground-meanings.rs`); the cache convention
  `data/cache/wikidata/{entity,lexeme,property}`; `data/overrides/wiktionary/…`.
- **Smallest next step:** generalise `ground-meanings.rs` into a batch importer
  that, given a meaning + a source lexeme id, emits the enriched surface block
  (grammatical_number + part_of_speech + sense) the tomato block now shows by
  hand. This turns the tomato edit into a template applied at scale.

### R10 audit hardcoded strings / concept coverage

- **Existing components:** `docs/design/no-hardcoded-natural-language.md` states
  the constraint; the meanings lexicon is the target store.
- **Smallest next step:** a test/CI lint that greps `src/` for user-facing string
  literals not routed through the lexicon, producing an allowlist to burn down —
  mirroring how ratchet/floor tests already guard grounded-meaning counts.

## C. Rust ⇄ JS ⇄ WASM (tracked)

### R11/R12

- **Existing components:** the demo **already** compiles Rust to a WASM worker
  (`src/web/wasm-worker/src/lib.rs` + `build.sh` → `src/web/formal_ai_worker.wasm`,
  issue #1 R16). The gap is the hand-written JS workers under `src/web/worker/`
  that still carry logic. `wasm-pack` + `wasm-bindgen` + Web Workers (research §4)
  are the standard path to widen the WASM surface.
- **Smallest next step:** pick one behaviour currently implemented in a
  `src/web/worker/*.js` file, move it behind the existing `wasm-worker` crate,
  and assert parity with a single test — proving the migration pattern before
  scaling it. The build target already exists, so no new toolchain is required.

## D/E. Self-inspecting universal meta algorithm (tracked, overlaps #559)

### R13–R21

- **Existing components:** issue #559's meta-algorithm registry
  (`src/method_registry.rs`, `src/meta_method_dispatch.rs`,
  `data/meta/recursive-core-recipe.lino`); `syn`/`proc-macro2` for Rust CST/AST;
  `mermaid` for diagrams; `docs/vscode/` exploratory notes for the debug view.
- **Smallest next steps (independent, small):**
  - R15/R16: generate one mermaid diagram from the existing method registry as a
    build artifact — a self-contained, testable slice.
  - R13/R14: snapshot the AST of one module with `syn` into `.lino` and prove a
    round-trip on that single module before scaling.
  - R21: add a contradiction detector that, given a set of formalized
    requirements, flags a scope conflict (this issue is a ready-made fixture).

## F. Solve via the Agent CLI (tracked, honestly reported)

### R22–R24

- **Existing components:** <https://github.com/link-assistant/agent> (Agent CLI);
  the Formal AI server in this repo.
- **Honest status:** not performed in this PR. Driving a still-maturing Agent CLI
  to self-solve a task of this breadth is itself a research programme; attempting
  it would have blocked the small, verifiable data improvement the issue centres
  on (the помидор/томат example). The concrete core is therefore delivered
  directly, and the self-hosting loop is planned as the follow-up in
  [README.md](README.md).
- **Smallest next step:** script a single Agent-CLI session that reproduces
  *one* of this PR's atomic edits (e.g. adding `томаты`) in a scratch repo, and
  capture its JSON session file — the minimal instance of R23/R24.

## G. Process (done)

### R26–R29

- This case study (R26), the single PR #601 (R27), the many small commits (R28),
  and per-binary `cargo test` runs (R29) satisfy the process requirements.
