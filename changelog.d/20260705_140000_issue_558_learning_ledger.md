---
bump: minor
---

### Added
- Issue #558 auto-learning (R558-03): `src/learning_ledger.rs` is the single, human-gated promotion protocol that terminates the self-healing loop. `LearningLedger::promote` records a `RepairCase` as a durable *approved learning record* only when **both** the benchmark gate is green **and** a human approves, and refuses every other case with a specific reason (`TestsNotGreen`, `NoReviewableProposal`, `SourceNotFaithful`, `HumanDeclined`, `AlreadyPromoted`). A repeated failure is then answered from the ledger instead of re-derived — the concrete payoff of "auto learning".
- Seventh agentic recipe (`src/agentic_coding/ledger.rs`): the promotion ledger is reachable through the agentic interface, emitting the approved learning record as a Links Notation document (`learning-ledger.lino`). The document records an already-approved decision, so nothing new is adopted and the recompile-and-reattach guardrail stays human-gated.
- `dump_learning_ledger` example prints the canonical approved ledger; `data/meta/learning-ledger.lino` is the generated, byte-for-byte-pinned artifact.
