# Native policy helper headroom

The maintained scanner reproduced three more 900-line warnings in release
policy, data-file integrity and benchmark regression owners. Ordinary child
modules now hold release assertions, text-row helpers and benchmark evaluation
helpers. Every moved body survives byte for byte after normalizing only the
helper visibility required by its parent. Existing assertions and budgets stay
in force. The complete scanner reports 81 warnings and zero violations under
the unchanged limits.

Review also caught an extraction error before publication: Cargo registers
`data_files.rs` as a standalone crate root, so an implicit child declaration
resolved to a missing sibling file. A regression follows the real Cargo target
and resolves its module path; it fails before an explicit path attribute and
passes afterward. Nine focused cases now pass. The preceding validation passed
141 maintained observation cases, 50 native-runner cases, three touched gates,
formatting and whitespace checks. The final path repair was checked with the
nine focused cases and formatting. No local native build ran.

Actual Git sparse-checkout rule evaluation also selects all nine historical
upstream-report fixtures in both core checkout attempts and excludes an
unrelated log. This verifies selection, not native execution. Hosted native
compilation/execution of the local commits, complete template/practice review
and the wider issue requirements remain unfinished. Supplied scripts and edits
earn zero independent authorship or reasoning credit.

Evidence: `experiments/issue-1580/evidence/checks/native-policy-headroom/`.
