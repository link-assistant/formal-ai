# Issue 760: T3 Code integration

This case study records the implementation and verification of the
`formal-ai with t3code` integration. It covers both wrapper aliases, OpenAI
Responses and Anthropic configuration, global setup, and a real T3 Code
session.

## Agent-driven reproduction

Formal AI was started locally in agent mode and connected to the installed
Link Assistant Agent CLI. The successful session
`ses_088533538fferPDfZC5Y7S22eG` authored the initial regression test in
`tests/issue_760.rs`; its stream, stderr, and generated plan are preserved in
`red-test-agent-final/`. The baseline test then failed three times with the
expected error:

```text
unsupported tool `t3code`; supported tools: codex, opencode, agent, gemini,
claude, qwen, grok, aider
```

Two earlier prompts were routed as issue/definition requests instead of file
edits, and a later production-edit prompt did not produce a diff. Their logs
are retained rather than hidden. The accidentally created issue from the first
attempt was immediately closed.

## Real T3 Code session

The real npm package `t3@0.0.28` was installed and run with Bun 1.3.14 because
the host's Node 20 runtime is older than T3 Code's declared Node requirement.
Formal AI ran in agent mode on `127.0.0.1:8786`; the new wrapper launched T3
Code with:

```bash
formal-ai with --no-start-server --non-interactive \
  --base-url http://127.0.0.1:8786 t3code \
  --auto-bootstrap-project-from-cwd
```

The T3 HTTP orchestration API was used exactly as its web UI does to create a
Codex-backed turn with model `formal-ai`. `snapshot-after.json` records the
completed assistant response and a ready Codex session. A second turn asked
the agent to run `ls`; `snapshot-tool-call.json` records:

- `tool.started` and `tool.completed` activities from the real Codex session;
- the captured command output in the assistant message;
- ready checkpoints containing changed-file metadata, which feeds T3 Code's
  diff viewer.

This proves that requests passed through T3 Code, Codex's app server, the
generated `CODEX_HOME/config.toml`, and Formal AI's local Responses endpoint.

The session also exposed two existing Formal AI behavior limitations. The
literal-response prompt was handled by the unknown-intent fallback instead of
echoing the requested marker. The `ls` route passed the descriptive tail as
arguments (`ls in the current directory ...`), so the real tool invocation
completed with exit code 2. Neither failure is hidden: the complete messages,
tool command, output, and diff metadata are in the saved snapshot. They do not
affect T3 transport/configuration, and the T3 UI surfaces them correctly.
T3's automatic title generator also requested `gpt-5.4-mini`, so Formal AI did
not rename the thread; the main session still used the configured `formal-ai`
model and remained ready for subsequent turns.

## Automated verification

- `focused-test.log`: four T3-specific tests passed.
- `wrapper-integration-test.log`: all 18 existing wrapper integration tests
  passed.
- `docs-traceability-test.log`: the existing seed/docs traceability test
  passed.

The T3-specific tests verify both aliases execute `t3`, isolated Codex TOML,
the model catalog and dummy key, OpenAI and Anthropic endpoint propagation,
interactive/headless behavior, protocol-specific global setup, and inclusion
in `--all`.
