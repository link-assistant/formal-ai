# Issue 94: UI/UX Improvements

## Inputs

- GitHub issue: https://github.com/link-assistant/formal-ai/issues/94
- Pull request: https://github.com/link-assistant/formal-ai/pull/95
- Issue comments: none when checked on 2026-05-16.
- Screenshots from the issue:
  - `screenshots/current-wide-controls.png`
  - `screenshots/current-crowded-controls.png`
- Screenshot from the implemented UI:
  - `screenshots/after-dark-ru-980.png`
- Raw GitHub and library data:
  - `raw-data/issue-94.json`
  - `raw-data/issue-94-comments.json`
  - `raw-data/pr-95.json`
  - `raw-data/pr-95-review-comments.json`
  - `raw-data/pr-95-conversation-comments.json`
  - `raw-data/pr-95-reviews.json`
  - `raw-data/lino-i18n-README.md`
  - `raw-data/lino-i18n-package.json`
  - `raw-data/lino-i18n-src-tree.json`
  - `raw-data/lino-i18n-index.js`
  - `raw-data/lino-i18n-index.d.ts`
  - `raw-data/i18n-library-npm-metadata.json`

## Requirements

1. Detect the user's light/dark theme automatically.
2. Detect the UI language automatically and support English, Russian, Chinese, and Hindi.
3. Send useful user context into both agent requests and prefilled GitHub issue reports.
4. Use `link-foundation/lino-i18n` if possible, and report missing upstream features if it is not possible.
5. Switch toolbar buttons to icon-only earlier, before labels wrap or force the action row to break.
6. Preserve the existing static web app shape and the deterministic local-worker behavior.

## Research Notes

- MDN documents `prefers-color-scheme` as the browser-facing CSS media feature for detecting a user's requested light or dark theme: https://developer.mozilla.org/docs/Web/CSS/Reference/At-rules/%40media/prefers-color-scheme
- MDN documents `navigator.languages` as an ordered list of preferred language tags, with `navigator.language` as the first entry: https://developer.mozilla.org/en-US/docs/Web/API/Navigator/languages
- Exact browser geolocation is inappropriate for an automatic issue-report context because `getCurrentPosition()` requires a secure context, can be blocked by policy, and requires explicit user permission: https://developer.mozilla.org/en-US/docs/Web/API/Geolocation/getCurrentPosition
- `link-foundation/lino-i18n` exists and describes itself as "internationalization done in Links Notation", but on 2026-05-16 it still had no releases and its source/package files were still template/example code: https://github.com/link-foundation/lino-i18n
- Upstream `lino-i18n` issue #1 already tracks the first real Links Notation i18n library and comparison work against i18next, i18n-js, and react-intl: https://github.com/link-foundation/lino-i18n/issues/1
- I added a formal-ai integration note to that upstream issue with the missing runtime/API requirements: https://github.com/link-foundation/lino-i18n/issues/1#issuecomment-4467551335

## Library Decision

`lino-i18n` is the desired direction, but it is not currently consumable by this static app: there is no stable runtime API, browser script export, or Links Notation catalog parser. Adding i18next/react-intl/i18n-js now would introduce a second catalog format and dependency model before the Links Notation library exists.

The implemented compromise is a tiny browser script, `src/web/i18n.js`, with a Links-Notation-style key namespace and the same future call shape the app needs: language negotiation, fallback, and interpolation. That keeps the UI shippable now and leaves a narrow replacement point for a real `lino-i18n` package later.

## Implementation Plan

1. Add failing e2e coverage for dark theme detection, Russian language auto-detection, required dictionaries, user context in reports, and early icon-only toolbar behavior.
2. Add a static `FormalAiI18n` browser module with dictionaries for `en`, `ru`, `zh`, and `hi`.
3. Load the i18n module before the app, detect the UI language from preferences or `navigator.languages`, and set `html[lang]`.
4. Localize the topbar, status, sidebar headings, composer, and report action labels while preserving English default text for existing tests.
5. Add no-permission user context to issue reports, memory exports, and worker requests.
6. Attach selected context values to worker evidence/thinking steps so diagnostics and agent state have the same environment signal.
7. Add `prefers-color-scheme: dark` CSS overrides and compact the topbar at 1100px before labels can wrap.

## Verification

- Red test log before the fix: `experiments/issue-94-red-tests.log`
- Green targeted test log after the fix: `experiments/issue-94-green-tests.log`
- Targeted command:

```sh
npm run test:local -- --grep "Issue #94"
```

Result: 5 passed.

- Full e2e suite:

```sh
npm run test:local
```

Result: 75 passed.

- Visual verification:
  - `screenshots/after-dark-ru-980.png` captures the automatic dark theme, Russian UI auto-detection, and compact icon-only toolbar at 980px width.

## Follow-Up

- Replace `src/web/i18n.js` with `lino-i18n` once the upstream package ships a browser-safe Links Notation catalog API.
- Consider a visible language selector only after explicit language preference is requested; Issue 94 asked for auto-detection, so no new control was added.
- Keep exact geolocation out of automatic reports unless a future issue explicitly asks for an opt-in permission flow.
