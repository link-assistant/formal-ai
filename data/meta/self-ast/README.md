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
