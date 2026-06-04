# Case study - Issue #388: topbar fit and dark-theme parity

## Summary

Issue #388 reported two visible UI regressions in the web interface:

- The desktop topbar action buttons kept full text labels at a reported 1824 x 1115 viewport, crowding the brand, status text, and action row.
- Some surfaces in the dark UI still looked light or lacked explicit dark-theme coverage, especially around diagnostic/detail-style content and newer controls.

This PR addresses both problems with a component-sized topbar container query, expanded dark-theme selectors, and focused Playwright coverage for the reported viewport and dark-mode surfaces.

## Archived data

- `raw-data/issue-388.json` - issue metadata and body captured with `gh issue view`.
- `raw-data/issue-388-comments.json` - issue comments captured with the paginated GitHub API.
- `raw-data/pr-389.json` - existing PR metadata.
- `raw-data/pr-389-review-comments.json` - PR inline review comments captured with the paginated GitHub API.
- `raw-data/pr-389-conversation-comments.json` - PR conversation comments captured with the paginated GitHub API.
- `raw-data/pr-389-reviews.json` - PR review records captured with the paginated GitHub API.
- `raw-data/issue-388-before-e2e.log` - focused e2e reproduction run before the fix.
- `raw-data/issue-388-after-e2e.log` - focused e2e verification run after the fix.
- `raw-data/build-web.log` - final `bun run build:web` output.
- `raw-data/sync-seed.log` - final `scripts/sync-seed.sh` output.
- `raw-data/check-web-tdz.log` - final web temporal-dead-zone guard output.
- `raw-data/diff-check.log` - final `git diff --check` output.
- `screenshots/reported-ui.png` - original issue screenshot. The environment did not provide `file`, so the download was validated with the PNG magic bytes `89 50 4e 47 0d 0a 1a 0a` before image inspection.
- `screenshots/after-dark-ui.png` - after screenshot captured at the reported viewport with dark mode and diagnostics open.

## Timeline

- 2026-06-04T15:17:59Z - Issue #388 opened by `konard` with the screenshot and requirements for topbar fit, dark-theme parity, e2e tests, and case-study documentation.
- 2026-06-04T15:18:45Z - Draft PR #389 opened from `issue-388-874e3e436c91`.
- 2026-06-04 - Issue, comments, PR metadata, PR comments, PR reviews, and the reported screenshot were archived under this directory.
- 2026-06-04 - A focused Playwright reproduction was added. Before the CSS fix, `npm run --prefix tests/e2e test:local -- --grep "Issue #388"` failed because `.topbar-actions .btn-label` remained visible at 1824px.
- 2026-06-04 - The CSS fix and dark-theme audit coverage were implemented.
- 2026-06-04 - The same focused Playwright command passed with three tests.

## Requirements

- Make the top buttons fit at desktop widths where the available topbar space is already constrained.
- Collapse topbar actions to icon-only controls sooner, while retaining accessible names.
- Recheck visible UI elements in dark mode and add explicit dark-theme coverage where selectors rely on light defaults.
- Add real e2e tests that cover the reported regression class.
- Preserve issue/PR data, logs, screenshots, requirements, research, root-cause notes, and verification artifacts in this repository.
- Add tracing only if root-cause data is insufficient.
- Report upstream only if the issue is caused by a dependency or browser bug.

## Root causes

The topbar used viewport media queries for label collapse. The existing `max-width: 1500px` breakpoint was too late for the reported desktop screenshot because the actual component had to share space with the brand, version, mode status, demo status, diagnostics state, agent state, and seven action controls. The buttons already had `aria-label` and `title` attributes, so hiding the visible `.btn-label` text earlier was safe for icon-only operation.

The dark-theme issue was a selector coverage gap. Newer or nested surfaces such as diagnostics details, diagnostic payload/reasoning blocks, conversation copy buttons, and settings reset controls needed explicit dark-mode rules in both the manual `[data-theme="dark"]` path and the system dark `prefers-color-scheme` fallback path. Without broad e2e coverage, these gaps were easy to miss when only primary panels were checked.

No upstream dependency bug was identified. The fix is local CSS and test coverage.

## Solution

- Added `container: formal-ai-topbar / inline-size` to `.topbar`.
- Added a `@container formal-ai-topbar (max-width: 1900px)` rule that hides `.topbar-actions .btn-label`, tightens action gaps, and keeps the icon controls compact before the reported width clips.
- Preserved accessible icon-only controls by relying on existing `aria-label` and `title` attributes.
- Added explicit dark styles for diagnostics details, diagnostics body sections, payload blocks, reasoning blocks, settings reset controls, and conversation copy hover states.
- Mirrored those dark rules into the system dark fallback so users who rely on OS theme preferences get the same surface coverage.
- Added `tests/e2e/tests/issue-388.spec.js` and wired it into the local Playwright config.

## E2e coverage

The new Playwright tests cover:

- A 1824 x 1115 viewport, matching the reported screenshot dimensions.
- Icon-only topbar action labels at that viewport.
- No topbar action overflow and no visible child outside the action row.
- Non-empty accessible names and at least 24 x 24 CSS pixel visible action targets.
- Dark-mode diagnostics/settings/sidebar/composer surfaces using computed background luminance checks.
- Basic text contrast on representative dark diagnostic and reset surfaces.

## Research notes

- CSS container queries were a better fit than another viewport-only breakpoint because the problem is about the available inline size of the topbar component, not the whole browser viewport: https://developer.mozilla.org/en-US/docs/Web/CSS/CSS_containment/Container_queries
- The dark-theme fallback follows the `prefers-color-scheme` model already used by the stylesheet: https://developer.mozilla.org/en-US/docs/Web/CSS/@media/prefers-color-scheme
- The stylesheet keeps using `color-scheme` to let browser-native controls match the active theme: https://developer.mozilla.org/en-US/docs/Web/CSS/color-scheme
- Icon-only buttons retain accessible names through `aria-label`: https://developer.mozilla.org/en-US/docs/Web/Accessibility/ARIA/Attributes/aria-label
- The e2e target-size assertion uses the WCAG 2.2 24 x 24 CSS pixel target-size threshold as the minimum floor for visible controls: https://www.w3.org/WAI/WCAG22/Understanding/target-size-minimum.html

## Verification

- `npm run --prefix tests/e2e test:local -- --grep "Issue #388"` - passed.
- `bun run build:web` - passed.
- `scripts/sync-seed.sh` - passed after the web build.
- `npm run --prefix tests/e2e check:web-tdz` - passed.
- `git diff --check` - passed.

The full focused e2e output is archived in `raw-data/issue-388-after-e2e.log`.
