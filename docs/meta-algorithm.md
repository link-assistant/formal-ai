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

Seven recipes are grounded today. The **recursive core** (issue #559) is the
general algorithm every prompt walks; the other six encode a topic handler or a
self-directed loop on top of it:

| Recipe | Issue | What it reproduces |
| --- | --- | --- |
| [`recursive-core-recipe.lino`](../data/meta/recursive-core-recipe.lino) | #559 | The general meta algorithm itself — 12 steps, 25 pinned functions |
| [`procedural-howto-recipe.lino`](../data/meta/procedural-howto-recipe.lino) | #444 | A chat intent handler for "how to X" |
| [`agentic-coding-recipe.lino`](../data/meta/agentic-coding-recipe.lino) | #468 | The deterministic agentic-CLI loop |
| [`response-language-followup-recipe.lino`](../data/meta/response-language-followup-recipe.lino) | #556 | Re-answering the previous turn in a new language |
| [`document-verification-recipe.lino`](../data/meta/document-verification-recipe.lino) | #535 | Verifying an attached document's claims |
| [`market-price-verification-recipe.lino`](../data/meta/market-price-verification-recipe.lino) | #493 | Fact-checking numeric market-price claims |
| [`dreaming-recipe.lino`](../data/meta/dreaming-recipe.lino) | #540 | Idle memory maintenance and self-generalization |

The other `data/meta/*.lino` files are catalogues, lexicons, and ledgers
(cue sets, route/method aliases, repair cases, the self-AST census, …) that the
recipes and handlers read — they are data, not recipes.

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
| `meta_role` | 11 | `pub const ROLE_* = "<role>"` in `src/seed/roles/intent.rs` **and** `role <role>` in `data/seed/meanings-how.lino` |
| `meta_function` | 8 | `fn <name>` in `src/solver_handler_how.rs` |
| `meta_stage` | 6 | each stage literal emitted in the handler; ordering 1..6 contiguous |
| `meta_parity` | 4 | `fn <rust>` in Rust **and** `function <js>` in the worker |
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

## The recursive core meta-algorithm (issue #559)

The recipes on this page each reproduce *one* handler or loop. The **recursive
core** is the algorithm they all run inside: the ordered procedure that turns
any message into a solved, link-native knowledge base. It is the one recipe
that describes the meta algorithm itself, which is why R335 requires it to be
grounded data rather than prose. It lives at
[`data/meta/recursive-core-recipe.lino`](../data/meta/recursive-core-recipe.lino)
and is grounded by
[`tests/unit/specification/recursive_core_recipe.rs`](../tests/unit/specification/recursive_core_recipe.rs).

Two properties make it different from the topic recipes. First, it is
**executable as data**: `src/recipe_interpreter.rs` parses the `records`
annotation on every trace-recorded step into an ordered program and runs the
recorder primitives in the order the data declares, and the proof obligation is
parity — executing the recipe must reproduce, event for event, the log that
`meta_core::record_meta_core` produces for the same input across every mode
combination (R343). Second, it is **self-improving in proposal-only form**:
`src/meta_self_improvement.rs` reads this recipe against the live pipeline,
detects drift between the algorithm-as-data and the algorithm-as-code, and
proposes the additions and stale-citation removals that reconcile them —
gated `off` by default, never writing the recipe back, so adoption stays a
human review step (R340).

### The twelve steps

Each step is one `meta_step` record in the recipe:

1. **Formalize the impulse to one meaning record.**
2. **Make the meaning a first-class problem frame** that enumerates every need.
3. **Decompose the frame as a recursive, bounded work-unit tree** (downward
   pass), stopping at `max_decomposition_depth`.
4. **Account for every need in a satisfaction ledger**, so a need with no
   method is recorded as blocked rather than silently dropped.
5. **Catalogue the resolving methods as link data** — the method registry
   derived from the live dispatch code.
6. **Attach white-box recursive reasoning to every step**, in both directions.
7. **Construct the answer back up the tree** (upward pass), composing each
   parent from its solved children.
8. **Resolve each atomic leaf through registry-backed method dispatch** — the
   registry is the sole authority (R344).
9. **Record every step as evidence in the append-only log.**
10. **Record the method the registry selects for each leaf**, or mark it
    unresolved.
11. **Project the answer from the event log.**
12. **Accumulate reusable skills and a curriculum from the outcome** —
    proposal-only, nothing auto-promoted (R342).

### What the recipe records

| Recipe record | Count | Grounded against |
| --- | --- | --- |
| `meta_step` | 12 | ordering 1..12 is contiguous; each trace-recorded step names the recorder primitive it drives |
| `meta_function` | 25 | `fn <name>` in the named source file (`src/meta_frame.rs`, `src/method_registry.rs`, `src/meta_reasoning.rs`, `src/meta_construction.rs`, `src/solution_evidence.rs`, `src/selection.rs`, `src/skill_ledger.rs`, `src/recipe_interpreter.rs`, …) |

### Running it

```sh
# Verify the recursive-core recipe still matches the live source:
cargo test --test unit specification::recursive_core_recipe -- --nocapture
```

Because this recipe is checked against the source too — and can be executed to
reproduce the pipeline's own event log — the meta core and its recipe cannot
silently diverge.

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

## The response-language follow-up meta-algorithm (issue #556)

The same grounded-recipe discipline records a third meta-algorithm: the
**deterministic response-language follow-up** that re-answers the *whole class*
of prior requests in a newly requested language. The maintainer's framing for
issue #556 was that we must "generalize to all similar requests (the whole class
of similar questions) in all languages … using our general and universal meta
algorithm and actual recursive reasoning steps, expressed in meta language",
with "all meanings … grounded in external data sources" and "every finest detail
… tested". Its recipe lives at
[`data/meta/response-language-followup-recipe.lino`](../data/meta/response-language-followup-recipe.lino)
and is grounded by
[`tests/unit/specification/response_language_meta_algorithm.rs`](../tests/unit/specification/response_language_meta_algorithm.rs).

The key move is that a bare turn such as *"I do not understand English, write in
Russian"* is **not a new question** — it asks the assistant to re-answer the
previous request in a named language. Rather than localizing one handler, the
follow-up replays the previous user request through the **whole solver** with the
target language forced at a single detection seam, so the retarget generalizes
across every answerable intent family (project lookup, capabilities, identity,
concept lookup, …) at once. This is the universal recursive-reasoning step: the
solver re-derives the prior answer, now constrained to speak the requested
language. The trigger vocabulary is seed data grounded in Wikidata
(`understanding` Q46744; the languages English Q1860, Russian Q7737, Hindi
Q1568, Chinese Q7850) recognised by role — never a hardcoded phrase table
(issue #386). Because translation passes through the language-neutral meta
language, the retarget round-trips (issue #526): an answer produced in one
language replays back into English on request.

### The eight steps

Each step is one `meta_step` record in the recipe; instantiate them in order to
add any *re-answer the previous turn under a new constraint* behaviour:

1. **Detect the requested response language by meaning** — `detect_response_language`
   queries the `response_language_marker` role from the seed lexicon.
2. **Confirm this is a re-answer, not a new question** — a seed-grounded
   comprehension-failure marker or a terse (≤ 4 word) language switch with no
   fresh subject.
3. **Recover the previous user request from history** so the replay sees the same
   context the original answer did.
4. **Force the target language at the single detection seam** — `set_forced_language`
   installs a thread-local override consulted at the top of `language::detect`,
   restored by an RAII guard.
5. **Replay the previous request through the whole solver** — the universal
   recursive-reasoning step that generalizes across every intent family.
6. **Guard recursion and inconclusive replays** — a replay already carries a
   forced language (so it can never re-enter), and unknown/ill-formed/clarify
   replays fall through.
7. **Splice re-answer provenance** onto the replayed answer's evidence
   (`response_language_followup:target:<lang>`, `language_to:<lang>`,
   `response_language_followup:handler:<intent>`).
8. **Mirror the handler in the JS worker** so the WASM/browser surface stays in
   parity (R15).

### What the recipe records

| Recipe record | Grounded against |
| --- | --- |
| `meta_step` | ordering 1..8 is contiguous; each `seed_file` exists |
| `meta_role` | `pub const <CONST>: &str = "<role>";` in `src/seed/roles/language.rs` and `role <role>` in the seed |
| `meta_grounding` | a cached Wikidata entity at `data/cache/wikidata/entity/Q<id>.lino`, and the Q-id present in the seed |
| `meta_function` | `fn <name>` in the named source file |
| `meta_seam` | the forced-language token in both `src/language.rs` and the JS worker |
| `meta_parity` | `fn <rust>` in Rust and `function <js>` in the JS worker |
| `meta_test` | the pinning test file exists and describes what it pins |

### Running it

```sh
# Verify the response-language recipe still matches the live source:
cargo test --test unit specification::response_language_meta_algorithm -- --nocapture
```

Because this recipe is checked against the source too, the response-language
follow-up and its recipe can never silently diverge — the retarget is itself a
reproducible artifact of the meta-algorithm.

## The document verification meta-algorithm (issue #535)

The same grounded-recipe discipline records a fourth meta-algorithm: the
**deterministic document verification** handler that checks an attached
document's originality, authenticity, and facts. The maintainer's framing for
issue #535 was that a request such as *"Проверь данный текст на уникальность и
на плагиат"* ("Check this text for uniqueness and plagiarism") with a text file
attached must not answer `intent: unknown`; that we must "fully support attached
files … in Desktop, Telegram bot, and Web app, and in other interface surfaces",
"generalize to all similar requests (the whole class of similar questions) in
all languages … expressed recursively through meanings (meta language)", ground
"all meanings … in external data sources", and "fully use
github.com/link-foundation/relative-meta-logic for relative statements
probability" — trusting original first sources, using "newspapers/journals for
their original content", and ignoring "any unoriginal content or reposting",
with "every finest detail … tested". Its recipe lives at
[`data/meta/document-verification-recipe.lino`](../data/meta/document-verification-recipe.lino)
and is grounded by
[`tests/unit/specification/document_verification_meta_algorithm.rs`](../tests/unit/specification/document_verification_meta_algorithm.rs).

The key move is that the request is recognised **by meaning, not by phrase**: it
asks the assistant to verify the attached document. Three seed roles must fire —
the check/verify action, the subject (plagiarism / originality / uniqueness /
authenticity / veracity), and the document (text / article / file, or an
attachment on its own). All three are seed data grounded in Wikidata
(`verification and validation` Q953429, `plagiarism` Q164666, `originality`
Q2914681, `authentication` Q2360032, `fact-checking` Q59555084, `document`
Q49848) recognised by role — never a hardcoded per-language phrase table
(issue #386). Because the vocabulary is data, the handler generalises across the
whole verification class in every supported language (en/ru/hi/zh) at once. The
attachment arrives through the **shared attachment context**, so Desktop, the
Telegram bot, the Web app, and the HTTP/CLI boundary all deliver the same
verifiable content to one handler.

Each atomic statement in the document is then weighed under
**relative-meta-logic**: `StatementAssessment::assess` starts every user
statement at the assumed-true prior (`ASSUMED_TRUE_PRIOR = 0.6`) and moves it
only under evidence, aggregating by `SourceTier` — original first parties (1.0)
and original journalism (0.85) are trusted, independent corroboration is weaker,
and unoriginal reposts contribute nothing and are ignored. Wikinews is the
registered original-journalism tier, opt-out-able via the
`externalServiceMediawikiFamily` settings toggle. The whole path is mirrored in
the JS worker so the WASM/browser surface stays in parity (R15).

### The eight steps

Each step is one `meta_step` record in the recipe; instantiate them in order to
add any *verify the attached content* behaviour:

1. **Recognise the verification request by meaning, not phrase** — all three
   seed roles must fire, queried from the lexicon.
2. **Ingest the attached document or inline text sample** through the shared
   attachment context, so every interface surface reaches one handler.
3. **Split the content into atomic statements** so each claim is verified on its
   own rather than the document as a whole.
4. **Ground each statement in a web-search fact-check query** so the fusion
   layer surfaces original first sources for or against it.
5. **Weigh the evidence under relative-meta-logic** with the trusted-source tier
   policy (original first sources trusted, unoriginal reposts ignored).
6. **Consult the registered original-journalism source** (Wikinews), opt-out-able
   from settings.
7. **Splice the grounded verdict onto the evidence trace** so the trace names
   exactly why each statement was trusted or doubted and which sources moved it.
8. **Mirror the handler in the JS worker** so the WASM/browser surface stays in
   parity (R15).

### What the recipe records

| Recipe record | Grounded against |
| --- | --- |
| `meta_step` | ordering 1..8 is contiguous; each `seed_file` exists |
| `meta_role` | `pub const <CONST>: &str = "<role>";` in `src/seed/roles/intent.rs` and `role <role>` in the seed |
| `meta_grounding` | a cached Wikidata entity at `data/cache/wikidata/entity/Q<id>.lino`, and the Q-id present in the seed |
| `meta_function` | `fn <name>` in the named source file; the handler is wired into `src/solver_dispatch.rs` |
| `meta_parity` | `fn <rust>` in Rust and `function <js>` in the JS worker |
| `meta_external_service` | `source` + `settings_key` in `data/seed/sources-registry.lino` |
| `meta_test` | the pinning test file exists and describes what it pins |

### Running it

```sh
# Verify the document-verification recipe still matches the live source:
cargo test --test unit specification::document_verification_meta_algorithm -- --nocapture
```

Because this recipe is checked against the source too, document verification and
its recipe can never silently diverge — the handler is itself a reproducible
artifact of the meta-algorithm.

## The market-price verification meta-algorithm (issue #493)

The same grounded-recipe discipline records a fifth meta-algorithm, a
**sub-algorithm of document verification** aimed at a specific, high-value class
of statements: the **deterministic market-price fact check**. Issue #493 showed a
screenshot that repeats *"ETH in 2024: $1,700"* across many years and scenarios,
and the maintainer's framing was that we must not just catch that one line but
"support the entire class of similar questions, not just the specific example" —
any checkable numeric claim about a traded asset's price in a period, generalised
across assets, years, and every supported language, grounded in external data.
Its recipe lives at
[`data/meta/market-price-verification-recipe.lino`](../data/meta/market-price-verification-recipe.lino)
and is grounded by
[`tests/unit/specification/market_price_verification_meta_algorithm.rs`](../tests/unit/specification/market_price_verification_meta_algorithm.rs).

The key move is that the check is **data-driven, not example-driven**. A single
registry,
[`data/seed/market-price-references.lino`](../data/seed/market-price-references.lino),
declares one `asset` per traded instrument, each carrying its external
`grounded-in` Wikidata entity (Ethereum `Q16783523`, Bitcoin `Q131723`), a
`quote-currency`, per-language `lexeme`/`surface` aliases (`$ETH`, `ethereum`,
`etherium`, `эфириум`, `एथेरियम`, `以太坊`, …), and one `reference <period>` per
checkable period holding the recorded daily-candle `observed-min-price` /
`observed-max-price` and their dates from a named first-party source. Adding an
asset or a year is a seed edit — never a new code path.

Every line of the sample is then run through the same chain: `asset_positions`
recognises assets by scanning the whole alias registry (issue #386: no hardcoded
per-language phrase table in Rust), `market_price_fragments` splits a line into
one fragment per asset it names, `parse_market_price_claim` reads the period and
the currency-marked amount, and `assess_market_price_claim` looks up the recorded
range and marks the claim `contradicted` **only** when its price is below the
recorded minimum or above the recorded maximum — otherwise `within_recorded_range`.
The verdict is a `RelativeEvidence` at `SourceTier::OriginalFirstParty` fed into
`StatementAssessment::assess`, so a price the asset actually reached in the period
is never flagged. This is why the screenshot's 2021–2023 lines stay uncontradicted
(\$1,700 was inside those ranges) while only the false 2024 line is caught, and why
the very same machinery flags an impossible `BTC in 2024: $1,700` with no
ETH-specific or year-specific code. The whole path is mirrored in the JS worker
so the WASM/browser surface stays in parity (R15).

### The eight steps

Each step is one `meta_step` record in the recipe; instantiate them in order to
add any *is this numeric market claim true?* behaviour:

1. **Register verifiable assets and recorded price ranges as seed data** in
   `data/seed/market-price-references.lino` — the single source of truth.
2. **Ground each asset in an external entity** (Wikidata), so every alias of an
   asset resolves to one language-independent meaning.
3. **Recognise assets by alias across languages**, scanning the whole registry
   rather than any single ticker (no per-language phrase table in Rust).
4. **Split each line into one fragment per named asset**, so two assets on one
   line yield one claim each.
5. **Parse the claimed period and numeric price** from each fragment into a
   structured claim independent of the surface wording.
6. **Weigh each claim against the recorded range under relative-meta-logic** —
   contradict only outside `[observed-min, observed-max]`, never over-claiming.
7. **Splice the grounded verdict onto the evidence trace** with `market_price_claim:*`
   markers naming status, source, range, and posterior.
8. **Mirror the whole path in the JS worker** so the WASM/browser surface stays
   in parity (R15).

### What the recipe records

| Recipe record | Grounded against |
| --- | --- |
| `meta_step` | ordering is contiguous from 1; each has a `detail` and an existing `seed_file` |
| `meta_grounding` | a cached Wikidata entity at `data/cache/wikidata/entity/Q<id>.lino`, and the Q-id present in the seed registry |
| `meta_data` | the registry `data_file` exists and declares its root node |
| `meta_function` | `fn <name>` in the named source file |
| `meta_parity` | `fn <rust>` in Rust and `function <js>` in the JS worker |
| `meta_test` | the pinning test file exists and describes what it pins |

### Running it

```sh
# Verify the market-price-verification recipe still matches the live source:
cargo test --test unit specification::market_price_verification_meta_algorithm -- --nocapture
```

Because this recipe is checked against the source too, the market-price fact
check and its recipe can never silently diverge — catching false numeric claims
is itself a reproducible, data-driven artifact of the meta-algorithm that scales
to the whole class of assets, periods, and languages by editing seed data alone.

## The dreaming meta-algorithm (issue #540)

The same grounded-recipe discipline records a sixth meta-algorithm, this time
turned **inward**: the low-priority **dreaming** planner that maintains memory
and lets Formal AI *change its own meta-algorithm* from stored experience. Issue
#540 asked that, while idle and never blocking the UI, the assistant restructure
deduplication by recalculated frequency of use, keep roughly a 20% free-space
reserve (issue #494) by forgetting only recomputable/refetchable data, and —
crucially — *learn more about the topics the user interacts with*, remember the
requirements the user has stated so he never has to repeat himself, and
**generalize** them so "when we solve similar tasks to previous tasks … new
user's requirements are baked in", after which "if we don't have enough space we
can forget specifics about test runs, but our general meta algorithm must keep
changes that allow it to solve all other tasks." Its recipe lives at
[`data/meta/dreaming-recipe.lino`](../data/meta/dreaming-recipe.lino) and is
grounded by
[`tests/unit/specification/dreaming_meta_algorithm.rs`](../tests/unit/specification/dreaming_meta_algorithm.rs).

The planner starts as one pure function, `plan_memory_dreaming`, in
[`src/dreaming.rs`](../src/dreaming.rs): it only reads memory and proposes work,
so planning is safe in the background. Learning follows memory links:
multilingual cue data lifts requirements, candidate tasks are replayed against
proposed amendments, and recurring structures are mined directly from repeated
task records. Only exact normalized replay grants coverage. Applied amendments
are stored as structured `meta_algorithm_amendment` events and read by
`src/dreaming_application.rs` on later protocol requests, which makes learned
rules change future answers. Physical removal additionally requires persisted
consent and real filesystem pressure measured by `src/storage_policy.rs`.

### The thirteen steps

Each step is one `meta_step` record in the recipe; instantiate them in order to
add any *dream about stored experience* behaviour:

1. **Classify every event by durability** into one `DreamingDurability`, so
   recomputability is known and raw experience and learning are protected.
2. **Recalculate how often each event is actually used**, driving deduplication
   and eviction by recalculated frequency of use.
3. **Restructure recomputable duplicates** around the most-reused copy.
4. **Recalculate which topics the user interacts with most** into ranked
   `TopicFrequency` records.
5. **Recover durable requirements from multilingual cue data** into
   `LearnedRequirement` records.
6. **Propose a meta-algorithm amendment** for each learned requirement.
7. **Derive and replay candidate tasks**, granting coverage only on a matching
   recorded output.
8. **Mine recurring task structures** independently of requirement cue words.
9. **Apply retained amendments to later similar tasks** through both
   OpenAI-compatible protocol surfaces.
10. **Measure real storage and actual incoming bytes** on the memory filesystem.
11. **Reclaim toward the 20% reserve** only from recomputable links.
12. **Forget replay-verified specifics** with
    `DreamingActionKind::ForgetCoveredSpecific`, retaining amendments and
    patterns.
13. **Run while truly idle**, yielding to foreground work and requiring a
    persisted user choice before automatic cleanup.

### What the recipe records

| Recipe record | Grounded against |
| --- | --- |
| `meta_step` | ordering is contiguous from 1; each has a `detail` and an existing `source_file` |
| `meta_function` | `fn <name>` in the named source file |
| `meta_constant` | the token present in the named source file, with a stated purpose |
| `meta_test` | the pinning test file exists and describes what it pins |

### Running it

```sh
# Verify the dreaming recipe still matches the live source:
cargo test --test unit specification::dreaming_meta_algorithm -- --nocapture
```

Because this recipe is checked against the source too, the dreaming planner and
its recipe cannot silently diverge: replay, application, storage, consent, and
runtime stages are all pinned to the live code.

## The promotion meta-algorithm (issue #656)

Every self-improvement loop above stops at *proposing* — the meta self-improvement
loop proposes recipe deltas, white-box learning proposes seed rules, dreaming
proposes amendments — and none of them writes `data/seed/`. Issue #656 (E37) adds
the missing, deterministic step that closes the loop safely: a **promotion**
protocol that materializes a proposal into seed data **only** after it clears its
benchmark ratchets, and even then only as a `.lino` seed edit written onto a
branch — never a direct push. Draft pull requests and human review stay the outer
gate. This section is grounded by
[`src/promotion.rs`](../src/promotion.rs) and pinned by
[`tests/unit/issue_656_promotion.rs`](../tests/unit/issue_656_promotion.rs) and
[`tests/integration/issue_656_improve.rs`](../tests/integration/issue_656_improve.rs).

The protocol runs through `formal-ai improve --promote`:

1. **Collect actual open proposals** — from the required `--proposals`
   `promotion_proposals` Links Notation document. The document declares source,
   summary, and desired seed edit; it cannot supply commands, floors, or observed
   counts. Adoptable learned rules bridge into candidates through
   `promotions_from_learning_run`. Demonstration data is confined to tests and
   examples, never the CLI default.
2. **Replay one canonical gate batch** — `src/promotion/gates.rs` executes the
   coding-modification suite (issue #362), industry suite (issue #304), and unit
   specifications from an internal allow-list. Manifest floors and pass-rate
   policy are authoritative. Exit failure blocks every proposal; successful
   output without parseable pass/fail evidence fails closed. A digest binds each
   event to the command, status, stdout, and stderr.
3. **Decide** — a proposal that clears every ratchet is `Promoted`; any failing
   ratchet makes it `Rejected`.
4. **Record the decision as an append-only event chain** — `promotion_proposal`,
   one `promotion_evidence` per ratchet, `promotion_decision`, then either
   `promotion_applied` (the materialized seed edit) or `promotion_rejection`.
   These custom-kind events round-trip through the bundle export/import path.
5. **Materialize through Formal AI's Agent path** — `--apply` requires
   `--confirm`, a clean Git worktree, and creates `promotion/<run-id>` locally.
   Accepted edits targeting the same file are coalesced. Formal AI executes the
   literal task through `run_agentic_task`; only an Agent-authored `write_file`
   call whose path and content match byte-for-byte is copied into `--seed-root`.
   The deterministic Agent session id is recorded. Rejected proposals are
   **never** applied; their
   `promotion_rejection` record keeps the un-applied change together with the
   failing benchmark evidence, mirroring the R425 `dreaming_candidate_failure`
   durability pattern.
6. **Stop on the local review branch** — the run yields a `PromotionBranchPlan`
   for committing and opening a draft pull request, but never pushes. After an
   authorized push, GitHub required checks run against the actual branch SHA and
   human review remains the final outer gate; local replay does not claim to
   predict that future CI result.

### What the protocol records

| Event kind | What it captures |
| --- | --- |
| `promotion_proposal` | the proposal link and which seed file it edits |
| `promotion_evidence` | which ratchet ran, at what floor, cleared or blocked |
| `promotion_decision` | `promoted` or `rejected`, with all evidence links |
| `promotion_applied` | the materialized seed edit (promoted proposals only) |
| `promotion_rejection` | the change kept but **not** applied (rejected only) |

### Running it

```sh
# Dry run: replay the gates and print the plan without touching any files.
formal-ai improve --promote --proposals ./open-promotions.lino

# Materialize the accepted seed edits into a workspace (never a push):
formal-ai improve --promote --proposals ./open-promotions.lino \
  --apply --confirm --seed-root ./clean-git-worktree

# Verify the promotion protocol end to end:
cargo test promotion_protocol
```

Because proposal input cannot choose its runner, floor, rate, or result, a
proposal cannot promote itself by fabricating evidence. The seed edit is written
only when fresh canonical output clears every policy and the Formal AI Agent
authors the exact requested bytes on a local review branch.
