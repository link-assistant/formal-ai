# Output and session debugging

Formal AI renders ordinary answers as friendly text. Structured values use a
JSON code fence rather than leaking a raw protocol envelope:

```json
{"status":"ok","items":["a","b"]}
```

Tool results are retained byte-for-byte in the client conversation, then
normalized into a localized friendly answer. A follow-up can therefore request
a particular line, URL, or the complete result without losing the transcript.

After `formal-ai with` exits, it reports only the session artifact created or
changed by that run and prints a resume command where supported. Temporary
homes containing a transcript are preserved. Set `FORMAL_AI_PROXY_LOG` to an
existing logging-proxy file to include it in the same report.

| Harness | Session/transcript location | Resume form |
| --- | --- | --- |
| Codex | `$CODEX_HOME/sessions/YYYY/MM/DD/rollout-*.jsonl` (normally `~/.codex/sessions/...`) | `codex resume <id>` |
| T3 Code (Codex) | isolated `CODEX_HOME/sessions/**/*.jsonl` | open the preserved T3/Codex session |
| OpenCode | `~/.local/share/opencode/opencode.db` | `opencode --session <id>` |
| Agent CLI | `~/.local/share/link-assistant-agent/storage/session/**/*.json` | `agent --resume <id>` |
| Gemini | `$GEMINI_CLI_HOME/.gemini/tmp/**/*.jsonl` | `gemini --resume <id>` |
| Claude | `$CLAUDE_CONFIG_DIR/projects/**/*.jsonl` | `claude --resume <id>` |
| Qwen | `~/.qwen/projects/**/*.jsonl` inside its selected home | `qwen --resume <id>` |
| Grok | `~/.grok/**/*.jsonl` inside its selected home | client-specific |
| Aider | no registry-declared session artifact | client-specific |
| Cursor | no registry-declared session artifact | Cursor history UI |

The displayed path is authoritative for an isolated one-shot run; do not copy a
hardcoded home from this table. For server-side debugging enable
`FORMAL_AI_TRACE_REQUESTS=1`, redirect server stderr/stdout to a log, and match
its request timeline to the client transcript or `FORMAL_AI_PROXY_LOG`.
