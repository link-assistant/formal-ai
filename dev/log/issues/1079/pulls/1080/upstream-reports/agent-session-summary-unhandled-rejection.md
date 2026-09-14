# Upstream report 8 - a failed session summary is an unhandled rejection, and it kills the run

**Target:** `link-assistant/agent` (`@link-assistant/agent`, read in the
published 0.26.1 source; observed on 0.26.0 in CI)

**Severity:** summarization is a side quest -- its entire product is a session
*title* -- it runs concurrently with the turn, and it is on by default. When it
rejects, the process exits 1 immediately and the turn that was still streaming
is aborted. Verified below: the failing run emits no `"type":"result"` event at
all.

---

## Title

`SessionSummary.summarize()` is called without `await` and without `.catch()`
from two sites; a rejection inside it reaches `process.on('unhandledRejection')`
and calls `process.exit(1)`

## Affected sites

Three files, 0.26.1 (`npm pack @link-assistant/agent`).

`src/session/prompt.ts:835-840` -- fire-and-forget on the first step:

```ts
      if (step === 1) {
        SessionSummary.summarize({
          sessionID: sessionID,
          messageID: lastUser.id,
        });
      }
```

`src/session/processor.ts:511-514` -- fire-and-forget after a patch part:

```ts
                  SessionSummary.summarize({
                    sessionID: input.sessionID,
                    messageID: input.assistantMessage.parentID,
                  });
```

Neither is awaited, neither has a `.catch()`, and neither is `void`-marked, so
the returned promise has no rejection handler anywhere.

`src/session/summary.ts` -- the promise that rejects. The file guards *model
resolution* carefully and the generation not at all:

```ts
      model = await Provider.getModel(
        compactionModel.providerID,
        compactionModel.modelID
      ).catch(() => null);        // guarded
...
      const result = await generateText({   // line 183 -- not guarded
```

`grep -c 'try {' src/session/summary.ts` is `0`. The second `generateText` in
the file (line 252, the description pass) does carry `.catch((err) => {...})`;
the title pass at line 183 does not. So the shape of the bug is narrow and
specific: a provider that is reachable enough to resolve a model but errors on
the actual request produces a naked rejection.

`src/index.js:126-153` -- what that rejection becomes:

```js
process.on('unhandledRejection', (reason, _promise) => {
  hasError = true;
  ...
  outputError(errorOutput);
  ...
  process.exit(1);
});
```

## Observed failure in the wild

From a CI artifact (`agent-stderr.log`), one line, the whole file:

```json
{"type":"error","errorType":"UnhandledRejection",
 "message":"Error from provider (Console): Request is missing x-opencode-session and cannot be routed efficiently. Please see https://opencode.ai/docs/go/#where-can-i-use-it",
 "stack":"AI_APICallError: ...\n    at <anonymous> (/home/runner/.bun/install/global/node_modules/@ai-sdk/openai-compatible/node_modules/@ai-sdk/provider-utils/dist/index.mjs:2961:18)\n    at async <anonymous> (.../index.mjs:2657:34)\n    at processTicksAndRejections (native:7:39)",
 "data":{"error":{"type":"MissingSessionID"}}}
```

The only thing that failed was the title, and it was the run's exit status.

The `at async` frame with no application frame under it is the signature of the
missing handler: the rejection surfaced from the AI SDK's fetch wrapper with no
call site of ours on the stack, because there is no `await` connecting it to
one.

## How it is reached by default

Two defaults compose into it:

1. `--summarize-session` defaults to `true`, so every session summarizes.
2. `--compaction-model same` does not take effect -- see report 7 -- so the
   summarizer uses the head of the default cascade, `opencode/big-pickle`, even
   when the session's only configured provider is local.

The result is that a run against a purely local provider makes an outbound
request to a hosted gateway, and that gateway's error becomes the run's exit
status. The `MissingSessionID` response above is intermittent: the same
reproduction from a different network answered normally and generated a title.
That intermittency is the practical problem -- it makes runs flaky rather than
broken, which is much harder to attribute.

## Reproducible example

Deterministic, offline, and independent of the hosted gateway's mood: the only
configured provider is a stdlib-only HTTP server on `127.0.0.1` that answers
the *streaming* request (the turn) normally and the *non-streaming* request
(the summary) with HTTP 400.

- driver: <https://github.com/link-assistant/formal-ai/blob/main/experiments/issue_1079_agent_compaction_flag/run.sh>
- mock provider: <https://github.com/link-assistant/formal-ai/blob/main/experiments/issue_1079_agent_compaction_flag/mock_openai_server.py>

```bash
bun add -g @link-assistant/agent          # reproduced on 0.26.0
git clone https://github.com/link-assistant/formal-ai && cd formal-ai
./experiments/issue_1079_agent_compaction_flag/run.sh
```

Third case of three, ~12 seconds end to end:

```
== fatal: the summarizer was refused, exit=1
"errorType":"UnhandledRejection","message":"mock summary failure"
  no "type":"result" event: the turn was aborted mid-stream
```

Two details of the harness are load-bearing, and both say something about the
defect:

- **The mock answers 400, not 500.** A 5xx goes through `retry-fetch` and then
  three rounds of the AI SDK's exponential backoff -- about 54 seconds -- by
  which time a short turn has finished and the process has already exited
  cleanly. A 4xx is not retried, so the rejection lands promptly. The real
  failure was likewise a client error from the gateway.
- **The mock holds the stream open for `MOCK_STREAM_DELAY` seconds** (default
  5) so the session is still running when the rejection arrives.

Those two together are why this presents as flakiness rather than as a
reproducible failure: whether the run dies depends on a race between the
summary's rejection and the turn's completion. A real multi-minute turn loses
that race every time.

## Workaround

Turn the feature off. Any of:

```
--no-summarize-session
LINK_ASSISTANT_AGENT_SUMMARIZE_SESSION=false
```

Both are in use downstream: the flag at 27 wrapped invocation sites, the
environment variable at the two CI jobs that launch the CLI indirectly. Pinned
by tests, because the failure mode is invisible until a provider has a bad
minute:
<https://github.com/link-assistant/formal-ai/pull/1080>.

There is no workaround that keeps the feature *and* the exit status, which is
why this is worth fixing rather than documenting.

## Suggested fix

Attach a handler at the two call sites. `void` marks the fire-and-forget as
deliberate for the reader and for `no-floating-promises`:

```diff
-        SessionSummary.summarize({
-          sessionID: sessionID,
-          messageID: lastUser.id,
-        });
+        void SessionSummary.summarize({
+          sessionID: sessionID,
+          messageID: lastUser.id,
+        }).catch((error) => {
+          // A session title is not worth a failed run: the turn it describes
+          // has already succeeded. Log and continue.
+          log.warn(() => ({
+            message: 'session summarization failed',
+            error: error?.message ?? String(error),
+          }));
+        });
```

and the identical change at `src/session/processor.ts:511`.

Better still, defend once instead of twice: wrap the body of
`SessionSummary.summarize` so it cannot reject at all, which is what its two
callers already assume. The file's own conventions point that way -- the
description pass at line 252 has a `.catch`, and both `Provider.getModel` calls
have one; the title pass at line 183 is the outlier.

Two related hardening items:

1. **`generateText` at line 183 deserves the same `.catch` as line 252**,
   independent of the call sites, so a future third caller does not reintroduce
   this.
2. **Consider whether summarization should ever leave the session's provider.**
   Issue #217 is titled "Turn on `--summarize-session` by default with the same
   model"; if that is the intent, the default cascade should not begin with a
   hosted model, and report 7 is the mechanism that breaks it.

## Related

- <https://github.com/link-assistant/agent/issues/22> -- "Agent exits with code
  0 despite throwing errors". This is the mirror image: exits 1 despite the work
  succeeding. Both come from exit status being decided by an out-of-band handler
  rather than by the turn.
- <https://github.com/link-assistant/agent/issues/217> -- summarize-session on
  by default, "with the same model".
- <https://github.com/link-assistant/agent/issues/223> -- referenced in
  `summary.ts` as the reason the compaction model is used for summarization.
- <https://github.com/link-assistant/formal-ai/pull/1080> -- the downstream
  workaround and the reproduction harness.
