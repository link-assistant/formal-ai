# Plan 09 — Handler migration ratchet and the doublets store as the read path (bottleneck B9 of #1138)

Written before the code, in the convention `docs/case-studies/issue-710/plans/README.md`
establishes: a box is ticked in the commit that lands it, and a step that turns
out wrong is struck through with the reason, never deleted.

Standing doctrine, unchanged: associative stack only (seed data in `.lino`,
registry methods, doublets store) over specialized `try_*` Rust handlers;
generalization over memoization with held-out paraphrases in en, ru, hi, zh, es;
deterministic and honest; no deferral, no budgets, no bypasses; nothing
hard-coded for any test; ratchets only move in the strict direction.

---

## Issues addressed

- **#559** (closed; PR #560) — "Previous specialized logic must be removed… only
  memory + meta algorithm." PR #560 delivered the registry as the sole *route*
  authority; the handlers themselves were never migrated. This plan delivers the
  remaining half: the method bodies.
- **#699** (closed; PR #877) — the migration ledger plus batches 1–3
  (`number_constraint_reasoning`, `who_is`, `definition_merge`, the
  `program_synthesis` dead end). Requirement 4 of #699 — "The number of
  specialized handler files and `try_*` dispatch entries must **monotonically
  decrease** per batch" — is the ratchet this plan repairs and then drives down.
- **#959** — the five open items: a CI assertion that pending and both ceilings
  may only decrease; migrate the contextual + prelude match arms; derive the
  browser worker's handler order from `handler-precedence.lino`; move the
  hard-coded promotion predicates into seed; fix the self-satisfying closure
  metric. This plan delivers all five.
- **#948** — canned summaries (`summary-topics.lino` + `benchmark_prompts.rs`),
  the three `contains(...)` comparison blocks in `research_table.rs`, the
  per-idiom policy handlers, and duplicated identity/greeting answers. Items 1–3
  land here as migration batches; item 4 (seed answer de-duplication) lands as
  its own leaf.
- **#433** (closed; PR #434) — the audit that first classified ~10
  fixed-enumeration handlers. Its "prototype one handler purely through rule
  primitives" acceptance criterion is now the general mechanism
  (`src/rule_interpreter.rs`), and this plan generalizes it to the rest.
- **#918** — the minimal-core boundary ledger and its burn-down gate. This plan
  keeps the boundary and folds its file census into one shared definition.
- **#1126** — the debt ratchet. This plan makes it strict-downward and adds the
  measures that are currently unratcheted.
- **#663** — precedence as data. Delivered for Rust; this plan completes it for
  the browser.
- **#1085 D1.3 / #1095 / #1101 / #1115** — the four most recent migrations and
  misroutes; they supply the mechanism (`data/seed/handler-rules.lino`) and the
  evidence that precedence order, not handler quality, decides many outcomes.
- **#558** — source ↔ links translation, the end state in which the doublets
  store is what the solver reads. This plan delivers the first two stages.
- **Issues plan 13's coverage table names this plan as a deliverer of** (added by
  the 2026-09-16 reconciliation, so both documents agree): **#934** (E82, the
  worker budget turns downward — partial, absorbing all ~26,700 JS lines is
  multi-slice); **#942** (E90, the redaction skill lands as a registry method
  over plan 04's formalized concepts); **#950** (E98, the terminology lint —
  leaf 42, added by the reconciliation); **#951** (E99, the first `main.jsx`
  clusters move — partial); **#952** (E100, the browser reads the seed through
  the WASM parser — leaf 13); **#953** (E101, the desktop permission logic moves
  — partial).
- **#1138 B9** — the umbrella.

---

## Current state

### Gate numbers, measured in this worktree on 2026-09-16

All three read-only gates were run; all three pass.

```
$ rust-script scripts/check-minimal-core-boundary.rs
minimal-core boundary: 45 handler sources, 18466 outside-core lines      (exit 0)

$ rust-script scripts/check-hardcoded-language.rs
Detected prose literals: 1286 | allowlisted: 1286
No new hardcoded natural language; allowlist is in sync (1286 entries).   (exit 0)

$ rust-script scripts/check-debt-ratchet.rs
  handler_files: measured 42 / ceiling 42
  handler_migration_pending: measured 40 / ceiling 40
  hardcoded_language_rows: measured 1286 / ceiling 1286
  literal_predicates: measured 548 / ceiling 549
debt ratchet holds                                                        (exit 0)
```

Sources of those ceilings: `data/meta/core-boundary-ledger.lino:4-7`
(`source_file_count_max 45`, `source_lines_max 18466`,
`outside_core_file_count_max 45`, `outside_core_lines_max 18466`) and
`data/meta/debt-ratchet.lino:7-22` (the four `ceiling` blocks).

### Census: how many handlers there actually are

| Thing counted | Value | Evidence |
| --- | --- | --- |
| Rows in the precedence table | 58 | `data/seed/handler-precedence.lino:2-59` |
| Ledger entries (must equal the above) | 58 | `data/meta/handler-migration-ledger.lino`, `handler ` heads |
| … `status migrated` | 16 | ledger; asserted at `tests/unit/issue_699_handler_migration.rs:240-245` |
| … `status "justified-native"` | 2 | `arithmetic` (ledger:99-102), `javascript_execution` (ledger:103-106) |
| … `status pending` | 40 | ledger; asserted at `tests/unit/issue_699_handler_migration.rs:251-255` |
| Native function rows in `HANDLER_FUNCTIONS` | 46 | `src/solver_dispatch.rs:286-385` |
| … whose function is a `try_*` | 39 | same table; the other 6 are `solve_number_constraints`, `handle_arithmetic`, `handle_javascript_execution`, `merge_definitions`, `handle_concept_lookup`, `resolve_who_is` |
| Seed rule-set handlers | 12 | `data/seed/handler-rules.lino`, `handler ` heads (46 + 12 = 58) |
| Prelude methods, **not** in the precedence table or the ledger | 5 | `src/solver_dispatch.rs:179-185` (`diagnostic`, `nl_tool`, `behavior_rules`, `feature_capability`, `playwright_script`) |
| Contextual-override names | 13 | `src/solver_dispatch.rs:152-171` |
| Browser worker `syncHandlers` entries | 31 | `src/web/worker/formal_ai_worker_20.js:577-693` |
| Shared routing-parity invariants pinned | 5 | `tests/fixtures/routing-parity.lino` |

### The three different counts of "handler files"

Three gates count handler files three different ways, and all three pass, so no
gate notices the disagreement:

| Gate | Definition | Measured | Ceiling |
| --- | --- | --- | --- |
| `tests/unit/issue_699_handler_migration.rs:175-196` | `read_dir` (non-recursive) on `src/solver_handlers`, minus `mod.rs`/`modules.rs` | **36** | 37 (`:19`) |
| `scripts/check-debt-ratchet.rs` via `data/meta/debt-ratchet.lino:8-10` | recursive `*.rs` minus `mod.rs`/`modules.rs` | **42** | 42 |
| `scripts/check-minimal-core-boundary.rs:136-141,150-169` | recursive `*.rs` minus `modules.rs` only (so `mod.rs` counts) | **45** | 45 |

The `try_*` ceiling is worse: measured **39**, recorded max **50**
(`tests/unit/issue_699_handler_migration.rs:20`), i.e. eleven entries of
headroom in which new handlers may be added while the gate stays green. The
ledger itself admits how that happened —
`data/meta/handler-migration-ledger.lino:4`:

> `ceilings "issue #1085: the raisable specialized_handler_files_max / try_dispatch_entries_max ceilings were raised twice (2026-08-02, 2026-08-11); the measured, lower-only ceilings now live in data/meta/kernel-ratchet.lino"`

`data/meta/kernel-ratchet.lino` **does not exist** in the tree (`find . -name
"kernel-ratchet*"` returns nothing); the measures moved to
`data/meta/debt-ratchet.lino` under issue #1126 and the ledger note was never
updated. ROADMAP.md:362 still quotes the pre-#1126 figures, "37 handler files /
48 `try_*` registry entries", which is where #1138's B9 text gets its numbers.

`check-debt-ratchet.rs` is **at-or-below**, not strict: `literal_predicates` is
already stale by one (measured 548, ceiling 549) and nothing fails.
`check-minimal-core-boundary.rs:286-290` is the correct model — it errors on
both directions, "improved from {ceiling} to {actual}; lower the reviewed
ceiling".

### The 40 pending handlers and the files that implement them

Line counts are the reviewed `baseline_lines` of `data/meta/core-boundary-ledger.lino`
(verified equal to the files on disk by the gate run above). Files marked † are
**outside** `src/solver_handlers/` and therefore outside the minimal-core
ledger entirely.

| Pending handler (ledger) | Implementing file(s) | Lines |
| --- | --- | --- |
| `http_fetch`, `url_navigate`, `web_search` | `src/solver_handlers/web_requests.rs` | 992 |
| " | `src/solver_handlers/web_search_intent.rs` | 872 |
| " | `src/solver_handlers/web_requests/live_search.rs` | 87 |
| `document_originality_check` | `src/solver_handlers/document_originality.rs` | 350 |
| `learn_from_source` | `src/solver_handlers/mod.rs:824-876` | (in mod.rs, 941) |
| `research_comparison_table`, `research_result_followup` | `src/solver_handlers/research_table.rs` | 485 |
| `procedural_how_to`, `how_it_works` | `src/solver_handler_how.rs` † | 888 |
| `procedural_how_to` (contextual) | `src/solver_handler_how_synthesis.rs` † | 135 |
| `procedural_how_to_followup` | `src/solver_handler_how.rs` † | (above) |
| `conversation_memory` | `src/solver_handlers/conversation_memory/mod.rs` | 813 |
| " | `…/memory_write.rs` | 294 |
| " | `…/program_query.rs` | 269 |
| " | `…/conversation_summary.rs` | 156 |
| " | `…/link_query.rs` | 126 |
| `software_project_followup` | `src/solver_handlers/software_project_followup.rs` | 396 |
| `summarization`, `brainstorming`, `conversation_topic`, `fact_lookup`, `coreference`, `roleplay` | `src/solver_handlers/benchmark_prompts.rs` | 543 |
| `text_manipulation` | `src/solver_handlers/text_manipulation.rs` | 823 |
| " | `src/solver_handlers/text_edit_ops.rs` | 328 |
| `translation` | `src/solver_handlers/mod.rs:475-690` | (in mod.rs) |
| `response_language_followup` | `src/solver_handlers/response_language_followup.rs` | 122 |
| `calendar_reasoning`, `calendar_create_event` | `src/solver_handlers/calendar.rs` | 976 |
| " | `src/solver_handlers/calendar_ics.rs` | 198 |
| `compound_interest` | `src/solver_handlers/compound_interest.rs` | 557 |
| `numeric_list` | `src/solver_handlers/numeric_list/mod.rs` | 853 |
| " | `src/solver_handlers/numeric_list/codegen.rs` | 521 |
| `shell_command_transform` | `src/solver_handlers/shell_command_transform.rs` | 269 |
| `pattern_inference` | `src/solver_handlers/pattern_inference.rs` | 359 |
| `concept_lookup` | `src/solver_handlers/mod.rs:207-390` | (in mod.rs) |
| `meta_explanation` | `src/solver_handlers/meta_explanation.rs` | 324 |
| `network_query` | `src/solver_handlers/mod.rs:417-474` | (in mod.rs) |
| `execution_failure` | `src/solver_handlers/mod.rs:755-785` | (in mod.rs) |
| `installation_conversion` | `src/solver_handlers/installation_conversion.rs` | 866 |
| `write_script` | `src/solver_handlers/mod.rs:691-728` | (in mod.rs) |
| `document_generation_plan` | `src/solver_handlers/document_request.rs` | 645 |
| `software_project` | `src/solver_handlers/software_project.rs` | 909 |
| " | `src/solver_handlers/software_project_code.rs` | 224 |
| `algorithm` | `src/solver_handlers/mod.rs:729-754` | (in mod.rs) |
| `source_refresh` | `src/solver_handlers/mod.rs:786-823` | (in mod.rs) |
| `source_conflict` | `src/solver_handlers/mod.rs:878-905` | (in mod.rs) |
| `proof_request` | `src/solver_handlers/user_intent.rs` | 236 |
| `incompatible_units` | `src/solver_handler_units.rs` † | 150 |

Files in the boundary ledger that **no** pending ledger row names, because they
implement prelude or contextual methods that the precedence census does not
cover: `behavior_rules.rs` (951), `feature_capability.rs` (650),
`self_awareness.rs` (550), `natural_language_tools.rs` (334),
`playwright_script.rs` (140), `agent_workspace.rs` (128),
`behavior_rule_followups.rs` (102), `behavior_rule_matching.rs` (93),
`task_decomposition.rs` (243), `fact_checking.rs` (109), `world_state.rs` (99),
`procedure_rules.rs` (112), `program_blueprint.rs` (98),
`curated_project_fetch.rs` (54), `calculator_rate.rs` (126),
`program_synthesis.rs` (143, already `migrated`), `mod.rs` (941).

### The 19 hard-coded promotion predicates

`src/intent_formalization/prompt_relevants.rs:29-158` builds one array of
`(handler_name, bool)` pairs; every `true` hoists that handler ahead of the
*whole* precedence table (documented at `:18-25`). Nineteen entries today, not
the eighteen #959 counted:

`conversation_control` (:31), `execution_failure` (:35), `program_synthesis`
(:42), `arithmetic` (:47), `web_search` (:48), `task_decomposition` (:58),
`procedural_how_to` (:62), `proof_request` (:67), `fact_checking` (:71),
`world_state` (:78), `installation_conversion` (:91), `write_script` (:95),
`write_program` (:99), `text_manipulation` (:103), `software_project` (:107),
`meta_explanation` (:111), `pattern_inference` (:121), `concept_lookup` (:125),
`calendar_create_event` (:130-157).

The calendar entry still carries the inline glue #959 named:
`normalized.contains("в ") || normalized.contains(':')` at
`prompt_relevants.rs:154`, with the comment at `:148-151` explaining that the
composite "stays in code".

`src/meta_method_dispatch.rs` still holds four `if name == "…"` special cases
(`:63`, `:95`, `:105`, `:122`) plus a five-armed `match name` prelude at `:189-203`
inside the supposedly uniform executor.

### The self-satisfying closure metric

`data/seed/closure-generated-01.lino` … `-16.lino`: **16 files, 17,791 lines,
3,555 glosses, every one `lexeme en` only**. They are explicitly *not* loaded:
`data/meta/seed-registry.lino:457-459` lists them under

> `unregistered closure-generated-*` / `owner "python3 scripts/close-total.py"` /
> `reason "Derived from the other seed files by the total-closure pass, so embedding a snapshot of them would embed the same statements twice."`

so they get no `include_str!` constant (`src/seed/embedded_registry.rs` has
none) and the browser never fetches them (`src/web/seed-files.js` does not list
them). Their only consumer is `scripts/audit-total-closure.py`, whose output
`tests/unit/total_closure.rs:47-60` asserts must be exactly zero unresolved.
The generator writes the definitions that the auditor then finds — a closed
loop that reports 0 % gap while the runtime lexicon has no more coverage than
before. #959 item 5 names this precisely.

### The doublets store is written but never read

`VISION.md:320-326`:

> "the solver reasons over Rust structures -- `MemoryStore` is a vector of
> events, the seed is parsed into Rust tables by `src/seed.rs`, and
> `src/solver.rs`, `src/engine.rs` and `src/main.rs` never read the doublets
> store. The link-cli store is a write-behind projection, filled by
> `memory_sync.rs` after each `.lino` write."

Confirmed by grep: `src/solver.rs` has no `link_store` reference;
`src/link_store.rs:1-9` describes itself as the "swappable Links Notation and
link-cli storage boundary" whose `.lino` projection is the export format;
`src/memory_sync.rs:1-19` is the write-behind sync layer. The read path for
every routing decision is `crate::seed::lexicon()` and `cue_lexicon::matches`,
both Rust tables parsed once at boot.

### The browser worker's handler list is a hand-written JS copy

`src/web/seed-files.js:22` fetches `seed/handler-precedence.lino` at startup.
Grepping the whole of `src/web/` for any consumer of that file
(`handler_precedence`, `handlerPrecedence`, `parseHandlerPrecedence`) returns
**nothing**. The browser's actual order is the declaration order of the
31-entry `syncHandlers` array literal at
`src/web/worker/formal_ai_worker_20.js:577-693`, iterated at `:694`.
`tests/fixtures/routing-parity.lino:8-10` states the current position:

> "Full order-parity is impossible on purpose — the worker names its handlers
> differently and runs its async fetch/network handlers in a later phase"

and pins only five shared invariants, checked by
`tests/unit/specification/routing_precedence.rs:221-260` with a *substring
search of the JS source text* (`worker.find(&format!("\"{name}\""))`, `:234-238`).

### Canned answers still live (#948)

- `data/seed/summary-topics.lino:9-18` holds three topics (`Rust`, `Wikipedia`,
  `formal-ai`) with English `body` strings returned verbatim by
  `src/solver_handlers/benchmark_prompts.rs:34-57`, in any prompt language.
- `src/solver_handlers/research_table.rs:414`, `:430`, `:446` — three
  `normalized.contains("machine learning algorithm" | "deep learning" +
  "traditional ml" | "neural network")` blocks returning hard-coded English
  prose; the column labels are Rust consts at `:26-33`.
- The per-idiom policy handlers #948 named (`kupi_slona`,
  `physical_action_question`, `shell_refusal`, `punctuation_only_prompt`,
  `opinion_question`, `ill_formed`) **have** since migrated — ledger lines
  149-186 — into `data/seed/handler-rules.lino` under `src/rule_interpreter.rs`.
  That is the proof the mechanism works; item 3 of #948 is done, items 1, 2 and
  4 are not.

---

## Root causes

1. **The ratchet measures the wrong noun, in three incompatible ways.**
   "Handler files" is counted non-recursively by the unit test (36/37),
   recursively-minus-`mod.rs` by the debt ratchet (42/42) and
   recursively-with-`mod.rs` by the boundary gate (45/45). *Mechanism:* a
   migration that moves code from `web_requests.rs` into a new
   `web_requests/live_search.rs` lowers no ceiling and can raise two of the
   three counts; a migration that moves code out of `src/solver_handlers/`
   entirely — into `src/solver_handler_how.rs`, `src/agentic_coding/`, or
   `src/rule_interpreter.rs` — lowers all three while changing nothing about
   how specialized the behaviour is. `src/agentic_coding/` is already 79 files
   and 22,125 lines with its own ~45-arm hand-ordered `if let Some(plan)` chain
   (`src/agentic_coding/planner.rs:326-650`), entirely outside every handler
   ratchet.

2. **The ratchet is at-or-below, not strict, so it ratchets nothing.**
   `check-debt-ratchet.rs` only fails on `measured > ceiling`;
   `literal_predicates` is already 548 against a ceiling of 549 and the gate is
   green. The `try_*` ceiling sits at 50 against a measured 39 — eleven free
   slots — and was demonstrably *raised* twice
   (`data/meta/handler-migration-ledger.lino:4`), which directly violates #699
   requirement 4. Only `check-minimal-core-boundary.rs:286-290` implements the
   strict rule.

3. **The migration ledger's census is the precedence table, not the registry.**
   `tests/unit/issue_699_handler_migration.rs:220-239` asserts the ledger covers
   `handler-precedence.lino` exactly. The five prelude methods
   (`src/solver_dispatch.rs:179-185`) are registry methods with real Rust
   bodies — `behavior_rules.rs` alone is 951 lines — and appear in no ledger
   row, so they are unmigratable by construction and invisible to the pending
   count.

4. **Promotion is a second, higher-precedence routing authority written in
   Rust.** `prompt_relevants.rs:18-25` documents that a promoted handler is
   hoisted "ahead of the *whole* `handler-precedence.lino` table". So the seed
   file that ARCHITECTURE.md:167-172 calls the dispatch authority is routinely
   overridden by 19 Rust booleans, two of which (`calendar_create_event`'s
   `contains("в ")`/`contains(':')` at `:154`, and `write_program`'s
   `requested_write_program_parameters` at `:101`) are literal-string glue. Each
   new capability adds a twentieth predicate rather than a seed row.

5. **`try_dispatch` is not uniform.** `src/meta_method_dispatch.rs:63,95,105,122`
   name handlers by string inside the executor, and `:189-203` is a five-armed
   prelude match. A method the registry resolves therefore does not execute the
   same way as its neighbours, so "the registry is the sole dispatch authority"
   (R344) is true of *selection* and false of *execution*.

6. **The closure audit's denominator contains its own generator's output.**
   `scripts/close-total.py` writes `closure-generated-NN.lino`;
   `scripts/audit-total-closure.py` reads `data/seed/**.lino` including those
   files; `tests/unit/total_closure.rs:47-60` asserts the result is zero.
   Nothing in `src/` or `src/web/` loads them
   (`data/meta/seed-registry.lino:457-459` marks them `unregistered`), so the
   number measures the generator, not the lexicon. *Mechanism:* a real
   grounding gap is silently closed by appending an English-only gloss, and the
   gate reports success.

7. **The store the doctrine names as the substrate is write-only.** Because
   recognition reads `seed::lexicon()` (Rust tables) and answers read
   `seed::localized_response`, every migrated handler still migrates into
   *another Rust table*, one level less specialized but not into the links
   network. `VISION.md:320-326` states this plainly. Until the read path moves,
   "migrated to data" means "migrated to a parsed file", and the meta algorithm
   cannot modify its own behaviour by writing links (B3/B7 depend on this).

8. **The browser is a second implementation with no shared authority.** The
   worker fetches `handler-precedence.lino` and ignores it
   (`src/web/seed-files.js:22`, zero consumers), routes through a 31-entry JS
   array literal, and the parity test checks five invariants by substring
   search. *Mechanism:* every migration must be done twice, and the fixture's
   own header declares full parity "impossible on purpose", so the divergence
   is chartered rather than measured.

9. **Canned bodies survive because nothing forbids them.** No gate asserts that
   an answer is not byte-equal to a seed `body` field. `summary-topics.lino`'s
   three English paragraphs and `research_table.rs`'s three `contains` blocks
   are on the hardcoded-language allowlist (67 rows for `behavior_rules.rs`, 21
   for `research_table.rs`, per the allowlist tally), which records them as debt
   but does not require derivation.

---

## Solution options

### Option A — Continue handler-by-handler batches into `handler-rules.lino`

*Description.* Keep the mechanism #1085 D1.3 and #1095 used: express each
pending handler as a rule set (conditions over seed roles plus a seeded
multilingual response) in `data/seed/handler-rules.lino`, executed by
`src/rule_interpreter.rs:159-168`. Retire its Rust file. Repeat 40 times.

*Architecture sketch.* No new component. `specialized_handlers()`
(`src/solver_dispatch.rs:396-430`) already resolves a precedence name to either
a native function or a rule set; each migration moves one name across the
boundary and deletes the function.

*Pros.* Proven — 12 handlers already moved this way; every step is independently
reviewable and reversible; ratchets fall monotonically by construction; no new
abstraction to justify.

*Cons.* The rule grammar is a recognizer plus a fixed response. It cannot
express `web_requests.rs` (992 lines of retrieval, ranking, provenance),
`numeric_list/codegen.rs` (521 lines of code generation) or
`calendar.rs` (976 lines of date arithmetic). Roughly 9 of the 40 fit; the
remaining 31 would become rule sets whose "response" is a call back into the
same Rust — data-hardcoding, which #699 requirement 2 forbids by name.

*Doctrine fit.* Good for the tail, insufficient for the body. Held-out
paraphrases are easy to add per batch.

*Effort.* ~9 small commits for the fitting handlers; the rest blocked.

*Risk.* Low, and low ceiling. Ends with the ratchet stuck around 31 pending.

### Option B — Replace precedence with scored selection over the doublets store

*Description.* Delete the ordered first-match-wins table entirely. Every method
publishes its applicability as links; selection scores candidates against the
formalized intent read from the store; the highest-evidence method runs.

*Architecture sketch.* `MethodRegistry::from_dispatch` becomes `from_store`;
`handler-precedence.lino` becomes a tiebreak-only `rank` attribute;
`prompt_relevants.rs` disappears because promotion is just a higher score;
`link_store.rs` gains a query surface the solver reads on every turn.

*Pros.* Removes root causes 4, 5, 7 and 8 at once; matches `VISION.md:327-335`
("the meta-language representation is the system"); makes a learned item change
the next answer (#701/#7 criterion) without a Rust edit.

*Cons.* It is a rewrite of the dispatch core with no intermediate green state.
Every one of the 3,542 unit tests routes through `try_dispatch`. Scoring
introduces a new failure mode — a near-tie flips behaviour between releases —
which is hostile to the determinism invariant in `NON-GOALS.md` that
`src/solver.rs:19-22` restates. The 5 pinned precedence invariants
(`tests/unit/specification/routing_precedence.rs:108-152`) encode real bug
fixes (#395, #423, #425, #552) that a score must reproduce exactly.

*Doctrine fit.* The right end state; wrong as a single step ("no deferral" does
not mean "no ordering of work").

*Effort.* Very large; months, with a long red period.

*Risk.* High. A failed attempt leaves the tree worse than the hybrid.

### Option C — Freeze and starve: forbid new handlers, migrate only the largest

*Description.* Make the ratchet strict and set every ceiling to today's measured
value, so no handler may be added. Migrate the top-N files by line count and
leave the rest.

*Architecture sketch.* Gate-only change plus mechanical extraction of
`web_requests.rs`, `calendar.rs`, `behavior_rules.rs`, `mod.rs`,
`software_project.rs`.

*Pros.* Cheapest honest improvement; immediately stops root cause 2; the
numbers fall fastest per commit.

*Cons.* Line count is not specialization. Extracting 992 lines into three files
of 330 satisfies `source_lines_max` and changes nothing. It also leaves B1–B8
with nowhere to land new capability, which the doctrine's "no deferral" clause
forbids: freezing without an alternative landing site means the next capability
either does not land or lands as a `src/agentic_coding/` arm outside the gate.

*Doctrine fit.* Honest but hollow — it optimizes the measure.

*Effort.* Small.

*Risk.* Medium: it makes the metric worse as a proxy while looking better.

### Option D — Five meta-methods plus a strict, single-definition ratchet (selected)

*Description.* Group the 40 pending handlers into five families by *what kind of
operation they are*, implement one meta-method per family that reads its
vocabulary, its conditions, its procedure and its rendering from data, and
migrate each family as a batch with held-out paraphrases in five languages.
Simultaneously: unify the three file counts into one definition, make every
ratchet strict-downward, move the 19 promotion predicates into seed, remove the
four `if name ==` cases, fix the closure denominator, and make the browser
derive its order from the fetched seed. Stage the doublets store in behind the
rule interpreter so the read path moves without a dispatch rewrite.

*Architecture sketch.* Detailed in the next section.

*Pros.* Every pending handler has a named destination, so the ratchet has a
floor of zero rather than 31. Each family shares one evaluator, so the
per-handler cost falls as the batch proceeds. The store migration is a
substitution behind one interface (`rule_interpreter`'s condition evaluation)
rather than a dispatch rewrite, so the tree stays green. The browser change is
additive (derive the order, keep the functions) and makes the parity test a real
assertion instead of a substring search.

*Cons.* Five new interpreters is five new pieces of core; each must be justified
against the minimal-core boundary (`docs/design/minimal-core-boundary.md`) as a
**Generic interpreter**, not smuggled in as a handler. The families are a
judgement call and a misfiled handler costs a re-batch.

*Doctrine fit.* Strongest available. Associative stack only: the operation lives
in data, the interpreter is generic. Generalization over memoization: a family
interpreter must pass paraphrases the retired handler never saw. Ratchets
strictly downward by construction.

*Effort.* Large but decomposable: 5 batches, ~6–10 commits each, plus 12
infrastructure leaves that are each commit-sized.

*Risk.* Medium. Mitigated by keeping every retired handler's prompts in a
behaviour-preservation fixture per batch (#699 requirement 5).

---

## Decision

**Option D is selected.**

Reasons:

1. It is the only option with a defined destination for all 40 pending rows, so
   the ratchet's target is 0 and not an arbitrary stopping point. Option A
   stalls at ~31, Option C at 40.
2. It removes root causes 1, 2, 3, 4, 5, 6, 8 and 9 in commit-sized leaves that
   each leave the tree green, which the doctrine's "no deferral" clause requires
   of a plan spanning releases.
3. It reaches Option B's end state by substitution rather than rewrite: once the
   five interpreters read their conditions through one evaluator, pointing that
   evaluator at the doublets store (root cause 7) is a single change with a
   parity fixture, not a dispatch rewrite.
4. It is consistent with what already worked: the 12 handlers migrated under
   #1085 D1.3 and #1095 are exactly Option D's family M1, already delivered.

Rejected:

- **Option A** — retained *inside* D as family M1. Rejected as the whole plan
  because 31 of 40 handlers do not fit a recognizer-plus-response grammar, and
  forcing them in produces data-hardcoding that #699 requirement 2 forbids.
- **Option B** — rejected as a first step only. Its scoring layer is adopted as
  the stated end state in "Architecture → stage 3", after the read path moves,
  so the five pinned precedence invariants can be replayed against it.
- **Option C** — its gate work is adopted (strict ratchets, one definition); its
  "migrate by line count" strategy is rejected because line count is not
  specialization and extraction would satisfy the gate while changing nothing.

---

## Architecture

### 1. Migration order, grouped by the meta-method that replaces each family

Batches run in this order. Each batch retires its handlers' Rust bodies, moves
their vocabulary and rendering to seed, lowers every ceiling it touches in the
same commit, and ships a held-out paraphrase set in en/ru/hi/zh/es.

**M1 — `rule_interpreter` (exists: `src/rule_interpreter.rs:159-168`,
`data/seed/handler-rules.lino`).** Recognition = conditions over seed roles,
words, substrings, history and route; answer = a seeded multilingual response
record. Already owns 12 handlers. Takes no new pending rows; it is the residue
target for any condition a family interpreter cannot express, and its growth is
itself ratcheted (`rule_count()`, `src/rule_interpreter.rs:219`).

**M2 — `retrieval_method` (new generic interpreter, `src/retrieval_method.rs`,
batch 1, 14 handlers).**
**reconciled: was named `source_lookup`; now `retrieval_method`, because
`SourceLookup` is plan 00 §4.2's contract trait and plan 01 owns its one
implementation. This family *calls* that trait; it is not one, and a family
interpreter named after the contract would be unreadable and would invite a
second implementation (plan 00 §9 R4).**
One retrieval procedure: resolve the subject → choose a source kind from
`data/seed/sources-registry.lino` (13 kinds) → fetch or read cache → render with
provenance in the prompt's language. Absorbs `http_fetch`, `url_navigate`,
`web_search`, `learn_from_source`, `research_comparison_table`,
`research_result_followup`, `summarization`, `brainstorming`, `fact_lookup`,
`concept_lookup`, `network_query`, `source_refresh`, `source_conflict`,
`document_originality_check`.
Retires: `web_requests.rs` (992), `web_search_intent.rs` (872),
`web_requests/live_search.rs` (87), `research_table.rs` (485),
`document_originality.rs` (350), `benchmark_prompts.rs`'s summarization and
brainstorming halves (of 543), and `mod.rs`'s `try_concept_lookup` (:207-390),
`try_network_query` (:417-474), `try_source_refresh` (:786-823),
`try_learn_from_source` (:824-876), `try_source_conflict` (:878-905).
**Depends on plan 01** (`01-live-concept-lookup.md`, B1, live concept lookup
through the sources registry):
M2 *is* the universal-loop consumer plan 01 builds, so this batch lands after
plan 01's `UnknownConceptLookup` implementation and replaces
`src/solver.rs:874-884`'s `policy:no_fetch_capability` stub with a real
retrieval. Delivers #948 items 1 and 2: the three canned summary bodies and the
three `contains(...)` comparison blocks are deleted, and their answers derive
from cached source records with provenance.

**M3 — `procedure_interpreter` (new, batch 2, 8 handlers).** One stored
procedure = ordered steps with typed slots, bound from the prompt's operands and
rendered per target surface. Absorbs `procedural_how_to`,
`procedural_how_to_followup`, `installation_conversion`, `write_script`,
`algorithm`, `software_project`, `document_generation_plan`,
`shell_command_transform`. Retires `src/solver_handler_how.rs` (888),
`solver_handler_how_synthesis.rs` (135), `installation_conversion.rs` (866),
`software_project.rs` (909), `software_project_code.rs` (224),
`document_request.rs` (645), `shell_command_transform.rs` (269), and `mod.rs`'s
`try_write_script` (:691-728) and `try_algorithm` (:729-754).
**Depends on plan 04** (`04-formalization-depth.md`, B4, formalization that
emits needs and stores concepts and procedures):
the step list a procedure executes is the concept-and-procedure record plan 04
produces, so M3 consumes plan 04's output rather than a hand-written IR.
`src/skill_procedure.rs` and `src/solver_handlers/procedure_rules.rs` (112) are
the existing nucleus and become M3's core.

**M4 — `structural_operator` (new, batch 3, 9 handlers).** One typed operation
over parsed operands: parse the operand run, select the operation from the seed
operation vocabulary, apply a native kernel, render from seed. Absorbs
`numeric_list`, `text_manipulation`, `pattern_inference`, `compound_interest`,
`calendar_reasoning`, `calendar_create_event`, `incompatible_units`,
`translation`, `proof_request`. Retires `numeric_list/` (1,374),
`text_manipulation.rs` (823), `text_edit_ops.rs` (328), `pattern_inference.rs`
(359), `compound_interest.rs` (557), `calendar.rs` (976), `calendar_ics.rs`
(198), `user_intent.rs` (236), `src/solver_handler_units.rs` (150), and
`mod.rs`'s `try_translation` (:475-690). The *kernels* that stay native and are
re-declared `justified-native` with written justifications: exact arithmetic
(`src/calculation.rs`), interval enumeration (`src/number_constraints.rs`),
proof search (`src/proof_engine/`), and civil-date arithmetic. Everything else —
which operation a verb names, in which language, and how the result reads — is
seed, as `data/seed/operation-vocabulary.lino` already does for text
manipulation.

**M5 — `dialogue_state_query` (new, batch 4, 9 handlers).** One reader over the
conversation's own links: resolve a reference into the event log / link store and
answer from what is there. Absorbs `conversation_memory`, `coreference`,
`conversation_topic`, `roleplay`, `response_language_followup`,
`software_project_followup`, `execution_failure`, `meta_explanation`,
`how_it_works`. Retires `conversation_memory/` (1,658),
`software_project_followup.rs` (396), `meta_explanation.rs` (324),
`response_language_followup.rs` (122), `benchmark_prompts.rs`'s remaining halves,
and `mod.rs`'s `try_execution_failure` (:755-785). `src/memory_query_language/`
and `src/solver_handlers/conversation_memory/link_query.rs` are the nucleus; this
is the family that most directly becomes a store query in stage 2 below.

**Batch 5 — the unledgered prelude (5 methods).** `diagnostic`, `nl_tool`,
`behavior_rules`, `feature_capability`, `playwright_script` are added to the
ledger as pending rows in leaf 1 (so the pending count rises once, honestly,
from 40 to 45 before it falls) and then migrated: `diagnostic` is a host surface
and becomes `justified-native`; `nl_tool`, `feature_capability` and
`playwright_script` are M1 rule sets; `behavior_rules` (951 lines) is M5.

### 2. Seed schema replacing the hard-coded promotion predicates

New file `data/seed/handler-promotions.lino` (name checked free: no file,
no reference anywhere in `src`, `data`, `scripts`, `tests`). New module
`src/handler_promotion.rs` (free; deliberately *not* `src/promotion.rs`, which
is the unrelated benchmark-gated self-improvement protocol).

The condition grammar is **exactly** the one `data/seed/handler-rules.lino`
already uses and `src/rule_interpreter.rs` already evaluates — `role`,
`role_prefix`, `role_padded`, `role_lead`, `word`, `substring`, `route_exact`,
`history_role`, and the `all` / `any` / `none` combinators, each with an
`of cleaned|trimmed|lowercase` input selector. One evaluator, two callers.

```
handler_promotions
  issue 1138
  purpose "Which handlers a prompt hoists ahead of data/seed/handler-precedence.lino.
           Conditions use the handler-rules grammar and are evaluated by the same
           interpreter, so a promotion is a seed edit, never a Rust edit."
  promotion web_search
    rank 50                       # lower rank wins when two promotions fire
    when
      any
        role web_search_action of cleaned
        all
          role web_search_news_subject of padded
          role web_search_news_recency of padded
        all
          role web_search_records_subject of padded
          role web_search_topic_marker of padded
    because "issue 745: an explicit search act, a news request, or a records request"
  promotion calendar_create_event
    rank 30
    when
      all
        any
          role calendar_day_reference of padded
          shape digit of trimmed
        any
          role calendar_schedule_action of padded
          role calendar_event of padded
          all
            role calendar_date_marker of padded
            any
              role temporal_preposition of padded
              shape time_separator of trimmed
    because "issue 869: a date signal conjoined with a scheduling act"
```

Two grammar additions are required and are the only new primitives:
`shape <digit|time_separator|url|path|quoted>` (a structural predicate over the
input, carrying no natural language) and `of padded` (the space-padded form
`prompt_relevants.rs` builds at `:650` today). `contains("в ")` becomes
`role temporal_preposition of padded`, seeded with the Russian, Hindi, Chinese
and Spanish prepositions; `contains(':')` becomes `shape time_separator`.

`append_prompt_relevants` shrinks to: read the promotions, evaluate each
`when` through `rule_interpreter::conditions_match`, push
`handler:<name>` for every match in `rank` order. The function keeps no handler
names. A test injects a new promotion row into a fixture and asserts routing
changes with no Rust edit (#959 "How to test" clause 4).

The four `if name == "…"` sites in `src/meta_method_dispatch.rs` (`:63`, `:95`,
`:105`, `:122`) and the five-armed prelude `match` (`:189-203`) become registry
attributes on the method record: `response_language_variant`,
`definition_fusion_variant`, `project_lookup_fallback`, `runtime` — read from
`data/seed/method-registry.lino` and applied uniformly by `try_dispatch`. A test
asserts `src/meta_method_dispatch.rs` contains zero occurrences of
`name == "` (#959 "How to test" clause 2).

### 3. Fixing the self-satisfying closure metric

Chosen branch of #959 item 5: **exclude the generator's output from the audit's
denominator and report the true, non-zero gap** — and then delete the files,
because nothing loads them.

1. `scripts/audit-total-closure.py` gains a `GENERATED_PREFIX` exclusion
   (`closure-generated-`) applied to the set of files it reads *as definitions*,
   mirroring `scripts/close-total.py:64`'s own constant. Value tokens from the
   authored seed are still required to resolve; they may no longer resolve to a
   gloss the generator wrote.
2. The audit's `--json` report gains `unresolved_distinct_honest`; the human
   report prints it beside the old number for one release so the change is
   auditable.
3. New ledger `data/meta/closure-audit.lino` (name free) records
   `unresolved_distinct_max <N>` where N is whatever the honest run reports on
   the first commit — an unknown number that the plan does not predict, recorded
   as measured. `tests/unit/total_closure.rs:47-60` changes from `!= 0 → panic`
   to `> ceiling → panic` **and** `< ceiling → panic, lower the reviewed
   ceiling`, the strict rule from
   `scripts/check-minimal-core-boundary.rs:286-290`.
4. The 16 `closure-generated-*.lino` files (17,791 lines, 3,555 English-only
   glosses) are deleted, and `data/meta/seed-registry.lino:457-459`'s
   `unregistered closure-generated-*` block with them.
   `scripts/close-total.py` is retained but re-pointed: it emits a *work list*
   to `data/meta/closure-audit.lino` instead of writing seed files, so the
   generator proposes grounding work and never satisfies the gate itself.
5. The honest gap then falls only by real grounding: each closed token becomes a
   record with `lexeme en|ru|hi|zh|es` in an authored meanings file, which the
   runtime actually loads, and the ceiling drops in that commit.
6. `scripts/regenerate-derived-artifacts.sh:43` ("total closure
   (data/seed/closure-generated-*.lino)") is removed from the regeneration
   chain; `scripts/analyze-merge-conflicts.py:53`'s exclusion pattern loses the
   dead prefix.

### 4. How the doublets store becomes what the solver reads

Three stages, each independently green.

**Stage 1 — project seed and rules into the store at boot (read path unchanged).**
`src/seed_links.rs` (290 lines, already exists and already produces
`LinkRecord`s) is extended to project (a) every meaning/lexeme/surface, (b)
every `handler-rules.lino` rule, and (c) every `handler-promotions.lino`
promotion into `LinkRecord`s written through `src/link_store.rs`. A new gate
asserts the projection is total and content-addressed: every seed record has
exactly one `LinkRecord` and the `stable_id` is reproducible. Behaviour is
unaffected; the store gains the content.

**Stage 2 — one evaluator, two backends, with a parity fixture.**
`src/rule_interpreter.rs`'s condition evaluation is extracted behind a
`ConditionSource` trait with two implementations: `SeedTables` (today's
`seed::lexicon()`) and `LinkStore` (`link_store::query`). A new
`SolverConfig::condition_source` selects between them, defaulting to
`SeedTables`. A parity fixture `data/parity/condition-source.lino` replays every
rule and promotion through both and asserts identical results, in the same
shape as `data/parity/cross-runtime-synthesis.json` does for the browser. When
parity holds across a full release, the default flips to `LinkStore` and
`SeedTables` is deleted. At that point `src/solver.rs` reads the store on every
turn and `VISION.md:320-326` is no longer true — which is exactly what the
revised VISION text must say.

**Stage 3 — selection from the store (Option B's end state).**
`MethodRegistry::from_dispatch` (`src/method_registry.rs`) gains `from_store`;
`handler-precedence.lino` becomes `rank` attributes on the method links; the
five invariants in
`tests/unit/specification/routing_precedence.rs:108-152` are replayed against
the store-derived order. This stage is *planned here and not scheduled here*: it
is only meaningful after all five batches land, and it is tracked as the closing
leaf.

A new ratchet measure, `store_read_share`, records the fraction of routing
decisions in one full unit-suite run that resolved through `LinkStore` rather
than `SeedTables`. It starts at 0 and may only rise — the one measure in this
plan whose strict direction is upward, which the ledger states explicitly so the
rule is not ambiguous.

### 5. How the browser worker derives its handler list from seed

`src/web/seed-files.js:22` already fetches `seed/handler-precedence.lino`; the
change is to consume it.

1. The WASM seed parser gains `handler_precedence_from(text) -> Vec<String>`,
   the exact three-line reader
   `tests/unit/specification/routing_precedence.rs:64-69` already specifies (an
   indented, non-comment row's first whitespace-delimited token), and
   `src/web/seed_loader.js` calls it rather than reimplementing it in JavaScript
   (#952; plan 00 §9 X5).
2. `src/web/worker/formal_ai_worker_20.js` replaces the array *literal* at
   `:577-693` with a **registry object** keyed by the same handler names the
   seed uses — `{ web_search: () => …, numeric_list: () => …, … }` — plus a
   `workerHandlerAliases` map in seed for the names that genuinely differ
   (`tryExactMemoryQuery`, `tryHistorical`, `tryLinkNativeSynthesis`,
   `tryMemoryProgram*` have no Rust precedence row; they become explicit
   `browser_only` rows in `handler-precedence.lino` with a `# browser-only`
   guard note, so the two surfaces share one vocabulary).
3. The loop at `:694` iterates `parseHandlerPrecedence(seed)` and calls the
   registry, with a startup assertion mirroring
   `src/solver_dispatch.rs:399-407`: the precedence must be an exact permutation
   of the registered worker handlers, or the worker throws at load.
4. `tests/fixtures/routing-parity.lino`'s header claim — "Full order-parity is
   impossible on purpose" — is deleted, and the fixture becomes a *reorder*
   test: swap two rows in a fixture copy of the seed, feed it to both the Rust
   `handler_precedence_from` and the worker's call into the same WASM export, and
   assert both dispatch orders change identically. This is #959's "How to test"
   clause 3 and its manual check verbatim.
5. `tests/unit/specification/routing_precedence.rs:207-238`'s substring search
   of the JS source is deleted; the test reads the worker's registry keys
   instead, so a renamed handler fails loudly rather than silently passing a
   `find`.

The async fetch/network handlers that "run in a later phase" keep doing so; they
are marked `phase async` in the seed row, and the parity assertion partitions by
phase rather than pretending the phases do not exist.

### 6. Ratchet mechanics

**One definition of a handler file.** `scripts/check-minimal-core-boundary.rs`'s
`source_files()` (`:150-169`) becomes the single implementation, and both
`scripts/check-debt-ratchet.rs` and
`tests/unit/issue_699_handler_migration.rs:175-196` call it (the script is
already compiled as a module by `tests/unit/issue_918.rs`, per the doc comment
at `:144-149`, so the wiring exists). Definition: every `*.rs` under
`src/solver_handlers/**` except the generated `modules.rs`. The scan root widens
to include the four handler files that today live outside it —
`src/solver_handler_how.rs`, `src/solver_handler_how_synthesis.rs`,
`src/solver_handler_units.rs`, `src/solver_handler_oracle.rs` — so a migration
cannot lower a count by moving a file one directory up.

**Which numbers are ratcheted.** All of these live in
`data/meta/debt-ratchet.lino` as `ceiling` blocks, and the boundary ledger keeps
its four:

| Measure | Start (measured today) | Target | Checker |
| --- | --- | --- | --- |
| `handler_files` | 42 → **46** after the scan root widens (4 files join) | 0 | `check-debt-ratchet.rs` |
| `handler_migration_pending` | 40 → **45** after the 5 prelude rows are added | 0 | `check-debt-ratchet.rs` |
| `try_dispatch_entries` *(new)* | 39 | 0 | `check-debt-ratchet.rs` |
| `promotion_predicates` *(new)* | 19 | 0 | `check-debt-ratchet.rs` |
| `dispatch_name_special_cases` *(new)* | 9 (4 `if name ==` + 5 prelude arms) | 0 | `check-debt-ratchet.rs` |
| `literal_predicates` | 548 (ceiling stale at 549) | 0 | `check-debt-ratchet.rs` |
| `hardcoded_language_rows` | 1286 | 0 | `check-debt-ratchet.rs` |
| `closure_unresolved` *(new)* | honest N, measured on the first run | 0 | `tests/unit/total_closure.rs` |
| `worker_sync_handler_literals` *(new)* | 31 | 0 | `check-debt-ratchet.rs` |
| `store_read_share` *(new, upward)* | 0 | 1.0 | `check-debt-ratchet.rs` |
| `source_file_count_max` | 45 | 0 | `check-minimal-core-boundary.rs` |
| `source_lines_max` | 18466 | 0 | `check-minimal-core-boundary.rs` |
| `outside_core_file_count_max` | 45 | 0 | `check-minimal-core-boundary.rs` |
| `outside_core_lines_max` | 18466 | 0 | `check-minimal-core-boundary.rs` |

Two of those numbers **rise once**, in leaf 1, and the plan says so in advance
rather than hiding it: `handler_files` 42→46 and `handler_migration_pending`
40→45. Both rises are the correction of an undercount, they are recorded in the
ledger's `note` with the reason, and no rise is permitted afterwards.

**The strict-downward rule.** `scripts/check-debt-ratchet.rs` adopts
`check-minimal-core-boundary.rs:284-291`'s exact two-sided comparison: a
measured value above its ceiling fails ("grew from C to A"); a measured value
below its ceiling also fails ("improved from C to A; lower the reviewed
ceiling"). `store_read_share` inverts the comparison and says so in its `how`
field. The `--base <rev>` mode (`scripts/check-debt-ratchet.rs:15-17`) already
forbids a ceiling being higher than at the target branch; that stays, and the
`--base` check is promoted from optional to required in the pull-request gate
`data/meta/ci-gates/check-debt-ratchet.lino`.

**Where the ceilings live.** `tests/unit/issue_699_handler_migration.rs:19-20`'s
`RECORDED_SPECIALIZED_HANDLER_FILES_MAX` and `RECORDED_TRY_DISPATCH_ENTRIES_MAX`
are deleted; the test reads `data/meta/debt-ratchet.lino`. One file holds every
number, so a migration commit touches one ledger and cannot leave a second copy
stale — which is how 37/50 drifted from 36/39 in the first place.

---

## Tests first

Every batch ships its tests before its migration. Paraphrases are **held out**:
the exact strings below must not appear in any file under `data/seed/`, asserted
by the existing no-memorization scan pattern.

### Held-out paraphrase sets (five languages, per family)

New suite `data/benchmarks/handler-family-paraphrases/` with
`{en,ru,hi,zh,es}.lino` members and a
`data/benchmarks/handler-family-paraphrases-suite.lino` header in the shape of
`data/benchmarks/local-path-discovery-suite.lino:1-18`. Twelve prompts per
family per language — 5 families × 5 languages × 12 = **300 cases**,
`minimum_pass_count` set to the honest first measurement.

*M2 `source_lookup` — a subject absent from every seed file:*

- en: `What is a fufloмицин — summarise it in one paragraph with the source.`
  / `Look up the term "zarquon particle" and tell me where you found it.`
- ru: `Что такое «кварковый глюонный суп»? Дай источник.`
  / `Найди определение слова «флогистон» и укажи, откуда оно.`
- hi: `"फ्लॉजिस्टन" शब्द का अर्थ खोजिए और स्रोत बताइए।`
- zh: `查一下"燃素"这个词的意思，并说明来源。`
- es: `¿Qué es el "flogisto"? Resúmelo en un párrafo y cita la fuente.`

*M3 `procedure_interpreter` — a procedure named by an unseen verb:*

- en: `Turn the setup section of this README into a PowerShell script.`
- ru: `Преврати раздел установки из этого README в сценарий PowerShell.`
- hi: `इस README के इंस्टॉल हिस्से को PowerShell स्क्रिप्ट में बदलिए।`
- zh: `把这个 README 的安装部分改写成 PowerShell 脚本。`
- es: `Convierte la sección de instalación de este README en un script de PowerShell.`

*M4 `structural_operator` — an operation stated without its catalogue verb:*

- en: `Put these numbers the other way round and show the Go code: 4 9 1 7`
- ru: `Расположи эти числа в обратном порядке и покажи код на Go: 4 9 1 7`
- hi: `इन संख्याओं को उलटे क्रम में रखें और Go कोड दिखाएँ: 4 9 1 7`
- zh: `把这些数字倒过来排，并给出 Go 代码：4 9 1 7`
- es: `Pon estos números al revés y muestra el código en Go: 4 9 1 7`

*M5 `dialogue_state_query` — a reference resolvable only from the log:*

- en: `Which of the things I asked you about earlier had no source?`
- ru: `О чём из того, что я спрашивал раньше, не нашлось источника?`
- hi: `पहले मैंने जो पूछा था, उनमें से किसका स्रोत नहीं मिला?`
- zh: `我之前问过的那些里，哪一个没有找到来源？`
- es: `De lo que te pregunté antes, ¿para qué no hubo fuente?`

*M1 residue — a policy shape in a language the rule was not written in:*

- es: `¿Puedes ir a la tienda y comprarme pan?` (must reach
  `physical_action_question`, whose rule today carries en/ru/hi/zh only —
  `data/meta/handler-migration-ledger.lino:161-166`).

### Unit, integration and specification tests

| Path | Name | Asserts |
| --- | --- | --- |
| `tests/unit/issue_1138_handler_promotions.rs` *(new, free)* | `a_seeded_promotion_row_changes_routing_with_no_rust_edit` | inject a fixture promotion; routing changes | 
| " | `promotion_conditions_use_the_handler_rules_grammar` | every `when` parses through `rule_interpreter` |
| " | `no_handler_name_appears_in_prompt_relevants` | `src/intent_formalization/prompt_relevants.rs` contains zero `"handler:` literals |
| `tests/unit/issue_1138_uniform_dispatch.rs` *(new)* | `try_dispatch_has_no_name_special_cases` | `src/meta_method_dispatch.rs` contains zero `name == "` |
| " | `prelude_methods_are_ledgered` | every `PRELUDE_METHOD_NAMES` entry has a ledger row |
| `tests/unit/issue_1138_family_migration.rs` *(new)* | `held_out_family_paraphrases_route_to_the_family_interpreter` | the 300-case suite |
| " | `no_answer_is_byte_equal_to_a_seed_body_field` | closes #948 items 1–2 permanently |
| `tests/unit/specification/routing_precedence.rs` *(exists)* | `rust_and_browser_worker_share_specialized_precedence` | rewritten: reorder a fixture row, both surfaces change |
| " *(new case)* | `worker_handler_registry_is_a_permutation_of_the_seed` | replaces the substring search at `:207-238` |
| `tests/unit/issue_699_handler_migration.rs` *(exists)* | `handler_migration_ratchet` | reads ceilings from `data/meta/debt-ratchet.lino`; `RECORDED_*` consts deleted |
| " | `migration_ledger_is_a_complete_live_registry_census` | census widens from the precedence table to precedence + prelude |
| `tests/unit/total_closure.rs` *(exists)* | `seed_has_total_reference_closure` | renamed `seed_closure_gap_only_shrinks`; strict two-sided against `data/meta/closure-audit.lino` |
| `tests/unit/issue_1138_store_read_path.rs` *(new)* | `condition_sources_agree_on_every_rule_and_promotion` | the stage-2 parity fixture |
| `tests/integration/issue_1138_family_behaviour.rs` *(new)* | per batch | every prompt the retired handler answered still answers identically or better (#699 req. 5) |

Registration follows `tests/unit/mod.rs` (`mod issue_699_handler_migration;` at
`:152`, `mod issue_745;` at `:179`); the four new modules are added in
alphabetical position.

### Gates and ratchets, with exact starting and target numbers

New gate files, one per gate, per the convention in
`data/meta/ci-gates/check-debt-ratchet.lino`:

- `data/meta/ci-gates/check-closure-audit.lino` *(name free)*, stage `rust`,
  running the honest closure audit against `data/meta/closure-audit.lino`.
- `data/meta/ci-gates/check-condition-source-parity.lino` *(free)*, stage
  `rust`, running the stage-2 parity fixture.
- `data/meta/ci-gates/check-worker-handler-registry.lino` *(free)*, stage `web`,
  running the reorder test against both surfaces.

`data/meta/ci-gates/check-debt-ratchet.lino`'s `run` line gains `--base` so the
pull-request comparison is mandatory rather than opportunistic.

Starting and target numbers are the table in "Architecture → 6". The two
one-time honest rises (42→46, 40→45) happen in leaf 1 and are recorded in the
ledger's `note` field with the words "corrected undercount", so a reader can see
that the ratchet was repaired rather than relaxed.

---

## Implementation leaves

Ordered; each is independently verifiable and commit-sized.

**Repair the ratchet (no behaviour change).**

- [x] 1. Widen `scripts/check-minimal-core-boundary.rs`'s scan root to include
      `src/solver_handler_how.rs`, `solver_handler_how_synthesis.rs`,
      `solver_handler_units.rs`, `solver_handler_oracle.rs`; add their ledger
      rows; record the new `source_file_count_max` / `source_lines_max`.
- [x] 2. Make `scripts/check-debt-ratchet.rs` strict two-sided (adopt
      `check-minimal-core-boundary.rs:284-291`); lower `literal_predicates`
      549→548 in the same commit; add `--base` to the gate's `run` line.
- [x] 3. Point `check-debt-ratchet.rs` and
      `tests/unit/issue_699_handler_migration.rs:175-196` at
      `check-minimal-core-boundary.rs::source_files`; delete
      `RECORDED_SPECIALIZED_HANDLER_FILES_MAX` and
      `RECORDED_TRY_DISPATCH_ENTRIES_MAX` (`:19-20`); record `handler_files 46`.
- [x] 4. Add `try_dispatch_entries 39`, `promotion_predicates 19`,
      `dispatch_name_special_cases 9`, `worker_sync_handler_literals 31`,
      `store_read_share 0` to `data/meta/debt-ratchet.lino` with `how` strings;
      implement each measure in the checker. **This leaf is the gate every other
      plan's debt measure enters through: plan 03's `authored_ladder_rules`,
      plan 06's `hardcoded_setup_hints` and `hardcoded_execution_environments`,
      plan 08's generic-interpreter declaration and plan 12's
      `non_binary_work_unit_nodes` all land after leaves 1-5, so no measure is
      ever added to an at-or-below checker (plan 00 §9 X6). Capability ratchets
      — `capability-routing-ratchet`, `selection-heuristic-ratchet`,
      `adoption-effect-ratchet`, `obligation-evidence-ratchet`,
      `toolchain-ledger` — keep their own files.**
- [x] 5. Add the 5 prelude methods as `status pending` ledger rows; widen
      `migration_ledger_is_a_complete_live_registry_census` to precedence +
      prelude; record `handler_migration_pending 45`; fix
      `data/meta/handler-migration-ledger.lino:4`'s dead reference to
      `data/meta/kernel-ratchet.lino`.

**Two deviations from leaves 3 and 4, recorded rather than silent (wave I1).**

1. Leaf 3 said `check-debt-ratchet.rs` and
   `tests/unit/issue_699_handler_migration.rs` should *call*
   `check-minimal-core-boundary.rs::source_files`. They cannot: that function
   counts every `*.rs` minus `modules.rs`, which is 49 with the widened scan
   root, while the migration count excludes the three dispatch `mod.rs` files
   and is 46 — the number this plan records. Compiling the boundary script as a
   module inside `check-debt-ratchet.rs` would also give it a second `main` and
   a `walkdir` dependency it does not have. The single definition is therefore
   the **boundary ledger** rather than the boundary function:
   `check-debt-ratchet.rs` counts the ledger's `source` rows whose disposition
   is not `delete`, minus `mod.rs`/`modules.rs`, and `check-minimal-core-
   boundary.rs` proves those rows equal the tree file for file. One census, read
   from data, and no second directory walk — which is the property leaf 3 was
   for.
2. Leaf 4's `store_read_share` is recorded as a **count of store-reading entry
   points** (`from_store(` occurrences in `src/`), not as the fraction 0 → 1.0
   the table above sketches. The ledger's `value` field is an integer, and plan
   00 §6.8 forbids stating a number that has not been run; a fraction of routing
   decisions can only be measured after stage 3 lands. Its `how` field declares
   `direction upward`, and the checker inverts both comparisons for it.

A third mechanism this wave added, which no leaf named: a ceiling may rise only
when the measure's own `note` says `corrected undercount` **and names the value
it corrects**. Without the second half the marker would be a standing bypass —
left in place it would license a second rise — so `check_against_previous`
requires the old number to appear in the note, which it cannot once the
correction has landed. Both permitted rises of this wave (`handler_files`
42 → 46, `handler_migration_pending` 40 → 45) pass through it and are printed by
the checker.

**Make the closure number honest.**

- [x] 6. Exclude `closure-generated-*` from the audit's definition set; emit
      `unresolved_distinct_honest`; print both numbers.
- [x] 7. Create `data/meta/closure-audit.lino` with the measured honest N;
      convert `tests/unit/total_closure.rs` to the strict two-sided rule; add
      `data/meta/ci-gates/check-closure-audit.lino`.
- [x] 8. Delete the 16 `closure-generated-*.lino` files (17,791 lines) and their
      `unregistered` block at `data/meta/seed-registry.lino:457-459`; re-point
      `scripts/close-total.py` to emit a work list; drop the step at
      `scripts/regenerate-derived-artifacts.sh:43`.

**Leaf 7 added one file the leaf did not name (wave I1).** A gate needs
something to run, and the comparison leaf 7 specifies lives in a Rust unit test
that requires the crate to be built. `scripts/check-closure-audit.py` is the
cheap form of exactly that comparison — it reads
`data/meta/closure-audit.lino` and the audit's `--json` and applies the same
strict two-sided rule — so `data/meta/ci-gates/check-closure-audit.lino` can
answer the question in seconds. The unit test remains authoritative; the script
duplicates no resolver logic, only the comparison.

**The first honest number, recorded whatever it is (leaf 6/7).**
`python3 scripts/audit-total-closure.py .` on 2026-09-16 reports 1,067 honestly
defined meanings against 4,622 counting the generated shards, an honest gap of
**3,585** distinct tokens over 9,135 occurrences, and 30 distinct tokens that
resolve to nothing even with the generated shards counted as definitions. The
last 30 are wave T's own seed additions for leaf 9's `shape` / `of padded`
grammar, and leaf 10 grounds them. 3,585 is the size of the grounding work the
generated glosses were standing in for; `data/meta/closure-audit.lino` is where
it shrinks, and the script's exit code keeps its historical meaning so no step
that runs it as a pass/fail verification flips on the day the honest number is
first reported.

**What leaf 8 moved, beyond the files it names (wave I1).** Deleting the 16
shards removed 3,555 of the 4,250 rows
`tests/unit/issue_918.rs::coding_path_has_complete_metadata_and_every_other_gap_is_data`
counted, because a generated record carries `defined-by` and an English
`lexeme` and none of the five reviewed metadata fields, so every one of them was
a gap by construction. That floor now reads **695**, the hand-written gaps the
audit has always been about, and the comment beside it says the drop is the
leaf's doing and not progress. `data/meta/seed-metadata-gaps-*.lino` are
regenerated in the same commit.
`total_closure::generated_closure_shards_are_content_addressed` is deleted with
its subject — it pinned that each meaning sat in the shard its own digest
selects, an invariant about files that no longer exist — and a comment in its
place says so. `data/meta/merge-conflict-policy.lino`'s `seed_total_closure`
artifact keeps its 261 measured conflicts and its `.gitattributes` union driver
(a branch still carrying the deleted files must merge cleanly against this one)
and gains a `retired` note. The honest closure number settles at **3,569** over
6,342 occurrences: the first run measured 3,585, and 16 of those tokens occurred
only inside the generator's own output.

**Move promotion and dispatch specialisation into data.**

- [x] 9. Add `shape` and `of padded` to the rule grammar in
      `src/rule_interpreter.rs`; unit-test each new primitive.
- [ ] 10. Create `data/seed/handler-promotions.lino` with all 19 promotions
      transcribed; add `src/handler_promotion.rs`; register the seed file in
      `data/meta/seed-registry.lino` and regenerate.
- [ ] 11. Rewrite `src/intent_formalization/prompt_relevants.rs` to evaluate the
      seed promotions; delete the 19-entry array and the
      `contains("в ")`/`contains(':')` glue at `:154`; record
      `promotion_predicates 0`.
- [ ] 12. Move the four `if name ==` cases and the five prelude arms in
      `src/meta_method_dispatch.rs` to method-record attributes; record
      `dispatch_name_special_cases 0`.

**Make the browser derive its order from seed.**

- [ ] 13. Expose `handler_precedence_from` through the **WASM seed parser**
      (`src/web/wasm-worker/`) and have `src/web/seed_loader.js` call it, with
      unit tests; add `browser_only` and `phase async` guard notes to
      `data/seed/handler-precedence.lino` for the worker-only entries.
      **reconciled: was "add `parseHandlerPrecedence` to
      `src/web/seed_loader.js`" — a new JavaScript parser. #952 (E100), which
      plan 13 lists as fully closed by this plan, asks for the JS seed parser to
      be *deleted* in favour of the WASM one; adding a second JS parser would
      have made this plan close an issue by doing the opposite of what it asks
      (plan 00 §9 X5).**
- [x] 14. Remove the moved array literal from
      `src/web/worker/formal_ai_worker_dispatch.js`. The ordered bindings,
      argument paths and result guards now live in
      `data/seed/browser-handler-precedence.lino`; `src/web/seed_loader.js`
      projects that document and the worker resolves it generically at startup.
      `worker_sync_handler_literals` remains 0, but its scanner now discovers
      the real owner across all of `src/web/worker/` instead of looking only in
      the obsolete `formal_ai_worker_20.js` location. A seed reorder test and
      `check-worker-handler-registry` gate prove both properties. This is the
      recovered-Claude completion slice; leaf 13's WASM parser unification and
      leaf 15's full native/browser vocabulary parity remain separately visible
      rather than being claimed by this browser-only generalization.
- [ ] 15. Rewrite `tests/unit/specification/routing_precedence.rs:205-260` as a
      reorder test over both surfaces; delete the fixture header's
      "full order-parity is impossible" claim; add
      `data/meta/ci-gates/check-worker-handler-registry.lino`.

**Batch 1 — M2 `retrieval_method` (14 handlers).** *Blocked on plan 01 (`01-live-concept-lookup.md`).*

- [ ] 16. Ship the M2 held-out paraphrase suite (5 languages × 12) and watch it
      fail honestly.
- [ ] 17. Implement `retrieval_method` as a generic interpreter over
      `data/seed/sources-registry.lino`; register it in the boundary ledger as a
      **Generic interpreter**, not a handler.
- [ ] 18. Migrate `concept_lookup`, `network_query`, `source_refresh`,
      `source_conflict`, `learn_from_source` out of
      `src/solver_handlers/mod.rs`; lower every ceiling touched.
- [ ] 19. Migrate `web_search`, `http_fetch`, `url_navigate`; delete
      `web_requests.rs`, `web_search_intent.rs`, `web_requests/live_search.rs`;
      route them through the retrieval plan 01 L11 already installed.
      **reconciled: was "replace `src/solver.rs:874-884`'s
      `policy:no_fetch_capability` with real retrieval". Plan 01 L11 owns that
      deletion and lands long before this batch; two plans deleting one event
      would leave the second with nothing to delete and no test to fail
      (plan 00 §9 X1).**
- [ ] 20. Migrate `summarization` and `brainstorming`; delete the three canned
      bodies at `data/seed/summary-topics.lino:9-18`; add
      `no_answer_is_byte_equal_to_a_seed_body_field` (#948 item 1).
- [ ] 21. Migrate `research_comparison_table`, `research_result_followup`,
      `fact_lookup`, `document_originality_check`; delete
      `research_table.rs:414,430,446` and the English column consts at `:26-33`
      (#948 item 2).

**Batch 2 — M3 `procedure_interpreter` (8 handlers).** *Blocked on plan 04 (`04-formalization-depth.md`).*

- [ ] 22. Ship the M3 paraphrase suite.
- [ ] 23. Implement `procedure_interpreter` over plan 04's stored procedures,
      with `src/skill_procedure.rs` + `procedure_rules.rs` as the nucleus.
- [ ] 24. Migrate `procedural_how_to`, `procedural_how_to_followup`,
      `how_it_works`; delete `src/solver_handler_how*.rs`.
- [ ] 25. Migrate `installation_conversion`, `write_script`,
      `shell_command_transform`, `algorithm`.
- [ ] 26. Migrate `software_project`, `document_generation_plan`.

**Batch 3 — M4 `structural_operator` (9 handlers).**

- [ ] 27. Ship the M4 paraphrase suite.
- [ ] 28. Implement `structural_operator`; re-declare the four native kernels
      `justified-native` in the ledger with written justifications.
- [ ] 29. Migrate `numeric_list`, `text_manipulation`, `pattern_inference`.
- [ ] 30. Migrate `compound_interest`, `calendar_reasoning`,
      `calendar_create_event`, `incompatible_units`.
- [ ] 31. Migrate `translation`, `proof_request`.

**Batch 4 — M5 `dialogue_state_query` (9 handlers).**

- [ ] 32. Ship the M5 paraphrase suite.
- [ ] 33. Implement `dialogue_state_query` over `src/memory_query_language/`.
      **This leaf lands before plan 10 leaf 13, which adds the
      re-render-the-previous-turn operation for the non-understanding class to
      this family rather than as a handler (plan 00 §9 X12).**
- [ ] 34. Migrate `conversation_memory`, `coreference`, `conversation_topic`,
      `roleplay`.
- [ ] 35. Migrate `response_language_followup`, `software_project_followup`,
      `execution_failure`, `meta_explanation`.

**Batch 5 — the prelude, and the seed de-duplication #948 item 4.**

- [ ] 36. Migrate `nl_tool`, `feature_capability`, `playwright_script` to M1;
      declare `diagnostic` `justified-native`; migrate `behavior_rules` to M5.
      **`clarification`'s rule set keeps its five-language role surfaces here;
      plan 10 leaf 13 then deletes the three memorized literals at
      `data/seed/intent-routing.lino:400-402` and the matching `lexeme zh`
      surfaces, and owns that deletion (plan 00 §9 X13).**
- [ ] 37. Replace duplicated answer fields in `data/seed/identity.lino` and
      `data/seed/greetings.lino` with `response_link` indirection; add a seed
      lint failing on a byte-identical answer value appearing more than once.
- [ ] 42. **Widen the terminology lint from route prefixes and module names to
      identifiers and emitted tokens** (#950 / E98, carry-over C30): rename the
      `Graph*` types, extend `scripts/check-terminology.rs` past `/v1/` and
      module paths to declared identifiers and the tokens the engine emits, and
      record the measure in `data/meta/debt-ratchet.lino`.
      **Added by the 2026-09-16 reconciliation: plan 13 lists #950 as fully
      closed by this plan, and no leaf delivered it. An issue in the `Closes`
      list with no leaf is the failure mode #710's audit named.**

**The read path.**

- [ ] 38. Stage 1: project seed, rules and promotions into the link store via
      `src/seed_links.rs`; gate the projection's totality.
- [ ] 39. Stage 2: extract `ConditionSource`; add the `LinkStore` backend, the
      `data/parity/condition-source.lino` fixture and its gate; `store_read_share`
      begins to rise.
- [ ] 40. Stage 2 completion: flip the default to `LinkStore`, delete
      `SeedTables`, record `store_read_share 1.0`, and rewrite
      `VISION.md:320-326`.
- [ ] 41. Stage 3 (closing leaf): `MethodRegistry::from_store`; precedence
      becomes `rank` links; replay the five pinned invariants.

### Capability/family arbitration checkpoint — 2026-09-17

- The source-capability executor exposed a precedence regression in the
  300-case held-out suite: five dialogue/retrieval requests were claimed as
  `compose_from_sources` or `concept_measurement_lookup` before the applicable
  family method could run.
- The correction is an architectural boundary, not five prompt exceptions:
  after the capability table resolves a capability, the shared family catalog
  may preempt it through the same seed-declared `preempts` relation already
  used for registry methods and honest capability gaps.
- `dialogue_state_query` declares precedence over composition and measurement;
  `retrieval_method` declares precedence over composition. The matcher remains
  one generic interpreter and the held-out prompts remain absent from seed.
- The full family module passed 3/3, including all 300 held-out cases, while
  the source-capability module passed 4/4 and the frontier-class suite passed
  8/8, proving that unrelated source composition and measurement still execute.

---

## Docs to update

The exact quoted statements and their replacement text moved to plan 11's
findings table on 2026-09-16, so there is one docs authority and no document
is described in two places (plan 00 §8). This plan's entries are rows
**D239-D248** of
[`11-docs-consistency-audit.md`](11-docs-consistency-audit.md) §"Issue #1138
plan doc replacements", and plan 11's leaves apply them after the ledger rows
they cite exist (plan 00 §7).

| row | document |
| --- | --- |
| D239 | `VISION.md:320-326` |
| D240 | `ROADMAP.md:362` |
| D241 | `ROADMAP.md:421` |
| D242 | `ARCHITECTURE.md:181-185` |
| D243 | `ARCHITECTURE.md:162-165` |
| D244 | `ARCHITECTURE.md:660-673` |
| D245 | `docs/requirements/issue-0559-general-meta-algorithm.md:15-16` |
| D246 | `docs/requirements/issue-0918-*.md` |
| D247 | `docs/requirements-traceability.md` |
| D248 | `data/README.md` |

Any further document this plan's implementation touches is added as a new
plan 11 row, never as a second copy here.

## Risks and open questions

1. **The two honest rises look like a relaxed ratchet.** `handler_files` 42→46
   and `handler_migration_pending` 40→45 in leaf 1 will read, in a diff, exactly
   like the raises the ledger already apologises for at
   `data/meta/handler-migration-ledger.lino:4`. Mitigation: both rises land in a
   commit that changes no `src/` behaviour, with the ledger `note` naming them
   "corrected undercount" and the leaf number; and `--base` becomes mandatory in
   the same commit so no later rise is possible.

2. **A family interpreter can become a handler in disguise.** If
   `retrieval_method` grows a `match subject_kind` with fourteen arms, nothing has
   been migrated. Mitigation: `dispatch_name_special_cases` counts `name == "`
   and `match name` arms across all of `src/`, not only
   `meta_method_dispatch.rs`; and each family interpreter must pass held-out
   paraphrases that the retired handlers never matched, which a disguised
   handler cannot.

3. **Batches 1 and 2 are blocked on plans 01 and 04.** M2 without live lookup
   would be a rename, and M3 without stored procedures would need a
   hand-written IR — the thing #433 asked to remove. If plans 01/04 slip, the
   correct response is to run batches 3, 4 and 5 first (they have no such
   dependency) rather than to build a stub, which would be a bypass.

4. **The honest closure number is unknown until leaf 6 runs.** The plan records
   N as measured and does not predict it. If N is very large, the temptation
   will be to re-admit the generated glosses. It must not be: a large N is a
   true statement about the lexicon and is the input B1/B4 need.

5. **Deleting 17,791 lines of seed will move other gates.**
   `scripts/check-file-size.rs`, `check-disk-usage-policy`, `check-seed-registry`
   and the merge-conflict analyser all touch those paths. Each must be
   re-measured in leaf 8 rather than exempted.

6. **Stage-2 parity may not be total.** A rule whose condition depends on
   ordering inside a Rust table may not have an exact store equivalent. Open
   question: whether `LinkStore` query results need a declared total order in
   `src/link_store/node_addresses.rs`, or whether every condition can be made
   order-independent. To be answered by the first ten rules ported, before the
   backend is declared.

7. **The browser-only handlers have no Rust counterpart.**
   `tryExactMemoryQuery`, `tryHistorical`, `tryLinkNativeSynthesis`,
   `tryMemoryProgram`, `tryMemoryProgramGap`, `tryMemoryWrite` and
   `tryCurrentDialogueFactChecking` exist only in the worker. Adding them to
   `handler-precedence.lino` as `browser_only` rows makes the seed the shared
   vocabulary, but it also puts names in a file the Rust permutation assertion
   reads (`src/solver_dispatch.rs:399-407`). Open question: whether that
   assertion should partition by surface, or whether the Rust side should
   acquire the seven behaviours. Leaf 13 must answer it before leaf 14.

8. **`store_read_share` is the only upward ratchet in the ledger.** A single
   ledger holding measures with two different strict directions is a source of
   reviewer error. Open question: a separate `data/meta/read-path-ratchet.lino`
   may be cleaner. Deferred to leaf 39, when the measure first becomes non-zero.

9. **`literal_predicates` may not reach 0.** A structural predicate such as
   `contains('/')` carries no natural language and is legitimate core. The
   target of 0 assumes every such predicate becomes a `shape` primitive in the
   rule grammar. If a residue survives review, the ledger must record it as a
   named, justified floor with a written reason per predicate — never as a
   silently frozen ceiling.
