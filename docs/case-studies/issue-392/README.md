# Case study - Issue #392: conversation copy as Markdown does not work

Raw artifacts for this study live in [`raw-data/`](./raw-data/):
[`issue-392.json`](./raw-data/issue-392.json),
[`issue-392-comments.json`](./raw-data/issue-392-comments.json),
[`pr-393.json`](./raw-data/pr-393.json),
[`pr-393-conversation-comments.json`](./raw-data/pr-393-conversation-comments.json),
[`pr-393-review-comments.json`](./raw-data/pr-393-review-comments.json),
[`pr-393-reviews.json`](./raw-data/pr-393-reviews.json), and
[`ci-runs-initial.json`](./raw-data/ci-runs-initial.json).

## Summary

The conversation-list "Copy" button serialized the conversation only after
awaiting an IndexedDB read. Code-block copy and message Markdown copy already
had their copy payload available synchronously, so they invoked
`navigator.clipboard.writeText()` during the click handler. The conversation
button did not, which made it vulnerable to browsers that require clipboard
writes to happen during a transient user activation window unless a broader
clipboard-write permission is already granted. The existing e2e tests granted
clipboard permissions, so they covered the Markdown payload but masked the
activation-sensitive failure mode.

## Timeline

- 2026-06-04 13:25 UTC - PR #387 merged the conversation-level Markdown copy
  feature while testing it with explicit clipboard permissions.
- 2026-06-04 16:18 UTC - Issue #392 was opened: "Copy as markdown for
  conversations does not work".
- 2026-06-04 16:38 UTC - Draft PR #393 was opened for this issue.
- This PR archived the issue/PR data, reproduced the failure with a
  permissionless clipboard shim, and fixed the handler to copy from cached
  conversation events before doing any asynchronous memory refresh.

## Requirements

- **R1 - Fix conversation copy.** Clicking the conversation-list copy button
  must copy the whole conversation as Markdown and visibly confirm success.
- **R2 - Test every copy action.** Code-block copy, whole-message Markdown copy,
  and whole-conversation Markdown copy must all have e2e coverage.
- **R3 - Preserve issue data and case-study analysis.** Store issue/PR/CI data
  under `docs/case-studies/issue-392/` and document timeline, requirements,
  root cause, and solution options.
- **R4 - Search relevant external facts.** Use current web-platform references
  to verify clipboard constraints.
- **R5 - Add debug output only if needed.** If root cause cannot be found, add
  disabled-by-default tracing for the next iteration.
- **R6 - Report upstream only if external.** If an upstream library or browser
  defect is found, file a reproducible issue there.

## Root Cause

`handleCopyConversation()` awaited `window.FormalAiMemory.listEvents()` before
calling `copyTextToClipboard(markdown)`. That IndexedDB read moved the actual
clipboard write away from the direct click-handling path. The W3C Clipboard API
specification gates `writeText()` through the clipboard-write permission check,
which succeeds when a write-without-gesture permission is granted or when the
relevant global object has transient activation. The HTML standard exposes that
state through `navigator.userActivation.isActive`; MDN summarizes transient
activation as a short-lived user-interaction state that can expire or be
consumed.

The old tests used `context.grantPermissions(['clipboard-read',
'clipboard-write'])`. That made `writeText()` succeed even after the async
memory read, so the tests did not model a normal user session where no explicit
clipboard-write grant exists.

## Implemented Fix

The conversations sidebar already refreshes from `FormalAiMemory.listEvents()`
whenever it needs to show current entries. This PR stores that latest event list
in `conversationEventsRef`. The copy handler now serializes from that cached
projection and calls `copyTextToClipboard()` immediately in response to the
click. After a successful copy it refreshes the sidebar asynchronously, which
keeps the cache fresh without delaying the protected clipboard write.

This keeps the existing `copyTextToClipboard()` helper shared across all copy
actions. No external component or new dependency is needed; the platform
Clipboard API plus the existing `execCommand("copy")` fallback remain the right
implementation for this small browser-only copy surface.

## Test Coverage

`tests/e2e/tests/issue-392.spec.js` installs a small activation-bound clipboard
shim instead of granting clipboard permissions. It sends a real prompt through
the web app and verifies:

- the code-block button copies raw code without Markdown fences;
- the message Markdown button copies the full rendered message Markdown;
- the conversation button copies the title, user turn, assistant turn, and code
  fence as one Markdown document.

The new spec is included in `tests/e2e/playwright.local.config.js`.

## External Research

- [W3C Clipboard API and events](https://www.w3.org/TR/clipboard-apis/) -
  `writeText()` runs the clipboard write permission check; write permission can
  depend on transient activation unless a broader permission is granted.
- [WHATWG HTML Standard - user activation](https://html.spec.whatwg.org/multipage/interaction.html#tracking-user-activation)
  - `navigator.userActivation.isActive` reports transient activation.
- [MDN transient activation](https://developer.mozilla.org/en-US/docs/Glossary/Transient_activation)
  - summarizes transient activation as a short-lived state created by meaningful
  user interaction.
- [MDN Clipboard.writeText](https://developer.mozilla.org/en-US/docs/Web/API/Clipboard/writeText)
  - documents secure-context constraints and the click-handler usage pattern.

## Upstream and Debugging Notes

No upstream issue was filed: this is local application timing, not a defect in
React, Playwright, IndexedDB, or the Clipboard API. Extra debug output was not
added because the activation-sensitive test reproduces the failure and the
cached-copy change fixes it directly.
