# Access and diagnostics after the repair

Read-only source retrieval is enabled when no restriction is supplied. To
deliberately disable it, set `FORMAL_AI_LIVE_FETCH=0`, `FORMAL_AI_OFFLINE=1` or
`FORMAL_AI_NETWORK=denied`. Explicit restrictions win over an enable switch.
Remove those restrictions to restore the default. A connection, proxy, provider
error or missing capture is distinct from a permission denial.

Ask “How do I enable internet access?” or “Как включить доступ к интернету?”
for harness guidance. Claude users can open `/permissions` and allow the needed
read tool. Explicit ask/deny rules still apply. Removing an operator-level
environment restriction requires changing that environment and restarting the
server. Shell execution and filesystem writes retain their existing controls.

Routine diagnostic logs go to `~/.formal-ai/logs/serve-<pid>.lino` (or
`FORMAL_AI_LOG_DIR`). They are single-line typed Lino by default and redact
credentials before writing. The wrapper's child harness keeps its own output.

```sh
# Explicit terminal diagnostics, including temporary server/session details:
FORMAL_AI_LOG_STDERR=1 formal-ai with claude
# Indented typed records, in the file and any enabled terminal sink:
FORMAL_AI_LOG_FORMAT=nested FORMAL_AI_LOG_STDERR=1 formal-ai with claude
# Silence diagnostic sinks entirely:
FORMAL_AI_SILENT=1 formal-ai with claude
```

The JavaScript server also accepts `--log-stderr` and `--log-format nested`.
Trace payload collection remains opt-in. Machine orchestration control records
are separate from routine diagnostics and keep their canonical framing.

`formal-ai with --global <tool>` prepares persistent client settings, preserves
existing unrelated settings, and keeps backups for `--undo`. Claude receives the
documented context override; a remote server's published capacity is preferred
to the launching machine's capacity. Gemini CLI still has an upstream fixed
input-context fallback; see the case study before relying on an all-clients
context guarantee.
