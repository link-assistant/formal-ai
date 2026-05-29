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
- Compositional `comments` axis: a blueprint program is now a *projection* of
  its decomposed capabilities rather than a single frozen string. When the
  request asks for comments the documented program is emitted; when it does not,
  whole-line documentation (and a leading Python docstring) is stripped, leaving
  a smaller but still byte-for-byte compilable program. This makes the
  `comments` sub-task observably change the output — reasoning from the
  decomposition instead of memoizing one answer (`NON-GOALS.md`). Verified by
  unit tests in `src/coding/blueprint.rs`, mirrored in the JS worker, and
  compile-checked offline via `examples/issue_340_emit_variants.rs`.

### Changed
- Browser worker parity (R7): `src/web/formal_ai_worker.js` mirrors the
  blueprint synthesizer byte-for-byte, so the GitHub Pages WASM/JS demo answers
  composite program requests identically to the Rust core. A `vm`-sandboxed
  parity experiment (`experiments/issue-340-worker-parity.mjs`) asserts both
  engines agree across English/Russian Rust, Python, and JavaScript variants,
  that the `comments` capability composes the program identically in both
  engines, and that partial requests (no statistics) stay honestly unsupported.
