# Issue 699: specialized-handler migration, batch 1

Issue [#699](https://github.com/link-assistant/formal-ai/issues/699) corrects
the scope claim made when #559 closed. The registry is the sole *route
authority*, but most executable methods are still intent-specific Rust
handlers. This PR is deliberately the first generality-first batch; it does not
close #699.

## Reproducible census

At `main` v0.312.1 the live registry contained 55 methods, 51 of whose function
symbols began `try_`, and `src/solver_handlers/` contained 39 Rust files. The
batch moves `number_riddle.rs` to the language-neutral
`src/number_constraints.rs` primitive and renames its registry function, so the
new ratchet ceilings are 38 files and 50 `try_*` entries.

The 38 remaining files are:

```text
agent_workspace.rs                 behavior_rule_followups.rs
behavior_rule_matching.rs         behavior_rules.rs
benchmark_prompts.rs              calculator_rate.rs
calendar.rs                       calendar_ics.rs
compound_interest.rs              curated_project_fetch.rs
definition_merge.rs               document_originality.rs
document_request.rs               fact_checking.rs
feature_capability.rs             github_repository_traffic.rs
installation_conversion.rs       meta_explanation.rs
mod.rs                            natural_language_tools.rs
playwright_script.rs              procedure_rules.rs
program_blueprint.rs              program_synthesis.rs
research_table.rs                 response_language_followup.rs
self_awareness.rs                 shell_command_transform.rs
software_project.rs               software_project_code.rs
software_project_followup.rs      task_decomposition.rs
text_edit_ops.rs                  text_manipulation.rs
user_intent.rs                    web_requests.rs
web_search_intent.rs              world_state.rs
```

`data/meta/handler-migration-ledger.lino` is the machine-checked full
55-method census. It lists one migrated method, two explicitly justified native
methods, and 52 pending methods in the exact live precedence order. The
`migration_ledger_is_a_complete_live_registry_census` test rejects omissions,
duplicates, reorderings, or an understated pending count.

## Why number constraints are the first batch

The issue #433 audit identified `number_riddle` as a fixed-enumeration
recognizer. Its Rust function mixed two different responsibilities:

1. English/Russian phrase arrays decided whether a prompt was an interval
   constraint and where its bounds were.
2. Language-neutral code enumerated integer solutions and invoked the proof
   engine.

The first responsibility is now link data:
`data/seed/meanings-number-constraints.lino` declares entity, query, hidden
value, and strict/inclusive lower/upper relation roles in en/ru/hi/zh. The
second is a justified primitive in `src/number_constraints.rs`. Its extractor
handles both relation-before-number and number-before-relation grammar, plus
CJK relations adjacent to digits. Those are structural arrangements, not
language branches.

The held-out suite uses wording absent from the original recognizer:
`exceeds/below`, `превышает/не достигает`, Hindi operand-before-relation
phrasing, and attached Chinese comparisons. All four resolve the unique integer
5. The original issue #403 Russian prompt remains covered by its existing
behavior-preservation test.

## Batch 2: `who_is` and `definition_merge`

Both rows were substitution methods holding memorized specifics.

`who_is` carried a fixed table of eight people, each with three hand-written
misspellings. The table is gone. The only native part left is nearest-surface
search under a length-scaled edit budget, and its candidates come from the
entity registry, concept terms and aliases, and fact labels — so a corrected
name is derived from what the system remembers, never from a stored typo. The
held-out suite spells four names the retired table never listed
(`ada lovlace`, `alan turring`, `альберт эйнштеин`, `निकोला टेस्ल`), and a
guard asserts `data/seed/entity-names.lino` stores no misspelling at all, so a
later edit cannot quietly restore memoization.

`definition_merge` kept a host-to-language mapping and rendered labels in Rust.
The mapping moved into the language-detection rules and every label is now a
seed response with en/ru/hi/zh coverage; deduplication, ordering and rendering
stay native because they are language-neutral list operations.

## Batch 3: `program_synthesis` fails with a named skill gap

Requirement 3 has two halves, and only one of them was missing. Synthesis
already reaches outside the curated catalogue: a request the verified templates
cannot serve is retried against the composite blueprint recipes
(`src/coding/blueprint.rs`), the cached coding oracle
(`src/solver_handler_oracle.rs`), and the seed idiom composer
(`data/seed/coding-idioms.lino`), each of which derives code the catalogue
never stored.

What was missing is the honest failure. When every route missed, the engine
answered by reciting what it happens to hold — "Supported tasks: hello_world,
count_to_three, …". Under the issue's generality-first rule that is doubly
wrong: it advertises memorized specifics as the capability surface, and it
names no gap anyone can act on.

`src/program_skill_gap.rs` replaces it with the `skill_gap` protocol already
established for procedure compilation (issue #674): a stable English gap
*identity* appended to the evidence trail as a `skill_gap` event, and a
localized reply rendered from
`data/seed/multilingual-responses-synthesis.lino` that names the gap and the
routes that were tried, in en/ru/hi/zh. The browser worker was migrated in
lockstep (`programSkillGapName`/`programSkillGapAnswer`), verified by
`experiments/check_worker_program_skill_gap.mjs`. An anti-recitation guard in
the test suite scans both engines for the retired catalogue sentence.

## Migration plan

Later one-batch PRs should select pending rows by general mechanism rather than
by file proximity:

1. substitution and rewrite methods (`text_manipulation`, `translation`,
   `definition_merge`);
2. compiled procedures and skills (`procedural_how_to`, installation and
   software-project methods);
3. link-store queries (`fact_lookup`, conversation/coreference, source
   conflict/refresh);
4. policy and clarification projections;
5. network and host boundaries, separating their data-driven recognition from
   a small justified-native I/O set.

Every batch must first lower both maxima in the ledger, add held-out
paraphrases, and preserve the affected existing suite. A row cannot become
`migrated` merely because its precedence name is seed data.

The `write_program` meta-builder dependency is no longer the v0.285 catalogue
gap described by the issue: R289 now uses the knowledge oracle for
outside-catalogue languages, and #674 compiles arbitrary procedures with an
honest `skill_gap`. This batch does not overstate either slice as arbitrary
program synthesis.

## Research notes

Production rule systems separate facts, rule memory, matching, conflict
resolution, and firing. Drools documents that working-memory/agenda split and
both forward and backward chaining; it supports this migration's separation of
link data from a small execution host rather than merely relocating a phrase
table into another Rust table:
<https://docs.jboss.org/drools/release/latest/drools-docs/drools/rule-engine/index.html>.

Forgy's RETE paper describes sharing partial matches across productions, the
important generality lesson for later batches: compile reusable relation
structure once instead of scanning every intent recognizer independently:
<https://doi.org/10.1016/0004-3702(82)90020-0>.

Maude's reflection model reifies rewrite theories as terms and executes them
through a universal reflective theory. That is the stronger destination for
substitution-heavy pending rows: rules should be inspectable data consumed by a
general interpreter, not a differently named Rust dispatch table:
<https://maude.cs.illinois.edu/maude1/manual/maude-manual-html/maude-manual_20.html>.

No third-party code or text was copied from these sources.

## Reproduction

```bash
cargo test --test unit issue_699_handler_migration -- --nocapture
cargo test --test unit issue_403 -- --nocapture
```

The smallest self-coding leaf is replayed with:

```bash
cargo build --release --bin formal-ai
experiments/issue-699-agent-cli/run.sh
```

That harness boots the local server, drives it through the real Agent CLI, and
captures the raw stream plus the migration verification record under
`agent-cli-evidence/`.

The first whole-issue `solve` attempt remains visible in PR #877: its configured
service incorrectly tried to write the `./examples` directory. The focused
before/after test logs are retained locally under `experiments/issue-699-agent-cli/`.
