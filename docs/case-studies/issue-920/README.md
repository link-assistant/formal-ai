# Issue 920: question-necessity protocol

Formal AI now treats a user-facing question as an authorized last resort. Before
projecting one into an answer, the solver records a stable question identifier
and three ordered checks:

1. conversation memory did not answer it;
2. workspace evidence could not derive it;
3. reachable sources did not answer it within the configured budget.

The seed-backed classifier then distinguishes a requirement only the user can
choose from a factual unknown the solver must research. A missing trace, factual
unknown, or second semantic question is refused and removed from the answer.
The refusal and any research handoff stay in the append-only event log.

## Reproduction and verification

Before the implementation, this focused command produced two failures: the
ambiguity question had no `question_necessity:memory` evidence, and the unknown
route delegated a factual lookup to the user, producing two questions.

```text
cargo test --test unit issue_920_ -- --nocapture
```

The regression suite now covers:

- replay-identical three-stage traces for clarify-vs-guess;
- refusal of a question whose trace is incomplete;
- factual-unknown research handoff;
- proof follow-up lists without question-mark punctuation;
- classifier and budget values loaded from seed data;
- a five-task questions-per-task benchmark.

The benchmark fixture is
`data/benchmarks/question-necessity-suite.lino`. Its initial ceiling is 60
questions per 100 tasks. `scripts/check-question-necessity-ratchet.rs` permits
that ceiling to stay equal or move down, never up, and the dedicated workflow
runs both the checker and benchmark.

## Replayable trace

For each candidate question, `links_notation` and `evidence_links` expose these
events in order:

```text
question_necessity:memory
question_necessity:workspace
question_necessity:sources
question_necessity:classification
question_necessity:authorized | question_necessity:refused
question_necessity:asked | question_necessity:research_required
```

Question IDs are content-addressed from normalized candidate text. Replaying the
same prompt and configuration therefore produces the same necessity trace.

## Self-hosting evidence

The implementation was decomposed into five smallest leaves in
`self-hosting-authorship/decomposition.lino`. Formal AI's real server and the
external Agent CLI authored the seed-classifier leaf (one of five, 20%). The
replay harness is `experiments/issue_920_self_authoring/run.sh`; retained Agent,
server, and output artifacts live in `self-hosting-authorship/`.
