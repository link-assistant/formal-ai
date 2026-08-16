# Issue #936 Case Study: Compile Substitution Rules

Issue [#936](https://github.com/link-assistant/formal-ai/issues/936) asks E84
to compile Formal AI's existing substitution rules to Rust, JavaScript, and
WebAssembly. It explicitly drops #331's broader execution-stack requirement,
so this change stays at the rule/program-plan boundary and ships in one pull
request: [#1016](https://github.com/link-assistant/formal-ai/pull/1016).

## Reproduction and root cause

Before this change, `src/substitution.rs` could parse and interpret ordered
`when`/`replace` rules, and `src/program_plan.rs` plus `src/rule_synthesis.rs`
could construct and semantically verify a modified plan. No representation
connected those layers to a code target. There was no compiler IR, emitter,
artifact contract, verified-plan export gate, or target vocabulary. A proven
rule therefore stopped at an in-process `SubstitutionGraph` trace.

The red-first regression is
`tests/unit/issue_936_substitution_compiler.rs`. Its counter uses three ordered
rules, shared variables, conditions, multiple replacements, and a preserved
unrelated link. The initial compile failed only because
`compile_substitution_rules`, `CompiledSubstitutionProgram`, and
`SubstitutionCompilationTarget` did not exist; the exact diagnostic is retained
in [`red-test.log`](red-test.log).

## Design

The compiler has one semantic route:

```text
SubstitutionRuleSet
  -> SubstitutionProgramIr
  -> generated Rust runtime
       -> native Rust executable
       -> WebAssembly module
       -> JavaScript ES-module interop
```

The IR records the rule-set id, the interpreter's 64-application guard,
ordered manual rules, conditions, actions, and literal/variable/prefix-variable
patterns. Each export includes a pretty-printed `.substitution-ir.json` file and
a stable trace naming its source rule set, target, stages, and parity
requirement.

Rust is canonical. WebAssembly is the same generated `no_std` Rust runtime with
a bounded allocator and buffer ABI. The JavaScript target contains only
encoding, memory-copy, instantiation, and Node/browser glue around that WASM;
it deliberately contains no matching or rewrite algorithm. This follows the
standing architecture rule that JavaScript is interface glue and logic remains
Rust or Rust-to-WASM.

`ProgramPlan::compile` is stricter than the public low-level compiler: it
requires at least one applied rewrite, rejects unchanged plans, and rejects a
termination-guard result. The solver export route first calls the existing
unknown-rule constructor, including its generated-program semantic fixture,
then compiles that same plan. This prevents a natural-language request from
exporting an unproved candidate.

## Verification

- `counter_loop_executes_identically_in_rust_javascript_and_webassembly`
  compares exact sorted TSV output from the interpreter, generated native Rust,
  the JavaScript/WASM interop target, and the standalone WASM target.
- `program_plan_exports_only_after_a_verified_finite_rewrite` checks the proof
  gate's positive, unchanged-plan, and termination-guard paths.
- `verified_program_plan_exports_are_seeded_in_four_languages` drives English,
  Russian, Hindi, and Chinese conversation follow-ups across Rust, JavaScript,
  and WebAssembly. It checks localized output, semantic-verification trace,
  parity trace, named artifact, and execution recipe.
- `cargo run --example issue_936_export_and_execute` starts with a natural
  language prompt, exports its verified plan, compiles the returned source, and
  executes it over a graph. The captured successful run is
  [`manual-export-run.log`](manual-export-run.log).
- `experiments/agent_cli_e2e/run_issue_936.sh` boots the release server, drives
  the real Agent CLI for two turns, resumes session
  `ses_ff77cb103ffe3hhhGqf2qquEkR`, writes JavaScript/WASM/IR/input artifacts,
  compiles the generated Rust to WASM, and executes Node. Its exact output is
  `request:task\tlist_files_reverse_sort`; the raw traces and sources are in
  [`agent-cli-export-e2e/`](agent-cli-export-e2e/).

The Ubuntu full-test CI leg installs `wasm32-unknown-unknown`, so the WASM
executable assertion runs in CI rather than silently skipping when the target
is absent.

## Formal AI / Agent CLI development

The task was first attempted through a release build of `formal-ai serve` and
the real Agent CLI. The decomposition used five leaves: red test; compiler IR
and Rust emitter; WASM/JavaScript interop; verified multilingual solver export;
and documentation/release integration.

Three early attempts at the smallest test/fixture leaf are preserved verbatim here.
The first was misclassified as GUI rendering. The second exposed a completion
gate feedback loop because its live log was inside the observed worktree. The
third isolated logs outside the repository and reached the file-tool loop, but
the client repeatedly selected `read` for a requested new file and exhausted
recovery without a workspace effect. The exported failed session is
[`agent-cli-session.json`](agent-cli-session.json), with raw client/server logs
beside it. Those attempts do not count as self-authored lines.

After the concrete agentic gaps were repaired, session
`ses_ff77c472cffej9Hmz346niSMgQ` successfully wrote and verified
`substitution-compiler-contract.lino` through `formal-ai serve` and the actual
Agent CLI. The captured artifact under
[`self-hosting-authorship/`](self-hosting-authorship/) is byte-identical to
`data/meta/substitution-compiler-contract.lino`, and a regression test pins the
identity and raw session marker. This is one of the five decomposed leaves, so
self-authorship is **1/5 (20%)**. Only the artifact/evidence commit receives
Formal-AI authorship trailers; the manually implemented compiler is not
relabelled.

## Scope

This change compiles the existing bounded substitution runtime; it does not
restore #331's dropped general execution stack. It introduces no external
dependency and no visual UI change, so screenshots and visual-regression tests
are not applicable. Broader browser solver migration remains owned by the
existing Rust-to-WASM work; E84 adds no JavaScript solver debt.
