# Issue 890 Requirements

| ID | Requirement | Verification |
| --- | --- | --- |
| R890-1 | Represent a proof independently from natural-language prose and programming-language syntax. | `proof_meaning_is_independent_from_its_programming_language_presentations`. |
| R890-2 | Translate the same solved proof into at least two programming languages through the general code-translation path. | `same_solved_proof_uses_general_translation_path_for_two_targets` checks Rust and Python share one proof meaning. |
| R890-3 | Compile or execute generated proof programs where supported. | `generated_proof_programs_compile_and_execute` invokes `rustc`, the compiled binary, and `python3`. |
| R890-4 | Cover every registered supported natural language. | `every_registered_natural_language_can_request_proof_translation` compares its English, Russian, Hindi, and Chinese fixtures to `supported_languages()`. |
| R890-5 | Preserve native/browser behavior and verify the complete solve-to-execution composition. | `whole_issue_890_workflow_solves_translates_and_executes` and `tests/e2e/tests/issue-890.spec.js`. |
| R890-6 | Preserve issue, PR, related-work, research, requirement, plan, architecture, and release evidence. | `issue_890_case_study_and_release_metadata_are_traceable`. |
| R890-7 | Reproduce one of five reviewed implementation leaves through the real Formal AI/Agent CLI loop, with byte-exact evidence. | `agent_cli_authorship_leaf_is_byte_exact_and_reproducible` and `experiments/issue_890_agent_cli.sh`. |

## Reviewed Leaf Accounting

The smallest independently reviewed leaves are: (1) semantic proof IR,
(2) native routing and data-defined projections, (3) browser parity, (4)
requirement and E2E regressions, and (5) the proof-translation invariant
documentation artifact. The real Formal AI/Agent CLI session authored leaf 5
without a manual byte correction. That is one of five leaves, meeting the
repository's 20% floor without attributing the manually implemented program
logic to the agent.
