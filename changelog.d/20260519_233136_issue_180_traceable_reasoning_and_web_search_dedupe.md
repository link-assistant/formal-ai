---
bump: minor
---

### Added

- Issue #180: every `solve()` turn now ends with a `deformalize` reasoning
  step that projects the resolved formalization back to natural language. A
  new `formalization-context` payload is threaded through every handler so
  the worker can emit `formalize → <handler> → formalize_resolved →
  deformalize` for the fact-style and `web_search` flows, and `formalize →
  <handler> → deformalize` for greeting / unknown / agent / memory flows. The
  diagnostics row carries the worker-emitted `projection.summary` (with the
  `⇒` glyph) so the symbolic-to-natural-language hand-off is visible in the
  UI, not only in the underlying step payload.
- Google-style rendering for `web_search` results: each hit is now formatted
  as `url + title (≥ domain) + fragment containing query + "Read more"` with
  the source priority order **DuckDuckGo, Internet Archive, Wikipedia,
  Wikidata, Wiktionary, then everything else**. Duplicate Wikipedia /
  Wikidata entries for the same canonical Q-id collapse into a single bullet
  with the alternate URLs surfaced under a localized `"Другие
  источники:" / "Other sources:"` sub-line.
- Per-session CORS availability cache. Each provider is probed once per tab
  and the result is kept in RAM until the tab is closed, so unreachable
  providers no longer add latency to subsequent searches in the same session.
- Diagnostics mode: raw HTTP request / response panels per provider call and
  a unified Links Notation block per reasoning step. Every diagnostics row
  also exposes a stable `data-step` attribute so automation can assert raw
  step kinds (`impulse`, `formalize`, `formalize_resolved`, `deformalize`, …)
  without depending on the i18n-localised display label.
- Six new unit tests in `src/web_search_core.rs` pinning the issue-180
  contract: provider priority order, language-line trimming, Internet Archive
  CORS readability, the Cormack / Clarke / Buettcher RRF formula
  (`score = 1 / (k + rank)`, `k = 60`), human-readable provider labels in the
  default plan, and the default plan being a subset of the provider
  registry. All 16 `web_search_core` tests pass (10 existing + 6 new).
- Playwright spec `tests/e2e/tests/issue-180.spec.js` with three scenarios
  (greeting prompt ends with `deformalize`, unknown prompt ends with
  `deformalize`, `web_search` emits `formalize` → `formalize_resolved` →
  `deformalize` with the `⇒` projection summary). Registered in
  `tests/e2e/playwright.local.config.js`. All 136 local Playwright tests
  pass.
- Node-side smoke test `experiments/issue-180-deformalize-trace.mjs` boots
  the Web Worker inside a `vm.createContext` shim and asserts the full step
  list across the greeting / unknown / fact-style / web-search flows (24
  assertions). Useful for fast local regressions without booting Playwright.
- Issue #180 case study under `docs/case-studies/issue-180/`: the issue and
  comment payloads, the three screenshots referenced in the issue, and a
  deep dive into requirements R210–R220 (Google-style rendering, dedupe,
  priority order, session-CORS cache, dark theme, single-column menu,
  diagnostics badges, raw HTTP panels, always-on deformalize, 2× test
  coverage, case study).

### Fixed

- Dark theme parity in the new UI: topbar collapse / expand affordance and
  the source-code button now honor the active palette; broader audit of the
  remaining surfaces.
- Left menu now renders as a single column on both mobile and desktop and
  stays collapsible.
- Diagnostics badges: sizing and markup are now consistent across providers
  so long provider labels no longer break the row layout.
