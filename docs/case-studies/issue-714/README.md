# Issue 714: agentic report and web-tool routing

Issue: <https://github.com/link-assistant/formal-ai/issues/714>

Pull request: <https://github.com/link-assistant/formal-ai/pull/726>

## User-visible failure

In OpenCode 1.17.19, the dialog captured in the issue showed two failures:

1. Sending `Report` produced an embedded web-search explanation instead of creating a GitHub issue with `gh`.
2. Sending `Search for Elon Musk` produced provider prose instead of calling OpenCode's real web-search tool.

The original screenshot is preserved at
[`raw-data/issue-714-screenshot.png`](raw-data/issue-714-screenshot.png). The issue,
comments, pull-request metadata, all three pull-request comment/review channels, and
the failing regression output are preserved in [`raw-data/`](raw-data/).

## Reproduction and root cause

The live reproduction used `with-formal-ai` against a traced agent-mode server.
Without `OPENCODE_ENABLE_EXA`, OpenCode advertised these tools:

```text
bash, edit, glob, grep, read, skill, task, todowrite, webfetch, write
```

There was no search capability for Formal AI to select. The planner correctly
fell through, after which the ordinary answer path rendered the misleading search
prose visible in the screenshot.

With `OPENCODE_ENABLE_EXA=1`, OpenCode also advertised `websearch`; the same exact
`Search for Elon Musk` prompt emitted a real `websearch` call and returned live
results. This matches OpenCode's current documentation: `websearch` is available
with the OpenCode provider or when `OPENCODE_ENABLE_EXA` is truthy, while
`webfetch` is a separate built-in tool. The hosted search MCP is Exa-backed and
does not require a separate API key. See [OpenCode tools](https://opencode.ai/docs/tools/).

The report failure had a separate cause. The deterministic planner had no report
action. Bare `Report` therefore reached the broad ordinary answer/search path;
there was no code capable of composing or emitting `gh issue create`.

OpenCode Zen is an optional model gateway, not the documented search backend, so
this change does not incorrectly route search through Zen. See
[OpenCode Zen](https://opencode.ai/docs/zen/). No upstream issue was necessary:
OpenCode behaves as its documentation describes; Formal AI's wrapper simply was
not opting into the tool.

## Implemented behavior

- The OpenCode ephemeral integration now sets `OPENCODE_ENABLE_EXA=1`, causing the
  CLI to advertise its native `websearch` tool. OpenCode's own permission rules
  remain authoritative, so users can allow, ask, or deny `websearch`/`webfetch` as
  documented in [OpenCode permissions](https://opencode.ai/docs/permissions/).
- An explicit, seed-backed report vocabulary recognizes whole-turn interface
  actions such as `Report`, localized variants, and `Create issue`.
- The report planner selects any advertised run-capability alias (`bash`, `shell`,
  or `run_command`), builds a shell-quoted, non-interactive `gh issue create`
  command, and includes up to the latest 20 user/assistant messages in the issue
  body. The command shape follows the official
  [`gh issue create` manual](https://cli.github.com/manual/gh_issue_create).
- After `gh` returns, Formal AI completes with its output, including the created
  issue URL. Without a run capability it explains the missing shell access and
  does not misroute the action to web search.
- Exact action matching prevents ordinary requests such as “write a report” from
  opening a GitHub issue.

The capability classifier is protocol- and CLI-name-independent. Integration
coverage proves the same report action over OpenAI Chat Completions, OpenAI
Responses, and Gemini `generateContent`, using three distinct run-tool aliases.

## Regression-first evidence

The minimal unit regression was run before implementation. Four assertions failed:

```text
report action did not emit a tool call: None
expected a shell-access explanation, got None
expected completion after gh returned, got None
```

The fourth failure only normalized the existing search query's case. Full output
is in [`raw-data/reproducing-unit-test-before-fix.log`](raw-data/reproducing-unit-test-before-fix.log).

After the fix:

```text
cargo test --test unit issue_714_agentic_mode
# 5 passed

cargo test --test integration issue_714_agentic_mode
# 3 passed

cargo test --test integration with_formal_ai_opencode_ephemeral_writes_temp_config_and_model_flag
# 1 passed
```

## Real Agent CLI verification

[`experiments/issue_714_agentic_mode/run_report_e2e.sh`](../../../experiments/issue_714_agentic_mode/run_report_e2e.sh)
boots the production release server, drives the installed `@link-assistant/agent`
CLI, and places a sandboxed `gh` fixture first on `PATH`. The fixture records the
arguments and returns a representative GitHub URL; it cannot mutate GitHub.

The real CLI completed the report loop in two chat rounds:

```text
Agent CLI invoked gh successfully in 2 chat rounds.
```

The run is preserved in
[`raw-data/agent-cli-report-e2e.log`](raw-data/agent-cli-report-e2e.log), and the
same experiment is part of the repository's Agent CLI CI job.
