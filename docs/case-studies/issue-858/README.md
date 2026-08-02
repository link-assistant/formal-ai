# Issue #858: Claude Code returning-user recap

Issue [#858](https://github.com/link-assistant/formal-ai/issues/858) reports
that Claude Code's `/recap` command reaches Formal AI as an unknown request.
The issue's original screenshot is preserved as
[`raw-data/claude-code-missing-recap.png`](raw-data/claude-code-missing-recap.png).
It shows a completed Rust hello-world exchange followed by Formal AI's
"could not determine" fallback instead of a recap.

## Root cause

Claude Code does not send the literal slash command to the model. Claude Code
2.1.220 expands it into a returning-user request: the user stepped away, is
coming back, and needs a short plain recap led by the overall goal and current
task. Formal AI already supported ordinary conversation summaries, but its seed
lexicon had no semantic role for this returning-user situation.

The first implementation replay exposed a second defect. Claude adds a
`<system-reminder>` content part before the user's first request. The ordinary
solver adapter stripped that client metadata, but the agentic conversation
recall adapter independently projected `plain_text()` into history. Its recap
therefore began with the injected current date instead of the user's goal.

## Resolution

The change adds one `conversation_return_recap` semantic role to seed data with
English, Russian, Hindi, Chinese, and Spanish surfaces. Production Rust names
only that role; it does not hardcode Claude's full internal prompt or a
language-specific phrase list.

The returning-user route reuses the dialog summarization module with explicit
bounds: under 40 words, one or two plain sentences, and no Markdown. It
selects the latest real user goal and the latest assistant status after that
goal. Ordinary `Summarize` requests retain their existing detailed report,
title, and user-turn list.

Agentic recap now uses the canonical protocol history projection, which removes
`<system-reminder>` blocks and echoed system text from user turns. The same
projection is reused by conversation-aware web research. The browser worker
mirrors the semantic role and compact recap contract; the executable parity
harness loads the real worker modules and generated seed data.

## Reproduction and acceptance

The protocol regression uses an Anthropic Messages request with Claude's
multi-part reminder, real `tool_use` / `tool_result` history, and the exact
returning-user request observed from the client. Before the history fix it
failed with:

```text
client-injected context must not displace the user's goal:
As you answer the user's questions, you can use the following context:
currentDate Today's date is 2026-08-02. ...
```

The same build was then exercised through the installed Claude Code 2.1.220
binary and the live Anthropic-compatible server. A fresh session received a
representative Rust goal; a resumed invocation sent `/recap`:

```sh
claude --bare --model formal-ai --permission-mode acceptEdits \
  --output-format json --verbose --session-id <uuid> --print \
  "Create and verify a Rust hello-world program in main.rs."

claude --bare --model formal-ai --permission-mode acceptEdits \
  --output-format json --verbose --resume <uuid> --print "/recap"
```

| Build | Live `/recap` result | Contract |
| --- | --- | --- |
| Before protocol-history fix | `As you answer the user's questions ... currentDate Today's date is 2026-08-02. ...` | Metadata displaced the goal. |
| After fix | `Create and verify a Rust hello-world program in main.rs. Here is a minimal Rust hello world program: Execution status: compiled and ran in issue-8 local verification harness (isolated sandbox).` | 29 words, two plain sentences, no Markdown, no reminder metadata. |

Machine-readable summaries of those two real runs are retained in
[`live-recap-before.json`](raw-data/live-recap-before.json) and
[`live-recap-after.json`](raw-data/live-recap-after.json). Authenticated issue
and initial pull-request snapshots, including every empty comment/review
collection, are beside them in [`raw-data/`](raw-data/).

Run the focused acceptance checks with:

```sh
cargo test --test unit issue_858 -- --nocapture
node experiments/issue-858-worker-recap-parity.mjs
```

[`requirements.md`](requirements.md) maps each acceptance property to its
implementation evidence and executable regression.
