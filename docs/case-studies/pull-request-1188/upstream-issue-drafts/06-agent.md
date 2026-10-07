<!-- repo: link-assistant/agent -->
<!-- title: A binary file in the workspace makes the session diff summary NaN: 0.26.0 exits 1, 0.26.11 drops the summary -->

### Problem

`Snapshot.diffFull` (`src/snapshot/index.ts`, 0.26.11) reads `git diff --numstat`. For a binary file git prints `-` instead of a count. The function already detects that case (`isBinaryFile = additions === '-' && deletions === '-'`). It uses the flag to skip `git show`, but still stores the counts:

```ts
      result.push({
        file,
        before,
        after,
        additions: parseInt(additions),   // parseInt('-') === NaN
        deletions: parseInt(deletions),   // NaN
      });
```

`SessionSummary.summarizeSession` (`src/session/summary.ts`) sums those values into `draft.summary`. The `z.number()` schema then rejects them:

```
[{"expected":"number","code":"invalid_type","received":"NaN",
  "path":["summary","diffs",0,"additions"],
  "message":"Invalid input: expected number, received NaN"}, …"deletions"…]
```

- **0.26.0**: the rejection is unhandled. The CLI prints `{"type":"error","errorType":"UnhandledRejection",…}` and exits 1, aborting the turn that is still running.
- **0.26.11**: the #304 guard catches it and logs `session summarization failed`, so the run exits 0. But every session that touches a binary file still loses its summary and its diff stats. The cause is the same `NaN`.

Any tool call that creates a binary file triggers it: `python3 -m py_compile` writing `__pycache__/*.pyc`, a build output, an image, a compiled binary.

### Reproduction

Any model works; the tool call is what matters. This one uses a local OpenAI-compatible provider that answers with one `bash` call.

```bash
mkdir repro && cd repro && git init -q
printf 'print("hi")\n' > greet.py && git add -A && git -c user.email=a@b -c user.name=a commit -qm init
# opencode.json pointing at any provider, then:
agent run --prompt 'Run python3 -m py_compile greet.py' --disable-stdin \
  --no-summarize-session --compaction-models "(same)" --model <provider>/<model> < /dev/null
echo "rc=$?"
```

- 0.26.0: `rc=1`, with the `UnhandledRejection` above as the last event.
- 0.26.11: `rc=0`, and the log contains `"message": "session summarization failed"` with the same Zod error.
- `Run python3 greet.py` writes no binary file and is clean on both versions.

Environment: macOS 15.7.7 (arm64), Bun 1.2.20, git 2.50.1, Python 3.12.4; `@link-assistant/agent` 0.26.0 (global install) and 0.26.11 (`bunx @link-assistant/agent@0.26.11`). The provider was a local Formal AI server (`formal-ai/formal-ai`). It was found while dogfooding Formal AI's JavaScript server through the CLI ([formal-ai PR #1188](https://github.com/link-assistant/formal-ai/pull/1188), ledger `docs/case-studies/pull-request-1188/formal-ai-dogfood.md`, client defect under T18).

### Proposal

Record a binary file with zero line counts. `isBinaryFile` is already there:

```ts
        additions: isBinaryFile ? 0 : parseInt(additions),
        deletions: isBinaryFile ? 0 : parseInt(deletions),
```

If the UI should show binary files differently, add a `binary: true` field instead of zero. Add a test: a snapshot diff over a commit that adds a binary file yields finite counts, and `summarizeSession` succeeds.

### Benefit

Sessions that build, compile or generate assets keep their title, summary and diff stats. On releases before the #304 guard, the CLI no longer exits 1 in the middle of a turn.
