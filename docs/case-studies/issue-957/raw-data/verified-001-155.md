# Delivery verification — requirements from issues/PRs #1–#155 (216 records)

Verified against the current codebase at `main` (v0.326.0, de61602f). Every record in
`req-chunk-001-155.ndjson` received a verdict with concrete evidence; results in
`verified-001-155.ndjson`.

## Verdict counts

| Verdict | Count |
| --- | --- |
| DELIVERED | 159 |
| PARTIAL | 50 |
| NOT-DELIVERED | 5 |
| OBSOLETE | 2 |
| UNVERIFIABLE-LOCALLY | 0 (5 DELIVERED verdicts carry a `handcheck` runtime note instead) |

The high DELIVERED rate is real, not rubber-stamped: most of konard's #1–#155 demands
that looked dropped inside the range were delivered by later work — desktop app
(`desktop/`), lino-i18n adoption, the «Сосал?» factual answer (`src/solver_handlers_policy.rs`
+ `chat_surface.rs:479`), the #140 report-compression spec (`GITHUB_URL_MAX_LENGTH=8192`,
`fitIssueUrl`, omitted-lines truncation), the #144–#149 batch (unknown_opener.rs,
behavior_rules chat editing, capabilities.rs, issue_146.rs, meta_explanation.rs LLM answer,
"Test passed"), and the closed-issue re-audit itself (#123 meta-demand → issue-710 raw-data
reports).

## NOT-DELIVERED + untracked → new issues (4 proposed, covering 5 records)

1. **Telegram compile-before-answer pipeline (#8)** — R8-1, R8-2, R8-4 (one issue).
   No compile/execute path in `src/telegram*.rs`; `environments.lino` telegram tools exclude
   execution; no timeout-halving retry; no 10-minute hard fail. konard allowed
   interface-first "for now" but nothing tracks the pipeline.
   Proposed: "Telegram surface: compile and run code examples before answering (docker
   execution pipeline from issue #8)".
2. **Local WebSocket+WebRTC server (#107)** — R107-5. Zero WebSocket/WebRTC code in `src/`;
   `network_endpoint.rs` is plain HTTP. Proposed: "Local WebSocket+WebRTC memory server with
   the CLI acting as both server and client (issue #107 follow-up)".
3. **Per-language docker-box project testing (#119)** — R119-5. Box DinD is only the CI
   runtime image; no legs run generated projects inside language-matched box images.
   Proposed: "Exercise generated language projects inside matching link-foundation box
   images in CI (issue #119 follow-up)".

Plus two PARTIALs severe enough to flag needs_issue:

4. **≥5-variations-per-language CI enforcement (#103/#123/#134)** — R123-1 (and weakens
   R96-2, R103-1, R134-4). `check:language-parity` / `check:intent-coverage` /
   `check:language-test-coverage` enforce language *presence*, and
   `prompt_variations.rs` hand-implements 5–10×4 matrices for major categories, but no CI
   rule enforces the variation minimum per test case — the exact check konard asked for
   twice.
5. **Finish the Rust→WASM port (#133/#134)** — R134-2 (and caps R133-6, R14-1). 24 JS
   worker modules (`src/web/worker/formal_ai_worker_00..23.js`) still hold substantial
   non-UI logic (capability answers, fact queries, search fusion mirrors). Only
   `scripts/check-worker-line-budget.rs` ratchets growth; no open issue tracks completing
   the port konard explicitly declared in-scope when rejecting the R194 deferral.

## PARTIAL landscape (50)

Clusters, with the open tracker where one exists:

- **Data-as-the-AI / anti-memoization doctrine** (R12-2, R12-3, R14-2, R20-2, R27-11,
  R104-2, R133-7): real seed-driven routing exists (117 seed files, intent-routing.lino),
  but the minimal-core boundary is exactly open epic **E71 #918**; #892 (stale Spider-Man
  seed) proves residual hardcoded facts.
- **Universal-solver depth** (R1-4, R13-2, R115-1, R115-3): substantial artifacts
  (solver.rs, proof_engine, meta-algorithm.md) — the remaining gap is what epics
  **#922/#923** track.
- **Coverage/quality ratchets** (R119-6, R134-4, R153-9): tracked by **#895**.
- **Doublets-in-browser** (R16-4, R103-7): native surfaces use doublets-rs; the browser
  falls back to an "indexeddb-lino-mirror" unless a DoubletsWeb global is injected
  (`memory.js:178-186`) — doublets-web is not actually bundled.
- **Never-adopted dependencies** (R1-9, R16-5, R2-1, R1-11): command-stream, link-cli,
  react-chat-ui (library), browser-commander (e2e) were all quietly substituted
  (doublets-rs direct, custom UI→Chakra, Playwright). Substitutions look permanent and
  accepted-in-practice; none has an acknowledgment on record.
- **Misc**: R63-2 (no ≥99% round-trip-translation metric), R82-2 (no explicit
  preferred-location setting; only inferred tz/locale), R27-3/R112-5 (no check pinning
  that ALL code-supported cases appear in Example prompts — the exact #112 complaint),
  R129-2 (hosted web-capture proxy promise untracked), R133-5 (≤5-parallel-provider cap
  not found), R4-2/R4-4 (upstream template filings → #894), R96-3/R68-1 corpus growth
  (→ #891), R8-3/R8-5 (code execution beyond browser eval_js; WebVM → #670),
  R12-11/R27-10 (docker/WebVM sandboxed agent execution → #670), R17-2 (no dynamic
  Rust/JS compilation), R103-8 (rules don't map to dynamically compiled code), R12-12
  (code→NL direction unpinned), R12-13 (no automatic meaning-splitting test), R14-5
  ("do more than asked" never codified).

## OBSOLETE (2)

- R112-1 — iOS input-accessory bar removal: PR #113 documented it as non-removable
  platform UI, mitigations shipped; konard accepted.
- R150-1 — a clarification question, closed not_planned when the reporter never replied.

## Hand-check list (runtime verification recommended)

- R1-14 — live Pages e2e (`test:pages`) actually green against the deployed URL.
- R8-7 — Telegram bot behavior in a real group chat.
- R108-5 — real-device mobile keyboard-focus layout.
- R128-1 — deployed demo answers capital-of for an uncommon country from live wikidata.
- R133-1 — DuckDuckGo live availability (reported flaky in #153; #801/#821 open on search).

## Surprising discoveries

1. **The #123 "double-check all closed issues" meta-demand was eventually executed** — the
   issue-710 case study contains full closed-issue audit reports (1–350 and 351+), and E68
   #710 remains open for fixes. In-range it was silently dropped; out-of-range delivered.
2. **The #140 URL-compression spec was implemented almost literally** (8192 documented
   limit, safety margin, last-messages-first retention, "omitted X lines/characters"
   labels, merged UI-languages field) in the shared `src/issue_report.rs` + web mirror —
   despite #140 being closed with no in-range PR.
3. **The desktop app finally exists** (Electron, `desktop/`, auto-updater, permission
   panel) — the #1 requirement that looked dead through the whole range.
4. **konard's «Сосал?» correction (#39) did get honored** later: a dedicated policy handler
   answers factually without lecturing, with a pinning test.
5. **The biggest structural residue of the range is the JS worker**: the browser still
   ships ~24 hand-written JS logic modules besides the WASM core — the exact
   "logic duplicated in JavaScript" konard flagged in his last #134 comment — and nothing
   tracks finishing that port.
