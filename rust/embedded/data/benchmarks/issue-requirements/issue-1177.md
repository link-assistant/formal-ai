Part of the E127 umbrella (#1183); parity classes **code generation, code explanation, debugging, regex, SQL**.

## Evidence

- 0.347.0: Python "second largest distinct number" → "I cannot write this program: no synthesis route reaches this request"; code explanation → canned web-search paragraph; debugging → "terminal command"; regex → TypeScript extension plan; SQL → canned web-search paragraph.
- Upstream: HumanEval 14/164, MBPP 49/500 (60/500 online), SWE-bench Lite 0/1.

- Second batch: code review, refactoring (promises → async/await), shell command ("find .log files larger than 10 MB under /var") and JSON→YAML → canned web-search paragraph; test generation ("pytest tests for `is_palindrome(s)` ignoring case and spaces") → "names neither what the program must do nor which programming language".

## Formal versions

- **Explanation**: parse the code into the meta language (`LinkNetwork::parse`), map node kinds to meanings (comprehension, set, count, mapping), and explain from those meanings with complexity notes.
- **Debugging**: parse, derive the intended property from names and the request ("average"), execute with generated inputs, compare with the property, locate the node that breaks it (`- 1`).
- **Regex and SQL**: formal languages with their own grammars; compose from the formalized requirement (character classes, optional groups; joins, aggregates, ordering, limit) and verify on generated positive/negative examples or an in-memory database.
- **Review**: parse, match the CST against rules retrieved and formalized from linters' documentation (bare `except`, mutable defaults, shadowing), report each finding with the rule's source.
- **Refactoring**: a meaning-preserving rewrite in the meta language (promise chain → `async`/`await` with `try`/`catch`), verified by running both versions on the same inputs.
- **Test generation**: formalize the specification into properties and examples (case- and space-insensitive palindrome), render tests in the named framework, run them against a reference implementation.
- **Shell commands and format conversion**: compose from formalized manual pages (`find -name -size`) and from the data models of the two formats (JSON ↔ YAML via Links Notation).
- **Generation**: via discovery and recomposition (E129 (#1164), E130).

## How to test

HumanEval/MBPP full slices re-measured; held-out sets of 50 explanation, 50 debugging, 50 regex and 50 SQL tasks with executable checks.

## Requirements

R1. Every one of the sub-classes explanation, debugging, regex, SQL, review, refactoring, test generation, shell commands, format conversion returns an answer that carries at least one derivation link (parsed CST node, matched rule with its source, or execution trace) instead of the canned "Web search requested for" paragraph (`rust/src/solver_handlers/web_requests.rs`) or a terminal-command misroute (`rust/src/solver_terminal.rs`).

R2. A code-explanation answer names, for every explained construct, the CST node kind `meta_language::LinkNetwork::parse` assigned it (e.g. `comprehension`, `set`, `mapping`), sourced from `data/seed/program-cst-grammars.lino`'s declared engine for the input language; an explanation of a construct the parser does not recognize states that gap by name rather than guessing.

R3. A debugging answer derives the intended property from the function's own name and the request text, executes the candidate with at least 3 generated inputs, and when it disagrees with the derived property, names the exact CST node (with its byte or line span) responsible.

R4. A regex-synthesis answer is composed from the formalized requirement's character-class, quantifier and group constraints (not memorized from a template) and is verified against both a positive and a negative generated example set before being returned; a request the composer cannot satisfy is refused by name, never guessed.

R5. A SQL-synthesis answer is composed from the formalized requirement's join, aggregate, filter, order and limit constraints and is verified by executing the generated query against an in-memory instance of the stated schema, showing the returned rows as evidence.

R6. A code-review answer's every finding cites the specific rule and the specific external documentation URL (from `data/seed/sources-registry.lino`) it was formalized from; no finding is a hardcoded Rust literal.

R7. A refactoring answer is verified as meaning-preserving by executing both the original and the rewritten code on the same generated inputs and reporting agreement; disagreement blocks the answer.

R8. A test-generation answer formalizes the stated specification into named properties and examples before rendering test code in the requested framework, and the rendered tests pass when run against a reference implementation of the specification.

R9. A shell-command answer is composed from a formalized manual-page fact base (flag meaning, argument shape) fetched and cached per `data/seed/sources-registry.lino`, not from `shell_command_transform.rs`'s existing screen/infinite-loop-only pattern set.

R10. A format-conversion answer (e.g. JSON→YAML) round-trips through the Links Notation pivot losslessly on at least the held-out probe set (R value equal before and after).

R11. None of the nine sub-classes' answers ever recite a fetched Rosetta Code example verbatim (`rust/src/coding/rosetta_request.rs`'s `render_example` path is for the "example" intent only, not for explanation/debugging/review/etc.); the no-memorization gate (`rust/tests/unit/coding_discovery/no_memorization.rs`) is extended to scan the new handlers' output surfaces the same way it scans `coding-discovery-runtime.lino`.

R12. HumanEval and MBPP are re-measured on their full upstream slices after the change and the resulting pass counts are recorded, whatever they are, in `data/benchmarks/external-results.lino` (consumed by `docs/status.md`'s "Latest external benchmark rows" table).

## Design

**What answers today.** No `rust/src/solver_handlers/*.rs` file implements explanation, debugging, review, refactoring, regex synthesis, SQL synthesis or test generation (`git grep -n "fn try_.*debug\|fn try_.*refactor\|fn try_.*regex\|fn try_.*review\|fn try_.*test_gen" rust/src/` returns nothing on `origin/main`). The closest existing modules are adjacent, not owning, this scope:
- `rust/src/solver_handlers/shell_command_transform.rs` (269 lines) only rewrites a prompt into a `screen` session command or wraps a given command in an infinite loop (`build_screen_command`, `wants_infinite_loop`); it has no general command-composition-from-manual-pages path.
- `rust/src/solver_handlers/pattern_inference.rs` (346 lines) infers structure in 1D/2D *data* sequences and grids (`infer_sequence_patterns`, `infer_grid_patterns`); it is unrelated to regex synthesis over text.
- `rust/src/coding/rosetta_request.rs` fetches a Rosetta Code page only when the prompt names both a seeded task alias (`program_task_by_alias`) and a seeded language alias (`program_language_by_alias`), then renders the fetched example **verbatim** (`render_example`) — no decomposition or recomposition. Generation-by-discovery is E129/#1164 and E130/#1165's scope, not duplicated here; R11/R12 above are this issue's own stake in that work.
- `rust/src/coding/cst.rs` (188 lines) only *validates* already-rendered source through `meta_language::LinkNetwork::parse` (`#[cfg(feature = "meta-language")]`); it has no node-kind-to-explanation mapping today. `data/seed/program-cst-grammars.lino` declares the engine for 11 languages (javascript, python, rust, java, csharp, c, cpp, typescript, go, ruby, php — confirmed by `program_language "..."` entries) and confirms, per #1183, no kotlin/scala entry.

**New modules** (`rust/src/solver_handlers/`, all new files):
- `code_explanation.rs` — `pub fn try_code_explanation(prompt, normalized, log) -> Option<SymbolicAnswer>`: parses via `crate::coding::cst`, maps parsed node kinds to a new `data/seed/meanings-code-structure-explanations.lino` table (one general meaning per node kind, e.g. `dictionary_comprehension` → "builds a mapping from each element of an iterable to a computed value"; what a particular program does is composed from these meanings plus the call meanings such as `count`, never stored per program), renders in the request's language.
- `code_debugging.rs` — `pub fn try_code_debugging(...)`: derives intended property from function name + request via the existing formalization pipeline (`translation::formalization::formalize_prompt_candidates`), executes with `crate::proof_engine` or a small interpreter over the parsed CST, and reports the failing node.
- `regex_synthesis.rs` / `sql_synthesis.rs` — compose from a new `data/seed/formal-language-projections.lino`-style grammar table (that file already carries a `formal_language "fol"` entry; regex/SQL get sibling `formal_language "regex"` / `"sql"` entries with their own `statement` templates) and verify against generated example sets (`rust/src/proof_engine/decision/sat.rs` or a small executor).
- `code_review.rs` — matches parsed CST against `data/seed/coding-guidance.lino` (existing file — verify its schema fits or extend it) plus new rule records citing linter documentation URLs registered in `data/seed/sources-registry.lino`.
- `code_refactoring.rs` — meaning-preserving rewrite via `rust/src/es_meta` (the promise→async/await direction is a JS/TS projection, live per `meta_translate.rs`'s `pending_leg`: JS↔TS is not pending) plus a differential-execution check.
- `test_generation.rs` — formalizes spec into `data/seed/meanings-*` property/example pairs, renders via the language's test-framework template, executes against a reference implementation built the same way E129 builds programs.

**Rust roots and parity.** JS↔TS transformations (refactoring) route through `meta_translate::translate` (`rust/src/meta_translate.rs`), whose `pending_leg` already reports both `JavaScript → TypeScript` and `TypeScript → JavaScript` as live (not `Pending`) — confirmed by reading the full match arm on `origin/main`, so #1177's refactoring handler can call this pivot directly rather than adding a new translation leg. Every other language (Python, the language named in most probes) is **not** a `SourceRoot` in `meta_translate.rs` at all (`SourceRoot::parse` only accepts `rust|rs`, `js|javascript`, `ts|typescript`, `meta|lino`) — explanation/debugging/regex/SQL/review handlers for Python and the other 7 `program-cst-grammars.lino` languages must go through `crate::coding::cst` directly, not through `meta_translate`. Browser/worker parity: each new handler is translated with `formal-ai translate --from rust --to js|ts --input <file> --write`, and `data/parity/cross-runtime-synthesis.json` gains one case per sub-class answered identically by all three roots.

## Probe set and tests

New data file `data/benchmarks/code-tasks/{en,ru,hi,zh}.lino`, following the existing per-language layout of `data/benchmarks/handler-family-paraphrases/{en,ru,hi,zh}.lino`: 50 explanation, 50 debugging, 50 regex, 50 SQL prompts per language (200 total per class), each carrying an `expected_check` field naming an executable check (compare rendered explanation's node-kind citations against a fixture, run generated tests against a reference implementation, execute the regex/SQL against fixed positive/negative corpora).

Test files (register each in `rust/tests/unit/mod.rs`, which currently lists 569 `mod` entries alphabetically):
- `rust/tests/unit/issue_1177_code_explanation.rs`
- `rust/tests/unit/issue_1177_code_debugging.rs`
- `rust/tests/unit/issue_1177_regex_synthesis.rs`
- `rust/tests/unit/issue_1177_sql_synthesis.rs`
- `rust/tests/unit/issue_1177_code_review.rs`
- `rust/tests/unit/issue_1177_code_refactoring.rs`
- `rust/tests/unit/issue_1177_test_generation.rs`

Run: `RUSTUP_TOOLCHAIN=1.98.1 cargo test --test unit issue_1177_` (and one invocation per module above).

HumanEval/MBPP re-measurement updates `data/benchmarks/external-results.lino`; `rust-script scripts/render-status.rs --check` fails until `docs/status.md`'s benchmark table is regenerated with the new rows via `rust-script scripts/render-status.rs --write`.

`docs/llm-task-parity.md` (E136/#1171) class rows for code_generation, code_explanation, debugging, regex, sql, code_review, refactoring, test_generation are regenerated by the same generator #1171 defines, run against these new probe sets.

## Definition of done

- [ ] Requirement shard added under `docs/requirements/issue-1177-*.md`, assembled into `REQUIREMENTS.md` via `rust-script scripts/assemble-requirements.rs --write` (confirmed this script exists and is the sole generator of `REQUIREMENTS.md` from per-issue shards in `docs/requirements/`).
- [ ] Row added by hand to `docs/requirements-traceability.md` (confirmed hand-maintained, not generated, per its own header) recording the automated test and the manual confirmation for each R1-R12.
- [ ] Case study at `docs/case-studies/issue-1177/` (following the `docs/case-studies/issue-1138/` layout: root cause note plus per-stage plans).
- [ ] Changelog fragment via `rust-script scripts/create-changelog-fragment.rs --bump-type minor --description "code-task formal answers (explanation, debugging, regex, SQL, review, refactoring, test generation)"`, landing in `changelog.d/`.
- [ ] `RUSTUP_TOOLCHAIN=1.98.1 rust-script scripts/run-ci-gates.rs --stage rust` green.
- [ ] `rust/tests/unit/coding_discovery/no_memorization.rs` still passes with the new handlers' output surfaces included in its scan — a "formal version" of code generation/explanation must not recite a Rosetta Code template or a hand-written explanation string; this ties directly to the E129/E130 gap #1183 already documents (`rust/src/coding/rosetta_request.rs`'s verbatim rendering).
- [ ] `docs/llm-task-parity.md` rows for the nine sub-classes regenerated and green.

## Depends on / blocks

- Depends on #1164 (E129) — code examples must be decomposed into formal parts before this issue's handlers can recompose explanations/tests/refactors from them rather than from templates.
- Depends on #1165 (E130) — discovery must run on the production path so these handlers are not another set of stored templates.
- Depends on #1167 (E132) — the parse → change the network → render → check path `code_refactoring.rs` needs for languages outside the JS/TS pivot.
- Depends on #1175 (E140) — routing by the formalized request so "Find the bug" stops reaching `solver_terminal.rs`'s terminal-command misroute, and on #1173 (E138) — no canned search fallback.
- Uses #1185 (E149) for debugging and generation that fail on the first run, and #1184 (E148) for the derivation links R1 requires.
- Feeds #1171 (E136) — this issue's probe sets and formal answers are what let the `docs/llm-task-parity.md` rows for these classes turn from "0 useful" to measured passes.
- Part of #1183 (E127 umbrella).

