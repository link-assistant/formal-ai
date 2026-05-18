# Issue 112 Mobile UI Case Study

## Source Material

- Issue: <https://github.com/link-assistant/formal-ai/issues/112>
- Pull request: <https://github.com/link-assistant/formal-ai/pull/113>
- Submitted screenshot: [screenshots/mobile-screenshot.png](screenshots/mobile-screenshot.png)
- After screenshot, full-width mobile drawer: [screenshots/after-mobile-drawer.png](screenshots/after-mobile-drawer.png)
- After screenshot, auto-growing mobile composer: [screenshots/after-mobile-composer.png](screenshots/after-mobile-composer.png)
- Raw GitHub issue data: [raw-data/issue-112.json](raw-data/issue-112.json)
- Raw issue comments: [raw-data/issue-112-comments.json](raw-data/issue-112-comments.json)
- Raw PR metadata and comments: [raw-data/pr-113.json](raw-data/pr-113.json), [raw-data/pr-113-conversation-comments.json](raw-data/pr-113-conversation-comments.json), [raw-data/pr-113-review-comments.json](raw-data/pr-113-review-comments.json), [raw-data/pr-113-reviews.json](raw-data/pr-113-reviews.json)
- E2e dependency installation log: [raw-data/npm-ci-e2e.log](raw-data/npm-ci-e2e.log)
- Local verification log: [raw-data/issue-112-e2e.log](raw-data/issue-112-e2e.log)
- Full e2e regression log: [raw-data/e2e-full.log](raw-data/e2e-full.log)
- Rust verification logs: [raw-data/cargo-fmt.log](raw-data/cargo-fmt.log), [raw-data/cargo-clippy.log](raw-data/cargo-clippy.log), [raw-data/cargo-test.log](raw-data/cargo-test.log), [raw-data/check-file-size.log](raw-data/check-file-size.log)

The screenshot was downloaded from the GitHub issue attachment and verified as a PNG before visual inspection. At the time of collection there were no issue comments, PR conversation comments, PR inline review comments, or PR reviews.

## Timeline

1. The issue reported a mobile Safari chat view with a focused composer, a visible iOS keyboard accessory bar, clipped text inside the textarea, and an off-center menu glyph.
2. The issue scope was expanded beyond the visible screenshot to include drawer behavior, conversation soft delete, tool registry localization, complete supported tool listing, complete supported examples, and a case-study artifact.
3. The current app state was inspected in `src/web/app.js`, `src/web/styles.css`, `src/web/i18n.js`, `src/web/seed_loader.js`, `data/seed/tools.lino`, and the e2e test suite.
4. Failing regression coverage was added to assert the requested mobile drawer, composer, conversation deletion, tool registry, localization, and example prompt behavior.
5. The implementation was updated in the web app, seed parser, seed data, translations, and styles.

## Requirements And Root Causes

### iOS Form Accessory Bar

Requirement: If possible, disable the iOS form filling up/down plus Done panel because the chat composer is not a multi-field form.

Root cause: The visible bar is user-agent and platform UI shown by iOS Safari when a text control has focus. The app can influence keyboard hints but does not have a reliable standards-based switch to remove that accessory UI from Safari.

Research:

- WHATWG HTML defines `enterkeyhint` as a hint for the action label or icon on virtual keyboard enter keys, including the `send` value used by chat inputs: <https://html.spec.whatwg.org/dev/interaction.html#input-modalities:-the-enterkeyhint-attribute>
- MDN documents that `enterkeyhint` customizes the virtual keyboard enter key presentation for form controls such as `<textarea>`: <https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Global_attributes/enterkeyhint>
- MDN documents `VisualViewport` as the API for handling layout changes when mobile browser UI such as the on-screen keyboard shrinks the visible viewport: <https://developer.mozilla.org/en-US/docs/Web/API/VisualViewport>
- W3C VirtualKeyboard API and MDN describe layout adaptation and keyboard geometry/inset APIs, not removal of iOS Safari's accessory toolbar: <https://www.w3.org/TR/virtual-keyboard/> and <https://developer.mozilla.org/en-US/docs/Web/API/VirtualKeyboard_API>
- WebKit tracks the iOS accessory bar as browser behavior, for example in bugs around next/previous button behavior: <https://bugs.webkit.org/show_bug.cgi?id=203292>

Solution: Add practical, standards-backed hints to the composer (`autocomplete="off"`, `autocorrect="off"`, `enterkeyhint="send"`, `inputmode="text"`) and keep the app anchored to the visual viewport. No upstream issue was opened because this is not a formal-ai defect and the platform already exposes the relevant behavior through WebKit/browser channels.

### Composer Text Clipping

Requirement: The composer text should auto-resize to content, use balanced padding, and the input section should never exceed 50% of the chat space.

Root cause: The textarea had a fixed mobile height and multi-row configuration. With Safari's focused viewport and text metrics, the visible content area clipped the top line while showing the last lines.

Solution: The textarea now starts at one row, auto-resizes to `scrollHeight`, uses equal padding, and caps the composer and textarea height using the visual viewport CSS variable. Overflow becomes scrollable only after the cap is reached.

### Menu Glyph Alignment

Requirement: Center the mobile menu symbol.

Root cause: The hamburger was a text glyph. Font metrics vary between platforms, so the symbol appeared visually off-center even when the button itself was aligned.

Solution: Replace the text glyph with a small CSS-drawn icon using fixed dimensions and pseudo-elements. The same component renders the close glyph in the drawer.

### Mobile Drawer Width And Actions

Requirement: The mobile menu should open to 100% width, and all topbar buttons should appear in a `Menu` section before conversations.

Root cause: The drawer used a constrained width from earlier mobile work and only contained the sidebar sections. The topbar actions were hidden or icon-only on mobile and had no equivalent full-label drawer placement.

Solution: The mobile drawer now uses the full visual viewport width and includes a `Menu` section before the conversation list with issue reporting, memory export/import, diagnostics, chat/agent mode, and demo mode actions.

### Soft Delete Conversations

Requirement: Deleting a conversation should mark it deleted, hide it by default, and allow viewing deleted dialogs separately.

Root cause: Conversations were only derived from message events. There was no append-only deletion marker and no projection that separated active and deleted threads.

Solution: Add a `conversation_deleted` memory event. The conversation projection hides deleted entries by default, exposes a deleted-only toggle, preserves the message history, and clears the active thread if the current conversation is deleted.

### Tool Description Localization

Requirement: Tool descriptions should be fully translated when Russian or another non-English UI language is selected.

Root cause: The seed loader only extracted the base English tool fields, and the tool registry renderer used those fields directly.

Solution: Seed data now includes localized tool name and description entries for Russian, Chinese, and Hindi. The seed loader parses those localized entries, and the renderer selects the UI-language version with English fallback.

### Complete Tool List

Requirement: All supported tools should be listed.

Root cause: The tool seed contained only a subset of the tools exercised by the demo and examples.

Solution: The canonical tool seed now lists the supported browser/demo tools, including fetch, search, lookup, calculator, JavaScript evaluation, local file read, memory import/export/append, conversation recall, intent routing, fact lookup, summarization, brainstorming, coreference, and roleplay.

### Complete Example List

Requirement: All supported examples should appear in examples.

Root cause: The example prompt list lagged behind the response/tool families supported in the code and seed data.

Solution: The prompt examples now cover greeting, farewell, identity, clarification, capabilities, calculation, concept lookup, summarization, brainstorming, fact Q&A, coreference, roleplay, recall, export memory, and import memory.

## Solution Plan Executed

1. Add e2e coverage for issue #112 behavior before implementing the fix.
2. Keep conversation deletion append-only by adding a deletion event instead of mutating or removing existing message events.
3. Localize tool data at the seed layer so the UI, exports, and any future seed consumers can share the same structured data.
4. Use native textarea behavior with measured auto-resize rather than replacing the composer with a custom editor.
5. Use viewport APIs already present in the app to keep mobile sizing tied to the actual visible viewport.
6. Preserve raw issue/PR data and screenshots in this case-study directory.

## Verification Notes

- JavaScript syntax was checked with `node --check` against the changed web app, i18n, seed loader, and e2e spec files.
- The canonical seed was synced with `scripts/sync-seed.sh`.
- The first local targeted e2e attempt failed before running tests because local e2e dependencies were not yet installed (`@playwright/test` missing). The dependencies were installed with `npm ci`, and the final targeted e2e run in [raw-data/issue-112-e2e.log](raw-data/issue-112-e2e.log) passed all five issue #112 tests.
- `cargo fmt --all -- --check`, `rust-script scripts/check-file-size.rs`, `cargo clippy --all-targets --all-features`, and `cargo test` passed.
- The full local Playwright suite passed with 92 tests in [raw-data/e2e-full.log](raw-data/e2e-full.log).

Additional verification logs and screenshots are added to this directory as the PR is finalized.
