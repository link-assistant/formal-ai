# Plan 05 — Obligation execution evidence (bottleneck B5 of #1138)

Status: design recorded before implementation. Nothing here is a claim that any
obligation executes today; every claim below carries a `file:line`.

## Issues addressed

- **#1138 B5** — "obligations are marked satisfied by route selection, not by runtime
  evidence." This plan delivers the whole bottleneck: one execution record, an obligation
  ledger that can only reach `Satisfied` while carrying one, and decomposition — not
  completion prose — as the response to an unsatisfied node.
- **#559 / R340–R344** — `docs/requirements/issue-0559-general-meta-algorithm.md:18-26`
  scopes the gap honestly: "a selected method is **planned**, not **satisfied** … Runtime
  per-need verification feedback is still open." This plan delivers that feedback stage
  and the recipe step that describes it, without weakening R343 parity.
- **#710 / R710-R9** — "Retain every requirement as a verifiable obligation; unknown
  clauses cannot be silently discarded"
  (`docs/requirements/issue-0710-repository-and-retention-continuation.md:18`). Today a
  clause no composer can read is dropped at `src/agentic_coding/task_obligations.rs:55`.
  This plan keeps it as an `Underivable` node and recurses.
- **#710 / R710-R4** — "A successful tool result must be bound to the requested path,
  bytes and ordered command before it counts as execution evidence" (same shard, `:13`).
  Delivered here as the binding rule in `ObligationLedger::observe`, generalized from the
  recipe path to every obligation on every surface.
- **#1099** — a two-file prompt finished after one file. Fixed structurally, but its
  satisfaction test is still "a write tool call returned success"
  (`src/agentic_coding/task_obligations.rs:132`). This plan raises it to command, exit
  code and observed-output hash.
- **#674 / R550–R558** — `src/agentic_coding/procedure.rs` already does write →
  read-back → conformance → execution verification (R557,
  `docs/requirements/issue-0674-arbitrary-natural-language-programs.md:20`). This plan
  lifts that one handler's discipline into the shared obligation contract so it stops
  being procedure-specific; R553's "fail honestly with a named gap" becomes the general
  `Unsatisfiable` outcome.
- **#847** — `data/meta/task-decomposition-strategies.lino:19` records
  `failure_evidence single_clause_issue_was_certified_atomic_without_an_operation_contract`.
  The `ObligationExpectation` value added here *is* that operation contract, made a value instead of
  a review note.
- **#1073 / R1073-1..7** — "Evidence before claims. Every statement about the world is
  backed by an executed command and its observed output, never by impression." B5 is that
  standard applied to the ledger rather than to prose.
- **#1066** — `src/agentic_coding/evidence_record.rs:57-67` invented a second, private
  `Obligation` with its own write-only completion test (`:271`, `:274`). This plan gives
  both call sites one shared type.

## Current state

### Two disjoint ledgers, neither of which observes a run

**Ledger A — the planning ledger, `src/meta_frame.rs` (765 lines).**

`NeedStatus` (`src/meta_frame.rs:48-61`) declares six states including `Satisfied` (`:54`).
`NeedLedger::resolve` (`:654-682`) is the only production writer of a status, and it
writes exactly two:

```rust
// src/meta_frame.rs:661-667
let leaf = best_leaf_for(&leaves, &need.source_span);
let status = leaf.map_or(NeedStatus::Blocked, |leaf| {
    if leaf.route.is_some() {
        NeedStatus::Planned
    } else {
        NeedStatus::Blocked
    }
});
```

The entire satisfaction test is `leaf.route.is_some()` — a route slug produced by
`formalize_span` (`src/meta_frame.rs:36`), not even a registry lookup. `Satisfied`,
`Deferred` and `Rejected` are **never constructed anywhere in `src/`**. They are only
read: `src/meta_frame.rs:714` (a header count), `src/solution_evidence.rs:140`
(`fully_resolved`), `src/skill_ledger.rs:254` (`demonstrated`). The only places they are
*written* are three test files that assign the field by hand
(`tests/unit/specification/solution_evidence.rs:73`, `:79`;
`tests/unit/specification/skill_ledger.rs:65`).

Live consequences:

- `SolutionEvidence::fully_resolved()` (`src/solution_evidence.rs:135`) can never return
  true in production.
- `SkillLedger::from_evidence` (`src/skill_ledger.rs:250`) can never produce a
  `CandidateSkill`; every trail becomes a `CurriculumItem` (`:269-278`). `stable_count()`
  and `promotable_count()` are documented as "always 0 at trace time" (`:293-294`,
  `:306-307`) — but the cause is not policy, it is that the input state is unreachable.
- `SolutionEvidence::accounted_for()` (`:128-130`) is `all(connected)`, and `connected` is
  `row.unit_id.is_some() && row.status != NeedStatus::Pending`
  (`src/solution_evidence.rs:106`) — **true for a `Blocked` row**. A frame in which every
  need is blocked reports `accounted_for = true`.
- `NeedLedger::every_need_accounted_for()` (`src/meta_frame.rs:695-701`) is
  `all(status != Pending)` — the same weakness.
- `tests/unit/specification/recipe_interpreter.rs:229-231` *pins* `satisfied "0"` in the
  emitted Links Notation, so the zero-satisfaction state is currently a regression floor.
  Its sibling `native_and_data_driven_planning_do_not_fabricate_execution_evidence`
  (`:210`) is a correct test of a correct thing — this plan must not break it.

**Ledger B — the agentic obligation list, `src/agentic_coding/task_obligations.rs`
(134 lines).**

```rust
// src/agentic_coding/task_obligations.rs:30-36
pub struct Obligation {
    pub request: String,
    pub target: String,
}
```

Satisfaction (`:127-133`):

```rust
pub fn outstanding(request: &str, messages: &[ChatMessage]) -> Option<Obligation> {
    let obligations = obligations(request)?;
    let progress = Progress::scan(messages);
    obligations
        .into_iter()
        .find(|obligation| !progress.successful_write_for(&obligation.target))
}
```

`Progress::successful_write_for` (`src/agentic_coding/progress.rs:263-272`) returns a
`bool` derived from `ToolAttempt::succeeded` (`progress.rs:17`), itself the collapse of
`tool_result::step_outcome` inside `Progress::scan` (`progress.rs:59`). No command, no
exit code, no bytes and no hash survive.

Three silent-discard points in the same file:

- `:46-48` — `if clauses.len() < 2 { return None; }`.
- `:54-56` — `let Some(plan) = compose_general_change_plan(&clause) else { continue; };`
  A clause the composer cannot read an artifact out of is dropped with no record at all.
- `:68` — `(obligations.len() > 1).then_some(obligations)` — when only one clause yields
  an artifact, the whole list is thrown away.

"Recognized artifact" means one of exactly three shapes,
`src/agentic_coding/general_planner.rs:26-34`: `LiteralFile`, `CommandOutput`,
`RepositoryWorkItem`. The third already terminates in
`PlanTerminalState::PlannedNotExecuted` with an **empty** `verification_command`
(`general_planner.rs:284-285`) — planning wearing an obligation's clothes.

The sole call site is `src/agentic_coding/planner.rs:373-379`, immediately followed by the
single-target fallback at `:380-385`.

### The exit code the system already parses and then throws away

`src/agentic_coding/tool_result.rs` is a complete exit-code layer:

```rust
// src/agentic_coding/tool_result.rs:42-53
pub fn step_outcome(raw: &str) -> StepOutcome {
    let result = normalize(raw);
    if result.error.is_some() { return StepOutcome::Failed; }
    match result.exit_code {
        Some(0) => StepOutcome::Succeeded,
        Some(_) => StepOutcome::Failed,
        None if looks_like_error(&result.payload) => StepOutcome::Failed,
        None => StepOutcome::Unreported,
    }
}
```

with `pub(super) fn reported_exit_code(raw: &str) -> Option<i64>` (`:78`) and
`struct ShellEnvelope { … exit_code: i64 … }` (`:509-515`), scanning `"exit_code"` and
`"exitCode"` (`:418`, `:443`, `:461`). The number reaches `StepOutcome`, `StepOutcome`
reaches a `bool`, and the number is gone. Output hashing does not exist on this path at
all: `grep -rn "output_hash\|observed_output" src/` returns zero;
`grep -rn "exit_code\|ExitStatus" src/` hits only seven files, of which only
`src/orchestration/runner.rs` retains the value.

### The record shape that already exists, in the wrong subsystem

```rust
// src/orchestration/runner.rs:196-204
pub struct VerificationResult {
    pub program: String,
    pub args: Vec<String>,
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub timed_out: bool,
    pub passed: bool,
}
```

plus `pub fn session_sha256(session: &AgentSession) -> Result<String, serde_json::Error>`
(`:498`) delegating to `crate::source_fetch::sha256_hex` (`src/source_fetch.rs:383`), and
`let passed = !output.timed_out && output.exit_code == Some(0);` (`:862`). Nothing
connects this to `NeedLedger` or to `task_obligations` in either direction. It also
carries `wall_time_ms`, which is machine-dependent and therefore cannot enter a
deterministic identity.

### Trace of one concrete task, today

Prompt: *"Two files. First, create file notes/attribution.md containing Gemfile.lock.
Second, create file changelog/fragment.md containing bump patch."* — the literal `TWO_FILES`
constant at `tests/unit/issue_1099_multiple_obligations.rs:27`.

1. `src/meta_core.rs:87` `record_problem_frame` → three needs, each born
   `NeedStatus::Pending` (`src/meta_frame.rs:158`, `:171`).
2. `src/meta_core.rs:88` `record_work_units` → `WorkUnit::build`
   (`src/meta_frame.rs:304`) splits on sentence terminators through `decompose_once`
   (`:456`), n-ary, bounded by `max_decomposition_depth` (default `4`,
   `src/solver.rs:167`).
3. `src/meta_core.rs:89` `record_need_ledger` → each leaf whose span formalizes to a
   route becomes `Planned`. **The ledger is complete before a byte has been written.**
4. `:90` `record_method_registry`; `:104` `record_solution_evidence` reports
   `accounted_for = true`; `:110` `record_selection` names the method per leaf; `:112`
   `record_skill_ledger` produces zero skills and three curriculum items.
5. On the agentic side, `src/agentic_coding/planner.rs:373-379` calls
   `task_obligations::outstanding`, gets obligation 1, plans a write. The harness returns
   a result. `Progress::scan` records `succeeded = true`. Obligation 1 disappears.
6. Obligation 2 likewise. `outstanding` returns `None`, the planner falls through to
   `AgenticPlan::Final`, and the session answers completion prose.
7. **What was never checked:** that `notes/attribution.md` exists, that its bytes are
   `Gemfile.lock`, that any command ran, or what it exited with. The harness at
   `tests/unit/issue_1099_multiple_obligations.rs:56-58` answers every tool call with the
   literal string `"ok"` — which `step_outcome` classifies as `Unreported`, and which is
   nevertheless enough to satisfy both obligations today.

### Trace of one *unrecognized* clause, today

Prompt: *"Two files. First, create file notes/attribution.md containing Gemfile.lock.
Then tell me what changed."*

`split_at_enumeration_cues` (`task_obligations.rs:77`) yields three clauses (cues from
`seed::ROLE_ENUMERATION_CUE`, `src/seed/roles/decomposition.rs:75`, surfaces at
`data/seed/meanings-conversation.lino:617-640`). `compose_general_change_plan` returns
`None` for *"Then tell me what changed"* → `continue` at `:55`. One obligation remains →
`:68` returns `None` → the caller uses the single-target path. The third clause is gone
from every artifact: no need row, no curriculum item, no event. That is exactly the
failure R710-R9 names, and plan 07 of #710 already recorded it
(`docs/case-studies/issue-710/plans/07-prerequisite-discovery-bridge.md:160-164`:
"`task_obligations` discards clauses without a recognized artifact, and the `NeedLedger`
projection currently marks a selected route satisfied without runtime evidence").

## Root causes

1. **Satisfaction is a projection of planning, because the ledger runs before dispatch.**
   `src/meta_core.rs:79-117` runs `record_need_ledger` at `:89`; every method dispatch
   happens later, in `src/meta_method_dispatch.rs:39`. No stage exists after dispatch that
   can revise a row. *Mechanism:* a one-pass recorder cannot express a two-phase fact.
2. **`NeedStatus` is a plain `Copy` enum, so `Satisfied` costs nothing to assert.**
   `src/meta_frame.rs:47-61`. The invariant lives in doc comments
   (`src/meta_frame.rs:640-642`: "Runtime validation must provide per-need evidence before
   a result can become `Satisfied`") instead of in a constructor. The three tests that
   assign the field by hand prove it is freely writable.
3. **`connected` / `accounted_for` conflate "has a row" with "was addressed."**
   `src/solution_evidence.rs:106` admits `Blocked`. *Mechanism:* the predicate was written
   to prove the artifacts are *joined*, and is now read as proving the task is *done*.
4. **Obligation satisfaction is a boolean about a tool call, not an observation of the
   world.** `task_obligations.rs:132` → `progress.rs:263` → `progress.rs:17`.
   *Mechanism:* `Progress` is a transcript reader whose job is "what has been tried", and
   it was reused for "what is true."
5. **The exit code is parsed and then discarded.** `tool_result.rs:42-53` collapses
   `Option<i64>` to a three-valued enum; `ToolAttempt` keeps only `bool`. *Mechanism:* the
   only consumer at the time needed a retry decision, not a record.
6. **An unreadable clause is discarded rather than decomposed.**
   `task_obligations.rs:54-56`. The comment says this is conservative, and it is —
   conservative about *inventing* work, at the price of *losing* work. *Mechanism:* the
   module has one escape hatch (`continue`) where it needs two: record as unrecognized,
   then recurse.
7. **Two independent `Obligation` types exist.** `task_obligations.rs:31` (public) and
   `evidence_record.rs:57` (private), each with its own write-only completion test.
   *Mechanism:* the second was written when the first did not fit, instead of generalizing
   the first.
8. **The recursive core's own recipe states the gap as permanent prose.**
   `data/meta/recursive-core-recipe.lino:35`: "Runtime checks must supply per-need evidence
   before satisfaction." A grounded recipe that *describes* a missing stage keeps the
   stage missing, because `tests/unit/specification/recursive_core_recipe.rs` pins the
   description.

## Solution options

### Option A — Evidence-carrying status inside `NeedStatus`

*Description.* Change `NeedStatus::Satisfied` into `Satisfied(EvidenceId)` so the
variant cannot be constructed without a record; revise rows in place after dispatch.

*Architecture sketch.* `NeedStatus` loses `Copy`; `LedgerRow.status` becomes owned;
`NeedLedger::resolve` still produces `Planned`/`Blocked`; a new
`NeedLedger::apply_evidence(&mut self, records: &[Evidence])` upgrades rows.

*Pros.* Strongest possible enforcement — a satisfied row without evidence is a compile
error. One ledger, one vocabulary, one place to read.

*Cons.* `NeedStatus` is `Copy` and matched across `src/solution_evidence.rs`,
`src/skill_ledger.rs` and six test files.
`tests/unit/docs_requirements_issue_559.rs:107-141` grep-pins the literal strings
`"pub struct NeedLedger"` and `"fn record_need_ledger"`. Mutating a recorded artifact in
place also contradicts the append-only discipline (`VISION.md:233` "Append-Only Event
Log"), and the recipe-interpreter parity obligation (R343) compares *event streams*, so an
in-place revision would have to be re-emitted anyway.

*Doctrine fit.* Good on honesty, poor on append-only.

*Effort.* Large — touches the core type every meta stage reads.

*Risk.* High: the widest mechanical refactor in the most-pinned module in the tree.

### Option B — A separate runtime obligation ledger the need ledger is projected from

*Description.* Leave `NeedLedger` exactly as it is (planning). Add
`src/obligation_ledger.rs` carrying a tree of `ObligationNode`s whose `Satisfied` outcome
*holds* an `Evidence`. After dispatch, a second recorder emits the obligation
ledger and a **new** need ledger whose rows reach `Satisfied` only where an obligation
discharged. Nothing is mutated; both ledgers stand side by side in the log.

*Architecture sketch.*
`meta_core` (planning) → dispatch → `obligation_ledger::record_obligation_ledger`
(runtime) → `need_ledger_with_execution(&planned, &obligations)`.

*Pros.* Append-only preserved. `NeedStatus` keeps `Copy` and every existing pin keeps
passing. Type-level enforcement lands where it matters — on the runtime variant. One
`ObligationNode` serves the agentic and symbolic paths, retiring both duplicate
`Obligation` types. A node that cannot be verified is a first-class `Unsatisfiable` with a
named reason, satisfying R710-R9 without inventing work.

*Cons.* Two artifacts instead of one; a reader must join them by `need_id`, and the join
must itself be tested or it becomes a third place to lie.

*Doctrine fit.* Strong. Associative-stack-only: the contract and the derivation rules are
`.lino`; the Rust is the recorder. Deterministic: content-addressed records with no wall
clock. Honest: `Unattempted` and `Unsatisfiable` are reportable states, not silence.

*Effort.* Medium — one new module pair, one recorder in two dispatch tables, one recipe
step, and the `task_obligations` rewrite.

*Risk.* Medium: the recorder must be registered in
`src/recipe_interpreter.rs:269-354` or the data-driven path errors with
"recipe binds unknown recorder" (`:352`).

### Option C — Reuse `src/orchestration/runner.rs` as the evidence store

*Description.* Route every obligation through an `AgentSession`, so `VerificationResult`
(`runner.rs:196-204`) is the record and `session_sha256` (`:498`) the hash.

*Pros.* No new record type; hashing and exit codes already work; real process execution.

*Cons.* `AgentSession` spawns processes. The agentic path is *client-owned* — the harness
runs the tool and we read the transcript. Forcing local execution either breaks every
harness client or requires a second code path anyway. `wall_time_ms` is machine-dependent
and cannot enter a deterministic identity.

*Doctrine fit.* Poor — non-determinism, and execution becomes a privilege of one surface.

*Effort.* Medium. *Risk.* High.

### Option D — Ledger-free: make the answer projection refuse to finalize

*Description.* No new types. Emit `AgenticPlan::Final` only when a verification command
was observed with exit 0 for every named path.

*Pros.* Very small diff; closes the visible #1099-shaped symptom.

*Cons.* Fixes one surface. `NeedStatus::Satisfied` stays unreachable; no record is
retained, so "why do you believe this is done?" is unanswerable; nothing feeds B7's
before/after criterion or B12's experiment outcomes.

*Doctrine fit.* Fails "everything discoverable must be forgettable and rediscoverable" —
there is nothing to rediscover.

*Effort.* Small. *Risk.* Low, but it does not close B5.

## Decision

**Option B is selected.**

Reasons:

1. It is the only option that keeps the append-only discipline (`VISION.md:233-241`)
   *and* makes `Satisfied` type-enforced, because the enforcement lives in a new variant
   that carries its evidence rather than in a mutated old one.
2. It is the only option that serves both surfaces with one type. Today there are two
   `Obligation` types and zero shared evidence vocabulary; Option B retires both.
3. It preserves every existing pin — `tests/unit/docs_requirements_issue_559.rs:107-141`,
   `tests/unit/specification/recipe_interpreter.rs:210` — because `NeedLedger`,
   `record_need_ledger` and their behaviour are untouched; the new stage is additive.
4. It gives plan 07 and plan 12 what they need. B7's "a learned item must change the next
   answer" requires a before/after that is an *observation*, not a route; B12's
   refutation-first search requires an experiment *outcome*. Both consume `Evidence`.

Rejected: **A**, because mutating a recorded artifact contradicts the append-only log and
carries the largest blast radius for the smallest marginal gain over B. **C**, because it
binds evidence to local process spawning the client-owned surface cannot do, and imports
wall-clock fields that cannot be hashed deterministically. **D**, because it closes one
symptom on one surface and leaves the ledger, the learning loops and the selection
heuristics with nothing to read.

## Architecture

### New module `src/execution_evidence.rs`

> **reconciled: was `Evidence`; now `Evidence`, plan 00 §4.3's contract
> name, because plans 03 and 06 already implement against `Evidence` and the
> contract is authoritative. The module keeps its name, the identity rule is
> unchanged — no wall clock, pid or machine identity in the fingerprint — and
> `recorded_at` joins the record as an event-log field that is excluded from the
> id, which is how plan 00 §4.3's `recorded_at` and this plan's determinism
> requirement are both satisfied (plan 00 §9 R2).**

Names verified free (`grep -rn "Evidence\b.*struct\|ObservationKind\|EvidenceSource" src tests --include='*.rs'` → 0 for the struct form).

```rust
//! The observation that turns a planned obligation into a satisfied one (#1138 B5).

/// What kind of observation the record holds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObservationKind {
    /// A command ran and reported a process exit status.
    CommandExit,
    /// A file was read back and its bytes hashed.
    FileBytes,
    /// A client-owned tool returned a result the harness gave no exit code for.
    ToolResult,
    /// The engine re-evaluated its own generated check; no external effect.
    SymbolicCheck,
}

impl ObservationKind {
    #[must_use] pub const fn slug(self) -> &'static str;
}

/// Where the observation came from, so a record can never claim more than its source.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvidenceSource {
    /// Read out of the conversation transcript (client-owned tools).
    Harness,
    /// Produced by `crate::orchestration::runner` in this process.
    LocalProcess,
    /// Produced by the engine itself.
    Engine,
}

/// One executed observation, content-addressed and deterministic.
///
/// Every field is reproducible from the same inputs: no wall clock, no pid, no
/// machine identity, so `record_id` is stable across runs and machines. This is
/// deliberately narrower than `orchestration::runner::VerificationResult`, which
/// carries `wall_time_ms` and therefore cannot be hashed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Evidence {
    /// `stable_id("evidence", &fingerprint)`. The fingerprint is command, argv,
    /// exit, output hash, output length, kind and source — never a wall clock.
    pub evidence_id: String,
    /// The need this observation discharges (plan 00 §4.1).
    pub for_need: String,
    /// The method that produced it (plan 00 §4.3).
    pub produced_by: String,
    /// The exact command line as issued, or the canonical rendering of the tool
    /// call when the harness owns execution.
    pub command: String,
    /// The command's arguments, split, so the record is inspectable unparsed.
    pub argv: Vec<String>,
    /// The reported process exit status. `None` is honest; it is never zero.
    pub exit_code: Option<i64>,
    /// SHA-256 of the observed bytes — stdout, or the file's content on read-back.
    pub observed_output_sha256: String,
    /// Length of the observed bytes, so an empty observation is distinguishable.
    pub observed_byte_length: usize,
    /// Content ids of every source consulted (plan 00 §4.3).
    pub source_ids: Vec<String>,
    pub kind: ObservationKind,
    pub source: EvidenceSource,
    /// Deterministic, hashable per-kind detail. Raw stdout and stderr are hashed
    /// into `observed_output_sha256`, never stored; a caller that must show them
    /// receives the non-persisted sibling `ObservedOutput { stdout, stderr }`
    /// from the same call. This is what plan 03's test-run fields become
    /// (plan 00 §9 R2).
    pub detail: EvidenceDetail,
    /// Event-log field, excluded from `evidence_id` so the id stays stable.
    pub recorded_at: Option<String>,
}

/// Per-kind detail that is deterministic and therefore hashable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EvidenceDetail {
    None,
    Tests { passed: Vec<String>, failed: Vec<String>, timed_out: bool },
}

impl Evidence {
    #[must_use]
    pub fn observed(
        command: impl Into<String>,
        argv: Vec<String>,
        exit_code: Option<i64>,
        observed: &[u8],
        kind: ObservationKind,
        source: EvidenceSource,
    ) -> Self;

    /// Read a record out of one client-owned tool result, reusing the exit-code
    /// parser that already exists.
    #[must_use]
    pub fn from_tool_result(command: &str, raw: &str, source: EvidenceSource) -> Self;

    /// Whether the record itself reports success: a reported zero exit, or a
    /// non-empty observation for kinds that carry no exit code. It says nothing
    /// about whether an expectation was met — that is the ledger's judgement.
    #[must_use]
    pub const fn reports_success(&self) -> bool;

    #[must_use] pub fn to_links_notation(&self) -> String;
}
```

Hashing reuses `crate::source_fetch::sha256_hex` (`src/source_fetch.rs:383`) — no second
digest implementation. `reported_exit_code` (`src/agentic_coding/tool_result.rs:78`) is
promoted from `pub(super)` to `pub(crate)` so `from_tool_result` uses the one parser that
already exists; nothing else about it changes. Note the name is duplicated at
`src/agentic_coding/command_reroute.rs:312` (an `i32` variant that delegates to it at
`:349`); the promoted one is the `tool_result` `i64` version.

`.lino` projection, appended to the event log as kind `evidence`:

```
evidence_9f2c1a77
  record_type "evidence"
  command "cat notes/attribution.md"
  argv "cat"
  argv "notes/attribution.md"
  exit_code "0"
  observed_output_sha256 "2b4a1c7e0f5d…"
  observed_byte_length "13"
  kind "command_exit"
  source "harness"
```

### New module `src/obligation_ledger.rs`

Names verified free: `ObligationNode`, `ObligationLedger`, `ObligationOutcome`,
`ObligationExpectation`, `ObligationStep` (0 hits each in `src/` and `tests/`). The plain
name `Expectation` is deliberately avoided: it is already taken by
`src/external_benchmarks/cases.rs:11` (`PythonUnitTest` / `PythonAsserts` / `Value` /
`SweBench`) — which is itself the prior art for this design, an expectation vocabulary that
already decides pass/fail from an executed observation, confined to one benchmark runner.

```rust
/// What must be observed before this node may be called satisfied.
///
/// This is *not* plan 08's `TaskExpectation`, which declares the shape a task's
/// **answer** must take. The two are bridged once, by plan 08's
/// `TaskExpectation::to_obligation_expectation(&self, check_id) ->
/// ObligationExpectation`, so a verifiable task's satisfaction flows through
/// this ledger and there is exactly one satisfaction rule in the tree
/// (plan 00 §9 R15).
///
/// This is the operation contract `data/meta/task-decomposition-invariant.lino`
/// already demands of an atomic task ("require an observable completion contract
/// and no pending children"), made a value instead of prose.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ObligationExpectation {
    /// The named path must exist and its bytes must hash to `sha256` when given.
    FileBytes { path: String, sha256: Option<String> },
    /// The named command must run and exit with `expected_exit`.
    CommandExit { command: String, expected_exit: i64 },
    /// The named command's observed output must hash to `sha256`.
    OutputHash { command: String, sha256: String },
    /// One of the generated checks of loop step 6 must pass.
    ///
    /// **reconciled: this plan's risk 2 left `check_id` undefined and asked for
    /// it to be settled jointly with plan 12 before either lands. Settled:
    /// `check_id = "<VerifiedAnswer::derivation_id>:<check slug>"`, owned by
    /// plan 08, whose five self-checks each emit one `Evidence` row; plan 12's
    /// `CandidateScore::checks` counts the satisfied rows of the same set
    /// (plan 00 §9 R15).**
    SymbolicCheck { check_id: String },
    /// No expectation could be derived from this clause. Never discarded: such a
    /// node is split, and when it cannot be split it is reported as a gap.
    Underivable { reason: String },
}

/// The outcome of one obligation node. `Satisfied` cannot exist without a record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ObligationOutcome {
    /// No observation has been made yet.
    Unattempted,
    /// An observation was made and did not meet the expectation.
    Refuted { record: Evidence, mismatch: String },
    /// An observation was made and met the expectation.
    Satisfied { record: Evidence },
    /// No observation is reachable and no split helped; the reason is named.
    Unsatisfiable { reason: String },
}

/// One node of the obligation tree: the runtime counterpart of a `WorkUnit`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObligationNode {
    /// `stable_id("obligation", &format!("{parent:?}:{depth}:{clause}"))`.
    pub node_id: String,
    pub parent: Option<String>,
    /// The clause exactly as the user wrote it.
    pub clause: String,
    /// UTF-8 byte span of `clause` in the original request, the same provenance
    /// discipline `requirement_span_integrity` already enforces (R710-R9).
    pub span: (usize, usize),
    /// The `Need::need_id` this node discharges, when it maps to one.
    pub need_id: Option<String>,
    pub depth: u8,
    pub expectation: ObligationExpectation,
    pub outcome: ObligationOutcome,
    pub children: Vec<Self>,
}

impl ObligationNode {
    /// Build the obligation tree for a request: derive an expectation per clause,
    /// and recurse through `task_decomposition::split_once_checkable` for clauses
    /// whose expectation is `Underivable`.
    #[must_use]
    pub fn build(request: &str, max_split_depth: u8) -> Self;

    /// Post-order: discharged when every child is discharged and this node's own
    /// outcome is `Satisfied` or `Unsatisfiable`.
    #[must_use] pub fn discharged(&self) -> bool;

    /// The first node that is neither discharged nor has an undischarged child.
    #[must_use] pub fn next_open(&self) -> Option<&Self>;

    pub fn collect_leaves<'a>(&'a self, out: &mut Vec<&'a Self>);
    #[must_use] pub fn to_links_notation(&self) -> String;
}

/// The runtime counterpart of `NeedLedger`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObligationLedger {
    pub frame_id: String,
    pub root: ObligationNode,
}

impl ObligationLedger {
    #[must_use]
    pub fn for_frame(frame: &ProblemFrame, request: &str, max_split_depth: u8) -> Self;

    /// Apply one observation to the node whose expectation it answers. Returns the
    /// discharged node id, or `None` when no node expected it — an unrelated
    /// result can never clear a step (R710-R4).
    pub fn observe(&mut self, record: Evidence) -> Option<String>;

    /// Every obligation is `Satisfied` or `Unsatisfiable` with a named reason.
    #[must_use] pub fn every_obligation_discharged(&self) -> bool;

    #[must_use] pub fn satisfied_count(&self) -> usize;
    #[must_use] pub fn unsatisfiable_count(&self) -> usize;
    #[must_use] pub fn unattempted_count(&self) -> usize;
    #[must_use] pub fn to_links_notation(&self) -> String;
}

/// The join: a *new* need ledger whose rows are upgraded from `Planned` to
/// `Satisfied` exactly where an obligation carrying an `Evidence`
/// discharged the same need. Never mutates its input; the planning ledger stays
/// in the log beside it.
#[must_use]
pub fn need_ledger_with_execution(
    planned: &NeedLedger,
    obligations: &ObligationLedger,
) -> NeedLedger;

/// Emit the obligation ledger and the executed need ledger as append-only events.
pub(crate) fn record_obligation_ledger(
    log: &mut EventLog,
    frame: &ProblemFrame,
    planned: &NeedLedger,
    obligations: &ObligationLedger,
) -> NeedLedger;
```

`need_ledger_with_execution` is the **only** function in the tree that produces
`NeedStatus::Satisfied`, and it can only do so from an `ObligationOutcome::Satisfied`,
which cannot be constructed without an `Evidence`. That is the type-level guarantee
B5 asks for, obtained without touching `NeedStatus`.

### The execution record every obligation node must carry before `Satisfied`

Stated as the contract the tests enforce:

| Field | Source | Why it is required |
| --- | --- | --- |
| `command` | the issued command line, or the canonical rendering of the tool call | answers "what did you run?" — R1073's evidence-before-claims |
| `argv` | split arguments | makes the binding to `ObligationExpectation::CommandExit` checkable without reparsing |
| `exit_code` | `tool_result::reported_exit_code`, or `runner::ProcessOutput::exit_code` | `None` is recorded honestly; a missing code is never rendered as zero |
| `observed_output_sha256` | `source_fetch::sha256_hex` over the raw observed bytes | answers "what came back?" without storing the whole output in the ledger |
| `observed_byte_length` | `observed.len()` | distinguishes an empty observation from a missing one |
| `kind`, `source` | the call site | a `Harness` record can never claim a `LocalProcess` guarantee |

### How an unsatisfied node triggers decomposition rather than completion prose

```rust
/// What the session must do next about its obligations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ObligationStep {
    /// Make the observation this node expects.
    Observe(ObligationNode),
    /// The node's expectation is `Underivable` and it can still be split.
    Decompose(ObligationNode),
    /// Nothing is left to split and nothing can be observed; report the gap.
    ReportGap { node_id: String, clause: String, span: (usize, usize), reason: String },
}

/// Replaces `task_obligations::outstanding`.
#[must_use]
pub fn next_step(request: &str, messages: &[ChatMessage]) -> Option<ObligationStep>;
```

`Decompose` calls the splitter that already exists —
`crate::task_decomposition::split_once_checkable` (`src/task_decomposition.rs:447`), which
accepts a split only when it yields at least two pieces that each satisfy `is_checkable`
(`:470`) — bounded by `crate::recursive_execution::DEFAULT_SPLIT_DEPTH_BOUND`
(`src/recursive_execution.rs:216`, already `pub const … : u8 = 4`). No new bound is
invented, and no second splitter is written.

`src/agentic_coding/planner.rs:373-379` changes from

```rust
.and_then(|_| task_obligations::outstanding(task, messages))
```

to a match on `ObligationStep`. `ReportGap` produces an `AgenticPlan::Final` whose text
names the clause, its byte span and the reason — never a completion sentence. A
*successful* `Final` becomes reachable only when `every_obligation_discharged()` holds and
`satisfied_count() >= 1`.

`src/agentic_coding/task_obligations.rs` keeps its module path and its public
`obligations()` (so `tests/unit/issue_1099_multiple_obligations.rs:9` still compiles), but
`Obligation` becomes a projection of `ObligationNode`, and the `continue` at `:55` becomes:

```rust
let expectation = match compose_general_change_plan(&clause) {
    Some(plan) => ObligationExpectation::FileBytes { path: plan.target, sha256: None },
    // Never discard: an unreadable clause becomes a node the tree must still
    // discharge, which is what R710-R9 requires.
    None => ObligationExpectation::Underivable { reason: String::from("no_artifact_in_clause") },
};
```

The private `Obligation` at `src/agentic_coding/evidence_record.rs:57-67` is deleted and
its two write-only checks (`:271`, `:274`) route through `ObligationLedger::observe`.

### Pipeline wiring

`src/meta_core.rs:79-117` keeps its thirteen stages unchanged. The new stage runs **after**
dispatch, so `meta_core` gains a second entry point rather than a fourteenth inline call:

```rust
/// Runtime feedback pass, called by `src/solver.rs` after `meta_method_dispatch`
/// has run, with whatever observations the turn produced.
pub fn record_meta_core_execution(
    log: &mut EventLog,
    frame: &ProblemFrame,
    planned: &NeedLedger,
    obligations: &ObligationLedger,
) -> NeedLedger;
```

`src/recipe_interpreter.rs` gains `"record_obligation_ledger"` in `run_recorder`
(`:269-354`), a `require_obligation_ledger` guard beside the existing `require_need_ledger`
(`:356-384`), and an `obligation_ledger: Option<ObligationLedger>` slot in
`ExecutionContext` (`:252-263`). Without this the data-driven path errors with "recipe
binds unknown recorder" (`:352`) and R343 parity breaks.

### Recipe step 14, `data/meta/recursive-core-recipe.lino`

```
step_verify_obligations
  record_type "meta_step"
  order "14"
  id "verify_obligations"
  title "Discharge every obligation against an execution record"
  detail "ObligationLedger::observe binds each observation to the node that expected it, so an unrelated result can never clear a step. need_ledger_with_execution then projects a second need ledger whose rows reach Satisfied only where an obligation discharged while carrying an Evidence: the command, its exit code or an explicit none, and a SHA-256 of the observed bytes. A node whose expectation is Underivable is split through split_once_checkable and re-entered; when nothing is left to split the node becomes Unsatisfiable with a named reason and the answer reports that gap instead of completion prose."
  source_file "src/obligation_ledger.rs"
  records "record_obligation_ledger"
```

### New `.lino` contract, `data/meta/obligation-evidence-contract.lino`

```
obligation_evidence_contract
  record_type "meta_invariant"
  satisfied "a node reaches satisfied only while carrying an execution record with a command, an exit code or an explicit none, and a sha256 of the observed bytes"
  binding "an observation discharges only the node whose expectation names its command or path; an unrelated result clears nothing"
  underivable "a clause with no derivable expectation is split; a clause that cannot be split is reported as an unsatisfied gap with its byte span"
  honesty "unattempted and unsatisfiable are reportable outcomes; absence of an observation is never rendered as completion"
  determinism "record identity derives from command, argv, exit code and observed hash only; no wall clock, pid or machine identity enters the id"
  hashing "the full observed bytes are hashed; output that is not byte-stable must use a command-exit or file-bytes expectation, never an output hash"
  source_reader "src/obligation_ledger.rs"
```

And the derivation rules from clause shape to expectation, so a new expectation shape is a
data edit wherever it can be:

```
obligation_expectation_rules
  record_type "meta_rule_set"
  rule literal_file_write
    when "general_plan_mode literal_file"
    expectation "file_bytes"
  rule command_output_write
    when "general_plan_mode command_output"
    expectation "output_hash"
  rule repository_work_item
    when "general_plan_mode repository_work_item"
    expectation "underivable"
    reason "planned_not_executed_has_no_verification_command"
  rule generated_check
    when "leaf has generated_test"
    expectation "symbolic_check"
```

### Failure and honesty behaviour

- No observation → `Unattempted` → the need row stays `Planned` → the answer names the
  planned-but-unobserved obligations with their byte spans.
- Observation contradicts expectation → `Refuted { record, mismatch }` → the node reopens
  for decomposition; past the split bound it becomes `Unsatisfiable`.
- Harness reports no exit code (the `"ok"` case at
  `tests/unit/issue_1099_multiple_obligations.rs:56-58`) → `exit_code: None`,
  `kind: ToolResult`. Against a `FileBytes { sha256: Some(..) }` expectation that
  **refutes**, because a bare `"ok"` hashes to the wrong value. The existing test is
  updated to return real bytes; the gate is not weakened.
- Every discarded-clause path disappears: `obligations()` produces a node per clause.

## Tests first

### Held-out multilingual cases, with the actual prompt text

New file `tests/unit/issue_1138_obligation_evidence.rs`. Each prompt names an artifact in
its first clause and, in its second, a *check* no composer can read an artifact out of —
so the case fails today by silent discard at `task_obligations.rs:55`, and passes only
when the clause becomes an `Underivable` node that decomposes.

Seeded wordings:

- **en**: `"Two things. First, create file notes/attribution.md containing Gemfile.lock. Second, confirm the first line of that file is exactly Gemfile.lock."`
- **ru**: `"Две вещи. Сначала создай файл notes/attribution.md с содержимым Gemfile.lock. Затем подтверди, что первая строка этого файла — ровно Gemfile.lock."`
- **hi**: `"दो काम। पहले notes/attribution.md फ़ाइल बनाओ जिसमें Gemfile.lock हो। फिर पुष्टि करो कि उस फ़ाइल की पहली पंक्ति ठीक Gemfile.lock है।"`
- **zh**: `"两件事。首先，创建文件 notes/attribution.md，内容为 Gemfile.lock。然后确认该文件的第一行正好是 Gemfile.lock。"`
- **es**: `"Dos cosas. Primero, crea el archivo notes/attribution.md con el contenido Gemfile.lock. Después confirma que la primera línea de ese archivo es exactamente Gemfile.lock."`

Held-out paraphrases — different surface wording, asserted to produce the *same* obligation
tree shape (two nodes: one `FileBytes`, one `SymbolicCheck` after decomposition):

- **en**: `"I need two things done. Write Gemfile.lock into notes/attribution.md. After that, check that its opening line reads Gemfile.lock and nothing else."`
- **ru**: `"Нужно сделать две вещи. Запиши Gemfile.lock в notes/attribution.md. После этого проверь, что его первая строка — это Gemfile.lock и ничего больше."`
- **hi**: `"दो चीज़ें चाहिए। notes/attribution.md में Gemfile.lock लिखो। उसके बाद जाँचो कि उसकी पहली पंक्ति सिर्फ़ Gemfile.lock है।"`
- **zh**: `"需要完成两件事。把 Gemfile.lock 写入 notes/attribution.md。之后检查它的首行只有 Gemfile.lock。"`
- **es**: `"Hay dos cosas que hacer. Escribe Gemfile.lock en notes/attribution.md. Luego revisa que su primera línea sea solo Gemfile.lock."`

`data/seed/meanings-conversation.lino:617-640` currently seeds `en` and `ru`
`enumeration_cue` surfaces only. Spanish (`primero`, `después`, `luego`), Hindi (`पहले`,
`फिर`, `उसके बाद`) and Chinese (`首先`, `然后`, `之后`) surfaces are added there as seed
data — never as Rust literals, which `scripts/check-hardcoded-language.rs` would reject.

### Unit and specification tests

| Path | Name | Asserts |
| --- | --- | --- |
| `tests/unit/specification/execution_evidence.rs` | `a_record_id_is_stable_across_runs_and_machines` | same command/argv/exit/bytes → same `record_id`; no wall clock in the fingerprint |
| | `an_absent_exit_code_is_recorded_as_none_not_as_zero` | `from_tool_result("cat x", "ok", Harness).exit_code == None` |
| | `the_observed_hash_matches_source_fetch_sha256_hex` | exactly one digest implementation |
| | `a_harness_record_cannot_claim_a_local_process_source` | `source` is set by the call site, not the payload |
| `tests/unit/specification/obligation_ledger.rs` | `satisfied_is_unconstructible_without_an_execution_record` | enumerates every constructor; the variant's only field is the record |
| | `need_ledger_with_execution_is_the_only_producer_of_satisfied` | greps `src/` for `NeedStatus::Satisfied` construction sites |
| | `an_unrelated_observation_discharges_nothing` | `observe` returns `None` when no expectation names the command or path (R710-R4) |
| | `a_clause_with_no_derivable_expectation_becomes_a_node_not_a_discard` | the second clause appears in `to_links_notation` with its span |
| | `an_underivable_node_is_split_before_it_is_called_unsatisfiable` | `Decompose` precedes `ReportGap` |
| | `the_split_is_bounded_by_the_existing_split_depth_bound` | depth ≤ `DEFAULT_SPLIT_DEPTH_BOUND`; no new constant |
| | `a_refuted_observation_reopens_the_node_instead_of_finishing_it` | |
| | `every_obligation_discharged_is_false_while_any_node_is_unattempted` | |
| `tests/unit/specification/meta_frame.rs` (extend) | `the_planning_ledger_still_records_planned_not_satisfied` | the pre-existing behaviour is unchanged |
| `tests/unit/specification/recipe_interpreter.rs` (extend) | `the_execution_pass_is_bound_to_a_known_recorder` | `"record_obligation_ledger"` resolves; an unknown name still errors at `:352` |
| | `native_and_data_driven_execution_produce_the_same_events` | R343 parity for the new stage |
| `tests/unit/issue_1138_obligation_evidence.rs` | `a_second_clause_without_an_artifact_is_never_silently_dropped` | five languages |
| | `a_held_out_paraphrase_produces_the_same_obligation_tree_shape` | five languages |
| | `the_session_does_not_finalize_while_any_obligation_is_unattempted` | |
| | `a_gap_is_reported_with_its_clause_and_byte_span_not_as_completion` | |
| | `a_bare_ok_tool_result_does_not_satisfy_a_file_bytes_expectation` | replaces the `"ok"` shortcut |
| `tests/unit/docs_requirements/issue_1138.rs` (**reconciled: was `tests/unit/docs_requirements_issue_1138.rs`; now the directory form plans 01, 03 and 06 use, so there is one file — plan 00 §9 R11**) | `issue_1138_obligation_ledger_is_traceable` | grep-pins `pub struct ObligationLedger`, `fn record_obligation_ledger`, and the `crate::obligation_ledger::record_obligation_ledger` call, matching `tests/unit/docs_requirements_issue_559.rs:107-141` |
| `tests/unit/specification/recursive_core_recipe.rs` (extend) | existing grounding | step 14 exists, order 1..14 contiguous, `source_file` exists, `records` binds a real recorder |

### Gates and ratchets

- **New ratchet** `data/meta/obligation-evidence-ratchet.lino` with
  `satisfied_without_record "0"`, asserted across the benchmark corpus; the floor never
  rises above 0.
- **Amended pin.** `tests/unit/specification/recipe_interpreter.rs:229-231` pins
  `satisfied "0"` on the *planning* ledger. It stays. **This strictly adds and is
  not a loosened gate under plan 00 §6.7; it is the only existing assertion this
  plan touches (plan 00 §9 X10).** A sibling assertion is added: the
  *executed* ledger for the same input reports `satisfied "0"` when no observation was
  supplied, and `satisfied "N"`, `N >= 1`, when one was. This is the one place the plan
  touches an existing assertion, and it strictly adds.
- `every_need_accounted_for` is **not** weakened; the agentic `Final` gate reads the new
  `every_obligation_discharged` instead.
- `rust-script scripts/check-hardcoded-language.rs` must not gain an allowlist entry: every
  new cue surface is seed data. The 1,286 allowlist count must not rise.
- `cargo run --example regenerate_self_ast_census` after the two new `src/` modules, so
  `data/meta/self-ast/` stays 1:1 with `src/` (`VISION.md:61-62`).

## Implementation leaves

- [ ] Add the `es`, `hi`, `zh` `enumeration_cue` surfaces to
      `data/seed/meanings-conversation.lino`; pin them in a multilingual specification
      test. No Rust change.
- [x] Promote `agentic_coding::tool_result::reported_exit_code` (`:78`) from `pub(super)`
      to `pub(crate)`; one line plus its doc comment, no behaviour change.
- [x] Add `src/execution_evidence.rs` with `ObservationKind`, `EvidenceSource`,
      `Evidence`, `observed`, `from_tool_result`, `reports_success`,
      `to_links_notation`; register in `src/lib.rs`; tests in
      `tests/unit/specification/execution_evidence.rs`.
- [ ] Add `ObligationExpectation` and `ObligationOutcome` to a new `src/obligation_ledger.rs` with
      `to_links_notation` for both; test the type-level `Satisfied` guarantee.
- [ ] Add `ObligationNode` with `build`, `discharged`, `next_open`, `collect_leaves`,
      `to_links_notation`; `build` derives expectations, no recursion yet.
- [ ] Add the derivation rules to `data/meta/obligation-evidence-contract.lino` and read
      them in `build`, so a new expectation shape is a data edit.
- [ ] Wire `build` to `task_decomposition::split_once_checkable` for `Underivable` clauses,
      bounded by `recursive_execution::DEFAULT_SPLIT_DEPTH_BOUND`.
- [ ] Add `ObligationLedger` with `for_frame`, `observe`, `every_obligation_discharged`,
      the three counts and `to_links_notation`; `observe` returns `None` for an unrelated
      record.
- [ ] Add `need_ledger_with_execution`; prove by test it is the only producer of
      `NeedStatus::Satisfied` in `src/`.
- [ ] Add `record_obligation_ledger` and `meta_core::record_meta_core_execution`; call it
      from `src/solver.rs` after `meta_method_dispatch`.
- [ ] Register `"record_obligation_ledger"` in `src/recipe_interpreter.rs::run_recorder`
      plus `require_obligation_ledger`; extend `ExecutionContext`; parity test.
- [ ] Add step 14 to `data/meta/recursive-core-recipe.lino`; extend
      `tests/unit/specification/recursive_core_recipe.rs` for order 1..14.
      **This leaf changes every recorded trace, as does plan 07's
      `SelfImprovementMode` default flip. Plan 14 orders this one first and gives
      each its own R343 parity run, so a parity failure has exactly one cause
      (plan 00 §9 X11).**
- [ ] Rewrite `src/agentic_coding/task_obligations.rs` over `ObligationNode`: replace the
      `continue` at `:55`, keep `pub fn obligations` compiling.
- [ ] Add `next_step` and `ObligationStep`; switch `src/agentic_coding/planner.rs:373-379`
      to match on it; `ReportGap` renders clause + span + reason, never completion prose.
- [ ] Delete the private `Obligation` at `src/agentic_coding/evidence_record.rs:57-67`;
      route `:271` and `:274` through `ObligationLedger::observe`.
- [ ] Replace the `"ok"` tool-result shortcut at
      `tests/unit/issue_1099_multiple_obligations.rs:56-58` with real observed bytes.
- [ ] Add `data/meta/obligation-evidence-ratchet.lino`; ground it.
- [ ] Add `tests/unit/issue_1138_obligation_evidence.rs` (ten prompts, five languages);
      register in `tests/unit/mod.rs`.
- [ ] Add `tests/unit/docs_requirements/issue_1138.rs` grep-pins.
- [ ] Regenerate `data/meta/self-ast/`; run
      `rust-script scripts/assemble-requirements.rs --write`.
- [ ] Add the `changelog.d/` fragment.

## Docs to update

The exact quoted statements and their replacement text moved to plan 11's
findings table on 2026-09-16, so there is one docs authority and no document
is described in two places (plan 00 §8). This plan's entries are rows
**D185-D197** of
[`11-docs-consistency-audit.md`](11-docs-consistency-audit.md) §"Issue #1138
plan doc replacements", and plan 11's leaves apply them after the ledger rows
they cite exist (plan 00 §7).

| row | document |
| --- | --- |
| D185 | `docs/requirements/issue-0559-general-meta-algorithm.md:18-26` |
| D186 | `docs/requirements/issue-0710-repository-and-retention-continuation.md:18` |
| D187 | `docs/requirements/issue-0710-repository-and-retention-continuation.md:13` |
| D188 | `docs/meta-algorithm.md:144` |
| D189 | `docs/meta-algorithm.md:172-173` |
| D190 | `docs/meta-algorithm.md:152-153` |
| D191 | `data/meta/recursive-core-recipe.lino:5` |
| D192 | `data/meta/recursive-core-recipe.lino:6` |
| D193 | `data/meta/recursive-core-recipe.lino:35` |
| D194 | `docs/requirements-traceability.md:406` |
| D195 | New shard `docs/requirements/issue-1138-bottleneck-audit.md` |
| D196 | `VISION.md:192` |
| D197 | `ROADMAP.md` |

Any further document this plan's implementation touches is added as a new
plan 11 row, never as a second copy here.

## Risks and open questions

1. **R343 parity is the hardest gate.** Executing the recipe must reproduce the native log
   event-for-event across every mode combination
   (`docs/requirements/issue-0559-general-meta-algorithm.md:42`). The new stage runs
   *after* dispatch, which the interpreter does not model. Open: does the interpreter gain
   a second program (`execution_sequence`), or does `ExecutionContext` carry observations
   forward? Leaning to the former, because one ordered program spanning a dispatch it does
   not perform would be dishonest data.
2. **Where do observations come from on a purely symbolic turn?** `SymbolicCheck` is the
   answer, but `check_id` must name something re-runnable. The candidate is the generated
   tests of loop step 6 (`VISION.md:189`), which `PortfolioLeaf::run_tests`
   (`src/draft_portfolio.rs:87`) already produces. Plan 12 needs the same vocabulary, so
   this must be settled jointly with it before either lands.
3. **`satisfied "0"` is a pinned floor.** Amending
   `tests/unit/specification/recipe_interpreter.rs:229-231` is the single place this plan
   makes an existing assertion less strict about the planning ledger. It must be replaced
   by a strictly stronger assertion about the executed ledger, never merely deleted.
4. **Hash of what, exactly, for a long or unstable output?** Hashing the full raw bytes is
   deterministic but brittle (timestamps, absolute paths). Options: hash the full bytes
   (chosen — honest), or hash a normalized projection (rejected — a normalizer is a place
   to hide a mismatch). Consequence: tasks whose output is not byte-stable must use
   `CommandExit` or `FileBytes`, never `OutputHash`. Stated in the contract.
5. **`RepositoryWorkItem` plans carry an empty `verification_command`**
   (`src/agentic_coding/general_planner.rs:284`). Under this plan they become
   `Underivable`, so every repository work item will now decompose. That is correct, but it
   changes behaviour on a live path and needs a dedicated regression over the #848 ladder
   before it lands.
6. **Migration order.** Plan 06 (prerequisite discovery) and plan 03 (workspace protocol)
   both want `Evidence`. If B5 lands after them, each will invent one. B5 should
   land before 06 and 03, even though the issue's proposed order puts it fourth.
7. **Unresolved:** whether `ObligationExpectation` belongs entirely in seed data. The doctrine
   prefers `.lino`; the counter-argument is that an expectation is a *type*, not a
   vocabulary. Current leaning is the split above — variants in Rust, derivation rules in
   `data/meta/obligation-evidence-contract.lino`.
8. **`ObligationNode` duplicates part of `WorkUnit`.** Two trees over the same prompt
   invite drift. The join test must assert that every `WorkUnit` leaf span is covered by an
   obligation node span, or the two will diverge silently within a release.
   **Reconciled ordering: plan 12 rewrites `WorkUnit::build` into a binary tree,
   which changes the leaf set this join matches against. The join test is written
   here, before plan 12's leaf, and must stay green through it; plan 14 orders
   this plan before plan 12's `WorkUnit` leaf for that reason (plan 00 §9 X14).**
