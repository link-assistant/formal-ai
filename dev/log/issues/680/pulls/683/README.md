# Data log — Issue #680 / PR #683

Compiled data related to
[issue #680](https://github.com/link-assistant/formal-ai/issues/680) and
[pull request #683](https://github.com/link-assistant/formal-ai/pull/683).

- **Repo:** `link-assistant/formal-ai`
- **Version at collection:** `0.282.0`
- **Base commit:** `33a7d07fff32105e7dd1dc8134db4a882a83c87c`
- **Branch:** `issue-680-795d4fb2b64f`
- **Collected:** 2026-07-13

## Issue summary

> **Agentic CLIs: tool calls are phrasing-gated, not intent-based — web search &
> web fetch never fire; write/edit/shell mostly fail (all 5 CLIs).**

Driving `formal-ai serve` with the supported agentic CLIs, tool-call emission is
gated on a handful of hard-coded phrasings rather than on natural-language
intent. Two whole capabilities — **web search** and **web fetch** — never emit a
tool call for any phrasing. This is a single shared root cause in the planner, so
it affects all five targeted CLIs (codex, opencode, gemini, qwen,
`@link-assistant/agent`) identically.

### Live matrix (correct tool_call emitted / 10 phrasings)

| tool | agent | opencode | codex | qwen | gemini | total |
|------|:-----:|:--------:|:-----:|:----:|:------:|:-----:|
| shell | 2/10 | 2/10 | 2/10 | 0/10 | 0/10 | 6/50 |
| read | 6/10 | 6/10 | 7/10 | 7/10 | 7/10 | 33/50 |
| write | 1/10 | 1/10 | 1/10 | 0/10 | 0/10 | 4/50 |
| edit | 0/10 | 0/10 | 0/10 | 0/10 | 0/10 | 0/50 |
| web_search | 0/10 | 0/10 | 0/10 | 0/10 | 0/10 | 0/50 |
| web_fetch | 0/10 | 0/10 | 0/10 | 0/10 | 0/10 | 0/50 |

## Folder contents

```
dev/log/issues/680/pulls/683/
├── README.md                          ← this index
├── issue/
│   ├── issue-680.json                 ← issue #680 (structured)
│   ├── issue-680.md                   ← issue #680 (rendered)
│   └── issue-680-comments.json        ← issue comments (empty: [])
├── pull/
│   ├── pr-683.json                    ← PR #683 (structured)
│   ├── pr-683.md                      ← PR #683 (rendered)
│   ├── pr-683.diff                    ← PR #683 diff
│   ├── pr-683-conversation-comments.json  ← (empty: [])
│   ├── pr-683-review-comments.json        ← (empty: [])
│   └── pr-683-reviews.json                ← (empty: [])
├── related/
│   ├── issue-681.md   ← write→read wrong-tool + qwen 400 (split out of #680)
│   ├── issue-607.md   ← agent CLI could not run `ls` (prior art)
│   ├── issue-602.md   ← Codex CLI: no SSE streaming on /v1/responses
│   ├── issue-604.md   ← Chat Completions streaming malformed
│   ├── issue-628.md   ← docs: agentic CLI tools testing guide
│   └── issue-671.md   ← E52: multi-CLI agentic E2E matrix in CI
└── code/
    ├── code-references.md             ← where the root cause lives
    ├── solver_handler_how.rs
    ├── solver_handlers/
    │   ├── natural_language_tools.rs
    │   ├── web_requests.rs
    │   └── feature_capability.rs
    └── agentic_coding/
        └── planner.rs
```

## Related issues (referenced by #680)

| # | Title |
|---|-------|
| 681 | Agentic CLIs: a file-creation request emits a `read` tool_call on the (nonexistent) target instead of `write` |
| 607 | Agent CLI cannot run shell commands (`ls`) via natural language |
| 602 | OpenAI-compatible server cannot be driven by Codex CLI: no SSE streaming on `/v1/responses` |
| 604 | OpenAI Chat Completions streaming is malformed |
| 628 | docs: add an agentic CLI tools testing guide |
| 671 | E52: Multi-CLI agentic end-to-end matrix in CI |

## Notes on data completeness

- Issue #680 has **0 comments** at collection time.
- PR #683 has **0 conversation comments, 0 review comments, 0 reviews**, and its
  only commit is the initial `.gitkeep` bootstrap.
- All comment/review JSON files are intentionally kept even though empty (`[]`)
  so re-runs can diff against them.
