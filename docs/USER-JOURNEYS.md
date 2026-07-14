# User Journeys

This document answers a question raised in issue
[#454](https://github.com/link-assistant/formal-ai/issues/454): reading
[`VISION.md`](../VISION.md) alone, it is not obvious **what pain Formal AI
closes** or **who would reach for it instead of a neural chatbot**. The vision
describes the *machine*; this document describes the *people who use it* and the
*moment of pain* that brings each of them to the project.

It is the single place that enumerates **every user journey we support today**
and **every journey we could plausibly support next**, so that the docs make a
concrete promise rather than an abstract architecture pitch. Each journey is
traceable to the implemented surfaces in [`README.md`](../README.md), the
principles in [`VISION.md`](../VISION.md), and the build status in
[`ROADMAP.md`](../ROADMAP.md).

## The Pain Formal AI Closes

Mainstream AI assistants are powerful, but they share a cluster of pains that
this project is built to remove:

1. **You cannot see why.** A neural model returns an answer, not its reasoning.
   When it is wrong, you cannot inspect the step that failed, and you cannot
   point at the evidence it used.
2. **It is not reproducible.** The same prompt can produce different answers,
   so a result you got yesterday may not return today, and a colleague running
   the same prompt may see something else.
3. **It needs a GPU and a vendor.** Useful answers require cloud inference on
   specialized hardware, which means cost, latency, network dependence, and
   handing your prompts to someone else's servers.
4. **You cannot own or move its memory.** What the assistant "knows" about your
   session lives in an opaque store you cannot export as one file, audit, or
   carry between devices and interfaces.
5. **You cannot teach it by editing data.** Changing its behavior means
   prompt-engineering around a frozen model, not adding a fact, retiring an
   answer, or registering a tool that the system then simply *uses*.

Formal AI's bet is that a large class of everyday requests — greetings,
translation, arithmetic, unit and currency conversion, fact lookup, code
generation, text editing — can be answered by **deterministic reasoning over an
inspectable network of links** instead of opaque neural inference. The same
prompt with the same configuration always yields the same answer, every step is
recorded in an append-only log you can interrogate, the whole state exports as
one Links Notation file, and behavior is defined by editable seed data rather
than hidden weights. See [`NON-GOALS.md`](../NON-GOALS.md) for what the project
deliberately does **not** try to be (for example, a GPU-backed neural model or
a memoized answer cache).

## Who This Is For

The journeys below revolve around a small set of personas. They are archetypes,
not literal accounts — a single real user often wears several of these hats.

| Persona | What they want | The pain they feel today |
| --- | --- | --- |
| **Dana, the auditor** | An answer she can defend, with the reasoning attached | Black-box answers she cannot verify or cite |
| **Mikhail, the multilingual user** | To work in Russian, Hindi, Chinese, or English and be understood equally | Assistants that treat non-English as second-class |
| **Ltoo, the learner** | A "hello world" or small program, with an honest note on whether it ran | Confident code that was never executed |
| **Priya, the analyst** | Fast math, currency, and unit conversions she can trust | Plausible numbers with no shown work |
| **Sven, the integrator** | A drop-in OpenAI-compatible endpoint with no GPU bill | Cloud lock-in, cost, and latency |
| **Olúwa, the privacy-conscious owner** | To export and move everything the assistant saw and did | Memory she cannot inspect or carry |
| **Wei, the tinkerer** | To change behavior by editing data, not retraining | Behavior frozen inside model weights |

## How To Read A Journey

Every journey is written in the same shape so they stay comparable:

- **Persona & pain** — who arrives and what hurts.
- **Trigger** — the first message or action.
- **Journey** — the ordered steps the user takes.
- **Why it is different** — what the symbolic, link-native core gives them that
  a neural assistant does not.
- **Surfaces** — where this works today (library, CLI, HTTP API, web demo,
  Telegram, desktop, VS Code).
- **Status** — `Supported today` or `Potential future`, with a pointer.

---

## Currently Supported Journeys

These journeys exercise capabilities that are built and tested on `main`. The
short transcripts are illustrative of the supported behavior; the exact wording
is data-driven and configurable through the seed.

### J1 — Dana asks "why did you answer that?"

- **Persona & pain:** Dana the auditor cannot ship an answer she cannot defend.
- **Trigger:** `What is 8% of $50?`
- **Journey:**
  1. Dana asks the arithmetic question and gets `$4`.
  2. She follows up: `Why did you answer that?`
  3. The assistant replies with a meta-explanation built from the most recent
     trace — the impulse it recorded, how it formalized the request, the
     decomposition, the validation check, and the final simplification.
  4. She asks `What do you know about percentages?` and gets the links involved.
- **Why it is different:** the answer is a *projection* of an append-only event
  log, not a one-off generation, so "why" is answerable from recorded
  experience rather than re-imagined after the fact.
- **Surfaces:** library, CLI, HTTP API, web demo, Telegram.
- **Status:** Supported today. See *Append-Only Event Log* in
  [`VISION.md`](../VISION.md#append-only-event-log).

### J2 — Mikhail works in his own language

- **Persona & pain:** Mikhail wants Russian, Hindi, and Chinese treated as
  first-class, not bolted on.
- **Trigger:** `Посчитай 1000 рублей в долларах`
- **Journey:**
  1. Mikhail asks a currency conversion in Russian and gets a Russian answer.
  2. He switches to `как у тебя дела?` and the assistant replies in kind.
  3. He asks the assistant to translate a phrase; the reply preserves his
     original capitalization and terminal punctuation (`как у тебя дела?` →
     `how are you?`, lowercase in, lowercase out).
- **Why it is different:** every operation is recognized equally across
  `en | ru | hi | zh` because the operation vocabulary lives in shared seed
  data (`data/seed/operation-vocabulary.lino`), and translation rides the same
  formalize → meaning → deformalize pipeline rather than a one-off rewrite.
- **Surfaces:** library, CLI, HTTP API, web demo, Telegram.
- **Status:** Supported today. See *Chat-first interface* and *Meaning And
  Identity* in [`VISION.md`](../VISION.md#meaning-and-identity).

### J3 — Lin asks for a "hello world"

- **Persona & pain:** Lin the learner wants real, runnable code and an honest
  note about whether it was executed.
- **Trigger:** `Write me hello world program in Rust`
- **Journey:**
  1. Lin asks for the program and receives the code block.
  2. The reply states whether the code was compiled, run, not run, or cannot be
     run in the current environment.
  3. Lin follows up `now in Python` and the assistant reuses the active program
     context to answer the parametric request.
- **Why it is different:** generated code carries honest execution metadata
  instead of an unqualified claim that it works, and follow-ups resolve against
  the previous artifact through coreference rather than starting over.
- **Surfaces:** library, CLI, HTTP API, web demo (JS execution), Telegram.
- **Status:** Supported today. See *Product Shape* in
  [`VISION.md`](../VISION.md#product-shape).

### J4 — Priya converts and computes

- **Persona & pain:** Priya the analyst needs numbers she can trust at a glance.
- **Trigger:** `What is 8% of $50?` then `Посчитай 1000 рублей в долларах`
- **Journey:**
  1. Priya runs a percentage calculation and a currency conversion.
  2. She converts between units (length, mass, time, temperature, data size).
  3. When she wants the working, she asks why and gets the decomposition and the
     source basis for the exchange rate.
- **Why it is different:** arithmetic is evaluated deterministically and the
  exchange-rate / unit basis is recorded with provenance, so the number is
  reproducible and auditable.
- **Surfaces:** library, CLI, HTTP API, web demo, Telegram.
- **Status:** Supported today. See *Configurable Solver Knobs* in
  [`VISION.md`](../VISION.md#configurable-solver-knobs).

### J5 — Dana checks a fact and a definition

- **Persona & pain:** Dana needs a sourced fact, not a confident guess.
- **Trigger:** `What is the capital of Russia?` / `Merge Wikipedia definitions of color`
- **Journey:**
  1. Dana asks a fact question; the assistant searches local links first, then
     external public knowledge (Wikipedia, Wikidata, Wiktionary) only when local
     data is insufficient.
  2. External lookups are cached with `source:`, `fetched_at`, and `sha256`
     metadata under the source-cache TTL policy.
  3. In offline mode the assistant refuses external lookups and emits a
     `policy:offline` event instead of inventing a fact.
- **Why it is different:** the public internet is treated as a database and the
  local store as its cache, so facts arrive with provenance and a deterministic
  replay, not as unattributed text.
- **Surfaces:** library, CLI, HTTP API, web demo, Telegram.
- **Status:** Supported today. See *Growable Memory And Public Knowledge As
  Cache* in [`VISION.md`](../VISION.md#growable-memory-and-public-knowledge-as-cache).

### J6 — Olúwa exports and moves her memory

- **Persona & pain:** Olúwa wants to own and relocate everything the assistant
  saw and did.
- **Trigger:** the web demo's **Export memory** button, or `formal-ai memory export`.
- **Journey:**
  1. Olúwa clicks **Export memory** and gets a single `formal-ai-memory.lino`
     bundle — seed, modified data, UI preferences, and the full append-only log.
  2. She moves it to the CLI and imports it; the supported migration flows are
     browser ↔ CLI, browser ↔ browser, and CLI ↔ CLI.
  3. If the seed version differs, the import prints a `Migration: <message>`
     notice instead of silently dropping data.
- **Why it is different:** the entire agent state is one reviewable Links
  Notation document — "here is everything my agent saw and did" in a single file
  — regardless of which surface she was on.
- **Surfaces:** web demo, CLI, library; the environment directory
  (`formal-ai environments`) documents each surface's export/import command.
- **Status:** Supported today. See *Single-File Reproducibility* and
  *Self-Aware Environments* in
  [`VISION.md`](../VISION.md#single-file-reproducibility).

### J7 — Sven wires up an OpenAI-compatible endpoint

- **Persona & pain:** Sven the integrator wants a drop-in endpoint without a GPU
  bill or a cloud dependency.
- **Trigger:** `cargo run -- serve --host 127.0.0.1 --port 8080`
- **Journey:**
  1. Sven starts the loopback HTTP server, which exposes the same symbolic
     engine through OpenAI Chat Completions, OpenAI Responses, and Anthropic
     Messages envelopes.
  2. He points Codex, Claude Code, OpenCode, or the Link Assistant Agent CLI at
     the local `/v1` base URL.
  3. Optionally he sets `FORMAL_AI_API_BEARER_TOKEN` to require bearer auth on
     `/v1/*` routes.
- **Why it is different:** existing OpenAI/Anthropic-shaped tooling connects
  with no code changes, and the engine answers with deterministic symbolic
  reasoning on loopback — no GPU, no external inference call.
- **Surfaces:** HTTP API server, Docker microservice; see *Agentic AI Tools* in
  [`README.md`](../README.md).
- **Status:** Supported today. See *Product Shape* in
  [`VISION.md`](../VISION.md#product-shape).

### J8 — Mikhail chats from Telegram

- **Persona & pain:** Mikhail wants the same assistant in the messenger he
  already uses.
- **Trigger:** `TELEGRAM_BOT_TOKEN=123:abc cargo run -- telegram`
- **Journey:**
  1. An operator starts the Telegram bot (long polling by default, opt-in
     webhook server) or runs the Docker-in-Docker image.
  2. Mikhail messages the bot the same prompts he would type anywhere else and
     gets the same multilingual, traceable answers.
  3. Environment limits are reported honestly rather than hidden.
- **Why it is different:** the messenger surface is backed by the same symbolic
  core and seed data as every other interface, so behavior does not fork per
  channel.
- **Surfaces:** Telegram (polling and webhook), Docker microservice.
- **Status:** Supported today. See *Product Shape* in
  [`VISION.md`](../VISION.md#product-shape).

### J9 — Wei reconfigures the agent by editing data

- **Persona & pain:** Wei the tinkerer wants to change behavior without
  retraining or recompiling logic.
- **Trigger:** editing a file under `data/seed/`.
- **Journey:**
  1. Wei adds a greeting, a fact, a tool registration, or an intent-routing rule
     to the shared seed.
  2. Every surface — library, CLI, HTTP server, Telegram, and (via
     `scripts/sync-seed.sh`) the browser demo — reads the same `data/seed/`
     directory, so the change shows up everywhere.
  3. Wei can also teach the running agent through natural language: `replace x y`,
     `when n do m`, "remember that …", and `Forget X` (refused unless the
     explicit retraction protocol is used, because the log is append-only).
- **Why it is different:** the shell code is *interface*; the agent's identity,
  responses, concept table, tools, and rules are *data*. Reconfiguring the agent
  is editing a graph, not rewriting a model.
- **Surfaces:** all surfaces share `data/seed/`.
- **Status:** Supported today. See *Data Is The Interface* in
  [`VISION.md`](../VISION.md#data-is-the-interface).

### J10 — Sven runs a bounded agent task

- **Persona & pain:** Sven wants the assistant to *do* something, not just
  answer, but only inside a boundary he controls.
- **Trigger:** an agent-mode request that needs to run a command.
- **Journey:**
  1. Sven opts into agent mode (off by default via `SolverConfig.agent_mode`).
  2. The assistant exposes the actions it intends to take and runs allowlisted
     commands inside an isolated workspace (Docker image, server sandbox, or
     container action).
  3. Every action and log lands in the event log; nothing runs hidden.
- **Why it is different:** agent autonomy is explicit and bounded — actions are
  shown, isolation is required, and unbounded loops are a declared non-goal.
- **Surfaces:** CLI / server with isolation (for example the Docker-in-Docker
  image); see *Operating Principles* in
  [`VISION.md`](../VISION.md#operating-principles).
- **Status:** Supported today (bounded, allowlisted). See *Explicit agent
  autonomy* in [`VISION.md`](../VISION.md#operating-principles).

### J11 — Lin edits the previous answer

- **Persona & pain:** Lin wants to refine the last result, not retype it.
- **Trigger:** a follow-up edit such as `replace "world" with "Formal AI"` on a
  previously generated text or code block.
- **Journey:**
  1. Lin gets an answer (text or a code block).
  2. She asks for an edit that targets the previous assistant artifact.
  3. The supported edit operations resolve through the shared multilingual
     operation vocabulary and apply to the right artifact.
- **Why it is different:** follow-up edits are deterministic operations over the
  active artifact, recognized identically in every supported language, and the
  behavior is benchmark-backed.
- **Surfaces:** library, CLI, HTTP API, web demo, Telegram.
- **Status:** Supported today. See the issue #408 paragraph in
  [`VISION.md`](../VISION.md#current-direction) and
  [`ROADMAP.md`](../ROADMAP.md).

---

## Potential Future Journeys

These journeys are consistent with the vision and partially scaffolded, but are
not fully delivered on `main` today. They are listed so the docs describe the
**whole** intended surface, not only what already ships. Each points at the
vision text that motivates it; build status stays in
[`ROADMAP.md`](../ROADMAP.md).

### F1 — Dana inspects the link graph beside the chat

- **Persona & pain:** Dana wants to *see* the associative network, not just read
  a textual trace.
- **Future journey:** while chatting, Dana opens a visual graph of the links the
  assistant activated for her question and explores meanings, sources, and
  traces side by side with the conversation.
- **Why it matters:** *Visual network on demand* is a stated operating
  principle; the graph complements chat without replacing it (a declared
  non-goal of replacing chat as the primary interface).
- **Status:** Potential future; the interactive step-by-step debugging view is
  tracked by [#667](https://github.com/link-assistant/formal-ai/issues/667). See
  *Operating Principles* and the *visual graph* non-goal.

### F2 — Wei compiles a natural-language skill

- **Persona & pain:** Wei wants to write a skill in plain language and have the
  agent execute or compile it.
- **Future journey:** Wei stores a `.lino` skill describing a procedure; the
  runtime either walks it step by step or compiles it on demand into one of the
  supported rule shapes (pure-data rule, compiled Rust/JS handler, or
  dynamically compiled code stored as data).
- **Why it matters:** *Computation Model* describes five rule shapes ranked from
  most reviewable to most flexible, with natural-language skills at the flexible
  end.
- **Status:** Potential future / partially scaffolded (skill-compiler seed and
  design exist); compiling arbitrary freely-phrased procedures is tracked by
  [#674](https://github.com/link-assistant/formal-ai/issues/674). See
  *Computation Model* in [`VISION.md`](../VISION.md#computation-model).

### F3 — Olúwa syncs memory to the cloud and across devices

- **Persona & pain:** Olúwa wants her bundle to follow her between machines
  automatically, not only through manual export/import.
- **Future journey:** the same `formal_ai_bundle` document syncs to additional
  persistent storage (cloud sync) so her memory is available on every device.
- **Why it matters:** *Growable Memory* names "future cloud sync" as an
  additional persistent storage target beyond disk and IndexedDB.
- **Status:** Potential future; tracked by
  [#669](https://github.com/link-assistant/formal-ai/issues/669). See *Growable
  Memory And Public Knowledge As Cache*.

### F4 — Priya tackles a problem that needs search

- **Persona & pain:** Priya hits a problem with no reusable prior part and no
  single rule that solves it.
- **Future journey:** the solver combines reasoning, random search, and
  evolutionary search according to the available compute budget, seeded
  deterministically from the impulse hash, instead of giving up.
- **Why it matters:** *Solution synthesis* and the *Universal Solver Goals*
  describe budget-driven search while preserving reproducibility.
- **Status:** Potential future / partially built (deterministic synthesis path
  exists; broader search is staged); budget-driven search is tracked by
  [#662](https://github.com/link-assistant/formal-ai/issues/662) and parallel
  candidate portfolios by
  [#704](https://github.com/link-assistant/formal-ai/issues/704). See
  *Reasoning Model* and [`ROADMAP.md`](../ROADMAP.md).

### F5 — Lin runs heavier code in the browser

- **Persona & pain:** Lin wants browser-only execution beyond JavaScript
  evaluation.
- **Future journey:** the web demo experiments with WebVM so more languages can
  run locally in the browser, while still reporting execution limits honestly.
- **Why it matters:** *Product Shape* notes browser mode can start with
  JavaScript evaluation and later experiment with WebVM, and browser mode must
  not claim host-level execution (a non-goal).
- **Status:** Potential future; the time-boxed experiment is tracked by
  [#670](https://github.com/link-assistant/formal-ai/issues/670). See *Product
  Shape*.

### F6 — Wei shares an associative package

- **Persona & pain:** Wei wants to package datasets, skills, rules, and handlers
  and share them with another instance.
- **Future journey:** Wei exports a reusable associative package (with
  permissions) and another user imports it to gain new concepts, tools, and
  rules — Deep.Foundation-style packages adapted to local Rust/browser/CLI
  modes.
- **Why it matters:** *Computation Model* cites Deep.Foundation as a reference
  for associative packages, handlers, and permissions stored inside memory.
- **Status:** Potential future; tracked by
  [#668](https://github.com/link-assistant/formal-ai/issues/668). See
  *Computation Model*.

---

## Journey-To-Surface Coverage

This matrix makes the "where does it work" promise explicit. ● = supported
today, ○ = potential future on that surface.

| Journey | Library | CLI | HTTP API | Web demo | Telegram | Desktop / VS Code |
| --- | :---: | :---: | :---: | :---: | :---: | :---: |
| J1 Why did you answer | ● | ● | ● | ● | ● | ● |
| J2 Multilingual chat | ● | ● | ● | ● | ● | ● |
| J3 Hello world / code gen | ● | ● | ● | ● | ● | ● |
| J4 Math / units / currency | ● | ● | ● | ● | ● | ● |
| J5 Fact / definition lookup | ● | ● | ● | ● | ● | ● |
| J6 Export / move memory | ● | ● | ○ | ● | ○ | ● |
| J7 OpenAI-compatible endpoint | ● | — | ● | — | — | ● |
| J8 Telegram chat | — | ● | ● | — | ● | — |
| J9 Edit-the-data config | ● | ● | ● | ● | ● | ● |
| J10 Bounded agent task | ● | ● | ● | ○ | ● | ○ |
| J11 Edit the previous answer | ● | ● | ● | ● | ● | ● |
| F1 Visual graph | ○ | — | — | ○ | — | ○ |
| F2 Compiled skills | ○ | ○ | ○ | ○ | ○ | ○ |
| F3 Cloud memory sync | ○ | ○ | ○ | ○ | ○ | ○ |
| F4 Search-based solving | ○ | ○ | ○ | ○ | ○ | ○ |
| F5 WebVM execution | — | — | — | ○ | — | ○ |
| F6 Shared packages | ○ | ○ | ○ | ○ | ○ | ○ |

The desktop and VS Code surfaces wrap the same web chat and local HTTP boundary,
so they inherit the chat-facing journeys above.

## A Worked Example Journey

To ground the abstract pipeline, here is one journey end to end — the concrete
example requested in issue #454. Dana the auditor needs a defensible answer.

> **Dana:** `What is 8% of $50?`
>
> **Formal AI:** `$4`

Behind that one line, the universal 11-step loop ran and appended an event per
step to the log:

1. **Impulse** — the raw message is recorded.
2. **Formalization** — it becomes a `requirement` (a percentage-of-amount
   computation), expressed as a Links Notation intent.
3. **Context** — language `en`, chat surface, agent mode off.
4. **History lookup** — no prior identical requirement, so no `cache_hit`.
5. **Decomposition** — `8% of $50` splits into `8 / 100` then `× 50`.
6. **Validation** — a TDD-style check asserts the result equals `0.08 × 50`.
7. **Synthesis** — the partial results compose to `4`.
8. **Combination** — the answer is framed as `$4`.
9. **Verification** — the candidate passes its generated check.
10. **Simplification** — the smallest sufficient form (`$4`) is selected.
11. **Documentation** — the reply plus a `trace:` pointer is produced.

Now Dana follows up:

> **Dana:** `Why did you answer that?`
>
> **Formal AI:** a meta-explanation assembled from the most recent trace — the
> decomposition, the validation it passed, and the simplification that produced
> `$4`.

The same prompt with the same configuration will produce the same `$4` and the
same trace tomorrow, on a colleague's machine, offline, with no GPU. That
reproducible, inspectable, ownable answer is the pain Formal AI closes — and the
through-line of every journey above.

## See Also

- [`VISION.md`](../VISION.md) — what we are building and why.
- [`GOALS.md`](../GOALS.md) and [`NON-GOALS.md`](../NON-GOALS.md) — scope.
- [`README.md`](../README.md) — the surfaces and how to run them.
- [`ROADMAP.md`](../ROADMAP.md) — how much of the vision is built versus planned.
- [`ARCHITECTURE.md`](../ARCHITECTURE.md) — how the pipeline is wired.
