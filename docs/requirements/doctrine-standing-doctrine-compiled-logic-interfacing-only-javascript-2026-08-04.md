## Standing Doctrine: Compiled Logic, Interfacing-Only JavaScript (2026-08-04)

Stated by the project owner as a standing architectural requirement; it
strengthens R194/R249/R380 from "as much as possible" to a boundary rule.

| ID | Requirement | Status / Evidence |
| --- | --- | --- |
| R536 | JavaScript must be used only as interfacing glue and for JSX (React) UI components. All logic must be compiled Rust — native on the server side, WebAssembly in the web app — and the same WASM web engine must be reused by the desktop shell and other surfaces (VS Code, etc.) rather than reimplemented. | Partially implemented: `src/web/wasm-worker/` owns the parity-sensitive primitives and `src/web/app/main.jsx` is the JSX UI; `src/web/worker/*.js` still carries ~27,700 lines of mirrored solver logic under the shrink-only ratchet `scripts/check-worker-line-budget.rs`. Desktop serves the same `src/web/` engine bundle and prefers the native `formal-ai serve` process; the VS Code web host runs the in-process WASM engine. Full absorption of the JS worker into Rust→WASM is tracked by [#658](https://github.com/link-assistant/formal-ai/issues/658) (R380); after absorption the JavaScript surface is capped and lint-enforced as UI/glue. |
