# Runtime hand-check suite (issue #955, E103)

The 2026-09 audit (`handchecks.ndjson`, 49 rows) produced findings that
static review cannot confirm: they need a deployed demo, a real device, a
live GitHub Action, an upstream tracker, or a maintainer decision. This
file is the checklist those rows became, worked through per the issue's
honesty rule: **an item is checked off only with recorded evidence;
unchecked items stay visibly open.**

Row grammar (the runner below parses exactly this):

```
| <check-id> | <requirement> | <the live/manual action> | <status> |
```

`status` is one of:

- `pending` — nobody has run it yet; this is the honest default,
- `pass — <evidence link>` — ran, worked, evidence linked,
- `fail — <tracking issue>` — ran, broken, follow-up filed,
- `n/a — <reason>` — the surface no longer exists; the reason names it.

## Checks

| check | requirement | live action | status |
|---|---|---|---|
| HC-01 | R1-14 | GitHub Pages demo published and e2e-tested in PRs and against the deployed URL | pending |
| HC-02 | R8-7 | Telegram bot answers 1:1 private messages and public group chats (live bot) | pending |
| HC-03 | R108-5 | Mobile UI input-section/top-bar layout; real-device keyboard focus behavior | pending |
| HC-04 | R128-1 | Capital-of-country answers for an uncommon country resolve live via Wikipedia/Wikidata/Wiktionary, not a hardcoded table | pending |
| HC-05 | R133-1 | DuckDGo default search engine live availability (was flaky in #153) | pending |
| HC-06 | R171-2 | FRAME_POLICY_CHECK_ENDPOINT reachable from the deployed GitHub Pages app | pending |
| HC-07 | R180-5 | Diagnostics mode renders expandable raw HTTP request/response for a live web search | pending |
| HC-08 | R304-1 | External-benchmark suite pass ratios match the recorded ratchet floors on a live run | pending |
| HC-09 | R312-1 | Unseen coding prompts end-to-end vs quoted Gemini/DeepSeek answer quality | pending |
| HC-10 | R331-6 | No expected-output shown before a real verified execution (cross-ref open #905/#908) | pending |
| HC-11 | R353-1 | VS Code extension loads as a web extension in vscode.dev | pending |
| HC-12 | R439-2 | `agent --model formalai/formal-ai` against `formal-ai serve`, compared with `claude -p` JSON output shape | pending |
| HC-13 | R444-3 | Live how-to prompt with network access confirms multi-source guide synthesis | pending |
| HC-14 | R468-5 | Weekly external-benchmarks workflow: honest passed/total rows, ratchet green | pending |
| HC-15 | R520-1 | Upstream agent#271/#272, agent-commander#39/#40 closed with shipped features | pending |
| HC-16 | R534-2 | link-assistant/hive-mind shared-sccache-container request tracked upstream | pending |
| HC-17 | R552-3 | web-capture#141 status; whether formal-ai consumes the meta-language document model | pending |
| HC-18 | R620-1 | with-formal-ai gemini/--global re-verified on a machine with cached Google OAuth (cross-ref #909) | pending |
| HC-19 | R635-2 | Standing single-PR-until-complete clause: process check only | pending |
| HC-20 | R645-2 | Deep-review comment 4939858814 items spot-checked against dreaming_runtime tests | pending |
| HC-21 | R649-3 | relative-meta-logic dependent-statement recalculation: code-call integration status (narrative-only today, proof_engine/mod.rs:179) | pending |
| HC-22 | R651-6 | gh api graphql sub-issue listing for #651 confirms all E-epics linked | pending |
| HC-23 | R687-3 | Manual sweep: every web/desktop UI control drivable via natural language; gap log | pending |
| HC-24 | R671-3 | Streamed-capture depth vs #841 ambitions in a live e2e matrix run | pending |
| HC-25 | R702-2 | WorldModel::new()/proof_engine<->relative_meta_logic wiring gap; confirm narrative-only is acceptable by design | pending |
| HC-26 | R708-1 | Tally the >=15 required NL-memory-query families | pending |
| HC-27 | R716-2 | Desktop/telegram execution lands in a one-shot container when Docker is present; behavior when absent | pending |
| HC-28 | R717-1 | release.yml/desktop-release.yml green-badge state vs the four link-foundation pipeline templates (live Actions) | pending |
| HC-29 | R730-1 | desktop-release workflow failure status (live Actions check) | pending |
| HC-30 | R736-1 | auto-release/desktop-release/docs-generation badge status (live) | pending |
| HC-31 | R745-2 | explain/summarize/translate/fix out-of-box capability probe across languages | pending |
| HC-32 | R747-2 | Desktop+VS Code tool-set enumeration vs the #758 shared list at runtime | pending |
| HC-33 | R753-1 | grok integration functional check; "grok build" subcommand clarification (flag for a direct question) | pending |
| HC-34 | R644-1 | PR #644 open/unmerged state — maintainer closure decision | pending |
| HC-35 | R781-8 | Multi-turn action E2E coverage across opencode/agent/claude/codex | pending |
| HC-36 | R781-10 | Live agentic session shows tool-call explanation before each call | pending |
| HC-37 | R800-1 | Re-run the amazon.in Russian product-search prompt against current HEAD | pending |
| HC-38 | R801-1 | Re-run "Search online for Elon Musk" end-to-end | pending |
| HC-39 | R819-4 | OpenCode re-render bug: confirm upstream report filed | pending |
| HC-40 | R821-1 | "Search for Elon Musk" quality vs Claude/ChatGPT/Google AI-mode | pending |
| HC-41 | R826-1 | Re-run "ФБС vs ФБО" + "Зарепорти баг" against current HEAD | pending |
| HC-42 | R827-1 | Re-run "Что такое фуфломицин?" + anaphora follow-up against current HEAD | pending |
| HC-43 | R841-2 | command-stream#175/#180, agent-commander#43/#46 upstream status; local PTY code deletable yet? | pending |
| HC-44 | R876-1 | Live multi-subagent orchestration with corrective-feedback resume against a provably wrong statement | pending |
| HC-45 | R883-2 | link-foundation/meta-language issue tracker filings from the #883 window | pending |
| HC-46 | R887-1 / R888-1 | Maintainer merge decision on CI-green PRs #887/#888 | pending |
| HC-47 | R904-1 | Confirm whether PR #926 merged remotely after the audit's clone snapshot | pending |
| HC-48 | R912-1 | link-assistant/web-search/web-capture post-#912 upstream filings | pending |
| HC-49 | test-report item 7 | Live "agent --task ignored" regression re-verification after the dedicated fix lands | pending |

## Reading the table

49 rows, one per audit finding; the audit's own identifiers (R…-n) are kept
so the ndjson and this table stay cross-referenceable. Nothing above is
`pass` yet — this suite lands with the audit's state carried forward
honestly rather than with pre-checked boxes. Results are recorded by
editing the status cell and linking evidence (a run URL, a screenshot path
under `docs/case-studies/issue-955/`, or the filed tracking issue); the
runner validates the grammar so a malformed row fails instead of silently
disappearing.
