# Source roots

The repository has three source roots (issue #1138, plan 16 L1). Under the
standing doctrine of 2026-09-24 (R992-R996) all three are full implementation
roots — client and backend — kept equivalent through the meta language rather
than by shrinking any root out; the honest per-root state below says how far
each is today. Commands below run from the repository root.

| Root | Role | Edited by |
| --- | --- | --- |
| `rust/` | The Rust crate: `rust/src/` (library and binary sources), `rust/tests/` (unit, integration, source-mirror and web suites), `rust/examples/`, `rust/Cargo.toml`. Cargo commands need `--manifest-path rust/Cargo.toml` from the root. Today this is the complete implementation root. | hand |
| `js/` | The JavaScript root: `js/app/` (site), `js/worker/` (the worker, including `formal_ai_worker.js`, and the handler mirrors of `rust/src/solver_handlers/`), `js/wasm-worker/`, `js/tests/`. `scripts/sync-seed.sh` copies `data/seed/*.lino` into `js/seed/` as a deploy artefact for GitHub Pages; the site reads it through `js/seed_loader.js` and `js/seed-files.js`. `js/formal_ai_worker.wasm` sits at the root of `js/` next to its loader in `js/worker/`. Parity state: partial — the web app plus the generated solver mirror (35 modules under the shrink ratchet `scripts/check-worker-line-budget.rs`); the full-backend parity migration is plan 16 L2-L5. | hand |
| `ts/` | The TypeScript root. `formal-ai translate --from js --to ts` is its intended production path (plan 16 L2) and is pending: today the command reports the honest gap naming the leaf, and `ts/` is a README stub, not yet generated output. See `ts/README.md`. | translator (pending L2) |

Supporting trees that are not source roots: `data/seed/` (the canonical
knowledge surface every interface reads), `data/meta/` (ledgers and ratchets),
`scripts/` (verification gates and tooling), `docs/`, `experiments/`
(reproducible experiment harnesses), and `.github/workflows/`.

The layout is pinned by `rust/tests/unit/issue_1138_three_source_roots.rs`.
