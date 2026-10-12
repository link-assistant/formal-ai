# Issue #1186 (E150) — formalization as a user-facing task

Issue [#1186](https://github.com/link-assistant/formal-ai/issues/1186), part
of the E127 umbrella [#1183](https://github.com/link-assistant/formal-ai/issues/1183),
parity class `formalization` of #1171. The requirement shard is
`docs/requirements/issue-1186-formalization-as-a-user-facing-task.md`; the raw
issue is [`raw-data/issue-1186.json`](raw-data/issue-1186.json). It has no
comments, so every requirement comes from the body.

## Timeline

| When | Event |
| --- | --- |
| 2026-09-29 10:51 UTC | Issue opened. Probe on 0.347.0: `formal-ai chat --silent --prompt "Formalize in first-order logic: Every student who studies passes the exam."` → "Web search requested for … In the browser demo formal-ai defaults to the DuckDuckGo Instant Answer endpoint …". |
| 2026-09-30 11:12 UTC | PR [#1188](https://github.com/link-assistant/formal-ai/pull/1188) opened for the bulk batch. |
| 2026-09-30 (+07:00) | `dc3671dc7` — `data/seed/formal-targets.lino`, `rust/src/solver_handlers/formalization_task{,_render}.rs`, the response seed, and `rust/tests/unit/issue_1186_formalization_task.rs`. Dispatch wiring deliberately left to integration. |
| 2026-09-30 (+07:00) | `fca05d36d` — `formalization_request` enters the dispatch table in `rust/src/solver_dispatch.rs`, just before `product_search` and `web_search`. |
| 2026-10-06 (+07:00) | `151cb0ed2` — JS twin `js/worker/formal_ai_worker_formalization_request.js` (TS twin under `ts/worker/`) and `rust/tests/web/issue-1186-formalization-parity.test.mjs`. |
| 2026-10-06 (+07:00) | `4566820cb` — the Rust reader found its records empty: `formal-targets.lino` wraps every record under one root, which the JS twin already descended into. Fixed, together with Hindi combining marks in the tokenizer and a deformalize cue winning over the "formalize" it contains. |
| 2026-10-07 (+07:00) | `2e1d6cb62` — "No cat that sleeps hunts" rendered `¬∃x (…)` in FOL but a positive `∃` in Lean and Rocq. Fixed in both roots; changelog line in `changelog.d/20261007_120000_full-suite-repairs.md`. |

## Requirements

| ID | Issue | Requirement (short) | Status |
| --- | --- | --- | --- |
| R1186-1 | R1 | formalization commands in en/ru/hi/zh route to the handler, never the search paragraph | implemented |
| R1186-2 | R2 | quantified sentence → nested clause | implemented |
| R1186-3 | R3 | FOL with seeded symbols, `∀x (P(x) ∧ Q(x) → R(x))` | implemented |
| R1186-4 | R4 | Lean 4 / Rocq via relative-meta-logic in process; compile when provers exist | partial — seeded templates, no rml, provers only detected |
| R1186-5 | R5 | deformalization FOL/Lean/Rocq → request language, structural round trip | partial — FOL only |
| R1186-6 | R6 | `formal-ai explain` prints the derivation | partial — inline `Derivation:` block, no stored record |
| R1186-7 | R7 | three-roots parity + `cross-runtime-synthesis.json` cases | partial — JS/TS twins and web parity test; no synthesis cases |
| R1186-8 | R8 | `docs/llm-task-parity.md` formalization row verified | pending |
| R1186-9 | R9 | no-memorization gate extended to the clause seed | pending |
| R1186-10 | Probe set and tests | `data/benchmarks/formalization/{en,ru,hi,zh}.lino` | pending |

Process items from the definition of done: this shard and case study (done
here), traceability rows (drafted for the maintainer), a `changelog.d/`
fragment for the feature itself (not written — only the negation fix has a
changelog line), and a green `run-ci-gates.rs --stage rust` (not re-run for
this write-up).

## Root causes

1. **No route.** `try_translation` was the only handler that read
   `formal_language_in_prompt`, and it gates on a translation verb from the
   `translation_action` role; "formalize" was in no role, so the prompt fell
   through to the search handler — the same fallthrough #1173 and #1175
   document for other classes.
2. **No clause representation.** `translation::formal_statement` renders one
   Wikidata-anchored triple (`P31` only). A universally quantified
   conditional has no entity subject or object, so it would have been
   `UnrecognizedStatement` even if routed. `rust/src/formalization/` is a
   research-document formalizer, and `proof_engine` has no formula type.
3. **No Lean/Rocq bridge.** relative-meta-logic can export Lean and Rocq
   from its own repository, but no crate is published
   (link-foundation/relative-meta-logic#185), so nothing could be linked.
4. **The negation bug (after delivery).** Both FOL and Links Notation took
   the quantifier symbol from the seed, but the Lean and Rocq
   `clause_conjunctive` templates spelled `∃`/`exists` literally, so the
   `no` quantifier vanished in exactly those two targets. Rocq additionally
   needs parentheses because `~` binds tighter than `exists`; that is why it
   gained a separate `clause_negative` template while Lean simply reads
   `{quantifier}` and renders `¬∃ (x : U), …`. Neither output has been run
   through `lean` or `coqc` (R1186-4).
5. **The empty grammar (after delivery).** The seed wraps its records under
   a `formal_targets` root, and the Rust reader did not descend into it, so
   the grammar, cue roles and response templates read as empty while the JS
   twin read them correctly (`4566820cb`'s message) — a divergence between
   the roots that the parity work surfaced.

## Prior art

- [link-foundation/relative-meta-logic](https://github.com/link-foundation/relative-meta-logic):
  `rml export lean` (typed, non-probabilistic fragments) and `rml export rocq`
  (typed LiNo subset), plus `Pi`/`lambda`/`apply` kernel rules. The intended
  in-process backend for R1186-4, blocked on #185.
- [miniF2F](https://github.com/openai/miniF2F) and ProofNet — the benchmark
  families `docs/llm-task-parity.md` names for the class; they pair informal
  statements with Lean formalizations and are the natural source for a wider
  R1186-10 probe set.
- Classical textbook translation of "Every A that B C" into
  `∀x (A(x) ∧ B(x) → C(x))` and of "No A that B C" into `¬∃x (A(x) ∧ B(x) ∧
  C(x))` — the two clause shapes the seed declares (conditional for
  universal, conjunctive for existential and negative readings).

## What was built

- `data/seed/formal-targets.lino` (mirrored under `rust/embedded/`) — the
  `cues` record (formalize/deformalize verbs per language, the role suffix
  being the answer language), four target grammars (FOL, Lean 4, Rocq, Links
  Notation) with atom templates, join and quantifier symbols, the two clause
  shapes and Rocq's `clause_negative`, and five natural-language recognition
  tables with deformalization templates.
- `rust/src/solver_handlers/formalization_task.rs` — seed loading, the
  tokenizer, `parse_quantified_clause` (comma-delimited relatives for ru/hi,
  article-introduced objects for en/es, segmentation for zh).
- `rust/src/solver_handlers/formalization_task_render.rs` — rendering of
  every target with the named one first, the honesty block (`prover_check`
  probes `PATH`, never executes), the inline derivation, `parse_fol_clause`
  and `deformalize_answer` with the structural round-trip verdict.
- JS/TS twins under `js/worker/` and `ts/worker/`.

## Verification

`rust/tests/unit/issue_1186_formalization_task.rs` (13 tests): the issue
probe across targets, the exact objectless FOL, ru/hi/zh conditionals,
existential and negative readings, `negative_quantifier_keeps_its_negation_in_lean_and_rocq`,
named-target ordering, the deformalization round trip and its localized
prose, an honest refusal for an unparseable sentence, non-claiming of
unrelated prompts, and `engine_answers_formalization_request` through
`UniversalSolver`. `rust/tests/web/issue-1186-formalization-parity.test.mjs`
(11 tests) replays the same cases against the browser worker, including the
negation regression. The regression tests assert the exact strings:

```text
theorem formalized : ¬∃ (x : U), Cat x ∧ Sleeps x ∧ Hunts x := by sorry
Theorem formalized : ~ (exists (x : U), Cat x /\ Sleeps x /\ Hunts x).
Proof.
Admitted.
```

Neither suite was re-run while writing this case study; statuses reflect
what the test bodies assert against the code on `qa-reasoning-coding-bulk-fixes`.

## Residuals

R1186-4 (relative-meta-logic export and an actual `lean`/`coqc` compile),
R1186-5 (Lean/Rocq reverse parse), R1186-6 (a stored derivation record for
`formal-ai explain`), R1186-7 (`cross-runtime-synthesis.json` cases),
R1186-8, R1186-9, R1186-10, and the feature changelog fragment.
