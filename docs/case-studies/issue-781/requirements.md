# Requirements trace

| ID | Requirement | Evidence |
|---|---|---|
| R1 | Find the best supportable answer for the A325-45 charger search. | `recommendation.md` separates verified requirements from conditional Amazon fallbacks. |
| R2 | Search and capture actual sources. | Official, marketplace, shared-dialog, and Amazon attempts are inventoried under `raw-data/`; capture failures are retained too. |
| R3 | Ingest the supplied ChatGPT share. | `chatgpt-web-capture.json`, both generated `.lino` files, unit coverage, and the CLI integration test. |
| R4 | Prepare a Google shared-dialog adapter without guessing unavailable text. | Normalized web-capture JSON is supported; `google-ai-mode-browser.json` preserves the provider diagnostic. |
| R5 | Improve Formal AI generally, not with charger-specific phrases. | The planner performs bounded multi-source capture for arbitrary languages and research subjects; product words occur only in the issue regression. |
| R6 | Use recursive/cross-source reasoning. | One search fans out to up to three independent fetches, whose evidence returns to the next planning step with URL identity intact. |
| R7 | Reproduce the bug before fixing it. | `raw-data/reproduction-before-fix.log` records the one-versus-three failing assertion. |
| R8 | Preserve provenance in Links Notation. | Each converted event carries the share URL; direct and adapter conversions are byte-identical. |
| R9 | Research relevant libraries and related work. | `investigation.md` traces Formal AI #552, web-capture #141, meta-language #168, and the installed web-capture adapter contract. |
| R10 | Avoid unsafe or unsupported purchase claims. | Amazon browser captures contain the automated-access notice; `recommendation.md` makes the fallback conditional on seller confirmation. |
| R11 | Cover the whole task with automated tests. | Issue regression, shared-dialog unit tests, real-fixture CLI integration test, and existing web-research suites. |
