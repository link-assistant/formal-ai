## Summary

`retryTransientFailure` waits at least `HIVE_MIND_MIN_TRANSIENT_ERROR_DELAY_MS` (3 minutes) before retrying a "Socket/connection closed unexpectedly" error. `createTransientRetryBudget` only checks that wait against its 12-hour outage budget. It does not check it against the time the run has left. So under a hard deadline, `solve` spends minutes sleeping, and it can start a sleep that cannot finish before the deadline.

## Evidence

Run: link-assistant/formal-ai Hive Mind draft run [43738765561](https://github.com/link-assistant/formal-ai/actions/runs/43738765561). The `formal-ai-draft.mjs` attempt ran under a 1200 s deadline in `konard/hive-mind:2.40.5`.

| Time (UTC) | Event |
|---|---|
| 17:19:12 | `⚠️ Socket/connection closed unexpectedly detected. Retry 1 in 3 min (session preserved)` |
| 17:26:29 | the same message again; another 3-minute sleep starts |
| 17:28:43 | the deadline interrupts the sleep; the attempt ends with exit code 124 |

Two 3-minute sleeps took 6 of the 20 minutes. The second sleep could not finish before the deadline. The upstream reset itself is the agent's 255 s idle timeout (filed separately in link-assistant/agent).

## Reproduction

Run `solve` under `timeout 600` with a tool whose session call fails with `ECONNRESET` after 30 s. The log shows `Retry 1 in 3 min` and then the wait. The process is killed during the second wait, without a third attempt.

## Workaround

None from the calling workflow: `formal-ai-draft.mjs` forwards only `GH_TOKEN` and `FORMAL_AI_INSTALL_GRANT`, and the run-context variables, into the container. So `HIVE_MIND_MIN_TRANSIENT_ERROR_DELAY_MS` cannot be lowered for a draft.

## Suggested fix

1. Let `solve` take a wall-clock deadline: a `--deadline-at` flag, or an environment variable that `formal-ai-draft.mjs` sets from its own budget.
2. In `createTransientRetryBudget().evaluate`, refuse a delay that would end past that deadline, using a new reason `deadline`, so the run fails fast with a clear message.
3. Alternatively, cap `minDelayMs` at a fraction of the remaining time when `HIVE_MIND_EPHEMERAL_CONTAINER=1`.
4. Forward that variable from `buildDockerArgv` in `scripts/formal-ai-draft.lib.mjs`.
