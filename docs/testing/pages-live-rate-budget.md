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

## Cold public-worker readiness

Main run 41882656145, job 126815222022, exhausted the localhost-calibrated
20-second composer wait. Preserved diagnostics reported "loading worker", a
disabled composer, no errors, and source 241e9fc5f9450bc8c25c42979508cf240434d042.
The browser entry imports the module inventory serially; public CDN transport
is part of this cold start. This establishes that startup exceeded the old bound,
not observed eventual readiness or a browser acceptance pass.

The public smoke allows one cold start up to 60 seconds within a 90-second test
and a 180-second global cap. Local artifact waits retain their 20-second bound;
the original full Pages configuration remains unchanged. There are still no
retries and only one live context. Readiness, real WASM mode, the actual native
arithmetic answer, HTTP failures and fallback refusals remain mandatory. A ready
worker reports measured startup time. A failure preserves bounded pending network
requests and recent completions, with query strings removed, along with runtime
errors and the original assertion. Unit replays verify these oracles; the hosted
live smoke must establish actual production readiness.

Reducing serial public asset imports is deferred performance work. These timing
allowances do not establish a faster worker or full public-matrix coverage.
