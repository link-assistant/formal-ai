Part of the E127 umbrella (#1183).

## Gap

The request is not formalized into what it demands; phrases are matched:

- "Create a GitHub Actions workflow" → `ci_workflow::requested_in` = `seed::lexicon().mentions_role(ROLE_CI_WORKFLOW_REQUEST, …)` (`rust/src/agentic_coding/ci_workflow.rs`).
- Task identity → alias lists (`data/seed/hello-world-programs.lino`: `aliases "hello world, хелло ворлд"`).
- The literal to print was taken twice from the issue body (once from "print exactly: `Hello, World!`", once from the Expected Output block), so Kotlin printed it twice (#1156).
- Issue requirements the executor never addressed: "Add clear comments", "follows Kotlin best practices", "meaningful name like `test-hello-world.yml`" (it wrote `run.yml`), "Trigger on push to main branch", "CI badge (optional)".

## Requirements

R1. An issue body is parsed into an `ObligationGraph`; every enumerated requirement produces an `ObligationNode` whose subject, action, and object are each anchored to a seed meaning or a Wikidata entity from `data/meta/request-obligation-rules.lino`.

R2. Two or more mentions of the same quoted literal in one issue body — regardless of which clause they appear in — produce exactly one output-literal `ObligationNode`; the output value appears in the graph once.

R3. The executor derives its execution plan from obligation nodes; it does not match role phrases; it discharges each node or reports it as a gap with the node's byte span.

R4. A completed run emits an obligation report via `gap_answer` (already in `rust/src/agentic_coding/task_obligations.rs`) for each node that was not discharged; no obligation is silently omitted.

R5. A requirement clause for which no expectation can be derived produces `ObligationExpectation::Underivable`; it is split or reported as a gap, never discarded (R710-R9).

R6. A translated issue body (same meaning, surface language en/ru/hi/zh/es) yields the same obligation graph modulo the `language` tag.

R7. A paraphrased issue body ("make a small Kotlin app that writes Hello, World! to the console; add CI") produces obligation nodes that are equivalent to those from the canonical issue body.

## Design

### New module: `rust/src/agentic_coding/request_formalization.rs`

```rust
/// Parse an issue body into an obligation graph.
/// Callers replace all ad-hoc role matches with this function.
pub fn formalize_request(text: &str) -> ObligationGraph

pub struct ObligationGraph {
    /// One node per distinct logical obligation; coreference already applied.
    pub nodes: Vec<ObligationNode>,
    /// The language tag extracted from the request ("kotlin", "rust", …).
    pub language: Option<String>,
}

impl ObligationGraph {
    /// The unique output literal this request requires, or None.
    pub fn unique_output_literal(&self) -> Option<&str>
    /// Whether the graph contains an obligation of the given kind.
    pub fn has_obligation(&self, kind: ObligationKind) -> bool
}

/// The semantic kinds the rule set in data/meta/request-obligation-rules.lino defines.
pub enum ObligationKind {
    OutputLiteral,
    CiWorkflow,
    ProgramFile,
    CodeStyle,   // "add clear comments", "best practices"
    FileNaming,  // "meaningful name like …"
    CiBadge,
}

/// Merge nodes that share the same output literal (coreference pass).
fn coreference_pass(nodes: Vec<ObligationNode>) -> Vec<ObligationNode>
```

`ObligationNode` and `ObligationExpectation` are imported from `crate::obligation_ledger` (plan 05, `rust/src/obligation_ledger.rs`); no new ledger type is introduced.

### Modify `rust/src/coding/program_contract.rs`

Replace `explicit_stdout` (lines 25–43, which collects every `print_stdout`-meaning clause independently and joins them with `\n`) with:

```rust
pub fn explicit_stdout(prompt: &str) -> Option<String> {
    crate::agentic_coding::request_formalization::formalize_request(prompt)
        .unique_output_literal()
        .map(str::to_owned)
}
```

This is the direct fix for #1156: coreference collapse happens inside `formalize_request` before the literal is returned.

### Modify `rust/src/agentic_coding/ci_workflow.rs`

Replace `requested_in`:

```rust
pub(super) fn requested_in(objective: &str) -> bool {
    request_formalization::formalize_request(objective)
        .has_obligation(ObligationKind::CiWorkflow)
}
```

The old `seed::lexicon().mentions_role(ROLE_CI_WORKFLOW_REQUEST, …)` becomes an internal implementation detail of `formalize_request`, not a public API decision point.

### Data file: `data/meta/request-obligation-rules.lino`

```lino
request_obligation_rules
  record_type "meta_rule_set"
  source_reader "src/agentic_coding/request_formalization.rs"
  rule output_literal
    when "clause carries print_stdout meaning and contains a quoted literal"
    obligation "output_literal"
    coreference "merge all nodes with identical literal values"
    wikidata "Q131303"
    purpose "The task concept (Hello world program); the anchor is fetched live and cached under data/cache/wikidata/entity/ by the existing Wikidata source."
  rule ci_workflow_request
    when "clause carries ci_workflow_request role"
    obligation "ci_workflow"
    wikidata "Q965769"
    purpose "Continuous integration."
  rule file_naming
    when "clause names a specific output file (regex: meaningful name like …)"
    obligation "file_naming"
    wikidata "Q1144928"
    purpose "Filename."
  pins "Every rule, Wikidata grounding, and source file match is pinned by issue_1166_request_formalization tests."
```

Style follows `data/meta/obligation-evidence-contract.lino` (flat record, `record_type` first, `source_reader` names the owning Rust file) and `data/meta/budget-search-recipe.lino` (`wikidata` + `purpose` per anchor). Anchors verified 2026-09-29: Q131303 = Hello world program, Q965769 = continuous integration, Q1144928 = filename.

### JS/TS parity

Three-roots parity: `request_formalization.rs` is translated with `formal-ai translate --from rust --to js|ts --input rust/src/agentic_coding/request_formalization.rs --write`; the rules stay in the shared data file; `data/parity/cross-runtime-synthesis.json` gains the three Hive Mind issue bodies with their expected obligation graphs, answered identically by all three roots (`issue_1166_multilingual_parity.rs` checks the Rust side against the fixture).

## Tests

Register all three modules in `rust/tests/unit/mod.rs` beside the existing `issue_1138_*` entries:

```rust
mod issue_1166_request_formalization;
mod issue_1166_coreference;
mod issue_1166_multilingual_parity;
```

### `rust/tests/unit/issue_1166_request_formalization.rs`

```rust
// formalize_canonical_hello_world_kotlin_yields_output_literal_node
// formalize_canonical_hello_world_kotlin_yields_ci_workflow_node
// formalize_canonical_hello_world_kotlin_yields_file_naming_node
// formalize_canonical_hello_world_kotlin_output_literal_is_hello_world
// unknown_requirement_clause_yields_underivable_node_not_panic
```

Fixture: `rust/tests/fixtures/issue-1166/hello-world-kotlin-en.txt` — the canonical Kotlin issue body verbatim (create as part of this issue; no such file exists today under `rust/tests/fixtures/`).

### `rust/tests/unit/issue_1166_coreference.rs`

```rust
// two_identical_output_clauses_produce_one_node
// two_output_clauses_with_distinct_literals_produce_two_nodes
// coreference_does_not_merge_filename_with_output_literal
```

These are fully offline; no live fetch. No `FORMAL_AI_LIVE_FETCH` gate needed.

### `rust/tests/unit/issue_1166_multilingual_parity.rs`

```rust
// kotlin_issue_ru_yields_same_obligation_count_as_en
// kotlin_issue_hi_yields_same_output_literal_as_en
// kotlin_issue_zh_yields_same_output_literal_as_en
// kotlin_issue_es_yields_same_obligation_count_as_en
// paraphrase_en_yields_same_obligation_count_as_canonical_en
// three_hive_mind_issue_bodies_yield_same_graph_modulo_language_tag
```

Fixtures: `rust/tests/fixtures/issue-1166/hello-world-kotlin-{ru,hi,zh,es}.txt` and `hello-world-kotlin-paraphrase-en.txt` (new; no multilingual issue-body fixtures exist today — `rust/tests/fixtures/` has `issue-991/`, `issue-1138-b1/`, `issue-1138-b4/`, `coding-discovery/`, `memory/`, `actionlint/`, `routing-parity.lino`; none hold multilingual issue bodies). The three Hive Mind issue bodies for en/Kotlin/Scala/Rust are already used as string literals in `rust/tests/unit/issue_1133_hive_mind_three_runs.rs`; extract them into `rust/tests/fixtures/issue-1166/` so the parity test can read them from disk rather than duplicating them inline.

Run command:

```
RUSTUP_TOOLCHAIN=1.98.1 cargo test --test unit issue_1166_request_formalization
RUSTUP_TOOLCHAIN=1.98.1 cargo test --test unit issue_1166_coreference
RUSTUP_TOOLCHAIN=1.98.1 cargo test --test unit issue_1166_multilingual_parity
```

## Definition of done

- [ ] Requirement shard `docs/requirements/issue-1166-request-formalization.md` with rows R1–R7 in the table format of `docs/requirements/issue-1138-prerequisite-discovery.md` (ID, requirement, status/evidence).
- [ ] Regenerate via `RUSTUP_TOOLCHAIN=1.98.1 rust-script scripts/assemble-requirements.rs --write`; `REQUIREMENTS.md` reflects R1–R7.
- [ ] Traceability rows in `docs/requirements-traceability.md`: one row per R1–R7, each citing the test function name and the version that ships it.
- [ ] Case study `docs/case-studies/issue-1166/` with the Kotlin doubled-output root cause, the coreference fix, and evidence that the same fixture now produces one node.
- [ ] Changelog fragment `changelog.d/<YYYYMMDD>_<HHMMSS>_issue-1166-request-formalization.md` (repository root; front matter `bump: minor`; category `Fixed` for the coreference regression, `Added` for the obligation graph).
- [ ] `RUSTUP_TOOLCHAIN=1.98.1 rust-script scripts/run-ci-gates.rs --stage rust` green with all three test modules registered and passing.
- [ ] `explicit_stdout` in `rust/src/coding/program_contract.rs` delegates to `formalize_request`; the old multi-collect path is gone.
- [ ] `ci_workflow::requested_in` delegates to `formalize_request`; `ROLE_CI_WORKFLOW_REQUEST` mention-match is an internal detail of the rule evaluator, not a call site.

## Depends on / blocks

- **#1163/E128** (internet sources → links): independent — #1166 formalizes the issue body text already in memory, not external URLs.
- **#1164/E129** (code examples → meta language): independent to start; E129's meaning-anchored parts could enrich obligation grounding in a follow-on, but R1–R7 do not require them.
- **#1165/E130** (discovery-first execution): consumes the obligation graph to bind literals and choose steps; this issue lands first.
- **#1167/E132**: independent.
- **#1168/E133** (generated code uses latest versions): independent — version discovery is a separate concern from request formalization; both can land in any order.
- **#1156/E121** (Kotlin doubled output): #1166 directly repairs the root cause (`explicit_stdout` collecting duplicate print clauses); the coreference pass in `formalize_request` is the structural fix #1156 calls for.

