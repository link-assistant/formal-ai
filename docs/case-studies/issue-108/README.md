# Issue #108 Mobile UI Case Study

## Source Material

- Issue: https://github.com/link-assistant/formal-ai/issues/108
- PR: https://github.com/link-assistant/formal-ai/pull/109
- Raw GitHub exports are saved in `raw-data/`.
- Downloaded issue screenshots are saved in `screenshots/`.
- PNG downloads were verified by magic bytes and IHDR dimensions because the `file` utility was not available in this environment. The verification records are in `raw-data/screenshot-metadata.txt`.

## Problem Summary

Issue #108 reported that the web demo was unusable on mobile: the initial composer could be partly hidden, focusing the input made the top menu unreachable, and the mobile topbar spent scarce width on the logo/wordmark. The issue also requested configurable input affordances, flat-by-default styling, Apple-glass-style transparent options, desktop version display near the logo/title, and strict mobile e2e coverage.

## Evidence

- `screenshots/current-mobile-load-01.png` and `screenshots/current-mobile-load-02.png`: input area hidden or crowded on first mobile load.
- `screenshots/current-mobile-focused-01.png` and `screenshots/current-mobile-focused-02.png`: focused input state loses practical access to the top menu.
- `screenshots/competitor-01.png` through `screenshots/competitor-07.png`: competitor references showing compact one-row input bars with a left-side attach/plus action and a small send control.
- `screenshots/after-mobile-focused-390x780.png`: fixed focused state at 390x780 with menu reachable and one-row composer.
- `screenshots/after-mobile-drawer-390x780.png`: fixed drawer state with logo, title, version, and sidebar controls inside the menu.

## Research Notes

- MDN documents that the visual viewport can shrink when the dynamic keyboard or browser chrome appears, while the layout viewport can remain unchanged. That explains why a `100vh` shell plus fixed bottom composer can be clipped on mobile focus: https://developer.mozilla.org/en-US/docs/Web/CSS/Guides/CSSOM_view/Viewport_concepts
- MDN notes that default `vh` currently maps to the large viewport unit, which can hide content when browser UI expands. Dynamic viewport units (`dvh`) are designed to fit the currently visible viewport, with a performance caveat while resizing: https://developer.mozilla.org/en-US/docs/Web/CSS/Reference/Values/length
- web.dev describes `dvh` as the dynamic viewport unit that tracks expanded/retracted browser toolbars and is available across major engines: https://web.dev/blog/viewport-units
- MDN describes `env(safe-area-inset-bottom)` as a way to keep fixed or sticky bottom UI clear of device/browser insets: https://developer.mozilla.org/en-US/docs/Web/CSS/Reference/Values/env
- MDN documents `backdrop-filter` for translucent glass effects, with the requirement that the element/background be transparent or partially transparent: https://developer.mozilla.org/en-US/docs/Web/CSS/Reference/Properties/backdrop-filter

## Implementation

- Replaced the mobile composer stack with a stable one-row grid: action button, textarea, compact send button.
- Added a configurable composer action (`attach` or `plus`) and configurable composer style (`flat`, `glass-soft`, `glass-clear`, `bubble`) persisted through existing Links Notation preferences.
- Kept `flat` as the default composer style for predictable performance.
- Added a composer menu next to the input with Attach files, Export memory, Import memory, and Report issue actions.
- Moved mobile branding into the drawer and kept the mobile topbar focused on a left menu button plus existing action icons.
- Added desktop version display next to the logo/title.
- Added `dvh` and `window.visualViewport`-backed app height handling, plus bottom safe-area padding for mobile browser/device UI.

## Verification

- `node --check src/web/app.js`
- `node --check src/web/i18n.js`
- `cd tests/e2e && npm run test:local -- --grep "Issue #108"`: 4 passed.
- `cd tests/e2e && npm run test:local`: 86 passed.
- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features`
- `cargo test --all-features --verbose`: 266 passed, 69 ignored.
- `cargo test --doc --verbose`: 0 doctests.
- `rust-script scripts/check-file-size.rs`: passed with existing warnings for `src/solver_helpers.rs` and `src/seed.rs` approaching their configured line limits.
