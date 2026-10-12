# Issue #901 Case Study: Automating TRIZ and General Paradox Resolution

Issue [#901](https://github.com/link-assistant/formal-ai/issues/901).
Case study written on the `qa-reasoning-coding-bulk-fixes` branch.

## What the issue asks

> TRIZ … the most attention goes to abstraction of ways to generalize
> technical contradiction … each such contradiction or union of
> different criteria/metrics of trade offs are links in our theory.
> Where we can assign value from 0 to 1 or from -1 to 1 … to actually
> solve the binary contradiction by selecting 50% or 10% or 80% …
> We should also collect all other common ways to solve paradoxes …
> We should test that on top 20 commonly discussed tasks … And we need
> to make sure we put it in use in all relevant places in our codebase.

## What existed

- `data/seed/triz-principles.lino` (issue #1138): the forty inventive
  principles and four separation principles as link data.
- `rust/src/selection_heuristics.rs` (#901's lane, plan 12): candidate
  contradictions as links with a **basis-point value counted from the
  requirement's clauses** — 0-1, no default 50 %. Part (1) of the issue
  already held there, and this change does not touch that file.

## Solution (this pull request)

Part (2) — the *other* ways, as data. Twelve `triz_resolution_family`
records appended to `data/seed/triz-principles.lino` (mirrored at
rust/embedded/data/seed/): range selection (the user-facing face of the
basis-point link), dimension change, separation in time / in space /
upon condition, escalate to supersystem, descend to subsystems, phase
and state transition, accumulate then release, invert the problem,
bypass and redefine, partial-spectrum solution. Each carries
`applies_to` (which kind of contradiction it dissolves), `mechanism`,
and a worked `example`. No Rust branch names a family — the catalogue
stays forgettable and rediscoverable, like the principles.

Part (3) — the top-20 corpus. Twenty `triz_benchmark_task` records:
commonly discussed invention challenges (pill coating, wheat from
stones, bulletproof glass, turbine blade cooling, the umbrella in a
crowd, the drinks can, pipeline pig tracking, Mars dust, bulb testing,
chimney height, blind riveting, the ice-cream cart, smartwatch battery,
thin-coating metrology, firefighting water, truck braking, bridge
corrosion, vending coins, bottle cutting, wet-grass mowing), each with
its `contradiction` and the `methods` that solve it. Contradictions are
free text "A vs B"; criterion slugs are used only where
meanings-selection-criteria.lino grounds them, per the seed's header.

Part (4) — put in use. `rust/src/triz_solver.rs`: a user-facing entry
(`triz_families`, `triz_benchmark_tasks`, `handle_triz`) that reads the
seed, triggers on `triz_cues` phrases (противоречи-, парадокс, triz,
contradiction, trade-off, inventive problem…), cites the benchmark
tasks that share words with the prompt (umbrella prompts find the
umbrella task), and renders the localized template from
`data/seed/multilingual-responses-triz.lino` (five languages). The
answer states the range-selection link explicitly: value in 0-1 from
the requirement's clauses, basis points, no default 50 %.

## Verification

- `rust/tests/unit/issue_901_triz_solver.rs` (CI): the seed carries
  twelve families and exactly twenty tasks, every task states a
  contradiction and cites only real method ids (a module unit test
  validates the citations against families + principle ids); a Russian
  contradiction prompt gets the family map with the link note; the
  umbrella prompt finds its precedent with `family_space_separation`;
  the TRIZ/paradox cues trigger and ordinary prompts decline; the
  handler logs `triz_solver:cued` and `triz_solver:precedent`.

## Integration site left for main

- `rust/src/lib.rs`: `pub mod triz_solver;`.
- `rust/tests/unit/mod.rs`: register `issue_901_triz_solver`.
- Seed registry row for `data/seed/multilingual-responses-triz.lino`
  (the extended `triz-principles.lino` is already registered; its
  embedded mirror is byte-identical, `cmp`-verified).
- Dispatch entry for `handle_triz` (precedence after the concrete
  task handlers — it is a teaching/meta layer, not a task executor).

No Cargo.toml changes: std only.

## Residuals

- The link note inside the answer is English in every locale (the
  template around it localizes); localize `LINK_NOTE` when the response
  registry grows a per-language field for it.
- The benchmark replay is lexical (word overlap), which is honest about
  what it is; a criterion-slug contradiction extractor over prompts
  would tighten it and belongs with the selection-criteria seed's own
  lane.
- Deeper "use in all relevant places": the coding meta-algorithm could
  consult `family_bypass_redefinition`/`family_partial_spectrum` when a
  build constraint conflicts with a requirement — a follow-up for the
  agentic-coding lane (not this fork's ownership).
