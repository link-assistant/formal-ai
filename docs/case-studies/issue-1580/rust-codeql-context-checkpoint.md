# Original CodeQL producer reports and Rust project contexts

Checkpoint: 2026-11-20. Issue #1580 and PR #1581 remain unfinished. This
supersedes the earlier checkpoint's statement that producer reports had not yet
been obtained; it does not turn the current source repairs into hosted evidence.

## Published-head observations

Head `ad59ff56dafe3377f48ebf7739526d89642198fd` completed all 23 automatic
workflows successfully. The explicit read-only Security producer run
[44710630318](https://github.com/link-assistant/formal-ai/actions/runs/44710630318)
failed both its Actions and Rust CodeQL gates. Its Cargo audit passed. All 24
workflow logs, native job logs and collection responses are retained in
[`native-families-ad59ff56`](../../../experiments/issue-1580/evidence/ci/native-families-ad59ff56/).
All 286 stdout/stderr streams have verified compressed and original identities,
byte lengths and line counts. The collector is terminal before this checkpoint.

The automatic native jobs pass the workflow (104), coding (32), reasoning (141),
interfaces (78), benchmark (11) and Hive (20/10/7) suites. The registry helper
passes three native tests, and its actual release CLI fixture passes eight cases
(job 136201211171, lines 742–764). These observations do not establish all
original-source contracts: the strict dated replay denominator remains 31/53.

The original reports are retained byte-for-byte, compressed, in
[`codeql-producer-ad59ff56`](../../../experiments/issue-1580/evidence/ci/codeql-producer-ad59ff56/):

| Producer report | Original bytes | Original SHA-256 |
| --- | ---: | --- |
| Rust, artifact 14617183486 | 5,499,579 | `afc98984294a56e1e69fcf06a91c89a8b887bce6aef88d83a7c81c3d6d04a91c` |
| Actions, artifact 14616259795 | 2,459,242 | `7020757b8bdbda622e8742bf1778f6eaeee09f52c1ff340b585dd2080237bb2e` |

Both are SARIF 2.1.0 from CodeQL 2.27.4 with successful execution metadata.
Actions contains two unresolved cache-poisoning findings. Rust contains no
reported findings but **511 extraction warnings**: 406 macro-expansion failures
and 105 files without a discovered manifest (101 scripts and four WASM files).
A finding-free Rust report therefore does not establish complete analysis.
The strict result gate correctly rejects it; no diagnostic is suppressed.

The same full automatic workflow logs contain 123 actual source-size warnings.
The other 33 warning candidates are quoted shell source, including cache
fallback commands, rather than emitted warnings. The first investigation filter
was too broad; both its output and the corrected classification are retained.
No source-size warning is resolved by this classification.

## Source repairs requiring the next hosted producer

The Actions repair binds initial source selection to literal `refs/heads/main`
checkouts with credentials disabled. The job verifies the checked-out HEAD
against the checkout's main tracking ref before executing draft source, records
that exact SHA once, and reuses it in dependent jobs. Shell regressions use real
Git repositories to check matching heads, mismatches, missing tracking refs and
changes after sealing. Both original findings still need a fresh producer run.
[CodeQL #23176](https://github.com/github/codeql/issues/23176) records the
upstream heuristic investigation; filing it does not repair either finding.

The Rust producer log identifies compiled macros from Rust 1.99.0 but a
proc-macro server from the former 1.94.0 sysroot. The workflow now pins its
binary sysroot and rust-src to 1.99.0 for the measured CodeQL 2.27.4 extractor.
No application compiler policy changes with this security-analysis pin.

`scripts/codeql-rust-projects.py` discovers the actual standalone declarations,
including the rust-script default edition. The current source has 19 distinct
dependency contexts. Each resolves independently, preserving conflicting exact
versions, dependency aliases and features. Dependency-only collector libraries
are checked on the explicit host target with one Cargo job, no incremental
cache and no debug information. Metadata, Cargo lockfiles, build-script cfg/env,
compiled proc-macro libraries and original maintained source paths form the
combined `scripts/rust-project.json`. Helpers retain their declaring module
context. The direct-rustc worker gets its actual edition 2024 and
`wasm32-unknown-unknown` context from the maintained build script. Unsupported
manifest constructs, missing build output, failed dependency builds or existing
project files stop preparation; no synthetic replacement source is analyzed.
Both project manifests are written only after all contexts have succeeded.

The workflow preserves context receipts, Cargo inputs, compiler identity,
metadata and build logs even when later analysis fails. Its branch verification
job also has `security-events: read`, matching the producer's stated API
requirement, while result upload remains disabled on that job.

## Reproduction and verification

Run the adjacent JavaScript regression without building Rust:

```sh
timeout 600 node --test --test-concurrency=1 rust/tests/web/security-codeql-results.test.mjs
timeout 600 node --test --test-concurrency=1 rust/tests/web/hive-mind-draft-workflow.test.mjs
node experiments/formal_ai_subagent/local-gates.mjs
```

Before repair, controls reject the old compiler pin and absent standalone
contexts. The final security regression passes 15 groups, including exact-pin
separation, aliases/features, module ownership, actual WASM target, mocked Cargo
build metadata, explicit host target and failure-before-publication behavior.
The Actions source-binding regression passes 14 groups. Retained main-policy
checks pass 97 cases and the refreshed production-profile checks pass 18.
The complete local inventory passes 87 executable gates, with 53 explicit
CI-only exclusions and no failures. Focused tests repeat after the explicit
host-target follow-up; full and focused logs remain separate. No local Rust
compilation, application build or native test execution is performed.

See the complete before/after logs and command/source identities in
[`rust-codeql-project-contexts`](../../../experiments/issue-1580/evidence/checks/rust-codeql-project-contexts/)
and [`main-source-checkout-binding`](../../../experiments/issue-1580/evidence/checks/main-source-checkout-binding/).
Supplied commands, scripts and patches earn zero autonomous-authorship credit.

## Open acceptance

A fresh exact-head hosted producer must prove that both Actions findings and
all 511 Rust warnings are resolved. Inspect original invocations, extraction
coverage for all 105 affected source files, and non-warning coverage notes as
well as finding totals. A generated context receipt or local mock cannot prove
those facts. Both hosted repair-verification flags remain false here.

The original budget/native contracts, all eight complete template comparisons
and applicable runtime practices, dependency maintenance and source-size
warnings, registry publication certainty, historical cache/Docker evidence,
live delivery/model identity, general provenance and complete PR review remain
unfinished. Rust template source review is 237/373 files and 34,513 literal
lines, with 136 files and applicable practice validation open. Independently
accepted Formal AI reasoning remains 0/20. Keep PR #1581 draft and unmerged
until the full issue acceptance inventory is verified.
