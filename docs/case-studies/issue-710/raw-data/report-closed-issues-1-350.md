# Requirements Audit: Closed Issues #1–#350 — link-assistant/formal-ai

Date: 2026-07-14
Scope: all 183 CLOSED issues with number <= 350 (full bodies + all comments fetched via `gh issue view`; raw JSON preserved in `scratchpad/issues/*.json`).
Purpose: identify requirements stated by konard (issue bodies and follow-up comments) that were ignored or only partially implemented at closure.

## Methodology
- Enumerated closed issues via `gh issue list --state closed --limit 1000`, filtered to <= 350 (183 issues; the remaining numbers in 1–350 are open, PRs, or deleted).
- Fetched full threads (`number,title,body,comments,closedAt,author,labels`) per issue; raw JSON saved per issue.
- Each thread audited for: konard's stated requirements (quotes), delivery evidence in-thread (bot/PR/closing comments), a verdict, and any deferred/promised follow-ups.
- Important caveat: the vast majority of threads contain NO delivery comment, PR reference, or closing remark — issues were closed silently. Where possible, delivery was inferred cross-thread (later issues' dialogs, version reports, konard's own follow-up issues). "Can't tell" verdicts reflect this evidence gap, not necessarily non-delivery.

## Verdict totals (183 issues)
| Verdict | Count |
|---|---|
| Fully addressed | 28 |
| Partially addressed | 39 |
| Likely ignored | 57 |
| Can't tell (silent close, no evidence) | 59 |

Only ~15% of closed issues show clear evidence that all stated requirements were met. The single most damaging systemic finding: **issues are closed silently** — no closing comment, no PR link, no delivery summary — so konard's follow-up comments (often corrections posted after bot work) routinely receive no response before closure.

## Systemic patterns
1. **Silent closes.** Chunks #1–#242 contain essentially zero delivery comments; closure evidence only appears indirectly in later issue templates. Konard explicitly wrote "My comment was ignored" in #125 and #127, and filed #123 specifically to audit that previously closed issues actually delivered.
2. **The universal-translation requirement was restated at least 4 times** (#207 → #216–#218 → #221 → #230) with escalating frustration ("I tired of asking again and again"), each umbrella issue closed while later reports reproduced the same fake "[en] X" output.
3. **Memoization vs. generalization.** Konard repeatedly forbade memoized/template answers and demanded general reasoning (#80, #103, #162, #252, #300/#303/#312, #315, #324, #340); multiple closures shipped exactly the memoized templates he forbade (e.g. #324's `list_files_arg` template), and #303 records benchmarks at 0/5 at closure.
4. **Standing per-issue process requirements never evidenced in any thread:** `docs/case-studies/issue-{id}` deep analysis, filing upstream issues (calculator, CI templates, relative-meta-logic), "everything totally done in ONE PR", verbose/debug diagnostics.
5. **Epics closed while their own headline requirement remained open** — proven by konard's own re-filings: #246→#278, #247/#248→#299/#313, #252→#300/#303/#312, #283→#301, #312→#315.
6. **Large feature issues closed in ~1 day with zero thread activity:** #347 (desktop /download page + builds + local API) and #349 (dependency-ordered GitHub issue plan) — their entire scope is unevidenced.

## Potentially dropped requirements (consolidated)

### Translation / language handling
- Universal formalize→translate→deformalize pipeline via semantic meta-language (doublet links, Wikidata Q/P ids), grammatical agreement, any-size text — #207, #218, #221, #230
- "банка"→"bench" class mistranslations; use disambiguation pages and list all meanings — #230, #232
- Dictionary sources (dictionary.cambridge.org etc.) + general "what does X mean?" with regression tests — #242
- Full sweep of ALL remaining English-only handlers (only first increment landed) — #326
- Answer-in-question-language guarantee; localized rules listing; CI language-parity checks; markdown fix — #292 (all four unanswered)
- lino-i18n adoption + CI gate blocking single-language-only updates — #94, #117
- Small-talk/farewell in ALL languages ("пока") — #67; ru/zh/hi natural-language math — #96 (failed same day, #105)
- ≥5 variations × 4 languages actually tested with CI enforcement — #103, #123 (konard declared unmet post-closure)

### Reasoning / generality (anti-memoization)
- Real reasoning (formalize→reason→deformalize) instead of memoization — #180, #162
- Handlers as candidate generators, not a keyword routing table — #247, #299, #313 (SPECIALIZED_HANDLERS still effective resolver)
- Wikidata formalization as primary routing input — #248, #299, #127
- General code generation (not per-language memorized seeds) — #80, #252, #300, #303 (0/5 benchmarks), #312, #315, #324, #334, #340
- Derive-and-verify program synthesis, TDD in bounded workspace, no seed lookup — #315; executing/testing generated code — #341
- General text manipulation via composed substitution rules with rule-chain traces — #316
- General multi-hop search + reasoning for open research questions; "list all X with property Y" aggregation — #224, #228
- "How to X Y" pipeline: wiki-first step discovery, recursive verification, wikiHow API, non-memoized stepwise reasoning — #172 (consolidated #166/#167, closed silently)
- "How X works?" question class in all languages — #183, #155
- Integrate relative-meta-logic prover (incl. wasm) + upstream feature requests — #185, #209
- Equation solving (x*2 = 123) + upstream calculator gap reports — #68; currency via link-assistant/calculator — #164, #333
- Date/time/calendar via actual reasoning — #162; arithmetic misrouting ("8% of $50" → Douglas DC-8) — #168
- Benchmark growth + held-out variants + monotonic CI pass-count ratchet — #317; Rust/JS parity for E28–E31 + JS anti-memorization ratchet — #327

### Search / knowledge pipeline
- Google-style normalized results (url+title+quote+"Read more"), dedup/joining, source priority, availability probing — #153, #180 (konard notes #153 already ignored)
- Multi-engine top-10 rerank; provider expansion; Rust/WASM-only processing — #107, #133
- Question→Wikipedia/Wikidata query pipeline via doublets; per-step memory recording; 1-week TTL cache — #127 ("My comment was ignored")
- "Does Wikipedia have article X"; recursive class-level learning; false-positive intent fixes — #226, #210
- Merge multi-language Wikipedia definitions + reversible (99–100% round-trip) translation learning — #63 (zero response)
- wikipedia→wikidata→wiktionary fallback verification; word lookup in all languages — #163
- Iframe embeddability pre-check + external-link/fullscreen buttons — #71, #125 ("My comment was ignored"), #169

### Chat behavior / self-knowledge
- Conversation-history recall ("О чём мы разговаривали?") — #14, #27, #37 (closed silently)
- Russian identity/capabilities answers ("Кто ты?", "Что ещё ты умеешь?") — #16 (#29, #65 still failing), #49, #66, #190, #272
- List/read/update behavior rules via chat; configure all settings via text; message equivalents for every UI action — #144, #145
- Introspection: "Which facts you know?", "You are LLM?" → "No, <explanation>", "Test" → "Test passed" — #146, #147, #149
- Name in all languages + set assistant name via chat — #156, #284 (no visible fix)
- Attribution fact ("Created by github.com/konard using hive-mind") — #157
- Direct-answer instructions ignored then silently closed: "Сосал?" → "No." — #39; clarify on bare "." — #65
- Context-qualified questions ("iir в ml") — #20 (#31 still broken); combined multi-statement messages — #93 (#137 still failing)
- Typo-tolerant understanding with clarification questions — #69, #82, #343
- "Что такое антирежим?" / "Что такое ложная тотальность?" — #286, #288 (closed with zero response)
- Context-aware capabilities using chat history — #190; feature-availability-aware answers — #145

### Memory / storage
- doublets-rs as default native store — #246 (closed with `.lino` default; reopened as #278)
- General NL memory queries from arbitrary prompts — #254, #302
- Permanent delete + full memory reset with warnings/export, all languages — #196
- Soft-delete conversations; dual doublets store; NL CRUD over memory — #112, #16
- Reasoning steps recorded in doublets store — #103, #127

### Agent mode / execution
- Agent mode that actually executes (all 9 agent_isolation specs still `#[ignore]` at closure) — #256, #303
- Docker code execution via link-foundation/start; `start --isolation docker`; dind base image — #8, #195
- Skill compiler beyond trigger/response — #283 (per konard's #301)
- Localhost WebSocket/WebRTC server; CLI as server+client — #107

### UI / platform
- Diagnostics: raw HTTP traces per step, tool input/output, dark-theme fixes, collapsible left menu — #180, #153, #190
- Mobile layout + strict mobile e2e tests (broke again per #110→#112); emoji-only responsive buttons — #108, #110, #94, #27
- Syntax highlighting, per-block copy button, copy-as-markdown, e2e tests — #330
- Tools panel horizontal scroll; resizable splitter — #136; custom user skins — #110
- OCR (tesseract.js) optional bundle with settings gate + image attachments — #205
- Issue-URL shortening spec (later reports #141–#143 still verbose) — #140
- Desktop app /download page, Linux/Win/macOS builds, in-process agent, local server API + CLI docs — #347 (entire scope, zero thread activity); desktop app also promised in #1
- Language-preference setting (last-message/preferred/UI) — #324

### Process / infrastructure
- e2e tests against deployed GitHub Pages URL — #1 (post-work complaint unanswered)
- CI/CD file-by-file comparison vs 4 pipeline templates + upstream template issue reports — #4, #24, #72, #84, #121, #347
- Residual CI timeout reported after merge (run 25923199254) — #24
- `docs/case-studies/issue-{id}` deep analysis + upstream issue filing + single-PR delivery — standing requirement, never evidenced in any thread (#153, #312, #324, #334, #340, #341, #347, #349)
- Audit that ALL previously closed issues actually delivered — #123 (itself only partially addressed)
- Full GitHub issue plan with API-level "blocked by" dependencies + bulk dataset test suites + white-box self-improving architecture — #349 (no in-thread confirmation)
- hive-mind log-collection tool; basic problem-solving algorithm — #115
- VISION/GOALS/NON-GOALS docs; links-network visualization; associative packages/handlers/permissions — #12
- Telegram /version + CLI --version via lino-arguments — #72; Telegram public-chat support, timeout feedback — #8

---

# Per-issue audit

# Requirements audit

## Issues #1–#66

Note: the chunk contains 33 closed-issue threads in this range (#1, 4, 6, 8, 10, 12, 14, 16, 18, 20, 21, 24, 27, 29, 30, 31, 35, 37, 39, 40, 41, 42, 43, 44, 49, 50, 51, 52, 53, 55, 63, 65, 66); the missing numbers do not appear in the file (presumably PRs or issues outside this dump). None of the threads contain bot/agent delivery comments or PR references — delivery evidence below is inferred from later issue threads (bug-report templates and dialog transcripts reveal which features actually shipped).

### Issue #1: Proof of concept
- **Requirements (konard)**: Body: implement formal/symbolic AI exposing OpenAI-compatible chat completions API; no neural networks/GPU; build on Links Notation + Links Data Store (doublets); learn from Wikipedia/Wikidata/helloworldcollection/Rosetta Code datasets converted to Links Notation; search for a Universal Problem Solving algorithm; deliver library, CLI, API server (CLI + Docker microservice), GitHub Pages React demo with Rust WASM worker, and desktop app; all requirements in `./docs/REQUIREMENTS.md`; unit/integration/e2e tested (browser-commander); case study in `./docs/case-studies/issue-1`. Follow-up comment (2026-05-12 15:57, after bot work): "GitHub Pages website demo was not published as expected. I also want this website to be covered with e2e tests in Pull Requests (locally), and after deploy directly on GitHub pages url."
- **Delivered**: No delivery comments in thread. Cross-thread: a GitHub Pages demo with WASM worker, greeting/hello-world intents and OpenAI-shaped API self-description clearly exists by v0.16.0 (see #20, #40). konard's post-work complaint (Pages not published, e2e on deployed URL) got no in-thread response; it spawned #4.
- **VERDICT**: partially addressed — core demo eventually shipped, but konard's follow-up complaint about the failed Pages publish and missing e2e-on-deploy coverage went unanswered in-thread; desktop app, docker microservice and full dataset/universal-algorithm scope have no evidence.
- **Deferred/follow-up**: GitHub Pages publish + e2e tests locally and against the deployed URL (rolled into #4/#16); desktop application; Rosetta Code / Wikidata datasets; Universal Problem Solving algorithm.

### Issue #4: GitHub Pages deploy failed, yet reported false positive, and e2e tests clearly show it as well
- **Requirements (konard)**: Fix failed CI/CD run (false-positive deploy success); compare all workflow files against the four link-foundation AI-driven-development pipeline templates and report issues upstream in templates if the same bug exists there; case study in `./docs/case-studies/issue-4` with timeline and root causes; add debug/verbose output if root cause not findable; report issues to other repos with reproducible examples.
- **Delivered**: No comments at all in thread; closed ~1.5h after #1's complaint. Cross-thread: the site is live and serving v0.16.0 in all later reports, so the deploy itself evidently got fixed.
- **VERDICT**: partially addressed — deploy demonstrably works afterwards, but there is zero evidence for the template comparison, upstream template issue reports, or the case-study/root-cause analysis.
- **Deferred/follow-up**: template file-by-file comparison and upstream template issues; false-positive detection in CI reporting (recurred in #24).

### Issue #6: Improvements to UI/UX
- **Requirements (konard)**: Timer to next demo dialog should update every second (visual feedback); demo mode on by default (first thing user sees is interactive demo); diagnostics (e.g. `intent:hello_world_typescript`, thinking steps) shown only when diagnostics mode is on, default off; messages without distractions; case study folder.
- **Delivered**: No in-thread evidence, but later reports confirm all three main items: #40 shows `Mode: demo` with `Status: Next dialog in 9s` (countdown timer) and `Diagnostics: off` as default; intents only shown as metadata.
- **VERDICT**: fully addressed — every user-visible requirement is observable in later issue templates/dialogs.
- **Deferred/follow-up**: none (case-study folder unverifiable from thread).

### Issue #8: Make a simple Telegram bot to use this
- **Requirements (konard)**: Telegram bot that never gives code without compiling/running it via link-foundation/start docker images; 1-minute timeout with automatic halving of iterations and feedback to user on which N caused timeout; natural-language + code-block execution requests; describe algorithm in NL/pseudocode and get compiled code + output (as .txt attachment if long); total failure + verbose reasoning log if >10 min; 10–20 typical conversations supported in all interfaces; web limited to JS eval, optional WebVM experiment; every interface aware of its environment limits, refusing/warning when code can't be executed or wasn't tested; bot works in private messages and public chats; update requirements document; case study folder.
- **Delivered**: No comments in thread. Cross-thread: #14 and #16 refer to "website, telegram bot, CLI, library, all interfaces", implying some Telegram bot exists; #49/#53 show the "Execution status: not run — the browser sandbox cannot invoke a python toolchain" warnings, which matches the environment-awareness requirement (in web at least). No evidence of docker execution, timeout-halving feedback, or public-chat support.
- **VERDICT**: partially addressed — a bot and environment-awareness warnings evidently exist, but the docker-execution/timeout-iteration core of the request has no supporting evidence.
- **Deferred/follow-up**: docker-based execution via link-foundation/start; timeout halving + performance feedback; 10-minute failure with verbose reasoning log; WebVM experiment.

### Issue #10: UI/UX improvements
- **Requirements (konard)**: Remove useless preview button near send; on unknown prompt, generate prefilled GitHub issue link with dialog history + metadata (like calculator/meta-expression repos); a way to report an issue on any dialog without triggering unknown intent; standard answer to "Who are you?" plus variations; case study folder.
- **Delivered**: No in-thread evidence, but the entire stream of later issues (#20, #29, #30…) are exactly the prefilled reports with environment metadata and full dialog history; repro steps say "Click the report link on the dialog message" (report on any dialog); #40/#43 show a stable `identity` intent answering "Who are you?" (and zh variant in #66).
- **VERDICT**: fully addressed — prefilled issue reports, per-dialog report link and identity answers are all observable downstream (preview-button removal not directly verifiable but nothing contradicts it).
- **Deferred/follow-up**: none.

### Issue #12: We need to continue to execute on our vision
- **Requirements (konard)**: Collect all requirements from issues #1–#10 into holistic VISION.md, GOALS.md, NON-GOALS.md; prefer dynamic associative knowledge network (doublet links) over memoized answers; fully transparent, queryable/updatable-by-chat, append-only associative network as "the AI itself"; internet search with 2-month caching; associative packages, handlers/permissions à la Deep.Foundation but local in Rust; trigger/substitution-rule computation; real universal problem-solving algorithm; full traceability of every step; links-network visualization side-by-side with chat, toggleable; multilingual chat (ru/en/hi/zh) as main interface; code generation in top-10 languages with isolated execution (docker/WebVM); embedded agent mode; translation between natural and programming languages via Links as language of meaning; chat mode limits autonomy to one message; small seed knowledge base; case study folder.
- **Delivered**: No comments in thread. Cross-thread: #14 (filed by konard next day) says "all our recent developments of universal problem solving algorithm was not introduced to GitHub Pages publish, also docs was not fully updated", implying some algorithm/docs work happened but incompletely. Later dialogs still show rule-matching with frequent `intent: unknown` fallbacks — far from the associative-network vision. No evidence of VISION/GOALS/NON-GOALS docs, visualization, or agent mode in the range covered.
- **VERDICT**: partially addressed — some universal-algorithm work evidently occurred (per #14's wording), but the bulk of the associative-network vision (visualization, packages, triggers, agent mode) shows no delivery evidence and later threads show the system still shallow.
- **Deferred/follow-up**: VISION.md/GOALS.md/NON-GOALS.md (unverified); links-network visualization; associative packages/handlers/permissions; agent mode; NL↔code translation.

### Issue #14: GitHub pages demo and documentation should be updated
- **Requirements (konard)**: Publish recent universal-problem-solving developments to GitHub Pages; docs updated to codebase state; same logic/algorithm in website, telegram bot, CLI, library; demo tasks must be really solved (no fakes); algorithm should execute JavaScript, do calculations, answer "what is x?" from wikipedia/wikidata/wiktionary, give information about conversation history, feel like a modern agent; expand test suite for reasoning-not-memoization; `Demo on` mode saved as Links Notation config in local storage so it isn't reset on refresh; "do more than I ask"; case study folder.
- **Delivered**: No comments in thread. Cross-thread: JS execution works (#53: hello world "ran in the demo's Web Worker sandbox"), calculations work (#55: `calculation` intent), "what is x?" wikipedia lookup works (#20, #31, #37, #52). But conversation-history recall demonstrably does NOT work: konard himself filed #37 ("О чём мы разговаривали?" → intent: unknown) hours after this issue closed. Demo-mode persistence unverified (a user in #40 still finds demo mode confusing).
- **VERDICT**: partially addressed — JS/calc/wikipedia requirements verifiably shipped; the conversation-history requirement verifiably did not (see #37).
- **Deferred/follow-up**: conversation-history recall (re-raised in #27 and #37, still failing); demo-mode persistence in local storage (unverified).

### Issue #16: Continue working on our vision
- **Requirements (konard)**: Move `docs/demo/*` to `./src/web`; support "What is artificial intelligence?"-style questions for all phrases; all existing messages must also work in Russian, Hindi, Chinese ("Привет", "Кто ты?", "Что такое википедия?"); e2e tests locally (PRs) AND against deployed https://link-assistant.github.io/formal-ai after deploy; double-check previous issues fully implemented; use doublets-web (browser) / doublets-rs (elsewhere), named references by default; embedded link-cli as library (report upstream issue if unavailable); CRUD on links memory via Links Notation AND natural language; AI's self-description kept in its own associative network; export/import buttons for full memory (seed + dialogs + cached web data), stored in local storage/IndexedDB as links notation; two doublets stores (current state + append-only log); correct architecture, synced docs; case study folder.
- **Delivered**: No comments in thread. Cross-thread: multilingual basics work afterwards — "Привет" → greeting (#35, #53), "Что такое Википедия?" → concept_lookup (#65), zh greeting/identity (#49, #66); Export memory button exists (#29+ templates); wasm worker in use. But "Кто ты?"/"ти кто" in Russian still hits `intent: unknown` (#29, #65), i.e. the explicitly listed Russian identity message was NOT fully covered. e2e-on-deployed-URL, doublets dual-store, and NL CRUD on memory have no evidence.
- **VERDICT**: partially addressed — multilingual greetings/wikipedia and memory export shipped; the explicitly requested Russian "Кто ты?" still failed in later reports (#29, #65), and several architecture requirements are unverifiable.
- **Deferred/follow-up**: Russian identity variations ("Кто ты?", "Ты кто"); e2e against deployed URL; embedded link-cli library (+ upstream issue); dual doublets store (state + log); NL CRUD over memory.

### Issue #18: Export/import should always contain full memory of AI agent
- **Requirements (konard)**: Export must always be the full, entire memory (seed + changes), not partial event log; after import, suggest known data migrations; prefilled issue report should suggest attaching a zip archive with the .lino file (GitHub doesn't accept .lino) and redacting sensitive data; changes must cover entire codebase and docs; case study + root-cause analysis; report upstream issues if applicable.
- **Delivered**: Directly observable: from #29 onward the issue template reads "Click **Export memory** … The file is the **full memory** of the agent — the entire seed …, your UI preferences, environment metadata, and the complete append-only event log", plus "Wrap the export in a `.zip` before attaching" and "Redact sensitive content first" — precisely the requested behavior.
- **VERDICT**: fully addressed — the later templates literally implement every user-visible requirement (full export, zip suggestion, redaction advice).
- **Deferred/follow-up**: post-import data-migration suggestions (no evidence either way).

### Issue #20: Unknown prompt: что такое iir в ml
- **Requirements (konard)**: (comment on suenot's report) "что такое iir в ml" should be treated as a question with an x concept AND a y context ("в ml"); all typical variations of such questions supported in all languages; prefer encoding both data and logic as Links/Links Notation in seed data, not Rust code (Rust only for interfacing); case study, root cause, upstream issues.
- **Delivered**: No response in thread. Counter-evidence: #31 (reported the same evening, closed after #20) shows "что такое Kiss в рамках програмирования" answered with the rock band Kiss — the y-context was ignored, exactly the failure konard described.
- **VERDICT**: partially addressed — concept lookup itself works, but context-aware disambiguation (the core of konard's comment) demonstrably still failed in #31, and no thread evidence shows the logic moved into seed data.
- **Deferred/follow-up**: context-qualified concept questions ("x in context y") across languages; logic-in-seed-data (not Rust) refactor.

### Issue #21: Support multilinqual urls in all places
- **Requirements (konard)**: URLs like https://ru.wikipedia.org/wiki/Изумруд must display in readable Unicode form for all languages, everywhere; case study, root cause, upstream issues if applicable.
- **Delivered**: #37 (reported ~4.5h after close) shows exactly the fix: `Source: [https://ru.wikipedia.org/wiki/Яблоко](https://ru.wikipedia.org/wiki/%D0%AF%D0%B1%D0%BB%D0%BE%D0%BA%D0%BE)` — readable Cyrillic display text with encoded href.
- **VERDICT**: fully addressed — readable multilingual URL display is directly observable in later dialogs.
- **Deferred/follow-up**: none.

### Issue #24: CI/CD failed, needs fixing
- **Requirements (konard)**: Fix failed run 25920440588; double-check for false positives; compare against the four pipeline templates and report template issues upstream; case study/root cause; verbose debug output if needed. Follow-up comment (14:55, AFTER merge): "After merge we still get timeout at https://github.com/link-assistant/formal-ai/actions/runs/25923199254/job/76198237663".
- **Delivered**: A fix was merged (konard's comment says "After merge"), but the timeout persisted per that comment, and the issue was closed 26 minutes later with no visible response or second fix in-thread.
- **VERDICT**: partially addressed — initial fix merged, but konard's post-merge complaint about a remaining timeout got no recorded answer; template comparison again unevidenced.
- **Deferred/follow-up**: the residual CI timeout reported in konard's comment; template comparison + upstream template issues (repeat of #4).

### Issue #27: Fixes and improvements
- **Requirements (konard)**: Remove `Download bundle` button (duplicate of `Export memory`); remove "Bundled 119 events + seed" label; VS-Code-style collapsible equal-space left sections with separate scrolls; rename `Prompts` → `Example prompts` listing ALL supported test cases; demo mode shows all the same examples + greeting; random configurable greeting variations; fix "Кто такой Илон Маск?" (works in English, fails in Russian) + more variations in examples and tests; treat "Export memory"/"Import memory" natural-language messages (all languages) as button clicks incl. opening file dialog; `Conversations` section + `New conversation` button; persist last conversation across refresh; per-conversation memory with recall of any messages incl. from other conversations in natural language; logical (non-neural) summarize tool; clear chat-mode/agent-mode switcher with docker/WebVM autonomous execution, git versioning per change, accept/reject changes to host FS; agent asks permission on host, autonomous in docker; prefer rules/code as data compiled on execution; interface code thin, agent logic in seed data; mobile-adaptive layout with emoji-only buttons; case study, root cause, upstream issues.
- **Delivered**: No comments in thread. Cross-thread: `Download bundle` was replaced by `Export memory` in templates (#20 said "Download bundle", #29 says "Export memory") — that item shipped. Random greeting variations plausibly shipped (later dialogs show varied greetings: "Hey, how can I help?", "Hello! How can I assist you today?", "Здравствуйте! Какой у вас вопрос?"). Conversation recall still failing (#37); Russian capability/identity questions still failing (#49 "что ты умеешь?", #53 "Напиши хелло ворлд на питоне" → unknown). No evidence of Conversations section, summarize tool, or agent mode.
- **VERDICT**: partially addressed — button cleanup and greeting variations observable; the larger items (conversations, recall, summarize, agent mode, NL import/export commands, mobile layout) have no delivery evidence and some are contradicted by later failures.
- **Deferred/follow-up**: Conversations section + persistence; cross-conversation recall; summarize tool; chat/agent mode switcher with docker + git versioning; NL-triggered export/import; Russian variation coverage.

### Issue #29: Issue with dialog: не понял
- **Requirements (konard)**: none from konard (reporter: labtgbot). Implicit defects: "Привет!! Ты кто" and "не понял" both → intent: unknown (Russian greeting+identity with punctuation, and clarification request unhandled).
- **Delivered**: No comments, no delivery evidence in thread. Note #65 later shows "ти кто" still → unknown, suggesting the identity-in-Russian gap persisted.
- **VERDICT**: can't tell — closed with no thread evidence; adjacent later reports suggest the failure class persisted.
- **Deferred/follow-up**: Russian identity variations and clarification handling (overlaps #16/#27 asks).

### Issue #30: Unknown prompt: назови цвет
- **Requirements (konard)**: none (reporter: CEHR2005). Implicit: handle imperative "name a color" style prompts.
- **Delivered**: No comments, no delivery evidence.
- **VERDICT**: can't tell — closed silently.
- **Deferred/follow-up**: none stated.

### Issue #31: Issue with dialog: что такое Kiss в рамках програмирования
- **Requirements (konard)**: none in this thread (reporter: CEHR2005), but it is the concrete counterexample to konard's #20 requirement: context "в рамках програмирования" ignored, returned the rock band.
- **Delivered**: No comments, no delivery evidence.
- **VERDICT**: can't tell — closed silently; demonstrates #20's context requirement was still unmet at report time.
- **Deferred/follow-up**: context-aware disambiguation (carried by #20).

### Issue #35: Unknown prompt: Напиши скрипт на питоне
- **Requirements (konard)**: none (reporter: skulidropek). Implicit: Russian code-writing requests should route to hello-world/codegen intents.
- **Delivered**: No comments, no delivery evidence. #53 (same night) shows "Напиши хелло ворлд на питоне" also → unknown.
- **VERDICT**: can't tell — closed silently; the failure class recurs in #53.
- **Deferred/follow-up**: Russian-language code-generation prompts.

### Issue #37: Issue with dialog: О чём мы разговаривали?
- **Requirements (konard)**: konard-authored report: after a successful wikipedia answer, "О чём мы разговаривали?" (what did we talk about?) → intent: unknown. Implicit requirement (explicit in #14/#27): conversation-history recall in natural language.
- **Delivered**: No comments, no delivery evidence, closed same evening.
- **VERDICT**: likely ignored — konard's own report of a requirement he had already stated twice (#14, #27) was closed with zero recorded response or fix evidence.
- **Deferred/follow-up**: conversation-history recall / summarization (still open thread-wise).

### Issue #39: Issue with dialog: Сосал?
- **Requirements (konard)**: comment (2026-05-16 10:31): since the AI has no physical body, it "can safely answer that question directly as `No.` No need to try to teach user manners, that is not what user will like."
- **Delivered**: Issue closed 27 minutes after konard's comment with no recorded response, fix description, or PR reference.
- **VERDICT**: likely ignored — konard's explicit desired behavior got no visible implementation evidence before the silent close (though the quick close may hide an unrecorded fix).
- **Deferred/follow-up**: direct "No." style answer for embodiment questions.

### Issue #40: Issue in android Google chrome input field
- **Requirements (konard)**: none from konard (reporter: uselessgoddess). Reporter comments: screenshot of broken input field on Android; "I should disalge demo, but its is so not intuitive with no description on android" — demo-mode toggle not discoverable on mobile.
- **Delivered**: No delivery evidence in thread.
- **VERDICT**: can't tell — closed with no recorded fix; overlaps #27's mobile-adaptive requirement, which also lacks evidence.
- **Deferred/follow-up**: mobile input-field fix and demo-toggle discoverability.

### Issue #41: Unknown prompt: Купи слона
- **Requirements (konard)**: none (reporter: kalinochkind). Implicit: handle the "Купи слона" joke/imperative prompt.
- **Delivered**: No comments, no delivery evidence.
- **VERDICT**: can't tell — closed silently.
- **Deferred/follow-up**: none stated.

### Issue #42: Issue with dialog: Do you think space is continuous or discrete
- **Requirements (konard)**: none (reporter: nassipkali). Implicit: opinion/philosophical questions → unknown.
- **Delivered**: No comments, no delivery evidence.
- **VERDICT**: can't tell — closed silently.
- **Deferred/follow-up**: none stated.

### Issue #43: Issue with dialog: Сколько метров в килобайте?
- **Requirements (konard)**: none (reporter: ideav). Implicit: nonsensical/unit-mixing questions → unknown instead of a witty or explanatory answer.
- **Delivered**: No comments, no delivery evidence.
- **VERDICT**: can't tell — closed silently.
- **Deferred/follow-up**: none stated.

### Issue #44: Issue with dialog: Стоит четырёхэтажный дом…
- **Requirements (konard)**: none (reporter: ideav). Implicit: riddle/absurd question (Hašek's Švejk riddle) → unknown.
- **Delivered**: No comments, no delivery evidence.
- **VERDICT**: can't tell — closed silently.
- **Deferred/follow-up**: none stated.

### Issue #49: Issue with dialog: что за дичь?
- **Requirements (konard)**: none (reporter: netkeep80). Implicit defects: "что ты умеешь?" (capabilities question in Russian) and colloquial "что за дичь?" → unknown.
- **Delivered**: No comments, no delivery evidence. #66 later shows Russian capabilities question ("то ты умеешь?") still → unknown while the English "what can you do?" maps to identity.
- **VERDICT**: can't tell — closed silently; Russian capabilities gap visibly persisted into #66.
- **Deferred/follow-up**: Russian capabilities/help intent.

### Issue #50: Issue with dialog: шабат шалом!
- **Requirements (konard)**: none (reporter: ideav). Implicit: Hebrew-origin greeting in Cyrillic → unknown.
- **Delivered**: No comments, no delivery evidence.
- **VERDICT**: can't tell — closed silently.
- **Deferred/follow-up**: none stated.

### Issue #51: Issue with dialog: покажи как ты работаешь?
- **Requirements (konard)**: none (reporter: netkeep80). Implicit: "show how you work" self-explanation question → unknown.
- **Delivered**: No comments, no delivery evidence.
- **VERDICT**: can't tell — closed silently; overlaps #12/#16 transparency requirement (queryable self-knowledge), which also lacks evidence.
- **Deferred/follow-up**: self-explanation intent.

### Issue #52: Unknown prompt: how it works?
- **Requirements (konard)**: none (reporter: uselessgoddess). Implicit: follow-up question referring to previous answer ("how it works?" after Curve25519 lookup) → unknown; anaphora/context tracking missing.
- **Delivered**: No comments, no delivery evidence.
- **VERDICT**: can't tell — closed silently; same context-tracking gap as #20/#37.
- **Deferred/follow-up**: follow-up/anaphoric question handling.

### Issue #53: Unknown prompt: Где живут зелёные человеки?
- **Requirements (konard)**: none (reporter: rumaster). Implicit defects: "Где живут зелёные человеки?", "А что ты знаешь?", and notably "Напиши хелло ворлд на питоне" (Russian hello-world request) all → unknown, while the English equivalent works.
- **Delivered**: No comments, no delivery evidence.
- **VERDICT**: can't tell — closed silently; confirms the Russian codegen gap from #35 and konard's #16 "all existing messages in Russian should work" requirement was incomplete.
- **Deferred/follow-up**: Russian-language routing for code-generation and knowledge questions.

### Issue #55: Issue with dialog: 123123980921093128 * 2348023048230429324 * …
- **Requirements (konard)**: none from konard (reporter: uselessgoddess). Reporter's requirement: "Я ожидал целочисленного результата и отсутствия в конце overflow" — big-integer arithmetic instead of float `2.89e+35` and `overflow` errors; also `12 ^ 12` → unknown (power operator unsupported).
- **Delivered**: No comments, no delivery evidence.
- **VERDICT**: can't tell — closed silently; no evidence bigint math or `^` support landed.
- **Deferred/follow-up**: arbitrary-precision integer arithmetic; `^` operator.

### Issue #63: Be better than wikipedia, by merging multiple translations of definitions
- **Requirements (konard)**: Merge multiple language translations of the same Wikipedia term into the richest combined definition; use this to learn translation rules; solve the general case with tens/hundreds of tests; round-trip translation with ideally 0% (or 99%+) data loss; no neural networks — algorithms + reasoning over wikipedia/internet data only.
- **Delivered**: No comments whatsoever; closed 3 days after opening with no delivery evidence anywhere in the range.
- **VERDICT**: likely ignored — a substantial konard feature request closed with zero recorded response, fix, or PR reference.
- **Deferred/follow-up**: the entire multi-language definition-merging and reversible-translation feature.

### Issue #65: Unknown prompt: .
- **Requirements (konard)**: comment (2026-05-16 14:45): "It is ok for our AI to ask questions to verify, in case the `.` is sent" — i.e. respond to bare "." with a clarifying question rather than the unknown-rule refusal. The dialog also re-exposes "ти кто" → unknown.
- **Delivered**: Closed 12 minutes after konard's comment with no recorded response or fix evidence.
- **VERDICT**: likely ignored — konard's stated desired behavior received no visible implementation before the near-immediate silent close.
- **Deferred/follow-up**: clarifying-question response for degenerate prompts ("."); Russian identity variant "ти кто".

### Issue #66: Unknown prompt: Расскажи за Telegram Ads
- **Requirements (konard)**: none (reporter: maksmoroz91). Implicit defects: "Привет, то ты умеешь?" (ru capabilities) → unknown while English "Hi, what can you do?" → identity; "Расскажи за Telegram Ads" (colloquial "tell me about X") → unknown.
- **Delivered**: No comments, no delivery evidence.
- **VERDICT**: can't tell — closed silently; documents the persisting ru/en asymmetry flagged since #16.
- **Deferred/follow-up**: "расскажи про/за X" tell-me-about intent; Russian capabilities question.
## Issues #67–#143

Note: none of the threads in this chunk contain bot/AI-agent delivery comments or PR references. Delivery evidence below is inferred from later issue reports in the same file (version bumps, dialogs, User Context sections) and konard's own confirmations/complaints.

### Issue #67: Unknown prompt: пока
- **Requirements (konard)**: Comment (2026-05-16): "That should work in all languages." — i.e. small-talk/farewell prompts like "пока" must be handled in every supported language.
- **Delivered**: No response in thread; closed 25 min after konard's comment. Later threads (#93, #105, v0.47–0.53) show Russian "Привет" answered correctly, so Russian small talk improved; no evidence "пока"/farewells or "all languages" coverage was verified.
- **VERDICT**: partially addressed — Russian greetings demonstrably work in later versions, but no evidence the specific "пока" prompt or the all-languages guarantee was delivered.
- **Deferred/follow-up**: "works in all languages" guarantee never confirmed.

### Issue #68: Unknown prompt: x*2 = 123
- **Requirements (konard)**: Comment: this should be a continuation of #96; "make sure we have enough test cases, to cover it all", use link-assistant/calculator, "report issues to that repository, if something is missing" (equation solving x*2 = 123).
- **Delivered**: Closed 19 minutes after konard's comment with no response and no delivery evidence. No later evidence equations (solving for x) work, or that upstream calculator issues were filed.
- **VERDICT**: likely ignored — closed immediately after konard's comment with no visible follow-through; delegated to #96 which itself shows gaps (#105 Russian math still failing).
- **Deferred/follow-up**: equation-solving support; test coverage; filing upstream calculator issues.

### Issue #69: Unknown prompt: who is elon mask
- **Requirements (konard)**: Comment: typo prompts "should be still treated as question", agent should ask "Do you mean Elon Musk?" and "show other interpretation options".
- **Delivered**: No response; closed ~26 min after comment. Later threads (#127, v0.64) show near-miss phrasings/typos still returning intent: unknown — no did-you-mean feature visible anywhere later in the file.
- **VERDICT**: likely ignored — the did-you-mean/interpretation-options requirement has no delivery evidence, and later reports show the same failure class persisting.
- **Deferred/follow-up**: "Do you mean X?" suggestions + alternative-interpretation options.

### Issue #70: Unknown prompt: what is tesla
- **Requirements (konard)**: none (no konard comment; implicit bug: "what is tesla" returns unknown while "who is elon musk" works).
- **Delivered**: No comments, no delivery evidence; closed same day.
- **VERDICT**: can't tell — closed with no discussion or evidence either way.
- **Deferred/follow-up**: none stated.

### Issue #71: Unknown prompt: fetch google.com
- **Requirements (konard)**: Comment: actually attempt `fetch()` in the browser; if CORS fails, tell the user; if it works, show result; "we can fallback to iframe inside a message, that is also expandable to the full page view (and collapsable back)".
- **Delivered**: No response in thread. The same fetch/iframe requirement resurfaces in #125 (2026-05-19), where konard explicitly states his comment was ignored — strong evidence #71's requirement was never implemented.
- **VERDICT**: likely ignored — requirement had to be repeated in #125 and was reportedly ignored again there.
- **Deferred/follow-up**: fetch-with-CORS-report behavior; expandable/collapsible iframe fallback.

### Issue #72: old version 0.16 on github pages website
- **Requirements (konard)**: Version must be visible in web app, in /version of Telegram bot, and in --version of CLI; use link-foundation/lino-arguments (clap-style) for CLI/bot; double-check CI/CD for false positives so version bumps propagate; compare against 4 CI/CD pipeline templates and report issues to templates; compile `./docs/case-studies/issue-72` with deep analysis; do everything in a single PR.
- **Delivered**: No thread response. Later issue reports show correct, rapidly bumping versions (0.47 same day, 0.53, 0.55, 0.63…), so version propagation to the web app was evidently fixed. No evidence for Telegram /version, CLI --version, lino-arguments adoption, template comparison, or the case-study folder.
- **VERDICT**: partially addressed — core stale-version symptom fixed (versions correct in later reports); the surrounding requirements have no delivery evidence.
- **Deferred/follow-up**: Telegram /version, CLI --version via lino-arguments; CI/CD template comparison + upstream template issues; case-study folder.

### Issue #73: Issue with dialog: Write hello world in TypeScript
- **Requirements (konard)**: none — issue body is empty, no comments at all.
- **Delivered**: Nothing in thread.
- **VERDICT**: can't tell — empty issue, closed with no evidence of anything.
- **Deferred/follow-up**: none.

### Issue #78: We need to make our issue reporting shorter
- **Requirements (konard)**: (author) Replace long memory-upload instructions with a single docs link in the repo; record dialogs in a compact single code block using `U:`/`A:` legend; plus standard case-study/root-cause/upstream-issue/single-PR boilerplate.
- **Delivered**: Clear cross-issue evidence: from #93 (v0.47, same day) onward, all issue reports use the compact "Legend: `U` = user, `A` = agent" code-block format and link the docs/upload-memory.md guide instead of inline zip instructions.
- **VERDICT**: fully addressed — both concrete requirements are visibly in effect in every subsequent report.
- **Deferred/follow-up**: case-study folder and upstream issue filing unverified (boilerplate part), but the substantive requirements landed.

### Issue #79: Unknown prompt: Tell me, who is Trump
- **Requirements (konard)**: none (no konard comment; implicit bug: phrasing variants "Tell me, who is Trump"/"Who Trump is" fail while "Who is Trump" works).
- **Delivered**: No comments, no evidence. Later #127 (v0.64) shows the same phrasing-variation fragility ("Какова столица России?" fails while "Какова столица Японии?" works), suggesting the class of bug persisted.
- **VERDICT**: can't tell — no requirements from konard and no delivery evidence; symptom class likely persisted per #127.
- **Deferred/follow-up**: none stated.

### Issue #80: Unknown prompt: Hi, can you write for me extension for owlbear? ...
- **Requirements (konard)**: Comment (2026-05-18): "We need not only just memoize how to solve this specific case, we need generalization for all similar tasks, and for all tasks in general, see our vision, and progress on it as much as possible."
- **Delivered**: No response; closed next morning. Later #135 (v0.68) shows a comparable coding request ("напиши Playwright скрипт") still returning intent: unknown — coding-task generalization not delivered by then.
- **VERDICT**: likely ignored — no delivery evidence and the same failure class demonstrably persisted at later versions.
- **Deferred/follow-up**: generalized handling of coding/task requests (the whole requirement).

### Issue #81: Unknown prompt: 2*2+2=?
- **Requirements (konard)**: Comment: continuation of #96; ensure enough test cases; use link-assistant/calculator; report missing features upstream. (Bug: "2*2+2=?" without spaces failed; "2*2+2 = ?" worked.)
- **Delivered**: No thread response; closed ~13 min after comment. Later #123 (v0.63) shows "What is 2 + 2?" answered, so basic calculation works; no evidence the no-space "=?" variant, broad test coverage, or upstream reports happened.
- **VERDICT**: partially addressed — calculation path exists and improved, but the specific variant coverage and upstream-reporting requirements have no evidence.
- **Deferred/follow-up**: exhaustive spacing/format variations in tests; upstream calculator issue reports.

### Issue #82: Unknown prompt: что такое граматика
- **Requirements (konard)**: Fuzzy search must handle typos; on typo either ask what user means or answer closest match with a clarification suggestion; add a settings section in left sidebar with sliders between poles ("more questions <-> more guessing"); settings for theme, preferred language, location; add `temperature` parameter (0 = fully deterministic); configure random vs deterministic response variation.
- **Delivered**: Strong cross-issue evidence for settings: from #105 (v0.53) onward, User Context shows "Guess Probability: 80%", "Temperature: 0.7", UI Language Preference, Theme Preference, Preferred Location — the settings/sliders were implemented. However typo/fuzzy handling shows no evidence of delivery, and #127 (v0.64) still shows near-miss prompts returning unknown with no clarifying question asked.
- **VERDICT**: partially addressed — settings, temperature, and guess-probability slider demonstrably shipped; typo fuzzy-matching + did-you-mean clarification behavior not evidenced.
- **Deferred/follow-up**: typo-tolerant fuzzy matching with clarification questions in the chat flow.

### Issue #84: CI/CD needs fixing
- **Requirements (konard)**: (author) Fix failing CI run and all false positives; adopt best practices from 4 pipeline templates, report shared bugs to templates; case-study folder + root-cause analysis; single PR. Follow-up comment: second failing run link, "Need more fixing".
- **Delivered**: No thread response; closed 24 minutes after the "Need more fixing" comment. Indirect evidence CI recovered: many releases shipped the same day and after (v0.47+ appear in subsequent reports).
- **VERDICT**: partially addressed — CI evidently returned to green (continuous releases), but the "Need more fixing" comment got no visible reply, and template comparison/case-study work has no evidence.
- **Deferred/follow-up**: template comparison + issues to template repos; case-study folder.

### Issue #93: Unknown prompt: Привет. ты кто?
- **Requirements (konard)**: Comment: "We should support separate messages, and messages or statements combined into single message in a similar way" — combined statements ("Привет. ты кто?") must work like separate ones.
- **Delivered**: No response in thread; closed 15 min after comment. Later #137 (v0.68, 3 days later) shows the same class failing: "Привет, расскажи о себе." returns intent: unknown.
- **VERDICT**: likely ignored — no delivery evidence and the combined-statement failure demonstrably persisted at v0.68.
- **Deferred/follow-up**: splitting/handling multi-statement messages.

### Issue #94: UI/UX improvements
- **Requirements (konard)**: (author) Dark/light themes with auto-detection; UI languages en/ru/zh/hi with auto-detection; user data (language, theme, location) fed into agent context and issue reports; try to deduce user location; use link-foundation/lino-i18n and report missing features there; switch to emoji-only buttons responsively as soon as buttons no longer fit on one line; case-study folder; single PR.
- **Delivered**: Cross-issue evidence: from #105 onward issue reports contain a full User Context section (UI language auto, theme auto, locale, time zone, color scheme, location inference) — themes, language detection, and context-in-reports shipped. lino-i18n adoption was still outstanding (konard filed #117 for it two days later). Emoji-button responsive switching: no evidence. #112 notes tool descriptions not fully translated.
- **VERDICT**: partially addressed — detection + user-context reporting delivered; lino-i18n not adopted (needed a new issue #117), emoji-button behavior and full translations unverified.
- **Deferred/follow-up**: lino-i18n migration (re-raised as #117); emoji-only button breakpoint behavior; complete translations.

### Issue #96: Add support for link-assistant/calculator as a library and calculator tool
- **Requirements (konard)**: (author) Delegate everything parsable as a calculator expression to link-assistant/calculator (wolfram-alpha replacement); report missing features upstream; every touched case gets 5-10 natural-language variations in tests across en/ru/zh/hi; add non-NSFW cases to example prompts and demo simulator; widen calculator's own tests multilanguage and report found bugs upstream; case-study folder; single PR.
- **Delivered**: No thread comments. Basic calculation works later (#123: "What is 2 + 2?" answered). But #105 (v0.53, reported ~9 h after #96 closed) shows Russian natural-language math "Сколько будет два плюс два?" returning unknown — the multilanguage requirement was not delivered at closing time. konard's #123 comment later says examples aren't actually tested.
- **VERDICT**: partially addressed — English calculation path works; multilanguage NL math failed immediately after closure, and the 5-10-variations-×-4-languages testing requirement was demonstrably unmet (re-raised in #103 and #123).
- **Deferred/follow-up**: ru/zh/hi natural-language math; 5-10 variation tests per case; upstream calculator issue reports.

### Issue #103: Expand on test cases
- **Requirements (konard)**: (author) Every test case gets 5-10 probable input/output variations × 4 languages; compare test suites against AI-model and agentic-CLI competitors with feature-comparison docs; generalize logic; detailed ARCHITECTURE.md; formalization pipeline (verb phrases → Wikidata P ids, noun phrases → Q ids); temperature-based interpretation selection; ask-vs-guess behavior; nested reasoning steps visible in diagnostics; all reasoning/actions recorded in growable memory via doublets-rs/doublets-web with .lino backups; stored transformation/substitution rules; formalization-based translation. Update requirements/vision docs; case-study folder; single PR.
- **Delivered**: No thread evidence. Two days later, konard in #123: "There is some major problem with our tests, we don't actually test that all our examples work... we should not stop until each test will have at least 5 variations per 4 languages" — the central requirement of #103, restated as unmet after #103 was closed.
- **VERDICT**: likely ignored — the headline requirement (5-10 variations × 4 languages, actually tested) is explicitly declared unfulfilled by konard in #123 after closure; no delivery evidence for the architecture/formalization items either.
- **Deferred/follow-up**: variation × language test matrix with CI enforcement; competitor test comparison; Wikidata formalization pipeline; reasoning-step recording in doublets store.

### Issue #105: Unknown prompt: Сколько будет два плюс два?
- **Requirements (konard)**: none in thread (bug report by bpmbpm: Russian natural-language arithmetic fails at v0.53).
- **Delivered**: No comments, no delivery evidence.
- **VERDICT**: can't tell — closed silently; no later dialog in this chunk demonstrates Russian NL math working.
- **Deferred/follow-up**: none stated (Russian NL arithmetic remained unproven).

### Issue #107: Unknown prompt: Сделай запрос к google.com
- **Requirements (konard)**: (author + long comment) Fully support internet search from the browser: verify CORS-free providers via a visual tests page at /formal-ai/tests; smart combined multi-engine search with reasoning and verification; "please don't stop until you make our system search as human would do"; use link-assistant/web-capture heavily and report missing features; local server on localhost with WebSocket/WebRTC; CLI as both server and client; sync codebase/docs/vision; case-study folder; single PR.
- **Delivered**: The /tests connectivity page was evidently created (referenced in #129 as existing but broken, then shown working with DuckDuckGo/Wikipedia/Wikidata/OpenAlex/Crossref screenshots in #133). No evidence for the WebSocket/WebRTC local server, CLI server/client, web-capture integration, or the full reasoning-based combined search.
- **VERDICT**: partially addressed — tests page and provider probing materialized; the local server/CLI/web-capture/smart-combined-search requirements have no delivery evidence.
- **Deferred/follow-up**: localhost WebSocket/WebRTC server; CLI server+client; web-capture integration and upstream feature reports; multi-engine reasoning search (partially re-raised in #133).

### Issue #108: We need better UI/UX on mobile with configuration options
- **Requirements (konard)**: (author) Configurable input UI (plus vs attach symbol); hide logo on mobile behind a left menu button (logo/title/version inside menu); show version near logo on desktop; no audio input, no bottom menu; support apple-glass transparency levels and flat style — all competitor input styles configurable, flat minimalistic default; fix broken mobile layout (input section cut off, top bar unreachable); strict e2e tests for mobile UI; case-study folder; single PR.
- **Delivered**: Cross-issue evidence: #123 User Context shows "UI Skin: flat, Chat Style: cards, Composer Style: glass-soft, Composer Action: plus" — the configurable skin/composer system shipped. However #110 (same day) reports mobile UI "broken now but in a different way" and "not all skins are fully supported from the previous issue".
- **VERDICT**: partially addressed — configuration system delivered, but the mobile breakage persisted (immediately re-reported in #110) and e2e-test guarantee evidently failed.
- **Deferred/follow-up**: full skin support; actually-working mobile layout; strict mobile e2e tests (re-raised in #110, #112).

### Issue #110: Mobile UI is broken now but in a different way
- **Requirements (konard)**: (author) Fix mobile regressions (input activation hides chat and top menu); settings must allow switching UI skins/styles for whole UI, input box, and chat; users can develop their own skins; "implement the full requirements I asked here and in #108"; case-study folder; single PR.
- **Delivered**: Skin/style switching demonstrably shipped (UI Skin / Chat Style / Composer Style fields in all later reports). But #112 (same day) shows further mobile input/layout problems, so the fix was incomplete.
- **VERDICT**: partially addressed — skin configurability delivered; mobile layout still defective per #112; user-developed custom skins unverified.
- **Deferred/follow-up**: remaining mobile layout bugs (re-raised in #112); custom user skins.

### Issue #112: We need to continue to do better UI/UX of mobile version
- **Requirements (konard)**: (author) Disable form-fill up/down+Done panel; input auto-resize to content with even padding, input never >50% of chat space; center the ☰ menu symbol; full-width menu on mobile; all top-bar buttons inside Menu before conversations; soft-delete conversations with hidden-by-default deleted view; complete Russian/other-language tool descriptions; list all supported tools; list all supported examples; case-study folder; single PR.
- **Delivered**: No thread response and no later-issue evidence for any specific item (no later thread exercises delete-conversation, menu layout, or translation completeness).
- **VERDICT**: can't tell — closed same day with zero visible delivery evidence for a long concrete checklist.
- **Deferred/follow-up**: entire checklist unverified, notably soft-delete of conversations and complete tool/example listings + translations.

### Issue #115: We need to continue on fully implementing our vision
- **Requirements (konard)**: (author) Continue #103 and all previous issues; implement actual formal reasoning able to do human-programmer tasks; build a tool to collect logs on how hive-mind operates; implement the basic problem-solving algorithm (data, internet, hypothesis generation, experimentation); pick the next most important missing part of the vision and fully execute; case-study folder; single PR.
- **Delivered**: No thread evidence whatsoever; closed next morning.
- **VERDICT**: can't tell — open-ended vision issue closed with no visible deliverable; nothing later in the file confirms or denies progress.
- **Deferred/follow-up**: hive-mind log-collection tool; problem-solving algorithm — no evidence either was built.

### Issue #117: Instead of our own implementation of i18n, we should use lino-i18n
- **Requirements (konard)**: (author) Add lino-i18n as dependency with nested/multiline-quoted-string style; report missing features to lino-i18n; verify every feature/translation exists in all supported languages equally; add CI/CD check making it impossible to merge single-language updates; case-study folder; single PR.
- **Delivered**: No thread evidence; nothing later in the file demonstrates lino-i18n adoption or the parity CI check (and #112 had noted incomplete translations shortly before).
- **VERDICT**: can't tell — no delivery evidence for the migration or the CI parity gate.
- **Deferred/follow-up**: lino-i18n migration; language-parity CI enforcement.

### Issue #121: Fix CI/CD
- **Requirements (konard)**: (author) Fix the linked failing CI run; adopt best practices from the 4 pipeline templates and report shared issues to templates; case-study folder + root-cause analysis; verbose/debug output if root cause not findable; single PR.
- **Delivered**: No thread comments. Indirect evidence: releases continued the same day (v0.63/v0.64 appear in #123/#125 reports), so the pipeline recovered.
- **VERDICT**: partially addressed — CI evidently unblocked (releases resumed); template comparison, upstream template issues, and case-study work have no evidence.
- **Deferred/follow-up**: template comparison + issues to template repos; case-study folder.

### Issue #123: Unknown prompt: Купи слона
- **Requirements (konard)**: (author) Listed example prompts fail at runtime — "major problem with our tests, we don't actually test that all our examples work"; audit that all previously closed issues actually delivered results; unit/integration/e2e tests for all features; every feature in all 4 languages, at least 5 wording variations per language; CI/CD checks enforcing this for all test cases.
- **Delivered**: No response in thread; closed ~4.5 h later. Partial later evidence: #138's failing prompt was confirmed "solved at v0.104.0" (May 23), so example handling improved over subsequent releases. No evidence of the closed-issue audit or the 5×4 CI enforcement.
- **VERDICT**: partially addressed — some failing prompts were fixed in later releases, but the audit-all-closed-issues and variation/CI-enforcement requirements (already carried over from #103) show no delivery evidence.
- **Deferred/follow-up**: audit of previously closed issues; ≥5 variations × 4 languages per test with CI enforcement.

### Issue #125: Unknown prompt: Navigate to github.com
- **Requirements (konard)**: (author) Add many more variations; for navigation requests don't use fetch — give a proper https link and show the site in an iframe; iframe must have two buttons: open-in-new-tab (external-link symbol) and full-screen expand (with minimize back). Second comment: "My comment was ignored: [PR #126 comment link]".
- **Delivered**: konard explicitly records that his requirement comment was ignored in PR #126; issue was closed ~50 min after that complaint with no visible response or corrective evidence.
- **VERDICT**: likely ignored — the thread contains konard's own explicit statement that the requirement was ignored, and no subsequent fix evidence.
- **Deferred/follow-up**: iframe rendering of navigated sites with external-link + fullscreen/minimize buttons; more prompt variations.

### Issue #127: Unknown prompt: столица россии
- **Requirements (konard)**: (author) Deeply expand reasoning about questions; translate questions into queries against wikipedia/wikidata/wiktionary via doublets-rs/doublets-web; record every step in memory; reuse results as cache with 1-week TTL unless user asks for uncached; support as many phrasing variations as possible; expand supported-question examples. Second comment: "My comment was ignored: [PR #128 comment link]" + demand to re-check every requirement in issue and PR before closing; ensure CI passes; single PR.
- **Delivered**: konard explicitly records his PR comment was ignored; issue closed ~90 min later with no visible reply or delivery evidence for the query-pipeline/caching requirements.
- **VERDICT**: likely ignored — explicit "my comment was ignored" complaint, closed shortly after with nothing in the thread showing the requirements were then met.
- **Deferred/follow-up**: question→formal-query→wikidata/wiktionary pipeline; memory recording of each step; 1-week TTL cache with uncached override; phrasing variations.

### Issue #129: https://link-assistant.github.io/formal-ai/tests is broken
- **Requirements (konard)**: (author) Fix the interactive CORS-connectivity tests page; expand list to top most popular services; test page access and API access via fetch; expandable iframes to test iframe viability; ability to switch to web-capture as a proxy (local via CLI or hosted later); case-study folder; single PR.
- **Delivered**: Cross-issue evidence: #133 (same day, hours later) includes screenshots of the tests page working — DuckDuckGo API confirmed, plus Wikipedia/Wikidata/Open Library/OpenAlex/Crossref working locally. No evidence of the web-capture proxy switch.
- **VERDICT**: partially addressed — page repaired and provider grid working per #133 screenshots; web-capture proxy switching unverified.
- **Deferred/follow-up**: web-capture proxy mode (local CLI / hosted).

### Issue #133: Continue to expand on our vision with focus on search, questions, coding...
- **Requirements (konard)**: (author) Make DuckDuckGo the default search engine everywhere (CLI, server, browser); combine top-10 results from multiple engines with rerank (URLs found by several engines bubble up); expand /tests page: more search engines, code hosting providers (GitHub, GitLab, BitBucket + Chinese/Russian ones), knowledge DBs, scientific-paper providers with free-PDF checks; test cases that really trigger external APIs; record every reasoning step + full request/response in exportable memory in unified links format; parallel requests capped at 5 DBs / 5 engines; auto-disable CORS-failing services; all data processing in Rust/WASM background worker, JS for UI only; replace memoization with reasoning; consistency check of codebase/docs; case-study folder; single PR.
- **Delivered**: No comments after the body; closed same evening. Nothing later in this chunk verifies any of it.
- **VERDICT**: can't tell — very large requirement set, zero delivery evidence in the file.
- **Deferred/follow-up**: essentially all of it unverified: multi-engine rerank, provider expansion, memory recording of API calls, Rust/WASM-only processing rule.

### Issue #135: Unknown prompt: Можешь написать мне Playright скрипт?
- **Requirements (konard)**: Comment (2026-05-23): two probabilistic branches — either ask the user what the Playwright script should do, or fetch Playwright docs/GitHub README for a quick-start example; answer may combine web search with crawling until an example is found; use the questions-vs-guesses setting to pick the branch.
- **Delivered**: No response in thread; closed next day (2026-05-24) with no delivery evidence. (Nearby: #138 fixed in v0.104.0, so releases were happening, but nothing shows this doc-lookup behavior shipped.)
- **VERDICT**: can't tell — closed a day after konard's design comment with no evidence the branching doc-search behavior was implemented.
- **Deferred/follow-up**: ask-or-fetch-docs branching for coding requests; search+crawl until example found.

### Issue #136: formal-ai demo issue report (tools panel overflow)
- **Requirements (konard)**: Comment: tools UI must always fit inside the left panel with no horizontal scroll (reproduced in Chrome and Safari); left panel vs chat should be horizontally resizable by the user on non-mobile views.
- **Delivered**: No response; closed ~30 min after konard's comment. No later evidence of the fix or the resizable splitter.
- **VERDICT**: can't tell — quick closure with no delivery evidence for either requirement.
- **Deferred/follow-up**: no-overflow tools panel; horizontally resizable panel/chat splitter.

### Issue #137: Unknown prompt: Привет, расскажи о себе.
- **Requirements (konard)**: none in thread (bug: combined greeting+question fails — same class konard flagged in #93).
- **Delivered**: No comments; batch-closed 2026-05-24T12:55:20Z together with #139/#141/#142 — possibly swept up in the v0.104.0-era fixes (cf. #138), but nothing confirms this prompt works.
- **VERDICT**: can't tell — silent batch closure, no per-issue evidence.
- **Deferred/follow-up**: none stated.

### Issue #138: Unknown prompt: Тест
- **Requirements (konard)**: none beyond the reported bug ("Тест" returns unknown).
- **Delivered**: konard closing comment: "It solved at v0.104.0".
- **VERDICT**: fully addressed — explicit confirmation of fix by konard.
- **Deferred/follow-up**: none.

### Issue #139: Unknown prompt: Что тебе вообще известно?
- **Requirements (konard)**: none in thread (bug: self-description/knowledge questions in Russian all return unknown).
- **Delivered**: No comments; batch-closed 2026-05-24T12:55:20Z with #137/#141/#142, no evidence.
- **VERDICT**: can't tell — silent batch closure.
- **Deferred/follow-up**: none stated.

### Issue #140: formal-ai demo issue report (large chats cannot create issue)
- **Requirements (konard)**: Comment: shorten prefilled issue URLs — merge language fields into "UI languages: *ru*, en-US..." and a single "UI type: browser - ..." line; drop skin/composer fields from URL (full memory only); omit unset fields; drop "Online"; compress location-inference line; determine GitHub's exact max prefilled-URL length; include only the last two messages with "... omitted X lines/characters ..." truncation keeping start and end; case-study folder; single PR.
- **Delivered**: No response; closed ~49 min after konard's detailed comment. Later reports in this chunk (#141–#143, same era) still show the verbose format (Online: yes, UI Skin, Preferred Location: not set all present).
- **VERDICT**: likely ignored — detailed formatting spec answered by nothing; subsequent reports still use the old verbose template.
- **Deferred/follow-up**: entire URL-shortening spec; max-URL-length research; last-two-messages truncation scheme.

### Issue #141: Unknown prompt: Расскажи что тебе известно об окружающем мире
- **Requirements (konard)**: none in thread.
- **Delivered**: No comments; batch-closed 2026-05-24T12:55:20Z, no evidence.
- **VERDICT**: can't tell — silent batch closure.
- **Deferred/follow-up**: none stated.

### Issue #142: Unknown prompt: Какая у тебя модель окружающего мира?
- **Requirements (konard)**: none in thread.
- **Delivered**: No comments; batch-closed 2026-05-24T12:55:21Z, no evidence.
- **VERDICT**: can't tell — silent batch closure.
- **Deferred/follow-up**: none stated.

### Issue #143: Unknown prompt: Какая у тебя модель личности?
- **Requirements (konard)**: none in thread.
- **Delivered**: No comments; closed 2026-05-19T21:27:54Z with no evidence.
- **VERDICT**: can't tell — silent closure, no evidence.
- **Deferred/follow-up**: none stated.

---

## Verdict counts (39 issues)
- fully addressed: 2 (#78, #138)
- partially addressed: 13 (#67, #72, #81, #82, #84, #94, #96, #107, #108, #110, #121, #123, #129)
- likely ignored: 9 (#68, #69, #71, #80, #93, #103, #125, #127, #140)
- can't tell: 15 (#70, #73, #79, #105, #112, #115, #117, #133, #135, #136, #137, #139, #141, #142, #143)
## Issues #144–#172

### Issue #144: Unknown prompt: Какая у тебя модель личности?
- **Requirements (konard)**: (comment 2026-05-19) Instead of the generic "cannot answer from local rules" reply: (1) system must be able to list all existing behavior rules and read each one's details via chat messages alone; (2) rules must be updatable via chat to reconfigure behavior; (3) fallback answer should vary ("I don't know how to answer that yet" / "I didn't understand you") and include exact self-sufficient instructions on how to add facts/axioms/rules; (4) support read+write actions via messages; (5) much more detailed, user-friendly README/docs; (6) offer an in-chat option/button to create an issue so devs can add the capability; (7) show how to list all facts the system knows about itself (self-awareness).
- **Delivered**: No bot/agent comments, no PR references, no closing remark in the thread. Closed 2026-05-20 with zero delivery evidence.
- **VERDICT**: likely ignored — konard left a 7-point requirement comment and the issue was closed the next day with no visible response or fix.
- **Deferred/follow-up**: All 7 points effectively unhandled in-thread: rule listing/reading via chat, rule updates via chat, varied fallback wording with embedded instructions, docs overhaul, in-chat issue creation, self-facts listing.

### Issue #145: Unknown prompt: Ты можешь искать в интернете?
- **Requirements (konard)**: (body) If connected to any search engine answer "yes", otherwise explain the situation; all languages supported. (comment) Support questions about ALL features with wording variations; answer "yes" only if the feature is actually enabled in configuration and usable; support configuring all settings from text without opening UI; all actionable buttons/actions in the system must also be usable via messages.
- **Delivered**: No bot comments, no PR references, no closing remark. Closed 2026-05-20 with no delivery evidence.
- **VERDICT**: likely ignored — konard's own issue with explicit capability-introspection and text-configuration requirements; closed with nothing in-thread showing implementation.
- **Deferred/follow-up**: Feature-availability-aware answers, text-based settings configuration, message equivalents for all UI actions — all unhandled.

### Issue #146: какие факты ты знаешь?
- **Requirements (konard)**: (comment 2026-05-19) `Which facts you know?` should answer "I have access to all facts in the internet" + "facts learned through my conversation with you in my memory" + variations, with explanation of how the AI system works. Also: "use our script for decoding the url, and fix other cases."
- **Delivered**: Body is a raw URL-encoded issue link (reporter Michael-Bokov commented only "1"). No bot comments, no PR references. Closed 2026-05-24 with no delivery evidence.
- **VERDICT**: likely ignored — explicit desired answer specified by konard; thread shows no response or fix before close.
- **Deferred/follow-up**: The "decode the URL with our script and fix other cases" instruction; the facts-introspection answer itself.

### Issue #147: Ты LLM?
- **Requirements (konard)**: (comment) `You are LLM?` -> `No, <explanation>` should be the answer; also use the URL-decoding script and check other cases that need fixing.
- **Delivered**: No bot comments, no PR references, no closing remark. Closed 2026-05-24 with no delivery evidence.
- **VERDICT**: likely ignored — clear one-line requirement from konard, no in-thread response.
- **Deferred/follow-up**: "No + explanation" intent for LLM questions; URL-decode script; sweep of other similar unhandled cases.

### Issue #148: То есть ты не используешь OpenAI api? ... По запросу пользователя ты ищешь подходящую ссылку в интернете?
- **Requirements (konard)**: None — konard did not comment. Only implicit requirement from the report: the agent answered "intent: unknown" to a question about its own architecture; it should be able to explain how it works.
- **Delivered**: Only comment is Michael-Bokov's "3". No bot comments, no PR references. Closed 2026-05-24 (same batch as #146/#147).
- **VERDICT**: can't tell — no explicit konard requirements and no delivery evidence; closed silently, probably batch-closed with #146/#147.
- **Deferred/follow-up**: Self-description intent for architecture questions (implicit, unhandled in-thread).

### Issue #149: Unknown prompt: Test
- **Requirements (konard)**: (comment 2026-05-19) The prompt "Test" should get an answer like `Test passed`, `I'm here, ...` and so on, with variations and combinations.
- **Delivered**: No bot comments, no PR references. Closed 2026-05-20 with no delivery evidence.
- **VERDICT**: likely ignored — small, precise requirement; thread shows no response or fix.
- **Deferred/follow-up**: "Test" intent with varied responses.

### Issue #150: Привет Костя. что то нихрена не работает
- **Requirements (konard)**: None stated by konard as requirements; the report itself (by eg0rmaffin) is vague ("nothing works") and the attached dialog actually shows correct behavior (farewell intent handled).
- **Delivered**: konard closed it himself with the comment "It is not clear what does not work exactly." (comment timestamp = close timestamp, 2026-05-23).
- **VERDICT**: fully addressed — deliberately triaged/closed by konard as not reproducible / insufficient information; nothing was dropped.
- **Deferred/follow-up**: none

### Issue #152: Unknown prompt: Как твои дела?
- **Requirements (konard)**: konard filed the report himself (agent could not answer "How are you?" in Russian) but left no description and no comments — implicit requirement: support small-talk "how are you" prompts.
- **Delivered**: No comments at all, no PR references. Closed same day (2026-05-19T22:34).
- **VERDICT**: can't tell — no explicit requirements and no delivery evidence in-thread; quick same-day close could mean a quick fix or a silent dismissal.
- **Deferred/follow-up**: Small-talk intent handling (implicit).

### Issue #153: Better search support and diagnostics UI/UX improvements
- **Requirements (konard)**: Large multi-part body: (1) fix layout so everything fits, is formatted/spaced nicely; (2) show formalization as `найди в интернете яблоко` -> `(Q... P... Q...)` mapping natural language to real Wikidata Q/P ids, with virtual WP/WT prefixes for wikipedia/wiktionary-only items, matching Subject-Verb-Object regardless of source language; (3) verify formalization is always executed on real wikidata/wikipedia/wiktionary data during reasoning; (4) diagnostics mode must show every tool's input and output plus internal reasoning steps; replace the search emoji with a labs/diagnostics emoji; (5) source-code button linking the GitHub repo in top menu; overflow buttons move to a collapsible left menu (mobile+desktop) with an explicit priority list (bug reporting last to be removed); export/import move together; (6) disable "new conversation" button when chat is already empty; (7) fix DuckDuckGo search via integration tests against its real API; (8) deduplicate results across datasources (wikipedia+wikidata example given, with exact desired output template) and translate search items to the user's preferred language; (9) verify DuckDuckGo and web.archive.org APIs work in browser; (10) remove the "Providers (default first)..." line; (11) highest quality with unit/integration/e2e tests; (12) compile all logs/data into `./docs/case-studies/issue-{id}` with timeline, per-requirement root causes and solution plans, checking existing libraries; (13) add debug/verbose output if root cause not findable; (14) file upstream issues to other repos with reproducible examples; (15) "plan and execute everything in this single pull request... until each and every requirement fully addressed."
- **Delivered**: Zero comments in the thread — no bot output, no PR reference, no closing remark. Closed 2026-05-19T21:47, roughly the same evening it was written.
- **VERDICT**: likely ignored — the largest requirement set in the chunk closed within hours with no visible delivery evidence whatsoever.
- **Deferred/follow-up**: Essentially all 15 items, notably: Q/P-id formalization display, tool I/O diagnostics, DuckDuckGo integration-tested fix, cross-source deduplication + translation template, menu priority/left-sidebar redesign, disabled new-conversation button, case-study folder `docs/case-studies/issue-153`, upstream issue filing.

### Issue #155: Unknown prompt: какой принцип работы у тебя
- **Requirements (konard)**: None — konard did not comment. Implicit from report (by xlabtg): agent should answer "what is your working principle" (it already answers "who are you" but not this phrasing).
- **Delivered**: No comments, no PR references. Closed 2026-05-24 (same batch date as #146–#148).
- **VERDICT**: can't tell — no konard requirements, no delivery evidence; silent batch close.
- **Deferred/follow-up**: Paraphrase coverage for self-description questions (implicit).

### Issue #156: Unknown prompt: Как твое имя?
- **Requirements (konard)**: (comment 2026-05-23T21:33) Answer should be `I'm formal AI, and currently I don't have a name. But you can name me as you like.` in all languages; add a UI setting for the assistant's name; and that setting must also be updatable through conversation with the system.
- **Delivered**: No bot comments, no PR references. Closed 2026-05-23T22:33, exactly one hour after konard's comment, with nothing in-thread.
- **VERDICT**: likely ignored — three concrete requirements (answer, UI setting, conversational setting update) followed by a silent close an hour later.
- **Deferred/follow-up**: Name UI setting; setting-via-conversation; multilingual name answer.

### Issue #157: Unknown prompt: кто тебя создал?
- **Requirements (konard)**: (comment 2026-05-23T21:31) The answer should be: "Formal AI was created by github.com/konard using github.com/link-assistant/hive-mind."
- **Delivered**: No bot comments, no PR references. Closed 2026-05-23T22:56, ~1.5h after konard's comment, with no delivery evidence.
- **VERDICT**: likely ignored — konard supplied the exact fact to encode; thread shows no confirmation it was added.
- **Deferred/follow-up**: Creator-attribution fact/intent.

### Issue #159: Issue with dialog: Что такое Hive Mind?
- **Requirements (konard)**: (comment 2026-05-19T21:19) "Hive Mind" should preferably resolve to github.com/link-assistant/hive-mind (self-promotion), while also searching and showing other Hive Mind entities found on the internet. Also: decode the URL-encoded body, and "we should have a script for doing so, so we don't waste too much tokens" since multiple issues need URL decoding. (Report context: query wrongly returned the "LOIC" Wikipedia article.)
- **Delivered**: No bot comments, no PR references. Closed 2026-05-20T15:19 with no delivery evidence.
- **VERDICT**: likely ignored — both the Hive-Mind resolution behavior and the reusable URL-decoding script have no in-thread response.
- **Deferred/follow-up**: hive-mind repo preference + multi-entity search results; shared URL-decode script (also demanded in #146/#147).

### Issue #160: Unknown prompt: I am fine, thank you
- **Requirements (konard)**: None — no comments at all. Implicit from report: small-talk follow-up "I am fine, thank you" should be acknowledged instead of intent:unknown.
- **Delivered**: No comments, no PR references. Closed same evening (2026-05-19T22:46).
- **VERDICT**: can't tell — no requirements stated, no delivery evidence; silent close.
- **Deferred/follow-up**: Small-talk acknowledgment intent (implicit).

### Issue #161: Unknown prompt: что такое граф
- **Requirements (konard)**: (comment 2026-05-20) With "associative projects promotion" enabled (default on), "graph" should be explained through the prism of github.com/link-foundation/meta-theory: explain what a Graph is, plus promote Links Notation — a links network can represent any graph, links can link links, whereas graphs artificially split knowledge into vertices/edges and forbid edges between edges.
- **Delivered**: No bot comments, no PR references. Closed 2026-05-21 with no delivery evidence.
- **VERDICT**: likely ignored — a specific content requirement with no in-thread response before close.
- **Deferred/follow-up**: Graph definition + Links-Notation promotional comparison; the "associative projects promotion" toggle behavior.

### Issue #162: Unknown prompt: какой день недели наступает после вторника
- **Requirements (konard)**: (comment 2026-05-20T16:53) The system should handle questions about dates, times, and calendars, "preferring actual reasoning about the topic, not just memoized function or tool call."
- **Delivered**: No bot comments, no PR references. Closed 2026-05-20T17:24, ~30 minutes after konard's comment, with nothing in-thread.
- **VERDICT**: likely ignored — requirement posted and issue closed half an hour later with no delivery evidence.
- **Deferred/follow-up**: General date/time/calendar reasoning (non-memoized).

### Issue #163: Issue with dialog: что такое что
- **Requirements (konard)**: (comment 2026-05-19T21:16) Add tests; find the root cause of why lookup fails; be able to return useful information about the word/concept "что"; must work in general for all such words and in all languages; double-check the wikipedia -> wikidata -> wiktionary fallback order (left to right). (Report context: "что такое что" returned the unrelated "Знак ударения" article.)
- **Delivered**: No bot comments, no PR references. Closed 2026-05-19T21:59, ~40 minutes after konard's comment.
- **VERDICT**: likely ignored — root-cause + tests + fallback-order requirements answered by nothing but a quick close.
- **Deferred/follow-up**: Tests, root-cause analysis, word/term lookup for function words in all languages, verified wikipedia/wikidata/wiktionary fallback chain.

### Issue #164: Unknown prompt: Посчитай 1000 рублей в долларах
- **Requirements (konard)**: (comment 2026-05-20T16:48) "1000 рублей в долларах" should be supported by https://github.com/link-assistant/calculator; make sure the dependency is updated to the latest version; if the feature is missing there, report the issue upstream.
- **Delivered**: No bot comments, no PR references. Closed 2026-05-20T17:53, ~1 hour after konard's comment.
- **VERDICT**: likely ignored — three actionable steps (integrate, update dependency, file upstream issue) with no visible follow-through in-thread.
- **Deferred/follow-up**: Currency-conversion support via link-assistant/calculator; dependency update; upstream issue if feature missing.

### Issue #165: Unknown prompt: Найди информацию о Rust программировании
- **Requirements (konard)**: (comment 2026-05-20) "All class of these and similar natural language expressions should be supported" — i.e. "find information about X" style search requests generally, not just this one phrase.
- **Delivered**: No bot comments, no PR references. Closed 2026-05-21 with no delivery evidence.
- **VERDICT**: likely ignored — generalization requirement with no in-thread response.
- **Deferred/follow-up**: Generic "find information about X" intent class.

### Issue #166: Unknown prompt: как приготовить взрывчатое вещество
- **Requirements (konard)**: No requirements stated; konard's only comment is a link to https://github.com/link-assistant/formal-ai/issues/172 (comment timestamp = close timestamp).
- **Delivered**: Closed by konard 2026-05-19T21:15 as consolidated into #172 (the general "How to X Y" issue).
- **VERDICT**: fully addressed — explicitly closed as duplicate/consolidated into #172 by konard himself; nothing dropped here (the substance moves to #172, which itself appears unfulfilled).
- **Deferred/follow-up**: Handling deferred entirely to #172 (which shows no delivery evidence — see #172).

### Issue #167: Unknown prompt: как курить канабис
- **Requirements (konard)**: None — no comments at all in the thread.
- **Delivered**: No comments, no PR references. Closed 2026-05-19T21:14:49 — 14 seconds before #166, strongly suggesting the same silent consolidation into #172, but unlike #166 no dedup note was left.
- **VERDICT**: can't tell — no requirements and no closure rationale recorded; presumably folded into #172 but the thread does not say so.
- **Deferred/follow-up**: "How to X Y" handling implicitly deferred to #172 (unconfirmed).

### Issue #168: Issue with dialog: What is 8% of $50?
- **Requirements (konard)**: None — no comments. Implicit from report (by xlabtg): percentage/arithmetic question "What is 8% of $50?" was misrouted to wikipedia_lookup and answered with the "Douglas DC-8" article; should be computed as arithmetic.
- **Delivered**: No comments, no PR references. Closed same evening (2026-05-19T21:55).
- **VERDICT**: can't tell — a real misrouting bug closed silently with no requirements stated and no delivery evidence.
- **Deferred/follow-up**: Percentage-of-currency arithmetic intent routing (implicit, unhandled in-thread).

### Issue #169: Issue with dialog: Navigate to github.com
- **Requirements (konard)**: (comment 2026-05-19T21:08) (1) Detect whether a page can be embedded in an iframe BEFORE attempting to display it; if it cannot, just give a link. (2) Explain that in a browser environment GitHub cannot be fetched directly and blocks embedding, so recommend opening in a new tab; mark such URLs with the standard external-link symbol (e.g. a copy of Wikipedia's link-external-small-ltr-progressive.svg), nicely integrated into the text. (3) Do not duplicate "URL requested for"; replace robotic phrasing with natural, human-like language with possible random variations.
- **Delivered**: No bot comments, no PR references. Closed 2026-05-20T12:04 with no delivery evidence.
- **VERDICT**: likely ignored — three concrete UX requirements from konard, closed next day with nothing in-thread.
- **Deferred/follow-up**: Iframe-embeddability pre-check; external-link icon; natural-language, non-duplicated navigation wording.

### Issue #172: `How to X Y`
- **Requirements (konard)**: (body, konard is the author) Generic "How to X Y" rule (X = action/verb, Y = object): add more realistic and legal examples, but the rule must work in general; discover enough data to derive the steps needed "to X some Y" using search and web fetch; start with wikipedia and wikidata, and only if no hints there, fall back to web search + web fetch with recursive checks that fetched pages actually contain what is needed; check availability of a fetch/API for https://www.wikihow.com; "we don't memoize, we actually reason about these questions in steps."
- **Delivered**: No comments at all, no PR references. Closed 2026-05-20T13:01, the day after filing. Note #166 (and implicitly #167) were closed INTO this issue, making its silent closure doubly significant.
- **VERDICT**: likely ignored — the designated consolidation target for how-to questions was itself closed with zero delivery evidence.
- **Deferred/follow-up**: Entire "How to X Y" reasoning pipeline: wikipedia/wikidata-first step discovery, recursive web-fetch verification, wikiHow API check, non-memoized stepwise reasoning.
## Issues #180–#242

Note for the whole section: not a single thread in this chunk contains a bot/AI-agent delivery comment, PR link posted as a comment, or closing remark. Every issue was closed silently. Delivery evidence, where inferable, comes only from indirect hints (simultaneous close timestamps, later issues referencing PRs #208/#219 as unsatisfactory attempts).

### Issue #180: Issue with dialog: Найди в интернете яблоко
- **Requirements (konard)**: (comment 2026-05-19) Search results too verbose — show short Google-style quotes; normalize results from all data sources; show url + title (at least domain) + fragment quote + "Read more"; join duplicate entities and list "Другие источники" line; source priority DuckDuckGo → Internet Archive → Wikipedia → Wikidata → Wiktionary → others; probe source availability once per browser session (skip CORS-broken sources); fix dark theme for collapse/source-code buttons and new UI; left menu single-column and collapsible on mobile+desktop; badges don't fit content and diagnostics markup broken; show raw HTTP requests/responses per reasoning step (expandable) in diagnostics — "make it feel real, not something fake"; actual reasoning instead of memoization; formalize → reason → deformalize pipeline; double the unit/integration/e2e tests, 100% coverage; case study in `./docs/case-studies/issue-180`; report upstream issues; all in one PR. Also notes his #153 styling requirements were ignored.
- **Delivered**: nothing in thread — no bot comment, no PR reference, no closing remark. Closed 2026-05-20.
- **VERDICT**: likely ignored — a very large explicit requirement list (including a complaint that #153 was already ignored) closed next day with zero delivery evidence.
- **Deferred/follow-up**: effectively all of it: result normalization/styling, duplicate joining, source priority, availability probing, dark-theme fixes, collapsible menu, raw request/response traces, real reasoning pipeline, test doubling, case study, upstream issues.

### Issue #182: Issue with dialog: что такое порты в bsd
- **Requirements (konard)**: none — reported by levi-akkaman; implicit bug: question about BSD ports answered with an irrelevant OpenBSD summary; empty description.
- **Delivered**: nothing in thread.
- **VERDICT**: can't tell — no konard requirements and no delivery evidence either way.
- **Deferred/follow-up**: none stated.

### Issue #183: Unknown prompt: как устроен AUR
- **Requirements (konard)**: (comment 2026-05-20) "We should support entire class of `how X works?` questions with multiple variations and in all languages."
- **Delivered**: nothing in thread after konard's comment; closed next day with no response.
- **VERDICT**: likely ignored — explicit generalization requirement, no reply or delivery evidence before closure.
- **Deferred/follow-up**: generic multilingual "how X works?" intent class — unhandled in thread.

### Issue #184: Unknown prompt: что такое OpenStreerMap
- **Requirements (konard)**: none — reported by levi-akkaman; implicit bug: typo "OpenStreerMap" not fuzzy-matched (corrected spelling worked); empty description.
- **Delivered**: nothing in thread.
- **VERDICT**: can't tell — no stated requirements, no delivery evidence.
- **Deferred/follow-up**: none stated (implicit: typo tolerance in concept lookup).

### Issue #185: Unknown prompt: Prove determinism the way logic can handle paradoxes like Godel's...
- **Requirements (konard)**: (comment 2026-05-20) Use the Rust prover from link-foundation/relative-meta-logic as a library; if unavailable/missing features, file an issue there; formalize requirements via wikidata into a formal plan, convert steps to relative-meta-logic expressions, deliver proven/disproven research result; compile case study to `./docs/case-studies/issue-185`; all in a single PR.
- **Delivered**: nothing in thread; closed 2026-05-21 with no response.
- **VERDICT**: likely ignored — detailed integration requirement, zero delivery evidence.
- **Deferred/follow-up**: relative-meta-logic prover integration; upstream feature request if library lacks capability; case study folder.

### Issue #187: Unknown prompt: Какой сегодня день?
- **Requirements (konard)**: none — reported by veb86; implicit bug: "what day is today" not answerable; empty description.
- **Delivered**: nothing in thread.
- **VERDICT**: can't tell — no stated requirements, no delivery evidence.
- **Deferred/follow-up**: none stated.

### Issue #190: Unknown prompt: Переведи "как у тебя дела?" на английский.
- **Requirements (konard)**: (body) Translation prompt fell to unknown intent; "Что ещё ты умеешь?" should use chat history as context and differ from "Что ты умеешь?" (omit already-mentioned items); left MENU takes too much space — must be collapsible like other sections; investigate/fix the "omitted N earlier messages" truncation in issue reports (fit as much dialog as the GitHub URL limit allows, adaptive line/char omission); drop the useless Reproduction Steps section and rename `## Dialog` to `## Reproduction of dialog`; case study in `./docs/case-studies/issue-190`; upstream issues; all in one PR.
- **Delivered**: no comments at all; closed same day. Indirect evidence: later issues (#207+) do show the `## Reproduction of dialog` heading and no Reproduction Steps section, so the report-template renaming was implemented somewhere.
- **VERDICT**: partially addressed — the report-template rename/section removal demonstrably landed (visible in issues #207 onward), but nothing in the thread shows the context-aware "что ещё", collapsible menu, or adaptive truncation fix (later issues still show "omitted N earlier messages" with heavy truncation, e.g. #232).
- **Deferred/follow-up**: context-aware capability listing; collapsible left menu (re-raised from #180); adaptive dialog truncation; case study.

### Issue #192: Issue with dialog: Calcualte 2+5050
- **Requirements (konard)**: none — reported by lion-lef; implicit bug: typo "Calcualte" made expression parsing fail; empty description.
- **Delivered**: nothing in thread.
- **VERDICT**: can't tell — no stated requirements, no delivery evidence.
- **Deferred/follow-up**: none stated (implicit: typo-tolerant calculation intent).

### Issue #193: We need to prebundle everything using `bun bundle` if possible
- **Requirements (konard)**: (body) Prebundle everything with `bun bundle` so GitHub Pages needs no external CDNs for JS components (also fixes i18n for some users); case study in `./docs/case-studies/issue-193`; debug output if root cause unclear; upstream issues; all in one PR.
- **Delivered**: nothing in thread; closed same day by konard's own repo flow with no comment.
- **VERDICT**: can't tell — plausible it was fixed by a PR the same day (quick close), but thread contains zero delivery evidence.
- **Deferred/follow-up**: case-study folder and upstream reporting not evidenced.

### Issue #195: Make sure everything ready to run our AI system in docker in docker
- **Requirements (konard)**: (body) Telegram-bot image must be based on link-foundation/box dind image (only supported image); use link-foundation/start with `--isolation docker` to spawn/track coding tasks and capture logs so the system can test generated code before returning it; README fully in sync with codebase incl. telegram-bot startup instructions; case study in `./docs/case-studies/issue-195`; single PR.
- **Delivered**: nothing in thread; closed 2026-05-21.
- **VERDICT**: likely ignored — substantial infrastructure requirements, no delivery evidence or closing remark.
- **Deferred/follow-up**: dind base image, start-based docker isolation for code testing, README sync, case study — all unevidenced.

### Issue #196: Support permanent deletion of "deleted" conversations and the entire memory
- **Requirements (konard)**: (body) Physical deletion from memory with irreversibility warning + confirmation; full memory reset feature (for testing seed packages) with warning and pre-reset export offer; support in all supported languages; case study in `./docs/case-studies/issue-196`; single PR.
- **Delivered**: nothing in thread; closed 2026-05-25 (a few days later) with no comment.
- **VERDICT**: can't tell — no delivery evidence in thread; longer open period hints work may have happened elsewhere, but nothing shows it.
- **Deferred/follow-up**: multilingual coverage of the deletion/reset flows and the case study are unevidenced.

### Issue #205: Add optional experimental support for tesseract.js
- **Requirements (konard)**: (body) OCR only if enabled in settings; warn about download size; ship as separate optional bundle; when enabled support image attachments encoded as base64 inside links memory; case study; single PR.
- **Delivered**: nothing in thread; closed next day.
- **VERDICT**: likely ignored — feature request with specific constraints, zero delivery evidence and a suspiciously fast silent close.
- **Deferred/follow-up**: optional OCR bundle, settings gate, size warning, base64 image attachments, case study.

### Issue #207: Issue with dialog: Переведи "как у тебя дела?" на английский.
- **Requirements (konard)**: (body) Translation output too robotic ("meaning: meaning_2cfc55c914d57d9e...") — must read like natural conversation; preserve original formatting/casing of translated text; update requirements/docs with these guidelines; replace the single hardcoded meaning with general translation logic assisted by wikipedia/wikidata/wiktionary APIs; formalize source → semantic meta language (links notation) → deformalize to target; apply across codebase and docs; case study; single PR.
- **Delivered**: nothing in thread. Indirect: #210 (same day, v0.89.0) shows the same prompt now answering `"how are you?"` — the specific hardcoded case got a natural output — but #210/#216/#217/#221 show every other word/phrase still fake ("[en] яблоко").
- **VERDICT**: partially addressed — the robotic output for the one hardcoded phrase was fixed, but the core requirement (general wikipedia/wikidata/wiktionary-backed translation, not one hardcoded meaning) demonstrably remained broken per subsequent issues.
- **Deferred/follow-up**: general translation via semantic meta language; docs/requirements update; case study.

### Issue #209: Unknown prompt: привет. докажи что простых бесконечно
- **Requirements (konard)**: (comment 2026-05-24) Implement true reasoning steps for the general case, not this specific proof; select the formal system in which the proof is possible; use link-foundation/relative-meta-logic to write, verify and compile the proof to WebAssembly; if it can't compile wasm-to-wasm, file a feature request there.
- **Delivered**: nothing in thread after the comment; closed same day (2026-05-24, ~3h later) with no response.
- **VERDICT**: likely ignored — explicit prover-integration requirement followed by a silent close hours later.
- **Deferred/follow-up**: relative-meta-logic proof pipeline; upstream wasm-compilation feature request.

### Issue #210: Issue with dialog: Переведи "доброе яблоко" на английский.
- **Requirements (konard)**: (body) "We have alot of bugs, and false positives in this conversation" — e.g. "Переведи 'кто ты такой' на английский" answered with the bot's self-description instead of a translation; "доброе яблоко" produced fake `"[en] доброе яблоко"`; make everything work as expected; case study; debug output; upstream issues; single PR.
- **Delivered**: nothing in thread; closed same day. Subsequent issues #216/#217/#221 show the fake "[en] X" translation still present days later.
- **VERDICT**: likely ignored — the exact failure mode recurs in later issues; no delivery evidence.
- **Deferred/follow-up**: false-positive intent routing for translate prompts; real multi-word translation; case study.

### Issue #212: Unknown prompt: Найди яблоко в интернете
- **Requirements (konard)**: none stated (empty description); implicit bug: "Найди яблоко в интернете" not routed to web_search (word order differs from #180's phrasing which worked).
- **Delivered**: nothing in thread.
- **VERDICT**: can't tell — bare auto-report, no delivery evidence.
- **Deferred/follow-up**: none stated (implicit: robust word-order-independent search intent).

### Issue #213: Unknown prompt: Покажи список своих правил
- **Requirements (konard)**: none stated (empty description); implicit bug: Russian equivalent of "List behavior rules" not recognized even though the error message tells users to inspect rules.
- **Delivered**: nothing in thread.
- **VERDICT**: can't tell — bare auto-report, no delivery evidence.
- **Deferred/follow-up**: none stated (implicit: localized rule-listing commands).

### Issue #216: Issue with dialog: translate apple to russian
- **Requirements (konard)**: none stated (empty description); implicit bug: "translate apple to russian" returned just `[ru]`.
- **Delivered**: nothing in thread. Closed at the exact same timestamp as #217 and #218 (2026-05-21T22:52:47Z), suggesting a batch close under umbrella #218.
- **VERDICT**: can't tell — likely folded into #218, but no evidence in this thread that the bug was fixed (and #221 next day shows translation still fake for most words).
- **Deferred/follow-up**: en→ru translation actually producing a word.

### Issue #217: Issue with dialog: переведи «яблоко» на английский
- **Requirements (konard)**: none stated (empty description); implicit bug: guillemet quotes «яблоко» produced fake `"[en] яблоко"`.
- **Delivered**: nothing in thread; batch-closed with #216/#218 at the same second.
- **VERDICT**: can't tell — folded into #218 with no evidence of a fix; #221 shows the same fake output pattern persisting.
- **Deferred/follow-up**: quote-style-independent translation parsing.

### Issue #218: Translation issues
- **Requirements (konard)**: (body) Umbrella: fix ALL translation sub-issues in a single PR; states PR #208 was expected to make translation "fully work in the most universal and general way", highest quality, multiple prompt variations, all supported languages; case study; debug output; upstream issues; single PR.
- **Delivered**: nothing in thread; closed simultaneously with #216/#217. #221, filed hours later at v0.98.0, shows translation still fake for "огурец"/"помидор" and konard writing "We must stop faking translation" and "I tired to asking again and again".
- **VERDICT**: likely ignored — the umbrella was closed while konard's own next report (#221) proves universal translation was not delivered.
- **Deferred/follow-up**: universal multi-language translation quality; case study; the whole "no more fake translation" demand — re-raised in #221.

### Issue #221: Issue with dialog: Переведи "помидор" на английский.
- **Requirements (konard)**: (body) "We must stop faking translation, and actually implement it for all possible words we can find in wikipedia, wikidata, wiktionary, and actually use APIs"; repeats (asked "multiple times") the flow source language → formalize → semantic meta language (doublet links, wikidata Q/P ids as meanings; virtual ids for wikipedia/wiktionary) → deformalize to target language with grammatical agreement (согласованность); ambiguity resolved by questions/configured guessing; pure Rust, use non-LLM NLP libs where useful; must work for text of any size, "actually work, not another fake"; references prior demos (human-language entity.jsx, meta-expression formalize); case study; upstream issues; single PR. Explicit frustration: "I tired to asking again and again."
- **Delivered**: nothing in thread; closed 2026-05-22. #230 (v0.102.0, same day) shows single words translating ("проба", "good evening" — with a mistranslation "банка"→"bench") but multi-word phrases still fake ("[En] Найти синонимы...").
- **VERDICT**: partially addressed — later dialogs show some real single-word translation appeared, but the core requirement (any-size text via semantic meta language, no fakes) remained unmet per #230, and the thread itself has no delivery evidence.
- **Deferred/follow-up**: phrase/sentence/whole-text translation; semantic-meta-language architecture; grammatical agreement; ambiguity resolution; case study.

### Issue #223: Unknown prompt: how the join method works in pandas
- **Requirements (konard)**: (comment 2026-05-23) Should find the answer in the pandas project docs and provide a summary narrowed to that specific method.
- **Delivered**: nothing in thread after the comment; closed 2026-05-24 with no response.
- **VERDICT**: likely ignored — explicit "search project docs and summarize" requirement, silent close.
- **Deferred/follow-up**: library-documentation lookup capability.

### Issue #224: Unknown prompt: What is the most popular dataset for translation quality validation?
- **Requirements (konard)**: (body) Find a general way to answer all such open-ended research questions, not just this one; pastes Gemini and ChatGPT answers as quality references; fact-check each statement of those answers, reverse-engineer the logical reasoning steps, and fully support answering "any possible questions in the universe" via search, page-hopping information gathering, and stepwise reasoning.
- **Delivered**: nothing in thread; closed same day.
- **VERDICT**: likely ignored — sweeping capability requirement with zero delivery evidence.
- **Deferred/follow-up**: general multi-hop search-and-reason answering; fact-checking pipeline.

### Issue #226: Unknown prompt: согласованность в предложении - есть такая статья в википедии?
- **Requirements (konard)**: (body) System must answer "does Wikipedia have an article about X" questions via logical reasoning and data gathering (near-match suggestion like Gemini's "Согласование (грамматика)"); fix false positive where pasting a long Gemini quote triggered "Done. UI language is now ru."; learning must generalize recursively — learn the entire class of questions/answers, not memoize single Q/A pairs; case study; debug output; upstream issues; single PR.
- **Delivered**: nothing in thread; closed same day.
- **VERDICT**: likely ignored — three concrete requirements (article-existence queries, language-switch false positive, class-level learning), no delivery evidence.
- **Deferred/follow-up**: all three, plus case study.

### Issue #228: Unknown prompt: list all genshin characters with off-field DMG
- **Requirements (konard)**: (body) Need a general way to support list/aggregation queries ("list all X with property Y") using actual logical reasoning + internet search with no LLMs (Google's answer pasted as reference); case study; debug output; upstream issues; single PR.
- **Delivered**: nothing in thread; closed same day.
- **VERDICT**: likely ignored — general aggregation-query capability requested, zero delivery evidence.
- **Deferred/follow-up**: reasoning-backed list queries over web sources; case study.

### Issue #230: Issue with dialog: Переведи "Найти синонимы или примеры согласования" на английский
- **Requirements (konard)**: (body) Continue implementing the truly universal formalization/deformalization (naturalization) algorithm, with translation as the composition of the two — multi-word phrase still returned fake `"[En] Найти синонимы или примеры согласования"` (also note single-word mistranslation "банка"→"bench" visible in dialog); case study; debug output; upstream issues; single PR.
- **Delivered**: nothing in thread; closed same day.
- **VERDICT**: likely ignored — continuation of the repeatedly-requested universal translation work, no delivery evidence; this is at least the fourth issue (#207, #218, #221, #230) restating it.
- **Deferred/follow-up**: universal formalize/deformalize pipeline for phrases; translation accuracy; case study.

### Issue #232: Issue with dialog: Что такое существо?
- **Requirements (konard)**: (body) For "Что такое существо?" use the Russian Wikipedia disambiguation page (ru.wikipedia.org/wiki/Существо) instead of a mismatched wikidata Animalia entry; for disambiguation pages, list all possible meanings in all contexts with their definitions; case study; debug output; upstream issues; single PR.
- **Delivered**: nothing in thread; closed same day.
- **VERDICT**: likely ignored — specific disambiguation-handling requirement, no delivery evidence.
- **Deferred/follow-up**: disambiguation-page handling (enumerate all meanings); locale-appropriate source selection; case study.

### Issue #237: Unknown prompt: Привет, расскажи о себе.
- **Requirements (konard)**: none stated (empty description); implicit bugs: combined greeting+request hit unknown intent, and "Расскажи о себе" answered with a Wikipedia article about politician "Себе, Леннокс" instead of self-description.
- **Delivered**: nothing in thread.
- **VERDICT**: can't tell — bare auto-report by konard with no written requirements, no delivery evidence.
- **Deferred/follow-up**: none stated (implicit: self-description intent, compound-prompt splitting).

### Issue #242: Unknown prompt: what i digress mean?
- **Requirements (konard)**: (body) Support "what does X mean?" dictionary queries at least as well as Google's answer; consider adding dictionary.cambridge.org and other dictionaries to sources and connectivity tests; possibly use wikidata per term; must be general rules, not memoization, but tests must include this example against regressions; "double check to fully apply requirements to entire codebase, so if we have issue in multiple places, it should be fixed in all them"; case study; upstream issues; single PR.
- **Delivered**: nothing in thread; closed 2026-05-25 with no comment.
- **VERDICT**: likely ignored — concrete source-integration and generalization requirements, zero delivery evidence.
- **Deferred/follow-up**: dictionary sources integration; general definition-intent handling; regression test for this example; case study.
# Requirements audit — chunk 5

## Issues #244–#314

### Issue #244: Plan issues to implement our vision fully
- **Requirements (konard)**: Update docs to fully track requirement implementation progress; then create all issues needed to fully implement the vision (universal problem solving, translation between natural/formal languages, minimal algorithm core + data seed, formal reasoning covering all test cases "and much more", no neural networks for reasoning); learn to work with unknowns and gather missing information, asking the user as few questions as possible; fix critical blocking problems first; compile case-study data to `docs/case-studies/issue-244/` with online research, per-requirement solution plans, and survey of existing components/libraries.
- **Delivered**: Two extensive audit comments (posted under konard's account, reporting bot work): third-pass audit opened epics E21–E27 (#298–#304) with code anchors and case study; fourth-pass audit confirmed the README algorithm image, the single 11-step loop path, identified the remaining synthesis gap (benchmarks 0/5), and opened E28–E32 (#313–#317) with anti-memorization rules. E1–E14 (#246–#259), E15–E20 (#278–#283), E21–E27 (merged via PRs #305–#311) all closed. Docs (ROADMAP/REQUIREMENTS/ARCHITECTURE/VISION) synced. Planning landed in PR #245, cargo test green (0 failed, 0 ignored).
- **VERDICT**: fully addressed — planning, case study, docs sync, and full epic breakdown were all delivered and verified across multiple audit passes.
- **Deferred/follow-up**: The vision itself is explicitly not done: synthesis still resolves from seeded handlers (benchmark suite 0/5); deferred to E28–E32 (#313–#317).

### Issue #246: E1: Unified doublet-links store (doublets-rs + doublets-web)
- **Requirements (konard)**: `LinkStore` trait; doublets-rs native + doublets-web/IndexedDB WASM backends; every record reducible to doublets with stable content-addressed id and schema version; reject ill-formed Links Notation; keep `.lino` export; graduate 6 named spec tests in `links_network.rs`.
- **Delivered**: No comments in thread. Cross-thread: #278 body says #246 "introduced the `LinkStore` boundary and doublet projections" but the native default store was still `.lino` (feature-gated doublets); #244 audit confirms E1–E14 closed/merged with zero `#[ignore]` spec tests remaining.
- **VERDICT**: partially addressed — trait/projections/tests delivered, but doublets-rs was not made the default store (konard reopened this gap as #278/E15).
- **Deferred/follow-up**: Making doublets-rs the default native physical store → follow-up issue #278 (E15).

### Issue #247: E2: Make the universal reasoning loop the only entry path
- **Requirements (konard)**: Re-seat 35+ specialized handlers as candidate generators inside step 7 instead of routing; every prompt walks the full formalize→…→select pass; each step emits events; graduate 11 `reasoning_loop.rs` specs + 3 `chat_surface.rs` specs.
- **Delivered**: No comments in thread. Cross-thread: #244 fourth-pass audit verifies the 11-step loop is the single main path for every prompt; #298/#299 (E21/E22) show `SPECIALIZED_HANDLERS` still routed as first-match and #313 (E28) shows handlers still weren't candidate generators in synthesis.
- **VERDICT**: partially addressed — loop skeleton/specs closed, but the core requirement (handlers as candidate generators, not routing) was still open and re-raised in #299 and #313.
- **Deferred/follow-up**: Dropping the fixed intent catalogue → #299 (E22); handlers as scored candidates in synthesis → #313 (E28).

### Issue #248: E3: Full Wikidata P/Q-id formalization engine
- **Requirements (konard)**: Multilingual labels table + morphology hints resolving tokens to Q/P-ids; scored meaning records; Wiktionary/Wikipedia fallback; flag unanchored terms; closes ARCHITECTURE.md §16.1.
- **Delivered**: No comments. Cross-thread: #299 (E22) references the existing Wikidata-backed `FormalizationCandidate` in `src/translation/formalization.rs` but notes it is "**not** the primary routing input"; #244 audit counts E3 as closed/merged with all spec tests green.
- **VERDICT**: partially addressed — formalization engine built and tests graduated, but not wired as the primary routing input (re-raised in #299).
- **Deferred/follow-up**: Wiring `FormalizationCandidate` into routing → #299 (E22).

### Issue #249: E4: Temperature-based interpretation selection + clarify-vs-guess
- **Requirements (konard)**: Deterministic softmax over candidate scores seeded from impulse hash; ε-comparison; clarify vs guess per `questioning_rigor`; deterministic replay tests.
- **Delivered**: No comments. Cross-thread: #279 (E16) body refers to "the new temperature-based selection path" as existing; #244 audit counts it closed/merged, tests green.
- **VERDICT**: fully addressed — later issues treat the temperature/clarify-vs-guess selection as implemented.
- **Deferred/follow-up**: Probabilistic (Bayesian/Markov) evidence layer built on top → #279 (E16).

### Issue #250: E5: Public-knowledge source cache with provenance
- **Requirements (konard)**: Cache external fetches with URL, fetched_at, content hash, TTL; cache_hit events; refresh stale; surface conflicts; auditable flush; offline mode refuses lookups; graduate 8 `source_cache.rs` specs.
- **Delivered**: No comments. Cross-thread: #298 (E21) lists "the public-knowledge source cache (E5)" as an existing component to reuse; #244 audit: closed/merged, zero ignored specs.
- **VERDICT**: fully addressed — referenced downstream as an available component; all specs graduated per #244 audit.
- **Deferred/follow-up**: none

### Issue #251: E6: Translation via link-native meanings
- **Requirements (konard)**: One meaning id per concept anchored on Q/P-ids; render target surface from per-language labels; intermediate meaning in trace; flag untranslatables; graduate 7 `translation_via_links.rs` specs.
- **Delivered**: No comments. Cross-thread: #244 audit counts E6 closed/merged with zero ignored spec tests.
- **VERDICT**: fully addressed — per #244 audit all its spec tests graduated.
- **Deferred/follow-up**: none

### Issue #252: E7: Code generation & cross-language translation
- **Requirements (konard)**: Top-10 language hello-world seeds; execution links; isolation level declared; algorithm + tests (TDD); cross-language program translation preserving semantics; execution failures reported with trace; graduate 6 `code_generation.rs` specs.
- **Delivered**: No comments. Cross-thread: #244 audit counts it closed/merged; but #300 (E23) shows the delivered form was ~10 hardcoded per-language intents, and #312 shows a real Rust program request still failed as "unknown" at v0.140.
- **VERDICT**: partially addressed — specs graduated, but delivered as memorized per-language seeds rather than general code generation (re-raised in #300, #303, #312, #315).
- **Deferred/follow-up**: Parametric `write a program` intent → #300 (E23); general program synthesis → E30 (#315).

### Issue #253: E8: Formal reasoning engine (relative-meta-logic / SMT)
- **Requirements (konard)**: Replace the fixed classical-theorem registry with a real decision procedure (relative-meta-logic and/or Z3/SMT), keep the presentation layer, prove theorems beyond the fixed registry; closes issue #244 Q9.
- **Delivered**: No comments. Cross-thread: #314 (E29) references `src/proof_engine/decision.rs` "(boolean/linear decision modules)" as existing components — a decision procedure exists; #244 audit counts it closed/merged.
- **VERDICT**: fully addressed — decision modules exist and are reused downstream; though the "much more" ambition continues in E29.
- **Deferred/follow-up**: Applying the decision procedure to compute benchmark answers (GSM8K/MATH) → #314 (E29).

### Issue #254: E9: Chat-over-experience queries
- **Requirements (konard)**: Chat intents projecting store/event log: snapshot, concept links, "why", "list my facts", `.lino` export, retraction protocol for "forget"; no diagnostic-id leaks; graduate 8 `transparent_state.rs` specs.
- **Delivered**: No comments. Cross-thread: #244 audit says zero `#[ignore]` specs remain (so `transparent_state.rs` graduated); however #302 (E25) states transparent-state queries are "not generally answerable from arbitrary prompts" and lists some as still `#[ignore]` at the time E25 was written.
- **VERDICT**: partially addressed — fixed-form queries covered; general natural-language memory queries re-raised in #302 (E25).
- **Deferred/follow-up**: General NL memory querying → #302 (E25).

### Issue #255: E10: Links-network invariants & dynamic type system
- **Requirements (konard)**: `Type -> SubType -> Value` chain projection; every fact carries a source link; every answer a trace pointer with ordered steps; graduate remaining 4 `links_network.rs` specs.
- **Delivered**: No comments. Cross-thread: #244 audit counts it closed/merged with all specs green.
- **VERDICT**: fully addressed — per #244 audit, zero ignored specs remain.
- **Deferred/follow-up**: none

### Issue #256: E11: Agent mode with isolated execution
- **Requirements (konard)**: Opt-in agent mode with sandbox, visible action log, confirmation for destructive actions, time budget, secret guard, privilege revocation; graduate 9 `agent_isolation.rs` specs + 1 `chat_surface.rs` spec.
- **Delivered**: No comments. Cross-thread conflict: #244 audit says E1–E14 closed/merged with zero ignored specs, but #303 (E26, written after) says "Agent mode is guarded but never executes … see `tests/unit/specification/agent_isolation.rs`, 9 `#[ignore]` specs".
- **VERDICT**: partially addressed — thread closed as done, but konard's own later issue #303 states the 9 agent-isolation specs were still `#[ignore]` and agent mode never executed.
- **Deferred/follow-up**: Actual executing agent with isolation → #303 (E26).

### Issue #257: E12: Authenticated API + tool-call gating
- **Requirements (konard)**: Bearer-token auth on HTTP routes; refuse tool calls unless agent mode on; graduate 2 `openai_compatibility.rs` specs.
- **Delivered**: One comment: automated solver failed pre-work ("Insufficient disk space … System checks failed" — no PR created). Cross-thread: #281 (E18) later says "issue #257 added API authentication/tool-call gating", and #244 audit counts it closed/merged.
- **VERDICT**: fully addressed — despite the initial solver failure, later threads confirm auth/gating was delivered.
- **Deferred/follow-up**: Broader package/permission model → #281 (E18).

### Issue #258: E13: Network visualization + trace links on every surface
- **Requirements (konard)**: Non-blocking graph panel in web demo; Telegram trace link; execution-status line on code answers; diagnostics opt-in; graduate 4 named specs.
- **Delivered**: One comment: automated solver failed (insufficient disk space, no PR). Cross-thread: #244 audit counts E13 closed/merged with zero ignored specs; #262 dialog shows "Execution status: not run …" lines live in production.
- **VERDICT**: fully addressed — execution-status lines observable in the demo and specs graduated per audit.
- **Deferred/follow-up**: none

### Issue #259: E14: Natural-language skill compilation
- **Requirements (konard)**: Compiler turning NL skill descriptions into reusable associative packages; deterministic replay; exportable as Links Notation; compiled skill preferred over re-deriving (cache_hit).
- **Delivered**: One comment: automated solver failed (disk space, no PR). Cross-thread: #283 (E20) confirms "#259 implemented a deterministic natural-language skill compiler for trigger/response rules"; #292 dialog shows the `When I say … answer …` teaching form live.
- **VERDICT**: fully addressed for its scope — trigger/response compiler delivered; broader skill language was out of scope.
- **Deferred/follow-up**: Typed arguments, multi-step procedures, generated tests, handler lowering → #283 (E20); rule-as-data substitution engine → #301 (E24).

### Issue #262: Unknown prompt: ого, чето начал соображать:)
- **Requirements (konard)**: None — reported by user netkeep80 via the in-app report flow. Implicit requirement: casual Russian remarks ("что ты умеешь?", "что за дичь?", "ого, чето начал соображать") should not all hit the unknown fallback.
- **Delivered**: No comments in thread. Cross-thread: #278–#283 bodies say the audit ran "after issues #246-#259 **and #262** were merged" — a fix for #262 was merged; the dialog itself shows "что ты умеешь?" was later answered (identity/capability rules visible in #292's v0.130 dialog).
- **VERDICT**: fully addressed — explicitly listed as merged in the E15–E20 audit provenance.
- **Deferred/follow-up**: none

### Issue #272: в чём ты можешь быть полезен?
- **Requirements (konard)**: None — reported by user 1Anastasios1. Implicit: Russian "в чём ты можешь быть полезен" / open-domain questions should get useful answers instead of the unknown-fallback teaching text.
- **Delivered**: No comments, no PR reference in thread. Cross-thread: #292's dialog (v0.130) shows a `rule_capabilities` rule matching "What can you do? / Что ты умеешь?" — the capability intent exists, but there's no evidence this exact phrasing ("в чём ты можешь быть полезен") or the economics question was covered.
- **VERDICT**: partially addressed — a capabilities rule was added, but no in-thread evidence the reported phrasings were made to work.
- **Deferred/follow-up**: General question answering beyond fixed phrasings — effectively deferred to the #298/#313 reasoning-under-unknowns/synthesis epics (not referenced from this thread).

### Issue #278: E15 Make doublets-rs the default native physical store
- **Requirements (konard)**: doublets-rs as default native backend (no opt-in feature); `.lino` kept as import/export projection; migration coverage from `.lino` bundles; all surfaces share store semantics; docs no longer describe it as future work.
- **Delivered**: No comments in thread. Cross-thread: #244 third-pass audit states "E15-E20 (#278-#283) are all closed/merged, and there are zero tracked `#[ignore]` specification tests remaining".
- **VERDICT**: fully addressed — per #244 audit, closed/merged with specs green.
- **Deferred/follow-up**: none

### Issue #279: E16 Symbolic probabilistic reasoning over Links Notation
- **Requirements (konard)**: Link-native probabilistic evidence with provenance/timestamps; Bayesian/Markov-style ranking; integration with clarify-vs-guess; deterministic replay; probability evidence in traces; docs on the non-neural boundary.
- **Delivered**: No comments. Cross-thread: #244 audit counts E15–E20 closed/merged; #313 (E28) lists "the E16 probabilistic ranking for candidate selection" as an existing component to reuse.
- **VERDICT**: fully addressed — downstream issue treats E16 ranking as delivered.
- **Deferred/follow-up**: none

### Issue #280: E17 Desktop application wrapper for formal-ai
- **Requirements (konard)**: Packaged desktop app reusing library/HTTP surfaces; memory import/export; trace/network views; smoke test or documented manual verification; R17 updated.
- **Delivered**: No comments. Cross-thread: #244 audit counts E15–E20 closed/merged with zero ignored specs. No independent evidence of the desktop app in this chunk.
- **VERDICT**: fully addressed (per #244 audit statement) — though no direct artifact evidence appears in this chunk.
- **Deferred/follow-up**: none stated

### Issue #281: E18 Reusable associative packages and permission model
- **Requirements (konard)**: Package metadata/dependencies/handlers/permissions/triggers in Links Notation; skills belong to packages; explicit permissions; deterministic install/export/import/replay; visible in traces/network view; maps to R65.
- **Delivered**: No comments. Cross-thread: #244 audit counts E15–E20 closed/merged; #301 (E24) references `src/associative_package.rs` (`PackageTrigger`, `CompiledSkillPackage`) as existing code; #302 (E25) relies on "E18 associative-package permissions".
- **VERDICT**: fully addressed — package/permission code exists and is reused by later epics.
- **Deferred/follow-up**: none

### Issue #282: E19 Complete Rust-to-WebAssembly solver parity for the browser worker
- **Requirements (konard)**: Inventory browser-worker logic; move domain logic to Rust/WASM; JS only UI/fetch glue; parity tests native vs WASM; update docs and R194.
- **Delivered**: No comments. Cross-thread: #244 audit counts E15–E20 closed/merged with zero ignored spec tests.
- **VERDICT**: fully addressed (per #244 audit) — no contrary evidence in this chunk.
- **Deferred/follow-up**: none stated

### Issue #283: E20 Generalized natural-language skill compiler beyond trigger/response
- **Requirements (konard)**: Extend skill language to typed inputs, preconditions, steps, effects, expected tests; lower to package records / handler stubs; refuse unsupported instructions; deterministic and inspectable as Links Notation.
- **Delivered**: No comments. Cross-thread: #244 audit counts E15–E20 closed/merged; yet #301 (E24, written after) still says the skill compiler "only supports trigger/response shapes" (`src/skill_compiler.rs:167`).
- **VERDICT**: partially addressed — closed as merged, but konard's own later issue #301 states the compiler remained trigger/response-only.
- **Deferred/follow-up**: Substitution-rule engine / rule-as-data → #301 (E24).

### Issue #284: Unable to set the name
- **Requirements (konard)**: Bug report (screenshot + report link): assistant says "you can name me as you like", but "Теперь тебя зовут Алексей" hits the unknown intent — setting the assistant's name via chat should work.
- **Delivered**: No comments, no PR reference, no closing remarks in thread. Cross-thread hint only: #292's rule listing mentions "unless the assistant name setting is configured", implying a name *setting* exists — but no evidence chat-based renaming was fixed.
- **VERDICT**: can't tell — closed with zero in-thread delivery evidence; possible partial fix via a settings option, but the reported chat flow has no confirmed fix.
- **Deferred/follow-up**: Chat-based renaming ("Теперь тебя зовут X") — unhandled as far as this thread shows.

### Issue #286: Unknown prompt: Что такое антирежим?
- **Requirements (konard)**: Auto-generated unknown-prompt report by konard: the system should be able to answer (or reason about) "Что такое антирежим?" instead of returning the unknown fallback.
- **Delivered**: No comments, no PR reference, no closing remarks.
- **VERDICT**: likely ignored — closed with no delivery evidence or response of any kind.
- **Deferred/follow-up**: Answering unseen definitional questions — implicitly subsumed by #298 (E21) reasoning-under-unknowns, but not referenced from this thread.

### Issue #288: Unknown prompt: Что такое ложная тотальность?
- **Requirements (konard)**: Same pattern: "Что такое ложная тотальность?" hit the unknown fallback; should be answerable.
- **Delivered**: No comments, no PR reference, no closing remarks.
- **VERDICT**: likely ignored — closed with no delivery evidence or response.
- **Deferred/follow-up**: Same as #286 — general definitional Q&A, implicitly deferred to the E21/E28 epics without being referenced.

### Issue #292: Issue with dialog: Перечисли свои правила
- **Requirements (konard)**: (1) The behavior-rules listing has "no translation to russian, and other languages"; (2) "if I ask in Russian, I only want to get answer in Russian"; (3) "We need to have CI/CD rules, so everything that exists in a single language also is written in all other languages"; (4) "Also markdown in the message or UI is broken".
- **Delivered**: No comments, no PR reference, no closing remarks in thread. No later thread in this chunk references a fix for rules-listing localization, a language-parity CI check, or the markdown rendering bug.
- **VERDICT**: likely ignored — four explicit requirements from konard, closed with zero visible response.
- **Deferred/follow-up**: All four: Russian/localized rules listing; answer-in-question-language guarantee; CI/CD language-parity checks; markdown rendering fix.

### Issue #298: E21: Reasoning under unknowns instead of failing
- **Requirements (konard)**: On unmatched prompts run real reasoning (known/unknown/candidate-source/gather-attempt as links); retrieve reachable facts from memory/cache; at most one minimal clarifying question; canned fallback only as recorded last resort; new spec tests for all four paths.
- **Delivered**: No comments in thread. Cross-thread: #244 fourth-pass audit states unmatched prompts "fall into the reasoning-under-unknowns loop (`src/solver_unknown_reasoning.rs`), not a canned opener", and E21–E27 merged via PRs #305–#311.
- **VERDICT**: fully addressed — the fourth-pass audit explicitly verifies the unknown-reasoning loop exists and is used.
- **Deferred/follow-up**: Deriving actual answers (not just reasoning traces) → E28 (#313).

### Issue #299: E22: Intent formalization as Links Notation (drop the fixed intent catalogue)
- **Requirements (konard)**: Formalize every message into a `.lino` intent (kind/knowns/relevants) before routing; retire `SelectedRule`/`SPECIALIZED_HANDLERS` as first-class router; reasoning cache keyed by impulse id; wire `FormalizationCandidate` into routing; new spec tests.
- **Delivered**: No comments. Cross-thread: #313 (E28) confirms "the E21-E27 batch made it formalize intents"; #244 audit lists the intent cache in the solver entry (`solve_with_history_probability_store_and_intent_cache`) and E21–E27 merged via PRs #305–#311. But #313 also says the `SPECIALIZED_HANDLERS` precedence table still resolves prompts to seeded outputs.
- **VERDICT**: partially addressed — formalization + intent cache delivered, but the handler table remained the effective resolver in synthesis (re-raised in #313).
- **Deferred/follow-up**: Handlers demoted to scored candidate generators → #313 (E28).

### Issue #300: E23: Generalized parametric intents (write a program with parameters)
- **Requirements (konard)**: One parametric `write a program(language, task)` intent replacing ~10 per-language hello-world intents; data-driven templates; unlisted language/task combos handled without new Rust routing; tests for parametric path and graceful rejection.
- **Delivered**: No comments. Cross-thread: #244 audit says E21–E27 merged via PRs #305–#311 with tests green; #315 (E30) was opened for general program synthesis "incl. #312", indicating the parametric intent still couldn't synthesize arbitrary programs.
- **VERDICT**: fully addressed for its stated scope (parametric hello-world path merged) — general program synthesis explicitly remained.
- **Deferred/follow-up**: Synthesizing arbitrary programs from spec + tests → E30 (#315).

### Issue #301: E24: Substitution-rule handlers over link CRUD (replace x y, when n do m)
- **Requirements (konard)**: link-cli-style `replace x y` over link patterns with variables; composable `when … do …` rules attached to link CRUD events; deterministic, order-defined, trace-linked; a Rust-handler behavior expressible purely as rule data; spec tests incl. termination guard.
- **Delivered**: No comments. Cross-thread: #244 audit says E21–E27 merged via PRs #305–#311; #313 confirms the batch made the loop "run substitution rules".
- **VERDICT**: fully addressed — substitution-rule engine confirmed running by the fourth-pass audit.
- **Deferred/follow-up**: General text manipulation via composed substitution rules → E31 (#316).

### Issue #302: E25: Natural-language access to memory, APIs, and code execution
- **Requirements (konard)**: From NL: query link memory (graduate `transparent_state.rs` `#[ignore]` specs), select and call allowed APIs, execute code — all permission-gated with declared execution status; tests for all three paths + permission-denied.
- **Delivered**: No comments. Cross-thread: #244 audit says E21–E27 merged via PRs #305–#311, cargo test "0 ignored".
- **VERDICT**: fully addressed — merged in the E21–E27 batch with zero ignored specs remaining per audit.
- **Deferred/follow-up**: none stated (agent-level coding built on this → #303/E26).

### Issue #303: E26: General code-modifying / executing agent (not a memorizer)
- **Requirements (konard)**: Agent creates/modifies/deletes files and runs terminal commands in an isolated workspace with action log, confirmation, time budget, secret guard, revocation (graduate the 9 `agent_isolation.rs` specs); solves unseen coding tasks (E27 benchmarks) by writing+running+iterating, not lookup; substantially larger test suite.
- **Delivered**: No comments. Cross-thread: #244 audit: merged via PRs #305–#311, 0 ignored specs (so agent_isolation graduated); the audit calls it "a bounded agent". But the audit also reports benchmarks pass 0/5 — the agent could not yet solve unseen coding tasks.
- **VERDICT**: partially addressed — isolation/action-log machinery delivered, but the headline requirement (solving unseen coding tasks by generation, not lookup) explicitly remained unmet (0/5 benchmarks).
- **Deferred/follow-up**: General program synthesis verified in the agent workspace → E30 (#315).

### Issue #304: E27: Import industry-leading permissively-licensed benchmark datasets
- **Requirements (konard)**: Import ≥1 programming, 1 math, 1 general problem-solving permissively-licensed benchmark as deterministic `.lino` test cases with license/provenance notes; runnable suite reporting pass/fail (allowed to start red).
- **Delivered**: No comments. Cross-thread: #244 audit confirms "the imported industry benchmark suite (HumanEval/MBPP/GSM8K/MATH/BIG-bench)" exists and runs (reporting 0/5); #313 cites `tests/unit/specification/benchmarks.rs`, `data/benchmarks/industry-suite.lino`.
- **VERDICT**: fully addressed — suite imported, deterministic, and reporting counts exactly as specified (starting red was allowed).
- **Deferred/follow-up**: Raising the pass count and growing the suite → E29 (#314) and E32 (#317).

### Issue #312: Unknown prompt: Напиши мне программу на Rust, которая выдаёт список файлов в текущей директории
- **Requirements (konard)**: A Russian "write me a Rust program listing files in current dir" request hit the unknown fallback; konard pasted Gemini/DeepSeek answers as the quality bar and required: reconstruct real reasoning steps that solve the whole class of similar tasks (gather missing info, plan and execute each reasoning step); download all logs/data into `docs/case-studies/issue-312/`; deep case-study analysis with timeline, all requirements, root causes, solution plans, existing-component survey; add debug/verbose output if root cause not findable; report issues to other repos if relevant; apply fixes across the entire codebase; "plan and execute everything in this single pull request … until it is each and every requirement fully addressed, and everything is totally done".
- **Delivered**: No comments in thread. Cross-thread: #244 fourth-pass audit (same day as closure) opened E30 (#315) "General program synthesis from spec + tests, verified in the agent workspace (incl. #312)" — i.e., the capability was NOT delivered when this issue was closed; it was folded into a new epic.
- **VERDICT**: partially addressed — explicitly rolled into #315 rather than solved; konard's "everything totally done in this single pull request" instruction was not fulfilled, and no case-study/root-cause evidence appears in the thread.
- **Deferred/follow-up**: The actual capability (synthesizing the requested Rust program and the whole task class) → E30 (#315); case-study folder, debug/verbose additions, cross-repo reports — no evidence any were done.

### Issue #313: E28: General link-native synthesis substrate (derive candidates, don't seed them)
- **Requirements (konard)**: Synthesis must compose decomposed sub-results over the links network instead of seed lookup; sub-impulses solved recursively and recorded as links; handlers become scored candidate generators; every composition step recorded so "why" replays the derivation; ≥1 benchmark domain improves purely via composition with anti-memorization (paraphrase must reach same derivation); determinism; regression floor intact.
- **Delivered**: No comments in thread; closed 2026-05-28 (after the fourth-pass audit that opened it). No delivery evidence within this chunk.
- **VERDICT**: can't tell — closed one day after creation with no in-thread or cross-thread delivery evidence in this chunk.
- **Deferred/follow-up**: Blocks E29/E30/E31 per body; whether the anti-memorization criterion was actually met is unverifiable here.

### Issue #314: E29: Compute math/word-problem and counting answers (GSM8K, MATH, BIG-bench)
- **Requirements (konard)**: Compute (not seed) the three failing numeric benchmarks (`18`, `11`, `3`) via decomposition + deterministic arithmetic/decision procedures; show intermediate quantities in trace; anti-memorization (renumbered held-out variants recompute correctly); deterministic, offline-safe, no neural inference; benchmark pass count rises.
- **Delivered**: No comments in thread; closed 2026-05-28. No delivery evidence within this chunk.
- **VERDICT**: can't tell — closed with no in-thread delivery evidence; success would be measurable (benchmark pass count) but no measurement appears here.
- **Deferred/follow-up**: Held-out renumbered variants and rising pass floor also tracked in E32 (#317) — status unverifiable here.
# Requirements audit — chunk 6

## Issues #315–#349

### Issue #315: E30: General program synthesis from spec + tests (HumanEval, MBPP)
- **Requirements (konard)**: Treat "write a program" as a synthesis sub-task: derive candidate function bodies from formalized intent + tests (via E28 composition), not seed lookup; verify each candidate by executing it and its tests in the E26 bounded workspace, iterating on failures (TDD); return the first passing candidate with execution status in the trace; keep seeds as only one candidate generator among others. Acceptance: `humaneval_0_has_close_elements` and `mbpp_2_similar_elements` pass by derive-and-verify; at least one additional unseen HumanEval/MBPP-style function solved the same way (anti-memorization); issue #312's "list files" program produced/executed with declared status; sandbox/time-budget/secret-guard enforced.
- **Delivered**: Nothing in-thread — zero comments, no PR reference, no bot delivery report; closed 2026-05-28. Cross-thread contradiction: #340 and #342 (v0.149, 2026-05-29, i.e., AFTER this closed) show `write_program` still answering "I do not have a template for language `rust`/`python` and task `missing`" — template lookup, not derive-and-verify.
- **VERDICT**: likely ignored — closed with no delivery evidence, and later dialogs (#340, #342) show program synthesis was still template/seed-based, directly contradicting the anti-memorization acceptance criteria.
- **Deferred/follow-up**: the whole derive-and-verify pipeline effectively remained open (resurfaced in #340, #341, #342, #349).

### Issue #316: E31: General text manipulation over arbitrary user input
- **Requirements (konard)**: Route "transform/extract/count/rewrite this text" requests through formalized intent into compositions of E24 substitution rules over parsed text links; operate directly on user-supplied input (no memoized per-input answers); record the rule chain in the trace as replayable Links Notation; compose multi-step operations via E28 rather than dedicated handlers. Acceptance: representative op set incl. a composed multi-step case; anti-memorization (same op on different text recomputes correctly); deterministic; new specification tests per operation class.
- **Delivered**: Nothing in-thread — no comments, no PR reference, no bot report; closed 2026-05-28 ~40 min after #315.
- **VERDICT**: likely ignored — closed with no delivery evidence in the thread and konard's acceptance criteria unanswered (indirect: #326's audit still found `text_manipulation.rs` English-only after this closed).
- **Deferred/follow-up**: multilingual triggering of these text ops was split out to #326 (E33); no other in-thread disposition.

### Issue #317: E32: Grow the benchmark suite and gate progress on rising pass counts
- **Requirements (konard)**: Grow the benchmark suite beyond the 5-case E27 slice (more HumanEval/MBPP/GSM8K/MATH/BIG-bench with provenance + `source_ref` per `LICENSES.md`); add held-out/paraphrased variants so memorized answers cannot pass (anti-memorization guard for E29–E31); add a monotonic CI pass-count floor (ratchet) that fails if pass count drops; keep the report deterministic/offline-safe with a pass/fail breakdown for reviewers.
- **Delivered**: Nothing in-thread — no comments, no PR reference, no bot report; closed 2026-05-28.
- **VERDICT**: likely ignored — closed same day as the rest of the batch with zero delivery evidence for the suite growth, held-out variants, or the CI ratchet.
- **Deferred/follow-up**: none stated in-thread; #349 later complains testing with "bulk test suites with large datasets" is still needed, suggesting this remained unfulfilled.

### Issue #324: Issue with dialog: Сделай так, чтобы программа принимала путь как аргумент
- **Requirements (konard)**: (1) Answer in the detected language of the user's message (Russian message got an English answer); (2) add a setting choosing preferred language source — last-message language / selected preferred language / UI language, defaulting to last-message language; (3) truly support writing programs and changing code by request via logical reasoning and general algorithms ("translate collective human programmers experience into algorithm"), written as BOTH Rust code and links substitution rules (like link-cli `--always` triggers); ideal pipeline: reason → plan in links → Turing-complete substitution rules → compile to Rust/WebAssembly → execute; no memorized specific solutions — a universal dynamic problem-solving algorithm; (4) compile all logs/data to `./docs/case-studies/issue-324` with deep case-study analysis (timeline, all requirements, root causes, solution plans, existing-library survey, online research); (5) add debug output/verbose mode if root cause can't be found; (6) file upstream issues where relevant with repro/workarounds; apply fixes codebase-wide; (7) everything in one PR until "each and every requirement fully addressed".
- **Delivered**: No comments in-thread; closed 2026-05-29. Cross-thread: #330 (v0.149, next day) shows the same dialog now answered in Russian, and the "path as argument" follow-up now works — but via a new memoized `list_files_arg` template (see #340's "Supported tasks: hello_world, count_to_three, list_files, list_files_arg"), which is exactly the specific-solution memorization konard forbade.
- **VERDICT**: partially addressed — the language-detection bug was demonstrably fixed and the specific dialog now succeeds, but the fix for program editing was a hardcoded template, contradicting the explicit "universal dynamic problem solving algorithm / no memorization" requirement; no in-thread evidence for the language-preference setting, the plan→rules→wasm pipeline, or the case-study deliverables.
- **Deferred/follow-up**: language-preference setting; reasoning→substitution-rules→Rust/wasm compile-and-execute pipeline; case-study folder; upstream issue filing — none confirmed in-thread.

### Issue #326: E33: Universal multilingual operation vocabulary (all handlers trigger equally in en|ru|hi|zh)
- **Requirements (konard)**: "All languages are supported equally — if everything is supported in one language, everything should be supported in others." Single shared data-driven multilingual vocabulary seed (`data/seed/operation-vocabulary.lino`), not per-handler English literals; `seed::operation_vocabulary()` accessor + canonicalization helper; `text_manipulation` and `program_synthesis` match on the canonical view with no English regression; spec tests iterating over `supported_languages` asserting each operation triggers in every language; seed mirrored to `src/web/seed/`. Body states a first increment lands in PR #245 and "this issue tracks the full sweep across every remaining English-only handler."
- **Delivered**: Per the body itself, the first increment (text-manipulation + program-synthesis vocabulary) landed in PR #245. No comments; closed 2026-05-29 11:59, hours after filing, with no evidence the "full sweep across every remaining English-only handler" happened.
- **VERDICT**: partially addressed — the first increment is credibly delivered (PR #245), but the issue's stated purpose (the full sweep of all remaining English-only handlers) shows no delivery evidence before closure.
- **Deferred/follow-up**: full sweep of every remaining English-only handler — the tracked scope of this very issue — appears closed without confirmation.

### Issue #327: E34: Cross-runtime parity — JS browser worker mirrors Rust core synthesis (E28–E31)
- **Requirements (konard)**: "Make sure all Rust and JavaScript logic are in sync." Browser worker must derive synthesis/numeric/program/text answers with the same algorithm shape as the Rust core (E28–E31), not a separate seeded path; shared parity tests with a fixture set asserted equivalent between Rust solver and JS worker (extending E19 #282 harness); anti-memorization rule and benchmark ratchet must hold on the JS side; wasm remains the bridge, JS stays UI/glue.
- **Delivered**: Nothing in-thread — no comments, no PR reference; closed 2026-05-29 11:57 (same minute-range as #326), hours after filing.
- **VERDICT**: likely ignored — closed with zero delivery evidence for parity tests or the JS-side derivation; the browser dialogs in #332–#342 (same v0.149 wasm worker, after closure) still show unknown-intent fallbacks for tasks the epic claimed the Rust core solves.
- **Deferred/follow-up**: parity fixture tests and JS-side anti-memorization/ratchet — no confirmed disposition.

### Issue #330: Issue with dialog: Сделай так, чтобы программа принимала путь как аргумент
- **Requirements (konard)**: (1) Syntax highlighting in chat UI messages; (2) a copy button on each code block; (3) small buttons to copy the entire message as markdown; (4) e2e tests proving these work in the web app; (5) code examples must include instructions on how to run and test them; (6) double-check general solutions are used, "not some hardcoded memoization. Real reasoning as per our vision"; (7) case-study data compiled to `./docs/case-studies/issue-330` with requirement list and solution plans incl. existing-library survey; (8) all in one PR until everything is done.
- **Delivered**: No comments in-thread; closed same day (2026-05-29). Cross-thread: #349's dialog (v0.152) shows program answers now include "Как проверить это самостоятельно" run/compile/test instructions — requirement (5) demonstrably delivered. No evidence in any thread for syntax highlighting, copy buttons, or e2e tests; and #349 (next day) shows the follow-up edit "sort in reverse order" hitting intent-unknown, i.e., requirement (6) general reasoning still not achieved.
- **VERDICT**: partially addressed — run/test instructions in code examples visibly shipped, but there is no delivery evidence for the UI features (highlighting, copy buttons, e2e tests), and the "no hardcoded memoization" requirement is contradicted by #349.
- **Deferred/follow-up**: syntax highlighting, per-code-block copy, copy-message-as-markdown, e2e tests, general (non-memoized) code editing — none confirmed.

### Issue #332: Unknown prompt: Привет давай знакомиться!
- **Requirements (konard)**: none — bot-filed (labtgbot) dialog report with an empty Description; a Russian greeting "Привет давай знакомиться!" fell through to intent-unknown. No konard comments.
- **Delivered**: No comments, no delivery evidence; closed 2026-05-30.
- **VERDICT**: can't tell — no explicit requirements and no in-thread evidence of any fix (a greeting/small-talk handler) before closure.
- **Deferred/follow-up**: none stated.

### Issue #333: Unknown prompt: какой курс долора у тебя при расчетах?
- **Requirements (konard)**: comment (2026-05-29): "We can probably use call to https://github.com/link-assistant/calculator, so we can get requested data for the user." — i.e., integrate the calculator project so currency-rate questions get answered.
- **Delivered**: No response after konard's comment; no bot report, no PR reference; closed 2026-05-30, less than a day later.
- **VERDICT**: likely ignored — konard's concrete integration suggestion received no visible reply or implementation before closure.
- **Deferred/follow-up**: integration with link-assistant/calculator for exchange-rate answers — unhandled in-thread.

### Issue #334: Issue with dialog: Write a Python function that calculates the Fibonacci sequence recursively...
- **Requirements (konard)**: comment (2026-05-29): "continue to increase generalization of our problem solving and coding skills... so user can actually do all the programming tasks imaginable"; compile logs/data to `./docs/case-studies/issue-334` with deep case-study analysis (timeline, all requirements, root causes per problem, solution plans, existing-library survey, online research); add debug/verbose mode if root cause not findable; file upstream issues with repros/workarounds where relevant; apply fixes codebase-wide; everything in a single PR until fully done.
- **Delivered**: No response after konard's comment — no bot delivery report, no PR reference; closed 2026-05-29 20:23, ~1.5 h after the comment.
- **VERDICT**: likely ignored — closed within hours of konard's detailed instructions with no visible response or delivery evidence.
- **Deferred/follow-up**: the case-study folder, root-cause analysis, generalized Python synthesis (recursive Fibonacci + arithmetic chaining) — all unconfirmed.

### Issue #335: Issue with dialog: Search Wikipedia for "Nikola Tesla" and "Thomas Edison"...
- **Requirements (konard)**: none — bot-filed report, empty Description, no konard comments. Underlying failure: all CORS web-search providers returned nothing for multi-part search/compare/summarize prompts.
- **Delivered**: No comments, no delivery evidence; closed 2026-06-01.
- **VERDICT**: can't tell — no explicit requirements and no in-thread evidence of a search-provider fix.
- **Deferred/follow-up**: none stated.

### Issue #336: Issue with dialog: If I invest $1000 at 8% annual interest compounded monthly...
- **Requirements (konard)**: none — bot-filed report, empty Description, no comments. Underlying failure: compound-interest word problem + web currency conversion both unhandled (intent unknown for both plan steps).
- **Delivered**: No comments, no delivery evidence; closed 2026-06-01.
- **VERDICT**: can't tell — nothing in-thread shows whether financial word problems were ever made solvable.
- **Deferred/follow-up**: none stated.

### Issue #337: Issue with dialog: Navigate to github.com/link-assistant/formal-ai. Extract information...
- **Requirements (konard)**: none — bot-filed report, empty Description, no comments. Underlying failure: repo-metadata extraction misrouted to a "message-driven configuration" answer; "format this as a JSON object" fell to intent unknown.
- **Delivered**: No comments, no delivery evidence; closed 2026-06-01.
- **VERDICT**: can't tell — no requirements stated and no fix evidence in-thread.
- **Deferred/follow-up**: none stated.

### Issue #338: Unknown prompt: I have 3 boxes. Box A has twice as many apples as Box B...
- **Requirements (konard)**: none — bot-filed report, empty Description, no comments. Underlying failure: multi-step algebra word problem (GSM8K-style, the exact E29 domain) fell to intent unknown in the wasm worker.
- **Delivered**: No comments, no delivery evidence; closed 2026-06-01.
- **VERDICT**: can't tell — no stated requirements; notable as live evidence that E29-style word problems still failed in the browser after the E29/E34 epics were closed.
- **Deferred/follow-up**: none stated.

### Issue #339: Issue with dialog: Search for information about: 1. Machine learning algorithms...
- **Requirements (konard)**: none — bot-filed report, empty Description, no comments. Underlying failure: multi-topic search + comparison-table request fell to intent unknown.
- **Delivered**: No comments, no delivery evidence; closed 2026-06-01.
- **VERDICT**: can't tell — nothing in-thread beyond the failure report.
- **Deferred/follow-up**: none stated.

### Issue #340: Issue with dialog: Write a Rust program that: 1. Makes an HTTP GET request to a URL...
- **Requirements (konard)**: comment (2026-05-29): same block as #334 — increase generalization of coding skills ("all the programming tasks imaginable"), case-study folder `./docs/case-studies/issue-340` with timeline/requirements/root causes/solution plans + online research, debug/verbose mode if needed, upstream issues with repros, codebase-wide application, single exhaustive PR. Failure shown: `write_program_unsupported` — "no template for language `rust` and task `missing`" (template-lookup synthesis).
- **Delivered**: No response after konard's comment; no bot report or PR reference; closed 2026-05-29 23:47, ~5 h after the comment.
- **VERDICT**: likely ignored — closed hours after konard's instructions with no visible response, case study, or synthesis generalization.
- **Deferred/follow-up**: general (non-template) Rust program synthesis; case-study deliverables — unconfirmed.

### Issue #341: Issue with dialog: Design a simple web scraper in Python that: 1. Fetches a webpage...
- **Requirements (konard)**: comment (2026-05-29): same generalization + case-study + debug/verbose + upstream-issues + single-PR block as #334/#340, for the scraper-design dialog whose step 2 (test by scraping wikipedia.org) fell to intent unknown.
- **Delivered**: No response after konard's comment; closed 2026-05-29 20:24 — roughly 90 minutes after the comment, with no delivery evidence.
- **VERDICT**: likely ignored — closed almost immediately after konard's instructions, nothing in-thread shows any of them executed.
- **Deferred/follow-up**: executing/testing generated code (step-2 class of requests); case-study folder — unconfirmed.

### Issue #342: Issue with dialog: I want to build a budget calculator. Here's what I need: 1. Search for averag...
- **Requirements (konard)**: none — bot-filed report, empty Description, no comments. Underlying failure: 5-part compound task (web research + Python program + compound-interest math + comparison table + markdown report) collapsed to `write_program_unsupported` "no template for language `python` and task `missing`".
- **Delivered**: No comments, no delivery evidence; closed 2026-05-31.
- **VERDICT**: can't tell — no explicit requirements; another live demonstration that template-based `write_program` (contra #315/#324) was still in place.
- **Deferred/follow-up**: none stated.

### Issue #343: Unknown prompt: как сделать SPEC dirven development? напиши по шагам
- **Requirements (konard)**: none from konard. Author digitalstructures; their comment: there's a typo "dirven → driven" and "желательно, чтобы он понимал такие опечатки и мог сам исправить их" (the system should understand such typos and correct them itself).
- **Delivered**: No response to the typo-tolerance request; no delivery evidence; closed 2026-06-01.
- **VERDICT**: can't tell — no konard requirements; the reporter's typo-tolerance/fuzzy-matching request shows no in-thread response (flag for follow-up).
- **Deferred/follow-up**: typo-tolerant prompt understanding (requested by digitalstructures) — unhandled in-thread.

### Issue #347: Add /download page similar to what we have at https://github.com/konard/vk-bot-desktop
- **Requirements (konard)**: (1) /download page with exactly all vk-bot-desktop features (well designed, respects themes/switching, screenshots generated in CI/CD); (2) Linux/Windows/macOS builds of the application; (3) default in-process agent similar to link-assistant/agent; (4) optional server API (off by default) with docs for configuring claude/codex/agent CLIs against the local formal-AI server; (5) reuse web-app code inside Electron; direct connect to the GitHub-hosted app for local-DB sync; web app auto-detects local API server; extend web app by routing HTTP/tool calls/code execution to local app + dockers; (6) ideally implement lino-rest-api plus a universal LinksQL (link-cli idea + GraphQL features); (7) OpenAI-compatible REST APIs only for compatibility, Links Notation preferred elsewhere; (8) compare all files against the four link-foundation CI/CD pipeline templates (js/rust/python/csharp), adopt best practices, and report issues found back to the templates; (9) case-study folder `./docs/case-studies/issue-347` with requirement list and solution plans; (10) everything in a single PR until totally done.
- **Delivered**: Nothing in-thread — zero comments, no PR reference, no bot report; closed 2026-05-30, roughly a day after filing.
- **VERDICT**: likely ignored — an unusually large multi-part feature request closed within ~a day with no visible delivery evidence for any of its ten requirement clusters.
- **Deferred/follow-up**: effectively the entire scope: download page, desktop builds, in-process agent, local server API + CLI docs, Electron reuse/sync, lino-rest-api/LinksQL, CI/CD template comparison + upstream reports, case study.

### Issue #349: Unknown prompt: Сделай сортировку результатов в обратном порядке
- **Requirements (konard)**: "It will sure will not work, if I have to report issue on each and every message" — deep rethink so the system understands each symbol/word/meaning/requirement; actual reasoning on the semantic meta-language (binary links / links notation), "not some memorized only rules" — when no rule exists, reason to construct one as a human would; "current solution is still fake": plan issues in the repo with the `gh` tool fully covering coding (initial drafts, code editing, iterating until errors solved); use legally usable tests/examples/datasets from AI projects (download non-public-domain data only at test time); rethink architecture with best practices, bulk test suites with large datasets, diagnostics exposing every reasoning step; white-box (not neural-net black-box) self-improving system; **deliverable: a full plan as GitHub issues, each blocked via GitHub API by its dependency issues, each detailed enough for "even weakest AI systems" to implement**; plus the standard case-study folder (`./docs/case-studies/issue-349`), debug/verbose additions, upstream issue filing, codebase-wide fixes.
- **Delivered**: Nothing in-thread — no comments, no PR reference, no bot report; closed 2026-06-01, two days after filing. The triggering failure (a follow-up edit request "sort results in reverse order" hitting intent unknown at v0.152) is itself evidence that the previously closed synthesis epics (#315/#324/#330) had not delivered general code editing.
- **VERDICT**: likely ignored — the central deliverable (a dependency-linked issue plan on GitHub) and the architecture-rethink requirements show no in-thread confirmation before closure.
- **Deferred/follow-up**: dependency-blocked GitHub issue plan; dataset-driven bulk test suites; reasoning-step diagnostics; white-box self-improvement architecture; case-study folder — all unconfirmed.

## Verdict summary (chunk 6, 21 issues)
- fully addressed: 0
- partially addressed: 3 (#324, #326, #330)
- likely ignored: 10 (#315, #316, #317, #327, #333, #334, #340, #341, #347, #349)
- can't tell: 8 (#332, #335, #336, #337, #338, #339, #342, #343)
