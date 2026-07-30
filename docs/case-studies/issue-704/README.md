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

## Implemented design

`SolverConfig::draft_count` defaults to one and is exposed as
`--draft-count` / `FORMAL_AI_DRAFT_COUNT`. A value of one retains the previous
single-search path.

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
  four-language comparison explanations.
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
