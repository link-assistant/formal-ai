## Summary

`agent` serves `POST /session/:id/message` from `Bun.serve({ idleTimeout: 255 })` (`js/src/server/server.ts:251`). A session message holds that HTTP response open while the model works. If no byte is written for 255 s, Bun closes the socket, and the client sees `ECONNRESET`. Slow providers hit this on every long session message, even though the model is still making progress.

## Evidence

Run: link-assistant/formal-ai Hive Mind draft run [43738765561](https://github.com/link-assistant/formal-ai/actions/runs/43738765561), job 132887682467. The run used `konard/hive-mind:2.40.5` with `solve --tool agent --model formal-ai`.

| Time (UTC) | Event |
|---|---|
| 17:13:44 | solve starts |
| 17:16:41 | first provider turn: `"durationMs": 96537`; later turns take about 37–40 s each |
| 17:19:12 | `POST /session/<id>/message` fails with `ECONNRESET` at `"durationMs": 255997` |
| 17:26:29 | second `ECONNRESET` at `"durationMs": 256268` |

Both resets land at about 256 s, which is the 255 s idle limit. #109 fixed Bun's 10 s default. The 255 s ceiling is Bun's maximum and is not tracked anywhere.

## Reproduction

1. Start a fake OpenAI-compatible provider that sleeps 260 s before it answers `/v1/chat/completions`.
2. Point `agent` at it and `POST /session/<id>/message`.
3. The request fails with `ECONNRESET` after about 255 s, while the provider call is still pending.

## Workaround

None on the caller's side: the limit is fixed inside the server. Callers can only keep every provider turn well under 255 s.

## Suggested fix

Either of these:

- Write keep-alive bytes on the session-message response while the provider is working (an SSE comment, or chunked whitespace before the JSON body).
- Disable the idle timeout for that route only, using `server.timeout(req, 0)` in the handler. This keeps the #213 shutdown behaviour for every other route.

Also add a regression test that uses a provider stub slower than 255 s.
