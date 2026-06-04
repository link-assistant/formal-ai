# Case study — Issue #395: "sort these numbers in JavaScript, give me the code and the result"

> Raw inputs for this study are preserved under
> [`raw-data/`](./raw-data/) (`issue-395.json`, `issue-395-comments.json`,
> `issue-395-body.md`), captured verbatim from GitHub so the analysis below can be
> reproduced and audited.

## 1. Summary

The deployed wasm assistant (v0.179.0) answered the prompt

> У меня есть числа 3, 5, 6, 7, 8 отсортируй их в JavaScript, дай мне код и результат
>
> *("I have the numbers 3, 5, 6, 7, 8, sort them in JavaScript, give me the code
> and the result.")*

with `intent: unknown` — the "I didn't understand you" fallback. The user asked
for two concrete things: **the code** that sorts the numbers, and **the result**
of running it. The system delivered neither.

This PR makes that prompt — and its multilingual / multi-language-target
variants — route to `write_program`, emit idiomatic code in the requested
programming language, and show the **deterministically-computed sorted result**.

## 2. Timeline / sequence of events

| When | Event |
| --- | --- |
| 2026-06-04T17:50:31Z | User runs the Russian prompt against the GitHub Pages wasm build (v0.179.0). |
| — | The worker formalizes it to `(@USER OP:express ?…)` — a generic "express" impulse with no recognized operation. |
| — | Every specialized tool is probed (`wikipedia_lookup`, `fact_query`, `http_fetch`, `web_search`, `docs_method_explanation`, `procedural_how_to`, …) and each returns `no_match`. |
| — | The solver falls through to `fallback: unknown` and renders the Russian "I didn't understand" message. |
| 2026-06-04 | Issue #395 filed with the full reasoning trace and the broader architectural vision (continuation of #387). |

The reasoning trace in [`issue-395-body.md`](./raw-data/issue-395-body.md) is the
key evidence: the formalization is `OP:express` (the catch-all), proving **no
operation was recognized at all** — not "sort", not "write a program". The prompt
contained three strong signals (a sort verb, a programming language, a list of
numbers) and the pipeline bound to none of them.

## 3. Requirements extracted from the issue

The issue body is deliberately broad (it explicitly continues PR #387's vision).
We separate the **concrete, testable defect** from the **long-horizon vision**.

### 3.1 Concrete requirements (addressed in this PR)

1. **R1 — Don't answer `unknown`.** The exact prompt must route to a real intent.
2. **R2 — Give the code.** Produce runnable code in the *requested* language
   (here JavaScript).
3. **R3 — Give the result.** Show the actual sorted output, not just code.
4. **R4 — Meaning-based, multilingual recognition.** Recognition must be driven by
   *meanings* in seed data (not hardcoded English words), so the same request in
   Russian / Hindi / Chinese works identically (vision: "use not english words,
   but meanings themselves").
5. **R5 — Nothing hardcoded per case.** No memorized final answer; a generalized
   algorithm parameterized by seed data + the parsed prompt (vision: "no
   memoization, only generalization of algorithms… instead of hardcoding we
   should use data seed").
6. **R6 — Fix it everywhere.** The defect lives in both runtimes (the Rust solver
   and the JS worker that powers the wasm/browser build); the vision demands
   "fully apply requirements to entire codebase… fixed in all of them".
7. **R7 — Execute and give the real result.** "If environment supports… we can
   also execute the task… and actually give the result for the user."
8. **R8 — Case study.** Download issue data into `docs/case-studies/issue-395/`
   and reconstruct timeline, requirements, root causes, and solution plans
   (this document).

### 3.2 Vision-level requirements (scoped, tracked for follow-up)

The issue also restates the project's north star. These are **not fully solvable
in one PR** and are explicitly out of scope here, recorded honestly so the PR is
not mistaken for completing them:

- **V1 — Full CST/tree-sitter reasoning** for arbitrary programming tasks.
- **V2 — Meanings as a connected type system** (Type → SubType → Value doublets).
- **V3 — Every meaning rooted in Wikipedia/Wikidata/Wiktionary/WordNet** with
  cached upstream responses and translation algorithms.
- **V4 — Append-only `link-cli` transactions** for all data mutations (time
  travel).
- **V5 — General code execution** for *any* synthesized program (sandboxed JS
  `eval`, etc.).

This PR advances V5 for the *decidable* sort case (see §5) and follows the
existing seed-driven meaning pattern toward V2, but does not claim to deliver the
full vision. These remain in the project ROADMAP.

## 4. Root-cause analysis

**Root cause:** there was no handler that recognized a "sort these literal
numbers in language X" request. The prompt's three signals were each individually
recognizable by *existing* machinery, but nothing combined them:

- The **sort verb** already exists as the `reverse_sort` operation family in
  `data/seed/operation-vocabulary.lino` (added for the #349/#386 cancel-sort
  work), but there was **no plain `sort` operation** — only `sort_lines` (a text
  operation) and `reverse_sort`. So a bare "отсортируй / sort" matched nothing.
- The **programming language** is recognizable via
  `crate::coding::program_language_by_alias` (the #386 alias meanings), but only
  the program-synthesis / catalog handlers consulted it, and those require a
  *program artifact* verb ("write a program"), which this prompt lacks.
- The **list of numbers** is the kind of thing the `arithmetic` handler inspects,
  but arithmetic looks for operators, not a sort request.

Because no handler claimed the prompt, dispatch fell through to `unknown`.

A secondary, subtler root cause surfaced during implementation: the dispatch
passes handlers `prompt.to_lowercase()` (which **keeps punctuation**), so a
language word glued to a comma — `JavaScript,` → `javascript,` — fails the
token-boundary alias matcher. Any new handler that resolves a language must
re-normalize the prompt the same way the worker does. (Fixed by re-normalizing
inside `solve_sort_numbers` via `crate::web_engine_core::normalize_prompt`.)

## 5. Why the result can be computed deterministically

The issue asks the system to "actually give the result". For a general program
that would require sandboxed execution (vision V5). **Sorting a list of literal
numbers is a pure, total, decidable function** — there are no inputs, no I/O, no
nontermination. So the solver computes the result *by construction*: it performs
the very same comparison the generated code performs and shows the output. The
generated code and the shown result are guaranteed consistent because they derive
from one ordering. No untrusted code is executed; the answer is verified by
construction. This is the safe, correct first slice of V5.

## 6. Solution implemented

A new seed-driven specialized handler, `sort_numbers`, mirrored across both
runtimes:

| Concern | Rust | JS worker |
| --- | --- | --- |
| Handler | `src/solver_handlers/sort_numbers.rs` | `trySortNumbers` in `src/web/formal_ai_worker.js` |
| Sort verb | `sort` / `reverse_sort` ops in `data/seed/operation-vocabulary.lino` | inline `OPERATION_VOCABULARY_LINO` (byte-identical copy) |
| Language | `crate::coding::program_language_by_alias` | `programLanguageFromPrompt` → `WRITE_PROGRAM_LANGUAGES` |
| Dispatch slot | `SPECIALIZED_HANDLERS`, before `arithmetic` & `program_synthesis` | `syncHandlers`, before `tryProgramSynthesis` & `tryArithmetic` |

The handler fires only when **all three** signals are present (a sort verb, ≥2
numbers, and a known programming language), so it never steals plain prose. It:

1. Re-normalizes the prompt (punctuation → space) so `JavaScript,` resolves.
2. Confirms the `sort` (or `reverse_sort`, for descending) operation via the seed
   vocabulary — in any of en/ru/hi/zh.
3. Resolves the target programming language from its alias meanings.
4. Parses every signed/decimal number, preserving the user's surface text.
5. Sorts ascending (or descending if `reverse_sort` matched).
6. Generates idiomatic code for the resolved language (10 supported: JavaScript,
   TypeScript, Python, Rust, Go, Ruby, Java, C#, C, C++).
7. Renders a localized answer (en/ru/hi/zh): an intro sentence, the code fence,
   and `Result: …` with the computed order.

### Example (the exact issue prompt)

```
Вот код на JavaScript, который сортирует числа 3, 5, 6, 7, 8 по возрастанию:

```javascript
const numbers = [3, 5, 6, 7, 8];
const sorted = [...numbers].sort((a, b) => a - b);
console.log(sorted.join(", "));
```

Результат: 3, 5, 6, 7, 8
```

## 7. Existing components surveyed (reuse over reinvention)

| Need | Existing component reused |
| --- | --- |
| Multilingual verb recognition | `OperationVocabulary` (`src/seed/operation_vocabulary.rs`) — added a `sort` op; reused `reverse_sort` for descending. |
| Programming-language detection | `crate::coding::program_language_by_alias` + `program_language_<slug>` alias meanings (#386). |
| Prompt normalization parity | `crate::web_engine_core::normalize_prompt` / `normalizePrompt`. |
| Response-language localization | `crate::language::detect` / `detectLanguage` (en/ru/hi/zh). |
| Handler finalize/eventlog plumbing | `finalize_simple` and the append-only `EventLog`. |
| Cross-runtime parity convention | The inline-LINO + node-VM experiment pattern (e.g. `experiments/issue-386-js-cancel-sort.mjs`). |

No new third-party libraries were introduced; the work is entirely additive on
top of the existing seed-driven architecture.

## 8. Verification

- **Rust** — `tests/integration/issue_395_sort_numbers.rs`: the exact Russian
  prompt is no longer `unknown`; English/JS computes the sorted result;
  Python descending uses `reverse=True`; Hindi/Chinese verbs are recognized;
  guard rails reject no-language / single-number prompts. Plus
  `examples/repro_issue_395.rs`.
- **JS worker** — `experiments/issue-395-js-sort-numbers.mjs`: replays the five
  reproduction prompts through the worker and asserts byte-identical code and
  results to the Rust path.
- Full suite: 767 unit + 25 integration tests pass; `cargo fmt`, `clippy
  -D warnings`, and the file-size guard are clean.

## 9. Scope and honesty note

This PR resolves the concrete, reproducible defect in issue #395 (R1–R8) using a
generalized, seed-driven, multilingual, dual-runtime approach. It deliberately
does **not** claim to deliver the full architectural vision (V1–V5); those remain
open and are tracked in the project ROADMAP. Computing sort results
deterministically is the correct, safe first slice of "actually give the result"
— general sandboxed execution of arbitrary synthesized programs is future work.
