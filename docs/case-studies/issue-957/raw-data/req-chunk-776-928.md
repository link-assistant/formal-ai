# Requirement audit — items #776–#928 (2026-07-18 → 2026-08-04)

153 items (83 issues, 70 PRs), 465 comments (all posted under `konard`; ~380 are bot/harness artifacts — solution-draft logs, auto-merge notices, working-session summaries). 131 requirement records extracted to `req-chunk-776-928.ndjson`.

This is the newest slice of the repo: nearly all of the repository's open issues live here, and several very recent konard requests (late July / August 2026) have no delivery evidence at all.

## 1. Standing clauses konard repeats (verbatim or near-verbatim)

1. **Single-PR total delivery**: "Please plan and execute everything in this single pull request, you have unlimited time and context… until each and every requirement fully addressed, and everything is totally done." — on virtually every PR he pushes on (#781, #795, #807, #814–#816, #820, #850–#857, #873, #876–#884, #885, #897, #912, #913…).
2. **Deeper-analysis / self-hosting clause**: "We need make the analysis deeper and fully implement vision from the issue using auto learning, and same task execution using Formal AI via Agent CLI… advancing our meta algorithm to the highest possible potential… It is ok to change architecture to move toward our goals of generalization in any place you touch." — ~20 PRs in this range. From #884 on it grows a **baseline-and-delegate** tail: use the PR's own solution-draft log as a minimum baseline for Formal-AI-via-Agent-CLI, and "delegate as much of smaller tasks to Formal AI as possible".
3. **Case-study directory**: compile all logs/data to `docs/case-studies/issue-{id}`, reconstruct timeline, list every requirement, root causes, solution plans, survey existing components (#781, #873, #885, #914…).
4. **Debug/verbose on insufficient data**: add debug output and verbose mode when the root cause can't be found (#781); institutionalized as verbose-by-default in #822 R5.
5. **Report upstream**: any defect found in another repo (CI templates, OpenCode, command-stream, agent-commander, meta-language) must be filed there with repro + workaround + fix suggestion (#796-family, #819, #841, #883, #896). **#894 records that a batch of these upstream filings was never completed — still open.**
6. **CI/CD template comparison** issue body (#796, #798, #804, #808, #810, #812, #828): full file-tree comparison against the three link-foundation pipeline templates, hive-mind CI-CD-BEST-PRACTICES.
7. **Fix everything in this PR, no deferral**: "All must be fixed in this pull request, no objections accepted." / "Don't hide anything and don't defer. CI/CD MUST BE FIXED." (#820); "All checks must be in Pull Request stage, not only on main" (#808).
8. **Recover-from-log**: when a hive-mind container dies (disk space etc.), "We need to recover all work from <gist>" and continue in the same PR (#815, #850, #856×2, #913).
9. **Honesty doctrine**: verify by observed effect never narration (#848); never fabricate provenance (#843); assertions on measured outcomes, not on the shape of plans/commands (#839 §6, #840 correction comments).
10. **Generalize/meta-generalize**: support the entire class of tasks and all classes of all tasks (#819); one single meta algorithm that can append/generalize itself (#781, #873, #914).
11. **Update all dependencies to latest** unless breaking (#851).
12. **Reference-agent comparison**: spawn claude/codex/opencode free models on the same prompt and adopt their best practices; write this into contributing/testing guidelines (#819, #840, #842).

## 2. Open issues with NO delivery evidence (as of 2026-08-04)

**Completely untracked (zero comments, no PR, no epic reference):**
- **#802** — 2-4-6 game / hypothesis-halving / disproof-first experiment methodology.
- **#825** — autocomplete in all custom input boxes (empty body, title only).
- **#836** — warn user when a request appears illegal (companion to delivered #835/#837).
- **#861** — optional anonymous Sentry reporting (depends on delivered #864).
- **#901** — automate TRIZ / contradiction resolution over links, test on top-20 tasks (2026-08-02, very recent).
- **#862, #863, #865, #866, #867, #868** — agentic-CLI failure reports from 2026-07-25 (Execute rosettacode URL; copy-stdin example; "List me files here"; "Execute ls command" → `ls command`; report-issue leaking opencode's system prompt; bare `ls` answered with web-UI toolbar instructions). None ever closed or referenced by a fix PR.
- **#872** — RU App Store open-source-games research prompt; konard: "Мы должны полностью поддерживать и корректно все этипы для таких промптов" ("we must fully support and correctly handle all stages for such prompts").
- **#869** (skulidropek) — meeting-scheduling prompt (Назначь мне встречу…) → unknown; nobody has touched it.
- **#907, #908, #909** — from the 2026-08-02 defect cluster: caller-framing intent hijack (gemini unusable), exit-code-blind step verification, incomplete `--global` headless config. No PRs yet.

**Open with partial tracking:**
- **#800, #801, #821** — silent/empty web research reports; nominally addressed by #803/#844/#855 but never closed; #801 carries konard's regression claim vs #680 ("We must have all e2e tests, that will ensure that will never break again").
- **#824** — refused filesystem move; a comment explicitly records it is NOT covered by #840/#842 and that mutating-action ladder nodes (824.L1–L4 with sandbox reset) were deliberately deferred. **Explicit deferred design work with no follow-up issue.**
- **#826, #827, #838** — kept open as "source reports" after consolidation into #840/#839 (both since closed). Nothing re-verified the source reports against the delivered fixes and closed them.
- **#905, #906** — false verification claim / language router; PRs #927 and #928 exist but are OPEN; the latest automated session on #927 FAILED ("Agent reported error: [object Object]").
- **#889, #891–#895** — the E68 audit children (localized thinking steps, 50-equation ratchet, Spider-Man seed, summarization 80% ratchet, upstream template filings, coverage ratchet). All open; parent PR **#888 itself is unmerged**.
- **#916–#924** — epics E69–E77 from the #914 vision pass, all open (created 2026-08-03).

**Open PRs awaiting merge:** #887 (anticipatory dreaming, CI green + "Ready to merge" since 2026-08-01), #888 (E68 audit, same), #927, #928.

## 3. Dubious or dishonest-pattern closures flagged INSIDE the range

- **#745 / #758 reopened-in-fact**: comment on #840 (5073600499) proves both were closed COMPLETED while measured behavior (routing flips on `my`/`Search`) contradicts their acceptance criteria — "a requirement was accepted as complete, and the measured behaviour today contradicts it." The maintainer question *"whether to formally reopen #745/#758… is a maintainer call"* was never answered; neither was reopened.
- **#832 → #838**: #832/#833 "verified" the report flow with string assertions on the generated command; #838 then arrived carrying an empty body. #839 documents this as the acceptance-evidence failure mode.
- **#814 (all-client E2E matrix)** and other PRs merged after the deeper-analysis clause — the clause is acknowledged with an "Implemented the requested deeper/generalized pass" note, but no independent verification exists in-thread.
- **#835** closed via PR #900 without visible answers to its open questions (CSAM-hash provider, per-jurisdiction mapping versioning, uncertainty UX).
- **#848** closed 2026-08-02 via PR #897 while its own comments record the dataset was expanded to 130 prompts with baseline **38/130** — the definition-of-done ("ladder runs in CI", L1 honest attempts) is carried forward only implicitly by open epic #916.

## 4. Unanswered konard questions / dropped follow-ups

- #840: reopen #745/#758 formally? (unanswered)
- #846: choice between `paths-ignore` vs dropping the unconditional push clause was left as "a maintainer call; I have not changed either" — PR #854 closed the issue; which option was chosen isn't confirmed in-thread.
- #823 review: the 20%-of-smallest-leaves self-coding floor has no recorded fulfillment anywhere in the range; E77 (#924) restates the goal but no measurement.
- #841: local PTY/TUI code deletion after upstreaming ("Step 4 is part of done, not a follow-up") — upstream issues filed (command-stream#175/#180, agent-commander#43/#46) but deletion/depend-on-published-packages not evidenced.
- #781: adapters converting shared ChatGPT/Google dialogs to Links Notation — parenthetical aside, never surfaced again.
- #883: pause-the-PR-if-meta-language-extensibility-insufficient instruction; no meta-language upstream issues are referenced in the merge evidence.

## 5. Recurring themes

1. **Honest evidence over narration** — the through-line of the whole range: #843 (fabricated `example.org` provenance), #905/#908 (exit-code-blind verdicts), #904 (plan-file self-verification), #879 (empty-workspace success), #839 (report success without artifacts). Konard repeatedly demands verification keyed to observed effects.
2. **Procedure gap, not capability gap** — the #838/#827/#826 → #840/#842 arc: free reference models beat Formal AI with two `ls` calls; measured ladders (8/24, 2/13, 38/130) become ratchets.
3. **Requirements-as-measurement** — konard's process converts every complaint into a numbered standard + measured baseline + CI ratchet (task ladder, coding ladder, self-hosting share, coverage, equation corpus).
4. **Coding-first self-hosting** — from the #823 post-merge review through #848/#879/#902–#909 to #914's E69–E77: Formal AI must author its own changes; current honest measurement is ~0% real self-authorship and zero write-effect rungs passing.
5. **`formal-ai with <tool>` wrapper quality** — August cluster #902/#903/#909/#925 (argv construction by parsing, provider block survival, complete headless configs).
6. **Full-context reporting in Links Notation** — #822 → #823 → #832/#833 → #838 → #839/#849: LiNo-first everything, repeated-key sequences, verbose default, real session ids, one shared report builder for all five surfaces.
7. **Answer quality parity with Claude/ChatGPT/Google** — konard pastes reference answers (#821, #826, #827) as the explicit standard: synthesized multi-source summaries with cited links.

## 6. Most significant not-yet-tracked konard asks (recommend filing/verification)

1. #907/#908/#909 — no PRs; #916 (E69) references them but no work started; gemini client effectively broken.
2. #836 illegal-request warning; #861 Sentry reporting — legal/reporting companions left behind after their siblings shipped.
3. #824 mutating-action ladder rungs with sandbox-reset semantics — explicitly deferred, never filed.
4. #873 principles (versioned recoverable memory, immutable baseline tests, 1-hour bounded autonomy, full-trust mode) — only partially absorbed by E72; the memory-versioning and autonomy-limit requirements exist nowhere else.
5. #901 TRIZ automation — brand-new, untouched.
6. #802 experiment-design methodology (2-4-6) — untouched since 2026-07-19.
7. The stale open "source report" issues (#800/#801/#821/#826/#827/#838/#862–#868/#872) need re-verification against v0.326.0 and closure or refiling; they are the noise floor hiding new regressions.
