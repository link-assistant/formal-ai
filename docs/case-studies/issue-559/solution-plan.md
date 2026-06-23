# Issue 559 Solution Plan

This plan preserves the issue's first-session constraint: document and plan first,
then implement behavior changes after maintainer approval or requested changes.
The June 23, 2026 PR feedback is folded into this revision: the core solver must
be fully recursive, must use links as the native representation, must learn from
Voyager without adding neural-network runtime dependencies, and must include an
upstream dependency audit before implementation starts.

## Planning Status

Current PR scope:

- Planning artifacts only.
- No runtime behavior changes.
- No migration of current caches, overrides, meanings, or `.lino` assets.
- No direct replacement of current specialized handlers.
- No new upstream issues opened, because the dependency audit found no blocker
  for the next behavior-preserving phases.

Recommended next approval target:

- Approve Phase 1A and Phase 1B only.
- Phase 1A adds an observable `ProblemFrame` trace while preserving current
  routing.
- Phase 1B adds a recursive `WorkUnit` trace while preserving current answers.
- Later phases can move selection into data once parity evidence exists.

## Target Architecture

The system should be a recursive link-native problem solver. The implementation
can still use Rust structs, functions, and standard-library calls, but the control
plane should be representable as links, serializable through existing `.lino`
assets, and eventually translatable through `meta-language`.

The target is not "add another handler." The target is to route every prompt
through a common shape:

1. Capture an impulse.
2. Convert it into a structured problem frame.
3. Split the frame into needs and recursive work units.
4. Use known methods and accumulated skills as executable links.
5. Execute the smallest work units directly.
6. Compose results upward until every need has a visible status.
7. Record enough trace data to learn new methods behind tests and review gates.

### Link-Native Contract

All planning and execution records should be expressible as links:

- A user message is a link to source text and context.
- A need is a link between a requested outcome, constraints, evidence, and status.
- A method is a link between preconditions, input shape, implementation hook,
  output shape, and validation policy.
- A work unit is a link between a parent need, child work units, selected method,
  observations, validation results, and composed output.
- A piece of code can be linked as data through `meta-language` snapshots and
  source spans, then linked back to executable hooks.

Use "link network" terminology in code and docs. Avoid introducing a separate
conceptual model where point-like and relation-like structures are different
primitive kinds. In the meta-theory sense, both are links, and a
self-referential link can stand for a point-like element when needed.

### `ProblemFrame`

Introduce a general `ProblemFrame` before specialized behavior runs. The exact
Rust type can evolve, but the frame should have these conceptual fields:

- `frame_id`: stable trace id for the prompt.
- `impulse`: original user message plus conversation context.
- `mode`: chat, agentic coding, task execution, diagnostic, or
  offline-constrained mode.
- `language`: detected language, source spans, and translation metadata.
- `needs`: extracted questions, commands, constraints, preferences, safety
  constraints, source requirements, freshness requirements, and follow-up
  references.
- `evidence_policy`: local memory, source cache, web search, crawl, tool output,
  test execution, user clarification, or offline-only policy.
- `root_work_unit`: top-level recursive unit linked to all child units.
- `candidate_methods`: method or skill candidates with evidence for why each
  applies.
- `selected_methods`: methods chosen for execution, preserving deterministic
  order and compatibility information.
- `validation_plan`: tests, source requirements, checks, expected evidence, and
  answer-quality constraints.
- `observations`: tool outputs, source documents, test results, search results,
  method outputs, and errors.
- `composition`: how subresults were merged into the final answer.
- `presentation`: final answer requirements, omitted details, and unmet need
  statuses.

Initial implementation rule: build and trace this frame, but keep current
handler dispatch unchanged until comparison tests are available.

### Need Records

Every prompt should decompose into need records, not just one intent label.
Minimum fields:

- `need_id`
- `source_span`
- `kind`: question, command, constraint, preference, citation request,
  freshness request, clarification need, validation need, or presentation need.
- `satisfaction_criteria`
- `evidence_requirements`
- `dependencies`
- `status`: pending, satisfied, deferred, blocked, rejected, or superseded.
- `status_reason`

This is the mechanism that prevents the solver from answering one part of a
multi-part prompt while silently dropping the rest.

### Recursive `WorkUnit`

`WorkUnit` is the recursive execution unit. It should be stored as link data and
can be represented by a Rust struct during execution.

Minimum fields:

- `work_unit_id`
- `parent_work_unit_id`
- `linked_need_ids`
- `input_links`
- `output_links`
- `constraints`
- `evidence_policy`
- `atomicity_decision`
- `candidate_methods`
- `selected_method`
- `child_work_units`
- `observations`
- `validation_results`
- `composition_result`
- `status`

Atomic units should be small enough to execute by one direct method call,
standard-library call, crate call, tool call, search query, parser invocation, or
simple deterministic transformation. Non-atomic units must split into children
or retrieve existing building blocks before executing.

### Method And Skill Records

Current handlers should become method records first, then skill records can
accumulate over time. A method or skill record should include:

- `method_id`
- `aliases`
- `meaning_links`
- `preconditions`
- `negative_preconditions`
- `input_schema`
- `output_schema`
- `required_capabilities`
- `evidence_policy`
- `validation_policy`
- `implementation_hook`
- `fallback_hooks`
- `compatibility_precedence_group`
- `old_dispatch_handler`
- `source_files`
- `related_tests`
- `benchmark_fixtures`
- `last_successful_examples`
- `failure_examples`
- `promotion_status`: seed, experimental, stable, deprecated, or retired.

The registry should start as data that mirrors existing behavior. Only after old
and new selection agree should the registry become the primary control plane.

## Recursive Core Algorithm

The general solver should reason downward by decomposition and upward by
composition. Both directions are needed: downward recursion finds small tasks;
upward construction searches for existing pieces that can be combined.

### Downward Pass

1. Capture the impulse and context.
2. Create a `ProblemFrame`.
3. Extract all needs, constraints, preferences, and validation requirements.
4. Decide the evidence policy for each need.
5. Create a root `WorkUnit` linked to the frame.
6. Test whether the unit is atomic.
7. If atomic, select the smallest applicable method or standard-library call.
8. If not atomic, split into child work units.
9. Recurse on each child until every leaf is atomic, blocked, deferred, or
   rejected.
10. Record every split as link data.

### Upward Pass

1. Search the method and skill registry for reusable pieces.
2. Search accumulated examples, experiments, source cache, and current code
   hooks for solved subproblems.
3. Propose compositions from available pieces.
4. Validate each composition against the parent unit's criteria.
5. If a composition satisfies the parent, mark the child set complete.
6. If no composition satisfies the parent, decompose further or request more
   evidence.

### Execution Loop

Pseudo-code for the target behavior:

```text
solve(frame):
  root = create_work_unit(frame)
  recurse(root)
  result = compose(root)
  validate_frame(frame, result)
  return present(frame, result)

recurse(unit):
  attach_candidate_methods(unit)
  if is_atomic(unit):
    execute_atomic(unit)
    validate(unit)
    return unit

  children = decompose(unit)
  if children are empty:
    children = construct_from_known_pieces(unit)

  for child in children:
    recurse(child)

  compose_children(unit)
  validate(unit)
  if unit is not satisfied and can_split_further(unit):
    refine_and_recurse(unit)
  return unit
```

Atomic execution should be boring and inspectable:

- A Rust standard-library method call.
- A crate API call such as parser, calculator, search cache, or source resolver.
- A browser or CLI tool call.
- A deterministic string, number, date, or source-cache transformation.
- A registry method whose hook is already tested.

The recursive algorithm must not require a neural network. It may call tools or
web search when the evidence policy requires them, but core planning, trace
construction, validation, and composition should stay deterministic and
testable.

## Voyager-Inspired Adaptation

Voyager is useful as a reference pattern, not as a dependency. The relevant ideas
map into this project as follows:

| Voyager pattern | Formal AI adaptation |
| --- | --- |
| Automatic curriculum | Coverage backlog generated from unsatisfied needs, unknown traces, and missing method records. |
| Skill library | Link-native method and skill registry backed by `.lino`, examples, experiments, and tests. |
| Iterative prompting with feedback | Deterministic execute, observe, validate, patch, and retry cycle recorded in work-unit traces. |
| Self-verification | Critic methods that check every need against evidence and validation policy. |
| Open-ended exploration | Proposal-only self-improvement loop gated by tests, benchmarks, and human review. |

Do not add Voyager, GPT, embeddings, or any neural runtime to the core algorithm.
Use the reference to shape the architecture:

- accumulate skills as data;
- prefer small reusable methods;
- let failures generate curriculum items;
- require execution feedback before promotion;
- keep skill promotion behind validation gates.

## Evidence And Search Pipeline

The plan must support fresh external data because some user prompts require
current facts, direct citations, or changing online state. The pipeline should
be general rather than tied to one handler.

### Query Expansion

For a need that requires external evidence, generate search candidates from:

- terms;
- phrases;
- full sentences;
- direct questions;
- related source names;
- repository, issue, or package identifiers;
- time constraints such as "latest", "today", or an explicit date.

Every generated query should link back to the need and source span that produced
it. This makes reranking and citation decisions reviewable.

### Search, Rerank, Crawl, Extract

The evidence pipeline should:

1. Choose allowed providers from the evidence policy.
2. Search with multiple query shapes when appropriate.
3. Deduplicate results by canonical URL and source identity.
4. Rerank by source authority, freshness, directness, and historical reliability.
5. Fetch or crawl selected pages.
6. Extract bounded evidence snippets, dates, entities, and claims.
7. Link each fact to source URL, retrieval time, and freshness status.
8. Identify contradictions or missing evidence.
9. Form hypotheses only when evidence is incomplete, and mark them as hypotheses.
10. Validate the final answer against citation and freshness requirements.

### Offline And Cache Behavior

When network access is unavailable or disallowed:

- use local source cache and stored facts;
- surface freshness limits in the frame;
- avoid pretending cached data is current;
- leave needs blocked or deferred when required evidence is unavailable.

Tests should use deterministic cached fixtures. Live search should be separated
from normal unit tests.

## Phased Implementation

### Phase 0A: First Planning Artifact

Status: complete in the earlier PR commit.

Deliverables:

- Case-study requirements.
- Architecture inventory.
- External research notes.
- Initial phased plan and risk assessment.

Exit criteria:

- Maintainers can review the proposed direction without runtime risk.

### Phase 0B: Feedback Integration And Upstream Audit

Status: this revision.

Deliverables:

- Expanded recursive solver plan.
- Voyager mapping.
- Link-native terminology and algorithm-as-data constraints.
- Upstream dependency audit.
- Updated requirements covering the PR feedback.

Tests/checks:

- Documentation diff review.
- Changelog fragment remains valid.
- No code behavior changes.

Exit criteria:

- PR documents whether upstream blockers exist.
- Next behavior-preserving phases are specific enough to approve or reject.

Pause criteria:

- A maintainer rejects the recursive-work-unit direction.
- A maintainer asks to open a specific upstream issue before implementation.

### Phase 1A: Add `ProblemFrame` Without Behavior Changes

Goal: make the general frame observable before changing routing.

Tasks:

- Add a Rust `ProblemFrame` or equivalent internal structure.
- Populate it from current prompt/formalization data.
- Link each frame to the original impulse, context, and mode.
- Extract initial needs without using them for routing.
- Record evidence-policy guesses without enforcing them yet.
- Emit the frame in solver events or trace diagnostics.
- Add a feature flag or debug setting if trace output would be noisy.

Tests:

- Unit tests for frame construction.
- Fixtures for single-question, multi-need, follow-up, and constraint-heavy
  prompts.
- Snapshot or structured assertions for needs and evidence policy.
- Existing answer tests remain unchanged.

Exit criteria:

- Every prompt path can emit a `ProblemFrame`.
- Current answers and handler selection are unchanged.
- The frame includes enough source-span data for later need tracking.

Pause criteria:

- Current formalization data cannot produce source spans without a larger parser
  change.
- Browser worker parity cannot represent the frame without new serialization
  work.

### Phase 1B: Add Recursive `WorkUnit` Trace Without Behavior Changes

Goal: prove the recursive shape while old dispatch still controls answers.

Tasks:

- Add `WorkUnit` records linked to `ProblemFrame` needs.
- Add deterministic atomicity decisions.
- For current direct handlers, create one root unit and one leaf unit wrapping
  the selected handler.
- For multi-need prompts, create sibling child units even if old dispatch still
  answers with one route.
- Record selected old handler as the leaf method hook.
- Record validation status as trace-only metadata.

Tests:

- Unit tests for parent-child work-unit links.
- Atomicity tests for arithmetic, translation, sorting, lookup, source-cache, and
  agentic prompts.
- Trace tests showing multi-need prompts create multiple linked child units.
- Existing answer tests remain unchanged.

Exit criteria:

- Recursive traces are emitted for representative prompt classes.
- Leaf units are small enough to map to a single current handler, crate call, or
  standard-library call.
- No user-visible behavior changes.

Pause criteria:

- Trace size becomes too large for existing serialization.
- Links Notation parsing or rendering becomes a blocker for large trace fixtures.

### Phase 2: Need Satisfaction Ledger

Goal: make dropped requirements visible before changing execution.

Tasks:

- Promote need statuses to a ledger on the frame.
- Add validation that every need ends as satisfied, deferred, blocked, rejected,
  or superseded.
- Add a final-answer coverage check for multi-part prompts.
- Add diagnostics when old dispatch answers only one need.

Tests:

- Multi-need prompt fixtures.
- Negative tests where constraints are intentionally unsatisfied.
- Regression tests for issue families that historically lost follow-up context.
- Answer text remains unchanged unless diagnostics are explicitly enabled.

Exit criteria:

- The solver can say which needs were covered by the current response path.
- No direct dispatch behavior is replaced yet.

### Phase 3: Inventory Current Handlers As Methods

Goal: make every current specialized handler visible as data.

Tasks:

- Create a method registry in `.lino` or a generated seed file.
- Add one method entry for each `SPECIALIZED_HANDLERS` member.
- Add entries for contextual overrides and seed meanings that behave like
  methods.
- Preserve current precedence as compatibility metadata.
- Add implementation hooks pointing to current Rust or JS code.
- Add related tests and benchmark fixture links.

Tests:

- Registry covers every specialized handler.
- Every registry implementation hook resolves.
- Current dispatch order can be reconstructed from compatibility metadata.
- Guard test fails when Rust handler table and registry diverge.

Exit criteria:

- The method registry is a complete mirror of current dispatch.
- No runtime path depends on registry selection yet.

### Phase 4: Registry Selection In Comparison Mode

Goal: run old dispatch and registry selection side by side.

Tasks:

- Add a selector that reads method preconditions and frame needs.
- Execute old dispatch as the source of truth.
- Execute registry selection in diagnostic mode.
- Record agreements and disagreements.
- Add comparison output to traces, not final answers.

Tests:

- Old and registry selection agree for benchmark and prompt-variation fixtures.
- Precedence corner cases remain covered.
- Unknown prompts still produce reasoning traces instead of silent failures.
- Browser worker and native Rust selection produce the same candidate ordering.

Exit criteria:

- Agreement rate is high enough for targeted migration.
- Every disagreement has a known reason or a tracked issue.

### Phase 5: Move Cue Recognition Out Of Rust

Goal: reduce hardcoded natural-language trigger logic.

Tasks:

- Move recognizer cues from code such as `append_prompt_relevants` into seed
  meanings or method preconditions.
- Replace Rust string lists with data lookup and structural predicates.
- Keep syntax-level checks in code only when they are truly parser concerns.
- Add migration notes for each moved cue family.

Tests:

- No-hardcoded-natural-language guard updated for migrated cues.
- Prompt variation tests for every moved cue family.
- Backward compatibility tests for current supported prompts.
- Multilingual fixtures for cue families that already have language coverage.

Exit criteria:

- Migrated cues are reviewable as data.
- Behavior matches old dispatch in comparison mode.

### Phase 6: Fresh Evidence And Search Generalization

Goal: make online research a reusable evidence policy, not a one-off feature.

Tasks:

- Implement query expansion for terms, phrases, sentences, and questions.
- Add deterministic source-cache fixtures for tests.
- Add source authority and freshness scoring.
- Add crawler/fetch boundaries and citation extraction.
- Add contradiction and hypothesis records.
- Connect evidence artifacts to needs and work units.

Tests:

- Cached search fixtures for current-date, source-sensitive, and citation
  prompts.
- Reranking tests with stale and authoritative sources.
- Contradiction tests.
- Offline-mode tests that clearly block fresh-data needs.
- Live-search smoke test outside normal deterministic CI.

Exit criteria:

- Any method can request fresh evidence through the frame.
- Final answers can be validated against citation and freshness requirements.

### Phase 7: Skill Accumulation

Goal: accumulate reusable methods from successful traces.

Tasks:

- Store successful work-unit leaves as candidate skills.
- Promote examples from `experiments/` to `examples/` when they demonstrate a
  real reusable use case.
- Record failure examples and blocked needs as curriculum items.
- Add promotion rules requiring tests and benchmark deltas.
- Add deprecation and retirement status for old skills.

Tests:

- A successful trace can propose a new skill record.
- A failed trace creates a curriculum item without changing behavior.
- Proposed skills cannot become stable without tests.
- Bad proposals are rejected by benchmark or validation checks.

Exit criteria:

- Skill accumulation is reviewable and reversible.
- No unreviewed self-modification occurs.

### Phase 8: Link-Native Algorithm-As-Data Integration

Goal: represent algorithms as data that can round-trip with source.

Tasks:

- Use `meta-language` source spans and snapshots for method definitions where
  practical.
- Add `.lino` representations for method preconditions, validation policies, and
  simple compositions.
- Add code-to-data and data-to-code round-trip fixtures for small methods.
- Keep complex Rust implementations as hooks until translation is proven.

Tests:

- Lossless parse/reconstruct fixtures for method metadata.
- Round-trip tests for simple validation and composition records.
- Structural query/replace tests for registry edits.
- Compatibility tests proving hooks still execute after metadata round-trip.

Exit criteria:

- Simple algorithm records can be reviewed as links and reconstructed.
- Existing hand-written code remains the execution source for complex methods.

Pause criteria:

- Required `meta-language` APIs are not available in the pinned version.
- NPM/browser packaging becomes required before Rust-side integration can
  proceed.

### Phase 9: Gated Self-Improvement

Goal: let the algorithm propose improvements without silently changing itself.

Tasks:

- Reuse `docs/design/self-improvement-loop.md`.
- Convert unknown traces into proposed method, skill, or rule patches.
- Require tests and benchmark deltas before accepting a proposal.
- Require human review as the final gate for persistent behavior changes.

Tests:

- Unknown trace can produce a proposed method entry.
- Proposal is not applied without verification.
- Rejected proposals remain inspectable for future analysis.
- Accepted proposals include tests and changelog updates.

Exit criteria:

- The system can reason about improving itself without mutating production
  behavior automatically.

### Phase 10: Retire Direct Specialized Dispatch

Goal: make the method registry the control plane after parity is proven.

Tasks:

- Switch selected method families from old dispatch to registry selection one at
  a time.
- Keep implementation hooks for existing Rust handlers.
- Remove direct handler-table precedence only after registry parity is proven.
- Update docs and architecture diagrams.

Tests:

- Full local CI.
- Browser worker parity.
- Benchmarks and prompt variations.
- Changelog and documentation checks.
- Old/new comparison evidence for every retired direct-dispatch family.

Exit criteria:

- The registry controls method selection.
- Specialized Rust handlers remain available as implementation hooks.
- No previously supported prompt family is removed without explicit approval.

## Concrete First Implementation PR After Approval

After this planning PR is approved, the first code PR should be deliberately
small:

1. Add `ProblemFrame` and `Need` structures.
2. Populate them from existing formalization inputs.
3. Add trace-only serialization.
4. Add unit tests for frame construction.
5. Add fixtures for multi-need prompts.
6. Confirm existing answer tests are unchanged.
7. Add a changelog fragment.

The second code PR should add `WorkUnit` traces:

1. Add `WorkUnit` structures.
2. Link root units to frames and child units to needs.
3. Wrap current handler dispatch as leaf work units.
4. Add atomicity classification tests.
5. Add trace fixtures for direct and decomposed prompts.
6. Confirm existing answer tests are unchanged.

Only after those PRs should method registry selection start.

## Verification Matrix

Before switching behavior:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features`
- `rust-script scripts/check-file-size.rs`
- `cargo test`
- existing benchmark or prompt-variation commands used by this repo
- browser worker parity tests where changed code crosses runtime boundaries

For each implementation phase:

- Add focused unit tests first.
- Add fixture prompts for the class being migrated.
- Add old/new comparison tests before routing changes.
- Add deterministic cache fixtures for fresh-data behavior.
- Update docs and changelog in the same PR.
- Keep behavior changes narrow enough to revert as one commit if necessary.

## Upstream Dependency Gates

The upstream audit found no blockers for Phase 1A or Phase 1B. Existing upstream
issues that may matter later:

- `link-foundation/links-notation#197`: streaming parser for large message
  handling. This may matter if trace export grows too large for current parsing.
- `link-foundation/meta-language#168`: shared-dialog source-description schema.
  This may matter when issue-559 traces need to interoperate with shared-dialog
  source descriptions.
- `link-foundation/meta-language#165`: npm package publication. This may matter
  only if browser-side registry tooling needs the package from npm before the
  repo has another browser integration path.
- `linksplatform/doublets-rs` has older build and FFI issues. They do not block
  current default builds unless Phase 8 or later depends on optional features
  that reproduce those failures.

Open a new upstream issue only when a phase hits a concrete missing feature that
cannot be worked around locally without distorting the architecture.

## Solution Options Considered

Option A: Planning only now.

- Recommended for this PR because the issue explicitly asks for planning before
  execution.

Option B: Data registry plus compatibility dispatch.

- Recommended implementation path.
- Lowers risk by preserving existing handlers and tests.
- Moves control metadata into `.lino` incrementally.

Option C: Full Links Notation method interpreter immediately.

- Good long-term target.
- Too risky as the first implementation step because current behavior depends on
  many specialized handlers and precedence cases.

Option D: Adopt an external agent orchestrator.

- Not recommended as the core implementation.
- External systems are useful design references, but the repo already has Rust,
  browser worker, Links Notation, offline cache, and no-hardcoded-natural-
  language constraints.

## Requirement Mapping

- R5, R6, R7, R8: addressed by `ProblemFrame`, need records, and evidence
  policy.
- R9: addressed by recursive `WorkUnit` records and Phase 1B.
- R10: addressed by the fresh evidence and search pipeline.
- R11: addressed by method registry, skill records, and the recursive algorithm.
- R12: addressed by gated self-improvement and skill promotion rules.
- R13, R14, R16: addressed by compatibility mode, parity tests, and staged
  retirement of direct dispatch.
- R15: addressed by keeping `.lino`, cache, meanings, overrides, and source
  cache as first-class architecture.
- R17: addressed by phase-sized PRs and commits.
- R18: addressed by the Voyager-inspired adaptation section without neural
  dependency.
- R19: addressed by recursive decomposition to atomic calls.
- R20: addressed by downward decomposition and upward composition.
- R21: addressed by method and skill records that can wrap Rust, crate, and
  standard-library calls.
- R22: addressed by the evidence and search pipeline.
- R23: addressed by link-native terminology and records.
- R24: addressed by Phase 8 algorithm-as-data integration.
- R25: addressed by the upstream dependency audit.
- R26: addressed by the no-blocker conclusion and future upstream issue gate.
- R27: addressed by detailed phase deliverables, tests, exit criteria, and pause
  criteria.

## Risks

1. Handler precedence regressions.
   - Mitigation: reconstruct current precedence from registry data and run
     old/new comparison tests before switching.

2. Hardcoded language logic moves but does not become more general.
   - Mitigation: store meanings and preconditions as data, then test
     multilingual and variation prompts.

3. Recursive work units become too heavy for simple chat.
   - Mitigation: direct prompts can produce one root unit and one atomic leaf
     with minimal trace detail.

4. Fresh-data policy creates flaky tests.
   - Mitigation: separate live web behavior from deterministic source-cache
     tests.

5. Rust and JS worker behavior drifts.
   - Mitigation: require mirror tests or generated fixtures for every migrated
     method family.

6. Algorithm-as-data work outruns available upstream APIs.
   - Mitigation: keep implementation hooks in Rust until metadata round-trips
     are proven.

7. Self-improvement becomes unsafe.
   - Mitigation: proposal-only by default, with tests, benchmarks, and human
     review gates.

## PR Review Notes To Surface

The PR description or review comment should state:

- This PR is still planning-only.
- The plan now integrates Voyager as a design reference without adding neural
  dependencies.
- The proposed core is recursive through `WorkUnit` records.
- The plan uses links as the native representation and avoids a separate
  point/relation primitive model.
- Upstream dependency audit found no blockers for the next behavior-preserving
  phases.
- No new upstream issue was created.
