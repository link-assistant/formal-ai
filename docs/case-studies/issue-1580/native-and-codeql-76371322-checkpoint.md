# Exact-head native and original CodeQL producer results

Pull request: https://github.com/link-assistant/formal-ai/pull/1581

All 27 workflows for 763713221c0681393da9266d9ffc9cc7c2bbefe2 are terminal:
25 success, one explicit skip and one Layered CI failure. Every one of the 276
collected stdout/stderr streams passes original/compressed SHA-256, byte and line
verification. The failed assertion and local reproduction are documented in the
script/secret-inventory checkpoint; its correction awaits the next hosted head.
No skipped main-only application build or publication receives execution credit.

The native jobs pass 528 Rust cases, including the previously failing Pages
contracts and real-Git detector controls, plus eight actual release CLI cases.
The 528 include three registry helper cases. Coding/reasoning/interfaces,
benchmark and all three Hive families pass. Their logs preserve exact job IDs,
line-numbered summaries and filtered-case counts. This execution does not verify
later uncommitted script splits, actual live Pages delivery or historical replay
acceptance. Hosted logs retain 122 actual source-size warnings and one honest
publish-token certainty warning; these remain open except for separately drafted
structural repairs still awaiting their own hosted execution.

The explicit read-only Security producer 44754743681 uses the same source head.
Original CodeQL 2.27.5 Rust and Actions SARIF both have zero results and successful
invocations, with zero warning/error notifications. The Rust report's original
SHA-256 is 55b13f140c5f9c0a2d3c08dc93a7260b9762c8362b3e4ab10a80654acc1566c4;
Actions is 57b9861421e99056fda2e3be6b5a3f51381d597d8e2f8aed1ab7e5cee4b63d54.
Artifact metadata binds both original files to the producer run and head.

The actual extractor names all 2,573 expected Rust paths as successfully
extracted, with no path missing or added. All 105 paths previously lacking a
semantic manifest are present in the current original extraction. The historical
406 macro ABI and 105 missing-context warnings are absent without suppression.
Omitted SARIF notification levels default to warning: the initial assessment
forgot that default and then used an overly broad context substring. Both
incorrect intermediate assessments and the corrected exact-message crosswalk
remain retained; neither incorrect intermediate gets verification credit.

Remaining informational limits are explicit: Rust has 56 notes and 631
unresolved-macro notifications at level none. Their exact source locations,
source hashes and source lines are preserved: 282 in rust/src, 347 in rust/tests
and two examples. Successful file extraction does not establish every macro's
semantic coverage. That investigation and broader coverage acceptance stay open.
The Cargo audit still records Chrono's blocked dated exception and the historical
unsafe-libyaml maintenance warning; zero CodeQL findings does not resolve either.

Complete terminal logs and assessments are under
`experiments/issue-1580/evidence/ci/native-families-76371322/` and
`experiments/issue-1580/evidence/ci/codeql-producer-76371322/`.
Every broader unfinished issue requirement remains open; PR #1581 stays draft.
Supplied commands, helpers and patches earn zero independent reasoning credit.
