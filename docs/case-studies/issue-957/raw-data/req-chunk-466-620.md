# Requirement audit — items #466–#620 (link-assistant/formal-ai)

Scope: 155 issue/PR numbers (2026-06-13 → 2026-07-04, plus a late #534 comment 2026-07-22).
Sources: 53 konard-authored issues, 76 konard-numbered PRs (mostly agent-drafted delivery reports),
110 genuine human comments by konard (out of 506 comments under his login — the rest are hive-mind
automation posts: "Solution Draft Log", "Working session summary", "Ready to merge", "Auto-merged").
Extracted: **137 requirement entries** in `req-chunk-466-620.ndjson`.

## 1. Recurring standing clauses (konard's boilerplate contract)

These blocks are pasted, near-verbatim, on most issues in the range. Recorded once per issue in the
NDJSON with `scope: "standing"`:

1. **Case-study clause** — compile all issue data to `./docs/case-studies/issue-{id}`, deep analysis
   with online research, enumerate every requirement, propose solution plans, survey existing
   components/libraries. (#476, #479, #482, #483, #488, #492, #493, #498, #499, #511, #523, #526,
   #527, #531, #538, #540, #541, #546, #548, #550, #552, #554, #557, #558, #563…)
2. **Single-PR / no-deferral clause** — "Please plan and execute everything in this single pull
   request, you have unlimited time and context… until each and every requirement fully addressed,
   and everything is totally done." Escalated forms: "Nothing can be deferred or delayed", "no
   refusals, no delays, no deferral, no follow ups, no fake solutions" (PR #601), "We do it here and
   now. No need for delay" (PR #525).
3. **Debug-output clause** — if root cause can't be found, add debug output + verbose mode for the
   next iteration. (#479, #492, #523, #541, #546, #550, #561)
4. **Upstream-reporting clause** — report issues to any related repo with reproducible examples,
   workarounds, fix suggestions; fix the same problem in all places in this codebase.
5. **CI/CD template clause** — compare full file tree against the four link-foundation
   ai-driven-development-pipeline templates (js/rust/python/csharp); report shared bugs upstream.
   (#479, #492, #523, #561; actually exercised upstream only once: rust template #85 from #492)
6. **Generalization clause** (the most consequential): "We must generalize to all similar
   questions/requests (the whole class) in all languages, and we must use our general and universal
   meta algorithm and actual recursive reasoning steps, expressed in meta language. Everything must
   be expressed recursively through meanings (meta language), and all meanings must be grounded in
   external data sources. Every finest detail must be tested." (#484, #485, #493, #495–#497, #500,
   #501, #505–#508, #521, #535, #556, #571, #600)
7. **Ambition clause** — "Make sure to be ambitious, take into account our vision, requirements,
   roadmap (the latest have stronger weight), contributing guide lines, testing guidelines."
   (#482, #498, #499, #540)
8. **Agent-CLI-drivability clause** — "Ideally this task itself must be fully and partially solvable
   by Formal AI connected to Agent CLI. Your task is to drive Formal AI to make all the actions; if
   something fails, you improve algorithms by generalization." (#482, #498, #499, #540, #538, #564)
9. **Final-QA sweep** — "ensure all changes are correct, consistent, validated, tested, documented,
   logged and fully meet each and all discussed requirements in deepest and widest possible sense…
   list each and every requirement before checking… scope is the entire repository." (#512, #560, #564)
10. **Documentation-requirement block** (new in the #602–#608 cluster) — an issue is not complete
    until copy-pasteable, end-to-end-verified client-configuration docs ship.

Standing engineering rules established inside this range:

- **No hardcoded natural language** anywhere: triggers and responses come from seed data / meanings,
  naturalized per language; enforced by CI guards (`check-web-hardcoded-ui`, worker-mirror check,
  total-reference-closure). Born on PR #525, hardened on PR #512 and #587.
- **JS only interfaces with Rust**; all logic in Rust (WASM Web Workers, no UI blocking). (#525, #538, #587)
- **File-size caps**: no code file > 1500 lines; data files also ≤ 1500 lines; data lives in
  `./data` / `./data/seed`, never embedded in code. (PR #587; Rust 1000-line guard predates)
- **No force pushes — "we don't edit history."** (PR #525)
- **Reasonable per-test and whole-suite E2E timeouts.** (PR #525)
- **Never wire host claude/codex into Formal AI** (subscription safety); autonomous execution only in
  the isolated container; Agent CLI only through agent-commander. (#468, #511; CI guard in E4)
- **Implement-as-is even under disagreement**: "while we don't agree with requirements we still
  should by default implement them as is." (#468)
- **Associative doctrine**: never "graph"; vertices and edges are links; sequences over sets;
  meta-theory as basis; LLMs "never at the steering wheel" (#483, #531, #560).
- **Bun is the default JS bundler**, Chakra UI + JSX + styles-in-JS for the web app (forced through
  on PR #551 "no matter the cost").
- **Development method itself** (from #538): all code reading/editing driven through Agent CLI with
  a Formal AI server; refusal-anti-pattern.md is required CONTRIBUTING reading.

## 2. Major threads and how they resolved

- **#479 desktop releases** — 5 PRs (#480, #486, #487, #490, #510), four konard rejections
  ("По прежнему не исправлено… скриншоты macOS фейковые", "linux app is not available still",
  "Still no release for macOS"). Root causes were three coupled CI bugs + an electron-builder-26
  signing skip. Eventually delivered.
- **#488/#489 deep thinking** — three "apply it deeper / entire codebase" rounds; delivered as
  `src/thinking.rs` + all surfaces. One sub-ask likely lost: *thinking on TOP of the message* and a
  *separate Telegram thinking message updated with 1–5 s debounce* (PR shipped an expandable
  blockquote AFTER the answer) — flagged `unclear` (R488-4).
- **#511/#512 agent mode** — cleanly decomposed into E1–E8 (#513–#520) after konard demanded the
  sub-issues be really created via `gh`; upstream loop (agent#271→#272, agent-commander#39/#40)
  fully closed. The best-executed epic in the range.
- **#538/#601 "make meanings more detailed" + Agent-CLI self-hosting** — the range's biggest fight.
  Six escalating konard comments ("That is opposite of my requirements", "That looks fake",
  "The issue is not fully done"). Final merged state is an honest partial: delivered R1–R9 (lexeme
  detail), R13 (self-AST census), R15/R16 (generated diagrams), R22–R25 (Agent-CLI authorship, clean
  copy reproduction, CI e2e); explicitly NOT built: hardcoded-string audit (R10), Rust→WASM worker
  widening (R11/R12), CST/AST→Rust round-trip (R14), interactive debug view (R17), universal meta
  algorithm (R18/R19), contradiction detection (R21). konard himself acknowledges the shortfall by
  opening #558 ("deeply analyze what went wrong at PR #601, and why my requirements from #538 were
  not fully delivered") — so despite the no-deferral doctrine, #538 closed only partially delivered.
- **#559/#560 generalize meta algorithm** — plan-first process konard approved ("Ok, now go and
  implement it all"); mid-PR he inverted the plan's non-goals into goals (remove all specialized
  handlers NOW). Merged with the method registry as sole dispatch authority and the legacy mapper
  deleted.
- **#550/#551 Chakra migration** — agent tried twice to defer (staged multi-PR; then a claimed CSP
  architectural block). konard: "I see nothing is blocking our transition as system architect. Just
  do it already, don't make up excuses… Use bun bundler… no matter the cost." Agent retracted its
  claim (both premises false) and shipped the full migration. Also: hive-mind#1964 was konard's own
  misfiled issue/PR to be closed after incorporation.
- **#509/#529/#590/#597 memory queries** — two escalations ("Not only history queries — all
  queries", "My requirements are ignored 2-nd time") before Turing-complete NL read+write
  (link-cli-style substitutions) shipped in #597.
- **#602–#608 + #606/#615/#620 protocol cluster** — konard-account (agent-drafted) issues with crisp
  acceptance criteria; all seven delivered by PRs #609–#615 within days; #620's gemini defect fixed
  by #623 (outside range) and independently re-verified.

## 3. Items that look silently dropped

- **R468-5** — testing on public AI benchmarks (1–2 tasks each) — never surfaced again.
- **R471-2** — asking link-foundation/si-units to publish Rust + JS libraries — no filing evidence;
  the "support exactly all measuring units" generalization (R471-1) also unverified.
- **R473-1** — reporting meta-language mixed-grammar gaps upstream — agent explicitly declined
  ("the missing piece was in formal-ai"); konard never acknowledged, recorded as `rejected`.
- **R546-2** — use link-foundation/start + command-stream for command execution — PR #547 used a
  host runner and filed no upstream issues; adoption not evidenced.
- **R505-1 (partially)** — translating all search results into meta language, merging, and
  deformalizing into the target language — PR #588 shipped only intent routing.
- **R506-1 (partially)** — event extraction/dedup/multi-source and add-to-calendar (Apple, Google,
  Microsoft) — PR #589 shipped only routing.
- **R501-1 (partially)** — official-docs steps parsed through meta language — #587 prefers official
  docs but the meta-language parse/deformalize pipeline is not evidenced.
- **R534-2** — the promised hive-mind sccache issue ("we can report an issue for all Rust
  projects") — no filing evidence.
- **R479-2 (half)** — "All our templates also should include" the landing/app/docs/download CI/CD
  structure — done for formal-ai, no evidence of the upstream template work.
- **R620-2** — documenting gemini-cli's headless no-tools limitation — raised, no in-range delivery.

## 4. Issues whose delivery landed OUTSIDE this range (verified via prs_meta)

These closed "completed" with no delivering PR inside #466–#620; cross-checking `prs_meta.json`
shows each WAS later delivered by a merged PR beyond the range (entries updated to
claimed-delivered with the later PR as followup_ref):

- **#482** Nemotron training-data tests → PR **#639** (2026-07-13)
- **#494** free-space policy → PR **#645** (issue-540 branch, 2026-07-12)
- **#498 / #499** Google Trends tooling → PRs **#640 / #641** (2026-07-09)
- **#527** generate-all-questions → PR **#638** (2026-07-07)
- **#531** patterns inference / Doublets.Sequences in Rust → PR **#642** (2026-08-03)
- **#534** repo disk usage → PR **#860** (2026-07-28); the promised hive-mind sccache issue filing
  remains unverified
- **#540** dreaming daemon → PR **#645** (2026-07-12)
- **#558** auto-learning loop → PR **#637** (2026-07-05)

Every PR numbered 466–620 in `prs_meta.json` is MERGED (none abandoned), which supports the
claimed-delivered labels.

Still **open** (correctly, not dubious): #483 (small-model fallback), #491 (principle of least
action), #557 (embedded input buttons + skins).

## 5. Unanswered konard questions / dangling threads

- **#534 body**: "Why is that? What is the root cause? Do we have some tests that eat disk without
  cleanup? How much is our repository in size? Can we reduce Rust compilation size?" — no visible
  answers; konard partially self-answered a month later with the sccache suggestion.
- **PR #575**: the agent asked konard whether to proceed despite 43 pre-existing local-demo E2E
  failures shared with `main` — no konard reply in-thread (PR merged anyway).
- **PR #525**: agent asked "flat per-language phrase list or meanings ontology?" — konard replied
  only with the general "we do it here and now" directive; the specific design question was decided
  by the agent (operation-vocabulary pattern).

## 6. Notable tensions

- The **memoization vs generalization** tension is visible throughout: many "unknown prompt" issues
  (#467, #477, #462-era, #574, #576, #577) were closed with *seeded single facts* — precisely the
  memoization konard's doctrine forbids — and konard responded by escalating the generalization
  clause into its long canonical form (#497 onward) and eventually the structural-reasoning
  requirement (#571/#618, #493/#619).
- The **no-deferral doctrine vs honest partial delivery** collided head-on in PR #601; the outcome
  (refusal-anti-pattern.md + "honest status labels" in the requirement matrix) is itself now a
  standing convention.
- Comments under konard's login are ~80% automation; auditors must filter hive-mind boilerplate
  before treating "konard said" as authoritative.
