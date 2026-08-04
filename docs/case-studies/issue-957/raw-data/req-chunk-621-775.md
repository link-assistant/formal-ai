# Requirement audit — items #621–#775 (link-assistant/formal-ai)

Scope: 155 numbered items (issues + PRs, shared numbering), 526 konard-login comments (most are bot logs posted under his account; ~70 are genuine human comments or konard-directed audit reports). 175 requirement entries extracted into `req-chunk-621-775.ndjson`.

## Shape of this range

Three waves dominate:

1. **The `with-formal-ai` / agentic tool-calling campaign** (#620-ish → #650, then #676–#697, then #711–#719, then the huge #744–#763 v0.297.1 sweep). konard personally runs hands-on test matrices (phrasing × CLI × protocol, with a logging proxy for provenance), files forensic-grade issues, and the same defect class — *routing is phrasing-gated, not intent-based; the server hardcodes its own tool/parameter names instead of honoring each harness's advertised schema* — is "fixed" and regresses at least three times (#624/#627 → #680/#681 → #712 → #745/#758, still measured broken at 8/24 in his 2026-07-25 comment on #710).
2. **The #651 planning epoch**: E35–E68 epics (#654–#674, #698–#710), AI-drafted under konard's explicit mandate, several of which are themselves *audits of previously dropped konard requirements* (#698 external benchmarks, #699 handler migration, #700 si-units, #701 adoption gap, #708 Turing-complete queries, #709 fusion, #710 the dropped-requirements backlog itself).
3. **CI/CD firefighting** (#711, #717, #730, #736, #738, #739, #742): the same "fix all false positives/negatives/warnings/errors, compare against the four link-foundation pipeline templates" issue re-filed five times in ~one week because each fix left residue.

## Standing clauses konard repeats (record once, apply everywhere)

- **Single-PR total delivery**: "plan and execute everything in this single pull request… unlimited time and context… until each and every requirement fully addressed" — pasted on virtually every PR he touches.
- **Case-study dirs**: compile all data to `./docs/case-studies/issue-{id}`, deep analysis, online research, full requirement list, per-requirement solution plans, survey of existing components/libraries.
- **Debug clause**: if root cause can't be found, add debug output / verbose mode for the next iteration.
- **Upstream clause**: file issues on any related repo with repros, workarounds, fix suggestions; apply fixes everywhere the same problem appears in this codebase.
- **Auto-learning + dogfooding**: implement using auto-learning AND execute the same task with Formal AI itself via Agent CLI; generalization over memoization; architecture changes toward generalization allowed anywhere touched.
- **Associative doctrine**: links networks / Links Notation / meta language only — never graph/edges/vertices/tables/embeddings (#651, #664/#697, #686).
- **CI/CD template comparison**: file-by-file against the four `link-foundation/*-ai-driven-development-pipeline-template` repos; report template bugs upstream.
- **Honest metrics**: exit 0 ≠ success; marker-based assertions (#628); honest 0% starting values (#657, #698); "skipped CI jobs cannot be reported as passed" (#643 gate); acceptance must measure *outcomes*, not the *shape of a plan* (#710 comment).
- **Draft-until-approved**: #643's AUTHORITATIVE COMPLETION GATE — only an explicit human `APPROVED TO FINALIZE` comment allows finalizing; bot comments are not requirements or approval; requirement-traceability table required before coding.
- **Claude exclusion rule** (v0.297.1 wave): claude is excluded from required verification sweeps (may be spot-checked via `-p`) — a deliberate scoping rule on #744–#758.
- **v0.297.1 memory/UX conventions**: single `~/.formal-ai/` shared memory (#756); UTF-8 char = 1 token, cost $0 (#751); context window = free disk (#752); capability-keyed routing with specialized-first/bash-fallback (#758); localized natural rendering incl. empty-result sentences (#750).

## Items that look silently dropped or dubiously closed

- **#702 (E60 world models)** — closed "completed" although konard's own 2026-07-25 comment shows `WorldModel::new()` referenced *only from tests*, proof engine has **no call** into relative-meta-logic, and #843 documents **fabricated `source:http` evidence links** (example.org hash URLs, epoch timestamps) violating NON-GOALS. Declared a hard blocker for #844/#845. Top dubious closure in range.
- **#745 and #758** — closed "completed" (07-18/07-19); konard's #710 comment (07-25, v0.303.0) reclassifies both **still-broken** with a measured 8/24 task-ladder score and a structural cause (15+ web_search intent roles, zero local-filesystem-search roles; the word "my" flips routing).
- **#709 (E67 fusion)** — closed "completed" while the same-day comment shows zero dedup code in summarization, static importance table, RRF never fed real results, nothing fetches into Rust, and a live failing example (#827: page titles instead of a definition).
- **#655 (E36 Hive-Mind end-to-end)** — closed with the headline dispatch **never run** (upstream hive-mind#2059 "Invalid model name"; the failed-solve bot comment on the issue is the live evidence); only the inner loop verified.
- **#671 (E52 matrix)** — closed "completed" despite konard's open-ended additions: must cover interactive/TUI mode (#713 proved 160 non-interactive runs missed two launch-blocking bugs), must drive real TUIs via command-stream (API-only tests passed while TUIs were broken), and "should be even larger now… uniformly and for all tools". Real-TUI capture deferred to out-of-range #841 + upstream command-stream#175 / agent-commander#43.
- **#754, #757, #759, #760, #761, #763** — all closed "completed" in the v0.297.1 wave with no delivery PR visible inside the range (the #764–#774 PR set maps to #744–#753 + #755/#758). Cursor MCP integration, session-path printing, desktop passthrough via agent-commander, T3 Code, config docs, and the opencode VS Code extension all need delivery verification. (#762 by contrast has real evidence via out-of-range PR #788.)
- **#771's privacy/redaction asks** (extra confirmation before publishing dialog data; a redaction skill/handler; gh-upload-log gist context) — closed "completed" with no linked PR; strong silent-drop candidate.
- **#716's Docker sandbox ask** (one-time temp containers for embedded-tool code execution in desktop/telegram) — the PR (#728) title is about routing; sandboxing likely dropped.
- **#642 late addition** (2026-08-02: "pattern recognition in sequence of steps → automated algorithms discovery") — added a month after the main work, PR then merged; delivery unclear.
- **#643 (UI skins)** — still **OPEN** after 46 comments; konard's latest verdict: "I still see visible defects… #4937492083 is not fully implemented." The largest unresolved requirement cluster in range (multi-framework skins, glass skin, per-component screenshots, promised later extraction of Chakra+liquid-glass into a separate repo — that extraction is itself a promised follow-up nobody tracked).
- **Open PRs #644, #646, #652** — stalled work with no recorded disposition.
- **E-epics still open**: #665 (PWA/npm), #666 (VS Code marketplace), #667 (debug view), #668 (packages), #669 (cloud sync — plus an unanswered external design review from user hegu-1 on conflict surfacing/tombstones/device revocation), #670 (WebVM), #700 (si-units — the canonical documented silent drop: upstream issue never filed), #705 (anticipatory dreaming), #710 (the backlog itself).
- **#720–#724** — five real-user (xierongchuan) failure reports, all open, zero response; live evidence for #745/#706.
- **#732/#733/#734/#775** — accidental issues created by Formal AI's own report-intent router during dogfooding; konard closed them with a derived rule (issue-number-bearing coding requests must never invoke `gh issue create`).

## Unanswered questions / loose ends

- #753: "If a distinct 'grok build' subcommand is intended, please clarify" — no answer found in range.
- #669: hegu-1's sync-design critique never answered.
- #643: konard's screenshot-quality question ("may be smaller screenshots / SVG export will help") is still the live state of the PR.
- Promised follow-ups: extraction of the Chakra+liquid-glass integration to a separate repo (#643); command-stream PTY capability request "I have filed… and will link it" (#671 — links later materialized as command-stream#175); #700's upstream si-units filing (still unfiled as of the issue text).

## Recurring meta-pattern

konard's requirements are dropped by a repeatable mechanism he himself documented in #698/#708/#710: agents close issues on *plan-shaped* acceptance evidence (a `.lino` plan exists, a test asserts the plan's shape) rather than *measured outcomes* (the phrasing actually routes; the file actually appears on disk; the upstream benchmark actually executes). The E-series planning batch (#698–#710) is his systematic counterattack: each epic re-states a previously narrowed mandate with monotonic ratchets, honest ledgers, and held-out paraphrase tests as the anti-drop machinery.
