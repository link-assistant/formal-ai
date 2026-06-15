# Issue 488 Case Study: Deep Thinking Preview

## Inputs

- GitHub issue: https://github.com/link-assistant/formal-ai/issues/488
- Pull request: https://github.com/link-assistant/formal-ai/pull/489
- Raw GitHub data:
  - `raw-data/issue-488.json`
  - `raw-data/issue-488-comments.json`
  - `raw-data/pr-489.json`
  - `raw-data/pr-489-review-comments.json`
  - `raw-data/pr-489-conversation-comments.json`
  - `raw-data/pr-489-reviews.json`
- Code search snapshots:
  - `raw-data/github-code-search-thinking-steps.json`
  - `raw-data/github-code-search-diagnosticsSteps.json`
  - `raw-data/recent-merged-prs-thinking-diagnostics.json`
- Online research: `raw-data/online-research.md`
- Verification screenshot: `screenshots/thinking-preview-expanded.png`

## Requirements

| ID | Requirement | Implementation |
| --- | --- | --- |
| R488-1 | Show thinking while the model is working. | The pending assistant bubble now renders the same `ThinkingPreview` surface with a localized working step. |
| R488-2 | Show the last thinking paragraph by default. | Completed assistant messages render a collapsed preview whose current paragraph is the latest human-readable structured step. |
| R488-3 | Show part of the previous thinking paragraph with a fade. | The collapsed preview shows the previous step in a clipped, masked line above the current step. |
| R488-4 | Provide an expand button. | `ThinkingPreview` toggles between collapsed current/previous view and the full ordered list. |
| R488-5 | Use human-readable target-user language instead of meta labels. | `buildThinkingPreviewSteps` maps structured step ids such as `match_rule` and `deformalize` into localized summary templates. |
| R488-6 | Preserve raw diagnostics for debugging. | Existing `.thinking-steps`, `.diagnostics-steps`, evidence, tool calls, and report traces remain gated by diagnostics mode. |
| R488-7 | Let users control thinking granularity. | Settings now include a localized `Thinking detail` select (`brief`, `standard`, `detailed`) that reshapes previews without changing raw diagnostics. |
| R488-8 | Compile a case study and use existing components/libraries. | This directory preserves raw data and research; the implementation reuses the app's existing React renderer, i18n catalog, and structured solver steps. |

## Root Cause

Assistant messages already carried `thinkingSteps`, but `Message` only rendered them when diagnostics mode was enabled. Those strings were raw debug labels such as `match_rule: greeting`, which are useful for maintainers but do not satisfy a user-facing thinking UI. The pending assistant bubble also had no thinking surface; it only displayed the generic working text.

## Solution

- Added `buildThinkingPreviewSteps` and `naturalizeThinkingStep` in `src/web/app.js` to convert existing structured trace events into localized summaries.
- Added `ThinkingPreview`, a collapsed/expanded React component that shows the previous faded step and the current step by default.
- Added a `Thinking detail` preference so the same structured trace can render as brief, standard, or detailed user-facing steps.
- Added light/dark CSS for the preview without changing the existing diagnostics panels.
- Added i18n catalog entries and checker coverage for English, Russian, Chinese, and Hindi.
- Added `tests/e2e/tests/issue-488.spec.js` to reproduce the missing default preview and verify the expanded human-readable summaries.

## Verification

Pre-fix reproduction:

```sh
npm --prefix tests/e2e run test:local -- --grep "Issue #488"
```

Result: failed because `[data-testid="thinking-preview"]` was not found. Saved in `raw-data/repro-before-e2e.log`.

Post-fix checks:

```sh
npm --prefix tests/e2e run check:i18n
npm --prefix tests/e2e run check:web-tdz
npm --prefix tests/e2e run test:local -- --grep "Issue #488"
npm --prefix tests/e2e run test:local -- --grep "diagnostics toggle shows trace"
rust-script scripts/check-file-size.rs
cargo fmt --check
cargo clippy --all-targets --all-features
cargo test
```

Result: all passed. Logs are saved under `raw-data/`.

## Follow-Up

The current worker returns structured steps when an answer completes; the pending preview therefore uses a neutral localized working step. Streaming true intermediate step deltas during a long-running worker request would require a worker progress event protocol and a stateful per-turn progress buffer.
