# Source roots

The repository has three source roots (issue #1138, plan 16 L1). Under the
standing doctrine of 2026-09-24 (R992-R996) all three are full implementation
roots — client and backend — kept equivalent through the meta language rather
than by shrinking any root out; the honest per-root state below says how far
each is today. Commands below run from the repository root.

| Root | Role | Edited by |
| --- | --- | --- |
| `rust/` | The Rust crate: `rust/src/` (library and binary sources), `rust/tests/` (unit, integration, source-mirror and web suites), `rust/examples/`, `rust/Cargo.toml`. Cargo commands need `--manifest-path rust/Cargo.toml` from the root. Today this is the complete implementation root; the js → rust leg translates part of the agentic crate through the meta language (R1000); the rest of the Rust root is hand-written. | hand |
| `js/` | The JavaScript root: `js/app/` (site), `js/worker/` (the worker, including `formal_ai_worker.js`, and the handler mirrors of `rust/src/solver_handlers/`), `js/wasm-worker/`, `js/tests/`. `scripts/sync-seed.sh` copies `data/seed/*.lino` into `js/seed/` as a deploy artefact for GitHub Pages; the site reads it through `js/seed_loader.js` and `js/seed-files.js`. `js/formal_ai_worker.wasm` sits at the root of `js/` next to its loader in `js/worker/`. Parity state (re-checked 2026-10-08): every native handler has a JavaScript twin — `scripts/check-js-parity.mjs` measures 0 native-only handler-registry rows against the ceiling in `data/meta/js-parity-ratchet.lino` (R997-R998) — and `js/server/` serves the HTTP surface from the same worker (R1013, still partial); the remaining gaps are listed with their reasons in `docs/case-studies/pull-request-1188/README.md`. Worker modules stay under the per-module ceilings of `scripts/check-worker-line-budget.rs` (R995, R999). | hand |
| `ts/` | The TypeScript root, generated from `js/`: `node scripts/translate-es.mjs --write` regenerates it and `--check` fails when it drifts (R1000, delivered 2026-10-06; the JavaScript translator is a byte-for-byte twin of the native `formal-ai translate --from js --to ts`). Hand edits are forbidden. See `ts/README.md`. | translator |

Supporting trees that are not source roots: `data/seed/` (the canonical
knowledge surface every interface reads), `data/meta/` (ledgers and ratchets),
`scripts/` (verification gates and tooling), `docs/`, `experiments/`
(reproducible experiment harnesses), and `.github/workflows/`.

The layout is pinned by `rust/tests/unit/issue_1138_three_source_roots.rs`.
