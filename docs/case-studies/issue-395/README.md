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
Rust handlers now also validate the rendered source through the
[link-foundation/meta-language](https://github.com/link-foundation/meta-language)
links network (`meta_language::LinkNetwork`) — a single, mutable CST/AST
representation — before accepting the program.

This PR also updates the related Python program-synthesis handler so it no
longer stores a completed Python function as the candidate. It now stores a
`PythonFunctionTree` with statement nodes, renders source from that tree, and
records the meta-language CST for the generated Python source.

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
- Generated code for supported programming languages must be parsed by the
  meta-language links network (the sole CST/AST engine) rather than a
  hand-built parser. meta-language ships real tree-sitter grammars for every
  language we target, so all of them go through the same links network.
- Related code-writing handlers should expose the same structural synthesis
  trace when they build code.

## Research Anchors

The implementation now uses the same split as common syntax tooling: semantic
meaning trees are projected into source, and concrete source is validated by a
language parser:

- [link-foundation/meta-language](https://github.com/link-foundation/meta-language):
  "a language about languages" — parses source into a single mutable
  links-network CST/AST and ships real tree-sitter grammars for every language
  we target.
- [Tree-sitter basic parsing](https://tree-sitter.github.io/tree-sitter/using-parsers/2-basic-parsing.html):
  concrete syntax trees keep token-level structure, while named-node traversal
  can behave like an AST.
- The engine for each language is declared in
  `data/seed/program-cst-grammars.lino`; the trace records
  `component meta-language` and the meta-language label used for the parse.
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

Code generation itself is seed data, not code. `data/seed/coding-idioms.lino`
declares, for each of JavaScript, TypeScript, Python, Rust, Go, Ruby, Java, C#,
C, and C++, a scaffold per operation family plus named idioms — code fragments
whose cases are selected by operation and value class — inherited through
`extends` (TypeScript extends JavaScript). The composer in
`src/solver_handlers/numeric_list/codegen.rs` discovers the composition at
execution time: it walks the language's inheritance chain, picks the scaffold
for the program's operation family, and recursively expands `{idiom}` slots,
so there are no per-language renderer functions and covering a new language or
coding task is a seed-data change. When the knowledge base has no matching
case, composition fails explicitly (`None`) rather than falling back silently.
The program tree is logged as `synthesis:syntax_tree` before the rendered
`composition:code_fragment`.

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
`numericListProgramLinks`, and composer flow — it embeds the same
`coding-idioms.lino` knowledge base (regenerated with
`experiments/generate-coding-idioms-embed.mjs`) and expands it with the same
algorithm — so browser evidence and Rust evidence describe the same program
shape and the generated code is byte-identical.
`experiments/issue-395-cross-runtime-codegen-parity.mjs` proves this
exhaustively: it replays all 170 (operation × language × value class) prompts
dumped by `examples/numeric_list_matrix.rs` through the worker and requires
every answer to match the Rust engine's answer byte for byte.

### Meta-language CST/AST Validation

`src/coding/cst.rs` reads engine metadata from
`data/seed/program-cst-grammars.lino` and validates each supported language
through the meta-language links network (the primary CST/AST engine). After a
handler renders source, meta-language parses it; a real grammar parse populates
`LinkType::Syntax` links, so the handler only proceeds when those syntax links
are present, the text round-trips, and `has_error` is false. meta-language 0.39
ships grammars for every language we target — JavaScript, Python, Rust, Java, C,
C++, C#, TypeScript, Go, and Ruby — so all of them go through the same links
network.

TypeScript, Go, and Ruby were validated through a thin direct tree-sitter bridge
in earlier revisions of this PR while meta-language gained coverage. Those gaps
were reported upstream and resolved
([#41](https://github.com/link-foundation/meta-language/issues/41),
[#42](https://github.com/link-foundation/meta-language/issues/42),
[#43](https://github.com/link-foundation/meta-language/issues/43)), and the
bridge has since been removed.

One missing upstream feature still keeps code *generation* outside the links
network: meta-language 0.39 can parse and round-trip existing source
(`reconstruct_text` renders only token links that carry byte spans from a prior
parse), but a programmatically constructed syntax network cannot be unparsed
into target-language source text. Until that exists, the composer renders text
from the coding-idioms seed and meta-language validates it after the fact —
reported upstream as
[#64](https://github.com/link-foundation/meta-language/issues/64) (render
source from a constructed syntax network), which would make generated code
syntactically valid by construction.

The trace records the engine and the CST evidence:

```text
synthesis:cst_engine meta_language
synthesis:cst_tree cst_tree
  language javascript
  engine meta_language
  component meta-language
  source_repository https://github.com/link-foundation/meta-language
  language_label javascript
  projection concrete_syntax
  syntax_link_count 45
  has_error false
  text_preserved true
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
the function tree. Native Rust also logs a `synthesis:cst_engine` and a
`synthesis:cst_tree` entry for the rendered Python function, validated through
meta-language.

### Meaning-Driven Recognition

The numeric-list handler reads operation meanings from
`data/seed/numeric-list-operations.lino` and operation surface forms from the seed
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
- `cargo run --example numeric_list_execution` — compiles and runs every
  generated program with the real toolchains and asserts stdout equals the
  solver's computed result
- `node --check src/web/formal_ai_worker.js`
- `node experiments/issue-395-js-numeric-list.mjs`
- `node experiments/issue-395-cross-runtime-codegen-parity.mjs` — byte-compares
  all 170 (operation × language × value class) answers between the Rust engine
  and the worker mirror

The reproducing tests assert that the exact Russian issue prompt routes to
`write_program`, that an unsorted English JavaScript prompt computes the sorted
result, that a quoted-string JavaScript sort uses `value_type string`, and that
the Links Notation trace includes both `program_syntax_tree` semantic nodes and
the meta-language CST (`component meta-language`, `has_error false`).
