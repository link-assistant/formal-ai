## Issue #1138 Prerequisite Discovery

Missing tools become observable, consent-bounded prerequisite needs. The
contract is covered by `rust/tests/unit/issue_1138_prerequisite_need.rs` and
`rust/tests/unit/issue_1138_install_scope.rs`.

| ID | Requirement | Status / evidence |
| --- | --- | --- |
| R1138-6-1 | Toolchain availability is observed; an unprobed tool is `NotProbed`, never unavailable. | Covered by `rust/tests/unit/issue_1138_prerequisite_need.rs`. |
| R1138-6-2 | The check command runs before any output is called observed; unobserved output is labelled in every supported language. | Covered by `rust/tests/unit/issue_1138_prerequisite_need.rs`. |
| R1138-6-3 | A missing executable is distinguished from permission denial and ordinary compile failure by observed status. | Covered by `rust/tests/unit/issue_1138_prerequisite_need.rs`. |
| R1138-6-4 | A missing executable becomes a blocked prerequisite need until a re-probe observes it present. | Covered by `rust/tests/unit/issue_1138_prerequisite_need.rs`. |
| R1138-6-5 | Setup instructions come from the declared trusted publisher; lookalike hosts are refused and recorded. | Covered by `rust/tests/unit/issue_1138_setup_publisher.rs`. |
| R1138-6-6 | Installation is separately consented, workspace-scoped, and refuses any out-of-scope write before execution. | Covered by `rust/tests/unit/issue_1138_install_scope.rs`. |
| R1138-6-7 | A recipe without a postcondition is refused; a failing postcondition remains `StillMissing`. | Covered by `rust/tests/unit/issue_1138_prerequisite_need.rs`. |
| R1138-6-8 | Memory retains recipe and provenance rather than the installed payload, and rediscovery preserves the content id. | Covered by `rust/tests/unit/issue_1138_prerequisite_need.rs`. |
| R1138-6-9 | Execution uses an isolated selectable environment with network disabled unless the contract grants it. | Covered by `rust/tests/unit/issue_1138_install_scope.rs`. |
| R1138-6-10 | Deadlines report elapsed time, limit, partial output, and every attempted ladder level. | Covered by `rust/tests/unit/issue_1138_named_tests.rs`. |
| R1138-6-11 | The browser advertises runtime size and loads it only after explicit action; only an actual run yields observed output. | Covered by `rust/tests/unit/issue_1138_surface_honesty.rs`. |
| R1138-6-12 | With no execution environment, every language receives an honest refusal rather than fabricated observed output. | Covered by `rust/tests/unit/issue_1138_surface_honesty.rs`. |
