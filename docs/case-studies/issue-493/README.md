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
document-verification path. Multi-line OCR text is preserved, ETH/year/price
claims are extracted from the sample, and the 2024 ETH claim is assessed against
captured Binance ETHUSDT daily kline data.

## Requirements

This case study tracks the requirements from the issue and prepared work order:

| Requirement | Evidence |
|---|---|
| Preserve all issue and PR data. | Raw GitHub JSON, comments, reviews, and the issue screenshot are saved under this directory. |
| Work with text inside an image. | The screenshot is preserved and a Tesseract.js OCR experiment is saved in [`raw-data/tesseract-issue-screenshot-ocr.json`](raw-data/tesseract-issue-screenshot-ocr.json). |
| Catch the false price statement. | The unit regression asserts that `ETH in 2024: $1,700` is contradicted. |
| Search online and compare external data. | Binance API data and CoinGecko API/page captures are preserved in [`raw-data/`](raw-data/). |
| Use the meta algorithm, recursive reasoning, and relative-meta-logic. | The handler logs per-statement verification, source-tier priors, market-source evidence, and the lowered posterior for contradicted claims. |
| Generalize beyond one screenshot. | The parser accepts asset aliases, year periods, and currency-marked amounts in OCR/text samples, with aliases for English, Russian, Hindi, and Chinese ETH mentions. |
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

The Binance ETHUSDT 2024 daily kline capture contains 366 UTC daily candles.
The parsed range is:

| Metric | Date | Price |
|---|---:|---:|
| Minimum daily low | 2024-01-03 | 2100.00 |
| Maximum daily high | 2024-12-16 | 4107.80 |

Since USD 1,700 is below the minimum recorded daily low, the statement
`ETH in 2024: $1,700` is contradicted by the captured market data.

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
  amount.
- Extracted claims are checked against a versioned market-data reference. For
  this issue, the reference is Binance ETHUSDT daily klines for 2024.
- Contradicting market data is represented as original-first relative evidence,
  lowering the statement posterior and preserving the full trace in
  `market_price_claim:*` event-log links.
- User-facing verification answers add a concise `Price claim check` section
  when any extracted market claim is contradicted.
- The browser worker mirrors the Rust path so uploaded image/OCR checks surface
  the same contradiction in the web UI.

## Alternatives Considered

| Option | Result |
|---|---|
| Leave the generic web-search plan only. | Insufficient, because the issue asks for a concrete answer on whether the claim is false. |
| Hardcode a response for this image. | Rejected because it would not generalize to similar OCR/text claims or other supported languages. |
| Add a structured market-data verifier. | Chosen. It keeps the deterministic solver offline, records source evidence, and extends by adding new market references. |
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
- Assessment of `ETH in 2024: $1,700` as contradicted by Binance ETHUSDT 2024
  klines.
- Event-log evidence preserving the market-data range and lowered posterior.
- The answer body summarizing the contradicted price claim.

Additional verification commands saved in `raw-data/`:

```text
cargo fmt --all -- --check
cargo clippy --all-targets --all-features
cargo test --all-features --verbose
cargo test --doc --verbose
cargo test --test unit document_originality -- --nocapture
cargo test --test unit issue_535 -- --nocapture
cargo test --test source statement_verification -- --nocapture
node --check src/web/worker/formal_ai_worker_21.js
node tests/e2e/scripts/check-language-test-coverage.mjs
bun run build:web
npx playwright test tests/issue-493.spec.js --config=playwright.local.config.js
npx playwright test tests/issue-535.spec.js --config=playwright.local.config.js
```
