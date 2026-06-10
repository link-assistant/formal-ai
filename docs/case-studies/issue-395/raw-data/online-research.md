# Online Research

Captured on 2026-06-04 while revising PR #396 after maintainer feedback about
using CST/AST structures instead of code-string quotes.

## Primary References

- Tree-sitter, "Basic Parsing":
  https://tree-sitter.github.io/tree-sitter/using-parsers/2-basic-parsing.html
  - Relevant point: Tree-sitter syntax nodes expose children, siblings, parents,
    positions, named nodes, and anonymous token nodes; CSTs preserve tokens while
    named-node traversal can support AST-like analysis.
- ESTree specification:
  https://github.com/estree/estree
  - Relevant point: JavaScript tooling standardizes typed program and statement
    nodes for source manipulation.
- Babel parser documentation:
  https://babeljs.io/docs/babel-parser
  - Relevant point: Babel emits an AST derived from ESTree, with documented node
    deviations and parser output options.
- syn crate documentation:
  https://docs.rs/syn/latest/syn/
  - Relevant point: Rust procedural macro tooling parses token streams into
    syntax tree nodes and can print syntax trees back to tokens/source.
- syn `parse_quote!` documentation:
  https://docs.rs/syn/latest/syn/macro.parse_quote.html
  - Relevant point: quasi-quotation can infer and parse into any syntax-tree node
    implementing `Parse`, reinforcing the "build/manipulate tree, then render"
    workflow.

## Design Consequence

The PR uses a project-native intermediate tree instead of adding new parser
dependencies: `NumericProgram` for numeric-list programs and
`PythonFunctionTree` for the related Python synthesis handler. These trees are
logged in Links Notation before rendering source code.
