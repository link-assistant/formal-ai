Part of the E127 umbrella (#1183); parity class **factual Q&A and explanation**.

## Evidence

- `formal-ai chat --prompt "What is the capital of Australia?"` (0.347.0) answers **"The capital of the United States is Washington, D.C."** `data/seed/facts.lino` holds capital facts for Japan (`:144`), Russia (`:180`) and the United States (`fact_capital_usa`, `:410`), each selected by `question_keywords ("capital" …)`; the keyword matched and the United States fact was returned although the question's subject is Australia, which has no fact. Unchanged on `main`.
- "Explain how the TCP three-way handshake works." and "What does the word ephemeral mean?" return the canned web-search paragraph (Wiktionary is a registered source).
- "Compare Rust and Go for writing web servers." returns only the Wikipedia definition of Rust.
- "Read this and answer: \"The meeting moved from Tuesday to Thursday at 3 pm in room 204.\" When and where is the meeting?" returns the canned web-search paragraph although the answer is in the prompt.

A wrong answer is worse than no answer; the seed fact table is memorization the 2026-09-14 note says not to rely on.

## What to do

- Answer factual questions from formalized knowledge: formalize the question (subject Q-id, property P-id, e.g. Australia Q408, capital P36), query Wikidata live, cite the statement and its references, and state the relative probability (E144). A seeded fact may be used only when its subject and property match the formalized question.
- A fact may be selected only when the formalized subject and property both match; otherwise the live path runs. Test: "capital of Australia" never returns another country's capital.
- Definitions from Wiktionary/Wikidata lexemes; comparisons as aligned property tables for both subjects (Wikidata properties plus retrieved documentation), with the differences stated.
- Questions over given text: formalize the text, answer from its statements (time, place), no search.
- Explanations: retrieve (E128), formalize the pages, select the statements that define the concept and its steps, and render them in the question's language with sources.
- Delete `data/seed/facts.lino` entries that the live path reproduces (keep a cache, not a table).

## How to test

Held-out capitals, populations, dates, authors across 50 countries/works in en/ru/hi/zh; any answer about a subject other than the one asked fails the test.

## Root cause (exact, verified against `main`)

The evidence entry above blames `question_keywords`, but the actual defect is one line further in: **`matches_normalized` accepts a `subject_alias` as a raw substring, not a whole-word match, and one of the United States' aliases is a substring of "Australia".**

- `data/seed/facts.lino:410` `fact_capital_usa` declares `subject_aliases ("united states" … "usa" "us" "america" …)`.
- `rust/src/seed/facts.rs:126-144` `FactRecord::matches_normalized` — its own doc comment: "at least one subject alias **and** at least one question keyword appear as **substrings** of `normalized`" — calls `normalized.contains(alias.as_str())`, never a token/word-boundary check.
- `"what is the capital of australia?"` lower-cased contains the substring `"us"` ("a**us**tralia") *and* the keyword `"capital"`, so `fact_capital_usa` matches before `fact_capital_japan`/`fact_capital_russia` are even reached (no fact for Australia exists to compete).
- The live selection call site is `rust/src/solver_handlers/benchmark_prompts.rs:315-322` `pub fn try_fact_lookup(...)`: `fact_records().iter().find(|record| record.matches_normalized(normalized))?` — first match wins, declaration order in `facts.lino` (Japan, Russia, … USA) decides ties, not subject correctness.
- `rust/src/solver_handlers/benchmark_prompts.rs:262-276` `fact_store_resolves` uses the same `matches_normalized` to tell the open-web planner (issue #989) the fact store "already answers" this prompt, so the wrong cached fact also *suppresses* the live Wikidata fallback that would have answered correctly.

**A prior requirement shard already claims this class is delivered and is now contradicted by measurement.** `docs/requirements/issue-0127-structured-fact-query-reasoning-requirements.md` (issue #127) lists R173-R180 as "Implemented", including R177 "On cache miss the browser worker must resolve the prompt against the live Wikidata API". The Australia probe is not a cache miss — it is a **substring false-hit** that never reaches the miss path, in the Rust engine this issue's probes were run against (the shard's own implementation notes are JS-worker-first: `parseFactQuestion`/`tryFactQuery`/`resolveFactQueryViaWikidata` in `js/worker/formal_ai_worker.js`; the JS worker has the identical substring bug: `js/worker/formal_ai_worker_05.js:495` `containsAny(normalized, fact.subjectAliases)`). This issue's fix must correct R173/R174/R177's status from "Implemented" to reflect the substring defect, not just add new requirements.

## Requirements

R1. `FactRecord::matches_normalized` (or its replacement) must match a subject alias only at a word boundary (tokenized comparison), never as a raw substring — "us" must not match inside "australia".
R2. A fact record may answer only when the formalized question's subject (Wikidata Q-id, once formalized) equals the record's `subject_qid`; alias/keyword substring matching is retired as the primary mechanism and kept only as a last-resort surface hint logged, never trusted alone.
R3. When no seeded fact's subject matches the formalized question subject, the engine queries Wikidata live (`sources-registry.lino`'s `wikidata` source, `wikidata_entity_v1` extractor) for the requested property and answers from the returned statement with its reference URL.
R4. `fact_store_resolves` must return `false` for a prompt whose formalized subject has no matching seeded record, so the open-web/live-query planner is not blocked by a false substring hit.
R5. Definitions ("What does X mean?") resolve via Wiktionary (`wiktionary_entry_v1`) or Wikidata lexemes, never the canned web-search paragraph (shared root cause with #1173).
R6. Comparisons ("Compare X and Y") retrieve both subjects' aligned Wikidata properties and documentation, and answer with a table naming stated differences for both sides, not just the first subject.
R7. Document Q&A over prompt-supplied text is answered purely from the formalized text's own statements (subject/predicate/object/time/place) with zero network calls.
R8. Explanations retrieve pages (E128/#1163), formalize them, select the statements that define the asked concept, and render an answer in the question's language, each claim citing its source.
R9. Every seeded `facts.lino` entry that the live Wikidata path can reproduce is deleted in favor of a fetched-and-cached record (cache, not a hand-authored table); entries kept must document why they cannot be derived live.
R10. The fix lands in all three roots: Rust (`rust/src/seed/facts.rs`, `rust/src/solver_handlers/benchmark_prompts.rs`), JavaScript (`js/worker/formal_ai_worker_05.js:495`, same substring defect) and TypeScript, pinned by a parity case.
R11. `docs/requirements/issue-0127-structured-fact-query-reasoning-requirements.md` rows R173, R174, R177 are corrected to reflect the substring defect (status changed from "Implemented" to "partial" with the defect cited) rather than left standing as contradicted by this issue's measurement.

## Design

- **`rust/src/seed/facts.rs`**: change `matches_normalized` (currently lines 126-144) to tokenize `normalized` (reuse `word_tokens`-style splitting already used in `rust/src/solver_terminal.rs`) and compare whole tokens/token-spans for `subject_aliases`, or better, drop alias matching as the gate entirely once R2's subject-Q-id check lands.
- **New/modified `rust/src/solver_handlers/factual_lookup.rs`** (new file; `benchmark_prompts.rs` currently owns `try_fact_lookup` at line 315 and is misnamed for this purpose — split fact lookup out, since `benchmark_prompts.rs` is meant for the benchmark corpus per its existing doc comments): formalizes the question via `crate::translation::formalization::formalize_prompt_candidates` (already used in `solver.rs:358`) to get a subject anchor (`FormalizationAnchorKind::WikidataItem`, e.g. Q408 for Australia) and a property anchor (P36 for capital), matches only records whose `subject_qid` equals the anchor, and on no match calls a new live-query function.
- **Live query**: a thin wrapper over `crate::source_fetch::CachedSourceClient` hitting the `wikidata` source's `api` template from `data/seed/sources-registry.lino` (`https://www.wikidata.org/wiki/Special:EntityData/{id}.json`), extracting the property statement, its `references`, and rendering via `localized_response`-style templates per language (en/ru/hi/zh/es already have `sources-registry.lino api_language` coverage).
- **Definitions/comparisons**: route through the same formalization path into `wiktionary`/`wordnet` sources (already registered with extractors `wiktionary_entry_v1`, `wordnet_sense_v1`); comparison renders a table of Wikidata properties present on both Q-ids.
- **Document Q&A**: reuse `rust/src/formalization/segment.rs` (sentence/clause segmentation) and `rust/src/formalization/needs.rs`/`concepts.rs` to formalize the supplied text only, with no `source_fetch` call — a router check in `capability-routing.lino` must classify "text is given in the prompt" as `locus prompt`, distinct from `locus web`.
- **JS/TS parity**: the browser worker has the identical defect: `js/worker/formal_ai_worker_05.js:495` selects a fact with `containsAny(normalized, fact.subjectAliases)` (substring). Fix the Rust side and translate it (`formal-ai translate --from rust --to js|ts --input rust/src/seed/facts.rs --write`), or fix `formal_ai_worker_05.js` and its TS twin in the same pull request; `data/parity/cross-runtime-synthesis.json` gains the Australia case for all three roots.

## Probe set and tests

New file layout, one per language, mirroring the existing per-language benchmark corpora (e.g. `data/benchmarks/capability-routing/en.lino`):

- `data/benchmarks/factual-qa/en.lino`, `ru.lino`, `hi.lino`, `zh.lino` — ≥10 prompts per sub-class per language: **factual lookup** (10 held-out country/capital/population/currency prompts not in `facts.lino`, including the literal regression `"What is the capital of Australia?"`), **definition** (10 words not seeded), **comparison** (10 two-subject pairs), **document Q&A** (10 prompt-embedded-text questions). Each record: `prompt`, `expected_subject_qid`, `expected_property_pid` (or `expected_answer_contains` for document Q&A), `must_not_contain` (the wrong-country/wrong-subject string, e.g. `"Washington"` for the Australia case).
- Test file: `rust/tests/unit/issue_1172_factual_qa_probes.rs` (new), registered as `mod issue_1172_factual_qa_probes;` in `rust/tests/unit/mod.rs` — asserts (a) the answer's cited subject matches `expected_subject_qid`, (b) `must_not_contain` is absent, (c) a citation URL is present for live-path answers.
- Run: `RUSTUP_TOOLCHAIN=1.98.1 cargo test --test unit issue_1172_factual_qa_probes`.
- `docs/llm-task-parity.md` (E136/#1171) rows for `factual_qa`, `explanation`, `definition`, `comparison`, `document_qa` are regenerated by re-running `formal-ai benchmark --suite llm-task-classes` (per #1171's design) and `rust-script scripts/render-status.rs --write`.

## Definition of done

- [ ] Requirement shard `docs/requirements/issue-1172-factual-qa-subject-verified-lookup.md` (naming follows the existing `issue-NNNN-slug-requirements.md` convention seen in `docs/requirements/`), assembled with `rust-script scripts/assemble-requirements.rs --write`.
- [ ] `docs/requirements-traceability.md` gets a row per new requirement ID; note this file is maintained **by hand** (its own header states so explicitly), not generated — the row is added in the same PR, with the automated test cited and a manual-confirmation note (`not yet confirmed` is honest and acceptable per the file's stated convention until someone runs it by hand).
- [ ] Case study `docs/case-studies/issue-1172/` recording this root-cause analysis, the substring-match defect, and the correction to issue-0127's R173/R174/R177 status.
- [ ] Changelog fragment via `rust-script scripts/create-changelog-fragment.rs --bump-type minor --description "Factual Q&A answers only for the formalized question's own subject; live Wikidata fallback for uncached subjects"`.
- [ ] `RUSTUP_TOOLCHAIN=1.98.1 rust-script scripts/run-ci-gates.rs --stage rust` green.
- [ ] No-memorization gate (`rust/tests/unit/coding_discovery/no_memorization.rs`) respected: the live-query path must not grow a second hardcoded per-country answer table; only the formalization → Wikidata → render pipeline may answer uncached subjects.

## Depends on / blocks

- Depends on #1163 (E128 — internet as formal knowledge) for the generic fetch-and-formalize path the live Wikidata/Wiktionary queries need.
- Uses the existing question formalizer `crate::translation::formalization::formalize_prompt_candidates` for the subject/property anchors (R2); R1 (word-boundary matching) needs nothing else and lands first.
- Shares its routing precondition with #1175 (E140): the router must send "text is given in the prompt" (document Q&A) to a no-search path before this issue's document-Q&A requirement can pass.
- Shares its "no canned paragraph" outcome with #1173 (E138): definitions/explanations currently fall through to the same canned text this issue must also stop producing.
- Feeds #1171 (E136): its `docs/llm-task-parity.md` rows for factual_qa/explanation/definition/comparison/document_qa are the measured outcome of this issue.
- Part of umbrella #1183 (E127).

