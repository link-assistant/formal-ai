# Case study — Issue #341: a decomposed agent step ("test it …") falls out of the project dialogue

> The user asked the agent to **design a Python web scraper** *and then* **test
> it against wikipedia.org**. Agent mode split the prompt into two steps on the
> `Then` separator. Step 1 was formalized correctly as a
> `software_project_request`. **Step 2 — "test it by scraping wikipedia.org and
> show me the top 10 most frequent words" — was misrouted**: in the deployed
> offline WASM worker it produced the *unknown-intent opener*; on current `main`
> it produces a *`wikipedia` concept lookup*. Both answers throw away the
> connection to step 1. This study reconstructs the timeline, enumerates every
> requirement, finds the root cause, surveys existing libraries, and records the
> implemented fix and its verification.

- **Issue:** [#341](https://github.com/link-assistant/formal-ai/issues/341) — *Issue with dialog: Design a simple web scraper in Python that: …*
- **Reported version:** 0.149.0 · WASM worker · manual mode · UI language `ru` · locale `ru-RU` (`Asia/Yekaterinburg`) · diagnostics on
- **Pull request:** [#345](https://github.com/link-assistant/formal-ai/pull/345) (branch `issue-341-7e64bf1210ad`)
- **Predecessors:**
  - [#27](https://github.com/link-assistant/formal-ai/issues/27) — agent-mode task decomposition (`decomposeAgentTask`). Introduced the multi-step split that surfaces this bug.
  - [#80](https://github.com/link-assistant/formal-ai/issues/80) — generic `software_project_request` handler (plan → approve → implement). The dialogue this follow-up extends.
- **Raw data:** [`raw-data/`](./raw-data/) — `issue.json`, `issue-comments.json`, `pr-345.json`, `reproduction-dialog.md`.

---

## 1. Timeline / sequence of events

| When (UTC) | Event |
| --- | --- |
| Issue #27 | Agent mode learns to decompose a multi-step prompt into sequential steps on a small separator set (`;`, `then`, `потом`, `затем`, `然后`, …). Each step is solved independently and the per-step answers are merged. |
| Issue #80 | The generic `software_project_request` handler lands: a build/design prompt is formalized into a `lino` meaning record, a reviewable plan, and approval gates; code is only emitted after `approve plan`. |
| 2026-05-29 18:44 UTC | In the GitHub Pages WASM worker the user sends the two-part scraper prompt. Agent mode splits it on `Then`. Step 1 → correct `software_project_plan`. **Step 2 → unknown-intent opener.** Issue #341 is filed. |
| 2026-05-29 18:52 UTC | Maintainer comment asks to generalize problem-solving/coding skills, compile a deep case study under `docs/case-studies/issue-341/`, find root causes, propose solutions, survey libraries, add debug output where data is missing, and fix the issue everywhere it occurs — all in one PR. |
| 2026-05-29 (this PR) | Root cause found and fixed with a dedicated `software_project_followup` handler, regression tests, a runnable reproduction example, and this case study (PR #345). |

The only issue comment is the maintainer's process comment (see
`raw-data/issue-comments.json`); every product requirement comes from the issue
body and the reproduction dialog.

---

## 2. Requirements (every explicit and implicit ask)

### From the reported dialog
1. **R1 — Keep follow-up steps inside the active task.** When agent mode splits
   "Design X **then** test X", the second step ("test it …") must stay bound to
   the software project from step 1 instead of being treated as an unrelated
   prompt.
2. **R2 — No misrouting to a concept lookup.** "scraping **wikipedia.org**" must
   not resolve the *Wikipedia encyclopedia* concept; the user is not asking what
   Wikipedia is.
3. **R3 — No unknown-intent dead end.** Offline, the step must not fall to the
   "I cannot answer that from local Links Notation rules yet" opener.
4. **R4 — Honour the approval-gated stance.** The system cannot actually fetch a
   live page or run code in the sandbox, so a "test it / run it" follow-up must
   be **formalized** (recorded as a verification goal behind approval gates),
   not silently executed.

### From the maintainer comment (process requirements)
5. **R5 — Generalize coding skills.** Increase the generality of the project's
   problem-solving so more programming tasks are handled (vision alignment).
6. **R6 — Deep case study.** Download issue/PR logs and data into
   `docs/case-studies/issue-341/`, reconstruct the timeline, list every
   requirement, find root causes, propose solutions, and survey existing
   libraries/components.
7. **R7 — Online research.** Search online for additional facts and relevant
   prior art.
8. **R8 — Debug/verbose where data is missing.** If there is not enough data to
   find the root cause, add debug output / a verbose mode so the next iteration
   can. (Here the root cause was reproducible directly — see §3 — so a runnable
   example was added instead of new logging.)
9. **R9 — Report to other repositories if relevant.** File issues upstream with
   reproducible examples and fix suggestions where the bug belongs to another
   project. (Not applicable — see §6.)
10. **R10 — Fix everywhere.** Apply the fix to the whole codebase, not just one
    site, including any worker mirror.
11. **R11 — Single PR with reproducible examples.** Plan and execute everything
    in PR #345 with reproducible examples, workarounds, and code fix
    suggestions.

---

## 3. Root-cause analysis

### How a step is routed

Agent mode lives in the browser (`src/web/app.js`):

- `decomposeAgentTask(text)` splits on `AGENT_STEP_SEPARATORS` (one of which is
  `/\s+then(?:\s*,)?\s+/i`). The scraper prompt becomes two steps.
- `runAgentPlan(steps, history)` solves each step **through the same solver the
  chat path uses**, threading the running conversation `history` into each call.
  So step 2 is solved with step 1's user prompt + assistant plan already in the
  log.

Every step ends up in the Rust universal solver
(`UniversalSolver::solve_with_history`), which walks an **ordered dispatch
table** (`SPECIALIZED_HANDLERS` in `src/solver.rs`). The first handler that
returns `Some` wins.

### Cause — there was no follow-up handler, and the project handler is gated and ordered late

Two facts combine to misroute step 2:

1. **`try_software_project_request` only recognizes *new* projects or the literal
   approval prompt.** Its meaning parser
   (`SoftwareProjectMeaning::from_prompt`) requires *both* an **action word**
   (`design`, `build`, `write`, …) **and** an **artifact phrase** (`scraper`,
   `app`, …). Step 2 — "test it by scraping wikipedia.org …" — has neither a
   recognized action word (`test` is not in the list) nor a fresh artifact, so
   `from_prompt` returns `None`. The only context-aware branch
   (`prior_software_project_meaning`) was reachable **only** when the prompt was
   an exact approval phrase (`is_approval_prompt`), which "test it …" is not.

2. **`software_project` sits *after* `concept_lookup` in the dispatch table.**
   Even if the project handler had matched, `concept_lookup` (which matches the
   seeded `wikipedia` concept against "wikipedia.org") runs first and wins. In
   the offline deployed worker the seed/cache differ enough that no handler
   matched at all, so the prompt reached the unknown-intent opener.

So the verb that signals *"exercise the thing we just designed"* (`test`,
`run`, `verify`, `show me`) had **no home**. The fix introduces that home and
gives it precedence over the general lookups while a project dialogue is active.

> R8 note: the failure was directly reproducible (see the runnable example in
> §7), so no new always-on logging was required. The reproduction example
> doubles as the diagnostic harness for future regressions.

---

## 4. The fix

A new handler `try_software_project_followup`
(`src/solver_handlers/software_project.rs`), registered in `SPECIALIZED_HANDLERS`
**immediately after `conversation_memory` and before `summarization` /
`concept_lookup`**:

- Fires **only** when the previous assistant turn already formalized a
  `software_project_request` (recovered by `prior_software_project_dialogue`,
  which also reports whether the plan was already approved). Outside an active
  project dialogue the handler is inert, so unrelated prompts are untouched.
- Skips approval prompts so `approve plan` still advances to the implementation
  starter via the original handler.
- Detects the follow-up verb and classifies it:
  `verification` (`test it`, `verify`, `протестируй`, `测试`, `परीक्षण`, …),
  `execution` (`run it`, `запусти`, `运行`, `चलाओ`, …), or
  `demonstration` (`show me`, `покажи`, `显示`, `दिखाओ`, …) — across all
  supported languages (en, ru, hi, zh), so a user who designs in one language
  and then asks to exercise the artifact in another stays inside the dialogue
  (R5).
- Extracts the **target site** (first domain-like token, e.g. `wikipedia.org`)
  and the **expected output** (the clause after `show me`, e.g. "the top 10
  most frequent words").
- Formalizes a `software_project_followup` `lino` meaning record linking back to
  the parent request, and answers with reasoning steps + a verification plan,
  keeping live fetches and code execution behind **`generated_code`**,
  **`test_execution`**, and **`network_access`** approval gates (R4).

### Before / after (offline, the reported environment)

```
# before
=== STEP 2 intent: concept_lookup ===
Wikipedia (encyclopedia): Wikipedia is a free, multilingual online encyclopedia …

# after
=== STEP 2 intent: software_project_followup ===
Recorded a verification follow-up for the scraper from the active plan.
… target_site "wikipedia.org" … expected_output "the top 10 most frequent words" …
```

### Applying the fix everywhere (R10)

The project deliberately maintains **two** copies of the solver that must stay
at parity:

1. **The Rust engine** (`src/solver.rs` + `src/solver_handlers/`) — used by the
   CLI, HTTP server, library surface, and compiled to WASM for in-browser
   helper calls.
2. **The browser worker** (`src/web/formal_ai_worker.js`) — a hand-written
   JavaScript mirror that drives the deployed GitHub Pages chat. It owns its own
   ordered `syncHandlers` dispatch list and its own `trySoftwareProjectRequest`,
   `tryConceptLookup`, etc. (the `wasm*` calls are per-function accelerators with
   JS fallbacks, **not** a single delegation to the Rust solver).

So the bug existed in **both** dispatch tables — in the JS worker
`trySoftwareProjectRequest` is registered *after* `tryConceptLookup`, the same
ordering trap. The fix was therefore mirrored:

- Rust: `try_software_project_followup` registered before `concept_lookup`.
- JS: `trySoftwareProjectFollowup` registered before `tryConceptLookup` in
  `syncHandlers`, producing byte-for-byte the same `lino` record and answer.

`decomposeAgentTask` in `src/web/app.js` is unchanged — the split itself is
correct; only the per-step routing was wrong.

---

## 5. Existing libraries / prior art (survey)

### For the *engine* change (what we built on)
- **`nom`** (already a dependency) — the meaning parsers
  (`parse_action_word`, `parse_artifact_phrase`) are `nom` combinators; the
  follow-up detector reuses the same boundary helpers.
- **The `software_project` dialogue from issue #80** — the follow-up record
  reuses the existing `delivery_mode`, `implementation_language`, approval-gate,
  and `lino` rendering vocabulary so the two turns read as one conversation.

### For the *task the user described* (what an approved plan should reference)
A simple Python heading-scraper + word-frequency tool maps cleanly to mature,
well-known libraries — useful context for the generated plan and for future
generalization (R5):

| Need | Standard option(s) |
| --- | --- |
| Fetch a webpage | [`requests`](https://requests.readthedocs.io/) or [`httpx`](https://www.python-httpx.org/); stdlib `urllib.request` for zero-dependency |
| Parse `h1/h2/h3` | [`beautifulsoup4`](https://www.crummy.com/software/BeautifulSoup/) over an `lxml`/`html.parser` backend; `soup.find_all(["h1","h2","h3"])` |
| Tokenize + count words | stdlib [`collections.Counter`](https://docs.python.org/3/library/collections.html#collections.Counter) (`Counter(words).most_common(10)`) + `re` for tokenizing; optional `nltk` stopwords |
| Markdown summary | plain f-strings, or [`markdownify`](https://pypi.org/project/markdownify/) / `jinja2` for templated output |
| Deterministic tests | `pytest` against a **captured HTML fixture** (no live network), exactly the gated plan the follow-up proposes |

The follow-up's verification plan deliberately recommends a **captured fixture**
first and live `wikipedia.org` only after the `network_access` gate — matching
the robots-friendly, reproducible-testing practice these libraries encourage.

---

## 6. Upstream / other-repository reports (R9)

Not applicable. The defect is entirely in this repository's solver dispatch and
software-project handler. No third-party library behaves incorrectly here — the
Python libraries in §5 are only *referenced* by the generated plan, never
invoked by the engine.

---

## 7. Verification

- **Runnable reproduction (Rust engine):**
  [`examples/repro_issue_341.rs`](../../../examples/repro_issue_341.rs)
  (`cargo run --example repro_issue_341`) — prints step-1 and step-2 intents in
  the offline configuration from the bug report. Before the fix step 2 was
  `concept_lookup`; after, it is `software_project_followup`.
- **Runnable reproduction (browser worker):**
  [`experiments/issue341_js_worker_repro.mjs`](../../../experiments/issue341_js_worker_repro.mjs)
  (`node experiments/issue341_js_worker_repro.mjs`) — loads
  `src/web/formal_ai_worker.js` in a VM sandbox and drives the same two-step
  dialogue through the JS `solve()`, asserting `software_project_followup` for
  the English prompt and the ru/hi/zh follow-ups.
- **Regression tests:** `tests/unit/software_project.rs`
  - `software_project_followup_keeps_test_step_in_the_project_dialogue` — the
    exact issue prompt now stays in the dialogue, records the target site and
    expected output, exposes the new approval gates, and contains **neither**
    the Wikipedia-concept text **nor** the unknown opener.
  - `software_project_followup_requires_an_active_plan` — "test it" with no prior
    plan does **not** trigger the handler (no false positives).
  - `software_project_followup_reports_approved_state_after_implementation` — a
    follow-up after `approve plan` reports the approved state.
  - `software_project_followup_detects_verbs_across_supported_languages` — design
    in English, then verify in en/ru/hi/zh; every follow-up routes correctly.
- **CI guards:** `check-language-test-coverage.mjs` (all four languages covered
  by the new tests) and `check-multilingual-intent-coverage.mjs` both pass.
- **Full suite + lints:** `cargo test` (690 green), `cargo clippy
  --all-targets` (clean), and `node --check src/web/formal_ai_worker.js`.

---

## 8. Follow-up opportunities (generalization, R5)

- Teach the follow-up handler to *advance* an already-approved project by
  appending the verification goal as a new requirement/subtask, so a later
  `approve plan` regenerates code that bundles the named test.
- Make the parent `software_project_request` itself multilingual (the meaning
  parser still needs English action+artifact words), so a fully non-English
  multi-step coding dialogue works end to end. The follow-up verbs are already
  multilingual (en/ru/hi/zh); the design step is the remaining gap.
- Generalize target/output extraction (multiple sites, multiple expected
  outputs, non-English "show me" markers) as more multi-step coding dialogs are
  collected.
