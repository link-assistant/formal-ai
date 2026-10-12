**Problem statement.**
Source: #331 PR comment (https://github.com/link-assistant/formal-ai/pull/331#issuecomment-4577374430). konard: "When we have substitution rules that are Turing complete we should be able to convert them to Rust/JavaScript/WebAssembly — all options should be available."
Current-code evidence: no substitution-rules-to-Rust/JavaScript/WebAssembly compiler exists anywhere. src/substitution.rs is a pure matcher/instantiator; src/skill_compiler lowers natural language to associative packages only. No emit_rust/to_js/wasm-compile path exists in src.

**What to do.**
1. Design an IR-to-target-language emitter for Turing-complete substitution rules (replace x y, when n do m and their compositions).
2. Implement a Rust code-emission backend first (aligns with the JS=glue/all-logic-in-Rust doctrine — this is the canonical target).
3. Implement JS and WASM emission as secondary backends, explicitly for interop/embedding use cases (e.g. shipping a standalone compiled rule as a browser snippet) — not as a parallel logic implementation that violates the JS doctrine.
4. Wire this into the existing rule-synthesis / program-plan handlers so a proven substitution-rule program can be exported in any of the three forms on request.

**How to test.**
- Automated: unit tests compiling a small Turing-complete rule set (e.g. a counter/loop construct) to each of the three targets and executing the emitted code, asserting identical output to the interpreted rule.
- Manual: run formal-ai against a substitution-rule prompt, request Rust/JS/WASM export, and execute each emitted artifact outside the process.
- Multilingual: emission target language is independent of prompt language, but confirm the prompt that produces the rule set works in en/ru/hi/zh.
- Standing clauses: docs/case-studies/issue-{id}; single PR; verbose trace of the compilation steps if correctness is hard to verify statically.

**Source refs:** #331 (R331-3). **Dedup:** none.

