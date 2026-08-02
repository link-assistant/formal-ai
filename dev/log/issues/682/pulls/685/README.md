# Data log — Issue #682 / PR #685

Compiled reference data for the work of resolving
[issue #682](https://github.com/link-assistant/formal-ai/issues/682) on
[pull request #685](https://github.com/link-assistant/formal-ai/pull/685).

- Repository: `link-assistant/formal-ai`
- Branch: `issue-682-6b8246dfc2bf`
- Base branch: `main`
- Compiled on: 2026-07-13

## Issue in one line

The OpenAI Chat Completions parser rejects an assistant message that carries an
explicit `"content": null` together with `tool_calls`, returning
`HTTP 400 invalid chat request: data did not match any variant of untagged enum
MessageContent`. This is the exact shape Qwen Code (`qwen`) emits, so the qwen
agent loop dies mid-conversation.

## Contents of this folder

| File | What it holds |
| --- | --- |
| `README.md` | This index. |
| `issue-682.md` | Full text of issue #682 (summary, repro, root cause, suggested fix). |
| `pull-685.md` | PR #685 state and the (currently empty) comment/review threads. |
| `related-issues.md` | Companion issues #680 and #681 (same live matrix, related defects). |
| `root-cause.md` | Source-level analysis: the exact `ChatMessage` / `MessageContent` code and why explicit `null` fails. |
| `raw/` | Machine-readable captures: `gh` JSON and plain-text views of the issue, PR, and related issues. |

## Status snapshot (at compile time)

- Issue #682: **OPEN**, label `bug`, author `konard`, 0 comments.
- PR #685: **DRAFT**, base `main`, head `issue-682-6b8246dfc2bf`, 0 comments, 0 reviews.
  - Current diff vs `main`: only a `.gitkeep` addition (no fix committed yet).
- Related: #680 (OPEN, phrasing-gated tool routing), #681 (OPEN, write→read misclassification).
