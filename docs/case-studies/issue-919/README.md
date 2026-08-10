# Issue 919: researched coding procedures

Issue [#919](https://github.com/link-assistant/formal-ai/issues/919) asks Formal
AI to turn an observed coding skill gap into reusable, verified capability. The
implemented boundary is deliberately narrower than answer-oriented retrieval:
captured prose cannot execute, while a typed procedure compiled from exact
source bytes can execute only after it matches the task's expected workspace
output and receives named review.

## Reproduction and root cause

`FormalAiEngine.answer("Write a Ruby program that counts to three")` exhausts
the catalog, blueprint recipes, coding oracle, and seed idiom composer. It
correctly returns `write_program_skill_gap` with the stable identity produced by
`program_skill_gap::gap_name(Some("count_to_three"), Some("ruby"))`.

Before this change that identity was a terminal diagnostic. The repository had
all of the necessary pieces, but no procedure-oriented composition:

- `source_fetch` and `source_research` captured exact external bytes with a
  deterministic offline cache;
- `search_fusion` turned captured statements into answer context;
- `research_learning` versioned candidates, gates, and recovery;
- `workspace_change_learning` compiled and bounded verified rewrites;
- `program_skill_gap` named synthesis misses.

None moved a named coding gap through those boundaries into a durable
source-derived procedure. Search could therefore improve an answer without
improving the next equivalent coding execution.

## Implemented loop

`coding_research_learning` composes the existing primitives in this order:

1. retain the exact `program_skill_gap` identity and deterministically plan a
   language/task query;
2. discover and capture a bounded set of sources through `CachedSourceClient`;
3. accept only the versioned `Formal AI coding procedure` source shape with an
   explicit SPDX license expression, task, language, operation, and operands;
4. formalize that captured procedure into content-addressed Links Notation;
5. propose it as `KnowledgeKind::Procedure` in `ResearchLearningCycle`;
6. execute it through `execute_workspace_rewrite`, the same bounded Normal
   Markov executor used by the hand-seeded workspace-learning path;
7. promote only when exact expected output and named review both pass; and
8. retain query, source URL, declared license, fetch time, source SHA-256,
   formalization, executor, verification receipt, and reviewer in a
   content-addressed ledger.

The data-authored policy is
`data/meta/coding-research-learning-contract.lino`. A default
`CachedSourceClient` remains offline. An explicitly online client populates the
existing content-addressed source cache; a new offline client can then reproduce
the same research proposal, procedure identifier, ledger, and output without
calling its transport.

No malformed, mismatched, unverified, or unreviewed candidate enters the
ledger. A failed round is appended to the gap record with its query and reason,
and the next query is deterministically widened to `alternative evidence round
N`.

## Dependency and scope

The E69 write-effect ladder was completed by
[#916](https://github.com/link-assistant/formal-ai/issues/916) and
[#966](https://github.com/link-assistant/formal-ai/pull/966) before this work.
Issue #919 reuses the execution machinery completed by
[#897](https://github.com/link-assistant/formal-ai/pull/897), the recoverable
research cycle from [#873](https://github.com/link-assistant/formal-ai/issues/873),
and the source boundary adopted by
[#896](https://github.com/link-assistant/formal-ai/issues/896). The first learned
operation is intentionally the already verified workspace-rewrite family; new
operation kinds must add their own bounded executor and verification oracle
instead of interpreting arbitrary fetched code.

## Verification

`tests/unit/issue_919.rs` is the minimum end-to-end case. It first observes the
real Ruby skill gap, performs an opt-in fixture fetch, learns and executes a
licensed procedure, checks every required provenance field, replays the whole
research round offline from cache, restores the content-addressed ledger, and
uses the learned procedure on held-out source through the same executor. A
second regression proves a wrong execution is rejected, retained in the gap
history, and schedules round two. The contract test pins the data-authored
safety boundaries.

There were no issue comments, screenshots, or attachments when the issue was
implemented on 2026-08-10. Requirement mapping and primary external research
are preserved beside this document.
