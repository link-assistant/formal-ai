# Case study — Issue #334: a 2-step Fibonacci agent plan where both steps failed

> The GitHub Pages WASM-worker demo decomposed a single prompt into a correct
> two-step agent plan, then **failed both steps**: step 1 ("Write a Python
> function that calculates the Fibonacci sequence recursively") answered "I
> didn't understand you", and step 2 ("calculate the 10th Fibonacci number and
> multiply it by 8% of 500. Show me the code and the final result") answered
> "unparseable". This study reconstructs the context, enumerates every
> requirement, traces each failure to its root cause across **all four** code
> mirrors, records the implemented solution, surveys related libraries, and
> lists the verification.

- **Issue:** [#334](https://github.com/link-assistant/formal-ai/issues/334) — *Issue with dialog: Write a Python function that calculates the Fibonacci sequence recursively. …*
- **Reported version:** 0.149.0 · WASM worker · manual mode · UI language `ru` (UI `ru-RU`, `Asia/Yekaterinburg`) · diagnostics on
- **Pull request:** [#344](https://github.com/link-assistant/formal-ai/pull/344) (branch `issue-334-229eab37b96b`)
- **Raw data:** [`raw-data/`](./raw-data/) — `issue.json`, `issue-comments.json` (1 comment, quoted in §2), `pr-344.json`.

---

## 1. Timeline / sequence of events

| When (UTC) | Event |
| --- | --- |
| 2026-05-29 18:39:09 | In the GitHub Pages WASM worker (v0.149.0, manual mode) the user pastes a single instruction containing two tasks: "Write a Python function that calculates the Fibonacci sequence recursively. Then calculate the 10th Fibonacci number and multiply it by 8% of 500. Show me the code and the final result." |
| 2026-05-29 18:39 | The agent correctly classifies the intent as `agent_plan` and decomposes the prompt into **two steps** (split on the "Then …" separator). Both steps then fail: step 1 → "I didn't understand you", step 2 → "unparseable". |
| 2026-05-29 18:39:46 | Issue #334 is filed (label `bug`) with the full diagnostic envelope and the reproduction dialog. The **Description** field is left blank. |
| 2026-05-29 18:4x | The maintainer (`konard`) adds the standard generalization comment (quoted in §2): increase coding/problem-solving generality, archive all data under `docs/case-studies/issue-{id}`, write a deep case study with online research and a library survey, add debug/verbose output if root-cause data is insufficient, report any upstream-repo bugs with reproductions, apply fixes across the **entire** codebase, and do it all in one PR. |
| 2026-05-29 (this PR) | Fibonacci coding task, word-problem normalizer, shared `% of` arithmetic support (Rust + WASM + JS), unit-misroute fix, bare-dot OOM guard, tests, example, e2e, and this case study added in PR #344. |

The agent-plan **decomposition itself was already correct** — the failure was
entirely in the two per-step handlers.

---

## 2. Requirements (every explicit and implicit ask)

The issue **Description** is blank, so the requirements come from (a) the
reproduction dialog and (b) the maintainer comment (verbatim):

> We should continue to increase generalization of our problem solving and
> coding skills … We need to download all logs and data … to
> `./docs/case-studies/issue-{id}` … do deep case study analysis (also … search
> online …), reconstruct timeline …, list … each and all requirements …, find
> root causes …, and propose … solution plans … (also … check known existing
> components/libraries …). If there is not enough data to find actual root
> cause, add debug output and verbose mode … If issue related to any other
> repository/project … please [report it] … with reproducible examples,
> workarounds and suggestions for fix … double check to fully apply requirements
> to entire codebase … in all [places]. Please plan and execute everything in
> this single pull request …

### Functional requirements (from the dialog)
1. **R1 — Step 1 must generate a recursive Python Fibonacci program.** "Write a
   Python function that calculates the Fibonacci sequence recursively" must
   return a verified program, not "I didn't understand you".
2. **R2 — Step 2 must evaluate the word problem.** "calculate the 10th Fibonacci
   number and multiply it by 8% of 500" must reduce to `55 * 8% of 500` and
   evaluate to **2200**, not "unparseable".
3. **R3 — The full agent plan must work end-to-end** in the production WASM
   worker: decompose into the two steps and resolve each.

### Process requirements (from the maintainer comment)
4. **R4 — Increase generalization** of coding / problem-solving, not just a
   memorized special case.
5. **R5 — Archive all logs/data** under `docs/case-studies/issue-334/`.
6. **R6 — Deep case study** (this document): timeline, every requirement, root
   causes, solution plans, library survey, online research.
7. **R7 — Add debug/verbose output** if root-cause data is insufficient.
8. **R8 — Report upstream bugs** in related repos with reproductions, workarounds
   and fix suggestions.
9. **R9 — Apply the fix across the entire codebase** (every mirror).
10. **R10 — One pull request** (#344).

---

## 3. Root-cause analysis

The deterministic engine resolves a coding prompt to a `(task, language,
template)` triple by alias matching, and resolves an arithmetic prompt through a
calculation pipeline. Both failed for distinct reasons.

### Cause A — `fibonacci` was not a catalog task (causes R1)
The coding catalog had no `fibonacci` task and no aliases for "Fibonacci
sequence" / "recursive Fibonacci", so step 1 matched no coding intent and fell
through to the unknown-answer opener ("I didn't understand you"). The catalog is
mirrored in **four** places that must all agree, so adding the task in only one
would have left the WASM demo (the exact thing #334 reports) still broken:

1. the Rust catalog under `src/coding/catalog/`,
2. the portable `data/seed/hello-world-programs.lino` seed (parity-tested),
3. the JS worker fallback in `src/web/formal_ai_worker.js`,
4. the standalone `#![no_std]` `src/web/wasm-worker/src/lib.rs` crate that is
   compiled to the committed `src/web/formal_ai_worker.wasm` artifact.

### Cause B — the word problem never reduced to a calculator expression (causes R2)
"calculate the 10th Fibonacci number and multiply it by 8% of 500. Show me the
code and the final result" is natural language, not arithmetic. Nothing resolved
the symbolic Fibonacci reference (`the 10th Fibonacci number` → 55), rewrote the
spelled-out operator (`and multiply it by` → `*`), or dropped the trailing
instruction sentence (`Show me the code …`). The calculator received the whole
sentence and reported "unparseable".

### Cause C — even the reduced expression was unparseable in the WASM worker (causes R2, R3)
This is the subtle one, discovered via the end-to-end test. Once the normalizer
reduced step 2 to `55 * 8% of 500`, the **native CLI** evaluated it to 2200 (its
pipeline calls `link-calculator` first). But the production demo does **not**
use `link-calculator`: `engine_evaluate_arithmetic` in the WASM worker delegates
to `evaluate_fallback_formatted` in the shared `no_std` `src/arithmetic.rs`,
which treated `%` only as the **modulo** operator and choked on the word `of`.
So the real WASM artifact still returned "unparseable" for `55 * 8% of 500`. The
JS fallback `evaluatePercentOfExpression` only matched an anchored `^X% of Y$`
and could not handle `55 * 8% of 500` either. **CI never rebuilds the WASM** (it
is a committed binary; `bun run build:web` only bundles JS), so the fix had to
include regenerating `formal_ai_worker.wasm`.

### Cause D — coding prompts misrouted as unit-incompatibility (latent, surfaced while fixing)
A separate handler flagged "unit incompatibility" by **substring** match, so
"nu**mb**er" matched the `mb` unit and "pro**gram**" matched `gram`, stealing
prompts like "the 10th Fibonacci **number**" / "Write a **program** …".

### Cause E — a bare-dot expression crashed `link-calculator` with an OOM (latent)
While hardening the calculation candidates, an expression such as `2. 3` (a bare
dot from sentence splitting) drove the upstream `link-calculator` rational parser
into an unbounded `decimal_places` allocation → out-of-memory abort of the whole
process (see §5 / R8).

---

## 4. Solution plans (per requirement) and what was implemented

### R1 — recursive Python Fibonacci program · **done**
Added a `fibonacci` coding task with aliases ("fibonacci", "fibonacci sequence",
"recursive fibonacci", "nth fibonacci", …) and a verified recursive template
across **all four** mirrors (`src/coding/catalog/{tasks,templates_extended,
guidance}.rs`, `data/seed/hello-world-programs.lino`, `src/web/formal_ai_worker.js`,
`src/web/wasm-worker/src/lib.rs`). The program prints F(10) = 55 and ships a
"How it works" / "How to test" explanation, matching the R9 convention from #330.

### R2 / R3 — step-2 word problem and the WASM evaluator · **done**
Two complementary changes:

1. **Word-problem normalizer** (`src/calculation_word_problem.rs`,
   `normalize_word_problem`): resolves "(the) N-th Fibonacci number" → its value
   (F(10) = 55, convention F(1)=F(2)=1), rewrites spelled-out operators
   ("and multiply it by" → `*`, "divided by" → `/`, longest phrase first), and
   drops trailing instruction sentences while keeping decimals (`3.14`) intact.
   `calculate the 10th Fibonacci number and multiply it by 8% of 500. Show me …`
   → `calculate 55 * 8% of 500`. Mirrored in the JS worker.
2. **`% of` support in the shared `no_std` evaluator** (`src/arithmetic.rs`,
   `rewrite_percent_of`): rewrites `N% of M` → `( N * M / 100 )` before
   tokenizing, so `8% of 500` = 40 and `55 * 8% of 500` = **2200**. A bare `%`
   not followed by `of` still parses as modulo. This module is `#[path]`-mounted
   into the WASM crate, so rebuilding `formal_ai_worker.wasm` makes the
   production demo evaluate step 2. Mirrored in the JS fallback
   (`normalizeArithmeticWords` → `rewritePercentOf`) so all three evaluation
   backends agree.

### R4 — generalization · **done**
The normalizer resolves *any* ordinal/cardinal Fibonacci reference ("fifth",
"10th", "tenth"), the operator rewrites are phrase-driven (multiply/divide), and
`% of` works for arbitrary `N`/`M` — not a memorized answer to this one prompt.

### R5 / R6 — data + case study · **done** (this document + `raw-data/`).

### R7 — debug/verbose output · **N/A — root cause fully isolated.** The
diagnostics envelope in the issue plus the e2e trace ("agent_2_dispatch_handler:
tryArithmetic → calculation_error: unparseable") pinpointed the WASM evaluator
without new tracing.

### R8 — upstream report · **prepared** (see §5): the bare-dot OOM in
`link-assistant/calculator` with a reproducer, workaround (the bare-dot guard in
`src/calculation.rs`), and a suggested fix.

### R9 — entire codebase · **done**: all four catalog mirrors, all three
arithmetic backends (CLI fallback, WASM, JS), plus the rebuilt WASM binary.

### R10 — single PR · **done** (PR #344).

---

## 5. Existing components / libraries reviewed (R7/R8)

| Component | Role here | Verdict |
| --- | --- | --- |
| [`link-calculator`](https://github.com/link-assistant/calculator) | The native CLI's primary arithmetic backend; already evaluates `55 * 8% of 500` = 2200. | Keep on the native path. **Not** available in the `no_std` WASM worker (no Cargo deps), so the shared evaluator had to learn `% of` itself. **Bug found & to be reported:** the input `2. 3` triggers an unbounded `decimal_places` allocation → OOM. Reproducer: feed `"2. 3"` to the rational parser. Workaround shipped here: the bare-dot guard in `src/calculation.rs` keeps such candidates away from `link-calculator`. Suggested fix: validate/cap `decimal_places` in `src/types/rational.rs`. |
| Hand-written recursive-descent evaluator (`src/arithmetic.rs`) | The `no_std` fallback shared by CLI, WASM and JS. | Extended with `rewrite_percent_of`; arbitrary-precision integers keep `N * M` exact before the `/ 100`. |
| Word-problem / NLP-to-math libraries (e.g. `nltk`, `quantulum3`, `sympy.parsing`) | Could parse free-text math. | Rejected: heavyweight, non-deterministic, and impossible in a `no_std`/WASM target. A small deterministic normalizer matches the project's symbolic vision. |
| Syntax-highlight / code-block presentation (issue #330) | Renders the generated program. | Reused as-is; this issue only needed the program to be *generated*. |

**Online research notes.** The Fibonacci convention is ambiguous in the wild
(F(0)=0 vs F(1)=1 indexing). We standardized on **F(1)=F(2)=1** so the generated
program's printed F(10) = 55 matches the symbolic reference the normalizer
substitutes — keeping the two halves of the user's compound prompt internally
consistent. Percentage-of phrasing ("X% of Y") is conventionally `Y * X / 100`,
which is what `rewrite_percent_of` emits.

---

## 6. Files changed

| File | Change |
| --- | --- |
| `src/coding/catalog/tasks.rs`, `templates_extended.rs`, `guidance.rs` | `fibonacci` task, recursive Python template, explanation. |
| `src/intent_formalization.rs` | Route Fibonacci coding prompts to the new task. |
| `data/seed/hello-world-programs.lino` | Portable seed mirror of the task (parity-tested). |
| `src/web/formal_ai_worker.js` | JS mirror of the task, the word-problem normalizer, and `rewritePercentOf`. |
| `src/web/wasm-worker/src/lib.rs` + `src/web/formal_ai_worker.wasm` | WASM mirror of the task; rebuilt binary with `% of` support. |
| `src/calculation_word_problem.rs` (new) | `normalize_word_problem` + helpers. |
| `src/calculation.rs`, `src/lib.rs` | Wire in the normalizer; bare-dot OOM guard. |
| `src/arithmetic.rs`, `src/web_engine_core.rs` | `rewrite_percent_of` + tests. |
| `src/solver_handler_units.rs` | Word-boundary unit matching (Cause D). |
| `examples/repro_issue_334.rs` | Runnable reproduction of all four cases. |
| `tests/e2e/tests/issue-334.spec.js`, `playwright.local.config.js` | End-to-end coverage in real "wasm worker" mode. |
| `tests/unit/specification/{calculator_delegation,code_generation}.rs` | Unit regressions. |
| `changelog.d/20260529_120000_*.md` | Changelog fragment (`bump: minor`). |

---

## 7. Verification

- `cargo test` — 285 lib + 690 integration + 13 `web_engine_core` tests pass.
- `cargo fmt --all --check`, `cargo clippy --all-targets --all-features` — clean
  (MSRV 1.70: `map_or(true, …)` instead of `is_none_or`).
- `rust-script scripts/check-file-size.rs` — all files within limits (the
  normalizer was extracted into its own module to keep `calculation.rs` < 1000).
- `sh src/web/wasm-worker/build.sh` — rebuilds `formal_ai_worker.wasm`
  (101,149 → 102,388 bytes) with `% of` support.
- `node experiments/issue-334/check_js_worker.mjs` — JS fallback (WASM stubbed)
  reduces step 2 to `55 * 8% of 500 = 2200`.
- `npx playwright test --config=playwright.local.config.js issue-334` — **all 3**
  e2e tests pass against the freshly built bundle in "wasm worker" mode:
  step 1 (Python program, F(10)=55), step 2 (2200), and the full two-step agent
  plan (no "I didn't understand you" / "unparseable").
