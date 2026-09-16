# Plan 07 — Learning loops that change behavior (bottleneck B7 of #1138)

Status: design recorded before implementation. No loop below is claimed to change an
answer today except the two named in "Current state"; every claim carries a `file:line`.

## Issues addressed

- **#1138 B7** — "every learning loop is proposal-only, so nothing learned changes later
  behavior." This plan delivers the whole bottleneck: learned items that execute, a
  before/after delta backed by an observation, the human gate relocated to review time,
  the PR #887 decision, and automatic end-to-end source-cache reconstruction.
- **#701 (closed)** — "learned items must demonstrably change answers." R701-2 proved it
  for **one** knowledge class: 60 before/after pairs over request-opener surfaces in four
  languages (`docs/requirements/issue-0701-auto-learning-adoption-gap.md:14`). This plan
  generalizes the same criterion across the whole method registry, and adds `es` to the
  language matrix.
- **#922 (closed)** — R922-2 "keep all learned candidates inert until benchmark-gated,
  human-confirmed promotion"; R922-5 "Learned records are separate from compiled handlers,
  so dispatch order is unchanged"
  (`docs/requirements/issue-0922-method-learning-from-experience.md:13,16`). One method was
  adopted (`data/seed/learned-methods.lino`) and is loaded on every answer and never used.
  This plan makes adopted learned methods executable.
- **#656 (closed)** — the promotion protocol stops at a local branch and prints the
  `git`/`gh` commands a human must run (`src/promotion.rs:365-389`). R472
  (`docs/requirements/issue-0656-benchmark-gated-promotion-protocol.md:26`) is right that
  GitHub checks and human review are the final authority; this plan moves the gate to
  *that* review instead of to inertness, by opening a draft pull request.
- **#364 (closed)** — "periodically attempt to synthesise new seed rules." The loop exists
  (`src/self_improvement.rs:167`) but its ingestion path hard-codes a 0/0 benchmark gate
  (`:248-252`), so every proposal is `BlockedByBenchmark` by construction, and its
  destination `data/seed/learned-program-rules.lino` (`src/promotion.rs:47`) **has never
  existed on disk**.
- **#558 (closed)** — "actually recompile itself and reattach to the UI." The approved-lesson
  read path is live (`src/solver.rs:448`), but the ledger is a one-element constant
  (`src/learning_ledger.rs:311-320`, `:337-339`).
- **#705 (open), PR #887** — anticipatory dreaming, `CONFLICTING`/`DIRTY` and untouched
  since 2026-08-01 (49 files, +6,804/−203). This plan decides its fate and says why.
- **#710 / R710-R7** — "End-to-end automatic source-cache reconstruction remains open"
  (`docs/requirements/issue-0710-repository-and-retention-continuation.md:16`). **Issue
  #1138 cites this as R710-R8; that is an error.** R710-R8 is the Agent-CLI meaningful-work
  row (`:17`). The plan targets R710-R7 and says so.
- **#540 (closed)** — dreaming amendments must change solving, not decorate answers. They
  do (`src/dreaming_application.rs:176-184`) — but only through `src/protocol.rs`; the
  engine path never sees them.

## Current state

### Exactly two learned artifacts reach a live answer, and both are one item wide

| Loop | Read at answer time? | Read path | Corpus size |
| --- | --- | --- | --- |
| `learning_ledger` approved repair lessons (#558) | **yes** | `src/learning_ledger.rs:347` → `src/rule_synthesis.rs:247` → `src/solver.rs:448` | **1** |
| `dreaming_application` retained amendments (#540/#701) | **yes** | `src/dreaming_application.rs:65` → `src/protocol.rs:477`, `:777` | topic-matched, protocol surface only |

`canonical_ledger()` is a constant, `src/learning_ledger.rs:311-320`:

```rust
pub fn canonical_ledger() -> LearningLedger {
    let mut ledger = LearningLedger::new();
    ledger.promote(&crate::self_healing::canonical_case(), &HumanApproval::granted("maintainer"))
        .expect("the canonical self-healing case is green and approvable");
    ledger
}
```

and its fast-miss guard is a one-element vector, `:337-339`:

```rust
pub fn canonical_ledger_failure_prompts() -> Vec<String> {
    vec![crate::self_healing::canonical_failure_trace().prompt]
}
```

`approved_lesson_for` (`:347`) returns `None` for anything outside it (`:353-359`), and no
code path promotes a second lesson into the process-global
`static APPROVED_LEDGER: OnceLock<LearningLedger>` (`:348`).

`dreaming_application`'s read path is exhaustively `src/protocol.rs:460, 477, 669, 766,
777, 891` and `src/dreaming_runtime.rs:125`. `FormalAiEngine::answer` never passes
`memory_events`, so the CLI and library surfaces get no amendments at all.

### Everything else terminates in a `.lino` string nobody reads

`grep -n "fs::write\|write_locked_atomic\|File::create"` over `self_improvement.rs`,
`method_learning.rs`, `learning_cycle.rs`, `learning_ledger.rs`,
`learning_adoption_ledger.rs`, `self_healing.rs`, `meta_self_improvement.rs` → **zero
hits**. The only writer in the whole learning stack is
`src/promotion/materialize.rs:98` (`fs::write`), reachable only through
`formal-ai improve --promote --apply --confirm`.

**Method learning (#922) — the sharpest case.** `data/seed/learned-methods.lino` holds one
adopted record, `learned_recursive_core_e17957243eaaf6db`, with fifteen `operation` rows.
It **is** embedded (`src/seed/embedded_registry.rs:47`) and **is** parsed into
`MethodRegistry::from_dispatch()` (`src/method_registry.rs:142`), which runs on every
answer (`src/meta_method_dispatch.rs:38`, `src/intent_formalization.rs:252`, `:696`).
And then it is ignored:

```rust
// src/method_registry.rs:124-131
pub struct MethodRegistry {
    pub methods: Vec<Method>,
    /// Promoted learned abstractions, kept out of compiled dispatch until an
    /// implementation supplies an executable handler.
    pub learned_methods: Vec<LearnedMethod>,
}
```

`ordered_method_names_for_relevants` (`:237-264`) — the function
`meta_method_dispatch::try_dispatch` calls at `src/meta_method_dispatch.rs:39` — iterates
`self.methods` in three loops (`:239`, `:246`, `:255`) and never touches
`learned_methods`. The accessor `learned_method(&self, name)` (`:215`) has **zero
production callers**; its only callers are `tests/unit/issue_922_method_learning.rs:140`,
`:171`, `:184`.

The withhold is stated twice more: `src/method_learning.rs:43-48` returns the constant
`"proposal_only"` from `mode()`, and `docs/meta-algorithm.md:772-775` says "Learned
records are observable in the registry event but are separate from compiled handlers, so
adoption cannot silently introduce executable behavior or alter precedence."

**Self-improvement (#364) — blocked by construction.** `src/self_improvement.rs:248-252`:

```rust
// Ingestion never claims that CI ran. A real benchmark result is supplied
// later when a maintainer constructs and approves the repair case.
let learning = learn_rules_from_unknown_traces(
    std::slice::from_ref(&trace),
    BenchmarkGateReport::issue_362_from_counts(0, 0),
);
```

`permits_adoption()` is `self.passed >= self.minimum_pass_count` (`:150-154`), so a 0/0
report can never clear a non-zero floor: every ingested trace is `BlockedByBenchmark`
(`:552-556`). `learned_program_rule_lino` (`:586-595`) mints a complete, parse-validated
seed body for `data/seed/learned-program-rules.lino` (`src/promotion.rs:47`) — a file that
does not exist (`ls data/seed/learned-program-rules.lino` → no such file), unlike its
siblings `learned-methods.lino` and `learned-request-openers.lino`, which do.

**Meta self-improvement (#559/R340) — off by default.**
`src/meta_self_improvement.rs:41-48` makes `Off` the `#[default]`; `propose_recipe_update`
(`:243-251`) returns `None` in that mode. `MetaRecipeProposal::to_links_notation` (`:146`)
emits a complete recipe delta that nothing writes back.

**Promotion (#656) — stops one step before review.** `PromotionBranchPlan`
(`src/promotion.rs:302-314`) is documented as "printed for a human to run; the protocol
never executes them", `branch_plan()` (`:365-389`) *builds* the `git commit` and
`gh pr create --draft --fill` strings, and `prepare_local_branch`
(`src/promotion/materialize.rs:114-133`) runs only `git switch -c`. `src/cli_improve.rs:113-124`
prints the rest to stderr. So the artifact that a human is supposed to review never
reaches a place where a human reviews artifacts.

**Learning cycle (#701) — a proposal-only literal.** `src/learning_cycle.rs:260-261`
writes `mode "proposal_only"` and `human_gated "true"` as hard-coded strings;
`dreaming_runtime::write_learning_cycle_record` (`:133-145`) drops a
`<memory>.learning-cycle.lino` on every idle run that **nothing reads back**.

**The eleven `*_learning.rs` reports are one derivation with eleven names.**
`src/agentic_coding/learning_report.rs:46-58` lists them; each ranks a compile-time
`include_str!` fixture (`memory: &'static str`, `:90`) and renders
`decision "awaiting_human_review"` (`:179-182`). None is read back at answer time:
`execution_learning.rs` → `client-execution-learning-report.lino` (`:10`),
`routing_learning.rs` → `tool-routing-learning-report.lino` (`:9`),
`code_rewrite_learning.rs` → `code-rewrite-learning-report.lino` (`:10`),
`algorithm_learning.rs` → `discovered-algorithms.lino` (`:25`) with
`human_gated "true"` (`:208`). `AlgorithmCandidate::promote`
(`src/algorithm_discovery.rs:214`) has zero production callers.

### Source-cache reconstruction is half-built

`src/dreaming/retention.rs:75-94` builds the authorization:

```rust
let reconstructable = event.role.as_deref() == Some("cache")
    && (event.evidence.iter().any(|evidence| {
        evidence.strip_prefix("rediscover:https://")
            .is_some_and(|location| !location.is_empty() && …)
    }) || (event.evidence.iter().any(|e| e == "reconstruct:embedded-seed") && …));
```

`reconstruction_record` (`:10-26`) keeps `kind: "cache_reconstruction"`, the original id,
the tool, the conversation and the evidence, and drops `content`. The forget executes
(`src/dreaming/apply.rs:123-129`), the stub survives — and **nothing in `src/` ever
executes a `rediscover:https://…` edge**. The real fetcher is `src/source_fetch.rs`
(`source_cache_url_mismatch:281`, `source_cache_content_hash_mismatch:298`), never wired to
`reconstruction_record`. That is precisely the open half of R710-R7.

### PR #887 as it stands

`gh pr view 887 --json` on 2026-09-16: `mergeable: CONFLICTING`, `mergeStateStatus: DIRTY`,
`createdAt: 2026-08-01T13:01:51Z`, `updatedAt: 2026-08-01T16:46:59Z`, 49 files,
+6,804/−203, 6 commits, base `main`, head `issue-705-5bb14827217a`.

Split by conflict class:

| Class | Files | Conflict risk |
| --- | --- | --- |
| **New source, no conflict** | `src/anticipation.rs` (+934), `src/anticipation/expansion.rs` (+223), `src/anticipation/ledger.rs` (+94) | none — new paths |
| **New tests/examples/case study** | `tests/unit/issue_705_anticipation.rs` (+484), `tests/unit/docs_requirements_issue_705.rs` (+202), `examples/issue_705_anticipatory_dreaming.rs`, two `experiments/*.sh`, 16 files under `docs/case-studies/issue-705/` | none — new paths |
| **Generated artifacts** | `data/meta/self-ast/src/lib.lino` (+145/−144), `data/meta/self-ast/index.lino`, four other `self-ast` files | regenerate, never merge |
| **Aggregating registries** | `src/lib.rs` (+1), `tests/unit/mod.rs` (+2) | one-line re-adds |
| **Now-generated document** | `REQUIREMENTS.md` (+22) | **cannot be merged**: `REQUIREMENTS.md` gained the banner "Generated by `rust-script scripts/assemble-requirements.rs --write` from `docs/requirements/`. Edit the shard for your issue, never this file" (`scripts/assemble-requirements.rs:44`). PR #887 predates the sharding, so its 22 lines must become a new shard |
| **Hand-maintained docs** | `ROADMAP.md` (+2/−2), `ARCHITECTURE.md` (+10/−2), `docs/meta-algorithm.md` (+19/−3), `data/meta/dreaming-recipe.lino` (+71/−3) | six weeks of drift; re-apply by hand |
| **True semantic edits to live files** | `src/dreaming_application.rs` (+7/−1), `src/dreaming_runtime.rs` (+4), `src/memory_sync.rs` (+4/−4), `tests/unit/issue_540_agent_cli.rs` (+2/−2), `tests/unit/specification/dreaming_meta_algorithm.rs` (+4/−4), `docs/case-studies/issue-540/dreaming-gap-analysis.lino` (+29/−1) | must be re-applied against today's code either way |

## Root causes

1. **Inertness is used as the safety mechanism, so the gate and the capability are the same
   switch.** `src/method_registry.rs:128-130`, `src/meta_self_improvement.rs:44-45`,
   `src/method_learning.rs:43-48`. Turning the gate off is the only way to get the
   capability, so the capability is never on. *Mechanism:* safety was implemented as
   "never run it" instead of "run it visibly, and let a human review the diff."
2. **A learned item has no executable form.** `LearnedMethod`
   (`src/method_registry.rs:76-89`) holds `operations: Vec<String>` and nothing that can
   run them, and the doc at `:70-74` says the blocker is "no executable handler". *But the
   operations are recorder event kinds* — `need:status`, `method_registry`,
   `work_unit_reasoning`, `upward_construction`, `solution_evidence`, `selection`,
   `skill_ledger`, `reasoning_standard` — the exact vocabulary
   `src/recipe_interpreter.rs:269-354` already dispatches on. An interpreter for them
   exists; nothing connects the two. *Mechanism:* the two modules were built for different
   issues (#559 and #922) and never met.
3. **Adoption has no definition of "changed the answer" that a machine can check
   generally.** `AdoptionPair::adopted()` (`src/learning_adoption_ledger.rs:66-73`) is
   `before == "unknown" && after != "unknown" && topic_recovered()` — an intent-slug
   comparison usable only for the `unknown`→routed transition of one class. *Mechanism:*
   the predicate was written for the Trends frontier and is shaped like it.
4. **Each loop can express exactly one knowledge class, so "learned" never compounds.**
   #364 → program-plan substitution rules only (`src/self_improvement.rs:586-595`);
   #558 → one approved repair lesson keyed by failure prompt
   (`src/learning_ledger.rs:117`); #701 → request-opener lexeme surfaces only
   (`TERM_INFORMATION_ROLE`, `src/learning_cycle.rs:129`); #922 → ordered event-kind
   sequences only (`src/method_learning.rs:21-24`). *Mechanism:* four independent
   pipelines, four independent seed formats, no shared adoption contract.
5. **The gate is evaluated where no evidence can exist.** `src/self_improvement.rs:248-252`
   supplies a 0/0 report at *ingestion*, so a proposal is judged before any benchmark could
   have run. *Mechanism:* the gate type is required at construction and there was nothing
   honest to put in it.
6. **The review artifact never reaches review.** `src/promotion.rs:365-389` prints the
   commands; `src/cli_improve.rs:113-124` echoes them to stderr. A local branch on a
   maintainer's laptop is not a human gate; it is an unreviewed local edit that nobody sees.
   *Mechanism:* "never a direct push" was read as "never any network action", conflating
   pushing to `main` with opening a draft PR.
7. **The one working amendment path is wired to one surface.**
   `src/dreaming_application.rs:65` is called only from `src/protocol.rs`.
   *Mechanism:* the plumbing (`memory_events`) exists on the protocol type and not on
   `FormalAiEngine`.
8. **Reconstruction is authorized but never performed.** `src/dreaming/retention.rs:75-94`
   proves a payload *may* be refetched; `src/source_fetch.rs` knows *how*; no edge joins
   them. *Mechanism:* forgetting and fetching were built in different releases and the
   forget side only needed to prove safety, not to demonstrate recovery.
9. **PR #887 rotted against a moving sharding policy.** Its `REQUIREMENTS.md` edit was
   legal on 2026-08-01 and is forbidden today (`data/meta/merge-conflict-policy.lino`;
   `scripts/assemble-requirements.rs:1-12` cites 64 manual resolutions in that one file).
   *Mechanism:* a long-lived branch touching a file whose editing rules changed under it.

## Solution options

### Option A — Keep proposal-only; add a "shadow execution" report

*Description.* Learned items stay out of dispatch. A parallel shadow run executes them and
records what the answer *would* have been, in a report a human reads.

*Architecture sketch.* A `shadow_dispatch` that runs the learned registry beside the
compiled one and diffs the two event logs.

*Pros.* Zero behaviour risk. Produces the before/after evidence #701 wants. Compatible with
every existing gate as written.

*Cons.* It does not close B7. The bottleneck is "nothing learned changes later behavior",
and a shadow report changes nothing. It also doubles the per-answer cost and creates a
second dispatch authority, which R344
(`docs/requirements/issue-0559-general-meta-algorithm.md:43`) explicitly removed.

*Doctrine fit.* Fails "human gates apply at review time, not by making learned items inert."

*Effort.* Medium. *Risk.* Low, and pointless.

### Option B — Learned items become executable registry methods, gated at review time

*Description.* A `LearnedMethod`'s `operations` are compiled into a `RecipeProgram` and
executed through the existing interpreter. Adopted learned methods enter
`ordered_method_names_for_relevants` at a declared precedence, **after** every compiled
method, and every answer that used one says so in its trace. Adoption requires a proven
`BehaviorDelta` on held-out paraphrases in five languages. The human gate is the review of
the pull request that carries the seed edit — which the promotion protocol now actually
opens, as a draft.

*Architecture sketch.*

```
event logs ─► method_learning ─► MethodProposal
                                    │
                              behavior_delta::prove  (held-out, 5 languages,
                                    │                 each side an Evidence)
                                    ▼
                              promotion (canonical gates replayed)
                                    │  --apply --confirm
                                    ▼
                    data/seed/learned-methods.lino  (status "adopted")
                                    │  --open-draft-pr
                                    ▼
                              draft PR ──► human review ──► merge
                                    │
                                    ▼
          MethodRegistry::from_dispatch → ordered_method_names_for_relevants
                                    │
                              recipe_interpreter executes the operations
```

*Pros.* Closes B7 as stated. Reuses the interpreter that already exists rather than adding
a second one. Keeps one dispatch authority (R344) by putting learned methods *inside* the
registry order rather than beside it. The delta proof is the #701 criterion, generalized.
The gate becomes the thing the doctrine names: review time.

*Cons.* Learned methods can now change answers, which is the point and also the risk. A
bad adoption is a behaviour regression, so the ratchets must be real and the precedence
must be last. Requires plan 05's `Evidence` to land first.

*Doctrine fit.* Strong on every clause: associative stack (the learned item is `.lino`,
the interpreter is the existing recipe runner); generalization (one adoption contract for
all four classes); deterministic (content-addressed, replayable); honest (the trace names
the learned method); human gate at review time; nothing hard-coded for a test.

*Effort.* Large, but mostly wiring — the interpreter, the promotion protocol, the
adoption ledger and the delta vocabulary all exist in parts.

*Risk.* Medium-high, mitigated by last-place precedence and a strict-downward ratchet.

### Option C — Promote learned items into compiled Rust, regenerate, rebuild

*Description.* Take the vision's "recompile itself and reattach to the UI" (#558) at face
value: a learned method is emitted as Rust from the meta language
(`VISION.md:53-62`), the crate is rebuilt, the binary is swapped.

*Pros.* The literal reading of #558 and of `VISION.md:53-66`. Maximum capability.

*Cons.* Requires a working meta-language → Rust emitter, which is `src/self_ast_census.rs`
plus `data/meta/self-ast/` — a *census*, not a compiler. Requires a rebuild loop
(`src/agentic_coding/rebuild_plan.rs` is a recipe document, not an executor). It cannot be
delivered in this pull request, and it would make every learned item a build-time artifact,
which destroys forget-and-rediscover: a forgotten learned method could not be rediscovered
without a rebuild.

*Doctrine fit.* Right direction, wrong increment. Fails "everything discoverable must be
forgettable and rediscoverable" at this scale.

*Effort.* Very large. *Risk.* Very high.

### Option D — One unified `learning` module replacing the four pipelines

*Description.* Delete `self_improvement`, `learning_cycle`, `method_learning` and the
eleven `*_learning.rs` reports; replace them with one adoption pipeline.

*Pros.* Removes root cause 4 at the source. Removes eleven near-duplicate modules
(`src/agentic_coding/learning_report.rs:46-58`).

*Cons.* Each of the four has closed-issue requirement rows pinned to its exact symbol names
(R364 via `REQUIREMENTS.md:629`; R701-1..6; R922-1..6; R459-R472). Deleting them
invalidates a large block of traceability in one change, and the eleven reports are a
*reporting* surface, not a learning one — they belong in the #959 handler-migration ratchet
(plan 09), not here.

*Doctrine fit.* Good on generalization, bad on honesty about what a single PR can prove.

*Effort.* Very large. *Risk.* High.

## Decision

**Option B is selected**, with two pieces of Option D adopted as a *shared contract* rather
than a rewrite: the four pipelines keep their modules and their requirement rows, but they
all produce the same `BehaviorDelta` and all pass through the same adoption ledger.
Option A is rejected because a shadow report is a report, not a behaviour change, and it
resurrects the second dispatch authority R344 removed. Option C is rejected as the right
direction at the wrong increment: without a real meta-language compiler it is not
deliverable, and it would make learned items un-forgettable. Option D is rejected as a
rewrite whose only new capability is deduplication, at the cost of invalidating four
issues' worth of pinned traceability in one PR.

### PR #887 decision: cherry-pick, re-derive, re-apply

**Cherry-pick the new files, re-derive every generated and aggregating file, and re-apply
the six semantic edits by hand.** Not a rebase; not a re-implementation.

Why not a plain rebase:

- `REQUIREMENTS.md` is now a **generated** file (`scripts/assemble-requirements.rs:44`
  banner). A rebase would replay a 22-line direct edit onto a file the merge-conflict
  policy forbids editing (`data/meta/merge-conflict-policy.lino`), and would then conflict
  again on every subsequent regeneration. The correct destination is a new shard,
  `docs/requirements/issue-0705-anticipatory-dreaming.md`, which a rebase cannot produce.
- `data/meta/self-ast/src/lib.lino` (+145/−144) and five sibling `self-ast` files are
  produced by `cargo run --example regenerate_self_ast_census`. Rebasing them is replaying
  a diff of a generated artifact — guaranteed conflict, zero information.
- Six weeks and an unknown number of merges separate the branch from `main`; `ROADMAP.md`
  and `docs/meta-algorithm.md` have both been restructured since (the promotion section at
  `docs/meta-algorithm.md:669` and the method-learning section at `:748` did not exist on
  2026-08-01).

Why not a re-implementation:

- 6,804 lines including 484 lines of behavioural tests
  (`tests/unit/issue_705_anticipation.rs`) and 202 lines of traceability tests, none of
  which conflict.
- The PR contains a byte-pinned, Formal-AI-authored artifact — SHA-256
  `a7d58aa0003dd965f4fc53d3e02f5fbeb93fece526c3803e000bb867ebdedb6f` — with its session
  log, its failed attempt and its prompts preserved under
  `docs/case-studies/issue-705/self-hosting-authorship/`. That is 20 % CLI authorship
  evidence for the self-hosting ledger (#657/#924). Re-implementing discards it, and it
  cannot be re-created after the fact honestly.
- The primary-source research (`docs/case-studies/issue-705/raw-data/online-research.md`)
  and the GitHub snapshots are evidence, not code; they carry forward unchanged.

The mechanics: cherry-pick the three `src/anticipation*` files, the two test files, the
example, the two experiment scripts and the sixteen case-study files onto a branch from
today's `main`; re-add the one line in `src/lib.rs` and the two in `tests/unit/mod.rs`;
write the `REQUIREMENTS.md` delta as a new shard and run
`rust-script scripts/assemble-requirements.rs --write`; run
`cargo run --example regenerate_self_ast_census`; re-apply by hand the six semantic edits
to `src/dreaming_application.rs`, `src/dreaming_runtime.rs`, `src/memory_sync.rs`,
`data/meta/dreaming-recipe.lino`, `tests/unit/issue_540_agent_cli.rs` and
`tests/unit/specification/dreaming_meta_algorithm.rs`; re-apply the `ARCHITECTURE.md`,
`ROADMAP.md` and `docs/meta-algorithm.md` paragraphs against today's structure. PR #887 is
then closed with a comment pointing at the carrying PR, so its history stays reachable.

## Architecture

### New module `src/behavior_delta.rs`

Names verified free (`BehaviorDelta`, `DeltaVerdict`, `AdoptionEffect`, `behavior_delta`
→ 0 hits each). Deliberately avoided: `AdoptionPair`/`AdoptionLedger`
(`src/learning_adoption_ledger.rs:38`, `:91`), `LearningRun`
(`src/self_improvement.rs:423`), `PromotionGate` (`src/skill_ledger.rs:119`),
`HeldOutTest` (`src/learning_cycle.rs:156`).

```rust
//! The #701 criterion, generalized: a learned item must demonstrably change the
//! next answer, and the change must be an observation rather than a claim.

use crate::execution_evidence::Evidence;   // plan 05, plan 00 §4.3
// **reconciled: was `Evidence`; now `Evidence`, the contract name plans
// 00, 03 and 06 already use (plan 00 §9 R2).**

/// What one learned item did to one prompt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeltaVerdict {
    /// The answer is byte-identical with and without the item: it changed nothing.
    Unchanged,
    /// The answer changed and the after side satisfies its expectation.
    Improved,
    /// The answer changed and the after side fails an expectation the before side met.
    Regressed,
    /// The answer changed and neither side has a checkable expectation. Honest,
    /// and never sufficient for adoption.
    ChangedUnverified,
}

impl DeltaVerdict {
    #[must_use] pub const fn slug(self) -> &'static str;
    /// Only `Improved` may count toward adoption.
    #[must_use] pub const fn supports_adoption(self) -> bool;
}

/// One before/after observation for one learned item on one held-out prompt.
///
/// Both sides are `Evidence`s, so "the answer changed" is a hash
/// comparison over observed bytes, not a prose judgement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BehaviorDelta {
    /// `stable_id("behavior_delta", &format!("{item_id}:{language}:{prompt}"))`.
    pub delta_id: String,
    /// The learned item under test, e.g. a `LearnedMethod::name`.
    pub item_id: String,
    /// Which learning pipeline produced it: `method`, `request_opener`,
    /// `program_rule`, `repair_lesson`, `amendment`, `anticipation`.
    pub item_kind: String,
    /// BCP-47 tag: `en`, `ru`, `hi`, `zh`, `es`.
    pub language: String,
    /// The held-out prompt, never one the item was inferred from.
    pub prompt: String,
    /// The answer observed with the item absent.
    pub before: Evidence,
    /// The answer observed with the item present.
    pub after: Evidence,
    pub verdict: DeltaVerdict,
}

impl BehaviorDelta {
    #[must_use] pub fn to_links_notation(&self) -> String;
}

/// Every delta proved for one learned item, and whether they are enough.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdoptionEffect {
    pub item_id: String,
    pub item_kind: String,
    pub deltas: Vec<BehaviorDelta>,
}

impl AdoptionEffect {
    /// The adoption contract, identical for every pipeline:
    /// at least one `Improved` delta in each of `en`, `ru`, `hi`, `zh`, `es`;
    /// zero `Regressed` deltas anywhere; every prompt held out from inference.
    #[must_use] pub fn qualifies(&self) -> bool;

    #[must_use] pub fn languages_covered(&self) -> Vec<&str>;
    #[must_use] pub fn regressions(&self) -> Vec<&BehaviorDelta>;
    #[must_use] pub fn to_links_notation(&self) -> String;
}

/// Prove the effect of one learned item by answering each held-out prompt twice:
/// once against a registry with the item removed, once with it present.
/// Deterministic: same seed data, same prompts, same deltas.
#[must_use]
pub fn prove_effect(
    item_id: &str,
    item_kind: &str,
    held_out: &[(&str, &str)],   // (language, prompt)
    with_item: &MethodRegistry,
    without_item: &MethodRegistry,
) -> AdoptionEffect;
```

`.lino` schema, appended to `data/meta/learning-adoption-ledger.lino` beside the existing
60 request-opener pairs so one ledger holds every pipeline's evidence:

```
behavior_delta_3f81c0a2
  record_type "behavior_delta"
  item_id "learned_recursive_core_e17957243eaaf6db"
  item_kind "method"
  language "es"
  prompt "¿Cuántas veces aparece la letra a en la palabra alfabeto?"
  before_record "execution_record_11aa22bb"
  after_record "execution_record_33cc44dd"
  verdict "improved"
adoption_effect_learned_recursive_core_e17957243eaaf6db
  record_type "adoption_effect"
  item_id "learned_recursive_core_e17957243eaaf6db"
  item_kind "method"
  language_covered "en"
  language_covered "ru"
  language_covered "hi"
  language_covered "zh"
  language_covered "es"
  improved "5"
  regressed "0"
  qualifies "true"
```

### How a learned item demonstrably changes the next answer, across the whole registry

The load-bearing observation: a `LearnedMethod`'s `operations` are already the recorder
event-kind vocabulary that `src/recipe_interpreter.rs:269-354` dispatches on. The one
adopted record in `data/seed/learned-methods.lino` lists `need:status`, `method_registry`,
`work_unit_reasoning`, `upward_construction`, `solution_evidence`, `selection`,
`skill_ledger`, `reasoning_standard` and their `:count`/`:steps` siblings — every one of
which is an event kind the interpreter's `run_recorder` already knows.

So no new interpreter is written. `src/method_registry.rs` gains the following. **Reconciled: plan 12 also grows
this struct, with a `heuristics` collection. The authoritative declaration is
plan 12's three-field `MethodRegistry { methods, learned_methods, heuristics }`;
this plan owns the execution of `learned_methods` and plan 12 owns `heuristics`,
and neither is ever returned by `method_for_route`. R344's one dispatch
authority survives because both live in one registry rather than beside it
(plan 00 §9 R16).**

```rust
impl LearnedMethod {
    /// Compile this learned abstraction into the same ordered recorder program
    /// the recursive-core recipe executes. `Err` names the first operation the
    /// interpreter does not bind, so an unrunnable learned record is reported,
    /// never silently skipped.
    pub fn to_recipe_program(&self) -> Result<crate::recipe_interpreter::RecipeProgram, String>;

    /// Whether every operation binds to a known recorder.
    #[must_use] pub fn is_executable(&self) -> bool;
}

impl MethodRegistry {
    /// Method names for these relevants, in precedence order, with adopted and
    /// executable learned methods appended **after** every compiled method.
    ///
    /// Learned methods are last by construction: a learned abstraction may add a
    /// capability the compiled table lacks, and may never pre-empt one it has.
    /// The chosen name is emitted as `method:learned` in the trace, so an answer
    /// that used a learned method says so.
    #[must_use]
    pub fn ordered_method_names_for_relevants(&self, relevants: &[String]) -> Vec<String>;
}
```

`ordered_method_names_for_relevants` keeps its name and signature — R331 and
`tests/unit/docs_requirements_issue_559.rs` pin both — and gains a fourth loop after the
existing three (`:239`, `:246`, `:255`) over `self.learned_methods` filtered by
`is_executable()`. `src/meta_method_dispatch.rs:39` needs no change at all.

`learned_method(&self, name)` (`:215`) gains its first production caller in
`meta_method_dispatch::try_dispatch`, which, on a learned name, runs
`to_recipe_program()?.execute(...)` instead of `handler_for_method`.

The same contract covers the other four pipelines, each keeping its own module and seed
file but sharing `AdoptionEffect`:

| Pipeline | Learned item | Read back at answer time by |
| --- | --- | --- |
| #922 method learning | `LearnedMethod` in `data/seed/learned-methods.lino` | `ordered_method_names_for_relevants` (new fourth loop) |
| #701 learning cycle | request-opener surfaces in `data/seed/learned-request-openers.lino` | already read at `src/solver_handlers/web_search_intent.rs:463` |
| #364 self-improvement | substitution rules in `data/seed/learned-program-rules.lino` (**to be created**) | `SubstitutionRuleSet`, the same path the checked-in rules already take |
| #558 learning ledger | `LedgerEntry` rows in a new `data/seed/approved-lessons.lino` | `approved_lesson_for` (`src/learning_ledger.rs:347`), unchanged, reading data instead of a constant |
| #540/#701 dreaming | `meta_algorithm_amendment` memory events | `dreaming_application::apply_retained_amendments`, extended to the engine surface |

### Where the human gate moves

Today the gate is *inertness*: `Off` by default, learned methods outside dispatch, a
one-element ledger, a branch nobody sees. After this plan the gate is the **draft pull
request**, in three enforced layers:

1. **Adoption requires proven effect.** `AdoptionEffect::qualifies()` — five languages,
   held-out prompts, zero regressions — is a precondition of writing a seed edit. A
   proposal without it is `Rejected` with its evidence preserved, the R425
   `dreaming_candidate_failure` pattern the promotion protocol already uses
   (`src/promotion.rs:622`, `:650`).
2. **Promotion replays the canonical gates.** Unchanged from #656:
   `src/promotion/gates.rs:102` executes the coding-modification, industry and unit-spec
   suites; proposal documents may not supply runners or counts (`src/promotion.rs:740-745`).
3. **The seed edit is published as a draft pull request.** `formal-ai improve --promote
   --apply --confirm` gains `--open-draft-pr`, which runs the commands
   `branch_plan()` currently only prints: `git add <seed files>`, `git commit`,
   `git push -u origin promotion/<run-id>`, `gh pr create --draft --base main --fill`.
   It never targets `main` directly, never merges, never marks the PR ready. Without the
   flag the behaviour is exactly today's.

```rust
/// Publish a materialized promotion as a draft pull request: the artifact a
/// human reviews reaches the place humans review artifacts.
///
/// Never merges, never marks ready, never pushes to the default branch. The
/// required GitHub checks run against the actual head SHA, and review remains
/// the outer gate (R472). Refuses when the run promoted nothing.
pub fn open_draft_pull_request(
    outcome: &PromotionApplyOutcome,
    workspace_root: &Path,
) -> io::Result<DraftPullRequest>;

/// The opened draft, recorded as an append-only `promotion_published` event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DraftPullRequest {
    pub branch: String,
    pub url: String,
    pub head_sha: String,
}
```

And the two places where inertness is removed, with the replacement gate named:

- `src/meta_self_improvement.rs:44-45` — `#[default] Off` becomes `#[default] Propose`.
  The mode was never a safety property: `propose_recipe_update` writes nothing either way
  (`:243-251`). Its doc comment changes from "the loop is dormant" to "the loop proposes;
  the seed edit it feeds is gated by promotion and reviewed as a pull request."
- `src/self_improvement.rs:248-252` — the hard-coded `issue_362_from_counts(0, 0)` is
  replaced by the real gate report from `promotion::gates::replay_promotion_gates`, so a
  proposal is judged against a benchmark that actually ran. Ingestion that cannot run a
  gate records `BenchmarkGateReport` as *absent* rather than as 0/0, and the proposal is
  `BlockedByBenchmark` with the reason `"no_gate_evidence"` instead of silently failing a
  floor it was never measured against.

### How source-cache reconstruction becomes automatic end to end (R710-R7)

New module `src/source_reconstruction.rs` (name free):

```rust
//! Executing the `rediscover:` edge that `dreaming::retention` authorizes (R710-R7).
//!
//! Forgetting a cache payload is already safe: `reconstruction_record` keeps the
//! URL, the provenance and the conversation, and drops only `content`. What was
//! missing is the other half — actually fetching it back when it is next needed,
//! and proving the recovered bytes are the same bytes.

/// Why a reconstruction was or was not performed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReconstructionOutcome {
    /// The payload was refetched and its hash matches the retained fingerprint.
    Recovered { record: Evidence },
    /// The payload was refetched and its hash differs: today's page is not the
    /// historical evidence. The stub is kept and the difference is reported.
    Diverged { retained_sha256: String, observed_sha256: String },
    /// Offline, or the edge names no reachable source.
    Unavailable { reason: String },
}

/// Find the `cache_reconstruction` stub for `event_id` and execute its edge.
pub fn reconstruct(
    events: &[MemoryEvent],
    event_id: &str,
    offline: bool,
) -> ReconstructionOutcome;

/// Reconstruct on demand: called when a lookup misses a cache entry that has a
/// reconstruction stub, so recovery is a property of the read path rather than a
/// maintenance command.
pub fn reconstruct_on_miss(
    events: &[MemoryEvent],
    tool: &str,
    offline: bool,
) -> Option<ReconstructionOutcome>;
```

`reconstruct` reuses `src/source_fetch.rs` for the fetch and its existing mismatch
vocabulary (`source_cache_url_mismatch:281`, `source_cache_content_hash_mismatch:298`) for
the `Diverged` case, and emits an `Evidence` (plan 05) so the recovery is itself an
observation. `Diverged` is deliberately not an error: R710-R7 already says "a public URL
authorizes reacquisition, not replacement of historical evidence with today's content."

The end-to-end proof is the forget-and-rediscover cycle the doctrine asks for: seed a cache
entry, dream under pressure until it is forgotten, confirm the payload is gone and the stub
remains, answer a prompt that needs it, and observe `Recovered` with a matching hash — no
human command in between.

### Failure and honesty behaviour

- A learned method whose operations do not all bind is `is_executable() == false`: it stays
  in the registry event as data, is never dispatched, and the unbound operation is named in
  the trace. Silent skipping is forbidden.
- `AdoptionEffect::qualifies()` false → the proposal is rejected with its deltas preserved,
  including which languages were missing.
- `DeltaVerdict::ChangedUnverified` never counts toward adoption, and the ledger reports how
  many there were.
- A regression found after adoption is a ratchet failure in CI, and the remedy is a revert
  commit on the seed file — an ordinary reviewed change, not a hidden kill switch.
- Reconstruction that `Diverged` reports both hashes and keeps the stub; it never
  substitutes today's bytes for the historical record.

## Tests first

### Held-out multilingual cases, with the actual prompt text

New file `tests/unit/issue_1138_learned_items_change_answers.rs`. The prompts are held out
from the inference corpus: they must be absent from `data/meta/learning-frontier-*.lino`
and from the two support traces the adopted method was learned from.

The class under test is the adopted recursive-core method's tail — a request that needs a
counted-scan answer, which the compiled table routes to `unknown` today:

- **en**: `"In the word alphabet, how many times does the letter a appear?"`
- **ru**: `"Сколько раз буква а встречается в слове алфавит?"`
- **hi**: `"शब्द वर्णमाला में अक्षर व कितनी बार आता है?"`
- **zh**: `"在「字母表」这个词里，字母表这两个字出现了几次？"`
- **es**: `"¿Cuántas veces aparece la letra a en la palabra alfabeto?"`

Held-out paraphrases of the same class, asserted to receive the same verdict:

- **en**: `"Count the occurrences of a inside alphabet and tell me the number."`
- **ru**: `"Посчитай, сколько букв а внутри слова алфавит, и назови число."`
- **hi**: `"वर्णमाला के अंदर व की गिनती करो और संख्या बताओ।"`
- **zh**: `"数一数「字母表」里面有几个「字」，把数目告诉我。"`
- **es**: `"Cuenta cuántas veces está la a dentro de alfabeto y dime el número."`

### Unit, integration and specification tests

| Path | Name | Asserts |
| --- | --- | --- |
| `tests/unit/specification/behavior_delta.rs` | `an_unchanged_answer_is_never_an_adoption` | `Unchanged` and `ChangedUnverified` fail `supports_adoption` |
| | `adoption_requires_all_five_languages` | `qualifies()` false when `es` is missing |
| | `one_regression_anywhere_blocks_adoption` | |
| | `a_delta_is_deterministic_across_runs` | same seeds → byte-identical `delta_id` |
| | `both_sides_of_a_delta_are_execution_records` | no prose judgement anywhere in the type |
| `tests/unit/specification/method_registry.rs` (extend) | `an_adopted_learned_method_is_dispatchable` | it appears in `ordered_method_names_for_relevants` |
| | `learned_methods_rank_after_every_compiled_method` | precedence is last, always |
| | `a_learned_method_with_an_unbound_operation_is_not_dispatched_and_is_named` | no silent skip |
| | `an_answer_that_used_a_learned_method_says_so_in_the_trace` | the `method:learned` event |
| `tests/unit/issue_1138_learned_items_change_answers.rs` | `the_adopted_method_changes_the_answer_to_a_held_out_prompt` | five languages, before ≠ after, after is `Improved` |
| | `a_held_out_paraphrase_gets_the_same_verdict` | five languages |
| | `removing_the_seed_record_restores_the_old_answer` | the delta is caused by the seed edit and nothing else |
| | `no_prompt_in_the_delta_set_appears_in_the_inference_corpus` | the anti-memorization check |
| `tests/unit/issue_1138_review_time_gate.rs` | `a_proposal_without_a_qualifying_effect_is_rejected_with_its_deltas` | |
| | `self_improvement_ingestion_records_absent_gate_evidence_not_a_zero_floor` | replaces the `issue_362_from_counts(0, 0)` behaviour |
| | `meta_self_improvement_proposes_by_default_and_still_writes_nothing` | the mode flip changes proposals, not writes |
| `tests/integration/issue_1138_draft_pull_request.rs` | `open_draft_pull_request_never_targets_the_default_branch` | |
| | `open_draft_pull_request_refuses_when_nothing_was_promoted` | |
| | `the_published_draft_is_recorded_as_an_append_only_event` | `promotion_published` |
| | `without_the_flag_the_protocol_behaves_exactly_as_before` | #656 regression |
| `tests/unit/specification/source_reconstruction.rs` | `a_forgotten_cache_payload_is_refetched_on_the_next_miss_without_a_human_command` | R710-R7 end to end |
| | `a_diverged_reconstruction_reports_both_hashes_and_keeps_the_stub` | |
| | `offline_reconstruction_reports_unavailable_and_never_synthesizes` | |
| | `reconstruction_emits_an_execution_record` | plan 05 join |
| `tests/unit/issue_705_anticipation.rs` (carried from PR #887) | all 15 existing cases | unchanged, re-registered in `tests/unit/mod.rs` |
| `tests/unit/docs_requirements_issue_705.rs` (carried) | all cases | re-pointed at the new shard instead of `REQUIREMENTS.md` |
| `tests/unit/docs_requirements/issue_1138.rs` (extend; **reconciled: was `tests/unit/docs_requirements_issue_1138.rs` — plan 00 §9 R11**) | `issue_1138_behavior_delta_is_traceable` | grep-pins `pub struct AdoptionEffect`, `fn prove_effect`, `fn open_draft_pull_request` |

### Gates and ratchets

- **New ratchet** `data/meta/adoption-effect-ratchet.lino`:
  `adopted_items_with_qualifying_effect` must equal `adopted_items` — every adopted learned
  item in every seed file must have a qualifying `AdoptionEffect` in
  `data/meta/learning-adoption-ledger.lino`. The floor is total and never lowered.
- **Strictly-downward ratchet** on `learned_items_never_read_back`: the count of adopted
  learned items with zero production read path, currently at least 1
  (`learned_recursive_core_e17957243eaaf6db`). It must reach 0 in this PR and stay there.
- The existing #701 ratchet (`the_trends_corpus_unknown_rate_is_ratcheted_to_zero`,
  `docs/requirements/issue-0701-auto-learning-adoption-gap.md:15`) is untouched.
- The #656 gates (`src/promotion/gates.rs:121` `required_gates()`) are untouched; the draft
  PR is strictly downstream of them.
- `cargo run --example regenerate_self_ast_census` after every `src/` change, including the
  cherry-picked `src/anticipation*` files.
- `rust-script scripts/check-file-size.rs`: `src/anticipation.rs` arrives at 934 lines
  against a 1,000-line Rust ceiling (`scripts/check-file-size.rs:24`). Any fix applied
  during re-application must split the file rather than approach the limit.

## Implementation leaves

- [x] Land plan 05's `src/execution_evidence.rs` first; `BehaviorDelta` depends on
      `Evidence`.
- [x] Add `src/behavior_delta.rs` with `DeltaVerdict`, `BehaviorDelta`, `AdoptionEffect`,
      `prove_effect`, `to_links_notation` for all three; register in `src/lib.rs`.
- [ ] Extend `data/meta/learning-adoption-ledger.lino` with the `behavior_delta` and
      `adoption_effect` record shapes; keep the 60 existing #701 pairs byte-identical.
- [x] Add `LearnedMethod::to_recipe_program` and `is_executable` in
      `src/method_registry.rs`, reusing `recipe_interpreter::RecipeProgram`.
- [x] Add the fourth loop to `ordered_method_names_for_relevants` (learned methods last)
      plus the `method:learned` trace event; keep the signature and name unchanged.
- [x] Teach `meta_method_dispatch::try_dispatch` to execute a learned name through
      `to_recipe_program()`; the first production caller of `learned_method`.
- [ ] ~~Prove and record the `AdoptionEffect` for `learned_recursive_core_e17957243eaaf6db`
      in five languages; if it does not qualify, record that honestly and mark the record
      `status "adopted_not_effective"` rather than forcing a delta.~~ **Struck 2026-09-16.**
      The effect was measured, in all five languages, with leaf 6 landed: the selection
      changes in every one of them (`answer_changed` is true five times over) and the
      verdict is `ChangedUnverified` five times over, because the adopted record's
      operations are the meta-core recorders and the held-out class is a counted scan.
      Nothing the record executes can produce a count, so no expectation the after side
      satisfies exists without inventing one — and inventing one is exactly the forced
      delta this leaf forbids. The leaf's own escape hatch,
      `status "adopted_not_effective"`, is not available either: `parse_learned_methods`
      admits only `adopted`, and the wave T test
      `specification::method_registry::an_adopted_learned_method_is_dispatchable`
      requires the shipped record to stay dispatchable, so writing the status would turn
      one red test into two. `issue_1138_learned_items_change_answers::the_adopted_method_changes_the_answer_to_a_held_out_prompt`
      therefore stays red, un-ignored and un-weakened, until a learned record exists whose
      operations answer the class it is measured on.
- [ ] Create `data/seed/learned-program-rules.lino` (empty with a schema header) so #364's
      destination exists; ground it.
- [x] Replace `issue_362_from_counts(0, 0)` at `src/self_improvement.rs:248-252` with
      absent-gate-evidence semantics; add `"no_gate_evidence"` as a rejection reason.
- [ ] Move `canonical_ledger()` (`src/learning_ledger.rs:311`) onto a new
      `data/seed/approved-lessons.lino` containing today's single entry byte-for-byte, so
      behaviour is unchanged and a second entry becomes a data edit.
- [x] Flip `SelfImprovementMode` default from `Off` to `Propose`
      (`src/meta_self_improvement.rs:44-45`); update the doc comment and R340's shard text.
      **This leaf lands alone, after plan 05's recipe step 14, and carries its own
      R343 parity run: both change every recorded trace, and sharing a commit
      would leave a parity failure with two possible causes (plan 00 §9 X11).**
- [ ] Pass `memory_events` through `FormalAiEngine::answer` so
      `dreaming_application::apply_retained_amendments` reaches the engine surface, not only
      `src/protocol.rs`.
- [x] Add `open_draft_pull_request` and `DraftPullRequest` to `src/promotion.rs`; add the
      `promotion_published` event kind; add `--open-draft-pr` to `src/cli_improve.rs`.
- [ ] Make `dreaming_runtime::write_learning_cycle_record` (`:133-145`) readable: either
      wire its record into the next promotion run's proposal input, or delete it. Decide
      in the leaf; do not leave a file nobody reads.
- [x] Add `src/source_reconstruction.rs` with `ReconstructionOutcome`, `reconstruct`,
      `reconstruct_on_miss`; wire `reconstruct_on_miss` into the source-cache read path.
- [ ] Add `data/meta/adoption-effect-ratchet.lino` and the two ratchet tests.
- [ ] **PR #887 carry:** create the branch, cherry-pick the 24 new-path files, re-add the
      three registry lines, write
      `docs/requirements/issue-0705-anticipatory-dreaming.md`, regenerate
      `REQUIREMENTS.md` and `data/meta/self-ast/`, re-apply the six semantic edits, re-apply
      the three doc paragraphs, run the full suite, then close #887 with a pointer.
- [ ] Add `tests/unit/issue_1138_learned_items_change_answers.rs`,
      `tests/unit/issue_1138_review_time_gate.rs`,
      `tests/integration/issue_1138_draft_pull_request.rs`,
      `tests/unit/specification/behavior_delta.rs`,
      `tests/unit/specification/source_reconstruction.rs`; register each.
- [ ] Add the `changelog.d/` fragment and the traceability rows.

## Docs to update

The exact quoted statements and their replacement text moved to plan 11's
findings table on 2026-09-16, so there is one docs authority and no document
is described in two places (plan 00 §8). This plan's entries are rows
**D210-D225** of
[`11-docs-consistency-audit.md`](11-docs-consistency-audit.md) §"Issue #1138
plan doc replacements", and plan 11's leaves apply them after the ledger rows
they cite exist (plan 00 §7).

| row | document |
| --- | --- |
| D210 | `docs/meta-algorithm.md:772-775` |
| D211 | `docs/meta-algorithm.md:761-764` |
| D212 | `docs/meta-algorithm.md:671-678` |
| D213 | `docs/meta-algorithm.md:137-142` |
| D214 | `docs/requirements/issue-0922-method-learning-from-experience.md:13` |
| D215 | `docs/requirements/issue-0922-method-learning-from-experience.md:16` |
| D216 | `docs/requirements/issue-0559-general-meta-algorithm.md:39` |
| D217 | `docs/requirements/issue-0710-repository-and-retention-continuation.md:16` |
| D218 | `docs/requirements/issue-0656-benchmark-gated-promotion-protocol.md:26` |
| D219 | `ROADMAP.md:369` and `:428` |
| D220 | `ROADMAP.md:440` |
| D221 | `ROADMAP.md:364` and `:423` |
| D222 | `VISION.md:377-382` |
| D223 | New shard `docs/requirements/issue-0705-anticipatory-dreaming.md` |
| D224 | `docs/requirements/issue-1138-bottleneck-audit.md` |
| D225 | `docs/requirements-traceability.md` |

Any further document this plan's implementation touches is added as a new
plan 11 row, never as a second copy here.

## Risks and open questions

1. **Executing a learned method can change answers, which is the point and the danger.**
   Mitigations: last precedence, a five-language proved effect, zero-regression adoption,
   and a ratchet. Residual risk: a learned method that improves five held-out prompts and
   regresses a sixth nobody tested. The honest answer is that the corpus is finite and the
   ratchet is what catches it, not the adoption proof.
2. **Does the one adopted method actually qualify?** `learned_recursive_core_e17957243eaaf6db`
   is a twelve-to-fifteen-operation recursive-core *tail* — a trace shape, not obviously a
   capability. It may well produce `Unchanged` on every held-out prompt. If so, the honest
   outcome is `status "adopted_not_effective"` in the seed and a B7 that is *architecturally*
   closed with zero effective items — which is a real result and must be reported as one,
   not papered over by inventing a prompt the method happens to change.
3. **`--open-draft-pr` performs a network action from a learning loop.** It requires
   `--confirm`, refuses the default branch, and never marks ready — but it is still the
   first time this repository lets an automated path create a PR. Open question for review:
   should it additionally require an explicit `FORMAL_AI_ALLOW_PUBLISH` environment opt-in,
   the way `agent_mode` guards tool use?
4. **Flipping `SelfImprovementMode` to `Propose` changes a default.** It writes nothing
   (`src/meta_self_improvement.rs:243-251`), but it does add events to every trace, which
   `tests/unit/specification/meta_self_improvement.rs` and R343 parity will notice. The
   flip may need to be a separate commit with its own parity run.
5. **PR #887's `src/anticipation.rs` is 934 lines against a 1,000-line ceiling.** Six weeks
   of drift will require edits during re-application. If the file crosses 1,000 it must be
   split along the `expansion`/`ledger` seam that already exists, not granted an exception.
6. **Whether `write_learning_cycle_record` should be wired or deleted is left open in a
   leaf.** Writing a file nobody reads is exactly the pattern this plan exists to remove;
   leaving the decision to implementation time is a small deferral and is flagged as such.
7. **`FormalAiEngine::answer` gaining `memory_events` changes a public signature.** Either
   an overload (`answer_with_memory`) or a builder field. The overload duplicates; the field
   changes construction sites. Not yet decided.
8. **Five languages, not four.** Every existing adoption artifact is en/ru/hi/zh
   (R701-2, R552, R556). Adding `es` to the adoption contract means the existing 60 pairs do
   not satisfy the new contract. They are **not** retrofitted: the ledger records that the
   #701 class was proved in four languages on its date, and the new contract applies to
   items adopted from here on. Claiming five-language coverage for a four-language
   measurement would be exactly the dishonesty this issue exists to remove.
