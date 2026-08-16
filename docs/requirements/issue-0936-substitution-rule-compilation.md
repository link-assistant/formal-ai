# Issue #936 Substitution-Rule Compilation

Issue [#936](https://github.com/link-assistant/formal-ai/issues/936) is E84:
compile the existing Turing-complete substitution-rule representation to
embeddable targets while retaining Rust ownership and the verified synthesis
path. The implementation and evidence are described in
[`../case-studies/issue-936/`](../case-studies/issue-936/README.md).

| ID | Requirement | Status / Evidence |
| --- | --- | --- |
| R936-1 | Lower `replace x y`, `when n do m`, variables, and ordered composition through one target-neutral compiler IR. | `src/substitution_compiler/mod.rs` defines the serializable IR and lowers every parsed condition/action/pattern; `counter_loop_executes_identically_in_rust_javascript_and_webassembly` exercises conditions, multiple actions, shared bindings, and three composed rewrites. |
| R936-2 | Make generated Rust the canonical executable target. | `rust.rs` and `rust_runtime.rs` emit a standalone stdin/stdout program whose bounded fixpoint and ordered graph semantics match `SubstitutionGraph`; the parity test compiles it with `rustc -D warnings`. |
| R936-3 | Provide JavaScript and WebAssembly for embedding without growing parallel JavaScript logic. | Both targets execute the generated Rust runtime as WASM. The JavaScript primary is an ES-module interoperability bridge with no matching/rewrite implementation; the test rejects an `applyRule` mirror and executes both targets through Node. |
| R936-4 | Export only a proven synthesized program plan. | `ProgramPlan::compile` rejects unchanged or guard-terminated plans. `try_export_substitution_program` reuses `construct_rule_from_unknown` and its semantic fixture before compiling; `program_plan_exports_only_after_a_verified_finite_rewrite` pins the gate. |
| R936-5 | Let users request any target through the existing solver in English, Russian, Hindi, and Chinese. | Target/export cues and localized responses live in seed data; `verified_program_plan_exports_are_seeded_in_four_languages` asserts routing, verification evidence, named source, target, and executable recipe for all four languages and all targets. |
| R936-6 | Verify a small loop/counter against the interpreter on every target and export and execute a prompt result. | The cross-target test performs exact output comparison; the example records a native Rust run in `manual-export-run.log`. `run_issue_936.sh` additionally resumes a real Agent CLI session, writes JavaScript/WASM/IR/input artifacts, compiles them, and pins the exact Node output in `agent-cli-export-e2e/`. |
| R936-7 | Preserve root cause, red/green tests, self-development attempts, release metadata, one-PR traceability, and at least one genuinely self-authored leaf. | Session `ses_ff77c472cffej9Hmz346niSMgQ` authored `data/meta/substitution-compiler-contract.lino`; its byte-identical artifact and raw traces are under `self-hosting-authorship/`. The case study, this matrix, E84 roadmap/architecture entries, changelog fragment, and PR [#1016](https://github.com/link-assistant/formal-ai/pull/1016) form the review record. |
