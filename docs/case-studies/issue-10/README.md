# Issue 10 Case Study: Demo Feedback Links and Identity Intent

## Summary

Issue [#10](https://github.com/link-assistant/formal-ai/issues/10) requests demo UI cleanup and better feedback capture:

- remove the unused composer Preview button;
- provide a GitHub issue link from unknown-intent responses, prefilled with dialog history and environment metadata;
- make issue reporting available from ordinary dialogs too;
- answer "Who are you?" and close variations with a standard symbolic response;
- collect issue data and analyze possible solutions in this case-study folder.

The implementation keeps the static React demo architecture. It adds native `URLSearchParams`-based GitHub issue links to assistant messages and the header, removes the preview mode state and controls, adds an `identity` symbolic rule to the Rust engine, WebAssembly classifier, JavaScript fallback, and seed data, and extends E2E and unit coverage.

## Collected Data

Raw GitHub, visual, and reference data is stored in this directory:

- `issue-screenshot.png`: screenshot from the issue body, verified by PNG magic bytes (`89 50 4e 47 ...`).
- `raw-data/issue-10.json`: issue title, body, labels, author, timestamps, and screenshot URL.
- `raw-data/issue-10-comments.json`: issue comments. The issue had no comments at collection time.
- `raw-data/pr-11.json`: prepared PR metadata before implementation.
- `raw-data/pr-11-conversation-comments.json`: PR conversation comments. The PR had none at collection time.
- `raw-data/pr-11-review-comments.json`: inline PR review comments. The PR had none at collection time.
- `raw-data/pr-11-reviews.json`: PR review records. The PR had none at collection time.
- `raw-data/pr-branch-runs.json`: recent CI run metadata for the issue branch.
- `raw-data/calculator-reportIssue.ts`: related issue-report URL helper from `link-assistant/calculator`.
- `raw-data/meta-expression-page-report.js`: related page report helper from `link-assistant/meta-expression`.

## Requirements Extracted

| Requirement | Evidence | Implemented behavior |
| --- | --- | --- |
| Remove the Preview button near Send. | Issue text: "I don't see any use of preview button ... it should be removed" | Removed `previewInput`, `.preview-toggle`, `.composer-preview`, and the composer toolbar. |
| Unknown responses should include a prefilled GitHub issue link. | Issue text requests a link with dialog history and metadata | Assistant messages with `intent: unknown` render `Report missing rule`, linking to `/issues/new` with title, body, and `bug` label. |
| Include dialog history and metadata in reports. | Issue text names history, version, and related metadata | Report bodies include version, URL, user agent, worker mode, demo/manual mode, diagnostics state, timestamp, and every dialog message. |
| Allow reporting on any dialog without triggering unknown intent. | Issue text asks for reporting "on any dialog" | Every assistant message gets a report action; the header also has a current-transcript `Report issue` link. |
| Support "Who are you?" variations. | Issue text explicitly names this question | Added `identity` intent for "Who are you?", "What are you?", formal-ai self-description prompts, and "tell me about yourself" variants. |
| Keep symbolic sources reviewable. | Existing data layout stores seed Links Notation | Added `data/seed/identity.lino`. |
| Collect case-study data under `docs/case-studies/issue-10`. | Issue text explicitly requests this folder | This folder contains raw data, the screenshot, related reference files, and this analysis. |
| Search online for relevant facts and libraries. | Issue text explicitly requests online research | Sources and component candidates are listed below. |
| Execute everything in one PR. | Issue text requests a single pull request | Work is staged for PR [#11](https://github.com/link-assistant/formal-ai/pull/11). |

## Root Cause

The demo already had a markdown preview mode, but the issue screenshot shows it as an isolated button above the input rather than a useful part of the current chat workflow. The existing UI also had no feedback path: users could only trigger an unknown response, and that response told them to add a symbolic rule without providing a direct way to report the missing rule or attach context.

The classifier also only recognized greetings and hello-world requests. The natural identity prompt "Who are you?" had no symbolic rule, so it fell through to the unknown intent in the Rust engine, WebAssembly classifier, and JavaScript fallback.

## Online Research

- [GitHub Docs: Creating an issue from a URL query](https://docs.github.com/en/issues/tracking-your-work-with-issues/using-issues/creating-an-issue#creating-an-issue-from-a-url-query) documents `title`, `body`, and `labels` query parameters for `/issues/new`. This supports using a normal link rather than a custom API integration.
- [MDN: `URLSearchParams.toString()`](https://developer.mozilla.org/en-US/docs/Web/API/URLSearchParams/toString) documents browser-native query-string serialization and percent encoding. This supports using `URLSearchParams` for robust prefilled issue URLs.
- [W3C WAI ARIA22: `role="status"`](https://www.w3.org/WAI/WCAG21/Techniques/aria/ARIA22) describes polite status announcements. The existing demo status already follows this pattern and remains unchanged.

## Related Implementations

| Reference | Pattern | Decision |
| --- | --- | --- |
| `link-assistant/calculator` `generateIssueUrl` | Builds a markdown report, serializes it with `URLSearchParams`, and opens GitHub issues with environment metadata | Reused the same native URL-query approach, simplified for this static demo. |
| `link-assistant/meta-expression` `page-report.js` | Generates page-specific reports with environment, current state, reproduction steps, and labels | Reused the page-state/report-section structure for chat transcript reports. |
| Native `URLSearchParams` | Browser standard for encoding long `title`, `body`, and `labels` query strings | Used. No dependency is needed. |
| GitHub Issues query parameters | Standard `/issues/new` prefill route | Used. It works as a plain link and does not require GitHub API tokens in the browser. |
| A new report modal | Could collect a free-form description before leaving the page | Not used. The issue asks for prefilled GitHub links, and a modal would add more UI state without improving the core workflow. |
| A full chat history export dependency | Could serialize richer state | Not used. The transcript is small and already lives in React state. |

## Solution Plan by Requirement

Remove Preview:

- Delete preview state and preview rendering from `docs/demo/app.js`.
- Remove related CSS selectors.
- Replace the previous E2E preview-toggle test with an absence assertion.

Unknown feedback:

- Generate issue URLs with `URLSearchParams`.
- Include `Unknown prompt: ...` titles for unknown assistant messages.
- Render `Report missing rule` inside unknown assistant messages.
- Verify the generated body contains environment metadata, dialog history, the prompt, and `intent: unknown`.

Any-dialog reporting:

- Render `Report issue` actions for non-unknown assistant messages.
- Add a header-level `Report issue` link for the current transcript.
- Keep links as normal anchors so users can open them in a new tab.

Identity intent:

- Add `IDENTITY_ANSWER` and `identity` rule selection to the Rust engine.
- Add the same classifier branch to the no-std WebAssembly source and rebuild the checked-in WASM file.
- Add matching JavaScript fallback behavior for browsers where the WASM worker fails.
- Add reviewable Links Notation seed data in `data/seed/identity.lino`.

Documentation:

- Save raw issue, PR, CI, screenshot, and related reference data.
- Document requirements, root cause, alternatives, solution plan, and regression coverage here.

## Regression Coverage

Before the fix, the new tests failed with:

- Rust unit: `Who are you?` returned `unknown` instead of `identity`.
- Playwright E2E: assistant messages had no `.message-actions` report links.
- Playwright E2E: "Who are you?" showed the unknown response.
- Playwright E2E: `.preview-toggle` still existed.

After the fix, the expected passing checks are:

- `cargo test identity_questions_return_standard_self_description --test unit`
- focused Playwright checks for prefilled issue links, identity response, and preview removal
- full `cargo test`
- full `npm run test:local` in `tests/e2e`
