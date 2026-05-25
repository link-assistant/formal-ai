# Issue #242 Case Study: Unknown Prompt for `what i digress mean?`

## Scope

Issue #242 reports that the browser demo answered `what i digress mean?` with
the unknown-prompt fallback. The expected behavior is a best-effort dictionary
answer based on general parsing rules and existing online knowledge sources,
not a memoized response for one exact string.

## Captured Artifacts

- `raw-data/issue-242.json` and `raw-data/issue-242-comments.json`: issue body
  and comments.
- `raw-data/pr-243*.json`: prepared PR metadata, comments, review comments,
  and reviews.
- `raw-data/ci-runs-branch.json`: recent branch CI state before the fix.
- `raw-data/rust-parser-before.log`: failing parser regression for the exact
  prompt.
- `raw-data/e2e-issue242-before.log`: failing browser regression showing the
  unknown answer.
- `raw-data/*after.log`: focused verification after the fix.
- `raw-data/online-research.md`: dictionary and source research notes.
- `raw-data/related-prs-*.json` and `raw-data/github-code-search-*.txt`:
  related PR/code search context.

## Timeline

- The issue was opened on 2026-05-25 with Safari/GitHub Pages/browser worker
  context and a manual-mode dialog where the prompt routed to `unknown`.
- The prepared PR was still draft and had no review/conversation comments.
- The latest branch CI run before changes was successful for SHA
  `b3457cd2b1d65a3b0be2349bd182b92da884627b`.
- Local reproduction added `concepts::tests::issue_242_definition_prompt_typo_extracts_dictionary_term`
  and `tests/e2e/tests/issue-242.spec.js`; both failed before the parser fix.

## Root Cause

The solver already had enough online fallback capability once it extracted a
term: the browser worker tries local concepts, Wikipedia, Wikidata, then
Wiktionary. The missing step was extracting `digress` from the malformed
English meaning question.

The existing parser recognized prefix forms such as `what does X mean` and
`what is X`, then stripped trailing `mean`. It did not recognize typo-like
meaning forms such as `what i X mean`, so the prompt never reached Wikipedia,
Wikidata, or Wiktionary.

## Solution

- Added a general English meaning-question parser shape in Rust and the browser
  worker mirror:
  - `what i X mean`
  - `what do X mean`
  - `what does X mean`
  - `what is X meaning`
  - `what is the meaning of X`
- Kept the rule term-based rather than memorizing `digress`.
- Added browser coverage that forces Wikipedia and Wikidata misses and verifies
  the extracted term reaches Wiktionary.
- Added dictionary page sources to:
  - `data/source-index.lino`
  - `src/web_search_core.rs`
  - `src/web/tests/connectivity.js`
  - `tests/e2e/tests/connectivity.spec.js`

## Source Strategy

Wikidata and Wiktionary remain the direct live fallback path because MediaWiki
APIs are CORS-readable with `origin=*`. Cambridge Dictionary, Merriam-Webster,
Dictionary.com, and Collins are listed as page/proxy connectivity targets, not
default browser fusion providers, because dictionary pages commonly block
direct browser/API assumptions.

## Verification

Before:

- `cargo test issue_242_definition_prompt_typo_extracts_dictionary_term -- --nocapture`
  failed because `extract_concept_query("what i digress mean?")` returned
  `None`.
- `npx playwright test tests/issue-242.spec.js --config playwright.local.config.js`
  failed with the unknown-prompt answer.

After:

- `cargo test issue_242_definition_prompt_typo_extracts_dictionary_term -- --nocapture`
  passed.
- `cargo test meaning_question_variants_extract_dictionary_terms -- --nocapture`
  passed.
- `cargo test dictionary_sources_are_non_cors_knowledge_providers -- --nocapture`
  passed.
- `npx playwright test tests/issue-242.spec.js --config playwright.local.config.js`
  passed.
- `npx playwright test tests/connectivity.spec.js --config playwright.local.config.js --grep "dictionary page sources"`
  passed.
- `src/web/wasm-worker/build.sh` passed after installing the local
  `wasm32-unknown-unknown` Rust target.
- `cargo fmt --check` passed.
- `cargo clippy --all-targets --all-features -- -D warnings` passed.
- `rust-script scripts/check-file-size.rs` passed with pre-existing warning
  annotations only.
- `cargo test` passed: 484 passed, 69 ignored.
- `npm run test:local` from `tests/e2e` passed: 193 passed.
