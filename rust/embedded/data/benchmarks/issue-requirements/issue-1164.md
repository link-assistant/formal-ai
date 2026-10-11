Part of the E127 umbrella (#1183).

## Gap

Code found on the internet is either rendered verbatim (Rosetta Code path, `rust/src/coding/rosetta_request.rs`, only for tasks in the seeded alias list) or not used. Nothing parses a retrieved example into the meta language, assigns meanings to its parts, or recomposes those parts. `meta-language` 0.58.2 already ships tree-sitter grammars for C, C#, C++, CSS, Go, HTML, Java, JavaScript, Kotlin, Lua, Pascal, PHP, Python, R, Ruby, Rust, Scala, Swift, TypeScript and more, so parsing is available for every Hello World language Hive Mind tests.

## Requirements

R1164-1. `decompose_code_node` accepts a source string and a language slug; if the slug has no entry in `data/seed/program-cst-grammars.lino` it returns a `DecomposeError::UnknownGrammar` that names the missing slug — it never guesses.

R1164-2. For each language slug registered in `program-cst-grammars.lino` (currently javascript, python, rust, java, csharp, c, cpp, typescript, go, ruby, php) `decompose_code_node` parses the source via `LinkNetwork::parse` and returns at least one `CodePart` for each of: output operation, string literal, and entry point where the language has one (Python and Ruby scripts have none).

R1164-3. The meaning assigned to each `CodePart` is derived from the CST node kinds produced by `LinkNetwork::parse` and from prose links passed by the caller (e.g. "compile with", "run with") — no per-language meaning table is permitted in `code_node_decomposer.rs`.

R1164-4. When prose links supply a build command or run command they are stored as `CodePart::BuildCommand` and `CodePart::RunCommand` with the source URL of the prose retained.

R1164-5. `generalize_examples` aligns two or more `DecomposedCodeNode` values for the same need and returns a `GeneralizedProcedure` whose shared structure is the procedure body and whose language-specific values are `Parameter` entries.

R1164-6. `recompose_for_requirement` binds `ParameterBindings` into a `GeneralizedProcedure` and returns a `CodeRecomposition` that carries the source URL of every contributing `CodePart`.

R1164-7. `adopt_decomposed_procedure` (new entry in `rust/src/coding_research_learning.rs`) promotes a `GeneralizedProcedure` through the existing execution-and-approval gate in `adopt_extracted_procedure` (line 859); it must not bypass the license and approval checks already present there.

R1164-8. A recomposed program for a held-out literal (e.g. `Hello, Formal AI!`) must compile and print that literal in every language whose grammar is registered in `program-cst-grammars.lino` and whose Hello World example was used to build the `GeneralizedProcedure`.

R1164-9. Held-out language: Pascal, which `meta-language` parses (`tree-sitter-pascal`) and for which no program is stored anywhere (not in `data/seed/hello-world-programs.lino`, `data/meta/stdout-program-contracts.lino`, `rust/src/knowledge.rs` `ORACLE_SNAPSHOTS`, or `rust/examples/hello-world/`), gets a correct Hello World from the Free Pascal documentation alone (compiled with `fpc` in the bounded workspace). Lua is not held out: `knowledge.rs:209-214` stores its program. A language with no grammar in the dependency (e.g. Zig) returns `DecomposeError::UnknownGrammar("zig")`, and the answer states that need and links link-foundation/meta-language#195 (CST for all languages).

R1164-10. Every `DecomposedCodeNode`, `GeneralizedProcedure`, and `CodeRecomposition` is expressible as a Links Notation document; the schema is recorded as a data file `data/seed/code-node-decomposition.lino`.

R1164-11. Three-roots parity: `code_node_decomposer.rs` is translated to the JS and TS roots with `formal-ai translate --from rust --to js|ts --input rust/src/coding/code_node_decomposer.rs --write`, and `data/parity/cross-runtime-synthesis.json` gains decomposition cases all three roots answer identically.

## Design

### Existing paths to build from (verified on origin/main)

| Path | Status | Notes |
| --- | --- | --- |
| `rust/src/coding/rosetta_request.rs` | existing | `try_rosetta_code_request_with_client` starts at line 32; the function uses `program_language_by_alias` and `program_task_by_alias` to look up slugs from `data/seed/hello-world-programs.lino`. |
| `rust/src/coding/cst.rs` | existing | Bridge from language slug to `LinkNetwork::parse` via `data/seed/program-cst-grammars.lino`. The registered grammars are javascript, python, rust, java, csharp, c, cpp, typescript, go, ruby, php — no kotlin, scala, or swift entries exist. |
| `rust/src/coding_research_learning.rs` | existing | `research_coding_skill_gap` (line 492), `execute_researched_coding_procedure` (line 618), `adopt_extracted_procedure` (line 859). All three are called only from `rust/tests/unit/issue_919.rs` and `rust/tests/unit/issue_1138_formalization_depth.rs` — no production callers. |
| `rust/src/formalization/procedures.rs` | existing | `ExtractedProcedure`, `ProcedureStep`, `procedure_from_steps`. Entry point for E129's output to reach the existing approval gate. |
| `rust/src/meta_translate.rs` | existing | `js ↔ meta`, `ts ↔ meta`, `js → ts`, `ts → js` via the token-tree pivot. JS/TS parity for this issue routes through this module. |

### New module

**`rust/src/coding/code_node_decomposer.rs`** (new)

```rust
/// A single named part extracted from a parsed code example.
pub struct CodePart {
    pub kind: CodePartKind,
    pub source_text: String,
    pub cst_node_kind: String,
    pub source_url: String,
}

pub enum CodePartKind {
    EntryPoint,
    OutputOperation,
    StringLiteral,
    Import,
    BuildCommand,
    RunCommand,
    TestAssertion,
}

pub struct DecomposedCodeNode {
    pub language_slug: String,
    pub parts: Vec<CodePart>,
}

pub struct Parameter {
    pub name: String,
    pub per_language: std::collections::BTreeMap<String, String>,
}

pub struct GeneralizedProcedure {
    pub id: String,
    pub shared_structure: Vec<CodePartKind>,
    pub parameters: Vec<Parameter>,
    pub source_urls: Vec<String>,
}

pub struct ParameterBindings(pub std::collections::BTreeMap<String, String>);

pub struct CodeRecomposition {
    pub language_slug: String,
    pub source: String,
    pub part_source_urls: Vec<String>,
}

#[derive(Debug)]
pub enum DecomposeError {
    UnknownGrammar(String),
    ParseFailed(String),
}

pub fn decompose_code_node(
    source: &str,
    language_slug: &str,
    prose_links: &[ProseLink],
) -> Result<DecomposedCodeNode, DecomposeError>;

pub fn generalize_examples(
    examples: &[DecomposedCodeNode],
) -> GeneralizedProcedure;

pub fn recompose_for_requirement(
    template: &GeneralizedProcedure,
    bindings: &ParameterBindings,
    target_language: &str,
) -> Result<CodeRecomposition, DecomposeError>;
```

### Extend `rust/src/coding_research_learning.rs`

Add `adopt_decomposed_procedure` after line 859, delegating license/approval checks to the existing gate in `adopt_extracted_procedure`. Signature:

```rust
pub fn adopt_decomposed_procedure(
    procedure: &GeneralizedProcedure,
    execution: Option<&CodingResearchExecution>,
    approval: &CodingResearchApproval,
) -> Result<ResearchedCodingProcedure, CodingResearchError>;
```

### Data file

**`data/seed/code-node-decomposition.lino`** (new) — schema and one example snippet:

```
code_node_decomposition_schema
  description "Decomposed code node parts and generalized procedures for Hello World examples."
  version "1"
  code_part_kind "entry_point"
    label "Entry point"
    cst_node_kinds ("function_item" "main_method_declaration" "function_declaration")
  code_part_kind "output_operation"
    label "Output operation"
    cst_node_kinds ("call_expression" "macro_invocation" "expression_statement")
  code_part_kind "string_literal"
    label "String literal"
    cst_node_kinds ("string_literal" "interpreted_string_literal")
generalized_hello_world
  id "generalized_hello_world_v1"
  shared_structure ("entry_point" "output_operation" "string_literal")
  parameter "print_call"
    rust   "println!(\"{}\", {value});"
    go     "fmt.Println({value})"
    kotlin "println({value})"
  parameter "output_literal"
    bound_from "the formalized requirement (#1166), never from the example"
  source_url "https://rosettacode.org/wiki/Hello_world/Text"
  source_url "https://doc.rust-lang.org/book/ch01-02-hello-world.html"
```

### Grammar registration

Kotlin, Scala, Swift and R entries in `data/seed/program-cst-grammars.lino` are added by #1167 (E132), generated from the dependency's grammar crates. This issue depends on that for those four languages; the eleven registered languages work without it.

### JS/TS parity

Per R1164-11: translated JS/TS roots plus parity fixture cases; no hand-written mirror.

## Tests

### Test modules (register in `rust/tests/unit/mod.rs`)

- `mod issue_1164_decompose;` → `rust/tests/unit/issue_1164_decompose.rs` (new)
- `mod issue_1164_recompose;` → `rust/tests/unit/issue_1164_recompose.rs` (new)

### Fixtures

Create `rust/tests/fixtures/coding-discovery/issue-1164/` (new directory) with captured Hello World pages for Kotlin (`kotlin-hello-world.html`), Scala (`scala-hello-world.html`), Go (`go-hello-world.html`), Swift (`swift-hello-world.html`), and Rust (`rust-hello-world.html`). No such language-specific Hello World fixtures exist today under `rust/tests/fixtures/coding-discovery/`.

### Offline tests (`issue_1164_decompose.rs`)

```
fn decompose_rust_hello_world_produces_entry_output_literal()
fn decompose_go_hello_world_produces_entry_output_literal()
fn unknown_grammar_slug_returns_unknown_grammar_error()
fn generalize_rust_and_go_produces_shared_structure_with_literal_parameter()
```

### Offline tests (`issue_1164_recompose.rs`)

```
fn recompose_with_formal_ai_literal_yields_rust_source()
fn recompose_with_formal_ai_literal_yields_go_source()
fn recomposed_parts_carry_all_source_urls()
```

### Online gate (gated by `live_fetch_requested()` — copy this exact idiom from `rust/tests/unit/issue_991_how_to_synthesis.rs`)

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

Online test functions: `live_held_out_pascal_hello_world_from_official_docs` — fetches the Free Pascal documentation page, decomposes its example, recomposes with `Hello, Formal AI!`, compiles with `fpc` and runs it in the bounded workspace and asserts the output; `zig_without_grammar_reports_the_need` (offline) asserts `DecomposeError::UnknownGrammar("zig")` and that the rendered answer links meta-language#195.

### Run commands

```
RUSTUP_TOOLCHAIN=1.98.1 cargo test --test unit issue_1164_decompose
RUSTUP_TOOLCHAIN=1.98.1 cargo test --test unit issue_1164_recompose
FORMAL_AI_LIVE_FETCH=1 RUSTUP_TOOLCHAIN=1.98.1 cargo test --test unit issue_1164_decompose live_held_out_pascal
```

## Definition of done

- [ ] `docs/requirements/issue-1164-code-node-decomposition.md` exists with a table using columns `| ID | Requirement | Status / evidence |` (matching the format of `docs/requirements/issue-1138-prerequisite-discovery.md`) and one row per R1164-N requirement.
- [ ] `REQUIREMENTS.md` regenerated: `RUSTUP_TOOLCHAIN=1.98.1 rust-script scripts/assemble-requirements.rs --write`.
- [ ] One row per R1164-N in `docs/requirements-traceability.md` with columns `| ID | Shard | Delivered | Automated test | Manual confirmation |`.
- [ ] Case study data in `docs/case-studies/issue-1164/` (at minimum: one decomposed example and the generalised procedure in Links Notation).
- [ ] Changelog fragment `changelog.d/YYYYMMDD_HHMMSS_issue-1164-code-node-decomposition.md` (repository root; front matter `bump: minor`, section `### Added`).
- [ ] CI gate green: `RUSTUP_TOOLCHAIN=1.98.1 rust-script scripts/run-ci-gates.rs --stage rust`.

## Depends on / blocks

- Depends on #1163 (E128): code blocks arrive as formalized page nodes.
- Depends on #1167 (E132): Kotlin/Scala/Swift/R grammar entries and the render entry point `compose_and_validate` used by `recompose_for_requirement`.
- Blocks #1165 (E130): the production path recomposes through `recompose_for_requirement`.
- Blocks the replay rungs of #1170 (E135) that ask for programs in languages without templates.

