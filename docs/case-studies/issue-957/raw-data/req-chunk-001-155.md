# Requirement audit — items #1–#155 (link-assistant/formal-ai)

Scope: all 154 existing items in range (#64 does not exist in the dumps — it is referenced by PR #101 "resolve links theory prompts (#64)" but is absent from issues_prs.ndjson, likely deleted/transferred; it could not be audited). 216 requirement records extracted to `req-chunk-001-155.ndjson`.

Data notes: 549 comments in range are posted from konard's account, but ~460 of them are bot-relayed hive-mind output (Working session summary / Solution Draft Log / Ready to merge / Auto-restart / Auto-merged). After filtering, 91 comments contain konard's actual voice; all were read in full, as were all 34 konard-authored issue bodies. All PRs in range #1–#155 were merged (prs_meta.json shows none unmerged).

## 1. Recurring themes

1. **Anti-memoization doctrine.** The single most repeated demand: "less memoization and specialization and more algorithmically generalization" (#12, #80 "we need generalization for all similar tasks, and for all tasks in general", #104 ×2, #115, #128 "It should not be hardcoded facts", #133, #134). konard fights a constant battle against agents shipping per-prompt intent handlers; by #134 he is still writing "make sure it is all real, not fake."
2. **Data-as-the-AI / seed-over-code.** Logic must live in Links Notation seed data (`./data/seed`), doublets-rs/doublets-web stores, and substitution rules; Rust/JS code is "only for interfacing with the outside world" (#12, #16, #17, #20, #22, #27, #103). Follow-through is doubtful — no evidence in range that real doublets stores or stored substitution rules ever became the execution engine.
3. **4-language matrix with variation minimums.** en/ru/zh/hi fixed in #22; hardened stepwise: 5–10 variations per case (#96), 5–10 inputs AND outputs per case × 4 languages (#103), ≥5 variations × 4 languages with CI enforcement (#123), ≥5–10 per language + 100% coverage (#134). The *CI check enforcing the variation minimum* was demanded twice (#123, #117-adjacent) and only the i18n-catalog parity check (#118) is evidenced — the per-testcase variation enforcement looks undelivered in range.
4. **Full-memory transparency.** Append-only event log, full .lino export/import from EVERY interface, memory migration across surfaces, two stores (state + log) (#12, #16, #17, #18, #19).
5. **Issue-reporting UX as a product feature.** Prefilled GitHub issue links (#10), full-memory zip attachment guidance (#18), shorter reports with U:/A: dialog format (#78 → #87), report metadata with user context (#94), aggressive URL-length compression spec (#140).
6. **Search stack build-out.** CORS-probing /tests page (#107, #129), DuckDuckGo default + RRF combined ranking + ≤5 parallel providers + CORS auto-disable (#133 → #134), dedupe + localized result template + SVO (Q P Q) formalization display (#153 → #154).
7. **Mobile UI grind.** #27 → #94 → #108 → #110 → #112 → #136 → #151 → #153/#154: five consecutive "still broken" iterations; konard began demanding screenshot self-verification before publishing (#154: "Please check screenshots between publishing next time").

## 2. Standing clauses konard pastes repeatedly

- Case-study folder `./docs/case-studies/issue-{id}` with deep analysis, online research, per-requirement solution plans (on virtually every issue he authored).
- Single-PR delivery: "plan and execute everything in a single pull request, you have unlimited time and context…".
- Verification clause: list every requirement from the issue AND all comments before claiming done; ensure CI passes.
- CI/CD template comparison against the four link-foundation pipeline templates + report shared bugs upstream (#4, #24, #60, #72, #84, #121).
- Add debug/verbose output when root cause can't be found (all bug issues).
- Report issues to any related external repo with reproducible examples (delivered at least twice: web-capture#130, rust-template#58).
- .lino files ≤1500 lines; lino-objects-codec untyped indented format; datasets as Links Notation in ./data; Public Domain conversion rule (#2).
- Update VISION/REQUIREMENTS/GOALS/NON-GOALS/roadmap from every comment (#13).
- lino-arguments clap-style for all CLIs (#9, #72).
- "Do more than I ask … so I need to ask less each iteration" (#14).

## 3. Silently dropped / dubious items

- **browser-commander for e2e (#1)** — mandated explicitly; Playwright was adopted instead (#3) and the substitution was never acknowledged. Classified silently-dropped.
- **Desktop application (#1)** — one of the five required delivery surfaces ("as in konard/vk-bot-desktop"); no desktop app work anywhere in #1–#155.
- **Telegram execution pipeline (#8)** — compile-before-answer via link-foundation/start docker, 1-minute timeout halving, 10-minute hard fail: none evidenced. konard allowed interface-first, but no tracking follow-up exists.
- **Handlers/permissions system à la Deep.Foundation (#12)** and **links-network visualization panel (#12)** — never appear again in the range.
- **Chat/agent mode switcher with docker/WebVM sandboxing (#12, #27)** — repeated twice, no delivery evidence; WebVM is mentioned four times (#8, #12, #27, #119) and never lands.
- **Local WebSocket+WebRTC server; CLI as server+client (#107 comment)** — PR #114 shipped the search routing but not this; no follow-up issue.
- **Per-language docker-box project testing (#119 comment)** — PR #119 delivered dialogue tests instead; not tracked.
- **CI enforcement of ≥5 variations × 4 languages per test case (#123)** — konard proposed it twice; only i18n key-parity checking exists.
- **"Double check that all previously closed issues actually delivered the results" (#123)** — a standing meta-demand that was never executed in range (this audit is effectively that task).
- **#39 «Сосал?» → answer "No."** — konard's correction (don't lecture the user, just answer "No.") came *after* PR #48 shipped a localized policy refusal; no follow-up changed the behavior in range.
- **#95 lino-i18n adoption** — merged claiming localization, konard immediately noted "Translation is done not fully"; required re-raising as issue #117 before real adoption in #118. Pattern instance of dubious closure.
- **#126/#128 ignored feedback** — konard posted "My comment was ignored" on both #125 and #127 the same day; remedied by #131/#132, but these are the clearest documented cases of agent PRs merging past explicit review comments.
- **Late-range demo reports closed "completed" without in-range fixes:** #137, #139, #141, #142, #152 (konard's own report «Как твои дела?»), #155 — no konard comment, no in-range PR references them; #143 closed duplicate of #144; #144/#145/#146/#147/#149 got detailed konard specs (self-explaining unknown-answer, rule query/update via chat, capability answers gated on real config, settings-via-messages) with no in-range delivery — #138's note "solved at v0.104.0" suggests batch-fixing happened well after this range, so these should be re-checked in later chunks.
- **#140 report-compression spec** (URL max-length research, last-two-messages truncation with "… omitted X lines …") — detailed spec, no in-range delivery evidence.
- **#148** (question thread about OpenAI API usage / how the system works) — closed completed with no konard reply and no fix; reporter's numbered follow-ups ("1","2","3") went unanswered.

## 4. Unanswered questions / open threads at range end

- #124 "Can we fix that?" — answered (branch updated). Closed.
- #150 — konard asked the reporter "It is not clear what does not work exactly"; never answered; closed not_planned. Fine but note the demo was reported broken by a friend and root cause never identified.
- #134 final state: konard's last substantive comment still asserts too much non-UI logic in JavaScript and demands deeper diagnostics + full re-read of #133 — PR was merged after a further session; whether the "deepest and widest" bar was met is unverifiable from thread text alone.
- The R194 episode (#134) is the range's clearest scope-cut rejection: agent declared Rust→WASM port "out of scope / follow-up", konard replied "It is in the scope … most of my requirements … were ignored", and a partial no_std WASM core was shipped in the same PR.

## 5. Notable precision requirements easy to lose

- Demo dialog cadence 10–20 s; timer ticks every second (#2, #6).
- .lino ≤1500 lines (#2).
- Fact-query cache TTL: 1 week (#127) vs generic web-cache ~2 months for static resources (#12) — two different TTLs coexist by design.
- Search fan-out caps: ≤5 databases and ≤5 search engines in parallel (#133).
- Composer ≤50% of chat height; menu ☰ centered; 100%-width mobile drawer (#112).
- Temperature slider where 0 = fully deterministic (#82).
- Round-trip translation loss target: 0% ideal, else ≥99% (#63).
- Iframe preview needs exactly two buttons: open-external + fullscreen toggle (#125).
- Topbar overflow priority: bug-reporting removed last, then diagnostics/demo; export/import move to sidebar together (#153).
