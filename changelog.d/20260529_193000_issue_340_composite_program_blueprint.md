---
bump: minor
---

### Added
- Composite `write_program` **blueprints**: a request the verified template
  catalog cannot resolve to a single alias (e.g. "make an HTTP GET request,
  parse the JSON, compute the mean and median, and output the results with error
  handling and comments") no longer dead-ends on `write_program_unsupported`.
  The blueprint synthesizer (`src/coding/blueprint.rs`) decomposes the prompt
  into capabilities (http_request, json_parse, statistics, output_results,
  error_handling, comments — each matched in English, Russian, Hindi, and
  Chinese), matches a recipe (`http_json_stats`), and returns a real, idiomatic
  program for Rust, Python, or JavaScript together with a numbered decomposition
  plan, the required libraries, and how-to-run instructions.
- Honest execution contract for blueprints: because composite programs need
  external libraries and network access the offline sandbox cannot provide, the
  blueprint is always reported as **"not run"** and never claims it "compiled and
  ran". The decomposition is recorded as `program_blueprint:` trace links and a
  `response:write_program:blueprint:<recipe>:<language>` evidence link.
- Case study `docs/case-studies/issue-340/` with timeline, requirements,
  root-cause analysis, solution plans, and an existing-components review.
- Two independent compositional axes: a blueprint program is now a *projection*
  of its decomposed capabilities rather than a single frozen string.
  - `comments` axis — when the request asks for comments the documented program
    is emitted; otherwise whole-line documentation (and a leading Python
    docstring) is stripped.
  - `error_handling` axis — optional defensive blocks are wrapped in
    `// region:error_handling … // endregion:error_handling` markers (`#` for
    Python/Ruby): the Rust empty-input guard, the Python `raise_for_status` +
    empty-list guard, and the JavaScript `!response.ok` + empty-array guard. The
    marker lines are always stripped from output; the region body is kept only
    when the request asks for error handling.
  The axes are orthogonal, so one recipe yields the full cross-product of four
  distinct, still-compilable programs (`documented`, `comments_only`,
  `errors_only`, `stripped`) — reasoning from the decomposition instead of
  memoizing one answer (`NON-GOALS.md`). Verified by unit tests in
  `src/coding/blueprint_tests.rs`, mirrored in the JS worker, and compile-checked
  offline via `examples/issue_340_emit_variants.rs` (each emitted Python/JS
  variant passes `py_compile` / `node --check`).
- `BlueprintComposition` setting ("Program composition"): switches the synthesis
  strategy between `Composed` (default — project the program from the decomposed
  capabilities) and `Documented` (always emit the fully annotated program with
  every region and comment). Exposed as a dropdown in the demo UI, toggleable by
  natural language ("documented programs", "полная документация", …), persisted
  in preferences, forwarded to the worker, localized across all four lino-i18n
  locales (en/ru/hi/zh), and reported in the self-facts inventory as
  `relation "blueprint_composition"` across the Rust core, JS worker, and app.js
  local fallback.

### Changed
- Browser worker parity (R7): `src/web/formal_ai_worker.js` mirrors the
  blueprint synthesizer byte-for-byte, so the GitHub Pages WASM/JS demo answers
  composite program requests identically to the Rust core. A `vm`-sandboxed
  parity experiment (`experiments/issue-340-worker-parity.mjs`) asserts both
  engines agree across English/Russian Rust, Python, and JavaScript variants,
  that both the `comments` and `error_handling` axes compose identically in both
  engines, that the `Documented` strategy keeps every region/comment, that the
  active composition is reported in the self-facts, and that partial requests
  (no statistics) stay honestly unsupported.
