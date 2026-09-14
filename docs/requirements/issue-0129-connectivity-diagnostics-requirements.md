## Issue #129 Connectivity Diagnostics Requirements

Issue [#129](https://github.com/link-assistant/formal-ai/issues/129) reports
that the deployed `/tests` route is broken and asks for an interactive
diagnostics page for browser provider connectivity, public knowledge APIs,
iframe checks, `web-capture` proxy mode, and preserved case-study evidence.

| ID | Requirement | Status |
| --- | --- | --- |
| R165 | Serve a real GitHub Pages diagnostics route at `/formal-ai/tests/` instead of the GitHub Pages 404 page. | Implemented by `src/web/tests/index.html`, with the existing Pages artifact upload publishing the nested static directory. |
| R166 | Provide interactive checks for search-engine pages and public knowledge databases. | Implemented by the provider matrix in `src/web/tests/connectivity.js`, covering Google, Bing, DuckDuckGo, Brave, Yahoo, Wikipedia, Wikidata, Open Library, OpenAlex, Crossref, and Semantic Scholar. |
| R167 | Test both web-page access and API access through browser `fetch()`. | Implemented by per-row `Fetch page` and `Fetch API` controls, status pills, final URL display, response preview, and JSON export. |
| R168 | Make iframe diagnostics expandable. | Implemented by per-row inline iframe panels and a full-window expanded iframe overlay. |
| R169 | Let maintainers switch fetches through a local `web-capture` proxy. | Implemented by direct/proxy mode controls with configurable proxy base and `/fetch`, `/html`, or `/markdown` endpoint selection. |
| R170 | Keep the diagnostics route covered locally and after Pages deploy. | Implemented by `tests/e2e/tests/connectivity.spec.js` and by including that spec in both Playwright configs. |
| R171 | Stamp nested diagnostics HTML with the same version and asset cache-busting placeholders as the root demo. | Implemented by updating `scripts/stamp-pages-artifact.sh` to process every HTML file under the Pages artifact and by unit smoke coverage in `tests/unit/ci-cd/workflow_release.rs`. |
| R172 | Preserve issue data, live responses, related upstream data, online research, root-cause analysis, and follow-up planning. | Implemented in `docs/case-studies/issue-129/` with raw data under `raw-data/`. |
