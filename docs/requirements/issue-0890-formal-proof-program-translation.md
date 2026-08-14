## Issue #890 Formal Proof Program Translation

Issue [#890](https://github.com/link-assistant/formal-ai/issues/890) closes the
gap between solving a formal constraint and presenting its proof in an
executable programming language. The proof is a semantic value first; prose and
program syntax are projections of that value.

| ID | Requirement | Status |
| --- | --- | --- |
| R890-1 | A formal proof must be represented independently from natural-language prose and programming-language syntax. | `FormalProof::IntegerInterval` stores bounds, inclusivity, satisfiability, and a witness; `proof_meaning_is_independent_from_its_programming_language_presentations` verifies canonical statement round trips and renderer independence. |
| R890-2 | The same solved proof must translate to at least two programming languages through the general code meta-language path. | `CodeMeaning::FormalProof` formalizes once and `render_code_meaning` projects through `data/seed/proof-program-templates.lino` to Rust or Python without a direct source-target pair; `same_solved_proof_uses_general_translation_path_for_two_targets` checks both targets share one `meaning:` link. |
| R890-3 | Generated proof programs must compile or execute where the environment supports it. | `generated_proof_programs_compile_and_execute` compiles the Rust renderer with `rustc`, executes it, runs the Python renderer with `python3`, and requires both witnesses to print `2`. |
| R890-4 | Every registered supported natural language must be able to request proof translation. | `every_registered_natural_language_can_request_proof_translation` covers and compares the live `supported_languages()` registry: English, Russian, Hindi, and Chinese. |
| R890-5 | The native and browser surfaces must expose the same proof-translation behavior, with a whole-task regression. | `whole_issue_890_workflow_solves_translates_and_executes` covers the native composition; `tests/e2e/tests/issue-890.spec.js` covers Rust and Python plus all registered natural-language request forms in the browser worker. |
| R890-6 | Issue, PR, related-work, online-research, requirement, plan, and release evidence must remain traceable in the repository. | `issue_890_case_study_and_release_metadata_are_traceable` guards `docs/case-studies/issue-890`, this matrix, architecture, roadmap, and the minor changelog fragment. |
| R890-7 | At least one of the five reviewed implementation leaves must be authored through the real Formal AI/Agent CLI loop and reproducible byte-for-byte. | Session `ses_03b44e557ffeSQeAuCYzxfc3BR` authored the proof invariant leaf, captured under `docs/case-studies/issue-890/agent-cli-evidence`; `agent_cli_authorship_leaf_is_byte_exact_and_reproducible` guards the artifact, session, raw stream, and replay script. |
