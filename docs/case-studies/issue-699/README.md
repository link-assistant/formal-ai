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
