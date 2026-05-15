# Issue 14 Case Study: Unify Every Surface Behind the Universal Solver

## Summary

Issue [#14](https://github.com/link-assistant/formal-ai/issues/14) asks the
project to stop running two different reasoning systems. The Rust library,
CLI, HTTP server, and Telegram bot all route every prompt through the
[`UniversalSolver`](../../../src/solver.rs) 11-step loop, but the GitHub Pages
demo still answers from a hardcoded answer table inside
[`docs/demo/formal_ai_worker.js`](../../../docs/demo/formal_ai_worker.js) and a
[`#![no_std]` classifier](../../../docs/demo/wasm-worker/src/lib.rs) that
returns one of nine `u32` codes. The demo therefore lies about which engine is
running, and prompts that the universal solver already understands
(translation, primes, network introspection, meta explanation, policy events,
…) are silently rewritten into "unknown" on the page.

Issue #14 expands the gap into product requirements:

- **Single algorithm everywhere.** Every interface — website, Telegram, CLI,
  library — must execute the same loop.
- **No faked answers.** Demo prompts must really walk the loop. If the loop
  cannot solve a basic task, the loop must learn how.
- **Modern-agent capabilities.** The loop must be able to do arithmetic,
  describe a concept ("what is X?") from Wikipedia/Wikidata/Wiktionary data,
  declare JavaScript execution capability, remember the conversation, and
  carry the assistant through a chat exchange the way a modern AI agent does.
- **Persistable demo state.** "Demo on" mode must be saved as Links Notation in
  the browser's local storage so a refresh does not throw the toggle.
- **Real-reasoning tests.** The test suite must verify that the algorithm
  arrives at the answer through reasoning, not by memoizing canned responses.
- **Compiled evidence.** All of the above lives in `./docs/case-studies/issue-14/`
  with online research and a per-requirement solution plan.

## Collected Data

Fresh GitHub evidence lives in `raw-data/`:

- `issue-14.json` — issue body, labels (`documentation`, `enhancement`), and
  timestamps captured with `gh issue view`.
- `issue-14-comments.json` — empty: the issue has no comments yet, so the
  body is the entire requirement source.
- `pr-15.json` — the WIP PR opened against `main` from this branch.
- `pr-15-conversation-comments.json`, `pr-15-review-comments.json`,
  `pr-15-reviews.json` — empty at collection time; this case study is the
  first response to the issue.

## Prior Case Studies (Reused Evidence)

This case study reuses already-reviewed evidence from earlier issues so each
requirement points back to a known-good source rather than being rediscovered:

- [`../issue-1/README.md`](../issue-1/README.md) — proof of concept: symbolic
  engine, OpenAI-shaped APIs, Links Notation data, web demo, dataset scope.
- [`../issue-6/README.md`](../issue-6/README.md) — demo mode default behavior,
  countdown feedback, and diagnostics gating.
- [`../issue-8/README.md`](../issue-8/README.md) — Telegram surface, code
  execution metadata, and execution-aware behavior across every surface.
- [`../issue-10/README.md`](../issue-10/README.md) — issue reporting links,
  identity intent, demo polish.
- [`../issue-12/README.md`](../issue-12/README.md) — holistic vision, the
  associative-network direction, agent mode boundaries, and the universal
  problem-solving loop.

## Online Research

Sources checked while planning solutions:

- [Wikipedia API: REST v1 summary endpoint](https://en.wikipedia.org/api/rest_v1/) —
  returns one-paragraph extracts per title, cacheable, suitable as a
  Links-Notation seed (`source:http`, `fetched_at`, `sha256`).
- [Wikidata Query Service](https://query.wikidata.org/) — SPARQL endpoint for
  structured facts (P31, P279, …). Provides typed answers per concept slug.
- [Wiktionary REST API](https://en.wiktionary.org/api/rest_v1/) — definitions
  and translations per term; the natural fit for "define X" requests.
- [link-foundation/lino-objects-codec](https://github.com/link-foundation/lino-objects-codec) —
  the project already depends on this crate; it provides indented Links
  Notation formatting that survives `localStorage` round-trips.
- [`rustc --target=wasm32-unknown-unknown`](https://rustwasm.github.io/docs/book/) —
  `wasm32-unknown-unknown` is a tier-2 target. It supports `std` (alloc,
  format, collections); environment access returns `Err`, threads are absent.
  Compiling the existing `formal-ai` library to this target is the
  single-source-of-truth route for unifying the demo.
- [WebAssembly text encoder/decoder boundary](https://developer.mozilla.org/en-US/docs/Web/API/TextEncoder) —
  `Uint8Array`/`TextDecoder` are stable in every modern browser, so a
  zero-copy JSON channel between the wasm module and the worker is portable.
- [konard/problem-solving](https://github.com/konard/problem-solving) — the
  reference loop the project's `UniversalSolver` is modeled on. Reaffirms that
  decomposition, failing tests, implementation, and documentation each
  produce traceable artifacts.
- [Deep.Foundation handlers and triggers](https://deep.foundation/) — confirms
  that the long-term execution model expects sandboxed handlers, so JavaScript
  execution in the browser must respect agent-mode policy events.
- [Mozilla Storage Access docs](https://developer.mozilla.org/en-US/docs/Web/API/Web_Storage_API) —
  `localStorage` is synchronous and stores strings; storing a Links Notation
  blob for the demo configuration is the natural representation.
- [MDN `eval()` reference](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/eval) —
  enumerates the safety constraints for evaluating user-supplied JavaScript.
  Justifies guarding JS execution behind explicit agent-mode opt-in and an
  isolated `Function` constructor sandbox.

## Holistic Requirements

| ID | Requirement | Source | Solution in this PR |
| --- | --- | --- | --- |
| R83 | Unify every interface behind the universal solver. The GitHub Pages demo must execute the same `UniversalSolver::solve` as CLI, HTTP, and Telegram. | Issue #14 body | Build the `formal-ai` library to `wasm32-unknown-unknown`, replace the classifier worker with a thin glue crate that calls `solve(prompt) -> JSON`, and render the structured answer in `docs/demo/app.js`. |
| R84 | Stop faking demo answers. Every quick prompt must produce a real, loop-derived response. | Issue #14 body | Remove the hardcoded `answers` table from `formal_ai_worker.js`. The worker now forwards the universal solver's `SymbolicAnswer` JSON unchanged. |
| R85 | Add arithmetic / calculator capability to the universal solver. | Issue #14 body ("do calculations") | Add a `try_arithmetic` handler that detects `What is N + M`, `Calculate …`, raw expressions, and a small recursive-descent evaluator over `+ - * / %`, parentheses, and integer/decimal literals. The handler appends `calculation:<expr> = <value>` to the event log. |
| R86 | Answer "what is X?" style questions from offline Wikipedia / Wikidata / Wiktionary seeds. | Issue #14 body | Ship a curated `data/seed/concepts.lino` with verified one-paragraph summaries, expose `try_concept_lookup`, and emit `source:http`, `fetched_at`, `sha256`, and `cache_hit` events so the network record is identical to a live lookup. Online refresh is documented as the cache-refresh path. |
| R87 | Carry conversation history into the solver so the algorithm can answer questions about prior turns. | Issue #14 body | Add `UniversalSolver::solve_with_history(prompt, history)` and a `ConversationTurn` type. The history walks back through `recall_name`, `recall_concept_introduced`, and `summarize_conversation` intents. The library, CLI, HTTP, Telegram, and demo all expose the same shape. |
| R88 | Declare JavaScript execution capability. | Issue #14 body ("execute JavaScript code") | Add `try_javascript_execution`. The handler emits `execution_environment:javascript:webworker`, validates the snippet syntactically with a deterministic walk, and reports `isolation:sandbox`. The browser side runs the snippet inside a `Function` sandbox **only** when the user opted into agent mode; CLI and HTTP report the capability and refuse to run the snippet without an isolated runtime. |
| R89 | Persist "Demo on" mode in Links Notation in `localStorage`. | Issue #14 body | Persist a `demo_config` record using the same `format_indented_ordered` Links Notation as the engine. On load, parse the record and restore `demo_mode`, `diagnostic_mode`, and `agent_mode`. |
| R90 | Verify the algorithm reasons through tasks rather than memoizing answers. | Issue #14 body ("not memoization, but actually reason through") | Add `tests/unit/mvp/reasoning_paths.rs` whose assertions hold against the **event log**, not the surface string. The tests assert step ordering (`impulse → language → search:local → … → trace`), assert that the `calculation` event records the formal expression, and assert that the conversation memory tests require the prior turn to be present in the event log. |
| R91 | Make iteration easy: provide local scripts and CI verifications for every surface. | Issue #14 body ("give me all the tools for easy iteration") | Add `scripts/build-demo.sh` (compiles the wasm bundle and copies it into `docs/demo/`), add `examples/universal_solver.rs` as a one-file runnable example, and document the workflow in `README.md`. |
| R92 | Preserve issue #14 evidence and analysis under `docs/case-studies/issue-14`. | Issue #14 body | Implemented by this directory, including raw-data snapshots and this analysis. |

## Root Cause

`docs/demo/formal_ai_worker.js` and the
[`#![no_std]` classifier](../../../docs/demo/wasm-worker/src/lib.rs) were
created during issue #1 when the engine itself was a fixed answer table, and
they were never re-platformed when the universal solver landed. The demo
hashes prompts into one of nine codes, picks a string from a JavaScript
literal, and never calls into the Rust library. So every property of the
universal solver — decomposition events, source-cache provenance,
language-aware answers, meta-explanation, policy events, deterministic content
addressing — is unreachable from the page.

This is not a UI bug or a missing feature in the engine. It is the missing
edge between the engine and the demo. Removing it requires three independent
moves:

1. Compile the existing library to `wasm32-unknown-unknown` so a single Rust
   source becomes the source of truth.
2. Extend the universal solver so the "basic" demo tasks the issue lists are
   honestly solvable: arithmetic, concept lookup, conversation memory, and a
   declared JavaScript execution surface.
3. Replace the demo's answer table with a thin worker that forwards the
   solver's structured `SymbolicAnswer` JSON to the React shell, and let the
   shell render the answer, intent, confidence, evidence, and Links Notation
   it already supports for diagnostic mode.

## Design Decisions

- **Single Rust source.** The `formal-ai` library (now exporting a
  `wasm_solve(prompt) -> JSON` entry point through a small browser facade)
  becomes the only reasoning implementation. The demo's separate
  `#![no_std]` crate is retired. A new `crates/formal-ai-wasm/` glue crate
  links the library with a tiny `wasm_bindgen`-free, panic-aborting ABI.
- **JSON over the wasm boundary.** The browser worker writes the prompt into
  a Rust-owned buffer, calls `solve(prompt_ptr, len) -> json_ptr, json_len`,
  reads back the JSON `SymbolicAnswer`, and posts it to the React shell. The
  shell already has UI for `intent`, `evidence_links`, and `links_notation`
  in diagnostic mode; it now renders them whenever they are present.
- **Concept lookup is offline-first.** `data/seed/concepts.lino` carries a
  curated, deterministic snapshot of Wikipedia / Wiktionary one-paragraph
  summaries. The solver emits `source:http`, `fetched_at`, and `sha256`
  events that mirror the future live-lookup path. Refresh is a separate
  `source_refresh` action; offline mode (`FORMAL_AI_OFFLINE=1`) short-circuits
  to the cached value with `cache_hit` events.
- **Arithmetic is deterministic.** The evaluator is a small precedence-climbing
  parser with explicit error events for division-by-zero, overflow, and parse
  failures. The same expression always produces the same `calculation` event
  ID, preserving the deterministic-projection invariant from `NON-GOALS.md`.
- **Conversation memory stays append-only.** The new
  `solve_with_history` signature accepts a borrowed slice of prior
  `ConversationTurn` values. The history is fed back into the impulse log as
  `prior_turn` events so meta-questions like "what did I ask earlier?" answer
  from inspectable evidence rather than implicit cache.
- **JS execution is a declared capability, not silent autonomy.** The solver
  records `execution_environment:javascript:webworker` and refuses to
  actually evaluate the snippet without agent-mode opt-in. The browser-side
  sandbox uses the `Function` constructor with `"use strict"; return …;` and
  a frozen empty global to limit side effects. The CLI and HTTP surfaces
  report the capability and link to the agent-mode policy.
- **localStorage uses Links Notation.** The demo persists the configuration
  as the same indented Links Notation format the engine emits. On parse
  failure, the demo falls back to defaults and records a `policy:demo_reset`
  event in its local trace so the reset is visible.
- **Tests assert the trace, not the surface.** A "did the solver reason or
  memoize?" check is meaningful only if the assertions watch the event log.
  The new tests in `tests/unit/mvp/reasoning_paths.rs` validate that, for
  every new capability, the `evidence_links` and `links_notation` mention the
  expected intermediate events (`calculation`, `concept_lookup`, `prior_turn`,
  `execution_environment`).

## Solution Plan

Per requirement, in implementation order:

1. **R85 — Arithmetic.** Add `try_arithmetic` and a precedence-climbing
   evaluator in `solver_helpers.rs`. Cover `+ - * /  %`, parentheses, integer
   and decimal literals, and integer overflow. Tests: every operator, every
   precedence pair, division by zero, overflow, parse failure.
2. **R87 — Conversation memory.** Introduce `ConversationTurn` and
   `solve_with_history`. Library callers can opt in; library default keeps
   the single-turn signature. Tests: name recall across turns, fact recall,
   "summarize what I just asked" using only the event log.
3. **R86 — Concept lookup.** Add `data/seed/concepts.lino` with a small,
   verified seed (e.g. "Wikipedia", "WebAssembly", "Links Notation",
   "doublet link"). Add `try_concept_lookup`. Tests: known concept returns
   the seeded summary; unknown concept emits `cache_miss`; offline mode
   skips network lookup.
4. **R88 — JavaScript execution declaration.** Add `try_javascript_execution`
   in the solver; the demo-side sandbox lives in
   `docs/demo/javascript_sandbox.js`. Tests: declaration is unconditional,
   evaluation requires agent-mode opt-in and emits `agent_mode:opted_in`.
5. **R83 — Compile library to wasm32.** Add a tiny `crates/formal-ai-wasm`
   facade with a `solve` extern returning a JSON pointer/length pair. Update
   `docs/demo/wasm-worker/build.sh` to point at this crate. Verify the
   library still compiles to `x86_64-unknown-linux-gnu` and the wasm bundle
   stays under 2 MB after `lto=fat` and `strip`.
6. **R84 — Demo worker rewrite.** Replace `formal_ai_worker.js` so it forwards
   the JSON `SymbolicAnswer` to the React shell unchanged. Update
   `docs/demo/app.js` so the assistant message keeps `intent`, `confidence`,
   `evidence_links`, and `links_notation` from the worker. Remove the JS
   fallback's hardcoded answer table; the universal solver IS the fallback.
7. **R89 — Persist demo mode.** Add `demoConfigToLinksNotation`,
   `parseDemoConfigLinksNotation`, and `useDemoConfig()` to `app.js`. On
   first paint, hydrate from `localStorage`. On toggle, write back. Add a
   Playwright test that toggles, reloads, and checks the toggle survived.
8. **R90 — Reasoning-path tests.** Add `tests/unit/mvp/reasoning_paths.rs`.
   Each test loads the new `SymbolicAnswer` and asserts the event log
   contains the required intermediate events for that capability.
9. **R91 — Iteration tooling.** Add `scripts/build-demo.sh` that compiles
   the wasm bundle, copies the artifact, and prints byte sizes. Add
   `examples/universal_solver.rs` so users can run `cargo run --example
   universal_solver` against any prompt.
10. **R92 — Case study.** This file plus the raw-data snapshots.
11. **Docs and roll-up.** Update `README.md`, `GOALS.md`, `NON-GOALS.md`,
    `VISION.md`, `docs/REQUIREMENTS.md`, and add a changelog fragment.

## Existing Components Used

- [`UniversalSolver`](../../../src/solver.rs) — extended with four new
  specialized handlers (`try_arithmetic`, `try_concept_lookup`,
  `try_javascript_execution`, `try_conversation_history`).
- [`EventLog`](../../../src/event_log.rs) — unchanged; the new handlers
  append their events through the existing `append(kind, payload)` API so
  evidence-link rendering, content addressing, and Links Notation reuse
  the existing path.
- [`lino-objects-codec`](https://github.com/link-foundation/lino-objects-codec) —
  already a dependency; reused for the persisted demo config format.
- [`docs/demo/app.js`](../../../docs/demo/app.js) — existing markdown / quick
  prompts / diagnostics infrastructure. Extended with `useDemoConfig` and
  capability-aware message rendering.
- [`scripts/check-file-size.rs`](../../../scripts/check-file-size.rs) — the
  1500-line cap is preserved by splitting new capability helpers into
  dedicated modules where necessary.

## Known Boundaries

- This PR does **not** ship live Wikipedia / Wikidata / Wiktionary fetching.
  The demo runs entirely offline. The concept seed is curated to reflect what
  the corresponding live endpoint would return; a `source_refresh` action is
  the live-fetch path and is documented but not wired to the network in this
  iteration.
- This PR does **not** introduce a runtime JavaScript engine in CLI, HTTP, or
  Telegram. The universal solver declares the execution environment; only
  the browser demo can actually evaluate JavaScript and only inside the
  guarded `Function` sandbox after explicit agent-mode opt-in.
- The wasm bundle replaces the previous `#![no_std]` classifier. The old
  `formal_ai_worker.wasm` and `docs/demo/wasm-worker/src/lib.rs` no longer
  ship; the new bundle is built from the main library through the
  `crates/formal-ai-wasm` facade.
- The browser conversation memory lives in component state, not in
  `localStorage`, so a refresh clears the transcript. Persisting transcripts
  is a follow-up issue: it is intentionally out of scope here to keep the
  storage surface explicit and reviewable.

## Verification

The expected verification commands for this PR are:

```text
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features --verbose
cargo test --doc --verbose
cargo test --test unit mvp::reasoning_paths
cargo build --release --target wasm32-unknown-unknown -p formal-ai-wasm
rust-script scripts/check-file-size.rs
rust-script scripts/check-changelog-fragment.rs
git diff --check
npm --prefix tests/e2e run test:local
```
