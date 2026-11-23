# Actual CodeQL producer and included payloads, 2026-11-23

The read-only [Security run 45093409488](https://github.com/link-assistant/formal-ai/actions/runs/45093409488)
actually finishes Actions and Rust extraction and both custom diagnostic queries
on `cec9207c3e139e2212f6d4ea478ff1b88e17be7c`. CodeQL is 2.27.5,
Rust queries 0.1.47, Rust 1.99 and the action 4.39.4. Both SARIF invocations are
successful with zero query results and no warning/error notifications. Rust
retains 57 note-level and 9,221 level-none notifications; Actions retains 5,967
level-none notifications. These observations qualify producer execution only.

The diagnostic artifacts are IDs 14783126107 (contexts), 14783061566 (complete
Rust results) and 14781679819 (Actions). The complete original artifacts, full
workflow log, query sources, decoded output, evaluator traces and source-bound
comparison are retained in the 159-member archive at
`experiments/issue-1580/evidence/ci/codeql-producer-cec9207c3/`.
It has 93,718,695 original bytes and 15,572,400 archived bytes, with SHA-256
`06d37e0c530bd49b467b296ce526f46c6157a96eb3c0c8b234a759dc1990d4c0`.
Every member is round-trip verified against the manifest.

The macro query returns 123,763 rows. The included-string query returns 2,108
rows in 259 source files: 1,854 literal expansions, 253 unexpanded contexts and
one other expansion. It has 285 rows with an expansion CFG node and 1,823 without.
Its complete JSON SHA-256 is
`f4c8deba276a13b8804423cef2b4c62c1e536c9395326936547624d5a6140a6e`.
The query completes with the existing 8,192-MB CI memory cap, 6,245-MB evaluator
heap and four threads; the retained evaluator trace records its execution.

Literal text is a Rust token, not automatically a decoded payload. The retained
comparison checks exact source spans at this immutable commit, resolves literal
relative include paths, and decodes only JSON-compatible quoted tokens. Of those
rows, 541 are empty expansions of nonempty included sources, nine exactly match
source bytes, 24 require a Rust-specific decoder, 1,279 have a nonliteral or
unbound span, and one included source is unavailable. These are context rows,
not unique applications or a security-coverage percentage.

For example, `rust/src/google_trends_catalog.rs:33:5` includes
`rust/embedded/data/seed/google-trends-prompts.lino`, whose 2,372 bytes hash to
`589a2477b200e941d00f819f0f8c0a7b00855f1961d5a22845dbc273f0cd4faa`.
The extractor reports an empty literal, with no expansion CFG node. The same
source-bound mismatch occurs for handler-promotion and language seed payloads.
Therefore a successful producer and zero SARIF findings do not prove semantic
extraction of these actual included files.

Current rust-analyzer [include_str expansion](https://github.com/rust-lang/rust-analyzer/blob/37721d141370e0904033f7fc3a3abe11c446c4c1/crates/hir-expand/src/builtin/fn_macro.rs#L874)
explicitly returns an empty string when a relative file cannot be loaded and
documents excluded non-Rust files as a limitation. The pinned source matches the
retained bytes. This is consistent with the observed mismatch, but the exact
rust-analyzer revision vendored by this CodeQL bundle is not established here;
that causal attribution remains an inference. The observed payload mismatch
itself comes from the actual current producer.

The earlier [macro-expansion issue #19982](https://github.com/github/codeql/issues/19982)
is closed and its maintainer asks for distinct remaining problems to be reported
separately. [Issue #23176](https://github.com/github/codeql/issues/23176) concerns
Actions checkout classification and remains a separate open report. No findings
are suppressed and no payloads are replaced with empty stubs. Included-file
semantic coverage, future main-branch alert state and the wider issue 1580
acceptance criteria remain unfinished. Supplied analysis receives zero
independent authorship or reasoning credit.
