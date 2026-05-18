# Issue #110 Mobile Viewport and Skin Configuration Case Study

## Source Material

- Issue: https://github.com/link-assistant/formal-ai/issues/110
- PR: https://github.com/link-assistant/formal-ai/pull/111
- Related issue: https://github.com/link-assistant/formal-ai/issues/108
- Related merged PR: https://github.com/link-assistant/formal-ai/pull/109
- Raw GitHub exports are saved in `raw-data/`.
- Issue #110 screenshots and after-fix verification screenshots are saved in `screenshots/`.
- Issue #108 already has its full screenshot evidence in `../issue-108/`.
- PNG downloads were verified by PNG magic bytes and IHDR dimensions because the `file` utility was not available in this environment. Verification records are in `raw-data/screenshot-metadata.md`.

## Timeline

- 2026-05-18 08:38:50 UTC: Issue #108 opened with the original mobile UI and input styling requirements.
- 2026-05-18 09:36:24 UTC: PR #109 merged for issue #108.
- 2026-05-18 09:49:24 UTC: Issue #110 opened, reporting a remaining focused-input mobile breakage and incomplete skin support.
- 2026-05-18 09:58:19 UTC: PR #111 opened as a draft for issue #110.

## Problem Summary

Issue #110 reported that the mobile page starts in a usable state, but after focusing the chat input the top menu and chat history disappear. The issue also clarified that the previous skin work was incomplete: settings need separate configurable styles for the whole UI, the chat UI, and the input box, while keeping the existing mobile requirements from issue #108.

## Evidence

- `screenshots/current-mobile-load.png`: the current mobile load state starts with topbar, chat, and composer visible.
- `screenshots/current-mobile-focused.png`: after activating the input, the keyboard is open but the topbar and chat are pushed out of view.
- `screenshots/current-unsupported-skin.png`: settings expose theme, input style, and input action, but not separate whole-UI or chat UI styles.
- `raw-data/issue110-test-before.log`: reproducing Playwright test failed before the fix with topbar `y` at `0` while the visual viewport offset was `180`.
- `screenshots/after-mobile-focused-offset.png`: after the fix, the app shell is pinned to the simulated visual viewport offset.
- `screenshots/after-mobile-skin-settings-visible.png`: after the fix, mobile settings expose UI Skin, Chat Style, and Input Style controls.

## Requirements

- Keep the #108 mobile layout improvements: left mobile menu, hidden mobile wordmark, drawer branding/version, desktop version display, compact one-row composer, configurable input action, and strict mobile e2e coverage.
- Fix the focused mobile input state so the chat and top menu remain reachable when the keyboard changes the visual viewport.
- Add whole-UI skin configuration in addition to theme configuration.
- Add chat-specific style configuration.
- Keep input box style configuration.
- Persist user selections so experiments survive reloads.
- Include the selected UI configuration in issue reports so future screenshots and reports are easier to reproduce.
- Save raw issue data, logs, screenshots, and analysis under `docs/case-studies/issue-110/`.

## Root Causes

- The app used `visualViewport.height` but ignored `visualViewport.offsetTop` and `offsetLeft`. On mobile browser focus, especially with an on-screen keyboard, the visual viewport can move relative to the layout viewport. The shell was still anchored to the layout viewport origin, so the topbar could land outside the visible area.
- Existing e2e coverage checked the focused composer layout, but did not simulate a non-zero visual viewport offset. That left this post-#108 regression untested.
- The configuration model only had theme plus composer style/action. There were no persisted preferences, CSS hooks, labels, or tests for whole-UI skin and chat-message style.

## Research Notes

- MDN's Visual Viewport API reference describes the distinction between the layout viewport and the visual viewport, and notes that on-screen keyboards can shrink the visual viewport without changing the layout viewport: https://developer.mozilla.org/en-US/docs/Web/API/Visual_Viewport_API
- MDN documents `VisualViewport.offsetTop`, `offsetLeft`, `height`, `width`, and the `resize`/`scroll` events needed to keep fixed UI aligned with the visible viewport: https://developer.mozilla.org/docs/Web/API/VisualViewport
- web.dev documents the limitations of viewport units on mobile and notes that virtual keyboards are not consistently reflected in viewport units, so JavaScript visual viewport handling remains necessary for keyboard focus cases: https://web.dev/blog/viewport-units
- MDN documents CSS environment variables such as safe-area and keyboard inset values. They are useful supporting primitives, but support varies enough that the fix should keep the existing `visualViewport` runtime path: https://developer.mozilla.org/en-US/docs/Web/CSS/Guides/Environment_variables

## Solution Plan

- Use `window.visualViewport` as the primary source for shell width, height, and offset when available.
- Store the visual viewport metrics as CSS variables and position the app shell with those variables.
- Preserve `100dvh` and existing fallbacks for browsers that do not expose `visualViewport`.
- Add persisted `uiSkin` and `chatStyle` preferences with conservative defaults.
- Add CSS class hooks for UI skins and chat styles so future skins can be developed without rewriting component logic.
- Extend e2e coverage with a minimal fake `visualViewport` that reproduces the non-zero offset failure before the fix.

## Implementation

- The app shell now sets `--formal-ai-viewport-width`, `--formal-ai-viewport-height`, `--formal-ai-viewport-offset-left`, and `--formal-ai-viewport-offset-top` from `window.visualViewport` on window and visual viewport `resize`/`scroll` events.
- `.app` is fixed to the current visual viewport offset and sized to the current visual viewport dimensions.
- Added UI skins: `flat`, `glass`, and `contrast`.
- Added chat styles: `cards`, `compact`, and `bubbles`.
- Added settings controls and localized labels for UI Skin and Chat Style.
- Added persisted preference fields and issue-report context fields for `uiSkin` and `chatStyle`.
- Added Playwright tests for the visual viewport offset regression and for persisted UI/chat/composer style settings.

## Verification

- `node --check src/web/app.js`: passed.
- `node --check src/web/i18n.js`: passed.
- `node --check tests/e2e/tests/demo.spec.js`: passed.
- `node --check tests/e2e/tests/multilingual.spec.js`: passed.
- `cd tests/e2e && npm run test:local -- --grep "Issue #110"`: 1 passed after the fix.
- `cd tests/e2e && npm run test:local -- --grep "Issue #108"`: 4 passed after the fix.
- `cd tests/e2e && npm run test:local -- --grep "Issue #(108|110)"`: 5 passed after final CSS review.
- `cd tests/e2e && npm run test:local`: 87 passed.
- `cargo fmt --all -- --check`: passed.
- `cargo clippy --all-targets --all-features`: passed.
- `rust-script scripts/check-file-size.rs`: passed with existing warnings for `src/solver_helpers.rs` and `src/seed.rs` approaching their configured line limits.
- `cargo test --all-features --verbose`: 266 passed, 69 ignored, plus 0 doctests.
- `cargo test --doc --verbose`: 0 doctests.
- Final verification logs are saved in `raw-data/`.

## External Issue Decision

No upstream browser or library issue is required for this PR. The reproduced failure comes from this application anchoring the shell to the layout viewport while only using visual viewport height. The browser behavior matches the documented Visual Viewport API model.
