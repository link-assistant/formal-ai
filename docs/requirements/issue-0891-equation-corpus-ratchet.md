## Issue #891 Equation Corpus Ratchet

Issue [#891](https://github.com/link-assistant/formal-ai/issues/891) (child of
[#710](https://github.com/link-assistant/formal-ai/issues/710)) closes the issue
[#406](https://github.com/link-assistant/formal-ai/issues/406) requirement of at
least fifty *verified* equation-type examples. The delegation tests asserted
equation categories but nothing defined them in machine-readable form and
nothing counted them, so no ratchet could fail on a regression. The corpus lives
in `data/benchmarks/equation-type-corpus.lino`, the ratchet in
`tests/unit/specification/equation_corpus.rs`, and the analysis in
`docs/case-studies/issue-891/`.

| ID | Requirement | Status / Evidence |
| --- | --- | --- |
| R891-1 | Define a machine-readable corpus with at least 50 distinct equation types. | Implemented: `data/benchmarks/equation-type-corpus.lino` defines 72 distinct `equation_type` records; `issue_891_equation_corpus_is_well_formed` asserts distinctness and the 50-type floor. |
| R891-2 | Run every case through the production solver and verify the result. | Implemented: every expected answer is the observed output of `FormalAiEngine::answer` (`examples/issue_891_equation_probe.rs`); `issue_891_equation_corpus_solves_every_type` replays all 72 cases and compares intent, engine and exact answer. |
| R891-3 | Add a CI ratchet that fails below 50 verified types or on any corpus regression. | Implemented: the ratchet asserts `passed >= minimum_pass_count` (72) and `verified_types >= minimum_verified_types` (50) inside the default `cargo test --test unit` job. |
| R891-4 | Record category coverage. | Implemented: seven categories (linear one-step / multi-step, placeholder, symbolic multi-variable, polynomial, natural-language wrapper, evaluation-and-percent) with counts in `docs/benchmarks.md`; the category set is pinned by the well-formedness test, which reads the expected language coverage from `registered_languages()`. |
| R891-5 | Record upstream calculator limitations. | Implemented: ten `benchmark_limitation` records (irrational/complex roots, contradiction, malformed input, identity, unit-carrying equations, named-unknown declarations, command-shaped prompts) asserted by `issue_891_recorded_limitations_never_fabricate_answers` to keep declining rather than fabricating. |
| R891-6 | Solve the class, not the prompts: new equation-solving cues belong to the seed. | Implemented: `data/seed/meanings-calculator.lino` gains `calculation_request_cue` surfaces for en/ru/zh/hi equation phrasings and a first Spanish lexeme; the Rust engine and the JavaScript worker read them from the same seed and `src/` carries no hardcoded phrase. |
