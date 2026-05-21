# Issue 207 Case Study

## Scope

Issue: <https://github.com/link-assistant/formal-ai/issues/207>

Pull request: <https://github.com/link-assistant/formal-ai/pull/208>

Branch: `issue-207-40aa862c0b6e`

The report follows up on issue 190. After PR 191 wired up the Russian
prompt `Переведи "как у тебя дела?" на английский.` to a dedicated
translation handler, the resulting answer body was still composed as a
robot-shaped `meaning: … / surface (ru): … / surface (en): …` block. The
user pointed out three remaining problems:

1. The response feels robotic instead of like natural conversation.
2. The translation does not preserve the original formatting — when the
   user starts the quoted phrase with a lowercase letter, the answer
   capitalizes it (`How are you?`) instead of keeping the original case.
3. Only a single hardcoded meaning ID (`meaning_2cfc55c914d57d9e`,
   which encodes the canonical token `greeting_how_are_you`) actually
   resolves to a localized surface form; every other prompt falls back
   to a placeholder like `[en] …`.

Issue 207 asks the project to address all three through a single
**formalize → meaning → deformalize** pipeline, document the supporting
guidelines in `REQUIREMENTS.md` / `VISION.md` / `ARCHITECTURE.md`, and
record the case study analysis in this folder.

## Local Evidence

Downloaded artifacts live alongside this README:

- `raw-data/issue-207.json`: issue payload at collection time.
- `raw-data/issue-207-comments.json`: issue comments (empty at the time
  of collection).
- `raw-data/pr-208.json`: PR 208 metadata snapshot.
- `raw-data/pr-208-conversation-comments.json`: PR conversation
  comments (empty at the time of collection).
- `raw-data/pr-208-review-comments.json`: PR review comments (empty at
  the time of collection).
- `raw-data/online-research.md`: curated external references that
  ground the solution plan (Wikipedia / Wikidata / Wiktionary API
  shapes, Abstract Wikipedia, casing style guides, reusable
  components).

## Online Research

A complete reading list lives in `raw-data/online-research.md`. The
most load-bearing references are:

- Wikipedia REST page summary, interlanguage links, and Wikidata
  `EntityData` endpoints — these let a deterministic renderer translate
  a Q-id into any supported language without bundling a static label
  table.
- Wiktionary REST definition API — already integrated through
  `fetchWiktionaryEntry` in `src/web/formal_ai_worker.js`; it already
  exposes a `translations` block for source-language headwords.
- Abstract Wikipedia / Wikifunctions — confirms that a renderer
  parameterized by `(formalized_graph, target_language)` is a viable
  architecture for formal-ai's offline first symbolic pipeline.
- English / Russian style guides on mid-sentence capitalization and
  terminal punctuation — these justify the requirement to preserve the
  source-fragment formatting in the translated surface.

## Timeline

- 2026-05-21 12:27 UTC: User reported issue 207 from the GitHub Pages
  demo (formal-ai v0.86.0), highlighting the robotic translation feel
  and the hardcoded meaning ID.
- 2026-05-21 12:59 UTC: Branch `issue-207-40aa862c0b6e` prepared and
  PR 208 opened as a draft.
- 2026-05-21: Issue, PR, and online-research artifacts downloaded into
  `docs/case-studies/issue-207/raw-data/`.
- 2026-05-21: Pipeline redesign implemented — Rust solver and browser
  worker both now route translation through a shared
  `formalize → meaning → deformalize` pipeline with explicit case and
  terminal-punctuation preservation.

## Root Causes

1. **Robotic body builder.** `try_translation` in
   `src/solver_handlers/mod.rs` (and the matching `tryTranslation` in
   `src/web/formal_ai_worker.js`) packed the meaning ID and both
   surface forms into a multi-line `meaning: … / surface (ru): … /
   surface (en): …` body. The Links Notation trace already lives in the
   `links_notation` field and `evidence_links`, so the multi-line body
   duplicates that information in a form that no human conversational
   partner would write.
2. **Case/punctuation loss.** `translate_surface` in
   `src/solver_helpers.rs` mapped a lowercase Russian phrase
   (`как у тебя дела?`) to a capitalized English template
   (`How are you?`) with no awareness of the source-fragment casing or
   terminal punctuation. The browser worker mirrored the same loss.
3. **Single-meaning coverage.** `canonical_meaning_token` only knew
   `greeting` and `greeting_how_are_you`. Every other quoted phrase
   produced an opaque `meaning_<hash>` ID with no localized deformalize
   target, so the response fell back to `[en] …` placeholders. The same
   limit applied to the browser worker.

## Requirement Traceability

| Requirement | Implementation | Verification |
| --- | --- | --- |
| Translation responses must feel like natural conversation. | The Rust handler in `src/solver_handlers/mod.rs` and the browser worker in `src/web/formal_ai_worker.js` now answer with just the deformalized target surface (no `meaning: … / surface (…): …` block). The meaning ID, source/target language, and quoted phrase remain in `evidence_links` and `links_notation` for traceability. | `tests/unit/specification/translation_via_links.rs` asserts the natural-form answer and the absence of the `surface (` template. |
| Translations must preserve the source formatting (initial casing and terminal punctuation). | `match_source_formatting` in `src/solver_helpers.rs` and `matchSourceFormatting` in `src/web/formal_ai_worker.js` apply the source's leading capitalization and terminal punctuation to the target surface. | Tests cover both lowercase `как у тебя дела?` and uppercase `Как у тебя дела?` source variants. |
| The pipeline must translate every registered meaning, not only `meaning_2cfc55c914d57d9e`. | `formalize_surface` / `deformalize_meaning` in `src/solver_helpers.rs` route through a shared meaning registry covering greetings, farewells, gratitude, polite responses, identity probes, and yes/no answers in English, Russian, Hindi, and Chinese; the browser worker uses the same registry. | Tests cover Russian↔English greetings, gratitude, and yes/no answers, plus Hindi and Chinese variants. |
| The formalize → meaning → deformalize architecture must be documented. | `REQUIREMENTS.md` records R213 / R214 / R215; `ARCHITECTURE.md` section 10 references the registry and the casing-preservation rule. | `tests/unit/docs_requirements.rs` already pins the requirement IDs; new entries appear in the on-disk file. |
| The Wikipedia / Wikidata / Wiktionary fallback path must be documented for the future online enrichment. | `raw-data/online-research.md` records the API shapes, and `ARCHITECTURE.md` section 10 lists them as the documented enrichment fallback. | Inspected manually. |
| The case study must include issue, PR, comment, and online-research artifacts. | This folder. | Local file listing. |

## Fixes

- Added a shared offline meaning registry (`MEANING_SURFACE_REGISTRY`
  in `src/solver_helpers.rs`) covering greetings, polite follow-ups,
  gratitude, farewells, identity probes, yes/no answers, time-of-day
  greetings, and well-being checks in English, Russian, Hindi, and
  Chinese.
- Replaced the hand-written `translate_surface` with a
  `formalize_surface` / `deformalize_meaning` pipeline. Source-language
  surfaces collapse to a canonical token, hash to a stable meaning ID,
  then re-render into the target language.
- Added `match_source_formatting` (Rust) and `matchSourceFormatting`
  (JS) helpers that copy the leading capitalization and terminal
  punctuation from the source surface onto the translated surface.
- Rewrote the translation response body so the answer is just the
  deformalized surface form (still preserved within quotes when the
  user quoted the source) instead of the `meaning: … / surface (…): …`
  template. The meaning ID, source, and target are still emitted to the
  event log and to `evidence_links` so the trace remains inspectable.
- Mirrored every change in the browser worker so the deployed
  GitHub Pages demo matches the Rust core.
- Documented the new requirements (R213 / R214 / R215) in
  `REQUIREMENTS.md`, extended the **Translation Between Languages**
  section in `ARCHITECTURE.md`, and added a short note on
  source-formatting preservation to `VISION.md`.

## Verification Plan

- `cargo test russian_translate_how_are_you_prompt_returns_english_surface`
- `cargo test translation_via_links`
- `cargo test natural_translation_preserves_source_formatting`
- `cargo test translation_meaning_registry_covers_extended_phrases`
- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features`
- `rust-script scripts/check-file-size.rs`
- `cargo test`
- `node --check src/web/formal_ai_worker.js`
- `npm run --prefix tests/e2e check:intent-coverage`
- Browser screenshot of the demo after typing
  `Переведи "как у тебя дела?" на английский.` (captured for the PR
  description).

## Future Work

The offline meaning registry is the deterministic core. The next
iteration in `raw-data/online-research.md` documents the path to wire
Wikipedia / Wikidata / Wiktionary into the same `formalize →
deformalize` pipeline so unseen surfaces still translate accurately.
The work is intentionally scoped here: the registry already unblocks
the cases the issue calls out, the online enrichment can land in a
follow-up PR without changing the public contract.
