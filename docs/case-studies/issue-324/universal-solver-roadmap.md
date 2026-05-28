# Roadmap — Universal dynamic problem solving (Issue #324, R4)

> Issue #324 sketches a long-horizon vision: instead of memorizing specific
> solutions or even specific algorithms, the system should *reason → build a
> plan in links → translate that plan into Turing-complete substitution rules →
> compile those rules to Rust in WebAssembly → execute*, adapting dynamically to
> each request the way a human programmer does.
>
> This document records that vision as a staged roadmap and maps each stage onto
> components that already exist, so future work has a concrete starting point.
> **Nothing here changes runtime behavior in PR #325** — the shipped fix uses
> the deterministic template catalog (the honest baseline). This is the plan for
> growing beyond it.

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
than memorizing a new one. The roadmap generalizes that step.

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

### Stage 2 — Plan as Links Notation (foundation exists)
- **Have:** Links Notation is the project's native knowledge format; the engine
  already emits Links Notation traces (`program_parameter:task …`,
  `language:ru`, etc.).
- **Next:** represent the *solution plan itself* as links — a small graph of
  sub-goals ("open dir", "filter files", "sort", "print") that downstream stages
  can rewrite. The trace format is the natural carrier.

### Stage 3 — Plan → substitution rules (design needed)
- **Reference:** [`link-foundation/link-cli`](https://github.com/link-foundation/link-cli)
  `--always` triggers / substitution actions are the cited model for
  Turing-complete rewriting over links.
- **Next:** define a rule schema that maps plan-node patterns to code-fragment
  rewrites, and a fixpoint driver that applies `--always`-style triggers until
  the plan is fully lowered. This is the largest unbuilt piece.

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

## Suggested first increment (smallest useful step beyond this PR)

Generalize the path-argument modifier into a tiny **rule table**: a set of
`(modifier-pattern → task-variant)` rewrites expressed as links, applied by a
fixpoint loop. `list_files → list_files_arg` becomes the first entry; adding
"sort descending", "count instead of list", etc. becomes data, not new code.
This validates Stages 2–3 end-to-end on a constrained vocabulary before tackling
open-ended program synthesis.
