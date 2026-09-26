## Issue #1138 Learning Effects

Learning is accepted only when promotion changes held-out execution without a
regression. The contract is covered by
`rust/tests/unit/issue_1138_learning_ratchet.rs` and
`rust/tests/unit/issue_1138_learned_items_change_answers.rs`.

| ID | Requirement | Status / evidence |
| --- | --- | --- |
| R1138-B7-1 | An adopted learned item executes in live dispatch and its use is named in the trace. | Covered by `rust/tests/unit/issue_1138_learned_items_change_answers.rs`. |
| R1138-B7-2 | Adoption requires an improved before/after execution-record pair on held-out prompts in five languages with zero regressions. | Covered by `rust/tests/unit/issue_1138_learning_ratchet.rs`. |
| R1138-B7-3 | The human gate reviews a draft pull request carrying the seed edit; inertness is not treated as approval. | Covered by `rust/tests/unit/issue_1138_learning_ratchet.rs`. |
| R1138-B7-4 | A forgotten cache payload is refetched on demand, and a hash divergence is reported rather than substituted. | Covered by `rust/tests/unit/issue_1138_learning_ratchet.rs`. |
