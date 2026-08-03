# Issue 531 Case Study: Pattern Inference

Status: implemented and under final verification in PR #642. The first two
passes delivered the research record, link-native sequence substrate,
associative compression, 1D/2D inference, localized solver handler, and seed
ontology. The August 2026 review follow-up extends that substrate from
recognizing patterns to discovering reusable algorithms from ordered runtime
evidence. The implementation infers parameters and cross-step data flow, tests
later observations without training on them, retains failures, and keeps every
candidate inert until a named reviewer and a green test gate promote it.

## Source Material

- Issue: [#531](https://github.com/link-assistant/formal-ai/issues/531).
- Prepared PR: [#642](https://github.com/link-assistant/formal-ai/pull/642).
- Raw issue, PR, upstream repository, and source excerpts are saved in
  `docs/case-studies/issue-531/raw-data/`.
- The review follow-up that asks for discovery from logs, events, step
  sequences, guides, auto-learning, and Agent CLI execution is preserved in
  `raw-data/pr-642-latest-feedback.md`.
- The upstream sequence reference is
  [linksplatform/Data.Doublets.Sequences](https://github.com/linksplatform/Data.Doublets.Sequences),
  checked at commit `6a6a69fc3ce42b0bd3e421c17c810ec2f37cb12b`.
- Related theory repositories captured in raw data:
  [link-foundation/meta-theory](https://github.com/link-foundation/meta-theory)
  and
  [link-foundation/relative-meta-logic](https://github.com/link-foundation/relative-meta-logic).

## Problem Statement

Formal AI stores facts, solver events, conversations, tool calls, and compiled
guides as links, but those ordered records previously remained isolated. The
first implementation pass supplied the missing sequence and transformation
machinery. The follow-up problem was operational: turn repeated ordered records
into a parameterized, inspectable algorithm; distinguish supporting examples
from held-out tests; feed safe proposals into idle learning; and prove the same
workflow through the public CLI and a real Agent CLI transport.

## Findings

- `src/link_store.rs` is the current boundary for importing/exporting doublet
  records and selecting a native `doublets-rs` backend.
- `src/substitution.rs` already models data-driven link-pattern substitution,
  which is the closest local analogue to a future pattern matcher.
- `src/solver.rs` and `src/meta_core.rs` already describe simplification,
  recursive work units, method selection, and evidence recording. Pattern
  inference should plug into those methods rather than bypassing them.
- `src/solver_handlers/text_manipulation.rs` has ordinary line deduplication,
  but it is string-level only. It does not build link-native sequences,
  compression traces, or reusable structural patterns.
- The C# sequence package has three converter families worth porting or
  adapting: `BalancedVariantConverter`, `OptimalVariantConverter`, and
  `CompressingConverter`.
- The C# and C++ copies of `CompressingConverter` differ in the max-frequency
  selection condition. The C++ version selects larger frequencies as expected;
  the captured C# source appears to compare in the opposite direction in
  `UpdateMaxDoublet`. That discrepancy must be verified before porting.
- Prior art maps cleanly to the request: SEQUITUR infers hierarchical repeated
  phrases, Re-Pair repeatedly replaces frequent pairs, and ARC-AGI supplies
  small transformed 2D grids that exercise rotation, reflection, translation,
  and analogy-like operations.
- Process-mining research adds a critical distinction: discovering a model and
  checking conformance are separate activities, and quality is not frequency
  alone. Fitness, precision, generalization, and simplicity must be considered.
- DreamCoder, LAPS, and ReGAL show why reusable abstraction libraries matter to
  program search. Formal AI adopts their library-learning direction, but not a
  neural recognizer or unreviewed self-modification path.
- `src/algorithm_discovery.rs` now treats operation names as link addresses,
  mines maximal non-overlapping episodes, uses the first two occurrences only
  for schema inference, and checks later exact or same-entry traces as held-out
  evidence. Varying argument vectors become shared parameters; stable vectors
  remain constants. Exhaustive mining is bounded to 4,096 observed steps and
  32 steps per candidate; larger inputs fail closed without partial proposals.
- Sequence/grid explanations and the new algorithm-learning CLI summaries are
  loaded from `data/seed/multilingual-responses-pattern.lino`, preserving the
  repository's data-driven language boundary.
- The common trace adapters cover portable memory, append-only `EventLog`,
  compiled natural-language procedures, and completed Agent CLI transcripts.
- `formal-ai learn algorithms` writes a portable proposal document;
  `formal-ai algorithm conformance` integrity-checks and materializes it with
  `side_effects "false"`. Actual execution is reachable only through
  `ApprovedAlgorithm`, which requires held-out success, a non-empty green gate,
  and named approval.
- Default-on dreaming now stores validated algorithm candidates as
  `algorithm_learning_candidate` events. It does not promote or execute them.

## Requirements And Plans

The full decomposition is in `requirements.md`. The implementation direction is
in `solution-plan.md`, and the current-code/upstream inventory is in
`architecture-inventory.md`.

The delivered data flow is:

1. Normalize memory events, solver logs, guides, or Agent transcripts into
   `ExecutionTrace { id, steps }`.
2. Intern operation names as unique link symbols and losslessly compress the
   global boundary-delimited sequence as an associative proof.
3. Infer a candidate only from two non-overlapping support occurrences,
   parameterizing equal value-vectors across steps.
4. Replay the schema over excluded observations, retaining constant,
   parameter, operation, and missing-step failures.
5. Persist validated candidates as human-gated proposals during idle learning,
   or inspect them via the public CLI.
6. Parse and integrity-check the portable artifact before side-effect-free
   conformance; promote it only through explicit review and test gates.

The deterministic benchmark is
`data/benchmarks/issue-531-algorithm-traces.lino`; the library/CLI example is
`examples/issue_531_algorithm_discovery.rs`; and the real transport replay is
`experiments/agent_cli_e2e/run_issue_531.sh`. Its retained first-attempt result
is under `agent-cli-evidence/`: five HTTP chat rounds, the external Agent log,
the Formal AI request trace, and the independently readable proposal artifact.

## Generalization Boundary

This is a bounded sequential routine learner, not an unrestricted program
synthesizer. It learns contiguous ordered steps, constants, parameters, and
parameter reuse; repeated windows in one trace represent loop bodies. A run
considers at most 4,096 observations and candidates up to 32 steps, failing
closed rather than learning from a truncated input. It does not invent branch
predicates, concurrent partial orders, recursion, or new host operations. Those
structures require explicit counterexamples and semantics, not a frequency-only
guess. Unknown or contradictory traces stay visible as failed proposals, while
all side effects remain behind the existing host and promotion boundaries.

## Verification

- The raw research artifacts are preserved under `raw-data/`.
- `REQUIREMENTS.md` records the original issue #531 rows R396-R407 and the
  review-follow-up rows R531-17 through R531-25.
- `tests/unit/docs_requirements_issue_531.rs` keeps the root requirements,
  case-study files, and raw-data evidence connected.
- `tests/unit/issue_531_algorithm_discovery.rs` covers discovery, loop episodes,
  constants/parameters, structural and value counterexamples, artifact
  integrity, promotion safety, adapters, dreaming, public CLI, and the in-repo
  Agent CLI whole-task replay.
- `.github/workflows/release.yml` runs the real Agent CLI replay, whose retained
  artifact is held-out validated, human-gated, and proposal-only.
