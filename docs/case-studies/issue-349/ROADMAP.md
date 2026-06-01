# Roadmap — issue #349 fix, as a GitHub-issue dependency DAG

This roadmap operationalises the [case study](./README.md) into GitHub issues. Per
[#349](https://github.com/link-assistant/formal-ai/issues/349): *"I want to have full plan as issues on
GitHub … where each issue clearly blocked via GitHub API by other issues that it depends on. Each issue plan
should be detailed as possible, so even the weakest AI systems can implement them … Each issue must contain
reproducible examples, workarounds and suggestions for fix the issue in code."*

Dependencies are wired with the GitHub **issue dependencies API** (GA 2025):

```sh
# issue_id is the GLOBAL DATABASE id, not the #number:
blocker_id=$(gh api repos/link-assistant/formal-ai/issues/<BLOCKER_NUMBER> --jq .id)
gh api -X POST repos/link-assistant/formal-ai/issues/<BLOCKED_NUMBER>/dependencies/blocked_by \
  -f issue_id="$blocker_id"
# verify:
gh api repos/link-assistant/formal-ai/issues/<BLOCKED_NUMBER>/dependencies/blocked_by --jq '.[].number'
```

## The DAG

```
        R1: #1 Reproduce + lock (failing test + diagnostics fixture)
             |        \            \
             v         v            v
   R4: #6 Diagnostics  R2: #2 Design doc   R2/D: #3 Coreference / follow-up rewrite
             |              \         \        /
             |               v         v      v
             |        R2/C: #4 General modification model (kills the allowlist)
             |               |          \
             |               v           \
             |        R2/A: #5 Reasoned rule construction on unknown
             |          /        |          \
             v         v         v           v
   R6: #8 Bulk dataset   R5: #7 Cross-runtime parity   R7: #9 Reduce "Report issue"
             \              |                 /
              v             v                v
            R8: #10 Self-improvement loop (white-box)
                          |
                          v
                   #11 Epic / tracking (blocked by #1–#10)
```

## Dependency table

| Issue | Title | Blocked by |
|---|---|---|
| #1 | Reproduce & lock issue #349 with a failing test + diagnostics fixture | — |
| #2 | Design: general rule-synthesis over links notation (reason, don't memorise) | #1 |
| #3 | Coreference: bind bare-imperative follow-ups to the active program artifact | #1 |
| #4 | General program-modification model (remove the hard-coded `PROGRAM_MODIFIERS` allowlist) | #2, #3 |
| #5 | Reasoned rule construction on unknown (construct a candidate rule; ≤1 clarifying question) | #2, #4 |
| #6 | Diagnostics / verbose mode for every reasoning step (default off) | #1 |
| #7 | Cross-runtime parity (Rust + JS worker + wasm worker + seed) | #4, #5 |
| #8 | Bulk multilingual multi-turn coding-modification benchmark + ratchet | #5, #6, #7 |
| #9 | Reduce "Report issue" pressure — reasoning-first, report as last resort | #5, #7 |
| #10 | Self-improvement: learn rules from accumulated unknown-traces (white-box) | #8, #9 |
| #11 | Epic / tracking issue for #349 | #1–#10 |

## Created issues (logical id → GitHub issue)

These issues were created on GitHub and wired with `blocked_by` edges via the dependencies API
(see `experiments/create_issue_349_roadmap.py` and `experiments/issue-349-issue-map.json`). The
**logical** ids above map to the **actual** GitHub issue numbers below; the epic is
[#365](https://github.com/link-assistant/formal-ai/issues/365).

> **Status: all CLOSED.** The entire roadmap was implemented and merged into `main`; the originally-reported
> failure no longer reproduces (case study [§0](./README.md#0-resolution-status)). Regression lock:
> `tests/integration/issue_349_reverse_sort.rs`.

| Logical | GitHub | Title | Status |
|---|---|---|---|
| #1 | [#355](https://github.com/link-assistant/formal-ai/issues/355) | Reproduce & lock issue #349 with a failing test + diagnostics fixture | ✅ closed |
| #2 | [#356](https://github.com/link-assistant/formal-ai/issues/356) | Design: general rule-synthesis over links notation | ✅ closed |
| #3 | [#357](https://github.com/link-assistant/formal-ai/issues/357) | Coreference: bind bare-imperative follow-ups to the active program artifact | ✅ closed |
| #4 | [#358](https://github.com/link-assistant/formal-ai/issues/358) | General program-modification model (remove the hard-coded allowlist) | ✅ closed |
| #5 | [#359](https://github.com/link-assistant/formal-ai/issues/359) | Reasoned rule construction on unknown | ✅ closed |
| #6 | [#360](https://github.com/link-assistant/formal-ai/issues/360) | Diagnostics / verbose mode for every reasoning step | ✅ closed |
| #7 | [#361](https://github.com/link-assistant/formal-ai/issues/361) | Cross-runtime parity (Rust + JS worker + wasm worker + seed) | ✅ closed |
| #8 | [#362](https://github.com/link-assistant/formal-ai/issues/362) | Bulk multilingual multi-turn coding-modification benchmark + ratchet | ✅ closed |
| #9 | [#363](https://github.com/link-assistant/formal-ai/issues/363) | Reduce "Report issue" pressure | ✅ closed |
| #10 | [#364](https://github.com/link-assistant/formal-ai/issues/364) | Self-improvement: learn rules from accumulated unknown-traces | ✅ closed |
| #11 | [#365](https://github.com/link-assistant/formal-ai/issues/365) | Epic / tracking issue for #349 | ✅ closed |

---

Below, each issue is written ready to paste into `gh issue create --title … --body …`. Every body carries:
**Context**, **Reproducible example**, **Root cause**, **Suggested fix (file:line)**, **Acceptance
criteria**, **Workaround**, and a back-link to this case study. Line numbers refer to the codebase at
`Cargo.toml` version 0.156.0 and may drift; the *symbols* are the durable anchor.

## #1 — Reproduce & lock issue #349 with a failing test + diagnostics fixture

**Context.** The 5-turn Russian dialog in #349 answers turn 5 ("Сделай сортировку результатов в обратном
порядке") with `intent: unknown`. Before any fix we need a committed, failing, automated reproduction so the
fix is verifiable and cannot silently regress.

**Reproducible example.** `docs/case-studies/issue-349/raw-data/repro_issue_349.rs` already replays the dialog
via `formal_ai::{solve_with_history, ConversationTurn}`. Captured output:
`docs/case-studies/issue-349/raw-data/reproduction-output.txt` (turn 3 → `write_program`/1.00; turn 5 →
`unknown`/0.00).

**Root cause.** See case study §3 (root causes A–D).

**Suggested fix.**
- Add `tests/issue_349_reverse_sort.rs` (integration test) that builds the 5-turn history and asserts turn 5
  is **not** `unknown` and that its answer reverses the sort. Mark `#[ignore = "tracks #349 until #4/#5 land"]`
  so CI stays green, *or* assert the current `unknown` and flip the assertion in the fixing PR — pick one and
  document it in the test.
- Promote `repro_issue_349.rs` into `examples/` (only in the fixing PR, to keep this PR docs-only).
- Capture a diagnostics fixture (golden file) once #6 exists.

**Acceptance criteria.** A test exists that encodes turn-5-must-not-be-unknown; it fails today (or is
`#[ignore]`d with a comment pointing at #4/#5); `cargo test` references it.

**Workaround (for users, until fixed).** Phrase the follow-up with a program-noun, e.g. "Измени программу,
чтобы она сортировала результаты в обратном порядке" so it re-routes to `write_program`.

_Part of the #349 roadmap — see `docs/case-studies/issue-349/`._

## #2 — Design: general rule-synthesis over links notation (reason, don't memorise)

**Context.** #349 demands *actual reasoning* over the links-notation meta-language, not memorised rules. Today
a modification only works if someone pre-wrote an allowlist entry + a seed substitution + a catalog task. The
issue calls this "still fake."

**Root cause.** Case study §3 root causes A & C: routing is literal-match-only with no reasoning fallback
(`src/intent_formalization.rs:296-307,340-367`); modifications are a one-entry allowlist
(`src/intent_formalization.rs:506-520`) + one seed rule (`data/seed/program-plan-rules.lino`).

**Deliverable.** A design doc (`docs/design/rule-synthesis.md`) specifying:
1. How a bare imperative is decomposed into `(operation, target)` over links notation.
2. How a candidate substitution rule is *constructed* from `data/seed/operation-vocabulary.lino` primitives
   when no seed rule matches (white-box, verifiable).
3. How the constructed rule is verified (TDD test) before being offered, and optionally persisted.
4. The interaction with coreference (#3) and the modification model (#4).

**Prior art to evaluate** (case study §5.4): Neuro-Symbolic Program Synthesis
(<https://arxiv.org/pdf/1611.01855>), Learning Compositional Rules via Neural Program Synthesis
(<https://arxiv.org/pdf/2003.05562>), Proof of Thought (<https://arxiv.org/html/2409.17270v2>).

**Acceptance criteria.** Doc merged; #4 and #5 reference it; it explicitly states what is kept (the symbolic
substitution engine) vs. replaced (the allowlist).

_Part of the #349 roadmap — see `docs/case-studies/issue-349/`._

## #3 — Coreference: bind bare-imperative follow-ups to the active program artifact

**Context.** "Сделай сортировку результатов…" has no program-noun, so it never routes to `write_program` and
recovery never runs. A human binds "результатов"/"the results" to the program from prior turns.

**Root cause.** Case study §3 root causes B & D: `recover_write_program_rule`
(`src/intent_formalization.rs:420-493`) only fires when the current message already routes to
`write_program`; `software_project_followup.rs` (#341) uses a narrow hard-coded marker list;
`data/seed/coreference.lino` binds only "it"→Rust.

**Reproducible example.** `repro_issue_349.rs` turn 5 (no program-noun) vs turn 3 (kept alive by the
parameter extractor).

**Prior art** (case study §5.3): CREAD — Combined Resolution of Ellipses and Anaphora in Dialogues
(<https://arxiv.org/abs/2105.09914>, code <https://github.com/apple/ml-cread>), MuDoCo, RiSAWOZ. Model the
follow-up as **query rewriting against history**: rewrite "sort the results in reverse" → "modify the
file-listing Rust program to sort its output in reverse," then lower.

**Suggested fix.**
- Add a coreference/rewrite step that, when the active conversation has a program artifact and the current
  message is a bare imperative, binds the referent and re-enters `write_program` recovery.
- Extend `data/seed/coreference.lino` beyond the single "it"→Rust antecedent to cover "результаты/results",
  "программа/program", and the language set (en/ru/hi/zh).

**Acceptance criteria.** Turn 5 routes to a program-modification path with the prior program as the bound
target; covered by a test; works in all four languages.

**Workaround.** Same as #1.

_Part of the #349 roadmap — see `docs/case-studies/issue-349/`._

## #4 — General program-modification model (remove the hard-coded `PROGRAM_MODIFIERS` allowlist)

**Context.** Reverse-sort is unrepresentable: there is no `reverse_sort` modifier and no substitution rule.
The allowlist is the architectural defect #349 targets.

**Root cause.** Case study §3 root cause C: `PROGRAM_MODIFIERS`
(`src/intent_formalization.rs:506-520`) has one entry; `data/seed/program-plan-rules.lino` has one rule.

**Reproducible example.** `repro_issue_349.rs` turn 5.

**Suggested fix.**
- Replace the hard-coded `PROGRAM_MODIFIERS` slice with a data-driven, composable modifier model sourced from
  `data/seed/operation-vocabulary.lino` (which already has `sort_lines`, `reverse_words`, …).
- Add a `reverse_sort` operation + a substitution rule (e.g. `names.sort()` → `names.sort(); names.reverse()`
  or `sort_by(|a,b| b.cmp(a))`) as the **first general transform**, demonstrating the model generalises.
- Keep lowering through `src/program_plan.rs`; modifiers must compose (path_argument **and** reverse_sort).

**Acceptance criteria.** Reverse-sort works via the general model (no bespoke allowlist entry); ≥2 modifiers
compose; `repro_issue_349.rs` turn 5 emits a program whose output is reverse-sorted; covered by tests in the
four languages.

**Workaround.** Same as #1.

_Part of the #349 roadmap — see `docs/case-studies/issue-349/`._

## #5 — Reasoned rule construction on unknown (construct a candidate rule; ≤1 clarifying question)

**Context.** When no rule matches, the system must *try to reason a rule*, as a human would — not bail to
`unknown`. Today every non-matching prompt becomes `SelectedRule::Unknown`.

**Root cause.** Case study §3 root cause A: `select_rule_for_intent`
(`src/intent_formalization.rs:296-307`) maps everything unmatched to `Unknown`; there is no construction step.

**Suggested fix.**
- In the solver's unknown path (`src/solver.rs:591-617`), before emitting an unknown answer, attempt
  rule construction per the #2 design: decompose into `(operation, target)`, propose a candidate substitution
  from the operation vocabulary, verify it with a TDD test, and if it passes, answer with the modified program.
- If ambiguous, ask **at most one** clarifying question instead of `unknown`.
- Never emit a bare "unknown" for a resolvable modification.

**Acceptance criteria.** For the reverse-sort class, the unknown path is not reached; a constructed rule is
verified before use; at most one clarifying question; covered by tests. White-box: the construction is
inspectable via #6 diagnostics.

**Workaround.** Same as #1.

_Part of the #349 roadmap — see `docs/case-studies/issue-349/`._

## #6 — Diagnostics / verbose mode for every reasoning step (default off)

**Context.** #349 requires a diagnostics mode showing *every* reasoning step, and: "If there is not enough
data to find the actual root cause, add debug output and verbose mode if not present." White-box, off by
default.

**Root cause / current state.** `SolverConfig::diagnostic_mode` and `try_diagnostic`
(`src/solver.rs:788-825`) exist but do not trace the new steps (routing attempts, coreference binding,
modifier detection, rule construction, verification).

**Suggested fix.**
- Extend `diagnostic_mode` to emit each step as inspectable links-notation trace; keep default `false`.
- Surface in the web UI's existing Diagnostics toggle (it was *off* in the #349 repro).
- Add a golden diagnostics fixture for the #349 dialog (ties into #1).

**Acceptance criteria.** With diagnostics on, the full reasoning chain for turn 5 is emitted and asserted by a
golden test; default-off behaviour unchanged.

_Part of the #349 roadmap — see `docs/case-studies/issue-349/`._

## #7 — Cross-runtime parity (Rust core + JS worker + wasm worker + seed)

**Context.** #349 requires fixing the defect *in all places*. The answer users actually saw came from the
web runtime, and there are several distinct unknown-answer exits.

**Root cause.** Case study §3 "fix everywhere": six unknown-exit sites (`solver.rs:591-617`,
`solver_unknown_reasoning.rs:124-140`, `unknown_opener.rs:54-63`, `solver_handlers/user_intent.rs:96-114`,
`engine.rs:38`, `data/seed/multilingual-responses.lino`) **plus** runtime mirrors `src/web/app.js:28`,
`src/web/formal_ai_worker.js`, `src/web/wasm-worker/src/lib.rs`, `src/web/seed_loader.js`.

**Suggested fix.** One behaviour spec; reconcile every unknown-exit; ensure the Rust core, the JS worker, the
wasm worker, and the seed produce identical answers for the #349 dialog. Use the existing parity checks
(`check:language-parity`, `check:intent-coverage`).

**Acceptance criteria.** The reverse-sort fix is observable in all runtimes; parity checks pass; no unknown
exit answers the reverse-sort class in any runtime.

_Part of the #349 roadmap — see `docs/case-studies/issue-349/`._

## #8 — Bulk multilingual multi-turn coding-modification benchmark + ratchet

**Context.** #349 asks for bulk test suites driven by large, legally-usable open datasets, downloaded only at
test time.

**Suggested fix.**
- Build a multi-turn coding-modification benchmark (initial-draft → edit → edit) across en/ru/hi/zh.
- Integrate open datasets **download-on-test** (case study §5.2): CanItEdit
  (<https://github.com/nuprl/CanItEdit>), HumanEvalFix, EDIT-Bench (<https://arxiv.org/pdf/2511.04486>); read
  the audit "Edit, But Verify" (<https://arxiv.org/html/2604.05100>) before trusting any as a ratchet; verify
  licenses at integration time.
- Add a ratchet so the pass-rate cannot regress.

**Acceptance criteria.** A CI-runnable (possibly nightly) bulk suite exists; datasets are fetched at test
time, never committed; a ratchet guards regressions; the #349 dialog is in the suite.

_Part of the #349 roadmap — see `docs/case-studies/issue-349/`._

## #9 — Reduce "Report issue" pressure — reasoning-first, report as last resort

**Context.** #349 wants less pressure toward "Report issue"; the live answer pushes the user to report. The
prefilled report URL is built at `src/web/app.js:3571`.

**Suggested fix.** Make reasoning-first (#5) the default; only when reasoning genuinely fails, offer "Report
issue" pre-filled with the reasoning trace (#6) so human triage is cheap. Soften the multilingual fallback
copy (`data/seed/multilingual-responses.lino`).

**Acceptance criteria.** For resolvable modifications, "Report issue" is never the primary response; when it
does appear, it carries the reasoning trace; covered by tests across runtimes.

_Part of the #349 roadmap — see `docs/case-studies/issue-349/`._

## #10 — Self-improvement: learn rules from accumulated unknown-traces (white-box)

**Context.** #349's long-term goal: when the system can code itself, it should learn from accumulated
unknown-traces — white-box, not black-box.

**Suggested fix.** Accumulate unknown-traces (from #6); periodically attempt to synthesise new seed rules
(#5) from them; gate every learned rule behind the #8 benchmark so it cannot regress; keep every learned rule
inspectable as links-notation.

**Acceptance criteria.** A documented, gated loop that proposes learned rules, each verified against the
benchmark before adoption; learned rules are human-readable; no regression.

_Part of the #349 roadmap — see `docs/case-studies/issue-349/`._

## #11 — Epic / tracking issue for #349

**Context.** Umbrella tracking the #349 program of work. Blocked by #1–#10.

**Body.** Links the case study (`docs/case-studies/issue-349/`), the DAG above, and each child issue with its
requirement mapping (R1–R12). Closes when #1–#10 are done and the #349 dialog is green across all runtimes and
in the bulk suite.

_Part of the #349 roadmap — see `docs/case-studies/issue-349/`._
