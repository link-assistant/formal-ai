## Issue #1138 Verifiable Task Routing

Checkable natural-language tasks share the coding discovery and execution
machinery. The contract is covered by `tests/unit/verifiable_task/mod.rs`.

| ID | Requirement | Status / evidence |
| --- | --- | --- |
| R1138-B8-1 | One shared task type represents expected answer shape, inputs, procedure needs, and checks. | Implemented by `src/verifiable_task.rs`; covered by `tests/unit/verifiable_task/identity.rs`. |
| R1138-B8-2 | Recognition vocabulary is seed-driven in English, Russian, Hindi, Chinese, and Spanish. | Covered by `tests/unit/verifiable_task/recognition.rs`. |
| R1138-B8-3 | A recognized task projects onto the shared discover-compose-execute-verify path. | Covered by `tests/unit/verifiable_task/execution.rs`. |
| R1138-B8-4 | The answer is projected only from observed execution evidence. | Covered by `tests/unit/verifiable_task/execution.rs`. |
| R1138-B8-5 | Type, range, relation, recomputation, and provenance checks are explicit and all must pass. | Covered by `tests/unit/verifiable_task/ledger.rs`. |
| R1138-B8-6 | Derivation memory stores a recomputable recipe rather than replaying the previous answer. | Covered by `tests/unit/verifiable_task/ledger.rs`. |
| R1138-B8-7 | Task categories are derived from seed meanings, not a hard-coded answer table. | Covered by `tests/unit/verifiable_task/recognition.rs`. |
| R1138-B8-8 | The no-memorization gate scans production source and seed data for benchmark prompts and answers. | Covered by `tests/unit/verifiable_task/ratchets.rs`. |
| R1138-B8-9 | Every solver generation re-measures the held-out five-language corpus and records gaps honestly. | Covered by `tests/unit/verifiable_task/rendering.rs`. |
