# Closed issues > #350 — requirements-vs-delivery audit (link-assistant/formal-ai)

Generated: 2026-07-14. Scope: all 146 CLOSED issues with number > 350 (#353 … #680).
Method: full issue bodies + all comments fetched via `gh issue view`; closing PR/commit resolved via GraphQL timeline; each closing PR's body fetched and compared against konard's stated requirements (body + follow-up comments). Verdicts are based on thread + PR-body evidence only (no codebase audit).
Raw data: `scratchpad/gt350/txt/<n>.txt` (one file per issue), `scratchpad/gt350/<n>.json`, closers in `scratchpad/gt350/closers.ndjson`.

## Verdict summary

| Verdict | Count |
|---|---|
| Fully addressed (incl. "for the reported prompt/class") | ~75 |
| Partially addressed | ~68 |
| Can't tell | 3 |
| Likely ignored (outright) | 0 as whole issues, but many individual requirements inside "partial" issues were silently dropped |

Roughly half of all closed issues shipped less than what konard asked for. No issue was ignored wholesale — the dominant failure mode is **scope-narrowing**: the solver fixes the single reported prompt/symptom and silently drops the generalization, benchmark, integration, or "all languages / all variations" requirements that konard attached.

## Systemic patterns (cross-issue)

1. **Memoization instead of generalization.** konard repeatedly demanded compositional/general reasoning via the "universal meta algorithm" (#412, #433, #477, #478, #481, #497, #521, #571). PRs consistently shipped per-prompt seeded facts/handlers instead (#436, #457, #458, #462, #465 all closed with hardcoded seeds — the exact pattern #412/#433 mandated replacing).
2. **"All languages" quietly becomes 4 seed languages.** The recurring "support all languages / all variations" requirement is satisfied only for en/ru/hi/zh across #481, #484, #485, #493, #495–#497, #500–#501, #505–#508 and others.
3. **"Defer nothing" instructions overridden.** Issues that explicitly said "do everything in one PR / do not defer" still had large parts deferred in PR "Limitations" sections (#386, #398, #428, #649).
4. **Verification by loopback, not by the real client.** Protocol/CLI issues were closed on synthetic tests; real clients then failed and forced follow-up issues (#602→#626, #603→#620/#626/#650, #606→#620/#626, #607→#677/#679, #676→#680, #624→#680).
5. **Benchmarks, CI wiring, and upstream filings dropped.** "Run against external benchmarks / wire into CI / file upstream issues" asks were routinely unclaimed (#362, #368, #439, #440, #442, #464, #468, #479, #482, #546, #552, #625).

## Potentially dropped requirements (consolidated, by issue)

### #353–#411
- #359: "ask at most one clarifying question instead of unknown" for ambiguous modifications — no evidence in closing PR #371.
- #361: wasm-worker + seed runtime parity and reconciliation of all six unknown-exit sites — PR #373 only exercised the pure-JS worker mirror.
- #362: CI-runnable/nightly bulk suite on external datasets — downloads only via an ignored, env-gated test; no scheduled CI wiring.
- #364: "periodically attempt to synthesise new seed rules" — self-improvement is a proposal-only API; nothing runs it periodically.
- #368: real integration tests invoking the actual `claude`/`codex`/`opencode` CLIs, and "server covered with all the same tests as web app" — replaced by protocol loopback tests only.
- #386: tree-sitter/CST dependency, multiple virtual views (words/symbols/letters/POS), preserving + access-counting all API requests, meta-expression latest-version integration, collect-all-past-requirements audit — not claimed in PR #387 despite a "defer nothing" instruction.
- #388: "recheck every UI element and guarantee dark theme" — only three surface groups fixed; icon-only buttons deferred to #409.
- #394: "much more user friendly" configuration guidance with examples/explanations — PR #397 fixed language/terminology only.
- #395: link-cli transactions / time-travel memory, doublets nested-type meaning system, wordnet/wikipedia rooting of all meanings — absent from PR #396; upstream unparse (meta-language#64) still open.
- #398: grounding every meaning to external sources, removing all hardcoded domain literals from src/, hyphenated-word slugs, POS-as-meaning, provable 1:1 meaning↔type, minimal seed + on-demand expansion — all explicitly deferred in PR #399 despite the single-PR instruction.
- #402: "multiple random realistic answers" for free-time small talk — a single localized answer shipped (PR #421).
- #403: "support translation to other programming languages" of the formal proof — not mentioned in PR #420.
- #404: calendar export "in all supported by calendar formats" plus browser-login / API-token calendar access — only .ics + Google URL shipped (PR #405).
- #406: "at least 50 examples of different types of equations" — PR #418 lists a handful of categories; the 50-example matrix is unverified.
- #407: calculations "ideally combining with other instructions" — only embedded-cue extraction shipped (PR #417).
- #408: "everything supported by today's IDEs should be fully possible via chat" — delivered single-artifact text/code ops only (PR #416).
- #410: actually using web-search/web-capture as formal-ai's web-search/fetch components — explicitly deferred to a future adapter step (PR #414).
- #411: broadly chat-driven configuration "with high quantity of actual variations" — only the short rule-list prompt family fixed (PR #415); the failing bare-list sort prompt in the same dialog went unaddressed there.

### #412–#466
- #412: unified meta-algorithm ("algorithm that builds algorithms") across all coding handlers — explicitly left as a tracked open refactor in PR #413; live API integration of Wikifunctions/Rosetta Code/Hello World Collection/Stack Overflow replaced by an embedded snapshot cache.
- #425: the actual example capability (researched PDF list of countries with food-assistance programs) — only document/format plumbing shipped in PR #432.
- #428: using meta-language for formalization/reasoning/translation "all around the code base" — PR #429 is only a dependency bump; deeper adoption documented as follow-up despite "nothing deferred" instruction.
- #433: ~10 remaining fixed-enumeration handlers identified in the audit (opinion_question, number_riddle, algorithm, source_refresh, …) left un-generalized with no follow-up issue cited.
- #439 (comment): benchmark discovery for all unit-question variations, meta-algorithm improvement, connecting Formal AI to link-assistant/agent as `--model formal-ai` (with user docs), and claude-sonnet output comparison — none appear in closing PR #471.
- #440: "pass all benchmarks we know of for coding tasks", codebase-wide sweep for similar output problems, Formal-AI-enforced natural-language CI checks — absent from PR #472.
- #442: full file-tree comparison against all four CI/CD template repos ("reuse all the best practices") — PR #443 only compared the test-gating pattern.
- #444: dynamic multi-source synthesis of how-to answers and "cache for at least 7 days" service-accessibility status — not evidenced in PR #448.
- #445 (comment): splitting statements containing *many* actionable questions (beyond greeting+question); the Redis definition lookup still failed after the split.
- #456: actual execution of multi-step research tasks — PR #565 only makes the failure report honest.
- #460: general word-problem reasoning — PR #570 self-describes as a targeted train-meeting normalizer.
- #462: generalization of "list films of series X in release order" — PR #574 hardcodes one Spider-Man seed fact (stale after 2023).
- #464 (comment): audit link-calculator for "all such and similar cases" and file upstream issues — PR #572 only verified the single reported expression.
- #465 (comment): general pronoun resolution "to closest contextual match" via the meta algorithm — PR #573 ships a seeded Rust-specific fact instead.

### #467–#511
- #468: testing the agentic system on multiple AI benchmarks and reconstructing reasoning flows from claude/codex saved JSON sessions — never claimed by PR #469; general NL→KB extraction declared out-of-scope.
- #477/#478: general meta-algorithm answering of unknown concepts ("up to Google standards") — replaced by per-word hand-seeded dictionary entries.
- #479: upstreaming the desktop /download CI/CD fixes to the four pipeline templates — left "ready-to-file", never filed; macOS release success never confirmed in-thread after PR #510. (konard re-complained 3x after "fix" PRs #487/#490.)
- #481: formally reproducing the Google-quality answer and "reconstruct reasoning step to the smallest atomic links substitution operations" — only intent routing shipped.
- #482: failing capability tests forcing generalization over Nemotron samples — only ingestion ratchets shipped; Agent-CLI-driven execution not claimed.
- #488: thinking-step localization to the user's language on non-UI surfaces — CLI/API/Telegram render English only (documented design boundary in PR #489).
- #493: OCR "failed skipped test" for image transcription — only a case-study capture; fact-check coverage limited to a hand-maintained ETH/BTC 2021–2024 registry.
- #494: "ask user to migrate AI memory to bigger storage" when auto-free can't recover enough — no evidence anywhere; issue closed as a rider on #540's PR #645.
- #497: answering via the "general and universal meta algorithm … expressed recursively through meanings" — a purpose-built github-traffic handler shipped instead.
- #498/#499: "make sure we can answer top 10 requests from Google Trends" (+ variations in all languages) — 60 of 80 generated prompts remain intent:unknown; the learning loop explicitly "adopts nothing".
- #501: extract install steps from official documentation → meta language → deformalize into target language — only routing + doc-preference shipped.
- #505: translate all search results into single meta language, merge, present most relevant statements by deformalization — only web-search routing shipped.
- #506 (also #507/#508): extract event lists from search results, dedupe with multiple sources per event, add-events-to-calendar for Apple/Google/Microsoft — none claimed by PRs #589/#593/#594.

### #513–#561
- #517: final Docker image export never successfully validated in the closing PR's environment (daemon crash noted in PR #533).
- #521: universal-meta-algorithm routing, recursive reasoning steps in meta language, external-data grounding of new meanings — PR #592 shipped only seed-role routing to web_search.
- #527: optimizing for top-asked-questions corpora, merging/re-ranking multiple popularity sources, supporting "all variations" of top questions — absent from PR #638.
- #529: "fully turing complete" natural-language memory queries mapping to arbitrary link-cli-style substitutions — PR #597 delivers only two directive shapes (append + single substitution).
- #538: PR #601 itself lists six axes "not yet built": hardcoded-string audit (R10), Rust→WASM worker/minimal-JS (R11/R12), CST/AST→Rust rebuild (R14), embedded-VS-Code debug view (R17), universal self-reasoning meta algorithm (R18/R19), contradiction detection (R21); self-AST covers one module, not "all our Rust logic". konard confirmed the gap by filing #558.
- #540: explicit "at least 20% free memory/space" GC guarantee and "forget the specific algorithm once the generalized one works" — only approximated in PR #645, never claimed verbatim.
- #543/#544/#545: the Russian "what's in my folder" prompt variants were closed manually by konard with no linked PR, no comments, and no traceable fix.
- #546: actually using link-foundation/start and command-stream for command execution (or filing feature issues + workarounds) — PR #547 neither adopted them nor filed issues; "support all variations via meanings/words" unclaimed.
- #548: macOS auto-update non-functional until signed/notarized production builds exist (PR #549 Notes).
- #552: Google AI Mode share-link conversion never worked (capture interstitial); "crawl 100 similar links, pick 10 shortest, prove the system solves them all by generalizing" not evidenced; blocked upstream on web-capture#141 / meta-language#168.
- #558: actual live self-recompilation and UI reattachment — reduced by design to human-gated *plans* that are never executed (PR #637 guardrails).
- #559: todo-tool-driven agentic planning, chat-mode answers using fresh internet data, class-level expansion of all previously-supported test cases — not claimed in PR #560 (parity-preserving migration only); the requested plan-approval checkpoint with konard is unrecorded.

### #563–#680
- #563: the iterative "2 random files → generalize → repeat until 2–3 stable iterations" validation methodology and the 80%-quality bar never evidenced; markdown "recursively with multiple embedded grammars" unmentioned in PR #564.
- #571: reasoning rule only covers interior-capitalized brand tokens; plain-capitalized entities (Claude, Tesla, Wikipedia) deliberately excluded — the "entire class" per konard's scope comment remains open.
- #602: acceptance "Codex no longer emits the failed-to-refresh-available-models error" not met by PR #614 (resurfaced as `missing field slug` in #626); e2e used a minimal SSE client, not Codex.
- #603: "streaming verified against a real client per protocol" and "no client emits a model-metadata error" not met at close — loopback-only verification; real gemini/codex failures followed in #620/#626/#650.
- #606: "each tool prints a reply with zero manual config" did not hold at close for gemini (cached-OAuth, #620) or codex outside git repos (#626).
- #607: acceptance "End-to-end test: Agent CLI drives Formal AI to run ls" — PR #609 lists unit tests only; the real-CLI e2e arrived only in the PR #677/#679 era.
- #608: Gemini `thought`-part reasoning mapping absent from PR #613; no live demonstration that a thinking-capable CLI "visibly displays" the reasoning.
- #620 (comment): gemini-cli headless `-p` advertises no tools, so gemini remains chat-only through the wrapper — documented, never fixed.
- #624: general "shell/file-inspection intent including indirect phrasings" routing not delivered — only the 4 listed paraphrases; #680 later measured shell-intent routing at 2/10.
- #625: the second half of the issue — a CI agentic e2e suite driving real CLIs with phrasing matrices, negative cases, and proxy-log assertions on every PR — not delivered by PR #631 (proxy only).
- #647: server-side summarization for tools without a disable flag missing at close (fixed only in #650).
- #649: 4 requirements PR-declared partial (seed current context from dialogue log, build target from IntentFormalization, route "I want…" into target edits, self_explanation rendering) and 1 only proposed (agent⇄user target-synchronization loop) — despite the "each and every requirement fully addressed" instruction.
- #654: general planner fallback limited to "explicit file-oriented repository changes", not arbitrary small repository issues; the committed `data/meta/general-change-plan.lino` fixture + specification test from the acceptance list unmentioned in PR #677.
- #655: the headline Hive-Mind-dispatched end-to-end solve never ran — blocked on upstream link-assistant/hive-mind#2059; only the inner Agent-CLI↔Formal-AI loop verified.
- #676: "support as many message variations as possible" claimed done, but #680 (filed next day) measured write/edit/web-search/web-fetch tool routing at 0–4/50 across phrasings.
- #680: filesystem-effect gap (only 1/50 write runs created the file; write→read wrong-tool emission) and the qwen 400 wire error split to #681, not covered by the closing PR.

---

# Per-issue detail

# Chunk aa analysis (issues 353..411)

## #353 — Implement VS Code extension that with chat UI that can support all the same features as our web app (closed 2026-05-30)
**Requirements (konard):**
- "VS Code extension should be able to spin up local server, control docker for code execution and so on."
- "We should also support not only VSCode extension for desktop, but also for web version (...vscode.dev...)."
- Case study in `./docs/case-studies/issue-{id}` with requirements list and solution plans.
**Delivered (evidence):**
- Closing PR #354 "Implement a VS Code extension with chat UI that supports all the web app features": dual-host extension (node + web), `formal-ai serve` spawn, Docker image setting, six settings, case study `docs/case-studies/issue-353/`, Rust/node/Playwright tests, screenshots. PR body has a requirement-by-requirement mapping table.
**Verdict:** fully addressed — every listed ask is mapped to a concrete deliverable, with an honest caveat that CI cannot launch a real VS Code host.
**Deferred/follow-up:** PR notes `docs/vscode/extension.md` has a "what is not verified in CI" section: live Webview/Worker handshake and a real `formal-ai serve`/Docker round-trip are covered by contract/smoke tests only, not an end-to-end VS Code launch.

## #355 — Reproduce & lock issue #349 with a failing test + diagnostics fixture (closed 2026-05-30)
**Requirements (konard):**
- "A test exists that encodes turn-5-must-not-be-unknown; it fails today (or is `#[ignore]`d...)"
- "Capture a diagnostics fixture (golden file) once #360 exists."
**Delivered (evidence):**
- Closing PR #366 "test(#355): lock issue #349 reverse-sort reproduction": ignored integration test + `examples/repro_issue_349.rs`; PR explicitly says "The diagnostics golden fixture remains deferred as described in #355 until #360".
**Verdict:** fully addressed — acceptance criteria (failing/ignored test + repro) met; the fixture deferral was sanctioned by the issue text itself.
**Deferred/follow-up:** diagnostics golden fixture deferred to #360 (PR #366 body).

## #356 — Design: general rule-synthesis over links notation (reason, don't memorise) (closed 2026-05-30)
**Requirements (konard):**
- "A design doc (`docs/design/rule-synthesis.md`) specifying" decomposition, rule construction from vocabulary, TDD verification, interaction with #357/#358.
- "it explicitly states what is kept ... vs. replaced (the allowlist)."
**Delivered (evidence):**
- Closing PR #367 "Design: rule synthesis over Links Notation": adds the doc covering decomposition, candidate construction, TDD verification, diagnostics, persistence; explicitly documents kept vs. replaced; traceability test added.
**Verdict:** fully addressed — all acceptance criteria of the design issue are claimed point-by-point in the PR body.
**Deferred/follow-up:** none noted.

## #357 — Coreference: bind bare-imperative follow-ups to the active program artifact (closed 2026-05-30)
**Requirements (konard):**
- "when the active conversation has a program artifact and the current message is a bare imperative, binds the referent and re-enters `write_program` recovery."
- "Extend `data/seed/coreference.lino` ... works in all four languages."
**Delivered (evidence):**
- Closing PR #369 "Coreference: bind bare imperative program follow-ups": detection + re-entry into recovery, worker mirror, en/ru/hi/zh seed extension, 4-language regression tests + negative case. PR body: "The actual reverse-sort code transformation remains scoped to the later modification work tracked separately" (that was #358's scope anyway).
**Verdict:** fully addressed — the routing/binding acceptance criteria are met; the transform itself belonged to #358.
**Deferred/follow-up:** reverse-sort transformation deferred to #358 by design (PR body).

## #358 — General program-modification model (remove the hard-coded `PROGRAM_MODIFIERS` allowlist) (closed 2026-05-30)
**Requirements (konard):**
- "Replace the hard-coded `PROGRAM_MODIFIERS` slice with a data-driven, composable modifier model"
- "≥2 modifiers compose; ... covered by tests in the four languages." Follow-up comment pointed to the #356 design doc contracts.
**Delivered (evidence):**
- Closing PR #370 "Generalize program modifiers for reverse sort": allowlist removed, modifiers discovered from seed rules/vocabulary, composed path_argument+reverse_sort task, worker mirror, 4-language tests; the formerly-ignored #355 test now passes.
**Verdict:** fully addressed — each acceptance criterion has matching evidence in the PR body.
**Deferred/follow-up:** none noted.

## #359 — Reasoned rule construction on unknown (construct a candidate rule; ≤1 clarifying question) (closed 2026-05-30)
**Requirements (konard):**
- "before emitting an unknown answer, attempt rule construction per the #356 design ... verify it with a TDD test"
- "If ambiguous, ask **at most one** clarifying question instead of `unknown`."
- "Never emit a bare 'unknown' for a resolvable modification."
**Delivered (evidence):**
- Closing PR #371 "Reason unknown program follow-ups into constructed rules": routes unknown bare follow-ups into rule construction, verification gate before `WriteProgram`, inspectable traces, 4-language regressions.
**Verdict:** partially addressed — construction + verification gate delivered, but the "≤1 clarifying question on ambiguity" behaviour is nowhere claimed in the PR body or tests.
**Deferred/follow-up:** none noted (the clarifying-question ask silently absent).

## #360 — Diagnostics / verbose mode for every reasoning step (default off) (closed 2026-05-30)
**Requirements (konard):**
- "Extend `diagnostic_mode` to emit each step as inspectable links-notation trace; keep default `false`."
- "Surface in the web UI's existing Diagnostics toggle"; "Add a golden diagnostics fixture for the #349 dialog".
**Delivered (evidence):**
- Closing PR #372 "Surface write-program diagnostics chain": trace appended under `diagnostic_mode`, worker mirror shows route attempts/coreference/modifier/construction/verification in the Diagnostics toggle, integration test `issue_349_diagnostic_mode_emits_full_turn_5_reasoning_chain` + Playwright regression.
**Verdict:** fully addressed — the integration test asserting the full turn-5 chain effectively serves as the golden fixture; default-off preserved.
**Deferred/follow-up:** none noted.

## #361 — Cross-runtime parity (Rust core + JS worker + wasm worker + seed) (closed 2026-05-30)
**Requirements (konard):**
- "reconcile every unknown-exit; ensure the Rust core, the JS worker, the wasm worker, and the seed produce identical answers for the #349 dialog."
- "The reverse-sort fix is observable in all runtimes; parity checks pass".
**Delivered (evidence):**
- Closing PR #373 "Cross-runtime parity for reverse-sort follow-up": adds one experiment harness (`experiments/issue-361-cross-runtime-parity.mjs`) that "executes the browser worker's pure-JS fallback mirror" and re-runs the Rust integration test.
**Verdict:** partially addressed — only the pure-JS fallback mirror is exercised; the wasm worker and the six named unknown-exit reconciliation sites are not evidenced in the PR body, and existing parity checks are not shown to have been run.
**Deferred/follow-up:** none noted (wasm-worker verification simply absent).

## #362 — Bulk multilingual multi-turn coding-modification benchmark + ratchet (closed 2026-05-30)
**Requirements (konard):**
- "Build a multi-turn coding-modification benchmark ... across en/ru/hi/zh."
- "Integrate open datasets **download-on-test** (CanItEdit, HumanEvalFix, EDIT-Bench) ... verify licenses"
- "A CI-runnable (possibly nightly) bulk suite exists ... a ratchet guards regressions."
**Delivered (evidence):**
- Closing PR #374 "Add coding-modification benchmark ratchet": manifest with en/ru/hi/zh local ratchet cases incl. the #349 dialog; dataset provenance/license/download metadata recorded; an *ignored* network download test gated behind `FORMAL_AI_BULK_BENCHMARK=1`.
**Verdict:** partially addressed — local ratchet + metadata exist, but the external datasets are only touched by an opt-in ignored test; no nightly/CI wiring for the bulk suite is claimed, so the "bulk test suites driven by large open datasets" ask is thin.
**Deferred/follow-up:** external parquet payloads fetched only under `FORMAL_AI_BULK_BENCHMARK=1` (PR Notes section); no scheduled CI run mentioned.

## #363 — Reduce "Report issue" pressure — reasoning-first, report as last resort (closed 2026-05-30)
**Requirements (konard):**
- "For resolvable modifications, 'Report issue' is never the primary response; when it does appear, it carries the reasoning trace; covered by tests across runtimes."
- "Soften the multilingual fallback copy".
**Delivered (evidence):**
- Closing PR #375 "fix(#363): reduce report issue pressure": report action only on unresolved unknown turns; report URLs embed compact reasoning trace; softened copy across seed/Rust/worker; e2e + 4-language checks.
**Verdict:** fully addressed — all three acceptance criteria have direct evidence.
**Deferred/follow-up:** none noted.

## #364 — Self-improvement: learn rules from accumulated unknown-traces (white-box) (closed 2026-05-31)
**Requirements (konard):**
- "Accumulate unknown-traces ...; periodically attempt to synthesise new seed rules ...; gate every learned rule behind the #362 benchmark ...; keep every learned rule inspectable".
- "A documented, gated loop that proposes learned rules, each verified against the benchmark before adoption."
**Delivered (evidence):**
- Closing PR #376 "Add white-box self-improvement loop for unknown traces": public `self_improvement` API accumulating traces, proposing learned rules, benchmark-gated adoption state; policy recorded in seed + design docs; spec coverage.
**Verdict:** partially addressed — the loop exists as a proposal-only API with a gate, but nothing "periodically" runs it; it is a library capability, not an operating loop.
**Deferred/follow-up:** loop is "proposal-only by default" (PR body); actual periodic/automatic execution not delivered or scheduled.

## #365 — Epic / tracking issue for #349 (closed 2026-05-31)
**Requirements (konard):**
- "Closes when #355–#365 are done and the #349 dialog is green across all runtimes and in the bulk suite."
**Delivered (evidence):**
- Closing PR #377 "docs(#365): close issue #349 reverse-sort roadmap": closure report mapping #355–#365 to PRs, refreshed ROADMAP/REQUIREMENTS/VISION/ARCHITECTURE; reran the roadmap verification commands.
**Verdict:** fully addressed as a tracking issue — though its "green across all runtimes" condition inherits the #361 wasm-runtime evidence gap.
**Deferred/follow-up:** none noted.

## #368 — We need dedicated instructions for agentic AI tools in README (closed 2026-05-31)
**Requirements (konard):**
- "instructions on how to start local server from crates/cargo, and connect Claude Code, Codex, OpenCode."
- "Make sure server is feature reach, covered with all the same tests as web app."
- "We should have real integration tests with claude/codex/opencode commands, and with ... Agent CLI."
- "Instructions should be user friendly and 100% verified that are working." "Nothing should be defered or delayed."
**Delivered (evidence):**
- Closing PR #378 "Document agentic CLI server setup": README section for Codex/Claude Code/OpenCode/Agent, corrected wire-API docs, docs traceability test, and a "live loopback integration test" hitting `/v1/models`, `/v1/responses`, `/v1/chat/completions`, `/v1/messages` with bearer auth.
**Verdict:** partially addressed — docs plus protocol-level loopback tests delivered, but no integration test actually invokes the `claude`/`codex`/`opencode` CLI commands, and "server covered with all the same tests as web app" is not evidenced.
**Deferred/follow-up:** none noted (real-CLI integration tests silently replaced by protocol loopback tests).

## #386 — Unknown prompt: Отмени сортировку (closed 2026-06-04)
**Requirements (konard):**
- "In issue reporting we should do not show settings, that are set exactly to default"; worker folded into version; shorter memory-attach text; trace dropped when dialog trimmed.
- "in settings UI we should be able to reset each setting to default, as well as all of them."
- "copy full dialog as markdown, when diagnostics mode is enabled, reasoning steps should also be converted".
- "we need to deeply rethink our vision and architecture. First we should collect all previous issues, comments... fully list all the requirements in our docs... evaluate how fully each of them implemented."
- "use best experience from ... meta-expression ... latest version of it for requests for translation."
- "seed data fully self described... view our links memory through multiple virtual views like meanings, words, symbols, letters, nouns, verbs..."
- "CST/AST of programming languages, the tree-sitter ... should be added as dependency."
- "rooted in real data from wikipedia/wikidata/wiktionary"; "All API requests we do must be absolutely preserved... count how many times we did access each link".
- "Nothing should be hardcoded in the code... plan and execute everything in this single pull request."
**Delivered (evidence):**
- Closing PR #387 "Make seed data self-describing across the codebase; fix 'Отмени сортировку' via inverse derivation": cancel-sort fixed via seed-declared inverses (no hardcode), whole-codebase concept-based recognizers, Wikidata/Wiktionary caches, report trimming (all 4 asks), reset settings, copy-as-markdown with diagnostics folding, case study, large test matrix.
- PR admits: "The broader architecture vision (R-h, self-describing seed data) is mapped to the existing vision track for the parts beyond this operation."
**Verdict:** partially addressed — the bug, the concrete report/UI asks, and a substantial self-describing-seed sweep are delivered; but tree-sitter/CST support, the multiple virtual views (words/symbols/letters/POS phrases), API-request preservation with access counting/eviction, meta-expression integration, and the collect-all-previous-requirements audit are not claimed anywhere in the PR body despite the "defer nothing" instruction.
**Deferred/follow-up:** broader architecture vision mapped "to the existing vision track" (PR body); tree-sitter arrived only later via #395/PR #396; views/API-cache-accounting have no noted follow-up.

## #388 — UI/UX fixes and improvements (closed 2026-06-04)
**Requirements (konard):**
- "Buttons on top are clearly not fit, so we should change to icon only buttons sooner (...use their actual size... modern css and adaptive layout)."
- "Some UI elements do not support dark theme, we need to recheck every UI element, and guarantee they support dark theme."
- "For all these we should add real e2e tests." Plus the standard case-study ask.
**Delivered (evidence):**
- Closing PR #389 "Fix topbar fit and dark-theme parity": container-query label collapse (measured width), dark-theme parity for diagnostics/copy/reset surfaces, issue-388 Playwright coverage, case study with before/after screenshots.
**Verdict:** partially addressed — topbar fit and targeted dark-theme fixes with e2e landed, but "recheck every UI element and guarantee dark theme" was only done for three surface groups, and true icon-only buttons came later (PR #422 for #409).
**Deferred/follow-up:** none noted in PR #389; icon-only toolbar effectively deferred until issue #409.

## #390 — GitHub Releases are broken for a week (closed 2026-06-04)
**Requirements (konard):**
- "No new GitHub releases recently, and looks like CI/CD shows false positive at the same time."
- "Use all the best practices from CI/CD templates ... if the same issue is found in template report issue also in templates" (4 template repos listed); case study ask.
**Delivered (evidence):**
- Closing PR #391 "Fix GitHub release automation recovery": four root causes fixed (validation-error parsing, 120KB body cap, workflow_run tag resolution, electron-builder config); regression tests; case study; four upstream template issues filed (rust/js/python/csharp) with links.
**Verdict:** fully addressed — root cause, fix, tests, case study and the requested upstream template reports are all evidenced.
**Deferred/follow-up:** none noted.

## #392 — Copy as markdown for conversations does not work (closed 2026-06-04)
**Requirements (konard):**
- "When button clicked nothing happens. That must be fixed."
- "for all copy actions in the application we should have e2e tests to guarantee these features will never break." Plus case-study ask.
**Delivered (evidence):**
- Closing PR #393 "Fix conversation Markdown copy": root cause (clipboard write after await lost user activation), fix, activation-sensitive e2e spec covering code-block/message/conversation copy in all 4 UI languages, case study archived.
**Verdict:** fully addressed — fix + e2e for every copy action + case study; upstream reporting reasonably waived ("local app timing, not a dependency defect").
**Deferred/follow-up:** none noted.

## #394 — If we don't have links rules, we should use the same language of last message by default (closed 2026-06-04)
**Requirements (konard):**
- "user will be able to configure the system using prompts in Russian, if he talked last time in Russian" (no en/ru jumping in fallback guidance).
- "the text of description how to configure local copy of formal AI should be much more user friendly, for example it may contain examples and so on, and may be more explanations."
- "it is about `links rules`, not `links notation` rules."
**Delivered (evidence):**
- Closing PR #397 "Keep Russian unknown links-rule guidance localized": Russian guidance stays Russian; terminology switched to "local links rules"; response bodies moved to seed; regression coverage; case study.
**Verdict:** partially addressed — language consistency and terminology fixed, but the "much more user friendly, with examples and more explanations" rewrite of the configuration guidance is not claimed anywhere in the PR body.
**Deferred/follow-up:** none noted (user-friendliness rewrite silently absent).

## #395 — Unknown prompt: У меня есть числа 3, 5, 6, 7, 8 отсортируй их в JavaScript... (closed 2026-06-10)
**Requirements (konard):**
- Fix the unknown for the sorting prompt with code + result; "Our system still supports very narrow range of tasks. And we need to continue PR #387."
- "Programming tasks should clearly use CST system like in tree-sitter (or directly it)."
- "no memoization, only generalization of algorithms... instead of hardcoding we should use data seed."
- "All meanings and other data links, should be rooted in the wikipedia/wikidata/wiktionary and other sources like word net."
- "All changes should use by default transactions from github.com/link-foundation/link-cli, so we never delete any data ... full history of data changes, so we can time travel."
- Meanings described via type system ("Type -> SubType ... nested types" in doublets).
**Delivered (evidence):**
- Closing PR #396 "Validate generated programs with link-foundation/meta-language for issue #395": seed-driven codegen (`coding-idioms.lino`), meta-language (tree-sitter-based) CST validation for 10 languages, 170-cell Rust↔worker parity, in-browser execution example, upstream issues filed and 3 resolved; one remaining upstream gap (#64 unparse) reported.
**Verdict:** partially addressed — the reported bug, CST integration, seed-driven generality and multilingual delivery are strongly evidenced; but link-cli transactions/time-travel memory, the doublets nested-type meaning system, and wordnet/wikipedia rooting of all meanings are not claimed in the PR body.
**Deferred/follow-up:** meta-language#64 (render code from a constructed syntax network) explicitly deferred upstream — "With #64, generated code becomes valid by construction" (PR body).

## #398 — Full recursive definition of all meanings (deeper meta-understanding), better semantic meta language as foundation (closed 2026-06-10)
**Requirements (konard):**
- "describe each meaning with meanings, that are described by meanings."
- "we don't have any left overs for hardcoded constants and values about any language ... manipulate only meanings in the code."
- "Each and every data item we have in links, should be routed in actual external sources, with references to them."
- Views for "meanings, words, types, categories, concepts, groups"; comparison with "wikipedia, wikidata, wiktionary, wordnet, other thesauruses"; pre-cached original responses; "Please plan and execute everything in this single pull request."
**Delivered (evidence):**
- Closing PR #399 "Validate canonical LiNo data and grounding caches": canonical LiNo projection, typed Wikidata/Wiktionary caches, readable quoted scalars, pipe-list migration, total token closure (0 unresolved / 1,410 tokens), WordNet OEWN 2024 ingestion (312 lemmas), `data/view/` multi-source merge layer, sources registry, overrides layer, many CI guards.
- PR body has explicit "Deferred to follow-up issues" sections: hyphenated-word slugs (Defect 3/rule 4), no hardcoded domain-data string literals in `src/` (rule 6), full grounding of every meaning (R282 "partial", "a corpus-import effort larger than one PR"), POS-as-meaning/reverse-dictionary, provable 1:1 meaning↔type via relative-meta-logic, minimal seed + on-demand expansion, long-lived-quote sweep.
**Verdict:** partially addressed — very large delivery, but multiple core asks (ground every meaning in external sources, eliminate all hardcoded language literals in src, meaning↔type merge) are explicitly deferred despite the single-PR/defer-nothing instruction.
**Deferred/follow-up:** slug rename, rule-6 literal sweep, grounding floor for all meanings, POS-as-meaning, relative-meta-logic 1:1 typing, minimal-seed/on-demand expansion — all tracked only in `docs/case-studies/issue-398/README.md` (PR body).

## #400 — Unknown prompt: последние новости (closed 2026-06-08)
**Requirements (konard):**
- Issue authored by xierongchuan (bug report: "последние новости" → unknown). konard comment (2026-06-08): "We should get news from https://www.wikinews.org".
**Delivered (evidence):**
- Closing PR #401 "Handle latest-news prompts with Wikinews": latest-news prompts route to `web_search` with Wikinews provider in all 4 languages; CORS verified against the live Wikinews API; worker + wasm rebuilt.
**Verdict:** fully addressed — konard's single requirement (Wikinews as the source) is directly implemented and tested.
**Deferred/follow-up:** none noted.

## #402 — Unknown prompt: Что делаешь в свободное время? (closed 2026-06-11)
**Requirements (konard):**
- Issue authored by rumaster. konard comment (2026-06-09): "formal AI at the moment stops existing when user is not using it. We can provide multiple random realistic answers to that."
**Delivered (evidence):**
- Closing PR #421 "Answer assistant free-time small talk": `assistant_free_time` seed route with localized en/ru/hi/zh responses; the answer explains the assistant "is idle between prompts and helps during active dialog"; Rust + Playwright regressions.
**Verdict:** partially addressed — the unknown is fixed with the truthful "stops existing" framing konard wanted, but "multiple random realistic answers" (answer variety) is not claimed; the PR describes a single localized answer per language.
**Deferred/follow-up:** none noted (random-variant answers silently absent).

## #403 — Unknown prompt: Я загадал число больше 1 но меньше 3. что это за число? (closed 2026-06-11)
**Requirements (konard):**
- Issue authored by rumaster. konard comment (2026-06-09): "That is pure reasoning task, that should be possible to be solved as formal proof converted to format supported by github.com/link-foundation/relative-meta-logic so we not only translate, we also check and execute it. We should also support translation to other programming languages."
**Delivered (evidence):**
- Closing PR #420 "Answer interval number riddles with formal reasoning": `number_constraint_reasoning` handler, bounds parsed, integer answer computed, formalized as `x > 1 and x < 3 is satisfiable` and verified through the relative-meta-logic / SMT proof engine with trace.
**Verdict:** partially addressed — the relative-meta-logic proof requirement is delivered and verifiable, but "translation to other programming languages" of the proof/task is not mentioned in the PR body.
**Deferred/follow-up:** none noted (multi-language translation of the proof silently absent).

## #404 — Unknown prompt: Забей мне 18 число в 17:00 по грузии на встречу с Леваном (closed 2026-06-13)
**Requirements (konard):**
- Issue authored by skulidropek (calendar-assistant scenario). konard comment (2026-06-09): "We can do it using export of calendar event file in all supported by calendar formats, also we can use login in the browser, to actually access calendars, if it is possible... We can also ask API token... Brainstorm best possible solutions in all environments we support. And support all the languages."
**Delivered (evidence):**
- Closing PR #405 "feat: natural language calendar event creation (#404)": `calendar_create_event` intent; RFC 5545 `.ics` export + Google Calendar render-template URL ("preferring the simplest available for the user at each situation, per @konard's brainstorm note"); en/ru/hi/zh; Rust↔WASM parity; false-positive hardening.
**Verdict:** partially addressed — event creation works in all languages via two login-free artifacts, but "all supported by calendar formats" is reduced to .ics+URL, and the browser-login and API-token access paths konard raised are not implemented or explicitly ruled out in the PR body.
**Deferred/follow-up:** none noted (OAuth/API-token calendar access absent without a stated follow-up).

## #406 — Unknown prompt: ?+2=4 (closed 2026-06-12)
**Requirements (konard):**
- Issue authored by skulidropek. konard comment (2026-06-11): "Looks like that should be reported in calculator, we should report issues there, and add failing tests here, once issues in link calculator will be resolved... give it at least 50 examples of different types of equations, so we support them as much as possible. And if there is single unknown variable, we can also use ? and * instead of x variable."
**Delivered (evidence):**
- Closing PR #418 "Support upstream calculator equation solving": upstream work landed as link-assistant/calculator#175, consumed via `link-calculator` 0.18.2; `?`/`*` placeholders, multi-variable linear, and polynomial roots covered with repository tests; worker fallback aligned.
**Verdict:** partially addressed — upstream routing, `?`/`*` support and expanded categories are delivered; but the "at least 50 examples of different types of equations" volume is not evidenced in the PR body (only a handful of categories/examples listed).
**Deferred/follow-up:** none noted; whether calculator#175 received the 50-example matrix is not verifiable from this thread.

## #407 — Unknown prompt: хочу понять сколько будет 2+2 (closed 2026-06-12)
**Requirements (konard):**
- Issue authored by yukakust. konard comment (2026-06-11): "We should make x10 variations support for all kinds of ways to ask about making calculations. If message in any statement contains calculation request we should always support it. Ideally combining with other instructions. So we should generalize much more."
**Delivered (evidence):**
- Closing PR #417 "Recognize embedded calculation requests": cues recognized anywhere in the prompt, worker mirror, "10-case Rust embedded calculation variation matrix" plus 4-language Playwright regression.
**Verdict:** partially addressed — embedded detection and a 10-variation matrix match the "x10 variations" ask literally, but "combining with other instructions" (calculation mixed with other requests in one message) is not claimed.
**Deferred/follow-up:** none noted.

## #408 — Хочу что бы он умел заменять информацию в тексте (closed 2026-06-12)
**Requirements (konard):**
- Issue authored by skulidropek (replace "Hello World" with "Bye world" in prior answer). konard comment (2026-06-11): "We need to support not only that, but wide range of possible variations and action about text editing in general and code editing in particular. So everything that supported by todays IDEs should be fully possible to do only using the chat."
**Delivered (evidence):**
- Closing PR #416 "Generalize issue #408 text/code edit benchmarks": contextual replacement on the prior assistant artifact; ~30 deterministic text/code operations (case conversions, extraction, counting, line ops, comment/indent); 48-source x 30-variation local benchmark ratchet (1,440/1,440); en/ru/hi/zh vocabulary; PR states "No issue #408 benchmark work is left for a separate pull request."
**Verdict:** partially addressed — the reported bug and a genuinely wide operation range are delivered with a strong ratchet, but "everything supported by today's IDEs via chat" (multi-file edits, refactoring/rename, etc.) is far beyond the delivered single-artifact string/line operations.
**Deferred/follow-up:** PR explicitly scopes the benchmark as "repository-local executable edit benchmark profile, not an external leaderboard publication"; full IDE-parity has no tracked follow-up.

## #409 — Change emoji to names or icons (closed 2026-06-12)
**Requirements (konard):**
- Issue authored by kogeletey ("the emojis not correct"). konard comment (2026-06-11): "We should support multiple popular icons like FontAwesome by default, and other top 5 most popular and widely used fonts for icons should be switchable in settings."
**Delivered (evidence):**
- Closing PR #422 "Replace toolbar emoji with icon packs": emoji replaced with self-contained icon rendering; persisted `Toolbar icons` setting with Font Awesome default plus Material Symbols, Bootstrap Icons, Ionicons, Remix Icon, Tabler Icons, and a Names option; localized labels, e2e, screenshot.
**Verdict:** fully addressed — FontAwesome default + six switchable alternatives meets the "top 5 switchable" ask exactly.
**Deferred/follow-up:** none noted.

## #410 — Make sure link-assistant/web-search have all supported features, and use it as component for web search (closed 2026-06-15)
**Requirements (konard):**
- "use it as component for web search"; "make sure that we use github.com/link-assistant/web-capture for web fetch / web capture."
- "Both ... should support all features required by as s libraries, CLIs, micro-services... If these repositories have any missing features, we should report issues to them, and we can continue development once all these issues are resolved." Plus case-study ask.
**Delivered (evidence):**
- Closing PR #414 "chore(#410): use latest web-search/web-capture, stable Rust 1.96, and refresh all dependencies": upstream blockers (web-search#5, #6; web-capture#135) filed and now closed; MSRV/toolchain aligned; full dependency refresh; React 19 regression fix; refreshed case study which "recommends the FormalAI-side configuration-gated adapter + provider-parity tests as the next, separately-tested step before any production default change."
**Verdict:** partially addressed — upstream readiness and issue-reporting were done, but the headline requirement (actually using web-search/web-capture as the components in formal-ai) is explicitly left as a recommended next step, not implemented.
**Deferred/follow-up:** FormalAI-side adapter + provider-parity tests deferred as "the next, separately-tested step" (PR body / case study).

## #411 — Unknown prompt: Покажи правила (closed 2026-06-12)
**Requirements (konard):**
- "Покажи правила" must not be `unknown` (also "Отсортируй 4, 3, 1, 17, 8, 9, 15" shown failing in the same dialog).
- "We need configuration of our AI system much more user friendly, so everything can be done fully at chat, with high quantity of actual variations supported, that all should be formalized correctly in meta language, and be reasoned about." Plus case-study ask.
**Delivered (evidence):**
- Closing PR #415 "Fix short behavior-rule list prompts": short rule-list forms recognized in en/ru/hi/zh (`Show rules` / `Покажи правила` / `नियम दिखाओ` / `显示规则`), seed-backed patterns, browser mirror, repro example, e2e coverage, case study.
**Verdict:** partially addressed — the specific reported prompt is fixed in 4 languages, but the broader "high quantity of variations, fully chat-driven configuration, formalized and reasoned" ask is not evidenced, and the second failing prompt in the dialog (sort a bare number list) is not mentioned in this PR.
**Deferred/follow-up:** none noted.


# Chunk ab analysis — issues 412..466

## #412 — Unknown prompt: Отсортируй 4, 3, 1, 17, 8, 9, 15 (closed 2026-06-11)
**Requirements (konard):**
- "We need to focus on increasing our generalization in the code on this and 10 more similar tasks ... our code should contain universal problem algorithm that discovers required data to actually solve the task using external knowledge."
- "we should incorporate https://www.wikifunctions.org and rosettacode.org and ... http://helloworldcollection.de, may be stackoverflow ... we should treat them as external APIs even if they don't natively support any APIs", with popular examples cached and merged into views.
- "we should never cache everything (not more than 1% or 512 (if 1% is less than 512) items per data set / API / merged data topic/category)."
- "we should prefer to have algorithm builder, not just building by template, but meta algorithm, building algorithm that builds algorithms, to solve exactly all tasks."
- "make sure we actually use https://github.com/link-foundation/meta-language for all coding manipulation tasks"; deep case study in `./docs/case-studies/issue-412`; report upstream issues; everything in this single PR.
**Delivered (evidence):**
- Closing PR #413 "fix(numeric_list): recover coding context for bare follow-ups (#412)". Fixed the bare follow-up via history-aware language/code-request inheritance, JS worker parity, case study with R1–R12 breakdown. R6: the four knowledge sources modeled with an *embedded popular-case cache*; live refresh only gated ("A gated live-refresh ... is what would materialise per-source `data/cache/<slug>/` buckets"). R8: bounded cache min(1%, 512) with CI ratchet. R7: PR body admits only the "compositional foundation ships"; "Unifying the remaining bespoke coding handlers under one task-agnostic composer is a larger first-principles refactor; its status and the scoping question are tracked in ... §7.4 and the PR thread."
**Verdict:** partially addressed — bug fix plus cache-cap/knowledge scaffolding shipped, but the central meta-algorithm (unifying all coding handlers) is explicitly left open, and external sources are embedded snapshots rather than real API integration.
**Deferred/follow-up:** task-agnostic composer unification (PR body R7 + case study §7.4); live per-source cache materialization behind `FORMAL_AI_LIVE_API`.

## #423 — Support conversion between README install guides and sh/powershell scripts (closed 2026-06-13)
**Requirements (konard):**
- "we should be able to convert any of such script to documentation installation/deploy guide" and back for "any GitHub repository".
- "We should have at least 50 test cases for most popular GitHub project"; "Even better to take top 50 most popular GitHub projects that have both installation scripts and manual installation guides".
- "And we also need a meta algorithm, the algorithm that constructs such algorithm by this specific task."; "Nothing should be delayed or deferred into separate pull requests"; case-study folder.
**Delivered (evidence):**
- Closing PR #424 "Support README install guide script conversion": deterministic Markdown↔sh↔PowerShell conversion in Rust solver + browser worker; explicit meta-algorithm trace (corpus → ontology → IR → recognizers/renderers/validators → fixtures → mirrors); "Doubled the popular GitHub project regression matrix from 50 to 100 cases"; refreshed case-study snapshot.
**Verdict:** fully addressed — both directions, 100 > 50 repo cases, meta-algorithm trace delivered; residual generalization gaps were explicitly spun out by konard into #433 rather than dropped silently.
**Deferred/follow-up:** recognizer generalization (prefix whitelists, describe_command table, handler dispatch) deliberately split into follow-up issue #433.

## #425 — We need support more questions and tasks (closed 2026-06-14)
**Requirements (konard):**
- Example prompt to support: "Сделай мне пдф файл со списком стран, где есть пособия/скидки на еду для малоимущих, как в виде прямых денежных дотаций, так и в косвенной форме, например, талоны." (make me a PDF with a researched list of countries offering food assistance).
**Delivered (evidence):**
- Closing PR #432 "Support document generation and meta-language format conversion (#425)": recognizes document-generation requests as formal plans; meta-language 0.45.0; `document_formats` boundary (TXT/Markdown/HTML/PDF/DOCX profiles); conversion via `LinkNetwork::reconstruct_text_as`. PR repro examples: "Create a PDF document from this text" and Markdown→HTML conversion.
**Verdict:** partially addressed — PDF/document *format* plumbing shipped, but no evidence the actual asked capability (researching and producing the country-list content) was implemented; the PR only demonstrates format handling of user-supplied text.
**Deferred/follow-up:** none noted in the PR body (the knowledge-gathering half of the example prompt is silently absent).

## #426 — Unknown prompt: Financical records for boeing after crysis with icas system (closed 2026-06-25)
**Requirements (author: kogeletey, not konard; no konard comments):**
- Bug report: the records/financials prompt fell through to "intent: unknown" instead of being answered.
**Delivered (evidence):**
- Closing PR #431 "Route records/financials prompts to web search (#426)": new `records_information_request` recognizer with en/ru/hi/zh lexicon, routing gate, JS worker parity, unit + Playwright + negative coverage; after: routes to `intent: web_search`.
**Verdict:** fully addressed — the unknown-intent gap closed with a generalized multilingual lexicon-driven recognizer (as scoped; the answer is a web-search route, not financial data).
**Deferred/follow-up:** none noted.

## #427 — Unknown prompt: Сделай инверсию сортировки. (closed 2026-06-13)
**Requirements (konard):**
- Bug report: after a numeric-list sorting conversation, the follow-up "Сделай инверсию сортировки." returned "(intent: unknown, reported)".
**Delivered (evidence):**
- Closing PR #430 "fix: invert-sort follow-up is no longer unknown (#427)": invert phrasings added to `reverse_sort` vocabulary for en/ru/hi/zh ("no per-prompt hardcoding"); split inheritance (list from last list-bearing turn, language from last language-bearing turn); byte-for-byte worker mirror; 5 reproducing tests + parity harness; no-context case does not fabricate.
**Verdict:** fully addressed — root cause (vocabulary gap + missing list inheritance) fixed multilingually and generally, with negative-case coverage.
**Deferred/follow-up:** none noted.

## #428 — Update to use latest meta-language all around the code base (closed 2026-06-12)
**Requirements (konard):**
- "We should use latest version of ... meta-language for: formalization from natural languages or source code text ... manupulate the links network of it for reasoning, and all other tasks ... translate our meta language to any target language - natural or formal."
- "If any features are missing ... we should report issues to ... meta-language repository."; case-study folder; everything fully done in one PR.
**Delivered (evidence):**
- Closing PR #429 "Update meta-language to 0.40.0": dependency bump 0.39→0.40 + Cargo.lock, case study and research notes. Scope notes explicitly narrow the work: "This PR does not advertise every new upstream grammar as a public Formal AI language; adding public language support still needs catalog aliases, idioms, examples, execution/oracle policy, source tests, and worker parity." Case study documents "follow-up work for deeper link-network adoption." "No upstream blocker was found, so no new upstream issue was filed."
**Verdict:** partially addressed — only the version bump and documentation landed; the actual ask (meta-language used for formalization/reasoning/translation "all around the code base") is deferred as documented follow-up work, contradicting the "nothing deferred" instruction.
**Deferred/follow-up:** "follow-up work for deeper link-network adoption" (PR summary + case study); public language support for new upstream grammars (PR scope notes).

## #433 — Generalize solver recognizers from memoized lookup tables to compositional rules (closed 2026-06-13)
**Requirements (konard):**
- "A written audit listing each handler that relies on a fixed enumeration vs. compositional rules."
- "At least the installation-conversion command recognizer generalized away from the prefix whitelist with no regression in the 100-repo corpus and added false-positive (prose) coverage."
- "A documented plan (or prototype) showing one existing coding handler reconstructed from the meta-algorithm rule primitives."
**Delivered (evidence):**
- Closing PR #434 (same title): AC1 — case-study audit classifying ~46 recognizers (29 compositional, 4 hybrid, ~10 fixed-enumeration candidates, worst offenders ranked). AC2 — `looks_like_command` rewritten to shape+provenance reasoning; `describe_command` generalized to verb/object inference; corpus stays green + prose false-positive tests + worker parity. AC3 — `numeric_list` handler reconstructed from the seven meta-algorithm primitives with a migration recipe for the `algorithm` handler.
**Verdict:** fully addressed — all three acceptance criteria demonstrably delivered.
**Deferred/follow-up:** ~10 fixed-enumeration handlers identified in the audit (e.g. `opinion_question`, `number_riddle`, `algorithm`, `source_refresh`) remain un-generalized — documented as candidates, no follow-up issue cited.

## #435 — Unknown prompt: Можешь поставить мне созвон в кальндарь на завтра? (closed 2026-06-13)
**Requirements (author: skulidropek, not konard; no konard comments):**
- Bug report: a calendar-event request with a relative date ("на завтра") returned "(intent: unknown, reported)".
**Delivered (evidence):**
- Closing PR #419 "Fix multilingual calendar event requests" (also fixes #404): routes to a `calendar_create_event` draft; relative-date resolution (завтра/tomorrow/कल/明天 etc.); `.ics` VEVENT + Google Calendar render URL with confirmation, worker parity, negative case, multilingual tests, Playwright e2e.
**Verdict:** fully addressed — the exact prompt now yields a dated draft; lexicon-driven design covers ru/en/zh with negative-case protection.
**Deferred/follow-up:** none noted.

## #436 — Unknown prompt: theory of theory (closed 2026-06-13)
**Requirements (author: nivedano, not konard; no konard comments):**
- Bug report: "theory of theory" returned "(intent: unknown, reported)".
**Delivered (evidence):**
- Closing PR #437 "Seed metatheory concept for 'theory of theory' prompt": seeded `concept_metatheory` grounded in Wikipedia with en/ru/hi/zh aliases and summaries; regression tests across all four languages; total-closure gate clean.
**Verdict:** fully addressed for the reported prompt — though the fix is a per-concept seed (memoization), the very pattern konard's other issues (#412/#433) push against; no generalization to unseeded "X of X" concepts is claimed.
**Deferred/follow-up:** none noted.

## #438 — Fully prepared docker image with Telegram bot, single-line start (closed 2026-06-17)
**Requirements (konard):**
- "We need fully prepared docker image with Telegram bot, so we can start the system with minimum configuration ideally using single line in sh."
- "improve our docs and codebase, to simplify spinning up Telegram Bot ... as much as possible."; case study folder; everything in one PR.
**Delivered (evidence):**
- Closing PR #470 "Prepare Telegram Docker image + one-click/one-line Telegram bot and OpenAI server": GHCR image published on release; root `compose.yaml` for `TELEGRAM_BOT_TOKEN=... docker compose up` and one-line `docker run`; desktop one-click Services panel; server compose profile; docs (`docs/desktop/service-control.md`, README/ARCHITECTURE/REQUIREMENTS R327–R329); case study under `docs/case-studies/issue-438/`.
**Verdict:** fully addressed — one-line startup delivered, plus extra one-click desktop scope from a maintainer follow-up handled in the same PR.
**Deferred/follow-up:** none noted.

## #439 — Unknown prompt: Сколько метров в килограмме? (closed 2026-06-24)
**Requirements (author: ideav; konard comment 2026-06-13, before the closing PR merged):**
- "we should reason about each meaning in the sentence, step by step using our meta language, and understand that measure of length and mass are incompatible ... And we need to explain why."
- "We need to find benchmarks and tests for similar questions, and make sure our system correctly support all possible variations of them, and by generalizing the solution, not by memoizing."
- "we need to improve our meta algorithm ... we should try to connect our Formal AI to our https://github.com/link-assistant/agent as a model ... make sure that this flow is also documented for our users." Test via agent CLI `--model formal-ai`, compare with `claude -p` sonnet output, and "reconstruct all reasoning stepts that will lead to similar output".
**Delivered (evidence):**
- Closing PR #471 "Fix incompatible length/mass unit prompts": browser-worker parity handler for dimensionally incompatible units; Russian unit case-form seeds; Rust + Playwright regressions for the reported prompt "plus multilingual length/mass variations". After: `intent:unit_incompatibility` explaining meters=length vs kilograms=mass.
**Verdict:** partially addressed — the incompatibility explanation shipped, but konard's follow-up requirements (benchmark hunting for all variations, meta-algorithm improvement, agent CLI `--model formal-ai` integration + user docs, claude-sonnet output comparison) are not mentioned anywhere in the PR body.
**Deferred/follow-up:** agent-integration/benchmark/meta-algorithm asks from konard's comment appear dropped — no deferral note, no follow-up issue cited.

## #440 — List of files program output issues (closed 2026-06-14)
**Requirements (konard):**
- Fix wrong sample output ("main.py листает текущую папку, а значит ... main.py должен вывести и main.py"-style inconsistency with Cargo.toml/main.rs shown for a Python program).
- "on light theme, we use light theme also for code blocks in markdown."; "instructions for users are in separate paragraph (starting from words `Copy ...`)".
- "Find all similar problems in the codebase, make sure each our example is deeply tested"; "we need to make sure we pass all benchmarks we know of for coding tasks and so on."
- "we need have guarantees in CI/CD ... We should use our own Formal AI to construct a check in natural language for each rule, that our Formal AI will enforce as translation to code and execution."; case study; report upstream; everything in one PR.
**Delivered (evidence):**
- Closing PR #472 "Fix list-files sample output and light code blocks": language-aware sample output (`main.py`, `data.txt`, `README.md`); worker mirror + `Copy the snippet...` split into its own paragraph; light code surface in light theme; deep case study with before/after screenshots; standard test/lint battery.
**Verdict:** partially addressed — the three concrete UI/example bugs were fixed well, but "pass all benchmarks we know of for coding tasks", the codebase-wide sweep for all similar problems, and the Formal-AI-enforced natural-language CI rule checks are not claimed anywhere in the PR body.
**Deferred/follow-up:** benchmark coverage and self-enforcing natural-language CI checks — not delivered and not flagged as deferred.

## #441 — Unknown prompt: Что такое vulkan layer (closed 2026-06-24)
**Requirements (author: uselessgoddess, not konard; no konard comments):**
- Bug report: mixed-script definition prompt "Что такое vulkan layer" returned unknown (language misdetected as en).
**Delivered (evidence):**
- Closing PR #473 "Fix mixed-script Russian concept lookup": preserves non-Latin prompt language when Latin identifiers appear; term-first forms for hi/zh; concept prompt parser promoted into the intent formalization path; worker mirror + rebuilt WASM; regressions incl. mocked ru.wikipedia lookup.
**Verdict:** fully addressed — generalized mixed-script handling across supported languages, not a single-prompt patch.
**Deferred/follow-up:** none noted.

## #442 — Our CI/CD is broken, tests run on non-code changes (closed 2026-06-13)
**Requirements (konard):**
- "Such behaviour slows down the iteration, so it must be fixed."
- "Use all the best practices from CI/CD templates (check full file tree to compare for all GitHub workflow and CI/CD scripts file), if the same issue is found in template report issue also in templates" (4 template repos listed); "We should compare all files, so we don't have more CI/CD errors in the future and reuse all the best practices from these templates."; case study; everything in one PR.
**Delivered (evidence):**
- Closing PR #443 "fix(ci): stop running the test suite on non-code changes (#442)": root cause (test job gated on skipped changelog) fixed with detect-changes gating; regression tests pin the invariant; upstream issues filed in rust/csharp/js template repos (python was the correct reference); case study with timeline, requirements, root cause, prior-art survey.
**Verdict:** partially addressed — the bug, upstream reports, and case study are solid, but the PR body only evidences comparing the *test gating* across templates ("compared the `test` gating"), not the requested full file-tree comparison to import all remaining best practices.
**Deferred/follow-up:** full workflow-file-tree comparison / broader best-practice adoption from the four templates — not evidenced, not flagged.

## #444 — Unknown prompt: Can you give me specific instructions? (closed 2026-06-14)
**Requirements (konard):**
- "Sometimes simple search is not enough, we should do search on the topic, collect all the data, and answer on `how to` questions with a guide constructed from data we collected using logic reasoning."
- "We also need to support wikihow, stackoverflow and other similar resources"; "we can integrate also with wikibooks, wikiversity, wikivoyage"; "we also may crawl search results"; "we can try to access GitHub's readme's, software docs websites"; "each `how` question can be fully answered preferably using dynamic construction of an answer."
- "make sure we cache for at least 7 days status of accessibility of each of the services"; "all test cases for quality assurance should have their tests pre cached from actual services"; "Double check that we have contributing guide, that will help me not repeat myself all the time"; case study; everything in one PR.
**Delivered (evidence):**
- Closing PR #448 "Procedural how-to: elaboration-rebind fix + multi-source guide infrastructure (#444)": coreference/elaboration rebind fixed (Rust + worker parity); wikiHow/Stack Exchange/Wikibooks/Wikiversity/Wikivoyage/GitHub READMEs declared in `sources-registry.lino` with api endpoint, license, cache and settings toggles; ≥10 benchmark-style cases + central `docs/benchmarks.md` catalog; grounded meta-algorithm recipe; CONTRIBUTING "Project Conventions"; case study.
**Verdict:** partially addressed — the follow-up bug and a wide infrastructure layer landed, but "dynamic construction of an answer" from multi-source collected data is only registered/declared rather than demonstrated end-to-end, and the explicit "cache for at least 7 days" accessibility-status requirement is not mentioned in the PR body.
**Deferred/follow-up:** actual multi-source guide synthesis and the 7-day accessibility cache — no explicit deferral note.

## #445 — Unknown prompt: Hi, what is redis ? (closed 2026-06-24)
**Requirements (author: Georgepop; konard comment 2026-06-13, before the closing PR):**
- "We should be able to split every message on multiple actionable statements/questions ... our AI should first react to greating, and after that answer a question."
- "If single statement contains many actionable questions, we should make sure we also split them, when formalizing."
**Delivered (evidence):**
- Closing PR #475 "Fix compound courtesy and question prompts": splits unresolved compound prompts into sub-impulses; greeting answered first, then the question segment; source-ordered `compound_response` with trace evidence; en/ru/hi/zh coverage incl. fullwidth Chinese punctuation. Post-fix output still ends with "I could not determine `Redis` ..." — the question half remains unanswered.
**Verdict:** partially addressed — the greeting+question split (konard's first ask) shipped multilingually, but general splitting of "many actionable questions" inside one statement is not evidenced, and the underlying Redis definition lookup still fails, so the user-visible outcome is only half fixed.
**Deferred/follow-up:** multi-question decomposition beyond courtesy+question, and the actual Redis concept resolution — none noted as deferred.

## #446 — Issue with dialog: 10^100 (closed 2026-06-24)
**Requirements (author: AlexZenoo, not konard; no konard comments):**
- Bug report: "10^100" answered "10^100 = 1e+1" (wrong value) while "100^10" was correct.
**Delivered (evidence):**
- Closing PR #474 "Fix large integer exponent rendering": `^` parsing with bounded exact `BigUint` exponentiation in Rust; JS fallback keeps `BigInt` and stops trimming scientific notation; WASM rebuilt; source + Playwright regressions across UI languages; renders the exact 101-digit integer.
**Verdict:** fully addressed — exact-value fix in both runtimes with regressions.
**Deferred/follow-up:** none noted (`rust-script` size check skipped locally, noted in PR).

## #449 — Can arXiv:2605.00940 paper be useful in our project? (closed 2026-06-13)
**Requirements (konard):**
- "If it can, please apply all best practices from there, but use our associative technological stack."; case study in `./docs/case-studies/issue-449`; everything in one PR.
**Delivered (evidence):**
- Closing PR #450 "Apply interpretable experiential learning (arXiv:2605.00940) to symbolic probability ranking (#449)": verdict "yes"; ports evidence counts, counted utility, TU/TC thresholds, similarity fallback, episode feedback into `src/probability.rs`; generalized `ProbabilityDecisionPolicy` threaded through every selection use case; worked example; ARCHITECTURE §6.1; case study with R0–R10 all marked addressed ("future-work items 1 & 3 now done").
**Verdict:** fully addressed — mechanisms ported and generalized, with backward-compat proofs and full case study.
**Deferred/follow-up:** none noted (defaults keep new knobs opt-in).

## #451 — Reference the Symbolic AI Wikipedia article in docs (closed 2026-06-13)
**Requirements (konard):**
- "make sure we reference https://en.wikipedia.org/wiki/Symbolic_artificial_intelligence in docs"; "make sure we use all best known practices that are listed in this article and others in the domain ... but use our associative technological stack."; case study; everything in one PR.
**Delivered (evidence):**
- Closing PR #452: article referenced in README/VISION/ARCHITECTURE; 20-row best-practice audit ("16 applied / 3 partial / 0 proposed"); the one genuine code gap (SAT/constraint solving) closed by shipping a DPLL backend + Tseitin encoding; R298–R305 requirement matrix; case study with raw data + online research.
**Verdict:** fully addressed — docs reference, audit, and even a shipped decision procedure; the 3 remaining "partial" audit rows have named reuse targets rather than silence.
**Deferred/follow-up:** 3 partial best-practice audit rows (documented with reuse targets in `symbolic-ai-best-practices.md`); `splr`/`varisat` noted as the documented upgrade path for CDCL-scale workloads.

## #454 — В VISION.md не понятно, какую боль закрывает проект (closed 2026-06-13)
**Requirements (konard):**
- "В VISION.md не понятно, какую боль закрывает проект. Можете сгенерировать user journey для примера?"
- Follow-up comment (before close): "We need to make sure our docs include all user journeys, we currently support and can potentially support in the future."
**Delivered (evidence):**
- Closing PR #455 "docs(issue-454): document the pain VISION closes and add user journeys": new `docs/USER-JOURNEYS.md` (pain, personas, 11 current journeys J1–J11, 6 future F1–F6, coverage matrix, worked example); VISION.md opens with "Who This Is For And What Pain It Closes"; README link; docs-traceability test.
**Verdict:** fully addressed — both the original ask and the follow-up (all current + potential journeys) are covered.
**Deferred/follow-up:** none noted.

## #456 — Issue with dialog: What is the result? (closed 2026-06-25)
**Requirements (author: xlabtg, not konard; no konard comments):**
- Bug report: after a multi-step research task failed on CORS, "What is the result?" was answered with a dictionary definition of "result" instead of binding to the prior task.
**Delivered (evidence):**
- Closing PR #565 "Handle research result follow-ups": `research_result_followup` handler binds terse result/answer follow-ups to the previous research prompt; "reports that no CORS-readable search results were returned and that no verified source-backed analysis has been produced yet"; worker mirror.
**Verdict:** partially addressed — the absurd follow-up answer is fixed, but the fix is an honest failure report; the underlying multi-step research capability the dialog exercised still produces no analysis.
**Deferred/follow-up:** actually executing multi-step research tasks — not claimed, not flagged.

## #457 — Issue with dialog: Write a Rust program that parses its own source... (closed 2026-06-26)
**Requirements (author: xlabtg, not konard; no konard comments):**
- Bug report: complex self-source-metrics prompt hit "no template for language `rust` and task `missing`" dead end.
**Delivered (evidence):**
- Closing PR #566 "Fix Rust self-source metrics write_program prompt": curated `self_source_metrics_report` blueprint (include_str! self-parsing, counts, cyclomatic-style score, JSON output) plus a self-analysis addendum; worker mirror; blueprint extracted and executed in a temp Cargo project to verify output.
**Verdict:** fully addressed for the reported prompt — but via a hand-curated per-prompt blueprint (memoization), so nearby unlisted coding tasks would still dead-end; runs counter to konard's standing generalization mandate (#412/#433), though konard did not comment here.
**Deferred/follow-up:** none noted.

## #458 — Issue with dialog: Simulate a crypto portfolio tracker... (closed 2026-06-26)
**Requirements (author: xlabtg, not konard; no konard comments):**
- Bug report: composite search+code prompt dead-ended on `write_program ... task missing` (later fell to generic web search).
**Delivered (evidence):**
- Closing PR #567 "Fix crypto portfolio tracker blueprint routing": reviewed Python `crypto_portfolio_tracker` blueprint (mocked prices, holdings, alerts, markdown dashboard); guarded preemption so composite blueprints beat generic web-search fallback; worker parity; en/ru/hi/zh coverage; case-study note with pre-fix repro.
**Verdict:** fully addressed for the reported prompt — again a curated single-prompt blueprint rather than general program synthesis.
**Deferred/follow-up:** none noted.

## #459 — Issue with dialog: Build a "Smart Travel Planner" prototype... (closed 2026-06-26)
**Requirements (author: xlabtg, not konard; no konard comments):**
- Bug report: Python class-writing prompt dead-ended on `write_program ... task missing` / got hijacked by `Search:` bullets into web_search.
**Delivered (evidence):**
- Closing PR #568 "Fix smart travel planner class requests": `class` routed as a seed-backed program kind (so "Write a Python class" reaches write_program generally); Python `smart_travel_planner` blueprint with the requested methods, budget warnings, sample itinerary; worker parity; en/ru/hi/zh routing coverage.
**Verdict:** fully addressed for the reported prompt — class routing is a genuine generalization; the specific planner content is a curated blueprint.
**Deferred/follow-up:** none noted.

## #460 — Unknown prompt: Solve this step-by-step, but with verification... (closed 2026-06-26)
**Requirements (author: xlabtg, not konard; no konard comments):**
- Bug report: train-meeting word problem with required [STEP]/[VERIFY] format returned "(intent: unknown, reported)".
**Delivered (evidence):**
- Closing PR #570 "Fix train meeting word problem reasoning": "a targeted train-meeting word-problem normalizer" for prompts with opposing speeds and a stated distance; preserves bracketed [STEP n]/[VERIFY] tags; computes 700/(60+80)=5 h, 300/400 km split; source/unit/multilingual/Playwright regressions.
**Verdict:** partially addressed — the exact reported prompt now works, but the PR itself describes the fix as "targeted"; general word-problem reasoning (other kinematics/verification-format problems) is not claimed.
**Deferred/follow-up:** none noted.

## #461 — Unknown prompt: На php не получится написать? (closed 2026-06-26)
**Requirements (author: alexblizzard-star, not konard; no konard comments):**
- Bug report: after the capabilities answer advertising Hello World generation, the PHP follow-up returned unknown.
**Delivered (evidence):**
- Closing PR #569 "Fix Russian PHP Hello World follow-up": routes incomplete write-program follow-ups when the turn has a program-request verb plus a known catalog/oracle language; worker parity; regression for the exact dialog; case study; PHP answer served from the Hello World Collection oracle cache.
**Verdict:** fully addressed — recognizer-level fix (verb + language), not a single-string patch.
**Deferred/follow-up:** none noted.

## #462 — Unknown prompt: Перечисли фильмы про человека-паука в порядке выхода на экран? (closed 2026-06-27)
**Requirements (author: alexblizzard-star, not konard; no konard comments):**
- Bug report: Spider-Man films release-order question returned unknown.
**Delivered (evidence):**
- Closing PR #574 "Fix Spider-Man film release-order prompts": "seed-backed `fact_lookup` answer" for Spider-Man title-role films through 2023, ru/en variants, sourced to the Wikipedia article; regression test.
**Verdict:** partially addressed — the exact question is now answered, but purely via a hardcoded seed fact; any other "list films of X in release order" query remains unknown, and the seeded list will silently go stale after 2023.
**Deferred/follow-up:** none noted (no generalization to arbitrary film-series list queries).

## #464 — Time-of-day duration not routed to calculator (closed 2026-06-26)
**Requirements (author: unidel2035; konard comment 2026-06-26, ~90 min before close):**
- Report: route `HH:MM` arithmetic and "how long between A and B" to the existing datetime calculator.
- konard: "We also need to make sure our link calculator dependency fully supports all such and similar cases, if not - we should report issue there."
**Delivered (evidence):**
- Closing PR #572 "Fix clock-time duration calculator routing": seed-backed `time_duration_cue` role; prose normalized to `17:30 - 14:00`; worker + JS fallback; Rust/Playwright regressions; case study. PR notes "The `link-calculator` dependency already supports the direct expression; the missing piece was router/extractor coverage."
**Verdict:** partially addressed — the reported routing gap is fixed well, but konard's late comment asked for a full audit of the link-calculator dependency across "all such and similar cases" (with upstream issues if gaps exist); the PR only attests to the one direct expression.
**Deferred/follow-up:** link-calculator dependency audit / upstream issue filing (konard comment) — no evidence it happened.

## #465 — Coreference not applied: "Who created it?" after "What is Rust?" (closed 2026-06-27)
**Requirements (author: unidel2035; konard comment 2026-06-13, before the closing PR):**
- Report: run coreference before intent classification when a pronoun is unbound; after substitution the existing lookup path should resolve any topic.
- konard: "We need to support follow up questions for undefined words like `it`, with resolutions to closest contextual match, for fully logic context matching. And make sure we use our meta algorithm for reasoning."
**Delivered (evidence):**
- Closing PR #573 "Resolve Rust creator coreference follow-up": adds "a seed-backed Rust creator fact" (Graydon Hoare / Mozilla / Q575650); re-runs fact lookup after seeded coreference resolves the pronoun, recording rewrite traces; worker mirror; regression for the exact two-turn dialog.
**Verdict:** partially addressed — the exact Rust dialog works, but via a seeded Rust-specific fact plus "seeded coreference"; konard's ask for general closest-contextual-match pronoun resolution and meta-algorithm reasoning is not claimed, and unseeded topics ("What is Kafka?" → "Who created it?") are not evidenced to work.
**Deferred/follow-up:** general coreference for arbitrary antecedents + meta-algorithm use (konard comment) — not delivered, not flagged.

## #466 — Authorship intent doesn't generalize: "Who wrote War and Peace?" (closed 2026-06-28)
**Requirements (author: unidel2035, not konard; no konard comments):**
- Report: "for any work, resolve the author via Wikidata (e.g. property P50 'author') / Wikipedia ... generalizing the authorship intent beyond the seed."
**Delivered (evidence):**
- Closing PR #575 "Fix Wikidata authorship prompts": `author_of_book` added to the fact-query pipeline mapped to Wikidata P50; multilingual (en/ru/hi/zh) extraction + localized templates; seeded LOTR fact kept cacheable under the same relation; Playwright regression mocks P50 and resolves Leo Tolstoy in all four languages.
**Verdict:** fully addressed — this one actually generalizes via the live P50 relation instead of another seed, exactly as the reporter suggested.
**Deferred/follow-up:** none noted.


# Chunk ac analysis (issues 467..511, 30 issues)

## #467 — Unknown prompt: Даёт airindia бесплатную детскую коляску к багажу? (closed 2026-06-28)
**Requirements (konard):**
- Reported prompt fell to `unknown`; body attaches the Google answer ("Да, Air India позволяет провезти одну складную детскую коляску бесплатно...") as the target quality bar. No further comments.
**Delivered (evidence):**
- Closing PR #576 "Add Air India stroller baggage fact": seed-backed Air India stroller fact with aliases, localized answers, en/ru/hi/zh regression tests, official Air India source, Wikidata cache records.
**Verdict:** fully addressed — the reported prompt now resolves to a sourced fact; note the fix is a hand-seeded fact, not general web reasoning (konard did not explicitly demand generalization here).
**Deferred/follow-up:** none noted

## #468 — Our system should be able to solve such tasks in agentic mode (closed 2026-06-14)
**Requirements (konard):**
- Body (forwarded task): formalize texts (e.g. "Сказка о рыбаке и рыбке") into a KB with 9 primitives.
- Comment (before work): "our Formal AI system should have enough skills ... to actually call all the tools from any agentic CLI, understand errors from tools ... call bash commands, so web fetch and web search, to actually complete the task."
- "If it fails, you can use locally available claude and codex ... use their JSON output to actually reconstruct all reasoning steps."
- "We can also test our AI system on multiple AI benchmarks (at least 1-2 tasks/examples from each for agentic coding)."
- Case study folder + everything in one PR.
**Delivered (evidence):**
- Closing PR #469: deterministic server-driven tool loop across three OpenAI-compatible surfaces, offline driver, Links-Notation formalizer (all nine primitives as links, 37 records), CLI `formal-ai agent`, case study R306–R319. PR body has explicit "Honestly out of scope" section: general open-domain NL→KB extraction (needs neural inference, project NON-GOAL) and live network access in CI (offline corpus only).
**Verdict:** partially addressed — core agentic loop and example task delivered, but AI-benchmark testing (1-2 tasks per benchmark) and claude/codex JSON-session reasoning reconstruction are not mentioned in the PR body; open-domain NL→KB and live network explicitly declared out of scope.
**Deferred/follow-up:** "General open-domain NL→KB extraction ... needs neural inference — a project NON-GOAL"; "Live network access in CI" (PR #469 "Honestly out of scope" section). Benchmark testing not delivered/not claimed.

## #476 — Add on expandable sections button to the right, that when clicked means `expand only this section, collapse all others` (closed 2026-06-27)
**Requirements (konard):**
- "Add on expandable sections button to the right, that when clicked means `expand only this section, collapse all others`" with a best-fit icon.
- "click on the expandable header in other places should keep expand/collapse only single section"
- "By default we should only expand conversations and examples"
- "we should support SHIFT + CLICK as expand only this section"
**Delivered (evidence):**
- Closing PR #578 "Add sidebar section isolate action": isolate control on every section, header click stays a local toggle, Shift+Click maps to isolate, first-run defaults expand only Conversations and Example prompts, localized labels, regression tests, case study, screenshot.
**Verdict:** fully addressed — all four concrete asks covered one-to-one in the PR body.
**Deferred/follow-up:** none noted (only a note about a flaky parallel Playwright run).

## #477 — Unknown prompt: Что такое кубаторит? (closed 2026-06-27)
**Requirements (konard):**
- "our answers should be up to the standards or better (each statement should be verified and grounded to actual sources)" — with Google's answer as example.
**Delivered (evidence):**
- Closing PR #577 "Fix Russian kubatorit concept lookup": seeded concept dictionary entry for кубаторить/кубатурить with Academic.ru sources, localizations for en/ru/hi/zh, regression tests.
**Verdict:** partially addressed — the specific word is now answered with sources, but via a hand-seeded dictionary entry; the stated bar ("answers up to Google's standards" for such lookups generally) is only met for this one term.
**Deferred/follow-up:** none noted

## #478 — Unknown prompt: что такое нейросетевой инференс? (closed 2026-06-28)
**Requirements (konard):**
- (issue by AlexZenoo) konard comment 2026-06-28 15:04 (≈44 min before close, i.e. after PR work was done): "We need to use our best, most ambitious practices with generalization to meta algorithm for this case, and others."
**Delivered (evidence):**
- Closing PR #579 "Fix neural inference concept lookup": grounded multilingual `concept_neural_inference` seed record "instead of a prompt-specific branch", en/ru/hi/zh terms/aliases, Playwright + Rust regressions.
**Verdict:** partially addressed — reported prompt fixed with a grounded seed concept, but konard's late demand for meta-algorithm generalization ("for this case, and others") is not addressed: still per-concept seeding, not a general unknown-concept capability. (Flag: konard's comment landed the same day the fix merged.)
**Deferred/follow-up:** meta-algorithm generalization for unknown concept lookups (konard comment) not claimed anywhere in PR #579.

## #479 — `Not available in latest release` for all desktop apps (closed 2026-06-17)
**Requirements (konard):**
- "Desktop apps are not available." "screenshots for destop apps are obsolete." "macOS instructions don't have screenshots like in ... vk-bot-desktop"
- "All our templates also should include a way to have CI/CD for desktop application download page (/download) ... API docs (/docs/api) ... website (/app) ... landing page (/)"
- Follow-up 06-15 (after PR #487 work): "По прежнему не исправлено, скриншоты macOS фейковые" + NEW requirement: "убедись что исходный код на лендинге это большая кнопка".
- Follow-up 06-15: "After changes are applied linux app is not available still." / "there are now again no releases. Check CI/CD for false positives and errors, and fix them all."
- Follow-up 06-17 (after PR #490 merged): "Still no release for macOS ... Anyway we need fix it all."
**Delivered (evidence):**
- Three closing PRs: #487 (Linux assets + macOS signing config, real vk-bot screenshots, DMG smoke test), #490 (reseal ad-hoc macOS bundles after both macOS jobs failed in v0.205.0), #510 (root cause: electron-builder 26 skips the sign hook without `-c.mac.identity=-`; adds the flag + test; confirms web structure incl. big source-code CTA and real Gatekeeper screenshots; template comparison: "no upstream bug to file"; CI "skipped" runs judged correct behavior).
**Verdict:** partially addressed — konard re-opened the complaint three times (comments AFTER each closing PR); PR #510 is a plausible final fix but the issue closed same day with no in-thread confirmation a macOS asset actually shipped ("the next auto-release ... self-heals"), and the template /download CI/CD ask was recorded only as a "ready-to-file ... optional desktop-release upstreaming enhancement", i.e. not filed.
**Deferred/follow-up:** desktop-release upstreaming to the four templates left ready-to-file, not filed (PR #510 template-comparison section); actual macOS release success deferred to the next auto-release.

## #481 — Unknown prompt: how order 3d print in nan chang vietnam? (closed 2026-06-28)
**Requirements (konard):**
- "we need to make sure we can reason through it formally to reproduce the same result and reconstruct reasoning step to the smallest atomic links substitution operations" (Google's multi-step answer as the target).
**Delivered (evidence):**
- Closing PR #580 "Fix telegraphic how-order procedural prompts": seed-gated telegraphic `how <verb>` recognition (verb `order` seeded), routes to `procedural_how_to` with fallback query, guard against arbitrary `how <word>`, worker mirror, case study.
**Verdict:** partially addressed — routing fixed so the prompt is no longer `unknown`, but nothing reproduces the Google-quality answer or reconstructs reasoning "to the smallest atomic links substitution operations"; only the single action verb `order` was seeded.
**Deferred/follow-up:** the reproduce-Google-answer-formally requirement is silently narrowed to intent routing; nothing explicitly deferred in PR body.

## #482 — Find a way to use Nemotron-3-Ultra training data, to add tests for our AI system (closed 2026-07-13)
**Requirements (konard):**
- "we should try to add 10 random samples, and make sure we can use it to build tests, that will increase quality of our system, by writing failing tests, and make sure our code base generalizes to solve them all."
- "we need to have scripts to get more when needed." No LLM use, no full dataset download.
- "Ideally this task itself must be fully and partially solvable by Formal AI connected to Agent CLI. You task to drive Formal AI to make all the actions..."
- Case study + single PR.
**Delivered (evidence):**
- Closing PR #639 "Add Nemotron training data sample tests": no-full-download sampler script, deterministic 10-row CC-BY-4.0 sample, Links Notation benchmark fixture, unit ratchets for fixture validity/provenance/license and "10/10 ingestion pass count", case study, benchmark index.
**Verdict:** partially addressed — sampling, scripts, and ingestion ratchets delivered, but tests pin *ingestion* of the 10 rows rather than failing capability tests that force the codebase to "generalize to solve them all"; driving the task through Formal AI + Agent CLI is not claimed in the PR body.
**Deferred/follow-up:** Agent-CLI-driven execution and capability generalization are silently absent; nothing explicitly deferred.

## #484 — Unknown prompt: Расскажи за Telegram Ads на русском (closed 2026-06-28)
**Requirements (konard):**
- (issue by maksmoroz91) konard comment 06-28 16:46 (~1.5h before close, after PR work started): "We must generalize to all similar questions in all languages, and user our meta algorithm and actual reasoning."
**Delivered (evidence):**
- Closing PR #581 "Honor concept response language markers": seed-backed `response_language_marker` for en/ru/hi/zh; concept lookup strips the trailing language directive and renders the concept in the requested language; worker mirror; regression covers all four directives incl. the original prompt.
**Verdict:** partially addressed — the directive class is generalized across the 4 supported languages via seed markers, but "all languages" is limited to en/ru/hi/zh and it is marker matching rather than the demanded meta-algorithm reasoning.
**Deferred/follow-up:** none noted

## #485 — Unknown prompt: Привет, как подключить mysql к node js (closed 2026-06-28)
**Requirements (konard):**
- (issue by maksmoroz91) konard comment 06-28 16:46 (~1h before close): "We must generalize to all similar questions in all languages, and user our meta algorithm and actual recursive reasoning steps."
**Delivered (evidence):**
- Closing PR #582 "Fix multilingual elided how-to procedural prompts": generalizes weak elided `how <action>` recognition across seed languages, adds seeded `connect` action for en/ru/hi/zh, greeting-prefixed compound handling, Rust + Playwright coverage.
**Verdict:** partially addressed — greeting+how-to compound now returns a procedural plan, but generalization depends on individually seeded action verbs (only `connect` here; `order`/`install` in sibling PRs), not the universal recursive meta-algorithm konard demanded.
**Deferred/follow-up:** none noted

## #488 — Deep thinking (closed 2026-06-17)
**Requirements (konard):**
- "add thinking UI, that shows last thinking paragraph with expand button ... show half of second to last paragraph and do gradient"
- "we translate each reasoning step into its description in meta language, after that we translate meta language to target user language."
- "we should be able to configure granularity ... recursively composite steps"
- "we should be able to solve exactly entire class of tasks ... general universal problem solving algorithm"; case study; single PR.
**Delivered (evidence):**
- Closing PR #489 "Add concrete-by-default thinking across all surfaces": `src/thinking.rs`, naturalized steps on CLI/OpenAI/Anthropic/Telegram/web, collapsed preview with gradient fade + expand, `Thinking detail` granularity, `parent_id` composite steps, R1–R14 coverage table mapping every issue requirement. Documented "Design boundary": non-UI surfaces render the English meta-language summary as-is (localization only where the i18n catalog lives).
**Verdict:** fully addressed (with a documented boundary) — all R1–R14 mapped; the step→meta-language→user-language pipeline is complete only in the browser UI; CLI/API/Telegram stay English by design.
**Deferred/follow-up:** target-language localization of thinking on non-UI surfaces — explicit "Design boundary (intended, documented)" section in PR #489.

## #492 — Badges are `invalid` and `failed` in the GitHub releases (closed 2026-06-28)
**Requirements (konard):**
- "It must be fixed." "We also need to have all our traditional badges at README.md"
- Compare with the four pipeline templates; "if the same issue is found in template report issue also in templates"; case study; single PR.
**Delivered (evidence):**
- Closing PR #583 "Fix release badges and restore README badges": static release-version badges in release notes, restored README badge block (CI/CD, Desktop Release, crates.io, docs.rs, Rust version, Codecov, license), case study with evidence, upstream issue filed: link-foundation/rust-ai-driven-development-pipeline-template#85.
**Verdict:** fully addressed — both concrete asks done and the matching template got an upstream issue.
**Deferred/follow-up:** none noted

## #493 — We should be able to fact check such cases (closed 2026-07-03)
**Requirements (konard):**
- "I see no price of 1700 for Etherium in whole 2024 year ... we need to make sure if something is not true, we catch and formally verify."
- "We should be able to work with a text from the image, and also check if our experimental OCR allows to transcribe it correctly. And fix if possible with tessaract, if not - just keep it as failed skipped test."
- "We must generalize to all similar questions in all languages"; case study; single PR.
**Delivered (evidence):**
- Closing PR #619 "Fact-check market-price claims across assets, periods, and languages": data-driven registry (`market-price-references.lino`) for ETH/BTC 2021–2024 with en/ru/hi/zh aliases, within-range vs contradicted logic (only 2024 flagged), meta-algorithm recipe pinned by test, Tesseract OCR capture and first-party Binance kline data in the case study, Playwright coverage.
**Verdict:** partially addressed — the reported false claim is caught and the class generalized *within the registry* (2 assets, 4 years, 4 languages; new coverage requires seed edits); the OCR "failed skipped test" ask is only evidenced as a case-study capture, not clearly a test.
**Deferred/follow-up:** covering new assets/periods/languages is a seed-data edit (PR body); OCR test status ambiguous.

## #494 — Free space policy (closed 2026-07-12)
**Requirements (konard):**
- Keep "learned experience (generalized algorithms) and history of recorded raw events, that cannot be extracted from remote sources"; cached/external data and intermediate conclusions "can be freed".
- "count usage for cached data, so if something is not used in cached data (even in seed data) it can be freed first."
- "We should always free only enough space to store next required links/data/files."
- "By default system should free nothing, but when not enough space in memory we should ask user if to enable auto-free-space algorithm."
- "When even our algorithm cannot free enough space ... we should ask user to migrate AI memory to bigger storage."
**Delivered (evidence):**
- Closed by PR #645 "Complete verified idle dreaming and memory cleanup" — primarily issue #540's dreaming PR (Closes #540, Closes #494). #494 is mentioned once: "§4 issue #494 consent, backup, and incoming-bytes behavior tested"; plus "fall back to normal usage/priority eviction for unverifiable records".
**Verdict:** partially addressed — consent gating, usage/priority eviction and incoming-bytes sizing are evidenced, but the PR body never addresses the "migrate to bigger storage" prompt, seed-data eviction, or free-only-enough specifics; #494 closed as a rider on another issue's PR.
**Deferred/follow-up:** "ask user to migrate AI memory to bigger storage" — no evidence in thread or PR body.

## #495 — Unknown prompt: Сколько всего правил? (closed 2026-06-28)
**Requirements (konard):**
- (issue by Michael-Bokov) konard comment 06-28 (same day as close): "We must generalize to all similar questions in all languages, and user our meta algorithm and actual reasoning."
**Delivered (evidence):**
- Closing PR #584 "Handle behavior rule count questions": seed-backed `behavior_rules_count` intent for en/ru/hi/zh, counts built-in + dialog-local rules, explicit reasoning + `links` metadata (counting algorithm exposed), Rust/worker/browser mirrors, multilingual regressions.
**Verdict:** fully addressed — the question class handled across the 4 supported languages with exposed counting reasoning ("all languages" again means the 4 seed languages).
**Deferred/follow-up:** none noted

## #496 — Unknown prompt: А по русски кратко? (closed 2026-06-28)
**Requirements (konard):**
- (issue by Michael-Bokov) konard comment 06-28: "We must generalize to all similar questions in all languages, and user our meta algorithm and actual reasoning."
**Delivered (evidence):**
- Closing PR #585 "Fix behavior-rule follow-up prompts": seed-backed count and brevity cues across supported languages; follow-ups after a rules list route to `behavior_rules_count`/`behavior_rules_brief`, preserving localized response-language requests like `по русски`.
**Verdict:** fully addressed — the follow-up class (count/brief + language switch) covered for the supported languages.
**Deferred/follow-up:** none noted

## #497 — Unknown prompt: можно ли узнать заходил ли кто либо в твое репо на github? (closed 2026-06-28)
**Requirements (konard):**
- (issue by Michael-Bokov) konard comment 06-28 19:14: "We must generalize to all similar questions (the whole class of similar questions) in all languages, and we must use our general and universal meta algorithm and actual recursive reasoning steps, expressed in meta language. Everything must be expressed recursively through meanings (meta language), and all meanings must be grounded in external data sources. Every finest detail must be tested."
**Delivered (evidence):**
- Closing PR #586 "Answer GitHub repository traffic prompts": semantic seed roles for GitHub-traffic questions in en/ru/hi/zh, new `github_repository_traffic` handler in Rust + worker, answers from official GitHub docs, regression + case study.
**Verdict:** partially addressed — the reported question class is answered with grounded sources in 4 languages, but via a purpose-built handler, not the "general and universal meta algorithm" konard demanded; recursive meta-language expression not claimed.
**Deferred/follow-up:** none noted

## #498 — Обучать formal ai по популярным запросам в google (closed 2026-07-09)
**Requirements (konard):**
- (issue by skulidropek) konard comment 07-04: "We need to have automated scripts/tools, to actually being able to convert Google Trends to data collection for adding test cases ... And make sure we can answer top 10 requests from Google Trends ... and also variations of these requests in all languages are supported."
- "Ideally this task itself must be fully and partially solvable by Formal AI connected to Agent CLI."
**Delivered (evidence):**
- Closing PR #640 "Train Formal AI on Google Trends via a human-gated auto-learning loop": RSS snapshot → top-10 topics → 80 prompts (2 variations × 4 languages) answered by the engine; but only "20 the engine already routes" — the other 60 form a **learning frontier left at `intent \"unknown\"`**, and the human-gated learner "honestly adopts nothing". Agent CLI drives catalog + frontier generation; planner recipes consolidated.
**Verdict:** partially addressed — scripts/tooling and Agent-CLI-driven pipeline delivered, but the central ask "make sure we can answer top 10 requests from Google Trends" is NOT met: 60 of 80 generated prompts remain unanswered, framed as the honest outcome.
**Deferred/follow-up:** the 60-prompt learning frontier explicitly left unadopted ("the learner honestly adopts nothing — the proposal-only result is committed", PR #640); actually answering trending queries remains future work.

## #499 — Unknown prompt: Обратясь сюда ты узнаешь актуальные темы https://trends.google.com/... (closed 2026-07-09)
**Requirements (konard):**
- (issue by skulidropek) konard comment 07-04 (same text as #498) plus: "Here also it is a part of auto learning process, so user explains, where to get data for reasoning and leaning (for formal AI to improve itself)."
**Delivered (evidence):**
- Closing PR #641 "Recognize the #499 learn-from-source directive and drive it via the Agent CLI": seed-declared `learn_from_source` intent (data-driven registry of learnable sources), same directive drives the Agent CLI recipe live in CI, localized acknowledgement in en/ru/hi/zh. PR body: "nothing is auto-adopted — the value is the auditable frontier."
**Verdict:** partially addressed — the teach-a-source directive is recognized and wired to the learning loop end-to-end, but as in #498 the loop adopts nothing, so "answer top 10 requests ... in all languages" remains unmet.
**Deferred/follow-up:** actual adoption of learned answers from the trends frontier — explicitly not done (PR #641 "adopts nothing" paragraph).

## #500 — Unknown prompt: cursor (closed 2026-06-29)
**Requirements (konard):**
- (issue by skulidropek) konard comment 06-28: "That should result in search about the term. We must generalize to all similar requests (the whole class of similar questions) in all languages ... grounded in external data sources. Every finest detail must be tested."
**Delivered (evidence):**
- Closing PR #591 "Search unresolved bare terms": unresolved single-term prompts route to `web_search` (`unresolved_bare_term` query kind) after link-memory and public concept-cache miss; worker mirror requires grounded results; mocked e2e coverage.
**Verdict:** fully addressed — the whole bare-term class now falls through to grounded web search rather than `unknown`, matching konard's primary ask.
**Deferred/follow-up:** none noted

## #501 — Unknown prompt: how install cursor (closed 2026-06-29)
**Requirements (konard):**
- (issue by skulidropek) konard comment 06-28: "we should go to official documentation and take guiding steps from there (parsing them to meta language, if target language is not available), and giving them in requested/target language by deformalizating meta language in target language." + generalization boilerplate.
**Delivered (evidence):**
- Closing PR #587 "Route install how-to prompts to official docs": `install` seeded as a procedural action in 4 languages; install tasks "prefer official documentation and official repository install pages"; large worker refactor (seed tables to `.lino`, worker split into modules, CI guards).
**Verdict:** partially addressed — routing + official-docs preference delivered, but no evidence the system *extracts guiding steps* from official docs into meta language and deformalizes them into the target language; it points at docs rather than answering with the steps.
**Deferred/follow-up:** parse-official-docs-steps-to-meta-language pipeline — not claimed anywhere; silently dropped.

## #502 — Unknown prompt: А можешь на 10 часов по Грузии с Марией? (closed 2026-06-30)
**Requirements (konard):**
- (issue by skulidropek; no konard comments) Reported elliptical Russian calendar follow-up fell to `unknown`.
**Delivered (evidence):**
- Closing PR #596 "Handle Russian calendar spoken-hour prompts": recognizes `на 10 часов` spoken-hour wording, routes elliptical/reordered meeting prompts to `calendar_create_event` in Rust + worker, preserves participant titles and Georgia timezone, regressions for #502/#503/#504/#522.
**Verdict:** fully addressed — the reported prompt and its siblings now produce calendar events with .ics + Google Calendar links.
**Deferred/follow-up:** none noted

## #503 — Unknown prompt: Создай встречу на 10 часов с Марией (closed 2026-06-30)
**Requirements (konard):**
- (issue by skulidropek; no konard comments) Same spoken-hour calendar class as #502.
**Delivered (evidence):**
- Same closing PR #596; the exact prompt is listed among the four reproduction prompts now producing calendar responses.
**Verdict:** fully addressed — reported prompt covered by regression.
**Deferred/follow-up:** none noted

## #504 — Unknown prompt: Встречу с Марией на 10 часов (closed 2026-06-30)
**Requirements (konard):**
- (issue by skulidropek; no konard comments) Reordered variant of the same calendar class.
**Delivered (evidence):**
- Same closing PR #596; prompt listed in the reproduction set with regression coverage.
**Verdict:** fully addressed — covered by the same generalized spoken-hour parsing.
**Deferred/follow-up:** none noted

## #505 — Unknown prompt: Интересует Cursor AI (closed 2026-06-28)
**Requirements (konard):**
- konard comment 06-28: "That should result in search about the term. All search results should be translated into single meta language, merged, and the most relevant statements from results should be presented in target language by deformalization of meta language." + generalization boilerplate.
**Delivered (evidence):**
- Closing PR #588 "Route topic-interest prompts to web search": multilingual topic-interest surfaces (prefix/suffix/circumfix) for en/ru/hi/zh; the prompt routes to `web_search` with query `cursor ai`; worker parity; regressions.
**Verdict:** partially addressed — routing to search delivered, but the second half of the ask (translate results into meta language, merge, present most relevant statements via deformalization) is absent from the PR body.
**Deferred/follow-up:** meta-language result merging/synthesis — not claimed, silently dropped.

## #506 — Unknown prompt: Найди мне хакатоны (closed 2026-06-29)
**Requirements (konard):**
- konard comment 06-29 (day of close): "we should be ready to search list of actual events from search results, remove duplicates and for all that duplicated provide multiple sources. And we also consider adding ability to add events to calendar (with all popular systems, for Apple, Google, Microsoft and so on)." + generalization boilerplate.
**Delivered (evidence):**
- Closing PR #589 "Route event-listing prompts to web search": event-listing prompts route to `web_search` with query `хакатоны`; noise-word stripping (`мне`, `актуальные`); en/ru/hi/zh signals; Rust + JS parity coverage.
**Verdict:** partially addressed — routing fixed, but event extraction from results, deduplication with multiple sources, and add-to-calendar (Apple/Google/Microsoft) are all absent from the PR body.
**Deferred/follow-up:** event-list dedup with multi-source citations; add-events-to-calendar integration — both from konard's comment, not claimed anywhere.

## #507 — Unknown prompt: Где посмотреть актуальные хакатоны? (closed 2026-06-30)
**Requirements (konard):**
- konard comment 06-30 links back to his #506 comment (event dedup + calendar asks apply here too).
**Delivered (evidence):**
- Closing PR #593 "Guard Russian hackathon prompts with web-search regression": regression test that both dialog turns route to `web_search` with query `хакатоны` (routing itself came from PRs #589/#594); changelog.
**Verdict:** fully addressed for the reported prompt (regression-guarded web-search routing); konard's linked #506 asks remain unmet but are accounted under #506.
**Deferred/follow-up:** see #506 (dedup + calendar), referenced via konard's comment link.

## #508 — Unknown prompt: Какие хакатоны сейчас проходят? (closed 2026-06-30)
**Requirements (konard):**
- konard comment 06-30 links to the same #506 comment.
**Delivered (evidence):**
- Closing PR #594 "Route current event questions to web search": current-event questions route to `web_search` with query `хакатоны`; seed roles for public-event subjects, current-time markers, multilingual cleanup; Rust + JS parity.
**Verdict:** fully addressed for the reported prompt class; the linked #506 dedup/calendar asks remain unmet (tracked under #506).
**Deferred/follow-up:** see #506.

## #509 — Надо сделать что бы FormalAI умел работать с историей (closed 2026-06-30)
**Requirements (konard):**
- (issue by skulidropek) konard comment 06-29: "We need to support multiple natural language queries to the history of the dialog, and the whole memory. ... So we should have something much more powerful than SQL, GraphQL and so on. Natural language queries to the links (associative) memory."
**Delivered (evidence):**
- Closing PR #590 "Add natural-language conversation and memory recall": `conversation_recall` extended from dialog turns to persisted `MemoryEvent` records and full-memory bundles; all three API surfaces wired through `SyncStore`; new `formal-ai memory query --path ... --prompt ...` CLI; integration + unit tests.
**Verdict:** fully addressed — NL queries over both dialog history and whole persisted memory delivered on API + CLI; "more powerful than SQL/GraphQL" is unquantifiable aspiration but the concrete asks are covered.
**Deferred/follow-up:** none noted

## #511 — Unknown prompt: Выполни `ls ~` в терминале (closed 2026-06-20)
**Requirements (konard):**
- "from the start it will ask to switch to agentic mode (with configuration for each bash tool call)"; system message with per-permission grants; install/upgrade Agent CLI; start local OpenAI-compatible server.
- "add full integration and e2e tests to make sure our desktop app fully supports that case from the start"; render Agent CLI output in existing chat UI; "chat/agent/full auto modes should be single radio button group on top".
- Use agent-commander (never Agent CLI directly), report missing agent-commander features; use a separate docker container, never local claude/codex; case study; single PR.
**Delivered (evidence):**
- Closing PR #512 "agent-mode case study, requirements, and full implementation (E1–E8)": all eight milestones merged — terminal intent + 3-way mode radio (E1), onboarding + per-tool permission panel (E2), server auto-start (E3), agent-commander provider with default-deny router (E4), installable `formal-ai-agent` container (E5), NDJSON chat rendering (E6), cold-start `ls ~` e2e (E7), upstream re-verification (E8). Upstream issues agent#271, agent-commander#39/#40 filed and resolved. R1–R21 requirements inventory with per-requirement evidence.
**Verdict:** fully addressed — every enumerated ask maps to a merged milestone; one residual limitation documented as an upstream constraint.
**Deferred/follow-up:** "approve-each relays only for `agent`/`claude` because `codex`/`gemini`/`qwen` expose no headless approval handshake — a documented upstream-CLI constraint" (PR #512 body).


# Chunk ad analysis — issues 513..561 (30 issues)

## #513 — E1: Terminal-command intent + three-way Mode radio (visible fix for #511) (closed 2026-06-17)
**Requirements (konard):**
- "`Выполни \`ls ~\` в терминале` (ru) and `run \`ls ~\` in terminal` (en) no longer return `unknown`."
- "The top toolbar shows a three-way `chat`/`agent`/`full-auto` radio group; switching is one click and reflected in the status label."
- "Unit tests in both engines (JS + Rust) for the new intent; one e2e for the mode switch."
**Delivered (evidence):**
- Closing PR #525 "E1: Terminal-command intent + three-way Mode radio". PR body claims R5 (terminal intent in both JS worker and Rust solver, en/ru/hi/zh), R6 (three-way mode radio + `mode` preference + status label), Rust unit + JS parity + e2e tests, plus a review follow-up moving all NL to seed data with CI gates. Closed explicitly by hive-mind bot comment (PR targeted the non-default epic branch).
**Verdict:** fully addressed — every acceptance criterion is matched point-by-point in the PR body, including both engines and e2e.
**Deferred/follow-up:** none noted (pre-existing issue-221 e2e sandbox failures mentioned as environmental only).

## #514 — E2: Per-tool / per-command permission UI + onboarding message (closed 2026-06-18)
**Requirements (konard):**
- "Each tool (`shell`, `http_fetch`, `read_local_file`, …) and, in `agent` mode, each concrete command can be granted or declined independently."
- "`full-auto` runs granted tools without per-command prompts; empty grants refuse everything."
- "The onboarding system message is emitted once on first agent intent and the decision persists."
- "Regression test: no grant ⇒ every tool refused; the new UI never bypasses `isPermitted`."
**Delivered (evidence):**
- Closing PR #528 "E2: Add per-tool desktop permissions and command approval": persisted per-tool grants for 6 tools, one-time Agent/Full Auto onboarding system message, per-command shell approve/deny in Agent mode, Full Auto skips per-command prompt but keeps the tool gate, router + e2e coverage for default-deny/partial grants/denial/approval.
**Verdict:** fully addressed — all four acceptance criteria have direct claims and tests in the PR body.
**Deferred/follow-up:** none noted.

## #515 — E3: Auto-start the local OpenAI-compatible server for agent mode (closed 2026-06-20)
**Requirements (konard):**
- "Entering agent mode yields a ready local-server `apiBase`; an already-running server is reused (not double-started)."
- "Unit tests with a mocked server lifecycle (start, health-probe, reuse, failure)."
**Delivered (evidence):**
- Closing PR #530 "E3: Auto-start local server for agent mode": testable lifecycle manager (start, `/health` probe, reuse, token scrub, `apiBase` + provider metadata), wired to Agent/Full Auto mode entry; desktop unit tests and CI run passed.
**Verdict:** fully addressed — both acceptance criteria covered by the PR's manager + tests.
**Deferred/follow-up:** none noted.

## #516 — E4: AgentProvider seam — in-process provider + agent-commander provider (closed 2026-06-20)
**Requirements (konard):**
- "A read-only command executes via the in-process provider in tests."
- "The commander provider is selectable and never invokes a CLI directly or touches host subscriptions."
- "CI guard fails the build if a host `claude`/`codex`/`agent` spawn is introduced."
**Delivered (evidence):**
- Closing PR #532 "E4: AgentProvider seam for desktop agent execution": default hermetic in-process provider, opt-in `commander` provider via env var, grants mapped to `--plan-only`/`--read-only`/`--approve-each`, host-credential scrubbing, recursive static guard against direct host agent/claude/codex spawns, 6 targeted test cases.
**Verdict:** fully addressed — all three acceptance criteria have matching claims and tests.
**Deferred/follow-up:** grant→flag mapping is described as "coarse" restrictions; the upstream agent#271 permission gap was already tracked in the issue itself.

## #517 — E5: Installable Formal-AI container (server + agent + agent-commander) & CLI setup (closed 2026-06-20)
**Requirements (konard):**
- "One-click 'Install agent environment' produces a ready container; `agent --version` is present/current inside it."
- "Autonomous tools run only inside the container; no host CLI is invoked."
- "The applied hive-mind isolation practices are documented."
**Delivered (evidence):**
- Closing PR #533 "E5: Installable Formal-AI agent environment": DinD image bundling Node + `@link-assistant/agent` + `agent-commander`, CLI validation (`formal-ai --version`, `agent --version`, `start-agent --help`), one-click desktop install flow with health check, Compose profile, isolation-practice notes.
**Verdict:** fully addressed — with a verification caveat: the PR admits the local Docker daemon died during final image export ("rpc error ... EOF") and could not be restarted, so the complete image build was never validated end-to-end in that session.
**Deferred/follow-up:** final Docker image export validation left unverified in the PR environment (PR body "Docker image validation note").

## #518 — E6: Render Agent CLI (NDJSON) output into the existing chat UI (closed 2026-06-20)
**Requirements (konard):**
- "An agent turn renders like normal chat with tool steps from a recorded NDJSON fixture (unit test) and live (e2e)."
- "Assistant text, tool start/result, and error events each map to the correct chat element."
**Delivered (evidence):**
- Closing PR #536 "Render Agent CLI NDJSON in chat UI": NDJSON adapter for assistant text/tool start-result/permission/error events, normalized `answer` payloads for both providers, fixture unit coverage plus supported-language Playwright coverage.
**Verdict:** fully addressed — fixture + Playwright coverage matches the criteria; the "live" real-CLI stream variant effectively landed via the container-gated E7 test (#519/PR 537) rather than here.
**Deferred/follow-up:** none noted in the PR body.

## #519 — E7: Full integration + e2e for the cold-start `ls ~` journey (closed 2026-06-19)
**Requirements (konard):**
- "CI runs the hermetic journey green (onboarding → grant → mode switch → `ls ~` listing in chat)."
- "The container-gated variant (real CLI through agent-commander, inside the container) passes on demand."
**Delivered (evidence):**
- Closing PR #537 "Add cold-start agent ls home E2E coverage": hermetic in-process cold-start spec covering onboarding, mode switching, grants, denial, approval and a real home-listing render; `FORMAL_AI_E2E_AGENT_COMMANDER=1` gated commander variant (skipped by default, runs when a ready container exists).
**Verdict:** fully addressed — both criteria have direct matching claims; the gated variant is "on demand" exactly as specified, though there is no recorded proof in the thread that the container variant was ever actually run green.
**Deferred/follow-up:** commander-provider variant remains container-gated/skipped by default (PR body "Local Checks": "1 passed, 1 skipped").

## #520 — E8: Upstream feedback to agent-commander + best-practices write-up (closed 2026-06-20)
**Requirements (konard):**
- "Any further gaps found in E4–E7 are filed on `link-assistant/agent-commander` and linked here."
- "The best-practices doc is finalized and merged."
**Delivered (evidence):**
- Closing PR #539 "Close out agent-commander upstream feedback": reverified upstream state (agent 0.24.0, agent-commander 0.8.0, no open upstream issues), removed the stale `--tool agent --read-only` workaround, finalized the #511 best-practices write-up and case-study status docs. States "E4-E7 did not reveal a new agent-commander defect".
**Verdict:** fully addressed — no new gaps existed to file, and the doc closeout is claimed explicitly.
**Deferred/follow-up:** PR notes a remaining upstream CLI capability limitation (approve-each relay only for `agent`/`claude`; `codex`/`gemini`/`qwen`/`opencode` lack a relayable headless approval handshake) — left as upstream reality, not filed as a new issue.

## #521 — Unknown prompt: расскажи мне об языке Rust (closed 2026-06-29)
**Requirements (konard):** (issue authored by skulidropek; konard's comment 2026-06-29 sets the requirements)
- "That should result in search about the term."
- "We must generalize to all similar requests (the whole class of similar questions) in all languages, and we must use our general and universal meta algorithm and actual recursive reasoning steps, expressed in meta language."
- "all meanings must be grounded in external data sources. Every finest detail must be tested."
**Delivered (evidence):**
- Closing PR #592 "Route tell-me-about term prompts to web search": seed-driven `term_information_request_opener` role "across supported languages", routes to `web_search`, mirrored in browser worker, Rust unit coverage plus one browser regression for the exact reported prompt.
**Verdict:** partially addressed — the concrete routing fix and class-level (tell-me-about) generalization shipped, but the PR body claims nothing about the universal meta algorithm, recursive reasoning steps in meta language, or grounding the new meanings in external data sources, and test coverage is one class, not "every finest detail".
**Deferred/follow-up:** meta-algorithm/recursive-reasoning/grounding requirements from konard's comment are not mentioned in the PR body at all — silently dropped there (comparable issues #535/#556 got explicit grounding/recipe treatment).

## #522 — Unknown prompt: Поставь мне встречу с Леваном на 5 часов по Грузии (closed 2026-06-30)
**Requirements (konard):**
- None from konard — issue authored by skulidropek with no konard comments; the implicit ask is that the Russian spoken-hour meeting prompt should not return `unknown`.
**Delivered (evidence):**
- Closing PR #596 "Handle Russian calendar spoken-hour prompts": recognizes `на 5/10 часов` wording, routes to `calendar_create_event` in Rust solver and browser worker, `.ics` + Google Calendar links, regression coverage for #502/#503/#504/#522, timezone handling preserved.
**Verdict:** fully addressed — the exact reported prompt is listed as now producing a calendar event.
**Deferred/follow-up:** none noted.

## #523 — Fix all false positives and errors at CI/CD (closed 2026-06-17)
**Requirements (konard):**
- "Use all the best practices from CI/CD templates (check full file tree to compare for all GitHub workflow and CI/CD scripts file), if the same issue is found in template report issue also in templates."
- "We need to download all logs and data related about the issue ... compile that data to `./docs/case-studies/issue-{id}` folder, and use it to do deep case study analysis."
- "If there is not enough data to find actual root cause, add debug output and verbose mode if not present."
**Delivered (evidence):**
- Closing PR #524 "fix(ci): resolve 'No space left on device' in Deploy Demo job": root-caused the shared multi-GB target cache, isolated cache key, freed pre-installed SDKs, added `df -h` logging (the requested verbose output), full template comparison in the case study concluding "no upstream bug to file", triaged the two remaining warnings (codecov transitive deprecation = external; file-size warnings = intended advisory).
**Verdict:** fully addressed — the only hard failure is fixed, warnings are triaged with reasons, template comparison and case study delivered.
**Deferred/follow-up:** none noted (10 file-size advisory warnings deliberately left as intended lint output).

## #526 — Translation quality test (closed 2026-07-04)
**Requirements (konard):**
- "Translation to meta language from any other language must have zero data loss ... roundtrip from meta language and back must work always."
- "test roundtrips from Rust to meta language to JavaScript ... Russian -> meta language -> English ... tests for all roundtrips between each separate language and meta language, and between each 2 languages we support."
- "We should never support direct translation without meta language."
- "Make sure our vision, requirements, roadmap and contributing guide all ... include all relevant text about the case."
**Delivered (evidence):**
- Closing PR #635 "Add translation round-trip quality coverage": reworked `translate_program` through a language-neutral `CodeMeaning` meta layer (removes the N² direct-pair anti-pattern), Rust→JS→Rust code round-trip tests, "every directed en/ru/hi/zh pair round-trips through the shared meaning", a test proving never-hardcoded pairs route through one shared meaning, and updates to VISION/ARCHITECTURE/ROADMAP/REQUIREMENTS/CONTRIBUTING.
**Verdict:** fully addressed — within the supported language set every listed roundtrip axis (code langs via meta, natural langs via meta, docs) has a matching claim; the absolute "zero data loss ... always" guarantee is only as strong as the covered vocabulary/tests.
**Deferred/follow-up:** none noted.

## #527 — Generate all possible questions and answer them (closed 2026-07-07)
**Requirements (konard):**
- "some function that produces infinite sequence of questions from smallest to largest ... configurable what counts as question."
- "use only 10% of most frequent word[s] ... For 3 words and so on, we can take 5% ... For 4 we take 2.5%."
- "distinct between grammatically correct and incorrect. And grammatically correct also should be split in logically meaningful."
- "optimize our system for top asked questions: https://explodingtopics.com/blog/top-google-questions, and also find lots of similar sources, so we can merge them and re-rank ... We also need to support all variations of such questions."
**Delivered (evidence):**
- Closing PR #638 "Generate, classify, and answer questions via the Agent CLI": lazy infinite `QuestionGenerator` ordered smallest-first, frequency tier policy exactly matching the issue (10%/5%/2.5%, halving), grammar + logical-meaning classification gates, configurable `QuestionAcceptance`, answers delegated to `FormalAiEngine`, eleventh agentic recipe writing `question-catalog.lino`, seed lexicon grounding.
**Verdict:** partially addressed — the generator/classifier/config core is delivered in full, but the PR body makes no claim about ingesting explodingtopics-style "top asked questions" sources, merging/re-ranking multiple popularity sources, or supporting "all variations" of those top questions.
**Deferred/follow-up:** top-questions corpus merging/re-ranking not mentioned anywhere in the PR body — dropped without acknowledgment.

## #529 — Unknown prompt: что было написано в прошлом сообщении? (closed 2026-06-30)
**Requirements (konard):** (issue authored by skulidropek; konard's comment 2026-06-30 09:53 — before PR #597 merged — expands scope)
- "We must support wide range of natural language query to entire memory, not only to dialog ... history."
- "Queries should be fully turing complete, so they should map to substitutions (read + write) as in https://github.com/link-foundation/link-cli."
- "user must have full control over the links (associative memory) of the Formal AI though natural language messages."
**Delivered (evidence):**
- Closing PR #597 "Natural-language whole-memory read+write control (Rust + browser, 4 languages)": previous-message recall fixed in Rust + JS worker (the original bug), whole-memory reads over all MemoryEvent fields, natural-language append (`remember that ...`) and substitution (`replace X with Y in memory`) with occurrence counts, browser persistence via IndexedDB, en/ru/hi/zh coverage.
**Verdict:** partially addressed — the original bug plus whole-memory read and basic append/substitute writes shipped, but "fully turing complete" queries mapping to arbitrary link-cli-style substitutions is claimed only in the loose sense of one append + one single-pattern substitution form; arbitrary/composable query programs and "full control over the links" are not evidenced.
**Deferred/follow-up:** Turing-complete query language beyond the two recognized directive shapes — no follow-up tracked in the PR body.

## #535 — Unknown prompt: Проверь данный текст на уникальность и на плагиат (closed 2026-07-01)
**Requirements (konard):** (issue authored by kogeletey; konard's comment 2026-06-19 sets requirements)
- "We should fully support attached files for this case in Desktop, Telegram bot, and Web app, and in other interface surfaces."
- "We must generalize to all similar requests (the whole class ...) in all languages ... universal meta algorithm and actual recursive reasoning steps, expressed in meta language ... grounded in external data sources. Every finest detail must be tested."
- "fully use github.com/link-foundation/relative-meta-logic for relative statements probability ... take into account all trusted sources ... ignore any unoriginal content or reposting."
- "use our web search to check for each statement in the text ... That task must be implemented fully and on multiple examples."
**Delivered (evidence):**
- Closing PR #598 "Grounded multilingual attachment verification with relative-meta-logic": `document_originality_check` handler mirrored Rust/JS, whole verification class (verify/authenticate/fact-check families) in en/ru/hi/zh, relative-meta-logic per-statement priors with trusted-source tiers and ignored reposts, Wikinews registered as original-journalism source, Wikidata grounding for all trigger meanings, grounded meta-algorithm recipe pinned by tests, surfaces: CLI/HTTP + Telegram attachments + Web app; per-statement web-search queries; Playwright regression.
**Verdict:** fully addressed — the PR body maps essentially every sentence of konard's comment to a shipped mechanism, including the recipe and grounding demands.
**Deferred/follow-up:** none noted (VS Code surface not explicitly named, but "all surfaces" claim covers CLI/HTTP/Telegram/Web).

## #538 — Make our meanings and words more detailed (closed 2026-07-02)
**Requirements (konard):**
- "We need to make much more detailed descriptions of both meanings and words" (singular/plural, composition, bidirectional word↔meaning references).
- "we should have CST/AST of all our Rust logic (meta algorithm) in our data ... we should be able to rebuild Rust logic on demand from full CST/AST ... generated mermaid diagram split into parts."
- "make interactive debugging view ... using embedded VS Code ... split view with chat, data, mermaid diagram, rust version ..., javascript version."
- "you don't read or edit code or files yourself, you only use Agent CLI with Formal AI server connected to do it ... as the result we should get json file with Agent CLI session that fully solved this exact task."
- "find all previously hardcoded strings in the code"; "JavaScript should solve only interfacing with the UI ... compiled at build time from Rust to JavaScript"; "meta algorithm should be fully universal, should reason about itself"; "everything is totally done."
**Delivered (evidence):**
- Closing PR #601 "Make our meanings and words more detailed (grammatical number, part of speech, Wikidata grounding)": Agent-CLI-driven authorship with 4 committed session JSONs and clean-copy reproduction script, enriched tomato/potato meanings with grammatical number/POS grounded in Wikidata lexemes, generated mermaid recipe diagrams, self-AST census of ONE module (`planner.rs`) in `data/meta/self-ast.lino`, real Agent CLI ↔ formal-ai E2E in CI, CONTRIBUTING rules added.
- PR body's own "Requirement status" section explicitly lists NOT built: hardcoded-string audit (R10), Rust→WASM worker widening (R11/R12), CST/AST → Rust round-trip (R14), interactive debug view (R17), universal meta algorithm (R18/R19), automated contradiction detection (R21).
**Verdict:** partially addressed — the PR itself declares six requirement axes "not yet built in this PR" despite the issue's "everything is totally done / no deferral" instruction; konard later confirmed the shortfall by opening #558 ("deeply analyze ... why my requirements from issues/538 were not fully delivered").
**Deferred/follow-up:** R10 hardcoded-string audit, R11/R12 WASM-worker/minimal-JS, R14 CST/AST→Rust rebuild, R17 interactive debug view (embedded VS Code), R18/R19 universal self-reasoning meta algorithm, R21 contradiction detection — all named as unbuilt in the PR body; self-AST covers only one module, not "all our Rust logic".

## #540 — Dreaming (closed 2026-07-12)
**Requirements (konard):**
- "Restructure deduplication based on recalculated frequencies of use ... day dreaming is turned on ... in the background ... lowest possible priority."
- "on desktop it should be possible to activate dreaming daemon ... find patterns, regularities, trends, laws."
- "garbage collection, so there always for example at least 20% of free memory / space, so we selectively forget some things, that can be refetched ... recomputed."
- "use that time for generalizing our algorithms to forget about specific algorithms ... as soon as [the generalized algorithm] is working, specific algorithm can be forgotten."
- "apply fully this task also here .../issues/494. So the resulting pull request will close both tasks."
**Delivered (evidence):**
- Closing PR #645 "Complete verified idle dreaming and memory cleanup": amendments injected into solving context, chat exchanges recorded to memory log, replay-verified generalization with revocation, usage/priority eviction fallback, pattern mining synthesizing new trials, dreaming default-on after 60s idle at min OS thread priority in core/Telegram/desktop, atomic locked writes, multilingual lexicon grounding; addresses a deep review point-by-point (R540-29..33); "Closes #540, Closes #494".
**Verdict:** fully addressed — with two soft spots: the explicit "at least 20% free space" GC target and "forget the specific algorithm once the generalized one works" are only approximated by eviction/refinement mechanics in the PR body, not claimed verbatim.
**Deferred/follow-up:** none noted (review-amended acceptance criteria doc committed in the case study).

## #541 — Missing UI/UX improvements (closed 2026-06-20)
**Requirements (konard):**
- "Not all UI elements has correctly applied theme, may be we should migrate to chakra-ui.com."
- "`Docker unavailable` is wrong, because it is actually available."
- "Previous desktop conversation were deleted ... migrate our data to new form ... (that is critical feature, and must be done right)."
- "Demo mode should no[t] delete or overwrite any user conversations."
- Reasoning animation budget (default 2s, setting with 0 = immediate), reveal order (steps then body), collapsed preview showing at least one full step, human-readable hierarchical reasoning steps at 50% detail.
- "the message for granting permissions should also include button to grant all permissions and switch to agent mode ... actually evaluate pending task for execution."
**Delivered (evidence):**
- Closing PR #542 "Missing UI/UX improvements (#541)": R1–R10 table claiming every requirement — theme overrides, `docker-detect.cjs` PATH probing, non-destructive data migration + pinned userData path, isolated demo conversation, `minMessageAnimationMs` (default 2s), 72/28 reveal split, collapsed-preview CSS, naturalized reasoning steps, grant-all CTA that replays the pending command, case study. Chakra migration explicitly **rejected** with reasoning ("Notes for review": app is no-build hyperscript; the bug was missing overrides, not the wrong system).
**Verdict:** fully addressed — all nine concrete defects/features have per-requirement claims and tests; the Chakra suggestion (phrased as "may be") was consciously rejected rather than ignored, and konard re-asserted it in #550 where it was then delivered.
**Deferred/follow-up:** Chakra UI migration rejected in PR #542 notes (later demanded and shipped via #550/PR #551).

## #543 — Unknown prompt: Что находится в моей папке? (closed 2026-07-03)
**Requirements (konard):**
- Auto-filed report: the desktop prompt "Что находится в моей папке?" returned `intent: unknown` — implicitly it should route to a folder-listing/agent action.
**Delivered (evidence):**
- No closing PR, no comments, no cross-references in the timeline; closed manually by konard on 2026-07-03. Sibling issue #546 (closed 2026-06-20 via PR #547) said "See sub issues, and we should make a pull request, that will solve them all at once" — but #547 does not reference #543 and its body claims only host-shell routing, not natural-language folder-question intent.
**Verdict:** can't tell — closed by hand two weeks after PR #547 with zero recorded evidence that the "what's in my folder" phrasing was ever made to work.
**Deferred/follow-up:** none noted; the natural-language folder-content variants have no traced fix.

## #544 — Unknown prompt: Что в моей папке? (closed 2026-07-03)
**Requirements (konard):**
- Auto-filed report: "Что в моей папке?" returned `intent: unknown`.
**Delivered (evidence):**
- Identical situation to #543: no closing PR, no comments, no cross-references; manually closed by konard 2026-07-03.
**Verdict:** can't tell — no evidence in the thread or any PR body that this variant was fixed.
**Deferred/follow-up:** none noted.

## #545 — Unknown prompt: Покажи что в моей папке (closed 2026-07-03)
**Requirements (konard):**
- Auto-filed report: "Покажи что в моей папке" returned `intent: unknown`.
**Delivered (evidence):**
- Identical to #543/#544: no closing PR, no comments, no cross-references; manually closed by konard 2026-07-03.
**Verdict:** can't tell — nothing on record shows a fix; likely bulk-closed alongside its siblings.
**Deferred/follow-up:** none noted.

## #546 — Issue with dialog: Выполни в терминале `ls ~` (closed 2026-06-20)
**Requirements (konard):**
- "By default `ls ~` is expected to be executed on host machine, not in docker, and we need to be able to use both."
- "make sure to use https://github.com/link-foundation/start for commands execution, as well as https://github.com/link-foundation/command-stream, if any features are missing - report them, and do temporary workarounds."
- "We also should support all variations of this request by using meanings, words and so on."
- "See sub issues, and we should make a pull request, that will solve them all at once in a single pull request."
**Delivered (evidence):**
- Closing PR #547 "Run desktop shell tool on host by default": host-shell default in Electron + VS Code, Docker still available via `input.isolation = "docker"`, permission copy/seed/docs updated, case study. Explicitly states "No upstream issue was filed for link-foundation/start or command-stream because the investigation found an integration policy bug in this repo, not a confirmed upstream defect."
**Verdict:** partially addressed — the host-default/docker-optional core is delivered, but the instruction to actually *use* `start`/`command-stream` for execution is not claimed anywhere (only the decision not to file issues), the "all variations via meanings/words" ask has no claim, and the sub-issues (#543–#545) were not solved by this PR despite "solve them all at once".
**Deferred/follow-up:** adoption of link-foundation/start + command-stream — neither used nor tracked; sub-issue variants left to manual closure.

## #548 — Auto update mechanism for Desktop app (closed 2026-06-21)
**Requirements (konard):**
- "as soon as new version is released user is notified about that inside the app and when user presses update actual update happening."
- "Auto update must support all platforms - macOS, Linux, Windows."
- "fix version display in desktop app, as right now it only displays `vdev`."
**Delivered (evidence):**
- Closing PR #549 "Add desktop auto-update flow": `electron-updater` controller + preload IPC + renderer controls, version badge fixed via `app.getVersion()`, Electron Builder update metadata (`latest*.yml`, blockmaps) published for macOS/Linux/Windows, case study.
**Verdict:** fully addressed — all three asks have direct claims; one platform caveat disclosed.
**Deferred/follow-up:** PR "Notes": "macOS auto-update still depends on signed/notarized production builds" — i.e. macOS updates don't work until signing is in place.

## #550 — Unexpected UI/UX behavior (closed 2026-06-21)
**Requirements (konard):**
- Five defects: per-line thinking-fade gradient, clipped thinking steps, pending-message width jump, `services` box theming, partial top-bar hover.
- "must fully transition to https://chakra-ui.com and JSX, so we can ensure everything is nice and polished" + "if the issue is one place it should be fixed in all places."
- (konard's follow-up comment posted the full root-cause analysis and initially framed Chakra as a staged multi-PR migration.)
**Delivered (evidence):**
- Closing PR #551 "fix(web): five UI/UX polish fixes + full Chakra UI / JSX migration": all five defects fixed (P1–P5 table, both runtimes / both dark layers), plus — after the maintainer rejected deferral ("Nothing can be deferred or delayed to other pull requests") — a completed JSX + bun-bundler + ChakraProvider migration, with an explicit retraction of the earlier "CSP-blocked" claim; 332 Playwright tests green.
**Verdict:** fully addressed — the PR initially tried to defer the Chakra migration, konard pushed back mid-PR, and the final merged body claims the migration shipped in full.
**Deferred/follow-up:** none noted in the final body (the earlier staged-migration plan was superseded).

## #552 — Automated testing scripts/CLI commands (closed 2026-06-21)
**Requirements (konard):**
- "add automated script for conversion of [the ChatGPT share link] into our demo format ... able to answer not worse than ChatGPT did."
- "We should also do the same for https://share.google/aimode/VG0HhpnAXrBkC0QgP."
- "find similar links (crawl them on any target website), for example ... GitHub regexp search, take 100 of them, and out 100, find 10 shortest, and ensure our system is able to solve them all by thinking, generalizing."
- "ask web-capture to add support for meta-language as a basis for parsed documents base object model ... the pull request will be placed on pause once all issues are reported."
**Delivered (evidence):**
- Closing PR #553 "Add shared dialog replay conversion": `formal-ai shared-dialog convert` for ChatGPT share HTML/Markdown, multi-line memory preservation, solver coverage for the shell-loop/screen answers in 4 languages, upstream issues filed (web-capture#141, meta-language#168), case study with "public shared-link corpus samples". Google AI Mode capture failed: "could not be converted from a static HTTP capture because the saved response is a Google Search interstitial/challenge" — documented as a gap.
**Verdict:** partially addressed — ChatGPT-share conversion + replay + upstream reporting shipped, but Google AI Mode support did not (capture gap), and the 100-links/10-shortest crawl-and-solve exercise is only weakly evidenced as "corpus samples" with no claim the system solves them.
**Deferred/follow-up:** Google AI Mode conversion blocked on capture (documented in case study + upstream issues); web-capture meta-language support waiting on web-capture#141 / meta-language#168.

## #554 — Make support for VS Code extension more user friendly as well as other interfaces (closed 2026-06-21)
**Requirements (konard):**
- "separate page on our landing page for VS Code extension, with detailed instructions."
- "ability to install it from already installed Desktop app in one click."
- "the only way to install VS Code extension should be manual, by command downloadable from our repository `curl/wget ... .sh | bash`."
- "universal .sh + power shell installer, for all Desktop app, VS Code and all other interfaces."
- "add pages in landing for telegram bot and CLI."
**Delivered (evidence):**
- Closing PR #555: R1–R7 table with every requirement marked ✅ — `/vscode/`, `/cli/`, `/telegram/` landing pages, `scripts/install.sh` + `install.ps1` universal installers, one-click desktop VS Code install (`vscode-install.cjs`, 15 test cases), release workflow uploads the `.vsix`, case study; 11-case site spec across 4 languages.
**Verdict:** fully addressed — one-to-one requirement table with tests for each item.
**Deferred/follow-up:** none noted.

## #556 — Unknown prompt: я не понимаю по английски, напиши по русски (closed 2026-07-01)
**Requirements (konard):** (issue authored by netkeep80; konard's comment 2026-06-30 — day before close — sets requirements)
- "We must generalize to all similar requests (the whole class ...) in all languages ... universal meta algorithm and actual recursive reasoning steps, expressed in meta language ... grounded in external data sources. Every finest detail must be tested."
- "translation to meta language (with full meaning preservation) and translation from meta language must actually complete full cycle of translation."
- "use https://github.com/link-assistant/formal-ai/issues/526 principle for all translation testing."
**Delivered (evidence):**
- Closing PR #599 "Generalize response-language follow-ups to the whole request class": whole-solver replay with a single forced-language seam (Rust + JS), seed-grounded triggers with Wikidata grounding, grounded meta-algorithm recipe pinned by 8 tests, issue-#526 round-trip test file, and an explicit "Issue #556 requirements → evidence" table mapping each of konard's sentences to code/tests; 1245 unit tests green.
**Verdict:** fully addressed — the PR body answers konard's comment requirement-by-requirement, including the #526 round-trip principle.
**Deferred/follow-up:** none noted.

## #558 — Auto learning (closed 2026-07-05)
**Requirements (konard):**
- "our meta algorithm is advanced enough to actually recompile itself ... enabling fully dynamic self programming or self learning."
- "translate entire source code of our system to links/meta language ... present in the seed data ... translate that meta language representation back to the source code, recompile and reattach it to the UI."
- "user should be able to ask to introduce any changes in our AI system that way, and also any questions about how formal AI itself works can be answered."
- "deeply analyze what went wrong at .../pull/601, and why my requirements from .../issues/538 were not fully delivered."
**Delivered (evidence):**
- Closing PR #637 "complete human-gated auto-learning loop (R558-01…R558-06)": eleven deterministic agentic recipes; self-healing repair cases, repair-strategy classifier, learning ledger (green tests + human approval), whole-repo source→links projection with byte-for-byte round-trip (`coverage_permille == 1000`), self-explanation and change-request recipes, and `RebuildPlan` (recompile → regenerate worker → reattach → hot-swap → verify). Design guardrails: "Human-gated & proposal-only. Nothing is ever auto-applied ... Nothing is rebuilt or restarted: the plan is the reviewable product."
**Verdict:** partially addressed — all six formalized requirements are claimed implemented, but the headline ask ("actually recompile itself", "reattach it to the UI", "fully dynamic self programming") was deliberately reduced to deterministic *plans/proposals* that never execute a rebuild; live self-recompilation does not happen, by design.
**Deferred/follow-up:** actual execution of rebuild/reattach (vs. emitting a reviewable plan) is permanently out of scope per the PR's guardrails; source-in-seed is embedded via `build.rs`, not shipped as seed data per se.

## #559 — Generalize meta algorithm (closed 2026-06-24)
**Requirements (konard):**
- "hardcoded intents are dead end ... translate message to meta language and work on it directly, on conversion detect questions, requirements, needs ... address all of the[m] in our response."
- "If it is a task with big plan ... use todo tool in agentic mode ... In chat mode, we should give meaningful answer, using fresh data gathered in the internet."
- "merge all of our specific algorithms into single general meta algorithm ... all already supported test cases should be greatly expanded ... support entire class of each tasks."
- "the first working session should be dedicated to detailed planing ... After that I either approve the plan, or will ask to change it."
**Delivered (evidence):**
- Closing PR #560 "Issue 559: registry-backed recursive meta algorithm": link-native recursive meta core (problem frames, decomposition, need ledger, evidence, recipes) for R330-R344; method registry becomes the **sole dispatch authority**, legacy specialized mapper removed with a corpus-wide parity proof; per-leaf method-selection trace; 1144 unit tests; reproducible example artifact.
**Verdict:** partially addressed — the architectural merge into one registry-backed meta algorithm is delivered and proven behavior-preserving, but the PR body claims nothing about the todo-tool agentic planning path, chat-mode answers "using fresh data gathered in the internet", or the demanded class-level *expansion* of every previously-supported test category (parity ≠ expansion); no visible plan-approval checkpoint with konard either.
**Deferred/follow-up:** self-modification of the meta algorithm ("reason about itself and modify itself") acknowledged in the issue as a later stage; class-expansion of all existing test categories not tracked in the PR body.

## #561 — Versions are not in sync and other CI/CD errors (closed 2026-06-24)
**Requirements (konard):**
- "All versions on all releases must be equal. Double check all other false positives and errors in CI/CD."
- "Use all the best practices from CI/CD templates ... if the same issue is found in template report issue also in templates."
- Case-study folder with downloaded logs, root causes, debug output if needed.
**Delivered (evidence):**
- Closing PR #562 "fix: deploy Pages from the release commit": root cause (Pages deployed from the pre-release SHA still advertising 0.217.0), release SHA now passed to the Pages job and the live E2E wait, plus a retrying `install-rust-script.sh` wrapper for a reproduced transient crates.io failure, workflow regression tests, case study with downloaded CI logs and template comparison ("No matching upstream template defect was found, so I did not open template issues").
**Verdict:** fully addressed — version-sync root cause fixed at the pipeline level, secondary CI flake fixed, template comparison and case study delivered.
**Deferred/follow-up:** none noted.


# Chunk ae analysis — issues 563..680 (26 issues)

## #563 — Support summarization of any file in the repository (closed 2026-06-25)
**Requirements (konard):**
- "take random file of our repository and being able to summarize it correctly"
- "We first take 2 random files and do summarization for them manually, after that we teach our system by generalizing the algorithm... we should repeat iterations until we will actually have 2-3 times similar summarization" (iterative random-file validation methodology)
- "make sure each file is fully formalized as meta language (for markdown files recursively with multiple embedded grammars)"
- "Summarization itself must be a part of meta algorithm... solved using recursive reasoning steps"; "We should test for each and every detail I described"
- Case-study folder `./docs/case-studies/issue-563`; "at least 80% perfect" quality bar
**Delivered (evidence):**
- Closing PR #564 "Support repository resource summarization (files and folders)": generalizes summarization from files to any repo resource via decompose→summarize→compose recursion; new `src/summarization/resource.rs`; tests; case-study README + REQUIREMENTS rows R345–R359. No mention of the iterative random-file methodology, markdown multi-embedded-grammar formalization, or the 80% quality measurement.
**Verdict:** partially addressed — the file/folder summarization surface and case study shipped, but the prescribed iterative random-file generalization loop and recursive-embedded-grammar markdown formalization are not evidenced in the PR body.
**Deferred/follow-up:** none noted (the un-evidenced methodology items were silently omitted rather than explicitly deferred).

## #571 — Solve entire class of similar questions by reasoning and logic in meta algorithm (closed 2026-07-03)
**Requirements (konard):**
- Body is a Russian subscription-pricing example; title asks to "solve entire class of similar questions by using reasoning and logic in our meta algorithm"
- Comment (2026-07-03, before merge): "The scope of this task is full repository and full roadmap. We must support the entire class of similar requests."
**Delivered (evidence):**
- Closing PR #618 "Solve the whole class of external-entity questions by reasoning (#571)": structural reasoning rule (`extract_externally_verifiable_question`) routing interrogative prompts containing interior-capitalized Latin brand tokens (ChatGPT, iPhone…) to web research, en/ru/hi/zh; PR notes the maintainer flagged a seed-vocabulary-only fix as too narrow; tests across languages/topics.
**Verdict:** partially addressed — genuine generalization beyond the single example, but the "class" is bounded by an orthographic heuristic; plain proper nouns (Claude, Tesla, Wikipedia) deliberately do NOT match, so many similar external-entity questions remain outside the rule.
**Deferred/follow-up:** PR body itself documents that plain-capitalized names intentionally don't trigger the rule (design limitation, no follow-up issue).

## #595 — Calendar events (closed 2026-06-30)
**Requirements (konard):**
- "All sub issues must be implemented in a single pull request, and all of them must be closed, when pull request is merged."
**Delivered (evidence):**
- Closing PR #596 "Handle Russian calendar spoken-hour prompts": recognizes `на 10 часов` spoken-hour wording, routes elliptical/reordered meeting prompts to `calendar_create_event` in Rust solver and browser worker; body has "Fixes #595 #502 #503 #504 #522" — the sub-issues were closed by the single PR as demanded.
**Verdict:** fully addressed — single PR implemented and closed the sub-issues per the umbrella requirement; no complaints in thread.
**Deferred/follow-up:** none noted.

## #600 — Unknown prompt: а я что спрашивал? (closed 2026-07-03)
**Requirements (konard):**
- Issue authored by skulidropek (bug: "а я что спрашивал?" hit the unknown fallback). konard comment (2026-07-03): "The scope of this task is full repository and full roadmap. We must support the entire class of similar requests."
**Delivered (evidence):**
- Closing PR #617 "Fix previous-user question recall": seed-driven `conversation_recall_previous_user_message` role in en/ru/hi/zh; routes "what did I ask?" prompts before generic previous-message replay; skips recall/meta turns so the original substantive request is returned; unit + e2e tests.
**Verdict:** fully addressed — the reported failure and its multilingual class of "what did I ask" recall prompts are covered.
**Deferred/follow-up:** none noted.

## #602 — No SSE streaming on /v1/responses (Codex CLI cannot drive server) (closed 2026-07-03)
**Requirements (konard):**
- "`POST /v1/responses` with `stream: true` returns `content-type: text/event-stream` and emits the Responses SSE event sequence ending in `response.completed`"
- "`POST /v1/chat/completions` with `stream: true` returns an SSE stream of `chat.completion.chunk` events"
- "The reproduction command above (`codex exec \"hi\"`) completes and prints `Hi, how may I help you?`... exit 0"
- "Codex no longer emits the `failed to refresh available models` error for our `/v1/models`"
- "An automated end-to-end test drives the server over its real HTTP transport with a streaming client (ideally Codex itself...)"
- Follow-up comment (2026-07-02, before merge) ADDED: "clear, copy-pasteable documentation... for how to point the Codex CLI at Formal AI end to end, verified to print a reply"
**Delivered (evidence):**
- Closing PR #614 "Fix Codex Responses streaming compatibility": real loopback HTTP test for `/v1/responses` stream asserting Responses SSE events ending in `response.completed`; copy-paste Codex 0.142+ docs incl. `codex exec "hi"`; changelog. No mention of the `/v1/models` metadata fix; no real-Codex run (loopback SSE client only); Chat SSE handled separately (#604/PR #610).
**Verdict:** partially addressed — Responses SSE + Codex docs delivered, but the `/v1/models` codex metadata acceptance item was not fixed here (resurfaced as `missing field slug` in #626, fixed only by PR #630), and the e2e test used a minimal SSE client rather than Codex itself; #650 later showed a real `with codex "hi"` run still misrouted.
**Deferred/follow-up:** models-metadata gap effectively deferred to #626/PR #630; real-codex misrouting surfaced later in #650 (no explicit deferral in PR #614's body).

## #603 — Universal multi-protocol API server (OpenAI/Anthropic/Gemini/Vertex under /api/<protocol>/) (closed 2026-07-03)
**Requirements (konard):**
- "The server exposes OpenAI, Anthropic, Gemini, and Vertex request/response shapes, each under its own `/api/<protocol>/...` prefix"; `/v1` kept as deprecated alias
- "Streaming works correctly for every protocol... (verified against a real client per protocol)"
- "`codex exec \"hi\"` completes end-to-end"; "A Gemini/Vertex-shaped client (SDK or compatible CLI) completes `hi` against the native endpoint"
- "Per-protocol model-listing shapes are correct; no client emits a model-metadata error"; "End-to-end transport tests exist in CI for each protocol"
- Comments (2026-07-02, before merge) ADDED: verified per-CLI config docs (codex, opencode, gemini via `GOOGLE_GEMINI_BASE_URL`, vertex), each "verified end-to-end (`hi` returns a reply)"
**Delivered (evidence):**
- Closing PR #612 "Add multi-protocol API gateway": namespaced `/api/openai/v1`, `/api/anthropic/v1`, `/api/gemini/v1beta`, `/api/vertex/v1`, `/api/formal-ai/v1`; `/v1/*` aliases kept; Gemini/Vertex `generateContent` adapters; per-protocol model discovery; Responses SSE fix; documented Codex/OpenCode/Claude Code/Gemini/Vertex client configs; loopback integration test.
**Verdict:** partially addressed — protocol namespace, adapters, aliases and docs shipped, but verification was via loopback tests, not "a real client per protocol": real gemini-cli failed by default right after (#620), real codex still had tool/metadata failures (#626) and `hi` misrouting (#650), so the "no client emits a model-metadata error" and real-client acceptance items were not met at close.
**Deferred/follow-up:** real-CLI gaps landed as follow-up issues #620/#621/#622/#626/#650 (not flagged as deferred in PR #612's body).

## #604 — OpenAI Chat Completions streaming is malformed (breaks opencode & AI SDK clients) (closed 2026-07-02)
**Requirements (konard):**
- "`POST /v1/chat/completions` with `stream:true` returns `object: \"chat.completion.chunk\"` events with `choices[].delta.content`, terminated by `data: [DONE]`"
- "`opencode run -m formalai/... \"hi\"` prints `Hi, how may I help you?`... non-empty"
- "An automated end-to-end test drives `/v1/chat/completions` with `stream:true` over real HTTP"
- "Documentation for configuring opencode... against Formal AI is added and verified"
**Delivered (evidence):**
- Closing PR #610 "Fix OpenAI chat completion streaming compatibility": real loopback HTTP regression test asserting `chat.completion.chunk`/`delta` shape, role-first chunk, stop delta, usage chunk, `[DONE]`; updated OpenCode docs with the verified provider/model selector and `opencode run ... "hi"` smoke test.
**Verdict:** fully addressed — all four acceptance items matched by explicit PR claims; no post-merge complaints on this issue.
**Deferred/follow-up:** none noted (docs still used the pre-rename `formal-symbolic-production` id, superseded by #605 the same day).

## #605 — Model id should be formal-ai (rename), accept @link-assistant/formal-ai aliases (closed 2026-07-02)
**Requirements (konard):**
- "The canonical model id is `formal-ai`; `DEFAULT_MODEL == \"formal-ai\"`"; "`formal-symbolic-production` is removed — not advertised... and not surfaced to users anywhere"
- Aliases `@link-assistant/formal-ai`, `link-assistant/formal-ai`, `formal-ai-latest`/`latest`, case-insensitive, "driven by seed data, not a hardcoded match"
- "codebase, seed data, tests, and docs no longer reference formal-symbolic-production as a user-facing model name"; tests per alias; verified docs
**Delivered (evidence):**
- Closing PR #611 "Fix formal-ai model id and aliases": renames canonical id across API/docs/desktop/web/tests; seed-backed `data/seed/model-aliases.lino` resolver; `/v1/models` advertises only `formal-ai`; all listed aliases accepted case-insensitively; `rg "formal-symbolic-production"` shows no matches; extensive verification list.
**Verdict:** fully addressed — every acceptance criterion explicitly claimed with matching evidence (including the repo-wide grep).
**Deferred/follow-up:** none noted (rust-script helper checks left to CI per PR note).

## #606 — with-formal-ai: CLI helper to run/preconfigure codex, opencode, gemini (with -g/--global) (closed 2026-07-03)
**Requirements (konard):**
- "`with-formal-ai codex \"hi\"` / `opencode run \"hi\"` / `gemini -p \"hi\"` each print a Formal AI reply with zero manual config"
- "`-g` is idempotent, backs up prior config, and `--undo` restores it"; "`--start-server` launches `formal-ai serve` when nothing is listening"
- "Per-tool integration templates are seed data, not hardcoded"; docs page for every supported tool; uses the `formal-ai` alias
**Delivered (evidence):**
- Closing PR #615 "Add with-formal-ai wrapper for external CLIs": `formal-ai with` + standalone binary; templates in `data/seed/client-integrations.lino`; ephemeral runs, `--start-server`, `--base-url/--port/--protocol/--model`, `-g/--all/--undo` with backups and merge-preserving writes; README + desktop docs; integration tests.
**Verdict:** partially addressed — wrapper, seed templates, global/undo flows and docs shipped, but the headline acceptance "each tool prints a reply with zero manual config" did not actually hold: gemini failed by default on machines with cached OAuth (#620) and codex failed outside git repos (#626), both filed by konard the next day.
**Deferred/follow-up:** gemini default failure → #620; agent CLI missing from wrapper → #621; README overstatement → #622; codex sandbox flags → #626 (all follow-up issues, none flagged in PR #615's body).

## #607 — Agent CLI cannot run shell commands (ls): server never emits tool_calls for bash (closed 2026-07-02)
**Requirements (konard):**
- "`agent -p \"run ls...\"` results in a `bash`/shell `tool_calls` with `command: \"ls\"`, the CLI runs it, and the file list is summarized back"
- "server emits `tool_calls` (not prose) for a natural-language shell request"; "`bash` and `shell` tool names resolve to the shell/run-command capability"
- "Shell execution respects `--read-only` / `permission-mode`"; "A per-request / config way to enable agent tool execution exists (not only the FORMAL_AI_AGENT_MODE server env)"
- "End-to-end test: Agent CLI drives Formal AI to run `ls` and report the listing"; copy-pasteable docs
**Delivered (evidence):**
- Closing PR #609 "Emit Agent CLI shell tool calls for ls requests": routes NL `ls`/current-directory prompts to `bash`/`shell`/`run_command` tool calls in agent mode; adds `formal-ai serve --agent-mode` CLI opt-in; documents Agent CLI shell execution with `--permission-mode plan` and `--read-only`; unit tests only (`issue_607`, `agentic_coding`, `openai_compatibility`).
**Verdict:** partially addressed — routing, name mapping, `--agent-mode` opt-in and docs shipped, but no real Agent-CLI end-to-end test as the acceptance demanded (unit tests only), and konard's later testing showed NL phrasings still failed 4/10 (#624) and file reads 0/12 (#627).
**Deferred/follow-up:** NL phrasing coverage and read-tool routing effectively deferred to #624/#627 (not stated in PR #609's body).

## #608 — Expose reasoning/thinking in standard fields + stream it (closed 2026-07-03)
**Requirements (konard):**
- "OpenAI Chat responses include `reasoning_content` (non-streaming) and stream `delta.reasoning_content`"
- "OpenAI Responses stream reasoning summary events"; "Anthropic `/v1/messages` emits a `thinking` content block (and streamed `thinking_delta`)"
- Proposed: "Gemini (per #603 S4): map thinking onto Gemini's `thought`-part convention"
- "A thinking-capable CLI... driving Formal AI visibly displays the reasoning"; respects diagnostic/thinking-level config; e2e tests per protocol
**Delivered (evidence):**
- Closing PR #613 "Expose reasoning through standard API fields": OpenAI Chat `reasoning_content`/`reasoning` + streaming deltas; Responses reasoning output items + `response.reasoning_summary_*` SSE; shared renderer with Anthropic's existing opt-in `thinking` behavior; documents "copy-pasteable Chat, Responses, Anthropic, and native CLI reasoning checks"; unit tests.
**Verdict:** partially addressed — OpenAI Chat/Responses and Anthropic channels delivered with tests, but the Gemini `thought`-part mapping is absent from the PR, and no live thinking-capable CLI display was demonstrated (docs "checks" only).
**Deferred/follow-up:** Gemini thought parts silently omitted (no explicit deferral); human-quality thinking display resurfaced later as a konard ask in #676 (R8).

## #620 — with-formal-ai: gemini fails by default (cached OAuth overrides wrapper env) (closed 2026-07-03)
**Requirements (konard):**
- "Force API-key auth so a pre-existing OAuth login can't hijack it — export `GEMINI_DEFAULT_AUTH_TYPE=gemini-api-key`"; isolate from `~/.gemini`
- "The wrapper should set `GEMINI_CLI_TRUST_WORKSPACE=true` for the ephemeral run"; "re-verify `with-formal-ai gemini -p hi` returns the greeting on a machine that has an existing Gemini OAuth login"
- Comment (2026-07-03, after work started) added a finding, explicitly scoped out: headless gemini `-p` advertises no tools — "Worth documenting the limitation"
**Delivered (evidence):**
- Closing PR #623 "Fix with-formal-ai Gemini auth isolation": temporary `GEMINI_CLI_HOME` + generated settings selecting API-key auth; seed-driven auth/trust env (protocol-aware gemini vs vertex); docs + changelog; integration test seeds cached `oauth-personal` before the run.
- konard post-merge comment: "Verified fixed by #623 — keeping this closed ✅" — re-tested against the real Gemini CLI with cached OAuth present; all three gaps addressed.
**Verdict:** fully addressed — author independently re-verified the exact original repro end-to-end and confirmed nothing outstanding.
**Deferred/follow-up:** gemini headless `-p` advertising no tools (chat-only through the wrapper) — explicitly noted by konard as separate/unresolved, tracked only as a comment.

## #621 — with-formal-ai: add support for our own 'agent' CLI (closed 2026-07-03)
**Requirements (konard):**
- "It should be added to `data/seed/client-integrations.lino` so `with-formal-ai agent -p hi` works by default"
- Ephemeral via `LINK_ASSISTANT_AGENT_CONFIG_CONTENT` env (no temp file) + `FORMAL_AI_API_KEY` + `--model formalai/formal-ai`; global target `~/.config/link-assistant-agent/opencode.json`
**Delivered (evidence):**
- Closing PR #629 "Add with-formal-ai support for Agent CLI": adds the `agent` seed integration, inline config injection via `LINK_ASSISTANT_AGENT_CONFIG_CONTENT`, persistent setup/undo for the documented path, docs, changelog; integration test reproduces the original "unsupported tool `agent`" failure then verifies injected model flag/env/config; `-g --all` + undo coverage.
**Verdict:** fully addressed — delivery matches the suggested fix point-for-point.
**Deferred/follow-up:** none noted (upstream cosmetic compaction-log noise filed by konard as link-assistant/agent#275).

## #622 — docs: README overstates gemini/agent/codex with-formal-ai flows (closed 2026-07-04)
**Requirements (konard):**
- Fix three doc mismatches: (1) gemini flow shown as working when it fails on cached OAuth; (2) agent section lacks the `with-formal-ai agent` one-liner once supported; (3) codex manual examples pass `--skip-git-repo-check --sandbox read-only` but the wrapper seed omits both — "Either align the wrapper's codex args with the documented flags, or note the difference"
- Comment (2026-07-03, after work started): "Double check if after all recent changes requirements are still valid, double check docs and update them to correctly reflect the codebase in any case."
**Delivered (evidence):**
- Closing PR #636 "Fix Codex wrapper docs and sandbox flags": aligns the codex wrapper seed by adding `--sandbox read-only` (the `--skip-git-repo-check` half landed in PR #630); updates README/desktop/testing-guide docs to describe current gemini isolation, agent inline config, and codex flags; adds `issue_622_wrapper_docs_match_current_cli_seed_behavior` traceability test.
**Verdict:** fully addressed — all three mismatches resolved (items 1 and 2 via the intervening #623/#629 fixes plus doc refresh here); the "re-verify docs against codebase" comment answered by the doc-pinning test.
**Deferred/follow-up:** none noted (PR flags two unrelated flaky full-suite tests passing in isolation).

## #624 — agent-mode: natural-language file-listing requests return 'could not determine' (4/10 phrasings fail) (closed 2026-07-04)
**Requirements (konard):**
- "When a `tools` array is present and the user's message expresses a shell/file-inspection intent — including indirect phrasings — the server should route to the appropriate tool-call"
- "At minimum, common list-files paraphrases should map to the shell/ls intent" (the four failing phrasings listed)
- Comments (before merge): confirmed CLI-independent across opencode + agent; flagged file reading 0/12 (#627)
**Delivered (evidence):**
- Closing PR #632 "Fix agent-mode natural-language directory listings": routes NL current-directory listing prompts to the same `bash {"command":"ls"}` call; unit + HTTP integration regressions for all four issue phrasings; docs list supported paraphrases.
**Verdict:** partially addressed — the four enumerated paraphrases fixed with tests (the stated minimum), but the general ask ("shell/file-inspection intent, including indirect phrasings") remained phrasing-gated: konard's #680 matrix later measured shell routing at 2/10 phrasings, proving intent-level routing was not delivered here.
**Deferred/follow-up:** general intent-based routing effectively deferred until #680/PR #683; read-tool gap split to #627.

## #625 — Add a built-in Rust logging reverse proxy (formal-ai proxy) + agentic e2e suite in CI (closed 2026-07-04)
**Requirements (konard):**
- "`formal-ai proxy --listen <addr> --upstream <url> [--log <path>] [--body]`... appends one JSON line per exchange" incl. streaming pass-through
- Second headline ask: "use this proxy to grow agentic e2e/integration coverage in CI/CD" — "Drive the real client CLIs (at minimum opencode, plus codex, gemini, and our own agent)... and assert on the behavior, not just exit codes"; phrasing matrices, negative cases, marker assertions; "Run it on every PR"
- Comments (before merge) reinforced: suite must cover all three protocol paths incl. streamed SSE; coverage targets = the #627 12-dialog read matrix + #624 list-files matrix
**Delivered (evidence):**
- Closing PR #631 "Add logging reverse proxy": the `formal-ai proxy` subcommand with JSONL summaries for OpenAI Chat/Responses and Gemini `generateContent`/`streamGenerateContent`, `--body` option; unit + integration tests for the proxy itself. Nothing on the CI e2e suite.
**Verdict:** partially addressed — the proxy (ask 1) shipped fully, but the CI agentic e2e suite driving real CLIs with phrasing matrices and per-detail proxy-log assertions (ask 2, roughly half the issue plus three reinforcing comments) is absent from the PR body.
**Deferred/follow-up:** the CI e2e suite was never explicitly deferred in PR #631; pieces appeared later (real Agent CLI job in PR #677/#679, matrix work in #680), but the on-every-PR multi-CLI matrix as specified was dropped at close.

## #626 — codex tool-calling broken: args 'command' vs 'cmd', /models missing 'slug', non-git-dir hard fail (closed 2026-07-03)
**Requirements (konard):**
- "Server Responses-API tool-calls use the argument field codex expects (`cmd`)"
- "`/api/openai/v1/models` includes the `slug` field (and metadata) codex needs"
- "`with-formal-ai codex` passes `--skip-git-repo-check` so it runs outside git repos"
**Delivered (evidence):**
- Closing PR #630 "Fix Codex Responses compatibility": adapts Responses shell tool-call args to the advertised schema (`cmd` for codex, `command` preserved for command-schema clients); adds `slug` to the models response; adds `--skip-git-repo-check` to the seeded codex invocation.
**Verdict:** fully addressed — all three problems fixed one-for-one per the PR body.
**Deferred/follow-up:** none noted (`--sandbox read-only` alignment landed separately via #622/PR #636).

## #627 — File reading broken in agent mode: 0/12 read dialogs work (closed 2026-07-04)
**Requirements (konard):**
- "the server should return the appropriate tool-call (`read {\"filePath\":...}` or `bash {\"command\":\"cat ...\"}`)" for direct, natural-language, extraction/partial, and multi-step read requests (12-dialog matrix)
- "don't treat local filenames as URLs, and apply the agent-mode gate consistently so `cat` behaves like `ls`"
**Delivered (evidence):**
- Closing PR #633 "Fix agent-mode file read tool routing": deterministic file-read recipe covering local filename prompts, shell-shaped `cat`, list-then-read, nested reads, last-file, first-line, JSON value extraction, read-all summaries; `Read` capability + default agent package grant for `read`/`read_file`/`open_file`; `tests/unit/issue_627.rs` reproduces the original failures incl. the URL fallback for `beta.md`.
**Verdict:** fully addressed — every category of the 12-dialog matrix and both correctness bugs explicitly claimed with reproducing tests (later #680 measured read as the one capability that "mostly works", 33/50).
**Deferred/follow-up:** none noted.

## #628 — docs: add an agentic CLI tools testing guide (closed 2026-07-04)
**Requirements (konard):**
- "A `docs/testing/agentic-cli-tools.md`... capturing the above, with copy-pasteable commands and the fixture/marker convention, cross-referenced from CONTRIBUTING" (setup, per-CLI cheatsheet, proxy provenance, fixtures/markers, phrasing matrices incl. negative cases, results classification, CI wiring)
**Delivered (evidence):**
- Closing PR #634 "Add agentic CLI testing guide": adds the guide covering setup, fixture markers, `formal-ai proxy` provenance, per-CLI invocation notes, phrasing matrices, failure classification, and the CI e2e shape feeding #625; links from CONTRIBUTING; traceability test.
**Verdict:** fully addressed — the deliverable matches the proposed outline section-for-section.
**Deferred/follow-up:** the actual CI e2e suite remains #625's (dropped) scope; the guide only describes its shape.

## #647 — formal-ai with: auto-start temp --agent-mode server + disable summarization + temp-config-only + broaden tool coverage (closed 2026-07-12)
**Requirements (konard):**
- "auto-starting a temporary `--agent-mode` server and tearing it down on exit" with no `--start-server`; reuse an existing listener; `--no-start-server`; one-line security notice
- "Summarization/compaction is disabled by default for wrapped tools that support it (e.g. `agent` gets `--no-summarize-session`); `--summarize` restores it"
- "Non-global runs never modify any tool's persistent config — verified by before/after checksums... across every supported tool"
- "`claude`, `qwen`, `grok` (grok build) are supported as seed-data integrations; `aider` is either supported or explicitly documented as an `OPENAI_API_BASE` recipe"; docs page
**Delivered (evidence):**
- Closing PR #648 "Make formal-ai with zero-config across agent CLIs": idle-port auto-start in agent mode with notice + teardown + reuse + both escape hatches; agent `--no-summarize-session --compaction-model same` with `--summarize`/`--keep-summarization`; temp homes for codex/agent one-shots; seed integrations for Claude Code, Qwen Code, Grok Build, Aider; docs + changelog; automated coverage incl. "unchanged persistent config files for all eight non-global integrations"; a real Agent CLI run shown succeeding.
**Verdict:** fully addressed — every acceptance criterion has a matching explicit claim; konard's follow-up #650 treats #647 as "shipped in 0.278.0... verified working".
**Deferred/follow-up:** tools without a summarization-disable flag (codex/gemini/qwen) left open — raised by konard as #650 item 3; claude path untested locally by design (issue footnote).

## #649 — Predicting consequences of actions using world models/formal systems/contexts (closed 2026-07-13)
**Requirements (konard):**
- "we should not only build the meaning representation of dialogue itself, but we also need to build a representation of current state of the world... and the target state of the world that user want to have"
- "user can always synchronize the understanding of the target with the agent and vice versa"
- "we should be able to merge or split world models (contexts) as needed"; "Each context is always a links network"; use relative-meta-logic with dependent statements whose "probabilities are recalculated" on change
- Case-study folder `./docs/case-studies/issue-649`; "until it is each and every requirement fully addressed, and everything is totally done"
**Delivered (evidence):**
- Closing PR #675 "Issue #649: symbolic world models & contexts": new `src/world_model.rs` (Context as links network + dependent statements, WorldModel current/target/general, STRIPS-style Action, difference, predict-on-clone, JTMS-style recalc fixpoint, merge/split/commit); 10 unit tests; deep cited case study. PR body's own requirement status: "Of the 14 conceptual requirements: 9 realized... 4 partial (pipeline-integration steps: seed current from the append-only log, build target from IntentFormalization, route 'I want…' into target edits, render via self_explanation), 1 proposed (the agent⇄user target-synchronization loop)."
**Verdict:** partially addressed — the PR itself declares 4 requirements partial and 1 only proposed, against konard's explicit "each and every requirement fully addressed" instruction; the world model is not wired into the live dialogue pipeline and the user⇄agent target-sync loop doesn't exist.
**Deferred/follow-up:** the 4 pipeline-integration steps and the target-synchronization loop, per PR #675's requirement-status section and `docs/case-studies/issue-649/requirements.md`.

## #650 — formal-ai with: fix codex 'hi' misroute, uniform interactive mode, summarization handling, --globally alias (closed 2026-07-12)
**Requirements (konard):**
- "`formal-ai with codex \"hi\"` returns a greeting (Responses `instructions` no longer merged into the query...)"
- "`formal-ai with <tool>` with no message enters interactive mode for every supported tool"; "`--interactive` / `--non-interactive` force the mode uniformly... (mapped to each tool's native flag via seed data)"
- "Summarization/compaction disabled by default where the tool supports it; where it doesn't, the server returns a valid summary for summarize/compact requests instead of failing"
- "`--globally` is accepted"; "Non-global runs never write persistent config for any tool (regression test, incl. the new interactive/PTY path)"
**Delivered (evidence):**
- Closing PR #653 "Fix consistent formal-ai with behavior across tools": isolates the latest Responses user turn from `instructions` (codex `hi` fix); seed-driven interactive/one-shot mappings for all eight tools with uniform `--interactive`/`--non-interactive`/`--print`; inline conversation compaction handled through the summarization pipeline; `--globally` accepted; all-tools integration test verifies byte-identical persistent config; real Agent CLI e2e run captured in `docs/case-studies/issue-650/agent-cli-e2e-run.log`.
**Verdict:** fully addressed — all four defects have matching explicit claims and tests; interactive behavior for claude/grok/aider asserted via seed mappings/tests rather than live runs (konard's own footnote said those weren't locally testable).
**Deferred/follow-up:** live validation of claude/grok/aider "should be validated when testable" (issue footnote); not contradicted since.

## #654 — E35: Generalize the agentic planner beyond pinned task recipes (closed 2026-07-13)
**Requirements (konard):**
- "Make the planner compose plans from the requirement decomposition the universal solver already produces... instead of matching one `is_*_task` recognizer" so Formal AI can "take an arbitrary small repository issue end to end"
- Acceptance: `cargo test agentic_general_planner` (non-recipe request → multi-step plan, deterministic); existing recipe tests stay green; "A new fixture `data/meta/general-change-plan.lino` documents the plan shape and is pinned by a specification test"; `test-agent-cli-e2e` gains one general task; ≥3 differently-phrased (en/ru) non-recipe requests produce executable plans
**Delivered (evidence):**
- Closing PR #677 "Generalize agentic planning beyond pinned recipes": deterministic data-shaped fallback "for explicit file-oriented repository changes"; writes the `.lino` plan event before executing; ordered repeated capabilities; rejects ambiguous/absolute/traversal targets; recipes kept ahead as regression fixtures; 7 `agentic_general_planner` tests; three en/ru phrasings; "new real `@link-assistant/agent` job in test-agent-cli-e2e".
**Verdict:** partially addressed — the fallback covers only explicit file-oriented change requests, well short of "an arbitrary small repository issue end to end"; the committed `data/meta/general-change-plan.lino` fixture + specification test from the acceptance list is not mentioned (the PR describes a runtime-written `.formal-ai/general-change-plan.lino` plan instead).
**Deferred/follow-up:** generality beyond file-oriented changes implicitly remains open (the PR's own scoping phrase).

## #655 — E36: Hive-Mind-dispatched end-to-end issue solve by Formal AI (closed 2026-07-13)
**Requirements (konard):**
- "Hive Mind (`solve <issue-url> --tool agent --model formal-ai`) drives the Agent CLI, which drives `formal-ai serve --agent-mode`, and the session takes a small... issue from plan to draft PR"
- Acceptance: committed replayable session under `docs/case-studies/issue-651/self-coding-run/`; byte-for-byte offline replay test; the scratch diff passes its own verification; CONTRIBUTING documents the command sequence
- Thread evidence: konard's automated-solver comment (2026-07-13) shows the Hive Mind dispatch itself failing with "Invalid model name" for `--model formal-ai`
**Delivered (evidence):**
- Closing PR #679 "Add replayable Hive Mind self-coding verification": scratch-repo scenario in `examples/self-coding/`; captured Agent CLI stream events, request traces, plan, deterministic session JSON, verified diff; byte-for-byte replay in the real Agent CLI E2E job; CONTRIBUTING updated. PR body "Upstream gap discovered": "`solve ISSUE_URL --tool agent --model formal-ai` is rejected by Hive Mind 2.5.2... the outer live entry remains gated by that upstream fix" (link-assistant/hive-mind#2059).
**Verdict:** partially addressed — the inner Agent-CLI↔Formal-AI loop, replay, and docs delivered, but the issue's headline scenario (Hive-Mind-dispatched solve) never ran end-to-end; explicitly blocked upstream, so the "claim is unverified" problem the issue opens with is only half-resolved.
**Deferred/follow-up:** outer Hive Mind dispatch deferred to upstream link-assistant/hive-mind#2059 (PR body, "Upstream gap discovered" section).

## #676 — Some commands and prompts are not working in Formal AI, when accessed via OpenCode (closed 2026-07-13)
**Requirements (konard):**
- bash execution incl. `pwd`; "give me a list of files in current folder"; naming; "How are you"; "self healing / debugging / learning does not work"
- "we need to generate as much variations of different kinds of messages as possible, and support correctly them all"
- "check how we do auto learning, and make sure our meta algorithm is able to solve this task itself (partially and fully)"
- "increase quality in thinking display, thoughts should be more human like and less robotic... if needed, we still can show all the robotic like details"
- case study in `./docs/case-studies/issue-676`; debug/verbose output; upstream issues if applicable; apply fixes across entire codebase; single PR, "everything is totally done"
**Delivered (evidence):**
- Closing PR #678: a 13-row requirements table (R1–R13) all marked ✅ — any-shell-token execution (`pwd`, `git status`, …), broader NL file-listing, naming+recall via dialog-local memory, dedicated `wellbeing` intent (en/ru/hi/zh), self-heal on natural repair requests, per-intent first-person thinking narrative with expandable robotic detail (Rust + web, 4 locales), case study + upstream triage ("no upstream bug"), 2094 Rust tests + e2e green.
**Verdict:** fully addressed — every enumerated ask is claimed with dedicated changes and tests; caveat: R6 ("support as many variations as possible") only relatively — konard's #680 matrix the next day showed write/edit/web tools still 0–4/50 across phrasings, i.e. broad tool-intent generality was still missing beyond this issue's specific prompts.
**Deferred/follow-up:** none noted in PR #678; the remaining phrasing-gated tool routing became #680.

## #680 — Agentic CLIs: tool calls are phrasing-gated, not intent-based — web search & web fetch never fire (closed 2026-07-14)
**Requirements (konard):**
- "Route tool calls on intent (the advertised tool set + the request semantics) rather than matching a small set of literal phrasings" for shell, web_search, web_fetch, write, edit — across all five CLIs and all three wire surfaces
- Baseline: web_search 0/50, web_fetch 0/50, edit 0/50, write 4/50, shell 6/50; "only 1/50 write runs actually created the file" (split out to #681 along with the qwen 400 error)
**Delivered (evidence):**
- Closing PR #683 "feat: intent-based tool-call routing in formal-ai serve": all five capabilities route on intent from seed lexicons (`meanings-file-write.lino`, `meanings-file-edit.lino`, `shell-intents.lino`), en/ru/hi/zh, across Chat Completions, Responses, and Gemini `generateContent`; new `Capability::Edit`; `intent_router.rs`; per-capability reproducing unit tests (fail on main) + integration tests over all three surfaces; negative cases guard over-triggering.
**Verdict:** fully addressed — the scoped ask (intent-based emission for the five capabilities on three surfaces) delivered with per-capability reproducing tests; the wrong-tool write→read and qwen wire-error defects were explicitly split to #681 by konard himself.
**Deferred/follow-up:** #681 (write requests emitting `read` tool_calls; qwen 400 `MessageContent` error) — split out in the issue body, not covered by PR #683.


