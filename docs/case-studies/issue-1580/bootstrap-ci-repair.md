# Bootstrap CI repair checkpoint

The first hosted worker regeneration at source `d1d11e603` failed under the
existing `-D warnings` policy: an unused forwarding import remained after its
serializer moved into the dialogue-rendering child module. Remove that dead
import; the actual serializer and its caller remain unchanged. The complete
selected-source WASM regression passes 18 cases without compiling Rust locally.
Actual worker compilation and its generated receipt remain required.

Layered CI run 44950901905 found three additional source-level defects. The
behavior-range suite still begins in the first four-worker wave when a newer,
measured heavier suite sorts before it; retain the longest-first order, complete
partition, heavy-suite separation and reverse-input determinism. Route the new
native artifact consumer through the existing strict raw-download adapter,
including its extraction closure in both checkout attempts. The actual hosted
direct downloader emitted DEP0005 at coding-1 job log line 621; no suppression
is added. Preserve the eight consumed immutable source captures explicitly in
the sparse checkout, while leaving unrelated historical archives excluded.

The local gate runner also had a real false negative: main rebuilds the worker
before its binary diff, while the local runner only diffed the unchanged binary.
Use the maintained input/byte manifest checker locally without compiling. A
finite real-Git fixture passes clean inputs, fails changed source inputs, and
fails changed binary bytes. The source-change case failed before this repair
because the old runner returned success. The canonical main gate is unchanged.

The first native artifact interface execution passed 77 cases and failed its
standalone wrapper launch at `with_formal_ai.rs:809`. The artifact selected three
test executables and `formal-ai`, but discarded the actual Cargo-produced
`with-formal-ai` executable. Select, hash, transport, verify and restore both
companion binaries. Reject either missing executable or changed bytes before
execution; preserve the existing source/compiler/run/attempt/recipe binding,
strict Cargo argument adapter and original native integration assertion. The
compiler-JSON regression fails before the repair. A small executable fixture
launches the restored standalone wrapper afterward; no local Rust build runs.

The full 141-entry local inventory records 87 passes, 53 explicit hosted-only
exclusions and one genuine stale-worker failure. The maintained entry before
the companion repair passes 332 cases; the repaired artifact module is checked
separately. Both actual file-size checks report zero warnings under unchanged
limits. Earlier nonzero warning totals in the chronological plan are historical.
All initial failures, fixture corrections and passing retries are retained in
`experiments/issue-1580/evidence/checks/bootstrap-repair/` with byte hashes.
The strict production-checkout expectation initially passed 20 of 21 cases;
adding those same eight consumed fixture paths restores all 21 passes without
relaxing checkout, permission, duration or unrelated-history assertions.

An actual small autonomous request to repair the unused import returned an
unsupported declared-create response and changed no source. The complete request,
response and before/after files are retained. Supplied procedures and conclusions
earn zero independent Formal AI authorship or reasoning credit; the 20 percent
authorship floor is still unmet. Template/practice reconciliation and exact-head
hosted verification remain separately unfinished. The pull request stays draft.

Pull request: https://github.com/link-assistant/formal-ai/pull/1581.
