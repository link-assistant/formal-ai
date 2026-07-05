---
bump: minor
---

### Added
- Issue #558 auto-learning (R558-07): `src/change_request.rs` turns a natural-language "change Formal AI itself" request into a reviewable pull request through the *same* human-gated repair loop the ledger uses. A request plus a target module becomes a `ChangeRequest` — a normalised requirement, a proposed test name, and an ordered patch plan whose target is grounded against the owned manifest (`ChangeRequest::for_module` *panics* on any path the repository does not ship, so a request can never target fabricated source). `ChangeRequest::review` merges the change only when a `BenchmarkGateReport` is green *and* an explicit `HumanApproval` is granted, refusing every other case (`TestsNotGreen` / `HumanDeclined`); neural inference stays a NON-GOAL, and the patch is a deterministic plan a human or Agent CLI executes, not generated code.
- Ninth agentic recipe (`src/agentic_coding/change_request.rs`): the user-driven self-change is reachable through the agentic interface, emitting `requested-change.lino`. Like the source-graph and explain recipes it commits no byte-pinned artifact because the target's manifest content id tracks the whole source tree.
- `request_change` example prints the canonical change request and demonstrates the accept/decline review gate.
