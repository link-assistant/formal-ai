# Issue 526 Case Study

Issue [#526](https://github.com/link-assistant/formal-ai/issues/526) raises the
translation quality bar from "can produce a target surface" to "survives
round-trip translation." A translation is acceptable only when the source can be
formalized into the meta language, rendered to a target language, and rendered
back without losing the meaning or the observable source surface.

## 1. Collected Data

- Issue snapshot: `raw-data/issue-526.json`.
- Issue comments: `raw-data/issue-526-comments.json` (empty at collection time).
- Prepared PR snapshot: `raw-data/pr-635.json`.
- PR conversation, review-comment, and review snapshots:
  `raw-data/pr-635-comments.json`, `raw-data/pr-635-review-comments.json`, and
  `raw-data/pr-635-reviews.json` (all empty at collection time).
- Online prior art and source notes: `raw-data/online-research.md`.

No issue screenshots were present, so there were no image attachments to
download or verify.

## 2. Requirements

| ID | Requirement | Implementation |
| --- | --- | --- |
| R526-1 | Translation quality must be measured by round-trip survival, not only by a one-way surface. | `tests/unit/specification/translation_round_trip.rs` asserts meaning and surface survival. |
| R526-2 | Every supported source language must formalize to the meta language and deformalize back to itself without data loss. | `supported_language_surfaces_survive_meta_language_round_trip` covers en, ru, hi, and zh surfaces for the same seeded meaning. |
| R526-3 | Every supported natural-language pair must translate only through the shared meta-language meaning. | `every_supported_language_pair_round_trips_via_meta_language` covers the full directed en/ru/hi/zh pair matrix. |
| R526-4 | Rust and JavaScript code translation must preserve code meaning through a meta representation. | `rust_javascript_code_translation_round_trips_through_code_meaning` checks Rust -> JavaScript -> Rust meaning-link equality. |
| R526-5 | Direct translation without the meta language must stay out of the documented architecture. | `VISION.md`, `ARCHITECTURE.md`, `ROADMAP.md`, and `CONTRIBUTING.md` now describe the meta-language-only contract. |
| R526-6 | Research, requirements, and solution planning must be preserved under this case-study directory. | This README plus `requirements.md`, `solution-plans.md`, and `raw-data/online-research.md` keep the evidence in one place. |

## 3. Root Cause

The natural-language pipeline already resolved surfaces through
`TranslationPipeline`, but the tests only sampled a few English-source round
trips. That left two gaps:

1. Supported languages other than English were not required to survive
   language-to-meta-to-same-language projection.
2. The full pair matrix across en, ru, hi, and zh was not required to preserve
   the same meta-language meaning.

The code-translation path had a separate, deeper gap. `translate_program`
matched on `(source, target)` pairs directly — a Python -> Rust arm, a
Rust -> Python arm, and so on. This is the exact `N * N` direct-translation
practice issue #526 forbids ("we should never support direct translation without
meta language"): every new language pair needed its own arm, and every new
language multiplied the arms. Adding Rust <-> JavaScript to that table only made
the anti-pattern larger, and the code meaning hash was lexical rather than a
shared semantic identity.

## 4. Implemented Design

The natural-language test matrix uses the seeded apple meaning because it has
stable surfaces in all supported languages:

- English: `apple`
- Russian: `яблоко`
- Hindi: `सेब`
- Chinese: `苹果`

The tests call the real offline `translate_via_default_pipeline` path, so they
exercise `formalize -> meaning -> deformalize` rather than a direct pair table.
The pair matrix asserts both the target surface and the shared `MeaningId`.

The code translation fix removes the direct `(source, target)` table entirely
and gives code its own meta language, mirroring the natural-language pipeline.
`formalize_code_meaning` collapses a fragment into a language-neutral
`CodeMeaning` (currently `CodeMeaning::BinaryAddFunction`, slug
`function:add:binary_sum`), and `render_code_meaning` renders that meaning into
the requested target (Python, Rust, JavaScript, TypeScript, or Go). Thus
`translate_program` is just `render(formalize(code), target)`: `N` formalizers
plus `N` renderers, never `N * N` pairs. Because the source language never
enters `formalize_code_meaning`, the same add function written in any source
language collapses to one meaning, so any pair — including pairs that never had
a hardcoded arm, such as Python -> JavaScript or Rust -> Go — translates and
shares the same `meaning:` link. `normalize_code_meaning` now delegates to the
same formalizer, so the meaning a fragment translates *through* is exactly the
meaning its trace records. Unknown programs, and targets with no seeded
rendering, still return explicit, language-appropriate translation gaps.

## 5. Prior Art And Existing Components

Round-trip translation is useful because it reframes a bilingual evaluation
problem as a monolingual consistency check, but prior work also warns that it is
not a complete replacement for human or reference-based evaluation. The
implemented tests use round-trip survival as a regression invariant for
Formal AI's own meta-language pipeline, not as a universal MT score.

Existing components reused:

- `src/translation/pipeline.rs::TranslationPipeline` for natural-language
  formalize/deformalize.
- `src/translation/pipeline.rs::select_best_block` for round-trip-confirmed
  sense selection.
- `src/solver_handlers/mod.rs` for traceable code translation evidence links.
- `src/solver_helpers.rs::{CodeMeaning, formalize_code_meaning,
  render_code_meaning, normalize_code_meaning}` for the code meta language:
  `source -> CodeMeaning -> target` with a shared meaning slug.

## 6. Verification

Reproducer before the code fix:

```text
cargo test --test unit rust_javascript_code_translation_round_trips_through_code_meaning -- --nocapture
```

The test failed because Rust -> JavaScript returned:

```text
// translation gap from rust to javascript: fn add(a: i32, b: i32) -> i32 { a + b }
```

Verification after the fix:

```text
cargo test --test unit rust_javascript_code_translation_round_trips_through_code_meaning -- --nocapture
cargo test --test unit translation_round_trip -- --nocapture
```

The final PR verification also runs the docs traceability test and local quality
checks recorded in the PR body.
