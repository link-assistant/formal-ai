# Issue 230 Case Study

## Scope

Issue: <https://github.com/link-assistant/formal-ai/issues/230>

Pull request: <https://github.com/link-assistant/formal-ai/pull/231>

Branch: `issue-230-a9436b51254d`

The GitHub Pages browser demo, formal-ai v0.102.0 in manual wasm mode,
reported this dialog:

```text
U: Переведи "Найти синонимы или примеры согласования" на ангилйский
A (intent: translate_ru_to_en, reported): "[En] Найти синонимы или примеры согласования"
```

The issue asks for the universal formalize -> meaning -> deformalize
translation vision to keep improving, plus a local case-study folder
with downloaded issue/PR data, online research, root-cause analysis,
requirements, and solution planning.

## Artifacts

Downloaded and generated artifacts live under `raw-data/`:

- `issue-230.json`, `issue-230-comments.json`: issue payload and
  comments at collection time.
- `pr-231.json`, `pr-231-conversation-comments.json`,
  `pr-231-review-comments.json`, `pr-231-reviews.json`: PR metadata
  and comment snapshots.
- `repo.json`, `issues-recent.json`, `pulls-recent.json`,
  `actions-runs-recent.json`, `manifest.json`: repository context from
  the built-in GitHub log collector.
- `repro-before-cli.txt`: local CLI reproduction before the fix.
- `rust-regression-before.log`: focused regression failure before the
  implementation.
- `rust-translation-gap-before.log`: focused regression failure showing
  an unknown translation still rendered `[ru] ...` before the
  architecture fix.
- `rust-translation-gap-after.log`: focused regression after the
  architecture fix, proving the same unknown surface is reported as a
  traceable translation gap instead of a placeholder.
- `repro-after-cli.txt`: local CLI output after implementation.
- `rust-translation-suite-after.log`: focused Rust translation suite
  after implementation.
- `e2e-issue230-after.log`: focused browser/Playwright regression after
  implementation.
- `bun-install.log`, `npm-ci.log`, `web-build.log`: local web
  dependency and bundle verification logs.
- `cargo-fmt-check.log`, `clippy.log`, `check-file-size.log`,
  `i18n-catalog-check.log`, `language-parity-check.log`,
  `language-test-coverage-check.log`, `intent-coverage-check.log`,
  `web-bundle-diff.log`, `changelog-check.log`, `version-check.log`,
  `cargo-test.log`, `cargo-doc-test.log`: local CI-style verification
  logs.
- `online-research.md`: external references used for the analysis.

## Timeline

| Time (UTC) | Event |
| --- | --- |
| 2026-05-22 18:06 | Browser report captured formal-ai v0.102.0 returning the `[En]` placeholder for the quoted phrase. |
| 2026-05-22 18:18 | Issue #230 was opened with the dialog, environment metadata, and case-study requirements. |
| 2026-05-22 | Local CLI reproduction confirmed the same placeholder response. |
| 2026-05-22 | A failing Rust regression was added for the reported phrase. |
| 2026-05-22 | Online research confirmed `согласования` as the genitive form of `согласование` and the grammar sense as `agreement` / `concordance`. |
| 2026-05-22 | The shared Rust pipeline and browser worker compositional fallback were extended to render Russian search/action phrases with a genitive `of` relation. |
| 2026-05-22 | PR feedback requested an architectural guarantee that placeholders never surface for translation misses. |
| 2026-05-22 | A failing regression showed `Translate "zzqxqv" to Russian` still rendered `"[ru] zzqxqv"` when the pipeline had no candidates. |
| 2026-05-22 | Rust and browser projection code was changed to treat zero candidates as explicit `translation_gap` evidence and user-facing gap text. |
| 2026-05-22 | Focused Rust and Playwright regressions passed. |
| 2026-05-22 20:52 | PR feedback requested CI/CD rules requiring tests for every supported language, not only Russian. |
| 2026-05-22 | A diff-aware CI guard was added to require changed tests covering English, Russian, Hindi, and Chinese when a PR changes language-facing code. |

## Requirements And Status

| ID | Requirement | Status |
| --- | --- | --- |
| R1 | The reported prompt must translate to English instead of returning `[En] ...`. | Implemented. |
| R2 | The handler must remain `translate_ru_to_en`. | Implemented and covered by the Rust regression. |
| R3 | Preserve source capitalization and punctuation rules. | Preserved by the existing `match_source_formatting` stage. |
| R4 | The fix must apply to the browser worker, not only the Rust CLI. | Implemented and covered by `issue-230.spec.js`. |
| R5 | Use a general formalization/deformalization direction, not a new fake placeholder. | Implemented as a compositional fallback after Wiktionary/Wikidata lookup misses. |
| R6 | Compile issue/PR/log data under `docs/case-studies/issue-230`. | Implemented in `raw-data/`. |
| R7 | Search online for additional facts and data. | Implemented in `raw-data/online-research.md`. |
| R8 | Reconstruct timeline, requirements, root causes, and solution options. | Implemented in this case study. |
| R9 | Add debug output if the root cause cannot be found. | Not needed; the root cause was reproduced and isolated. Existing `FORMAL_AI_TRANSLATION_DEBUG=1` tracing remains available. |
| R10 | Report upstream issues if needed. | Not needed; no upstream Wiktionary/Wikidata defect was found. |
| R11 | No translation miss may render a bracketed language placeholder such as `[en] ...` or `[ru] ...`. | Implemented with active Rust and Playwright regressions. |
| R12 | CI/CD must require tests for every supported language: English, Russian, Hindi, and Chinese. | Implemented with `check:language-test-coverage`, stricter language-resource parity, and expanded gap regressions. |

## Root Cause

The translation request was recognized correctly. The failure happened
after intent routing:

1. `try_translation` extracted the quoted Russian source surface and
   selected `ru -> en`.
2. `TranslationPipeline::translate` normalized the whole phrase as a
   Wiktionary page title.
3. No source-edition, target-edition, reverse, or phrasal-variant
   lookup produced translation candidates for the full five-word
   phrase.
4. The final Russian -> English compositional fallback only covered a
   few phrases and two-to-four-word noun phrases. It did not know
   `найти`, `синонимы`, `или`, `примеры`, or `согласования`, and it
   rejected five-token input before trying a word-level rendering.
5. With zero candidates, the handler rendered the existing placeholder
   format: `[en] <source>`, then `match_source_formatting` capitalized
   the target marker to `[En]` because the source starts with `Н`.

The browser worker mirrored the same limitation in
`translateCompositionalSurface`, so fixing only the Rust pipeline would
not have addressed the reported wasm/manual surface.

The PR feedback exposed a broader architectural fault: zero-candidate
translation results and pipeline errors were treated as strings, so the
rendering layer could not distinguish "real target surface" from "we
could not translate this." That made placeholders possible for every
unknown language pair, even after the reported phrase was fixed.

## Solution Options

| Option | Tradeoff | Decision |
| --- | --- | --- |
| Add an exact phrase entry for `Найти синонимы или примеры согласования`. | Small but too narrow; it would fail on nearby search/action phrases. | Rejected. |
| Expand the existing compositional fallback with Russian search/action terms and a genitive relation renderer. | Still deterministic and small, covers the report plus nearby phrases like `найти примеры согласования`. | Implemented. |
| Treat zero-candidate pipeline output as an explicit translation-gap state instead of a target string. | Gives every caller a single invariant: only candidate surfaces can be quoted as translations. Requires user-facing gap copy and evidence links. | Implemented. |
| Add a full Russian morphology parser. | More principled long term, but much larger than needed for this regression. | Deferred. |

## Implemented Fix

- `src/translation/pipeline.rs`
  - Allows Russian -> English compositional fallback on two-to-eight
    token phrases.
  - Adds entries for `найти`, `синонимы`, `или`, `примеры`, and
    `согласование` inflections.
  - Renders known genitive noun relations such as `примеры
    согласования` as `examples of agreement`.
- `src/web/formal_ai_worker.js`
  - Mirrors the same fallback vocabulary and genitive relation renderer
    for the browser demo.
  - Returns structured translation results so the browser distinguishes
    target surfaces from translation gaps.
- `src/solver_handlers/mod.rs`
  - Stops manufacturing `[target] <source>` strings for zero-candidate
    translations and pipeline errors.
  - Renders explicit gap copy and records `translation_gap:<surface>` in
    evidence links.
- `tests/unit/specification/translation_via_links.rs`
  - Adds the reported prompt as a Rust regression.
  - Adds supported-target-language regressions proving gaps do not
    render `[en]`, `[ru]`, `[hi]`, or `[zh]` placeholders.
- `tests/e2e/tests/issue-230.spec.js`
  - Adds a manual-mode browser regression for the reported prompt.
  - Adds browser regressions for unknown translation gaps across every
    supported target language.
- `tests/e2e/scripts/check-language-test-coverage.mjs`
  - Fails PRs that change language-facing code without adding or
    updating tests for all supported languages.
- `tests/e2e/scripts/check-language-change-parity.mjs`
  - Now requires every supported language to change together in watched
    language resources, rather than only requiring Hindi and Chinese
    companion updates.

## Before / After

| Prompt | Before | After |
| --- | --- | --- |
| `Переведи "Найти синонимы или примеры согласования" на ангилйский` | `"[En] Найти синонимы или примеры согласования"` | `"Find synonyms or examples of agreement"` |
| `Translate "zzqxqv" to Russian` | `"[ru] zzqxqv"` | `I could not translate "zzqxqv" from en to ru with the available formalization data. I recorded this as a translation gap for follow-up.` |

## Verification

- Before fix:
  `cargo test issue_230_russian_compositional_translation_handles_search_phrase -- --nocapture`
  failed because the prompt returned the `[En]` placeholder.
- Before architecture fix:
  `cargo test --test unit specification::translation_via_links::translation_gaps_are_reported_without_language_placeholders -- --nocapture`
  failed because an unknown English -> Russian surface returned the
  `[ru]` placeholder.
- After architecture fix:
  `cargo test --test unit specification::translation_via_links::translation_gaps_are_reported_without_language_placeholders -- --nocapture`
  passed for English, Russian, Hindi, and Chinese target-language gaps.
- After fix:
  `cargo test --test unit specification::translation_via_links:: -- --nocapture`
  passed.
- After fix:
  `cargo run -- chat --prompt 'Переведи "Найти синонимы или примеры согласования" на ангилйский'`
  returned `"Find synonyms or examples of agreement"`.
- After fix:
  `npm --prefix tests/e2e run test:local -- issue-230.spec.js`
  passed.
- CI-style checks:
  `cargo fmt --all -- --check`,
  `cargo clippy --all-targets --all-features`,
  `rust-script scripts/check-file-size.rs`,
  `bun run build:web`,
  `git diff --exit-code -- src/web/vendor.bundle.js src/web/ocr.bundle.js`,
  `npm run --prefix tests/e2e check:i18n`,
  `npm run --prefix tests/e2e check:language-parity`,
  `npm run --prefix tests/e2e check:language-test-coverage`,
  `npm run --prefix tests/e2e check:intent-coverage`,
  `rust-script scripts/check-changelog-fragment.rs`,
  `rust-script scripts/check-version-modification.rs`,
  `cargo test --all-features --verbose`, and
  `cargo test --doc --verbose` passed.

Full local verification logs are stored in `raw-data/`.
