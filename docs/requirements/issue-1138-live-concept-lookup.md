## Issue #1138 Live Concept Lookup

The universal and coding paths share one bounded, attributable source walk.
The requirements below are pinned by
`rust/tests/unit/issue_1138_concept_lookup.rs` and
`rust/tests/unit/issue_1138_universal_loop_lookup.rs`.

| ID | Requirement | Status / evidence |
| --- | --- | --- |
| R1138-B1-1 | One source-lookup implementation walks the sources registry and serves both the universal loop and coding discovery. | Implemented by `rust/src/concept_lookup.rs::SourceLookup`; covered by `rust/tests/unit/issue_1138_concept_lookup.rs`. |
| R1138-B1-2 | Lookup is bounded by declared depth, pages, services, and capture age. | Implemented by `rust/src/source_walk.rs::LookupBounds`; covered by `rust/tests/unit/issue_1138_concept_lookup.rs`. |
| R1138-B1-3 | Every retrieved sense carries source id, exact URL, digest, fetch time, license, and depth; offline replay is byte-stable. | Implemented by the sense ledger and capture cache; covered by `rust/tests/unit/issue_1138_concept_lookup.rs`. |
| R1138-B1-4 | A retrieved gloss is attributed and is never inserted into generated code as an answer template. | Enforced by the sense projection; covered by `rust/tests/unit/issue_1138_universal_loop_lookup.rs`. |
| R1138-B1-5 | A miss names every consulted source and its outcome instead of guessing. | Implemented by the lookup outcome ledger; covered by `rust/tests/unit/issue_1138_concept_lookup.rs`. |
| R1138-B1-6 | Source-service opt-outs are authoritative for concept lookup. | Implemented by service preferences; covered by `rust/tests/unit/issue_1138_concept_lookup.rs`. |
| R1138-B1-7 | Held-out words absent from seed data resolve or report an attributable miss in English, Russian, Hindi, Chinese, and Spanish. | Covered by `rust/tests/unit/issue_1138_universal_loop_lookup.rs`. |
| R1138-B1-8 | Deleting the derived sense ledger and replaying the same captures reproduces the same content ids. | Covered by `rust/tests/unit/issue_1138_concept_lookup.rs`. |
| R1138-B1-9 | Native and browser runtimes use one source-walk contract and one parity fixture. | Covered by `rust/tests/unit/issue_1138_concept_lookup.rs` and the browser parity suite. |
