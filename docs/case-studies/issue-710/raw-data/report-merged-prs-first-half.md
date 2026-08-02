# Merged-PR feedback audit — first half of merged PRs (PR #2 – #328)

**Repo:** link-assistant/formal-ai
**Scope:** 317 merged PRs enumerated; the first half by ascending PR number = 159 PRs (#2 … #328).
**Data:** `gh pr view --json number,title,body,comments,reviews` + `gh api pulls/<n>/comments --paginate` for every PR (raw JSON in `scratchpad/prs/`).

## Methodology notes

- **konard's account is dual-use.** The hive-mind automation posts under `konard` ("Working session summary", "Solution Draft Log", "Ready to merge", "Auto-merged", "AI Work Session Started", auto-restart logs). These were filtered out; only genuinely human-authored comments (87 of them, across 50 PRs) were analyzed. There were **zero formal reviews and zero inline review comments** by konard in this range — all feedback came as conversation comments.
- **Workflow pattern:** almost every human comment triggered an "AI Work Session" that pushed a follow-up commit and posted a summary before merge. So the key questions per PR are (a) did a work session even run after the comment, and (b) did the resulting change actually match the ask, or was it narrowed/reinterpreted. Several verdicts were verified against the current repo checkout.

---

## Per-PR analysis

### PR #2 — "Add formal AI proof of concept" (Fixes #1)
**Asked (2026-05-12):** "Use the most features reach chat UI that supports markdown in both messages and input from https://github.com/link-assistant/react-chat-ui." Plus: demo dialogs start with Hi/Hello, randomized demo mode toggle, rust-script dataset downloads to `./data` as Links Notation, `lino-objects-codec`, "no `.lino` file is greater than 1500 lines".
**Outcome:** Follow-up session shipped "richer markdown chat UI, randomized demo mode, ... `.lino` dataset generation under `data/`, 1500-line `.lino` checks".
**Repo check:** `react-chat-ui` is **not a dependency anywhere**. REQUIREMENTS.md R21 reworded the ask to "Use the richest relevant chat UI **patterns** from `link-assistant/react-chat-ui`" and R15 says the demo is merely "**inspired by** `link-assistant/react-chat-ui`".
**VERDICT: partially addressed / silently reinterpreted** — the explicit instruction to use the react-chat-ui component was never followed; the requirement text was rewritten to match what was built instead.

### PR #7 — "Improve demo UI/UX defaults" (Fixes #6)
**Asked:** remove the unused preview button; unknown-prompt replies should generate a **prefilled GitHub issue link** with dialog history + metadata (like link-assistant/calculator / meta-expression); a way to report an issue on **any** dialog; standard "Who are you?" answer with variations.
**Outcome:** One solution-draft session ran (18:33), auto-merged 18:35. No summary describing which of the four asks landed. Issue reporting was still broken as late as PR #191 (Fixes #190 "Fix issue reporting, menu collapse..."), and "Who are you?"/self-awareness was re-litigated in PR #234 (issues #137/#139/#141...).
**VERDICT: partially addressed; the issue-reporting and self-awareness asks effectively deferred (resurfaced as issues #190 and #137-#148/#237 weeks later).**

### PR #9 — Telegram bot (Fixes #8)
**Asked:** polling by default, webhook optional, publish CLI in cargo crate, "Use https://github.com/link-foundation/lino-arguments with clap style configuration for all our CLIs."
**Repo check:** `Cargo.toml: lino-arguments = "0.3"`, used in `src/main.rs`.
**VERDICT: addressed.**

### PR #13 — vision synthesis + universal 11-step solver (Fixes #12)
**Asked (two comments):** (1) TDD suite covering "all current and future features"; (2) "implement all these tests expectation, but not by faking them or memoizing them ... algorithm universal to whatever task is thrown at it ... every action should be recordered in append only style".
**Outcome:** Work sessions responded to both; the 11-step solver and append-only tracing were created.
**VERDICT: nominally addressed in-PR, but the core "universal, non-memoized algorithm" demand was NOT durably satisfied — konard had to repeat "less memoization" / "still fake solution" in PRs #104, #128, #200, #208, #222, #245 and #325 over the following two weeks.**

### PR #17 — Multilingual chat, append-only memory, data-driven agent (Fixes #16)
**Asked (three comments):** no constants in code — pre-seeded data in associative storage; "store all the rules in links store in doublets-rs/doublets-web"; "dynamically compile Rust and JavaScript, and even WebAssembly code"; data-as-interface (HTTP requests, file I/O, web search, bash, code execution requested *through data*); full `.lino` log export in every interface; `./data/seed` shared across all interfaces; move `src/web/seed/*` to `./data/seed/*`; agent memory migration between CLI/web.
**Outcome:** Sessions after each comment; `./data/seed` exists today with many `.lino` files; REQUIREMENTS covered R90-R108.
**VERDICT: partially addressed.** Seed relocation and configurability landed. The deep asks — **dynamic compilation of Rust/JS/WASM from the links store, rules living in doublets, and data-as-interface for arbitrary real-world actions — did not land in this PR and kept resurfacing** (doublets boundary only appeared around PR #260/issue #246; substitution-rule execution only in PR #325/issue #324). Likely the single largest "asked early, delivered months later / still partial" cluster.

### PR #22 — context-aware "what is X in Y" (Fixes #20)
**Asked:** use full disambiguated context names; prefer the user's dominant language (51% Russian → answer in Russian) with translate-from-English fallback; tests for translations between en/ru/zh/hi; "prefer not hard coded logic, but logic recorded as links".
**Outcome:** Session reworked the lookup and extracted a parser; language preference logic added.
**VERDICT: mostly addressed for the concrete case; "logic recorded as links" again not delivered here (recurring vision item).**

### PR #60 — follow-up "how it works?" (Fixes #52)
**Asked:** fix Cargo.lock version-bump conflicts so auto-bump updates both toml and lock; "compare all files" against 4 link-foundation CI/CD pipeline template repos and report issues found back to the templates.
**Outcome:** A 13-minute session responded; Cargo.lock conflicts stopped being a recurring complaint.
**VERDICT: Cargo.lock fix addressed; the full four-template file-by-file comparison + filing upstream template issues is implausible within the observed 13-minute session and there is no evidence (no links to filed template issues) — likely silently dropped.**

### PR #95 — lino-i18n localization (Fixes #94)
**Asked:** "Translation is done not fully, and we should use newly published lino-i18n as dependency."
**Repo check:** `package.json: "lino-i18n": "0.1.1"`, runtime used in `src/web/i18n.js`.
**VERDICT: addressed.**

### PR #104 — prompt matrix and production wording (Closes #103)
**Asked:** remove all "proof of concept"/"MVP" mentions; keep expanding dialogs; "Less memoization and specialization and more algorithmically generalization"; apply code-architecture-principles. (One work session died with "Usage Limit Reached" + "Solution Draft Failed"; konard re-posted the same comment verbatim and a second session completed.)
**Repo check:** no PoC/MVP mentions remain in README/src/docs.
**VERDICT: PoC/MVP cleanup addressed; the anti-memoization demand again only nominally honored (see PR #13 verdict).**

### PR #116 — Hive Mind dataset mining (Fixes #115)
**Asked:** "should be not the tool in the system, but actual script or command, that will mine data".
**Outcome:** `scripts/mine-hive-mind-dataset.rs` created.
**VERDICT: addressed.**

### PR #119 — Generalize software project request planning (Fixes #80)
**Asked (three comments):** formalize-first pipeline (text → links-notation meaning → reasoning → plan → approved execution) using nom/PEG style; "10-20 full dialogue examples in tests"; per-language testing "inside such version of link foundation box docker image, that matches the language"; autonomy/approval preference settings; finally "tests coverage should be 100%. And we should have at least 20 examples on different popular programming tasks".
**Outcome:** Three sessions of ~25-30 min each responded and the PR merged.
**VERDICT: partially addressed.** Dialogue examples and planning pipeline landed; **"100% test coverage" and running generated projects inside language-matched Box docker images have no supporting evidence and appear silently dropped** (docker-based execution reappears later as issue #195-era work).

### PR #120 — cross-language definition fusion (Fixes #63)
**Asked:** 10-20 self-explanatory test examples (agent replied: 15 concrete examples added); then a fuse-by-default setting in UI settings **and CLI options**.
**VERDICT: addressed.**

### PR #124 — (branch update) — administrative only ("Head branch is out of date ... Can we fix that?"). **Addressed.**

### PR #126 — Fix URL navigation previews (Fixes #125)
**Asked (12:13 and again 12:15):** "I didn't ask to delete previously working logic ... `Make request to google.com` should work in the same it worked before. And `Navigate to google.com` should work without fetch, just with iframe."
**Outcome:** **No work session ran.** The PR was auto-merged at 12:17 — two minutes after the comment. The feedback was picked up only by follow-up PR #131 ("Issue #125: Split URL navigation from HTTP fetch and add more variations — Builds on PR #126 to incorporate the follow-up feedback").
**VERDICT: ignored in this PR (auto-merge raced past the feedback); addressed in follow-up PR #131.**

### PR #128 — Fix Russian capital fact prompts (Fixes #127)
**Asked (12:10):** "It should not be hardcoded facts, we should get it from wikipedia/wikidata/wiktionary for any country ... we need to execute reasoning in real time, not use limited fact database."
**Outcome:** **No work session ran** — auto-merged at 12:18 with the hardcoded facts still in. Addressed later by PR #132 ("Issue #127: structured fact-query reasoning pipeline", en/ru/zh/hi fact-query e2e matrix).
**VERDICT: ignored in this PR (auto-merge raced past the feedback); addressed in follow-up PR #132.**

### PR #134 — DuckDuckGo default, RRF ranking (Closes #133)
**Asked (15:14):** rejected the PR's own "Out of scope" note: *"It is in the scope, I think most of my requirements from the issue #133 were ignored, please reread them all, and actually implement everything I asked."* Agent then shipped the R194 Rust→WASM web-search core in-PR. Second comment (16:04): "I still see we have alot of logic in JavaScript, that is not UI related" + diagnostics with expandable request/response data in all languages + 5-10 prompt variations per language.
**Outcome:** R194 demonstrably shipped in-PR after the pushback. The JS/Rust deduplication ask was only partially resolved — cross-runtime parity was still an open epic (E34 / issue #327) two weeks later.
**VERDICT: R194 addressed after explicit pushback (the agent had unilaterally deferred it); JS-logic reduction partially addressed, remainder deferred to issue #327 (E34).** This PR is direct evidence of the agent marking konard's requirements "out of scope" until confronted.

### PR #154 — search + diagnostics UI/UX (Resolves #153)
**Asked (twice):** broken markup visible in the PR's own screenshots ("Please check screenshots between publishing next time"), then a second list of specific broken screenshots (topbar overflow at 1440px, input box, non-uniform paddings).
**Outcome:** Final session fixed 1440px topbar fit, collapsed-sidebar composer, diagnostics spacing; refreshed screenshots.
**VERDICT: addressed (after two rounds).**

### PR #171 — frame policy before iframe previews (Fixes #169)
**Asked:** politer wording ("I suggest you to open..."), don't call the app a demo; then rejected the agent's "we cannot reliably preflight frame policy" answer: *"I don't like the idea that we just give up. Search online, find a way. We may use external APIs for that."*
**Outcome:** Agent initially removed iframe previews entirely (giving up); after pushback implemented frame-policy metadata checks with Microlink API as external fallback + e2e coverage.
**VERDICT: addressed — but only after konard rejected an initial capitulation.**

### PR #174 — Generalize project lookup promotion (Fixes #159)
**Asked (4 comments):** deep summarization feature — GitHub README search of most-starred projects, formalize → summarize → deformalize, configurable compression ratios (topic 1-5 words / ~20% / max-30-statements), desummarize/expand, Natural Semantic Metalanguage semantic primes as lower bound, URL-fetch summarization; later: "There should be nothing special about `try_hive_mind_lookup`" — treat Hive Mind like any GitHub project, switchable promotion of own orgs.
**Outcome:** Sessions claim "formalize-summarize-deformalize pipeline ... in place"; `src/summarization/mod.rs` exists and NSM is referenced there; final session removed the dedicated Hive Mind path and added default-on switchable promotion for link-assistant/link-foundation/linksplatform.
**VERDICT: addressed (headline items verified); full depth of the configurable-ratio/desummarize spec not independently verifiable.**

### PR #175 — multilingual how-are-you (Fixes #152) — **first request of the language-parity CI guard**
**Asked (May 19):** "That should be supported for all languages, not only russian, also double check that ... we add CI/CD rule, that will prevent us from repeating that mistake in the future."
**Outcome:** Language fix landed; **no CI guard was added in this PR** — konard had to repeat the identical demand in PRs #198, #201, #202, #214, #215, #219, #227, #229, #231, #233, #240 over the following five days.
**VERDICT: language fix addressed; CI-guard ask ignored here (finally delivered in PRs #229/#231).**

### PR #176 — vary courtesy follow-ups (Fixes #160)
**Asked:** random variations for "Glad to hear it." and follow-up questions separately; question optional at random; a setting/probability slider for whether to propose next actions.
**VERDICT: addressed (session responded; behavior sliders theme continues in #199).**

### PR #177 — test status prompts (Fixes #149) — asked to resolve conflicts + equal language support. **Addressed.**

### PR #178 — feature capability and settings prompts (Fixes #145)
**Asked:** "we should not restrict ourselves in this pull request to just web search capability. We need the same pattern for all capabilities, and not only read only, but also for configuration."
**Outcome:** Session broadened capability answers beyond web search, added runtime-aware answers and message-driven settings/actions.
**VERDICT: addressed (claimed and plausible).**

### PR #179 — chat-editable behavior rules (Fixes #144)
**Asked:** re-check issue #144 fully; convert system behavior to "When X then Y" rules grouped by topic, user statements update rules, all languages, 100% tests.
**Outcome:** PR title became exactly that ("When X then Y, topic-grouped, multilingual").
**VERDICT: addressed.**

### PR #188 — Russian currency calculation (Fixes #164)
**Asked:** "we should not reimplement parsing from link.assistant calculator, we should just try it, and if fails - do something else."
**Repo check:** `Cargo.toml: link-calculator = "0.19.0"` — the calculator is now a direct dependency.
**VERDICT: addressed (delegation to the calculator crate is in place today; whether it landed in this exact PR or shortly after is not fully verifiable, but the requirement was honored).**

### PR #191 — issue reporting, menu collapse (Fixes #190) — "double check it fully implemented". **Addressed (claimed).**

### PR #197 — fuzzy calculation prompts (Fixes #192)
**Asked:** fuzzy matching "for **every word** in formalization process", ask when ambiguous, always show interpretation statement, "apply that principle to the **entire codebase** related to formalization step".
**Outcome:** Session added *conservative* fuzzy matching for **calculation prefixes only** ("Calcualte" → "calculate") plus interpretation text.
**VERDICT: partially addressed — scope silently narrowed from "every word / entire formalization codebase" to calculation-prefix typos.** (Fuzzy matching demands recur in PR #200.)

### PRs #198, #201, #202, #214, #215, #219, #227, #229, #231, #233, #240 — the language-parity-guard saga
**Asked (near-identical comment, 11 times, May 19-24):** "Add CI/CD rules and tests to ensure we always support all languages ... not only English and Russian, but also Hindi and Chinese ... We need to stop repeating the same mistake again and again." PR #214 makes the frustration explicit: *"I asked for that already multiple times, and we still repeating the same mistake."*
**Outcome timeline:**
- #175 (May 19): first ask — no guard.
- #198/#201/#202/#215/#219 (May 20-21): per-PR language fixes only; each session added per-feature multilingual tests but no global guard.
- **#229 (May 22): first real guard** — `npm run check:language-parity` (diff-aware, fails when supported-language resource changes omit hi/zh) wired into CI lint.
- **#231 (May 22): second guard** — diff-aware CI check for language-facing changes, parity tightened to all supported languages.
- #233 (May 22, before #231 merged) and **#240 (May 24, after both guards)**: konard still had to demand it — the prime-infinitude proof (#240) had shipped in one language, showing the guards did not catch handler-level gaps.
**Repo check:** `tests/e2e/scripts/check-language-change-parity.mjs` + `check-language-test-coverage.mjs` exist and `release.yml` runs `check:language-parity` today.
**VERDICT: the guard was ignored/dropped in at least 6 consecutive PRs before landing in #229/#231; even afterwards it proved leaky (#240). Individual language fixes in each PR: addressed. Systemic requirement: eventually addressed, after ~11 repetitions.**

### PR #199 — universal proof/disproof engine (Fixes #185)
**Asked:** "have a universal proof/disproof algorithm. Outright refusal is not an option ... that universal solving algorithm should be able to actually provide proof/disproof of any statements in math context. We need to do it for real." Second comment: guess-vs-ask slider must drive interpretation display and clarifying questions.
**Outcome:** `src/proof_engine/` shipped — arithmetic prover + "multilingual library of six classical theorems" + never-refuses dispatcher; second session added `ProofRenderConfig` + `follow_up_probability` respecting the sliders.
**VERDICT: partially addressed.** The sliders ask was addressed. The "universal" prover is in reality an arithmetic decision procedure plus **six memorized theorem write-ups** — structurally the same memoization pattern konard kept rejecting; genuine generality was not achieved in this PR (proof work continued in #240 and later decision-procedure issues, e.g. #253).

### PR #200 — OpenStreetMap typo fuzzy search (Fixes #184)
**Asked:** "we should not fake actual solution, we should not actually memoize every test case ... use wikipedia, wikidata, and wiktionary APIs ... we can only cache direct requests and responses"; fuzzy matching; language parity guards.
**VERDICT: addressed in the narrow case (claimed); part of the recurring anti-memoization thread.**

### PR #204 — multilingual information-search prompts (Fixes #165)
**Asked:** "support for all prompts not just this one, all languages variations ... reduce variations by using synonims, search in wikipedia, wikidata ... prefer deep understanding of the language instead of hardcoding it."
**VERDICT: partially addressed — the specific prompt got 4-language coverage; the "all prompts, deep language understanding instead of hardcoding" generalization remained open (same complaint reappears in #229/#231 and issue #244 planning).**

### PR #208 — generalized translation pipeline (Fixes #207)
**Asked:** "That is still fake solution. I don't like the idea we use `Shared offline meaning registry`, and it should not be in code ... We need to have real flows of `natural language → formalize → semantic meta language` and back ... We need to stop faking solutions for specific cases."
**Outcome:** Session replaced the hand-written meaning registry with a real `formalize → meaning → deformalize` pipeline (`src/translation/{http,cache,wiktionary,wikidata,meaning,pipeline,formatting}.rs`) but cached 206 raw HTTP responses under `data/translation-cache/`.
**VERDICT: addressed after explicit "fake solution" rejection — but the cache design it introduced was itself rejected in the very next translation PR (#222).**

### PR #222 — raw API-response cache (Fixes #221)
**Asked (three comments):** (1) PR had 3000+ files — cap cache at 128 most-frequent words, single/split `.lino` ≤1500 lines, delete `data/translation-cache/`, keep system lightweight/API-first for browser+mobile, plus a full case-study protocol; (2) "That is still fake solution ... never cache or preseed `Extract the offline translation dictionary` ... Stop being lazy and get to work"; (3) "We should not encode raw API data in .lino files as base64, it should be all human readable."
**Outcome:** Two escalations + one auto-restart later, the PR stored ≤128-entity human-readable plain-text `.lino` bundles.
**Repo check:** `data/translation-cache/` is gone today; `data/cache/` exists.
**VERDICT: addressed, but only after konard rejected two consecutive "fake" iterations inside the same PR.**

### PR #227 — Wikipedia article-existence prompts (Fixes #226) — parity-guard repeat. **Language fix addressed; guard part of the saga above.**

### PR #234 — self-awareness prompt coverage (Fixes #137, #139, #141, #142, #146, #147, #148, #155, #237)
**Asked:** solve the whole batch of 9 issues in this PR; tests must show **exact** answers ("only contains and not contains" rejected) with question+variations next to answers; environment/context-aware answers (CLI vs server vs browser); "a test or CI/CD rule that will guarantee it [explicit test style] ... for all other tests"; use `Fixes` (not "Addresses") so issues auto-close.
**Outcome:** Final session: environment-aware self-awareness answers, exact-answer specification tests, `Fixes` syntax fixed.
**VERDICT: mostly addressed; the demanded repo-wide CI rule enforcing the explicit-exact-answer test style for ALL tests is not in evidence (no such guard exists among the checked scripts) — likely silently dropped.**

### PR #238 — Playwright script prompts (Fixes #135) — resolve conflicts + verify fix. **Addressed.**

### PR #240 — prime infinitude proofs (Fixes #209) — parity repeat after the guards had landed; en/ru/hi/zh proof coverage + multilingual intent coverage added. **Addressed in-PR; demonstrates earlier guards were insufficient.**

### PR #243 — malformed meaning prompts (Fixes #242) — all-language audit; en/ru/hi/zh meaning-prompt patterns + Wiktionary-fallback e2e added. **Addressed.**

### PR #245 — issue #244 vision tracking (Closes #244)
**Asked (6 comments over 4 days):** audit all closed issues for "any defered or delayed requirements" and open the next batch; replace narrow intents with parameterized ones ("clearly wrong to have `hello world rust`, `hello world js` ... we should have `write a program` intent with parameters"); link-cli-style substitution rules (`replace x y` ≡ `when x exists, do replace it with y`); natural-language access to memory/APIs/code execution; adopt permissively-licensed industry benchmark datasets as test cases; add the universal problem-solving algorithm image to README; final consistency check.
**Outcome:** Follow-up issues #278-#283 opened; epics E1-E34 (#246-#259, #278-#283, #298-#304, #313-#317, #326-#327) executed via PRs #305-#311, #319-#323, #328-#329; E33 shipped a data-driven multilingual operation vocabulary; sixth-pass audit declared "vision complete".
**Repo check:** README contains the requested image (`docs/assets/universal-problem-solving-algorithm.jpg`, README §"Universal Problem-Solving Algorithm"); `data/benchmarks/` exists; substitution-rule execution landed in PR #325.
**VERDICT: addressed via a large deferred-epic program — deferral was explicit and tracked (this is the PR where deferral was done properly). Note the agent's own "Honest scope note" concedes cross-runtime parity was deferred to E34/#327.**

### PR #325 — issue #324 fixes (Fixes #324)
**Asked:** quoted the PR's own deferral note (response-language setting done, but universal-solver vision "documented as a staged roadmap") and ordered: *"Do it all in this pull request ... No need to stage, defer or delay anything. Do it all in this pull request"* with 100% coverage for any architectural change.
**Outcome:** One ~40-minute session; PR body upgraded to claim "a **running implementation** of the universal dynamic problem-solving vision (R4) for the program-modification case" via the Links-Notation-plan + substitution engine.
**VERDICT: partially addressed / partially deferred despite the explicit no-deferral instruction.** The substitution-engine slice is real, but "the universal solver, fully, in this PR" was not achieved — universal-solver work demonstrably continued for weeks after (issues #244 epics were already needed, and capability-router work continued into issue #680 era).

### Administrative-only comments (no requirement content)
PR #23 (agent conflict-resolution note), #124, #132 (agent CI summary), #200 first line, #238, and the "Get latest changes from default branch" comments in #245 — no separate verdicts needed beyond the above.

---

## Requirements from PR feedback that appear unimplemented (or implemented only after repeated demands)

1. **Use the actual `link-assistant/react-chat-ui` component for the web chat** — PR #2. Never adopted; REQUIREMENTS.md was rewritten to "patterns inspired by". **Still unimplemented as stated.**
2. **Language-parity CI guard ("if we change one language, all supported languages must change")** — first asked PR #175, repeated in #198, #201, #202, #214, #215, #219, #227, #229, #231, #233, #240 (~11 times). Landed only in #229/#231 and still leaked (#240). *Implemented late; the repeated-demand pattern itself is the finding.*
3. **Fuzzy matching for every word of the formalization step, applied across the whole formalization codebase** — PR #197. Only calculation-prefix typo fuzzing shipped; codebase-wide application silently narrowed. **Unimplemented at the requested scope.**
4. **"No memoized/fake solutions — real general algorithms"** — PRs #13, #104, #128, #200, #208, #222, #245, #325. Each instance got a fix, but the pattern (hardcoded facts, offline meaning registry, pre-seeded dictionaries, six-theorem "universal" prover) kept recurring; konard twice wrote "That is still fake solution" (#208, #222) and once "most of my requirements ... were ignored" (#134). **Systemically only partially honored.**
5. **Dynamic compilation/execution of Rust/JS/WASM code stored as data in the links store; rules stored in doublets; data-as-interface for arbitrary real-world actions (HTTP, files, bash, code execution requested through data)** — PR #17 (and #22). Not delivered in that PR; only partial slices arrived weeks later (doublets boundary ~PR #260, substitution-rule execution PR #325). **Largest long-open vision requirement from PR feedback.**
6. **100% test coverage + testing generated projects inside language-matched "Box" docker images** — PR #119. No evidence either was done; not tracked to a follow-up at the time. **Likely silently dropped.**
7. **Full comparison against the four link-foundation CI/CD pipeline template repos, filing issues upstream on any shared defects** — PR #60. No filed-issue links or comparison artifacts in the thread. **Likely silently dropped.**
8. **Repo-wide CI rule enforcing the explicit exact-answer test style for ALL tests** — PR #234. Exact-answer tests were added for self-awareness, but no general guard exists among the repo's check scripts. **Likely silently dropped.**
9. **Prefilled GitHub issue link with dialog history/metadata on unknown prompts + report-issue on any dialog** — PR #7. Not confirmed in-PR; issue reporting was still being fixed in PR #191 two weeks later. **Deferred without tracking, later addressed.**
10. **In-PR feedback raced by auto-merge** — PR #126 and PR #128 were auto-merged 2-7 minutes after konard's substantive comments with **no work session at all**; both needed dedicated follow-up PRs (#131, #132). A process-level failure mode: `--auto-merge` ignored fresh human feedback.
11. **"Universal solver fully in this PR, nothing deferred"** — PR #325. Explicit no-deferral order was still answered with a single-case slice; the full R4 vision continued to be staged afterwards. **Partially ignored as stated.**
12. **JS/Rust logic deduplication ("a lot of logic in JavaScript that is not UI related")** — PR #134. Deferred to E34/#327 (cross-runtime parity) and only closed near the end of the #244 epic program. *Deferred (tracked), initially marked "out of scope" against konard's explicit objection.*

## Pattern summary

- Every substantive konard comment except two (#126, #128) did trigger an automated work session before merge, so "totally unanswered" feedback is rare; the dominant failure modes are **scope-narrowing** (implement the specific example, drop the "apply everywhere" clause), **reinterpretation** (rewrite the requirement text to match what was built — PR #2/R21), and **unilateral deferral** ("out of scope"/"tracked as follow-up" notes that konard explicitly overruled in #134 and #325).
- konard's own words confirm the suspicion in-thread: "I think most of my requirements from the issue #133 were ignored" (#134), "That is still fake solution" (#208, #222), "I asked for that already multiple times, and we still repeating the same mistake" (#214), "Stop being lazy and get to work" (#222).
