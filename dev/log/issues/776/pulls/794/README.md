# Issue 776 / pull 794 evidence index

Collected on 2026-07-19 UTC for [issue 776](https://github.com/link-assistant/formal-ai/issues/776) and [pull 794](https://github.com/link-assistant/formal-ai/pull/794).

## Contents

- `raw/issue-776*`: issue body, comments, events, and GraphQL timeline.
- `raw/pull-794*`: PR metadata, diff, commits, all three GitHub comment/review streams, events, and timeline.
- `raw/related-*`: related translation issues and merged implementations.
- `raw/github-search-*`: repository-wide GitHub search results used to find related work.
- `raw/ci-*` and `ci-logs/`: the initial Actions run metadata and complete log.
- `ci-logs/ci-cd-29677905727.log` and `artifacts/run-29677905727/`: the complete
  current-head failure log plus the downloaded Playwright report, error
  contexts, screenshots, and traces for every failed/flaky local-web test.
- `ci-logs/version-modification-check-29676805792.log`: the superseded run's
  manual-version-bump failure, which led to restoring the release-managed crate
  version and relying on the patch changelog fragment.
- `ci-logs/lint-and-format-29676916779.log`: the corrected-head run's complete
  lint log; lines 1551–1555 identify 20 lines of worker-mirror regrowth past the
  26,809-line ratchet. The fallback was compacted back to the exact ceiling.
- `ci-logs/agent-cli-e2e-29676916779.log`: the same run's unrelated agent-CLI
  failure (`report request did not execute gh` at line 1833), preserved for the
  subsequent authoritative rerun. The failure reproduced locally because the
  custom OpenCode model omitted its context limits; `raw/agent-cli-issue-687-after.log`
  records the unchanged four-turn scenario passing after declaring those limits.
- `raw/reproduction-before.txt`: exact CLI reproduction of the reported fallback.
- `raw/repro-*-before.log`: failing automated regression evidence.
- `raw/repro-*-after.log`: focused verification after the fix.
- `raw/playwright-issue-776.log`: browser reproduction of the main-thread theme-command collision.
- `raw/playwright-issue-776-after-theme-fix.log`: passing browser verification after both routing layers were corrected.
- `raw/playwright-issue-776-final-after-seed.log`: final browser verification after regenerating the web seed.
- `raw/repro-worker-readiness-before.log`: deterministic pre-fix reproduction
  showing that the composer accepted input while the worker WASM request was
  deliberately held.
- `raw/repro-worker-critical-path-after.log` and
  `raw/playwright-issue-776-final.log`: passing cold-start and issue-776 browser
  regressions after worker readiness became explicit.
- `raw/playwright-issue-747-query-fix-repeat5.log`: five hermetic desktop-search
  fallback passes with every external browser request blocked.
- `raw/playwright-ci-failures-final-repeat5.log`: 65/65 stress passes across
  the two specs containing every test that failed or flaked in run 29677905727.
- `raw/playwright-full-local-final.log`: the complete local-web CI matrix after
  the fix (385 passed, 1 skipped).
- `raw/cargo-test-all-features-clean.log`: corrected full local suite (152 integration, 481 source, and 1,802 unit tests passed; 2 exhaustive tests ignored by default).
- `raw/cargo-{fmt,clippy}-final.log` and `raw/check-*-final.log`: final formatting, lint, file-size, language-policy, and seed-sync checks.
- `online-research.md`: external sources and component evaluation.
- `timeline-and-root-cause.md`: reconstructed event sequence, requirements, and causal analysis.

The initial CI run was green only for change detection and version checking. All substantive lint, Rust test, browser E2E, coverage, and packaging jobs were skipped because the PR contained only `.gitkeep`; it was not evidence that the issue passed.
