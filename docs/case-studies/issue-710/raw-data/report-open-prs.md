# Open PR Report — link-assistant/formal-ai

Generated 2026-07-14. 17 open PRs. All PRs are authored/managed by the hive-mind AI-solver automation (posting under `konard`'s account); comments marked "human feedback" below are the ones that read as genuine konard-authored requirement comments rather than bot logs.

CI legend: rollup conclusions summarized (SKIPPED entries are conditionally-skipped jobs; SUCCESS means the run passed).

---

## PR #697 — [WIP] E45: Associative terminology cleanup: links network, not graph
- **Linked issue:** #664 (Fixes #664) · branch `issue-664-a0a4184c732c`
- **Status:** DRAFT · CI: 2 SUCCESS / 12 skipped · created 2026-07-14 09:59 (today)
- **Implements:** Placeholder auto-generated body only ("Work in Progress — the AI assistant is currently analyzing…"). Terminology cleanup replacing graph/vertex/edge language with links-network terminology.
- **konard feedback:** none yet (0 comments, 0 reviews).
- **Stalled?** No — brand new, actively being worked by the solver.

## PR #696 — [WIP] E44: Retire the `SPECIALIZED_HANDLERS` precedence remnant into data-driven routing
- **Linked issue:** #663 (Fixes #663) · branch `issue-663-321cc5865928`
- **Status:** DRAFT · CI: 2 SUCCESS / 12 skipped · created 2026-07-14 09:49 (today)
- **Implements:** Placeholder WIP body only; goal is removing the SPECIALIZED_HANDLERS precedence remnant in favor of data-driven routing.
- **konard feedback:** none yet.
- **Stalled?** No — brand new, in progress.

## PR #695 — E43: Budget-driven random and evolutionary search in synthesis (F4)
- **Linked issue:** #662 (Fixes #662)
- **Status:** ready (not draft) · CI: 2 SUCCESS / 12 skipped · bot posted "Ready to merge" at 10:03 today
- **Implements:** Adds a `compute_budget` knob on `SolverConfig` (default 512) and a new search stage (`src/solver_search.rs`) that, when deterministic reuse/rule reasoning yields nothing, recognizes arithmetic-reachability problems and runs random + evolutionary search over candidate compositions, scored against step-6 generated equality tests.
- **konard feedback:** none (only bot session summaries / cost logs, $13.65).
- **Stalled?** No — completed today, awaiting merge.

## PR #694 — E42: Probability-weighted statement formalization with contradiction warnings (R384)
- **Linked issue:** #661
- **Status:** ready · CI: 2 SUCCESS / 12 skipped · "Ready to merge" at 08:54 today
- **Implements:** (A) emits a `statement_weight` evidence link per formalization interpretation (softmax posterior, sums to 1 across candidates, trace-only); (B) new `requirement_contradiction` module classifying directive polarity (require/forbid) across en/ru/hi/zh and warning when a new directive opposes a retained one.
- **konard feedback:** none (bot logs only, $11.63).
- **Stalled?** No — completed today, awaiting merge.

## PR #693 — E41: Bulk semantics importer from external lexical sources (R378)
- **Linked issue:** #660 (Fixes #660)
- **Status:** ready · CI: 6 SUCCESS / 1 skipped / **2 pending or unconcluded checks**; auto-restart iteration 1 triggered at 09:59 today ("CI failures detected") — fix cycle in flight
- **Implements:** Generalizes `scripts/ground-meanings.rs` into a `formal-ai import lexemes` CLI command: parses `<slug> <Qid>` concept files, grounds from committed Wikidata cache records, renders canonical 23-line grounded meaning blocks, and refuses entries failing validate-on-import (recorded as `import_rejected` events). Tests moved to `tests/unit/bulk_lexeme_import.rs` per repo rules.
- **konard feedback:** none (bot logs, $23.10).
- **Stalled?** No — active today; CI failure remediation loop running.

## PR #692 — E40: CI lint burning down hardcoded natural-language strings (R379)
- **Linked issue:** #659 (Closes #659)
- **Status:** ready · CI: 9 SUCCESS / 5 skipped · "Ready to merge" at 07:41 today
- **Implements:** `scripts/check-hardcoded-language.rs` CI gate using a sentence heuristic to detect user-facing prose literals in `src/**/*.rs`; new prose must be grounded in `data/seed/*.lino` or admitted as tracked debt that can only shrink. Second commit migrates 8 `engine_responses` fallbacks to seed, proving the burn-down loop. A stray auto-generated `.gitkeep` was cleaned up in a restart iteration.
- **konard feedback:** none (bot logs, $8.47).
- **Stalled?** No — green and awaiting merge.

## PR #691 — E39: WASM worker migration groundwork — CI guards, size budget, restored no_std build (R380)
- **Linked issue:** #658
- **Status:** ready · CI: 2 SUCCESS / 12 skipped · "Ready to merge" at 07:21 today (after one auto-restart for CI failures)
- **Implements:** Explicitly a "foundational first slice" of the full ~26,700-line JS→WASM worker migration: fixes the latent broken `no_std` wasm32 build (cfg-gated `forced_language` in `src/language.rs`), adds two CI guard scripts (wasm32 build + size budget), workflow wiring, capability inventory doc, and CI-CD test.
- **konard feedback:** none (bot logs, $8.11).
- **Stalled?** No — green, awaiting merge. Note it does NOT complete #658, only groundwork.

## PR #690 — E37: Benchmark-gated promotion protocol for self-improvement proposals
- **Linked issue:** #656 (Closes #656)
- **Status:** ready · CI: 9 SUCCESS / 5 skipped · "Ready to merge" at 05:54 today
- **Implements:** A `promotion` event protocol in the meta language (`src/promotion.rs`: `promotion_proposal` → per-gate `promotion_evidence` → `promotion_decision` → `promotion_applied`/`promotion_rejection`), plus `formal-ai improve --promote`. Promotion never pushes directly — the branch/PR step is emitted as a human-reviewed plan.
- **konard feedback:** none (bot logs, $13.06).
- **Stalled?** No — green, awaiting merge.

## PR #689 — Associative knowledge networks learning: usage-weighted persistence (#686)
- **Linked issue:** #686 (Closes #686)
- **Status:** ready · CI: 2 SUCCESS / 12 skipped · "Ready to merge" at 04:58 today (after one auto-restart: the issue-540 repo-wide terminology test flagged a case-study doc; fixed)
- **Implements:** New `src/associative_persistence.rs` `AssociativeMemory`: persists meta-language expressions as content-addressed nodes, counts reads (usage) and writes (changes), derives usage from incoming/outgoing links, keeps everything as links (not graph/edges/vertices), applying Wikontic-paper practices. Builds on existing `SubstitutionGraph`, `stable_id`, and the LFU precursor in `src/dreaming.rs`.
- **konard feedback:** none (bot logs, $10.39–14.39).
- **Stalled?** No — green, awaiting merge.

## PR #688 — Agentic mode: act on simple requests (report issue / web research / recall) (#687)
- **Linked issue:** #687 (Fixes #687)
- **Status:** ready · CI: 2 SUCCESS / 12 skipped · "Ready to merge" at 04:33 today (after one auto-restart fixing inline-unit-test guard violations, lint, coverage)
- **Implements:** Three new deterministic planner recipes in `plan_chat_step` so that report-issue / web-research / recall / factual prompts no longer dead-end on the "I could not determine…" blurb when Formal AI runs as an agentic backend (e.g. OpenCode). Recipes emit tool calls for the harness to execute (Formal AI has no HTTP client by design).
- **konard feedback:** none (bot logs, $19.03–24.71).
- **Stalled?** No — green, awaiting merge.

## PR #685 — fix: accept explicit content: null on assistant tool-call turns (#682, breaks qwen)
- **Linked issue:** #682 (Fixes #682)
- **Status:** ready · CI: 9 SUCCESS / 5 skipped · last "Ready to merge" 2026-07-13 22:46
- **Implements:** Fixes HTTP 400 "data did not match any variant of untagged enum MessageContent" when an assistant message carries explicit `"content": null` with `tool_calls` (standard OpenAI shape emitted by Qwen Code). Root cause: `#[serde(untagged)] MessageContent` has no unit variant and `#[serde(default)]` only applies to absent keys. Also includes analysis logs under `dev/log/issues/682/pulls/685`.
- **konard feedback (human, 2026-07-13 18:40):** "We need to add the analysis and fully implement vision from …/issues/682 using auto learning, and same task execution using Formal AI via Agent CLI. … I expect this pull request to cover the most ambitious of requirements through generalization of logic, reasoning, advancing our meta algorithm to the highest possible potential."
  - **Addressed?** Yes — a work session started 21:35 the same day, delivered the confirmed root-cause fix, and CI went green ("Ready to merge" 22:46). No further konard response since.
- **Stalled?** No — done ~1 day ago, awaiting merge/final human review.

## PR #684 — fix(agentic): route file-creation requests to write, not read (#681)
- **Linked issue:** #681 (umbrella #680)
- **Status:** ready · CI: 9 SUCCESS / 5 skipped, but konard reports **merge conflicts** need resolving
- **Implements:** Fixes "Create a file named hello.txt with the content hello world" emitting a `read` tool_call on the nonexistent file. Two defects: (1) `file_read_task_for` ran before the write planner and its intent classifier matched the word "content"; (2) the write planner missed the phrasing. Routes file-creation to `write`.
- **konard feedback (human):**
  - 2026-07-13 18:40: "We need to add the analysis and fully implement vision from …/issues/681 using auto learning, and same task execution using Formal AI via Agent CLI…" → addressed by the 19:42–21:00 session (fix delivered, CI green, "Ready to merge" 21:03).
  - **2026-07-14 06:43 (latest comment on the PR, UNADDRESSED):** "We need to resolve conflicts and fully implement vision from …/issues/681 using auto learning, and same task execution using Formal AI via Agent CLI. … I expect this pull request to cover the most ambitious of requirements through generalization of logic, reasoning, advancing our meta algorithm to the highest possible potential…"
  - **Addressed?** The first round yes; the 06:43 conflict-resolution/vision request has had **no work session since** — it is the pending action item.
- **Stalled?** Mildly — waiting on a new solver session to pick up today's feedback and resolve conflicts.

## PR #652 — Plan the critical missing features as tracked sub-issues; refresh vision and roadmap
- **Linked issue:** #651
- **Status:** ready · CI: 2 SUCCESS / 12 skipped · "Ready to merge" 2026-07-12 20:28
- **Implements:** A planning pass, not code: audits gaps vs the vision, files 21 maximum-detail sub-issues of #651, adds a case study at `docs/case-studies/issue-651/`, and rewrites `VISION.md` / `ROADMAP.md` so status is honest and partial requirements are tracked. Notably, the E-series open PRs (#690–#697) are those sub-issues being executed.
- **konard feedback:** none (bot logs, $33.49 Fable 5 run).
- **Stalled?** No, but idle ~2 days awaiting merge. Merging it is low-risk (docs/planning) and unblocks traceability for the E-series PRs.

## PR #646 — Fix sidebar splitter affordance and scrolling clarity
- **Linked issue:** #447 (Fixes #447)
- **Status:** ready · CI: 3 SUCCESS / 11 skipped · "Ready to merge" 2026-07-10 20:51
- **Implements:** Replaces the scrollbar-like sidebar resize track with a transparent 10px hit target and thin full-height sash; hover/focus/drag highlighting, light/dark themes, `ew-resize` cursor; preserves resizing, persistence, mobile, ARIA keyboard controls. Adds Playwright regression at the reported 1280x565 viewport plus a case study.
- **konard feedback:** none.
- **Stalled?** Idle 4 days, green, awaiting merge/review. Not abandoned but nothing is moving it.

## PR #644 — Add experimental formalization model fallback
- **Linked issue:** #483 (Fixes #483)
- **Status:** ready · CI: 2 SUCCESS / 12 skipped · "Ready to merge" 2026-07-09 00:41
- **Implements:** A formalization-safe small-model advisor contract (model may only select an existing `FormalizationCandidate`; low-confidence/synthetic advice ignored; normal selector stays authoritative); default-off browser settings gate with hardware-filtered WebGPU model catalog (no weights/runtime bundled); case study; plus a CI-flake fix moving issue-538 lexeme fetch fixtures from raw GitHub to jsDelivr after HTTP 429s.
- **konard feedback:** none (bot logs; expensive $73.07 GPT-5.5 run; a stray `.codex/` scratch dir was cleaned in auto-restart).
- **Stalled?** Idle ~5 days, green, awaiting merge. Oldest "ready" PR alongside 642/643.

## PR #643 — Add polished multi-framework UI skins and color themes
- **Linked issue:** #557 (Fixes #557)
- **Status:** ready (not draft) · CI: 8 SUCCESS / 6 skipped · last activity 2026-07-10 19:27
- **Implements:** Multi-framework UI skins: polished Chakra UI flat skin with unified rounded composer (transparent textarea in every skin); Material skin mounting real MUI (`MuiThemeProvider` + `ScopedCssBaseline`, `MuiIconButton` controls); Glass skin on `rdev/liquid-glass-react` with CSS frost fallbacks; persisted localized Glass controls (opacity/blur/refraction); seven persisted color palettes (Emerald, Ocean, Indigo, Violet, Rose, Amber, Graphite); Playwright coverage across skins/palettes.
- **konard feedback (human, FIVE rounds — the most contested PR):**
  1. 07-08 23:19: "Double check that is high quality polished UI, on screenshot I don't see something impressive. Re-read …/issues/557…" → follow-up glass-ambient polish pass landed 07-09 09:22.
  2. 07-09 10:28: "Not polished enough, check paddings/margins in all views, background of actual text field must be transparent… We should keep basic UI skin in Chakra UI that was before… For material UI we should also add support for MUI framework…" → addressed in 10:29–11:46 session.
  3. 07-09 12:30: "Double check that …#issuecomment-4924079586 is FULLY implemented. I don't see changes in screenshots of PR description. Also take https://www.reactbits.dev/ as example…" → session crashed (exit 144); konard 22:29 asked to continue from the failure log; addressed 07-10 01:51 (React Bits research, 4 skins x 7 palettes tests).
  4. 07-10 06:52: "I think my requirements were still ignored. Make sure we fully address requirements from: [two prior comments + issue #557]. https://github.com/rdev/liquid-glass-react — the most popular and best solution…" → composer restructured into single embedded pill (07-10 07:25), CI fixed.
  5. **07-10 16:40: "AUTHORITATIVE REQUIREMENTS AND COMPLETION GATE. This comment is the single authoritative requirement ledger for this PR…"** — establishes interpretation rules and requires the literal phrase `APPROVED TO FINALIZE` from a human before the PR can be finalized.
  - **Addressed?** The AI produced a requirement-traceability ledger and an implementation/evidence update (07-10 19:23) and explicitly did not finalize: "I found no later human comment containing the exact approval phrase `APPROVED TO FINALIZE`, so I did not mark ready…"
- **Stalled?** **Yes — blocked on konard.** All AI-side work halted 07-10 awaiting the `APPROVED TO FINALIZE` phrase; no human response in 4 days. This is the clearest action item for the user.

## PR #642 — Implement pattern inference over link-native sequences (1D/2D) — issue #531
- **Linked issue:** #531 (Closes #531)
- **Status:** ready · CI: 9 SUCCESS / 5 skipped · "Ready to merge" 2026-07-09 03:39
- **Implements:** Link-native sequence substrate under `src/sequences/` (doublet store with self-referential points, structurally deduplicated composites, lossless `expand()`; symbol allocation; converters), ported in spirit from `linksplatform/Data.Doublets.Sequences`, wired into the solver with pattern inference over 1D sequences/text and 2D grids, plus ontology, tests, and requirement traceability. Pattern-inference report was localized end-to-end (en/ru/hi/zh) to clear the language-coverage CI gate.
- **konard feedback (human, 07-08 23:21):** "I want not only research, but actual implementation in all relevant places in the code base. And if no such places, come up with tasks/questions/workloads that certainly will use all of them. Please plan and execute everything in this single pull request…"
  - **Addressed?** Yes — the 07-09 00:54 session converted the research pass into a full implementation, made it multilingual, and CI went green ("Ready to merge" 03:39). No further konard response.
- **Stalled?** Green and idle ~5 days awaiting merge. Not abandoned; waiting on human review/merge.

---

## Cross-cutting observations

1. **No inline review comments and no formal reviews exist on any open PR** (`pulls/<n>/comments` is empty for all 17; `reviews` arrays are empty). All feedback flows through issue-style PR comments.
2. **Two buckets:** (a) the fresh 2026-07-14 wave (684 follow-up, 688–697) executing #651's sub-issue plan — all healthy, most already "Ready to merge"; (b) the older 07-08..07-12 batch (642, 643, 644, 646, 652) — all CI-green and waiting on konard to review/merge.
3. **Truly blocked on the user:** #643 (needs the `APPROVED TO FINALIZE` phrase or more feedback) and #684 (konard's 06:43 conflict-resolution request has no session yet — needs the solver re-triggered or manual conflict resolution).
4. **In-flight risk:** #693 auto-restart for CI failures at 09:59; #697/#696 are WIP drafts with placeholder bodies.
5. **Nothing looks abandoned.** The hive-mind auto-restart-until-mergeable monitor has driven every non-draft PR to a green "Ready to merge" state; the bottleneck is human merge bandwidth.
