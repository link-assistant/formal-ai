# Issue 845 acceptance traceability

| Requirement | Implementation | Verification |
| --- | --- | --- |
| Probability is relative to a named formal system. | `FormalSystem`; `Context::with_formal_system`; every report carries the name and content id. | `probabilities_are_relative_to_a_named_formal_system` |
| Refutation precedes support and recurses. | `FactChecker::refute_recursive` records direct disproof, negation disproof, then dependency decomposition. | `recursive_verification_is_disproof_first_and_reports_a_counterexample` |
| Recursion is configured, not hard-coded. | `FactChecker::from_solver_config` consumes `max_decomposition_depth`. | `recursion_bound_comes_from_solver_config` |
| Refutations expose counterexamples. | Arithmetic `ProofOutcome::Disproven` is retained in `RefutationAttempt` and `StatementVerification`. | recursive unit test and public-boundary integration test |
| Support uses source tiers and ignores reposts. | Existing `RelativeEvidence` is preserved; proof evidence is labelled; RML remains the sole weighting kernel. | `support_fallback_uses_source_tiers_and_marks_prior_only_unknowns` |
| Dependents recalculate together and visibly. | Batch evidence application calls one JTMS fixpoint; `RecalculatedLink` records every consulted edge. | `recalculation_trace_names_every_dependency_link` |
| Missing dependencies are not evidence. | `build_statement_report` admits dependency evidence only when the target exists in the context. | `a_dangling_dependency_does_not_turn_a_prior_into_evidence` |
| Current dialogue is the default scope. | `AuditScope::default()` and the live solver handler use `CurrentDialogue`; the request itself is excluded. | `current_dialogue_is_the_default_scope_and_audit_enumerates_every_statement`; live integration tests |
| General memory requires explicit, logged permission. | Canonical `GeneralMemoryPermission` gates audit and commit; append-only events record the boundary. | `general_memory_requires_recorded_permission_and_commit_uses_the_same_gate` |
| Whole-context audit enumerates every statement. | `FactChecker::audit_context` iterates the context's sorted statement map. | current-dialogue unit test and integration test |
| Unknowns distinguish prior from measurement. | `ProbabilityBasis::PriorOnly` accompanies an unchanged declared prior. | support-fallback and dangling-dependency tests |
| Identical input replays byte-identically. | Sorted maps/sets, snapped `TruthValue`, stable ids, and deterministic Links Notation. | `identical_context_and_evidence_replay_byte_identically` |
| Fabricated placeholder evidence cannot contribute. | Known placeholder/fabrication labels are removed before recalculation and reported as rejected. | `fabricated_source_links_are_excluded_from_the_probability` |
| The feature is user-reachable in supported languages. | Shared meaning role, contextual Rust handler, and browser-worker mirror cover en/ru/hi/zh. | `solver_fact_checks_every_current_dialogue_statement_in_every_language`; `issue-845.spec.js` |
| Offline audit does not invent fetched provenance. | Runtime and worker paths admit local proof/caller evidence only and log no source/fetch/cache claim. | Rust evidence-link assertion and browser external-request assertion |
| No neural inference. | Proof outcomes and relative-meta-logic arithmetic are the only evidence/probability path. | source review plus complete unit/integration suites |
