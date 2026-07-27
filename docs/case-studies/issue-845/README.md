# Issue 845: context-relative, disproof-first fact checking

Issue: <https://github.com/link-assistant/formal-ai/issues/845>

Pull request: <https://github.com/link-assistant/formal-ai/pull/856>

## Problem

Formal AI had three deterministic pieces, but no operation joined them:

1. `proof_engine` could prove or disprove a narrow set of claims and return a
   counterexample.
2. `relative_meta_logic` could calculate a symbolic posterior from source-tiered
   support and contradiction.
3. `world_model::Context` could recalculate dependent statements to a JTMS
   fixpoint.

Consequently, callers could not recursively audit every statement, could not
name the formal system relative to which a probability applied, and had no
explicit, traceable boundary around general-memory access. Even after the
library implementation was recovered, the operation was unreachable through
the solver and browser worker.

Issue #702 was originally a blocker. It was completed by PR #818 and is
included in this branch through the merge of `main`; this implementation uses
its canonical `GeneralMemoryPermission` values and append-only memory events.
Issue #843 remains open, so this feature does not claim to fetch evidence.
Local proof results and caller-supplied evidence are the only admitted evidence.

## Root causes

The original gap was orchestration rather than probability arithmetic:

- no call path joined `ProofOutcome`, `RelativeEvidence`, and a whole
  `world_model::Context`;
- no first-class named formal system scoped a probability;
- no batch operation audited all statements and exposed each dependency edge;
- no explicit permission gate protected the current/general memory boundary.

Recovery and test-first runtime work exposed three additional defects:

- no solver handler made the completed library operation user-reachable;
- the browser worker treated the English request as a live web search and
  treated the Russian, Hindi, and Chinese requests as unknown;
- a dependency on an absent statement incorrectly changed the report label from
  `prior_only` to `evidence_weighted`.

## Prior art used

- PR #675 introduced the symbolic `WorldModel` and JTMS recalculation.
- PR #694 supplied deterministic statement-auditing and provenance reporting
  patterns.
- PR #598 established relative-meta-logic evidence attachment.
- PR #619 demonstrated source-tiered probability handling.
- PR #818 supplied the canonical permission and append-only event boundary
  required by issue #702.

The solution reuses those kernels and
`SolverConfig::max_decomposition_depth`. It does not add a neural confidence
model or a second probability formula.

## Solution

- `FormalSystem` names the universe, interpretation, and sorted axiom set. Its
  content-addressed id makes the probability scope replayable.
- `FactChecker` tries direct refutation, refutation of the negation, and then
  dependency decomposition up to the configured depth.
- Discharged proofs and counterexamples become labelled first-party symbolic
  evidence. Existing caller-supplied source evidence retains
  relative-meta-logic tier weights; reposts remain zero-mass.
- A context audit batches generated evidence and performs one final JTMS
  recalculation. Its report includes every statement, prior-only versus
  evidence-weighted basis, tiers, counterexamples, recursive attempts, rejected
  placeholder sources, and every dependency edge consulted.
- `WorldModel` requires `GeneralMemoryPermission::Allowed` for a general-memory
  audit or current-to-general commit. The canonical append-only event stream
  records boundary decisions and access.
- The contextual solver handler replays only earlier user statements into a
  `DialogueWorldModel`; the fact-check request is excluded, and the audit is
  always `AuditScope::CurrentDialogue`.
- A shared seed role recognizes complete requests in English, Russian, Hindi,
  and Chinese. Response prose is also seed data.
- The browser worker mirrors the current-dialogue operation before web-search
  fallbacks. It is offline and emits no source, fetch, or cache-hit claim.
- A dependency counts as evidence only when its target exists in the audited
  context.

## Test-first reproductions

The recovered compiler reproduction established that the original public
`FactChecker`, `FormalSystem`, audit types, trace fields, and permission
methods did not exist.

Three further regressions were captured before their fixes:

1. `a_dangling_dependency_does_not_turn_a_prior_into_evidence` reported
   `EvidenceWeighted` for an absent dependency.
2. `solver_fact_checks_every_current_dialogue_statement_in_every_language`
   returned non-fact-check intents because the runtime had no handler.
3. `issue-845.spec.js` showed the English request entering live search while
   the other supported languages returned unknown answers.

The durable command/error index is in
[`test-logs/regression-evidence.md`](test-logs/regression-evidence.md). The
killed-session recovery manifest, including the original red/green artifact
inventory, is in [`recovered-sessions.md`](recovered-sessions.md).

## Self-coding evidence

The recovered Agent CLI + local Formal AI loop was attempted three times:

- session `ses_067d9e0beffecsJsYArtPUFr9F` misclassified the file-writing task
  as an originality check;
- session `ses_067d8e127ffewICsdXBf7HWekH` produced an implementation approval
  plan, but after explicit approval emitted unrelated web-search calls inherited
  from a fisherman-story recipe; the Agent CLI then received HTTP 403.

No tool-authorship claim is made for the changelog or implementation. The raw
session containing those attempts remains available in the maintainer-provided
[14.5 MB recovery transcript](https://gist.githubusercontent.com/konard/475718ddace837f5a29ff4f579fbc09d/raw/e78372c37122e87a8ac03c25bd3a839606282a91/tmp-start-command-logs-isolation-docker-702c9ee1-6ad7-4791-b9b2-ac26742f1e03.log.txt).
The routing failure is recorded as evidence rather than hidden by an authorship
claim.

## Verification

Focused checks cover:

- all fact-checking unit and public-boundary integration tests;
- multilingual Rust routing and browser-worker behavior;
- a browser assertion that the current-dialogue audit makes no external
  request;
- total LiNo closure and generated role-registry parity;
- deterministic replay, file-size, hardcoded-language, formatting, Clippy,
  self-AST, and Rust documentation gates;
- the complete 437-case Playwright matrix, split into two sequential shards to
  stay below the repository's 15-minute global suite cap.

All 2,119 unit tests and all 241 integration tests pass. The browser matrix has
436 passing cases and one intentional skip. Two pre-existing multi-language
tests exceeded their 30-second per-test cap under parallel shard load; both
passed unchanged when immediately rerun alone (29.8 seconds for issue #336 and
22.7 seconds for issue #501). The issue #845 browser cases passed for en, ru,
hi, and zh in both the unsharded and sharded runs.

See [`test-logs/regression-evidence.md`](test-logs/regression-evidence.md) for
the exact commands and final outcomes.
