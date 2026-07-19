title:	Agentic CLIs: a file-creation request emits a read tool_call on the (nonexistent) target instead of write
state:	OPEN
author:	konard (Konstantin Diachenko)
labels:	bug
comments:	0
assignees:	
projects:	
milestone:	
issue-type:	
parent:	
sub-issues:	
sub-issues-completed:	
blocked-by:	
blocking:	
number:	681
--
## Summary

A natural-language **file-creation** request causes the server to emit a **`read` tool_call on the target file** (which does not exist yet) instead of a `write` tool_call — even when the client advertises both `read` and `write` tools. The CLI then tries to read a nonexistent file and the write never happens.

This is distinct from "no tool_call is emitted" (companion umbrella issue #680): here a tool_call *is* emitted, but it is the **wrong tool**, which is a correctness bug in its own right.

## Environment

- `formal-ai 0.282.0` (global install), agent mode (`FORMAL_AI_AGENT_MODE=1`)
- Reproduced directly on `POST /api/openai/v1/chat/completions`

## Reproduction

```bash
curl -sS http://127.0.0.1:8080/api/openai/v1/chat/completions \
  -H 'content-type: application/json' -H 'authorization: Bearer formal-ai' \
  -d '{
    "model":"formal-ai",
    "messages":[{"role":"user","content":"Create a file named hello.txt with the content hello world"}],
    "tools":[
      {"type":"function","function":{"name":"write","parameters":{"type":"object","properties":{"filePath":{"type":"string"},"content":{"type":"string"}}}}},
      {"type":"function","function":{"name":"read","parameters":{"type":"object","properties":{"filePath":{"type":"string"}}}}}
    ]
  }'
```

**Observed:** the assistant message contains `tool_calls: ["read"]` targeting `hello.txt`.
**Expected:** a `write` tool_call creating `hello.txt` with content `hello world`.

With only a `write` tool advertised (no `read`), the same request instead returns prose that begins **"I can read `hello.txt` when the client advertises a file read tool or a shell tool."** — i.e. the planner has classified a *write* request as a *read* intent.

End-to-end, this is why `with-formal-ai --non-interactive opencode "Create a file named hello.txt ..."` finishes with `File not found: .../hello.txt` and writes nothing.

**Confirmed live across CLIs.** In a 300-run matrix (5 CLIs × 6 tools × 10 phrasings), file-creation/edit requests that emitted a tool_call emitted the *wrong* one — a read — for every CLI that hit the case: `write → read` (agent, opencode) and `write → read_file` (qwen, gemini); likewise `edit → read` / `edit → read_file`. Only **1 of 50** write runs across all CLIs actually created the file.

## Impact

File creation/writing via natural language is broken for every CLI: the request is routed to the read path, so the file is never created and the CLI reports a missing file.

## Suggested direction

Classify create/write/save/generate-file intents as the `write` capability and emit a `write` tool_call (path + content) when a write tool is advertised; never route a file-creation request to `read`.

