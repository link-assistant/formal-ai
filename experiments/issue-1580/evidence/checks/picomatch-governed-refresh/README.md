# Reviewed manifest witness refresh

The security dependency update at 83882c0ec changes desktop/package.json and
vscode/package.json. Layered CI run 44156399182 fails two existing assertions:
the complete governed inventory comparison (log lines 8204-8206) and repository
fixture source binding (8507-8526). All other 23 workflows pass; the shard has
two assertion failures, not a timeout. The full failed log is retained under
../../ci/83882c0ec/.

The existing explicit refresh command reads the complete current inventory,
with exact previous-witness and reader hashes. The permanent experiment
experiments/issue-1580/governed-manifest-refresh.mjs compares every object key,
order, prototype and scalar, accepting only the exact two old/new manifest
texts or their hashes. It checks 7,016 object pairs and reports all 16 changed
fields: eight source values and eight hashes. All 221 supported and 964
unsupported cases, assertions and native source hashes remain unchanged.

Both existing failing test files pass after the refresh: 34 identity/inventory
tests and two fixture-integrity tests, with no skips. Complete logs, commands
and hashes are retained here. These are JavaScript source-fixture checks;
they do not execute the native specification or complete broader acceptance.
The pre-merge complete local gate set passes 86 gates with 52 explicit native/context exclusions. The default branch advanced during validation; merged-source gates and next-head hosted checks remain pending. All
supplied work receives zero independent Formal AI authorship credit.
