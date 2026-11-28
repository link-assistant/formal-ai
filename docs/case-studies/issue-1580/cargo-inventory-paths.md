# Complete Cargo audit inventories

The current Rust template's security workflow inventories tracked locks with
`git ls-files -z` and continues auditing later locks after a finding. Comparing
those practices found two real gaps in Formal AI's wrapper: newline-delimited
Git output changed quoted filenames, and `set -e` ended the audit after its
first failure. The primary binary lock could also be physically present but
absent from the committed inventory.

`rust/tests/web/cargo-audit-inventory.test.mjs` reproduces these failures with
a real temporary Git index and a recording Cargo stub. No Cargo resolution,
network access or Rust build runs. Before the repair, eight of nine cases fail;
afterward, all ten cases pass, including an additional missing-primary control.
Both `core.quotePath` settings preserve spaces, newlines, tabs, quotes,
backslashes and Unicode. Untracked files stay excluded. Failed, unterminated
and empty-record inventories stop before Cargo. Missing files retain an escaped
annotation, and later lockfiles are still audited. Multiple findings preserve
the first failing status.

The wrapper stores the exact inventory in a temporary file and checks the Git
producer status before proof or audit commands. It validates complete NUL
records and requires the tracked primary lock. Audit aggregation preserves the
existing primary and independent-lock policies and advisory proof checks.
Four related test files pass another 41 cases. The template source and all
before/after logs are content-addressed in
`experiments/issue-1580/evidence/checks/cargo-inventory-paths/manifest.json`.
These checks establish the wrapper protocol, not live advisory database results
or release authorization. Full issue acceptance remains unfinished in
[PR #1581](https://github.com/link-assistant/formal-ai/pull/1581).
