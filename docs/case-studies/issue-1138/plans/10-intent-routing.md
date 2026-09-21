# Plan 10 — Capability routing scored on held-out paraphrases (bottleneck B10 of #1138)

Written before the code, in the convention `docs/case-studies/issue-710/plans/README.md`
establishes: a box is ticked in the commit that lands it, and a step that turns
out wrong is struck through with the reason, never deleted.

Standing doctrine, unchanged: associative stack only (seed data in `.lino`,
registry methods, doublets store) over specialized `try_*` Rust handlers;
generalization over memoization with held-out paraphrases in en, ru, hi, zh, es;
deterministic and honest; no deferral, no budgets, no bypasses; nothing
hard-coded for any test; ratchets only move in the strict direction.

---

## Issues addressed

- **#745** (closed COMPLETED 2026-07-18; PR #765) — "10–20+ NL variations per
  intent per language, no cross-tool misroutes, no silent UNKNOWN". Re-measured
  **still-broken** by the maintainer on #710 (2026-07-25, `main` @ `ff38e2ab`,
  v0.303.0). This plan delivers the scored capability decision and the five-
  language variation matrix that #745 clause 5 asks for in CI.
- **#758** (closed COMPLETED 2026-07-19) — "Route by CAPABILITY not tool name".
  Also re-measured still-broken on the same comment. This plan delivers the
  object/act/locus decision that makes a capability independent of the verb, and
  the Spanish half of the alias/cue registry.
- **#710** — the dropped-requirements backlog whose maintainer comment is the
  measurement both closures failed. This plan makes that measurement a
  committed, re-runnable gate so a future closure cannot outrun its evidence.
- **#1087** — the frontier queue. This plan gives each of #720, #721, #722,
  #724, #869, #1063 a *class-level* treatment that reuses plan 01
  (`01-live-concept-lookup.md`, live concept lookup) and plan 04
  (`04-formalization-depth.md`, formalization), and separates #447 as a surface defect
  plus a feedback act rather than a routing bug.
- **#720** latest news · **#721** 我不明白 · **#722** essay on quantum mechanics ·
  **#724** say something in Chinese · **#869** schedule a meeting · **#1063**
  apple-tree root size · **#447** dialog UI — the seven reported prompts.
- **#840 / #842** — the 24-node task ladder that is the existing falsifiable
  instrument for this bottleneck; this plan extends it rather than replacing it.
- **#819** — the 56-case multilingual local-path discovery benchmark, the
  existing local-search counterpart; this plan extends it to five languages and
  to the scope-less and possessive-less phrasings the maintainer measured.
- **#706** — Spanish registered as the fifth language, data-only. This plan is
  where Spanish stops being registered-but-unrouted.
- **#1095 / #1101 / #1115** — three measured misroutes (a continuation cue to
  web search; a documentation question to web search in ru/hi/zh but not en; an
  additive edit instruction to web search). Each is a symptom of the same cause
  this plan removes, and each supplies a regression case.
- **Issues plan 13's coverage table names this plan as a deliverer of** (added by
  the 2026-09-16 reconciliation): **#801** / **#821** ("search online for Elon
  Musk" — an explicit search act must reach retrieval, the `(bare_term, retrieve,
  web)` row); **#826** (`ФБС vs ФБО` then `Зарепорти баг` — an unknown-term
  comparison followed by a report act, two rows of the table); **#827**
  (`Что такое фуфломицин?` and its elliptical follow-up — routing to plan 01's
  lookup, then coreference over the dialogue locus); **#838** ("find
  hive-mind on my desktop" — the `(path_scope, retrieve, workspace)` row, with
  plan 03 owning the workspace it reads); **#800** and **#872** (retrieval under
  a marketplace constraint — the routing half lands, the source's coverage does
  not).
- **#1138 B10** — the umbrella.

---

## Current state

### What the #1138 B10 text says, and what is actually true

B10 states: "ladder 8/24; `src/seed/roles/intent.rs` has 15+ web-search roles
and no local-search counterpart". Both halves are stale, and saying so is part
of the honesty clause:

- **The ladder is 24/24, not 8/24.** `experiments/issue_840_task_ladder/results.json`
  records `"total": 24, "passed": 24, "failed": 0`, by level L1 3/3, L2 6/6, L3
  7/7, L4 8/8. `experiments/issue_840_task_ladder/README.txt` explains: "The
  historical v0.303.0 measurement was 8/24 … The current committed
  `results.json` is the strict all-green baseline." 8/24 is the maintainer's
  2026-07-25 number, quoted from the #710 comment and never re-measured in
  #1138.
- **A local-search counterpart exists.** `src/seed/roles/intent.rs:455-475`
  declares eleven `local_path_*` roles (`ROLE_LOCAL_PATH_SEARCH_ACTION`,
  `_LIST_ACTION`, `_CONTENTS_REQUEST`, `_TYPE_REQUEST`, `_SCOPE_DESKTOP`,
  `_SCOPE_HOME`, `_SCOPE_CURRENT`, `_DIRECTORY_KIND`, `_FILE_KIND`,
  `_QUERY_NOISE`, `_ROUTE_QUESTION`), backed by
  `data/seed/meanings-local-search.lino` (427 lines), executed by
  `src/agentic_coding/local_search.rs` (709 lines), and measured by
  `data/benchmarks/local-path-discovery-suite.lino` (56 cases,
  `minimum_pass_count "56"`, `languages "en|ru|hi|zh"`).

The web-search role count in B10 is right: `src/seed/roles/intent.rs:359-453`
declares fourteen `web_search_*` roles plus `ROLE_HTTP_FETCH` (:324) and
`ROLE_URL_NAVIGATE` (:333).

So the asymmetry the maintainer diagnosed was real and has been partly closed.
What has **not** been closed is the thing that made it possible: routing is
still decided by which *surface tokens* are present, and the counterpart was
added as another token family rather than as a structural decision.

### The three maintainer prompts, today

All three are ladder nodes and all three are green in the committed baseline
(`experiments/issue_840_task_ladder/results.json`):

| Node | Prompt | Committed outcome |
| --- | --- | --- |
| `838.L1` | `Find hive-mind-control center folder on my desktop` | `bash`, `find "${FORMAL_AI_DESKTOP_DIR:-$HOME/Desktop}" -type d -iname 'hive-mind-control-center' -print`, then a widened `*hive*` scan; answers with the closest name and the scope searched. **pass** |
| `838.L3.a` | `Search hive-mind-control-center on my desktop` | identical two-command `bash` sequence and answer. **pass** |
| `838.L3.b` | `Find hive-mind-control center folder on desktop` (no `my`) | identical. **pass** |
| `838.L4.a` | `Найди папку hive-mind-control center на моём рабочем столе` | identical, answered in Russian. **pass** |

Caveats that keep this from being a closed issue:

- The ladder runs only in `.github/workflows/task-ladder.yml` against a live
  `formal-ai serve`; it is **not** part of `cargo test --test unit`. A routing
  regression in an ordinary pull request is not caught by it.
- The ladder is **en and ru only** — 11 English and 13 Russian nodes
  (`experiments/issue_840_task_ladder/tasks.json`). No hi, zh or es.
- 24 nodes is not 10–20 paraphrases per intent. Local search has four
  phrasings; every other intent has one or two.

### The #745 variation matrix, today

`tests/unit/issue_745.rs` (493 lines, registered at `tests/unit/mod.rs:179`)
enforces a matrix in CI. `assert_routes` (`:41-49`) refuses a set with fewer
than 15 rows. Covered families and their asserted destinations:

| Test | Family | Languages | Asserted route |
| --- | --- | --- | --- |
| `url_object_routes_fetch_variations_without_cross_tool_misroutes` (:52) | URL object × 16 action verbs incl. `read`, `summarize`, `check`, `grab`, `pull the page`, `tell me about`, `what does` | en/ru/hi/zh | `web_fetch` |
| `local_path_object_routes_read_variations_without_web_misroutes` (:135) | local path object | en/ru/hi/zh | `read_file` |
| `explicit_content_and_file_object_route_write_variations` (:216) | content + file object | en/ru/hi/zh | `write_file` |
| `directory_listing_routes_shell_variations_in_every_supported_language` (:285) | 15 listing phrasings each | en/ru/hi/zh | `exec_command`, `command == "ls"` |
| `web_search_routes_action_variations_in_every_supported_language` (:364) | search actions | en/ru/hi/zh | `web_search` |
| `code_search_prefers_an_advertised_grep_capability_over_shell_lowering` (:35) | `search the code for RouteIntent` | en | `grep_search` |

Every one of the concrete misroutes #745 listed (`read <url>`, `display 1.txt`,
`load 1.txt`, `view the file`, `set the contents of 1.txt to hello`, the nine
multilingual list-files phrasings) is a row in this matrix. **Spanish is
absent**: `grep '"es"\|Spanish\|español' tests/unit/issue_745.rs` returns
nothing. Five of the eleven #758 capabilities (`glob`, `list_dir`, `todo`,
`subagent`, `read_many`, `multi_edit`) have no variation test at all.

### The #758 capability registry, today

`data/seed/agentic-tool-capabilities.lino` (68 lines) declares **16
capabilities** with alias lists. Seven of them also carry `cues`:
`multi_edit`, `read_many`, `grep`, `glob`, `list_dir`, `todo`, `subagent` —
each with **exactly ten memorized phrases per language**, in en/ru/hi/zh only:
28 cue lines, 280 phrases, **zero Spanish**.

`src/agentic_coding/capability_router.rs::task_matches` (:285-295) decides
capability by `lower.contains(cue)` over that list. `classify_tool` (:156-219)
maps a tool *name* to a capability by the seed aliases first and then by a
substring cascade (`lower.contains("search")`, `.contains("fetch")`,
`.contains("open")`, `.contains("browse")`, …) — the same substring style that
produced #1133's 547 empty `browser_click` calls, now guarded by an
eleven-marker deny list at `:128-140`.

So #758's "route by capability, not tool name" is delivered for the **tool** side
(aliases + scope filtering at `:66-74` + research ranking at `:94-106`) and is
*not* delivered for the **request** side: which capability a request wants is
still decided by memorized phrases.

### The planner is a hand-ordered chain

`src/agentic_coding/planner.rs` is 844 lines. `plan_chat_step_routes` (:206-299)
and `plan_settled_routes` run roughly 45 `if let Some(plan) = …` arms in a fixed
source order — `git_commit` (:326), `code_task` (:340), `structured_edit`
(:343), `structured_document` (:350), … `intent_router::plan_edit_step` (:513),
`file_read` (:522), `local_search::plan_local_search_step` (:526),
`comparison` (:529), `capability_router::plan_shared_capability_step` (:532),
shell (:536-545), `intent_router::plan_web_fetch_step` (:564),
`task_structure` (:594), `intent_router::plan_web_search_step` (:608),
`tool_result::latest_turn_answer` (:632). The order is a Rust constant with no
seed counterpart — `data/seed/handler-precedence.lino` governs the *solver*, not
the agentic planner. This is the same structure plan 09 is retiring inside
`src/solver_handlers/`, living in a directory (79 files, 22,125 lines) that no
handler ratchet measures.

### Every #1087 frontier prompt: exact outcome today

Nothing in `tests/` references #720, #721, #722, #724, #869 or #1063
(`grep -rln` over `tests/` finds no such file), so none of them has a committed
measurement. The column below states what the tree says today, with the
evidence, and marks what has and has not been re-measured.

| # | Prompt | Reported (version, date) | Today, by inspection of the tree |
| --- | --- | --- | --- |
| **#720** | `последние новости` | `unknown` on 0.289.0 wasm, 2026-07-14 | **Routes to `web_search` in both runtimes.** `data/seed/meanings-web-search.lino:187-209` gives `web_search_news_subject` the surface `" новост"`; `:210-233` gives `web_search_news_recency` the surface `" последн"`. `src/intent_formalization.rs:649-654` (`looks_like_latest_news_search`) requires both on the padded prompt, and `src/intent_formalization/prompt_relevants.rs:48-53` promotes `handler:web_search` on it. The worker mirrors the same two roles at `src/web/worker/formal_ai_worker_16.js:968-969`, used at `formal_ai_worker_17.js:180-248`. **Not re-measured end to end.** |
| **#721** | `我不明白` | `unknown`, worker fallback, 0.289.0, 2026-07-14 | **Routes to `clarification`, by memorizing the literal.** `data/seed/intent-routing.lino:400-402` lists `我不明白`, `我不懂`, `听不懂` as `phrase` rows, and `data/seed/meanings-intent.lino:444-450` repeats them as `lexeme zh` surfaces of `ROLE_CLARIFICATION_REQUEST` (`src/seed/roles/intent.rs:14`). The `clarification` handler is `status migrated` (`data/meta/handler-migration-ledger.lino:143-148`). The bug-report string itself is now in the seed — the exact shape the doctrine forbids. |
| **#722** | `Привет, напиши мне эссе по квантовой механике` | `unknown`; formalized as `(@USER OP:greet ?напиши мне эссе…)` — the greeting clause swallowed the request, 0.289.0 | **Unchanged in kind.** There is no composition/essay act anywhere: `grep -rn "essay" data/seed src/seed` finds only `data/seed/meanings-web-research.lino:481` (`text essay`, a research-document noun) and the deleted-in-plan-09 `closure-generated-12.lino:112`. The decomposition step exists (`src/solver.rs:426-427`, `record_decomposition`), so the greeting/request split is reachable; nothing binds a `compose(genre, subject, length)` act. **Not re-measured.** |
| **#724** | `Скажи что то на Китайском` | `unknown`, 0.289.0 | **Unchanged.** No role, meaning or cue for a speak-in-language directive exists: `grep -rn "speak_language" src/seed data/seed` returns nothing. The machinery it needs is already there — `SolverConfig::forced_response_language` (`src/solver.rs:138`), consumed at `src/meta_method_dispatch.rs:45,95-101` — but only a `response_language_followup` can set it, and only as a *follow-up* to a prior answer. |
| **#869** | `Назначь мне встречу с Александром на 20:00 по Грузии` | `unknown`, 0.304.1 wasm, 2026-07-26 | **Unchanged: the verb is missing from the list.** `data/seed/meanings-calendar.lino:480-509` gives `calendar_schedule_action` the Russian surfaces `забей`, `забей мне`, `поставь`, `поставь мне`, `поставить`, `поставить мне` — **`назначь` is not among them** (`grep -n "назнач" data/seed/*.lino` finds it only inside an unrelated subagent cue at `agentic-tool-capabilities.lino:60`). The fallback cue set `calendar_fallback_verbs` (`data/meta/cue-lexicon.lino:85-94`) holds `забей`, `поставь`, `schedule`, `book`; `calendar_digit_actions` (:95-103) holds `schedule`, `book`, `add`. So the promotion at `prompt_relevants.rs:130-157` does not fire, and neither does the timezone reference `по Грузии` resolve anywhere. |
| **#1063** | `Какого размера средний корень яблони?` | `unknown`, 0.345.0 wasm, 2026-08-25 | **Unchanged.** No role marks a question whose expected answer is a quantity with a unit; `grep -rn "measurement" src/seed/roles/` finds only calculator unit words (`src/seed/roles/reasoning.rs:450-458`). The prompt carries no search imperative, so the web-search promotion does not fire; it carries no `concept_lookup` cue in the `Какого размера …` shape. The same dialog's earlier turn, `Каков корень слова корень?`, also returned `unknown`. |
| **#447** | `интерфейс ужасен.` + "левая часть с кнопками видна не вся, проматывать её не даёт мышкой" | 0.193.0 wasm, 2026-06-13, with screenshots in the issue comments | **Not a routing defect.** The maintainer's comment diagnoses it as the splitter being confused for a scrollbar and asks for a VS Code-style thin resizer. What *is* a routing question: a complaint about the assistant's own surface currently has no act, so it falls to the unknown opener instead of producing a report artifact — `src/agentic_coding/report_issue.rs` exists and is only reachable from an explicit report request. |

### The three measured misroutes that explain the mechanism

- **#1095** (closed): `Continue if you have next steps` routed to web search
  because "the cue words themselves are the route"; eight of seventeen failing
  ladder leaves failed for it; one node made 672 fetch+search calls. Fixed by
  adding `agentic_continuation` as a rule handler
  (`data/meta/handler-migration-ledger.lino:11-16`) and an early return at
  `src/agentic_coding/planner.rs:241-246`.
- **#1101** (closed): `как работает pandas DataFrame.join?` routed to
  `web_search` while its English twin routed to `docs_method_explanation`,
  "because `web_search` sits above `docs_method_explanation` in
  `data/seed/handler-precedence.lino`" — precedence, not recognition, decided
  the language asymmetry.
- **#1115** (closed): `Edit the tracked file F: add a line "…" after the line
  "on:"` composed to `None` in `compose_edit_request`, so the planner fell
  through to `websearch`/`webfetch` and searched GitHub's documentation instead
  of editing the file. An *additive* instruction had no shape.

---

## Root causes

1. **Capability is decided by verbs and phrases, not by the objects in the
   request.** `capability_router.rs:285-295` matches 280 memorized phrases;
   `prompt_relevants.rs:48-53` matches role surfaces. *Mechanism:* a request
   whose object unambiguously determines the capability — a URL, a path, a
   time-of-day, a language name, a quantity question — is still routed by the
   verb, so `Search X on my desktop` and `Find X on my desktop` can differ, and
   `назначь` fails where `поставь` succeeds. This is the maintainer's exact
   diagnosis on #710 ("the gate is surface tokens"), generalized.

2. **The fix for a missing phrasing is another phrasing.** #721 was closed by
   putting the reported string `我不明白` into
   `data/seed/intent-routing.lino:400` and
   `data/seed/meanings-intent.lino:446`. `data/seed/intent-routing.lino` is 477
   lines of `phrase`/`keyword` rows for greeting, wellbeing, http_fetch and
   friends. *Mechanism:* each report adds rows; the held-out paraphrase never
   gets tested because the reported prompt is now in the seed, so the suite is
   green and the class is still unrouted.

3. **Spanish is registered but unrouted.** `data/seed/languages.lino:31`
   registers Spanish and `data/seed/language-detection.lino:29-37` detects it
   (with the note "Registered as seed data only (issue #706): no Rust edit was
   needed to add the fifth language"). But `grep -c "lexeme es"` returns **0**
   for `meanings-web-search.lino`, `meanings-local-search.lino` and
   `intent-routing.lino`, and 2 for `meanings-intent.lino`; only 14 of ~117 seed
   files carry any Spanish at all. The whole Spanish capability today is the
   seven `concept_lookup` prompts adopted by the learning cycle
   (`data/meta/language-adoption-ledger.lino`, `before_after_pairs "7"`,
   `classes "2"`). Every routing suite declares `languages "en|ru|hi|zh"`.

4. **There is no locus concept, so "where does this act happen" is guessed.**
   The router has `Capability::{Search, Fetch, Read, Write, Edit, Run, Grep,
   Glob, ListDir, Todo, Subagent, ReadMany, MultiEdit, AskUser}` and a scope
   filter on the *tool* (`acts_in_capability_scope`, `:66-74`), but nothing on
   the *request*. `на моём рабочем столе` is matched as a scope role surface
   (`ROLE_LOCAL_PATH_SCOPE_DESKTOP`); drop the possessive and the surface fails
   and the request becomes a web search. Locus should be derived from the object
   — a path-shaped token, a filesystem noun, a URL, a bare term — not from a
   possessive pronoun.

5. **Precedence outranks recognition.** #1101 proved that a correctly
   recognisable prompt loses to a higher-ranked handler that also claims it. In
   the agentic planner the same role is played by source order
   (`planner.rs:206-650`), which has no data representation at all, so it cannot
   even be inspected the way `handler-precedence.lino` can.

6. **No act for a whole class of ordinary requests.** Compose (#722),
   demonstrate-in-language (#724), schedule (#869 — the act exists, the verb
   does not), measure (#1063), and complain (#447) have no formal act. *Mechanism:*
   the formalizer produces `OP:express` or `OP:greet` and the need ledger records
   nothing to satisfy, so every route declines and the unknown opener answers.
   This is B4's recursion — "the formalizer has no way to say which concept it
   lacks" — showing up as a routing symptom.

7. **The measurement instruments do not cover what is claimed.** #745 asked for
   a CI variation matrix; `tests/unit/issue_745.rs` delivers one for five of
   eleven capabilities in four of five languages. The ladder covers 24 nodes in
   two languages and does not run in the ordinary test job. No ratchet records a
   routing score at all — `data/meta/ladder-ratchet.lino` measures the *coding*
   ladder (`leaf_nodes_passing 15` of 32), not intent routing. *Mechanism:* both
   #745 and #758 could be, and were, closed COMPLETED on evidence that asserted
   the shape of a plan rather than measured routing outcomes — the maintainer's
   own words on #710.

---

## Solution options

### Option A — Extend the existing cue and role lists to five languages

*Description.* Add `es` lexemes to `meanings-web-search.lino`,
`meanings-local-search.lino`, `meanings-intent.lino` and
`agentic-tool-capabilities.lino`; add `назначь`, the missing calendar verbs, a
speak-in-language role, an essay role and a measurement-question role; widen
`tests/unit/issue_745.rs` to five languages and eleven capabilities.

*Architecture sketch.* No new code. Seed rows plus test rows.

*Pros.* Fastest visible improvement; every one of the seven frontier prompts can
be made to route within a week; no risk to existing behaviour; entirely
data-only, which reads as doctrine-compliant.

*Cons.* It is root cause 2 applied at scale. 280 cue phrases become ~350; the
next `назначить`/`agendar`/`निर्धारित` still fails. Held-out paraphrases would
have to be *withheld* from the very lists that make routing work, so the suite
would either be green-by-construction (if the paraphrases are seeded) or
red forever (if they are not). It cannot satisfy "10–20 held-out paraphrases per
intent with zero cross-tool misroutes" honestly.

*Doctrine fit.* Fails "generalization over memoization" directly.

*Effort.* Small (~2 weeks).

*Risk.* Low technically, high doctrinally: it would produce a green suite that
proves nothing, which is the exact failure #710 was filed about.

### Option B — Learn the routing from the corpus (statistical intent classifier)

*Description.* Train a classifier over the accumulated dialogs and benchmark
corpora to map prompt → capability, replacing the cue lists.

*Architecture sketch.* A feature extractor over the formalized intent, a scoring
model stored as links, a threshold below which the router asks.

*Pros.* Genuinely generalizes to unseen phrasings; multilingual for free where
training data exists; the model is data, which fits the associative stack in
letter.

*Cons.* `NON-GOALS.md` and `src/solver.rs:19-22` require the solver to be
deterministic for a given config and impulse; a learned scorer's decision
boundary moves between releases, and a paraphrase that flips class between two
commits is exactly the unreproducible behaviour the project forbids. It is also
unexplainable: a misroute has no citable rule. And training data for Spanish
(7 prompts, 2 classes) does not exist.

*Doctrine fit.* Fails "deterministic and honest".

*Effort.* Large.

*Risk.* High, and the failure is silent.

### Option C — Structural capability decision: object type × act × locus (selected)

*Description.* Decide capability from three structurally derived facts, each
independently testable, combined by a seed decision table:

1. **Object type** — what the request is *about*, read from the prompt's shape,
   not its vocabulary: `url`, `path`, `bare_term`, `quoted_content`,
   `time_expression`, `language_name`, `quantity_question`, `self_surface`,
   `none`.
2. **Act** — what is to be done, read from seed roles as today but at the level
   of an act rather than a tool: `retrieve`, `enumerate`, `transform`,
   `compose`, `schedule`, `explain`, `demonstrate`, `record`.
3. **Locus** — where the effect lands, derived from the object and any scope
   noun: `workspace`, `web`, `dialogue`, `self`.

The decision table `data/seed/capability-routing.lino` maps
`(object, act, locus) → capability`, with an explicit `ask` outcome for
genuinely ambiguous triples and an explicit `honest_gap` outcome when the
capability is known but no advertised tool provides it.

*Architecture sketch.* Detailed below. `src/capability_routing.rs` (name free)
holds the three derivations and the table lookup; `capability_router.rs` keeps
only the capability → advertised-tool-name mapping it already does well;
`planner.rs`'s 45-arm chain is replaced arm by arm as each act is covered.

*Pros.* A paraphrase that changes only the verb cannot change the route, because
the object and locus are unchanged — which is precisely the maintainer's
`Search`/`Find`/`my` observation. Held-out paraphrases are honest: the object
and locus derivations are structural, so a Spanish prompt with a path in it
routes without a Spanish cue list. Deterministic and citable: every decision
names the triple and the table row. Spanish needs ~8 act surfaces, not 10 cue
phrases × 11 capabilities.

*Cons.* Act still needs seed surfaces per language — there is no structural way
to tell `retrieve` from `transform`. Eight acts × five languages is a real
translation job, but it is 40 role blocks rather than 280 phrases, and each act
generalizes across every capability that uses it. The decision table can grow
sparse corners where a triple has no row; the `ask` outcome must be the default
so a gap is visible rather than silent.

*Doctrine fit.* Strongest available. Structural derivation is generalization;
the table is data; the `ask`/`honest_gap` outcomes satisfy "no silent UNKNOWN"
without fabricating a call; determinism is preserved because every step is a
total function of the prompt.

*Effort.* Medium-large: ~8 leaves for the derivations and the table, ~7 leaves
for the frontier classes, ~5 for the suites and gates.

*Risk.* Medium. Mitigated by keeping every existing `tests/unit/issue_745.rs`
assertion green throughout — the new path must reproduce all of them before any
old arm is deleted.

### Option D — Reuse the solver's precedence table for the agentic planner

*Description.* Give `planner.rs`'s 45 arms names and put their order in
`data/seed/handler-precedence.lino` (or a sibling), so the agentic order is data
like the solver's.

*Architecture sketch.* A `planner-precedence.lino` plus a registry of arm
closures, mirroring `src/solver_dispatch.rs:396-430`.

*Pros.* Cheap; makes the order inspectable and reorderable; removes root cause 5
for the agentic surface; composes with plan 09's browser work.

*Cons.* Ordering is not recognition. #1101 showed that a data-driven order still
misroutes when a higher arm over-claims; making the order data would have made
that bug *editable*, not absent. It fixes none of the seven frontier prompts.

*Doctrine fit.* Good but partial.

*Effort.* Small.

*Risk.* Low.

---

## Decision

**Option C is selected, with Option D adopted as a subordinate leaf.**

Reasons:

1. Only Option C can produce an honest held-out paraphrase score. Options A and
   D leave the route decided by the presence of a surface, so any paraphrase not
   in the seed fails and any paraphrase in the seed is not held out. Option C's
   object and locus derivations are structural, so a paraphrase can be withheld
   from the seed and still route.
2. Option C answers the maintainer's measurement directly: `Search X on my
   desktop`, `Find X on my desktop` and `Find X on desktop` share one object
   (`path`-ish bare name), one act (`retrieve`), one locus (`workspace`, from
   the filesystem noun `desktop`), so they cannot diverge. The possessive stops
   being load-bearing.
3. It is the cheapest route to five languages. Spanish needs eight act role
   blocks and a handful of scope nouns, not a Spanish translation of 280 cue
   phrases plus 477 lines of `intent-routing.lino`.
4. It preserves determinism, which rules out Option B regardless of its
   generalization power.
5. Option D is kept because making the planner's order data is independently
   right and is the mechanism by which each Option C act displaces an old arm
   without a flag day.

Rejected:

- **Option A** — rejected as the strategy; its Spanish seed work is retained as
  leaves, but attached to acts and scope nouns rather than to per-capability cue
  lists, and the 280 phrase cues are deleted as Option C's coverage lands.
- **Option B** — rejected outright on determinism.
- **Option D** — not rejected; demoted to leaves 18–19.

---

## Architecture

### 1. The three derivations

**Object type** — total function of the prompt, no natural-language vocabulary.
`src/capability_routing.rs::object_type(prompt) -> ObjectType`:

| Type | Derivation | Existing code reused |
| --- | --- | --- |
| `url` | a token parsing as an absolute URL | `src/agentic_coding/web_research.rs:634` (`urls_in`) |
| `path` | a token with a path separator, a known extension, or a filesystem-noun neighbour | `src/agentic_coding/file_path_shape.rs`, `local_search.rs:270-295` (`explicit_file_literal`) |
| `quoted_content` | a balanced quoted span longer than one token | `local_search.rs:296-331` (`quoted_literal`) |
| `time_expression` | a clock time, a date, or a weekday/relative-day reference | `ROLE_CALENDAR_DAY_REFERENCE`, plus a new `shape time_of_day` |
| `language_name` | a term resolving to a registered language in `data/seed/languages.lino` | `src/language.rs` |
| `quantity_question` | an interrogative whose expected answer is a magnitude: a dimension/measure noun in the subject position, or a unit word | `src/seed/roles/reasoning.rs:450-458` |
| `bare_term` | one or more content tokens that are none of the above | `looks_like_single_concept_lookup` (`intent_formalization.rs`) |
| `self_surface` | the subject is the assistant's own UI, answer, or behaviour | `ROLE_ASSISTANT_MECHANISM_INQUIRY` |
| `none` | no content token survives | — |

A prompt may carry several objects; they are returned ranked, and the table is
consulted for the highest-ranked first. `Find X on my desktop` yields
`[bare_term(X), path_scope(desktop)]`.

**Act** — seed roles, one meaning per act, five languages, in a new
`data/seed/meanings-acts.lino`:

`retrieve`, `enumerate`, `transform`, `compose`, `schedule`, `explain`,
`demonstrate`, `record`. Eight roles × five languages. `найди`, `search`,
`busca`, `खोजें`, `查找` are all `retrieve`; `назначь`, `поставь`, `schedule`,
`agenda`, `निर्धारित करें`, `安排` are all `schedule`. The act vocabulary is the
one place per-language surfaces remain, and it is shared by every capability,
so adding a language is eight blocks rather than eleven cue lists.

**Locus** — derived, never stated:

- an object of type `path`, or any filesystem-scope noun
  (`ROLE_LOCAL_PATH_SCOPE_*`), or a repository/code noun ⇒ `workspace`;
- an object of type `url`, or a recency+subject pair, or an explicit web medium
  ⇒ `web`;
- a reference to a prior turn, the conversation, or `self_surface` ⇒ `dialogue`;
- the assistant's own capabilities, source or behaviour ⇒ `self`;
- otherwise unresolved, and the table's `ask` outcome applies.

Crucially, `desktop`, `рабочий стол`, `escritorio`, `डेस्कटॉप`, `桌面` are
filesystem scope nouns *regardless of a possessive*, which is the single change
that makes the maintainer's three prompts structurally identical.

### 2. The decision table

`data/seed/capability-routing.lino` (name checked free):

```
capability_routing
  issue 1138
  purpose "Capability is a function of the object in the request, the act asked
           for, and where the effect lands. Verbs select the act; they never
           select the capability. A triple with no row asks rather than guesses."
  route
    object url
    act retrieve
    locus web
    capability web_fetch
    because "issue 745: a concrete URL is unambiguous whatever the verb"
  route
    object path
    act retrieve
    locus workspace
    capability read_file
  route
    object bare_term
    act retrieve
    locus workspace
    capability grep
    fallback shell
    because "issue 758: code navigation must never reach web_search"
  route
    object bare_term
    act retrieve
    locus web
    capability web_search
  route
    object path_scope
    act enumerate
    locus workspace
    capability list_dir
    fallback shell
  route
    object time_expression
    act schedule
    locus dialogue
    capability calendar_create_event
  route
    object language_name
    act demonstrate
    locus dialogue
    capability response_language_demonstration
  route
    object quantity_question
    act retrieve
    locus web
    capability concept_measurement_lookup
  route
    object self_surface
    act record
    locus self
    capability report_issue
  default ask
```

`fallback` names the capability used when the preferred one is not advertised —
#758's "specialized-first, bash as universal fallback" policy, stated as data
rather than as the Rust cascade at `capability_router.rs:278-281`.

`src/capability_routing.rs` evaluates the table; `capability_router.rs::tool_for`
(:23-58) is unchanged and still turns a capability into whichever alias the
client advertised. That split is exactly #758's expected item 1.

### 3. The local-search role counterpart, completed

The eleven `local_path_*` roles stay, with three changes:

1. `ROLE_LOCAL_PATH_SCOPE_DESKTOP/_HOME/_CURRENT`
   (`src/seed/roles/intent.rs:463-467`) are reclassified as **locus** evidence,
   matched without requiring an adjacent possessive.
   `data/seed/meanings-local-search.lino` gains `lexeme es` for every one of the
   eleven roles.
2. `ROLE_LOCAL_PATH_SEARCH_ACTION` and `ROLE_LOCAL_PATH_LIST_ACTION` are
   *retired into* the shared acts `retrieve` and `enumerate`; their surfaces
   merge into `data/seed/meanings-acts.lino`, so the same verb list serves web
   search, code search and file search, and the asymmetry B10 names cannot
   recur in any direction.
3. `data/benchmarks/local-path-discovery-suite.lino` grows from
   `languages "en|ru|hi|zh"` / 56 cases to five languages and 20 cases per
   language (100), including the possessive-less and verb-varied forms the
   maintainer measured.

### 4. Generalized treatment per frontier class

Each is a *class*, not a prompt. Plan 01 (B1, live concept lookup through
`data/seed/sources-registry.lino`, file `01-live-concept-lookup.md`) and plan 04
(B4, `04-formalization-depth.md`, formalization that emits
needs and stores concepts and procedures) are the two dependencies; where a
class needs them, it is stated.

**News / current events (#720).** Triple: `(bare_term, retrieve, web)` with a
`freshness: live` qualifier derived from `ROLE_WEB_SEARCH_NEWS_RECENCY`. The
capability is `web_search`; the *qualifier* changes the source kind selected
from the registry (news sources before encyclopedic ones) and requires the
answer to carry a publication date. **Reuses plan 01** for the retrieval and the
content-addressed cache. Honest failure: when no live capability is available,
the answer names the source kinds that would have been consulted and the date of
the newest cached record, never a canned apology. Generalization proof: five
languages × 12 paraphrases that never say "новости"/"news" (`что нового`,
`what's happening`, `qué está pasando`, `क्या चल रहा है`, `有什么新消息`).

**Non-understanding (#721).** Triple: `(self_surface, explain, dialogue)`.
The act is not an apology; it is *re-render the previous assistant turn at a
lower formalization depth* — the same answer with its steps expanded, in the
prompt's language. This is a `dialogue_state_query` operation in plan 09's
family M5, reading the event log. The three memorized `phrase` rows at
`data/seed/intent-routing.lino:400-402` are **deleted** in the same commit, and
the held-out set uses phrasings absent from every seed file
(`это непонятно`, `no te sigo`, `你在说什么`, `समझ में नहीं आ रहा`). With no
prior turn the honest answer says there is nothing to clarify and asks what was
unclear — not the capability menu.

**Essay / composition (#722).** Two defects, two fixes. (a) The formalizer must
not let a greeting clause consume the rest: decomposition
(`src/solver.rs:426-427`) already splits composite prompts, and a greeting
clause becomes its own sub-impulse whose answer is prepended, so the residual
`напиши мне эссе по квантовой механике` is formalized on its own. This is a
**plan 04** change and is claimed there, not here. (b) The act `compose` with
object `bare_term` and locus `web` routes to a composition procedure: retrieve
the subject's concept graph (plan 01), formalize it into sections (plan 04),
render section by section with per-section provenance. There is no essay
template and no length table — the section list comes from the retrieved
structure. Honest failure: when the subject cannot be grounded, the answer is
the outline plus a named gap per unresolved section, never prose invented to
fill it.

**Speak-in-language (#724).** Triple: `(language_name, demonstrate, dialogue)`.
The capability sets `SolverConfig::forced_response_language`
(`src/solver.rs:138`) — the mechanism that already exists for
`response_language_followup` (`src/meta_method_dispatch.rs:45,95-101`) — and
then answers a *grounded* question in that language: by default the assistant's
own identity record, which is real content the system holds, rendered through
the normal multilingual response path. No stock phrase is stored. Generalizes to
every registered language automatically, because the object is resolved through
`data/seed/languages.lino` rather than a per-language branch; asking for an
unregistered language produces an honest "I have no surfaces for X" naming the
five it has.

**Scheduling (#869).** Triple: `(time_expression, schedule, dialogue)`.
Three sub-parts, each general: (a) `назначь` joins the shared `schedule` act
with its four siblings in five languages, so the verb list is one place, not
`meanings-calendar.lino` plus two cue sets in `data/meta/cue-lexicon.lino:85-103`;
(b) the time expression `20:00` is an *object*, parsed structurally, replacing
`prompt_relevants.rs:154`'s `contains("в ") || contains(':')`; (c) `по Грузии`
is a **concept lookup** — the term resolves to a country, the country to an IANA
timezone, via plan 01 and the existing Wikidata cache — not a hard-coded
country→zone table. Participants (`с Александром`) bind as attendees through the
entity resolution that `who_is` already uses
(`data/meta/handler-migration-ledger.lino:115-120`). Honest failure: when the
timezone cannot be resolved, the event is created in the user's detected zone
and the answer says which zone it used and why.

**Factual measurement question (#1063).** Triple:
`(quantity_question, retrieve, web)`. The new object type is the general case:
an interrogative asking for a magnitude of a property of a concept. The
capability retrieves the concept (plan 01), looks for a property with a unit,
and answers with the value, the unit, the range if the source gives one, and the
source. **Reuses plan 04** for the quantity formalization (a magnitude with a
unit is a concept, not a sentence). Honest failure: names the concept
(`яблоня`), the property (`размер корня`) and the source kinds consulted. The
same triple covers `Какого размера средний корень яблони?`, `How deep do oak
roots go?`, `¿Cuánto pesa una manzana?`, `सेब के पेड़ की जड़ कितनी लंबी होती है?`,
`苹果树的根有多大？` with no per-prompt row.

**UI complaint (#447).** Triple: `(self_surface, record, self)` → `report_issue`.
The routing half is one table row: a complaint about the assistant's own surface
produces a structured report (the existing
`src/agentic_coding/report_issue.rs` flow) carrying the user's words, the
detected surface, and the user-context block the issue template already
collects, and asks the one question needed to localise it. The *defect* half is
not routing and is a separate leaf: replace the splitter's mid-handle with a
VS Code-style thin resizer that recolours on hover and shows a horizontal-resize
cursor, verified on a phone and a desktop browser with screenshots in
`docs/case-studies/issue-1138/`, as #1087 clause 3 requires.

### 5. Failure and honesty behaviour

Four outcomes, exhaustive, each with a distinct observable:

1. **Routed** — the triple has a row and the capability's tool is advertised.
   The plan emits the call and logs `capability_routing:(object, act, locus) → capability`.
2. **Lowered** — the preferred capability is not advertised and the row names a
   `fallback`. The plan emits the fallback and logs both, so a reader sees that
   `grep` became `bash`. This is #758's specialized-first policy.
3. **Honest gap** — the triple has a row but neither the capability nor its
   fallback is advertised. The answer names the capability that was needed and
   what was missing. Never a capability menu, never invented prose.
4. **Ask** — the triple has no row, or two rows tie. One question through the
   `ask_user` capability (`data/seed/agentic-tool-capabilities.lino:68`,
   `request_user_input`), naming the two readings. When no ask tool is
   advertised, the answer states the ambiguity and the two readings in prose.

"Silent UNKNOWN" becomes unreachable by construction: every path ends in one of
the four, and a test asserts that no prompt in any benchmark suite produces the
unknown opener. `#745` clause 3 is then structural rather than aspirational.

---

## Tests first

### Held-out paraphrase sets — actual prompt text

New suite `data/benchmarks/capability-routing/` with `{en,ru,hi,zh,es}.lino`
members and `data/benchmarks/capability-routing-suite.lino` in the shape of
`data/benchmarks/local-path-discovery-suite.lino:1-18`. **Twelve prompts per
intent per language**, seven intents, five languages = **420 cases**;
`languages "en|ru|hi|zh|es"`; `minimum_pass_count` set to the honest first
measurement and ratcheted upward only.

Every string below is held out: it must not occur in any file under
`data/seed/`, asserted by the same no-memorization scan the coding benchmarks
use. Three of the twelve per cell are shown.

**News / current events → `web_search` + `freshness: live`**

- en: `What's happening in the world right now?` · `Anything big I missed today?` · `Give me today's headlines with the dates.`
- ru: `Что сейчас происходит в мире?` · `Что важного было сегодня?` · `Расскажи, что нового, со ссылками.`
- hi: `इस समय दुनिया में क्या चल रहा है?` · `आज कुछ बड़ा हुआ क्या?` · `आज की मुख्य खबरें तारीख के साथ दीजिए।`
- zh: `现在世界上发生了什么？` · `今天有什么大事吗？` · `给我今天的头条，带上日期。`
- es: `¿Qué está pasando en el mundo ahora mismo?` · `¿Pasó algo importante hoy?` · `Dame los titulares de hoy con las fechas.`

**Non-understanding → `explain` over the previous turn**

- en: `That went over my head.` · `Can you put that another way?` · `You lost me there.`
- ru: `Это мне непонятно.` · `Можешь сказать иначе?` · `Я потерял нить.`
- hi: `यह मेरे सिर के ऊपर से चला गया।` · `क्या आप इसे दूसरे शब्दों में कह सकते हैं?` · `मैं समझ नहीं पा रहा।`
- zh: `这个我没跟上。` · `能换个说法吗？` · `你说的我没懂。`
- es: `Eso se me escapó.` · `¿Puedes decirlo de otra forma?` · `Me perdí ahí.`

**Essay / composition → `compose`**

- en: `Put together a few pages on how quantum mechanics got started.` · `I need a written piece about entanglement for a class.` · `Draft something long-form on wave-particle duality.`
- ru: `Составь мне несколько страниц о том, как возникла квантовая механика.` · `Нужен связный текст про запутанность для занятия.` · `Набросай большой материал о корпускулярно-волновом дуализме.`
- hi: `क्वांटम यांत्रिकी की शुरुआत पर कुछ पन्ने तैयार कीजिए।` · `कक्षा के लिए उलझाव पर एक लिखित रचना चाहिए।` · `तरंग-कण द्वैत पर एक लंबा लेख तैयार कीजिए।`
- zh: `写几页关于量子力学如何起步的内容。` · `我需要一篇关于纠缠的课堂文稿。` · `就波粒二象性写一篇长文。`
- es: `Prepárame unas páginas sobre cómo empezó la mecánica cuántica.` · `Necesito un texto sobre el entrelazamiento para clase.` · `Redacta algo extenso sobre la dualidad onda-partícula.`

**Speak-in-language → `demonstrate` with a forced response language**

- en: `Give me a line in Hindi.` · `Try answering me in Chinese for once.` · `Show me what you sound like in Spanish.`
- ru: `Дай фразу на хинди.` · `Ответь мне разок по-китайски.` · `Покажи, как ты звучишь по-испански.`
- hi: `मुझे चीनी में एक पंक्ति दीजिए।` · `एक बार रूसी में उत्तर दीजिए।` · `दिखाइए कि आप स्पेनिश में कैसे लगते हैं।`
- zh: `用印地语说一句吧。` · `这次用俄语回答我。` · `让我听听你的西班牙语。`
- es: `Dime una frase en hindi.` · `Contéstame en chino por una vez.` · `Muéstrame cómo suenas en ruso.`

**Scheduling → `calendar_create_event`**

- en: `Put me down with Alexander at eight tonight, Tbilisi time.` · `Block an hour with the team on Friday at 14:00 CET.` · `Set up a call with Maria for the 20th at noon.`
- ru: `Назначь мне встречу с Александром на 20:00 по Грузии.` *(the reported prompt, kept as the pinned regression)* · `Запиши меня к команде в пятницу на 14:00 по Берлину.` · `Организуй звонок с Марией двадцатого в полдень.`
- hi: `आज रात आठ बजे तिबिलिसी समय पर अलेक्जेंडर के साथ मुझे दर्ज कीजिए।` · `शुक्रवार 14:00 बर्लिन समय टीम के साथ एक घंटा रखिए।` · `बीस तारीख दोपहर मारिया के साथ कॉल तय कीजिए।`
- zh: `今晚八点第比利斯时间，帮我和亚历山大约一下。` · `周五 14:00 柏林时间给团队留一小时。` · `二十号中午和玛丽亚安排一次通话。`
- es: `Apúntame con Alejandro a las ocho de esta noche, hora de Tiflis.` · `Resérvame una hora con el equipo el viernes a las 14:00 CET.` · `Concierta una llamada con María el día 20 a mediodía.`

**Factual measurement question → `concept_measurement_lookup`**

- en: `How far down do oak roots usually reach?` · `About how much does a ripe apple weigh?` · `What span does a mature maple's root system cover?`
- ru: `Какого размера средний корень яблони?` *(the reported prompt, pinned)* · `Насколько глубоко обычно уходят корни дуба?` · `Сколько примерно весит спелое яблоко?`
- hi: `बलूत की जड़ें आमतौर पर कितनी गहरी जाती हैं?` · `एक पके सेब का वज़न लगभग कितना होता है?` · `परिपक्व मेपल की जड़ें कितनी दूर तक फैलती हैं?`
- zh: `橡树的根一般能扎多深？` · `一个熟苹果大概有多重？` · `成熟枫树的根系能覆盖多大范围？`
- es: `¿Hasta qué profundidad llegan las raíces del roble?` · `¿Cuánto pesa más o menos una manzana madura?` · `¿Qué extensión cubre la raíz de un arce adulto?`

**Local filesystem search → `bash`/`list_dir`, never `websearch`**

- en: `Find hive-mind-control center folder on my desktop` · `Search hive-mind-control-center on my desktop` · `Find hive-mind-control center folder on desktop` *(the maintainer's three, pinned)*
- ru: `Найди папку hive-mind-control center на моём рабочем столе` · `Поищи hive-mind-control-center на рабочем столе` · `Где лежит папка hive-mind-control-center?`
- hi: `मेरे डेस्कटॉप पर hive-mind-control-center फ़ोल्डर ढूँढिए।` · `डेस्कटॉप पर hive-mind-control-center खोजिए।` · `hive-mind-control-center फ़ोल्डर कहाँ है?`
- zh: `在我的桌面上找 hive-mind-control-center 文件夹。` · `桌面上搜一下 hive-mind-control-center。` · `hive-mind-control-center 文件夹在哪里？`
- es: `Busca la carpeta hive-mind-control-center en mi escritorio.` · `Encuentra hive-mind-control-center en el escritorio.` · `¿Dónde está la carpeta hive-mind-control-center?`

### Unit, integration and specification tests

| Path | Name | Asserts |
| --- | --- | --- |
| `tests/unit/issue_1138_object_type.rs` *(new, free)* | `object_type_is_structural_in_every_language` | the same object is derived from the same prompt translated five ways, with no per-language branch |
| " | `a_possessive_never_changes_the_locus` | `on my desktop` and `on desktop` yield locus `workspace` |
| `tests/unit/issue_1138_capability_routing.rs` *(new, free)* | `held_out_paraphrases_route_without_cross_tool_misroutes` | the 420-case suite; zero rows reach a capability outside their intent |
| " | `every_triple_resolves_to_a_row_or_asks` | no prompt in any committed benchmark yields the unknown opener |
| " | `a_new_route_row_changes_routing_with_no_rust_edit` | inject a fixture row; routing changes |
| " | `a_verb_synonym_never_changes_the_capability` | for each act, swap the verb across its five-language surfaces and assert the capability is invariant |
| `tests/unit/issue_1138_frontier_classes.rs` *(new, free)* | one test per class (`news`, `non_understanding`, `compose`, `demonstrate`, `schedule`, `measurement`, `ui_complaint`) | the class behaviour, including the honest-failure text when the capability is unavailable |
| " | `reported_frontier_prompts_are_not_in_the_seed` | the seven reported strings do not occur under `data/seed/` — this is the test that deletes `intent-routing.lino:400-402` |
| `tests/unit/issue_745.rs` *(exists)* | every existing assertion | must stay green throughout; Spanish rows added, `assert_routes` floor raised 15 → 20 |
| `tests/unit/issue_842.rs` *(exists)* | every existing assertion | unchanged |
| `tests/unit/specification/capability_routing_table.rs` *(new)* | `table_is_total_over_the_declared_axes` | every `(object, act, locus)` the derivations can emit has a row or is explicitly `ask` |
| `tests/integration/issue_1138_no_silent_unknown.rs` *(new)* | `no_benchmark_prompt_reaches_the_unknown_opener` | across every suite under `data/benchmarks/` |
| `experiments/issue_840_task_ladder/tasks.json` *(exists)* | appended nodes | the seven frontier prompts become ladder nodes with new stable IDs; per the README, IDs are appended, never renamed |

Registration follows `tests/unit/mod.rs` (`mod issue_745;` at `:179`,
`mod issue_842;` at `:196`); the four new modules take alphabetical positions.

### Gates and ratchets, with exact starting and target numbers

New ledger `data/meta/capability-routing-ratchet.lino` (name checked free):

| Measure | Start | Target | Strict direction |
| --- | --- | --- | --- |
| `intents_measured` | 7 | 11 (the full #758 shared set) | up |
| `languages_measured` | 4 (`en\|ru\|hi\|zh`, every existing suite) | 5 | up |
| `paraphrases_per_intent_per_language` | 15 (`tests/unit/issue_745.rs:41-45` floor) | 20 | up |
| `capability_routing_cases_passing` | honest first measurement of the 420 | 420 | up |
| `cross_tool_misroutes` | honest first measurement | 0 | down |
| `silent_unknowns` | honest first measurement | 0 | down |
| `memorized_capability_cues` | 280 (`agentic-tool-capabilities.lino`, 7 caps × 4 langs × 10) | 0 | down |
| `intent_routing_phrase_rows` | 477 lines of `data/seed/intent-routing.lino` | 0 | down |
| `planner_route_arms` | 45 (`src/agentic_coding/planner.rs`) | 0 | down |
| `frontier_prompts_open` | 7 (#720, #721, #722, #724, #869, #1063, #447) | 0 | down |

The strict rule is plan 09's: a measured value that beats its ceiling fails with
"lower the reviewed ceiling", copied from
`scripts/check-minimal-core-boundary.rs:286-290`. **Six of these measures ratchet
*upward*, which plan 00 §6.7 admits as a declared exception: each states its
direction in its own `how` field, and the checker fails in both directions
either way (plan 00 §9 X6).** New checker
`scripts/check-capability-routing.rs` *(name free)* and gate file
`data/meta/ci-gates/check-capability-routing.lino` *(free)*, stage `rust`,
following `data/meta/ci-gates/check-debt-ratchet.lino`'s shape.

The task ladder moves from its own workflow into the ordinary `rust` stage for
its route-only nodes (`FIXTURES=none`, per
`experiments/issue_840_task_ladder/README.txt`), so a routing regression fails an
ordinary pull request rather than only the nightly.

---

## Implementation leaves

Ordered; each independently verifiable and commit-sized.

**Measure honestly before changing anything.**

- [ ] 1. Run every #1087 frontier prompt and the maintainer's three prompts
      through the current `main` in both runtimes; commit the raw transcripts to
      `docs/case-studies/issue-1138/frontier-baseline/`; create
      `data/meta/capability-routing-ratchet.lino` with the measured starting
      numbers, including the honest counts for `cross_tool_misroutes` and
      `silent_unknowns`.
- [x] 2. Ship `data/benchmarks/capability-routing/` (420 cases, five languages)
      and its suite header; run it; record `capability_routing_cases_passing`
      at whatever it is. The suite is red and that is the honest baseline.
      **Done (wave I9): the suite landed and its first honest measurement after
      the routing half was 420/420 with `cross_tool_misroutes 0` and
      `silent_unknowns 0`; the numbers are recorded in
      `data/meta/capability-routing-ratchet.lino`.**
- [x] 3. Add `scripts/check-capability-routing.rs` and its gate file, strict
      two-sided.

**Build the three derivations.**

- [x] 4. `src/capability_routing.rs`: `ObjectType` and `object_type()`, reusing
      `web_research.rs:634`, `file_path_shape.rs`, `local_search.rs:270-331`;
      `tests/unit/issue_1138_object_type.rs` with the five-language cases.
- [x] 5. Add the `time_of_day` and `quantity_question` object shapes; unit-test
      each independently of any capability.
- [x] 6. `data/seed/meanings-acts.lino`: the eight acts, five languages each;
      register in `data/meta/seed-registry.lino` and regenerate.
- [x] 7. `locus()`: derive from object + scope nouns; add `lexeme es` to the
      eleven `local_path_*` roles in `data/seed/meanings-local-search.lino`;
      assert `on my desktop` ≡ `on desktop`.

**Land the table.**

- [x] 8. `data/seed/capability-routing.lino` with the nine seed rows plus
      `default ask`; `src/capability_routing.rs::route()`;
      `tests/unit/specification/capability_routing_table.rs`.
- [x] 9. Wire `route()` ahead of `capability_router::plan_shared_capability_step`
      (`src/agentic_coding/planner.rs:532`) behind a config flag, with every
      `tests/unit/issue_745.rs` assertion green on both paths.
- [x] 10. Implement the four outcomes (routed / lowered / honest gap / ask) and
      `tests/integration/issue_1138_no_silent_unknown.rs`.
- [x] 11. Flip the flag to default; delete the 280 cue phrases from
      `data/seed/agentic-tool-capabilities.lino` and `task_matches`
      (`capability_router.rs:285-295`); record
      `memorized_capability_cues 0`.
      **Done (2026-09-18). The flag defaults on (`table_routing_enabled`), the
      seven `cues` blocks and `task_matches` are gone, and the shared
      cue-phrase arm in the planner kept its *position* through the restricted
      `plan_named_capability_step` table call (the seven named capabilities
      must still beat the shell cascade that their own rows lower to).
      `memorized_capability_cues 0` is recorded; the 420-case suite holds at
      420/420 with `cross_tool_misroutes 0` and `silent_unknowns 0`.**

**The frontier classes.**

> **Routing half landed 2026-09-16 (wave I9 continuation).** All seven classes
> now *reach* their capability in five languages and name an honest gap when it
> is withheld: `tests/unit/issue_1138_frontier_classes.rs` is green, and the
> 420-case held-out suite passes 420/420 with `cross_tool_misroutes 0` and
> `silent_unknowns 0`. Leaves 12, 14, 15, 16 and 17 stay **open** because each
> also owes the *execution* half its text names -- news-first source selection,
> the composition procedure over a retrieved concept graph, the
> `forced_response_language` binding, the `cue-lexicon.lino` deletions, and the
> measurement lookup over a retrieved property with a unit -- and four of them
> are blocked on plans 01 and 04 (risk 5). Leaf 13 is ticked: its deletion of
> the three memorized literals is the whole of it.

- [ ] 12. News: the `freshness: live` qualifier and news-first source selection
      (**after plan 01**); honest-gap text; close #720 with its paraphrase set.
- [x] 13. Non-understanding: the re-render-previous-turn act, added as one
      operation of plan 09's `dialogue_state_query` family (leaf 33) rather than
      as a handler; **delete** `data/seed/intent-routing.lino:400-402` and the
      matching `lexeme zh` surfaces at `data/seed/meanings-intent.lino:444-450`;
      close #721. **This leaf lands after plan 09 leaves 33 and 36: leaf 36 keeps
      `clarification`'s five-language role surfaces, and this leaf removes only
      the three memorized literals, so the class stays routed while the
      memorization goes (plan 00 §9 X12, X13).**
- [x] 14. Compose: the composition procedure over a retrieved concept graph
      (**after plans 01 and 04**); close #722 once plan 04's clause-splitting
      leaf has landed.
      **Done (2026-09-20). The procedure is `compose_document`
      (`src/source_capability.rs`): over the distinct content-addressed senses
      the discovery walk captured, it emits a title, the localized boundary
      note, each gloss verbatim with its `[n]` marker, and a sources footer
      naming every source, URL and licence — and nothing else, so a
      "few pages" request with one captured statement composes one statement
      (pinned: the document's prose is exactly the boundary note plus the
      captured glosses). An empty graph composes nothing and says so in the
      response language (`compose_no_verified_capture`, four languages in
      `data/seed/multilingual-responses-agentic.lino`, alongside
      `compose_boundary_note` and `compose_sources_heading`). `execute` keeps
      the attributable LN graph in the event log (`compose:source_graph`) and
      answers with the document; composing without verified capture scores
      0.0. Plan 04's clause-splitting landed with its L6-L8 batch, and the
      `Привет, напиши мне эссе` decomposition case now reaches the act, so
      #722 closes.**
- [x] 15. Demonstrate: bind `language_name` to
      `SolverConfig::forced_response_language`; answer from a grounded record;
      close #724.
      **Done (2026-09-18). The conversation-established language binds at the
      same seam the #556 replay forces (`solve_with_history`'s forced-language
      guard, fed by `established_response_language`, seed-grounded through the
      response-language marker role), so a demonstrated language keeps
      answering in itself: `tests/unit/issue_724_response_language_binding.rs`.
      Two collisions fell out of landing: `explain_previous_turn` now yields to
      the #556 replay before re-rendering (an explicit retarget of the previous
      turn outranks re-rendering it), and the demonstrate act no longer carries
      the bare response-language markers ("in russian", "по русски", "en ruso",
      "用") as act evidence — those belong to the marker role, so a language
      reference is the *object* only when an act (demonstrate, transform) makes
      the language the goal; "Tell me about Telegram Ads in Russian" is about
      Telegram Ads, in Russian. The 420-case suite holds at 420/420,
      `cross_tool_misroutes 0`, `silent_unknowns 0`, with the act's verb
      coverage widened where held-out cases needed real verbs.**
- [ ] 16. Schedule: merge the calendar verbs into the `schedule` act; delete the
      `calendar_fallback_verbs`, `calendar_digit_actions` and
      `calendar_ru_date_marker` cue sets (`data/meta/cue-lexicon.lino:85-110`);
      resolve the timezone reference through concept lookup (**after plan 01**);
      close #869.
      **Partial (2026-09-18). The three cue sets are deleted
      (`data/meta/cue-lexicon.lino`) with the schedule act verified as their
      merge target (`tests/unit/specification/cue_lexicon.rs` pins the
      retirement); the timezone-through-concept-lookup half is still open.**
- [ ] 17. Measure: `concept_measurement_lookup` over a retrieved property with a
      unit (**after plans 01 and 04**); close #1063.

**Make the planner's order data (Option D).**

- [x] 18. Name every arm in `src/agentic_coding/planner.rs:206-650`; create
      `data/seed/planner-precedence.lino`; add the load-time permutation
      assertion mirroring `src/solver_dispatch.rs:399-407`.
      **Done (2026-09-18). All 59 route arms (5 chat-step + 54 settled) are
      named in `PLANNER_ROUTE_ARMS` in run order; the seed
      `data/seed/planner-precedence.lino` is loaded through the seed network
      (`src/seed/planner_precedence.rs`) and joined with the coded table by an
      exact ordered-permutation assertion
      (`checked_route_precedence`, run first in `plan_chat_step_routes`);
      `tests/unit/issue_1138_planner_precedence.rs` proves the join and that
      swapped/dropped rows are rejected. `planner_route_arms` is recorded
      honestly at 59 (the checker's own counting rule; the 58 noted earlier the
      same day was a miscount, corrected in the ledger rather than silently).**
- [ ] 19. Retire each arm the capability table now covers, lowering
      `planner_route_arms` in the same commit.
      **Partial (2026-09-20). The `web_fetch` arm is retired: the decision
      table's five `url` rows decide that class at the named-or-local stage
      with the same `fetch_arguments` lowering, `PLANNER_ROUTE_ARMS` and
      `data/seed/planner-precedence.lino` are edited together, and the ratchet
      records 58 with the 420-case suite still at 420/0/0. Four further arms
      were drafted-and-restored with the failing evidence named in the ledger:
      `workspace_inspection` (a scope-noun-less question derives locus web, so
      the table routes it to `web_search` — the #1066 defect), `file_read` (a
      recipe arm; the `read_file` row's shell fallback has no lowering for a
      client without the typed read tool), `code_search_fallback` (grep-only
      clients the table honest-gaps), and `intent_web_search` (the
      `tool_search` discovery call no row can express). Retiring those four
      needs table rows or lowerings that do not exist yet, which is future
      leaf work rather than this leaf's `now covers`.**

**Retire the phrase book.**

- [ ] 20. Migrate `data/seed/intent-routing.lino`'s 477 lines of `phrase` /
      `keyword` rows onto acts and objects, family by family, lowering
      `intent_routing_phrase_rows` per commit; each family ships its held-out
      paraphrases first.
      **Partial (2026-09-20). Family one, `http_fetch`, is retired: its
      eleven phrases and its `fetch` token row left the file (the measure
      reads 343, down from 354; the token row is not counted by it), the
      family block keeps its `slug` and `response_link`, and a URL-bearing
      fetch request is decided by the `url` object at the named-or-local
      stage plus the shared `http_fetch` role surfaces of
      `meanings-web-navigation.lino`. The held-out paraphrases shipped first
      in `tests/unit/issue_1138_intent_phrase_migration.rs`: English,
      Russian, Hindi and Chinese wrappings no row ever named each reach the
      same answer and confidence the same language's canonical phrasing
      reaches, before and after the deletion — derivation over memoization,
      pinned both sides of the commit. A row-count guard keeps the family
      from growing back. The exact-match rows only ever fired on whole-prompt
      equality, so their deletion changes no URL-bearing prompt's route.
      Family two, `web_search`, is retired the same day: its six phrase rows
      were each a duplicate of a `web_search_explicit_prefix` surface in
      `meanings-web-search-query.lino`, a surface-plus-query prompt never
      matched a row for the same whole-prompt-equality reason, the measure
      reads 337, and the same test file pins Hindi and Chinese search
      prompts that no row ever covered as derived before and after.
      Drafting family two exposed one honest debt, recorded in the ratchet
      note rather than papered over: the solver's web-search seed carries
      no Spanish lexeme, so an es search prompt reaches `web_search`
      nowhere in the intent derivation and answers as an unresolved
      concept lookup — closed later the same day by seeding the es
      lexeme of `web_search_explicit_prefix`
      (`data/seed/meanings-web-search-query.lino`), so es search
      prompts now derive with no row and no per-prompt code, pinned as
      the gap's closing in the migration suite.
      Family three, `url_navigate`, retired the same day after its bare
      verbs were re-examined: the family's thirty-seven rows were all
      bare lead forms duplicating `url_navigate` role prefix surfaces, a
      navigation prompt carries a host so whole-prompt equality never
      fired for one, the issue #125 suite's twenty-nine host-bearing
      prompts still route to `url_navigate` with the rows gone, and the
      measure reads 300. The held-out paraphrases vary the tail after a
      seeded lead (`open the page github.com and wait`,
      `открой ссылку github.com если не сложно`) and are pinned before
      and after. The recon for the remaining families found their
      boundary: `greeting`, `wellbeing`, `farewell`, `courtesy_response`,
      `test_status`, `assistant_name`, `identity` and
      `assistant_free_time` are load-bearing — their rows are bare
      whole prompts (`hi`, `как дела`) with no object to derive from,
      and the table is their only derivation, so retiring them requires
      seeding the conversational act/role derivation first (future
      family work, not a row deletion). `assistant_free_time` is
      additionally coupled to `data/seed/handler-rules.lino`'s
      `route_exact` condition and must migrate with its rule together.
      Families four through ten retired the eight conversational
      families the same day, and the coupling drafted above became the
      mechanism: each family's exact surfaces now live as a seeded
      conversation role's word inventory in
      `data/seed/meanings-conversation.lino` (the retired rows verbatim,
      plus a Spanish inventory no row ever held), and each family's own
      intent block declares the role with a `role_surface` field — the
      single retirement declaration. `IntentRoute.role_surfaces` carries
      it in both parsers (and the `tests/source` mirror), a
      `declared_role_surface_route` pass consults the declared roles
      ahead of the table scan in `route_for_prompt` so the retired
      families keep the file-order precedence their blocks held —
      `greeting`'s rows sat ahead of `write_program`'s `keyword hello`,
      a row shadowed since it was written, and the pass keeps that
      outcome — the rule interpreter's `route_exact` projection merges
      each role's surfaces into the route's surfaces so the capabilities
      rules' `assistant_free_time` veto keeps deciding on the same whole
      prompts, and the worker mirrors parse and match the declaration
      (`seed_loader.js` `roleSurfaces`, `matchesIntentRoute`'s
      whole-prompt fallback, wasm keyword-line serialization — identical
      exact-equality semantics, no wasm export change). 256 rows left:
      greeting 34, wellbeing 27, farewell 27, courtesy_response 41,
      test_status 44, assistant_name 24, identity 38,
      assistant_free_time 21; the measure reads 44, and each family's
      count is pinned by a guard test. The migration suite asserts the
      retired prompts in all five languages plus the new es surfaces —
      the retirement's generalization — and records five prompts the
      rows never decided at answer time (`how is it going`,
      `आपका नाम क्या है`, `你叫什么名字`, `आप कौन हैं`, `你是谁`):
      the `how_it_works`, `set_assistant_name` and `who_is_question`
      handlers claimed them even while the rows existed, a finding the
      drafting surfaced rather than fixed. What remains for this leaf:
      the `greet` token and the `test_status`, `assistant_name` and
      `identity` combos are contains matches — separate behavior with
      their own retirement to draft — and `write_program`'s four keyword
      rows with their shadowing to untangle. The es web-search lexeme
      debt from family two closed the same day in the seed, pinned as
      the gap's closing in the migration suite.**

**The ladder and the UI defect.**

- [ ] 21. Append the seven frontier prompts to
      `experiments/issue_840_task_ladder/tasks.json` with new stable IDs; add
      hi/zh/es nodes for the three maintainer prompts; regenerate
      `results.json`; move the route-only ladder into the `rust` CI stage.
      **Partial (2026-09-18). Sixteen nodes appended with new stable IDs
      (`1138.frontier.news`, `1138.frontier.non_understanding`,
      `1138.frontier.compose`, `1138.frontier.demonstrate`,
      `1138.frontier.schedule`, `1138.frontier.measurement`,
      `1138.frontier.ui_complaint`, plus `838.L4.e`–`838.L4.m` for the hi/zh/es
      maintainer translations), append-only with route-level
      expect/forbid-tool assertions. A live `formal-ai serve` ladder
      measurement run (2026-09-19, results redirected to `/tmp`, committed
      `results.json` untouched) reads **7/7 on the `1138.frontier` nodes** and
      **7/13 on `838.L4`**: the six reds are hi/es folder-search translations
      missing the `hive-control-center` capability token at L4 (zh passes) —
      an honest engine gap below the routing layer, so the ratchet does not
      advance and `results.json` stays as committed.
      Continued (2026-09-19): the route-only replay is drafted and green —
      `tests/unit/issue_840_route_ladder.rs` ports `ladder.py` one-to-one (the
      same chat-completions seam, tool definitions, client-side execution
      contract, four-step cap, and the route-level judge fields) over every
      committed node, which moves the route-only gate into the `rust` CI
      stage. Its first run caught 7 violations across 6 nodes — 4 dictionary
      nodes (827.L1, 827.L3.a, 826.L2.a, 826.L3.a) planning no search and the
      hi/es folder nodes misrouted — and root-causing corrected the note
      above: the gap was *in* the routing layer, not below it. The decision
      table classified every red prompt correctly; three planner gates kept
      the rows from being consulted. Fixed in the derivations and gates, all
      from seed data: a response-language obligation ("Answer in English")
      no longer steals the subject from the concept question it modifies
      (`is_response_language_obligation`, leaf 15's own doctrine); the
      open-web gate releases a definition the seed's concept lookup cannot
      resolve, the honest unknown, while "What is Links Notation?" stays with
      the symbolic engine (#989 pin re-run green); the #907 container rule
      now declines only the *default* retrieval, so "Busca … en mi
      escritorio" / "… खोजिए" reach the same `list_dir`→shell lowering as the
      English node; and the table's generic listing lowering defers to a more
      specific seed shell intent ("What is current directory?" → `pwd`, not
      `ls`), which also keeps the obligation ledger off shell-owned
      single-command requests. The replay, #989, #907, the frontier-class
      pins, and the obligation suites all read green locally on the throttled
      runner; `results.json` stays as committed.**
      
      Continued again (2026-09-19, later): the broad regression net left 6
      `issue_1066` failures and a HEAD-baseline rerun adjudicated them — 4
      pre-existing branch regressions, 2 introduced by the fixes above, every
      one now green. The pre-existing four: `verify-node.sh` ran
      `cargo check`/`cargo test` unconditionally and read a fixture that is
      deliberately not a Rust package as an *uncompilable change* — both
      gates now record `compile unavailable` when the workspace has no
      `Cargo.toml`, the same admission they already made for a missing cargo
      binary; and the #1066 open-web query test ("Three different routes form
      an open-web query") was red on all three routes at HEAD: the routed
      Search arm's no-derivation fallback returned the raw first request
      block *with its sentence-final period*, `intent_router` never cleaned
      punctuation, and the #989 hold refused the current-fact row
      (`(bare_term, retrieve, web)` → "Verify the current exchange rate
      …") that the table itself had decided. The hold is now exactly the
      doctrine: it keeps only what the symbolic engine can *answer* — a
      concept the seed lookup resolves, or a computation
      (`calculation_expression_candidates`) — while an unresolved concept
      ("What is a hash-consed trie?" → `hash consed trie`), a current-fact
      routing, and an explicit web request all release, and every route
      derives its query through one normalization
      (`open_web_query_for_block` / `stated_web_search_query_for_block`):
      concept questions search their term, no query carries the block's
      punctuation, and no query carries the worker-placement block. The two
      this session introduced: the computation release above initially
      downgraded "What is 480 divided by 15?" to a web search instead of the
      engine's `write` (the symbolic-engine-answer pin), and the named
      grep position claimed `(bare_term, retrieve, workspace)` rows ahead of
      the workspace-inspection route, falling back to a whole-sentence grep
      instead of the canonical literal query — the routed grep now consults
      the inspection subject rule first. The obligations gate uses the
      *semantic* shell resolver, so "copy" inside "a fresh repository copy"
      no longer hides the ledger arm. The new helpers first landed inside
      `src/solver_handlers/web_search_intent.rs`, which grew the two census
      files past their reviewed baselines and failed the core-boundary gate
      — and raising a baseline for newly written code is new debt, the one
      thing the #918 ratchet exists to refuse. The honest home is beside
      their only callers: the three helpers are generic routing
      orchestration (concept extraction over seed lookup, punctuation
      cleaning), not search-domain vocabulary, so they moved to
      `src/agentic_coding/web_research.rs` under the same promotion logic
      plan 09 leaf 18 applied to the concept-lookup orchestration, and both
      census files returned to exactly their reviewed baselines — the gate
      reads 51 sources, 19384 outside-core lines, ceilings untouched.
- [x] 22. #447 routing half: the `(self_surface, record, self)` row and the
      report artifact.
- [ ] 23. #447 defect half: replace the splitter handle with a thin
      hover-highlighted resizer with a horizontal-resize cursor; verify left-panel
      scrolling with a mouse on Windows/Firefox and on a phone; commit
      screenshots to `docs/case-studies/issue-1138/`.
- [ ] 24. Reopen or re-file #745 and #758 with the committed measurement, and
      close them only when `cross_tool_misroutes` and `silent_unknowns` read 0
      against the 420-case suite.

---

## Docs to update

The exact quoted statements and their replacement text moved to plan 11's
findings table on 2026-09-16, so there is one docs authority and no document
is described in two places (plan 00 §8). This plan's entries are rows
**D249-D256** of
[`11-docs-consistency-audit.md`](11-docs-consistency-audit.md) §"Issue #1138
plan doc replacements", and plan 11's leaves apply them after the ledger rows
they cite exist (plan 00 §7).

| row | document |
| --- | --- |
| D249 | `ROADMAP.md` |
| D250 | `VISION.md:337` |
| D251 | `ARCHITECTURE.md:160-165` |
| D252 | `docs/requirements/issue-0745-intent-routing-generalization.md` |
| D253 | `docs/requirements-traceability.md` |
| D254 | `experiments/issue_840_task_ladder/README.txt` |
| D255 | `tests/fixtures/routing-parity.lino` |
| D256 | `data/meta/learning-frontier-language-gap.lino` and
`data/meta/language-adoption-ledger.lino` |

Any further document this plan's implementation touches is added as a new
plan 11 row, never as a second copy here.

## Risks and open questions

1. **The act vocabulary is still per-language surfaces.** Eight acts × five
   languages is 40 role blocks, and a verb the list misses still fails — the
   same failure mode, one level up. It is a genuine reduction (280 phrases → 40
   act blocks shared by every capability, so `найди` serves file search, code
   search and web search at once), not an elimination. Open question: whether
   the act can be derived from the object plus a morphological imperative test
   in at least the four non-CJK languages, removing the last verb list. To be
   revisited after leaf 11 measures how many misroutes are act-misses rather
   than object-misses.

2. **Object-type derivation can be wrong in the CJK scripts.** `path` detection
   leans on separators and extensions, which survive script changes, but
   `bare_term` segmentation in unspaced Chinese uses a different branch
   (`local_search.rs:387`, `contains_cjk`). A Chinese prompt whose object is a
   multi-character term may segment differently from its Russian twin. Leaf 4's
   test asserts *the same object from five translations of the same prompt*
   precisely to catch this, and it is the leaf most likely to need a second
   pass.

3. **Deleting `intent-routing.lino` will move behaviour that no suite covers.**
   477 lines of greeting, wellbeing, farewell and identity phrases back the
   conversational-variation suite
   (`data/benchmarks/conversational-variations-suite.lino`,
   `minimum_pass_count "228"`). Leaf 20 must proceed family by family with that
   suite green at every commit, and the suite must gain Spanish before the
   Spanish rows are relied upon.

4. **Two reported prompts are pinned regressions.** `Назначь мне встречу…` and
   `Какого размера средний корень яблони?` appear verbatim in the benchmark
   suite. That is legitimate — they are the reported defects — but it means
   those two strings will exist in `data/benchmarks/`. The no-memorization gate
   must scan `data/seed/` and `src/`, not `data/benchmarks/`, and the class must
   additionally pass eleven held-out siblings, so a fix that only satisfies the
   pinned string fails.

5. **Leaves 12, 14, 16 and 17 are blocked on plans 01 and 04.** News needs live
   retrieval; compose and measure need formalized concepts; scheduling needs a
   timezone resolved by concept lookup. Building a stub for any of them would be
   a bypass. If those plans slip, the correct order is leaves 13, 15, 18–23
   first — non-understanding, demonstrate, the planner order, the ladder and the
   UI — none of which has that dependency.

6. **`ask` can become the new silent failure.** If the table is sparse, the
   router asks constantly and the experience is worse than a guess. The ratchet
   must therefore also record `ask_rate` over the benchmark suites, with a
   declared acceptable band rather than a target of zero — a genuinely ambiguous
   prompt *should* ask. Open question: what the band is. To be set from the
   first measurement in leaf 10, not chosen in advance.

7. **The browser and the CLI are still two implementations.** Four of the seven
   frontier reports (#720, #721, #722, #724) came from the wasm build and two
   (#869, #1063) from wasm as well; only the maintainer's three came from the
   server. A capability table that only the Rust planner reads fixes none of
   them for the reporters. This plan depends on plan 09 leaves 13–15 (the worker
   deriving its handler list from seed) for the browser half, and leaf 21's
   ladder nodes must be run in both runtimes.

8. **Re-opening #745 and #758 is a process question, not a technical one.** Both
   are closed COMPLETED. Leaf 24 proposes re-filing rather than reopening if the
   maintainer prefers; either way the closure must be tied to the committed
   number, which is the lesson #710 recorded: "both issues were closed COMPLETED
   on acceptance evidence that asserted on the *shape of a plan* rather than on
   *measured routing outcomes*".

## 2026-09-19. The two #904-follow-up pins moved with the structured read

The bulk waves landing (`9c31b0530`) made the structured `gh issue view`
read the first-choice work-item read whenever the client can run commands,
with the model-backed `web_fetch` kept as the fallback for clients without a
shell (`src/agentic_coding/general_execution.rs::plan_work_item_read`). Two
pins still asserted the old fetch-first order —
`solve_issue_request_reads_the_work_item_before_project_lookup` (issue #1069)
and `compound_github_work_item_routes_to_agentic_planning_before_project_lookup`
(#698 replay) — and failed at tip. They are re-pinned to the gh-first order,
not reverted, because the rationale is documented and better: the CLI read
stays in the checkout's credentials and cannot recursively solve the issue
inside a nested model prompt. The #698 replay stays deterministic everywhere
because the driver's default-deny allowlist
(`data/seed/repository-command-allowlist.lino`, plan 03 L3) refuses `gh`, so
the replay exercises read-refused → fetch-fallback → record honestly.

Making that replay honest exposed one real defect, fixed here: the in-repo
driver reported a never-ran command (`run_command produced no result …`) as a
*successful* tool result, so `Progress::scan` stored the transport message as
the issue's fetched page and `plan_work_item_read` never fell back to the
fetch capability — the documented "empty or failed CLI read falls back"
contract (`src/agentic_coding/progress.rs`, `attempted_work_item_reads`) was
silently dead for refused reads. The driver now flags refused/unsupported
calls with the protocol's error form (`ChatMessage::tool_result_error`), the
same `is_error` signal an external Agent CLI sends, and the fallback fires.

## 2026-09-20. Three routing regressions the word-boundary fix unmasked

The plan 03 wave-F word-boundary fix stopped substring false positives, and
the batch verification surfaced three requests that had only ever reached
their right answer *through* such a false positive. Each fix moves the
ownership question to the component that owns it, rather than restoring the
false positive.

**The capability table may not answer a request the policy layer owns.**
"Improve my codebase forever" used to reach the bounded-autonomy refusal via
"improve" embedding "prove" promoting a proof handler, which made
`try_capability_route` decline to a promoted interpreter. With the false
positive gone, the table read a grep-shaped gap off the prompt and answered
`capability:grep` before `handle_policy` ever ran. `try_capability_route`
now declines up front on unbounded-autonomy phrasing without an agent
opt-in, and on agent requests generally (an opted-in `[agent]` request is
the agent flow's to serve, never a chat capability gap), so `handle_policy`
answers both classes (`src/meta_method_dispatch.rs`). The four
`agent_isolation` pins (opted_in, time budget, destructive confirmation)
and the autonomy CLI probe verify the ownership, 13/13.

**RelativePeriod's hour evidence is read token-bounded.** Spanish
"¿Cuántos temas distintos llevamos hasta ahora?" routed to a web digest
because the raw substring reading of the hour role turned "ahora" (now)
into "hora" (hour). The note now reads the role through
`seed::lexicon().mentions_role`, the same word-boundary rule wave F
established for handler promotion, and the digest route still fires on
real hour-anchored prompts (es_dialogue_state_query_05 was the last
held-out failure; the family migration suite is 300/300).

**Structure surfaces that over-claim cross-linguistically are removed at
the seed.** The new `integer_at_least` meaning initially carried the bare
"at least"-class stems (en "at least", hi "कम से कम", zh "至少", es
"al menos"), which discovered the comparison structure inside quantifier
phrases — "at least one distinct pair" is `quantifier_any`'s to read, and
`quantifier_any` already owns the full phrase in every language. The
meaning keeps only the unambiguous comparison forms (en "no less than"/"no
fewer than", ru "не менее"/"не меньше", zh "不少于", hi "से कम नहीं", es
"no menos de"), the ru pattern that already behaved correctly; likewise
`conditional_expression`'s es lexeme keeps "de lo contrario" and drops
"si no", which fired inside "o -1 si no está" where no other language's
equivalent is a surface. The five-language paraphrase suite shares one
concept-map identity per family again, 6/6.
