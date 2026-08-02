# Issue 750: friendly, lossless tool results

## Root cause

The agentic planner treated a client tool result as display-ready text. Shell
results were always placed in a `text` fence, so JSON transport objects and
client wrappers such as `<untrusted_context>`, `Output:`, and process-group
metadata reached the user unchanged. Web completion paths also discarded the
original failed/empty result before presentation.

Conversation progress intentionally scans only messages after the latest user
turn. That is correct for deciding whether the current recipe step ran, but it
also meant a later question could not recover a URL, line, or full payload from
an earlier tool turn.

## Fix

`agentic_coding::tool_result` now provides one presentation boundary for shell,
web, edit, and otherwise unknown client tools. It:

- unwraps known transport envelopes while leaving the transcript message intact;
- distinguishes explicit errors, stderr, unsuccessful flags, nonzero exits, and
  HTTP failure statuses from successful output;
- detects strict JSON and script payloads for an appropriate Markdown fence;
- gives empty list, search, and generic successes distinct explanations;
- selects all surrounding prose from the English, Russian, Hindi, and Chinese
  response seed; and
- resolves seed-backed follow-up intents against the unchanged prior tool
  message, including first/second URL, numbered line, and complete-result asks.

Raw outputs continue through `chat_tool_executions` into the append-only memory
log when `FORMAL_AI_MEMORY_PATH` is configured. The browser, desktop, VS Code,
and autonomous CLIs all consume the same OpenAI-compatible server result; this
agentic server path has no separate JavaScript-worker implementation to mirror.

## Regression evidence

The focused test was written before the implementation. Its red run has six
failures showing the raw JSON envelope, generic empty output, validation error
misclassification, missing generic-tool completion, and missing cross-turn URL.
The expanded green matrix contains 11 tests with multilingual and per-tool case
tables covering shell, web fetch/search, glob, grep, list, edit, todo, task,
structured output, empty output, errors, HTTP statuses, transcript retention,
follow-ups, and the complete OpenAI chat surface:

```sh
cargo test --test unit -- issue_750 --nocapture
```

See [`raw-data/focused-red.log`](raw-data/focused-red.log) and
[`raw-data/focused-green.log`](raw-data/focused-green.log). The raw issue,
comments, and initial PR-review state are retained under [`raw-data/`](raw-data/).

## Real Agent CLI proof

The reproducible experiment builds a disposable Git repository with two known
files, starts a release Formal AI server with raw memory recording, and asks the
installed Link Assistant Agent CLI 0.25.0 to run `ls` through its real `bash`
tool:

```sh
cargo build --release --bin formal-ai
experiments/issue_750_tool_results/run.sh
```

The captured final answer is:

````text
The `ls` command completed. Output:

```text
alpha.txt
beta.json
```
````

Client events, classified diagnostics, server request traces, the extracted
answer, and durable raw memory are in
[`agent-cli-evidence/`](agent-cli-evidence/). The harness fails if the final
answer contains `untrusted_context`, process-group metadata, or an `exit_code`
field.

The repository's generic `examples/self-coding/run.sh --live` launcher was also
attempted first, but its outer `solve` wrapper rejected `formal-ai` as an Agent
CLI model before starting a client. That attempt is preserved in
[`raw-data/self-coding-live-attempt.log`](raw-data/self-coding-live-attempt.log)
and was reported transparently in
[the issue comment](https://github.com/link-assistant/formal-ai/issues/750#issuecomment-5012102050).
The direct Agent CLI experiment above is the successful self-hosted proof.
