# Case study — Issue #340: "Write a Rust program that … HTTP GET → parse JSON → mean/median"

> A composite `write_program` request dead-ended on `write_program_unsupported`
> because no single verified-catalog alias matched the four-step task. This study
> reconstructs the timeline, enumerates every requirement, isolates the root
> cause, and records the solution — a curated *blueprint* synthesizer that
> decomposes a prompt into capabilities, renders a real multi-step program, and
> reports it honestly as "not run".

- **Issue:** [#340](https://github.com/link-assistant/formal-ai/issues/340) — *Issue with dialog: Write a Rust program that: 1. Makes an HTTP GET request to a URL 2. Parses the JSON …*
- **Reported version:** 0.149.0 · WASM worker · manual mode · UI language `ru` · locale `ru-RU` (`Asia/Yekaterinburg`) · diagnostics on
- **Pull request:** [#346](https://github.com/link-assistant/formal-ai/pull/346) (branch `issue-340-dbe3f3891f1b`)
- **Predecessors:**
  - [#312](https://github.com/link-assistant/formal-ai/issues/312) / PR [#318](https://github.com/link-assistant/formal-ai/pull/318) — added the `write_program` `list_files` task and multilingual detection.
  - [#324](https://github.com/link-assistant/formal-ai/issues/324) / PR [#325](https://github.com/link-assistant/formal-ai/pull/325) — language-aware rendering, conversation-context recovery, and the data-driven `program_plan` substitution pipeline (the universal-solver vision's first stage).
  - [#330](https://github.com/link-assistant/formal-ai/issues/330) — split the coding catalog into focused files under the line-count limit.
  This issue is the direct sequel: the catalog now answers many single-task program requests, but a *composite* request that combines several tasks still dead-ends.
- **Raw data:** [`raw-data/`](./raw-data/) — `issue.json`, `issue-comments.json`, `pr-346.json`, `reproduction-dialog.md`.

---

## 1. Timeline / sequence of events

| When (UTC) | Event |
| --- | --- |
| 2026-05-29 18:44:07 | In the GitHub Pages WASM worker (v0.149.0) the user submits a four-step request: **"Write a Rust program that: 1. Makes an HTTP GET request to a URL 2. Parses the JSON response 3. Calculates statistics (mean, median) … 4. Outputs the results. Include error handling and comments."** |
| 2026-05-29 18:44:07 | The intent formalizer recognizes the `write_program` shape (program noun `program` + imperative verb `write`) and extracts the **language** (`rust`) correctly, but no catalog alias matches the composite **task**, so it produces `UnsupportedWriteProgram { task: None, language: Some("rust") }`. The engine renders the dead-end *"I do not have a template for language `rust` and task `missing`. … Supported tasks: hello_world, count_to_three, list_files, list_files_arg."* **(The bug.)** |
| 2026-05-29 18:44:43 | Issue #340 is filed quoting the dialog. |
| 2026-05-29 18:51:57 | Maintainer comment ([#issuecomment-4578776288](https://github.com/link-assistant/formal-ai/issues/340#issuecomment-4578776288)) adds the meta-instructions: increase generalization so users "can actually do all the programming tasks imaginable"; compile data + deep case study; add tracing if data is insufficient; report upstream if applicable; fix across the **entire** codebase (both engines); do everything in one PR. |
| 2026-05-29 (this PR) | Composite-program **blueprint** synthesizer added to the Rust core and mirrored byte-for-byte in the JS worker; case study written; parity experiment + integration tests added. |

The issue body carries **one** comment (the maintainer meta-instructions); every
requirement comes from the issue body and that comment.

---

## 2. Requirements (every explicit and implicit ask)

### From the failing behavior
1. **R1 — Answer composite program requests.** A multi-step request (HTTP GET →
   parse JSON → compute mean/median → output, with error handling and comments)
   must produce a real, idiomatic program covering all sub-requirements instead
   of dead-ending on `write_program_unsupported`.
2. **R2 — Stay honest about execution.** Such a program needs external libraries
   and network access the offline sandbox cannot provide, so the assistant must
   **not** claim it "compiled and ran" — it must report it as "not run" while
   still delivering the code.

### From the meta-instructions appended to the issue
3. **R3 — Increase generalization.** Move beyond a fixed alias catalog toward
   "all the programming tasks imaginable", consistent with the repository vision
   and prior issues.
4. **R4 — Case study.** Download all related data to
   `docs/case-studies/issue-340/` and analyze deeply: timeline, requirements,
   root causes, solution plans, and **online research** of existing components.
5. **R5 — Add tracing if data is insufficient.** If a root cause cannot be
   found, add debug output / a verbose mode (off by default).
6. **R6 — Fix everywhere.** Apply each fix across the *entire* codebase — both
   the Rust core and the JS worker (R7-parity of prior studies).
7. **R7 — Report upstream** to other repositories if the issue belongs there.
8. **R8 — Single PR.** Plan and execute everything in PR #346.

---

## 3. Root-cause analysis

### Root cause — the catalog is alias-keyed, so composite tasks have no match

The defect is **not** in detection or language extraction. The formalizer
correctly recognizes the `write_program` shape and extracts `language = rust`.
The problem is the **task** model: `write_program(language, task)` resolves the
task against a small set of *verified* aliases (`hello_world`, `count_to_three`,
`list_files`, `list_files_arg`). Each alias maps to a program the project has
actually compiled and run, which is what lets a catalog answer honestly claim
"compiled and ran".

A composite prompt ("HTTP GET, parse JSON, compute mean/median, output, with
error handling and comments") matches **no** single alias. The formalizer
therefore returns `UnsupportedWriteProgram { task: None, language: Some("rust") }`
and the engine renders the literal *"task `missing`"* dead-end. **The four
explicit sub-requirements in the prompt were never inspected** — the engine had
no mechanism to decompose a prompt into capabilities and assemble a program from
them.

There is also a structural tension the fix must respect: such a program needs
`reqwest`/`requests`/`fetch` and live network access, none of which the offline
WASM/browser sandbox can run. So it can never join the verified catalog (which
guarantees "compiled and ran"). The missing capability is a **second, clearly
separated path** for programs that are real and useful but honestly *not run*.

This root cause was found directly from the reported trace
(`reproduction-dialog.md`); no additional always-on tracing was required (R5),
though the blueprint path records a full `program_blueprint:` capability trace
(see §4) so future composite-request diagnostics are inspectable.

---

## 4. Solution plans (per requirement) and what was implemented

### R1 / R3 — Composite-program blueprint synthesizer · **done**

A new module keeps composite programs **separate** from the verified catalog so
the catalog's "compiled and ran" invariant is preserved while the broad class of
real-world requests is answered.

- **Capabilities** (`src/coding/blueprint.rs` `CAPABILITIES`): six recognizable
  capabilities — `http_request`, `json_parse`, `statistics`, `output_results`,
  `error_handling`, `comments` — each with keyword sets in **English, Russian,
  Hindi, and Chinese**. CJK keywords match by substring; Latin/Cyrillic match on
  whitespace token boundaries with a stem prefix for keywords ≥ 4 characters.
- **Recipe matching** (`RECIPES`, `select_blueprint`, `detect_capabilities`):
  the prompt is decomposed into the capabilities it contains; a recipe matches
  when all its `required_capabilities` are present. The first recipe,
  `http_json_stats` (required: `http_request`, `json_parse`, `statistics`),
  carries curated programs for **Rust** (`reqwest` blocking + `serde_json`),
  **Python** (`requests` + `statistics`), and **JavaScript** (Node 18+ built-in
  `fetch`).
- **Rendering** (`render`): emits a localized intro, a numbered decomposition
  plan, the fenced program, the required-libraries list, how-to-run
  instructions, and the honest execution report — all localized for en/ru/hi/zh.
- **Solver wiring** (`src/solver_handlers/program_blueprint.rs`,
  `src/solver.rs`): `try_program_blueprint` runs *after* the issue #324 context
  recovery and *before* the `UnsupportedWriteProgram` dead end is rendered. It
  uses the language the formalizer already extracted (falling back to alias
  matching), and returns `None` when no recipe matches so unmatched requests
  keep the honest unsupported answer.

#### Composition, not memoization — the `comments` axis

The maintainer's standing directive is to "reason from first principles … not
just fake solutions by memoization" and to "make the most general solution
possible". A blueprint that always emitted one frozen string for `http_json_stats`
would be exactly the memoized-answer table that `NON-GOALS.md` forbids ("A
memoized answer cache is not a substitute for reasoning from source data and
traceable steps"). To make the synthesis observably *compositional*, the
`comments` capability is now a **projection axis** over the curated program
rather than a label that is ignored:

- When the decomposition contains `comments`, the documented program is emitted
  verbatim.
- When it does **not**, `strip_comments` removes every whole-line comment and a
  leading Python module docstring, then `collapse_blank_runs` tidies the blank
  lines left behind. The result is a *different, smaller, still byte-for-byte
  compilable* program (only non-semantic lines are dropped; inline trailing
  comments are deliberately left untouched so the stripper can never cut a
  `//`/`#` that lives inside a string literal).

So the same recipe yields two genuinely different programs depending on the
decomposed sub-tasks — the emitted code is a function of the capabilities the
solver found, which is the smallest honest demonstration that the blueprint is
assembled from the decomposition rather than recalled from a table. The behavior
is unit-tested in `src/coding/blueprint.rs`
(`comments_requested_keeps_the_documented_program`,
`comments_omitted_strips_documentation_but_keeps_logic` across rust/python/js,
`stripped_program_is_smaller_than_documented`), mirrored byte-for-byte in the JS
worker, and the two variants per language are compile-checked offline by
`examples/issue_340_emit_variants.rs`.

### R2 — Honest execution contract · **done**

Blueprints are never claimed to have run. `render` always emits the localized
"not run — requires external libraries and network access" report, and
`try_program_blueprint` logs `execution_status: not run …` plus
`execution_environment: offline sandbox cannot install libraries or reach the
network`. A test asserts the answer **never** contains "compiled and ran".

### R4 — Case study · **done** (this document + `raw-data/`).

### R5 — Tracing · **already sufficient; capability trace added**
The root cause was found from the reported trace alone. The blueprint path
additionally records `program_blueprint:recipe <slug>`, one
`program_blueprint:capability <slug>` per decomposed capability, the
`program_parameter:language` / `program_parameter:task blueprint:<slug>`
parameters, and a `response:write_program:blueprint:<recipe>:<language>` evidence
link — so the diagnostic chips read like a catalog answer except the status is
explicitly "not run". No new always-on logging was required.

### R6 — Fix everywhere (both engines) · **done**
The JS worker (`src/web/formal_ai_worker.js`) mirrors the synthesizer
byte-for-byte: the `BLUEPRINT_CAPABILITIES`, the three curated program templates,
the `BLUEPRINT_RECIPES`, `detectBlueprintCapabilities`/`selectBlueprint`, and the
localized `renderBlueprint` all match the Rust source, and the worker's
`tryWriteProgram` calls the blueprint before returning the unsupported answer.
A `vm`-sandboxed parity experiment (`experiments/issue-340-worker-parity.mjs`)
asserts the worker matches the Rust core for English/Russian Rust, Python, and
JavaScript variants, that partial requests (no statistics) stay unsupported, and
that an unsupported language (Go) stays unsupported. `examples/repro_issue_340.rs`
runs the same cases through the Rust engine for direct comparison.

### R7 — Upstream reports · **none warranted**
The defect is entirely within this repository's engine and worker. The fix uses
only in-repo components (the existing language detector, the coding catalog
infrastructure, and the event log); no third-party bug was found, so no upstream
issue was filed.

### R8 — Single PR · **done** (PR #346).

---

## 5. Existing components / libraries reviewed (R4)

| Need | Existing component considered | Decision |
| --- | --- | --- |
| Capability/keyword matching across languages | In-repo `src/language.rs` detector + the coding catalog's normalization (`normalize_prompt`) | Reused. Capability keywords are matched with the same normalization and token-boundary rules the catalog already uses (CJK substring, Latin/Cyrillic stem-prefix), so no new NLP dependency was added — consistent with the no-neural-inference constraint. |
| Verified template catalog | In-repo coding catalog (`src/coding/`, issues #300/#312/#324/#330) | Kept intact. Blueprints are deliberately a **separate** path so the catalog's "compiled and ran" guarantee is never weakened by programs that cannot run offline. |
| Program synthesis from a plan | In-repo `src/program_plan.rs` substitution pipeline (issue #324) | Considered. `program_plan` lowers single-task *modifiers* (e.g. `list_files` → `list_files_arg`); it does not assemble a multi-capability program. The blueprint synthesizer complements it by handling the composite case with curated recipes. The two share the honest-execution philosophy. |
| HTTP/JSON/statistics in the generated code | `reqwest` + `serde_json` (Rust), `requests` + `statistics` (Python), Node 18+ built-in `fetch` (JavaScript) | Selected as the idiomatic, widely-used choices for each language and listed explicitly in the rendered "Required libraries" section so the user can reproduce locally. These are dependencies of the *generated* program, not of this repository. |
| Cross-engine parity harness | In-repo `vm`-sandbox pattern from issue #324 (`experiments/issue-324-js-worker.mjs`) | Reused the pattern for `experiments/issue-340-worker-parity.mjs`. |

---

## 6. Verification

- `cargo test` — **702 passed**; new integration tests in
  `tests/unit/specification/code_generation_blueprint.rs`
  (`rust_/python_/javascript_http_json_statistics_request_returns_blueprint_program`,
  `russian_http_json_statistics_request_returns_blueprint_in_russian`,
  `partial_composite_request_without_statistics_stays_unsupported`) plus the
  `src/coding/blueprint.rs` module unit tests (including the three `comments`
  composition tests).
- `cargo fmt --check` and `cargo clippy --all-targets -- -D warnings` — clean.
- File-size guard `scripts/check-file-size.rs` — all files within the 1000-line
  Rust hard limit. After merging `main`, the blueprint hook's dispatch table is
  provided by `src/solver_dispatch.rs` (main's superset extraction, which also
  carries issue #341's handler); the new integration tests live in
  `code_generation_blueprint.rs`.
- JS parity: `node experiments/issue-340-worker-parity.mjs` — all checks pass
  (English/Russian Rust, Python, JavaScript; the `comments` capability composes
  identically in both engines; partial stays unsupported; Go stays unsupported;
  `selectBlueprint`/`renderBlueprint` structural anchors).
- Compositional variants: `cargo run --example issue_340_emit_variants` writes
  documented + stripped programs per language to `target/issue-340-variants/`;
  the stripped Python passes `python3 -m py_compile` and the stripped JavaScript
  passes `node --check`.
- Cross-engine render parity: `examples/repro_issue_340.rs` and the parity
  experiment confirm the Rust core and JS worker produce byte-for-byte matching
  blueprints for all six localized cases.
