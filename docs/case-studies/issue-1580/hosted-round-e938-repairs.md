# Complete e938 hosted round and remaining repairs

All 34 workflows on `e93881cb64a78a19253088c19f09a1f1b816229c` are terminal:
33 succeed and one explicit scope gate skips. All ten focused native groups pass.
The complete original streams, 168 check records, 24 annotations and 42 diagnostic
candidates are retained with byte hashes. This success does not erase annotations:
there are 21 notices, two warnings and one failure annotation.

Workflows run 44982575871, line 1297, reports a missing named artifact on macOS.
The next attempt downloads artifact 14738044265 and verifies its exact digest at
lines 1334–1343. The actual pinned Node action now scopes only that complete named
lookup diagnostic to a retry notice on attempts one and two. Its failed process
status still drives the bounded retry. Attempt three, unrelated diagnostics,
digest failures, Python extraction failures and cancellation remain unchanged.
The pinned action also warns and exits successfully when some requested IDs are
missing. The same process guard makes that incomplete selection fatal on every
attempt, so a subset cannot become an accepted complete transfer.

Reproduce the exact annotation boundary with
`node --test --test-concurrency=1 experiments/issue-1580/artifact-download-annotations.test.mjs`.
The eight bounded process cases include before/after severity and partial-ID exit
status, buffers, CRLF, escaped names, terminal attempts and unrelated errors.
The diagnostic suffix was compared byte for byte with the pinned action
package-lock dependency `@actions/artifact@6.3.1`, whose downloaded package was
checked against its SHA-512 integrity. Both public and internal lookup paths
construct the same suffix, without a trailing newline.
The maintained entry is `rust/tests/web/native-test-artifact.test.mjs`; its
existing eleven transport controls still execute the actual reset/extractor steps
with a mocked remote transport. No remote-service execution credit is claimed.

Layered CI run 44982575709, line 16053, warns after 270 seconds. Shard one finishes
in 291 seconds of its 360-second budget. Import 913 complete file observations in
raise-only mode: 44 weights increase; every prior weight, the fallback, eight
shards and the 75% warning threshold remain. The complete ordered partition is
preserved. A four-worker replay estimates a maximum near 248 seconds, compared
with the prior plan's 271 seconds; it is not a fresh hosted wall measurement.
The behavior-range contract retains first-wave execution and checks that the
conservative four-worker queue fits the unchanged 360-second job budget. Its
historical weights are deliberately higher than the latest observations: this
assertion does not prove 270-second warning headroom. The new hosted run must
measure that separately. Arbitrary shard separation is not a timing guarantee.

Pipeline run 44982575990, line 196, still observes UNKNOWN crates.io token
authority. Public reads cannot prove publish permission. Scope skips do not
qualify omitted production builds. New native workspace-manifest and file-size
I/O error fixtures also require the next head's hosted execution.

Evidence: `experiments/issue-1580/evidence/ci/terminal-e93881cb/` and
`experiments/issue-1580/evidence/checks/artifact-annotation-and-shard-repair/`.
Complete template-practice execution, aggregate review, original full-suite
budgets, independent authorship and external delivery criteria remain unfinished.

Local validation: eight process regressions, eleven existing transport controls,
the maintained artifact entry and all 26 timing cases pass. The complete local
inventory passes 89 gates and explicitly excludes 53 hosted-only entries. Both
failed intermediate timing assertions and the corrected passing result remain
archived. No Rust compilation or remote-service replay was performed locally.
