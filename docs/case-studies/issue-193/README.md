# Issue #193 Case Study: Bun-Bundled Browser Dependencies

Issue: https://github.com/link-assistant/formal-ai/issues/193
Pull request: https://github.com/link-assistant/formal-ai/pull/194

## Timeline

| Date (UTC) | Event |
| --- | --- |
| 2026-05-20 18:43 | Issue #193 opened with a Firefox console screenshot showing blocked `esm.sh/lino-i18n` module requests and untranslated UI keys. |
| 2026-05-20 18:44 | PR #194 opened as a draft for branch `issue-193-64742fb93b00`. |
| 2026-05-20 18:46 | Baseline PR CI completed successfully before the fix. |
| 2026-05-20 18:52 | Local Bun bundle build produced `src/web/vendor.bundle.js`; the focused Playwright regression passed with the old CDN hosts blocked. |
| 2026-05-20 19:00 | Manual browser verification loaded the app from the local bundle with translated controls and no CDN script URLs. |

## Raw Data

- `raw-data/issue-193.json`: issue title, body, labels, author, and timestamps.
- `raw-data/issue-193-comments.json`: no issue comments at investigation time.
- `raw-data/pr-194.json`: draft PR metadata and baseline status checks.
- `raw-data/pr-194-review-comments.json`, `raw-data/pr-194-conversation-comments.json`, `raw-data/pr-194-reviews.json`: no review or conversation feedback at investigation time.
- `raw-data/ci-runs-before-fix.json`: recent branch CI run list before implementation.
- `raw-data/github-issue-193-screenshot.jpg`: downloaded issue screenshot. The file has a JPEG magic header.
- `raw-data/online-research.md`: external references used for the build and root-cause analysis.
- `screenshots/after-local-bundled-i18n.png`: local browser verification screenshot after the fix. The file has a PNG magic header.

## Requirements

| Requirement | Status |
| --- | --- |
| Prebundle browser JavaScript with Bun if possible. | Implemented with `bun build` through `bun run build:web`. |
| Avoid external JavaScript CDNs on GitHub Pages. | Implemented by replacing CDN script/import-map tags with `vendor.bundle.js`. |
| Fix UI i18n failures for users affected by blocked remote module requests. | Implemented by loading `lino-i18n@0.1.1` from the local vendor bundle. |
| Preserve issue data and analysis under `docs/case-studies/issue-193`. | Implemented with raw GitHub metadata, screenshot evidence, online research, and this analysis. |
| Add debug output or verbose mode if root cause is not knowable. | Not needed; the screenshot, HTML, and local reproduction identify the root cause directly. |
| Report upstream issues if another project is responsible. | Not needed; this repo was depending on remote CDN modules even though the product requirement is same-origin Pages assets. |

## Root Cause

`src/web/index.html` loaded React and ReactDOM from `unpkg.com`, markdown/sanitization libraries from `cdn.jsdelivr.net`, and `lino-i18n` through an import map pointed at `esm.sh`. The reported browser then attempted to load additional `lino-i18n` module files from `esm.sh`, and those cross-origin requests failed. When `lino-i18n` did not initialize, the UI fell back to raw i18n keys such as `buttons.importMemory` and `composer.placeholder.chat`.

The root issue is not the translation catalog itself. It is the runtime delivery path: the GitHub Pages app required external JavaScript CDNs at startup.

## Solution

The browser dependencies now build into a local vendor bundle:

- `src/web/vendor-entry.js` imports React, ReactDOM, marked, DOMPurify, and `lino-i18n`.
- `bun run build:web` emits `src/web/vendor.bundle.js` as a browser IIFE.
- `src/web/index.html` loads that local bundle before the app scripts.
- `src/web/i18n.js` reads `lino-i18n` from `window.FormalAiVendor` instead of dynamically importing from the import map.
- CI installs Bun, builds the bundle, verifies the committed bundle is current, and builds the same bundle before local e2e and Pages deployment.

## Verification

The focused regression is `tests/e2e/tests/issue-193.spec.js`. It aborts requests to `unpkg.com`, `cdn.jsdelivr.net`, and `esm.sh`, then verifies the app starts and Russian UI translations resolve through `lino-i18n@0.1.1`.

Manual commands run locally:

```text
bun install --frozen-lockfile
bun run build:web
node --check src/web/i18n.js
node --check src/web/vendor-entry.js
node --check tests/e2e/tests/issue-193.spec.js
git diff --check
npx --prefix tests/e2e playwright test --config=tests/e2e/playwright.local.config.js tests/e2e/tests/issue-193.spec.js
npm run --prefix tests/e2e check:i18n
npm run --prefix tests/e2e check:intent-coverage
npm run --prefix tests/e2e test:local
cargo fmt --all -- --check
rust-script scripts/check-file-size.rs
cargo clippy --all-targets --all-features
cargo test --all-features --verbose
cargo test --doc --verbose
cargo build --release --verbose
cargo package --list --allow-dirty
rust-script scripts/check-crate-package-size.rs
```

Result: all checks passed. The full local Playwright suite reported 147 passed, and the crate package-size check reported the generated archive within the crates.io limit.
