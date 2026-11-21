# Completed CI round and warning repairs

Published source `d1d11e6039845ead74f1cb229266acb9ea42823d` completed all
34 observed workflows: 30 succeeded, one skipped and three failed. The full
34 workflow streams and all 31 annotations from 173 checks are retained with
original and compressed byte hashes. A complete-stream diagnostic scan found
the worker compiler error, stale worker receipt, source/checkout failures,
missing interface companion, nine deprecated artifact-download calls, one
Hive timing warning, the deliberate BuildKit outage annotation, two SQLite
experimental imports and one experimental timer mock. Selected diagnostics
and their actual source were reviewed; the entire 68,174-line Pipeline stream
was not literally read. The passing test name containing “warn category” is
a scan false positive, not a runtime warning.

The source and native companion repairs are recorded in
[the bootstrap checkpoint](bootstrap-ci-repair.md). The standalone wrapper is
now receipt-bound together with the main binary. All ten family groups reuse
verified finite executables in the regression, with zero Cargo invocations.
This fixture result does not replace the next hosted native execution.

The Hive workflow test combined four complete language sessions into one
696.84-second case and crossed the unchanged 675-second warning boundary.
Extract exactly its existing session/assertion body into one helper and expose
Kotlin, Scala, Rust and Python as separate real tests. Both complete bodies
have the same normalized SHA256. Four measured-weight shards retain every
listed test exactly once, two concurrent jobs, single native test threads,
the 900-second command budget and the 20-minute job clock. New language weights
are explicitly labeled estimates of 169 seconds, derived from the historical
672.933-second aggregate; they are not new hosted duration observations.
All 62 runner/admission regressions pass; rustfmt passes without a Rust build.

The deliberate canonical BuildKit pull failure emitted a failure annotation
even though the real mirror recovered. Stop command parsing only around that
negative-control action, using a fresh random token. Always resume commands;
non-success outcomes then emit a real error and remain fatal. Actual mirror
image and daemon verification follows with ordinary annotations enabled.
All 11 controls pass, including failure, cancellation, skip and empty outcomes.
The production setup action and its failure behavior are unchanged.

The two SQLite consumers now explicitly select Node 24.22.3 instead of Node 22
or the runner's ambient executable. Its upstream SQLite module no longer calls
the experimental-warning emitter; the API is still a release candidate, not
fully stable. See the [official API history](https://nodejs.org/docs/latest-v24.x/api/sqlite.html)
and [versioned implementation](https://github.com/nodejs/node/blob/v24.22.3/lib/sqlite.js).
The regression exercises a real SQLite round trip and emits an independent
sentinel to prove that process warnings remain enabled. Local Node 26 execution
does not qualify the selected hosted Node 24 binary; that awaits CI. The link
test uses stable method mocks, observes the exact remaining 15 ms budget,
asserts cancellation and verifies timer cleanup. All 16 link tests pass.

The crates.io warning is an honest unavailable credential observation, not an
invalid-token verdict: a public crate lookup cannot verify a publish-scoped
token. No credential is sent to a cookie-only endpoint and no publication is
performed to manufacture verification. Registry uncertainty remains explicit.

The final maintained entry passes 335 cases. Production checkout tests pass
21/21; the two runtime checks, the touched size/readability/filename/scheduling
and workflow gates, and whitespace checks pass. Earlier incorrect gate names,
the mistaken test job key, source-selector corrections and failed before-fix
controls are retained separately and receive no verification credit. The full
local inventory's earlier 87 passes, 53 hosted-only skips and one genuine stale
worker failure remain its actual outcome, not 88 passes. Hosted worker
regeneration, final-head native checks, complete template/practice acceptance,
whole-PR review and the independent-authorship floor remain unfinished.
Supplied procedures and conclusions receive zero autonomous credit.

Evidence: `experiments/issue-1580/evidence/checks/terminal-warning-repairs/`.
Pull request: https://github.com/link-assistant/formal-ai/pull/1581.
