# Issue #955 (E103) — runtime hand-check suite

## What this carries

The audit behind this epic (`handchecks.ndjson`) ended with 49 findings
that static review cannot confirm — they need the deployed demo, a real
device, a live GitHub Action, an upstream tracker, or a maintainer
decision. The issue's choice was one tracked checklist instead of 49
micro-issues, **worked through incrementally and honestly: unchecked items
stay visibly open, nothing is marked done without evidence.**

## Deliverables

| Artifact | Role |
|---|---|
| `docs/handcheck/suite.md` | The 49 checks as one table: `HC-nn`, the audit's own requirement id, the live action, and a status cell. Landing state is 49 × `pending` — the audit's state carried forward, not pre-checked boxes. |
| `scripts/handcheck-runner.rs` | The runner: lists all/pending checks, and `--check` validates the row grammar (four cells, `HC-nn` ids, status vocabulary `pending \| pass \| fail \| n/a`, a settled status citing evidence after an em dash) plus the count staying at 49. It never marks anything done — evidence is recorded by a human editing the status cell with a run URL, screenshot path, or tracking issue. Self-tests cover the grammar rejections and the committed suite's shape. |
| this file | The tracking doc: progress is the settled count the runner prints. |

## Working an item

1. Run the live action named in the row (or make the maintainer decision
   it asks for).
2. Edit the status cell: `pass — <evidence link>`, `fail — <tracking
   issue>`, or `n/a — <reason naming the retired surface>`.
3. `rust-script scripts/handcheck-runner.rs --check` stays green: the
   grammar gate is what keeps an honest `pending` differentiable from a
   lost row.

## Standing honesty rule

A row may be added only by splitting an existing check (the runner fails
if the count leaves 49 for any other reason), and a `pass` without an
evidence link is a grammar violation, not a style nit. The suite completes
when the runner prints `49 checks, 49 settled, 0 pending` — until then
every outstanding row is visible in `--pending`.
