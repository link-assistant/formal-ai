## Issue #709 Multi-Source Search Fusion

Issue [#709](https://github.com/link-assistant/formal-ai/issues/709) composes
exact search/page capture, multilingual formalization, #844's statement merge,
relative source tiers, and normalized presentation into one ranked answer. See
`docs/case-studies/issue-709/` and PR #884.

| ID | Requirement | Status |
| --- | --- | --- |
| R709-1 | Formalize every captured search hit and fetched page with source provenance. | `execute_search_fusion` records a `FormalizedSearchObservation` plus event-log and learning-proposal receipts for each statement. |
| R709-2 | Merge equivalent meanings across languages and rank them using original, independent, and unoriginal source tiers. | Complete Q/P/Q meaning links enter #844's semantic signature; reposts are traced but excluded from evidence. |
| R709-3 | Deformalize the smallest sufficient ranked answer into the query language and show both conflict sides with posteriors. | The selection is bounded to three meanings, retains both polarities, and emits `conflict:source_disagreement`. |
| R709-4 | Normalize URL, title, quote, and read-more fields across web, CLI/HTTP, and Telegram. | `NormalizedSearchSource`, the shared Rust Markdown renderer, Telegram HTML conversion, and the browser worker source cards are covered by unit and Playwright fixtures. |
| R709-5 | Replay deterministically in CI while live search remains explicitly gated. | A three-source exact-capture fixture compares the live and offline render, trace, and proposal byte-for-byte; browser providers are intercepted. |
