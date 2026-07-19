# T3 Code setup

Install the `t3` executable, then let the wrapper launch it against Formal AI:

```bash
formal-ai with t3code
formal-ai with --non-interactive t3
```

Interactive mode opens T3's browser UI. `--non-interactive` maps to
`--no-browser`. The default **Codex** provider uses Responses with provider id
`formalai`, model `formal-ai`, base URL
`http://127.0.0.1:8080/api/openai/v1`, and a non-empty dummy key.

For a Claude-backed T3 session:

```bash
formal-ai with --protocol anthropic t3code
```

That sets `ANTHROPIC_BASE_URL` to `/api/anthropic` and supplies an auth token
while preventing a vendor API key from taking precedence. Permanent setup is
`formal-ai with --global --protocol openai t3code` (or `anthropic`); undo it
with `--global --undo t3code`.

T3 uses an isolated `CODEX_HOME` in one-shot mode. The wrapper preserves and
reports any new session JSONL after exit. T3's own checkpoints and diff viewer
remain client artifacts; durable Formal AI shared memory is the server's
`~/.formal-ai/memory.lino`, so start the server with the same memory path used
by Desktop, CLI, or Docker when those surfaces should share memory.

Verify by asking T3 to create a harmless file in a disposable repository, then
confirm the tool lifecycle, checkpoint diff, Formal AI server request, and
reported session artifact.
