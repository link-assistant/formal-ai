## Issue #1138 Composition From Retrieved Sources

Retrieved procedure text — not an enumerated shape catalog — is the source of
coding answers. The steps are formalized into a language-neutral IR and lowered
per language; the seeded idiom catalog that bootstrapped this path is itself
deletable data whose loss is provably recoverable. The contract is covered by
`tests/unit/coding_discovery/` (procedure text, program IR, IR lowering,
composition search, multilingual parity) and the full-suite upstream rows in
`data/benchmarks/external-results.lino`.

| ID | Requirement | Status / evidence |
| --- | --- | --- |
| R1138-B2-1 | Retrieved procedure text becomes an ordered, typed step list with per-step source spans, never a stored answer body. | Implemented by `coding_discovery::procedure_text`; covered by `tests/unit/coding_discovery/procedure_text.rs`. |
| R1138-B2-2 | The ordered step list lowers into a language-neutral `ProgramIr` whose nodes carry types, fragments, grounding, and license metadata. | Implemented by `src/coding/program_ir.rs`; covered by `tests/unit/coding_discovery/program_ir.rs`. |
| R1138-B2-3 | The `ProgramIr` renders per target language — the lowerer is selected after composition and preserves the IR identity — and equivalent requirements in English, Russian, Hindi, Chinese, and Spanish expose the same frontier. | Implemented by IR lowering and Python rendering; covered by `tests/unit/coding_discovery/ir_lowering.rs` and the 25 held-out cases in `tests/unit/coding_discovery/multilingual.rs`. |
| R1138-B2-4 | The seeded idiom catalog is a deletable bootstrap: every fragment is declared `bootstrap true` with a rediscovery query, and deleting it loses no source evidence. | Implemented by `data/seed/coding-composition-fragments.lino`; covered by `tests/unit/coding_discovery/fragment_catalog.rs`. |
| R1138-B2-5 | Forgetting a discovered procedure and rediscovering it from the same trusted sources reproduces the same content id. | Implemented by the discovered-procedure ledger; covered by `forgotten_procedures_are_rediscovered_from_the_same_sources` in `tests/unit/coding_discovery/ledger.rs`. |
| R1138-B2-6 | The upstream coding suites are measured at full slice (HumanEval 164, MBPP 500), online and cold-offline, with per-slice floors and the recorded failure frontier — a first-20 score is a regression control, never a suite score. | Measured by `benchmark run --suite humaneval --slice 164` and `--suite mbpp --slice 500`, each cold-offline and with `--online`; rows recorded 2026-09-17 in `data/benchmarks/external-results.lino` with `--frontier-record`; floors raised by `external_benchmarks::ratchet`. |
| R1138-B2-7 | Every held-out structural answer reports a `typed_search(…)` composition over seed fragments, and no benchmark identifier, task sentence, assertion, or expected-output literal may leak into source or seed data — an example's expected value is payable only as a conditional branch payload, never as a condition operand. | Covered by `tests/unit/coding_discovery/structural_composition.rs` and `tests/unit/coding_discovery/no_memorization.rs`; the observation-literal slot filter is enforced in `src/coding/composition_search.rs`. |
| R1138-B2-8 | OEIS and the Python standard-library documentation are selected through the trusted-source registry as declared sources with pinned extractors, endpoints, licenses, and content-addressed caches. | Declared in `data/seed/sources-registry.lino` (`oeis`, `python_docs`); replay pinned by `tests/unit/coding_discovery/oeis.rs` and `tests/unit/coding_discovery/python_docs.rs`. |
