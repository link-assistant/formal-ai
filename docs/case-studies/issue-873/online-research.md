# Online research snapshot

Research was performed on 2026-08-08. Primary or project-owned sources were
preferred. The notes below are paraphrases; no source code or prose was copied
into the implementation.

## Durable execution and replay

- [Temporal Workflow Execution](https://docs.temporal.io/workflow-execution)
  describes workflow state that persists through failure, resumes from the last
  event in event history, and reconstructs an evicted workflow cache by replay.
  This supports append-only cycle events, a state derived from replayable
  history, and the distinction between disposable cache payload and durable
  evidence identity.
- [Temporal workflow failure detection](https://docs.temporal.io/encyclopedia/detecting-workflow-failures)
  distinguishes a failed task from a failed workflow and describes timers as a
  way to take action after a duration. This supports treating an individual
  candidate/tool failure as recoverable and the one-hour boundary as a
  continuation decision rather than destroying the overall cycle.

Temporal is a useful reference but not a selected dependency. Formal AI already
has hash-linked event records, process isolation, replay, and continuation; a
Temporal service would add operational state without replacing its verification
or user-permission rules.

## Versioned and transactional state

- [Git internals: Git objects](https://git-scm.com/book/en/v2/Git-Internals-Git-Objects.html)
  describes content-addressed objects and trees/commits that refer to prior
  immutable state. This informed content ids, parent-linked memory versions, and
  recovering by moving an active reference rather than editing history.
- [SQLite atomic commit](https://www.sqlite.org/atomiccommit.html) explains the
  observable all-or-nothing boundary of a transaction. The cycle uses the same
  architectural rule at a smaller scale: verification is collected on a
  candidate and the active stable pointer moves only when the complete gate
  passes.
- [SQLite transactions](https://www.sqlite.org/lang_transaction.html) notes that
  changes can be committed or rolled back as a unit. This supports keeping
  candidate construction separate from promotion.

Git and SQLite are references, not new dependencies. The repository's existing
stable-id and append-only data structures are enough for the in-process cycle.

## Research-and-learning agents

- [ReAct](https://arxiv.org/abs/2210.03629) interleaves reasoning traces with
  external actions and observations. Formal AI's unknown trace followed by
  search, fetch, and evidence-backed synthesis has the same useful separation.
- [Reflexion](https://arxiv.org/abs/2303.11366) studies agents that use feedback
  signals and reflective text in episodic memory rather than updating model
  weights. This supports retaining explicit outcome history for recovery
  selection, but the implementation uses transparent integer evidence rather
  than an opaque learned score.
- [Voyager](https://arxiv.org/abs/2305.16291) describes an expanding executable
  skill library, iterative prompting with feedback, and environment feedback for
  verification. This supports versioning learned procedures as well as facts.

These projects do not by themselves guarantee an immutable baseline, atomic
promotion, stable rollback, explicit ambiguity handling, or a configurable
continuation boundary. The local lifecycle reducer adds those formal controls
around Formal AI's existing research and agentic tools.

## Provenance and license boundary

Only architectural facts and public interfaces were consulted. All links above
remain citations to their publishers. No third-party code, figures, fixtures,
benchmarks, or substantial text were incorporated, and `Cargo.toml`,
`package.json`, and the lockfiles receive no new dependency from this research.
