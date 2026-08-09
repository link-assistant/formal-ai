# Issue 873 requirements and solution map

The issue text and all comments were captured on 2026-08-08. There were no
issue comments. Requirements are split at the smallest independently testable
boundary.

| ID | Requirement | Implemented solution | Proof |
| --- | --- | --- | --- |
| R873-1 | An unresolved input must trigger research and still yield an answer. | General native unknown routing enters the existing grounded search/fetch/answer state machine. | `online_solver_researches_an_unknown_instruction`, `agentic_client_searches_an_unknown_instruction_without_question_punctuation`, `researched_unknown_returns_a_grounded_answer_after_search_and_fetch`. |
| R873-2 | Prefer reachable external-world data, including internet and local observations, over memorizing recomputable payloads. | `SourceReceipt` accepts any locator, separates content identity from cache payload, and retains provenance after eviction. Existing local and web tools provide the observations. | `external_payloads_are_disposable_but_receipts_are_versioned`. |
| R873-3 | Recomputable evidence can be removed and collected again. | `evict_source` drops only a recomputable payload; `recollect_source` restores only an identity-matching capture, while changed content gets a new receipt. | Same source lifecycle test. |
| R873-4 | Memory is versioned; every tested stable state can be recovered; failed compilation continues from the prior stable version. | Parent-linked `KnowledgeVersion` history, candidate/stable/rejected states, a separate active pointer, and `recover_stable`. | `failing_candidate_never_replaces_the_previous_stable_version`, `tested_version_promotes_and_any_prior_stable_version_can_be_recovered`. |
| R873-5 | Most tests are immutable and a new version cannot activate unless the complete baseline passes. | Promotion requires all results to pass, every configured baseline id to be immutable, and immutable gates to be a strict majority. | `mutable_or_incomplete_test_suites_cannot_promote_memory` plus promotion/rejection tests. |
| R873-6 | Every error produces recovery rather than a terminal stuck state. | `recover_from_error` accepts every error id and supplies `restore_stable_and_research` when no alternative was planned. | `per_command_mode_requires_permission_and_every_error_has_a_recovery`. |
| R873-7 | Ambiguous recovery asks the user; full-trust mode weighs prior advantages and disadvantages and selects automatically. | `AskOnAmbiguity` returns ranked option ids; `FullTrust` applies a deterministic score over successes, failures, advantages, and disadvantages. | `ambiguous_recovery_asks_the_user_and_full_trust_uses_outcome_history`. |
| R873-8 | Prevent endless loops with a configurable one-hour default; return the current plan and ask before continuing. Support autonomous and per-command modes. | Shared 3,600-second default for the lifecycle and external orchestration; `check_time_limit` returns `AwaitingContinuation { current_plan }`; continuation requires an explicit call. Modes are `AskOnAmbiguity`, `FullTrust`, and `PerCommand`. | `default_one_hour_limit_returns_the_current_plan_for_continuation`, `orchestration_uses_the_same_configurable_one_hour_default`. |
| R873-9 | Use one general meta-algorithm that can append/generalize itself. | A single reducer versions facts, procedures, and `MetaAlgorithm` records; its ordered phase recipe is Links Notation data and is itself eligible for proposal, verification, promotion, and rollback. | `one_data_recipe_drives_the_cycle_and_can_itself_be_versioned`, `cycle_history_is_hash_linked_and_rendered_for_replay`. |
| R873-10 | Compile a deep case study with all requirements, online facts, existing components, possible solutions, and plans. | This directory contains the issue snapshot, empty comments record, requirements, alternatives, research, implementation analysis, and self-application traces. | Documentation/source checks and review. |
| R873-11 | Plan and execute all work in one pull request. | Code, tests, data, example, changelog, case study, and evidence are contained in PR #983. | PR diff and CI. |

## Alternatives considered

### Only broaden the two research conditions

This fixes R873-1 but leaves no uniform model for evidence eviction, stable
memory versions, gated promotion, recovery policy, or continuation. It was used
as the routing fix, not treated as the full solution.

### Store every fetched response in learned memory

This makes replay easy but violates R873-2 and R873-3. A durable receipt plus an
optional payload preserves provenance without turning the memory into a web
cache. Identity-checked recollection also detects external change.

### Mutate the active memory in place and restore a backup on failure

That exposes an unverified candidate and makes partial failure ambiguous. The
chosen design appends the candidate first and changes only the active stable
pointer after the complete gate, equivalent to a small transactional commit.

### Retry every error automatically

Blind retry can repeat an invalid plan forever and ignores genuine ambiguity.
The chosen reducer makes recovery explicit: deterministic single-option
continuation, user choice for ambiguity, outcome-weighted full trust, or
per-command permission. The configurable time boundary returns a resumable plan.

### Add Temporal, a database, or an agent framework

Those systems provide valuable precedents but would duplicate the repository's
existing event log, stable ids, source receipts, promotion gates, and agent
orchestration. The small reducer expresses the missing invariant without a new
runtime, service, migration, or license surface.

## Execution plan and result

1. Capture the issue, all comments, PR metadata, recent related work, repository
   policies, and primary external sources. **Complete.**
2. Reproduce native drift with exact unknown imperatives. **Complete.**
3. Generalize direct and agentic routing while retaining the offline boundary.
   **Complete.**
4. Add the data-defined, append-only research/learning/recovery reducer.
   **Complete.**
5. Prove every requirement with focused regression tests and an example.
   **Complete.**
6. Have Formal AI and the real Agent CLI author one smallest same-task leaf.
   **Recorded in `self-hosting-authorship/`.**
7. Run local release gates, review the complete diff, update PR #983, and verify
   fresh CI. **Tracked in the PR.**
