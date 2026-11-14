# Solver switches

Formal AI keeps the choices between valid ways of solving a task in one record,
`SolverConfig` (`rust/src/solver.rs`). The record is deterministic: the same
prompt with the same values produces the same answer. This page lists every
switch, its values and its default, and says for each one whether it changes
the answer or only what is recorded in the trace.

The requirement behind the page is the architect's, from
[the 2026-11-10 note](../architect-notes/2026-11-10-what-we-do-most-wrong.md):

> If there multiple valid options to try our in our research, we should support
> configuration (switching) between them.

## How a switch is set

- **Environment variable** — read once by `SolverConfig::from_env`
  (`rust/src/solver_helpers/mod.rs::config_from_env`). The command line, the
  local server, the Telegram bot and the Docker image all start from it.
- **Library** — construct `SolverConfig` and change its fields.
- An unset or unrecognised value leaves the default in place.

For an on/off switch every value is truthy except an empty one, `0`, `false`,
`no` and `off`.

## Switches that change the answer

| Field | Environment variable | Values | Default | What it selects |
| --- | --- | --- | --- | --- |
| `offline` | `FORMAL_AI_OFFLINE` | truthy | off | Forbids every external lookup. |
| — | `FORMAL_AI_LIVE_FETCH` | truthy | off | Allows fetching from the network. Discovery from the internet does not run without it, even when `offline` is off. |
| `agent_mode` | `FORMAL_AI_AGENT_MODE` | truthy | off | Allows the permission-gated agent actions. |
| `diagnostic_mode` | `FORMAL_AI_DIAGNOSTIC_MODE` | truthy | off | Shows the diagnostic links in the reply. |
| `temperature` | `FORMAL_AI_TEMPERATURE` | `0.0`–`1.0` | `0.7` | How many candidate interpretations of the request stay alive during formalization. |
| `guess_probability` | `FORMAL_AI_GUESS_PROBABILITY` | `0.0`–`1.0` | `0.8` | Guess the formalization (`1.0`) or ask the smallest clarifying question (`0.0`). |
| `follow_up_probability` | `FORMAL_AI_FOLLOW_UP_PROBABILITY` | `0.0`–`1.0` | `0.75` | How readily a short message is read as a follow-up of the previous turn. |
| `cache_ttl_seconds` | `FORMAL_AI_CACHE_TTL_SECONDS` | seconds | `5184000` (60 days) | How long a cached external source is used before it is fetched again. |
| `compute_budget` | `FORMAL_AI_COMPUTE_BUDGET` | whole number | `512` | Candidate evaluations the search stage of step 7 may spend; `0` turns the stage off. |
| `draft_count` | `FORMAL_AI_DRAFT_COUNT` | `1`–`255` | `1` | Independent drafts evaluated for one synthesis leaf; `1` is the single-path behaviour. |
| `blueprint_composition` | `FORMAL_AI_BLUEPRINT_COMPOSITION` (or `FORMAL_AI_PROGRAM_COMPOSITION`) | `composed`, `documented` | `composed` | Project a program from the capabilities the request names, or emit the full documented program. |
| `definition_fusion_by_default` | `FORMAL_AI_DEFINITION_FUSION` | truthy or `auto`, `merge`, `fusion`; falsy or `explicit`, `manual`, `none` | off | Merge definitions from several sources without being asked. |
| `associative_project_promotion` | `FORMAL_AI_ASSOCIATIVE_PROJECT_PROMOTION` (or `FORMAL_AI_PROJECT_PROMOTION`) | truthy, falsy | on | Promote a growing request into a project. |
| `execution_surface` | `FORMAL_AI_EXECUTION_SURFACE` (or `FORMAL_AI_SURFACE`) | `rust`, `cli`, `http`, `browser`, `telegram`, `docker` | set by the surface | Which surface the answer is prepared for. |

Three fields of the record have no environment variable and are set only from
the library: `context_sensitivity` (default `0.6`), `questioning_rigor`
(default `0.4`) and `max_decomposition_depth` (default `4`).

## Switches that change only the trace

These four select what the solver records about its own reasoning. The source
says so itself: the selection record "changes trace verbosity, never routing or
any answer" (`rust/src/selection.rs`) and the skill ledger "changes neither
routing nor the answer" (`rust/src/skill_ledger.rs`). They are listed apart on
purpose: setting one of them does not make Formal AI solve the task in a
different way.

| Field | Environment variable | Values | Default | What is recorded |
| --- | --- | --- | --- | --- |
| `recursion_mode` | `FORMAL_AI_RECURSION_MODE` | `down`, `up`, `both` | `both` | The split of the task into parts, the composition back, or both. |
| `selection_mode` | `FORMAL_AI_SELECTION_MODE` | `off`, `record` | `record` | The candidates considered and why one was selected. |
| `skill_mode` | `FORMAL_AI_SKILL_MODE` | `off`, `accumulate` | `accumulate` | The ledger of skills proposed from the turn. |
| `world_model_mode` | `FORMAL_AI_WORLD_MODEL_MODE` | `off`, `track` | `off` | The world model rebuilt from the conversation. |

## Options that have no switch yet

The vision asks Formal AI to study several options, score each one against the
requirements and select the best. Where two valid options exist today, the
choice is made in code and cannot be switched. The known cases, and the issue
that owns giving each one a switch that changes behaviour, are:

| Choice | Today | Owned by |
| --- | --- | --- |
| Which candidate answers: the first matching handler, or the candidate with the best fit to the requirements | First match | [#1552](https://github.com/link-assistant/formal-ai/issues/1552), [#1556](https://github.com/link-assistant/formal-ai/issues/1556) |
| Who executes a task: a handler, or the recursive procedure (attempt, split, recompose) | Handler; the recursive procedure runs only for delegated work | [#1550](https://github.com/link-assistant/formal-ai/issues/1550) |
| How source found on the internet is used: ideas only, copy, library | Not decided per licence | [#1553](https://github.com/link-assistant/formal-ai/issues/1553) |
| Whether a stored program or a derived program answers a coding request | Stored program first | [#1554](https://github.com/link-assistant/formal-ai/issues/1554) |
| Whether `selection_mode` and `recursion_mode` change the procedure and not only the trace | Trace only | [#1556](https://github.com/link-assistant/formal-ai/issues/1556) |

## Check a switch

```bash
# The search stage of step 7 with its default budget of 512 evaluations:
formal-ai chat --prompt "Using the numbers 3, 5, and 7 with the operations + and *, find an expression that equals 26."

# The same request with the search stage turned off:
FORMAL_AI_COMPUTE_BUDGET=0 formal-ai chat --prompt "Using the numbers 3, 5, and 7 with the operations + and *, find an expression that equals 26."
```

The two replies differ, because `compute_budget` is a switch that changes the
answer. Measured with Formal AI 0.355.27 on 2026-11-10: the first reply starts
with `Found by budget-driven search: 3 * 7 + 5 = 26.`; the second has no search
stage to rely on, falls through to the web search handler and reports that no
captured source is available while live fetching is off.
