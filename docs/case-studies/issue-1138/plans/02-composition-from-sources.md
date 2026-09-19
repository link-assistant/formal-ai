# Plan 02 — Composition from retrieved sources, not a closed template catalog (bottleneck B2 of #1138)

## Issues addressed

- **#1138 B2** — "Composition is a closed template catalog, not reconstruction
  from sources … the composer takes its parts from what B1 and the function
  catalogs retrieved … the seeded idioms shrink to a bootstrap that can be
  deleted and rediscovered … then run the full 164 and 500 upstream suites and
  record the honest number, whatever it is." This plan delivers all four
  clauses: retrieval-to-step-list, a language-neutral program IR, a deletable
  idiom bootstrap with a content-hash round trip, and full-suite runs.
- **#315 (E30, closed)** — asked that a `write a program` request be answered by
  "deriving a candidate function body (composing known link-native code
  fragments via E28), not a seed lookup", and that "memorized seeds [stay] only
  as one candidate generator among others". Delivered partially: derivation
  exists (`src/coding/composition.rs:46`) but the *combinations* are still
  hand-written Rust (`src/coding/structural_composition.rs:25-341`). This plan
  delivers the remaining half: the combination itself is derived from retrieved
  procedure text rather than enumerated in code.
- **#317 (E32, closed)** — asked for a benchmark suite grown "beyond 5 cases"
  with a monotonic pass-count floor and held-out paraphrases. Delivered for the
  curated slice; this plan extends the same ratchet discipline to the *full*
  upstream suites (164 HumanEval, 500 MBPP) rather than their first 20.
- **#698 (E56, closed)** — R529 "Report honest `passed / total` per suite against
  the upstream case set — 0% is acceptable, fake floors are not". The harness
  exists; it has only ever been run at `--slice 20`
  (`data/benchmarks/external-results.lino`, all 60 result rows). This plan
  delivers the full-suite measurement the requirement's wording already implies.
- **#938 (E86, closed)** — asked that coding handlers share one
  meta-algorithm-builder rather than each carrying a bespoke construction. The
  shared builder exists (`src/coding/catalog/mod.rs:36-38`,
  `src/solver_handlers/program_synthesis.rs:30`); this plan keeps every new
  module on it and adds no second construction surface.
- **#923 (E76, closed)** — established the "widen the kernel, score it honestly
  through the #698 harness" discipline that this plan applies to composition.
- **Issues plan 13's coverage table names this plan as a deliverer of** (added by
  the 2026-09-16 reconciliation): **#948** (the idiom catalog shrinks, so the
  memoized-answer surface burns down with plan 09); **#1071** (counting to any
  number and the text-only reasoning tasks reach the composer through plan 08's
  route); **#722** (composition of a long-form answer from retrieved parts);
  **#940** (the research-to-document half, minus the PDF/DOCX rendering that is
  blocked upstream); **#827** / E5 (three correct sources are fetched and their
  titles emitted — fusion into a composed answer is this plan's stage 3).
- **#710 R710-D1/D2/D3/D9/D10** (`docs/requirements/issue-0710-dynamic-coding-discovery.md`)
  — the first-20 rows and the forget/rediscover ledger. This plan promotes D2/D3
  from "first 20" to "whole suite" and extends D10 from the *procedure* ledger
  to the *idiom seed* itself.

---

## Current state

### The template catalog is closed, and its size is exactly measurable

`src/coding/catalog/mod.rs:44-50` declares `TEMPLATE_GROUPS` as five static
slices. Counting `ProgramTemplate {` literals in each file:

| File | Templates | Distinct task slugs |
| --- | ---: | --- |
| `src/coding/catalog/templates_core.rs` | 46 | `count_to_three`, `hello_world`, `list_files`, `list_files_arg` |
| `src/coding/catalog/templates_extended.rs` | 65 | `factorial`, `fibonacci`, `fizzbuzz`, `reverse_string`, `sum_to_ten` |
| `src/coding/catalog/templates_listing.rs` | 26 | `list_files_reverse_sort`, `list_files_arg_reverse_sort` |
| `src/coding/catalog/templates_stdin.rs` | 13 | `copy_stdin_to_stdout` |
| `src/coding/catalog/templates_framework.rs` | 1 | `hello_world` (Laravel only) |
| **total** | **151** | **12** |

`src/coding/catalog/tasks.rs` holds 12 `ProgramTask` records;
`src/coding/catalog/languages.rs` holds 14 `ProgramLanguage` records.
`program_template_count()` (`src/coding/catalog/mod.rs:58-61`) is the
diagnostic that already exposes this number. Resolution is a two-key lookup:
`program_template(task_slug, language_slug)`
(`src/coding/catalog/mod.rs:74-79`) — a task with no `(task, language)` row
reaches the honest dead end described at
`src/coding/catalog/templates_framework.rs:15-18`.

The composite-blueprint escape hatch is equally closed:
`src/coding/blueprint_data.rs` holds 27 `Capability` records, 5
`BlueprintRecipe` records and 7 `RecipeProgram` rows, backed by 7
`pub(super) const` program bodies in `src/coding/blueprint_programs.rs`.
`src/coding/blueprint.rs:21-31` describes the selection as keyword detection →
recipe match → render, and `src/coding/blueprint.rs:29-31` is explicit that the
rendered program is "not run — needs external libraries / network".

`src/knowledge.rs` `CodingOracle` holds **7** `OracleSnippet` entries
(`src/knowledge.rs:170-234`), not the 25 that
`docs/case-studies/issue-710/plans/02-dynamic-discovery-design.md` §2 recorded —
six `hello_world` bodies plus one Kotlin factorial. The module doc at
`src/knowledge.rs:13-15` still states the live-refresh path "would materialise a
per-source `data/cache/<source-slug>/` bucket … until it runs, the embedded
snapshots are the cache of record". Grep confirms no caller refills it: the only
reader is `src/solver_handler_oracle.rs`.

### The discovery composer is real, but its combinations are hand-enumerated

The path that produced 20/20 is:
`recognise` (`src/coding/task_spec.rs:126`) → `discover`
(`src/coding/concept_discovery.rs:189`) → `compose`
(`src/coding/composition.rs:46`) → `verify`
(`src/coding/composition.rs:500`) → `DiscoveredProcedureLedger::remember`
(`src/coding/discovered_procedures.rs`), driven by
`src/solver_handlers/program_synthesis.rs:29-133`.

Three of its four draft generators are genuinely source-derived:

- `candidate_draft` (`src/coding/composition.rs:128-179`) wraps a `stdlib`,
  `wikifunctions_implementation`, `wikifunctions_recurrence` or `source_program`
  candidate. Action costs 1/2/3/4 respectively.
- `extend_with_sequence_programs` (`src/coding/synthesis_runtime.rs:158-200`)
  promotes OEIS-derived programs into `source_program` candidates.
- `discovery_catalog` (`src/coding/synthesis_runtime.rs:23-156`) fetches
  Wikifunctions and the Python stdlib index through `CachedSourceClient`.

The fourth is not. `structural_drafts`
(`src/coding/composition.rs:203-412`) contains **9** hand-written
`if has("…") && has("…") && names.len() == N` blocks, and
`additional_drafts` (`src/coding/structural_composition.rs:12-342`) contains
**31** more — 40 authored shape combinations in total, at
`src/coding/structural_composition.rs:25, 35, 58, 83, 91, 96, 101, 111, 123,
131, 144, 149, 165, 182, 186, 194, 207, 217, 233, 250, 260, 280, 293, 303, 313,
321, 325, 333, 338` and `src/coding/composition.rs:220, 232, 286, 307, 324,
345, 364, 386, 393`. Each block hard-codes both the *selection* condition and
the *assembly order*. Nothing generalizes past the 40 shapes: an unseen
combination of already-seeded meanings produces no draft at all.

### The idiom seed is not a bootstrap; parts of it are answers

`data/seed/meanings-coding-structure.lino` holds **82** top-level meanings, of
which **57** carry `role coding_structure` and **42** carry an `idiom` line; the
remaining 17 are `defined-by concept` prefix records
(`data/seed/meanings-coding-structure.lino:1-45`). Loaded by
`structural_meanings()` (`src/coding/concept_discovery.rs:241-258`) from a
`include_str!` at `src/coding/concept_discovery.rs:11`.

`data/seed/coding-discovery-runtime.lino` holds **27** `template` records,
loaded by `runtime_template` (`src/coding/python_render.rs:11, 31-56`). Seven of
them are not idioms at all — they are whole multi-statement algorithms:
`balanced_delimiter_groups`, `group_max_nesting`, `remove_boundary_occurrences`,
`rotation_period`, `one_bit_difference`, `grid_minimum_cost_path` and
`oeis_linear_recurrence`. A seed entry whose body is a complete solution is a
memorized answer wearing a meaning's name, even when no benchmark identifier
appears in it. The no-memorization gate does not catch these because it only
forbids upstream *entry-point names* and *docstring sentences*
(`tests/unit/coding_discovery/no_memorization.rs:26-56`).

`data/seed/coding-idioms.lino` (583 lines) is a separate, older per-language
scaffold/idiom table for the numeric-list composer of #395; its `description`
field (`data/seed/coding-idioms.lino:2`) states its scope is "the universal list
coding algorithm". It carries no source provenance fields at all.

Neither seed file can be deleted today: `include_str!` makes both compile-time
dependencies (`src/coding/concept_discovery.rs:11`,
`src/coding/python_render.rs:11`), and `idiom()`/`template()` *panic* on a
missing id (`src/coding/composition.rs:715, 721`).

### The retrieval side exists but is not wired to composition

`data/seed/sources-registry.lino` declares 13 sources
(`wikidata, wiktionary, wordnet, wikipedia, wikihow, stackexchange,
wikifunctions, rosetta_code, wikibooks, wikiversity, wikivoyage, github,
wikinews`). **OEIS and `docs.python.org` are absent from it**, although
`src/coding/function_catalog/oeis.rs:16` and
`src/coding/function_catalog/python_docs.rs:12` both fetch them — two trusted
sources used in production without a registry declaration.

`UnknownConceptLookup` (`src/coding/concept_discovery.rs:81-83`) has exactly one
implementation, `NoLookup` (`src/coding/concept_discovery.rs:85-91`), and
`discover()` (`:189-191`) passes it. So the one hook designed to bring in an
unknown concept is wired to a constant `None`.

`src/how_to_guide.rs` already contains the retrieval-to-step-list machinery this
plan needs and that composition does not use: `select_sources`
(`src/how_to_guide.rs:296-321`), `synthesize_how_to_guide`
(`src/how_to_guide.rs:325-…`), `GuideStep` with full provenance
(`src/how_to_guide.rs:127-152`), `extract_steps`
(`src/how_to_guide/extract.rs:301`), `GuideBounds`
(`src/how_to_guide.rs:40-64`) and `MIN_ACCEPTED_STEPS`
(`src/how_to_guide.rs:35`).

`src/coding_research_learning.rs` is the other near-miss: `SourceProcedure::parse`
(`src/coding_research_learning.rs:236-267`) rejects any page whose first line is
not `SOURCE_HEADER = "Formal AI coding procedure"`
(`src/coding_research_learning.rs:28, 239`). It can therefore only read pages
this repository wrote — exactly the "it has never learned from a real page"
verdict in `docs/case-studies/issue-710/plans/02-dynamic-discovery-design.md` §2.

### What happens today for a HumanEval task beyond the first 20

The benchmark payload cache (`target/formal-ai-benchmarks/`) is absent from this
worktree, so tasks 25 and 100 are not quotable here. The behaviour is
nevertheless fully determined by the code, and it is the same for every index:

1. `fetch::fetch_records` caches the **whole** `HumanEval.jsonl.gz`
   (`src/external_benchmarks/fetch.rs:20-23` — the `slice` parameter is named
   `_slice` and ignored), so records 21-163 are already on disk after any run.
2. `parse_cases` truncates at `slice` (`src/external_benchmarks/cases.rs:43-51`).
   At `--slice 164` every record becomes a case.
3. Each prompt is
   `"Complete this Python function. Reply with the full implementation in a
   ```python code block.\n\n{prompt}"`
   (`src/external_benchmarks/cases.rs:66-68`).
4. `solver.solve(&case.prompt)` (`src/external_benchmarks/mod.rs:156`).
   `recognise` matches the `def` shape
   (`src/coding/task_spec.rs:133-148`) and mines `>>>` doctests into `Example`s.
5. `discover` matches the requirement sentences against the 57 role-carrying
   structural meanings (`src/coding/concept_discovery.rs:260-293`) and against
   whatever Wikifunctions/stdlib/OEIS candidates the catalog returned.
6. `compose` builds drafts. For a task whose shape is not one of the 40 authored
   `if has(...)` blocks and whose need does not resolve to a single wrappable
   stdlib symbol or Wikifunctions implementation, **zero drafts are produced**.
7. `outcome.selected` is `None`, so
   `src/solver_handlers/program_synthesis.rs:62-88` renders the localized
   `program_skill_gap` prose plus a research trail. That body contains no
   ```` ```python ```` fence and no `def ` or `import ` token.
8. `extract_python` (`src/external_benchmarks/grade.rs:484-489`) returns `None`,
   and the case fails with the exact string `"answer contains no Python code"`
   (`src/external_benchmarks/grade.rs:60`).

That is the predicted failure mode for the great majority of tasks 21-163 and
21-499, and it is the same failure taxonomy
`docs/case-studies/issue-710/plans/05-benchmark-capability-generalization.md`
recorded for the *first* 20 before the 40 blocks were written: "Every other case
fails before upstream grading with `answer contains no Python code`."

### What happens today for GSM8K, MATH, object counting and CoEdIT

Summarised here because plan 08 owns them, and because they are the control that
proves the composer is Python-shaped rather than general: their prompts are the
bare upstream strings (`src/external_benchmarks/cases.rs:106, 118, 127, 139`),
`recognise` returns `None` because the conversational branch requires a literal
`python` token (`src/coding/task_spec.rs:190-193`), so the coding composer is
never entered. Latest ledger rows: GSM8K 2/20
(`data/benchmarks/external-results.lino:779-789`), MATH 0/20 (`:790-800`),
object counting 0/20 (`:801-811`), CoEdIT 0/20 (`:812-822`), all dated
`2026-09-07` at solver `0.347.0`.

### Ledger and gate state

`data/benchmarks/external-results.lino` holds 1 harness record
(`:1-15`), 10 suite records (`:16-175`), 60 result rows and 7
`benchmark_unavailable` rows. Every suite record carries `ratchet_slice "20"`
except `ascent_transitive_closure` (`ratchet_slice "5"`, `:144`). Floors:
humaneval 20 (`:31`), mbpp 20 (`:47`), gsm8k 2 (`:63`), math 0 (`:80`),
object_counting 0 (`:96`), coedit 0 (`:113`), egg_math 20 (`:129`), ascent 5
(`:145`), editeval 0 (`:158`), swebench_lite 0 (`:175`).

Two ratchet facts matter for a full-suite run:
`Ledger::raise_floor` early-returns when `ratchet_slice != slice`
(`src/external_benchmarks/ledger.rs:231-234`), and
`historical_floor_violations` groups by `(suite, slice)`
(`src/external_benchmarks/ratchet.rs:126-134`). A `--slice 164` row therefore
starts its own floor at 0 and cannot fail the existing ratchet — which is
honest, but means a new floor must be introduced deliberately rather than
inherited.

The no-memorization gate reads `const SLICE: usize = 20`
(`tests/unit/coding_discovery/no_memorization.rs:5`) and scans `src/` and
`data/seed/` (`:124`). It skips entirely when the payload cache is absent
(`:17-24`).

---

## Root causes

1. **Combination is code, not data.** The 40 `if has(...)` blocks
   (`src/coding/composition.rs:203-412`,
   `src/coding/structural_composition.rs:12-342`) encode both which meanings
   co-occur and how they nest. A new shape requires a new Rust arm, so the
   composer's reach is exactly the number of arms someone wrote. *Mechanism:* the
   search space is an enumeration, not a generated space, so generality cannot
   grow without an author.

2. **There is no intermediate representation between "meanings matched" and
   "Python source".** `idiom()` and `template()`
   (`src/coding/composition.rs:709-723`) do string substitution directly into
   Python text. *Mechanism:* because the only artifact is a Python string,
   `compose` must refuse every other language
   (`src/coding/composition.rs:47-51`, `renderer_unavailable:language=…`), and a
   step list retrieved from prose has nothing to be lowered *into*.

3. **The seeded idioms cannot be forgotten.** `include_str!` at
   `src/coding/concept_discovery.rs:11` and `src/coding/python_render.rs:11`
   makes them compile-time constants, and `idiom()`/`template()` panic on a miss
   (`src/coding/composition.rs:715, 721`). *Mechanism:* "delete the seed and
   rediscover it" is not expressible, so the doctrine's forget/rediscover proof
   can only be run on the *procedure* ledger, never on the knowledge that
   actually does the composing.

4. **Seven runtime templates are complete algorithms, not idioms.**
   `data/seed/coding-discovery-runtime.lino` carries `balanced_delimiter_groups`,
   `group_max_nesting`, `remove_boundary_occurrences`, `rotation_period`,
   `one_bit_difference`, `grid_minimum_cost_path`, `oeis_linear_recurrence`.
   *Mechanism:* each is a stored solution for one task family, so the 20/20 on
   those families measures recall of seed data, not composition — precisely the
   memoization the doctrine forbids, in the one place the gate does not look.

5. **Retrieved procedure text is never turned into steps for coding.** The
   how-to stack already does this (`src/how_to_guide/extract.rs:301`,
   `src/how_to_guide.rs:325`) but composition never calls it, and
   `coding_research_learning` accepts only this repository's own page format
   (`src/coding_research_learning.rs:239`). *Mechanism:* the only route from a
   source to a program is "the source already contains a runnable function"
   (Wikifunctions/OEIS/Rosetta). A source that describes an algorithm in prose —
   Wikipedia, Stack Exchange, a Python-docs narrative — is unusable.

6. **The one lookup hook is a constant `None`.** `NoLookup`
   (`src/coding/concept_discovery.rs:85-91`) is the only implementation.
   *Mechanism:* a need whose phrase matches no seeded meaning is recorded
   `blocked` (`src/coding/concept_discovery.rs:222-229`) and the run ends,
   instead of triggering retrieval. This is B1's bottleneck, and B2 is
   downstream of it: no retrieved parts means nothing new to compose.

7. **The measurement never left the first 20.** `DEFAULT_SLICE = 20`
   (`src/external_benchmarks/mod.rs:36`), CLI default 20
   (`src/cli_benchmark.rs:30-31`), CI `BENCHMARK_SLICE` default `'20'`
   (`.github/workflows/external-benchmarks.yml`), gate `SLICE = 20`
   (`tests/unit/coding_discovery/no_memorization.rs:5`), and every one of the 60
   ledger rows is at slice 20, 5 or 1. *Mechanism:* the number that would reveal
   the catalog's true reach has never been produced, so overfitting to 20 cases
   is invisible from inside the repository.

8. **OEIS and Python docs are trusted in code but undeclared in data.** Neither
   appears in `data/seed/sources-registry.lino`, although
   `src/coding/function_catalog/oeis.rs:16-18` and
   `src/coding/function_catalog/python_docs.rs:12-20` fetch them with hard-coded
   URLs and licenses. *Mechanism:* source policy is supposed to be data
   (`docs/case-studies/issue-710/plans/02-dynamic-discovery-design.md` §3); two
   exceptions mean the registry is not actually the authority, so a new source
   is still a code change.

---

## Solution options

### Option A — Widen the authored combination set

**Description.** Keep the architecture; write more `if has(...)` blocks and more
structural meanings until the full suites score higher. Extend
`data/seed/meanings-coding-structure.lino` and
`data/seed/coding-discovery-runtime.lino` with the families the 21-163 /
21-499 failures reveal.

**Architecture sketch.** No new modules. `src/coding/structural_composition.rs`
splits into `structural_composition_{a,b,c}.rs` to stay under the file-size
limit (`scripts/check-file-size.rs`). The seed grows by roughly one meaning per
failing family.

**Pros.** Zero risk to the existing 20/20. Smallest diff per increment. Every
step is independently verifiable by re-running the slice.

**Cons.** The number of arms is unbounded; HumanEval alone has 164 distinct
shapes. Each arm is a new memorization site. The seven algorithm-shaped runtime
templates would multiply rather than shrink. Nothing becomes forgettable.

**Doctrine fit.** Generalization: fails — this is memoization by another name.
No hard-coding: fails. Honesty: passes (numbers stay real). Associative stack:
partial (seed grows, but so does Rust). Forget/rediscover: fails.

**Effort.** High and open-ended. **Risk.** Low per step, certain failure of the
stated goal.

### Option B — Enumerative program search over the seeded idiom algebra

**Description.** Replace the 40 authored blocks with a bounded generate-and-test
search: enumerate well-typed compositions of the matched structural meanings up
to a depth bound, render each, execute each against the examples, keep the
cheapest that passes. Reuse `src/draft_portfolio.rs` (`plan_drafts`
`src/draft_portfolio.rs:150`, `run_portfolio` `:194`) and
`src/solver_search.rs`'s seeded-PRNG determinism discipline
(`src/solver_search.rs:12-16`).

**Architecture sketch.** New `src/coding/composition_search.rs` producing
`Vec<Draft>` from `ConceptMap` by typed enumeration; the 40 blocks delete;
`structural_composition.rs` shrinks to type signatures per meaning.

**Pros.** Removes the authored enumeration. Genuinely generalizes across unseen
*combinations* of seeded meanings. Deterministic and bounded. Fits the existing
`Draft`/`verify` contract exactly.

**Cons.** Does not add a single new *primitive*: a task needing an operation no
seeded meaning expresses still fails. The idiom seed therefore stays load-bearing
and un-forgettable. Combinatorics grow fast; a depth-3 search over 42 idioms with
argument binding is large enough to need aggressive typing to stay inside the
5-second workspace budget (`src/coding/composition.rs:521-524`).

**Doctrine fit.** Generalization: partial. No hard-coding: partial (the
primitives remain hard-coded). Honesty: passes. Associative stack: good (search
over seed data). Forget/rediscover: fails — deleting the seed still deletes the
capability.

**Effort.** Medium. **Risk.** Medium (search-cost regressions on the passing 20).

### Option C — Retrieved-procedure reconstruction over a language-neutral IR (selected)

**Description.** Introduce the missing middle. Retrieved procedure text becomes
an ordered, provenance-bearing **step list**; a step list is elaborated into a
**language-neutral program IR**; the IR is lowered by per-language renderers; the
rendered program is verified exactly as today. The seeded idioms become one
*bootstrap* source of IR fragments among several retrieved ones, declared in the
sources registry, deletable, and rediscoverable to the same content hash.
Enumerative search (Option B) runs *inside* this, over whatever fragments are
available — seeded or retrieved.

**Architecture sketch.**

```
sources registry ──▶ CachedSourceClient ──▶ ProcedureText
                                              │  extract_steps / ordered-list parse
                                              ▼
                                        Vec<ProcedureStepRecord>
                                              │  elaborate (bind to meanings + parameters)
                                              ▼
              seeded idiom bootstrap ──▶  ProgramIr  ◀── stdlib / Wikifunctions / OEIS parts
                                              │  lower
                                              ▼
                             python::lower | rust::lower | javascript::lower
                                              │
                                              ▼
                                  existing verify() + ledger
```

**Pros.** Every doctrine clause is directly expressible: parts come from
retrieval, combination is data, the seed is a deletable bootstrap, the IR makes
non-Python languages reachable for the first time
(`src/coding/composition.rs:47-51` stops being a dead end), and the forget →
rediscover → same-hash proof applies to the *knowledge*, not only the procedure
record. Reuses `how_to_guide` extraction, `CachedSourceClient`, `AgentWorkspace`,
`validated_program_cst` and `DiscoveredProcedureLedger` unchanged.

**Cons.** The largest design surface of the three. Prose-to-steps extraction is
lossy and will frequently yield nothing usable — which must be reported as a
blocked need, not papered over. The IR is a new abstraction the whole coding
stack must agree on. Full-suite runs will very likely record a *lower* per-case
rate than 20/20 once 164 and 500 cases are in scope, and that number has to be
published.

**Doctrine fit.** Generalization: yes — new primitives arrive by retrieval.
No hard-coding: yes — combination and primitives both become data. Honesty: yes,
and it forces the honest full-suite number. Associative stack: yes — IR, steps
and fragments are `.lino` records with provenance. Forget/rediscover: yes, and it
is the only option in which the seed itself is the thing forgotten.

**Effort.** High. **Risk.** Medium-high, mitigated by keeping the existing
generators as additional draft sources for the whole transition.

### Option D — Delegate composition to an external agentic CLI

**Description.** When the composer produces no draft, hand the task to the
agentic-coding loop (`src/agentic_coding/`, `docs/meta-algorithm.md:186-262`)
and let a client-side tool write the program.

**Pros.** Fastest route to a higher upstream number.

**Cons.** Violates the standing doctrine outright: the answer would not be
derived by this system from trusted sources, and the recipe is already pinned to
constants (`docs/meta-algorithm.md:214-218` — `SEARCH_QUERY`,
`CANONICAL_SOURCE_URL`, `KB_PATH`). It also makes the benchmark number
un-reproducible offline.

**Doctrine fit.** Fails generalization, no-hard-coding and forget/rediscover.

**Effort.** Low. **Risk.** Unacceptable. Recorded for completeness only.

---

## Decision

**Selected: Option C**, with Option B's bounded enumerative search adopted as the
*fragment-combination strategy inside* the IR rather than as a rival.

Reasons:

1. B2's own success criterion is "the composer takes its parts from what B1 and
   the function catalogs retrieved, reconstructs the step list from the retrieved
   procedure text, and the seeded idioms shrink to a bootstrap that can be
   deleted and rediscovered". Only Option C makes each of those three clauses a
   thing the code actually does. A and B leave the seed load-bearing.
2. The forget/rediscover proof is the doctrine's operational definition of "not
   memorized". Today it can only be run on `discovered-procedures.lino`; under
   Option C it runs on `meanings-coding-structure.lino` and
   `coding-discovery-runtime.lino` — the files that actually contain the
   knowledge.
3. The IR is the only way `compose` stops refusing non-Python
   (`src/coding/composition.rs:47-51`). That refusal is the reason the 151-row
   template catalog still exists for the other 13 languages; the IR is what
   eventually lets that catalog shrink too.
4. Option B alone cannot remove the seven algorithm-shaped runtime templates,
   because there is no mechanism to obtain those algorithms from anywhere else.

Rejections:

- **Option A rejected** because it is the failure mode the issue names. Adding
  arms is how the catalog reached 151 templates and 40 blocks; more of it cannot
  produce generality, and it makes the seed less forgettable, not more.
- **Option B rejected as the whole plan** (adopted as a component) because it
  generalizes over combinations only. The full-suite run would expose tasks
  needing primitives nobody seeded, and Option B has no answer for them.
- **Option D rejected** because it bypasses derivation entirely, which the
  standing doctrine forbids without qualification ("no deferral, no budgets, no
  bypasses"), and because it would make the recorded number depend on an
  external model.

---

## Architecture

### New and changed files

| Path | Status | Purpose |
| --- | --- | --- |
| `src/procedure_text.rs` | changed (declared by plan 04 L9) | retrieved page → ordered `ProcedureStepRecord` list with provenance. **reconciled: was `src/procedure_text.rs`, new here; now the top-level module plan 04 L9 declares and this plan L10 extends, because plan 04 lands first in plan 00 §5's order and both plans need one "ordered step with provenance" record — plan 04's `ExtractedProcedure::from_step_records` is its only constructor (plan 00 §9 R10).** |
| `src/coding/program_ir.rs` | new | the language-neutral IR: `ProgramIr`, `IrNode`, typing, cost |
| `src/coding/ir_lowering/mod.rs` | new | `LanguageLowering` trait + registry |
| `src/coding/ir_lowering/python.rs` | new | Python lowering (replaces direct idiom substitution) |
| `src/coding/ir_lowering/rust.rs` | new | second language, to prove the IR is not Python-shaped |
| `src/coding/fragment_catalog.rs` | new | unified fragment store: bootstrap seed + retrieved fragments, content-addressed |
| `src/coding/composition_search.rs` | new | bounded typed enumeration over available fragments |
| ~~`src/coding/source_lookup.rs`~~ | **withdrawn** | **reconciled: was a second `UnknownConceptLookup` implementation walking the registry; now this plan consumes plan 01's single `RegistrySourceLookup` (`src/concept_lookup.rs`), because B1's "fixed means" names one implementation used by two callers and two would re-open the parity defect #991 was filed to remove (plan 00 §9 R3).** |
| `src/coding/composition.rs` | changed | `compose` consumes `ProgramIr`; the 9 authored blocks delete |
| `src/coding/structural_composition.rs` | changed | 31 authored blocks delete; file becomes fragment *typing* declarations |
| `src/coding/concept_discovery.rs` | changed | `discover()` takes a real lookup; `structural_meanings()` reads the fragment catalog |
| `src/coding/python_render.rs` | changed | becomes the Python lowering backend only |
| `src/coding/synthesis_runtime.rs` | changed | `discover_and_compose` gains the procedure-text stage |
| `data/seed/sources-registry.lino` | changed | add `oeis` and `python_docs` with `need_kinds`. **reconciled: was "add `coding_role` per source"; now the existing `need_kinds` axis plan 01 L3 adds, because two selection axes over one registry is plan 01 root cause 5, and the primary/secondary split `coding_role` encoded is exactly what `PrimacyChain::derive_tier` already computes from `primacy` (plan 00 §9 R5).** |
| `data/seed/coding-discovery-runtime.lino` | changed | the 7 algorithm-shaped templates removed |
| `data/seed/meanings-coding-structure.lino` | changed | each meaning gains `bootstrap true` and `rediscovery_query` |
| `data/meta/coding-fragment-ledger.lino` | new (generated, ignored) | content-addressed rediscovered fragments |

No proposed name collides: `procedure_text`, `program_ir`, `ProgramIr`,
`IrNode`, `fragment_catalog`, `composition_search`, `source_lookup`,
`RegistryConceptLookup`, `ProcedureStepRecord`, `LanguageLowering` all return
zero matches under `grep -rn` over `src/`, `data/`, `tests/`. (`ProcedureStep`
alone is taken by `src/skill_procedure.rs`, hence `ProcedureStepRecord`;
`PlanStep` is taken by `src/computer_use/induction.rs`, hence `IrNode`.)

### Retrieved procedure text → step list

```rust
// src/procedure_text.rs

/// One instruction recovered from a retrieved page, with the bytes it came from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcedureStepRecord {
    pub ordinal: usize,
    pub text: String,
    pub source_id: String,
    pub source_url: String,
    pub sha256: String,
    pub fetched_at: String,
    pub license_name: String,
    pub license_url: String,
    pub depth: usize,
}

/// How a page's procedure was recovered, recorded so a replay is auditable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepShape {
    /// `<ol>` / `<li>` items, the wikiHow and Wikipedia "Algorithm" shape.
    OrderedList,
    /// Numbered prose lines ("1." / "Step 2:"), the Stack Exchange shape.
    NumberedProse,
    /// A pseudocode block delimited by `<pre>` / `<code>`.
    PseudocodeBlock,
    /// A definition sentence that states a recurrence or closed form.
    DefinitionSentence,
}

/// Extract an ordered procedure from one capture. Returns `None` rather than a
/// one-step guess: fewer than `MIN_PROCEDURE_STEPS` items is not a procedure.
pub fn steps_from_capture(
    capture: &crate::source_fetch::SourceCapture,
    source: &crate::seed::SourceRecord,
    bounds: &crate::source_walk::LookupBounds,
) -> Option<(StepShape, Vec<ProcedureStepRecord>)>;

/// Retrieve and extract for one need phrase across the registry's coding
/// sources that declare the need kind, in the registry's consultation order
/// (declared kind first, then derived tier descending, then registry order),
/// bounded by `bounds`.
pub fn retrieve_procedure<T: crate::source_fetch::SourceTransport>(
    phrase: &str,
    prose_language: &str,
    client: &crate::source_fetch::CachedSourceClient<T>,
    bounds: &crate::source_walk::LookupBounds,
    log: &mut crate::event_log::EventLog,
) -> Vec<(StepShape, Vec<ProcedureStepRecord>)>;

pub const MIN_PROCEDURE_STEPS: usize = 2;
```

`steps_from_capture` delegates HTML handling to the existing
`crate::how_to_guide::extract::{strip_html, decode_entities, compact_step_text,
extract_steps}` (`src/how_to_guide/extract.rs:177, 196, 238, 301`) so there is
one HTML extractor in the tree, and reuses `MIN_ACCEPTED_STEPS`' discipline
(`src/how_to_guide.rs:35`). Provenance fields mirror `GuideStep`
(`src/how_to_guide.rs:127-152`) field-for-field so the two can merge later.

Per-source contributions, declared as data in `data/seed/sources-registry.lino`
through plan 01 L3's `need_kinds` field plus the tier the source's existing
`primacy` chain already derives. **reconciled: this table was keyed by a new
`coding_role primary/secondary/none` field; now by `need_kinds` and the derived
tier, because two selection axes over one registry is plan 01 root cause 5 and
the primary/secondary split is exactly what `PrimacyChain::derive_tier` computes
(plan 00 §9 R5).**

| Source | `need_kinds` | derived tier | What it yields | Shape |
| --- | --- | --- | --- | --- |
| Python docs (`docs.python.org`, PSF-2.0) | `(part procedure)` | `primary_source` | symbol + one-sentence semantics → an `IrNode::Call` fragment | `DefinitionSentence` |
| Wikifunctions (CC0-1.0 / Apache-2.0) | `(part)` | `primary_source` | a ready implementation or an abstract recurrence | `PseudocodeBlock` |
| OEIS (CC-BY-SA-4.0) | `(part)` | `primary_source` | a formula/recurrence line → `IrNode::Recurrence` | `DefinitionSentence` |
| Rosetta Code (GFDL-1.2) | `(part)` | `independent_corroboration` | a per-language section, quoted with attribution, **never** silently pasted into a generated answer | `PseudocodeBlock` |
| Stack Exchange (CC BY-SA 4.0) | `(concept procedure)` | `editorial_synthesis` | accepted-answer prose steps | `NumberedProse` |
| Wikipedia (CC BY-SA 4.0) | `(concept)` | `editorial_synthesis` | an "Algorithm"/"Method" section's ordered list | `OrderedList` |
| Wiktionary / WordNet / Wikidata | `(concept)` | per `primacy` | concept meaning only, consumed by plan 01's `RegistrySourceLookup` — these declare no composition kind, so `retrieve_procedure` never selects them | — |

The ordering `retrieve_procedure` walks is therefore the one `select_sources`
already computes for every need kind (plan 01): declared kind first, then derived
tier descending, then registry order. There is no second ordering to keep in
sync, which is the whole point of withdrawing `coding_role`.

Share-alike licensing is enforced at lowering time, not at retrieval time: a
GFDL or CC BY-SA capture may contribute an *abstract step* (an `IrNode`) but its
verbatim code may never be emitted into a generated program. `ProgramIr` carries
the constraint so the check is structural:

```rust
// src/coding/program_ir.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReuseMode {
    /// The fragment's text may be emitted verbatim.
    Verbatim,
    /// Only the fragment's abstract shape may be used; text must be re-derived.
    ShapeOnly,
}
```

### The shared intermediate representation

```rust
// src/coding/program_ir.rs

/// A value shape the IR can type-check without committing to a language.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IrType {
    Integer, Float, Boolean, Text,
    Sequence(Box<IrType>),
    Pair(Box<IrType>, Box<IrType>),
    Mapping(Box<IrType>, Box<IrType>),
    /// Unconstrained; unifies with anything exactly once.
    Unknown(usize),
}

/// One node of a language-neutral program.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IrNode {
    Parameter { name: String, ty: IrType },
    Literal { text: String, ty: IrType },
    /// A named fragment applied to arguments. `fragment` is a
    /// `FragmentCatalog` id, never a language-specific symbol.
    Apply { fragment: String, arguments: Vec<IrNode> },
    /// Comprehension over `items`, binding `item`, yielding `body`,
    /// optionally filtered by `predicate`.
    Each { item: String, items: Box<IrNode>, body: Box<IrNode>, predicate: Option<Box<IrNode>> },
    /// Left fold with an initial value.
    Fold { item: String, accumulator: String, items: Box<IrNode>, initial: Box<IrNode>, body: Box<IrNode> },
    /// Bounded iteration with an explicit termination condition.
    Repeat { counter: String, from: Box<IrNode>, to: Box<IrNode>, body: Box<IrNode> },
    /// Named state, base cases and a transition — the recurrence shape OEIS and
    /// Wikifunctions both produce.
    Recurrence { state: Vec<String>, base: Vec<IrNode>, transition: Box<IrNode>, index: Box<IrNode> },
    Condition { test: Box<IrNode>, then_branch: Box<IrNode>, else_branch: Box<IrNode> },
    Bind { name: String, value: Box<IrNode>, body: Box<IrNode> },
    Emit { value: Box<IrNode> },
    Return { value: Box<IrNode> },
}

/// A complete program plan: signature, body, provenance and cost.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProgramIr {
    pub name: String,
    pub parameters: Vec<(String, IrType)>,
    pub result: IrType,
    pub body: IrNode,
    pub fragments: Vec<String>,
    pub source_urls: Vec<String>,
    pub source_licenses: Vec<String>,
    pub reuse: ReuseMode,
}

impl ProgramIr {
    /// Structural cost, the least-action key: node count plus fragment depth.
    #[must_use] pub fn action_cost(&self) -> usize;
    /// Unify parameter and result types through the fragment signatures.
    /// `Err` names the first node that cannot be typed.
    pub fn type_check(&self, catalog: &crate::coding::fragment_catalog::FragmentCatalog)
        -> Result<(), String>;
    /// The canonical `.lino` projection; also the content-hash input.
    #[must_use] pub fn to_links_notation(&self) -> String;
    /// Stable content id over `to_links_notation()`.
    #[must_use] pub fn content_id(&self) -> String;
}
```

Every IR node is representable as nested `.lino` through `push_lino_node`
(`src/links_format.rs`), the same projection `CodingTaskSpec::to_links_notation`
(`src/coding/task_spec.rs:83-121`) and `ConceptMap::to_links_notation`
(`src/coding/concept_discovery.rs:156-185`) already use. The IR is therefore
associative-stack data, not a Rust-only structure.

### Step list → IR

```rust
// src/coding/program_ir.rs

/// Elaborate an ordered step list into candidate IR bodies.
///
/// Each step is matched against the fragment catalog by the same meaning
/// machinery `structures_for` uses, then the steps are threaded: a step whose
/// output type unifies with the next step's input becomes its argument;
/// otherwise the pair becomes a `Bind`. Ambiguity yields several candidates;
/// the caller executes them all.
pub fn elaborate(
    spec: &crate::coding::task_spec::CodingTaskSpec,
    steps: &[crate::procedure_text::ProcedureStepRecord],
    catalog: &crate::coding::fragment_catalog::FragmentCatalog,
    bounds: ElaborationBounds,
) -> Vec<ProgramIr>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ElaborationBounds {
    pub max_candidates: usize,
    pub max_depth: usize,
}
```

The threading rule is the whole of the "reconstruct the algorithm" claim, and it
is deliberately small: a step contributes an `IrNode`; consecutive nodes compose
by type; a step that binds a name introduces `Bind`; a step whose text carries a
base case and a transition introduces `Recurrence`. Nothing in `elaborate` knows
a task name.

### Fragment catalog: the seed becomes a deletable bootstrap

```rust
// src/coding/fragment_catalog.rs

/// Where a fragment came from, and whether it may be deleted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FragmentOrigin {
    /// Shipped in `data/seed/`, marked `bootstrap true`, deletable.
    Bootstrap,
    /// Rediscovered from a trusted source into the ignored cache.
    Rediscovered,
}

/// One reusable operation, language-neutral.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fragment {
    pub id: String,
    pub signature: Vec<IrType>,
    pub result: IrType,
    pub origin: FragmentOrigin,
    pub reuse: ReuseMode,
    pub grounding: String,
    pub license: String,
    pub sha256: String,
    pub fetched_at: String,
    /// The query that rediscovers this fragment when it is forgotten.
    pub rediscovery_query: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FragmentCatalog { fragments: Vec<Fragment> }

impl FragmentCatalog {
    /// Bootstrap fragments from `data/seed/`, or an empty catalog when the
    /// seed files are absent. Never panics on a missing id.
    #[must_use] pub fn bootstrap() -> Self;
    /// Merge rediscovered fragments from the ignored cache ledger.
    #[must_use] pub fn with_rediscovered(self, ledger: &FragmentLedger) -> Self;
    #[must_use] pub fn get(&self, id: &str) -> Option<&Fragment>;
    /// Fragments whose result type unifies with `ty`, cheapest first.
    #[must_use] pub fn producing(&self, ty: &IrType) -> Vec<&Fragment>;
    /// Content hash over every fragment's canonical projection, origin excluded.
    /// This is the value the forget/rediscover test compares.
    #[must_use] pub fn content_id(&self) -> String;
}

/// Content-addressed store of rediscovered fragments under `FORMAL_AI_CACHE_DIR`.
pub struct FragmentLedger { path: std::path::PathBuf }

impl FragmentLedger {
    #[must_use] pub fn new(cache_directory: impl AsRef<std::path::Path>) -> Self;
    pub fn recall(&self, id: &str) -> std::io::Result<Option<Fragment>>;
    pub fn remember(&self, fragment: &Fragment) -> std::io::Result<()>;
}
```

The forget → rediscover → same-hash contract, concretely:

1. `FragmentCatalog::bootstrap()` reads the seed **at runtime** through
   `crate::seed::parser::parse_lino`, replacing the two `include_str!`
   compile-time dependencies (`src/coding/concept_discovery.rs:11`,
   `src/coding/python_render.rs:11`). A missing file yields an empty catalog and
   a logged `fragment_catalog:bootstrap_absent` event — never a panic. This is
   the single change that makes "delete the seed" expressible.
2. `idiom()` and `template()` (`src/coding/composition.rs:709-723`) stop
   panicking; their replacements return `Option` and a `None` becomes a blocked
   need in the `ConceptMap`.
3. Each seed meaning gains `bootstrap true` and `rediscovery_query "<phrase>"`.
   `rediscovery_query` is a *natural-language phrase*, not a URL and not a task
   name, so it carries no memorized answer — the gate can assert that.
4. `formal-ai coding forget-fragments` (new CLI verb under the existing
   `src/cli_benchmark.rs`-style subcommand pattern) moves the two seed files
   aside; `formal-ai coding rediscover-fragments --online` walks every
   `rediscovery_query` through `retrieve_procedure` and writes the recovered
   fragments to the `FragmentLedger`; the test asserts
   `FragmentCatalog::bootstrap().content_id()` before equals
   `FragmentCatalog::default().with_rediscovered(&ledger).content_id()` after.
5. Offline replay reproduces it from the content-addressed source cache
   (`src/source_fetch.rs:201-238`), so the round trip runs in CI without
   network.

The seven algorithm-shaped runtime templates
(`balanced_delimiter_groups`, `group_max_nesting`,
`remove_boundary_occurrences`, `rotation_period`, `one_bit_difference`,
`grid_minimum_cost_path`, `oeis_linear_recurrence`) are **deleted outright**, not
converted to fragments: each is a whole algorithm, and the point of the IR is
that `Repeat`, `Fold`, `Condition` and `Recurrence` can express them from
retrieved steps. `oeis_linear_recurrence` in particular becomes
`IrNode::Recurrence` produced by `src/coding/function_catalog/oeis.rs`, which
already parses the recurrence it currently renders into that template.

### IR → source, and browser/WASM parity

```rust
// src/coding/ir_lowering/mod.rs

/// Lower a typed IR to concrete source in one language.
pub trait LanguageLowering {
    /// Catalog slug, matching `ProgramLanguage::slug`.
    fn language(&self) -> &'static str;
    /// Render the whole program. `Err` names the first unsupported node so the
    /// gap is specific rather than "renderer unavailable".
    fn lower(&self, ir: &ProgramIr, catalog: &FragmentCatalog) -> Result<String, LoweringGap>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoweringGap { pub language: String, pub node: String, pub detail: String }

/// Every registered lowering, in catalog order.
#[must_use] pub fn lowerings() -> Vec<&'static dyn LanguageLowering>;
#[must_use] pub fn lowering_for(language: &str) -> Option<&'static dyn LanguageLowering>;
```

`compose` (`src/coding/composition.rs:46`) changes from

```rust
if spec.language != "python" { /* renderer_unavailable */ }
```

to looking up `lowering_for(&spec.language)`, so a Rust or JavaScript request
composes for the first time. The per-language *surface* forms stay data: the
lowering asks the fragment catalog for the language-specific realization of each
fragment id, which is where `data/seed/coding-idioms.lino`'s existing
per-language scaffolds (`data/seed/coding-idioms.lino:11-53`) are absorbed
rather than duplicated.

Verification is unchanged for languages with a runtime in the workspace
(`src/coding/composition.rs:500-563` runs `python3 solution.py` in
`AgentWorkspace`); for a language with no available toolchain the answer carries
`ExecutionStatus::Unavailable` (`src/coding/catalog/types.rs:205-211`) and says
so. **Browser/WASM parity:** the browser worker has no Python runtime
(`docs/case-studies/issue-710/plans/02-dynamic-discovery-design.md` §7). Under
this architecture the browser can do strictly more than today and must still be
honest: it runs `recognise`, `discover` against the bootstrap catalog,
`elaborate` and `type_check` — all pure and WASM-safe — then renders the program
and labels it **unverified**, listing the sources. It must not claim "tests
passed". The existing worker line budget
(`scripts/check-worker-line-budget.rs`, `scripts/check-wasm-worker-size.rs`)
constrains this: the IR and lowering modules must be `no_std`-friendly in the
sense of avoiding `std::process` outside the verification path, which is already
the case for everything except `composition::verify`.

### Concept lookup: the B1 seam this plan depends on

> **reconciled: was `src/coding/source_lookup.rs::RegistryConceptLookup`, a
> second implementation of the lookup trait; now this plan declares no lookup
> type at all and takes plan 01's single `RegistrySourceLookup` from its caller,
> because B1's fixed means names one implementation used by two callers (plan 00
> §9 R3, X2).**

```rust
// No new module. `discover()` (`src/coding/concept_discovery.rs:189-191`) stops
// hard-coding `NoLookup` and takes the contract trait from its caller:

pub fn discover_with_lookup(
    spec: &CodingTaskSpec,
    catalog: &DiscoveryCatalog,
    lookup: &mut dyn crate::source_walk::SourceLookup,   // plan 00 §4.2
    bounds: crate::source_walk::LookupBounds,
) -> ConceptMap;
```

`discover_and_compose` (`src/coding/synthesis_runtime.rs:204-220`) constructs
plan 01's `RegistrySourceLookup` when online and an offline one otherwise, and
passes it through. Nothing in this plan constructs a lookup of its own, so plan
01 L18 can delete `UnknownConceptLookup` without touching this plan's leaves.

B1 owns the universal-loop half of this (`src/solver.rs:874-884`
`record_external_search`, which today appends `policy:no_fetch_capability` and
returns). This plan owns only the coding seam: `discover()`
(`src/coding/concept_discovery.rs:189-191`) stops hard-coding `NoLookup` and
takes the lookup from its caller, and `discover_and_compose`
(`src/coding/synthesis_runtime.rs:204-220`) constructs a `RegistryConceptLookup`
when `online`, a cache-only one when offline. If B1 lands first, this plan uses
its implementation instead of adding a second one; the trait is the contract
either way.

### The revised pipeline

`discover_and_compose` (`src/coding/synthesis_runtime.rs:204-220`) becomes a
four-stage escalation, each stage entered only when the previous produced no
verified draft — the existing "bounded pass first, widen on failure" discipline
already present at `src/coding/synthesis_runtime.rs:212-218`:

1. **Bootstrap.** `FragmentCatalog::bootstrap()` + `composition_search` over the
   matched meanings. This is today's behaviour, minus the 40 authored blocks, and
   is what must keep the current 20/20 green.
2. **Ready parts.** `discovery_catalog` (unchanged) — stdlib, Wikifunctions
   implementations, Wikifunctions recurrences.
3. **Procedure text.** `retrieve_procedure` over the registry's
   sources declaring `need_kinds (part)` then `need_kinds (procedure)`, each in
   derived-tier order → `elaborate` → typed IR
   candidates. **This stage is new and is the heart of B2.**
4. **Sequence sources.** `extend_with_sequence_programs` (unchanged, OEIS).

Every stage's output is a `ProgramIr`; every `ProgramIr` is lowered, CST-checked
(`crate::coding::validated_program_cst`, `src/coding/cst.rs`), executed against
the examples, and ranked by `action_cost` then source length then id — the
existing ordering at `src/coding/composition.rs:92-97`.

### Running the full 164 and 500 suites honestly

No code change is needed to *run* them: `--slice` has no cap, `fetch` caches the
whole payload (`src/external_benchmarks/fetch.rs:20-23`), and `parse_cases`
truncates at `slice` (`src/external_benchmarks/cases.rs:43-51`). What is needed
is that the result be recorded in a way the ratchet can defend:

- Add `ratchet_slice "164"` / `"500"` companion suite records? **No** — one suite
  record per suite is the invariant `ratchet::violations` relies on
  (`src/external_benchmarks/ratchet.rs:22-28`). Instead, extend
  `external_benchmark_suite` with a second floor field
  `full_slice` + `full_minimum_pass_count`, and extend
  `best_pass_count`/`raise_floor` to consult whichever floor matches the run's
  slice. `Ledger::raise_floor`'s early return
  (`src/external_benchmarks/ledger.rs:231-234`) becomes a two-way match.
- `historical_floor_violations` already groups by `(suite, slice)`
  (`src/external_benchmarks/ratchet.rs:126-134`), so the full-slice series
  ratchets independently and correctly with no change.
- The first full-suite row is written with `--append` **after** the run, at
  whatever number it is. `docs/benchmarks.md` publishes both the slice-20 row and
  the full row side by side, with the slice stated in the same sentence as the
  number — the #1085 requirement that "the upstream row must stand beside the
  curated one wherever it is cited" (`VISION.md:343`) applied one level deeper.
- The scheduled workflow keeps `BENCHMARK_SLICE` at 20 for the weekly run (cost)
  and gains a separate `workflow_dispatch` input `full_slice` that runs
  `--suite humaneval --slice 164` and `--suite mbpp --slice 500`. A full run that
  scores lower than a previous full run is red, exactly as today.

### Failure and honesty behavior

- A need with no seeded meaning, no retrieved part and no lookup evidence stays
  `status "blocked"` (`src/coding/concept_discovery.rs:222-229`) and the answer
  is the localized skill gap plus the research trail
  (`src/solver_handlers/program_synthesis.rs:62-88`) — never a guessed program.
- A `LoweringGap` names the unsupported IR node, so "I cannot write this in Rust"
  becomes "no Rust lowering for `IrNode::Recurrence`" — actionable by the
  self-improvement loop rather than opaque.
- A retrieved procedure with fewer than `MIN_PROCEDURE_STEPS` steps is discarded
  and logged; it never becomes a one-step guess.
- A `ShapeOnly` fragment whose text would be emitted verbatim is refused at
  lowering time with a license event, not silently pasted.
- An answer without a passing execution never renders the "tests passed" wording
  (`src/solver_handlers/program_synthesis.rs:106-110`), and the browser surface
  labels its output unverified.
- Determinism: retrieval order is the registry's declared order; captures are
  content-addressed (`src/source_fetch.rs:232-237`); enumeration is by cost then
  id. Same prompt + same cache ⇒ same answer.

---

## Tests first

### Held-out cases, five languages, actual prompt text

New fixture `data/benchmarks/coding-composition-from-sources.lino`, following
the shape of the existing `data/benchmarks/coding-discovery-paraphrases.lino`
(#710, 25 cases, `docs/benchmarks.md:28`). Each case names an operation that no
seeded meaning expresses today, so passing it requires retrieval. None of these
sentences may appear in any seed file — asserted by the extended gate.

**Case 1 — run-length encoding (no seeded meaning exists).**

- en: `Write a Python function run_length(text) that returns a list of (character, count) pairs for each run of equal characters in text.`
- ru: `Напиши функцию на Python run_length(text), которая возвращает список пар (символ, количество) для каждой серии одинаковых подряд идущих символов.`
- hi: `एक Python फ़ंक्शन run_length(text) लिखो जो लगातार आने वाले समान अक्षरों के हर समूह के लिए (अक्षर, संख्या) जोड़ों की सूची लौटाए।`
- zh: `写一个 Python 函数 run_length(text)，返回文本中每一段连续相同字符的（字符，数量）对组成的列表。`
- es: `Escribe una función de Python run_length(text) que devuelva una lista de pares (carácter, cantidad) por cada racha de caracteres iguales.`

**Case 2 — binary search over a sorted sequence (a procedure best recovered as a
step list, not as a one-call wrapper).**

- en: `Write a Python function find_position(values, target) that returns the index of target in the sorted list values, or -1 when it is absent, without scanning every element.`
- ru: `Напиши функцию на Python find_position(values, target), которая возвращает индекс target в отсортированном списке values или -1, если его нет, не просматривая каждый элемент.`
- hi: `एक Python फ़ंक्शन find_position(values, target) लिखो जो क्रमबद्ध सूची values में target का सूचकांक लौटाए, न मिलने पर -1, और हर तत्व को न जाँचे।`
- zh: `写一个 Python 函数 find_position(values, target)，在已排序列表 values 中返回 target 的下标，不存在时返回 -1，并且不要逐个遍历。`
- es: `Escribe una función de Python find_position(values, target) que devuelva el índice de target en la lista ordenada values, o -1 si no está, sin recorrer todos los elementos.`

**Case 3 — Levenshtein distance (a dynamic-programming recurrence recovered from
a retrieved definition, the family `grid_minimum_cost_path` currently hard-codes).**

- en: `Write a Python function edit_steps(first, second) that returns the smallest number of single-character insertions, deletions or substitutions that turn first into second.`
- ru: `Напиши функцию на Python edit_steps(first, second), которая возвращает наименьшее число вставок, удалений или замен одного символа, превращающих first в second.`
- hi: `एक Python फ़ंक्शन edit_steps(first, second) लिखो जो first को second में बदलने के लिए एक-अक्षर के जोड़ने, हटाने या बदलने की न्यूनतम संख्या लौटाए।`
- zh: `写一个 Python 函数 edit_steps(first, second)，返回把 first 变成 second 所需的最少单字符插入、删除或替换次数。`
- es: `Escribe una función de Python edit_steps(first, second) que devuelva el número mínimo de inserciones, borrados o sustituciones de un carácter para convertir first en second.`

**Case 4 — the same operation, Rust (proves the IR is not Python-shaped).**

- en: `Write a Rust function that returns the list of runs of equal characters in a string as (char, usize) pairs.`
- ru: `Напиши функцию на Rust, которая возвращает серии одинаковых символов строки как пары (char, usize).`
- hi: `एक Rust फ़ंक्शन लिखो जो एक स्ट्रिंग में समान अक्षरों की लगातार श्रृंखलाओं को (char, usize) जोड़ों के रूप में लौटाए।`
- zh: `写一个 Rust 函数，把字符串中连续相同字符的段落作为 (char, usize) 对返回。`
- es: `Escribe una función de Rust que devuelva las rachas de caracteres iguales de una cadena como pares (char, usize).`

**Case 5 — an honest blocked need (proves failure stays honest).** A request
naming an operation no trusted source defines, in all five languages, asserted to
produce a skill gap with a research trail and **no** code block.

### Unit, integration and specification tests

| Path | Name | Asserts |
| --- | --- | --- |
| `tests/unit/coding_discovery/procedure_text.rs` | `ordered_list_capture_becomes_a_provenance_bearing_step_list` | `steps_from_capture` on a committed HTML fixture yields ≥2 `ProcedureStepRecord` with url, sha256, license, ordinal |
| `tests/unit/coding_discovery/procedure_text.rs` | `a_capture_with_one_instruction_is_not_a_procedure` | returns `None`, logs the reason |
| `tests/unit/coding_discovery/procedure_text.rs` | `share_alike_capture_is_marked_shape_only` | a GFDL/CC-BY-SA capture yields `ReuseMode::ShapeOnly` |
| `tests/unit/coding_discovery/program_ir.rs` | `ir_round_trips_through_links_notation` | `to_links_notation` → parse → equal IR |
| `tests/unit/coding_discovery/program_ir.rs` | `ill_typed_composition_is_rejected_by_name` | `type_check` error names the offending node |
| `tests/unit/coding_discovery/program_ir.rs` | `action_cost_orders_the_shorter_derivation_first` | cost is a total order over two equivalent IRs |
| `tests/unit/coding_discovery/ir_lowering.rs` | `every_ir_node_lowers_to_python_or_names_its_gap` | exhaustive over `IrNode` variants |
| `tests/unit/coding_discovery/ir_lowering.rs` | `the_same_ir_lowers_to_python_and_rust` | case 1 and case 4 share one `ProgramIr` `content_id` |
| `tests/unit/coding_discovery/fragment_catalog.rs` | `bootstrap_catalog_is_absent_without_the_seed_and_never_panics` | seed files moved aside ⇒ empty catalog + logged event, no panic |
| `tests/unit/coding_discovery/fragment_catalog.rs` | `forgotten_fragments_are_rediscovered_to_the_same_content_id` | forget → rediscover offline from committed captures → identical `content_id` |
| `tests/unit/coding_discovery/fragment_catalog.rs` | `rediscovery_queries_name_no_benchmark_identifier` | every `rediscovery_query` fails the upstream-name/sentence test |
| `tests/unit/coding_discovery/composition_search.rs` | `an_unseen_combination_of_seeded_meanings_composes` | a shape absent from the deleted 40 blocks still verifies |
| `tests/unit/coding_discovery/composition_search.rs` | `search_is_deterministic_across_runs` | two runs, identical draft id order |
| `tests/unit/coding_discovery/source_lookup.rs` | `an_unknown_phrase_resolves_through_the_registry_offline` | `RegistryConceptLookup` over committed captures yields `ConceptEvidence` with depth and url |
| `tests/unit/coding_discovery/multilingual.rs` | *(extend)* `composition_from_sources_holds_in_five_languages` | the 25 new held-out cases above |
| `tests/unit/coding_discovery/no_memorization.rs` | *(extend)* `upstream_full_suite_names_and_sentences_are_absent_from_runtime_and_seed` | `SLICE` raised from 20 to the full suite length |
| `tests/unit/coding_discovery/no_memorization.rs` | *(new)* `no_seed_template_is_a_whole_algorithm` | no `coding-discovery-runtime.lino` template body contains more than N statements or a `for`/`while` plus `return` |
| `tests/unit/specification/coding_discovery_meta_algorithm.rs` | *(extend)* | the new stages appear in the recorded meta-algorithm trace |
| `tests/unit/specification/external_benchmarks.rs` | `full_slice_rows_carry_their_own_floor` | a 164-row does not move the 20-floor and does move the full-floor |
| `tests/source/source_tests/coding/program_ir/…` | placement tests | new modules obey the source-placement convention |

### Gates and ratchets

1. **No-memorization gate, widened.** `const SLICE: usize = 20`
   (`tests/unit/coding_discovery/no_memorization.rs:5`) becomes the full record
   count per suite. Scanning 164 entry points and 500 MBPP assertions against
   `src/` and `data/seed/` is the only honest gate once the full suites are the
   measurement. `GENERIC_FUNCTION_NAMES`
   (`tests/unit/coding_discovery/no_memorization.rs:9`) will need a small, argued
   extension for further ordinary English verbs that are also upstream entry
   points; every addition must be justified in the test's own comment, and the
   list is itself ratcheted (it may not grow by more than the PR names).
2. **Seed-shape gate (new).** No `template` body in
   `data/seed/coding-discovery-runtime.lino` may be a complete algorithm. Concrete
   predicate: the body contains at most one statement-level construct, or is a
   single expression. This is what prevents the deleted seven from returning.
3. **Bootstrap-deletability gate (new).** A CI job runs the test suite with
   `data/seed/meanings-coding-structure.lino` and
   `data/seed/coding-discovery-runtime.lino` moved aside and asserts: nothing
   panics, every coding answer becomes an honest gap, and no test that is not
   explicitly marked `requires_bootstrap` fails.
4. **Fragment-provenance ratchet (new).** Every `Fragment` with
   `origin: Rediscovered` must carry a non-empty `source_url`, `license` and
   `sha256` — the same shape `DiscoveredProcedure::valid` already enforces
   (`src/coding/discovered_procedures.rs:118-129`).
5. **Existing ratchets unchanged and still binding.** `benchmark ratchet`
   (`src/external_benchmarks/ratchet.rs:16, 61`), the file-size limit, the
   hard-coded-language allowlist (1,286 entries — this plan must not raise it),
   the minimal-core boundary, the worker line and WASM size budgets.

### Benchmark commands and the honest numbers expected

```sh
# Regression control: the slice that is green today must stay green.
cargo run --bin formal-ai -- benchmark run --suite humaneval --slice 20 --online
cargo run --bin formal-ai -- benchmark run --suite mbpp --slice 20 --online

# Cold-cache control (the honesty check that the cache is not the capability).
FORMAL_AI_CACHE_DIR=$(mktemp -d) \
  cargo run --bin formal-ai -- benchmark run --suite humaneval --slice 20

# The measurement this plan exists to produce.
cargo run --bin formal-ai -- benchmark run --suite humaneval --slice 164 --online
cargo run --bin formal-ai -- benchmark run --suite mbpp --slice 500 --online

# Offline replay of the same full runs from the content-addressed cache.
cargo run --bin formal-ai -- benchmark run --suite humaneval --slice 164
cargo run --bin formal-ai -- benchmark run --suite mbpp --slice 500

# Forget / rediscover / replay round trip.
cargo run --bin formal-ai -- coding forget-fragments
cargo run --bin formal-ai -- coding rediscover-fragments --online
cargo test --test unit coding_discovery::fragment_catalog -- --nocapture

# Held-out five-language suite and the gates.
cargo test --test unit coding_discovery -- --nocapture
cargo test --test unit coding_discovery::no_memorization -- --nocapture
```

**What is expected of the numbers: measure and record whatever it is.** No score
is promised here, and none may be written into this plan, a test, a ledger floor
or a document before the run produces it. Three statements are commitments about
*process*, not about values:

- The slice-20 rows may not fall; a fall is a regression to fix in capability,
  never by lowering the floor (`src/external_benchmarks/ratchet.rs:61-101`).
- The first full-suite rows are appended at whatever they are, with the slice
  named in the same sentence as the number, and they become that series' floor.
- A run that cannot happen is recorded `benchmark_unavailable` with its reason
  (`src/external_benchmarks/manifest.rs:41-48`), never replaced by a proxy.

It is expected and acceptable that the full-suite percentage is far below the
first-20 percentage. That gap *is* the measurement of B2, and publishing it is
the point.

---

## Implementation leaves

- [x] **L1** — Add `oeis` and `python_docs` source records to
      `data/seed/sources-registry.lino` with license, api, cache path and a new
      `need_kinds` field (plan 01 L3's axis, not a second one); assert in
      `tests/unit/specification/…` that `src/coding/function_catalog/oeis.rs:16`
      and `python_docs.rs:12` read their URL and license from the registry.
- [x] **L2** — Add `src/coding/program_ir.rs`: `IrType`, `IrNode`, `ReuseMode`,
      `ProgramIr`, `to_links_notation`, `content_id`, `action_cost`. Round-trip
      and cost tests only; no caller yet.
- [x] **L3** — Add `ProgramIr::type_check` and `ElaborationBounds`; ill-typed
      composition is rejected by node name.
- [x] **L4** — Add `src/coding/fragment_catalog.rs` with `Fragment`,
      `FragmentOrigin`, `FragmentCatalog::bootstrap()` reading the seed at
      **runtime**; remove both `include_str!` (`concept_discovery.rs:11`,
      `python_render.rs:11`). Absent seed ⇒ empty catalog, logged, no panic.
- [x] **L5** — Replace the panicking `idiom()`/`template()`
      (`src/coding/composition.rs:709-723`) with `Option`-returning catalog
      lookups; a miss becomes a blocked need.
- [x] **L6** — Add `src/coding/ir_lowering/{mod,python}.rs`; move
      `render_function` (`src/coding/python_render.rs:29`) behind
      `LanguageLowering`. Existing composition still uses the old path.
- [x] **L7** — Add `src/coding/composition_search.rs`: bounded typed enumeration
      over `FragmentCatalog`, deterministic by cost then id.
- [x] **L8** — Port the 9 blocks of `structural_drafts`
      (`src/coding/composition.rs:203-412`) to fragment *type signatures*; delete
      the blocks; slice-20 stays green.
      **reconciled 2026-09-19: the port landed inside the typed-enumeration
      rewrite — `structural_drafts`/`additional_drafts` and
      `src/coding/structural_composition.rs` are deleted from the tree, and the
      full `coding_discovery` suite (including the no-memorization slice gate)
      runs 84/0 on the rewritten search.**
- [x] **L9** — Port the 31 blocks of `additional_drafts`
      (`src/coding/structural_composition.rs:12-342`) the same way; delete them;
      slice-20 stays green. **reconciled 2026-09-19: same rewrite as L8 — the
      file and its draft tables no longer exist; typed enumeration replaces
      them.**
- [x] **L10** — Add `src/procedure_text.rs`: `ProcedureStepRecord`,
      `StepShape`, `steps_from_capture` over the existing
      `how_to_guide::extract` helpers, with committed HTML fixtures.
- [x] **L11** — Add `retrieve_procedure` walking the registry in `need_kinds`
      consultation order through `CachedSourceClient`, bounded by `LookupBounds`.
- [x] **L12** — Add `program_ir::elaborate`: step list → candidate IR, with the
      type-threading rule and `Bind`/`Recurrence` introduction.
- [x] **L13** — Wire stage 3 into `discover_and_compose`
      (`src/coding/synthesis_runtime.rs:204-220`), after ready parts and before
      sequence sources.
- [x] **L14** — `discover()` (`src/coding/concept_discovery.rs:189`) takes
      `&mut dyn SourceLookup` from its caller instead of `NoLookup`;
      `discover_and_compose` constructs plan 01's `RegistrySourceLookup`.
      **reconciled: was "add `src/coding/source_lookup.rs` `RegistryConceptLookup`";
      now no new lookup type, because plan 01 owns the one implementation
      (plan 00 §9 R3).**
- [ ] **L15** — Delete the 7 algorithm-shaped templates from
      `data/seed/coding-discovery-runtime.lino`; make `oeis.rs` emit
      `IrNode::Recurrence`; re-run the slice-20 control **and the 13/13 curated
      industry control in the same commit** — one of the seven answers a curated
      case, and a fall in either is recorded, never repaired by restoring the
      template (plan 00 §9 X3).
- [x] **L16** — Add `bootstrap true` and `rediscovery_query` to every meaning in
      `data/seed/meanings-coding-structure.lino`; add the seed-shape gate.
- [x] **L17** — Add `FragmentLedger` plus the `coding forget-fragments` /
      `coding rediscover-fragments` CLI verbs.
- [x] **L18** — Add the forget → rediscover → same-`content_id` test running
      offline from committed captures; add the bootstrap-deletability CI job.
- [x] **L19** — Add `src/coding/ir_lowering/rust.rs`; make `compose`
      (`src/coding/composition.rs:47-51`) dispatch through `lowering_for`
      instead of refusing non-Python.
- [x] **L20** — Add the 25 held-out five-language cases as
      `data/benchmarks/coding-composition-from-sources.lino` plus the
      `coding_discovery::multilingual` extension.
- [x] **L21** — Browser/WASM parity: expose recognise→discover→elaborate→lower in
      the worker, label results unverified, keep the line and size budgets
      (worker-line-budget shards + check-file-size). The worker-mirror
      composition assertion that was red is this leaf's pin — made green
      2026-09-19: `node --test tests/web/worker-mirror.test.mjs` → 18/18 after
      mirroring the native selection gates in the new
      `src/web/worker/formal_ai_worker_closed_programs.js` (placeholder-atom
      closedness after lowering, input-domain grounding for membership-style
      predicates) plus least-action application-count ordering in
      `formal_ai_worker_program_ir.js`; `scripts/check-worker-line-budget.rs`
      exit 0 (new module shard recorded; program_ir ceiling 338→369 with
      rationale in its ledger); `scripts/check-file-size.rs` exit 0; results
      stay labelled unverified at the browser boundary.
- [x] **L22** — Extend `external_benchmark_suite` with `full_slice` /
      `full_minimum_pass_count` (**lands before plan 08 L22, so its
      re-measurement writes into a ledger that already knows two slices; the two
      floors are independent series and neither may be derived from the other —
      plan 00 §9 X8**); teach `raise_floor`
      (`src/external_benchmarks/ledger.rs:224-245`) and `best_pass_count`
      (`src/external_benchmarks/ratchet.rs:115-121`) the second floor.
- [x] **L23** — Raise the no-memorization gate's `SLICE`
      (`tests/unit/coding_discovery/no_memorization.rs:5`) to the full suite
      length; justify any `GENERIC_FUNCTION_NAMES` addition in-comment.
- [ ] **L24** — Run `--slice 164` and `--slice 500`, online and cold-offline;
      append the rows; record the failure frontier through the existing
      `--frontier-record` flag (`src/cli_benchmark.rs:57-61`).
- [ ] **L25** — Update every document listed below with the measured numbers and
      the new architecture.

### 2026-09-17 typed-search retirement checkpoint

The implementation for L8 and L9 is now present, but the leaves remain
unchecked until the focused Rust corpus is executed on the combined tree. The
two authored Rust generators have been deleted. `composition_search` now
performs bounded deterministic enumeration of nested typed fragment
applications, deriving argument names from target realizations and relevance
edges from source-seed `supports` links. Prompt and example literals are
hypotheses in that same bounded frontier; only executable examples can select
one. The held-out structural corpus now requires every selected answer to
report `typed_search(…)`, so reintroducing a compatibility generator cannot
satisfy the gate.

Browser parity L21 and live-network measurements L24 remain open. This
checkpoint adds no benchmark number: it awaits the coordinator's serialized
Cargo run and then the explicitly online slice-164 and slice-500 runs.

### 2026-09-17 completion audit checkpoint

This audit deliberately leaves unchecked every leaf whose whole acceptance
criterion has not run successfully on the combined tree. The browser now
performs seed-driven typed composition and emits the lowered source with its IR
content id, fragments, sources, licences, and an explicit unverified boundary.
`tests/web/worker-mirror.test.mjs` passes **18/18**, JavaScript syntax checks
pass, and the new module is **334/335** lines. The repository-wide line-budget
gate is not green yet: `formal_ai_worker_20.js` is 1311/1308,
`formal_ai_worker_dispatch.js` is 92/80, and
`formal_ai_worker_verifiable_task.js` has no budget shard. Therefore L21 stays
open rather than treating a focused test as proof of its entire budget clause.

The focused `composition_search` module passes **4/4** after preserving
complete fold roots outside the bounded subexpression beam, excluding untyped
renderer scaffolds from the operation catalog, ordering final programs by
action cost then content id, and rejecting lowered candidates whose catalog
placeholder names remain free. Its multilingual fixture supplies the
language-neutral `extend_run` identity that concept discovery is meant to
produce and proves that English and Russian prose then expose the same
frontier. This proves L7; L8–L9 remain unchecked because their required
slice-20 control has not passed on the combined tree. The separately corrected
empty-catalog blocked-need path still requires the coordinator's next
serialized Cargo run.

L15 remains materially incomplete. Six algorithm-shaped answers are absent,
and OEIS recurrence source now passes through `IrNode::Recurrence`, but
`grid_minimum_cost_path` is still a whole algorithm in
`meanings-coding-structure.lino`. Removing it requires a general recursive or
dynamic-programming IR composition, not renaming the answer as a primitive.
The required slice-20 and 13/13 controls have also not both passed on the
combined tree. L24 (the four live/cold full-suite measurements and frontier
rows) and L25 (documents updated from those measured rows) remain owned by the
coordinator and open. No full-suite number is inferred or fabricated here.

### 2026-09-19 L15 template-deletion control (fall recorded, leaf stays open)

The seven algorithm-shaped templates are deleted from
`data/seed/coding-discovery-runtime.lino` (aligned_binary_xor, explicit_value_mapping,
explicit_ordering, regex_minimum_word_length, regex_lowercase_chunks,
regex_lowercase_underscore, composite_number); the OEIS clause already holds (oeis.rs emits
`IrNode::Recurrence`, src/coding/function_catalog/oeis.rs:554). The seven pinning `selected(...)`
cases left `tests/unit/coding_discovery/structural_composition.rs` with them, and the
brace-rendering test repointed to a temp-seed probe (module 84/84 green). Both controls were run
on the same binary, before and after the data-only change:

- `./target/debug/formal-ai benchmark run --suite humaneval --slice 20 --online`: baseline
  `passed=13 failed=7`, after deletion `passed=12 failed=8`. HumanEval/19 (`sort_numbers`) fell:
  its baseline answer was the explicit_ordering template
  `(lambda order: ' '.join(sorted(numbers.split(), key=order.__getitem__)))({...word→rank...})`.
- `./target/debug/formal-ai benchmark run --suite mbpp --slice 20 --online`: baseline
  `passed=15 failed=5`, after deletion `passed=11 failed=9`. Four fell: MBPP/3 `is_not_prime`
  (composite_number), MBPP/7 `find_char_long` (regex_minimum_word_length), MBPP/15
  `split_lowerstring` (regex_lowercase_chunks), MBPP/16 `text_lowercase_underscore`
  (regex_lowercase_underscore). The other two templates answered nothing in either slice
  (HumanEval/11 `string_xor` failed even with aligned_binary_xor present).
- `cargo test --offline -j 2 --test unit issue_304_benchmark_suite_reports_pass_fail_counts`:
  passed=11 failed=2 before and after (bigbench object_counting ×2, the committed-tip counting
  fall) — no curated-suite fall.

Per plan 00 §9 X3 the falls are recorded and not repaired by restoring a template. The tick
criterion "both controls hold" is not met while composition cannot derive regex construction,
primality-by-divisor, or word→rank ordering as typed compositions; that generalization is the
remaining L15 scope, alongside `grid_minimum_cost_path` still living in
meanings-coding-structure.lino.

---

## Docs to update

The exact quoted statements and their replacement text moved to plan 11's
findings table on 2026-09-16, so there is one docs authority and no document
is described in two places (plan 00 §8). This plan's entries are rows
**D160-D169** of
[`11-docs-consistency-audit.md`](11-docs-consistency-audit.md) §"Issue #1138
plan doc replacements", and plan 11's leaves apply them after the ledger rows
they cite exist (plan 00 §7).

| row | document |
| --- | --- |
| D160 | `VISION.md:343` |
| D161 | `ROADMAP.md:145` |
| D162 | `ROADMAP.md:492` |
| D163 | `docs/benchmarks.md:284-295` |
| D164 | `docs/benchmarks.md:348-368` |
| D165 | `docs/benchmarks.md:16-33` |
| D166 | `docs/requirements/issue-0710-dynamic-coding-discovery.md` |
| D167 | New shard `docs/requirements/issue-1138-composition-from-sources.md` |
| D168 | `docs/requirements-traceability.md` |
| D169 | `docs/meta-algorithm.md` |

Any further document this plan's implementation touches is added as a new
plan 11 row, never as a second copy here.

## Risks and open questions

1. **Losing the 20/20 during the port.** Deleting 40 authored blocks in favour of
   search plus typing will almost certainly regress some of the currently passing
   20 mid-flight. Mitigation: L8 and L9 are separate leaves, each gated on the
   slice-20 control staying green, and the old generators stay in the tree until
   both land. Open question: whether a small number of blocks prove genuinely
   irreducible and should be kept as explicitly-typed fragments rather than
   deleted.
2. **Prose-to-steps is lossy.** Wikipedia and Stack Exchange prose frequently
   yields steps that `elaborate` cannot type. The honest outcome is a blocked
   need, which means the full-suite number may improve less than the effort
   suggests. This is the measurement, not a defect, but it should be stated in
   the PR rather than discovered by a reviewer.
3. **Share-alike contamination.** Rosetta Code is GFDL-1.2 and Stack Exchange is
   CC BY-SA 4.0. `ReuseMode::ShapeOnly` is the structural guard, but "abstract
   shape" versus "derivative work" is a judgement the code cannot make. Open
   question: whether `secondary` sources should contribute to *generated* code at
   all, or only to attributed examples as they do today
   (`src/coding/rosetta_request.rs`).
4. **Full-suite runtime and CI cost.** 164 + 500 cases, each a `solver.solve`
   plus a `timeout 20 python3` subprocess
   (`src/external_benchmarks/grade.rs:461-466`), is far beyond the weekly job's
   current shape. Open question: whether full runs are `workflow_dispatch`-only,
   monthly, or sharded.
5. **The no-memorization gate at full slice.** Scanning 164 entry points and 500
   MBPP assertion names against all of `src/` and `data/seed/` will surface
   collisions with ordinary English identifiers well beyond the current three
   (`tests/unit/coding_discovery/no_memorization.rs:9`). Open question: the
   principled rule for the allowlist. Proposal: a name is allowlistable only if it
   is a single common English word *and* it already appears in the repository
   with a meaning unrelated to the upstream task — checked, not asserted.
6. **The gate skips silently without the cache**
   (`tests/unit/coding_discovery/no_memorization.rs:17-24`). At full slice this
   matters more, because the gate is the only thing standing between "we
   generalized" and "we fitted 664 cases". Open question: should CI fail rather
   than skip when the cache is absent on the branch that touches coding seed data?
7. **Two floors per suite.** Adding `full_slice` / `full_minimum_pass_count` to
   `external_benchmark_suite` touches the ratchet's invariants
   (`src/external_benchmarks/ratchet.rs:43-53`) and the docs test that asserts
   the floor equals the best measurement
   (`tests/unit/specification/external_benchmarks.rs:177-181`). Risk of a subtly
   weakened ratchet. Mitigation: the two floors are independent series; neither
   may be derived from the other.
8. **B1 dependency.** Stage 3 without a working concept lookup retrieves for
   phrases it already understands, which is the least useful case. If B1 does not
   land first, `RegistryConceptLookup` in this plan must carry the full walk, and
   the two implementations must be reconciled later. Open question: sequence the
   two plans, or accept the temporary duplication with a named follow-up.
9. **IR expressiveness.** Ten `IrNode` variants will not express everything in
   164 + 500 tasks. `LoweringGap` and the blocked-need path make each shortfall
   visible and specific, but the variant set will need to grow — and each growth
   must be justified by a *retrieved* procedure that needs it, never by a
   benchmark case that fails without it. That distinction is the whole doctrine,
   and it has no automated check. Open question: can the fragment-provenance
   ratchet be extended to require that every new `IrNode` variant cite the
   retrieved procedure that motivated it?
10. **The browser will diverge.** Without a Python runtime it can never verify,
    so its answers are structurally weaker than the CLI's. The honest label is the
    mitigation, but it leaves a surface where "the system answered" and "the
    system verified" differ. Open question: is a WASM Python (Pyodide) acceptable
    under the no-vendored-dependency and worker-size policies, or does the browser
    stay an unverified surface indefinitely?

### 2026-09-18 L19/L20 checkpoint

`composition_from_sources_holds_in_five_languages` is green. Three changes
carried it, each general rather than case-shaped:

1. **Coverage became a preference over survivors, not a pre-filter.** The old
   bar — an expression must cover every structure the requirement reduced to
   before conversion — admitted full-coverage nonsense (nested fragment stacks
   that mention all five structures and then fail type inference) while
   excluding the one composition that types: the `extend_run` fold root, whose
   coverage is exactly itself. Selection now converts the bounded pool first
   and keeps the survivors with the most covered structures, full coverage
   still preferred; a partial composition lands in `unverified` unless the
   task's own oracle passes it, so silence is never mistaken for honesty.
2. **A callable whose requirement names no parameter reads one anonymous
   input.** The rust family has no signature in any of its five paraphrases
   ("a string", "строки", "एक स्ट्रिंग", "字符串", "una cadena"); without a
   variable to ground, every composition degenerates to constants.
   `recognise` now synthesizes `input` for a Function artifact whose
   parameters are empty; unification types it against the applied fragments.
3. **Seed fragments may carry per-language realizations.** `extend_run` grows
   a `rust` realization beside its python idiom (same grounding, same
   license); the search names its arguments from the target language's
   realization and the rust lowering interpolates it — the second language
   proof L19 asked for.

Cost, recorded honestly: the multilingual composition test runs ~172 s
(previously ~29 s) because the conversion pool widened. L21 (browser/WASM
parity) and L24 (slice runs) remain open.

### 2026-09-18 budget and parity checkpoint

The per-module worker-line-budget gate is green again: every shard passes,
including the three the 2026-09-17 audit named. `formal_ai_worker_20.js` and
`formal_ai_worker_dispatch.js` were re-baselined by their own leaves (1459 and
80, each with the growth rationale in its shard), and
`formal_ai_worker_verifiable_task.lino` carries a shard for its 150-line
module. One new growth happened in this plan's scope and was re-baselined with
its reason recorded rather than hidden:
`formal_ai_worker_program_ir.lino` moved 334 -> 338 because the browser
fragment catalog now bootstraps the second meanings seed part
(`meanings-coding-structure-2.lino`, issue #960 R222-1) exactly as
`src/coding/fragment_catalog.rs::bootstrap_from` does, so the browser parity
L21 asks for stays parity after the native side's seed split.

With those budgets green the remaining L21 clause is only the live browser
probe, and L24 remains gated on the serialized mbpp runs (slice 500 online leg
in flight, slice 164 chained behind it). L25 waits on L24's numbers by
design.

### 2026-09-19 mbpp slice checkpoint — three of four legs landed

The serialized runs finished for three of the four legs and their rows are
appended to `data/benchmarks/external-results.lino`, with the failure
frontier recorded through `--frontier-record` and the monotonic floor moved
to `full_minimum_pass_count "60"`: slice 500 offline **49/500**, slice 500
online **60/500**, slice 164 offline **25/164** (solver 0.350.0, upstream
grading, no proxy scores). The slice 164 online leg did not land: its log
truncated to zero bytes and no row was written, so the leg must be re-run
before L24 can tick. The honest reading of the three landed legs is
unchanged from the plan's premise — raw MBPP is mostly out of reach for the
seed-backed composer, and the value of these rows is the frontier they
record (which derivations compose, which fail and why), not the pass count.
