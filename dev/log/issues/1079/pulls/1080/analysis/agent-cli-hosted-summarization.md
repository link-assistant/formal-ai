# D11 — the Agent CLI E2E jobs failed on a session *title*

The basis for defect D11. Everything below was measured; the reproduction is
committed at `experiments/issue_1079_agent_compaction_flag/` and runs offline
in about twelve seconds.

## What CI showed

`Proactive failure report E2E`, run 34083226911, job 101622336256, step
`Failed command proactively offers an issue report`
(`../ci-logs/job-101622336256-agent-cli-failure-report.tail.log`):

```
2026-09-07T04:32:30.2855326Z experiments/agent_cli_e2e/run_issue_864.sh
...
2026-09-07T04:32:32.2836030Z ##[error]Process completed with exit code 1.
```

Two seconds, and not one line saying what went wrong. That is the whole
signal the job log carries: this was a **false negative about its own
failure** -- the step is red, and the log a maintainer reads first cannot
distinguish "the feature under test regressed" from "an unrelated network
call failed".

The diagnosis is in the uploaded artifact instead, which the same step
publishes as `issue-864-agent-cli-evidence`
(`../ci-logs/issue-864-artifact/agent-stderr.log`, the complete file):

```json
{"type":"error","errorType":"UnhandledRejection",
 "message":"Error from provider (Console): Request is missing x-opencode-session and cannot be routed efficiently. Please see https://opencode.ai/docs/go/#where-can-i-use-it",
 "data":{"error":{"type":"MissingSessionID"}}}
```

The harness under test never talks to `opencode`. Its provider is this
repository's own `formal-ai` binary, served locally.

## Where the hosted call comes from

Two upstream defaults compose:

1. `--summarize-session` defaults to `true` in `@link-assistant/agent`, so
   every session is summarized to produce a title.
2. `--compaction-model same` -- which every harness here passed -- has no
   effect, so summarization resolves against the head of the *default*
   compaction cascade, `opencode/big-pickle`.

Point 2 is the part that took a mock provider to prove. In 0.26.1's
`src/cli/model-config.js:383-455`:

```js
  const compactionModelsArg =
    cliCompactionModelsArg ??
    argv['compaction-models'] ??
    defaultCompactionModels;
  const modelNames = parseLinksNotationSequence(compactionModelsArg);
  if (modelNames.length > 0) { ... return ...; }
  // Fallback to single --compaction-model      <-- unreachable
```

`defaultCompactionModels` is never empty, so the cascade branch always returns
and the single-model branch below it is dead code. The flag is parsed and
appended to the default cascade rather than replacing it.

Measured against a stdlib-only provider on `127.0.0.1` that is the *only*
provider in the config -- so any other model name is unambiguously not
something the operator asked for:

| invocation | cascade the CLI logged | provider the summarizer used |
| --- | --- | --- |
| `--compaction-model same` | `["opencode/big-pickle","kilo/minimax-m2.5-free","same"]`, `"source":"default"` | `opencode` / `big-pickle` |
| `--compaction-models "(same)"` | `["same"]`, `"source":"cli"` | `mock` / `mock-model` |

## Why a failed title is a failed run

`SessionSummary.summarize()` is called without `await` and without `.catch()`
from `src/session/prompt.ts:836` and `src/session/processor.ts:511`, and the
title `generateText` inside it (`src/session/summary.ts:183`) is the one call
in that file with no guard -- `grep -c 'try {' src/session/summary.ts` is `0`,
while the description pass at line 252 and both `Provider.getModel` calls do
have `.catch`. The rejection therefore reaches
`process.on('unhandledRejection')` in `src/index.js:126-153`, which calls
`process.exit(1)`.

The third case of the reproduction shows the consequence with the summarizer
pointed at a provider that refuses it:

```
== fatal: the summarizer was refused, exit=1
"errorType":"UnhandledRejection","message":"mock summary failure"
  no "type":"result" event: the turn was aborted mid-stream
```

Not merely a wrong exit status on completed work -- the in-flight turn is
killed.

## Why it looked like flakiness

Whether the run dies is a race between the summary's rejection and the turn's
own completion. A turn that finishes first exits 0 with the summary still in
the air; a turn still streaming is aborted. Local runs of the same harness are
short and often win the race; CI runs are minutes long and always lose it. The
hosted gateway's `MissingSessionID` response is itself intermittent -- the same
reproduction from a different network answered normally and generated the title
`"Saying OK"` -- which is why this presented as an unstable job rather than a
broken one.

The reproduction removes both sources of nondeterminism deliberately: the mock
answers `400` (not retried, so the rejection lands promptly) and holds the
stream open for `MOCK_STREAM_DELAY` seconds (default 5) so the session is still
running when it does.

## The three places the defect lived here

The issue's instruction was to apply a fix everywhere it belongs, so the sweep
was for *every* path that launches the CLI, not only the failing job:

| Site | Count | Fix |
| --- | --- | --- |
| Shell harnesses under `experiments/` reachable from a workflow | 25 | `--no-summarize-session --compaction-models "(same)"` |
| Workflow jobs that reach the CLI indirectly (`proactive-failure-report-e2e.yml`, `issue-1028-agent-ladder.yml`) | 2 | job-level `LINK_ASSISTANT_AGENT_SUMMARIZE_SESSION: "false"` |
| `data/seed/client-integrations.lino` — the **product** path, consumed by `src/client_integrations.rs:592-599` for every `formal-ai with agent …` | 1 | `no_summarize_args` now carries the plural spelling |

The third is the one a fix limited to the red job would have missed: it is not
CI at all, it is what this repository ships. Every user of `formal-ai with
agent` was passing the ineffective flag.

Each site is pinned by a test in `tests/unit/ci-cd/issue_1079.rs`:
`every_agent_cli_harness_ci_runs_keeps_the_session_local`,
`every_ci_job_that_launches_the_agent_cli_disables_hosted_summarization`, and
`the_wrapper_pins_agent_compaction_to_the_session_model`. The first was proved
red before the fix: `git stash push -- experiments/` and re-running panics at
`issue_1079.rs` naming `run_agent_cli.sh`.

## Filed upstream

Both halves, with the reproduction, the workaround and a code-level fix:

- [link-assistant/agent#303](https://github.com/link-assistant/agent/issues/303)
  — `--compaction-model` never takes effect.
- [link-assistant/agent#304](https://github.com/link-assistant/agent/issues/304)
  — a failed session summary is an unhandled rejection.

The reproduction script exits non-zero when either defect stops reproducing, so
this repository will notice the day they are closed and can drop the
workaround.
