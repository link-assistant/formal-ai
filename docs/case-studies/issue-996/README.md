# Issue #996 — A step to Formal AI being able to code itself, via Hive Mind

Issue: <https://github.com/link-assistant/formal-ai/issues/996>
Pull request: <https://github.com/link-assistant/formal-ai/pull/997>
Hive Mind companion issue: <https://github.com/link-assistant/hive-mind/issues/2146>

## Collected evidence

`raw-data/` preserves everything the analysis below is built on:

| File | Origin | Content |
| --- | --- | --- |
| `raw-data/hive-mind-log-XOq3KX-sanitized.log.txt` | gist `konard/d0553b8e1b5ed88f1b8241f539ba4907` | Hive Mind `solve` session where `--model formal-ai --tool agent` was requested |
| `raw-data/hive-mind-log-PhU3X2-sanitized.log.txt` | gist `konard/3df8fd313b592842d9dcf34bd265d7ca` | Session that produced no code changes and triggered auto-restart/resume |
| `raw-data/hive-mind-log-8GWnaH-sanitized.log.txt` | gist `konard/111bc2c200954b7f524e5f314e49f43e` | Second session with the same no-code-changes problem |
| `raw-data/github/issue-996.json` | GitHub API | The formal-ai issue body and metadata |
| `raw-data/github/hive-mind-issue-2146.json` | GitHub API | The Hive Mind companion issue |
| `raw-data/github/hive-mind-issue-2146-comments.json` | GitHub API | Its comment thread (the Hive Mind/Formal AI responsibility split) |
| `raw-data/github/open-issues.json` | GitHub API | Open formal-ai issues at analysis time (2026-08-10) |
| `raw-data/github/open-prs.json` | GitHub API | Open formal-ai pull requests at analysis time |

## Requirements extracted from the issue

The issue text decomposes into these requirements. The Hive Mind companion
issue #2146 contributes three Formal-AI-side sub-requirements (R1a–R1c),
because #996 explicitly names it as "one of the issues we are working on on
Hive Mind side".

| ID | Requirement | Status in PR #997 |
| --- | --- | --- |
| R1 | Advance the Formal-AI side of the Hive Mind self-coding loop (#2146) | see R1a–R1c |
| R1a | The driving model for a `formal-ai` run must be Formal AI only; the Agent CLI must not silently substitute another LLM | analyzed below; fixed upstream fail-closed in agent#293/PR #294 (`js-0.25.8`), attestation follow-up agent#295 |
| R1b | A Formal-AI-driven coding run must make it from start to end with real code changes — no empty runs that trigger Hive Mind auto-restart/resume loops | analyzed below |
| R1c | Formal AI final messages must wrap tab/space-indented text and code in fenced codeblocks so GitHub comments render correctly | fixed in this PR with tests |
| R2 | Take the most critical open formal-ai issues and close them in this single PR | fixed set listed below |
| R3 | Skip issues already executing in open PRs (#991, #990, #988, #710, #705, #651, #447, #483, #557 at analysis time) | respected |
| R4 | Architectural changes are welcome when they generalize logic (never specialize) | respected in each fix |
| R5 | Keep improving the associative technological stack: workarounds here must be paired with tracking issues; report upstream issues where separation of concerns benefits | documented below |
| R6 | Propose entirely new libraries/components if any look missing | proposals below |
| R7 | Compile all related data into `docs/case-studies/issue-996` | this folder |
| R8 | Deep case-study analysis, including online research | this document |
| R9 | List each and all requirements from the issue | this table |
| R10 | Propose solutions and solution plans for each requirement, checking existing components/libraries | per-requirement sections below |
| R11 | Plan and execute everything in this single pull request | PR #997 |

## Session-log analysis (R1, R1a, R1b, R1c)

### Defect A — the Agent CLI silently substitutes another model (R1a)

`raw-data/hive-mind-log-XOq3KX-sanitized.log.txt` shows the run being started
correctly (line 322):

```
agent --model formalai/formal-ai --verbose
```

and then failing open. Lines 449–476 log, verbatim:

> `CRITICAL: --model flag detected in process.argv but both
> getModelFromProcessArgv() and yargs returned default. This is likely a
> Bun/yargs argument parsing bug (oven-sh/bun#22157). The requested model
> will NOT be used — the default model will be used instead.`

Line 485 confirms the substitution: `"modelID": "minimax-m2.5-free"`. The
whole session — advertised as a Formal-AI-driven run — was actually driven
by a free neural model; the Formal AI server never received the traffic.

This is not fixable inside formal-ai: the defect is in the external
`@link-assistant/agent` CLI, which detects the parsing failure, prints
CRITICAL, and then **continues with the wrong model** instead of aborting.
The generalizing fix is fail-fast: an explicitly requested model that cannot
be honored must terminate the run with a non-zero exit code. See "Upstream
reports" below for the report filed against `link-assistant/agent`.

### Defect B — runs that end with zero code changes (R1b)

`raw-data/hive-mind-log-PhU3X2-sanitized.log.txt` line 723 shows the
assistant (this time genuinely `"model": "formal-ai"`) answering a coding
task with conversational filler:

> `Let me run a quick command to get that for you.`

— no tool call follows, the run produces no code changes, and lines
1068–1091 show Hive Mind reacting with its auto-restart/resume loop.
`hive-mind-log-8GWnaH-sanitized.log.txt` records a second session with the
same shape.

Root cause on the Formal-AI side: a received `--task` that did not match a
recipe fell through to chat-style canned responses, and there was no
visibility into how the task was routed. Issue #956 is the concrete
instance — a custom formalization task had its quoted source text silently
discarded in favour of the seeded fairy tale. This PR fixes both legs:

- the formalization recipe honors inline-quoted source text
  (`src/agentic_coding/formalization_recipe.rs`), and
- `FORMAL_AI_TRACE_REQUESTS=1` now traces `agentic_task` and
  `formalization_source` routing decisions, so an empty run can be
  diagnosed from its log instead of guessed at.

The restart-budget half of the loop (don't resume a run that changed
nothing without bounding retries) is Hive-Mind-side work, tracked as R-B3
in hive-mind #2146.

### Defect C — final-message markdown collapses in GitHub comments (R1c)

The logs show `lino` knowledge-base text and plan events emitted as bare
indented text, which GitHub comments re-flow into prose. Fixed in this PR:
`issue_report::fenced_block` wraps every machine-text emitter (general-change
plan event, formalized knowledge base, report exports) in fenced code blocks,
with regression tests in `tests/unit/issue_996_markdown.rs`. The related
goal-preamble echo was already fixed by #904.

## Most critical open issues and what this PR closes (R2, R3)

`raw-data/github/open-issues.json` and `open-prs.json` snapshot the queue at
analysis time. Per R3, issues already executing in open PRs were excluded:
#991 (PR #995), #990 (PR #994), #988 (PR #993), #710 (PR #888), #705
(PR #887), #651 (PR #652), #447 (PR #646), #483 (PR #644), #557 (PR #643).

Of the remainder, criticality was ranked by how directly an issue blocks the
#996 self-coding loop (a defect that makes a Formal-AI-driven run lie, leak,
or stall outranks documentation debt):

| Issue | Why critical | Disposition in PR #997 |
| --- | --- | --- |
| #956 — custom `--task` silently replaced by the seeded tale | The exact "empty/wrong run" failure of R1b: the agent does work, but not the requested work | **Fixed**: inline-quoted source honored, canonical path byte-identical, routing trace added; 4 tests in `tests/unit/issue_956.rs` |
| #945 — `Report` flow drops `formal-ai-*.lino` dumps into the caller's cwd | Pollutes the repository checkout a Hive Mind run operates in, producing dirty worktrees | **Fixed**: exports go to a surviving temp dir and print their path; tests included |
| #979 — Russian liveness probe «Ты тут?» fell through to web search | A liveness check that returns search noise defeats the health-check contract | **Fixed**: «ты тут»/«вы тут»/«я тут» phrases added to the `test_status` intent seed; prompts added to `tests/unit/test_status.rs`. Bare «тест», reported against v0.333.1, already routes correctly on current `main` |
| #996 markdown fencing (via hive-mind #2146) | Final messages unreadable as GitHub comments | **Fixed**: fenced `lino` blocks + machine-value target matching; tests in `tests/unit/issue_996_markdown.rs` |
| #964 — 22 duplicate requirement IDs in `REQUIREMENTS.md` | Traceability corruption: duplicate R-IDs make requirement references ambiguous for every future issue | **Fixed**: duplicates renumbered to fresh IDs (R537–R558) with cross-references updated; guarded by `tests/unit/docs_requirements_issue_540.rs` |
| #943 — agent files issues without being asked | Trust defect in the report flow | **Analyzed, not closed here**: the fix needs a negation lexicon in the seed plus a refusal path mirroring `protocol_policy.rs`; plan recorded below so the issue can proceed independently |

## Solution plans per requirement (R10)

- **R1a (model substitution)** — Formal-AI side: nothing to change; the
  server honestly serves whatever reaches it. Agent-CLI side: fail fast when
  `--model` cannot be parsed (upstream report below). Hive-Mind side: verify
  the `modelID` echoed in the session log matches the requested model before
  counting a run as a Formal-AI run.
- **R1b (empty runs)** — solved in layers: (1) route more real tasks
  (the #956 fix); (2) make routing observable
  (`FORMAL_AI_TRACE_REQUESTS=1` → `[trace] agentic_task=…` /
  `formalization_source=…`); (3) bound restarts on the Hive Mind side
  (R-B3 in #2146). Remaining Formal-AI work: a fail-loud final answer for
  genuinely unroutable coding tasks instead of conversational fallthrough —
  kept general by routing through the existing intent/recipe machinery
  rather than special-casing prompts.
- **R1c (markdown fencing)** — solved with one shared helper
  (`issue_report::fenced_block`) used by all emitters, so any future
  machine-text surface inherits correct fencing instead of re-implementing
  it.
- **R2/R3** — ranking method and dispositions in the table above.
- **R4** — each fix generalizes: #956 handles *any* quoted source in four
  quote-pair styles, not a second hardcoded document; #945 fixes the export
  *path contract*, not one call site; #979 extends the seed data, not the
  router code.
- **R5/R6** — see "Upstream reports and new-component proposals" below.
- **R7–R9, R11** — this folder, this document, the requirements table, and
  PR #997 respectively.
- **#943 plan (for its own future PR)** — add a guard at the top of
  `plan_report_flow` that consults a negation lexicon stored in the seed
  (`data/seed/`), and return a refusal-shaped answer reusing the pattern in
  `src/protocol_policy.rs`; test-first with prompts from the issue thread.

## Existing components and libraries surveyed (R10)

Checked before writing anything new, per the issue's instruction:

- `src/agentic_coding/planner.rs` — the deterministic recipe router; already
  had `Progress`/`Capability`/`tool_for` machinery, so the #956 fix reuses
  them and only the recipe moved to a new module
  (`formalization_recipe.rs`) to respect the 1000-line file gate.
- `src/agentic_coding/formalize.rs` — `formalize_text_to_links(text, doc_id)`
  already formalized arbitrary text (falling back to `doc:input`); #956
  needed only the routing fix, not a new formalizer.
- `src/issue_report.rs` — gained the single `fenced_block` helper; all three
  emitters were converted to it rather than fencing inline.
- `data/seed/intent-routing.lino` — the intent seed already modeled
  keyword/phrase/combo routing; #979 is pure seed data plus tests.
- Trace infrastructure — `FORMAL_AI_TRACE_REQUESTS=1` gating already existed
  in `src/protocol.rs` and `src/dialog_log.rs`; `trace_route` follows the
  same convention instead of introducing a new env var.
- External: the `@link-assistant/agent` CLI and Hive Mind's `solve` loop are
  the two off-repo components; their defects are reported upstream rather
  than worked around here (separation of concerns, R5).

## Online research (R8)

The wider field confirms the direction #996 points at — agents that improve
the very tooling they run on:

- [Live-SWE-agent (arXiv:2511.13646)](https://arxiv.org/abs/2511.13646)
  demonstrates an agent that starts from a bash-only scaffold and evolves its
  own tools **while** solving issues, reaching 75.4% on SWE-bench Verified —
  no offline training, LLM-agnostic. The lesson for Formal AI: the
  self-coding loop does not need a finished toolbox up front; it needs a
  reliable start-to-end loop plus the ability to extend itself mid-run,
  which is exactly the gap the PhU3X2/8GWnaH logs expose (R1b).
- [OpenHands](https://github.com/OpenHands/OpenHands) (MIT-licensed,
  ex-OpenDevin) executes plans as code actions in a sandboxed Docker
  environment and iterates on test results — the same
  execute-verify-iterate contract CONTRIBUTING.md already mandates for the
  in-repo agentic driver.
- [SWE-agent](https://github.com/SWE-agent/SWE-agent) introduced the
  Agent-Computer Interface: a small, stable command surface an agent can
  reliably drive. Formal AI's OpenAI-compatible `serve` boundary plus the
  external `@link-assistant/agent` CLI is this project's ACI; keeping it
  small and deterministic is what makes symbolic self-coding testable.
- [SWE-EVO (arXiv:2512.18470)](https://arxiv.org/pdf/2512.18470) and
  [SWE-Chain (arXiv:2605.14415)](https://arxiv.org/pdf/2605.14415)
  benchmark long-horizon, multi-step evolution — evidence that "one issue,
  one PR, every requirement" (R11) is measured industry-wide as chained
  end-to-end completion, not single-patch success.

## Upstream reports and new-component proposals (R5, R6)

### Upstream (R5)

- **`link-assistant/agent` — fail-open model substitution (Defect A).**
  Reported upstream as
  [agent#293](https://github.com/link-assistant/agent/issues/293), fixed by
  [agent PR #294](https://github.com/link-assistant/agent/pull/294)
  (released in `js-0.25.8`): an unparseable `--model` (Bun/yargs,
  oven-sh/bun#22157) now fails closed instead of silently continuing on the
  default model. The remaining half — a machine-readable `model_resolved`
  attestation event downstream guards can switch on instead of matching an
  English log line — is tracked as
  [agent#295](https://github.com/link-assistant/agent/issues/295).
- **Hive Mind (#2146, R-B3)** — bound the auto-restart/resume loop when a
  run ends with zero code changes, and verify the driving `modelID` before
  attributing a run to Formal AI. Already tracked in the companion issue's
  thread; no new issue needed.

### New-component proposals (R6)

- **Routing-trace contract.** `trace_route` currently emits ad-hoc
  `[trace] route=value` lines. If more surfaces adopt it, promote it to a
  small shared module (`src/trace.rs`) with a stable line format that Hive
  Mind can parse to explain *why* a run did what it did. Not needed yet —
  two call sites do not justify a component.
- **No missing external library was identified.** The gaps found in this
  case study are contract gaps (fail-fast, fencing, routing coverage), all
  addressable inside the existing stack; introducing a new dependency would
  specialize rather than generalize.
