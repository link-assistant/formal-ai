# Issue 896: published web components in production

Issue [#896](https://github.com/link-assistant/formal-ai/issues/896) found that
Formal AI still owned its production web-search, fetch, cache, and browser
worker paths even though the `web-search` and `web-capture` components had been
published after the earlier issue 410 audit. The published projects were
documented, but they were not dependencies and no runtime receipt proved that
Formal AI crossed either component boundary.

## Result

Native DuckDuckGo execution now uses `web_capture::search` to build and parse
the provider capture and `web_search::merger` to fuse normalized rankings. The
existing `CachedSourceClient` remains the transport boundary, so live and
offline execution retain the exact captured bytes, URL, retrieval time,
SHA-256 digest, and cache state.

The browser worker bundles `@link-assistant/web-search` and uses its published
default-provider registry and `mergeResults` implementation in the production
query path. Explicit URL capture first calls web-capture's published HTTP
`GET /fetch?url=...` contract. A worker-owned abort deadline wraps that request,
and component failures become diagnostics before the direct-CORS compatibility
path is considered.

Both surfaces emit component-boundary receipts. A successful response or
ranking is still not source evidence by itself: only the existing exact
capture and normalized result records can enter downstream evidence.

## Root cause and bounded fallbacks

The earlier issue 410 case study correctly found publication and provider
parity gaps at that time, but the integration remained deferred after those
gaps were closed. Formal AI therefore continued to call providers directly and
run its local reciprocal-rank fusion without first invoking the now-published
libraries.

Four smaller capability gaps remain in the current component releases:

- [`web-search#21`](https://github.com/link-assistant/web-search/issues/21)
  requests caller-owned transport, cancellation, and per-provider errors;
- [`web-capture#147`](https://github.com/link-assistant/web-capture/issues/147)
  requests injectable transport, cancellation, and exact response receipts;
- [`web-search#22`](https://github.com/link-assistant/web-search/issues/22)
  requests a merge-only feature without the server and capture dependency
  graph;
- [`web-capture#148`](https://github.com/link-assistant/web-capture/issues/148)
  requests a pure search-adapter feature without browser and server
  dependencies.

Until those APIs ship, the native adapter injects Formal AI's already-reviewed
exact-byte cache client into web-capture's pure URL/parser contract. Provider
HTML that is unavailable, blocked, or empty falls back to the existing
DuckDuckGo Instant Answer adapter and records the component failure. In the
browser, the published web-search merger falls back to the existing WASM/local
merger only if the optional bundle is absent or rejects the input; web-capture
service failures are recorded before the existing direct request is attempted.
These workarounds are deliberately confined to the native and browser
component bridges.

The Rust crates currently pull native TLS through their unconditional runtime
graphs. The slim Docker builder installs `pkg-config` and `libssl-dev` as a
builder-only workaround; the runtime image remains unchanged. Once the two
feature-gating gaps ship, Formal AI can select only the adapter and merger
features and remove those build packages.

## Dependency and license review

The production dependencies are the published Rust crates `web-capture`
0.3.34 and `web-search` 0.3.1, plus the browser package
`@link-assistant/web-search` 0.10.3. All three declare the Unlicense, matching
this repository. No upstream source or fixture is copied into Formal AI; the
generated browser bundle is built from the locked package. Runtime web results
remain external observations and retain their existing provenance rules.

## Verification

The minimum native integration fixture accepts only web-capture's exact
DuckDuckGo URL. It checks parsed rankings, the published merger result,
component receipts, captured bytes, and cache-only replay without another
transport call. A companion assertion holds the published default-provider
list equal to Formal AI's production plan.

The browser integration runs the real worker. For all five supported languages,
it proves the bundled registry and merger receipts appear in a search answer.
It then intercepts web-capture's HTTP endpoint and verifies an exact target body
and 503 status are preserved without a direct target request. Separate cases
prove network failures and the two-second cancellation deadline are diagnosed
before the bounded direct-request fallback runs.

Run the focused checks with:

```sh
cargo test --test unit issue_896_component_boundaries -- --nocapture
bunx playwright test --config playwright.local.config.js tests/issue-896.spec.js
```

## Same-task self-application

The reviewed task decomposition has five smallest leaves. Formal AI serves the
model used by the external Agent CLI to author the component-boundary invariant
leaf, while the four implementation and verification leaves are manually
authored. The canonical invariant is compared byte-for-byte with the generated
artifact, making the same-task authorship share one of five leaves (20%). The
raw Agent CLI and server traces are retained under
`self-hosting-authorship/`; the repeatable harness is
`experiments/issue_896_self_authoring/run.sh`. The retained session is
`ses_03b4c6698ffedcgrdsX2ni15WK`.
