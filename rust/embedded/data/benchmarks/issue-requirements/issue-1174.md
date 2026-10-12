Part of the E127 umbrella (#1183); parity classes **summarization, translation, rewriting / style transfer**.

## Evidence (0.347.0)

- Summarize the Apollo 11 paragraph → the input echoed back plus "summary: the request is recorded as a bounded summarization task".
- "Translate to Russian: The weather is nice today, let's go for a walk." → "I could not identify a source phrase to translate from en to ru." (VISION says translation goes through the meaning pivot; it works for seeded phrases, not sentences.)
- "Rewrite this more formally: hey, can u send me the report asap? thx" → canned web-search paragraph.

- "Correct the grammar: She don't like apples and he have two cat." → canned web-search paragraph.
- "Draft a short polite email asking my manager for a day off next Friday." and "Write a git commit message for a change that fixes an off-by-one error in pagination." → canned web-search paragraph.

## Formal versions to build

- **Grammar correction**: parse into the grammar projection (subject, verb, agreement features), check agreement (person, number) and noun number against the lexeme data, correct the forms, report each change with the rule.
- **Writing (email, commit message, PR description)**: formalize the intent (request, reason, date; change kind, component, effect), fill the genre's structure retrieved from style guides (e.g. Conventional Commits), render in the requested register.

- **Summarization**: formalize the text into statements (subject, predicate, object, time), rank statements by relative weight (coverage of entities, novelty, the relative-meta-logic weight of each statement against the rest), select under the length constraint, deformalize. `formal-ai summarization` exists for repository files; generalize it to any text.
- **Translation**: formalize the sentence (P/Q anchors, grammatical features: tense, mood, person, number), render in the target language from Wikidata lexemes and the grammar projection rules; unknown words become needs resolved through Wiktionary/Wikidata lexemes.
- **Rewriting**: formalize, change the register features (formality, contractions, abbreviations "u"→"you", "asap"→"as soon as possible", "thx"→"thank you") from lexical sources, deformalize.

## How to test

Held-out paragraphs and sentences in en/ru/hi/zh: summaries keep every named entity and date of the top-weighted statements; round trip en→ru→en preserves the meaning graph (the #526 contract); rewrites contain no informal tokens from a lexicon of informal forms.

## Requirements

R1. `formal-ai chat` routes a request recognized as summarization, translation, rewriting, grammar correction, or writing (email/commit message/PR description) to its own handler and never falls through to `try_budget_search`'s canned "Web search requested for …" paragraph (`rust/src/solver.rs:643`, `crate::solver_search::try_budget_search`) — this is the E138/#1173 defect, and this issue's classes make up 5 of its 22 misrouted probes.

R2. Summarization of arbitrary free text (not just repository files or README prose) formalizes the input into `Statement`s (subject/predicate/object/time), ranks them by weight, and returns a summary that names every entity and date carried by the statements selected under the requested length bound; it never echoes the input prompt verbatim (as the current "summary: the request is recorded as a bounded summarization task" answer does).

R3. Translation of an arbitrary sentence (not only a single quoted lemma/phrase) formalizes the full sentence into a sequence of `MeaningId`s via `TranslationPipeline::translate` and renders each in the target language using `data/seed/grammar-projection-rules.lino`'s successor for natural-language word order (see Design — that file today only serves Rust→JS/TS code translation and must not be reused as-is), never returning "I could not identify a source phrase to translate" for a well-formed sentence with no quotes.

R4. Rewriting changes only register features (formality, contraction expansion, abbreviation expansion) of the formalized statement graph and leaves every subject/predicate/object node unchanged; a held-out check compares the rewritten statement graph to the original and fails if any node differs.

R5. Grammar correction formalizes the sentence, checks person/number agreement between subject and verb and singular/plural agreement between numeral and noun against lexeme data, and reports each correction paired with the rule it applied (e.g. "she don't → she doesn't: third-person singular agreement").

R6. Writing genres (email, commit message, PR description) formalize the request's intent (recipient/reason/date for email; change kind/component/effect for commit message) and fill a structure retrieved from a named, cited style guide (Conventional Commits for commit messages — no such source is registered in the repository today; see Design), rendering in the register the request asks for.

R7. Every answer for these five classes carries its derivation as links (formalized statements, the source-language surface, the target grammar rule or style-guide clause applied) inspectable via `formal-ai explain <answer>`, per the umbrella's white-box requirement (#1183 acceptance item 2).

## Design

**Current routing.** `try_translation` (`rust/src/solver_handlers/mod.rs:243-330`) recognizes a translation request by meaning role (`ROLE_TRANSLATION_ACTION` etc. from `data/seed/meanings-translation.lino`), then calls `extract_unquoted_translation_surface`/`extract_quoted_phrase` to pull one **surface** (a word or short quoted phrase) and hands it to `crate::translation::TranslationPipeline` (`rust/src/translation/pipeline.rs`, 847 lines). When no quotable surface is found — as with a bare declarative sentence — `render_translation_gap` (`rust/src/solver_handlers/mod.rs:509-519`) returns "I could not identify a source phrase to translate from {source} to {target}." verbatim: this is the exact code path behind the E139 evidence. `TranslationPipeline::translate` formalizes one surface via Wiktionary/Wikidata (`rust/src/translation/wiktionary.rs`, `rust/src/translation/wikidata.rs`) into a `MeaningId` (`rust/src/translation/meaning.rs`) and deformalizes it in the target language — this is lemma/phrase-level translation, not sentence-level. Summarization and rewriting/grammar-correction/writing requests are not recognized by any dedicated `try_*` function today, so they fall through the whole `solve()` chain (`rust/src/solver.rs:274-650`) to `try_budget_search` (line 643), which is `web_requests.rs`'s canned paragraph.

**Modules to modify:**
- `rust/src/translation/pipeline.rs` — add a sentence-level entry point, e.g. `pub fn translate_sentence(&self, source: &str, source_lang: &str, target_lang: &str) -> SentenceTranslation`, that segments the sentence with `rust/src/formalization/segment.rs` (`Segment`/`Script`, already used for #1138 plan 04 formalization), formalizes each content word via the existing per-lemma `translate`, and orders the rendered surfaces using a **new** per-language word-order table (see data files below) rather than assuming source word order.
- `rust/src/solver_handlers/mod.rs` — `try_translation` should call `translate_sentence` when `extract_unquoted_translation_surface` finds no single quotable surface but the sentence is otherwise well-formed (has a finite verb per the grammar-projection data), instead of unconditionally returning `render_translation_gap`.
- New `rust/src/solver_handlers/summarization_request.rs` — `try_summarize_text(prompt, normalized, log) -> Option<SymbolicAnswer>`, wired into `solve()` ahead of `try_budget_search` (line 643) analogous to how `try_translation` is called. It formalizes with `rust/src/summarization/mod.rs`'s existing `Statement`/`StatementKind`/`summarize`/`deformalize` (already generic prose-capable via `describe_readme`'s Markdown→Statement path) rather than the repository-file-specific `summarize_repository_file`.
- New `rust/src/solver_handlers/text_rewrite.rs` — `try_rewrite_register(...)`, `try_grammar_correction(...)`, `try_genre_writing(...)`, each finalized through `crate::solver_handlers::finalize_simple` like every existing handler.
- `rust/src/solver.rs` — three new `if let Some(answer) = crate::solver_handlers::text_rewrite::try_*(...)` calls, and one for summarization, inserted before line 643 (`try_budget_search`) so these classes never reach the canned paragraph. The calls go through the capability-routing table of #1175 (E140, plan 10), not another ad hoc `if` chain.

**Data files.**
- `data/seed/grammar-projection-rules.lino` exists today but is a **generated, URL-encoded tree-sitter rule table for Rust→JS/TS code translation** (`translation-rule`/`translation-rule-match`/`translation-rule-template` records keyed by `rust:function_item` etc., consumed by `rust/src/es_meta.rs`) — it is not natural-language grammar data and must not be repurposed. A **new** `data/seed/natural-language-word-order.lino` is needed, one record per language, e.g.:
  ```
  word_order en
    default svo
  word_order hi
    default sov
  word_order zh
    default svo
  ```
  reusing the existing `natural_language` blocks in `data/seed/formal-language-projections.lino` (`statement "{subject} {predicate} {object}"` for en/ru/es, `"{subject} {object} {predicate}"` for hi, `"{subject}{predicate}{object}"` for zh) as the template rather than inventing a second schema — that file already carries per-language statement order and predicate surfaces for `P31`; it should be **extended** with more relations rather than duplicated.
- A **new** `data/seed/agreement-rules.lino` for grammar correction: subject-verb person/number agreement pairs and noun-number agreement, cited against a source (there is no existing morphology/agreement data file in the repository — confirmed by `git grep -lni "subject.verb\|agreement\b"` returning no natural-language grammar file, only unrelated "agreement" hits in `coding/composition.rs`, `event_log.rs`, etc.).
- A **new** `data/seed/writing-genre-styleguides.lino` registering style-guide sources per genre, e.g. `genre commit_message / source "Conventional Commits" / source_url "https://www.conventionalcommits.org/en/v1.0.0/"` — no such record exists today (`git grep -lni "conventional.commit\|commit_message"` matches only `authoring_loop.rs`, `cli_solve.rs`, `protocol_responses.rs`, `repository-workspace-protocol.lino`, none of which cite a style-guide source; they describe this repository's own commit convention, not a retrievable external standard). Add `data/seed/sources-registry.lino` entries for each style-guide URL so the fetch is provenance-tracked like every other trusted source.
- Register-feature lexicon (contractions/informal forms): a **new** `data/seed/register-lexicon.lino`, `informal "u" formal "you"`, etc.

**Browser/JS/TS parity.** Per the three-roots doctrine: each new Rust module is translated with `formal-ai translate --from rust --to js|ts --input <file> --write`, the browser worker (`js/worker/formal_ai_worker.js`) calls the JS root, and `data/parity/cross-runtime-synthesis.json` gains one case per class so all three roots answer identically.

## Probe set and tests

New per-class, per-language probe files mirroring the existing `data/benchmarks/capability-routing/{en,ru,hi,zh}.lino` layout (record shape: `record_type`, `id`, `language`, `prompt`, plus an expected-answer field):
- `data/benchmarks/text-transform/summarization/{en,ru,hi,zh}.lino` — ≥10 paragraphs each, with `expected_entities` and `expected_dates` lists the generated summary must contain.
- `data/benchmarks/text-transform/translation/{en,ru,hi,zh}.lino` — ≥10 sentences each, `target_language` field, graded by round trip (source → target → source preserves the `MeaningId` sequence — the #526 contract: "test roundtrips between each supported language pair via meta language", delivered for code by `rust/tests/unit/specification/translation_round_trip.rs` per `rust/tests/unit/docs_requirements/issue_526.rs:25,58,89`; this issue extends that same contract to natural-language pairs, which it does not yet cover).
- `data/benchmarks/text-transform/rewriting/{en,ru,hi,zh}.lino` — ≥10 informal sentences each, `informal_tokens` list that must be absent from the answer.
- `data/benchmarks/text-transform/grammar-correction/{en,ru,hi,zh}.lino` — ≥10 sentences with a known agreement error each, `expected_correction`.
- `data/benchmarks/text-transform/writing/{en,ru,hi,zh}.lino` — ≥10 email/commit-message/PR-description requests each, `expected_structure_fields`.

Test files: `rust/tests/unit/text_transform/summarization_probes.rs`, `rust/tests/unit/text_transform/translation_round_trip_natural_language.rs`, `rust/tests/unit/text_transform/rewriting_probes.rs`, `rust/tests/unit/text_transform/grammar_correction_probes.rs`, `rust/tests/unit/text_transform/writing_probes.rs`, each registered as `mod text_transform;` (with its own `mod.rs` listing the five files) in `rust/tests/unit/mod.rs` (which today declares 569 sibling test files, e.g. `mod agentic_coding;`, `mod docs_requirements;`). Run with:
```
RUSTUP_TOOLCHAIN=1.98.1 cargo test --test unit text_transform
```
`docs/llm-task-parity.md` (E136/#1171) rows for `summarization`, `translation`, `rewriting`, `grammar_correction`, `email_drafting`, `commit_message` are regenerated by that issue's generator against these probe files, the same way `docs/status.md` is regenerated by `scripts/render-status.rs --write` from `data/benchmarks/external-results.lino` and friends.

## Definition of done

- [ ] Requirement shard `docs/requirements/issue-1174-text-transform-formal-versions.md` created, assembled into `REQUIREMENTS.md` via `rust-script scripts/assemble-requirements.rs --write`.
- [ ] `docs/requirements-traceability.md` (hand-maintained per its own header) gets one row per requirement (R1-R7 here, renumbered into the shard's global IDs) with `Shard`, `Delivered`, `Automated test`, `Manual confirmation` columns filled — no "not yet confirmed" left where a manual check was actually run.
- [ ] Case study `docs/case-studies/issue-1174/` with this audit, the per-class plan, and a survey of existing style-guide/lexicon retrieval libraries.
- [ ] Changelog fragment via `rust-script scripts/create-changelog-fragment.rs --bump-type minor --description "Formal summarization, translation, rewriting, grammar correction and genre writing for free text"`, landing in `changelog.d/<timestamp>_issue-1174-text-transform.md` (format matching `changelog.d/20260926_215107_issue-1149-coverage-budget.md`: a `bump:` frontmatter block plus a `### Added`/`### Fixed` section).
- [ ] `RUSTUP_TOOLCHAIN=1.98.1 rust-script scripts/run-ci-gates.rs --stage rust` green (the `rust` stage is one of `STAGES: [&str; 3] = ["rust", "wasm", "web"]` in `scripts/run-ci-gates.rs:48`).
- [ ] No-memorization gate `rust/tests/unit/coding_discovery/no_memorization.rs` (scans `src/` and seed data for verbatim upstream corpus text) passes unchanged — the new register/agreement/style-guide data must be sourced, not hand-memorized full sentences.
- [ ] `docs/llm-task-parity.md` rows for this issue's six classes show a real answer, not the canned search paragraph.

## Depends on / blocks

- Depends on #1175 (E140) — routing by the formalized request sends "translate"/"rewrite"/"summarize"/"correct the grammar"/"write an email" to these handlers; and on #1173 (E138) — the canned search paragraph stops being the fallback.
- Depends on #1163 (E128) — live retrieval of style-guide pages (Conventional Commits) and grammar/lexeme data needs the generic any-URL-to-links formalization path, which does not exist on the solve path today (only 6 bespoke extractors per `rust/src/concept_lookup.rs:272`).
- Is one of the eleven task-class issues tracked by #1171 (E136)'s generated `docs/llm-task-parity.md`; that issue's probe runner consumes this issue's probe files.
- Records derivations in #1184 (E148) (R7).
- Blocks part of #1183 (E127)'s acceptance item 6 ("`docs/llm-task-parity.md` shows a useful, verified answer for every task class").


