# Issue #1073: a reasoning depth floor that cannot be switched off

Issue [#1073](https://github.com/link-assistant/formal-ai/issues/1073) takes one
reference dialog — a disk-space cleanup session — and asks for the properties
that made it good to become the *floor* rather than the ceiling: evidence before
claims, adversarial self-check before acting, honest reporting of partial
failure, and re-measurement after every action. The instruction that makes this
hard is the one attached to it: the floor holds for the trivial request too, and
it is never conditional on being asked.

That rules out the shape this repository already had. Depth existed, and it was
opt-in.

## What contradicted the requirement

Three findings from the audit the issue asks for, each fixed rather than
documented.

### Depth was a mode

`RecursionMode` defaulted to `Down`, `SelectionMode` to `Off`, `SkillMode` to
`Off`. The upward construction pass, the selection trace, and the skill ledger
were all reachable, and all silent unless a caller turned them on — which is
exactly "conditional on explicit prompting". The defaults are now `Both`,
`Record` and `Accumulate`. The narrow modes still exist, but as a deliberate
*quietening* of a trace, never as the resting state.

The reasoning-standard audit itself takes the argument one step further: it has
no mode at all. `record_meta_core` calls it unconditionally, because a depth
floor with a switch is not a floor.

### Source trust was asserted

`data/seed/sources-registry.lino` carried a hand-written `source_tier` per
source, and `tier_from_seed` read it verbatim with a silent
`_ => IndependentCorroboration` fallback. Nothing derived the tier from
anything; nothing could disagree with it.

Every source now declares a **primacy chain**: how many hops it stands from the
primary record, what its upstream is at each hop, and the basis for saying so —
in each case the site's own policy. Wikipedia declares `editorial_synthesis`
upstream of "the published sources each article cites", citing
*Wikipedia:No original research*; a GitHub repository declares `self_published`
with no upstream, because a project's README is the subject speaking about
itself. `SourceRecord::tier` is now `primacy.derive_tier()`, and the old
hand-written value survives only as `asserted_tier` — which a test checks
against the derivation, so a disagreement is a failure instead of a preference.

Conflicts resolve toward the source closer to the primary record;
equal distance yields `ConflictResolution::Unresolved` rather than a coin flip.

### Refutation-first lived in one handler

`src/fact_checking.rs` knew how to try to break a claim. Nothing applied that to
conclusions in general. The `refutation_variety` gate now does, and it counts
variety rather than volume: three attempts with distinct *mechanisms*, spanning
at least two of the three kinds (a different mechanism, a different source, a
different denied assumption). Three restatements of one doubt are one
refutation. Until each attempt is itself refuted by evidence, or one of them
positively proves its alternative, the verdict is
`not_confirmed_not_refuted` — and the audit names the gates that failed and the
checks it could not run, which is the honest answer the issue asks for, not a
fallback.

## The standard is data, the audit is a predicate

[`data/meta/reasoning-standard.lino`](../../../data/meta/reasoning-standard.lino)
declares seven gates, each with an order, a trigger, a requirement, the
behaviour in the reference dialog it was adopted from, and the slug it reports
on failure — plus five numeric thresholds with the reason each number is what it
is. `standard()` loads it; `audit()` evaluates every gate as a pure predicate
over a `ReasoningEpisode`, the record of one pass: the observations it executed,
the claims it made, the sources it weighed, the refutations it attempted, the
actions it took.

Nothing in that path consults a model. Requirement 6 asks for the pipeline to
stay conclusive with the LLM removed, and the test
`the_standard_is_a_formal_procedure_that_replays_without_a_model` is the check:
the same episode replays to the same verdict and the same event payloads.

A gate that is not triggered reports `NotTriggered` **naming the trigger that
was false**. This is why the trivial request is covered: `open_episode` opens an
episode carrying only the request's identity and task class, so every gate
reports the obligation it would have imposed. The enumerated checklist is the
point — the obligations are listed on the trivial request exactly as on the hard
one.

The greeting `"hi"` is the smallest case there is, and it is not vacuous. Six of
its gates report `not_triggered` with the trigger that was false, and the
seventh, `instruction_formalization`, fires on the task class alone and reports
`violated` with `courtesy:no_instructions_gathered` and
`instruction_sources:0:required:2` — the standard says what it would have needed
rather than passing the request because it looked small. The verdict is
`not_confirmed_not_refuted` with all three blockers named. Both ledgers are
committed at
[`logs/reasoning-standard-audit.log`](logs/reasoning-standard-audit.log).

## What the thirteenth stage moved

Adding a stage to the recursive core is not a local change: several committed
artifacts are content-addressed over the trace it emits, and a stale one shows
up as a test failure minutes into the suite rather than as an error at the edit.
Each was regenerated through its own generator, never hand-edited to match.

- `data/meta/self-ast/src/` — the crate's census of its own AST gained
  `reasoning_standard/` and rewrote the modules that changed
  (`cargo run --example regenerate_self_ast_census`).
- `data/seed/closure-generated-*.lino` — the four primacy kinds `citation`,
  `first_hand_record`, `self_published` and `editorial_synthesis` are unquoted
  value tokens in the new `primacy` chains, so total closure had to define them
  (`python3 scripts/close-total.py`); `data/meta/seed-metadata-gaps-*.lino` then
  recorded their metadata gaps, taking the audited count from 3,940 to 3,944.
- `examples/issue-922-method-learning/open-proposals.lino` and
  `data/seed/learned-methods.lino` — the learned recursive-core method is
  content-addressed over the shared trace tail, which grew from twelve
  operations to fifteen. The proposal document is now regenerated by
  `cargo run --example regenerate_issue_922_open_proposals`, added to
  `scripts/regenerate-derived-artifacts.sh`; the adopted seed was re-derived
  through the production promotion path with its three canonical gates replayed
  fresh, because adopting a method is a human-confirmed step and not something a
  regeneration script may do. See
  [`logs/issue-922-promotion-rerun.lino`](logs/issue-922-promotion-rerun.lino).

Two failures found on this branch were **not** caused by it. `v0.346.0` deleted
the changelog fragments that `tests/unit/ci-cd/issue_1014.rs` and
`tests/unit/issue_1021_closed_circle.rs` read, so both had been failing on
`main` since the release; they now follow the entry across its lifecycle, the
way `tests/unit/docs_requirements_issue_656.rs` already did.

## What Formal AI wrote here

One commit on this branch was authored by Formal AI itself, through the live
loop [`scripts/author-change-with-formal-ai.sh`](../../../scripts/author-change-with-formal-ai.sh)
drives: `formal-ai serve` behind the real `@link-assistant/agent` CLI, given a
workspace holding the two documents that make claims about the standard --
[`contradiction-audit.md`](contradiction-audit.md) and the R1073 requirements
shard -- and asked to weigh the statements in them.

[`formal-ai-authorship/reasoning-standard-statement-audit.lino`](formal-ai-authorship/reasoning-standard-statement-audit.lino)
is what it wrote: 70 statements weighed, 0 contradictions, 9 findings, 0 paths
skipped. Each finding names the statement it declines to accept at face value.
The raw session traces are committed beside it under
[`formal-ai-authorship/evidence/`](formal-ai-authorship/evidence), and the commit
names that session in its `Formal-AI-Session` trailer.

To reproduce it, put the two documents in a scratch directory and point the
loop at it:

```sh
seed="$(mktemp -d)"
cp docs/case-studies/issue-1073/contradiction-audit.md \
   docs/requirements/issue-1073-reasoning-standard.md "$seed"/
scripts/author-change-with-formal-ai.sh \
  --task "Audit all statement-bearing repository prose, code comments, and structured facts; weigh conflicting requirements and captured original-source evidence with probabilities; persist findings and associations; and write statement-audit.lino." \
  --produces statement-audit.lino \
  --into docs/case-studies/issue-1073/formal-ai-authorship/reasoning-standard-statement-audit.lino \
  --evidence docs/case-studies/issue-1073/formal-ai-authorship/evidence \
  --pull-request https://github.com/link-assistant/formal-ai/pull/1074 \
  --message "docs(issue-1073): audit the claims this branch makes about the standard" \
  --seed "$seed" --contains repository_statement_audit
```

The scope is deliberate. An earlier run seeded the whole case-study directory
and produced a 2,936-line audit, past the 1,500-line ceiling R222-1 puts on a
`.lino` file; the answer there is to narrow what is audited, not to exempt the
artifact.

## Grounding

The reference dialog is encoded as
[`data/meta/reasoning-standard-reference-episode.lino`](../../../data/meta/reasoning-standard-reference-episode.lino)
and clears all seven gates with verdict `confirmed`. The regression the issue
asks for is the mutation set in
[`tests/unit/issue_1073_reasoning_standard.rs`](../../../tests/unit/issue_1073_reasoning_standard.rs):
every gate is shown to fail when the behaviour it exists to enforce is removed —
strip the evidence behind a claim, reorder it to arrive after the claim, drop
the re-measurement after an action, round a partial failure up to a success,
erase the reason it was partial, assert a tier with no primacy chain behind it,
leave an instruction step without a check, consult nothing primary, restate one
doubt three times. A gate that cannot be made to fail is not a gate.

[`data/meta/reasoning-standard-recipe.lino`](../../../data/meta/reasoning-standard-recipe.lino)
describes the procedure as data — eight steps, seven gates, nine pinned
functions — and
[`tests/unit/specification/reasoning_standard_meta_algorithm.rs`](../../../tests/unit/specification/reasoning_standard_meta_algorithm.rs)
keeps it grounded against the live source, including a test that asserts no
condition stands between the meta core and the audit call.

To reproduce both ledgers, run:

```sh
cargo run --example dump_reasoning_standard_audit
```
