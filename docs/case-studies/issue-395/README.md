# Case Study: Issue #395

> Raw GitHub inputs are preserved under [`raw-data/`](./raw-data/): the issue
> JSON, comments JSON, and issue body. This file reconstructs the defect,
> requirements, design decision, and verification.

## Summary

The deployed wasm assistant answered this Russian prompt with `intent: unknown`:

```text
У меня есть числа 3, 5, 6, 7, 8 отсортируй их в JavaScript, дай мне код и результат
```

The prompt asks for a program and for the computed result. The fix now routes
that request to `write_program`, resolves the operation and target language from
seed meanings, builds a semantic syntax tree, renders code from that tree, and
computes the deterministic result in both Rust and the browser worker.

This PR also updates the related Python program-synthesis handler so it no
longer stores a completed Python function as the candidate. It now stores a
`PythonFunctionTree` with statement nodes and renders source from that tree.

## Requirements

- The exact issue prompt must not fall through to `unknown`.
- The answer must contain runnable JavaScript and the actual result.
- Recognition must be meaning driven, multilingual, and shared by Rust and the
  web worker.
- The solution must be broader than one sorting phrase: the same engine covers
  `sort`, descending sort, `reverse`, `sum`, `product`, `minimum`, and `maximum`
  over numeric lists.
- Code generation must manipulate a CST/AST-like representation, not memorize
  final source strings.
- Related code-writing handlers should expose the same structural synthesis
  trace when they build code.

## Research Anchors

The implementation uses a small project-native syntax tree rather than adding a
parser dependency, but it follows the same split used by common syntax tooling:

- [Tree-sitter basic parsing](https://tree-sitter.github.io/tree-sitter/using-parsers/2-basic-parsing.html):
  concrete syntax trees keep token-level structure, while named-node traversal
  can behave like an AST.
- [ESTree](https://github.com/estree/estree): JavaScript tooling standardizes
  source manipulation around typed `Program` and statement nodes.
- [Babel parser output](https://babeljs.io/docs/babel-parser): Babel parses to a
  Babel AST derived from ESTree.
- [syn crate docs](https://docs.rs/syn/latest/syn/): Rust procedural macro
  tooling parses tokens into syntax tree nodes and prints them back to tokens.

## Root Cause

No handler combined the three meanings in the prompt:

- a numeric-list operation (`sort`);
- a literal list of numbers;
- a programming-language target (`JavaScript`).

Existing code could recognize pieces of that request, but dispatch had no
meaning-level numeric-list coding path. The old branch also generated final code
directly from language-specific strings, which did not satisfy the issue's
CST/AST requirement.

## Implementation

### Numeric-List Programs

`src/solver_handlers/numeric_list/codegen.rs` now lowers the prompt to a
`NumericProgram` tree:

```text
program_syntax_tree
  language javascript
  value_type integer
  operation sort
  semantic_node literal_list name=numbers mutable=false
  semantic_node sort_list source=numbers target=sorted direction=ascending
  semantic_node print_joined source=sorted separator=", "
```

Renderers for JavaScript, TypeScript, Python, Rust, Go, Ruby, Java, C#, C, and
C++ project that tree into source code. The same tree is logged as
`synthesis:syntax_tree` before the rendered `composition:code_fragment`.

`src/web/formal_ai_worker.js` mirrors the same `numericListBuildProgram`,
`numericListProgramLinks`, and renderer flow so browser evidence and Rust
evidence describe the same program shape.

### Related Program Synthesis

`src/solver_handlers/program_synthesis.rs` and the web mirror now build a
`PythonFunctionTree` instead of storing a completed function string on the
candidate. Supported synthesis tasks are represented as statement nodes such as
`pairwise_outer_loop`, `threshold_match_return`,
`vowel_membership_set_assignment`, and
`matching_character_count_return`.

The handler logs:

```text
synthesis:syntax_tree python_function_syntax_tree
  semantic_node function_definition signature="count_vowels(text: str) -> int"
  semantic_node vowel_membership_set_assignment ...
  semantic_node matching_character_count_return ...
```

The final Python code is still shown to the user, but it is now a projection of
the function tree.

### Meaning-Driven Recognition

The numeric-list handler reads operation meanings from
`data/seed/meanings-numeric-list.lino` and operation surface forms from the seed
operation vocabulary. Language aliases continue to use the existing
`program_language_*` meanings. Reductions are gated behind a `code_request`
meaning so ordinary arithmetic prose is not stolen by code generation.

## Verification

Focused checks used for this issue:

- `cargo test issue_395 -- --nocapture`
- `cargo test program_synthesis -- --nocapture`
- `cargo test --example numeric_list_execution`
- `node --check src/web/formal_ai_worker.js`
- `node experiments/issue-395-js-numeric-list.mjs`
- A node VM check that `tryProgramSynthesis` emits
  `synthesis:syntax_tree:python_function_syntax_tree`.

The reproducing tests assert that the exact Russian issue prompt routes to
`write_program`, that an unsorted English JavaScript prompt computes the sorted
result, and that the Links Notation trace includes `program_syntax_tree` and
semantic operation nodes.
