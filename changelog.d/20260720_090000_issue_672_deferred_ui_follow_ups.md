---
bump: minor
---

### Added

- A per-message control that reveals a withheld answer immediately (issue #672,
  F3). The animation budget stays a global preference, but a single message can
  be settled without changing it for every future answer. Reduced-motion users
  are unaffected: they never see the control because there is nothing to skip.
- Reasoning-step hierarchy editing in Diagnostics mode (issue #672, F4).
  Right-clicking a step offers "Bump to high level" / "Demote to sub-step" /
  "Restore the original level" in all four locales. Edits are appended to an
  event log and the visible hierarchy is a projection of it, so the trace the
  solver reported is never rewritten — each step keeps the solver's own label on
  `data-solver-level` beside the user's `data-level-override`.
- A desktop notice for profile migrations, with a replay button (issue #672,
  F2). `dataMigrationStatus` and `replayDataMigration` carry the result of the
  startup migration to the renderer, which names the profile the data came from
  and what moved. Failures surface as errors rather than as claimed successes,
  and a clean install is never interrupted.

### Changed

- The desktop profile migration now also copies `Cookies`, `Service Worker`,
  `WebStorage`, and `WebSocketStorage` (`DATA_VERSION` 2, issue #672 F2), so a
  user whose data moved to the pinned profile no longer arrives logged out. An
  existing v1 profile is topped up with only the new subtrees. `Cache` and
  `Code Cache` are deliberately excluded — they are derived, large, and unsafe
  to carry between Chromium builds.

- The per-message UI strings moved out of `src/web/i18n-catalog.lino` into
  `src/web/i18n-catalog-messages.lino`, the same way the permission strings were
  split earlier, so each catalog stays under the Links Notation line limit that
  the F3/F4 keys pushed the single file past. The loader merges all three files
  per locale and the catalog, parity, and coverage guards all watch the new file.

### Fixed

- `desktop/scripts/*.test.mjs` was written but never executed by any workflow.
  The Lint job now runs it, so the profile-migration code has a gate.

### Tests

- `tests/e2e/tests/issue-672-theme-snapshots.spec.js` (issue #672, F1) snapshots
  the computed colours of the five widgets issue #541's R1 fixed, across
  light/dark/auto and both the web and desktop surfaces, and runs in CI. Three of
  those widgets previously had no automated theme coverage at all.
- `tests/e2e/tests/issue-541-permissions-cold-start.spec.js` (issue #672, F5)
  re-runs the #541 R9 grant-all journey with the desktop provider behind a real
  `page.exposeFunction` boundary, so the replayed task and the granted tools are
  asserted on the payloads that actually left the browser context.
- The F3 control and the F4 menu are asserted per supported language (en, ru,
  zh, hi) on the labels the browser renders, and each of those tests performs
  the edit, so a locale cannot be labelled correctly and broken behaviourally.
