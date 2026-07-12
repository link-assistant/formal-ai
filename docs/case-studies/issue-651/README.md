# Issue 651 Case Study

Issue [#651](https://github.com/link-assistant/formal-ai/issues/651) asks for a
planning pass: find the most critical missing features and items that keep the
project from fully fulfilling its vision and roadmap, file them as
maximum-detail sub-issues, and make `VISION.md` and `ROADMAP.md` consistent
with the actual state of development. Unlike most case studies here, the
deliverable is not code — it is a verified requirement inventory, 21 new
tracked issues (E35–E55, [#654](https://github.com/link-assistant/formal-ai/issues/654)–[#674](https://github.com/link-assistant/formal-ai/issues/674)),
and a restructured roadmap that tracks general requirements with
done / partially done / not done status.

## 1. Collected Data

- Issue and PR snapshots: `raw-data/issue-651.json`,
  `raw-data/issue-651-comments.json`, `raw-data/pr-652.json`.
- Open-issue snapshots: `raw-data/open-issues-2026-07-12.json`,
  `raw-data/open-issues-detail.txt`, `raw-data/open-issues-full.txt`,
  `raw-data/open-issues-full2.txt`.
- Closed-issue history used for the deferred-work audit:
  `raw-data/closed-issues-2026-07-12.json`, `raw-data/closed-batch1.txt`,
  `raw-data/closed-detail-{a,b,c}.txt`, `raw-data/e-issues-closure.txt`,
  `raw-data/closing-prs.tsv`, `raw-data/pr-bodies.txt`.
- CI state for the branch: `raw-data/ci-runs-branch.json`.
- Deferred/incomplete work audit across issue conversations and repo docs:
  `raw-data/incomplete-work-audit.md`.
- Repository-state audit against `VISION.md`/`ROADMAP.md`/`REQUIREMENTS.md`:
  [`code-audit.md`](code-audit.md).
- Online research on Agent CLI, Hive Mind, and prior art for self-coding
  systems: [`online-research.md`](online-research.md).
- Full bodies of all 21 filed issues with acceptance criteria and the
  epic-to-issue mapping table:
  [`proposed-issues.md`](proposed-issues.md).

No issue screenshots were present, so there were no image attachments to
download or verify.

## 2. Requirements From The Issue

The issue text decomposes into eleven requirements. Each row states how it was
satisfied and where the evidence lives.

| ID | Requirement | How it was addressed |
| --- | --- | --- |
| Q1 | Created issues must carry maximum detail: exactly what to do and how to test that it is done. | Every E35–E55 body has **Problem / Approach / Existing components / Acceptance criteria** sections; acceptance criteria are concrete checks (tests, CI jobs, published artifacts). See `proposed-issues.md`. |
| Q2 | Collect all conversations on previous issues; find what was not fully done or was ignored by agents. | `raw-data/incomplete-work-audit.md` sweeps closed-issue comments and repo docs; it found ~19 deferred items (R378–R385, #625/#628 CI matrix, #620/#511 upstream constraints, #541 F1–F5, journeys F2–F6, and more) and five recurring deferral anti-patterns. All actionable items map to E35–E55 or an already-open issue. |
| Q3 | Update vision and roadmap to be consistent with the latest development. | `VISION.md` gained three sections (associative-only surface, the self-coding ladder, wide-audience reach) and a 2026-07 current-direction note; `ROADMAP.md` was restructured (see Q4). |
| Q4 | The roadmap must track requirements in general — not issues — with done / partially done / not done status. | `ROADMAP.md` is now eight requirement groups (associative core, universal solver, knowledge/translation, learning, self-coding, interfaces, distribution, quality) with a status per requirement and an invariant: no partial/not-done requirement without a linked open issue. |
| Q5 | Everything partially done without an issue must get an issue via `gh`. | 21 issues filed with `gh`: [#654](https://github.com/link-assistant/formal-ai/issues/654)–[#674](https://github.com/link-assistant/formal-ai/issues/674). The roadmap invariant makes this a standing rule, not a one-time cleanup. |
| Q6 | Be ambitious; make the vision and plan more ambitious. | The self-coding ladder (rungs 1–4 ending in a measured self-hosting percentage per release, E38/#657), full solver-in-WASM absorption of ~26.7k JS lines (E39/#658), and arbitrary NL-program compilation (E55/#674) raise the ceiling rather than trimming scope. The audit's "ambition silently downgraded" anti-pattern is explicitly countered. |
| Q7 | Focus only on associative technologies — links networks, Links Notation, the meta language; no graphs or tables. | The code audit confirmed the architecture complies; the remaining naming debt (`/v1/graph`, `*_source_graph.rs`, UI strings) is E45/[#664](https://github.com/link-assistant/formal-ai/issues/664), including a lint that keeps it out. `VISION.md` now states the constraint as a named principle. |
| Q8 | Plan for the project to code itself using itself, via Agent CLI directed by Hive Mind. | The four-issue self-coding track: general agentic planning (E35/#654) → Hive-Mind-dispatched end-to-end issue solve (E36/#655) → benchmark-gated promotion (E37/#656) → self-hosting metric (E38/#657), wired with `blocked_by` relations. `online-research.md` documents the existing `formal-ai` provider in Agent CLI and Hive Mind's model pass-through that make this feasible today. |
| Q9 | Reach a wide audience on web, desktop, and beyond. | Distribution track: offline PWA + npm engine package (E46/#665), VS Code Marketplace/Open VSX publication (E47/#666), shareable associative packages (E49/#668), cloud memory sync (E50/#669), WebVM browser execution (E51/#670), on top of the already-shipped crate/Docker/Pages/Telegram/desktop surfaces. |
| Q10 | All created issues must be sub-issues of #651 with explicit blocking relationships via the modern GitHub API. | All 21 issues registered via `POST /repos/{owner}/{repo}/issues/651/sub_issues`; blocking via `POST .../issues/{n}/dependencies/blocked_by` for #655←#654, #657←#655+#656, #665←#658, #667←#666. Verified by reading both endpoints back (section 5). |
| Q11 | Collect data to `./docs/case-studies/issue-651`; deep case study; online research; list all requirements; propose solution plans; check existing components. | This directory: raw data snapshots, two audits, online research, per-epic solution plans with existing-component inventories in `proposed-issues.md`, and this README. |

## 3. Key Findings

From [`code-audit.md`](code-audit.md) and
[`raw-data/incomplete-work-audit.md`](raw-data/incomplete-work-audit.md):

1. **Deferral migrated from GitHub into repo docs.** When an issue closed,
   its remainder often survived only as a "follow-up" sentence in
   `REQUIREMENTS.md` or a design doc (R378–R385 among others), invisible to
   issue-based planning. The roadmap invariant and the E35–E55 batch close
   this loophole.
2. **The self-coding foundation already exists but the generality does
   not.** Agent CLI ships a built-in `formal-ai` provider and Hive Mind
   passes `--model formal-ai` through, and recipe-driven self-edits landed
   (issues #538/#540) — but planning is pinned to `is_*_task` recipes.
   Generalizing the planner (E35) is the single highest-leverage item; three
   other epics are blocked behind it.
3. **The web worker is a second implementation of the solver.** About
   26,700 lines of JavaScript mirror Rust logic; every solver change must
   land twice. Absorbing it into the Rust→WASM worker (E39) is the
   foundation for the PWA/npm distribution track.
4. **Test infrastructure shipped as prose.** The multi-CLI verification
   matrix from #625/#628 exists as a guide, not a CI job, and claude, grok,
   and aider were never actually run against the server. E52 turns the guide
   into a gating matrix that also carries the #650 defect regressions and
   the #620/#511 upstream constraints as executable assertions.
5. **Closure claims outran verification** in several places (issue #650
   catalogues four verified protocol defects from work that was reported
   complete). The promotion protocol (E37) and the CI matrix (E52) are the
   structural countermeasures.

## 4. Filed Issues

The epic-to-issue mapping, dependency edges, and full bodies live in
[`proposed-issues.md`](proposed-issues.md). Summary: E35–E38 self-coding
track (#654–#657), E39–E45 core/associative track (#658–#664), E46–E51
distribution track (#665–#670), E52 CI matrix (#671), E53 UI follow-ups
(#672), E54 workspace self-AST census (#673), E55 arbitrary NL skill
compilation (#674). Items intentionally **not** re-filed: #552's upstream
work (already tracked in web-capture#141 and meta-language#168) and the #108
UI gaps (already tracked by open #557).

## 5. Verification

- `GET /repos/link-assistant/formal-ai/issues/651/sub_issues` returns exactly
  the 21 issues #654–#674.
- `GET .../issues/657/dependencies/blocked_by` returns #655 and #656; the
  other three edges (#655←#654, #665←#658, #667←#666) verified the same way.
- Every requirement in the restructured `ROADMAP.md` that is not *Done*
  links an open issue; the pre-existing open issues referenced
  (#447, #453, #482, #483, #491, #531, #534, #557, #649, #650) were checked
  to be open with matching topics on 2026-07-12.
