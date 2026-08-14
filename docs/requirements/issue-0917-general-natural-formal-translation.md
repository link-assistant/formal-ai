## Issue #917 General Natural-Formal Translation

Issue [#917](https://github.com/link-assistant/formal-ai/issues/917) makes a
formal language a first-class concrete syntax of the semantic meta language.
Natural and formal statements therefore share one meaning identity and use
seed-defined projections instead of direct language-pair translators.

| ID | Requirement | Status |
| --- | --- | --- |
| R917-1 | A statement in every registered seed language must translate to at least one formal target and back without changing meaning. | `every_seed_language_round_trips_through_a_seeded_formal_target` covers English, Russian, Hindi, Chinese, and Spanish through FOL and requires the stable meaning `statement:P31(Q89,Q3314483)`. |
| R917-2 | Formal targets, natural word order, and canonical relation surfaces must be seed-defined projections of one semantic statement, not per-pair translators. | `data/seed/formal-language-projections.lino` defines the projection catalog; `src/translation/formal_statement.rs` interprets it as one parser and one renderer per syntax. |
| R917-3 | The issue #526 round-trip contract must extend to every new natural/formal pair. | `specification::translation_round_trip::every_seed_language_round_trips_through_first_order_logic` checks every natural-to-FOL-to-natural path through the same Wikidata-grounded predicate and entity roles. |
| R917-4 | Native and browser surfaces must expose the same natural-formal translation behavior. | `whole_task_translation_uses_the_formal_projection_in_both_directions` covers the native engine; `tests/e2e/tests/issue-917.spec.js` covers both directions for every seed language through the Rust-to-WASM worker. |
| R917-5 | Adding another formal target or natural projection must remain a data change. | The projection interpreters use the catalog's formal alias, statement template, word order, and relation surface; no source/target pair table is present. |
| R917-6 | Issue, PR, related-work, research, requirements, plan, architecture, roadmap, and release evidence must remain traceable. | `issue_917_case_study_and_release_metadata_are_traceable` guards `docs/case-studies/issue-917`, the root documents, raw snapshots, and the minor changelog fragment. |
| R917-7 | At least one of five independently reviewed leaves must be authored through the real Formal AI/Agent CLI loop and reproduced byte-for-byte. | Session `ses_01c33a95effeAcU4AdF9Ec66Wr` authored the formal-projection invariant; `issue_917_agent_cli_authorship_leaf_is_byte_exact_and_reproducible` and `experiments/issue_917_agent_cli.sh` guard the artifact and raw evidence. |
