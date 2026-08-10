# Issue 873: unknowns become recoverable research frontiers

Issue [#873](https://github.com/link-assistant/formal-ai/issues/873) asks Formal AI
to treat missing knowledge as the start of research rather than a terminal
answer. It also asks for one general learning cycle: evidence may be recollected,
memory is versioned, candidates cannot replace a tested stable version until an
immutable baseline passes, every error produces a recovery path, and long work
returns its current plan for an explicit continuation decision.

## Reproduction and root cause

The minimum reproduction is an unfamiliar instruction without question
punctuation:

```text
Calibrate the snorflax against silent teal weather
```

Before this change, `UniversalSolver` exhausted local memory and returned
`intent: unknown`. The agentic planner did the same unless the unresolved input
ended in question punctuation. Two special cases had accidentally become the
only open-world gates:

- `solver_unknown_reasoning` researched only an unresolved *bare term*;
- `agentic_coding::web_research` researched an otherwise-unresolved task only
  when its final character was a question mark.

The browser worker already ran `unknown_intent_research` before
`fallback:unknown`, so this was native/browser drift rather than an absent web
research implementation. Existing research could already search, rank up to
three independent sources, fetch exact pages, retry bounded gaps, and synthesize
an answer. The missing connection was the general unresolved-intent handoff.

## Implemented cycle

Online unresolved inputs now emit `web_search` with query kind
`unknown_reasoning_fallback`; agentic clients issue `websearch`, capture a source
with `webfetch`, and answer from the captured evidence. Explicit offline mode
remains a hard boundary and retains the diagnostic unknown path.

`research_learning::ResearchLearningCycle` adds one append-only reducer for the
longer lifecycle:

```text
unknown frontier
  -> external or local source receipts
  -> candidate fact / procedure / meta-algorithm
  -> immutable-majority verification gates
  -> promote candidate OR retain/recover tested stable
  -> ranked, user-selected, or permission-gated recovery
```

The ordered phases live in
`data/meta/research-learning-recovery.lino`. The same version record represents a
fact, an executable procedure, or the meta-algorithm itself, so improvements use
the same proposal and promotion rules as ordinary learning.

External evidence is split into a durable receipt and a disposable cached
payload. The receipt retains its locator and content identity when the payload
is evicted. Recollection restores the payload only when the digest matches; a
changed observation must become a new receipt, preserving history instead of
rewriting it.

Knowledge versions form a parent-linked history. A proposal starts as a
candidate. Promotion requires every gate to pass, every configured baseline gate
to be present and immutable, and immutable gates to be a strict majority. A
compile or test failure marks only the candidate rejected and leaves the active
stable pointer untouched. Any earlier stable version remains recoverable.

Every error identifier enters the same recovery reducer. With one option,
user-led mode continues; with several, it asks the user. Full-trust mode ranks
recorded successes, failures, advantages, and disadvantages with a transparent
integer score and selects deterministically. Per-command mode returns a
permission request. With no supplied option, the safe default is to restore the
stable version and resume research.

The default research/orchestration budget is now 3,600 seconds and remains
configurable. Reaching it is represented as `AwaitingContinuation` with the
caller's current plan; permission extends the budget and resumes research. It is
a resumable boundary, not a failure state.

## Existing components reused

| Capability | Existing component | Role here |
| --- | --- | --- |
| Unknown trace and minimal clarification | issue #298 / PR #305 | Retained for offline and genuinely unextractable inputs. |
| Grounded search/fetch/answer procedure | issue #840 / PR #850 | Executes the newly generalized unknown-to-research handoff. |
| Exact source capture and replay | issue #843 / PR #853 | Supplies recomputable observations with provenance. |
| Multi-source evidence fusion | issue #709 / PR #884 | Keeps the answer grounded when multiple sources are available. |
| Human-gated self-healing proposals | issue #558 / PR #637 | Establishes that unverified changes are proposals, not live rules. |
| Trusted gate replay and promotion | issue #656 / PR #690 | Supplies the repository's established verification-before-adoption model. |
| Capability-delta learning ledger | issue #701 / PR #817 | Demonstrates that learning must change tested behavior. |
| Resumable external-agent orchestration | issue #703 / PR #876 | Supplies process isolation, permissions, event chains, and continuation. |

No parallel subsystem or dependency was added. The new reducer composes these
established boundaries and exposes their shared invariant directly.

## External research and design choices

The detailed source notes are in [online-research.md](online-research.md). The
key conclusions were:

- Temporal's persisted event history and replay support the append-only event
  chain and resumable state, while its workflow cache shows why disposable
  payloads must not be the source of truth.
- Git's content-addressed object graph supports parent-linked immutable memory
  versions and stable identities.
- SQLite's atomic-commit model supports moving the active pointer only after all
  verification gates pass.
- ReAct supports an explicit reason/action/observation research loop; Reflexion
  supports outcome feedback in memory; Voyager supports a growing executable
  skill library with verification. None alone provides the issue's immutable
  baseline, stable-pointer rollback, permission modes, and bounded continuation,
  so importing one as a replacement would not close the requirements.

These sources were used as architectural references only. No third-party code,
paper text, fixture, or dataset was copied, and no new license-bearing runtime
dependency was introduced.

## Verification

`tests/unit/issue_873.rs` proves the exact native regression, the complete
search/fetch/final-answer state machine, explicit offline behavior, source
eviction and recollection, candidate rejection and promotion, recovery of a
prior stable version, immutable-baseline enforcement, all three autonomy modes,
outcome ranking, the one-hour continuation boundary, hash-linked replay, recipe
versioning, orchestration-default parity, and existing browser ordering.

The repository's duplicate source-test crate contains the same native solver
change. Existing tests that intentionally exercise the final unknown diagnostic
now opt into offline mode, making the boundary explicit rather than weakening
their assertions.

Run the focused proof with:

```sh
cargo test --test unit issue_873 -- --nocapture
cargo run --example issue_873_research_learning
```

## Same-task self-application

The reviewed decomposition has five smallest leaves: native unknown routing,
the reusable lifecycle reducer, exact regression tests, the case-study and
requirements evidence, and the lifecycle invariant. Formal AI serves the real
external Agent CLI while it authors the invariant leaf. The generated artifact
is copied to `data/meta/research-learning-recovery-invariant.lino` and compared
byte-for-byte. This is one of five leaves (20%); raw server and CLI traces are
retained under `self-hosting-authorship/`, and the repeatable harness is
`experiments/issue_873_self_authoring/run.sh`. The retained Agent session is
`ses_01c561b3bffeG2Bl3eBvtDHYzq`; the successful first attempt completed four
Formal AI chat rounds and byte verification.
