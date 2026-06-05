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
computes the deterministic result in both Rust and the browser worker. Native
Rust handlers now also validate the rendered source with real Tree-sitter
grammars through `tree-sitter/lib/binding_rust` before accepting the program.

This PR also updates the related Python program-synthesis handler so it no
longer stores a completed Python function as the candidate. It now stores a
`PythonFunctionTree` with statement nodes, renders source from that tree, and
records the Tree-sitter CST for the generated Python source.

## Requirements

- The exact issue prompt must not fall through to `unknown`.
- The answer must contain runnable JavaScript and the actual result.
- Recognition must be meaning driven, multilingual, and shared by Rust and the
  web worker.
- The solution must be broader than one sorting phrase: the same engine covers
  `sort`, descending sort, `reverse`, `sum`, `product`, `minimum`, and `maximum`
  over numeric lists, and the transformation path also handles quoted string
  list data.
- Code generation must manipulate a CST/AST-like representation, not memorize
  final source strings.
- Generated code for supported programming languages must be parsed by real
  Tree-sitter language bindings rather than a hand-built parser.
- Related code-writing handlers should expose the same structural synthesis
  trace when they build code.

## Research Anchors

The implementation now uses the same split as common syntax tooling: semantic
meaning trees are projected into source, and concrete source is validated by a
language parser:

- [Tree-sitter basic parsing](https://tree-sitter.github.io/tree-sitter/using-parsers/2-basic-parsing.html):
  concrete syntax trees keep token-level structure, while named-node traversal
  can behave like an AST.
- Tree-sitter Rust bindings are loaded from grammar crates listed in
  `data/seed/meanings-program-cst.lino`; the trace records
  `binding tree-sitter/lib/binding_rust` and the grammar crate used.
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
CST/AST requirement or prove that generated code parsed as real source.

## Implementation

### Numeric-List Programs

`src/solver_handlers/numeric_list/codegen.rs` now lowers the prompt to a
`NumericProgram` tree. For the original numeric prompt, the trace looks like:

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

The same tree shape is used for quoted text lists:

```text
program_syntax_tree
  language javascript
  value_type string
  operation sort
  literal_values pear|apple|banana
  semantic_node literal_list name=numbers mutable=false
  semantic_node sort_list source=numbers target=sorted direction=ascending
  semantic_node print_joined source=sorted separator=", "
```

`src/web/formal_ai_worker.js` mirrors the same `numericListBuildProgram`,
`numericListProgramLinks`, and renderer flow so browser evidence and Rust
evidence describe the same program shape.

### Tree-sitter CST Validation

`src/coding/cst.rs` loads grammar metadata from
`data/seed/meanings-program-cst.lino` and maps each supported language to its
real Tree-sitter Rust grammar crate. After a handler renders source, it parses
that source and only proceeds when the CST root kind matches the expected
language root and `has_error` is false.

The trace records the concrete parser path:

```text
synthesis:cst_tree tree_sitter_cst_tree
  language javascript
  parser tree_sitter
  binding tree-sitter/lib/binding_rust
  grammar_crate tree-sitter-javascript
  expected_root_kind program
  has_error false
```

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
the function tree. Native Rust also logs a `synthesis:cst_tree` entry for the
rendered Python function.

### Meaning-Driven Recognition

The numeric-list handler reads operation meanings from
`data/seed/meanings-numeric-list.lino` and operation surface forms from the seed
operation vocabulary. Language aliases continue to use the existing
`program_language_*` meanings. Reductions are gated behind a `code_request`
meaning so ordinary arithmetic prose is not stolen by code generation. The
value domain (`integer`, `float`, or `string`) is inferred from parsed list data
and recorded in both formalization and synthesis evidence.

## Verification

Focused checks used for this issue:

- `cargo test issue_395 -- --nocapture`
- `cargo test cst -- --nocapture`
- `cargo test program_synthesis -- --nocapture`
- `cargo test --example numeric_list_execution`
- `node --check src/web/formal_ai_worker.js`
- `node experiments/issue-395-js-numeric-list.mjs`

The reproducing tests assert that the exact Russian issue prompt routes to
`write_program`, that an unsorted English JavaScript prompt computes the sorted
result, that a quoted-string JavaScript sort uses `value_type string`, and that
the Links Notation trace includes both `program_syntax_tree` semantic nodes and
the real Tree-sitter CST with `has_error false`.
