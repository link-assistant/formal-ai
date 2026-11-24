# Rust optional dependency context checkpoint, 20 November 2026

[PR #1581](https://github.com/link-assistant/formal-ai/pull/1581) remains draft and unfinished. This checkpoint separates original producer evidence from local repairs and pending hosted execution.

## Original producer results

Read-only [Security run 44720120798](https://github.com/link-assistant/formal-ai/actions/runs/44720120798) analyzes published source `1fd0c4fd131e318593299cfe05d0167f6efd35fc`. Its [complete producer evidence](../../../experiments/issue-1580/evidence/ci/codeql-producer-1fd0c4fd/) preserves 126 verified command/file streams, original artifact identities, full output and SHA-256 receipts.

Actions job `136233430592` succeeds. Original artifact `14622153988` contains CodeQL 2.27.4 SARIF 2.1.0 (2,449,633 bytes, SHA-256 `7f806dd7ff1e576995defc0723fa0e24731e4bceb4f14723e843982d5776b914`), with zero findings, successful invocation and zero warning/error notifications. Seventy-four Actions files have successful extraction notes. The strict result gate reports no unresolved findings at original job-log line 1940. This is actual producer evidence for the literal-main source-binding repair; no alert exclusion was added.

Rust job `136233430159` fails before CodeQL initialization, extraction or analysis. Original log line 287 reports `Build script output absent` for `serde_core@1.0.229`; line 288 records exit 1. No Rust SARIF exists for this run. Compatibility, all 105 formerly context-free source files and complete Rust extraction remain unverified.

Cargo audit job `136233430466` succeeds, but original lines 530–537 retain an allowed unmaintained warning for `unsafe-libyaml 0.2.11`, RUSTSEC-2024-0449, in the historical issue-1017 experiment. The main 555-dependency lockfile and the 91-dependency registry fixture are scanned separately. An allowed warning is not a resolved warning.

## Reproduction and repair

The actual context artifact `14621054940` preserves the complete metadata and Cargo JSON message stream. Cargo successfully checks thirteen dependency libraries and the empty collector. Metadata nevertheless contains the inactive optional `serde_core` edge of `toml_datetime`; its enabled `alloc/default/std` features contain only weak `serde_core?/feature` forwarding. The adapter traversed every normal metadata edge and incorrectly demanded build output for that inactive dependency.

The permanent compressed fixture retains both complete original inputs. The focused security file passes fifteen groups and fails both new groups before repair, including the same missing-build-output exception. The adapter now expands the owning package's features: `dep:name` and strong forwarding activate optional edges, nested definitions expand, aliases normalize, and weak forwarding alone does not activate an edge. Required and activated dependencies retain their build-script/proc-macro checks; another compiled unit cannot activate an unrelated weak edge.

All seventeen security regression groups pass after repair, including the exact captured thirteen-library graph, seven activation/alias cases, and rejection when required or activated build output is missing. Release/changelog gates pass. Local commit `32302e27f` preserves the fix. [Before/after logs and identities](../../../experiments/issue-1580/evidence/checks/codeql-active-optional-dependencies/) explicitly record that hosted verification requires the next published head. No local Rust compilation runs, and no source, finding or diagnostic is excluded.

## Native detector admission and review boundary

The embedded real-Git detector regression was not included in the focused compiled PR target because PR admission uses its JavaScript twin. Local commit `78ab5d46c` includes the existing complete detector module in `workflow_contracts` and admits changes to its source. Its new admission assertion fails before repair; all 97 maintained policy cases, changed-file formatting and the release gates pass afterward. [Original logs and receipts](../../../experiments/issue-1580/evidence/checks/native-detector-admission/) retain the negative control. Actual native execution remains pending the next head.

The [dated production diff receipt](../../../experiments/issue-1580/evidence/checks/production-diff-review-1fd0c4fd/) records complete reading of all 154 changed production paths at `1fd0c4fd`, combining verified earlier identities with 75 newly read diffs. The current adapter and its added tests are separately read completely. This does not establish complete reading of every unchanged source or the whole PR, nor preservation of every existing feature.

Rust template literal coverage is now 243/373 files and 34,777 lines. Six additional historical texts distinguish executable OCI architecture manifests from attestations and identify old artifact/Pages deprecation causes. Current Formal AI still contains DEP0005/DEP0040 warning suppressions and lacks the template's custom extractor counterpart; applicable warning repairs remain open. Historical reports do not establish current emitted warnings or current registry identity. The other 130 files and all unverified practice/runtime criteria remain unfinished.

## Remaining work

Finish every workflow on `1fd0c4fd`, inspect every actionable failure and respect the single push gate before publishing the combined batch. Then run both the actual native detector and the original read-only Rust producer on the new head. Preserve the original SARIF and extraction diagnostics; local metadata replay cannot replace them.

The wider issue remains open: strict dated original-source acceptance 31/53, three historical budget criteria, all remaining actual warnings and dependency migrations, complete template/practice comparisons, publication certainty, live identity/delivery, general binding/provenance and whole-PR review. Independently accepted Formal AI reasoning remains 0/20. Supplied commands, code and reviews receive zero independent authorship or reasoning credit.

The staged evidence first fails the filename abbreviation ratchet: nine artifact archive names copied `_temp` from producer paths, increasing the unchanged 90-name ceiling to 99. Descriptive archive names restore the ceiling while preserving compressed/raw hashes and original producer-relative paths. The failure, exact nine-file mapping and passing rechecks remain in [the publication validation](../../../experiments/issue-1580/evidence/checks/codeql-optional-context-publication/). Documentation policy (32), production profile (18), both source-size gates and the corrected abbreviation gate pass; the historical failure remains recorded.

## Historical Rust reproducer dependency repair, 24 November 2026

[Security run 45295359154](https://github.com/link-assistant/formal-ai/actions/runs/45295359154) still reports the allowed `unsafe-libyaml 0.2.11` warning at full log lines 782–790 for the issue-1017 experiment. [RUSTSEC-2024-0449](https://rustsec.org/advisories/RUSTSEC-2024-0449) identifies the crate as unmaintained. The original default graph remains available in the [published historical lock](https://github.com/link-assistant/formal-ai/blob/e6ed5b3faad87db3628dee9d271646c7f3127367/experiments/issue-1017-meta-language-quadratic/Cargo.lock).

The [published meta-language 0.58.1 crate](https://crates.io/crates/meta-language/0.58.1), SHA-256 `0f757c1730fe686e8fe69224de9dfd0dc8c28008b39c1c53069d4f104740b60a`, enables Lindera by default. Lindera brings in `serde_yaml_ng`, which depends on `unsafe-libyaml`. The reproducer selects the independently included Rust tree-sitter parser and still reaches the same `point_at_byte` implementation. Pin 0.58.1 and disable default features. Cargo resolution reduces this standalone lock from 339 to 171 package records, preserving every remaining version, source and checksum. The reproducer source is byte-identical.

Run `node --test --test-concurrency=1 --test-name-pattern='historical Rust parser reproducer' rust/tests/web/fast-pull-request-production-profile.test.mjs`. The original lock fails the forbidden-package assertion. The repaired lock keeps the exact original parser and omits Lindera, `serde_yaml_ng` and `unsafe-libyaml`. Cargo lock resolution also succeeds offline. Hosted Security must confirm the audit result. Rust compilation and execution were reserved for CI.
