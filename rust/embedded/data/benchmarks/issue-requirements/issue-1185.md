Part of the E127 umbrella (#1183). Architect (2026-09-29): "mimic the tradional way of coding by hand, which humans did previously".

## Gap

The architect's vision is to "mimic the traditional way of coding by hand": build/run, read the compiler or runtime error, search the error text, read the fix, apply it, retry. Formal AI today can *detect* a failed step honestly, and can *report* it, but nothing closes the loop back to a fix:

- `rust/src/agentic_coding/tool_result.rs:62-70` — `failed_verification(run_outputs, verification_command, prompt)` reads the harness's own reported exit status (`StepOutcome::{Succeeded, Failed, Unreported}`, `tool_result.rs:29-50`) and renders a report. It is a **verdict**, not a diagnosis: it never parses the error text into a file/line/code/message, and it does not act on it.
- `rust/src/agentic_coding/command_reroute.rs:67-84` — when `RecipeProgress.failure` is `Some`, `plan_symbolic_command_reroute` immediately returns `AgenticPlan::Final(failure.report(...))` (`command_reroute.rs:83`). There is no branch that reads `StepFailure { reported, exit_code, from_run }` (`command_reroute.rs:174-179`) as a diagnostic to search on, no retry, no bounded ladder — the recipe **stops at the first failed step** and reports it as the final answer.
- `rust/src/agentic_coding/repair_strategy.rs:1-27` classifies a failure into one of three canonical repair targets (solver method, data record, test) via `crate::repair_strategy::canonical_strategies()`, but only for `UnknownTrace` instances drawn from **three self-contained canonical failure traces** built into Formal AI's own self-improvement loop (`self_heal.rs` similarly operates on one canonical case, `data/meta/repair-strategies.lino`/`self-healing-case.lino`). Neither module ever sees an arbitrary compiler diagnostic (`kotlinc: error: unresolved reference`) produced while generating code for a user request.
- `rust/src/coding/synthesis_runtime.rs:169-183` logs `synthesis:source_miss` diagnostics — string messages about *discovery* misses (no matching stdlib function), not compiler/runtime errors from *running* generated code.
- `rust/src/agentic_coding/web_research.rs:1-46` sequences search→rank→fetch→answer for a **user-stated research subject** (`web_research_query_for`); nothing feeds an *error message* into this pipeline as the query.
- `docs/case-studies/issue-1138/plans/06-prerequisite-discovery.md` (plan 06, dated 2026-09-16) is the closest existing design: it defines `PrerequisiteNeed`, a `SourceLookup` trait client over `data/seed/sources-registry.lino`, and recovery "inside `Workspace::run`" — but scoped to **missing programs** (`kotlinc: command not found`) triggering install-and-retry, not to **compiler/runtime error messages** from a program that *did* run and *did* fail. Plan 06 §(b) states the exact current limit: `command_reroute::RecipeProgress` "stops at the first failed recipe step. It does not interpret later recovery evidence or retry that step" (issue-710 plan 07, quoted at plan-06's evidence (b)); the interpretation half ("what does exit code / stderr text *mean*, and what search finds a fix") was explicitly left undone.
- Plan 06 has since shipped as `rust/src/prerequisite/` (`probe.rs:37` `NOT_FOUND_MARKER = "command not found"`, pinned publishers, grant-gated install), so a *missing program* is recognised; but it is only used by the benchmark grader (`external_benchmarks/grade.rs`), and #1159 (E124) wires it into the executor. Nothing recognises or acts on the error text of a program that ran and failed.

So the pieces are: honest failure *detection* (tool_result.rs), honest failure *reporting* (command_reroute.rs, StepFailure), a *classifier* for three hand-picked self-referential cases (repair_strategy.rs), a *search pipeline* keyed on a stated subject (web_research.rs), and a *prerequisite* recovery design for missing executables only (plan 06). None of them are wired: diagnostic → search the error text → read the fix → apply → retry.

## Requirements

R1. A failed step's raw output (stdout/stderr, as `command_reroute.rs`'s harness already captures it) is formalized into a structured diagnostic: file, line (when the compiler/interpreter states one), error code (when the language has one, e.g. Rust's `E0308`, TypeScript's `TS2322`), and message text — via the meta language (E128/#1163's page-and-text formalizer), not per-language regexes.

R2. When `RecipeProgress.failure` is `Some` (`command_reroute.rs:67`) and the retry budget is not exhausted, the plan is **not** finalized immediately: the diagnostic from R1 becomes a search query (reusing E128 (#1163)'s "search as the entry point for any need"), fetched pages are formalized, and any code/prose fragment addressing the exact error code or message is retained as a candidate fix.

R3. A candidate fix is expressed and applied in the meta language (E132/#1167: compose in `LinkNetwork`, render to the target language), never as a raw text patch pasted over the previous source.

R4. Retries are bounded and the bound is reported honestly (no silent budget, per the no-deferral policy): each attempt is a rung on a ladder, and reaching the top is reported as such, mirroring the existing honest-limit pattern in plan 06 (`ExecutionBox` iteration ladder) rather than inventing a second one.

R5. Every repair attempt — diagnostic, search query, candidate fix, whether it resolved the failure — is recorded as an evidence chain reachable from the answer's derivation (#1184, E148), so a repair that took three tries is auditable, not silently absorbed into a single "succeeded" report.

R6. A diagnostic whose error text matches no fetched source is reported as an unresolved need (consistent with `PrerequisiteNeed`'s `Unsatisfiable` state in plan 06), never as a fabricated fix.

R7. The loop applies uniformly to any language Formal AI emits code in (the Hive Mind Hello World set: Kotlin, Scala, Rust, Java, Go, Python, C, C++, C#, Ruby, PHP, Swift, TypeScript, JavaScript — the same set E132/#1167 names), not a per-language handler.

R8. Three-roots parity: `error_diagnosis.rs` and `repair_loop.rs` are translated with `formal-ai translate --from rust --to js|ts --input <file> --write`; `data/parity/cross-runtime-synthesis.json` gains diagnostic-formalization cases answered identically by all three roots.

## Design

- New: `rust/src/error_diagnosis.rs` — `pub struct Diagnostic { pub file: Option<String>, pub line: Option<u32>, pub code: Option<String>, pub message: String, pub raw: String }` and `pub fn formalize_diagnostic(language: &str, raw_output: &str) -> Vec<Diagnostic>`, built on the same `LinkNetwork` document-formalization path E128 (#1163) adds for arbitrary text (a compiler's stderr is just unstructured text with a recognizable shape — line-prefixed entries — so the generic formalizer's list/paragraph structure applies, not a new per-language parser).
- New: `rust/src/agentic_coding/repair_loop.rs` — the wiring point. `pub struct RepairAttempt { pub diagnostic: Diagnostic, pub search_query: String, pub candidate_fix: Option<String>, pub applied: bool, pub resolved: bool }`, `pub fn plan_repair(failure: &StepFailure, ladder_rung: u8, max_rungs: u8) -> Option<AgenticPlan>`, called from `command_reroute.rs`'s failure branch (`command_reroute.rs:67-84`) **before** it finalizes, replacing the immediate `AgenticPlan::Final` with: formalize diagnostic (R1) → search via E128 (#1163)'s generic page formalizer with the diagnostic's `message`/`code` as query → decompose the fix from the matched fragment via E129 (#1164) → recompose/render via E132 (#1167) → re-run → re-check `StepOutcome`; on exhaustion (R4) or no match (R6), fall back to today's `failure.report(...)`.
- Modify `rust/src/agentic_coding/command_reroute.rs:67-84`: thread a rung counter through `RecipeProgress` (new field `repair_rung: u8`), and branch to `repair_loop::plan_repair` before constructing the `Final` plan.
- Modify `rust/src/agentic_coding/repair_strategy.rs`: generalize `canonical_strategies()`'s three-case classifier to also accept an arbitrary `Diagnostic` (R1's type) as a fourth, non-canonical input path, reusing its existing three-target classification (solver method / data record / test) as the *kind* of fix when the failure is Formal AI's own code, and returning a "target program" classification when the failure is in generated user-facing code.
- Data: no committed `.lino` fixture holds live error text (it is per-run), but a diagnostic taxonomy of known error-code shapes per language (used only to *recognize* the code, not to answer it) can seed `data/seed/diagnostic-code-shapes.lino`, styled like existing seed lexicon files, e.g.:

```
diagnostic_code_shapes
  language rust
    pattern "error[E{code}]: {message}"
  language typescript
    pattern "error TS{code}: {message}"
  language kotlin
    pattern "error: {message}"
```

This is a recognition aid only (where does "the code" sit in the line), never a per-language fix table — the fix always comes from a fetched, formalized source per R2/R3.

- CLI/API surface: no new subcommand; the loop is internal to the agentic executor and visible through the derivation record of #1184 (E148): `formal-ai explain <answer-id>` shows the `RepairAttempt` chain as part of `verification`.
- JS/TS parity: per R8.

## Tests

- New module `rust/tests/unit/issue_1185_repair_loop.rs` (register `mod issue_1185_repair_loop;` alphabetically in `rust/tests/unit/mod.rs`).
  - `formalize_diagnostic_extracts_file_line_code_and_message_from_rustc_output` — fixture: captured `rustc` stderr with `error[E0308]`.
  - `formalize_diagnostic_extracts_from_kotlinc_output_with_no_error_code` — Kotlin diagnostics have no `E####` code; `code` is `None`, not fabricated.
  - `repair_loop_stops_and_reports_final_when_no_fetched_source_matches_the_diagnostic` (R6) — offline fixture with no matching page; `plan_repair` returns `None`, falling back to today's `Final` report.
  - `repair_loop_is_bounded_and_reports_reaching_the_top_rung` (R4) — asserts the ladder terminates honestly rather than looping or silently truncating.
  - `repair_attempt_chain_is_recorded_for_the_answer_derivation` (R5) — asserts the chain appears in the answer's derivation record from #1184 (E148).
  - `command_reroute_calls_repair_before_finalizing_a_failed_step` — exercises `command_reroute.rs:67-84`'s branch directly, asserting `repair_loop::plan_repair` is consulted before `AgenticPlan::Final`.
- Online test gated like `rust/tests/unit/issue_991_how_to_synthesis.rs:52-61`'s `live_fetch_requested()` (`FORMAL_AI_LIVE_FETCH=1`): `repair_loop_resolves_a_real_rustc_borrow_error_from_a_fetched_stack_exchange_or_doc_page` — a program with a known, findable `E0502` borrow-checker error is generated, fails, repaired via a live fetch, and the second run's `StepOutcome::Succeeded`, with the fix's source URL recorded.
- Commands: `RUSTUP_TOOLCHAIN=1.98.1 cargo test --test unit issue_1185_repair_loop`; live variant `FORMAL_AI_LIVE_FETCH=1 RUSTUP_TOOLCHAIN=1.98.1 cargo test --test unit issue_1185_repair_loop` (the live test returns early without the variable; not `#[ignore]`d).

## Definition of done

- [ ] Requirement shard `docs/requirements/issue-1185-error-driven-repair-loop.md` with R1-R8; `RUSTUP_TOOLCHAIN=1.98.1 rust-script scripts/assemble-requirements.rs --write` run.
- [ ] Row for every requirement in `docs/requirements-traceability.md`.
- [ ] Case-study `docs/case-studies/issue-1185/` with this audit, a plan naming the exact `command_reroute.rs:67-84` seam, and a survey of existing repair-classification code (`repair_strategy.rs`, `self_heal.rs`) so the new loop reuses rather than duplicates their classification.
- [ ] Changelog fragment `changelog.d/<timestamp>_issue-1185-error-driven-repair-loop.md` (repository root), `bump: minor`.
- [ ] `RUSTUP_TOOLCHAIN=1.98.1 rust-script scripts/run-ci-gates.rs --stage rust` green.
- [ ] The three Hive Mind Hello World languages (Kotlin, Scala, Rust) each demonstrate one real repaired-and-retried failure in the case study, not just the offline unit fixtures.

## Depends on / blocks

- Depends on #1163 (E128): the loop's "search the error text" step is E128's generic page-formalizer used with a diagnostic as the query.
- Depends on #1164 (E129): "read the fix" is E129's decompose-and-recompose over the matched fragment.
- Depends on #1167 (E132): "apply the fix" composes and renders in the meta language, not as a raw text patch.
- Works with #1165 (E130) and #1159 (E124): #1159 handles a missing program (install, re-probe); this issue handles a program that ran and failed. Both hook the same failure branch in `command_reroute.rs:67-84`; the prerequisite check runs first.
- Depends on #1184 (E148) for R5 (the repair chain in the derivation record).
- Blocks umbrella #1183 acceptance #1 (three unseen languages "produce … a verification that passes") for any generated program that does not compile on the first attempt.

