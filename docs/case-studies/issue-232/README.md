# Issue 232 Case Study

## Scope

Issue: <https://github.com/link-assistant/formal-ai/issues/232>

Pull request: <https://github.com/link-assistant/formal-ai/pull/233>

Branch: `issue-232-b9341e555308`

The GitHub Pages browser demo, formal-ai v0.103.0, answered the Russian
prompt:

```text
Что такое существо?
```

with the Wikidata alias result for `Animalia` (`Q729`). The report asked
for the Russian Wikipedia page <https://ru.wikipedia.org/wiki/Существо>
to be used instead. Since that page is a disambiguation page, the answer
should list the possible meanings/definitions from the page rather than
falling through to a single Wikidata alias.

## Artifacts

Downloaded and generated artifacts live under `raw-data/`:

- `issue-232.json`, `issue-232-comments.json`: issue payload and
  comments at collection time.
- `pr-233.json`, `pr-233-conversation-comments.json`,
  `pr-233-review-comments.json`, `pr-233-reviews.json`: PR metadata
  and comment snapshots.
- `ci-runs-branch-before.json`: recent branch CI run list collected at
  the start of the investigation.
- `ci-runs-branch-after-language-matrix.json`: branch CI run list
  refreshed before the language-matrix update.
- `github-code-search-fetchWikipediaSummary.json`,
  `recent-merged-prs-wikipedia-disambiguation.json`: related GitHub
  search/PR research.
- `ruwiki-summary-suschestvo.json`, `ruwiki-parse-suschestvo.json`,
  `ruwiki-search-suschestvo.json`: live Russian Wikipedia API responses
  for `Существо`.
- `wikidata-search-suschestvo.json`: live Wikidata search response that
  ranked `Animalia` (`Q729`) first for the Russian alias.
- `repro-before-cli.txt`: local CLI reproduction before the fix. The CLI
  path returned the offline unknown fallback, confirming the reported
  `Animalia` answer was specific to the browser worker's online lookup
  path.
- `npm-ci-e2e.log`: e2e dependency install log.
- `npm-ci-e2e-language-matrix.log`: e2e dependency install log for the
  follow-up language-matrix verification.
- `e2e-issue232-before.log`: failing browser regression before the fix.
- `e2e-issue232-after.log`: passing focused browser regression after the
  fix.
- `e2e-issue232-language-matrix.log`: passing English/Russian/Hindi/Chinese
  browser regression after the CI guard update.
- `e2e-tesla-after-language-matrix.log`: passing Tesla fallback regression
  after the CI guard update.
- `e2e-wikipedia-regression-after.log`: passing focused regression that
  checks both Issue #232 and the earlier Issue #70 Tesla disambiguation
  fallback.
- `e2e-multilingual-after.log`: full multilingual browser spec after the
  fix.
- `cargo-fmt-check.log`, `clippy.log`, `check-file-size.log`,
  `cargo-test.log`, `cargo-doc-test.log`, `git-diff-check.log`: Rust
  and repository quality checks.
- `changelog-check.log`, `version-check.log`: PR-diff release guard
  checks.
- `i18n-catalog-check.log`, `language-parity-check.log`,
  `intent-coverage-check.log`: e2e catalog and multilingual coverage
  guards.
- Additional `*-language-matrix.log` files capture the follow-up local
  verification after the CI language-matrix rule was added.
- `intent-coverage-before-language-matrix.log`,
  `intent-coverage-after-language-matrix.log`: proof that the old
  multilingual guard did not cover the Issue #232 language matrix, followed
  by the updated guard passing with that matrix present.
- `online-research.md`: external API/source notes used for the analysis.

## Timeline

| Time (UTC) | Event |
| --- | --- |
| 2026-05-22 19:33 | Issue #232 was opened with the reported Russian prompt, actual `Animalia` answer, expected Russian Wikipedia page, and case-study requirements. |
| 2026-05-22 19:35 | Draft PR #233 was prepared for branch `issue-232-b9341e555308`. |
| 2026-05-22 | Local CLI reproduction returned the offline unknown fallback, so the investigation moved to the browser worker. |
| 2026-05-22 | Live Russian Wikipedia summary and parse API responses showed `Существо` is a disambiguation page with definition-style entries. |
| 2026-05-22 | Live Wikidata `wbsearchentities` response showed `Animalia` (`Q729`) ranked first because `существо` is an alias. |
| 2026-05-22 | A failing Playwright regression reproduced the browser worker choosing Wikidata over the Russian Wikipedia disambiguation page. |
| 2026-05-22 | The worker was updated to accept direct definition-style Wikipedia disambiguation pages and render their entries before trying Wikidata/Wiktionary fallbacks. |
| 2026-05-22 20:50 | PR feedback requested CI/CD rules requiring language coverage for English, Russian, Hindi, and Chinese rather than only the originally reported Russian prompt. |
| 2026-05-22 | The Issue #232 Playwright regression was expanded into a supported-language matrix, and `check:intent-coverage` was extended to require that matrix in CI. |

## Requirements And Status

| ID | Requirement | Status |
| --- | --- | --- |
| R1 | `Что такое существо?` must not answer with `Animalia`/Wikidata `Q729`. | Implemented. The browser worker now returns Russian Wikipedia entries for `Существо`. |
| R2 | Use <https://ru.wikipedia.org/wiki/Существо> when the exact Russian Wikipedia page exists. | Implemented for direct, context-free concept lookups where the page is a definition-style disambiguation page. |
| R3 | Because the page is disambiguation, list possible meanings/definitions. | Implemented by parsing page entries from the MediaWiki parse API and falling back to summary extract lines. |
| R4 | Preserve the existing Issue #70 behavior where generic disambiguation pages such as `Tesla` fall back to full-text search. | Verified. The targeted Tesla regression still passes. |
| R5 | Preserve issue/PR/log data under `docs/case-studies/issue-232`. | Implemented in `raw-data/`. |
| R6 | Search online for additional facts and data. | Implemented in `raw-data/online-research.md` and the saved live API payloads. |
| R7 | Reconstruct timeline, requirements, root causes, and solution options. | Implemented in this case study. |
| R8 | Add debug output if the root cause cannot be found. | Not needed; the failure was reproduced and isolated. |
| R9 | Require tests for English, Russian, Hindi, and Chinese in CI/CD for this class of language-sensitive regression. | Implemented in `tests/e2e/scripts/check-multilingual-intent-coverage.mjs`; the guard now fails if the Issue #232 definition-style disambiguation matrix loses any supported language. |

## Root Cause

The browser worker's `fetchWikipediaSummary()` treated every direct
Wikipedia summary response with `type: "disambiguation"` as unusable.
That behavior was correct for broad English ambiguous prompts such as
`what is tesla`, where the disambiguation page only says the term may
refer to many unrelated pages and the user usually expects the top
article from search.

For Russian `Существо`, the exact Wikipedia page is different: it is a
definition-style disambiguation page whose entries are themselves useful
definitions:

- `Существо — живой организм, живая особь, животное, человек.`
- `Существо — главное, существенное в ком-либо, чем-либо, его суть; сущность.`

Because the worker skipped the page before inspecting those entries, it
continued through the fallback pipeline. The subsequent Wikidata alias
lookup accepted the first exact alias match, and Wikidata ranked
`Animalia` (`Q729`) first for `существо`. That produced the reported
incorrect answer.

## Solution Options

| Option | Tradeoff | Decision |
| --- | --- | --- |
| Add a hard-coded `существо` concept record. | Fast but narrow, duplicates external knowledge, and would not help the same failure mode for other definition-style disambiguation pages. | Rejected. |
| Always render Wikipedia disambiguation pages. | Would regress Issue #70 and answer broad ambiguous prompts with low-signal lists. | Rejected. |
| Accept only definition-style direct disambiguation pages for exact, context-free concept lookups. | Small, general, and preserves search fallback for generic disambiguation pages. | Implemented. |

## Implemented Fix

- Added Russian/English/Hindi/Chinese MediaWiki action API hosts so the
  worker can fetch parsed page HTML for accepted disambiguation pages.
- Added deterministic HTML-to-text extraction for disambiguation list
  entries, scoped before `См. также`/`See also`/references sections.
- Added a conservative definition-style gate: at least one entry must
  begin with the requested term or page title followed by a spaced dash.
- Enabled that gate only for direct, context-free concept lookups, so
  context search and generic disambiguation search behavior stay intact.
- Rendered accepted disambiguation pages as sourced Wikipedia lookup
  answers with the collected entries.
- Added a Playwright regression matrix that mocks exact English, Russian,
  Hindi, and Chinese Wikipedia definition-style disambiguation pages and
  their competing Wikidata `Animalia` aliases.
- Extended the CI multilingual intent coverage guard so this regression
  remains covered for every supported language.

## Before / After

| Prompt | Before | After |
| --- | --- | --- |
| `Что такое существо?` | `существо: Animalia: kingdom of multicellular eukaryotic organisms` from Wikidata `Q729` in the browser worker. | A Wikipedia-sourced list of meanings from `https://ru.wikipedia.org/wiki/Существо`, including the living-organism sense, essence/sustnost sense, and culture entries from the page. |
| `what is tesla` | Search fallback to `Tesla, Inc.` after skipping the generic disambiguation page. | Same behavior; the Issue #70 regression still passes. |

## Verification

- Before fix:
  `npx playwright test --config=playwright.local.config.js multilingual.spec.js --grep "Issue #232"`
  failed because the browser worker answered with Wikidata `Q729`.
- After fix:
  `npx playwright test --config=playwright.local.config.js multilingual.spec.js --grep "Issue #232"`
  passed with English, Russian, Hindi, and Chinese cases.
- After CI guard update:
  `npm run --prefix tests/e2e check:intent-coverage`
  passed and now asserts the Issue #232 language matrix.
- After CI guard update:
  `npx playwright test --config=playwright.local.config.js multilingual.spec.js --grep "what is tesla"`
  passed, preserving the earlier generic disambiguation fallback behavior.
- After fix:
  `npx playwright test --config=playwright.local.config.js multilingual.spec.js --grep "what is tesla|Issue #232"`
  passed.
- After initial fix:
  `npx playwright test --config=playwright.local.config.js multilingual.spec.js`
  passed with 96 tests.
- After CI guard update:
  `npx playwright test --config=playwright.local.config.js multilingual.spec.js`
  passed with 99 tests.
- After fix:
  `cargo fmt --check`,
  `cargo clippy --all-targets --all-features`,
  `rust-script scripts/check-file-size.rs`,
  `cargo test --verbose`, and
  `cargo test --doc --verbose` passed.
- After fix:
  `npm run --prefix tests/e2e check:i18n`,
  `npm run --prefix tests/e2e check:language-parity`, and
  `npm run --prefix tests/e2e check:intent-coverage` passed.
- After commit:
  `rust-script scripts/check-changelog-fragment.rs` and
  `rust-script scripts/check-version-modification.rs` passed with PR-like
  environment variables.

Full local verification logs are stored in `raw-data/`.
