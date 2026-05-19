---
bump: minor
---

### Added

- Issue #153: every reasoning step is now formalized as a deterministic
  `(Subject Verb Object)` tuple using `@USER`, `OP:<verb>`, `Q<n>`, `WP:<key>`,
  and `WT:<word>` ids regardless of the prompt's source language. Diagnostics
  mode shows the raw message, the SVO tuple, and a numbered S / V / O slot
  list through a new `FormalizationView` React component. A second
  `formalize_resolved` step folds the matched Wikidata Q-id (when one is
  found) back into the tuple as `(@USER OP:search Q89)` so the trace records
  the symbolic mapping end-to-end.
- Cross-provider deduplication for web search results. `searchWikidataEntities`
  now requests `props=sitelinks/urls`, and a new
  `canonicalEntityKey` / `buildItemMetadataIndex` / `dedupeFusedEntries`
  pipeline collapses entries returned by multiple providers (Wikipedia +
  Wikidata for the same `Q89`) into a single bullet with the other URLs
  surfaced under an `"Other sources:"` sub-line in the user's language. Each
  merge is appended to memory as
  `web_search:dedupe:<key>:absorbed:<url>` so the trace stays replayable.
- Localized search results template covering `en`, `ru`, `zh`, `hi`. Header
  (`Search results for / Результаты поиска для / 搜索结果 / खोज परिणाम`), the
  empty-state line, and the "Other sources:" sub-line all render in the UI
  language picked up from `navigator.language` / saved preferences.
- "Source code" link in the top menu, pointing to
  `https://github.com/link-assistant/formal-ai`, with i18n labels for
  `buttons.sourceCode` / `titles.sourceCode` in all four locales.
- Collapsible left sidebar for desktop, persisted through a new
  `sidebarCollapsed` preference. The mobile drawer is unchanged. A
  `[data-testid="sidebar-toggle"]` button with i18n labels
  (`buttons.collapseSidebar` / `buttons.expandSidebar`) flips the state, and
  `.workspace.sidebar-collapsed` styles the collapsed layout.
- Playwright spec `tests/e2e/tests/issue-153.spec.js` with eight scenarios
  (lab emoji, source-code link, disabled `New conversation`, sidebar collapse,
  SVO formalization view, cross-source dedupe, DuckDuckGo signature
  regression, localized search header) registered in
  `tests/e2e/playwright.local.config.js`. All 127 local Playwright tests pass.
- Issue #153 case study under `docs/case-studies/issue-153/`, including raw
  issue JSON, the three screenshots from the issue description, and a deep
  analysis of requirements R195–R209.

### Fixed

- DuckDuckGo provider was silently returning zero results because
  `searchDuckDuckGo(query, limit)` was declared with two parameters while the
  dispatcher passed three (`(query, language, providerLimit)`). The new
  signature `(query, language, limit)` coerces `limit` to a numeric cap with
  `Math.floor`, defaults to 5 when missing, and forwards a `kl=<lang>-<lang>`
  region hint when the UI language is not English. A new regression test
  proves the fix.
- The diagnostics toggle's magnifying-glass icon (🔍) was replaced with a lab
  emoji (🧪) to match the issue's request for a "diagnostics" affordance.
- `New conversation` is now disabled when the chat is empty so the click is
  no longer a no-op.

### Removed

- Stripped the `Providers (default first): duckduckgo, wikipedia, wikidata.`
  footer from search responses. Providers still appear inline next to each
  bullet (`via wikipedia#2, wikidata#1`), so no information is lost.
