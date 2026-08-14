# Issue 923 Case Study

Issue [#923](https://github.com/link-assistant/formal-ai/issues/923) asked for
at least two general symbolic reasoning capabilities beyond propositional SAT
and linear arithmetic, with honest external scores and no neural inference.
The result adds bounded equality saturation and bounded positive Datalog, then
measures both against pinned upstream Rust examples.

## 1. Collected Data

`raw-data/github/` preserves issue #923, prepared PR #1006, and every issue and
PR feedback channel before implementation. All four comment/review collections
were empty. `raw-data/online-research.md` records the exact upstream revisions,
licenses, and repository precedents used by the design. The issue and comments
contained no screenshots or image attachments.

## 2. Requirements

The complete mapping is in `requirements.md` and root `REQUIREMENTS.md` as
R923-1 through R923-5. The two requested capabilities, real benchmark scores,
dependency policy, sound limit behavior, and reproducible evidence are all
covered in this pull request.

## 3. Reproduction And Root Cause

Before this change, `src/proof_engine/decision.rs` dispatched only Boolean SAT
and affine real arithmetic. The smallest regression initially failed to compile
because the external harness had no structured proof-status grader; after that
grader existed, a symbolic rewrite still had no decision path. A negative
rewrite experiment also exposed a deeper dispatch bug: when a symbolic equality
was not proved, it fell through to the linear parser, whose intentionally
permissive token scan could reduce an out-of-grammar prefix expression to
`0 = 0`. The dispatcher now treats a recognized symbolic equality as owned by
that procedure, so search failure is inconclusive and never a false proof or
disproof.

Rule inference was absent entirely. There was no runtime representation for a
generic rule program and no least-fixed-point evaluator to derive recursive
consequences such as transitive closure.

## 4. Implemented Design

`decision/equality.rs` parses generic symbolic S-expressions with `egg`, seeds
both sides into one e-graph, and applies a bounded general rewrite system. It
returns a structured proof only when both roots are in the same equivalence
class. The MIT-licensed dependency is optional, has its own default features
disabled, and is exposed by `equality-saturation`. Twelve iterations and 20,000
e-nodes are hard ceilings; exhausted search returns no decision.

`decision/rules.rs` parses explicit `facts`, `rules`, and one ground `query` for
range-restricted, function-free positive Datalog. It computes the finite least
fixed point with ceilings of 512 clauses, arity 16, 10,000 facts, 100,000 join
substitutions, and 256 rounds. Query absence is a valid disproof only after
fixed-point completion; malformed input or a resource ceiling is inconclusive.

The #698 external harness gains a pinned-Rust-source adapter and structured
`proof_outcome` grading. The adapter mechanically extracts the first 20
unconditional rewrite declarations from egg's `tests/math.rs`, alpha-renames
the source pattern variables at the prompt boundary, and extracts all five
asserted consequences from Ascent's transitive-closure example. It does not
vendor or silently replace either upstream source.

## 5. Honest External Scores

The project CLI produced these results on 2026-08-14 with solver version
0.341.0:

| Command | Passed | Total |
| --- | ---: | ---: |
| `formal-ai benchmark run --suite egg_math --slice 20` | 20 | 20 |
| `formal-ai benchmark run --suite ascent_transitive_closure --slice 5` | 5 | 5 |

The exact provenance, grading rule, score, and ratchet floor are committed in
`data/benchmarks/external-results.lino`. An initial egg run scored 18/20 because
bare upstream metavariable names were misclassified as incomplete natural
questions. Mechanical prompt-boundary alpha-renaming fixed the adapter rather
than changing or excluding the failed cases.

## 6. Verification

`tests/unit/issue_923.rs` covers a non-numeric equality, the false-fallthrough
regression, transitive rule inference, optional dependency and manifest
registration, exact committed scores, and a whole-task ignored test that
downloads and executes both pinned sources. `tests/unit/docs_requirements_issue_923.rs`
checks the requirements, provenance, release metadata, and self-hosting trail.
The scheduled benchmark workflow now keeps Ascent's complete five-case source
slice fixed while applying the normal bounded slice to egg.

## 7. Self-Hosting Evidence

The reviewed leaves are: (1) equality engine, (2) Datalog engine, (3) pinned
adapters and score ledger, (4) regressions, CI, and traceability, and (5) the
symbolic-kernel invariant document. The real Formal AI server and installed
Agent CLI authored leaf 5 without manual byte edits. The raw stream, server
log, task, exact output, session identifier, and worktree status live under
`agent-cli-evidence/`; the captured session is
`ses_001f733ceffe5UboLW4JATfkoZ`. This is one of five reviewed leaves, meeting
the repository's 20% floor without attributing manually implemented code to the
agent.
