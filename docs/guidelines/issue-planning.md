# Issue planning

How open work is recorded on GitHub, so that a person or Formal AI can read
from the issue list alone what blocks the vision, in which order, and why.
Written for [issue 1548](https://github.com/link-assistant/formal-ai/issues/1548);
the helpers named below are in `experiments/issue-1548/`.

## Every open issue has a type

Each open issue carries one native GitHub issue type and the label of the same
meaning: `Bug` with `bug`, `Feature` with `enhancement`, `Task` with
`documentation` or `enhancement`, whichever names the work. An issue without a
type is not planned yet.

```bash
bash experiments/issue-1548/set-issue-type.sh link-assistant/formal-ai 1234 Bug bug
```

## Title and body

The title starts with the type and the priority, then states the outcome:
`[Bug][P0] …`, `[Feature][P1] …`. The body has these sections:

- `**Type:** … · **Priority:** …` on the first line.
- **Problem** — what was observed, with the command, file and line, pull
  request or comment that shows it. A number is quoted with the file that
  records it.
- **Outcome** — what is true when the issue is closed.
- **Implementation steps** — an ordered list; each step is small enough to be
  verified on its own.
- **Acceptance** — a checklist of observable results. A check that only counts
  recorded events does not accept a behaviour.
- **Dependency contract** — `Blocked by` and `Blocks`, the same relations that
  are published natively.

## Dependencies are native relations

A dependency written only in prose is not a dependency. Publish it:

```bash
# the first issue is blocked by the second
bash experiments/issue-1548/add-blocked-by.sh 'link-assistant/formal-ai#1554' 'link-assistant/formal-ai#1550'
# the third argument becomes a sub-issue of the second
bash experiments/issue-1548/add-sub-issue.sh link-assistant/formal-ai 1561 1550
```

- An issue has one parent. When a blocker already belongs to another parent,
  link it to the second parent as `Blocked by` and leave it where it is.
- The relations form no cycle. After a batch of changes, fetch the graph and
  check it:

```bash
bash experiments/issue-1548/fetch-issue-graph.sh link-assistant formal-ai /tmp/graph.json
```

- An issue with no open blocker can start now. `Hive Mind draft` schedules
  from these relations.

## One parent issue for critical blockers

The critical blockers of the vision — coding by the recursive meta algorithm
and general question answering — are collected in
[#1561](https://github.com/link-assistant/formal-ai/issues/1561). A new issue
that blocks either of them becomes its sub-issue, or its native blocker when it
has another parent. An open issue that is not critical is listed there under
"Open issues that are not part of this plan", with the list kept current.

A parent issue has no change of its own; it carries the label
`no-formal-ai-draft` so that no draft is started for it.

## Status wording follows the ledgers

A status in `README.md`, `VISION.md`, `ROADMAP.md` or a requirement shard says
`Built`, `Done` or `Delivered` only for what a ratchet or a test measures as
delivered on the path that produces the answer. When the steps are recorded but
do not produce the answer, the status is `Partial`, and the sentence names the
issue that owns the rest. A status that a ledger contradicts is corrected in
the same pull request that finds it.

## Defects of the orchestrator

Formal AI is used by Hive Mind as any other provider of an OpenAI-compatible
API. A defect on the orchestrator side is reported in
[link-assistant/hive-mind](https://github.com/link-assistant/hive-mind) in
terms that hold for every provider, after a search for an existing report. A
report does not ask for special treatment of Formal AI. A defect of Formal AI
that the orchestrator only relays is reported here.

## Failed attempts are evidence

A draft of Formal AI that produced no useful change stays visible: its pull
request, its summary and the reason are recorded with the issue that would
advance it. The inventory of 2026-11-10 is in
[the case study of issue 1548](../case-studies/issue-1548/README.md).
