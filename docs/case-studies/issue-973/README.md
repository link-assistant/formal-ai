# Issue 973 case study — a failed run that left no recoverable evidence

**Issue:** [#973 — Run automated solve sessions with `--attach-logs --verbose`](https://github.com/link-assistant/formal-ai/issues/973)
**Triggering failure:** <https://github.com/link-assistant/formal-ai/pull/927#issuecomment-5174474849>
**Closing pull request:** <https://github.com/link-assistant/formal-ai/pull/974>

## Timeline (from `raw-data/pull-927-comment-manifest.json`)

| UTC | Comment | What it recorded |
| --- | --- | --- |
| 2026-08-04T04:04:57Z | `5174472832` | "AI Work Session Started" on PR #927 (linked issue #905). |
| 2026-08-04T04:05:17Z | `5174474849` | **The failure.** 20 seconds later: `AGENT execution failed with Agent reported error: [object Object]`, followed by "Logs were not attached because `--attach-logs` was not enabled." |
| 2026-08-04T10:41:58Z | `5177906033` | A second, unrelated failure (Codex, cut off mid-turn) — this one at least named a condition. |
| 2026-08-04T14:27:09Z | `5180445145` | The maintainer recovers by hand from a *separately uploaded* sanitized log URL, because the PR itself held no log. |
| 2026-08-05T05:49:10Z | `5188067670` | Recovery confirmed: "the session was cut off mid-turn (`turn.started=3, turn.completed=1`) after it had already pushed and claimed the PR was ready." |
| 2026-08-05T07:12:18Z | `5188709934` | A later run **with** `--attach-logs`: a full "Solution Draft Log" comment with cost and per-sub-session token usage. |

The contrast is the whole case. The 04:05:17Z run produced one unusable
sentence; the 07:12:18Z run produced a complete execution trace on the pull
request. The container behind the first run is gone, so the actual cause of that
22-second failure is unrecoverable — permanently.

## Root causes

**RC1 — Hive Mind rendered a structured error object into a template literal**
(upstream, fixed). The tool reported an error *object*; JavaScript string
interpolation turned it into `[object Object]`. Nine call sites were affected.
Root-cause analysis and fix:
[link-assistant/hive-mind#2141](https://github.com/link-assistant/hive-mind/issues/2141)
→ [link-assistant/hive-mind#2143](https://github.com/link-assistant/hive-mind/pull/2143)
(merged, "render structured tool error payloads as readable text instead of
`[object Object]`").

**RC2 — the Agent CLI's JSON `error` events carry no human-readable message
field** (upstream, reported):
[link-assistant/agent#289](https://github.com/link-assistant/agent/issues/289)
(closed). Consumers had nothing safe to print, which is what made RC1 easy to
hit.

**RC3 — fatal startup errors exit 0 with no error event** (upstream, still
open): [link-assistant/agent#290](https://github.com/link-assistant/agent/issues/290)
— a recurrence of `agent#22`, plus `ConnectionRefused` retried for seven days.
While this is open, a fatal startup can still surface as an empty or shapeless
record, and only the raw dump makes it readable.

**RC4 — this repository's run configuration (the part we own).** The run was
started without `--attach-logs`, so when the rendered reason turned out to be
useless there was no second source. Nothing in the repository required the flag,
and the one runnable entry point we publish
(`examples/self-coding/run.sh --live`) passed `--verbose` but not
`--attach-logs`.

**RC5 — a rendered reason is a lossy projection.** Even with RC1 fixed, the
renderer only knows payload shapes that exist today. As of hive-mind#2143 the
Agent adapter dumps the **raw JSON** of every error record and every fatal
startup log record *only in verbose mode*. That raw record is what survives the
next unknown shape — so `--verbose` is diagnosis infrastructure, not noise.

**RC6 — an unlearnable failure is worse than a loud one.** This repository is
built around a self/auto-learning loop. A failure whose recorded reason is
`[object Object]` gives the next iteration nothing to act on; it is unlearnable
by construction. Failing fast with a readable reason *and* an attached log is
what turns a failed run into training signal.

Full upstream case study (timeline, RC1–RC6, raw evidence):
<https://github.com/link-assistant/hive-mind/blob/main/docs/case-studies/issue-2141/README.md>

## Prior art surveyed

- Hive Mind already supports both flags; nothing needed building upstream. The
  gap was entirely in *how this repository asks for runs*.
- The repository's existing convention for holding a process rule to a test is
  the docs-guard test family (`tests/issue_885_docs.rs`,
  `tests/unit/ci-cd/issue_846.rs`): read the repository's own text and assert the
  contract. This case study's fix follows that pattern rather than inventing a
  new mechanism.
- `scripts/detect-code-changes.rs` already treats `experiments/`, `dev/log/`, and
  `docs/case-studies/` as recorded history; the new scan reuses the same
  exemption list so evidence of past runs stays byte-for-byte as it happened.

## The fix

1. **`examples/self-coding/run.sh`** — the `--live` entry point now runs
   `solve "$2" --tool agent --model formal-ai --attach-logs --verbose`. It
   previously passed `--verbose` alone, which is exactly the configuration that
   produced the unrecoverable failure.
2. **`CONTRIBUTING.md`** — a new section, "Always run automated `solve` sessions
   with `--attach-logs --verbose`", records the canonical command, why each flag
   is load-bearing and non-substitutable, and why this is a precondition of the
   learning loop rather than a style preference.
3. **`tests/issue_973_solve_flags.rs`** — the policy is enforced, not just
   written down:
   - `the_live_self_coding_entry_point_attaches_logs_and_runs_verbose` pins the
     runnable entry point;
   - `every_published_solve_invocation_carries_both_evidence_flags` scans the
     guides and scripts the repository publishes (`docs/`, `examples/`,
     `scripts/`, `.github/`, `src/`, and the root guides), joins shell/markdown
     line continuations so a wrapped command is judged whole, ignores prose such
     as "we do not solve a task by hand", and fails on any invocation missing
     either flag;
   - `contributing_explains_why_both_flags_are_load_bearing` and
     `the_case_study_records_the_unrecoverable_failure_and_the_fix` keep the
     documentation and this evidence from silently rotting away.
4. **`REQUIREMENTS.md` / `docs/requirements-traceability.md`** — R973-1..R973-3
   record the requirement, the delivering pull request, the pinning test, and
   the falsification runs that proved each guard actually turns red.

## Verification

```bash
cargo test --test issue_973_solve_flags
```

Reverting either flag in `examples/self-coding/run.sh` or in the CONTRIBUTING
command turns `every_published_solve_invocation_carries_both_evidence_flags`
red, naming the file, line, and missing flag.

## `raw-data/`

- `raw-data/pr-927-failure-comment.json` — the complete failure comment, as
  returned by the GitHub API: the `[object Object]` reason and the "Logs were
  not attached because `--attach-logs` was not enabled." line, `created_at`
  `2026-08-04T04:05:17Z`.
- `raw-data/pull-927-comment-manifest.json` — every comment on PR #927 with its
  timestamp and whether it carried the missing-logs note or an attached log;
  the source of the timeline table above.
- `raw-data/pull-927.json` — the pull request the failed run targeted.
- `raw-data/issue-973.json` — the issue as filed.
