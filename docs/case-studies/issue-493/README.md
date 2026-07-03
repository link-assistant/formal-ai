# Issue 493: OCR market-price fact checking

> **Status:** Implemented in PR #619.
> **Type:** Bug fix and case study for factual verification of image text.

- **Issue:** <https://github.com/link-assistant/formal-ai/issues/493>
- **Pull request:** <https://github.com/link-assistant/formal-ai/pull/619>
- **Raw data:** [`raw-data/`](raw-data/)
- **Image asset:** [`assets/issue-493-screenshot.jpg`](assets/issue-493-screenshot.jpg)

## Summary

Issue #493 asked the solver to read the attached screenshot, reason about the
text in the image, search external data, and detect false statements. The
important extracted claim is:

```text
ETH in 2024: $1,700
```

The fix adds a deterministic market-price claim layer on top of the existing
document-verification path. Multi-line OCR text is preserved, asset/year/price
claims are extracted from the sample, and each claim is assessed against captured
Binance daily kline data.

The maintainer's follow-up on PR #619 asked us to support the *entire class* of
similar questions, not just the single ETH 2024 example. The implementation is
therefore data-driven and meaning-grounded:

- **Multi-asset.** The reference registry
  [`data/seed/market-price-references.lino`](../../../data/seed/market-price-references.lino)
  covers ETH (grounded in Wikidata `Q16783523`) and BTC (`Q131723`), and grows by
  adding assets/periods to seed data — no Rust or JS change is needed to cover a
  new asset.
- **Multi-period.** ETH carries 2021–2024 references; BTC carries 2024. The
  parser accepts any four-digit year present in the registry.
- **Multilingual, meaning-grounded.** Asset aliases live in the seed data per
  language (English, Russian, Hindi, Chinese) following issue #386's convention
  that no per-language phrase list ever lives in Rust. The Chinese `以太坊` and
  Russian `эфириум` surfaces resolve to the same grounded ETH asset.
- **Within-range vs contradicted.** A claim is only contradicted when the price
  falls outside the observed daily-candle range for that asset and period. The
  false `$1,700` is *within* the recorded ETH range for 2021–2023 (so those lines
  are not over-claimed) but *below* the 2024 minimum, so only 2024 is flagged as
  contradicted. BTC `$1,700` in 2024 is caught by the same cross-asset machinery.

The procedure is recorded as a grounded meta-algorithm so the whole class stays
reproducible and the recipe cannot silently drift from the code — see
[The meta-algorithm](#the-meta-algorithm-recipe) below.

## Requirements

This case study tracks the requirements from the issue and prepared work order:

| Requirement | Evidence |
|---|---|
| Preserve all issue and PR data. | Raw GitHub JSON, comments, reviews, and the issue screenshot are saved under this directory. |
| Work with text inside an image. | The screenshot is preserved and a Tesseract.js OCR experiment is saved in [`raw-data/tesseract-issue-screenshot-ocr.json`](raw-data/tesseract-issue-screenshot-ocr.json). |
| Catch the false price statement. | The unit regression asserts that `ETH in 2024: $1,700` is contradicted. |
| Search online and compare external data. | Binance API data and CoinGecko API/page captures are preserved in [`raw-data/`](raw-data/). |
| Use the meta algorithm, recursive reasoning, and relative-meta-logic. | The handler logs per-statement verification, source-tier priors, market-source evidence, and the lowered posterior for contradicted claims. The whole procedure is recorded as a grounded recipe in [`data/meta/market-price-verification-recipe.lino`](../../../data/meta/market-price-verification-recipe.lino) with a specification test that keeps it grounded in live source. |
| Generalize to the entire class in all languages. | The verifier is driven by the data-driven registry [`data/seed/market-price-references.lino`](../../../data/seed/market-price-references.lino): multiple assets (ETH, BTC), multiple periods (2021–2024), currency-marked amounts, and per-language aliases (English, Russian, Hindi, Chinese). New assets/periods/languages are added in seed data with no Rust or JS change. Recognition is by grounded meaning (Wikidata Q-ids), following issue #386. |
| Distinguish within-range claims from contradicted ones. | Only prices outside an asset's observed daily-candle range are contradicted; ETH `$1,700` stays within range for 2021–2023 and is contradicted only for 2024, cross-checked in unit and e2e tests. |
| Propose and compare solutions. | Alternatives considered are documented below. |
| Implement and test in PR #619. | Regression tests live in [`tests/unit/issue_493.rs`](../../../tests/unit/issue_493.rs) and [`tests/e2e/tests/issue-493.spec.js`](../../../tests/e2e/tests/issue-493.spec.js). |

## Captured Data

| File | Contents |
|---|---|
| [`raw-data/issue-493.json`](raw-data/issue-493.json) | Issue title, body, metadata, and screenshot URL from `gh issue view`. |
| [`raw-data/issue-493-comments.json`](raw-data/issue-493-comments.json) | Paginated issue comments. |
| [`raw-data/pr-619.json`](raw-data/pr-619.json) | Prepared PR metadata. |
| [`raw-data/pr-619-conversation-comments.json`](raw-data/pr-619-conversation-comments.json) | PR conversation comments. |
| [`raw-data/pr-619-review-comments.json`](raw-data/pr-619-review-comments.json) | Inline PR review comments. |
| [`raw-data/pr-619-reviews.json`](raw-data/pr-619-reviews.json) | PR reviews. |
| [`assets/issue-493-screenshot.jpg`](assets/issue-493-screenshot.jpg) | Downloaded issue image; validated as JPEG before OCR. |
| [`raw-data/tesseract-issue-screenshot-ocr.json`](raw-data/tesseract-issue-screenshot-ocr.json) | OCR result from Tesseract.js 7.0.0, confidence 90, including the ETH 2024 claim. |
| [`raw-data/binance-ethusdt-2024-daily-klines.json`](raw-data/binance-ethusdt-2024-daily-klines.json) | Binance ETHUSDT 1d klines for 2024. |
| [`raw-data/binance-vision-ethusdt-2024-daily-klines.json`](raw-data/binance-vision-ethusdt-2024-daily-klines.json) | Binance Vision mirror of the same 2024 ETHUSDT kline range. |
| [`raw-data/binance-ethusdt-2024-summary.json`](raw-data/binance-ethusdt-2024-summary.json) | Parsed min/max summary used in the implementation. |
| [`raw-data/ethusdt-2021-daily-klines.json`](raw-data/ethusdt-2021-daily-klines.json) | Binance ETHUSDT 1d klines for 2021, used for the within-range reference. |
| [`raw-data/ethusdt-2022-daily-klines.json`](raw-data/ethusdt-2022-daily-klines.json) | Binance ETHUSDT 1d klines for 2022, used for the within-range reference. |
| [`raw-data/ethusdt-2023-daily-klines.json`](raw-data/ethusdt-2023-daily-klines.json) | Binance ETHUSDT 1d klines for 2023, used for the within-range reference. |
| [`raw-data/ethusdt-2024-daily-klines.json`](raw-data/ethusdt-2024-daily-klines.json) | Binance ETHUSDT 1d klines for 2024, used for the contradicted reference. |
| [`raw-data/btcusdt-2024-daily-klines.json`](raw-data/btcusdt-2024-daily-klines.json) | Binance BTCUSDT 1d klines for 2024, used for the cross-asset reference. |
| [`raw-data/market-price-references-summary.json`](raw-data/market-price-references-summary.json) | Parsed per-asset/per-period min/max ranges compiled into the seed registry. |
| [`raw-data/coingecko-ethereum-2024-market-chart-range.json`](raw-data/coingecko-ethereum-2024-market-chart-range.json) | CoinGecko public API rejection showing the historical public range limit. |
| [`raw-data/coingecko-ethereum-historical-data-page.html`](raw-data/coingecko-ethereum-historical-data-page.html) | Captured CoinGecko Ethereum historical page fallback. |
| [`raw-data/online-research.md`](raw-data/online-research.md) | Source URLs and notes for the online research step. |
| [`raw-data/bun-install.log`](raw-data/bun-install.log) | Dependency install log for the Tesseract.js OCR experiment. |
| [`raw-data/repro-before.log`](raw-data/repro-before.log) | Failing test log before the new extractor/assessor existed. |
| [`raw-data/repro-after.log`](raw-data/repro-after.log) | Passing focused regression log after implementation. |
| [`raw-data/document-originality-regression.log`](raw-data/document-originality-regression.log) | Existing document-originality regression suite after the fix. |
| [`raw-data/issue-535-regression.log`](raw-data/issue-535-regression.log) | Regression suite for the original OCR document-verification feature. |
| [`raw-data/statement-verification-source.log`](raw-data/statement-verification-source.log) | Existing statement-verification source tests. |
| [`raw-data/worker-21-syntax-check.log`](raw-data/worker-21-syntax-check.log) | Browser worker syntax check. |
| [`raw-data/playwright-issue-493.log`](raw-data/playwright-issue-493.log) | Browser regression covering the mocked OCR/image upload flow. |
| [`raw-data/playwright-issue-535-regression.log`](raw-data/playwright-issue-535-regression.log) | Browser regression for the prior OCR workflow. |
| [`raw-data/build-web.log`](raw-data/build-web.log) | Web build output. |
| [`raw-data/cargo-fmt-check.log`](raw-data/cargo-fmt-check.log) | Formatting check output. |
| [`raw-data/cargo-clippy.log`](raw-data/cargo-clippy.log) | Clippy output. |
| [`raw-data/cargo-test-all-features.log`](raw-data/cargo-test-all-features.log) | Full `cargo test --all-features --verbose` output. |
| [`raw-data/cargo-test-doc.log`](raw-data/cargo-test-doc.log) | `cargo test --doc --verbose` output. |
| [`raw-data/language-test-coverage.log`](raw-data/language-test-coverage.log) | Diff-aware language coverage guard confirming tests cover English, Russian, Hindi, and Chinese. |
| [`raw-data/manual-file-size-check.log`](raw-data/manual-file-size-check.log) | Direct line-count scan used because `rust-script` was not installed in this container; it reports warning-band files but no hard-limit violations. |

## Screenshot Transcription

Tesseract.js recovered the repeated claims with confidence 90. Relevant lines:

```text
$ETH
ETH in 2021: $1,700
ETH in 2022: $1,700
ETH in 2023: $1,700
ETH in 2024: $1,700
ETH in 2025: $1,700
ETH in 2026: $1,700
ETH before BitMine buying: $1,700
ETH after BitMine buying: $1,700
ETH before ETF approval: $1,700
ETH after ETF approval: $1,700
ETH during anti-crypto President: $1,700
ETH during pro-crypto President: $1,700
ETH before US-Iran war: $1,700
ETH after US-Iran war: $1,700

Performance of $ETH is an absolute joke.
```

## External Data Check

The registry captures the observed daily-candle range per asset and period from
first-party Binance kline data. The covered ranges are:

| Asset | Period | Min daily low | on | Max daily high | on |
|---|---|---:|---|---:|---|
| ETH | 2021 | 714.29 | 2021-01-01 | 4868.00 | 2021-11-10 |
| ETH | 2022 | 881.56 | 2022-06-18 | 3900.73 | 2022-01-04 |
| ETH | 2023 | 1190.57 | 2023-01-01 | 2445.80 | 2023-12-28 |
| ETH | 2024 | 2100.00 | 2024-01-03 | 4107.80 | 2024-12-16 |
| BTC | 2024 | 38555.00 | 2024-01-23 | 108353.00 | 2024-12-17 |

`$1,700` sits inside the ETH range for 2021, 2022, and 2023, so those claims are
assessed as **within the recorded range** rather than false. It is below the ETH
2024 minimum of `2100.00`, so `ETH in 2024: $1,700` is **contradicted**. `$1,700`
is far below the BTC 2024 minimum of `38555.00`, so `BTC in 2024: $1,700` is
contradicted too — proving the check generalises across assets.

## Root Cause

The issue-535 document verification path already planned searches for each
statement, but two gaps prevented issue #493 from being handled:

1. The handler reduced `OCR text:` and `Text excerpt:` blocks to a short sample,
   so repeated multi-line claims after the first line could be missed.
2. The per-statement verifier produced a search-and-weigh plan but had no
   structured numeric market-price assessment capable of lowering a claim after
   comparing it with captured original market data.

## Fix

The implementation changes are intentionally small and reusable:

- `document_originality` now preserves full multi-line `OCR text:`,
  `Text excerpt:`, and `Text sample:` blocks for statement verification.
- `statement_verification` now extracts market-price claims from OCR/text
  fragments by matching asset aliases, a four-digit year, and a currency-marked
  amount. Aliases and ranges come from the seed registry, so the extractor is
  data-driven and covers every asset/period/language in the registry.
- A new `src/seed/market_price_references.rs` loads the registry from
  `data/seed/market-price-references.lino` once, grounding each asset in a
  Wikidata entity and carrying one reference (observed daily-candle min/max) per
  covered period.
- Extracted claims are checked against these versioned references. A price inside
  the observed range is recorded as `within_recorded_range`; a price outside it is
  `contradicted`.
- Contradicting market data is represented as original-first relative evidence,
  lowering the statement posterior and preserving the full trace in
  `market_price_claim:*` event-log links.
- User-facing verification answers add a concise `Price claim check` section
  when any extracted market claim is contradicted.
- The browser worker mirrors the Rust path so uploaded image/OCR checks surface
  the same contradiction in the web UI.

## The meta-algorithm recipe

Issue #493 explicitly asks the solver to use the *meta-algorithm* and recursive
reasoning. To keep the market-price fact check reproducible for the whole class,
the procedure that produced it is recorded as a grounded recipe, modelled on the
existing document-verification meta-algorithm:

- [`data/meta/market-price-verification-recipe.lino`](../../../data/meta/market-price-verification-recipe.lino)
  names every ordered step (register reference data → ground each asset →
  recognise assets by alias → split per asset → parse period and price → weigh
  against the recorded range → splice the verdict → mirror the worker), each
  external Wikidata grounding, the data source, the handler functions, the
  Rust↔JS parity targets, and the tests that pin the behaviour.
- [`tests/unit/specification/market_price_verification_meta_algorithm.rs`](../../../tests/unit/specification/market_price_verification_meta_algorithm.rs)
  loads the recipe and asserts the live source still matches it: the steps are
  contiguously ordered, the groundings resolve to cached Wikidata entities and
  the seed grounds each asset in that Q-id, the data file declares the registry,
  every named function exists in Rust, every parity target exists in both the
  Rust source and the JS worker, and the doc links the recipe and test. If the
  recipe and the code drift apart, CI fails, so the recipe stays an accurate,
  executable description of how the code was produced.
- [`docs/meta-algorithm.md`](../../../docs/meta-algorithm.md) documents the eight
  steps and how to run the grounding test.

Run the grounding test with:

```text
cargo test --test unit specification::market_price_verification_meta_algorithm -- --nocapture
```

## Alternatives Considered

| Option | Result |
|---|---|
| Leave the generic web-search plan only. | Insufficient, because the issue asks for a concrete answer on whether the claim is false. |
| Hardcode a response for this image. | Rejected because it would not generalize to similar OCR/text claims or other supported languages. |
| Add a structured, data-driven market-data verifier. | Chosen. It keeps the deterministic solver offline, records source evidence, grounds recognition in Wikidata meaning, and extends to new assets/periods/languages purely by adding seed data. |
| Hardcode per-language ETH phrase lists in Rust/JS. | Rejected per issue #386: no per-language phrase list lives in code. Aliases live in seed data and recognition is by grounded meaning. |
| Build live multi-source price fetching in the request path. | Useful future extension, but not required for this deterministic regression. The captured source data and registry make the result reproducible. |

## Verification

The reproducing test was added before implementation and failed with unresolved
imports for the missing market-price extractor/assessor:

```text
cargo test --test unit issue_493 -- --nocapture
```

After implementation, the focused regression passes and records the OCR
evidence:

```text
cargo test --test unit issue_493 -- --nocapture
```

The regression covers:

- OCR text containing the key ETH 2024 claim.
- Extraction of repeated ETH/year/price claims from multi-line OCR text.
- Extraction when the asset ticker itself uses a dollar marker, such as
  `Buy $ETH in 2024 at $1,700`.
- Extraction of localized ETH aliases in English, Russian, Hindi, and Chinese.
- Cross-asset generalization: `BTC in 2024: $1,700` is contradicted by the
  Binance BTCUSDT 2024 reference through the same machinery.
- The within-range vs contradicted distinction: `$1,700` is assessed as within
  the recorded ETH range for 2021–2023 and contradicted only for 2024.
- Assessment of `ETH in 2024: $1,700` as contradicted by Binance ETHUSDT 2024
  klines.
- Event-log evidence preserving the market-data range and lowered posterior.
- The answer body summarizing only the contradicted price claims.
- The grounding specification test keeping the recipe aligned with live source.

Additional verification commands saved in `raw-data/`:

```text
cargo fmt --all -- --check
cargo clippy --all-targets --all-features
cargo test --all-features --verbose
cargo test --doc --verbose
cargo test --test unit specification::market_price_verification_meta_algorithm -- --nocapture
cargo test --test unit document_originality -- --nocapture
cargo test --test unit issue_535 -- --nocapture
cargo test --test source statement_verification -- --nocapture
node --check src/web/worker/formal_ai_worker_21.js
node tests/e2e/scripts/check-language-test-coverage.mjs
bun run build:web
npx playwright test tests/issue-493.spec.js --config=playwright.local.config.js
npx playwright test tests/issue-535.spec.js --config=playwright.local.config.js
```
