# Issue 701 Case Study: Closing The Auto-Learning Adoption Gap

Status: **delivered** — the loop now adopts, and every adoption is evidenced.

Issue [#701](https://github.com/link-assistant/formal-ai/issues/701) (E59) states
the problem precisely: the engine had a learning *loop* that never closed. It
observed its own failures, recorded them faithfully, and then stopped. Two
symptoms were named:

1. The Google Trends frontier from issues [#498](https://github.com/link-assistant/formal-ai/issues/498)
   and [#499](https://github.com/link-assistant/formal-ai/issues/499) sat at 60
   unanswered prompts, run after run, with nothing derived from it.
2. Dreaming amendments from [#540](https://github.com/link-assistant/formal-ai/issues/540)
   (R413) were *decoration*: the retained rule was appended to the answer as a
   compliance line, and the answer itself was unchanged.

The issue's demand is stronger than "learn something": **learned items must
demonstrably change answers**, with a before/after pair on record for each one.

## Source material

- GitHub issue: <https://github.com/link-assistant/formal-ai/issues/701>
- Pull request: <https://github.com/link-assistant/formal-ai/pull/817>
- Raw GitHub snapshots: [`raw-data/`](raw-data/)
- Online research notes: [`raw-data/online-research.md`](raw-data/online-research.md)
- Frozen pre-adoption frontier (the durable failure record):
  [`../../../data/meta/learning-frontier-google-trends.lino`](../../../data/meta/learning-frontier-google-trends.lino)
- Adoption ledger (the before/after capability deltas):
  [`../../../data/meta/learning-adoption-ledger.lino`](../../../data/meta/learning-adoption-ledger.lino)
- Trends learning report (the coverage split):
  [`../../../data/meta/google-trends-learning.lino`](../../../data/meta/google-trends-learning.lino)
- Promoted surfaces: [`../../../data/seed/learned-request-openers.lino`](../../../data/seed/learned-request-openers.lino)
- Scheduled proposal-only run: [`../../../.github/workflows/learning-cycle.yml`](../../../.github/workflows/learning-cycle.yml)

## 1. Frontier census — the before state

The committed Trends corpus is 10 trending topics × 4 supported languages
(`en`, `ru`, `hi`, `zh`) × 2 request variations = **80 prompts**. Before this
work the engine routed 20 of them and left 60 as `intent: unknown`:

| Measure | Before | After |
| --- | ---: | ---: |
| Corpus prompts | 80 | 80 |
| Routed by the engine | 20 | 80 |
| On the learning frontier | 60 | 0 |
| `intent: unknown` rate | 7500 bp | 0 bp |

The 60 failures split cleanly by language and variation, which is what made them
learnable as a *class* rather than 60 separate facts: the engine already knew
every topic term, and already handled the English `trends_context` variation.
What it lacked was the **request opener** — the surrounding surface a speaker of
each language wraps around a term to ask about it (`… के बारे में बताओ`,
`介绍一下 …`, `Дай контекст Google Trends для …`). That is a *slot form*, not a
fact, and it generalizes over every term.

The before state is not deleted. `data/meta/learning-frontier-google-trends.lino`
freezes all 60 pre-adoption verdicts permanently, so the failure stays auditable
after the gap closed (R425, R489).

## 2. The adoption contract

`formal-ai learn cycle --frontier google-trends --dry-run` runs one cycle:

```
frontier item  →  candidate surface  →  held-out validation  →  promotion proposal
```

- **Frontier item** — an unanswered prompt from the frozen record, with its
  language, variation, and the term it was built from.
- **Candidate** — a request-opener surface generalized from ≥2 distinct terms,
  so a candidate can never be a memorized single prompt. The `…` position gives
  the slot form (prefix / suffix / circumfix).
- **Validation** — every candidate is replayed against **held-out** prompts of
  its class: terms it was *not* derived from. A candidate that does not recover
  the topic on the held-out set is rejected and recorded as blocked.
- **Proposal** — a `PromotionProposal` in the exact shape issue
  [#656](https://github.com/link-assistant/formal-ai/issues/656) consumes,
  carrying a `SeedEdit` against `data/seed/learned-request-openers.lino` and the
  canonical gate set. Rendering round-trips through `parse_promotion_proposals`.

One recorded run: 60 frontier items → 6 candidates → 6 validated → 48 held-out
tests → 2 promotion proposals, 0 blocked classes. The run is deterministic and
offline: it reads only committed data, so two runs in the same tree are
byte-identical.

Nothing is adopted by the cycle. `mode "proposal_only"`, `human_gated "true"`,
and the scheduled workflow fails if `git diff -- data/seed` is non-empty.

## 3. Proving the capability delta

A proposal that is accepted must be shown to have *changed something*. The
adoption ledger (`data/meta/learning-adoption-ledger.lino`) records one row per
adopted item:

```
adoption_pair
  rank "1"
  topic "julián andrés quiñones"
  language "en"
  variation "trends_context"
  prompt "Give Google Trends context for julián andrés quiñones?"
  before_intent "unknown"
  before_routed_to "human_triage"
  after_intent "web_search"
  after_query "julián andrés quiñones"
  topic_recovered "true"
  capability_delta "unknown_to_web_search"
```

60 such pairs, across 10 topics and all 4 languages. Each is a real before/after:
`before_intent` is the recorded pre-adoption verdict, `after_intent` is what the
current engine produces for the same prompt, and `topic_recovered` checks that
the routed query actually carries the term rather than the opener boilerplate.

The ledger is tool-authored and byte-pinned: `committed_adoption_ledger_matches_a_fresh_run`
compares the committed file against a fresh render, so it cannot drift.

## 4. Deleting the decoration-only path

The audit is reproducible: `cargo run --release --example issue_701_amendment_body`
solves a prompt twice — plainly, and with a matching retained amendment — and
diffs the bodies with the appended compliance line stripped. Before the fix:

```
--- latex: solve a new recurrence proof
plain intent   = unknown
amended intent = unknown
body differs beyond the appended line = false
```

The root cause was not the projection but the *consumption*:
`solve_with_amendment_records` prepends a `Standing requirement (...)` user turn,
`solve_with_history` records it as a `prior_turn:user` event, and no downstream
handler ever reads it. The rule reached the log and the prose, never the routing.

The fix makes the amendment change the **classification** of the task, which a
string append cannot do: when a retained standing requirement matches a task the
solver could not otherwise route, the answer stops being reported as unresolved
(`STANDING_REQUIREMENT_INTENT`) and cites the amendment as evidence. The three
application paths (solver, memory recall, agentic final) were also collapsed onto
one selection rule and one projection, deleting the duplicated
`matching_amendment_lines` decorator.

`tests/unit/issue_701_dreaming_amendment_class.rs` raises the single-prompt
issue-#540 check into a class:

- 3 topics × 4 languages of **held-out paraphrases** — prompts absent from the
  stored requirement and run events — through both `create_chat_completion_with_solver_and_memory`
  and `create_response_with_solver_and_memory`;
- an assertion that a covered task is *never* returned unresolved;
- a **negative control**: an uncovered task must come back byte-identical to the
  empty-memory answer, so "the answer changed" is evidence of something.

## 5. Running it periodically

A loop that runs once is a script. Two schedules exist, both proposal-only:

- **In-process** — `run_core_dreaming_once` writes a learning-cycle record next
  to the memory log on every idle dreaming run.
- **Scheduled CI** — `.github/workflows/learning-cycle.yml` runs daily, publishes
  the proposals as a 30-day artifact, and asserts the run stayed gated and wrote
  no seed file. A cycle that stops proposing anything fails the job rather than
  passing quietly.

## 6. Adoption rate, before and after

| | Before | After |
| --- | ---: | ---: |
| Frontier items observed | 60 | 60 (frozen) |
| Items adopted | 0 | 60 |
| Adoption rate | 0% | 100% |
| Capability pairs on record | 0 | 60 |
| Topics covered | — | 10 |
| Languages covered | — | 4 |

The adoption rate is the number the issue was really about: the loop's output
used to be a report about itself.

## 7. What we borrowed, and what does not transfer

See [`raw-data/online-research.md`](raw-data/online-research.md) for the sourced
notes. In short: Voyager's *verified skill library* maps onto validated surfaces
promoted into `data/seed/` behind the human gate, with held-out tests as the
verification step; Reflexion's episodic reflection buffer maps onto the durable
frontier record of every failure to adopt; an automatic curriculum maps onto
driving the frontier until the corpus is exhausted. What does not transfer is
everything that depends on weights or sampling — there is no gradient step here,
no exploration noise, and no acceptance path that a human did not open.
