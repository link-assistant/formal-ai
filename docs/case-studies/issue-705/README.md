# Issue 705 Case Study: Anticipatory Dreaming

Status: **implemented, deterministic, and proposal-only**.

Issue [#705](https://github.com/link-assistant/formal-ai/issues/705) asks the
idle dreaming loop to predict likely next request *classes*, expand and test
those predictions before the request arrives, and prelearn public information
when the user has allowed network access. The implementation is symbolic: no
model weights, sampling, embeddings, or neural inference are involved.

## Source material

- Issue and prepared-PR snapshots: [`raw-data/`](raw-data/)
- First-order transition design: [`transition-design.md`](transition-design.md)
- Primary-source prior-art notes: [`raw-data/online-research.md`](raw-data/online-research.md)
- Reproduction probe: [`../../../experiments/issue_705_intent_probe.sh`](../../../experiments/issue_705_intent_probe.sh)
- Inspectable offline example: [`../../../examples/issue_705_anticipatory_dreaming.rs`](../../../examples/issue_705_anticipatory_dreaming.rs)
- Acceptance suite: [`../../../tests/unit/issue_705_anticipation.rs`](../../../tests/unit/issue_705_anticipation.rs)
- Live self-hosting evidence: [`self-hosting-authorship/`](self-hosting-authorship/)

The issue and its comments contain no image attachment, so there is no visual
artifact to download or compare. This is a background learning/runtime change,
not a UI change.

## Before and after

The reproduction uses a held-out name, `frobulator705`, that is absent from the
seed. Before prelearning, both `describe frobulator705 resonance` and its seeded
meaning paraphrase return `intent: unknown` in an offline, zero-compute solver.

After one consented fixture-backed prelearning run, the answer and exact source
capture are stored as append-only `anticipation_source` aliases. The held-out
paraphrase then answers offline through both Chat Completions and Responses,
with `source:http` evidence and no new transport request. Advancing the fixture
clock beyond the TTL makes the answer unavailable again.

## Acceptance evidence

The scripted history ends in `intent:greeting` and contains three observed
successors. One run therefore produces three ranked next-class predictions:

| Rank | Formal class | Transition evidence |
| ---: | --- | --- |
| 1 | `intent:calculation` | one `greeting → calculation` observation |
| 2 | `intent:text_transformation` | one `greeting → text_transformation` observation |
| 3 | `intent:unknown` | one `greeting → unknown` observation |

Equal counts are ordered by stable class id. Each prediction cites the exact
two memory events behind its transition and a `ProbabilityEvidence` record
whose model is `markov_transition`. `AnticipationPlan::why_prediction` exposes
that derivation directly.

Variants come from three inspectable sources: observed class members
(`parameter:<event>`), the seeded operation vocabulary (`operation:<slug>`),
and same-language forms in the meaning lexicon (`meaning:<slug>`). Every
variant is replayed offline. The acceptance test proves that the failed/unknown
probe set and the `anticipation` adoption frontier are identical, including the
issue-#701 `proposal_only` and `human_gated` promotion shape.

The anticipation ledger records predictions, probe results, prelearned source
captures, and subsequent `prediction_hit` links. Before any actual request the
fixture reports 0 hits and 0 basis points—0% is retained as the honest result.
A later live `2 + 2` request is recorded by `SyncStore` with evidence to both
the predicted class and the actual `chat_user_…` event.

## Runtime and safety boundaries

`run_core_dreaming_once` invokes anticipation only after the existing
foreground cancellation point. The worker already runs at the lowest practical
thread priority and only after the idle interval. It writes an inspectable
`.anticipation.lino` ledger next to the memory log.

Network work stays behind the repository's explicit `FORMAL_AI_LIVE_API=1`
consent convention. Without it, every candidate is recorded as
`consent_required` and the source transport is never called. With it, the
existing `CachedSourceClient` and source-research boundary retain URL,
`fetched_at`, SHA-256, cache status, and TTL. Warm answers come from memory and
do not fetch.

Anticipation never edits seed or behavior data. Every failed probe is fed into
the shared issue-#701 learning cycle, whose output is a human-gated promotion
proposal in the issue-#656 shape. Self-extension remains proposal-only.

## Scope and residual limits

The shipped model is intentionally first-order and bounded to the top three
classes and sixteen variants per class. Prelearned answers match normalized,
predicted aliases exactly; this avoids claiming semantic recall beyond the
evidence tested. The idle worker is native/server-side, while both OpenAI-
compatible response surfaces consume the same append-only cache. Broader
browser-only scheduling, higher-order histories, and measured prediction
quality on organic long-running logs remain future work; the ledger makes those
rates measurable without inventing a floor.
