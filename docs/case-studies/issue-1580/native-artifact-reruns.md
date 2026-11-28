# Native contract artifacts across workflow attempts

The native producer previously uploaded an immutable artifact named only by
commit SHA. Its receipt also binds the workflow run and producer attempt.
A full workflow rerun could collide with the first upload name; rerunning
failed consumers could instead reject the earlier successful producer's exact
receipt because the consumer attempt increased.

The producer now names uploads by run, attempt and commit, and exposes the
immutable upload ID plus its original attempt. Consumers validate both outputs
before download, select that exact artifact ID within the current run, and
compare the executable receipt against the selected producer attempt. Missing,
noncanonical or future attempts fail before download; selecting all artifacts
when outputs are empty is forbidden. Run, source, physical inputs, compiler,
recipe and every executable's byte hash retain their existing checks.

The finite transport regression uses the actual maintained runner with bounded
Node executable fixtures and a Cargo stub that must never run. Ten groups
execute exactly their complete selected inventory. An unbound second attempt
fails before execution; the exact earlier producer succeeds; another run,
incorrect or future producer attempt, empty upload ID and modified executable
all fail before execution. The separate selection regression rejects 24
malformed fields and a future attempt. Workflow checks require distinct full
rerun names, the exact producer ID and validation before download.

The newly added regressions fail against the previous implementation and both
complete JavaScript test files pass after the repair, together with six touched
gates and final syntax validation. The original failures, complete final logs
and final source diff are retained in the [evidence manifest](../../../experiments/issue-1580/evidence/checks/native-artifact-reruns/manifest.json).
No local native compilation or execution occurred. Actual hosted full-rerun
and failed-job-rerun qualification remain unfinished. Supplied work receives
zero authorship credit; wider issue acceptance remains open.
