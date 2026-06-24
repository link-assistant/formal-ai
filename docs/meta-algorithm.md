# Meta-Algorithm: Reproducing Topic Handlers On Demand

We do not only produce code changes for a topic — we **learn from our own source
code how to produce them**. This page describes the meta-algorithm: a fixed,
ordered procedure that reproduces a deterministic chat intent handler for any
language topic, together with a machine-readable recipe that stays grounded in
the live source.

The first fully-encoded instance is the **procedural how-to** topic (issue #444):
a "how to X" request and its elaboration follow-up ("can you give me specific
instructions?"). Its recipe lives at
[`data/meta/procedural-howto-recipe.lino`](../data/meta/procedural-howto-recipe.lino).

## Why a recipe, not just code

The recipe names every part the handler is made of — seed roles, handler
functions, evidence stages, JS parity targets, the external-service toggle, and
the benchmark — plus the eight ordered steps that generalise to any topic. A
test suite, [`tests/unit/specification/meta_algorithm.rs`](../tests/unit/specification/meta_algorithm.rs),
loads the recipe and asserts the **real source still matches** every entry. If
the recipe and the code drift apart, CI fails. So the recipe is always an
accurate, executable description of how the code was produced — never stale
documentation. To regenerate the handler from scratch, follow the steps; to
verify a regeneration, run the suite.

## The eight steps

Each step is one `meta_step` record in the recipe. To add a new topic,
instantiate every step in order:

1. **Declare topic meanings in the seed lexicon.** One meaning per linguistic
   role in `data/seed/meanings-*.lino`, listing surfaces across every supported
   language as slot-marked forms (prefix `how to …`, suffix `… по шагам`,
   circumfix `how … works`, or bare). Surfaces are data, not code.
2. **Expose stable `ROLE_*` constants.** Each meaning role gets a constant in
   `src/seed/roles/intent.rs`; its string value equals the seed `role`.
3. **Recognise by meaning, not by phrase table.** Recognisers query
   `seed::lexicon().meanings_with_role(..)` / `role_word_forms(..)`. No
   per-language phrase list ever lives in Rust (the issue #386 convention).
4. **Split prompts on slot-marked surfaces.** Extractors derive their matching
   strategy from each surface's slot and the position of the `…` marker,
   preserving declaration order so longer surfaces win first. This yields
   task/action/object decomposition with no language branches.
5. **Emit a deterministic discovery plan.** Append the same ordered evidence
   stages the browser worker executes: local decomposition → Wikimedia/wikiHow
   candidates → web search with reciprocal rank fusion → a recursive fetch check
   gated to explicit steps. The offline engine records the plan; the worker runs
   it live.
6. **Register external sources with a settings toggle.** Each external trusted
   source gets a record in `data/seed/sources-registry.lino` carrying
   `service_group external_trusted`, a `settings_key`, and `default_enabled true`
   (opt-out model). The UI reads the toggle; the worker skips the live fetch when
   it is `false`.
7. **Mirror the handler in the JS worker.** Reproduce each Rust handler as a JS
   function in `src/web/formal_ai_worker.js` so the WASM/browser surface stays in
   parity (R15). Evidence ordering matches byte-for-byte on the default path.
8. **Pin behaviour with a ratcheted benchmark.** Add a permissive-license slice
   with upstream-derived and held-out cases under `data/benchmarks/`, plus a
   ratchet test asserting `passed >= minimum_pass_count`.

## What the recipe records for the procedural how-to topic

| Recipe record | Count | Grounded against |
| --- | --- | --- |
| `meta_step` | 8 | ordering 1..8 is contiguous |
| `meta_role` | 9 | `pub const ROLE_* = "<role>"` in `src/seed/roles/intent.rs` **and** `role <role>` in `data/seed/meanings-how.lino` |
| `meta_function` | 7 | `fn <name>` in `src/solver_handler_how.rs` |
| `meta_stage` | 5 | each stage literal emitted in the handler; ordering 1..5 contiguous |
| `meta_parity` | 3 | `fn <rust>` in Rust **and** `function <js>` in the worker |
| `meta_external_service` | 1 | `source` + `settings_key` in `data/seed/sources-registry.lino` |
| `meta_benchmark` | 1 | suite in fixture **and** ratchet test in `procedural_howto_benchmarks.rs` |

## Running it

```sh
# Verify the recipe still matches the live source (the grounding suite):
cargo test --test unit specification::meta_algorithm -- --nocapture
```

## Generalising to a new topic

1. Copy `data/meta/procedural-howto-recipe.lino` to
   `data/meta/<topic>-recipe.lino` and rewrite each record for the new topic.
2. Work the eight steps in order, filling in the roles, functions, stages,
   parity targets, services, and benchmark you declared.
3. Add a grounding test for the new recipe modelled on
   `tests/unit/specification/meta_algorithm.rs` (or parameterise it over every
   `data/meta/*-recipe.lino`).

Because the recipe is checked against the source, the handler and its recipe can
never silently diverge — which is exactly what lets us treat the source as a
reproducible artifact of the meta-algorithm.

## The agentic-coding meta-algorithm (issue #468)

The same grounded-recipe discipline records a second, different kind of
meta-algorithm: not a chat intent handler, but the **deterministic agentic loop**
that lets our Formal AI *solve a task in agentic coding mode*. The maintainer's
framing for issue #468 was that "our Formal AI system should have enough skills
(meta algorithm, rust code) to actually call all the tools from any agentic CLI,
understand errors from tools, … to actually complete the task." Its recipe lives
at [`data/meta/agentic-coding-recipe.lino`](../data/meta/agentic-coding-recipe.lino)
and is grounded by
[`tests/unit/specification/agentic_meta_algorithm.rs`](../tests/unit/specification/agentic_meta_algorithm.rs).

The loop is a pure, deterministic function of the conversation so far — no
sampling, no hidden state, no neural inference (a NON-GOAL). Given the messages
exchanged and the tool names the agentic CLI advertised, the planner decides the
next step as a small state machine:

```text
web_search → web_fetch → write_file(formalize) → run_command(verify) → final
```

Each step is taken only if the conversation has no prior result for that
capability **and** the CLI advertised a tool providing it, so the planner adapts
to whatever subset of tools a given CLI exposes. Tool *errors* are observed: a
fetch result that looks like an error is not trusted as source text, and the
formalizer falls back to the canonical synopsis so the loop still completes with
a stable, all-nine-primitive knowledge base.

### The eight steps

Each step is one `meta_step` record in the recipe; instantiate them in order to
make the Formal AI solve a new task in agentic mode:

1. **Recognise the agentic task** from the latest user turn against a small
   closed keyword set — a non-match yields `None`, so agentic coding stays
   strictly opt-in and ordinary chat is untouched.
2. **Pin the canonical plan as named constants** (`SEARCH_QUERY`,
   `CANONICAL_SOURCE_URL`, `KB_PATH`) so the recipe is data, not scattered
   literals.
3. **Classify advertised tools into capabilities** (`Search`/`Fetch`/`Write`/
   `Run`) by substring, mirroring agentic-CLI naming so any CLI's tool set maps.
4. **Plan each step as a pure function of history** — the state machine above,
   in `plan_chat_step`.
5. **Understand tool errors and fall back deterministically** so a failed fetch
   never derails the loop.
6. **Wire the planner into the OpenAI-compatible server behind two gates**:
   `agent_mode` (the real guard — every tool is refused unless explicitly opted
   in) and the per-tool permission gate, then turn the plan into a completion
   (`tool_calls` or a final `stop`).
7. **Grant the client-executed tool capabilities** through the permission-only
   `pkg_agentic_coding` associative package, so a permitted agent can drive the
   full loop while granting it by default enables no hidden autonomous action.
8. **Execute the loop with an offline driver and corpus**, bounded by a hard
   `MAX_TURNS` cap, and expose it through the `agent` CLI subcommand, the
   `issue_468_agentic_loop` example, and the integration tests.

### What the recipe records

| Recipe record | Count | Grounded against |
| --- | --- | --- |
| `meta_step` | 8 | ordering 1..8 is contiguous; each `seed_file` exists |
| `meta_constant` | 3 | `pub const <name>: &str` in `src/agentic_coding/planner.rs` |
| `meta_tool` | 4 | `"<tool>"` in `DRIVER_TOOLS`, `Capability::<cap>` in the planner, and the `"<permission>"` / package name in `src/associative_package.rs` |
| `meta_stage` | 5 | `Step <n>:` markers in the planner; ordering 1..5 contiguous |
| `meta_function` | 14 | `fn <name>` in the named source file |
| `meta_primitive` | 9 | each appears in `PRIMITIVE_KINDS` in `src/agentic_coding/formalize.rs`; ordering 1..9 contiguous |
| `meta_bound` | 1 | `const MAX_TURNS: usize = 12;` in `src/agentic_coding/driver.rs` |
| `meta_surface` | 3 | the CLI subcommand, the example, and the integration test each contain their `needle` |

### Running it

```sh
# Verify the agentic recipe still matches the live source:
cargo test --test unit specification::agentic_meta_algorithm -- --nocapture
```

Because this recipe is checked against the source too, the agentic loop and its
recipe can never silently diverge — the loop is itself a reproducible artifact of
the meta-algorithm.
