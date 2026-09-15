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

All five new `requirement_span_integrity` tests failed on checkpoint
`f652eec2f`: quoted `North, and South!` became three fragments, a Python
documentation URL was split into five fragments, and an unknown bullet was
merged with its neighbors. The frame-level and Unicode-offset cases also
failed. Implement shared protected operand spans using the existing quotation
reader plus URI syntax; keep list markers as boundaries and filename periods
inside tokens. The sentence-level frame pass must use the same operand spans.

The first five structural regressions now pass. Before considering this leaf
verified, add boundary cases for numbered Markdown lists at the frame's earlier
sentence pass, fenced multiline literal content, and coordination matches that
land inside the lowercase expansion of one Unicode character. These are shared
syntax/offset obligations, not more memorized task solutions.

The expanded eight-test run passed six and failed two: the frame still treated
numbered-list periods as sentence ends, and matching `i` cut inside the
lowercase expansion of `İ`. Reuse list-marker spans in both structural passes
and require source-character boundaries for coordination cuts. Fenced literal
payloads already preserve their embedded list markers and punctuation.

All eight structural regressions now pass. Existing suites also passed:
17 problem-frame tests and 15 arbitrary-procedure compilation tests. Regenerated
three self-AST documents (541 total, no removals) and the requirements aggregate.
Full post-change regressions and a rebuilt-binary check remain pending. The
earlier repository/memory increment is safely committed as `f652eec2f`; it and
the separate evidence/plan commits are local only, awaiting the batch's one push.

The all-feature unit run now passes: **3,527 passed, zero failed, four ignored**
(192.53 s). Clippy identified two redundant `pub(crate)` declarations inside a
private module. Use `pub` inside that module while keeping the parent re-exports
crate-private; this does not expand the external API. Regenerate the self-AST
after that visibility correction and rerun the relevant gates. Free space was
25 GiB; no cleanup, pruning or user-data deletion was performed.

The broader `clippy --all-targets --all-features -- -D warnings` check also
found three pre-existing warnings in `generate_recurrence_source_cache`: two
unformatted usage commands and a hand-built hex digest. Correct the documentation
and reuse the shared `sha256_hex` helper; check that the generated cache remains
byte-identical. No lint allowance or gate weakening is appropriate.

Further tracing confirms two distinct remaining completion gaps:
`task_obligations` discards clauses without a recognized artifact, and the
`NeedLedger` projection currently marks a selected route satisfied without
runtime evidence. Neither is repaired by the operand-span leaf. Preserve these
as open semantic work, not a claim that all requirements now execute.

The next bounded prerequisite leaf uses the existing `RecipeProgress` rather
than a second recovery state machine: a failed step stays failed until a later,
properly bound successful retry of that same action is observed. A setup command,
another step, an orphaned result or duplicate old call cannot clear it. Add red
tests for command and write retries, an unchanged failure after unrelated setup,
out-of-order verification, and replay after interruption before changing code.
The transcript retains failed observations. This leaf enables recovery replay;
it does not by itself discover or authorize installation of a missing compiler.

### Live source-edit probe and authoring-evidence failure

The rebuilt branch ran through Agent CLI on port 8923, session
`ses_f59c6ecb8ffem38YNHcAzKa3Cr`, against an isolated seed containing the real
cache generator. The ordinary refactoring request named the shared digest
helper, unused import and documentation warnings; it did not supply replacement
file bytes. Formal AI read the file and answered with its contents in two model
rounds. The resulting file is byte-identical to the seed: **no edit was authored**.
Private evidence stays at `/private/tmp/formal-ai-888-source-edit.S5e5yi/evidence`.

The authoring helper incorrectly returned success because it checked only that
the seeded file existed, including with `--no-commit`. Add offline executable
harness regressions for no edit, rewriting identical bytes, a real modification
alongside an unchanged supporting artifact, and a newly created artifact.
Before copying any produced file to the destination, require at least one
produced artifact to differ from its seed (or be new). Preserve existing
destination bytes on failure. This is an authorship floor, not semantic proof;
the source-task completion defect itself remains open. Correct the misleading
`--no-commit` message as well: that mode does not stage files.

All four new retry regressions failed on the old reader (the existing three
passed): the reader stopped at the first failed observation, including after a
later successful retry or a more recent compilation error. Keep scanning only
for the same pending step; clear its failure only on bound successful evidence.
The root agent, not Formal AI, made the cache-generator lint corrections after
the failed live authoring probe. No self-authorship credit is claimed for them.

The expanded all-target lint check found another nine pre-existing warnings in
the local three-run replay example. Apply the suggested borrow/Option idioms
without changing its fixtures or behavior. The new authoring test initially
failed to compile because this crate does not depend on `tempfile`; use a private
`mktemp` sandbox with scoped cleanup instead of adding a dependency. This compile
failure is not the desired behavioral red evidence; rerun the executable cases.

Behavioral red evidence now exists: the offline helper incorrectly accepts an
unchanged seed and says files are staged in no-commit mode (two failures, eight
passes including all seven recipe-evidence cases). Also cover an output that
differs from its seed but is already identical to the destination: fresh logs
must not make repeated repository bytes a new self-authored contribution. Require
at least one produced artifact to differ from both seed and destination before
copying anything. Additional unchanged supporting artifacts remain allowed.

All 19 focused cases now pass: eight operand-span tests, seven recipe-evidence
tests and four executable authoring-helper tests. The helper checks actual byte
differences before publishing, including no-commit runs, without treating fresh
evidence logs as source authorship. Its shell syntax check and browser seed sync
check pass. The all-target lint rerun and full post-change suite remain pending.

Next live probe: a bounded identifier rename in the same real generator, from
`digest` to `source_digest`, using the grounded workspace rewrite path. Seed an
isolated workspace from the current file, let Formal AI read/edit/verify it,
then independently check the diff and regenerate the cache. This exercises an
existing general word-scoped operation; it must not be represented as success
at the earlier open-ended refactoring request.

The narrow rename succeeded in session `ses_f59bba59dffejPDpr0cYH2PYEf`
(Agent CLI 0.26.0, local Formal AI 0.350.0, four model rounds). Independent
diff inspection shows exactly the two word-scoped identifier replacements.
Input SHA-256: `92e1e42fca911fc67612e90779eddbb6da83aea192c0958610c458bd97804f75`;
output: `e61f63636226bb3c375c682ac71405600e9ebe974e0408456f5c89812205156c`.
Private artifacts/evidence: `/private/tmp/formal-ai-888-rename.DyDJ1S`.
Do not mix attribution: first checkpoint the root-agent changes, then land this
two-line edit separately with reviewed evidence and the real session trailers.

The one-off all-target Clippy probe passed after the example warnings were
fixed (49.80 s). Routine CI deliberately uses lib/bin/test lint plus example
type-checking for disk safety; keep that policy and use its targeted commands
for subsequent validation. Free space is 26 GiB; no cache/container pruning.

The broad unit run exposed a local permission limit, reproduced independently:
`the_deadline_exits_124_and_kills_the_whole_stalled_tree` cannot spawn `ps`
inside this sandbox (`Operation not permitted`). The deadline itself returned
124 in 1.29 s. Rerun with process-inspection permission; do not weaken the child
termination assertion or alter production deadline behavior without a defect.

The permission-enabled rerun passes **3,535 unit tests, zero failures, four
existing ignores** (198.02 s). The restricted run's sole failure was the denied
`ps` invocation; no assertion was waived. Secret scanning passed for all 22
changed/new files, workflow syntax and diff whitespace checks passed, and every
file remains within its configured size limit. Hardcoded prose stays at 1,286
allowlisted literals; core-boundary checks pass without increasing a ceiling.
The three external PR heads/checks are unchanged on the latest refresh. The
main checkout still contains only its pre-existing untracked continuation text.
