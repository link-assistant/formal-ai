# Issue 891 Case Study

Issue [#891](https://github.com/link-assistant/formal-ai/issues/891) (parent
[#710](https://github.com/link-assistant/formal-ai/issues/710)) records the
audit verdict *still-broken* for the issue
[#406](https://github.com/link-assistant/formal-ai/issues/406) requirement of
"at least 50 verified equation-type examples": the delegation tests covered
linear, placeholder, symbolic, polynomial and word-problem categories, but
nothing defined the corpus in machine-readable form and nothing counted it.

## 1. Collected Data

- Issue body, labels and comments: [`raw-data/issue-891.json`](raw-data/issue-891.json),
  [`raw-data/issue-891-comments.json`](raw-data/issue-891-comments.json) (none).
- Pull request snapshot: [`raw-data/pr-968.json`](raw-data/pr-968.json).
- Pre-existing coverage: `tests/unit/specification/calculator_delegation.rs`
  (assertion-per-prompt, no corpus, no count).
- **Production-solver probe**:
  [`raw-data/production-solver-probe.tsv`](raw-data/production-solver-probe.tsv)
  — every candidate prompt run through `FormalAiEngine::answer`, recording
  `prompt / intent / engine / answer`. This is the evidence every expected
  answer in the corpus is derived from; nothing was hand-written.
- **Ratchet run**: [`raw-data/ratchet-run.log`](raw-data/ratchet-run.log) —
  `passed=67 failed=0 total=67 verified_types=67 minimum_pass_count=67
  minimum_verified_types=50`.

## 2. Requirements

Per-requirement rows live in [`requirements.md`](requirements.md) and in the
`Issue #891` section of [`REQUIREMENTS.md`](../../../REQUIREMENTS.md)
(R891-1 … R891-6).

## 3. Root Cause

The capability was largely present — `link-calculator` already solves linear,
placeholder, symbolic and rational-root polynomial equations — but it was
*asserted*, not *counted*. Three gaps followed:

1. No machine-readable corpus: equation types existed only as inline
   assertions, so "how many types are verified?" had no answer.
2. No ratchet: a regression that stopped solving a whole category could pass CI
   as long as the specific asserted prompts still worked.
3. Equation-solving request cues were missing from the seed. `Solve 2 * x = 10`
   worked (the bare `solve` cue), but `Solve the equation …`,
   `Реши уравнение …`, `解方程 …` and `समीकरण हल करें …` did not: the wrapper was
   left in the expression and the calculator could not parse it. That is a
   *vocabulary* gap in `data/seed/meanings-calculator.lino`, not a code gap.

## 4. Implemented Design

- **Seed** — `data/seed/meanings-calculator.lino` gains equation-solving
  surfaces under the existing `calculation_request_cue` role, longest-first so
  the longer surface wins before the shorter one: `solve the equation` /
  `solve equation` (en), `реши уравнение` / `решите уравнение` / `реши` /
  `решите` (ru), `解方程` / `求解` (zh), `समीकरण हल करें` / `समीकरण हल करो` /
  `हल करें` / `हल करो` (hi). Because the cues are seed data, the Rust engine
  (`calculation_request_prefixes()`) and the JavaScript worker
  (`src/web/seed_loader.js`) pick them up from the same source — no production
  code carries a hardcoded phrase.
- **Corpus** — [`data/benchmarks/equation-type-corpus.lino`](../../../data/benchmarks/equation-type-corpus.lino):
  67 `benchmark_case` records (one per distinct `equation_type`) plus 10
  `benchmark_limitation` records, with `minimum_pass_count "67"` and
  `minimum_verified_types "50"`.
- **Ratchet** — [`tests/unit/specification/equation_corpus.rs`](../../../tests/unit/specification/equation_corpus.rs):
  well-formedness, a full replay of every case through the production entry
  point, and a limitations test.
- **Regeneration tooling** — `examples/issue_891_equation_probe.rs` (probe) and
  `experiments/issue-891-build-corpus.py` (join probe output with the category
  labels and emit the fixture). The generator *aborts* if a labelled prompt is
  missing from the probe output or did not solve, so the fixture cannot drift
  from observed behaviour.

## 5. Prior Art And Existing Components

- `tests/unit/specification/world_state_benchmarks.rs` (issue #702) — the
  fixture-loading + pass-count-floor harness shape reused here.
- `data/benchmarks/world-state-tracking-suite.lino` — the record format
  (`benchmark_suite` / `benchmark_source` / `benchmark_case`).
- `tests/unit/specification/calculator_delegation.rs` (issue #96) — kept
  unchanged; the corpus complements it with counted, machine-readable coverage.
- `link-calculator` 0.20.3 — the upstream solver every verified case delegates
  to (`calculation:engine:link-calculator`).

## 6. Verification

```sh
cargo test --test unit issue_891_equation_corpus -- --nocapture
```

```text
issue #891 equation-type corpus: passed=67 failed=0 total=67 verified_types=67 \
  minimum_pass_count=67 minimum_verified_types=50
```

### Category coverage

| Category | Verified types | Examples |
| --- | --- | --- |
| `linear_one_operation` | 10 | `x + 2 = 5`, `100 - x = 42`, `-2 * x = 8`, `0.5 * x = 2.5` |
| `linear_multi_operation` | 12 | `3 * (x - 1) = 2 * (x + 4)`, `x / 2 + x / 3 = 5`, `7 * x - 4 = 3 * x + 12` |
| `placeholder_unknown` | 8 | `?+2=4`, `2 * ? + 3 = 11`, `* / 4 = 3` |
| `symbolic_multi_variable` | 7 | `2 * x + 3 * y = 12` → `x = 6 - 1.5*y`, `x + y + z = 6` |
| `polynomial` | 14 | `x^2 - 5 * x + 6 = 0`, `x^3 - 6 * x^2 + 11 * x - 6 = 0`, `x^5 - x^3 = 0` |
| `natural_language_wrapper` | 13 | en/ru/zh/hi cues over the same equations |
| `evaluation_and_percent` | 3 | `2*2+2=?`, `x*2 = 123 ?`, `8% of x = 4` |
| **Total** | **67** | floor: 50 |

Language coverage across the corpus: 56 `en`, 4 `ru`, 4 `hi`, 3 `zh` cases; the
well-formedness test asserts all four supported languages stay represented.

### Upstream calculator limitations

Recorded as `benchmark_limitation` records and asserted to keep failing
*loudly* — the engine must decline, never fabricate. If upstream lifts one, the
assertion fires so the record is promoted into a verified case.

| Gap | Where | Example | Observed |
| --- | --- | --- | --- |
| Irrational roots | link-calculator | `Solve x^2 - 2 = 0` | `calculation_error` — rational roots only, no `sqrt(2)` |
| Complex roots | link-calculator | `Solve x^2 + 1 = 0` | `calculation_error` |
| Contradiction | link-calculator | `Solve 0 * x = 5` | `calculation_error`, not "no solution" |
| Malformed input | link-calculator | `Solve x + = 4` | `calculation_error` (pinned so malformed input can never yield an answer) |
| Identity | formal-ai | `Solve x = x` | `unknown` — no calculation signal, so the router declines instead of answering "any value" |
| Units on the unknown | link-calculator | `Solve x kg = 1000 g` | `calculation_error` — units are not converted before solving |
| Units on the constant | link-calculator | `Solve 2 * x = 10 kg` | `calculation_error` |
| `x if …` declaration | formal-ai | `What is x if x + 7 = 12?` | `calculation_error` — the named-unknown declaration is not stripped |
| `x for …` declaration | formal-ai | `Calculate x for 6 * x = 42` | `calculation_error` |
| Command-shaped prompt | formal-ai | `Find x: 5 * x = 45` | `agent_suggestion` — `find` is claimed by the shell router first |

The last three are formal-ai routing gaps rather than upstream limitations;
they are recorded here so a future change has a counted starting point.

### Regenerating the corpus

```sh
cargo run --example issue_891_equation_probe -- experiments/issue-891-equation-prompts.txt > /tmp/probe.tsv
python3 experiments/issue-891-build-corpus.py /tmp/probe.tsv > data/benchmarks/equation-type-corpus.lino
cargo test --test unit issue_891_equation_corpus -- --nocapture
```

Raise `minimum_pass_count` whenever the pass count rises; never lower it.
