# Issue 917 Requirements

| ID | Requirement | Verification |
| --- | --- | --- |
| R917-1 | Every registered seed language translates a statement to a formal target and back with one meaning. | `every_seed_language_round_trips_through_a_seeded_formal_target`. |
| R917-2 | Formal and natural concrete syntaxes are seed-defined projections, not pair translators. | `data/seed/formal-language-projections.lino` and the generic native/WASM interpreters. |
| R917-3 | Extend issue #526's round-trip contract to all new pairs. | `every_seed_language_round_trips_through_first_order_logic` exercises each natural-to-FOL-to-natural path through `statement:P31(Q89,Q3314483)`. |
| R917-4 | Preserve native/browser parity and verify the whole user task. | `whole_task_translation_uses_the_formal_projection_in_both_directions` and `tests/e2e/tests/issue-917.spec.js`. |
| R917-5 | Adding a concrete syntax remains a catalog change. | Projection templates, aliases, word order, and relation surfaces are interpreted from the seed. |
| R917-6 | Preserve issue, PR, related work, research, requirement, design, architecture, roadmap, and release evidence. | `issue_917_case_study_and_release_metadata_are_traceable`. |
| R917-7 | Reproduce one of five reviewed leaves with the real Formal AI/Agent CLI loop. | `issue_917_agent_cli_authorship_leaf_is_byte_exact_and_reproducible` and `experiments/issue_917_agent_cli.sh`. |

## Reviewed Leaf Accounting

The independently reviewed leaves are: (1) the seed projection catalog and
semantic statement core, (2) native engine routing, (3) Rust-to-WASM browser
parity, (4) native and browser regression contracts, and (5) the
formal-language projection invariant document. The real Formal AI/Agent CLI
session authors leaf 5 without a manual byte correction. This is one of five
leaves, meeting the repository's 20% floor without attributing the manually
implemented translation logic to the agent.
