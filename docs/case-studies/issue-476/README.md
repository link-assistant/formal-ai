# Issue 476 Case Study

Date: 2026-06-27

Issue: https://github.com/link-assistant/formal-ai/issues/476
Pull request: https://github.com/link-assistant/formal-ai/pull/578

## Raw Data

- `raw-data/issue-476.json`
- `raw-data/issue-476-comments.json`
- `raw-data/pr-578.json`
- `raw-data/pr-578-conversation-comments.json`
- `raw-data/pr-578-review-comments.json`
- `raw-data/pr-578-reviews.json`
- `raw-data/related-merged-prs.json`
- `raw-data/online-research.md`

## Requirements

- Add a right-side button to every sidebar expandable section.
- Clicking that button expands only that section and collapses all other sidebar sections.
- Use an appropriate icon; a fullscreen-style icon is acceptable.
- Preserve the normal header click behavior: it toggles only that section.
- On first run, only Conversations and Example prompts should start expanded.
- Shift+Click on a section title should perform the same "expand only this section" action.
- Keep issue research and analysis in `docs/case-studies/issue-476`.

## Existing System

The sidebar already uses a local `CollapsibleSection` component with persisted collapsed state for the main sections. Section bodies flex-share the remaining height when expanded, matching the behavior introduced for the earlier sidebar accordion work.

The relevant implementation surface is:

- `src/web/app/main.jsx`: sidebar state, `CollapsibleSection`, icon action maps, and the sidebar render tree.
- `src/web/styles.css`: header and body layout for the collapsible sections.
- `src/web/i18n-catalog.lino`: localized UI labels and titles.
- `tests/e2e/tests/multilingual.spec.js` and related e2e specs: assumptions about which sidebar sections are initially open.

## Research Summary

The WAI-ARIA accordion pattern keeps a focusable button as the expand/collapse control and uses `aria-expanded` to reflect state. It also allows persistent controls adjacent to the accordion header, which supports a separate right-side icon action without changing the existing section toggle.

MDN's `aria-expanded` guidance confirms that the expanded state belongs on the controlling focusable element. The implementation preserves this on the normal section toggle button.

Fullscreen/maximize/open-in-full iconography is a common expand-region metaphor. The project already has a `ToolbarIcon` abstraction with several icon pack names and local SVG fallbacks, so adding a new `isolateSection` icon action was preferred over adding a dependency.

## Options Considered

1. Make normal header click behave like exclusive accordion expansion.
   Rejected because the issue explicitly says normal clicks should continue to expand/collapse only the clicked section.

2. Add an explicit right-side action and centralize isolate behavior at the sidebar.
   Selected. It keeps existing header behavior intact, gives mouse users a clear control, and makes Shift+Click share the same code path.

3. Replace the local sidebar with a library accordion.
   Rejected. The current component already handles the app-specific flex layout, persistence, localization, and test IDs; a library would add risk without solving the specific issue better.

## Implementation Plan

- Add a reproducing Playwright spec for first-run defaults, normal toggle behavior, right-side isolate clicks, and Shift+Click isolation.
- Change first-run collapsed defaults so only Conversations and Example prompts start open.
- Render a right-side isolate button inside `CollapsibleSection`.
- Add a sidebar capture handler that detects isolate-button clicks and Shift+Clicks on section headers, then collapses every other known sidebar section.
- Add localized accessible labels/titles for the icon-only action.
- Update existing tests that inspect Menu, Settings, or Tools to explicitly open those sections first.

## Verification

- `npm run --prefix tests/e2e check:i18n`
- `npm run --prefix tests/e2e check:web-tdz`
- `npm run --prefix tests/e2e check:web-hardcoded-ui`
- `npm run build:web`
- `git diff --check`
- `npm run --prefix tests/e2e test:local -- tests/issue-476.spec.js --workers=1`
- `npm run --prefix tests/e2e test:local -- tests/issue-153.spec.js tests/demo.spec.js tests/multilingual.spec.js --grep "left menu actions section|priority-based topbar overflow|tool cards fit|settings sidebar exposes|Tool registry surfaces|mobile drawer lists topbar actions|collapsing a section gives" --workers=1`
