# Issue 190 Case Study

## Scope

Issue: <https://github.com/link-assistant/formal-ai/issues/190>

Pull request: <https://github.com/link-assistant/formal-ai/pull/191>

The report combined five user-visible defects:

- The browser worker answered `Переведи "как у тебя дела?" на английский.` with the unknown fallback.
- The follow-up `Что ещё ты умеешь?` ignored current chat history and repeated already covered web-search details.
- The left `MENU` action group consumed too much vertical space because it could not collapse like the other sidebar groups.
- The issue-report URL fitter often dropped every dialog turn except the last two instead of keeping as much preceding context as possible.
- The issue report body still used `## Dialog` and a low-value `## Reproduction Steps` section.

## Local Evidence

Downloaded artifacts are stored with this case study:

- `raw-data/issue-190.json`: issue title, body, author, labels, and timestamps.
- `raw-data/issue-190-comments.json`: issue comments; empty when collected.
- `raw-data/pr-191.json`: initial PR metadata.
- `raw-data/pr-191-review-comments.json` and `raw-data/pr-191-reviews.json`: PR review context; empty when collected.
- `raw-data/pr-branch-runs.json`: recent CI runs for the prepared branch.
- `raw-data/recent-merged-prs.json`: recent merged PRs used for style comparison.
- `screenshots/menu-wide.png`: original issue screenshot showing the expanded `MENU` section and the capabilities follow-up.
- `screenshots/menu-collapsed-after.png`: local Playwright verification screenshot showing the final collapsible `MENU` state.

## Online Research

GitHub supports prefilled issue creation through URL query parameters. The official documentation page is:

<https://docs.github.com/en/github/managing-your-work-on-github/about-automation-for-issues-and-pull-requests-with-query-parameters>

This matters because the demo generates `issues/new?title=...&body=...&labels=...` links. The URL fitter must keep the generated link under GitHub's practical URL length cap while preserving enough dialog context for maintainers to reproduce the bug.

## Timeline

- 2026-05-20: Issue 190 reported against formal-ai v0.82.0 from the GitHub Pages demo.
- 2026-05-20: Issue and PR data were downloaded into this directory.
- 2026-05-20: The screenshot was downloaded with authenticated `curl`; PNG magic bytes were verified before visual inspection.
- 2026-05-20: Focused Rust tests were added for the Russian translation prompt and the Russian history-aware capabilities follow-up.
- 2026-05-20: Browser-worker, Rust-core, sidebar, report-link, and test updates were implemented on branch `issue-190-8c706a31ad6a`.

## Root Causes

The Rust solver already had a translation handler, but its phrase table did not know the Russian surface form `как у тебя дела?`, and Russian target-language detection did not explicitly recognize `на английский`.

The browser worker did not have a matching translation handler, so the deployed demo could route the same prompt to the unknown fallback even when the Rust library had a related capability.

The capabilities handler treated `Что ты умеешь?` as a static whole-list question. The follow-up form `Что ещё ты умеешь?` did not use prior assistant turns and was able to repeat web-search details already shown in the conversation.

The left menu was implemented as a fixed action section instead of the shared collapsible sidebar section component.

The issue URL fitter used a two-phase strategy: try the full transcript, then keep only the last two turns and truncate those if needed. It had no backfill phase for older turns and no partial boundary turn.

## Fixes

- Added Russian-to-English support for `как у тебя дела?` in the Rust translation surface table and mirrored it in the browser worker.
- Added explicit `на английский` / `на английском` target detection.
- Added a browser-worker `tryTranslation` handler so the web demo and Rust solver agree on the reported prompt.
- Added history-aware handling for `Что ещё ты умеешь?`, returning additional capabilities without repeating DuckDuckGo or internet-search details from the prior turn.
- Reused the collapsible sidebar section component for the `MENU` action group and persisted its collapsed state.
- Renamed the report section to `## Reproduction of dialog` and removed `## Reproduction Steps`.
- Reworked the report URL fitter to keep the final two messages, backfill earlier messages while budget remains, and include a truncated boundary message when the next full turn is too large.

## Verification Plan

- Rust focused tests:
  - `cargo test russian_translate_how_are_you_prompt_returns_english_surface`
  - `cargo test russian_more_capabilities_follow_up_uses_history_without_repeating_web_search`
- JavaScript syntax checks:
  - `node --check src/web/formal_ai_worker.js`
  - `node --check src/web/app.js`
- Browser-focused checks:
  - Report-link e2e tests for the renamed section and URL fitting behavior.
  - Worker e2e coverage for the reported Russian translation prompt.
  - Sidebar e2e coverage for collapsing the `MENU` section.
  - Playwright screenshot evidence for the final collapsed menu state.

## Local Verification Results

- `cargo test russian_translate_how_are_you_prompt_returns_english_surface`: passed.
- `cargo test russian_more_capabilities_follow_up_uses_history_without_repeating_web_search`: passed.
- `cargo fmt --check`: passed.
- `cargo clippy --all-targets --all-features`: passed.
- `rust-script scripts/check-file-size.rs`: passed.
- `cargo test`: passed.
- `node --check src/web/formal_ai_worker.js`: passed.
- `node --check src/web/app.js`: passed.
- `npm run --prefix tests/e2e check:i18n`: passed.
- `npm run --prefix tests/e2e check:intent-coverage`: passed.
- Focused Playwright e2e checks for report links, browser-worker prompts, and menu collapse: passed.
- Playwright MCP sanity check: the `Menu` section toggled from expanded to collapsed and exposed only the section header after collapse.
