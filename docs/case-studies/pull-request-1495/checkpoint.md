# PR #1495: evidence to useful edits

The current work addresses [#1483](https://github.com/link-assistant/formal-ai/issues/1483),
the first unfinished item in the latest [#1170](https://github.com/link-assistant/formal-ai/issues/1170)
plan. [Requirements](../../requirements/issue-1483-evidence-to-useful-edit.md)
record the complete acceptance boundary. This checkpoint is partial, not a
claim that #1483 or general self-coding has been delivered.

## Observed failure and repair

The original PR contained only a marker and development logs. Its source build
succeeded, but actual ordinary Hive drafting returned planned_not_executed and
the independent useful-change classifier rejected it. That failed attempt is
preserved in the PR history. A read-only work-item preflight similarly collected
evidence and terminated without changing the requested artifact.

A bounded local task asked Formal AI to update a setup guide from a project
manifest, preserving existing text. The original JavaScript planner read files
and stopped, or attempted an unbound replacement; no useful edit was credited.
After a manually supplied general controller repair, the unchanged task produced
Read → Read → Edit → fresh source Read → target Read. The guide paragraph derives
from a uniquely bound manifest property rather than a supplied answer sentence.
A second invocation reads the accepted artifact and makes no write. Missing or
ambiguous bindings, unavailable reads, negated/quoted-only edit requests and
source changes remain explicit gaps or failures. Independent task acceptance
remains separate from successful byte readback.

The structural controller uses the existing complete recursive source model.
Its finite interval operation changes one comparison leaf, preserves all other
source, checks a fresh full-grammar parse and proves an exact inverse. It can
construct a regression artifact from the interval contract before selecting an
edit. Synthetic interval exercises are not a genuine production defect repair.

## Verification boundary

Focused local JavaScript checks passed: five structural transformation checks,
eight documentation-controller checks, two actual planner/driver interval checks
and ten hosted-effect/transport checks. They ran one test process at a time with
a bounded JavaScript heap. The semantic kernel was translated using the pinned
JavaScript compiler and Rust text formatting only. No local Rust compilation,
Rust tests, product WASM execution, Docker or application build occurred.

Native adapters and shared-vector tests require hosted CI. The new qualification
workflow is manual, main-only and uses authenticated released current-source
binaries. Its frozen task transport keeps the already-fixed guide and private
grader outside solver access. Original documentation semantics, genuine
production logic/regression tasks, actual all-client delivery, hosted no-op,
default-branch qualification and the resulting release are still pending.

## Authorship and sequencing

Engine/controller implementations and frozen bulk generators were supplied by
coding agents and receive zero autonomous credit. Maintained JavaScript Formal
AI executed the guarded bulk applications. The source-derived fixture edits are
separate observed runtime effects; no broad task-success, amplification or token
cost reduction is inferred from them. Initial failures remain failures.

Keep one finite source batch per push after contributors finish their drafts.
Use at most three GPT-6.1-Sol medium subagents plus the coordinator, including a
Formal AI JavaScript delivery agent and a CI observer. Do not start another
#1170 task while #1483 is incomplete. Full Rust ↔ meta-language ↔ JavaScript
production translation is [#1500](https://github.com/link-assistant/formal-ai/issues/1500)
and belongs at the very end; supported shared kernels and paired behavior remain
required now.
