## Summary

The automated `solve` run on PR #927 (linked issue #905) failed on 2026-08-04 at
04:05:17Z and left behind exactly one artefact — this comment:

> **Reason**
> ```text
> AGENT execution failed with Agent reported error: [object Object]
> ```
> Logs were not attached because `--attach-logs` was not enabled.

https://github.com/link-assistant/formal-ai/pull/927#issuecomment-5174474849

Two things went wrong, one on each side:

1. **Hive Mind's fault (fixed).** The tool reported a structured error object and
   Hive Mind interpolated it into a template literal, producing `[object Object]`.
   Root-cause analysis, the nine affected call sites and the fix are in
   link-assistant/hive-mind#2141 / link-assistant/hive-mind#2143. Two upstream
   reports were filed against `@link-assistant/agent`
   (link-assistant/agent#289, link-assistant/agent#290).
2. **This repository's run configuration.** The run was started **without**
   `--attach-logs`, so when the reason turned out to be useless there was no
   second source. The container is gone; the actual cause of that 22-second
   failure is unrecoverable.

## Request

Run automated `solve` sessions on this repository with `--attach-logs --verbose`.

```bash
solve https://github.com/link-assistant/formal-ai/issues/905 \
  --tool agent --model formal-ai --attach-logs --verbose
```

Why both flags matter for Formal AI specifically:

- **`--attach-logs`** publishes the session log to the PR, so a failure leaves
  behind evidence rather than a single line in a comment. Without it, a failed
  run teaches nothing.
- **`--verbose`** is load-bearing for diagnosis: as of
  link-assistant/hive-mind#2143 the agent adapter dumps the **raw JSON** of every
  error record and every fatal startup log record only in verbose mode. That raw
  record is what survives a future payload shape the renderer does not know about.

This directly serves the self/auto-learning loop this repository is built around:
a failure whose recorded reason is `[object Object]` is unlearnable by
construction — there is nothing for the next iteration to act on. Failing fast
with a readable reason *and* an attached log is what turns a failed run into
training signal.

## Context

Full case study (timeline, root causes RC1–RC6, raw evidence):
https://github.com/link-assistant/hive-mind/blob/main/docs/case-studies/issue-2141/README.md

