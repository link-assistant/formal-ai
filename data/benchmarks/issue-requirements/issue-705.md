Parent: #651

## Motivation and evidence

The dreaming loop (issue #540) already recalculates which topics the user interacts with most (`TopicFrequency`) and lifts durable requirements. The next step of the vision is **anticipation**: the meta-algorithm should predict the user's likely next requests, pre-solve representative members of those classes while idle, and pre-learn the knowledge they need — so similar future questions in the topic answer immediately and correctly. This extends (not duplicates) the adoption loop: adoption fixes what already failed; prediction prepares what has not been asked yet.

Evidence of demand: #540's "learn more about the topics the user interacts with"; #527's "support all variations of top questions" (absent from PR #638); #481/#497's "general meta algorithm" asks; the Markov-style transition records already specified in `VISION.md` (R6) as symbolic evidence.

## Requirements

1. **Next-request model, symbolic**: from the append-only log, build Markov-style transition records over formalized intent classes (not raw text): `after intent-class A, class B followed n times`. Records are Links Notation with provenance — the existing `probability.rs` shapes, never neural inference.
2. **Class expansion**: for each top predicted class, enumerate paraphrase/parameter variations (via the meaning lexicon and operation vocabulary) and probe the solver offline; every `intent: unknown` or failed probe becomes a frontier item for the adoption loop.
3. **Pre-learning**: while idle (dreaming runtime, lowest priority, consent-gated for anything that fetches), fetch-and-cache the external sources those classes need under the standard provenance/TTL policy, so future answers work offline from cache.
4. **Proposal-only self-extension**: any rule/seed change produced by prediction goes through the same promotion gates (#656); prediction never mutates behavior directly.
5. **Measurable anticipation**: an anticipation ledger records predicted classes, probe results, pre-learned sources, and — when the user later actually asks something in a predicted class — a `prediction_hit` event linking back. Honest metric: hit rate can start near 0%.
6. Deterministic and inspectable: same log → same predictions; "why did you prepare this?" answers from the ledger.

## Acceptance criteria

- A scripted conversation about a topic yields ≥ 3 predicted next-request classes with transition evidence links.
- A probe pass over predicted classes files frontier items for every failure (integration with the adoption loop verified by test).
- After pre-learning from cache, a held-out prompt from a predicted class answers correctly offline where it previously returned `unknown`.
- The dreaming recipe (`data/meta/dreaming-recipe.lino`) gains the prediction stages, pinned by the grounding suite.

## Dependencies

- Blocked by the learning-adoption loop issue (E59) and #656 (E37 gates promotions).
- Builds on #540 (closed), #686 usage counting (PR #689); related #661 (E42 weights), world-models issue (E60 target-state modeling).

## Process

Collect data to `docs/case-studies/issue-{id}` (transition-record design, prior art: prefetching/anticipatory computing, symbolic user modeling — no neural components); single PR until every requirement is addressed.

