# Wave F — Formal AI used as its own sub-sub-agent

This directory is the evidence half of **plan 14 wave F**
(`docs/case-studies/issue-1138/plans/14-implementation-order.md`). Formal AI is
given real tasks through the external `@link-assistant/agent` CLI and through its
own `chat` entry point, and **every transcript is committed whether the run
succeeded or not**.

## The binding rule

`docs/case-studies/issue-710/plans/07-prerequisite-discovery-bridge.md:80-81`:

> Do not install a compiler manually and count that as the system's recovery. Do
> not count source-token coverage or a read-back plan as executed semantics.

Nothing in this directory was repaired by hand and counted as the system's own
work. Where a run failed, the failure is recorded as it happened, a case is added
to `data/benchmarks/self-use-*.lino`, and a test in
`tests/unit/issue_1138_self_use_*.rs` asserts the behaviour the plans say the
system must have. Those tests are observed **failing** before they are committed
and are never weakened to make them pass.

## How a run is produced

Two harnesses, both in this directory:

| script | what it drives | why both |
| --- | --- | --- |
| `run_self_use_batch.sh` | the real `@link-assistant/agent` CLI against a local `formal-ai serve --agent-mode` | the client-level observation: what a third-party agentic CLI actually receives, including tool calls and the files it writes |
| `probe_chat.sh` | `formal-ai chat` (`solver::solve`, the same entry point the HTTP surface uses) | the library-level observation: the answer with no external CLI in the loop, which is what a unit test can assert |

Both read the same tab-separated case files under `cases/`, so the two halves of
an observation always come from the identical prompt string.

```sh
PORT=8911 bash docs/case-studies/issue-1138/self-use/run_self_use_batch.sh \
  docs/case-studies/issue-1138/self-use/cases/<file>.tsv \
  docs/case-studies/issue-1138/self-use
bash docs/case-studies/issue-1138/self-use/probe_chat.sh \
  docs/case-studies/issue-1138/self-use/cases/<file>.tsv \
  docs/case-studies/issue-1138/self-use
```

The server runs with `FORMAL_AI_AGENT_MODE=1`, `FORMAL_AI_TRACE_REQUESTS=1`, a
private empty memory per batch and `FORMAL_AI_DREAMING=0`, so one run cannot
teach the next one the answer.

## What each task directory holds

```
<task-slug>/<lang>/
  prompt.txt             the prompt verbatim
  run.env                binary version, commit, agent exit code, elapsed, POST count
  agent.log              the raw Agent CLI transcript, unedited
  answer.txt             the assistant text and tool calls extracted from agent.log
  chat-answer.txt        the same prompt through `formal-ai chat`
  server-tail.log        the Formal AI server trace for the run
  workspace-listing.txt  what the CLI left in its throwaway workspace
```

## Outcome classes

| class | meaning |
| --- | --- |
| `solved` | the task was actually done, and the evidence shows it was done rather than described |
| `honest_refusal` | the system declined and named what it lacked |
| `honest_refusal_without_a_trail` | it declined honestly but named no source it consulted, so the refusal cannot be distinguished from a search that never ran |
| `wrong_answer` | it answered, and the answer is wrong or is about something other than the question |
| `silent_unknown` | it produced neither an answer nor a refusal |
| `crash` | the run did not complete |

## Index

See `outcomes.md` for the per-task table, and the per-family README beside each
task directory.
