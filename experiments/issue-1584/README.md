# Controlled output development evidence — issue 1584

Work in progress for [PR 1585](https://github.com/link-assistant/formal-ai/pull/1585).
None of the issue's seven broad acceptance criteria is marked complete.

The JavaScript root is `js/agentic/crate/cnl_output.mjs`. The shared policy is
`data/seed/cnl-output.lino`; native code is in `rust/src/cnl_output/`.
Configuration selects a named standard independently from vocabulary density.
Explicit user directives can override each dimension. A standalone rewrite
uses the previous assistant response before the agentic planner runs. Tool
arguments, quoted input, and opaque spans retain their original bytes.

## What the current checks demonstrate

- Twelve focused JavaScript tests exercise option validation, independent mode
  switching, Unicode word counting, opaque input, and declared equivalence.
  The combined E-Prime and density regression rejects declared realizations
  that violate the profile's finite forbidden-word check.
- Three HTTP tests cover request-local overrides, invalid configuration, SSE
  metadata, and rewrite handling before tool planning. They check that the
  maximum-precision notice stays in response metadata when a body must be
  valid JSON or an exact label, and that tool calls retain their arguments
  and token accounting. The fast pull-request lane runs both files.
- One proposition has two declared realizations in each of English, Russian,
  Hindi, Chinese and Spanish. Its finite profile/density matrix preserves the
  declared lexical identity. This is not the solver's complete intent graph
  and is not a corpus across all requested input standards.
- Rust tests mirror the limited corpus and configuration boundaries. They have
  not been compiled locally; repository policy leaves Rust execution to CI.
- Real browser checks cover persisted settings and a rewrite turn. Screenshots
  are in `docs/screenshots/issue-1584/`. The browser and preview server were
  closed after verification. These screenshots do not prove vocabulary limits.

The [detailed requirement-to-evidence table](acceptance.md) retains the prose requirements.

## Acceptance inventory

| Requirement | Implemented evidence | Still unfinished |
| --- | --- | --- |
| Unrestricted input | Unknown text, quotes, polarity and quantifiers remain opaque and unchanged | General equivalence of free-form and every CNL input intent |
| Default intersection | Shared profile and sentence-length constraint | Complete multilingual intersection grammar, vocabulary and semantic validation |
| Named output standards and density | CLI/env, OpenAI fields, UI settings, localized directives | Strict renderers, official vocabularies, NSM decomposition, frequency corpus, all protocol and summarization surfaces |
| Maximum lexical density | Deterministic minimum over declared alternatives; localized dictionary notice | Global minimum proof over existing words; this is explicitly `unchecked` |
| Bidirectional paraphrasing | Declared realization round trips and per-turn settings | General meaning-preserving paraphrases |
| Candidate clarification | Existing solver clarification was inspected | Precise candidate CNL and confirmation before actions across runtimes |
| Cross-standard matrix | Small JS corpus plus unexecuted native tests | Full input-style/output matrix and native/JS/WASM behavioral parity |

A selected profile is metadata, not a compliance certificate. Missing checkers
produce `compliant: false` with named `unchecked` constraints. Opaque retention
is lossless storage, not evidence of understanding. In particular, `primes` and
`top-N` currently retain text without claiming that it satisfies their limits.

Arbitrary positive `top-N` includes `top-1`. A single natural-language word
cannot express every distinct proposition with exact preserved meaning by
ordinary natural-language composition. Encoding arbitrary meanings in counts
or identifiers would not satisfy a natural-language output requirement. The
strict universal output constraint therefore conflicts with unrestricted
meaning preservation for some tiers; the requested policy for an unsatisfiable
tier remains unresolved. Other unfinished work above is implementation work,
not an external blocker.

## Reproduction

Run each file separately, as required by AGENTS.md:

```sh
timeout 600 node --test --test-concurrency=1 rust/tests/web/cnl-output.test.mjs
timeout 600 node --test --test-concurrency=1 rust/tests/web/cnl-output-http.test.mjs
timeout 600 node --test --test-concurrency=1 rust/tests/web/server.test.mjs
node scripts/translate-es.mjs --check
node scripts/translate-js-rust.mjs --verify
node experiments/formal_ai_subagent/local-gates.mjs
```

The shared policy and seed registries require the full local gate set. Skipped
CI-only Rust gates must not be counted as locally passed. The generated JS-to-
Rust fixture contains carried source and refused constructs. The latest
generation and currency checks exit zero while reporting 49 refused modules;
that status does not establish executable translation. The native module is a
supplied manual port; automatic translation and runtime parity are unproven.

## Development provenance

`drive.mjs` is an externally supplied adapter around the maintained JavaScript
Formal AI planner and real tool host. `job.mjs` keeps long checks outside the
host's command deadline and records completion. Supplied requests, patches,
commands and scripts earn **zero autonomous authorship credit**. The frozen
substantive unit inventory is in PLAN.md: 8 frozen, 0 independently accepted
(0%). No claim that Formal AI authored this implementation is made.

Captured records retain exact requests, complete responses/tool traces,
failures, command outputs, runtime identities, tracked diffs and independent
checks. Early runtime records hash tracked diffs but omit untracked source;
that provenance limit is retained explicitly. A bounded evidence archive and
SHA-256 manifest accompany the final verification record. Temporary package
installations, caches and binaries are excluded from the archive.

## Captured verification

The first repair full gate run recorded 86 passes, 52 CI-only skips and no failures.
The latest inventory passed 85 gates and skipped 52; its ShellCheck PATH failure
passed a targeted retry with the existing binary available. All 86 runnable gates
therefore passed on the repaired source. The preceding stale-worker failure was
fixed with the maintained generator. The nine focused files passed 83 tests; the
HTTP file passed again after adding the public E-Prime evidence assertion.
The initial 880 bounded projections retained their declared lexical identity,
but two E-Prime compliance certificates lacked independent evidence. The
finite forbidden-word filter also omitted archaic elisions. A reproducing regression failed on
“’Tis ready.” The JavaScript and native validators now require independent
E-Prime evidence. The two earlier certificates are withdrawn. The new capture
contains 880 cases with violations or missing validators and no compliant
cases. Neither capture is the full issue matrix or independent semantic proof.

The [issue case study](../../docs/case-studies/issue-1584/README.md) contains
the bounded raw archive and SHA-256 inventory. The separate
[PR case study](../../docs/case-studies/pull-request-1585/README.md) records CI.

The first repair batch passed 82 focused tests across nine files, including procedure
execution, streaming/JSON report consistency, source-witness integrity, module
mapping, runtime inventory drift, and the audit wrapper.

The first pushed head had three failed workflows. The PR case study records
each root cause, the procedure-preserving rewrite repair, source-witness review,
and the exact-version upstream audit acknowledgement. Native behavior remains
unverified; passing source/type checks must not be counted as runtime parity.
