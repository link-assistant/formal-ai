# Issue 757: report wrapped CLI session files

Issue [#757](https://github.com/link-assistant/formal-ai/issues/757) asked
`formal-ai with` to finish interactive and one-shot runs with the concrete
session/transcript file paths needed for debugging, plus resume commands where
the client supports them. This case study records the investigation,
implementation, and reproducible verification for PR
[#780](https://github.com/link-assistant/formal-ai/pull/780).

## Root cause

`run_ephemeral` waited for the child process and immediately dropped every
temporary config directory. It neither knew where each client stored sessions
nor compared the filesystem before and after the child ran. For clients such as
Codex, Gemini, Qwen, and Claude, the isolation that prevents a user's normal
credentials from leaking into a wrapped run could also delete the transcript
when the wrapper exited.

The fix keeps client-specific knowledge in `data/seed/client-integrations.lino`.
Each invocation may declare a session root, artifact suffix, resume template,
and (for OpenCode's SQLite store) a command that resolves the latest session
identifier. The wrapper snapshots matching files before launch, reports only
the newest file created or changed by the child, and preserves a temporary home
only when the reported file is inside it. An existing `FORMAL_AI_PROXY_LOG` is
reported in the same final block.

## Supported locations

| Client | Declared session artifact | Resume command |
| --- | --- | --- |
| Codex | `.codex/sessions/**/*.jsonl` | `codex resume <id>` |
| Gemini | `.gemini/tmp/**/*.jsonl` | `gemini --resume <id>` |
| Qwen | `.qwen/projects/**/*.jsonl` | `qwen --resume <id>` |
| OpenCode | `.local/share/opencode/opencode.db` | `opencode --session <id>` |
| Agent CLI | `.local/share/link-assistant-agent/storage/session/**/*.json` | `agent --resume <id>` |
| Claude | temporary `CLAUDE_CONFIG_DIR/projects/**/*.jsonl` | `claude --resume <id>` |
| Grok | `.grok/**/*.jsonl` | not advertised by the client |

The issue also mentioned Cursor, but Cursor is not a supported `formal-ai with`
integration in the current seed registry. No path is fabricated for a command
the wrapper cannot launch. Aider remains supported as a wrapper, but the issue's
implementation list did not define a durable session artifact for it, so it
likewise produces no invented entry.

## Reproduction-first tests

The first integration test used a fake Codex executable that wrote the same
nested JSONL structure as the real client. Before the implementation, both
one-shot and interactive cases exited successfully with empty diagnostic
output; that failing run is preserved in
`agent-cli-evidence/red-test.log`. The fixed test additionally parses the
reported path and asserts that it still exists after the wrapper exits.

A second end-to-end test exercises Gemini, Qwen, OpenCode, Agent CLI, Claude,
and Grok from the seed registry and verifies the proxy-log line and each known
resume command. The green focused runs are in
`agent-cli-evidence/session-tests.log` and
`agent-cli-evidence/refactor-test.log`.

## Formal AI and Agent CLI evidence

The contribution workflow was attempted through the installed Agent CLI with a
locally built Formal AI server and request tracing enabled. The generic `solve`
launcher rejected `formal-ai` as a model before starting Agent CLI
(`agent-cli-evidence/solve.log`). A direct Agent CLI run did reach the local
Formal AI provider, but its only proposed tool call was the invalid shell
command `cp resume commands`; it made no repository changes. The complete trace
is preserved in `agent-cli-evidence/agent-cli.log`, with the server-side trace
in `agent-cli-evidence/formal-ai-server.log`. The implementation was therefore
completed manually and carries no false Formal-AI-authorship trailer.

## Evidence index

- `raw-data/` contains the issue, PR, and all three PR comment/review channels
  captured before implementation.
- `agent-cli-evidence/build.log` records the release build used for the direct
  Formal AI provider attempt.
- `agent-cli-evidence/cargo-check.log` records the all-target/all-feature compile.
- `agent-cli-evidence/red-test.log` and the green logs preserve the test-first
  regression timeline.

Together, the tests establish the whole requested behavior: both run modes emit
the final block, every specified supported integration uses its declared path,
resume commands contain real extracted IDs, the server log is conditional on an
existing configured file, and temporary session files survive wrapper exit.
