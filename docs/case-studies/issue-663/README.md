# Issue 663 case study — handler precedence as data Formal AI can re-derive

Issue [#663](https://github.com/link-assistant/formal-ai/issues/663) (E44 of the
[#651](https://github.com/link-assistant/formal-ai/issues/651) audit) retired the
last first-match-wins ordering still hard-coded in a Rust constant —
`SPECIALIZED_HANDLERS` in `src/solver_dispatch.rs` — into seed data. Handler
precedence is behavior, and behavior belongs in seed data ("Data Is The
Interface"), not a Rust constant.

## The two moves

1. **Precedence is data.** The ordering lives in
   [`data/seed/handler-precedence.lino`](../../../data/seed/handler-precedence.lino)
   as an ordered list of bare handler-name rows (first wins), each carrying an
   optional trailing `# …` guard note recording *why* it sits where it does.
   `src/seed/handler_precedence.rs` loads the rows; `specialized_handlers()` in
   `src/solver_dispatch.rs` joins that order with the code-side function pointers
   (`HANDLER_FUNCTIONS`) and asserts the two are an **exact permutation** — equal
   length, no duplicates, every seed name registered — so a seed edit can only
   reorder, never silently drop or duplicate, a handler.
   `tests/unit/specification/routing_precedence.rs` pins the behavior: reordering
   two rows in a fixture flips which handler a prompt routes to, while the shipped
   seed keeps today's dispatch invariants (`http_fetch` first, `incompatible_units`
   last, `numeric_list` < `arithmetic` #395, and the rest).

2. **Formal AI re-derives it itself, through its own Agent CLI.** Per
   [`CONTRIBUTING.md`](../../../CONTRIBUTING.md) ("drive the Agent CLI, never
   defer"), the precedence rationale is also a persisted associative links
   network,
   [`data/meta/issue-663-handler-precedence-learning.lino`](../../../data/meta/issue-663-handler-precedence-learning.lino),
   that Formal AI ranks into a review-gated proposal. The report is one row in the
   `REPORTS` table (`src/agentic_coding/learning_report.rs`) — data-routed, not a
   planner branch — rendered by
   `src/agentic_coding/learning_report/handler_precedence_learning.rs`.

## Evidence and reproduction

`tests/unit/issue_663_handler_precedence_learning.rs` pins three things offline:

- **Derived, not canned** — mutate the persisted network and the ranking shifts
  (`handler_precedence_learning_is_derived_and_review_gated`).
- **Formal AI authors it end-to-end** — `run_agentic_task` drives the in-process
  Agent CLI with a *differently-worded* task and the agent's `write_file` content
  equals the renderer output
  (`formal_ai_executes_the_handler_precedence_learning_task_through_agent_cli`).
- **Byte-for-byte provenance** — the committed
  [`agent-cli-evidence/handler-precedence-learning-report.lino`](agent-cli-evidence/handler-precedence-learning-report.lino)
  equals `render_document()`, so the tool — not a hand-edit — is the author and
  cannot silently regress (`committed_agent_cli_artifact_is_byte_reproducible`).

The real dual-provider Agent-CLI round-trip is
[`experiments/agent_cli_e2e/run_issue_663_learning.sh`](../../../experiments/agent_cli_e2e/run_issue_663_learning.sh),
wired into the `test-agent-cli-e2e` job in `.github/workflows/release.yml`:
`formal-ai serve` is the model provider (no external model, no API key) while the
real `@link-assistant/agent` CLI executes the differently-worded task and must
author a report that `diff -u` proves identical to the committed evidence.

## Why the phrasing differs each time

The whole-task test task, the Agent-CLI recipe task, and the seed `task` field
each use a *different* natural-language phrasing of "learn the precedence and write
the report". A passing run therefore proves the routing is general — it keys on the
artifact the request asks for, via `LearningReport::matches`, not on one memorized
sentence.
