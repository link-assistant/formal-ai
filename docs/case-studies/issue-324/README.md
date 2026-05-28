# Case study — Issue #324: "Сделай так, чтобы программа принимала путь как аргумент"

> A two-defect dialog bug plus a setting request and a long-horizon vision.
> The agent answered a Russian prompt in English, then lost all context on a
> follow-up modification. This study reconstructs the timeline, enumerates
> every requirement, isolates the root causes, and records the solution plans —
> including a roadmap for the universal dynamic problem-solving vision the issue
> sketches.

- **Issue:** [#324](https://github.com/link-assistant/formal-ai/issues/324) — *Issue with dialog: Сделай так, чтобы программа принимала путь как аргумент*
- **Reported version:** 0.146.0 · WASM worker · manual mode · UI language `en-US` · locale `en-US` (`Asia/Calcutta`)
- **Pull request:** [#325](https://github.com/link-assistant/formal-ai/pull/325) (branch `issue-324-cf8043055ef7`)
- **Predecessor:** [#312](https://github.com/link-assistant/formal-ai/issues/312) / PR [#318](https://github.com/link-assistant/formal-ai/pull/318) — added the `write_program` `list_files` task and multilingual detection. This issue is the direct sequel: the same prompt now answers, but in the wrong language, and the follow-up turn regresses.
- **Raw data:** [`raw-data/`](./raw-data/) — `issue.json`, `issue-comments.json` (`[]`), `pr-325.json`, `reproduction-dialog.md`.

---

## 1. Timeline / sequence of events

| When (UTC) | Event |
| --- | --- |
| 2026-05-28 18:47:53 | In the GitHub Pages WASM worker the user submits the Russian prompt **"Напиши мне программу на Rust, которая выдаёт список файлов в текущей директории"** ("Write me a Rust program that lists the files in the current directory"). |
| 2026-05-28 18:47:53 | Thanks to PR #318 the request now resolves to `write_program(rust, list_files)` and returns a working program — **but the surrounding prose ("Here is a minimal Rust … program:", "Execution status: not run …") is entirely English** even though the prompt is Russian. **(Bug 1)** |
| 2026-05-28 18:47:53 | The user follows up with **"Сделай так, чтобы программа принимала путь как аргумент"** ("Make the program accept a path as an argument"). The agent routes it to `write_program` but binds neither task nor language, replying *"I do not have a template for language `missing` and task `missing`"*. **(Bug 2)** |
| 2026-05-28 18:54:25 | Issue #324 is filed (no comments) quoting the dialog and adding three further asks: a response-language **setting**, a deep **case study**, and a **vision** for universal dynamic problem solving. |
| 2026-05-28 (this PR) | Both bugs fixed in **both** source-of-truth engines (Rust core + JS worker); response-language setting added to the React app, i18n catalog, and worker; the program-modification step lowered onto the data-driven Links Notation substitution pipeline (first stage of the R4 vision); case study written. |

The issue carries **no comments** (`issue-comments.json` is `[]`); every
requirement comes from the issue body.

---

## 2. Requirements (every explicit and implicit ask)

### From the failing behavior
1. **R1 — Respond in the detected language.** A Russian message must get a
   Russian answer. The issue notes the heuristic explicitly: *"> 51% of message
   is in Russian letters/words"*. Detection already worked (PR #318); the
   **answer rendering** did not honor it.
2. **R2 — Add a response-language setting.** A preference choosing which
   language drives responses: **`last message language`** (default),
   **`preferred selected language`**, or **`UI language`**.
3. **R3 — Support program modification by request.** The follow-up
   "make it accept a path as an argument" must reuse the conversation context
   (Rust + list-files) and apply the modification, like a human programmer would
   — not collapse to `missing`/`missing`.

### From the meta-instructions appended to the issue
4. **R4 — Universal dynamic problem-solving vision.** Express the
   program-writing algorithm as **both** Rust code and links substitution rules
   (à la `link-cli` `--always` triggers); the ideal pipeline is *reason → plan
   in links → Turing-complete substitution rules → compile to Rust/WASM →
   execute*, so the system adapts dynamically instead of memorizing solutions.
   Captured as a roadmap (see [`universal-solver-roadmap.md`](./universal-solver-roadmap.md)).
5. **R5 — Case study.** Download all related data to
   `docs/case-studies/issue-324/` and analyze deeply: timeline, requirements,
   root causes, solution plans, and **online research** of existing components.
6. **R6 — Add tracing if data is insufficient.** If a root cause cannot be
   found, add debug output / a verbose mode (off by default).
7. **R7 — Fix everywhere.** Apply each fix across the *entire* codebase — both
   parallel implementations.
8. **R8 — Report upstream** to other repositories if the issue belongs there.
9. **R9 — Single PR.** Plan and execute everything in PR #325.

---

## 3. Root-cause analysis

### Root cause A — `write_program` answers were hard-coded English (causes R1)

Detection is **not** the problem. `src/language.rs::detect()` counts characters
by Unicode block and returns the dominant script; the Russian prompt (mostly
Cyrillic, one Latin word "Rust") correctly detects as `ru`. The defect is in
**rendering**: the `write_program` answer builder emitted fixed English strings
regardless of the detected language.

At the reported version the intro and execution report were literal English:

```
Here is a minimal Rust … program:
Execution status: not run - the browser sandbox cannot invoke a rust toolchain. …
```

The Links Notation trace already carried `language:ru`, but neither
`src/engine.rs` (the Rust core, compiled to WASM for the Pages demo) nor
`src/web/formal_ai_worker.js` (the JS fallback) consulted it when composing the
prose. So a correctly-detected language never reached the user-visible answer.

### Root cause B — follow-up modifications had no context recovery (causes R3)

The intent formalizer recognizes "write a program" shapes via program nouns
(`программу`/`program`/`程序`/…) and program verbs (`напиши`/`сделай`/`制作`/…).
The follow-up "Сделай так, чтобы программа принимала путь как аргумент" matches
the *shape* (`сделай` + `программа`) but names **no language and no task**, so
`write_program_parameters` returns `task = None, language = None`. With nothing
to bind, the engine produced the literal placeholder answer "language `missing`
and task `missing`".

The missing capability was **conversational context recovery**: a follow-up
that modifies "the program" must look back at the most recent program-writing
turn, reuse its `(language, task)`, and apply the requested modification. No
such recovery existed — single-turn formalization was the only path.

### Root cause C — no path-argument task variant existed (causes R3)

Even with context recovery, "accept a path as an argument" needed a *target*.
The catalog only had `list_files` (lists the current directory). There was no
task representing "list files at a path supplied on argv", so the modifier had
nothing to upgrade to.

### Root cause D — no response-language preference existed (causes R2)

The app had a `uiLanguage` preference but nothing to decouple *response*
language from *UI* language or from the *last message*. There was no setting, no
persisted value, and no resolution logic mapping a mode to a concrete language.

---

## 4. Solution plans (per requirement) and what was implemented

### R1 — Localize `write_program` answers · **done**

- **Rust** (`src/engine.rs`, commit `5fe98e0`): the intro
  (`write_program_intro`), the unsupported message, and the execution report
  (status phrase + output label + "no output"/sandbox notes) now render per
  detected language for `ru`/`hi`/`zh`; English unchanged. The Links Notation
  trace stays language-independent.
- **JS worker** (`src/web/formal_ai_worker.js`, commit `6a7ab89`): a
  `WRITE_PROGRAM_I18N` dictionary (en/ru/hi/zh) + `writeProgramStrings()` feed
  the same localized prose into `writeProgramExecutionLines()` and
  `tryWriteProgram()`, keeping the Pages demo in parity (**R7**).

### R2 — Response-language setting · **done**

- **React app** (`src/web/app.js`, commit `a69f512`): new
  `responseLanguage` (`last_message` default / `preferred` / `ui`) and
  `preferredLanguage` preferences — state, refs, persistence, reducer cases, and
  two `<select>` controls (the *Preferred language* select appears only when the
  mode is `preferred`).
- **i18n catalog** (`src/web/i18n-catalog.lino`): `settings.responseLanguage`
  (+ `lastMessage`/`preferred`/`ui`) and `settings.preferredLanguage` translated
  for all four locales; the catalog allow-list check
  (`tests/e2e/scripts/check-i18n-catalog.mjs`) updated accordingly.
- **Worker** (`src/web/formal_ai_worker.js`): `responseLanguageFor(detected,
  preferences, userContext)` resolves the mode to a concrete language —
  `preferred` → the chosen language; `ui` → the UI language, falling back to the
  browser languages (array *or* comma-joined string) and finally the detected
  language. `solve()` computes `responseLanguage` and routes `tryWriteProgram`
  through it.

### R3 — Program modification by context · **done**

- **Context recovery** (`src/intent_formalization.rs` `recover_write_program_rule`
  + `src/solver.rs`; JS `recoverWriteProgramParameters`): when a follow-up routes
  to `write_program` with missing task/language, rebuild them from the most
  recent prior turn.
- **Path-argument modifier** (`detected_program_modifiers`; JS equivalent): a
  follow-up containing path-argument tokens (ru `путь`/`аргумент`, hi
  `पथ`/`तर्क`, zh `路径`/`参数`, en `path`/`argument`) is detected as the
  `path_argument` modifier slug. The slug → task-variant upgrade
  (`list_files` → **`list_files_arg`**) is **not** hard-coded: it runs through
  the data-driven substitution pipeline described in R4 below.
- **New `list_files_arg` task** with templates for all ten catalog languages
  (`src/engine_hello_world.rs`); also resolvable as an explicit single-turn
  request.

### R4 — Universal dynamic problem-solving vision · **first stage implemented**

The roadmap ([`universal-solver-roadmap.md`](./universal-solver-roadmap.md))
stages the *reason → plan in links → lower to substitution rules → compile to
Rust/WASM → execute* pipeline. **The lowering stage is now live runtime code**,
not just a plan:

- **Substitution rules as data** (`data/seed/program-plan-rules.lino`): the
  program-modification step is expressed as a [`substitution_rules`] document in
  Links Notation — `when "request:modifier -> path_argument"` then `replace
  "request:task -> list_files"` `with "request:task -> list_files_arg"`. This is
  the "links substitution rules" deliverable the issue asks for (à la `link-cli`
  `--always` triggers), and it is also bundled into `seed::seed_files()` so it
  ships in the downloadable knowledge bundle.
- **The pipeline runs the rules** (`src/program_plan.rs`): `lower(base_task,
  modifiers)` seeds a `SubstitutionGraph` with `request:task -> <base>` plus one
  `request:modifier -> <slug>` link per detected modifier, then applies the
  rule set to a fixpoint via the **same** [`crate::substitution`] engine that
  powers the text-manipulation chain. Adding a new modification (e.g. "count
  instead of list") is now *data* — a new rule in the `.lino` file — not new
  control flow. The `pipeline_is_data_driven` test proves this: a brand-new
  rule changes behavior with zero code change.
- **The plan is inspectable** (R6): `ProgramPlan::links_notation()` renders the
  plan graph and its full rewrite trace as Links Notation, and the solver logs
  it as a `write_program_plan` event surfaced in `evidence_links`.
- **Parity** (R7): the JS worker mirrors the engine and the pipeline
  (`src/web/formal_ai_worker.js`): a self-contained substitution parser/applier
  reads the byte-identical rule text and resolves the task the same way, and
  surfaces the same `write_program_plan` evidence link. The parity experiment
  asserts both implementations agree, including a data-driven custom-rule case.

The remaining stages (compile arbitrary plans to fresh Rust/WASM and execute)
stay on the roadmap; the template catalog remains the deterministic, honest
baseline for tasks no rule yet lowers.

### R5 — Case study · **done** (this document + `raw-data/`).

### R6 — Tracing · **already present, extended**
The solver records a `language:<slug>` event per impulse and a response-language
event/step when the response language differs from the detected one (worker
`solve()`); diagnostics mode in the app surfaces the step trace. No new
always-on logging was required to find the root causes above.

### R7 — Fix everywhere · **done**
Every behavioral change landed in **both** the Rust core and the JS worker —
including the new data-driven substitution pipeline (Rust `src/program_plan.rs`
+ `src/substitution.rs`; a self-contained JS mirror in
`src/web/formal_ai_worker.js`). A `vm`-sandboxed parity experiment
(`experiments/issue-324-js-worker.mjs`) asserts the worker matches the Rust
tests for ru/hi/zh follow-ups, that both surface the `write_program_plan`
evidence link, and that the worker's rule parser reads the canonical
`data/seed/program-plan-rules.lino` to the same ruleset its embedded copy uses.

### R8 — Upstream reports · **none warranted**
The defects are entirely within this repository's engine and worker. Detection
(`src/language.rs`) and the i18n runtime (`lino-i18n`) behaved correctly; no
third-party bug was found, so no upstream issue was filed.

### R9 — Single PR · **done** (PR #325).

---

## 5. Existing components / libraries reviewed (R5)

| Need | Existing component considered | Decision |
| --- | --- | --- |
| Language detection | In-repo `src/language.rs` (Unicode-block heuristic) | Reused as-is; it already detects ru/hi/zh/en correctly. No external lib (e.g. `whatlang`) needed — the deterministic, dependency-free heuristic matches the project's no-neural-inference constraint. |
| Response prose localization | In-repo `WRITE_PROGRAM_I18N` (worker) + per-language match arms (engine); `lino-i18n` for UI strings | Reused both. UI labels go through `lino-i18n`; engine/worker answer prose uses the in-repo dictionaries so the WASM core stays `no_std`. |
| Conversation-context recovery | None existed | Implemented in-repo; mirrors how the predecessor PR #318 structured `write_program` parameters. |
| Substitution-rule execution (vision) | In-repo `src/substitution.rs` (issue #301 `replace x y` engine); `link-foundation/link-cli` `--always` triggers / substitution actions; Links Notation | **Wired into the solver** for the program-modification step (`src/program_plan.rs` + `data/seed/program-plan-rules.lino`), mirrored in the JS worker. The in-repo engine already implemented the `--always`-style `replace`/`with` primitive; this PR reuses it rather than adding a dependency. |

---

## 6. Verification

- `cargo test --test unit` — **670 passed**; `cargo test --test integration` — **13 passed**; lib tests — **268 passed**.
- New tests: `russian_/hindi_/chinese_follow_up_path_argument_modification_reuses_context`, `explicit_list_files_with_path_argument_is_supported`, and `english_follow_up_modification_emits_substitution_plan_trace` (`tests/unit/specification/code_generation.rs`); the `src/program_plan.rs` module's 7 unit tests (rule parsing, `path_argument` upgrade, idempotence, unknown-task pass-through, data-driven custom rule, links-notation trace).
- `cargo fmt --all` and `cargo clippy --all-targets` (`-Dwarnings`) — clean.
- JS parity: `node experiments/issue-324-js-worker.mjs` — all checks pass (ru/hi/zh follow-ups, `responseLanguageFor` modes, the substitution pipeline, the `write_program_plan` evidence link, and rule-text/seed agreement).
- e2e guards: `check-i18n-catalog`, `check-language-test-coverage`,
  `check-language-change-parity`, `check-multilingual-intent-coverage` — all pass.
