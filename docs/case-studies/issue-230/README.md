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
- `repro-after-cli.txt`: local CLI output after implementation.
- `rust-translation-suite-after.log`: focused Rust translation suite
  after implementation.
- `e2e-issue230-after.log`: focused browser/Playwright regression after
  implementation.
- `bun-install.log`, `npm-ci.log`, `web-build.log`: local web
  dependency and bundle verification logs.
- `cargo-fmt-check.log`, `clippy.log`, `check-file-size.log`,
  `i18n-catalog-check.log`, `language-parity-check.log`,
  `intent-coverage-check.log`, `web-bundle-diff.log`,
  `changelog-check.log`, `version-check.log`, `cargo-test.log`,
  `cargo-doc-test.log`: local CI-style verification logs.
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
| 2026-05-22 | Focused Rust and Playwright regressions passed. |

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

## Solution Options

| Option | Tradeoff | Decision |
| --- | --- | --- |
| Add an exact phrase entry for `Найти синонимы или примеры согласования`. | Small but too narrow; it would fail on nearby search/action phrases. | Rejected. |
| Expand the existing compositional fallback with Russian search/action terms and a genitive relation renderer. | Still deterministic and small, covers the report plus nearby phrases like `найти примеры согласования`. | Implemented. |
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
- `tests/unit/specification/translation_via_links.rs`
  - Adds the reported prompt as a Rust regression.
- `tests/e2e/tests/issue-230.spec.js`
  - Adds a manual-mode browser regression for the reported prompt.

## Before / After

| Prompt | Before | After |
| --- | --- | --- |
| `Переведи "Найти синонимы или примеры согласования" на ангилйский` | `"[En] Найти синонимы или примеры согласования"` | `"Find synonyms or examples of agreement"` |

## Verification

- Before fix:
  `cargo test issue_230_russian_compositional_translation_handles_search_phrase -- --nocapture`
  failed because the prompt returned the `[En]` placeholder.
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
  `npm run --prefix tests/e2e check:intent-coverage`,
  `rust-script scripts/check-changelog-fragment.rs`,
  `rust-script scripts/check-version-modification.rs`,
  `cargo test --all-features --verbose`, and
  `cargo test --doc --verbose` passed.

Full local verification logs are stored in `raw-data/`.
