# Online research - Issue #428

Collected on 2026-06-12 for
<https://github.com/link-assistant/formal-ai/issues/428>.

## Sources

| Source | URL | Why it was checked |
| --- | --- | --- |
| Upstream repository | <https://github.com/link-foundation/meta-language> | Primary source for current capabilities, README, repository activity, issues, and tags. |
| Cargo package | <https://crates.io/crates/meta-language/0.40.0> | Registry evidence that 0.40.0 is the latest crate release consumed by Cargo. |
| Rust API docs | <https://docs.rs/meta-language/0.40.0> | Public API surface for the version adopted in this PR. |
| Upstream release | <https://github.com/link-foundation/meta-language/releases/tag/v0.40.0> | Release metadata and publication timestamp. |
| Version comparison | <https://github.com/link-foundation/meta-language/compare/v0.39.0...v0.40.0> | Commit and file-change evidence for the dependency bump. |
| Tree-sitter | <https://tree-sitter.github.io/tree-sitter/> | Existing parsing engine family used by upstream language grammars instead of a custom parser stack. |
| LinksPlatform Doublets | <https://github.com/linksplatform/Doublets> | Existing persistent links-network backend family relevant to upstream's optional `doublets` feature. |
| Formal AI code search | `gh search code --owner link-assistant meta-language` | In-organization evidence for where `meta-language` is already wired into Formal AI. |

The archived raw files in this directory contain the command output used for the
case study: Cargo search/info output, upstream GitHub metadata, release metadata,
tag data, issue lists, and `v0.39.0...v0.40.0` compare data.

## Findings

### Latest package status

`cargo search meta-language --limit 5` reported `meta-language = "0.40.0"`.
The upstream GitHub release list also reported `v0.40.0` as the latest release,
published on 2026-06-12. The previous Formal AI dependency was 0.39.

`cargo info meta-language@0.40.0` reports:

- version: 0.40.0
- license: Unlicense
- repository: <https://github.com/link-foundation/meta-language>
- Rust version: 1.77
- default features: none
- optional feature: `doublets`

The Rust-version requirement is not new for this PR; the previous 0.39 crate
also required Rust 1.77.

### Upstream capabilities relevant to Formal AI

The upstream README and issue list show that 0.40 includes the building blocks
requested by issue #428:

- `LinkNetwork` parsing, concrete syntax projection, text reconstruction, and
  full-match verification.
- Source-token and syntax-node insertion helpers plus `render_source`, which
  matter for generating target-language source from a constructed syntax
  network.
- Query and replacement APIs for structured link-network manipulation.
- Snapshot, mutable snapshot, and storage abstractions, including an optional
  Doublets-backed store.
- Translation-rule and concept-to-language mapping infrastructure.
- Incremental edit and diff APIs.
- Natural-language grammar fixtures and mixed-language region support.
- Expanded Tree-sitter-backed language coverage compared with Formal AI's
  current public program-language catalog.

Closed upstream issues confirm that several items which were previously missing
or speculative have landed before this bump:

- Source generation/unparsing.
- Parser registry and additional grammar waves.
- Natural-language grammar parsing.
- Concept space and translation-rule registry.
- Query/transform algebra.
- Incremental reparsing, snapshot, and diff support.

### Formal AI integration audit

Current Formal AI usage is concentrated in the coding CST layer:

- `Cargo.toml` declares the `meta-language` dependency.
- `src/coding/cst.rs` calls `LinkNetwork::parse`, checks
  `verify_full_match`, inspects concrete-syntax projected links, and verifies
  `reconstruct_text`.
- `data/seed/program-cst-grammars.lino` records that each currently catalogued
  Formal AI program language uses the `meta_language` CST engine.
- `tests/source/source_tests/coding/cst/tests.rs` enforces coverage and parse
  behavior for the language catalog.

The bump to 0.40.0 is therefore directly testable through the existing CST and
source suites. There were no source-level API changes required for the current
call sites.

### Why this PR does not add every new upstream grammar

Upstream 0.40 depends on many additional parser crates, but Formal AI exposes a
smaller supported program-language catalog. Adding a public language in Formal
AI is more than adding a parser: it needs aliases, code idioms, deterministic
examples, answer-rendering behavior, execution or oracle policy, source tests,
and browser-worker parity where applicable.

For that reason this PR updates the engine dependency and documents the larger
plan, but does not advertise new Formal AI languages merely because upstream can
parse them.

## Recommended follow-up work

1. Add a focused source-generation experiment using upstream `render_source` to
   construct a small program from a syntax network, validate it through the
   existing CST bridge, and compare the rendered result with Formal AI's current
   idiom-based snippets.
2. Prototype link-network query/replacement for one existing seed-editing task,
   using upstream `LinkQuery`/replacement APIs instead of local string or
   structure-specific logic.
3. Evaluate upstream translation-rule and concept-space data against Formal AI's
   meaning seed model, starting with a tiny reversible meaning-to-text example.
4. Add any newly supported public programming language only after catalog,
   idiom, execution/oracle, source-test, and worker-parity requirements are
   satisfied.
5. Consider a persistent link-store experiment with the optional `doublets`
   feature only after there is a concrete workflow that needs storage semantics
   beyond the in-memory network used by current tests.
