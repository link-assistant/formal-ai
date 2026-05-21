# Issue 210 Case Study

## Scope

Issue: <https://github.com/link-assistant/formal-ai/issues/210>

Pull request: <https://github.com/link-assistant/formal-ai/pull/211>

Branch: `issue-210-4d285057dbda`

The report came from the GitHub Pages demo, formal-ai v0.89.0, using
the browser worker in manual mode. The user asked for Russian
translation prompts to produce English translations:

- `Переведи "как у тебя дела?" на английский.` already returned
  `"how are you?"`.
- `Переведи "кто ты такой" на английский.` returned assistant
  identity text in the reported browser session, and returned
  `"[en] кто ты такой"` in the local Rust reproduction.
- `Переведи "что это такое?" на английский.` returned assistant
  capability text.
- `Переведи "доброе яблоко" на английский.` returned
  `"[en] доброе яблоко"`.

Issue 210 also asked for a case-study folder containing local
artifacts, online research, root-cause analysis, and verification data.

## Artifacts

Downloaded and generated artifacts live under `raw-data/`:

- `issue-210.json`, `issue-210-comments.json`: issue payload and
  comments at collection time.
- `pr-211.json`, `pr-211-conversation-comments.json`,
  `pr-211-review-comments.json`, `pr-211-reviews.json`: PR metadata
  and comment snapshots.
- `ci-runs-branch.json`: recent branch CI run list. The branch had one
  completed successful CI run for the initial prepared commit.
- `ci-CD-Pipeline-26243373855.log`: downloaded log for that CI run.
- `repro-before-*.txt`: local CLI reproductions before the fix.
- `repro-after-*.txt`: local CLI reproductions after the fix.
- `rust-regression-before.log`: failing regression-test run captured
  before implementation.
- `rust-issue210-after.log`, `rust-pipeline-fallback-after.log`:
  targeted passing Rust test logs after implementation.
- `e2e-issue210-after.log`: targeted passing browser-worker
  Playwright test log after implementation.
- `cargo-fmt-check.log`, `clippy.log`, `cargo-test.log`,
  `check-file-size.log`, `git-diff-check.log`: local verification
  logs.
- `online-research.md`: external references used for the analysis.

## Root Causes

1. **Broad handlers won precedence over explicit translation prompts.**
   In Rust, `SPECIALIZED_HANDLERS` placed `try_translation` after
   capabilities, concept, meta, and network handlers. In the browser
   worker, `tryCapabilities` and `isIdentityPrompt` ran before
   `tryTranslation`. Because `что это такое` and `кто ты такой` are
   valid standalone capability/identity phrases, a quoted translation
   request could be consumed by the wrong handler.
2. **The canonical pipeline had no compositional fallback.**
   `TranslationPipeline::translate` correctly tries Wiktionary
   translation tables, target-edition reverse lookup, phrasal variants,
   round-trip sense selection, and Wikidata sense upgrades. When no
   complete phrase candidate existed for `доброе яблоко`, callers saw
   an empty candidate list and formatted the placeholder
   `"[en] доброе яблоко"`.
3. **The browser worker mirrored the same coverage gap offline.**
   Its small offline registry covered demo phrases such as
   `как у тебя дела`, but not the issue phrases or the component words
   needed to compose `доброе яблоко`.

## Fix

- Moved Rust translation dispatch before broad capability/concept/meta
  handlers in `src/solver.rs`.
- Added a late Rust fallback in `src/translation/pipeline.rs` that
  only runs after Wiktionary/Wikidata candidates are empty. It covers
  short Russian-to-English phrases with traceable provenance:
  `compositional:ru->en:<surface>`.
- Added regression coverage in
  `tests/unit/specification/translation_via_links.rs` for the three
  issue prompts.
- Added browser-worker precedence in `src/web/formal_ai_worker.js` so
  explicit translation requests run before capabilities and identity.
- Added browser offline phrase/word fallbacks for the reported prompts.
- Added `tests/e2e/tests/issue-210.spec.js` and included it in the
  local Playwright config.
- Added a changelog fragment:
  `changelog.d/20260521_180300_fix_issue_210_translation.md`.

## Before / After

| Prompt | Before | After |
| --- | --- | --- |
| `Переведи "как у тебя дела?" на английский.` | `"how are you?"` | `"how are you?"` |
| `Переведи "кто ты такой" на английский.` | `"[en] кто ты такой"` | `"who are you"` |
| `Переведи "что это такое?" на английский.` | Russian capabilities response | `"what is this?"` |
| `Переведи "доброе яблоко" на английский.` | `"[en] доброе яблоко"` | `"good apple"` |

## Verification

Targeted tests captured during development:

- Before fix:
  `cargo test issue_210_russian_translation_prompts_keep_translation_intent -- --nocapture`
  failed because `кто ты такой` returned `"[en] кто ты такой"`.
- After fix:
  `cargo test issue_210_russian_translation_prompts_keep_translation_intent -- --nocapture`
  passed.
- After fix:
  `cargo test translate_uses_compositional_ru_en_fallback_for_short_phrases -- --nocapture`
  passed.
- After fix:
  `npx playwright test --config=playwright.local.config.js issue-210.spec.js`
  passed from `tests/e2e`.
- Full local checks:
  `cargo fmt -- --check`, `cargo clippy --all-targets --all-features -- -D warnings`,
  `rust-script scripts/check-file-size.rs`, `cargo test`, and
  `git diff --check` passed.

Full verification commands are recorded in the pull request and in the
final solver notes.
