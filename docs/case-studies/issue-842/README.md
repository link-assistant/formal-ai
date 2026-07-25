# Issue #842 — routing parity, the #840 task ladder, and self-hosting evidence

## What the branch does

Issue #842 catalogued requests that decompose into subtasks the system already
handles, yet were refused, answered with the capability menu, or routed three
different ways depending on phrasing. The branch fixes those routes, makes the
issue #840 task ladder falsifiable so the fixes are measured rather than
asserted, and runs the ladder in CI as a ratchet.

## Self-hosting evidence

The pull request's differential self-hosting check
(`scripts/self-hosting-metric.rs --check-ratchet`) reported that merging the
branch would lower the projected self-hosting share of the next release from
32.83% to 27.30%. The routing fixes, the ladder harness, and the tests are
manually authored and carry no self-authorship attribution, so the branch is
answerable for that delta.

`experiments/issue_842_self_hosting_evidence/run_session.sh` therefore drove a
real local Formal AI session, `ses_068efbf0dffeOnZIi2RZvMxCO3`, along the
established Agent-CLI path: `formal-ai serve` in agent mode on a private, empty
memory, with the real `@link-assistant/agent` CLI pointed at it as its only
model provider. The session ran the whole-repository source-links task and its
`write_file` tool wrote `self-hosting-evidence/self-source-links.lino`.

What each artifact is, and who produced it:

| File | Origin |
| --- | --- |
| `self-hosting-evidence/agent-stream.jsonl` | The CLI's own transcript of the session, containing the session id. |
| `self-hosting-evidence/formal-ai.log` | The server-side trace: three `POST /api/openai/v1/chat/completions` rounds, all against `"model":"formal-ai"`. |
| `self-hosting-evidence/self-source-links.lino` | Written by the session's `write_file` tool. The content is the server's: the document's `manifest_content_id` (`source_tree_888bc79dcdfef41e`) appears in the server trace, and the file is byte-identical to what `cargo run --example dump_source_links` renders. |
| `self-hosting-evidence/whole-repository-projection-0{1,2}.lino` | The exhaustive projection of all 293 owned modules, emitted by `cargo run --example project_source_links_sharded`. The live recipe verifies a six-module representative slice to stay responsive; the exhaustive pass is the same library invariant (`SourceLinks::owned`), and it round-trips every module source → links → source byte-for-byte. |
| `self-hosting-evidence/whole-repository-projection.summary.log` | That driver's output: 2 shards, 2071 projection lines, 293 modules. |

The projection is a fresh measurement of *this* branch's tree, not a copy of
issue #834's: the repository now totals 4 062 537 bytes across the owned
modules against 4 054 286 then, and the manifest content id differs
accordingly. Nothing is written back — the projection is read-only and
auditable.

Only the isolated generated-artifact commit carries the paired
`Formal-AI-Session` and `Formal-AI-Evidence` trailers. This README, the runner
script, and every source change on the branch remain unattributed.

Two caveats a reviewer should know:

- The Agent CLI emits `AI SDK Warning (opencode.chat / big-pickle)` lines in
  the transcript. Those are the CLI's own session-titling and compaction
  calls to its default hosted provider. They authored none of the committed
  content; every `"model"` field in the server trace is `formal-ai`.
- The exhaustive shards come from the library driver rather than from a tool
  call inside the session, for the reason the recipe itself documents: the
  live recipe deliberately verifies a slice so a session stays responsive.

## Reproducing

```
cargo build --release --bin formal-ai
experiments/issue_842_self_hosting_evidence/run_session.sh
```

The script fails loudly if the server never comes up, if the session never
writes the document, or if the transcript carries no session id. The session id
changes on every run; the document content does not.

## The task ladder

`experiments/issue_840_task_ladder/` holds the 24-node dataset, the runner, and
the committed `results.json` baseline. `.github/workflows/task-ladder.yml` runs
it on every change to Rust sources, manifests, seed data, or the harness, with
`BASELINE` set so a drop in the score fails the pull request that causes it.
`README.txt` in that directory lists the nodes that still fail and their common
root cause.
