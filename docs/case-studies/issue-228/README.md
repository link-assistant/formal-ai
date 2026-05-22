# Issue 228 Case Study

## Scope

Issue: <https://github.com/link-assistant/formal-ai/issues/228>

Pull request: <https://github.com/link-assistant/formal-ai/pull/229>

Branch: `issue-228-f1e6cdaa38f0`

The GitHub Pages browser demo, formal-ai v0.100.0, returned the
unknown fallback for:

```text
list all genshin characters with off-field DMG
```

The report asked for a general, deterministic way to support this class
of prompts through actual web search and symbolic reasoning, plus a
case-study folder containing downloaded issue/PR data, online research,
root-cause analysis, requirements, and solution options.

## Artifacts

Downloaded and generated artifacts live under `raw-data/`:

- `issue-228.json`, `issue-228-comments.json`: issue payload and
  comments at collection time.
- `pr-229.json`, `pr-229-conversation-comments.json`,
  `pr-229-review-comments.json`, `pr-229-reviews.json`: PR metadata
  and comment snapshots.
- `ci-runs-branch.json`: recent branch CI run list collected during
  implementation.
- `repro-before-cli.txt`: local CLI reproduction before the fix.
- `rust-regression-before.log`: failing targeted Rust regression test
  before implementation.
- `rust-targeted-after.log`, `rust-web-requests-after.log`: passing
  targeted Rust tests after implementation.
- `repro-after-cli.txt`: local CLI output after implementation.
- `npm-ci.log`, `e2e-issue228-after.log`: e2e dependency install and
  focused browser regression result.
- `cargo-fmt.log`, `cargo-fmt-check.log`, `clippy.log`,
  `check-file-size.log`, `cargo-test.log`, `docs-requirements-after.log`,
  `git-diff-check.log`: local verification logs.
- `online-research.md`: external references used for the analysis.

## Timeline

| Time (UTC) | Event |
| --- | --- |
| 2026-05-22 13:59 | Browser report captured formal-ai v0.100.0 returning `intent: unknown` for the Genshin off-field damage prompt. |
| 2026-05-22 14:03 | Issue #228 was opened with the reproduction, expected Google-style answer, and case-study requirements. |
| 2026-05-22 | Local CLI reproduction confirmed the same prompt returned the unknown fallback. |
| 2026-05-22 | A failing unit regression was added for enumeration-style research prompts. |
| 2026-05-22 | The shared Rust web-search recognizer and browser worker recognizer were extended with `enumeration_research_request`. |
| 2026-05-22 | Focused Rust and Playwright regressions passed with mocked search-provider results. |

## Requirements And Status

| ID | Requirement | Status |
| --- | --- | --- |
| R1 | The reported prompt must not return the unknown fallback. | Implemented. It now routes to `intent:web_search`. |
| R2 | The solution must be general for similar queries, not hard-coded to Genshin. | Implemented with an enumeration-research recognizer for prompts like `list all X with Y`. |
| R3 | Use actual deterministic logic and internet search, not LLM inference. | Implemented by routing to the existing symbolic web-search planner and browser provider fusion. |
| R4 | Preserve issue/PR/log data under `docs/case-studies/issue-228`. | Implemented in `raw-data/`. |
| R5 | Search online for additional facts and data. | Implemented in `raw-data/online-research.md`. |
| R6 | Reconstruct timeline, requirements, root causes, and solution options. | Implemented in this case study. |
| R7 | Add debug output if the root cause cannot be found. | Not needed; the root cause was reproduced and isolated. |
| R8 | Report issues to related upstream projects if needed. | Not needed; the failure was local intent recognition. |

## Root Cause

The shared web-search intent recognizer only covered three routes:

1. explicit prefixes such as `search online for ...`;
2. semantic search actions such as `find information about ...`;
3. a narrow class of question-style research prompts such as
   `What is the most popular dataset ...?`.

The reported prompt is a command-style enumeration request:

```text
list all <entity class> with <property>
```

It has no explicit search/source marker and does not begin with a
question word. Therefore both the Rust solver and browser worker fell
through every specialized handler and returned `intent: unknown`.

## Solution Options

| Option | Tradeoff | Decision |
| --- | --- | --- |
| Seed a static Genshin off-field character list. | Fast for this prompt but stale as the game changes, domain-specific, and contrary to the general requirement. | Rejected. |
| Add a dedicated game knowledge parser. | Could eventually extract structured character lists from role tables, but it is too narrow for the issue's "all such queries" requirement. | Deferred. |
| Route enumeration research requests to web search. | General, small, and uses the existing deterministic provider-fusion pipeline. | Implemented. |

## Implemented Fix

- Added `EnumerationResearchRequest` to the Rust web-search query kind.
- Recognized enumeration prefixes such as `list all`, `list every`,
  `show all`, `name all`, and `enumerate all`.
- Required a constrained enumerable target, such as `with`, `that`,
  `who`, `having`, `for`, `by`, or `in`, to avoid turning every local
  list command into web search.
- Mirrored the same recognition logic in `src/web/formal_ai_worker.js`.
- Recorded the reason as
  `web_search:query_kind:enumeration_research_request`.
- Added Rust and browser regressions for the exact reported prompt.

## Before / After

| Prompt | Before | After |
| --- | --- | --- |
| `list all genshin characters with off-field DMG` | Unknown fallback | `intent:web_search`; query `genshin characters with off field dmg` in Rust and `genshin characters with off-field DMG` in the browser worker |

## Verification

- Before fix:
  `cargo test enumeration_research_request_routes_to_web_search_handler -- --nocapture`
  failed because the prompt returned `intent: unknown`.
- After fix:
  `cargo test enumeration_research_request_routes_to_web_search_handler -- --nocapture`
  passed.
- After fix:
  `cargo test --test unit web_requests::` passed.
- After fix:
  `npx playwright test --config=playwright.local.config.js issue-228.spec.js`
  passed from `tests/e2e`.

Full local verification logs are stored in `raw-data/`.
