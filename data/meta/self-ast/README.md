# Self-AST census artifacts

This directory contains one independently generated `.lino` document for each
owned Rust module. Regenerate those documents with:

```sh
cargo run --example regenerate_self_ast_census
```

The whole-workspace aggregate is intentionally not committed. It includes
counts and content identifiers for every module, so tracking it made unrelated
source branches edit and conflict in the same `index.lino` file. Render the
same deterministic aggregate when needed with:

```sh
cargo run --example dump_self_ast_census
```

Runtime target resolution continues to use the in-memory `WorkspaceCensus`.

## Superseded source-model authority

The owner's [2026-10-18 doctrine](../../../docs/requirements/doctrine-readable-recursive-source-and-comment-meta-language-2026-10-18.md) rejects this census/signature layout as the full source representation. These files are legacy partial inputs until their planner, collectors and tests migrate. Do not add count-only refreshes as delivery of that migration. The replacement must retain complete readable recursive UTF-8 source content, comments/layout, formalized comment meanings and rule-based JS/TS/Rust synchronization.
