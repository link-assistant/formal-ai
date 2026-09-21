## Standing Doctrine: Compiled Logic, Interfacing-Only JavaScript (2026-08-04)

Stated by the project owner as a standing architectural requirement; it
strengthens R194/R249/R380 from "as much as possible" to a boundary rule.

| ID | Requirement | Status / Evidence |
| --- | --- | --- |
| R536 | JavaScript must be used only as interfacing glue and for JSX (React) UI components. All logic must be compiled Rust — native on the server side, WebAssembly in the web app — and the same WASM web engine must be reused by the desktop shell and other surfaces (VS Code, etc.) rather than reimplemented. | Partially implemented: `js/wasm-worker/` owns the parity-sensitive primitives (about 2,156 lines of Rust) and `js/app/main.jsx` is the JSX UI; `js/worker/` still carries 35 generated `*.js` modules totalling 30,179 lines (as of 2026-09-18) of mirrored solver logic under the shrink-only ratchet `scripts/check-worker-line-budget.rs`, whose per-module ceilings in `data/meta/worker-line-budget/` sum to the same figure and whose end-state target is 3,000 lines of UI glue. Desktop serves the same `js/` engine bundle and prefers the native `formal-ai serve` process; the VS Code web host runs the in-process WASM engine. The absorption epic [#658](https://github.com/link-assistant/formal-ai/issues/658) (R380) closed with PR #691 on 2026-07-18; what remains is the measured shrink itself, enforced downward by the budget script — no tracker owns it beyond the ratchet. After absorption the JavaScript surface is capped and lint-enforced as UI/glue. |
