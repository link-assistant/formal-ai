Part of the E127 umbrella (#1183).

## Gap

The machinery for coding by discovery is not on the path users and Hive Mind hit:

- `research_coding_skill_gap`, `execute_researched_coding_procedure`, `adopt_extracted_procedure` (`rust/src/coding_research_learning.rs:492,618,859`) are called only from `tests/unit/issue_919.rs` and `tests/unit/issue_1138_formalization_depth.rs`.
- `discover_and_compose` (`rust/src/coding/synthesis_runtime.rs:218`) is reached only from `program_synthesis` (`rust/src/solver_handlers/program_synthesis.rs:49`), which handles function-shaped tasks, not repository work items.
- A recognised `write_program` request takes precedence over everything (`rust/src/solver.rs:549–554`; lines 549–553 are the explanatory comment, line 554 is `let is_concrete_write_program = matches!(rule, SelectedRule::WriteProgram(_));`) and is answered from `data/meta/stdout-program-contracts.lino` / `data/seed/hello-world-programs.lino` templates; the work-item executor calls `UniversalSolver::default().solve(objective)` at `rust/src/agentic_coding/general_execution.rs:219` and takes the template's `execution_recipe` (used at line 224).
- A third store of complete programs lives in Rust code: `rust/src/knowledge.rs:163-233` `ORACLE_SNAPSHOTS` (Hello World in Kotlin, Swift, PHP, Bash, Lua, Haskell and a Kotlin factorial, attributed to helloworldcollection.de / Rosetta Code but compiled in), answered through `CodingOracle::lookup` from `rust/src/solver_handler_oracle.rs:76`; `implementation_language.rs:92` also uses `CodingOracle::knows_language` to decide which languages are supported.
- Plan 02 leaf L15 ("Delete the 7 algorithm-shaped templates from `data/seed/coding-discovery-runtime.lino`") is unchecked.

## Requirements

R1. When the solver selects `SelectedRule::WriteProgram` and no cached `CodingResearchGap` procedure exists for the requested language and task, `research_coding_skill_gap` is called on the miss path; the solver returns a discovery-backed answer, not a template literal.

R2. When `plan_work_item_execution` (`rust/src/agentic_coding/general_execution.rs:210`) calls `UniversalSolver::default().solve(objective)` and the result's `execution_recipe` originates from a seed template, the executor calls `research_coding_skill_gap` to replace it; `planned_not_executed` is never emitted for a recognised language and task pair.

R3. A stored procedure is accepted as a cache entry only when its `CodingResearchGap` records a `rediscovery_query` and a `rediscovery_source`; a procedure without those fields is not stored.

R4. `data/seed/hello-world-programs.lino` and `ORACLE_SNAPSHOTS` (`rust/src/knowledge.rs:163-233`) are deleted; every language/task they covered is reproduced by a cache miss that calls `research_coding_skill_gap`. `CodingOracle::knows_language` is replaced by "a grammar exists and discovery found a procedure".

R5. The `entry`, `operation`, `ci_setup`, and `documented_source` fields are removed from `data/meta/stdout-program-contracts.lino`; their content is produced by discovery and stored in the new `data/meta/coding-procedure-cache.lino` format with a `rediscovery_query` per row.

R6. The `check_command` constants in `rust/src/coding/catalog/languages.rs` are deleted once each language's check command is reproduced from a cached-and-verified procedure; no language row hard-codes a compile command.

R7. A test deletes the entire `data/meta/coding-procedure-cache.lino` cache and proves that `research_coding_skill_gap` recreates a procedure with the same `content_id` (FNV-1a of the program source) for Kotlin, Scala, Rust, Python, Go, and a language with no stored program anywhere (Pascal: grammar in `meta-language`, compiler `fpc`).

R8. The `no-memorization` gate (`rust/tests/unit/coding_discovery/no_memorization.rs`) is extended so that any per-language Hello World program string appearing verbatim in `data/` fails it.

R9. The discovery path serves the Hive Mind work-item executor; E121 (#1156) doubled output is eliminated when the obligation literal is bound once from the formalized requirement via the same miss path.

R10. Three-roots parity: `rust/src/coding/procedure_cache.rs` is translated to the JS and TS roots with `formal-ai translate --from rust --to js|ts --input rust/src/coding/procedure_cache.rs --write`, and `data/parity/cross-runtime-synthesis.json` gains cache hit/miss cases all three roots answer identically.

## Design

### New module: `rust/src/coding/procedure_cache.rs`

```rust
pub struct RediscoverableRecipe {
    pub language: String,
    pub task: String,
    pub rediscovery_query: String,
    pub rediscovery_source: String,
    pub entry: String,
    pub verified_output: String,
    pub content_id: u64,   // FNV-1a of the program source bytes
}

pub struct ProcedureCache { /* backed by data/meta/coding-procedure-cache.lino */ }

impl ProcedureCache {
    pub fn load() -> Self;
    pub fn lookup(&self, language: &str, task: &str) -> Option<&RediscoverableRecipe>;
    pub fn store(&mut self, recipe: RediscoverableRecipe);
    pub fn delete_all(&mut self);  // used by the rediscovery test (R7)
}
```

JS/TS roots per R10 (translated files plus parity cases).

### Modify `rust/src/solver.rs`

In the `SelectedRule::WriteProgram` branch (the guard is at `solver.rs:554`), after the template produces an `ExecutionRecipe`: check `ProcedureCache::load().lookup(language, task)`. On a miss, call `research_coding_skill_gap` with a `CachedSourceClient` holding the `FORMAL_AI_LIVE_FETCH` flag; store the result via `ProcedureCache::store`; return the discovery-backed recipe. On a hit, return the cached `RediscoverableRecipe` directly.

### Modify `rust/src/agentic_coding/general_execution.rs`

After line 219 (`UniversalSolver::default().solve(objective)`), add: if `answer.execution_recipe` is template-backed and `ProcedureCache::load().lookup(language, task)` misses, call `research_coding_skill_gap`; replace `answer.execution_recipe` with the discovered recipe. This eliminates the `planned_not_executed` path for recognised pairs (R2).

### New data file: `data/meta/coding-procedure-cache.lino`

```
coding_procedure_cache
  version 1
  purpose "Rediscovery-verified procedure per language×task. Entry is a cache; a test deletes it and proves research_coding_skill_gap recreates every row with the same content_id."

procedure_hello_world_rust
  language rust
  task hello_world
  rediscovery_query "Rust println macro print text newline"
  rediscovery_source "https://doc.rust-lang.org/std/macro.println.html"
  entry 'fn main() {\n    println!("{text}");\n}\n'
  verified_output "Hello, world!"
  content_id "…"   (FNV-1a of `entry`, computed by `procedure_cache.rs` on store)
```

Fields `entry`/`operation`/`ci_setup`/`documented_source` are moved here from `data/meta/stdout-program-contracts.lino` (which retains only `purpose`, `instruction`, `response`, and per-language `source`/`comment`/`unicode_escape`).

### Deletions (after R7 test passes)

- `data/seed/hello-world-programs.lino` — full file.
- `entry`, `operation`, `ci_setup`, `documented_source` fields from every language block in `data/meta/stdout-program-contracts.lino`.
- `check_command: Some(…)` constants in `rust/src/coding/catalog/languages.rs` — all rows.
- `ORACLE_SNAPSHOTS` and its lookups in `rust/src/knowledge.rs`, and `rust/src/solver_handler_oracle.rs` (or its reduction to a cache reader).

### JS/TS parity

Per R10.

## Tests

All test modules follow the naming convention `issue_1165_<topic>.rs` and are registered in `rust/tests/unit/mod.rs`.

**`rust/tests/unit/issue_1165_discovery_on_production_path.rs`**

```
mod issue_1165_discovery_on_production_path;
```

Test functions:
- `solver_write_program_miss_calls_research_coding_skill_gap` — assert the solver's `WriteProgram` branch invokes `research_coding_skill_gap` when the cache is empty; verify the answer is discovery-backed (no template literal in `execution_recipe.source`).
- `agentic_executor_miss_calls_research_coding_skill_gap` — call `plan_work_item_execution` with a Kotlin hello-world objective after clearing the cache; assert the returned plan carries a `rediscovery_source` field and `planned_not_executed` is absent.
- `solver_write_program_hit_returns_cache` — populate the cache for Rust/hello_world; assert the solver returns the cached recipe without calling `research_coding_skill_gap`.

Online gate (same idiom as `rust/tests/unit/issue_991_how_to_synthesis.rs:53–62`):

```rust
fn live_fetch_requested() -> bool {
    matches!(
        std::env::var("FORMAL_AI_LIVE_FETCH")
            .unwrap_or_default()
            .trim()
            .to_ascii_lowercase()
            .as_str(),
        "1" | "true" | "yes" | "on"
    )
}
```

Wrap every test that calls a real source with `if !live_fetch_requested() { return; }`.

**`rust/tests/unit/issue_1165_cache_rediscovery.rs`**

```
mod issue_1165_cache_rediscovery;
```

Test functions:
- `delete_cache_and_rediscover_six_languages` (online-gated) — call `ProcedureCache::delete_all`, then run `research_coding_skill_gap` for Kotlin, Scala, Rust, Python, Go, Pascal; assert each reproduced procedure's `content_id` equals the one recorded before deletion.
- `no_template_file_needed_after_rediscovery` — assert `data/seed/hello-world-programs.lino` is absent from the repository when this test runs (catches accidental re-addition).
- `no_memorization_rejects_program_literal_in_data` — confirm the extended `no_memorization` gate (`rust/tests/unit/coding_discovery/no_memorization.rs`) returns an error when a Hello World literal is injected into a `data/` file.

Run commands:
```
RUSTUP_TOOLCHAIN=1.98.1 cargo test --test unit issue_1165_discovery_on_production_path
RUSTUP_TOOLCHAIN=1.98.1 cargo test --test unit issue_1165_cache_rediscovery
FORMAL_AI_LIVE_FETCH=1 RUSTUP_TOOLCHAIN=1.98.1 cargo test --test unit issue_1165_cache_rediscovery::delete_cache_and_rediscover_six_languages
```

## Definition of done

- [ ] Requirement shard `docs/requirements/issue-1165-discovery-production-path.md` created in the table format of `docs/requirements/issue-1138-prerequisite-discovery.md` (ID column `R1165-*`, Requirement column, Status/evidence column).
- [ ] Regenerated via `RUSTUP_TOOLCHAIN=1.98.1 rust-script scripts/assemble-requirements.rs --write`.
- [ ] Traceability rows for R1165-1 through R1165-10 added to `docs/requirements-traceability.md`.
- [ ] Case-study data in `docs/case-studies/issue-1165/` (at minimum: a `plans/` entry describing the production-path wiring, and a `ci-evidence/` capture for the Pascal rediscovery run).
- [ ] Changelog fragment in `changelog.d/YYYYMMDD_HHMMSS_issue-1165-discovery-production-path.md` with `bump: minor` and a `### Changed` entry ("Discovery is now on the production path for every `write_program` request; seed templates are a deletable, rediscoverable cache").
- [ ] `data/seed/hello-world-programs.lino` absent from the tree.
- [ ] `entry`/`operation`/`ci_setup`/`documented_source` absent from `data/meta/stdout-program-contracts.lino`.
- [ ] `check_command: Some(…)` absent from `rust/src/coding/catalog/languages.rs`.
- [ ] `ORACLE_SNAPSHOTS` absent from `rust/src/knowledge.rs`.
- [ ] `cargo test --test unit issue_1165_discovery_on_production_path` green without `FORMAL_AI_LIVE_FETCH`.
- [ ] `FORMAL_AI_LIVE_FETCH=1 cargo test --test unit issue_1165_cache_rediscovery::delete_cache_and_rediscover_six_languages` green (Kotlin, Scala, Rust, Python, Go, Pascal all pass).
- [ ] Extended `no_memorization` gate green.
- [ ] `RUSTUP_TOOLCHAIN=1.98.1 rust-script scripts/run-ci-gates.rs --stage rust` green.

## Depends on / blocks

- Depends on #1163 (E128): the miss path fetches and formalizes pages.
- Depends on #1164 (E129): examples are decomposed and recomposed into the procedure.
- Depends on #1167 (E132): recomposed programs are rendered from networks.
- Uses #1166 (E131): the output literal is bound from the formalized obligations (this removes the doubled output of #1156 for good).
- Coordinates with #1168 (E133): both change `data/meta/stdout-program-contracts.lino` (#1168 removes version literals, this issue removes the program strings); whichever lands second rebases.
- Blocks #1170 (E135) and umbrella acceptance item 1 of #1183 (templates deleted, unseen languages from discovery alone).

