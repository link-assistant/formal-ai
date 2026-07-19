# Rule Synthesis Over Links Notation

Issue #356 defines the design contract for moving program modifications from
memorized allowlist entries to white-box rule synthesis over Links Notation. The
immediate motivating class is a bare imperative follow-up such as "sort the
results in reverse order" after the conversation already produced a
file-listing program.

The goal is not to replace the symbolic machinery already in the repository.
The goal is to add a construction step before the `unknown` path: when no seed
rule matches, decompose the request, construct a candidate substitution rule
from known operation primitives, verify it, then either apply it or ask one
clarifying question.

## Keep / Replace

Keep:

- The symbolic substitution engine in `src/substitution.rs`. It already parses
  link-pattern rules, applies them to a graph, and emits an inspectable trace.
- The program-plan lowering surface in `src/program_plan.rs`. `lower` should
  continue to accept a base task and modifiers, seed a graph, and apply
  substitution rules to a fixpoint.
- Seeded rules in `data/seed/program-plan-rules.lino`. A hand-reviewed seed is
  still the fastest path when the rule is already known.
- The operation lexicon in `data/seed/operation-vocabulary.lino`. The
  vocabulary is the source of natural-language operation candidates and should
  grow into a shared operation model instead of becoming another code allowlist.
- Existing diagnostic/event-log surfaces. New synthesis steps must extend those
  surfaces rather than creating an invisible side channel.

Replace:

- The hard-coded `PROGRAM_MODIFIERS` allowlist in
  `src/intent_formalization.rs`. It should become data-driven operation and
  modifier recognition sourced from seed data. **(Done — issue #358 removed the
  allowlist; modifier recognition now reads `data/seed/operation-vocabulary.lino`
  and `data/seed/program-plan-rules.lino`. The name survives in this document as
  the historical target.)**
- Literal-match-only failure for resolvable modifications. If route selection
  reaches `SelectedRule::Unknown` for a prompt with an active program artifact,
  the solver must try rule synthesis first.
- Single-rule-only program modification. `path_argument` remains a valid seed
  rule, but it is no longer the architecture.
- Unverified rule use. A synthesized candidate is not offered to the user until
  it passes TDD verification.

## Pipeline

The rule-synthesis pipeline runs after seed routing and seed substitution have
failed, but before any user-visible `unknown` answer.

1. Normalize the current prompt and collect the conversation state.
2. Bind a target artifact through coreference (#357).
3. Decompose the bare imperative into `(operation, target)`.
4. Search seed rules for a direct match.
5. If no seed rule matches, construct a candidate substitution rule from
   operation primitives.
6. Verify the candidate with a minimal TDD fixture.
7. Apply the verified candidate, optionally persist it, and emit diagnostics.
8. If decomposition is ambiguous, ask at most one clarifying question.

The important ordering is coreference before construction, construction before
unknown, and verification before user-visible output.

## Decomposition

A bare imperative is a command that omits the explicit object because the object
is recoverable from conversation history. For example, after a program was
generated, "sort the results in reverse order" has an implicit object: the
active program artifact's output ordering.

The decomposition result is a Links Notation graph, not just a Rust enum:

```text
rule_synthesis_request
  issue "#356"
  impulse "current_turn"
  artifact "program:last"
  artifact_language "rust"
  base_task "list_files"
  bare_imperative "true"
  operation "sort"
  operation_modifier "descending"
  target "program:last.output_order"
  target_kind "program_output"
```

The `(operation, target)` pair is therefore:

```text
operation: sort + descending
target: program:last.output_order
```

The decomposition should be assembled from inspectable evidence:

- Operation phrases and combos from `data/seed/operation-vocabulary.lino`.
- Artifact candidates from conversation history and the last `write_program`
  plan/evidence.
- Target nouns such as `results`, `output`, `lines`, `files`, and their
  multilingual seed equivalents.
- Existing task metadata from the program catalog, such as `list_files`.

If multiple targets have comparable evidence, the system asks one clarifying
question instead of constructing a rule blindly.

## Candidate Rule Construction

A candidate substitution rule is constructed as data. It is never introduced as
new hidden control flow.

Construction inputs:

- `base_task`: the current program-plan task, for example `list_files`.
- `operation`: the normalized operation primitive, for example `sort`.
- `operation_modifier`: optional operation metadata, for example `descending`.
- `target`: the bound target, for example `program:last.output_order`.
- `available_variants`: task variants already present in the program catalog.
- `seed_rules`: current rules from `data/seed/program-plan-rules.lino`.

For reverse sorting, the first candidate can be represented as a compound
modifier:

```text
rule_synthesis_candidate
  id "reverse_sort_list_files"
  source "constructed"
  base_task "list_files"
  modifier "reverse_sort"
  operation "sort"
  operation_modifier "descending"
  target "program:last.output_order"
```

The lowering rule remains ordinary Links Notation:

```text
substitution_rules
  id "synthesized_program_plan_rules"
  rule "reverse_sort_list_files"
    order "20"
    event "manual"
    when "request:modifier -> reverse_sort"
    replace "request:task -> list_files"
      with "request:task -> list_files_reverse_sort"
```

The rule constructor should prefer these strategies, in order:

1. Reuse an existing task variant if the catalog already has one.
2. Compose existing modifiers if their verified effects satisfy the target.
3. Create a new task variant candidate only when a renderer can verify it.
4. Stop and ask one clarifying question when the target or operation cannot be
   bound uniquely.

The constructed rule is deliberately the same shape as a seed rule. That keeps
the behavior auditable and lets #358 persist useful candidates back into
`data/seed/program-plan-rules.lino` after review.

## TDD Verification

TDD verification is mandatory. A candidate substitution rule is untrusted until
it passes the smallest fixture that proves the requested behavior.

For the reverse-sort class, the verification fixture should prove semantics, not
just a string pattern:

```text
rule_verification
  candidate "reverse_sort_list_files"
  fixture "list_files_output_order"
  input "a.txt,b.txt,c.txt"
  expected_order "c.txt,b.txt,a.txt"
  status "passed"
```

Implementation checks:

- `program_plan::lower_with_rules` resolves the base task to the expected task
  variant.
- The rendered program contains a descending-order behavior, for example
  `sort_by(|a, b| b.cmp(a))` or `sort(); reverse();`.
- A unit or integration test exercises the same multi-turn request shape.
- The verification trace is attached to diagnostics and to any persisted rule.

If the candidate fails verification, the solver must not offer it. It may try
another candidate within a bounded search budget; otherwise it should ask one
clarifying question or return an unknown answer with the failed trace.

## Interaction With #357 Coreference

Coreference owns target binding. Rule synthesis should not guess that a bare
imperative modifies the last program unless the history supports that binding.

Required coreference output:

```text
coreference_binding
  issue "#357"
  mention "results"
  antecedent "program:last.output"
  confidence "high"
```

When #357 cannot bind the target:

- If there is exactly one active program artifact, ask whether the user wants to
  modify that program.
- If there are multiple plausible artifacts, ask which artifact to modify.
- If there is no active artifact, do not synthesize a program-modification rule.

The synthesis step consumes `coreference_binding`; it does not own query
rewriting.

## Interaction With #358 Modification Model

Issue #358 (now closed) removed `PROGRAM_MODIFIERS` and turned modifiers into
data. This design gave #358 the target model:

- Modifier recognition comes from seed operation data, not a Rust slice.
- Modifiers compose as graph links: multiple `request:modifier -> ...` links
  can be present at once.
- Existing program-plan lowering remains the execution path.
- Synthesized candidates use the same rule format as seeded modifiers.

For example, path argument plus reverse sort should lower from:

```text
request
  task "list_files"
  modifier "path_argument"
  modifier "reverse_sort"
```

to a task or render plan that accepts a path argument and emits descending
results. If the task catalog represents these as separate variants, #358 must
define deterministic composition order and tests for at least two modifiers.

## Interaction With #359 Unknown-Path Construction

Issue #359 wires this design into the solver's unknown path. The required
control flow is:

```text
selected_rule
  initial "unknown"
  reason "no_seed_route"
  next "try_rule_synthesis"
```

The unknown path should only produce an unknown answer after:

- coreference failed or asked its single clarification,
- decomposition failed,
- candidate construction found no bounded candidate, or
- all candidates failed TDD verification.

For resolvable modifications, `unknown` is not reached.

## Diagnostics

Diagnostics are off by default, but when enabled the trace must show each
synthesis decision:

- route attempts and why no seed route matched,
- coreference binding from #357,
- `(operation, target)` decomposition,
- operation-vocabulary hits,
- seed-rule lookup result,
- constructed candidate rule,
- TDD verification fixture and result,
- final substitution trace from `src/substitution.rs`.

The trace should be emitted as Links Notation so it can be exported with memory
and reused by the self-improvement work later in the roadmap.

## Prior Art Evaluation

The relevant prior art supports a hybrid shape: use learned or heuristic search
to propose symbolic rules, then verify those rules symbolically.

- [Neuro-Symbolic Program Synthesis](https://arxiv.org/abs/1611.01855)
  proposes synthesizing explicit programs from examples, addressing the
  interpretability and verification limits of pure neural mappings. The useful
  lesson here is not to add a neural model now; it is to keep the output as an
  explicit candidate program/rule that can be checked.
- [Learning Compositional Rules via Neural Program Synthesis](https://arxiv.org/abs/2003.05562)
  frames few-shot generalization as induction of explicit rule systems. The
  useful lesson for this repository is compositionality: `path_argument` and
  `reverse_sort` should combine as rules rather than becoming bespoke cases.
- [Proof of Thought](https://arxiv.org/abs/2409.17270)
  emphasizes an intermediate symbolic representation with verification. The
  useful lesson is the verification gate: no generated reasoning artifact should
  be trusted until a checker validates it.
- [CREAD](https://arxiv.org/abs/2105.09914) jointly models dialogue
  coreference and query rewriting. The useful lesson for #357 is to make the
  bare imperative self-contained before rule synthesis begins.

## Acceptance Checklist

- `docs/design/rule-synthesis.md` is merged for Issue #356.
- The design decomposes a bare imperative into `(operation, target)`.
- Candidate substitution rule construction is specified using
  `data/seed/operation-vocabulary.lino` and the program-plan rule format.
- TDD verification is required before a constructed rule is offered.
- Coreference #357, modification model #358, and unknown-path construction #359
  each have an explicit integration contract.
- The document states what is kept and what is replaced, including keeping the
  symbolic substitution engine and replacing `PROGRAM_MODIFIERS`.
