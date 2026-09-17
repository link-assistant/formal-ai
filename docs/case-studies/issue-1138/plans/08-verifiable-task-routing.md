# Plan 08 — Route every verifiable task through recognise/discover/compose/verify/remember (bottleneck B8 of #1138)

## Issues addressed

- **#1138 B8** — "Non-coding reasoning suites sit at the floor with no derivation
  path … These are the same meta algorithm applied to word problems: understand
  each word, retrieve the method, reconstruct the steps, execute, verify. Their
  floor scores show the pipeline in B1-B5 is currently wired only to Python-function
  synthesis. Fixed means: the discovery path is invoked for any task with a
  verifiable expectation, not only for `CodingTaskSpec`, and the scheduled ledger
  records the new numbers." This plan delivers the routing, the shared task type,
  the execute-to-answer step, and the re-measurement.
- **#1138 B2** — its sibling. B2 makes composition general across *sources*; B8
  makes it general across *task kinds*. They share `ProgramIr`
  (`docs/case-studies/issue-1138/plans/02-composition-from-sources.md`), and this
  plan consumes it rather than defining a second IR.
- **#315 (E30, closed)** — "Keep memorized seeds only as one candidate generator
  among others; a previously-seeded program must still be reproducible, but
  unseen specs must be solved by derive-and-verify." Delivered for Python
  functions only. This plan applies the same clause to word problems, counting
  questions and text edits, whose current answerers are seeded tables
  (`src/solver_synthesis.rs:345-447`) and keyword cascades
  (`src/solver_handlers/pattern_inference.rs:29-43`).
- **#317 (E32, closed)** — "held-out / paraphrased variants … so a memorized
  answer cannot pass them". The curated slice has them; GSM8K/MATH/object
  counting/CoEdIT have none in this repository, and the object-counting answerer
  is a 9-category English table. This plan adds the five-language held-out set.
- **#698 (E56, closed)** — R529 honest scores, R530 scheduled ledger rows. The
  four suites' rows are honest but stale: their newest entries are dated
  `2026-09-07` at solver `0.347.0`
  (`data/benchmarks/external-results.lino:779-822`), before the discovery path
  landed on 2026-09-15. This plan re-measures and appends.
- **#891 (closed)** — "Define a machine-readable corpus with at least 50 distinct
  equation types … Add a CI ratchet that fails below 50 or on any corpus
  regression … Record category coverage and upstream calculator limitations."
  Delivered as 72 cases and 10 recorded limitations
  (`data/benchmarks/equation-type-corpus.lino:891-980`). This plan converts those
  10 loud failures from "recorded limitation" into "task the discovery path may
  attempt", without ever fabricating an answer.
- **#923 (E76, closed)** — established that a new reasoning capability lands with
  external benchmark scores recorded in `data/benchmarks/`. This plan follows
  that contract for the four non-coding suites.
- **Issues plan 13's coverage table names this plan as a deliverer of** (added by
  the 2026-09-16 reconciliation): **#1071** ("support counting to 100 or any
  number" — the route reaches any task with a verifiable expectation, and the
  `Count` expectation is exactly its shape); **#700** (the `unit_carrying_*`
  limitation class is fixed by check 5; adopting `link-foundation/si-units` as a
  dependency stays a maintainer decision, so #700 is partial); **#939** (the
  installation-guide corpus's verifiable half — a guide becomes a script whose
  run is the observation).
- **#938 (E86, closed)** — one shared meta-algorithm builder for coding handlers.
  This plan extends the same builder surface
  (`src/meta_algorithm_builder.rs`, used at
  `src/solver_handlers/program_synthesis.rs:30`) to the verifiable-task route so
  no second construction trace appears.

---

## Current state

### The discovery path has exactly one entrance, and it requires Python

`try_program_synthesis_with_online`
(`src/solver_handlers/program_synthesis.rs:23-134`) opens with
`let spec = recognise(prompt)?;` (`:29`). `recognise`
(`src/coding/task_spec.rs:126-244`) succeeds in exactly three ways:

1. a Python `def` signature in the prompt (`src/coding/task_spec.rs:133-148`);
2. an `assert name(...) == …` contract (`src/coding/task_spec.rs:150-181`);
3. a conversational request that carries `ROLE_CODING_REQUEST_VERB` **and**
   `ROLE_CODING_REQUEST_OBJECT` **and** a literal `python` token
   (`src/coding/task_spec.rs:190-193`).

A GSM8K question, a MATH problem, a BIG-bench counting prompt and a CoEdIT edit
instruction satisfy none of these, so `recognise` returns `None`, the `?`
short-circuits, and `discover_and_compose`
(`src/coding/synthesis_runtime.rs:204-220`) is never reached. That single `?` is
the whole of B8's mechanism.

`CodingTaskSpec` (`src/coding/task_spec.rs:48-62`) is the only solver-side type
carrying a checkable expectation — `examples: Vec<Example>` (`:56`) and
`expected_stdout: Option<String>` (`:60`). The harness-side `Expectation`
(`src/external_benchmarks/cases.rs:11-25`) is the only other one, and it lives
downstream of the solver: it is a scorer, never an input. There is no shared
notion of "a task whose answer can be checked".

### What the harness actually sends, per suite

`src/external_benchmarks/mod.rs:151-160` is the single call site:
`.map(|case| solver.solve(&case.prompt))`. No suite-specific routing exists on
the harness side. The prompts:

| Suite | Prompt built at | Prompt content | Grading |
| --- | --- | --- | --- |
| HumanEval | `cases.rs:66-68` | instruction + the Python stub | `PythonUnitTest`, real `python3` run (`grade.rs:55-64`, `:446-479`) |
| MBPP | `cases.rs:81-84` | text + instruction + the upstream asserts | `PythonAsserts` (`grade.rs:65-71`) |
| GSM8K | `cases.rs:106` | **the bare upstream `question`** | `NumericAnswer` → last number (`grade.rs:140-143`, `:504-522`) |
| MATH | `cases.rs:118` | **the bare upstream `problem`** | `BoxedAnswer` → last `\boxed{…}` or last line (`grade.rs:144`, `:546-577`) |
| object counting | `cases.rs:127` | **the bare upstream `input`** | `NumericAnswer` (`grade.rs:140-143`) |
| CoEdIT | `cases.rs:139` | **the bare upstream `src`** (which already carries its instruction) | `ExactText` (`grade.rs:145-148`) |
| SWE-bench Lite | `cases.rs:157-160` | instruction + problem statement | official harness (`grade.rs:172-322`) |

The asymmetry is load-bearing: the two suites that score above zero are the two
whose prompts carry an explicit answer-format instruction *and* whose format
happens to be the one shape `recognise` accepts. MATH's grader demands
`\boxed{…}` while its prompt never asks for one.

### What answers a word problem today

The registry dispatch order is seed data:
`data/seed/handler-precedence.lino`, 59 names, joined to the native table by
`specialized_handlers()` (`src/solver_dispatch.rs:396-430`) and executed by
`meta_method_dispatch::try_dispatch` (`src/meta_method_dispatch.rs:30-147`).
`arithmetic` is 34th. Before it sit `text_manipulation` (17th),
`compound_interest` (28th), `numeric_list` (29th),
`number_constraint_reasoning` (31st), `pattern_inference` (32nd),
`program_synthesis` (33rd).

For a GSM8K question the trajectory is:

1. `crate::coding::program_contract::answer` (`src/solver.rs:422-424`) — no
   program contract, `None`.
2. `record_decomposition` + `solve_sub_impulses` (`src/solver.rs:426-429`).
3. `try_synthesize_from_sub_results` (`src/solver.rs:534-542`) →
   `src/solver_synthesis.rs:97-161`, which tries
   `compose_algebra_substitution` (`:163`), `compose_remainder_sale` (`:209`),
   `compose_object_count` (`:248`), then `compose_compound_response` (`:303`).
4. `meta_method_dispatch::try_dispatch` (`src/solver.rs:551-560`). Only
   `arithmetic` can produce a number: `handle_arithmetic`
   (`src/solver_dispatch.rs:68-74`) → `try_arithmetic`
   (`src/solver_handlers/mod.rs:81-197`) → `calculation_expression_candidates`
   (`src/calculation.rs:877`) → `normalize_word_problem_detailed`
   (`src/calculation_word_problem.rs:550-614`) → `evaluate_calculation`
   (`src/calculation.rs:425`).
5. On failure: `solver_terminal::try_terminal_command` (`src/solver.rs:621-625`),
   `solver_search::try_budget_search` (`src/solver.rs:631-635`),
   `answer_unknown_prompt` (`src/solver.rs:639-647`).

`normalize_word_problem_detailed` handles three authored shapes — a box-total
problem, a train-meeting problem and a Fibonacci-reference rewrite — plus a
10-entry phrase table (`src/calculation_word_problem.rs:584-595`). That is the
entirety of the word-problem capability, and it explains a stable 2/20 across
seven recorded runs (`data/benchmarks/external-results.lino:198, 276, 359, 464,
569, 674, 779`).

### What answers an object-counting question today

`compose_object_count` (`src/solver_synthesis.rs:248-301`) is gated on two
literal English substrings:

```rust
let have_start = lower.find("i have ")? + "i have ".len();
let question_start = lower[have_start..].find("how many")? + have_start;
```

(`src/solver_synthesis.rs:254-255`). It then splits on commas, and classifies
each item against `OBJECT_CATEGORIES` (`src/solver_synthesis.rs:345-447`) — a
Rust `const` of 9 categories (`musical instruments`, `fruit`, `vegetables`,
`animals`, `vehicles`, `tools`, `utensils`, `furniture`, `clothing`) holding
roughly one hundred hard-coded English item names. It counts *items*, never
quantities: a listed "three trumpets" contributes one, not three.

This is a memorized ontology in Rust, in four ways the doctrine forbids at once:
English-only, closed, un-sourced, and un-forgettable. Its score is 0/20
(`data/benchmarks/external-results.lino:801-811`), so it is not even buying the
memorization back.

### What answers a text edit today

`text_manipulation` (`src/solver_handlers/text_manipulation.rs`) parses through
`TextRequest::parse_with_history` (`:124-174`) and requires either a quoted
operand or a colon-delimited payload; `TextRequest::build` (`:176-182`) returns
`None` when `operations.is_empty() || input.is_empty()`. A CoEdIT prompt such as
an instruction-prefixed sentence with no quotes and no vocabulary-slug verb match
therefore declines outright, and the suite scores 0/20
(`data/benchmarks/external-results.lino:812-822`). The 22 primitives in
`src/solver_handlers/text_edit_ops.rs` are pure and keyword-free; the gap is
entirely in recognition and in the absence of a verify step.

### The equation corpus and its loud-failure gaps

`data/benchmarks/equation-type-corpus.lino` (980 lines) declares
`minimum_pass_count "72"` (`:9`) and `minimum_verified_types "50"` (`:10`), and
holds 72 `benchmark_case` records across 9 categories plus **10**
`benchmark_limitation` records at `:891-980` — the issue names seven; the file
records ten:

| Record | Line | Prompt | Observed | Upstream cause |
| --- | --- | --- | --- | --- |
| `irrational_roots` | `:891` | `Solve x^2 - 2 = 0` | `calculation_error` | rational roots only |
| `complex_roots` | `:900` | `Solve x^2 + 1 = 0` | `calculation_error` | no complex-root support |
| `degenerate_no_solution` | `:909` | `Solve 0 * x = 5` | `calculation_error` | contradiction ≠ "no solution" |
| `identity_equation` | `:918` | `Solve x = x` | `unknown` | no calculation signal; router declines |
| `malformed_expression` | `:927` | `Solve x + = 4` | `calculation_error` | intentional: malformed input must never yield an answer |
| `unit_carrying_unknown` | `:936` | `Solve x kg = 1000 g` | `calculation_error` | units not converted |
| `unit_carrying_constant` | `:945` | `Solve 2 * x = 10 kg` | `calculation_error` | unit on the constant not stripped |
| `named_unknown_if_clause` | `:954` | `What is x if x + 7 = 12?` | `calculation_error` | `x if` declaration not stripped |
| `named_unknown_for_clause` | `:963` | `Calculate x for 6 * x = 42` | `calculation_error` | `x for` declaration not stripped |
| `named_unknown_colon_clause` | `:972` | `Find x: 5 * x = 45` | `agent_suggestion` | **`Find` is a shell command name, so the agent router claims the prompt before the calculator sees it** |

The last is a routing bug, not a math one: `solver_terminal::try_terminal_command`
at `src/solver.rs:621-625` claims the prompt. The whole equation capability is
one function, `evaluate_linear_equation` (`src/calculation.rs:375-401`), reached
only after `link-calculator` fails (`src/calculation.rs:426-433`).

The corpus's honesty discipline is exemplary and must be preserved:
`tests/unit/specification/equation_corpus.rs` asserts that each limitation still
does **not** produce a `"calculation"` intent, so a fix fires the test and forces
promotion rather than silently changing a number.

### Scheduled measurement is stale

`data/benchmarks/external-results.lino` holds 60 result rows. The `2026-09-15`
refresh touched only `humaneval` (`:862-872`) and `mbpp` (`:873-883`). GSM8K,
MATH, object counting, CoEdIT, egg, Ascent and SWE-bench were last measured
`2026-09-07` at `0.347.0`. `ratchet::stagnant`
(`src/external_benchmarks/ratchet.rs:158-189`) will already be emitting a
three-identical-rows warning for each of them — a warning only
(`src/cli_benchmark.rs:309-311`).

### The universal loop still cannot fetch

`record_external_search` (`src/solver.rs:874-884`) logs
`policy:no_fetch_capability` — "external search requested but no retrieval was
executed" — and returns. Retrieval exists only inside specific handlers. B1 owns
this; B8 depends on it for the "retrieve the method" step of a non-coding task.

---

## Root causes

1. **The gate is a Python recognizer, not a verifiability test.**
   `src/coding/task_spec.rs:190-193` requires a literal `python` token; the other
   two branches require Python syntax. *Mechanism:* the entire
   discover/compose/verify/remember machinery is behind a syntactic Python test,
   so no amount of improvement inside it can reach a word problem.

2. **There is no shared "task with a checkable answer" type.** Only
   `CodingTaskSpec` (`src/coding/task_spec.rs:48`) and the harness-side
   `Expectation` (`src/external_benchmarks/cases.rs:11`) exist, and the latter is
   not an input to anything. *Mechanism:* a handler cannot ask "does my answer
   satisfy the task?" because the task does not carry what would satisfy it, so
   every non-coding answer is unverified by construction.

3. **The answer is rendered, never executed.** Coding tasks execute a program in
   `AgentWorkspace` and let the exit code decide
   (`src/coding/composition.rs:521-563`). Word problems format a string
   (`src/solver_handlers/mod.rs:149-156`); counting formats `count.to_string()`
   (`src/solver_synthesis.rs:294-300`). *Mechanism:* without execution there is
   nothing to verify against, so there is no signal to select among candidates,
   and no evidence to remember.

4. **Non-coding capability is stored as Rust tables and keyword arrays.**
   `OBJECT_CATEGORIES` (9 categories, ~100 English nouns,
   `src/solver_synthesis.rs:345-447`); `INTENT_MARKERS` (13 English phrases,
   `src/solver_handlers/pattern_inference.rs:29-43`); the 10-entry word-problem
   phrase table (`src/calculation_word_problem.rs:584-595`); three authored
   word-problem shapes (`src/calculation_word_problem.rs:560-565`); the
   USD→EUR-only converter with a hard-coded `0.92` fallback
   (`src/solver_handlers/compound_interest.rs:15, 396-404`). *Mechanism:* each is
   a memorized, monolingual, un-sourced, un-forgettable answer store — the exact
   pattern B9 counts as handler debt and the doctrine forbids.

5. **Recognition is substring matching on English.** `"i have "` / `"how many"`
   (`src/solver_synthesis.rs:254-255`); `lowered.contains` over `INTENT_MARKERS`
   (`src/solver_handlers/pattern_inference.rs:56, 91`). *Mechanism:* a Russian,
   Hindi, Chinese or Spanish paraphrase of the same task cannot route at all, so
   the five-language generalization requirement is unreachable for these families
   no matter how good the reasoning behind them is.

6. **Ordering accidents claim prompts before the capable handler sees them.**
   `Find x: 5 * x = 45` is claimed by the terminal-command router
   (`src/solver.rs:621-625`, recorded at
   `data/benchmarks/equation-type-corpus.lino:972-980`). *Mechanism:* precedence
   is a flat 59-name list (`data/seed/handler-precedence.lino`) with no notion of
   "this task has a verifiable expectation, so prefer a route that can verify".

7. **The prompt-to-grader contract is asymmetric.** MATH's grader wants
   `\boxed{…}` (`src/external_benchmarks/grade.rs:144, 559-577`) while its prompt
   asks for nothing (`src/external_benchmarks/cases.rs:118`); GSM8K's grader wants
   a trailing number (`grade.rs:140-143`) while its prompt asks for nothing
   (`cases.rs:106`). *Mechanism:* even a correct derivation can score zero on
   presentation. This must be fixed on the *answer* side — by the solver knowing
   the expectation's shape — not by rewriting upstream prompts, which would make
   the score non-upstream.

8. **Nothing about a non-coding task is remembered.**
   `DiscoveredProcedureLedger` (`src/coding/discovered_procedures.rs`) is keyed
   on `spec_identity(spec: &CodingTaskSpec)`. *Mechanism:* a word problem solved
   once teaches nothing, so the forget/rediscover proof — the doctrine's
   definition of non-memorized knowledge — cannot even be attempted outside
   coding.

9. **Retrieval is absent from the universal loop.**
   `src/solver.rs:874-884`. *Mechanism:* "retrieve the method" has no
   implementation for a task that is not already inside a coding handler, so the
   middle step of the five-step meta algorithm is missing for every non-coding
   family.

---

## Solution options

### Option A — Add a handler per suite family

**Description.** Write `try_math_word_problem`, `try_object_count`,
`try_instructed_text_edit` as new entries in `HANDLER_FUNCTIONS`
(`src/solver_dispatch.rs:286-385`) and `data/seed/handler-precedence.lino`,
each with its own recognizer and its own answer format.

**Architecture sketch.** Three new files under `src/solver_handlers/`, three new
precedence names, three new response-template groups in
`data/seed/multilingual-responses-*.lino`.

**Pros.** Familiar, incremental, fast to a higher number. Each handler is small
and independently testable. No change to the solver's spine.

**Cons.** It is precisely the pattern #1138 B9 counts as the problem: "each new
capability tends to arrive as another handler, which is exactly the memoization
the doctrine forbids". It raises the 37-file / 48-entry migration backlog of
#699→#959 rather than lowering it, and the 18,466-line handler ceiling.
Nothing is verified, nothing is remembered, nothing generalizes across families.

**Doctrine fit.** Generalization: fails. No hard-coding: fails. Honesty: passes.
Associative stack: fails (new `try_*` arms). Forget/rediscover: fails.

**Effort.** Low. **Risk.** Low technically, certain doctrine violation.

### Option B — Teach the existing calculator and text handlers more shapes

**Description.** Extend `normalize_word_problem_detailed`
(`src/calculation_word_problem.rs:550`) with more authored shapes, widen
`OBJECT_CATEGORIES`, loosen `TextRequest::parse_with_history`'s quoted-operand
requirement, and fix the ten equation limitations one by one.

**Pros.** Smallest blast radius. The equation-corpus ratchet already forces each
fix to be promoted honestly. Some of it — the `Find x:` routing bug in
particular — is genuinely a bug fix that should happen regardless.

**Cons.** GSM8K problems are arbitrary multi-step narratives; there is no finite
set of shapes. Every widening of `OBJECT_CATEGORIES` adds English nouns to a
Rust `const`. Loosening the text handler's operand requirement makes it claim
prompts it cannot serve, which lowers other scores. Still no verification, still
no memory, still English.

**Doctrine fit.** Generalization: fails. No hard-coding: fails badly (the
category table grows). Honesty: passes. Associative stack: partial.
Forget/rediscover: fails.

**Effort.** Medium, unbounded. **Risk.** Medium — widened recognizers cause
cross-family misroutes (#745/#758, B10).

### Option C — One verifiable-task route: recognise → discover → compose → verify → remember, with the composed program executed to produce the answer (selected)

**Description.** Generalize `CodingTaskSpec` into a `VerifiableTask` carrying a
`TaskExpectation`, make *that* the entrance to the discovery path, and make the
final step *execution*: the composed `ProgramIr` is lowered and run, and its
output is the answer. A GSM8K question becomes a program that computes a number;
a counting question becomes a program that counts; a CoEdIT instruction becomes a
program that transforms the text. The expectation says what shape the answer must
have, so presentation is derived rather than guessed.

**Architecture sketch.**

```
prompt ──▶ recognise_verifiable(prompt) ──▶ VerifiableTask { expectation, quantities, subject }
                                                  │
                     ┌────────────────────────────┤
                     ▼                            ▼
              concept_discovery            RegistryConceptLookup (B1)
                     │                            │
                     └──────────▶ ConceptMap ◀────┘
                                      │
                                program_ir::elaborate  (B2)
                                      │
                                   ProgramIr
                                      │
                             ir_lowering::python
                                      │
                          AgentWorkspace: run the program
                                      │
                    ┌─────────────────┴──────────────────┐
                    ▼                                    ▼
      output satisfies the self-check          output does not
                    │                                    │
        render in the expectation's shape        next candidate / honest gap
                    │
        remember in the verifiable-task ledger
```

**Pros.** One route for every family, so improving it improves all of them.
Execution supplies the selection signal that today only coding has. Expectation
shape fixes the MATH/GSM8K presentation asymmetry without touching upstream
prompts. It makes the forget/rediscover proof applicable outside coding for the
first time. It lets `OBJECT_CATEGORIES` be deleted: category membership becomes a
retrieved fact (Wikidata `P279` subclass-of / Wiktionary hypernym), which is what
the sources registry is for.

**Cons.** The largest surface of the three. Executing a derived program to answer
a natural-language question is strictly harder than formatting a string, and will
produce many honest gaps before it produces many answers. Self-verification
without a gold answer is weaker than a unit test: the check can only be internal
consistency (does the program run, does it produce the expectation's shape, does
it agree with a second independently derived candidate), never correctness.

**Doctrine fit.** Generalization: yes — one path, five families, five languages.
No hard-coding: yes — the category table and keyword arrays are deleted, not
extended. Honesty: yes, and it is the option that most sharply distinguishes
"answered" from "verified". Associative stack: yes — the task, expectation, IR
and ledger are all `.lino`. Forget/rediscover: yes.

**Effort.** High. **Risk.** Medium-high.

### Option D — Keep the handlers, add a verification wrapper

**Description.** Leave every handler as it is, but wrap the dispatch so that when
a task carries a `TaskExpectation` the produced answer is checked for shape (a
number, a count, an edited text) and rejected if it does not fit.

**Pros.** Cheapest route to honest *presentation*: MATH's `\boxed{}` and GSM8K's
trailing number become derivable, which alone may move a score. Very low risk.

**Cons.** It verifies the *shape* of an answer, never its derivation. It leaves
every root cause in place — no execution, no retrieval, no memory, English-only
recognition, memorized tables. It is a presentation fix marketed as a reasoning
fix, which is the honesty failure mode the doctrine names.

**Doctrine fit.** Generalization: fails. No hard-coding: fails. Honesty: risky —
a score that rises for presentation reasons must be labelled as such, or the
number misleads. Associative stack: neutral. Forget/rediscover: fails.

**Effort.** Low. **Risk.** Low technically; high doctrinally.

---

## Decision

**Selected: Option C.** Option D's shape-derivation is adopted *inside* it as the
rendering rule (`TaskExpectation::render`), and Option B's `Find x:` routing fix
is adopted as an early leaf, because it is a real bug whose fix is independently
correct.

Reasons:

1. B8's stated fix is literally Option C: "the discovery path is invoked for any
   task with a verifiable expectation, not only for `CodingTaskSpec`". A and B
   leave the `?` at `src/solver_handlers/program_synthesis.rs:29` in place and so
   cannot satisfy the requirement as written.
2. It is the only option that *deletes* memorized data. `OBJECT_CATEGORIES`
   (`src/solver_synthesis.rs:345-447`) and `INTENT_MARKERS`
   (`src/solver_handlers/pattern_inference.rs:29-43`) go away under C and grow
   under A and B. The #959 handler ratchet must turn downward each release; C is
   the only option that turns it downward.
3. Execution is what makes an answer evidence. The coding path's 20/20 is
   credible precisely because a program ran and an exit code decided
   (`src/coding/composition.rs:521-563`). Extending that to counting and text
   editing is the same argument applied one level out — and it is what B5
   (obligations need runtime evidence) will need anyway.
4. Five-language generalization is unreachable under A and B: substring gates on
   `"i have "` cannot be paraphrased into Hindi. Under C, recognition is a
   seed-role and meaning question, which the repository already does well
   (`src/coding/catalog/mod.rs:200-243` for script-aware matching).

Rejections:

- **Option A rejected** because a new `try_*` arm per suite is the precise thing
  the doctrine forbids and #959 is ratcheting down. It would also make B9 worse
  while claiming to fix B8.
- **Option B rejected as the plan** (its `Find x:` fix adopted as leaf L1) because
  the shape sets are unbounded and the fixes are monolingual and un-sourced.
  Extending `OBJECT_CATEGORIES` in particular is memorization with extra steps.
- **Option D rejected as the plan** (its rendering rule adopted as a component)
  because a score that moves for presentation reasons, without a derivation
  behind it, is the dishonest kind of improvement. It is acceptable only as one
  step *of* a derivation, which is how C uses it.

---

## Architecture

### New and changed files

| Path | Status | Purpose |
| --- | --- | --- |
| `src/verifiable_task.rs` | new | `VerifiableTask`, `TaskExpectation`, `recognise_verifiable`, rendering |
| `src/verifiable_task/quantities.rs` | new | seed-driven quantity/entity extraction, five languages |
| `src/verifiable_task/ledger.rs` | new | content-addressed verifiable-task procedure memory |
| `src/solver_handlers/verifiable_task.rs` | new | the single dispatch entry, registered once |
| `src/coding/task_spec.rs` | changed | `CodingTaskSpec` gains `From<&VerifiableTask>`; `recognise` unchanged |
| `src/coding/synthesis_runtime.rs` | changed | `discover_and_compose` accepts a `VerifiableTask` |
| `src/solver_synthesis.rs` | changed | `compose_object_count` and `OBJECT_CATEGORIES` **deleted** |
| `src/solver_handlers/pattern_inference.rs` | changed | `INTENT_MARKERS` replaced by seed roles |
| `src/solver.rs` | changed | terminal-command route yields to a verifiable task (`:621-625`) |
| `data/seed/meanings-verifiable-task.lino` | new | expectation cues in en/ru/hi/zh/es |
| `data/seed/handler-precedence.lino` | changed | one new name, placed before `text_manipulation` |
| `data/benchmarks/verifiable-task-paraphrases.lino` | new | the held-out five-language set |

No collisions: `VerifiableTask`, `verifiable_task`, `TaskExpectation`,
`AnswerShape`, `ExpectationKind`, `recognise_verifiable`, `VerifiedAnswer`
return zero `grep -rn` matches across `src/`, `data/`, `tests/`.

### The shared task type

```rust
// src/verifiable_task.rs

/// What kind of observation would settle this task — the shape the **answer**
/// must take.
///
/// This is *not* plan 05's `ObligationExpectation`, which declares what must be
/// **observed** before an obligation node may be called satisfied. The two are
/// bridged once, here, so a verifiable task's satisfaction flows through plan
/// 05's ledger and there is exactly one satisfaction rule in the tree:
///
/// ```rust
/// impl TaskExpectation {
///     /// `check_id` is `"<VerifiedAnswer::derivation_id>:<check slug>"`.
///     #[must_use]
///     pub fn to_obligation_expectation(&self, check_id: &str)
///         -> crate::obligation_ledger::ObligationExpectation;
/// }
/// ```
///
/// **reconciled: plan 05 risk 2 left `SymbolicCheck { check_id }` undefined and
/// asked for it to be settled jointly with plan 12 before either lands; this
/// plan now owns `check_id` and plan 12's `CandidateScore::checks` counts the
/// satisfied `Evidence` rows of the same set (plan 00 §9 R15).**
///
/// Deliberately *not* the upstream `Grading` enum: this is what the prompt
/// itself declares, discovered from seed cues, never from a suite id. The two
/// happen to align for benchmark cases, which is why benchmark scores measure
/// this route rather than steering it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TaskExpectation {
    /// A single number is the answer. `unit` is `Some` when the prompt names one.
    Numeric { unit: Option<String> },
    /// A cardinality over a described collection.
    Count { subject: String },
    /// A transformed version of a supplied text.
    EditedText { source: String },
    /// A value for a named unknown, as an equation states it.
    Unknown { name: String },
    /// A named callable checked by examples — today's `CodingTaskSpec`.
    Callable { examples: Vec<crate::coding::task_spec::Example> },
    /// A process whose stdout is the observation.
    Stdout { expected: Option<String> },
    /// A yes/no claim.
    Boolean,
}

/// How an answer of this expectation must be presented.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnswerShape {
    /// The number stands alone at the end of the answer.
    TrailingNumber,
    /// The value is delimited the way the prompt's own convention requires.
    DelimitedValue,
    /// The edited text is the whole answer body.
    WholeBody,
    /// A fenced program.
    CodeBlock,
}

/// A task whose answer can be checked, in any domain.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiableTask {
    /// The prompt as received, unmodified.
    pub prompt: String,
    /// Sentences the requirement decomposes into — the same field
    /// `CodingTaskSpec::requirement_sentences` carries.
    pub requirement_sentences: Vec<String>,
    pub expectation: TaskExpectation,
    pub shape: AnswerShape,
    /// Named quantities the prompt states, in order of appearance.
    pub quantities: Vec<Quantity>,
    /// Entities the prompt lists, for `Count` tasks.
    pub entities: Vec<Entity>,
    /// Detected prose language slug (`en`, `ru`, `hi`, `zh`, `es`).
    pub prose_language: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Quantity {
    pub value: String,
    pub unit: Option<String>,
    pub label: Option<String>,
    /// Byte offset in `prompt`, so the trace can point at the evidence.
    pub offset: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entity {
    pub surface: String,
    /// Multiplicity the prompt states for this entity, default 1.
    pub multiplicity: String,
    pub offset: usize,
}

impl VerifiableTask {
    /// Stable identity for ledger recall: normalized sentences + expectation
    /// kind + quantity shape. Deliberately excludes literal values, so a
    /// renumbered paraphrase recalls the same *procedure* and recomputes.
    #[must_use] pub fn identity(&self) -> String;
    #[must_use] pub fn to_links_notation(&self) -> String;
    /// Present `value` the way `shape` requires, in `prose_language`.
    #[must_use] pub fn render(&self, value: &str, reasoning: &str) -> String;
}

/// Recognize any task carrying a checkable expectation.
///
/// Returns `None` for an open-ended request. Every cue is a seed meaning from
/// `data/seed/meanings-verifiable-task.lino`; no suite id, no benchmark name,
/// and no hard-coded English phrase appears here.
#[must_use] pub fn recognise_verifiable(prompt: &str) -> Option<VerifiableTask>;
```

`identity()` excluding literal values is the anti-memorization property that
matters: two GSM8K problems with the same narrative shape and different numbers
share a procedure and must both be *recomputed*, never recalled as an answer.

### Recognition is seed data, in five languages

`data/seed/meanings-verifiable-task.lino` declares the cues, following the shape
of `data/seed/meanings-coding-structure.lino`:

```
meanings
  expectation_numeric
    defined-by expectation
    role verifiable_expectation
    shape trailing_number
    grounding "https://en.wikipedia.org/wiki/Word_problem_(mathematics_education)"
    lexeme en
      surface "how much"
      surface "how many"
      surface "what is the total"
    lexeme ru
      surface "сколько"
      surface "какова сумма"
    lexeme hi
      surface "कितना"
      surface "कितने"
    lexeme zh
      surface "多少"
      surface "一共有多少"
    lexeme es
      surface "cuánto"
      surface "cuántos"
  expectation_count
    defined-by expectation
    role verifiable_expectation
    shape trailing_number
    …
  expectation_edited_text
    defined-by expectation
    role verifiable_expectation
    shape whole_body
    …
  expectation_unknown_value
    defined-by expectation
    role verifiable_expectation
    shape delimited_value
    …
```

Matching uses the existing script-aware helpers — `contains_cjk`
(`src/coding/catalog/mod.rs:200`) and `contains_devanagari`
(`src/coding/catalog/mod.rs:225`) — so Hindi and Chinese resolve without spaces,
exactly as coding-task recognition already does. Quantity extraction reuses
`crate::seed::lexicon().arithmetic_normalization_tables()`
(already used at `src/coding/composition.rs:645-654`) for spelled-out numerals in
all five languages, rather than the ASCII-digit scan of
`src/coding/composition.rs:637-643`.

The three `Solve x …` limitation prompts that carry a named unknown
(`data/benchmarks/equation-type-corpus.lino:954, 963, 972`) become
`TaskExpectation::Unknown { name: "x" }` through an `expectation_unknown_value`
cue whose lexemes include the `if`/`for`/`:` declaration forms — as *seed
surfaces*, not as Rust string literals.

### Entering the discovery path

```rust
// src/coding/task_spec.rs (changed)

impl From<&crate::verifiable_task::VerifiableTask> for CodingTaskSpec {
    /// Project a verifiable task onto the coding spec the discovery path
    /// already consumes. The artifact is always a `Program` whose stdout is the
    /// answer, except for `Callable`, which keeps today's function shape.
    fn from(task: &crate::verifiable_task::VerifiableTask) -> Self;
}
```

The projection is what lets B8 reuse *all* of B2's work without a second
pipeline:

| `TaskExpectation` | `artifact_shape` | `name` | `parameters` | `expected_stdout` |
| --- | --- | --- | --- | --- |
| `Numeric` | `Program` | `main` | none (quantities become literals in the IR) | `None` |
| `Count` | `Program` | `main` | none | `None` |
| `EditedText` | `Program` | `main` | none (source text is an IR literal) | `None` |
| `Unknown` | `Program` | `main` | none | `None` |
| `Callable` | `Function` | as parsed | as parsed | `None` |
| `Stdout` | `Program` | `main` | none | as parsed |

`recognise` (`src/coding/task_spec.rs:126`) is **not** changed: it remains the
Python-shape recognizer, and `recognise_verifiable` calls it first, so every
prompt that works today keeps working through exactly the same branch.

`src/solver_handlers/verifiable_task.rs`:

```rust
pub fn try_verifiable_task(prompt: &str, normalized: &str, log: &mut EventLog)
    -> Option<SymbolicAnswer>
{ try_verifiable_task_with_online(prompt, normalized, log,
    crate::coding::synthesis_runtime::live_fetch_enabled()) }

pub fn try_verifiable_task_with_online(
    prompt: &str, normalized: &str, log: &mut EventLog, online: bool,
) -> Option<SymbolicAnswer>;
```

registered in `HANDLER_FUNCTIONS` (`src/solver_dispatch.rs:286`) and in
`data/seed/handler-precedence.lino` as `verifiable_task`, placed immediately
before `text_manipulation` (currently 17th) so a verifiable edit instruction is
seen before the quoted-operand parser declines it, and well before `arithmetic`
(34th). Because `specialized_handlers()` asserts the seed is an exact permutation
of the native table (`src/solver_dispatch.rs:399-421`), both edits land in the
same commit.

`src/solver.rs:621-625` changes so `try_terminal_command` does not claim a prompt
for which `recognise_verifiable` returned `Some` — the `Find x:` fix, and the
general rule that a verifiable expectation outranks a lexical route.

### Compose, then **execute to produce the answer**

```rust
// src/solver_handlers/verifiable_task.rs

/// The composed program's run, and what it observed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiedAnswer {
    /// The value the program printed.
    pub value: String,
    /// The lowered source that produced it, for the trace and the ledger.
    pub source: String,
    /// The IR content id, so the same derivation is recognizable across runs.
    pub derivation_id: String,
    pub source_urls: Vec<String>,
    pub source_licenses: Vec<String>,
    /// Which self-checks passed, each as the observation that proves it.
    ///
    /// **reconciled: was `Vec<String>`; now `Vec<Evidence>` (plan 00 §4.3),
    /// because a self-check recorded as a string is a claim, not an
    /// observation, and plan 00 §6.6 requires the evidence (plan 00 §9 R14).**
    pub checks: Vec<crate::execution_evidence::Evidence>,
}

/// Lower every candidate IR, run it in the bounded workspace, and keep the
/// answers that satisfy the expectation's self-checks. Ranked by action cost.
fn execute_candidates(
    task: &VerifiableTask,
    candidates: &[crate::coding::program_ir::ProgramIr],
    log: &mut EventLog,
) -> Vec<VerifiedAnswer>;
```

Execution is the existing contract, unchanged:
`AgentWorkspace::for_prompt` with a 5-second budget, `create_file`,
`run_command("python3 solution.py")`, exit code decides
(`src/coding/composition.rs:521-563`). What differs is that the program's
**stdout is the answer**, not a test result.

**Self-checks** — what "verify" can honestly mean without a gold answer:

1. **Ran.** Exit code 0, not timed out. (Always available.)
2. **Shape.** The output parses as the expectation requires: a number for
   `Numeric`/`Count`, a non-empty text for `EditedText`, a value for `Unknown`.
3. **Agreement.** Two independently derived candidates — different `ProgramIr`
   content ids, disjoint fragment sets — produce the same value. This is the
   strongest available check and is what replaces a unit test.
4. **Round trip**, where the expectation admits one. For `Unknown { name }`:
   substitute the value into the stated equation and evaluate both sides through
   `evaluate_calculation` (`src/calculation.rs:425`). For `Count`: the count may
   not exceed the number of entities the prompt lists. For `EditedText`: the
   result must differ from the source and must preserve the source's non-edited
   spans.
5. **Unit consistency.** For `Numeric { unit: Some(u) }`, the program's own
   declared output unit matches `u`.

`checks` is recorded verbatim in the answer trace and in the ledger. An answer
that passed only check 1 is rendered with the seeded "computed, not
independently corroborated" wording; it never says "verified". An answer that
passes no check is not returned at all — the honest gap is.

### Remembering, forgetting, rediscovering

```rust
// src/verifiable_task/ledger.rs

/// A verified derivation for a *shape* of task, not for a case.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiedProcedure {
    pub id: String,
    pub task_identity: String,
    pub expectation: String,
    pub derivation_id: String,
    pub fragments: Vec<String>,
    pub source_urls: Vec<String>,
    pub checks: Vec<String>,
    pub verified_at: String,
    pub integrity_sha256: String,
}

pub struct VerifiableTaskLedger { path: std::path::PathBuf }

impl VerifiableTaskLedger {
    #[must_use] pub fn new(cache_directory: impl AsRef<std::path::Path>) -> Self;
    pub fn recall(&self, task: &VerifiableTask) -> std::io::Result<Option<VerifiedProcedure>>;
    pub fn remember(&self, task: &VerifiableTask, answer: &VerifiedAnswer)
        -> std::io::Result<VerifiedProcedure>;
}
```

Deliberately mirrors `DiscoveredProcedureLedger`
(`src/coding/discovered_procedures.rs:27-130`) including
`identity_payload`/`expected_integrity`/`valid` so the tamper-detection and
content-addressing properties are the same code shape. Crucially it stores the
**derivation**, never the value: a recall replays the `ProgramIr` and *re-runs*
it against this prompt's quantities. A renumbered paraphrase therefore recalls
the procedure and computes a different, correct number — which is the
generalization test, and is impossible to pass by memorization.

The forget/rediscover proof is the same round trip B2 defines for fragments:
delete the ledger and the fragment cache, replay offline from committed captures,
assert the identical `derivation_id`.

### Deleting the memorized tables

- `OBJECT_CATEGORIES` and `compose_object_count`
  (`src/solver_synthesis.rs:248-301, 339-447`) are **deleted**. Category
  membership becomes a retrieved fact: for each `Entity`, the discovery path asks
  the sources registry whether the entity is a subclass/hyponym of the question's
  named category — Wikidata `P279` (`data/seed/sources-registry.lino` `wikidata`),
  Wiktionary hypernyms, or WordNet synsets, all three already declared. The
  composed program counts entities whose retrieved membership holds, multiplied by
  their stated multiplicity — which also fixes the "three trumpets" bug the
  current code cannot express.
- `INTENT_MARKERS` (`src/solver_handlers/pattern_inference.rs:29-43`) is replaced
  by a `verifiable_expectation` seed role; the 13 English phrases become lexemes
  in five languages.
- The 10-entry phrase table (`src/calculation_word_problem.rs:584-595`) and the
  three authored shapes (`:560-565`) stay for now as *one candidate generator
  among others* — #315's own wording — but are demoted below the derived
  candidate and are covered by a ratchet that forbids growth.
- `USD_EUR_FALLBACK_RATE = 0.92` (`src/solver_handlers/compound_interest.rs:15`)
  is a fabricated number in production data. It must become a retrieved rate with
  provenance, or the handler must decline. Out of this plan's critical path but
  named here because it is the same class of defect.

### Browser/WASM parity

Recognition, quantity extraction, concept discovery, elaboration and type
checking are pure and compile to WASM. Execution does not: the browser has no
Python runtime. The browser surface therefore:

- recognises the task and shows the formalized `VerifiableTask` and the derived
  step list with provenance;
- renders the composed program;
- states **unverified** and does not present a computed value as the answer.

That is strictly more than today (the four families currently produce nothing in
the browser) and it is honest. The existing worker budgets
(`scripts/check-worker-line-budget.rs`, `scripts/check-wasm-worker-size.rs`)
bound the addition; no JavaScript reimplementation of the reasoning is permitted
— the doctrine treats worker-side logic as transitional
(`docs/case-studies/issue-710/plans/02-dynamic-discovery-design.md` §7).

### Failure and honesty behavior

- `recognise_verifiable` returning `None` leaves every existing route untouched.
  The new handler is strictly additive.
- No candidate passing check 1 ⇒ the localized skill gap plus the research trail,
  listing the phrases searched, the fragments found and the check that failed —
  the shape `src/solver_handlers/program_synthesis.rs:62-88` already produces.
- An answer passing only "ran" and "shape" is labelled as computed-but-uncorroborated.
  The word "verified" is reserved for agreement or round-trip.
- The equation-corpus limitations keep their honest-failure assertions: the test
  at `tests/unit/specification/equation_corpus.rs` asserting a limitation does
  **not** produce a `"calculation"` intent must be *updated deliberately* when a
  limitation is fixed, with the fix promoted into a `benchmark_case` — never
  silently.
- The `Grading` enum (`src/external_benchmarks/manifest.rs:52-69`) must remain
  invisible to `src/verifiable_task.rs`. A grep gate asserts no
  `external_benchmarks` import appears under `src/verifiable_task*`, so the
  solver cannot learn the shape of the grader.
- Determinism: candidate order is action cost then derivation id; execution is in
  a cleared workspace; retrieval is content-addressed.

---

## Tests first

### Held-out cases, five languages, actual prompt text

New fixture `data/benchmarks/verifiable-task-paraphrases.lino`, modelled on
`data/benchmarks/coding-discovery-paraphrases.lino` (#710, 25 cases). No sentence
below appears in any upstream suite or any seed file — asserted by the extended
no-memorization gate.

**Case 1 — multi-step arithmetic narrative (`TaskExpectation::Numeric`).**

- en: `A baker makes 24 rolls each morning and 18 each afternoon. She sells 35 rolls during the day and gives 4 to her neighbour. How many rolls does she have left at closing time?`
- ru: `Пекарь печёт 24 булочки утром и 18 днём. За день она продаёт 35 булочек и отдаёт 4 соседке. Сколько булочек остаётся у неё к закрытию?`
- hi: `एक बेकर हर सुबह 24 और हर दोपहर 18 रोल बनाती है। वह दिन भर में 35 रोल बेचती है और 4 अपनी पड़ोसन को देती है। बंद होने के समय उसके पास कितने रोल बचते हैं?`
- zh: `一位面包师每天早上做 24 个面包卷，下午做 18 个。她白天卖出 35 个，又送给邻居 4 个。打烊时她还剩多少个面包卷？`
- es: `Una panadera hace 24 panecillos cada mañana y 18 cada tarde. Vende 35 durante el día y le da 4 a su vecina. ¿Cuántos panecillos le quedan al cerrar?`

**Case 2 — counting with multiplicity and an unseeded category
(`TaskExpectation::Count`; passing requires retrieved category membership, since
`OBJECT_CATEGORIES` has no such category).**

- en: `I have two oboes, a stethoscope, three scalpels, a harp and a thermometer. How many medical instruments do I have?`
- ru: `У меня есть два гобоя, стетоскоп, три скальпеля, арфа и термометр. Сколько у меня медицинских инструментов?`
- hi: `मेरे पास दो ओबो, एक स्टेथोस्कोप, तीन स्कैल्पेल, एक वीणा और एक थर्मामीटर है। मेरे पास कितने चिकित्सा उपकरण हैं?`
- zh: `我有两支双簧管、一个听诊器、三把手术刀、一架竖琴和一支体温计。我有多少件医疗器械？`
- es: `Tengo dos oboes, un estetoscopio, tres bisturís, un arpa y un termómetro. ¿Cuántos instrumentos médicos tengo?`

**Case 3 — instructed text edit (`TaskExpectation::EditedText`, no quoted
operand, which today declines outright).**

- en: `Make this sentence more concise: The committee members who were present at the meeting all agreed unanimously with one another about the proposal.`
- ru: `Сделай это предложение короче: Члены комитета, которые присутствовали на заседании, все единогласно согласились друг с другом по поводу предложения.`
- hi: `इस वाक्य को अधिक संक्षिप्त बनाओ: बैठक में उपस्थित समिति के सभी सदस्य प्रस्ताव पर एक-दूसरे से सर्वसम्मति से सहमत थे।`
- zh: `把这句话改得更简洁：出席会议的委员会成员全都一致地彼此同意该提案。`
- es: `Haz esta frase más concisa: Los miembros del comité que estuvieron presentes en la reunión estuvieron todos de acuerdo unánimemente entre sí sobre la propuesta.`

**Case 4 — named unknown with a colon declaration
(`TaskExpectation::Unknown`; this is the `Find x:` misroute of
`data/benchmarks/equation-type-corpus.lino:972-980`, paraphrased so it is
held out from the corpus).**

- en: `Find y: 7 * y = 84`
- ru: `Найди y: 7 * y = 84`
- hi: `y ज्ञात करो: 7 * y = 84`
- zh: `求 y：7 * y = 84`
- es: `Halla y: 7 * y = 84`

**Case 5 — a number with a unit that must be converted
(`TaskExpectation::Numeric { unit }`; the `unit_carrying_unknown` limitation
class at `data/benchmarks/equation-type-corpus.lino:936-953`, paraphrased).**

- en: `A tank holds 2 litres of water and 750 millilitres are poured out. How many millilitres remain?`
- ru: `В баке 2 литра воды, из него вылили 750 миллилитров. Сколько миллилитров осталось?`
- hi: `एक टंकी में 2 लीटर पानी है और उसमें से 750 मिलीलीटर निकाल दिया जाता है। कितने मिलीलीटर बचते हैं?`
- zh: `一个水箱里有 2 升水，倒出 750 毫升。还剩多少毫升？`
- es: `Un depósito contiene 2 litros de agua y se vierten 750 mililitros. ¿Cuántos mililitros quedan?`

**Case 6 — an honest gap.** A verifiable-looking question whose method no trusted
source supplies, in all five languages, asserted to produce the skill gap with a
research trail and **no** numeric answer.

### Unit, integration and specification tests

| Path | Name | Asserts |
| --- | --- | --- |
| `tests/unit/verifiable_task/recognition.rs` | `each_expectation_kind_is_recognised_in_five_languages` | all five cases above, `TaskExpectation` and `AnswerShape` correct per language |
| `tests/unit/verifiable_task/recognition.rs` | `an_open_ended_request_is_not_a_verifiable_task` | `recognise_verifiable` returns `None`; existing routes untouched |
| `tests/unit/verifiable_task/recognition.rs` | `recognition_names_no_english_literal` | grep `src/verifiable_task*` for quoted natural-language strings; zero |
| `tests/unit/verifiable_task/quantities.rs` | `spelled_out_numerals_extract_in_five_languages` | "two"/"два"/"दो"/"两"/"dos" all yield `2` with an offset |
| `tests/unit/verifiable_task/quantities.rs` | `multiplicity_is_attached_to_its_entity` | case 2 yields `oboe×2`, `scalpel×3`, others ×1 |
| `tests/unit/verifiable_task/identity.rs` | `a_renumbered_paraphrase_shares_the_task_identity` | two cases differing only in numbers share `identity()` |
| `tests/unit/verifiable_task/identity.rs` | `identity_carries_no_literal_value` | no quantity value appears in `identity()` |
| `tests/unit/verifiable_task/execution.rs` | `the_composed_program_is_executed_and_its_output_is_the_answer` | trace shows the run, `VerifiedAnswer::value` equals stdout |
| `tests/unit/verifiable_task/execution.rs` | `two_independent_derivations_that_disagree_yield_a_gap` | disagreement ⇒ no answer, honest gap |
| `tests/unit/verifiable_task/execution.rs` | `an_answer_with_only_the_ran_check_is_not_called_verified` | wording assertion in five languages |
| `tests/unit/verifiable_task/execution.rs` | `an_unknown_value_round_trips_through_its_equation` | case 4: substituting the value satisfies both sides |
| `tests/unit/verifiable_task/ledger.rs` | `a_verified_derivation_is_content_addressed_and_tamper_detecting` | mirrors `coding_discovery::ledger` |
| `tests/unit/verifiable_task/ledger.rs` | `a_recalled_procedure_recomputes_rather_than_replays_a_value` | recall on a renumbered paraphrase produces the *new* correct number |
| `tests/unit/verifiable_task/ledger.rs` | `forgotten_derivations_are_rediscovered_to_the_same_id` | forget → rediscover offline → identical `derivation_id` |
| `tests/unit/verifiable_task/routing.rs` | `a_named_unknown_outranks_the_terminal_command_route` | case 4 is not claimed by `try_terminal_command` |
| `tests/unit/verifiable_task/routing.rs` | `an_edit_instruction_reaches_the_verifiable_route_before_text_manipulation` | case 3 routes to `verifiable_task` |
| `tests/unit/verifiable_task/routing.rs` | `no_existing_handler_loses_a_prompt_it_answers_today` | the full `conversational-variations-suite.lino` (228 cases) is unchanged |
| `tests/unit/verifiable_task/no_memorization.rs` | `object_categories_are_absent_from_the_runtime` | the deleted `OBJECT_CATEGORIES` may not return, under any name |
| `tests/unit/verifiable_task/no_memorization.rs` | `the_solver_never_imports_the_benchmark_grader` | no `external_benchmarks` import under `src/verifiable_task*` |
| `tests/unit/specification/equation_corpus.rs` | *(extend)* `every_limitation_is_still_honest_or_promoted` | a fixed limitation must have moved to `benchmark_case` in the same commit |
| `tests/unit/specification/external_benchmarks.rs` | *(extend)* `non_coding_rows_are_refreshed_at_the_current_solver_version` | the four suites' newest rows are not older than the newest coding row |
| `tests/source/source_tests/verifiable_task/…` | placement tests | new modules obey the source-placement convention |

### Gates and ratchets

1. **No-memorization gate (existing, extended).**
   `tests/unit/coding_discovery/no_memorization.rs` scans `src/` and
   `data/seed/` for upstream entry-point names and task sentences
   (`:26-56, :124`). Extend it to cover GSM8K `question` sentences, MATH
   `problem` sentences, BIG-bench `input` strings and CoEdIT `src`/`tgt` strings
   — the four suites it currently ignores entirely. This is the single most
   important new gate in this plan: without it, "route every verifiable task
   through discovery" is indistinguishable from "add the four suites' contents to
   the seed".
2. **Category-table ratchet (new).** No Rust `const` under `src/` may hold a list
   of natural-language nouns longer than a small bound. Enforced by extending the
   hard-coded-language checker (`scripts/check-hardcoded-language.rs`), whose
   allowlist currently stands at 1,286 entries and **must shrink** by the entries
   this plan deletes rather than grow.
3. **Word-problem-shape ratchet (new).** The count of authored shapes in
   `src/calculation_word_problem.rs` may not rise. Every new word-problem
   capability must arrive as a derivation.
4. **Handler-count ratchet (existing, #959).** This plan adds one handler entry
   (`verifiable_task`) and removes `compose_object_count` plus the
   `INTENT_MARKERS` recognizer; the net must not increase the 48-entry migration
   backlog.
5. **Equation-corpus ratchet (existing, #891).** `minimum_pass_count` 72 and
   `minimum_verified_types` 50 may only rise
   (`data/benchmarks/equation-type-corpus.lino:9-10`); a limitation that is fixed
   must be promoted to a `benchmark_case` in the same commit.
6. **Upstream ratchet (existing, #698).** `benchmark ratchet`
   (`src/external_benchmarks/ratchet.rs:16, 61`) — no recorded pass count may
   fall, no floor may be lowered.
7. **Browser honesty gate (new).** The worker's verifiable-task output must carry
   the unverified label; asserted in `tests/web/`.

### Benchmark commands and the honest numbers expected

```sh
# Re-measure the four stale suites at the current solver version.
cargo run --bin formal-ai -- benchmark run --suite gsm8k           --slice 20 --online
cargo run --bin formal-ai -- benchmark run --suite math            --slice 20 --online
cargo run --bin formal-ai -- benchmark run --suite object_counting --slice 20 --online
cargo run --bin formal-ai -- benchmark run --suite coedit          --slice 20 --online

# Cold-cache controls: the cache must not be the capability.
FORMAL_AI_CACHE_DIR=$(mktemp -d) \
  cargo run --bin formal-ai -- benchmark run --suite gsm8k --slice 20

# Wider slices, once the route exists, to see whether 20 was representative.
cargo run --bin formal-ai -- benchmark run --suite gsm8k --slice 200 --online
cargo run --bin formal-ai -- benchmark run --suite coedit --slice 200 --online

# Regression controls: the coding rows may not fall.
cargo run --bin formal-ai -- benchmark run --suite humaneval --slice 20 --online
cargo run --bin formal-ai -- benchmark run --suite mbpp      --slice 20 --online

# The equation corpus and its ten recorded limitations.
cargo test --test unit issue_891_equation_corpus -- --nocapture

# Held-out five-language suite, routing regressions, gates.
cargo test --test unit verifiable_task -- --nocapture
cargo test --test unit conversational_variation_benchmark_routes_every_case -- --nocapture
cargo test --test unit coding_discovery::no_memorization -- --nocapture

# Append whatever was measured.
cargo run --bin formal-ai -- benchmark run --suite all --slice 20 --online --append
```

**What is expected of the numbers: measure and record whatever it is.** No target
appears in this plan, in any test, in any ledger floor or in any document before a
run produces it. The commitments are about process:

- GSM8K's floor of 2 (`data/benchmarks/external-results.lino:63`) may not fall;
  MATH, object counting and CoEdIT stand at 0 (`:80, :96, :113`) and 0 is a
  number, not a failure to report.
- A re-measurement that produces the same number is still worth appending: it
  dates the row at the current solver version and clears the `stagnant` warning's
  ambiguity between "unchanged" and "unmeasured".
- If the route raises one suite and lowers another, both are recorded and the
  lowered one is a regression to fix in capability, never by lowering a floor.
- A score that rises only because the answer's *presentation* changed (the
  `AnswerShape` rendering) must be labelled as such in the PR and in
  `docs/benchmarks.md`. A presentation gain is real but is not a reasoning gain,
  and conflating them would be the dishonest outcome this plan most needs to
  avoid.

## Exact reconciliation checkpoint — 2026-09-17

This checkpoint was written before changing the remaining implementation or
checkboxes. The working tree already contains substantially more than the stale
leaf ledger records:

- L8–L12 are represented by the registered `verifiable_task` interpreter and
  its derive/execute/check pipeline in
  `src/solver_handlers/verifiable_task.rs`; it records process evidence, shape,
  independent agreement, equation/count/edit round trips and unit consistency.
- L14 is represented by recall, tamper, recompute and forget/rediscover tests in
  `tests/unit/verifiable_task/ledger.rs`.
- L15–L16 are represented by source-registry category lookup with multiplicity,
  deletion of the old object-category table, and the category-table ratchet in
  `tests/unit/verifiable_task/no_memorization.rs`.
- L20 is represented by the browser recognizer/formalizer/IR renderer in
  `src/web/worker/formal_ai_worker_verifiable_task.js` and the explicit
  `unverified:browser_execution_unavailable` web contract.
- L21 is represented by the authored-shape ceiling and routing-order ratchet in
  `tests/unit/verifiable_task/ratchets.rs`.

Those leaves remain unchecked below until their focused tests and static gates
run against this exact combined tree. L1 is genuinely incomplete: the colon
named-unknown corpus row is still a limitation and the floor is still 72 even
though the generic route now wins dispatch. L19 is also incomplete: the current
gate prohibits the retired category table and grader imports but does not scan
the pinned upstream non-coding case sentences. L22–L24 remain measurement work,
not implementation claims; no score or document will be updated without the
corresponding online/cold run evidence.

---

## Implementation leaves

- [ ] **L1** — Fix the `Find x:` misroute: `src/solver.rs:621-625` yields when the
      prompt carries a named-unknown declaration. Promote
      `named_unknown_colon_clause`
      (`data/benchmarks/equation-type-corpus.lino:972-980`) from limitation to
      `benchmark_case` in the same commit; raise `minimum_pass_count` to 73.
- [x] **L2** — Add `data/seed/meanings-verifiable-task.lino` with the four
      expectation meanings in en/ru/hi/zh/es and a `verifiable_expectation` role;
      no code consumes it yet.
- [x] **L3** — Add `src/verifiable_task.rs`: `TaskExpectation`, `AnswerShape`,
      `VerifiableTask`, `identity`, `to_links_notation`. No recognizer yet.
- [x] **L4** — Add `src/verifiable_task/quantities.rs`: `Quantity`, `Entity`,
      five-language numeral and multiplicity extraction over
      `arithmetic_normalization_tables`.
- [x] **L5** — Add `recognise_verifiable`, delegating to
      `crate::coding::task_spec::recognise` first so today's Python path is
      byte-identical; add the five-language recognition tests.
- [x] **L6** — Add `VerifiableTask::render` and the `AnswerShape` rules; localized
      wording in `data/seed/multilingual-responses-*.lino`, no literals in Rust.
- [x] **L7** — Add `impl From<&VerifiableTask> for CodingTaskSpec` with the
      projection table above.
- [x] **L17'** — *(moved ahead of L8 by the reconciliation)* Replace
      `INTENT_MARKERS` (`src/solver_handlers/pattern_inference.rs:29-43`) with
      the `verifiable_expectation` seed role in five languages; retire
      `pattern_inference` from `HANDLER_FUNCTIONS` and
      `data/seed/handler-precedence.lino`; shrink the hard-coded-language
      allowlist by the removed entries.
      **reconciled: was L17, after L8. `recognise_verifiable`'s expectation
      recognition *is* what `INTENT_MARKERS` approximates, so retiring
      `pattern_inference` first frees the dispatch slot `verifiable_task` takes,
      and `try_dispatch_entries` never rises. Plan 00 §6.2 forbids a rise and
      plan 09 makes the ratchet strict; raising a ceiling for one commit would
      have been a loosened gate (plan 00 §9 R12).**
- [ ] **L8** — Add `src/solver_handlers/verifiable_task.rs` with
      `try_verifiable_task_with_online`; register it in the slot L17' vacated, in
      `HANDLER_FUNCTIONS` (`src/solver_dispatch.rs:286`) and
      `data/seed/handler-precedence.lino`, in one commit; declare it a **generic
      interpreter** in `data/meta/core-boundary-ledger.lino`, not a handler;
      assert the 228-case conversational suite is unchanged. This leaf lands
      after plan 09 leaves 1-5 have made the ratchet strict.
- [ ] **L9** — Add `execute_candidates` and `VerifiedAnswer`: lower, run in
      `AgentWorkspace`, stdout is the answer. Checks 1 and 2 only.
- [ ] **L10** — Add check 3 (agreement between two disjoint derivations) and the
      disagreement-yields-a-gap test.
- [ ] **L11** — Add check 4 (round trip) per expectation kind, including the
      equation substitution for `Unknown` and the entity bound for `Count`.
- [ ] **L12** — Add check 5 (unit consistency); wire unit conversion as a
      retrieved fact, addressing the `unit_carrying_*` limitation class
      (`data/benchmarks/equation-type-corpus.lino:936-953`).
- [x] **L13** — Add `src/verifiable_task/ledger.rs` mirroring
      `DiscoveredProcedureLedger`; store the derivation, never the value.
- [ ] **L14** — Add the recall-recomputes test and the forget → rediscover →
      same-`derivation_id` round trip, offline from committed captures.
- [ ] **L15** — Retrieved category membership for `Count`: query the registry's
      `wikidata` / `wiktionary` / `wordnet` records for subclass/hypernym
      evidence; the composed program counts by retrieved membership × stated
      multiplicity.
- [ ] **L16** — **Delete** `compose_object_count` and `OBJECT_CATEGORIES`
      (`src/solver_synthesis.rs:248-301, 339-447`) and the call site at `:115`.
      Add the category-table ratchet. **L15 must be green in the same commit: the
      curated 13/13 industry slice contains an object-counting case the table
      answers today, and the ratchet will correctly refuse a fall (plan 00
      §9 X4; this plan's risk 7, now an ordering constraint in plan 14).**
- [ ] ~~**L17**~~ — struck through: moved ahead of L8 as **L17'** so the
      dispatch slot is freed before it is filled (plan 00 §9 R12).
- [x] **L18** — Add the 30 held-out five-language cases as
      `data/benchmarks/verifiable-task-paraphrases.lino` plus the routing and
      recognition suites.
- [ ] **L19** — Extend the no-memorization gate to GSM8K questions, MATH
      problems, BIG-bench inputs and CoEdIT source/target strings.
- [ ] **L20** — Browser/WASM parity: recognise, formalize, derive, render,
      label unverified; worker budgets respected; web honesty test.
- [ ] **L21** — Add the word-problem-shape ratchet and demote the three authored
      shapes below the derived candidate
      (`src/calculation_word_problem.rs:560-565`).
- [ ] **L22** — *(lands after plan 02 L22, which teaches the ledger its second
      floor, so a re-measurement writes into a schema that already knows two
      slices — plan 00 §9 X8)* Re-measure all four suites at slice 20, online and cold-offline;
      re-run the two coding controls; append every row; record the failure
      frontier through `--frontier-record` (`src/cli_benchmark.rs:57-61`).
- [ ] **L23** — Run the wider slices (200) for GSM8K and CoEdIT and append, so the
      first-20 representativeness question is answered with a number.
- [ ] **L24** — Update every document listed below with the measured values.

---

## Docs to update

The exact quoted statements and their replacement text moved to plan 11's
findings table on 2026-09-16, so there is one docs authority and no document
is described in two places (plan 00 §8). This plan's entries are rows
**D226-D238** of
[`11-docs-consistency-audit.md`](11-docs-consistency-audit.md) §"Issue #1138
plan doc replacements", and plan 11's leaves apply them after the ledger rows
they cite exist (plan 00 §7).

| row | document |
| --- | --- |
| D226 | `VISION.md:343` |
| D227 | `ROADMAP.md:145` |
| D228 | `ROADMAP.md:179` |
| D229 | `docs/benchmarks.md:284-295` |
| D230 | `docs/benchmarks.md:307-312` |
| D231 | `docs/benchmarks.md:16-33` |
| D232 | `docs/benchmarks.md:31` |
| D233 | `docs/requirements/issue-0891-equation-corpus-ratchet.md` |
| D234 | `docs/requirements/issue-0698-real-external-benchmark-harness.md` |
| D235 | New shard `docs/requirements/issue-1138-verifiable-task-routing.md` |
| D236 | `docs/requirements-traceability.md` |
| D237 | `docs/meta-algorithm.md:144-167` |
| D238 | `docs/meta-algorithm.md:214-218` |

Any further document this plan's implementation touches is added as a new
plan 11 row, never as a second copy here.

## Risks and open questions

1. **Self-verification is weaker than a unit test.** For HumanEval the upstream
   test decides; for a GSM8K question nothing external does. Agreement between two
   disjoint derivations is the strongest available check and it is not
   correctness — two derivations can share a wrong premise. The mitigation is
   labelling, not pretending. Open question: is there a principled third check —
   for instance, deriving the answer and then deriving the *question's own stated
   quantities back* from it — that is cheap enough to run per case?
2. **Routing regressions.** Inserting `verifiable_task` before
   `text_manipulation` in a 59-name precedence list risks claiming prompts other
   handlers answer today. The 228-case conversational suite
   (`docs/benchmarks.md:33`), the 1,440-case edit profile (`:20`) and the 56-case
   local-path suite (`:29`) are the regression floor, and all three must be green
   before L8 merges. Open question: should placement be *after*
   `text_manipulation` with a fallthrough instead, trading some CoEdIT coverage
   for zero risk?
3. **Retrieved category membership is slow and sometimes wrong.** A Wikidata
   `P279` walk per entity per case is many requests; Wiktionary hypernyms are
   inconsistent. The content-addressed cache makes replays cheap, but the first
   online run is expensive. Open question: what is the honest depth bound, and
   what does the answer say when membership is genuinely ambiguous (is a
   stethoscope a "medical instrument"? is a harp?) — a count with an ambiguous
   member should probably decline rather than pick.
4. **CoEdIT's `ExactText` grading is unforgiving.** `normalize`
   (`src/external_benchmarks/grade.rs:581-590`) lowercases and collapses
   whitespace, but a correct edit that differs from the gold rewrite still scores
   zero. The suite may stay near zero even with a working edit capability. That
   must be stated when the number is published, and it is a reason to also report
   a held-out edit measure this repository controls.
5. **MATH needs symbolic output, not a number.** `BoxedAnswer` compares a
   normalized expression (`grade.rs:144, 546-577`). Executing a Python program
   that prints a float will not match `\frac{3}{4}`. The `AnswerShape` rendering
   must produce the expression, which means the IR must carry exact arithmetic.
   Open question: is `fractions.Fraction` / `decimal` sufficient, or does MATH
   need the symbolic kernel (#923, `src/proof_engine/`) rather than this route?
6. **Executing a derived program per case is slow.** 20 cases × 4 suites × a
   5-second workspace budget, times several candidates each, is minutes per suite.
   At slice 200 it is much more. Open question: how the scheduled workflow is
   shaped, and whether candidate execution should be bounded by *count* rather
   than by time — noting the doctrine forbids budgets as a way of giving up, so
   any bound must produce an honest gap rather than a silent truncation.
7. **Deleting `OBJECT_CATEGORIES` may lower a curated score before it raises an
   upstream one.** The curated industry slice (13/13,
   `docs/benchmarks.md:18`) includes an object-counting case that the table
   currently answers. If the retrieved-membership route is not ready in the same
   commit, that 13/13 falls — and the ratchet will correctly refuse it. L15 must
   therefore land before L16, and the two must be verified together.
8. **The gate must grow with the route.** Extending no-memorization to GSM8K,
   MATH, BIG-bench and CoEdIT strings will flag ordinary English in seed response
   templates, because those suites' text is ordinary English. Open question: the
   principled predicate. Proposal: forbid any upstream *sentence* of more than N
   words verbatim, and any upstream `target`/`tgt` value in any form — the answer
   side is where memorization actually hides.
9. **B1 and B2 dependencies.** Without B1's live lookup, "retrieve the method" is
   limited to what is already cached; without B2's IR and lowering there is
   nothing to compose into. This plan is written to consume both and to degrade
   honestly without them (recognition, projection and execution of *seeded*
   derivations still work), but its headline claim needs both. Open question:
   sequence B1 → B2 → B8 strictly, or land L1-L8 of this plan early because they
   are independently valuable routing fixes?
10. **Handler-count accounting.** This plan adds one handler and deletes two
    recognizers, which should move #959's ratchet downward — but
    `verifiable_task` is a large handler, and the 18,466-line ceiling and the
    minimal-core boundary (`scripts/check-minimal-core-boundary.rs`) both apply.
    Open question: does `src/verifiable_task*` belong inside the minimal core (it
    is the meta algorithm's own route) or outside it (it is a capability)?
11. **`USD_EUR_FALLBACK_RATE = 0.92`** (`src/solver_handlers/compound_interest.rs:15`)
    is a fabricated number that production code will emit as an answer. It is not
    on this plan's critical path, but it is the same class of defect as the
    category table and should be filed rather than left unnamed.
