# Case study — issue #458: crypto portfolio tracker fell through routing

> Source issue: <https://github.com/link-assistant/formal-ai/issues/458>
> Branch: `issue-458-1fb70f7869d0` · PR: #567

## Summary

The reported prompt asked the assistant to simulate a crypto portfolio tracker:
search current prices for BTC, ETH, TON, and USDT; assume fixed holdings; compute
total USD value, 24h changes, and portfolio weights; then output dashboard-style
Markdown plus executable Python code with a mocked public API endpoint and
drop-alert logic.

The deployed response did not synthesize the requested program. The original
report showed a `write_program(language, task)` routing dead end with
`task=missing`; the current branch reproduced the same product failure as a
generic `web_search` answer because the leading "Search current prices" signal
claimed the prompt before composite program synthesis.

## Root cause

Composite program blueprints were only attempted when the selected rule was
`UnsupportedWriteProgram`. Issue #458 is a mixed-intent prompt: it contains both
real-time-data wording ("Search current prices") and an explicit coding request
("Write a Python script"). Because the web-search path is broader, the solver
could return search guidance without ever checking whether the full prompt
matched a supported program blueprint.

The existing blueprint catalog also had no recipe for the requested crypto
portfolio shape, so even the older unsupported-write-program path had no
reviewed code body to return.

## Fix

- Added a `crypto_portfolio_tracker` Python blueprint that models the requested
  portfolio, uses deterministic mocked price data, calculates total value and
  portfolio weights, emits 24h change percentages, renders a Markdown dashboard,
  and includes `notify_alerts` logic for assets dropping more than 5%.
- Added capability detection for crypto prices, holdings, portfolio
  calculations, alerts, and mocked API endpoints.
- Added a guarded solver preemption: for any non-concrete catalog rule, try a
  recognized program blueprint before later broad fallback handlers. Concrete
  catalog programs still keep precedence.
- Mirrored the recipe and direct blueprint probe in the JavaScript worker.

## Verification

- Reproduced the pre-fix behavior in [`raw-data/repro-before.txt`](./raw-data/repro-before.txt).
- Added `tests/integration/issue_458_crypto_portfolio.rs` for the reported
  prompt. It asserts `write_program`, a Python fence, all requested assets,
  dashboard Markdown, `portfolio_weight`, `notify` logic, and the
  `program_blueprint:recipe crypto_portfolio_tracker` trace.
- Added a source-level blueprint selection test for the new recipe.
