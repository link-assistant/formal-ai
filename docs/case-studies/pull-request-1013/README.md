# Pull request #1013 — complete CI/CD diagnostic audit

Pull request: <https://github.com/link-assistant/formal-ai/pull/1013>

Issue: <https://github.com/link-assistant/formal-ai/issues/1012>

The complete timeline, requirements, run classifications, template comparison,
online research, alternatives, upstream reports and verification evidence live
in the [issue case study](../issue-1012/README.md) and its
[canonical archive](../../../dev/log/issues/1012/pulls/1013/README.md).

## Initial pull-request state

PR #1013 was opened as a draft from `issue-1012-f43e3a18da30` with a generated
WIP title, placeholder body, and prepared head `d4e8488`. Conversation comments,
inline review comments and reviews were all empty. Five initial workflows were
downloaded and matched that SHA; they were green but retained the issue's
successful-run warning debt.

## Decisions

1. Preserve Pipeline Status as a true failure and fix the work boundary that
   timed out.
2. Keep the 25-minute bound and divide exhaustive macOS coverage into complete
   filter-aware slices rather than increasing the timeout.
3. Warn while work is running, because post-process telemetry cannot observe a
   killed command.
4. Apply third-party warning policies only to exact diagnostic codes and child
   processes.
5. Build the identical Box host executable once and preserve all seven language
   validations as parallel consumers.
6. Do not invent a backend root cause from sccache counters; expose an explicit
   verbose switch, disabled by default, for any remaining failure.
7. Keep all evidence in the user-requested archive and expose concise durable
   requirement and case-study entry points.
8. Use one patch changelog fragment and do not edit the crate version manually.

## Review checklist

- PR title and description replace generated WIP text and close #1012.
- All R1012 requirements map to evidence, implementation, and tests.
- Rust and JS template reports link exact reproductions and workarounds.
- No real warning/error class is globally suppressed.
- Existing public Rust exports and all seven Box language checks remain.
- Current `main` is incorporated before final verification.
- Local and remote checks match the final pushed SHA and are green.
- The worktree is clean and the PR is marked ready.
- No visual UI changed, so screenshots do not apply.
