# Recursive learning and self-coding evidence

PR review feedback exposed three missing back-edges: the recursive task model
did not execute failure recovery, uploaded #822 context did not enter the rule
learner, and approved ledger lessons were never queried by the live solver.
`decomposition.lino` records five smallest reviewed leaves and their tests.

Leaf `L5` was genuinely executed by the external Agent CLI against the local
release `formal-ai` server. Session `ses_075888666ffeU9fXsH9GrDSJ4k` authored
`agent-authored-learning-ledger.lino`; `cmp` verified it byte-for-byte against
the reviewed canonical ledger. `agent-stream.jsonl` is the raw real-client
session, `formal-ai.log` is the matching server trace, and `session.json` is the
deterministic in-repo replay. Run:

```bash
cargo build --release --bin formal-ai
experiments/issue-823-recursive-self-coding/run.sh
```

That is one genuinely self-coded leaf out of five: exactly the 20% acceptance
floor. The Rust controller and integration changes were manual tool extensions
and deliberately receive no self-authorship trailers.
