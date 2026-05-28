# Roadmap — Universal dynamic problem solving (Issue #324, R4)

> Issue #324 sketches a long-horizon vision: instead of memorizing specific
> solutions or even specific algorithms, the system should *reason → build a
> plan in links → translate that plan into Turing-complete substitution rules →
> compile those rules to Rust in WebAssembly → execute*, adapting dynamically to
> each request the way a human programmer does.
>
> This document records that vision as a staged roadmap and maps each stage onto
> components that already exist, so future work has a concrete starting point.
> **PR #325 implements the lowering stage** (Stages 2–3 below): the
> program-modification step now runs through a data-driven Links Notation
> substitution pipeline (`src/program_plan.rs` + `data/seed/program-plan-rules.lino`,
> mirrored in the JS worker). Stages 4–5 (compile arbitrary plans to fresh
> Rust/WASM and execute) remain future work; the deterministic template catalog
> stays the honest baseline for tasks no rule yet lowers.

## Where we are today (the baseline)

`formal-ai` is a **deterministic symbolic engine** — no neural inference. Today
a `write_program(language, task)` request binds to a hand-authored template in
the seed catalog (`data/seed/hello-world-programs.lino`, mirrored in
`src/engine_hello_world.rs` and `src/web/formal_ai_worker.js`). This is correct
and honest, but it only answers tasks someone has pre-authored a template for —
exactly the limitation the issue calls out ("we don't yet truly support writing
programs per requests and make changes to code by request").

The path-argument modifier added in this PR (`list_files` → `list_files_arg`) is
a first, deliberately small step toward *transforming* a known solution rather
than memorizing a new one — and it is now wired through the real Links Notation
substitution pipeline (Stages 2–3), not a hard-coded branch. The roadmap
generalizes that step.

## Target pipeline

```
user request
   │  (1) reason: classify intent, extract entities (language, task, modifiers)
   ▼
plan in Links Notation          ← a structured, inspectable plan, not prose
   │  (2) lower the plan to Turing-complete substitution rules
   ▼
substitution rules (link-cli --always triggers / rewrite actions)
   │  (3) compile rules to Rust
   ▼
Rust source  ──(4) build to WebAssembly──▶  WASM module
   │  (5) execute in the sandbox (or report honestly if execution unavailable)
   ▼
answer (localized — see Issue #324 R1)
```

## Stages and existing building blocks

### Stage 1 — Reason / formalize (partially exists)
- **Have:** `src/intent_formalization.rs` already extracts the `write_program`
  shape, language, task, and (new in this PR) follow-up modifiers via
  conversation-context recovery. `src/language.rs` tags the language.
- **Next:** generalize entity extraction beyond the fixed task vocabulary so
  novel tasks ("read a CSV and print column 2") produce a *plan* rather than an
  `unsupported` rule.

### Stage 2 — Plan as Links Notation (implemented for program modification)
- **Have:** Links Notation is the project's native knowledge format; the engine
  already emits Links Notation traces (`program_parameter:task …`,
  `language:ru`, etc.). **PR #325** now also represents the *solution plan
  itself* as links: `src/program_plan.rs` builds a `request:task -> <task>` /
  `request:modifier -> <modifier>` graph and emits an inspectable
  `program_plan` notation (with `resolved_task` and the applied trace).
- **Next:** grow the plan beyond a single task/modifier node into a multi-node
  sub-goal graph ("open dir", "filter files", "sort", "print") that downstream
  stages can rewrite. The current graph is the seed of that carrier.

### Stage 3 — Plan → substitution rules (implemented for program modification)
- **Reference:** [`link-foundation/link-cli`](https://github.com/link-foundation/link-cli)
  `--always` triggers / substitution actions are the cited model for
  Turing-complete rewriting over links.
- **Have:** **PR #325** lowers the plan with the project's substitution engine
  (`src/substitution.rs`, issue #301). The rule schema lives as data in
  `data/seed/program-plan-rules.lino` (`path_argument_list_files`: rewrite
  `request:task -> list_files` to `request:task -> list_files_arg` when
  `request:modifier -> path_argument` is present), and a fixpoint driver applies
  it until the plan stops changing. The JS worker carries a faithful mirror of
  the same engine and reads the same canonical `.lino`, so the two cannot drift.
  Adding a new `(modifier → task-variant)` rewrite is pure rule data — verified
  by the `pipeline_is_data_driven` test (Rust) and its JS-worker counterpart.
- **Next:** generalize rules from task-slug rewrites to code-fragment rewrites
  so a lowered plan emits source, not just a resolved task slug.

### Stage 4 — Rules → Rust → WASM (infra exists)
- **Have:** the engine is already a `no_std` Rust core compiled to WebAssembly
  for the GitHub Pages demo (see the WASM worker). The toolchain to produce and
  load WASM in the browser is in place.
- **Next:** emit Rust source from the lowered rules and compile it. In-browser
  this likely means a precompiled "interpreter of the rule IR" rather than
  invoking `rustc` client-side; server/CLI contexts could shell out to a real
  toolchain.

### Stage 5 — Execute / honest fallback (pattern exists)
- **Have:** the current execution report already distinguishes "ran in sandbox"
  from "not run — the sandbox cannot invoke a toolchain", and now says so in the
  user's language. That honesty contract carries forward.
- **Next:** when the rule IR *can* be interpreted safely in-sandbox (pure,
  filesystem-free computations), run it and show real output; otherwise keep the
  honest "not run" note.

## Guiding constraints (do not regress)

1. **Determinism / no neural inference** — every stage must be reproducible and
   inspectable. The plan and rules are data, not a black box.
2. **Honesty** — never claim code ran when the sandbox could not run it.
3. **Parity** — any capability must land in *both* the Rust core and the JS
   worker (Issue #324 R7).
4. **Localization** — answers follow the resolved response language (R1/R2).

## First increment — shipped in PR #325

The smallest useful step beyond a hard-coded branch was a tiny **rule table**: a
set of `(modifier-pattern → task-variant)` rewrites expressed as links, applied
by a fixpoint loop. **This is now implemented.** `list_files → list_files_arg`
is the first entry (`data/seed/program-plan-rules.lino`), and the tests prove
that adding "sort descending", "count instead of list", etc. is pure data, not
new code (the `count_only → count_files` case in both the Rust and JS-worker
data-driven tests). This validates Stages 2–3 end-to-end on a constrained
vocabulary; the next increment tackles open-ended program synthesis (Stages
4–5).
