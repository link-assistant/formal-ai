# Compile the focused native contract targets once

[CI/CD run 44938457098](https://github.com/link-assistant/formal-ai/actions/runs/44938457098)
reported 75%-of-900-second warnings for coding-1 and Hive-3, although all nine native contract groups passed.
The coding-1 log spends about 328 seconds compiling and listing before 353.78 seconds of tests;
the Hive-3 job spends about 175 seconds before 509.21 seconds of tests.
The nine jobs separately compiled the same source, profile and three test targets.

The focused native workflow now compiles `coding_contracts`, `reasoning_contracts` and
`interface_contracts` once in a bounded read-only producer. Cargo's actual JSON output selects
the three test executables and their `formal-ai` companion; guessed target hashes are rejected.
A same-run Actions artifact carries those bytes and a receipt. Consumers independently compare
the selected commit/tree, absolute compilation workspace, workflow run and attempt, actual compiler,
platform, architecture, flags, target and profile overrides, native inputs, materialized sparse inputs,
and all four executable hashes before listing or executing a test. Excluded historical files are
explicitly recorded as excluded; this receipt does not claim they were physically checked out.
The existing complete-workspace native release receipt is unchanged.

Every consumer uses the unchanged runner's live listing, weighted exact case selection and failure handling.
The existing core/default Cargo path, all nine groups, one test thread, 900-second execution budget,
75% warning ratio, checkout retries and 20-minute job clocks remain. A producer compilation failure
blocks its consumers; an independent core test failure does not. The artifact has one-day retention,
read-only repository permissions, no release outputs and no publication or deployment.

`experiments/issue-1580/native-contract-artifact.test.mjs` uses finite JavaScript executables.
The original runner invokes a deliberately failing Cargo stub; the changed real runner executes all
nine groups without Cargo and covers every listed mock case exactly once. Tampered bytes stop before
listing; a genuine child exit status 17 propagates. Additional cases cover sparse/missing/modified
physical source, untracked tests, compiler JSON target completeness, source/run mismatch, companion
tampering and undeclared arguments. Existing group, policy and checkout tests remain green locally.
No local Rust build or native runtime validation was performed. Actual producer compilation, native
test behavior, transport size and warning-free hosted timing remain for the next CI run.

Retained attempts and command output are under
`experiments/issue-1580/evidence/checks/native-contract-artifact/`.
The supplied implementation and conclusions earn zero independent Formal AI authorship/reasoning credit.
Pull request: https://github.com/link-assistant/formal-ai/pull/1581.

The final maintained entry passes all 323 tests. The file-size, changelog fragment, reconstructed changelog, abbreviation, CI speed and shell lint gates also pass. Raw output and exact digests are retained in the native contract artifact final-checks evidence directory. Hosted compilation, current runtime budgets and the other issue-wide acceptance criteria remain unverified.
