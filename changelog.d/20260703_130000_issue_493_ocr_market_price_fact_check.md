---
bump: minor
---

### Added
- Added deterministic OCR/text market-price claim extraction for the document verification path, including ETH aliases across supported languages and source-backed relative-meta-logic assessments.
- Mirrored market-price contradiction checks in the browser worker and added an e2e regression for the image/OCR flow.
- Preserved issue #493 evidence under `docs/case-studies/issue-493`, including the screenshot, OCR output, market-data captures, and before/after regression logs.

### Fixed
- Preserved full multi-line OCR/text samples during document verification so factual claims after the first line are checked instead of being dropped from the statement plan.
- Flagged `ETH in 2024: $1,700` as contradicted using captured Binance ETHUSDT 2024 daily klines.
