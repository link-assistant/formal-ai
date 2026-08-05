# Requirement audit — items #311–#465 (link-assistant/formal-ai)

Companion narrative to `req-chunk-311-465.ndjson` (149 requirement entries).
Range covers 2026-05-26 → 2026-08-01. All 71 PRs in range merged; 2 issues still open (#447, #453).
Note on authorship: most PR-thread traffic under the `konard` login is hive-mind automation
(solution-draft logs, auto-merge notices, AI working-session reports). The entries here are
extracted only from konard's genuinely hand-written text; agent self-reports were used solely
to judge `thread_resolution`.

## 1. Recurring themes

1. **Generalization over memoization — the dominant obsession.** Nearly every substantive
   comment restates it: no per-case handlers ("We should not have `sorting handler`"), no
   lookup tables, "universal reasoning algorithms. That is what I require" (#387), "true
   reasoning is when the solution is split or built from smallest possible substitutions
   (rules) that are yet meaningful" (#424). The recurring proof it wasn't achieved: the sort
   modification family regressed three times — reverse (#349), cancel (#386), invert (#427) —
   each requiring a new epic.
2. **The meta-algorithm.** From #412 onward konard escalates from "algorithm builder" to "meta
   algorithm, building algorithm that builds algorithms" (#412, #423, #424, #439, #448), up to
   "a meta algorithm that is able to reproduce our rust code on the topic on demand" (#448).
   Never delivered in full; agent on #413 openly said the cross-handler unification wasn't done
   and asked for scope guidance — **the question was never answered** (see §4).
3. **Semantic foundation / ontology (the #386→#398→#399 arc).** Meanings defined only by other
   meanings, never by single-language text; everything grounded in real external ids
   (Wikidata Q/L/P, Wiktionary, WordNet); lossless JSON→LiNo caches with raw `.json` alongside;
   overrides layer with forced removal when upstream catches up; multi-source merged `view`
   (`M-…` ids, per-field provenance, license-gated source registry); total reference closure
   with CI that fails naming every undefined token. PR #399 took ~10 hand-written review rounds
   in which konard repeatedly caught **fake/green-but-hollow tests** (comment-stripping
   validation, circular round-trip test, closure gate scoped to the subset that passes).
4. **Fake-work detection as a standing posture.** konard repeatedly audits with his own
   scripts/parsers and calls out overclaiming: "My requirements from original task were
   completely ignored" (#399), "My requirement were completely ignored" (#416), "the checks are
   effectively fake" (#399), "requirements are not fully done" (#387). He also models honesty:
   on #399 he publicly retracted his own miscount ("I was wrong about the magnitude — corrected
   and acknowledged") while holding the substantive line.
5. **Component-first + upstream reporting.** Everything must ride dedicated components:
   link-assistant/calculator (#333, #406, #464), web-search/web-capture (#410, #414),
   meta-language for CST/AST and documents (#396, #428, #432), relative-meta-logic for proofs
   (#399, #403), doublets-rs/doublets-web as the store (#331, #399), link-cli transactions
   (#395), agent CLI as a model host (#439). Missing features → file upstream issues, sometimes
   pause the PR until upstream ships (#432). On #418 he caught the agent NOT filing the promised
   calculator issue.
6. **Multilingual equality.** en/ru/hi/zh everywhere (E33 #326, calendar #404/#419, coreference
   #357); response language = last message's language by default (#324, #394); terminology rule:
   "links rules", not "links notation rules" (#394); "links network, not graph" (#398).
7. **Benchmarks + ratchets.** From E32 (#317) pass-count floors, through #362 (download-on-test
   datasets, never committed), to the #416 escalation: pass ≥10% of each of 8 named editing
   benchmarks, then "pass not single benchmark, but all of them", find 20 more, use "exactly all
   benchmarks that usually done for AI LLMs" — algorithmically.
8. **CI hygiene rules accreted in-range:** tests never in `src/` (#397, #399); no domain strings
   in `src/` (#399); drop test execution on Windows/macOS for iteration speed, only test latest
   commit (#396); don't run tests on non-code changes (#442); compare workflows against the four
   link-foundation pipeline templates (#347, #390, #442); 2x tests before any architecture
   change (#450, #452); file-size guards; "add the failing gate first, watch it fail, then fix"
   (#399).

## 2. Standing clauses konard pastes on many issues

- **Case-study folder**: download all logs/data → `./docs/case-studies/issue-{id}` with
  timeline, complete requirements list, root causes, solution plans, existing-components
  review (appears on ~20 issues in range).
- **Debug/verbose**: if root cause can't be found, add debug output/verbose mode for the next
  iteration.
- **Upstream reporting**: report issues to any other affected repo, with reproducible examples,
  workarounds, fix suggestions.
- **Fix-everywhere**: "every change in one place should also be applied in all places in the
  codebase and docs".
- **Single-PR total delivery**: "Nothing should be defered or delayed... plan and execute
  everything in this single pull request, you have unlimited time and context...". Konard
  explicitly reversed agent-proposed deferrals on #348, #387, #399, #416. Only once did he relax
  it: on #424 he allowed "everything you notice should be done separately please plan as
  additional issues" (→ #433).
- **Latest-overrides-earlier**: contradictions between requirements resolved in favor of the
  newest statement (#386, formalized into REQUIREMENTS.md during #399).
- **Hardcoding only in tests**: "ok to have hardcoded examples in tests, but not the code" (#386, #399).
- Ironically, #444 asks for a CONTRIBUTING guide precisely so he can stop repeating these
  clauses — with no in-range confirmation it was written (R444-5, unclear).

## 3. Items that look silently dropped

- **R331-3/4/5** — substitution-rules→Rust/JS/WASM compilation; browser exec fallbacks
  (isolated JS eval, Rust-in-WASM compiler, rust-web-box Linux VM); server-side docker
  lifecycle (detached box containers, zip-snapshot restore with replay fallback as a setting).
  Massive architecture asks in one #331 comment; PR merged without them and no follow-up issue
  was filed.
- **R386-11** — API-cache eviction policy (access counters; delete response bodies first,
  always preserve reasoning steps). Never mentioned again.
- **R386-7 (partial)** — "collect all previous issues/comments/PR comments and list every
  requirement in docs, evaluating how fully each is implemented" — a full historical
  requirements inventory; REQUIREMENTS.md (from #399) covers only data standards, not this.
- **R395-10** — link-cli transactional memory with full history/time-travel. Never delivered
  or re-raised in range.
- **R440-2** — CI checks authored in natural language and enforced by Formal AI itself
  (self-hosting dogfood). No trace of delivery.
- **R398-4 (sub-item)** — "compare what we have knowledge wise to all popular competitors":
  no delivery evidence inside the #399 marathon.

## 4. Unanswered konard-directed questions & dangling forks

- **#413 (agent → konard):** "should I land this complete increment... and pursue the
  cross-handler meta-builder unification as a dedicated follow-up PR, or would you prefer I
  expand the unification refactor inside this same PR?... I'll proceed either way on your
  word." **No reply from konard in the thread; PR merged next day.** The unification remained
  undone; related threads (#423/#424/#433/#448) keep circling it.
- **#346 (agent → konard, earlier form):** direction question on AST-driven axes was answered
  ("try all directions") — but per-language CST/AST rewriting still did not land there, and no
  dedicated follow-up issue exists for it until the #395/#396 arc.
- **#432:** konard's own instruction created a *deliberate* dangling state: PR paused pending
  meta-language#83–#86 (formatting ontology, PDF, DOCX, cross-format reconstruction) — yet PR
  #432 merged two days later; the promised "resume once every feature is delivered upstream"
  has no visible in-range resumption. PDF generation (#425's actual user ask) remains pending.

## 5. Dubiously closed / weakly evidenced items

- **#416 ("Хочу что бы он умел заменять информацию в тексте" → benchmarks):** konard demanded
  ≥10% pass of CoEdIT, EditEval, FineEdit, CodeEditorBench, CanItEdit, EDIT-Bench,
  HumanEvalPack, **SWE-bench**, then "all of them", plus 20 more benchmarks. Said twice his
  requirements were ignored. PR merged 2026-06-12 with 33 comments; nothing in-range
  demonstrates those pass rates. The most dubious closure in the range.
- **#399 merge:** final agent report claims total closure 0/1,410 tokens and multi-source view
  delivered — but sense-level merges are 0 at the chosen threshold, `M-` ids use `sha1[:12]`
  rather than the specified source-id concatenation, stopwords were "closed" by self-definition
  rather than grounding, and konard's #398 scorecard items "no hardcoded language constants in
  code" (~35k src literals) and RML-proved 1:1 meaning↔type were still open at merge time.
- **#445, #464, #465:** closed "completed" with konard requirements in comments (message
  splitting; upstream calculator verification; it-coreference via meta algorithm) but no
  in-range PR visibly delivering them.
- **#438/#439/#440** were delivered by PRs #470/#471/#472 just outside the range — legitimately
  closed, but the #439 "agent CLI with --model formal-ai, match sonnet output by reconstructed
  reasoning" harness (R439-2) exceeds what #471's title suggests.

## 6. Open items in range

- **#447** (interface complaint / splitter-vs-scroll): still open; konard added VS Code-style
  resizer design guidance on 2026-08-01 — the freshest actionable requirement in the range.
- **#453** (Moonshot tasks): still open; recursive 2-part task splitting with first-source
  tracing; Atari Breakout 860–864, symbolic ChatGPT architecture + benchmark, "strong AI with
  own will on demand". Later planned via PR #652 (outside range).

## 7. Notable one-off precision requirements (easy to lose)

- Cache cap: min(1%, 512) items per dataset/API/topic (#412).
- ≥50 (ideally top-50 GitHub projects) install-guide↔script conversion cases (#423).
- ≥50 equation-type examples fed to calculator; `?`/`*` accepted as unknown variable (#406).
- 7-day accessibility-status cache per external service per environment (#444).
- Icon fonts: FontAwesome default + top-5 switchable (#409).
- Response-language setting: last-message / preferred / UI, default last-message (#324).
- "2x the number of tests before architectural changes" (#450/#452).
- Report trimming rules and `<version> (wasm)` folding (#386).
