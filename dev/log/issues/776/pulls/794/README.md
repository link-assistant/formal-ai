# Issue 776 / pull 794 evidence index

Collected on 2026-07-19 UTC for [issue 776](https://github.com/link-assistant/formal-ai/issues/776) and [pull 794](https://github.com/link-assistant/formal-ai/pull/794).

## Contents

- `raw/issue-776*`: issue body, comments, events, and GraphQL timeline.
- `raw/pull-794*`: PR metadata, diff, commits, all three GitHub comment/review streams, events, and timeline.
- `raw/related-*`: related translation issues and merged implementations.
- `raw/github-search-*`: repository-wide GitHub search results used to find related work.
- `raw/ci-*` and `ci-logs/`: the initial Actions run metadata and complete log.
- `raw/reproduction-before.txt`: exact CLI reproduction of the reported fallback.
- `raw/repro-*-before.log`: failing automated regression evidence.
- `raw/repro-*-after.log`: focused verification after the fix.
- `raw/playwright-issue-776.log`: browser reproduction of the main-thread theme-command collision.
- `raw/playwright-issue-776-after-theme-fix.log`: passing browser verification after both routing layers were corrected.
- `raw/playwright-issue-776-final-after-seed.log`: final browser verification after regenerating the web seed.
- `raw/cargo-test-all-features-clean.log`: corrected full local suite (152 integration, 481 source, and 1,802 unit tests passed; 2 exhaustive tests ignored by default).
- `raw/cargo-{fmt,clippy}-final.log` and `raw/check-*-final.log`: final formatting, lint, file-size, language-policy, and seed-sync checks.
- `online-research.md`: external sources and component evaluation.
- `timeline-and-root-cause.md`: reconstructed event sequence, requirements, and causal analysis.

The initial CI run was green only for change detection and version checking. All substantive lint, Rust test, browser E2E, coverage, and packaging jobs were skipped because the PR contained only `.gitkeep`; it was not evidence that the issue passed.
