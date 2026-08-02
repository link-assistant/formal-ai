# Issue 531 Architecture Inventory

This inventory records the delivered sequence/pattern substrate and the
algorithm-learning paths added after the latest PR review.

## Link-Native Foundation

- `src/sequences/store.rs` owns the structurally deduplicated link store and
  exact expansion contract.
- `src/sequences/symbols.rs` assigns unique points to scalar, text, and typed
  marker symbols.
- `src/sequences/converter.rs` supplies balanced conversion, adjacent-pair
  indexing, and frequency caching. Issue #531's algorithm benchmark exposed
  and now covers the exactly-two-address converter boundary.
- `src/sequences/compression.rs` performs deterministic Re-Pair-style
  replacement and retains a lossless compression trace.
- `src/sequences/patterns_1d.rs`, `grid_2d.rs`, and `inference.rs` implement the
  original sequence/text/grid transformation requirements.

The implementation remains self-contained rather than adding another graph
dependency. The production `src/link_store.rs` boundary still handles imported
and native doublet records, while the inference store gives algorithms a small,
deterministic address space suitable for testing.

## Algorithm Discovery Core

`src/algorithm_discovery.rs` adds a format-neutral execution model:

- `ExecutionTrace` is one ordered case; `TraceStep` is an operation plus named
  string arguments.
- Operation names are interned through `SymbolTable`; unique boundary markers
  prove global compression without manufacturing cross-trace episodes.
- Every contiguous shape from two through 32 operations is considered.
  Occurrences are sorted, made non-overlapping per trace, and then reduced to
  maximal candidates so a three-step routine does not also flood memory with
  all its two-step windows. A failed longer routine never suppresses a shorter
  routine that independently passed held-out validation.
- The first two occurrences are support. Their argument-value vectors infer
  constants or parameters. Equal varying vectors share one parameter name,
  which preserves data flow when the same value enters several steps.
- Later exact occurrences are held out. A trace with the same entry operation
  but a changed or missing continuation is also held out, making structural
  counterexamples observable rather than invisible to exact-shape mining.
- Candidate and evidence IDs are length-framed, content-addressed identities.
  Artifact parsing recomputes both and checks proposal mode, human gate, status,
  support, and held-out consistency.

The learner is conservative. It requires at least two support observations and
one held-out observation. It does not claim a candidate from a single example.
Because contiguous-window mining is exhaustive inside that model class, a run
over more than 4,096 input steps fails closed with
`observation_limit_exceeded "true"` and no candidates; it never learns from a
partial prefix. These limits keep default-on idle learning bounded.

## Ordered-Source Adapters

`src/algorithm_discovery/adapters.rs` prevents each storage surface from growing
its own learner:

| Source | Case boundary | Operation | Arguments |
| --- | --- | --- | --- |
| Portable `MemoryEvent` | `conversation_id` | `tool`, falling back to `kind` | JSON or `name=value` `inputs` |
| Solver `EventLog` | caller-supplied log id | event `kind` | event `payload` |
| `CompiledProcedure` guide | procedure id | canonical seeded step `kind` | canonical objects and target language |
| Agent `DriverOutcome` | caller-supplied session id | executed tool name | requested JSON arguments |

Tool outputs are not reinterpreted as instructions by the Agent adapter. Only
the requested operation and structured inputs enter learning.

## Auto-Learning And Memory

`src/dreaming.rs` runs discovery over retained memory observations during the
existing idle-learning pass. Only held-out-validated candidates enter the plan.
`apply_dreaming_plan` appends each as an `algorithm_learning_candidate` memory
event with `intent "generalize"`; `src/dreaming_runtime.rs` persists the changed
store, and `src/cli_memory.rs` reports the count.

This is a proposal channel, not a promotion channel. It does not update seed
data, source, dispatch, or host permissions.

## Safety And Execution Boundary

Three distinct stages are encoded as distinct types and commands:

1. `AlgorithmCandidate`: observed, parameterized, held-out-tested, inert.
2. Side-effect-free conformance: artifact integrity and parameter
   materialization recorded with `side_effects "false"`.
3. `ApprovedAlgorithm`: constructible only from a validated candidate, a named
   green `AlgorithmGate`, and named granted `AlgorithmApproval`; execution then
   delegates each known operation to an explicit `AlgorithmHost`.

Discovery cannot synthesize a host implementation, grant approval, or create a
passing gate. Failed candidates remain serializable for diagnosis.

## Public And Agent Interfaces

- `formal-ai learn algorithms --from observations.lino [--output artifact]`
  loads the portable memory format and emits the same discovery document used
  by dreaming.
- `formal-ai algorithm conformance --artifact … --trigger … --binding …`
  parses and integrity-checks the artifact and produces a no-effects replay.
- `src/agentic_coding/algorithm_learning.rs` recognizes actual embedded
  `demo_memory` data, not an English keyword. The planner writes observations,
  calls the public learning command, reads the artifact back, compares it to the
  independently derived candidate, and calls public conformance.
- `src/agentic_coding/driver.rs` mirrors those allowlisted public commands in
  the deterministic in-repo sandbox.
- `experiments/agent_cli_e2e/run_issue_531.sh` boots `formal-ai serve` and asks
  the real `@link-assistant/agent` client to perform that same workflow in CI.

## Upstream Data.Doublets.Sequences Mapping

The preserved upstream sources remain the basis for the sequence layer:

- `BalancedVariantConverter` maps to balanced conversion.
- `SequenceIndex` and `LinkFrequenciesCache` map to explicit adjacent-pair data.
- `CompressingConverter` maps to lossless repeated-pair replacement; the C# vs
  C++ maximum-frequency discrepancy is still documented rather than copied.
- `StringToUnicodeSequenceConverter` informed the typed symbol/converter
  layering.

## Deliberate Boundary

The current learner discovers contiguous sequential routines of at most 32
steps and repeated loop bodies, from at most 4,096 observed steps per run. It
does not infer branch predicates, concurrency/partial orders, recursion,
temporal deadlines, semantic postconditions, or brand-new host operations.
Process-mining and program-synthesis sources in
`raw-data/online-research.md` show how those later model classes differ. Adding
one safely requires typed evidence and negative/held-out cases for that
structure; silently treating frequency as permission would contradict the
project's human-gated self-improvement architecture.
