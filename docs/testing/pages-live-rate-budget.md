# PAGES-LIVE-CDN-RATE-BUDGET

Temporary release-first coverage debt, authorized by the maintainer: the default
live GitHub Pages job runs one serial smoke in one browser context with no retries.
It checks successful landing/app/docs HTTP delivery, the mounted app, disabled
JavaScript diagnostic fallback, the real `wasm worker` mode, and an actual native
engine arithmetic answer. HTTP 429, an unavailable engine, and a fallback answer
remain failures. The smoke blocks service workers to avoid installing the bulk
offline precache during this public-CDN check; offline behavior remains covered
by the functional tests on the owned CI artifact.

Production Pipeline run 38799356473, Pages job 116706159002, ran 177 live tests in
two workers with retries. Its report artifact 12012356508 records `Rate limit
exceeded` instead of app HTML and `wasm fetch failed: 429`. The artifact SHA-256
is `35d17fea8515479696b5a441335b020dc2e0a38499fd44ccf3244fa9daa42cd9`.
The suite exhausted its unchanged 720-second budget. These are delivery throttling
failures; no app boot source regression was established.

All original functional spec bodies, assertions, and the full Pages configuration
remain unchanged. The owned local-artifact E2E shards retain their full functional
coverage. This reduction supplies no full public-matrix acceptance credit.

Run the original full live matrix explicitly, after arranging a safe request
budget (from `rust/tests/e2e`):

```sh
PAGES_URL=https://link-assistant.github.io/formal-ai/ npx playwright test --config=playwright.pages.config.js
```

The ordinary publication workflow selects `playwright.pages.smoke.config.js`.
Restore the original full configuration as that workflow's default only after a
measured cache/request-budget strategy lets the unchanged full matrix pass on
the real public deployment within its existing timeout, without retry hammering
or suppressing 429 and runtime errors. Preserve real native-engine and delivery
assertions. Hosted smoke acceptance remains required; local source controls do
not execute or establish browser/WASM production success.
