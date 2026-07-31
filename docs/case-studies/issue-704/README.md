# Issue 704: deterministic candidate-solution portfolios

## Root cause and reproduction

The universal loop recorded candidate labels, but step 7 delegated to a
single budget-search execution that returned the first answer found. There was
no draft identity, per-draft validation record, comparison ledger, deterministic
selector, composition backtracking, or recoverable explanation artifact.

The regression uses arithmetic reachability because its successful path must
compose operands under generated constraints. With `draft_count = 3`, the
reuse and rule-derivation drafts fail all three generated checks while the
search draft passes them. The test requires exactly three `draft:result`
events, one `draft_comparison`, winner index 2, and identical answers and event
traces across repeated runs.

The deeper root cause surfaced in review: the first implementation was welded to
arithmetic. Only the search strategy produced artifacts, the strategy list was a
Rust `match`, the generated tests were expression-specific, and the recorded
`draft_failure` events had no consumer. A portfolio that only one handler can
use is not a property of the meta algorithm.

## Implemented design

`SolverConfig::draft_count` defaults to one and is exposed as
`--draft-count` / `FORMAL_AI_DRAFT_COUNT`. A value of one retains the previous
single-search path.

The portfolio is a domain-independent engine (`src/draft_portfolio.rs`) driven
by a `PortfolioLeaf` trait. A leaf declares which seed-catalog strategies it
supports for a given instance, how to draft under one, how to test a draft, and
whether a draft composes with the rest of the answer. The strategies themselves
are data: `data/seed/draft-strategies.lino` declares reuse, rule derivation,
oracle lookup, search, and program synthesis, so a new draft generator is
introduced by adding a row, not by editing a match arm (issue #386, R379).

Two unrelated leaves plug into that engine, which is what makes the
generalization checkable rather than asserted:

- Arithmetic reachability (`src/solver_search/portfolio.rs`) implements all five
  strategies as real generators — replay in given order, greedy rule derivation,
  exhaustive oracle table for small instances, budget-driven search, and bounded
  program synthesis. `oracle_lookup` reports itself unsupported above two
  operands, so applicability is instance-aware rather than hardcoded.
- Rule synthesis (`src/rule_synthesis_portfolio.rs`) turns the learning-ledger
  recall and the operation-vocabulary derivation from an ordered fallback chain
  into independent drafts judged by the same verification fixture.

Failed drafts are retained learning with a real consumer:
`EventLog::append_to_link_store` persists each `draft_failure` as a memory link,
and `src/dreaming/draft_failures.rs` mines those links into per-strategy lessons
(`deprioritize_strategy`, `extend_strategy`, `raise_draft_count`) that reach
`DreamingPlan` and its rendering. The conclusions are language-neutral slugs, so
no English prose is minted in Rust.

For larger values, each candidate receives a seed derived from the impulse hash
and draft index. Candidate evaluations use isolated event logs and run on
scoped threads. Results are sorted by draft index before logging, so operating
system scheduling cannot affect the answer. Fully passing candidates are
ranked by expression size, evaluation count, and draft index. The chosen
candidate must also pass composition; otherwise selection backtracks through
the ranked passing set.

Each candidate produces a structured result with test counts, step/size cost,
and verdict. Failed candidates also produce bounded retry metadata compatible
with the existing maximum-three-attempt learning convention. A single
comparison record is embedded as Links Notation in the answer, making a later
“why this solution?” question answerable in English, Russian, Hindi, and
Chinese without relying on process memory.

## Research basis

The implementation adapts established algorithm-portfolio ideas to a
deterministic symbolic solver:

- [SATzilla](https://doi.org/10.1613/jair.2490) motivates selecting among
  complementary strategies because no individual strategy dominates every
  instance.
- [Hydra](https://ojs.aaai.org/index.php/AAAI/article/view/7565) motivates
  retaining learning value from candidates that do not win.
- [Algorithm portfolios](https://doi.org/10.1016/S0004-3702(00)00081-3)
  motivate concurrent independent attempts.
- [problem-solving](https://github.com/konard/problem-solving) reinforces the
  local project practice of first expressing a failing check and then deriving
  an implementation that satisfies it.

The adaptation deliberately excludes predictive or stochastic winner choice:
all randomness is seed-derived, all drafts are tested, and the final merge and
tie-break are explicit.

## Verification

- `tests/unit/issue_704.rs` covers deterministic three-draft rescue, exact
  default compatibility, environment configuration, failure learning, and
  four-language comparison explanations. The engine itself is covered
  behaviourally through a scripted test leaf: composition backtracking, ordered
  merge under concurrency, bounded retry-budget exhaustion, per-slot seed
  reproducibility, and the seed catalog overriding strategy order.
- `rule_synthesis_is_a_second_portfolio_leaf_with_the_same_engine` runs the
  second leaf end to end: `reuse` finds no approved lesson, drafts nothing and
  burns its bounded retry budget, and `rule_derivation` rescues the turn at
  draft index 1 with the comparison recorded — while the answer stays byte-for-byte
  identical to the sequential path.
- `losing_drafts_become_durable_lessons_the_dreaming_loop_mines` follows a
  failure from the event log through the memory store into the dreaming plan.
- The issue regression covers seed-separated deterministic traces,
  hierarchical-backtracking grounding, and the environment-gated parallel
  wall-clock bound.
- `data/benchmarks/industry-suite.lino` adds the portfolio reachability case and
  raises the pass-count ratchet from 12 to 13.
- `data/meta/draft-portfolio-recipe.lino` records the reproducible
  generate/evaluate/order/select/compose/learn/explain recipe.

## External Agent CLI evidence

The repository's source-built `formal-ai` binary was served as an
OpenAI-compatible model to `@link-assistant/agent` 0.25.0. The external agent
created `self-coding-result.txt`, verified its exact contents, and the same task
then passed the built-in deterministic replay.

- [`agent-stream.jsonl`](self-coding-run/agent-stream.jsonl) records the
  external Agent CLI session.
- [`general-change-plan.lino`](self-coding-run/general-change-plan.lino)
  records the generated plan and verification command.
- [`result.diff`](self-coding-run/result.diff) records the workspace change.
- [`session.json`](self-coding-run/session.json) records the deterministic
  built-in replay.
- [`formal-ai.log`](self-coding-run/formal-ai.log) records the server-side
  trace.
