# Source roots

The repository has three source roots (issue #1138, plan 16 L1). Commands below
run from the repository root.

| Root | Role | Edited by |
| --- | --- | --- |
| `rust/` | The Rust crate: `rust/src/` (library and binary sources), `rust/tests/` (unit, integration, source-mirror and web suites), `rust/examples/`, `rust/Cargo.toml`. Cargo commands need `--manifest-path rust/Cargo.toml` from the root. | hand |
| `js/` | The web surface: `js/app/` (site), `js/worker/` (the worker, including `formal_ai_worker.js`, and the handler mirrors of `rust/src/solver_handlers/`), `js/wasm-worker/`, `js/tests/`. `scripts/sync-seed.sh` copies `data/seed/*.lino` into `js/seed/` as a deploy artefact for GitHub Pages; the site reads it through `js/seed_loader.js` and `js/seed-files.js`. `js/formal_ai_worker.wasm` sits at the root of `js/` next to its loader in `js/worker/`. | hand |
| `ts/` | TypeScript generated from `js/` by `formal-ai translate --from js --to ts --write` (plan 16 L2). Hand edits are forbidden; fix the source or the translator. See `ts/README.md`. | generated |

Supporting trees that are not source roots: `data/seed/` (the canonical
knowledge surface every interface reads), `data/meta/` (ledgers and ratchets),
`scripts/` (verification gates and tooling), `docs/`, `experiments/`
(reproducible experiment harnesses), and `.github/workflows/`.

The layout is pinned by `rust/tests/unit/issue_1138_three_source_roots.rs`.
