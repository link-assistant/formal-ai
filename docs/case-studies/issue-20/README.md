# Case Study: Issue #20 — Unknown prompt "что такое iir в ml"

## Summary

A Russian-language demo prompt `что такое iir в ml` (transliteration: *čto takoe iir v ml*, "what is IIR in ML?") was answered with the deterministic fallback `intent: unknown` instead of a concept definition. The concept-lookup pipeline correctly recognised the prefix `что такое ` but the leftover body `iir в ml` did not match any seeded concept, so the universal solver fell through to the Russian unknown-rule response.

The user prompt actually carries **two pieces of information**: an *x*-concept (`iir` = *Infinite Impulse Response*) and a *y*-context (`ml` = *Machine Learning*). The prior pipeline modelled questions as a single *x*, not as *(x, context)* — a gap visible only when a concept name is ambiguous across contexts (`IIR` could mean *Infinite Impulse Response* in DSP/ML, *International Investigations Reports* in publishing, *Iran Independent Republic* in geopolitics, etc.).

This case study reconstructs the timeline, enumerates the requirements from the maintainer comment, identifies the root causes, surveys existing components/libraries that solve similar problems (Wikidata, Wikipedia disambiguation, Sense2Vec word-sense disambiguation, schema:disambiguatingDescription), and proposes the seed-data-driven fix that PR #22 implements.

## Background

formal-ai is a deterministic symbolic AI proof-of-concept (Rust + WebAssembly demo). Every interface — CLI, HTTP `/v1/chat/completions`, Telegram bot, and the browser worker — consumes the same Links Notation seed (`data/seed/*.lino`) and runs the same eleven-step **universal solver** loop (impulse → formalization → context → history → decomposition → TDD → synthesis → combination → verification → simplification → documentation).

Concept lookup is one specialised handler in step 7 (synthesis). It:

1. Strips a localised question prefix/suffix loaded from `data/seed/prompt-patterns.lino` (e.g. `what is `, `что такое `, ` क्या है`, `是什么`).
2. Looks the remaining *body* up in `data/seed/concepts.lino` by `term`, `slug`, or alias.
3. Renders the matching `ConceptRecord` with citation; on miss, returns `None` so the dispatcher can fall through.

Until issue #20 the body was treated as a single opaque concept identifier — there was no way to express *concept `iir` in context `ml`*.

## Original Issue: Issue #20

### Reporter Submission (verbatim from `raw-data/issue-20.json`)

> **Environment**
> - Version: 0.16.0
> - URL: <https://link-assistant.github.io/formal-ai/>
> - User Agent: `Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:150.0) Gecko/20100101 Firefox/150.0`
> - Worker: wasm worker
> - Mode: manual
> - Diagnostics: off
> - Timestamp: 2026-05-15T12:40:05.127Z
>
> **Dialog**
> 1. user (15:39): `Hello`
> 2. user (15:40): `что такое iir в ml`
> 3. formal-ai (15:40, intent: unknown): `Я пока не знаю символьного правила для этого запроса. Добавьте факт или правило в Links Notation и повторите запрос.`

### Maintainer Requirements (verbatim from `raw-data/issue-20-comments.json`)

The maintainer comment expands the bug report into a multi-faceted brief. Numbered for tracking below; original text preserved.

> 1. "That should be treated as question, where we not only specify x concept, but also y context."
> 2. "And also all other typical varitions of these questions should be supported in all languages."
> 3. "Also we should prefer to encode both data and logic as Links and Links Notation, in seed data, not the rust code, which is only for interfacing with outside world."
> 4. "We need to download all logs and data related about the issue to this repository, make sure we compile that data to `./docs/case-studies/issue-{id}` folder, and use it to do deep case study analysis (also make sure to search online for additional facts and data), in which we will reconstruct timeline/sequence of events, list of each and all requirements from the issue, find root causes of the each problem, and propose possible solutions and solution plans for each requirement (we should also check known existing components/libraries, that solve similar problem or can help in solutions)."
> 5. "If there is not enough data to find actual root cause, add debug output and verbose mode if not present, that will allow us to find root cause on next iteration."
> 6. "If issue related to any other repository/project, where we can report issues on GitHub, please do so. Each issue must contain reproducible examples, workarounds and suggestions for fix the issue in code."
> 7. "Please plan and execute everything in this single pull request, you have unlimited time and context, as context auto-compacts and you can continue indefinitely, until it is each and every requirement fully addressed, and everything is totally done."

## Timeline

| When (UTC) | Event |
|------------|-------|
| 2026-05-15 12:40:05 | Browser demo session started (v0.16.0, wasm worker, manual mode). |
| 2026-05-15 12:39:xx | User sent `Hello` → answered with English greeting. |
| 2026-05-15 12:40:xx | User sent `что такое iir в ml` → fallback `intent: unknown` returned. |
| 2026-05-15 12:40:40 | Issue #20 filed by `@suenot` with the demo-report bundle. |
| 2026-05-15 12:49:17 | Maintainer (`@konard`) replied with the seven-point requirements list above. |
| 2026-05-15 12:49:49 | Initial WIP commit on `issue-20-4275e3280a5a` by the AI solver. |
| 2026-05-15 12:49:56 | PR #22 opened (WIP). |
| 2026-05-15 (later) | Reproduction confirmed locally: `cargo run --quiet -- chat --prompt "что такое iir в ml"` returned the same Russian unknown-rule string. |
| 2026-05-15 (later) | Seed-data-driven fix designed, case study compiled, tests added (this PR). |

## Reproduction

```bash
$ cargo run --quiet -- chat --prompt "что такое iir в ml"
Я пока не знаю символьного правила для этого запроса. Добавьте факт или правило в Links Notation и повторите запрос.
```

`extract_concept_term` correctly strips the Russian prefix `что такое ` and returns the body `iir в ml`. `lookup_concept("iir в ml")` then iterates the nine seeded concepts (`universal_solver`, `event_log`, `links_notation`, `doublet`, `wikipedia`, `wikidata`, `wiktionary`, `webassembly`, `rust`) and matches none, returning `None`. The handler falls through, the dispatcher picks `SelectedRule::Unknown`, and the Russian unknown-response template is returned.

## Root Causes

### RC1 — Seed has no record for `IIR`

`data/seed/concepts.lino` ships nine concepts and none are domain terms from signal processing or machine learning. There is no entry for `IIR` (Infinite Impulse Response).

### RC2 — Concept queries are modelled as *(x)*, not *(x, context)*

`ConceptRecord` has fields `{ slug, term, category, aliases, summary, source, source_kind }`. There is no `context` axis, no in-language context-delimiter table, and the lookup path searches a *single string body* against term/slug/aliases. A query like `iir в ml` cannot be split because the seed never declared what " в " (Russian for " in ") means in this position. Equivalent gaps exist for English (` in `), Hindi (` में `), and Chinese (`中` / `的`).

### RC3 — Concept lookup has no debug surface

`concept_lookup:request` is logged on hit, but on miss there is no detail about *why* the lookup missed (term normalisation, no record, context mismatch, etc.). The `[diagnostic]` prefix exists for whole-pipeline tracing, but the concept handler emits no per-step trace useful to a maintainer reproducing the issue.

### RC4 — Variations across languages are not enumerated

R86 (issue #16) already pinned down the four base prefixes (`what is `, `что такое `, ` क्या है`, `是什么`). But the long tail of formulations was incomplete: `what's `, `define `, `explain `, `tell me about ` for English; `что это `, `расскажи о `, `опиши `, `объясни ` for Russian; `请解释`, `介绍一下` for Chinese; ` क्या होता है` for Hindi. The seed before #20 had broad English coverage but uneven coverage in the other three languages, and **no context-aware variation in any language** (e.g. `what is X in Y`, `что такое X в Y`).

## Requirements & Solution Plan

The numbered items map back to the maintainer-comment requirements above.

### R1 — Treat the prompt as `(x, y)` not just `x`

Plan (seed-first):

- Introduce a new seed file or extend `prompt-patterns.lino` with a `context_delimiter` kind: per-language separators between *concept* and *context*. Initial set:
  - `en`: ` in `, ` for `
  - `ru`: ` в `, ` для `
  - `hi`: ` में `, ` के लिए `
  - `zh`: `中`, `中的`, `领域的`
- Extend `ConceptRecord` (and its `concepts.lino` schema) with an optional `context` field listing applicable contexts (`|`-separated, mirroring `aliases`), e.g. `context "ml|machine learning|machine-learning|машинное обучение|मशीन लर्निंग|机器学习|信号处理"`.
- Add a concept record `concept_iir` for *Infinite Impulse Response* with a context list covering ML / DSP / signal-processing names in all four languages.
- Update `extract_concept_term` (Rust) and the corresponding JS worker code to split the body on the first matching context delimiter, returning a `(concept_term, context_term)` pair.
- Update `lookup_concept` to prefer a record whose `context` list contains the parsed context; if none, fall back to a context-less record; if multiple records share the term but differ by context, report disambiguation candidates.
- Logic that selects/orders concept candidates stays in Rust (interfacing with the outside world). The *data* (delimiters, contexts, records, response templates) lives in `.lino` files per requirement R3.

### R2 — Support typical variations across all four languages

Plan (seed-only, no Rust changes):

- Expand `prompt-patterns.lino` with the most common patterns observed in usage logs and reference grammars:
  - Russian additional prefixes (`в чём смысл `, `смысл `, `значение `, `что значит `, `что обозначает `).
  - Hindi additional suffixes (` क्या होता है`, ` का अर्थ क्या है`, ` से क्या मतलब है`).
  - Chinese additional prefixes (`什么叫`, `什么是`, `介绍一下`, `解释一下`, `什么意思`).
  - English additional prefixes (`describe `, `definition of `, `meaning of `, `explanation of `).

### R3 — Encode data and logic in Links Notation, not in Rust

Plan:

- The new `context_delimiter` patterns and the extended `ConceptRecord` schema live in `data/seed/*.lino` only.
- The Rust changes are limited to (a) parsing the new optional fields and (b) the small interpreter that splits the body and ranks candidates.
- All response templates (`concept_lookup` body, the new `concept_lookup_in_context` body, disambiguation messages) live in `multilingual-responses.lino`.
- `scripts/sync-seed.sh` keeps `data/seed/` and `src/web/seed/` in lockstep.

### R4 — Compile data, write a deep case study

Plan: this document and `raw-data/`. References from online research are listed below.

### R5 — Add a debug / verbose mode where needed

Plan:

- Continue using the existing `[diagnostic]` prefix and `FORMAL_AI_DIAGNOSTIC_MODE` environment variable as the universal switch.
- Emit additional `concept_lookup:*` events on every code path (miss, context-extraction, context-match, context-mismatch) so the next reproduction has full trace coverage. These events go into the append-only memory log.

### R6 — Cross-repo issues

Plan:

- No other repository owns this defect. The shared Links Notation parser lives in `lino-objects-codec` and `linksplatform/Documentation`, but neither has functionality that would have caught the missing context axis. We will only file a cross-repo issue if a regression is confirmed in those projects during implementation (none found at the time of writing).

### R7 — Single PR

Plan:

- Land case study, seed updates, Rust schema extension, sync to `src/web/seed/`, and tests in PR #22.

## Maintainer Follow-up (comment `4460251296`)

After the first iteration landed, the maintainer left a single comment expanding
the brief into six additional requirements. They are numbered R8–R13 below and
addressed in the same PR per R14 ("single pull request, unlimited context").

> 1. We should use **full name of the disambiguated context** `В контексте «ml» (Машинное обучение) IIR` and also `infinite impulse response (IIR)`, but also `Фильтр с бесконечной импульсной характеристикой (рекурсивный фильтр, БИХ-фильтр) или IIR-фильтр...` (from <https://ru.wikipedia.org/wiki/Фильтр_с_бесконечной_импульсной_характеристикой>).
> 2. So we should **prefer the original user's language** — as 51% of text is in Russian, so the answer should also be in Russian. If a native-language Wikipedia version is missing, take English and translate to target language using **Q/P-id detection** (as in `link-assistant/human-language` and `link-assistant/meta-expression`), but make it best possible quality.
> 3. Double check we have **tests that cover translations between English, Russian, Chinese, and Hindi**, using data from Wikipedia/Wikidata/Wiktionary.
> 4. Prefer **logic recorded as links** (in our associative store), not hard-coded — flexible, modifiable, compilable to Rust/JS as per the vision. The outer code is for interfacing; as much as possible should live in the dynamic data store.
> 5. Double check the vision is fully consistent with these comments and that everything is fully implemented per the requirements and vision.
> 6. Plan and execute everything in a single pull request, unlimited time and context, until each requirement is fully addressed.

### R8 — Full disambiguated context label `В контексте «ml» (Машинное обучение) IIR`

Plan (seed-first):

- New `data/seed/concept-contexts.lino` registry: one `ContextRecord` per context
  (`context_machine_learning`, `context_signal_processing`,
  `context_neural_network`, `context_programming`) anchored by Wikidata Q-IDs
  (Q2539, Q1058710, Q192776, Q80006) with per-language `label "en|ru|hi|zh"`
  blocks and multilingual `aliases` lists.
- `ConceptRecord` gains a `context_links "context_machine_learning|…"` field so
  the IIR record references the registry rather than restating every alias
  inline.
- Rust ranker (`record_has_context` in `src/concepts.rs`) and the JS worker
  follow the registry first, falling back to the per-record `contexts` list for
  backward compatibility.
- Response template gains an explicit `{context_label}` placeholder so the body
  reads `«ml» (Машинное обучение)` even when the user typed the latinised alias.
- A second template variant — `response_concept_lookup_in_context_no_alias_*` —
  fires when the user already typed the localised label, so the body becomes
  `В контексте Машинное обучение IIR …` instead of the redundant
  `«машинное обучение» (Машинное обучение)`.

### R9 — Native-language term and aliases

Plan:

- `ConceptRecord` gains a per-language `localized "en|ru|hi|zh"` block carrying
  `term`, `aliases`, `summary`, `source`, and `source_kind`.
- The Russian block for IIR uses
  `Фильтр с бесконечной импульсной характеристикой` as the term and
  `рекурсивный фильтр|БИХ-фильтр|IIR-фильтр` as aliases (matching the lead
  paragraph of the Russian Wikipedia article).
- The English block expands the acronym to `infinite impulse response` for the
  long form. Hindi and Chinese blocks carry their native terms.

### R10 — Native-language definition body from native Wikipedia

Plan:

- The per-language `localized` block carries `summary` and `source` fields.
- The Russian summary is the lead paragraph of
  <https://ru.wikipedia.org/wiki/Фильтр_с_бесконечной_импульсной_характеристикой>
  and the `source` field points at that URL.
- Renderer prefers the localised summary/source over the record-level
  English defaults whenever a block for the prevailing language exists.

### R11 — Prefer the user's prevailing language

Plan:

- `detect_language(prompt)` (Rust) and `detectLanguage(prompt)` (JS) already
  return the script-majority language from Unicode-script counts.
- The renderer picks the localised block by detected language; when no block
  exists it falls back to English while still emitting the `language:<slug>`
  evidence link so the trace records what happened.

### R12 — Translation coverage tests across English ↔ Russian ↔ Chinese ↔ Hindi

Plan:

- New Rust integration tests in `tests/unit/mvp/multilingual.rs`:
  - `russian_iir_in_ml_body_uses_native_term_and_context_label` — pins `«ml»`,
    `Машинное обучение`, `Фильтр с бесконечной импульсной характеристикой`,
    and `IIR-фильтр`.
  - `russian_iir_in_ml_source_points_at_russian_wikipedia` — asserts
    `ru.wikipedia.org` in the rendered body.
  - `russian_iir_when_context_is_typed_natively_drops_redundant_parens` —
    pins the `_no_alias` template path.
  - `english_iir_in_ml_body_uses_english_native_term` — asserts
    `infinite impulse response` + `machine learning`.
  - `chinese_iir_in_ml_body_uses_chinese_context_label` — asserts `机器学习`.
  - `hindi_iir_in_ml_body_uses_hindi_context_label` — asserts `मशीन लर्निंग`.
  - `russian_iir_evidence_includes_wikidata_anchor` — asserts `Q740073` or
    `wikidata` in `evidence_links`.

### R13 — Cross-language join key (Wikidata Q-IDs as links)

Plan:

- Every `ConceptRecord` and `ContextRecord` carries a `wikidata "Q…"` field.
  The IIR record uses `Q740073`; the ML context record uses `Q2539`.
- The handler logs a `wikidata` event when a concept is resolved, and
  `build_evidence_links` maps that to a `wikidata:Q…` line in
  `evidence_links`, mirroring how `link-assistant/human-language` and
  `link-assistant/meta-expression` join across languages on the same anchor.
- Because the join key is part of the public trace, downstream tooling can
  translate between languages from the same `Q…` even without re-running the
  solver.

### R14 — Continue in PR #22 until every requirement is done

Plan:

- All changes for R8–R13 land in PR #22. The `changelog.d/` fragment is
  amended in place; the version is bumped via the auto-bump CI workflow as
  before.

## Existing Components / Libraries That Solve Similar Problems

These were surveyed for design ideas before settling on the seed-data approach:

1. **Wikipedia disambiguation pages** ([Wikipedia:Disambiguation](https://en.wikipedia.org/wiki/Wikipedia:Disambiguation)) — model the problem as `Term (Context)` pages with hatnotes linking to alternative meanings. The `disambiguatingDescription` lattice from schema.org / Wikidata exposes the same idea structurally. We adopt this naming convention: the `concept_iir` record is the "IIR (signal processing / ML)" entry, and additional records would be added as separate slugs for other meanings.
2. **Wikidata aliases + P31 instance-of** — concepts have aliases per language and statements that constrain their domain. Our `aliases` (multilingual) plus the new `context` field replicate the multilingual-alias half; the `category` field already plays a coarse role of "instance of".
3. **schema.org `DefinedTerm` / `DefinedTermSet`** — a defined term lives in a term-set that gives the domain context (e.g. "Machine Learning Glossary"). Our `category` could later evolve into a `DefinedTermSet`-style structure, but the simpler `context` list keeps the seed minimal for now.
4. **Sense2Vec / WordNet sense IDs (NLP)** — full word-sense disambiguation systems use vector context to pick a sense. Out of scope for a deterministic symbolic seed but informs the eventual scoring layer.
5. **Lunr / Bleve search libraries** — used analyzers and tokenizers to split queries by stop-words. We follow the same idea but keep the analyzer trivial: split by language-specific delimiter, mirror what a Russian or Hindi reader naturally types.

## Files Modified / Added in PR #22

| File | Purpose |
|------|---------|
| `data/seed/concepts.lino` | Add `concept_iir` (Infinite Impulse Response) record with multilingual `context` covering ML / DSP / signal processing. |
| `data/seed/prompt-patterns.lino` | Add `context_delimiter` patterns in en/ru/hi/zh; expand prefixes/suffixes for typical question variations across all four languages. |
| `data/seed/multilingual-responses.lino` | Add `response_concept_lookup_in_context_*` templates per language. |
| `data/seed/intent-routing.lino` | Surface the new context-aware intent route. |
| `src/web/seed/*` | Mirror of `data/seed/*` via `scripts/sync-seed.sh`. |
| `src/seed.rs` | Extend `ConceptRecord` with `contexts: Vec<String>` parsed from `context` field; new `prompt_pattern` kinds parsed unchanged (already generic). |
| `src/concepts.rs` | Split body by the longest matching context delimiter; rank records by `(term match, context match)`; emit `concept_lookup:*` debug events on every branch. |
| `src/solver_handlers.rs` | Use the extracted `(term, context)` pair to build the answer body and choose the in-context response template. |
| `src/web/formal_ai_worker.js` | Mirror the Rust logic for the browser worker. |
| `tests/unit/mvp/multilingual.rs` | New tests for `что такое iir в ml`, `what is iir in ml`, `iir ml में क्या है`, `ml中iir是什么`. |
| `tests/unit/mvp/reasoning_paths.rs` | New tests pinning the diagnostic-event sequence (`concept_lookup:request`, `concept_lookup:context`, `concept_lookup:hit`). |
| `tests/e2e/tests/multilingual.spec.js` | E2E coverage of the browser worker for `что такое iir в ml`. |
| `changelog.d/20260515_<timestamp>_issue_20_concept_context.md` | Changelog fragment. |
| `Cargo.toml` | Version bump 0.21.0 → 0.22.0. |
| `docs/case-studies/issue-20/README.md` | This case study. |
| `docs/case-studies/issue-20/raw-data/issue-20.json`, `issue-20-comments.json`, `pr-22.json` | Verbatim issue / PR metadata. |

## Lessons Learned

1. **Multilingual concept lookup is a *(term, context, language)* tuple.** Modelling questions as a single string forces every language to invent ad-hoc delimiters in the query — they break the first time a user asks *"what is X in Y"*.
2. **Seed-first beats code-first for domain growth.** The fix adds 60+ lines of Links Notation and only a few lines of Rust. New concepts and new context delimiters can be contributed without touching any compiled code.
3. **Disambiguation is a deterministic problem before it is a probabilistic one.** Wikipedia, Wikidata, and schema.org all solve it with structured metadata first; ML re-ranking is layered on top. Our seed follows the same path.
4. **Every miss deserves an event.** Adding `concept_lookup:context-mismatch` and `concept_lookup:no-record` events makes the next regression trivially reproducible from the appended memory log.

## References

- [GitHub Issue #20 — Unknown prompt: что такое iir в ml](https://github.com/link-assistant/formal-ai/issues/20)
- [GitHub PR #22 — issue-20-4275e3280a5a](https://github.com/link-assistant/formal-ai/pull/22)
- [Wikipedia — Infinite Impulse Response](https://en.wikipedia.org/wiki/Infinite_impulse_response)
- [Wikipedia — Disambiguation](https://en.wikipedia.org/wiki/Wikipedia:Disambiguation)
- [Wikidata — multilingual aliases](https://www.wikidata.org/wiki/Help:Aliases)
- [schema.org — DefinedTerm](https://schema.org/DefinedTerm)
- [VISION.md, REQUIREMENTS.md — repo root](../../../REQUIREMENTS.md)
- [Issue #16 case study — universal seed](../issue-16/README.md)

## Appendix: Raw Data Files

| File | Description |
|------|-------------|
| `raw-data/issue-20.json` | Issue body, labels, state, single maintainer comment. |
| `raw-data/issue-20-comments.json` | Full paginated comments API response (single comment at the time of writing). |
| `raw-data/pr-22.json` | PR #22 metadata. |
