# Self-Improvement Loop From Unknown Traces

Issue #364 adds the first white-box self-improvement loop on top of the
rule-synthesis work from issues #356-#363. The loop does not train a hidden
model and it does not silently mutate seed data. It proposes explicit Links
Notation rules from accumulated unknown-origin traces, then blocks adoption
unless the existing verification and benchmark gates pass.

## Inputs

The loop consumes solver event logs, not opaque text transcripts. A trace is in
scope when it has either:

- an `intent` of `unknown`,
- a `reasoning:unknown` event, or
- a `selected_rule` event whose payload records `initial unknown`.

That third case matters for the issue #349 class: the final answer can become
`write_program` after rule synthesis, but the turn still started from the
unknown path and is useful learning material.

## Proposal

For each accumulated trace, the loop searches for:

- `rule_synthesis_candidate` with `id`, `base_task`, `modifier`, and
  `resolved_task`,
- `rule_verification` with `status passed`, and
- the issue #362 coding-modification benchmark report.

Only then does it render a candidate seed rule:

```text
substitution_rules
  id "learned_program_plan_rules"
  rule "reverse_sort_list_files"
    order "90"
    event "learned"
    when "request:modifier -> reverse_sort"
    replace "request:task -> list_files"
      with "request:task -> list_files_reverse_sort"
```

The learned artifact is ordinary Links Notation, so a reviewer can inspect it,
parse it with the same substitution engine as manual rules, and compare it with
the trace that produced it.

## Gates

A proposed rule is adoptable only when both gates pass:

1. The rule-synthesis trace says `rule_verification status passed`.
2. The benchmark report for `data/benchmarks/coding-modification-suite.lino`
   has `passed >= minimum_pass_count`.

If either gate fails, the loop records a rejection or a
`blocked_by_benchmark` proposal. No rule is appended to `data/seed/` by this
module; adoption remains a reviewed change.

## Runtime Contract

The Rust API is:

- `UnknownTrace::from_event_log` to accumulate unknown-origin traces.
- `BenchmarkGateReport::issue_362_from_counts` to bind the benchmark ratchet.
- `learn_rules_from_unknown_traces` to produce a `LearningRun`.
- `LearningRun::links_notation` to review proposed and rejected traces.

The seed policy is recorded in `data/seed/self-improvement-loop.lino` so memory
exports and bundled seed data describe the same loop.
