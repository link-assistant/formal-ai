# Issue 6 Case Study: Demo UI Feedback and Diagnostics

## Summary

Issue [#6](https://github.com/link-assistant/formal-ai/issues/6) requests a quieter first-run chat demo: demo mode should be active by default, the wait until the next demo dialog should update every second, and internal diagnostic details should stay hidden unless explicitly enabled.

The implementation keeps the existing static React demo and WebAssembly worker. It changes only the demo shell behavior:

- starts randomized demo mode on page load;
- exposes a visible `Next dialog in Ns` status that changes every second between demo cycles;
- adds a diagnostics toggle, off by default;
- hides trace cards, intent chips, evidence chips, worker mode, and thinking steps until diagnostics are enabled;
- expands E2E coverage so these behaviors are verified in a real browser.

## Collected Data

Raw GitHub and visual evidence is stored in this directory:

- `issue-screenshot.png`: screenshot from the issue body, verified by PNG magic bytes (`89 50 4e 47 ...`).
- `raw-data/issue-6.json`: issue title, body, labels, author, timestamps, and embedded screenshot URL.
- `raw-data/issue-6-comments.json`: issue comments. The issue had no comments at collection time.
- `raw-data/pr-7.json`: prepared PR metadata before implementation.
- `raw-data/pr-7-review-comments.json`: inline PR comments. The PR had none at collection time.
- `raw-data/pr-7-reviews.json`: PR review records. The PR had none at collection time.

## Requirements Extracted

| Requirement | Evidence | Implemented behavior |
| --- | --- | --- |
| Update the timer to next dialog every second. | Issue text: "Timer to next dialog should update every second" | `demoCountdown` state is updated by `setInterval(..., 1000)` and rendered as `Next dialog in Ns`. |
| Turn demo mode on by default. | Issue text: "demo mode should be turned by default" | `demoMode` starts as `true`; composer input is disabled until the user exits demo mode. |
| Make the first screen an interactive demo. | Issue text: "So the first thing user sees is the interactive demo" | Initial messages start empty and the demo effect immediately begins a randomized greeting and hello-world exchange. |
| Show diagnostics only when diagnostics mode is on. | Issue text names `intent:hello_world_typescript` and thinking steps as diagnostics | A new `Diagnostics` toggle gates trace cards, intent chips, evidence chips, worker mode, and thinking steps. |
| Keep diagnostics off by default. | Issue text: diagnostics mode "by default it should be off" | `diagnosticsMode` starts as `false`; E2E tests assert no `.trace-list`, `.intent`, `.evidence-list`, or `.thinking-steps` exist initially. |
| Reduce distractions in the message view. | Issue text: "user should see messages without distractions" | Normal messages render only author, time, and content; diagnostic chips and evidence are removed from the default chat transcript. |
| Collect case-study data under `docs/case-studies/issue-6`. | Issue body explicitly requests this folder | This folder contains raw GitHub data, the issue screenshot, and this analysis. |
| Search online for relevant facts and libraries. | Issue body explicitly requests online research | Sources and component candidates are listed below. |
| Execute everything in one PR. | Issue body requests a single pull request | Work is staged for PR [#7](https://github.com/link-assistant/formal-ai/pull/7). |

## Root Cause

The existing demo already had randomized demo-cycle behavior, but the UI state was optimized for developer inspection rather than a clean default user view:

- `demoMode` started as `false`, so the page opened in manual mode.
- The next-cycle wait label was stored as a one-time string such as `next dialog in 18s`; no state changed while the user waited.
- Trace metadata and message-level intent/evidence chips were rendered unconditionally, so implementation details were visible in the normal chat transcript.
- The worker status (`wasm worker`) was always visible in the top bar.

The underlying chat worker and symbolic classification logic did not need changes.

## Online Research

- [Nielsen Norman Group: Response Times: The 3 Important Limits](https://www.nngroup.com/articles/response-times-3-important-limits/) says delays above 1 second need visible working feedback, and longer waits should indicate when work is expected to finish. This supports a visible countdown instead of a static wait label.
- [MDN: ARIA live regions](https://developer.mozilla.org/en-US/docs/Web/Accessibility/ARIA/ARIA_Live_Regions) describes live regions for dynamic content updates and recommends polite announcements for noncritical updates. This supports rendering the demo countdown in a status region.
- [W3C WAI technique ARIA22](https://www.w3.org/WAI/WCAG21/Techniques/aria/ARIA22) describes `role="status"` as a polite status message pattern. The demo status uses `role="status"`.
- [React `useEffect` documentation](https://react.dev/reference/react/useEffect) shows interval setup and cleanup patterns for state updated over time. The countdown uses an effect cleanup to clear the interval when demo mode turns off.
- [react-aria-live](https://www.npmjs.com/package/react-aria-live) is a known React live-region helper. It was considered unnecessary here because the demo is a single static page and native `role="status"` is enough for this countdown.

## Known Components and Libraries Considered

| Candidate | Fit | Decision |
| --- | --- | --- |
| Native `role="status"` | Small status text that updates without stealing focus | Used. It avoids adding a dependency and matches the static demo style. |
| `react-aria-live` | Cross-component live announcements in larger React apps | Not used. The page only needs one local status region. |
| `@react-aria/live-announcer` | Another live-announcement package | Not used for the same reason: the current need is simpler than a global announcer. |
| Existing `docs/demo` React hooks | Already own demo cycle, pending state, and cleanup | Used. The countdown belongs beside the existing demo effect. |
| `link-assistant/react-chat-ui` | Richer long-term chat UI reference mentioned in earlier issue work | Not pulled in. This issue is a focused behavior change and the existing static demo already has markdown messages, prompts, preview, and worker integration. |

## Solution Plan by Requirement

Timer feedback:

- Replace the static wait string with `demoPhase` and `demoCountdown` state.
- After a demo cycle completes, set `demoCountdown` to the randomized 10-20 second wait.
- Run a one-second interval that decrements the state and starts the next cycle at zero.
- Clear the interval on cleanup so toggling demo mode off does not leave background timers.

Default demo:

- Initialize `demoMode` to `true`.
- Start with an empty transcript and let the demo effect generate the first visible user greeting and assistant reply.
- Keep the existing `Demo on` toggle behavior so users can return to manual chat input.

Diagnostics:

- Add `diagnosticsMode`, initialized to `false`.
- Gate trace cards, worker state, intent chips, evidence chips, and thinking steps behind the diagnostics toggle.
- Prefix visible intent diagnostics with `intent:` so the displayed form matches the issue example.

Message clarity:

- Keep author, timestamp, and markdown content visible in normal mode.
- Leave prompt buttons available, but remove trace metadata from the default sidebar until diagnostics are enabled.

Testing:

- Update Playwright E2E coverage for default demo mode, live countdown changes, default-hidden diagnostics, and diagnostics-toggle behavior.
- Keep existing manual-message tests by switching the demo off before typing into the composer.

## Regression Coverage

Before the fix, the new E2E test failed because the page opened with:

```text
<button type="button" class="mode-toggle" aria-pressed="false">Demo</button>
```

After the fix:

```text
npm run test:local
16 passed
```

The passing tests verify:

- `Demo on` is visible on first load;
- the composer is disabled while demo mode is active;
- `Next dialog in Ns` appears and changes after one second;
- diagnostics are absent by default;
- enabling diagnostics shows trace, `intent:...`, `source:...`, and thinking steps.

## Implementation Notes

The change intentionally does not alter the Rust engine, worker classifier, generated WebAssembly file, or dataset files. The issue is about demo presentation and feedback, so the patch stays in `docs/demo/app.js`, `docs/demo/styles.css`, and `tests/e2e/tests/demo.spec.js`.
