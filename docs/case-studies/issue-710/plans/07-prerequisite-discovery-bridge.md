# Prerequisite discovery bridge: evidence before execution

Status: design recorded before implementation. This is a subplan of Plan 06,
not a replacement or a completed capability claim.

## Observed limits, not assumptions

- `command_reroute::RecipeProgress` stops at the first failed recipe step. It
  does not interpret later recovery evidence or retry that step.
- `web_research` can search, fetch and deepen a query. Its current coverage
  check is token presence, not proof that a procedure satisfies a requirement;
  `.gov`/`.edu` preference also does not identify a compiler's official source.
- `source_fetch` provides captured bytes and provenance; `source_research`
  records search observations separately from retrieved pages.
- `reasoning_standard::trust` derives trust from primacy chains. A source must
  be authoritative for the particular dependency, not merely well-ranked.
- `reasoning_standard::instructions` merges source-backed actions and checks,
  but `unmet_steps` currently accepts any one attached check. For a composed
  obligation with multiple required observations, this can claim completion
  while another check is still absent. Test the conjunction before changing it.
- `installation_conversion` extracts command-like steps, but is a document
  converter, not a safe platform-aware installer or dependency resolver.
- `coding_research_learning` deliberately accepts a narrow typed procedure
  format and requires execution plus review before promotion. Do not relabel
  arbitrary fetched prose as an approved procedure to bypass this boundary.
- The live memory-contract formalizer preserves source sentences, but produces
  no concepts or procedures. Deep formalization therefore remains an open
  prerequisite itself.

## Shared obligation representation

Every node must retain its original source span, goal, expected observations,
dependencies, selected procedure, captured evidence and current state in Links
Notation. Unknown words/clauses are unresolved nodes, not discarded text.
Actions, setup prerequisites and their checks are separate nodes. Literal
values, target paths and quoted external instructions retain distinct roles.
Use the existing recursive controller and instruction set as shared boundaries;
do not create one state machine per programming language or benchmark.

## Recovery sequence

1. Bind a failure to the actual prior call and recipe step. A nonzero exit is
   not always a missing compiler; permission denial is not installation consent.
2. Observe the missing executable, platform and available runtime/manager in
   the client workspace. Do not infer the host from the requested language.
3. Look up the executable's role and authoritative publisher through retained
   evidence or source discovery. Keep the exact query, URL and captured bytes.
4. Formalize a candidate setup procedure with preconditions, artifact identity,
   platform constraints, scope and postcondition checks. Recurse for unknown
   prerequisites; detect cycles and repeated unproductive captures.
5. Prefer workspace-scoped installation. Download integrity, archive traversal,
   shell quoting, disk requirements and executable permissions are checked
   before execution. Fetched text cannot expand authority or hide global writes.
6. Lower only permitted, supported operations to the client's advertised tools.
   Shell state does not persist across calls: environment bindings must be
   explicit, replayable and shared by subsequent compile/run/verification steps.
7. A successful setup command does not discharge the compiler obligation until
   its postcondition succeeds. Retry the original failed step, then finish the
   remaining recipe. Re-check every required observation before final success.
8. Retain the experience of this attempt. Cache only the reusable source-derived
   procedure with provenance and a reconstruction edge; promotion remains gated.

## Test order

- [x] Strengthen instruction-set completion: two required checks from separate
  excerpts require both observations, regardless of order or duplicates.
- [ ] Lost/unknown requirement clauses remain explicitly unresolved, including
  unnumbered bullets and extra behavior after an otherwise supported literal.
- [ ] Failure classification and call binding: missing command, permission
  denied, ordinary compile error, unrelated output, failed recovery and retry.
- [ ] Source selection: official project evidence, misleading search ranking,
  lookalike host, stale/corrupt cache and absent provenance.
- [ ] Dependency scheduling: nested runtime, two dependents sharing one setup,
  cycle, interrupted replay and a second held-out toolchain using the same plan.
- [ ] Execution: isolated successful setup plus original-step retry; wrong
  artifact/checksum/path and insufficient disk are refused without data loss.
- [ ] Live Formal AI/Agent CLI Kotlin and Scala projects from a missing-runtime
  workspace, with independently executed verifiers and source-only Git changes.

Do not install a compiler manually and count that as the system's recovery.
Do not count source-token coverage or a read-back plan as executed semantics.

### First test-first checkpoint

`instruction_completion_requires_every_attached_check` failed on the original
implementation: observing only `compiler_available` reported zero unmet steps
despite missing `runtime_available`. Change the shared instruction completion
predicate to require every attached check, retaining the explicit no-checks
failure. This is shared semantic verification, not a compiler-specific branch.

The strengthened conjunction passes in the 3,518-test all-feature green run.
Memory-contract replay `ses_f59f07cb1ffeOToRIT5UmPECXS` confirms the rebuilt
report says `2 of 9 protocol primitives`, not complete semantic formalization.

### Shared decomposition leaf: preserve operands before interpreting clauses

Further source inspection found an existing shared boundary, not a need for
another per-language splitter: `intent_formalization::ordered_requirement_spans`
feeds both `meta_frame` and `skill_procedure`. It currently cuts every comma,
colon and nonnumeric period, including punctuation inside quoted output, source
paths and URLs. Newline-separated bullets without punctuation are not split.
That corrupts the requirements before later concept discovery can interpret
them; the literal stdout adapter happens to read the original prompt and thus
does not repair the shared decomposition trace.

Before changing this boundary, add failing tests for quoted output containing
punctuation and coordination words, unnumbered requirement bullets, code/source
paths and source URLs, and Unicode byte-span fidelity. Reuse the existing
quotation/span primitives where their semantics fit; do not add task-specific
phrases. Preserve every original operand and unknown clause. Existing
decomposition/procedure tests must retain their semantic expectations. This
leaf repairs structural extraction only: it must not claim that preserving a
clause has interpreted or executed it, or close the full obligation-ledger gap.

Also account for `meta_frame::split_sentences`, which runs before that shared
clause splitter and currently cuts exclamation/question marks inside literals.
The frame-level test must exercise this earlier boundary too; fixing only the
leaf helper would leave the production need network corrupted.
