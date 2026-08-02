# Issue #658 — WASM worker capability inventory (E39 / R380)

Goal: absorb the ~26,700 lines of solver logic in `src/web/worker/*.js` into the
Rust→WASM worker so only UI/glue (message plumbing, seed fetching, IndexedDB,
`postMessage`) stays in JavaScript. This document is the map that keeps the
migration honest: it records where every capability lives today, what already
runs in WASM, and the slices that move the remainder across.

It is intentionally a living document. Each migration slice should update the
"Owner" column below and lower `CEILING_TOTAL_LINES` in
`scripts/check-worker-line-budget.rs` so the reduction is locked in.

## Current state (2026-07)

- `src/web/worker/*.js`: **26,708 lines** across 22 auto-split modules
  (`formal_ai_worker_00.js … _21.js`), loaded by `src/web/formal_ai_worker.js`.
  These modules define **~1,088 top-level functions** — the bulk of the demo's
  reasoning.
- `src/web/formal_ai_worker.wasm`: **~93 KiB**, built from
  `src/web/wasm-worker/src/lib.rs` via `src/web/wasm-worker/build.sh`
  (`rustc --target wasm32-unknown-unknown -C opt-level=z …`, `#![no_std]`).
- The WASM crate `#[path]`-includes four shared source files that also compile
  into the native crate, so a single Rust definition powers both the CLI/library
  and the browser:
  - `src/language.rs`
  - `src/arithmetic.rs`
  - `src/web_engine_core.rs`
  - `src/web_search_core.rs`

### Already delegated to WASM

The JS worker already calls these exports (`wasm.*`) instead of running its own
copy — they are the beachhead the rest of the migration widens:

| WASM export | Rust source | Replaces in JS |
| --- | --- | --- |
| `engine_normalize_prompt` | `web_engine_core::normalize_prompt` | prompt normalization |
| `engine_detect_language` | `language::detect` | script/marker language detection |
| `engine_evaluate_arithmetic` | `arithmetic` + `web_engine_core::evaluate_arithmetic_expression` | arithmetic evaluation |
| `engine_stable_id` | `web_engine_core::stable_id` | deterministic event IDs |
| `engine_select_unknown_opener` | `web_engine_core::select_unknown_opener` | localized "I don't know" openers |
| `engine_match_intent_route` | `web_engine_core::matches_intent_route_payload` | intent-route keyword/phrase matching |
| `web_search_plan` / `web_search_fuse` / `web_search_request_evidence` / `web_search_registry_dump` / `web_search_*` scalars | `web_search_core` | provider registry, RRF fusion, request evidence |
| `classify` | `web_engine_core` | coarse prompt classification |

Memory protocol exports (`input_ptr`, `output_ptr`, `input_capacity`,
`output_capacity`) back the JS↔WASM byte bridge and stay in WASM permanently.

## Capability clusters still in JavaScript

Grouped by the dominant function-name prefixes in `src/web/worker/*.js`. The
"Owner" column is the migration target; "JS" means not yet moved.

| Cluster | Representative functions | Approx. surface | Owner | Rust counterpart / target |
| --- | --- | --- | --- | --- |
| Text detection & guards | `is*` (76), `contains*` (13), `has*` (11), `looks*` (11), `mentions*` (8) | ~120 fns | JS | fold into `web_engine_core` predicates |
| Extraction | `extract*` (71) | ~71 fns | JS | `web_engine_core` extractors returning typed structs |
| Rendering / formatting | `render*` (41), `format*` (8), `write*` (11), `localized*` (10) | ~70 fns | JS | template layer in Rust; JS only paints the returned string |
| Parsing | `parse*` (40) | ~40 fns | JS | `web_engine_core` / dedicated parser modules |
| Normalization & cleaning | `normalize*` (21), `strip*` (24), `clean*` (19) | ~64 fns | partial | extend `normalize_prompt` family |
| Language detection | `detect*` (25) | ~25 fns | partial (`language::detect` live) | remaining `detect*` variants |
| Arithmetic & numeric | `arithmetic*` (11), `numeric*` (14), `evaluate*` | ~30 fns | partial (`arithmetic` live) | widen `arithmetic` coverage |
| Algebra / programs | `polynomial*` (11), `linear*` (9), `program*` (9), `blueprint*` (9) | ~38 fns | JS | new Rust modules mirrored into WASM |
| Coding / software handlers | `coding*` (14), `software*` (12) | ~26 fns | JS | new Rust modules mirrored into WASM |
| Web search glue | `web*` (9), `search*` (10), `fetch*` (8) | ~27 fns | partial | `web_search_core` (planning/fusion live); `fetch*` stays JS (network I/O) |
| Relative-meta-logic mirror | `formal_ai_worker_21.js` | 679 lines | JS | mirrors `src/relative_meta_logic.rs` — migrate as a unit |

Counts are prefix approximations over the 1,088 top-level functions, not exact
module boundaries; they exist to size slices, not to be authoritative.

## What stays in JavaScript (the ≤3,000-line UI-glue target)

These are legitimately browser-side and are the residue the budget targets:

- `postMessage` plumbing and the worker message loop.
- `seed_loader.js` fetches and the `FORMAL_AI_WORKER_MODULES` bootstrap.
- IndexedDB / dialog-local memory persistence.
- `fetch*` network calls for live web search (the WASM computes the *plan*; JS
  performs the I/O and hands results back for fusion).
- Asset-version cache-busting (`currentAssetVersion`, `withAssetVersion`).
- The hard-coded `FALLBACK_*` strings used only when `seed/*.lino` fails to load
  from a `file://` URL.

## Migration slices

Ordered to move the most self-contained, highest-line clusters first while the
parity fixtures (`data/parity/*`, `data/meta/response-language-followup-recipe.lino`)
and the `tests/e2e/` Playwright suites hold behaviour constant at each step.

1. **Guards (this PR).** Fix the latent `no_std` build break so WASM is built in
   CI again; add the line-budget ratchet
   (`scripts/check-worker-line-budget.rs`) and the size budget
   (`scripts/check-wasm-worker-size.rs`); publish this inventory. No behaviour
   moves yet — this stops the mirror from silently regrowing and re-arms the
   build guard.
2. **Extraction + parsing** (`extract*`, `parse*`): pure string→struct functions
   with no I/O; large line savings, straightforward parity fixtures.
3. **Rendering / formatting** (`render*`, `localized*`, `format*`): move template
   construction into Rust; JS only paints the returned string.
4. **Algebra & program handlers** (`polynomial*`, `linear*`, `program*`,
   `blueprint*`, `coding*`, `software*`): new Rust modules, mirrored into WASM
   via `#[path]`.
5. **Relative-meta-logic** (`formal_ai_worker_21.js`): migrate against the
   existing `src/relative_meta_logic.rs` mirror as one unit.
6. **Residual predicates & normalization**: fold remaining `is*`/`contains*`/
   `normalize*`/`strip*` helpers into `web_engine_core`.

After each slice: run the parity + e2e suites, rebuild and commit the `.wasm`,
and lower `CEILING_TOTAL_LINES` to the new worker line total.

## Enforcement (CI)

The `lint` job in `.github/workflows/release.yml` runs, in order:

- `rust-script scripts/check-worker-line-budget.rs` — ratchet: worker JS may
  shrink but never exceed the recorded ceiling; fails the build on regrowth.
- `sh src/web/wasm-worker/build.sh` — rebuilds the worker for
  `wasm32-unknown-unknown`, catching any `no_std` regression (the class of bug
  that had silently broken the build before this issue).
- `rust-script scripts/check-wasm-worker-size.rs` — keeps the shipped `.wasm`
  under the agreed budget so the offline GitHub Pages demo stays small.
