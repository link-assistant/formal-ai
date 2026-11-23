# Source size warnings from native admission qualification

The [8682 pipeline](https://github.com/link-assistant/formal-ai/actions/runs/45109737000)
reported ten source-size warnings. Its preserved whole log records them at
lines 2438 through 2456. The maintained policy is unchanged.

| Source | Before | After |
| --- | ---: | ---: |
| Change detection script | 925 | 532 |
| Numeric list parser | 945 | 822 |
| Chat surface tests | 914 | 879 |
| Debug session tests | 910 | 881 |
| Coding idioms worker and TypeScript copy | 1425 each | 1393 each |
| Solver worker and TypeScript copy | 1416 each | 1359 each |
| Java historical evidence index | 1408 | 484 |
| Current template delta evidence index | 1449 | 385 |

The Rust extraction preserves every original function name and moved body.
The change-detection tests remain under the original test module. Literal
parsing keeps its public and crate-visible exports. Chat and debug assertions
remain in their original parent modules through child modules. Focused reasoning
CI also includes the complete original chat surface and shared behavior-rule
tests, so this extraction receives native execution before merge.

The numeric list syntax renderer moves to the existing arithmetic/list worker;
answer finalization moves to the existing solver-event worker. Both bodies are
byte-for-byte copies. The canonical ES translator regenerates all four affected
TypeScript files. The maintained prerequisite generator refreshes worker source
identities. Changing an identity-bearing source initially prevented Formal AI
from starting; the generator was run directly once to restore that boundary.
This bootstrap execution receives zero Formal AI authorship or verification
credit. Subsequent commands run through Formal AI.

The evidence indexes retain multiline JSON with short arrays and objects on one
line. Parsing the before and after bytes yields identical JSON values,
including every source identity, archive member, hash and qualification flag.
The original bytes are retained in the validation evidence; compressed evidence
payloads are unchanged.

Run the regression with node --test --test-concurrency=1
experiments/issue-1580/source-warning-headroom.test.mjs. The maintained scanner
first failed on the four unchanged Rust sources; an expanded run failed on the
six remaining worker, TypeScript and index warnings. After repair, that same
scanner reports no errors, violations or warnings for all eighteen parent and
extracted files. This test is included in the fast PR JavaScript inventory.
Rust formatting and extraction conservation pass locally; native compilation
and the full gate results are qualified separately.

The same qualification's [Layered CI](https://github.com/link-assistant/formal-ai/actions/runs/45114823928)
found a stale exact checkout expectation at
rust/tests/web/default-branch-build-policy.test.mjs line 195. Both failed jobs
reported this one mismatch. The expectation now includes the active harness
manifest and recursive registry children. Its complete 168-test file and the
ten real-checkout maintenance regressions pass locally. The exact assertion
and sparse checkout remain enforced.

These checks establish source conservation and policy behavior. They do not
complete the wider issue's native, release, template, upstream or reasoning
acceptance criteria. Supplied orchestration receives zero independent credit.
