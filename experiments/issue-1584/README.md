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

- Ten focused JavaScript tests exercise option validation, independent mode
  switching, Unicode word counting, opaque input, and declared equivalence.
- Three HTTP tests cover request-local overrides, invalid configuration, SSE
  metadata, and rewrite handling before tool planning. Seven existing server
  tests provide integration regression coverage.
- One proposition has two declared realizations in each of English, Russian,
  Hindi, Chinese and Spanish. Its finite profile/density matrix preserves the
  declared lexical identity. This is not the solver's complete intent graph
  and is not a corpus across all requested input standards.
- Rust tests mirror the limited corpus and configuration boundaries. They have
  not been compiled locally; repository policy leaves Rust execution to CI.
- Real browser checks cover persisted settings and a rewrite turn. Screenshots
  are in `docs/screenshots/issue-1584/`. The browser and preview server were
  closed after verification. These screenshots do not prove vocabulary limits.

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
timeout 600 node --test --test-concurrency=1 experiments/issue-1584/server.test.mjs
timeout 600 node --test --test-concurrency=1 rust/tests/web/server.test.mjs
node scripts/translate-js-rust.mjs --verify
node experiments/formal_ai_subagent/local-gates.mjs
```

The shared policy and seed registries require the full local gate set. Skipped
CI-only Rust gates must not be counted as locally passed. The generated JS-to-
Rust fixture contains carried source and refused constructs: its generation
exits nonzero, while source currency verification passes. The native module is
a supplied manual port; automatic translation and runtime parity are unproven.

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
