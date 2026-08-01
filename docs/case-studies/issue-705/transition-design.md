# Symbolic transition and prelearning design

## State definition

A Markov state is an `IntentClass`, never the raw prompt. The current class is
derived with the public intent formalizer and carries the canonical intent,
formalization kind, optional route, and detected operation slugs. Historic
events that already have a recorded intent retain that explicit provenance;
ordinary chat events are formalized from their prompt.

Only adjacent user requests in the same conversation contribute a transition.
For every `(from, to)` pair the planner records:

- count for that pair;
- total outgoing observations from `from`;
- `count / outgoing` probability;
- the exact source memory-event ids;
- a `ProbabilityEvidence` record with `MarkovTransition` and `transition_from`.

The last observed request class is the current state. Its outgoing transitions
are ranked by descending count and then stable class id, capped at three. This
makes tie handling and serialized output deterministic.

## Expansion and offline probing

For each predicted class, the planner walks historic members in append order.
It adds the observed request as a parameter variant, replaces matching seeded
operation phrases with other same-language phrases, and replaces matching
meaning forms with other forms of that meaning. A normalized-prompt set removes
duplicates without hash-order dependence.

Every variant is evaluated by `UniversalSolver` with `offline = true` and
`compute_budget = 0`. A probe is `passed`, `unknown`, or `failed`; all outcomes
retain solver intent and evidence. There is no sampling and no hidden score.
Every result other than `passed` is copied, one-for-one, into a `FrontierItem`
and sent to `run_learning_cycle("anticipation", …)`.

## Prelearning and recall

Prelearning considers the first unresolved probe for each top prediction. A
denied consent produces only a `consent_required` attempt. Granted consent uses
the shared source-research client, takes a grounded result excerpt, and retains
the corresponding `SourceCapture` metadata. Every generated alias sharing the
probe's base observation receives an append-only cache event with the same
answer, prediction link, source link, and expiry.

Production solving consults this cache only after the ordinary solver returns
`intent: unknown`. Existing capabilities therefore retain precedence. Recall
requires an exact normalized alias and an unexpired capture, and emits source,
prediction, and cache-hit evidence.

## Prediction hits and inspectability

When a later live chat exchange is persisted, its formal class is compared with
the stored prediction class ids. A match appends `prediction_hit` with links to
the prediction and actual request. The per-run ledger counts those links and
reports integer basis points over the number of predictions. With no hits it
reports zero; absence of evidence is never converted into a positive score.

Content-addressed ids, `BTreeMap`/`BTreeSet` ordering, stable tie-breaking, a
fixed maximum expansion, and no clock in planning make two plans from the same
history byte-identical. Time appears only at the source-capture/TTL boundary.
