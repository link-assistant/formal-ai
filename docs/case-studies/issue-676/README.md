# Case study — Issue #676

> Some commands and prompts are not working in Formal AI, when accessed via OpenCode

- **Issue:** [link-assistant/formal-ai#676](https://github.com/link-assistant/formal-ai/issues/676)
- **Author:** `konard`
- **Opened:** 2026-07-13T00:07:20Z
- **Pull request:** [#678](https://github.com/link-assistant/formal-ai/pull/678) (branch `issue-676-ea3feeb860dc`)
- **Reporter environment:** [OpenCode](https://github.com/sst/opencode) `1.17.18`, connecting to a local
  `formal-ai` server through its OpenAI-compatible HTTP API (`formal-ai with opencode`).

This folder collects the raw evidence attached to the issue and a deep analysis of every
requirement, its root cause, and the solution plan. The screenshots referenced below are
stored under [`images/`](./images) (downloaded from the GitHub issue attachments — see
[`image-urls.txt`](./image-urls.txt)). The raw issue payload is in
[`issue-676.json`](./issue-676.json).

---

## 1. Timeline / sequence of events

| # | Event | Evidence |
|---|-------|----------|
| 1 | Reporter launches `formal-ai with opencode` — OpenCode drives the Formal AI server as an agentic backend over the OpenAI-compatible HTTP API. | issue body |
| 2 | `hi` → *"Hi, how may I help you?"* — greeting works. | `images/01-execute-pwd.png` |
| 3 | `execute pwd` → **unknown fallback** (*"I could not determine Execute pwd …"*), while `execute ls` had worked in an earlier session. Bash execution, and `pwd` specifically, is not supported in agentic mode. | `images/01-execute-pwd.png`, `images/05-self-healing.png` |
| 4 | `give me a list of files in current folder` → does not list files. | issue body (2nd screenshot) |
| 5 | `What is your name?` → *"I'm formal AI … you can name me as you like."*; then `Now your name is Ineffa` → **unknown fallback**. Naming does not work. | `images/03-naming.png` |
| 6 | `How are you?` → generic *"Hi, how may I help you?"*, identical to the reply for `Hello`. | `images/04-how-are-you.png` |
| 7 | `Can you fix it yourself?` (self-healing / debugging / learning) → **unknown fallback**. | `images/05-self-healing.png` |
| 8 | Reporter files issue #676 with the six screenshots and the broader requirements (variation coverage, auto-learning meta-algorithm, human-like recursive thinking display, case study, upstream reporting). | issue body |

### What the reporter's thoughts panel revealed

For `Hello` and `How are you?` the OpenCode "Thought" block shows the *same* robotic template
(`images/04-how-are-you.png`):

```
Read the request: "How are you?".
Detect the request language: English.
Formalize the request as a greeting task.     ← both are "greeting"
Route to the greeting handler.
Verify the result against the rules.
Compose the answer: "Hi, how may I help you?".
```

This is direct evidence that `How are you?` is being *classified as a greeting* and that the
thinking display is a fixed six-line template rather than a human-readable, per-intent
explanation.

---

## 2. Complete list of requirements extracted from the issue

Each requirement is given a stable ID used throughout the PR and the rest of this analysis.

| ID | Requirement (verbatim intent) | Kind |
|----|-------------------------------|------|
| **R1** | Fix bash execution in agentic mode, specifically `execute pwd` (`execute ls` worked, `execute pwd` did not). | Bug |
| **R2** | Fix `give me a list of files in current folder` (natural-language file listing). | Bug |
| **R3** | Fix naming: assistant offers "you can name me as you like", but `Now your name is Ineffa` fails to *unknown*. | Bug |
| **R4** | Fix `How are you?` — must not return the generic greeting identical to `Hello`. | Bug |
| **R5** | Fix self-healing / debugging / learning (`Can you fix it yourself?` fell to *unknown*). | Bug |
| **R6** | Generate as many variations of different kinds of messages as possible and support them all correctly. | Coverage |
| **R7** | Check how auto-learning works; make the meta-algorithm able to solve this task itself — any decomposition and the full task. | Meta |
| **R8** | Increase quality of the thinking display: more human-like, less robotic; understandable by any human; recursive; still able to show all robotic detail on demand. | UX |
| **R9** | Download all logs/data to `./docs/case-studies/issue-676`; deep case study: timeline, requirements list, root causes, solution plans, existing-component survey, online facts. | Docs |
| **R10** | If root cause data is insufficient, add debug output / verbose mode for the next iteration. | Infra |
| **R11** | If the issue is related to another repository (e.g. OpenCode), report it upstream with reproducible examples, workarounds, and code suggestions. | Upstream |
| **R12** | Apply the requirements to the **entire** codebase — if a problem exists in multiple places, fix it in all of them (Rust server **and** JS web worker parity). | Scope |
| **R13** | Execute everything in a single pull request (#678). | Process |

---

## 3. Architecture recap (where the relevant behaviour lives)

Formal AI is a **deterministic symbolic** system — there is no neural inference. A prompt is
matched against Links Notation (`.lino`) seed rules and routed to a fixed handler. Two runtime
surfaces must stay in parity (R12):

- **Rust server** (`src/…`) — the surface OpenCode talks to over HTTP. This is where every bug
  in the issue actually reproduces.
- **JS web worker** (`src/web/worker/formal_ai_worker_*.js`) — the browser demo. Kept in parity
  by CI (`check:language-parity`, `check:web-tdz`, the seed-sync guard).

Relevant pipelines:

- **Agentic planning** (R1, R2): `src/agentic_coding/planner.rs::plan_chat_step()` turns a prompt
  into `AgenticPlan::ToolCalls` (e.g. a `bash` tool call) or `AgenticPlan::Final`. OpenCode uses
  this path. `shell_command_for_task()` decides *which* shell command to emit.
- **Intent routing** (R3, R4, R5): `data/seed/intent-routing.lino` → `seed::intent_routing()` →
  `route_for_prompt()` / `matches_route()` in `src/intent_formalization.rs` → `SelectedRule` in
  `src/engine.rs` → a localized response from `data/seed/multilingual-responses.lino` via
  `src/engine_responses.rs`.
- **Thinking display** (R8): the reasoning trace shown by OpenCode as a "Thought" block.
- **Self-healing** (R5, R7): `src/self_healing.rs`, `src/learning_ledger.rs`, and the family of
  agentic recipes added under issue #558.

---

## 4. Root-cause analysis per requirement

### R1 — `execute pwd` not supported (agentic bash) — **root cause found & fixed**

`plan_chat_step()` recognised a shell request but `shell_command_for_task()` could only ever emit
the literal command `ls`. Its predecessor (`45a4ebe9 fix(agentic): emit shell tool calls for ls
requests`, issue #607/#624) special-cased directory-listing and hard-coded `"ls"`. Any other
command — `pwd`, `git status`, `whoami`, `cargo test` — matched no branch, so the planner fell
through to `AgenticPlan::Final` and produced the *unknown* reasoning-trace fallback.

**Root cause:** the shell-command extractor was not *general*. It ignored the rich
`shell_tokens` / `run_verbs` vocabulary already present in `data/seed/terminal-commands.lino`.

**Fix (committed `7b06094f`):** `shell_command_for_task()` is now data-driven. It reads
`seed::terminal_command_vocabulary()` and, for any prompt that names a known shell token after a
run verb (or mentions one in a run context), emits the real command plus its arguments. `execute
pwd` → `pwd`, `run git status` → `git status`, and the directory-listing shortcut still yields
`ls`.

### R2 — `give me a list of files in current folder` — **root cause found & fixed**

Same function. Natural-language directory-listing detection (`asks_for_directory_listing`) was too
narrow — it did not recognise phrasings like *"a list of files in current folder"*. Broadened the
phrase lists (`list of files`, `list all files`, `files in current`, `current folder`, `the
folder`, …) plus the local-scope check. Covered by the new unit test
`issue_676_planner_maps_natural_language_file_listing_to_ls`.

### R3 — Naming (`Now your name is Ineffa`) — **root cause found**

`data/seed/intent-routing.lino` has `intent_assistant_name` (answers *"what is your name"*) and an
`intent_recall_name`, but **no intent that SETS the name**. `src/engine_assistant_name.rs` only
holds static answers to name *questions*; there is no handler that captures *"your name is X"*,
stores it, and acknowledges it. So `Now your name is Ineffa` matched no route and fell to
*unknown*. The assistant literally invites "you can name me as you like" and then cannot honour it.

**Solution plan:** add a `set_assistant_name` intent + patterns across the four supported
languages (`your name is …`, `I'll call you …`, `let your name be …`, `назову тебя …`, `तुम्हारा
नाम … है`, `你叫 …`), a handler that extracts the name, an acknowledgement response
(*"Nice to meet you — you can call me Ineffa from now on."*), and reflect the stored name in the
`assistant_name` / `recall_name` answers. Persistence uses the existing dialog-local memory
mechanism (the same one the *unknown* fallback references with `When I say … answer …`).

### R4 — `How are you?` returns the generic greeting — **root cause found**

`intent-routing.lino` bundles the whole *how-are-you* family (`how are you`, `how are you doing`,
`как дела`, `कैसे हो`, `你好吗`, …) as **phrases of `intent_greeting`**. Greeting routes to
`response:greeting` → *"Hi, how may I help you?"* — the exact generic reply the reporter saw. The
thinking panel confirms it: both `Hello` and `How are you?` are "Formalize the request as a
greeting task."

A CI guard (`tests/e2e/scripts/check-multilingual-intent-coverage.mjs`, `howAreYouGreetingPhrases`)
*enforces* that these phrases stay in `intent_greeting`, so any fix must update the guard in
tandem.

**Solution plan:** introduce a distinct `wellbeing` intent (declared before `greeting` so it wins
first-match), move the *how-are-you* family there, add a warm `response:wellbeing`
(*"I'm doing great, thanks for asking! I'm ready to help — what would you like to do?"* and
localized variants), wire `SelectedRule::Wellbeing` through `engine.rs`/`engine_responses.rs`,
mirror it in the JS worker, and repoint the CI guard + `prompt-patterns.lino` to the new intent.

### R5 / R7 — Self-healing / auto-learning (`Can you fix it yourself?`) — **root cause found**

Self-healing exists (`src/self_healing.rs`, `src/learning_ledger.rs`, issue #558 agentic recipes),
but its trigger vocabulary does not recognise the natural phrasing `Can you fix it yourself?`, so
the router produced *unknown*. The meta-algorithm can already run the closed human-gated
self-healing loop through the agentic interface, but there was no conversational on-ramp from a
plain question.

**Solution plan:** add a `self_heal_request` intent + multilingual phrases (`can you fix it
yourself`, `fix it yourself`, `debug yourself`, `heal yourself`, `learn from this`, …) that routes
to an explanation of the self-healing capability and how to invoke the loop; verify the
meta-algorithm can decompose the sub-tasks. Keep the deeper recipe machinery reachable.

### R6 — Message variation coverage

The bugs above are all *generality* failures: a slightly different phrasing than the seed anticipated
falls to *unknown*. The fix for each requirement adds a broad set of phrasings (and unit tests that
assert many variants), directly serving R6.

### R8 — Robotic thinking display — **fixed**

The reasoning trace was a fixed six-line template ("Read the request … Detect the request language …
Formalize … Route … Verify … Compose the answer …") applied to every intent, which is why `Hello`
and `How are you?` looked identical apart from the quoted text. It was not human-like, not per-intent,
and offered no way to expand into the full "robotic" detail.

**Delivered solution:** the reasoning trace now leads with a per-intent, first-person narrative of
what the assistant understood and decided, with the concrete step list kept beneath it as a
recursive "robotic detail" layer.

- **Rust / API / CLI** — `thinking_narrative(&[ThinkingStep])` (in `src/thinking.rs`) maps the
  resolved route to one human sentence ("You said hello, so I greeted you back.", "This was a
  calculation, so I worked it out step by step and checked the result.", with a humanized generic
  fallback for any unrecognized route). `render_thinking_steps` prepends it, so the API `reasoning`
  field an agentic client such as OpenCode renders now opens human and per-intent while every
  concrete step (which the surface tests pin by substring) still renders below, including the
  recursive `↳` sub-steps. Covered by `tests/unit/issue_676_thinking_narrative.rs`.
- **Web** — `ThinkingPreview` renders the same headline in a dedicated always-visible
  `thinking-narrative` element above the collapsed/expanded step list (which already provides the
  brief/standard/detailed levels and a recursive expandable diagnostics panel). The 18 `narrative*`
  strings live in `src/web/i18n-catalog.lino` for all four locales (en/ru/zh/hi); `thinkingNarrative`
  keys off the resolved intent. Covered by `tests/e2e/tests/issue-676-thinking-narrative.spec.js`.

Both surfaces summarize the *decision* (a stable meta-language headline), so the same route reached
in any input language yields the same reasoning headline while the composed answer stays localized.

### R9–R13 — Process requirements

- **R9** — this document plus the raw evidence in this folder.
- **R10** — verbose/debug tracing is added where a root cause was not already obvious from the
  existing trace.
- **R11** — the agentic bash path is Formal-AI-side; OpenCode behaves correctly (it forwards the
  prompt and renders whatever the server returns). If a genuinely upstream defect is found while
  reproducing, it is reported with a minimal repro. See [`upstream.md`](./upstream.md).
- **R12** — every conversational fix is applied to both the Rust engine and the JS worker mirror,
  guarded by the existing parity CI checks.
- **R13** — all work lands in PR #678.

---

## 5. Existing components / libraries surveyed

Rather than inventing new infrastructure, the fixes reuse machinery that already exists in the repo:

| Need | Reused component | Location |
|------|------------------|----------|
| Shell token + run-verb vocabulary | `TerminalCommandVocabulary` | `data/seed/terminal-commands.lino`, `seed::terminal_command_vocabulary()` |
| Intent routing | `intent_routing()` / `route_for_prompt()` / `matches_route()` | `data/seed/intent-routing.lino`, `src/intent_formalization.rs` |
| Localized responses + variants | `multilingual-responses.lino` + `cached_response()` | `data/seed/multilingual-responses.lino`, `src/engine_responses.rs` |
| Dialog-local memory ("When I say … answer …") | dialog-local rule store | referenced in every *unknown* fallback |
| Self-healing / learning loop | self-healing + learning ledger + agentic recipes | `src/self_healing.rs`, `src/learning_ledger.rs`, issue #558 recipes |
| Multilingual parity enforcement | intent-coverage / language-parity CI | `tests/e2e/scripts/*` |

External references consulted for design (deterministic, no runtime dependency added):

- **OpenAI / Anthropic chat completion shapes** — how OpenCode expects `reasoning`/`thinking` and
  tool-call blocks, which constrains the thinking-display format (R8) and the agentic tool-call
  emission (R1/R2).
- **OpenCode** `sst/opencode` — confirms the client simply forwards prompts and renders the
  server's tool calls / reasoning, so the generality gap is on the Formal AI side (R11).
- **POSIX shell command vocabulary** — informs which `shell_tokens` are safe, well-known commands to
  recognise (`pwd`, `ls`, `git`, `cargo`, `whoami`, …).

---

## 6. Verification strategy

Each behavioural fix ships with a reproducing unit test *before* the fix (red → green):

- R1/R2 — `tests/unit/agentic_coding.rs`: `issue_676_planner_maps_execute_pwd_to_pwd_command`,
  `issue_676_planner_maps_natural_language_file_listing_to_ls`,
  `issue_676_planner_ignores_shell_tokens_without_run_context`.
- R3/R4/R5 — new unit tests asserting the set-name, wellbeing, and self-heal routes resolve, plus
  many phrasing variants (R6).
- R12 — `check:intent-coverage`, `check:language-parity`, `check:web-tdz`, and the seed-sync guard
  keep the Rust and JS surfaces aligned.

See the PR #678 description for the consolidated before/after table.
