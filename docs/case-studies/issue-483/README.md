# Issue #483 Case Study: Small-Model Formalization Fallback

Issue [#483](https://github.com/link-assistant/formal-ai/issues/483).
Case study written on the `qa-reasoning-coding-bulk-fixes` branch.

## The ask

Smallest models that actually help formalization matching (surface →
grounded Wikipedia/Wikidata entity), given options to choose from and
selecting the best match, unit-tested; off by default and not loaded
unless explicitly enabled in settings; only hardware-fitting models
displayed; sorted by public ratings; on-demand download only — nothing
bundled in the package or web UI; formal first, LLMs never at the
steering wheel.

## Research data collected

- [link-assistant/model-in-browser](https://github.com/link-assistant/model-in-browser):
  the in-repo experiment runs **SmolLM2-135M-Instruct fully
  client-side** — Candle (HuggingFace's Rust ML framework) compiled to
  WASM, safetensors weights, ~270 MB downloaded on first load and
  browser-cached after, ~512 MB free memory recommended, inference in a
  Web Worker, Unlicense. That is the proven floor for "smallest model
  that runs where we run".
- The second family worth cataloguing: **small encoder rerankers**
  (MiniLM cross-encoders ~22M params, multilingual-e5-small ~118M).
  Cross-encoders are built for exactly the issue's shape — score
  (query, option) pairs, pick the best — and they are far smaller than
  any decoder; the multilingual ones cover ru/hi/zh, which our
  formalization traffic needs.
- The Habr article (1047344) was reviewed for ideas; its candidates
  (quantized decoders in browser) are the same family model-in-browser
  already demonstrates, so nothing there supersedes the proposal below.
  (Not fetched at draft time; re-verify before acting on this line.)

## Solution (this pull request)

- `data/seed/small-model-catalog.lino` (+ byte-identical
  rust/embedded/data/seed/ mirror): 8 model rows — decoders from
  SmolLM2-135M (the proven browser floor) up to Phi-3.5-mini, and the
  two encoder rerankers — each with `ram_required_mb`,
  `disk_required_mb`, an approximate public `rating` with named
  `rating_source` (Open LLM Leaderboard / MTEB tiers; sorting material,
  not correctness claims), license, and `packaged false` on every row.
  Plain data fields only (no lexeme blocks → no language-parity gaps).
- `rust/src/small_model_fallback.rs`:
  - `SmallModelOptions::default()` is **off**, with RAM 0 so even a
    caller that forgets to check `enabled` gets nothing;
    `enabled_for(ram_mb)` is the explicit user enable.
  - `eligible()`: hardware-fit gate (`ram_required_mb <= available`)
    plus a rating floor, **sorted by public rating, best first**
    (smaller params break ties).
  - `download_manifest()`: builds the on-demand plan (source, size);
    the module performs **no I/O** — nothing downloads, nothing ships.
  - `propose_best_match()`: options in → one
    [`ModelProposal`] out (index, confidence, `needs_review: true`),
    logged as `small_model:proposal`/`small_model:candidate`; silence
    (not a guess) at zero affinity. The scorer is a deterministic
    lexical stand-in so the contract is testable offline; a downloaded
    model replaces it behind the same signature.
  - **Steering wheel**: `confirm_proposal(proposal, rule_agrees)` —
    the model never commits; the formal rules hold the pen.

## Verification

- Unit tests in the module + integration tests in
  `rust/tests/unit/issue_483_small_model_fallback.rs` (CI): off-by-
  default loads nothing; the hardware gate hides 6 GB models on a
  1 GB machine and the list is rating-sorted; every catalog row is
  `packaged false` with a sized on-demand manifest; "metre" picks
  Q11573 from the offered options and logs it; the proposal is
  advisory and rules-gated; zero overlap stays silent.

## Known components / integration sites (for main)

- Runtime: model-in-browser's Candle-WASM path for decoders in the web
  UI; a Rust-side runner (candle) for CLI/desktop. **Dependency need**:
  `candle-core`/`candle-transformers` (+ `hf-hub` for on-demand pulls)
  in rust/Cargo.toml — NOT added by this fork (Cargo.toml is main's);
  the seed + guards are backend-agnostic until then.
- Settings surface: `.lenv` / `formal-ai with` needs a
  `small_model.enabled` + `small_model.ram_mb` pair writing
  `SmallModelOptions` (main's lane).
- lib.rs: `pub mod small_model_fallback;`; tests/unit/mod.rs:
  register `issue_483_small_model_fallback`; seed-registry row for
  `data/seed/small-model-catalog.lino`.

## Residuals

- Replace the lexical stand-in with real model scoring once a runtime
  crate lands (same signature, guards unchanged).
- Verify the catalog's approximate ratings before surfacing them in a
  UI (they gate ordering only today).
- Cross-check the Habr article's techniques when online access is
  available in a lane that can cite it.
