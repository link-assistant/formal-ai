# Plan 15 — repository world model and requirement delta

Status: implementation in progress. This document was written before the
implementation and is the recovery point for the bounded 2026-09-17 lane.

## Problem

The repository protocol can prove that it cloned, located, edited, ran a named
command, and produced a diff. Those observations are necessary, but they do not
yet answer the semantic question: **does the resulting repository match every
state the issue requires?** A green command can prove exact output while saying
nothing about comments or run instructions; a clean/ready pull request with no
configured checks can still contain the wrong source path; and an unchanged
placeholder tree can be mergeable while satisfying no implementation
requirement.

Completion therefore needs a first-class delta between two independently
recorded sides:

1. current state observed from the repository and hosted checks;
2. goal state formalized from the work item and applicable repository policy.

Narration, mergeability, and the mere presence of a diff are neither side of
that comparison.

## Latest canary evidence and attribution

The three public pull requests were read again on 2026-09-17. They are
benchmark fixtures, not production branches or special cases in the solver.

| canary | latest observed head/state | attribution boundary |
| --- | --- | --- |
| Kotlin / Claude Code | [`302f35a24da3327398302f365793ef4f8ed563c2`](https://github.com/konard/test-hello-world-019fb330-fa49-7c9d-a664-b7ea33bb698a/pull/2): `Main.java`, committed `Main.class`, no workflow and no checks | Formal AI supplied the whole solve request to model-backed `WebFetch`, which returned a nested solution instead of issue source. Hive Mind then committed the corrupt tree and called a no-check PR ready. The containment/readiness defect is [hive-mind#2263](https://github.com/link-assistant/hive-mind/issues/2263). |
| Scala / Agent CLI | [`724fb2be3c4e10808256368da910c0c3e9eb3d09`](https://github.com/konard/test-hello-world-019fb330-00e1-73b9-955e-f357a1600d5b/pull/2): `Main.scala` with explanatory/build/run comments and idiomatic standard-library-only source, exact-output verifier, workflow, two successful hosted `run` checks | This is the positive canary: the exact source and hosted evidence cover all fourteen formalized issue/policy obligations. It is not a current Agent CLI defect. |
| Rust / Codex | [`9d74fb354ad3ee68ea64167543be59b6604d0357`](https://github.com/konard/test-hello-world-019fb331-c107-78c7-8ff6-9f127a3c593c/pull/2): only `.gitkeep`, draft, no checks | Hive Mind's Codex configuration failed with `invalid transport` before a model turn. Formal AI and Codex did not execute. This is [hive-mind#2259](https://github.com/link-assistant/hive-mind/issues/2259), fixed after the image used by the canary; the canary remains unverified until rerun on a containing release. |

These facts supersede the older heads in issue-710 plan 06. No external
repository is modified by this plan.

## Representation

Add `repository_workspace::world_model`, reusing rather than shadowing the
existing contracts:

- every formalized repository requirement owns a canonical `needs::Need`;
- its expected state is an explicit `(subject, predicate, accepted values)`
  goal, not prose hidden in a completion message;
- every current-state fact names the `execution_evidence::Evidence` that
  supports it;
- each goal projects to an `obligation_ledger::ObligationNode` with a symbolic
  expectation;
- comparison produces `Satisfied` only with matching evidence, `Refuted` for a
  witnessed mismatch, and `Unattempted` for missing or unevidenced state;
- every mismatch/missing/unverified result remains a typed `RepositoryGap` and
  an open `Need`; and
- `completion_ready` is true only when the delta is empty and every obligation
  is evidence-backed satisfied. Merge status, PR readiness, a non-empty diff,
  or one green check has no completion authority.

The model consumes Links Notation supplied by the caller. Production code is
generic over case, language, path, predicate and expected value; the canary
names and Hello World details live only in benchmark data/tests.

## Machine-readable observations

Add `data/benchmarks/repository-world-model-canaries.lino` with, for each
canary:

- immutable issue/PR/head provenance and tool/orchestrator attribution;
- formalized issue and policy requirements;
- observed current-state facts;
- the exact evidence source for each observed fact; and
- component-level attribution that distinguishes Formal AI, Agent CLI/Codex,
  and Hive Mind.

The fixture records only facts supported by the latest observation. Scala's
source-quality facts come from inspection of the exact source at the recorded
head; the two green runs support execution/output facts and are not silently
promoted into evidence for unrelated requirements.

## Tests-first contract

Before implementation, add exact unit tests that require:

1. the Kotlin fixture to report the wrong extension, committed build artifact,
   missing workflow/checks and all other unobserved requirements; readiness
   must be false;
2. the Rust fixture to keep the placeholder/no-source/no-CI delta open and to
   preserve the pre-model Hive Mind attribution;
3. the Scala fixture to satisfy all fourteen requirements backed by its exact
   source, verifier/workflow and hosted-check observations and become the
   positive complete canary;
4. a synthetic, unrelated repository case to become complete only after every
   goal has a matching observation and evidence, proving the implementation is
   not a canary allowlist; and
5. Links Notation output to include current state, goal state, gap, obligation,
   need and evidence identities so an interrupted run can be reconstructed.

The Rust tests are authored but not run in this lane because the shared Cargo
target is owned by the parent. Static validation is `rustfmt +1.98.1 --check`,
fixture/gate checks runnable with Node, and `git diff --check`.

## Implementation order

- [x] Re-read current canary heads/checks and preserve attribution boundaries.
- [x] Persist this plan before source changes.
- [x] Add exact failing unit/static fixture tests.
- [x] Add the machine-readable canary observation corpus.
- [x] Implement the generic world-model/delta layer using existing Need,
  Evidence, ObligationLedger and RepositoryWorkspace concepts.
- [x] Register the module and focused tests.
- [x] Update plan 14 and the issue-1138 change evidence.
- [x] Run formatting and static checks only; leave Cargo execution to the
  serialized parent gate.

Static handoff evidence: the focused Node fixture/attribution suite passes 2/2,
Node syntax checking passes, Rust 1.98.1 formatting passes for the new module
and unit test, and the scoped diff check is clean. The Rust tests remain
authored but unexecuted in this lane; the parent owns the serialized Cargo run.
