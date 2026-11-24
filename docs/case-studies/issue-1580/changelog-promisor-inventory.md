# Changelog inventory without lazy blob fetches

Hosted head `754fd314f` failed the changelog gate at
https://github.com/link-assistant/formal-ai/actions/runs/44992281354/job/137115847126.
The complete job log has SHA-256
`45c7f6ad3f8268880ee22b6329428166c85b06213ccc325b59a38e64ad7e750d`.
Lines 1537–1545 show `git diff --name-only origin/main...HEAD` trying to
fetch a missing promisor blob and receiving HTTP 403 after the sparse checkout
removed credentials. Rename similarity detection needs file contents even when
the caller asks only for names.

The JavaScript checker and native Rust twin now use `--no-renames`. Both sides
of a rename remain visible as deletion/addition names. The sparse checkout and
credential removal remain bounded and the decision still rejects source changes
without a fragment.

Reproduce with the maintained `changelog-fragment-output.test.mjs`: a finite
local Git repository has a renamed-and-modified source, a blob-filtered clone,
and an unavailable file-protocol origin. The original query fails against missing
blobs in both fixtures. The corrected checker succeeds with a fragment and
reports the actual missing-fragment error without one. No network credentials
or production repository mutation are needed.

This physical fixture also exposed a source matcher defect: an empty root
prefix made `^?scripts/` invalid. Both implementations now recognize ordinary
root scripts and nested Rust scripts with exact prefixes. Seventeen positive
and negative classifier cases cover both layouts. The actual Rust checker is
imported by the focused native workflow target, with one native regression;
hosted execution of that addition remains pending.

All eight output/physical Git tests, 161 admission policy tests and 63 native
admission tests pass locally. Five focused gates and Rust 2024 formatting pass.
No local Rust build was run. Complete intermediate failures (including the
initial regex exposure and default-edition formatting error), final output and
hashes are retained in `experiments/issue-1580/evidence/checks/changelog-promisor-query/`.
This repair receives zero independent-authorship credit; the broader issue
criteria and the remaining hosted failures are still open.
