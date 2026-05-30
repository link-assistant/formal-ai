# formal-ai data

This directory stores reviewable Links Notation seed data for issue #1.
`data/benchmarks/` stores curated benchmark fixtures with source revisions and
license provenance for deterministic regression and gap-reporting suites. Large
external benchmark payloads are not stored here; issue #362 network checks fetch
them into `target/formal-ai-benchmarks` at test time.

Generate or refresh the checked-in `.lino` files with:

```bash
rust-script scripts/download-datasets.rs
```

The records use the indented, untyped formatting helpers from
`lino-objects-codec` so reviewers can read and edit them directly. Keep every
`.lino` file at or below 1500 lines; `rust-script scripts/check-file-size.rs`
enforces that limit.
