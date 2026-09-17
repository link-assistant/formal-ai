## Issue #1138 Composition From Sources

Coding answers are composed from typed, attributable fragments rather than
benchmark names or copied answers. The executable contract is covered by
`tests/unit/coding_discovery/composition.rs`.

| ID | Requirement | Status / evidence |
| --- | --- | --- |
| R1138-B2-1 | Retrieved procedure text is formalized into ordered steps before code is composed. | Implemented in the procedure-text and structural-composition stages; covered by `tests/unit/coding_discovery/composition.rs`. |
| R1138-B2-2 | Composition uses a language-neutral typed program IR. | Implemented by `src/coding/program_ir.rs`; covered by `tests/unit/coding_discovery/program_ir.rs`. |
| R1138-B2-3 | Target-language lowering is selected after composition and preserves the IR identity. | Implemented by the IR lowerers; covered by `tests/unit/coding_discovery/ir_lowering.rs`. |
| R1138-B2-4 | The bootstrap fragment catalog is deletable data, not a hard-coded task answer. | Implemented by the fragment catalog; covered by `tests/unit/coding_discovery/fragment_catalog.rs`. |
| R1138-B2-5 | Forgetting and rediscovering a fragment from the same capture reproduces its content id. | Covered by `tests/unit/coding_discovery/fragment_catalog.rs`. |
| R1138-B2-6 | Upstream suites are graded by their real tests and publish honest pass totals. | Covered by `tests/unit/specification/external_benchmarks.rs`. |
| R1138-B2-7 | Seed fragments use typed, human-readable shapes and may not encode benchmark answers. | Covered by `tests/unit/coding_discovery/no_memorization.rs`. |
| R1138-B2-8 | OEIS and Python documentation are selected through the trusted-source registry with provenance and license metadata. | Covered by `tests/unit/coding_discovery/oeis.rs` and `tests/unit/coding_discovery/python_docs.rs`. |
