# Issue 468 Case Study

> **Status:** Agentic-coding loop shipped in PR #469 — the Formal AI server solves
> the issue's example task in agentic mode, across all three OpenAI-shaped surfaces.
> **Type:** Documentation + enhancement (new `src/agentic_coding/` capability + case study).
> **Primary source:** [Формальный протокол для перевода текстов в базу знаний](https://telegra.ph/Formalnyj-protokol-dlya-perevoda-tekstov-v-bazu-znanij-06-10) (Igor Martynov, January 2026)

- **Issue:** <https://github.com/link-assistant/formal-ai/issues/468>
- **Pull request:** <https://github.com/link-assistant/formal-ai/pull/469>
- **Nine primitives as links:** [`formal-protocol-mapping.md`](formal-protocol-mapping.md)
- **Machine-readable recipe:** [`data/meta/agentic-coding-recipe.lino`](../../../data/meta/agentic-coding-recipe.lino)
- **Source-protocol summary (cited):** [`raw-data/article-summary.md`](raw-data/article-summary.md)
- **Online research (cited):** [`raw-data/online-research.md`](raw-data/online-research.md)
- **Worked examples:** [`examples/issue_468_agentic_loop.rs`](../../../examples/issue_468_agentic_loop.rs) (the full loop) and [`examples/issue_468_formalize_text.rs`](../../../examples/issue_468_formalize_text.rs) (the formalizer)
- **Capability module:** [`src/agentic_coding/`](../../../src/agentic_coding/)

All raw artifacts referenced below live in [`raw-data/`](raw-data/).

---

## 1. Summary

The issue **title** is the requirement: *"Our system should be able to solve such
tasks in agentic mode."* The maintainer's comment is explicit that the
text-formalization protocol in the issue body is **an example of a task** the
system should solve from the prompt, and that the **core task** is the agentic
capability itself:

> *"the core task is in the comment, we need to make sure our Formal AI is able to
> solve the task in agentic coding mode … start locally our OpenAI API compatible
> server, configure it with [link-assistant/agent] … So our Formal AI system
> should have enough skills (meta algorithm, rust code) to actually call all the
> tools from any agentic CLI, understand errors from tools, and so on, call bash
> commands, do web fetch and web search, to actually complete the task."*

So this case study is about an **agentic-coding loop**, demonstrated on the issue's
own example task — formalizing Pushkin's **«Сказка о рыбаке и рыбке»** ("The Tale
of the Fisherman and the Fish") into a knowledge base. Two binding constraints
shape the design:

> *"I personally [do] not agree with entities and ontologies (because for us
> everything is a link), while we don't agree with requirements we still should by
> default implement them as is."*

> *"Also don't use claude or codex to connect them to our Formal AI, as that may
> interrupt your own process, and also break execution of other tasks."*

The first means the knowledge base the task produces is emitted in **Links
Notation** — the project's own meta-language, *"that is already in our code
base"* — so the protocol's nine primitives are **realized as links**, not as
typed-struct entities/ontologies. The second means external CLIs (`claude`,
`codex`) are pointed **at** the server as front-ends; they are never wired **into**
the Formal AI engine.

What ships in PR #469 is the whole loop, running offline and deterministically:

1. **The server brain** — a deterministic [planner](../../../src/agentic_coding/planner.rs)
   (`plan_chat_step`) that reads the conversation so far and the tools the agentic
   CLI advertised, and decides the next step: a state machine
   `web_search → web_fetch → write_file(formalize) → run_command(verify) → final`.
   It is a pure function of the message history — no sampling, no neural inference.
2. **Three surfaces, one planner** — the planner backs `/v1/chat/completions`,
   `/v1/messages` (what `claude` speaks), and `/v1/responses` (what `codex`
   speaks), so the loop *"call[s] all the tools from any agentic CLI"*
   ([`src/protocol.rs`](../../../src/protocol.rs), `src/anthropic.rs`).
3. **Two gates** — tools are refused unless `agent_mode` is opted in *and* each
   requested tool passes a per-tool permission gate
   (`pkg_agentic_coding` in [`src/associative_package.rs`](../../../src/associative_package.rs)),
   so there is no hidden autonomous action.
4. **The client** — an in-repo [driver](../../../src/agentic_coding/driver.rs)
   (`run_agentic_task`) plays the external agentic CLI: it advertises
   `DRIVER_TOOLS`, executes every emitted tool call (`web_search`/`web_fetch`
   against an offline [corpus](../../../src/agentic_coding/corpus.rs),
   `write_file`/`run_command` in a sandboxed `AgentWorkspace`), feeds each result
   back, *understands tool errors*, and loops until the server returns the finished
   knowledge base — bounded by a hard `MAX_TURNS` cap.
5. **The example task's output** — the
   [formalizer](../../../src/agentic_coding/formalize.rs)
   (`formalize_text_to_links`) turns the fetched source text into a Links Notation
   knowledge base in which **all nine protocol primitives are links** (concept,
   entity, predicate, assertion, procedure, context, temporal, modal, annotation).

The deliverables are code **and** documentation, fully traceable:

1. The [`src/agentic_coding/`](../../../src/agentic_coding/) module: the planner,
   the driver, the offline corpus, the closed-class
   [lexicon](../../../src/agentic_coding/lexicon.rs), and the Links Notation
   formalizer.
2. The `formal-ai agent` CLI subcommand (`src/main.rs`) and two worked examples.
3. A machine-readable [recipe](../../../data/meta/agentic-coding-recipe.lino) of the
   meta-algorithm, grounded by a test so the recipe always matches the live code.
4. [`formal-protocol-mapping.md`](formal-protocol-mapping.md): each of the nine
   primitives shown as a Links Notation record.
5. This case study: collected data, deep analysis, online research, the exhaustive
   requirement list **R306–R319**, and a solution plan plus prior-art survey for
   each.
6. Regression tests (`tests/unit/agentic_coding.rs`,
   `tests/unit/agentic_surfaces.rs`,
   `tests/unit/specification/agentic_meta_algorithm.rs`) pinning the loop on every
   surface, the formalizer's record count and coverage-of-nine, and the recipe's
   fidelity to the code.

What is **out of current scope, stated honestly** (not hidden): general
open-domain natural-language extraction — the article's §7 learned pipeline (POS
tagging, dependency parsing, semantic role labeling, NER, coreference) — requires
neural inference, which is a project NON-GOAL. The formalizer therefore annotates
*every* sentence (fully general) but only emits a *structured* assertion when a
closed-class lexicon recognises the triple; unrecognised sentences become honest
natural-language assertions that carry the raw span and **never guess** a relation
(§4.6). Live network access is likewise out of scope in CI: the driver resolves
web tools against a fixed offline corpus, so the whole loop is reproducible.

---

## 2. Collected Data

The raw, third-party captures (exempt from authored-prose lints) are archived
under [`raw-data/`](raw-data/):

| File | What it is |
|---|---|
| [`raw-data/issue-468.json`](raw-data/issue-468.json) | The issue as filed (`gh issue view 468 --json …`). Labels: `documentation`, `enhancement`. Title: the agentic-mode requirement. Body (Russian) specifies the nine primitives and links the source article. |
| [`raw-data/issue-468-comments.json`](raw-data/issue-468-comments.json) | The maintainer comment — the authoritative reframing (the example-vs-core-task distinction, the agentic-mode brief, the case-study deliverable, the "everything is a link" position, and the "don't wire external CLIs into Formal AI" constraint). |
| [`raw-data/pr-469.json`](raw-data/pr-469.json) | The pull request this work lands in. |
| [`raw-data/article-summary.md`](raw-data/article-summary.md) | A paraphrased-and-cited summary of the source telegra.ph protocol: the nine primitives with their fields, the authoritative JSON example (quoted), the assertion BNF, the worked-example sentence, the §9 declarative query, the §12 nested-assertion form, the §10 metrics, and the §7 NL pipeline. This is the **spec for the example task**. |
| [`raw-data/online-research.md`](raw-data/online-research.md) | Summarized-and-cited research grounding each primitive in established practice: AMR, OpenIE, RDF reification / RDF-star / named graphs, Wikidata qualifiers, FrameNet/PropBank, the general NL→KB pipeline, the associative "everything is a link" model, and the Tale's own provenance. |

Per [NON-GOALS.md](../../../NON-GOALS.md) (*"Research notes should not copy large
external texts; they should summarize and cite sources"*), both research files
quote only short definitional phrases and link every claim to its source. The
Tale text itself is public-domain (Pushkin, 1833) and is referenced, not
bulk-copied; the formalizer's canonical input is a plain-prose synopsis in the
project's own wording ([`data/agentic-coding/fisherman-synopsis.txt`](../../../data/agentic-coding/fisherman-synopsis.txt)), **not** Pushkin's verse.

---

## 3. Holistic Requirements

Every requirement extracted from the issue body **and** the maintainer comment
(the two together are the complete specification). These are recorded in
[`REQUIREMENTS.md`](../../../REQUIREMENTS.md) as **R306–R319** under *"Issue #468
Agentic-Coding Mode"*.

| ID | Requirement (intent) | Status |
|---|---|---|
| **R306** | Code into the application the capability to solve the example task — translating texts into a knowledge base per the protocol, with «Сказка о рыбаке и рыбке» as the canonical input. | Done — `formalize_text_to_links` (`src/agentic_coding/formalize.rs`) drives the example task end to end; surfaced through `formal-ai agent` (`src/main.rs`) and re-exported from `src/agentic_coding/mod.rs`. |
| **R307** | Realize the **nine protocol primitives** (Concept, Entity, Predicate, Assertion, Procedure, Context, Temporal, Modal, Annotation). | Done — the nine kinds are `PRIMITIVE_KINDS`; each is emitted as a Links Notation record by the formalizer (§4.5). For the canonical synopsis `summary.covers_all_nine()` is `true`, pinned in `tests/unit/agentic_coding.rs`. The full article field set is the spec in `raw-data/article-summary.md`; the formalizer populates the deterministically-extractable subset. |
| **R308** | Make the representation **assertion-centric**: assertions are the units for search, inference, and aggregation. | Done — each fact is one atomic `assertion` record whose `subject`/`predicate`/`object` associations are links into the entity/predicate/concept catalogue and whose qualifiers are ordinary associations; search/inference/aggregation run over that assertion store (the crate's general Links Notation tooling), not a bespoke query engine. |
| **R309** | Keep the **operational format distinct from an ontology schema**: assertions + procedures + modality + provenance carry the facts; Concept/Predicate declarations remain a fact-free reference directory. | Done — text-derived assertions carry a `provenance` association into the source span, while lexicon-sourced catalogue records (concept/procedure/context) are tagged `source "lexicon:<doc>"`, keeping the reference directory structurally distinct from the facts. |
| **R310** | Carry the per-assertion qualifiers the protocol names: **modality, time, context, confidence, provenance**. | Done — assertion records carry optional `time` / `modal` / `context` associations plus `annotation` + `provenance`; the `temporal` and `modal` primitives (the latter with a `degree`) are emitted as their own links and referenced by id (§4.5, [`formal-protocol-mapping.md`](formal-protocol-mapping.md)). |
| **R311** | Honor *"everything is a link"*: implement the primitives **as is**, in the meta-language already in the code base. | Done — every primitive is emitted directly as a Links Notation record via `format_lino_record` (`src/links_format.rs`); there is no typed-struct or protocol-JSON layer. Mapped primitive by primitive in [`formal-protocol-mapping.md`](formal-protocol-mapping.md). |
| **R312** | Demonstrate the scheme on «Сказка о рыбаке и рыбке», exercising all nine primitives end to end. | Done — formalizing the canonical synopsis yields **37 records** (1 header + 3 concepts + 4 entities + 6 predicates + 1 procedure + 2 contexts + 3 temporals + 3 modals + 7 annotations + 7 assertions); both the count and coverage-of-nine are pinned in `tests/unit/agentic_coding.rs`. |
| **R313** | Provide **deterministic extraction** for the example, and scope **general open-domain extraction** honestly (it needs neural inference — a project NON-GOAL). | Done — every sentence becomes an annotation with real character offsets; a closed-class lexicon (`src/agentic_coding/lexicon.rs`, data in `data/agentic-coding/fisherman-lexicon.lino`) turns recognised subject-predicate-object triples into structured assertions, and unrecognised sentences become honest natural-language assertions that never guess; general extraction is scoped as future work (§4.6/§7). |
| **R314** | Address the title's **"agentic mode"**: the system must solve the task by being driven by an agentic CLI against the OpenAI-compatible server — calling tools, understanding errors, web search/fetch, bash — while honoring the constraint *not to wire external CLIs (claude/codex) into Formal AI*. | **Done — this is the core task and it ships.** The planner drives the loop across all three OpenAI-shaped surfaces (§4.1–§4.4); the in-repo driver exercises it offline; external CLIs are documented as front-ends *against* `formal-ai serve`, not embedded (§5). |
| **R315** | **Collect issue data** into `docs/case-studies/issue-468/`. | Done — §2 and [`raw-data/`](raw-data/). |
| **R316** | **Deep case-study analysis**, including **online research** for additional facts. | Done — §4 (analysis) + [`raw-data/online-research.md`](raw-data/online-research.md) (8 cited sources). |
| **R317** | **List each and all requirements** from the issue. | Done — this table (R306–R319) plus the prose enumeration below. |
| **R318** | Propose **solutions and solution plans per requirement**, checking **existing components/libraries**. | Done — §6 (per-requirement plans) + §7 (prior-art survey). |
| **R319** | **Plan and execute everything in the single PR #469.** | Done — the module, CLI, examples, recipe, tests, this case study, the mapping doc, the `raw-data/` captures, the `REQUIREMENTS.md` rows, and the changelog fragments all land in PR #469. |

### Why these fourteen and not more

The issue body is one specification paragraph plus a primitive list; the comment
reframes the body as an *example task* and states the *core* agentic-mode ask plus
the case-study brief and constraints. R306 is the "code the capability" ask; R307
is the primitive list; R308 is *"Assertions — единицы, по которым делаем поиск,
выводы, агрегации"*; R309 is the *"Чем отличается от онтологии"* paragraph; R310 is
the explicit assertion qualifier set; R311 is the everything-is-a-link position
combined with "implement as is". R312–R313 are the body's request to *"проверить
работоспособность"* ("check that the scheme works") — one demonstrated on the
canonical input, one on the extraction discipline. **R314 is the issue title — the
core task — read against the comment's constraint.** R315–R319 are the comment's
five-part case-study brief. The optional *"test on AI benchmarks"* sentence is
recorded as cross-referenced, not promoted to a requirement (§7), to avoid
overclaiming.

---

## 4. Deep Analysis

### 4.1 The agentic loop: a server brain and a client

The loop has two halves. The **server brain** is `plan_chat_step`
(`src/agentic_coding/planner.rs`): given the messages so far and the tool names
the CLI advertised, it returns the next `AgenticPlan` — either `ToolCalls` (emit
these calls and wait) or `Final` (the knowledge base inline). It is a deterministic
state machine:

```text
web_search → web_fetch → write_file(formalize) → run_command(verify) → final
```

Each step is taken only if (a) the conversation does not already contain a tool
result for that capability and (b) the CLI advertised a tool with that capability.
`classify_tool` maps whatever names a CLI uses (`web_search`, `web_fetch`,
`write_file`, `run_command`, `bash`, `shell`, …) onto a `Capability`
(`Search`/`Fetch`/`Write`/`Run`) by substring, so the planner adapts to *any*
subset of tools a given CLI exposes — the literal reading of *"call all the tools
from any agentic CLI"*. The plan is pinned by the constants `SEARCH_QUERY`,
`CANONICAL_SOURCE_URL`, and `KB_PATH`.

The **client** is `run_agentic_task` (`src/agentic_coding/driver.rs`). It plays the
external agentic CLI in-repo: advertises `DRIVER_TOOLS`
(`["web_search", "web_fetch", "write_file", "run_command"]`), sends a chat request,
and whenever the server answers with `tool_calls` it *executes* each call —
`web_search`/`web_fetch` against the offline corpus, `write_file`/`run_command`
against one reused sandboxed `AgentWorkspace` — appends the assistant turn followed
by the `tool` results (order matters: the planner maps each result's
`tool_call_id` back to a prior assistant `tool_calls` turn), and loops until
`finish_reason: "stop"`. The loop is bounded by `MAX_TURNS = 12` (the recipe needs
five), so an unbounded reasoning loop is impossible — a stated NON-GOAL.

### 4.2 One planner, three surfaces

The maintainer wants the system to be driven by *"any agentic CLI"*. Different CLIs
speak different OpenAI-shaped dialects: `codex` speaks the Responses API
(`/v1/responses`), `claude` speaks Anthropic Messages (`/v1/messages`), and
`opencode` / `link-assistant/agent` speak Chat Completions
(`/v1/chat/completions`). A single `agentic_outcome` decision in
`src/protocol.rs` — *refuse / plan / fall-through* — backs all three, so the loop
behaves identically everywhere:

- A `ToolCalls` plan becomes an assistant turn with `finish_reason: "tool_calls"`
  on Chat Completions, a `tool_use` content block with `stop_reason: "tool_use"`
  on Anthropic Messages, and a `function_call` output item on Responses.
- A fed-back tool result is *understood* in each protocol's own idiom — a `tool`
  message, an Anthropic `tool_result` block carried on a `user` message, or an
  OpenAI `function_call_output` item — and translated into the shared message
  shape so the loop **advances** rather than restarting.

`tests/unit/agentic_surfaces.rs` (9 tests) pins this mirror on the Messages and
Responses surfaces, including the `input_json_delta` streaming shape an agentic CLI
assembles tool calls from.

### 4.3 Two gates, no hidden autonomy

Tool execution is gated twice (`agentic_outcome`, `src/protocol.rs`). First,
`agent_mode` must be opted in — without it every tool is refused with a policy
message (`agent_mode_required_for_tools`). Second, each requested tool must pass a
per-tool permission gate. The default package set installs `pkg_agentic_coding`
(`src/associative_package.rs`), a permission-only package granting the
client-executed `web_fetch` / `write_file` / `run_command` capabilities
(`web_search` comes from the core package). Granting these *by default* enables no
hidden action, because `agent_mode` remains the real guard: an agent that has not
opted in can call nothing. This is the "agentic mode is strictly opt-in" posture
the NON-GOALS demand — no autonomous action without an explicit, auditable switch.

### 4.4 Understanding tool errors

The issue asks the system to *"understand errors from tools"*. The planner's
`looks_like_error` heuristic treats a tool result containing `error` / `failed` /
`not found` / `404` as untrustworthy: such a fetch result is **not** adopted as the
source text. The formalizer then falls back to the canonical synopsis, so the loop
still completes with a stable, all-nine-primitive knowledge base. The offline
corpus exercises this directly: `web_fetch` of an unknown URL returns
`web_fetch error: 404 not found …`, exactly as a real 404 would, and the loop
recovers deterministically.

### 4.5 The example task's output: nine primitives as links (R311/R312)

The fetched source text is formalized by `formalize_text_to_links`. The maintainer's
*"everything is a link"* is realized literally: every record is emitted directly as
Links Notation by `format_lino_record`, so each of the nine primitives **is** a link
(a node headed by its kind, whose `key "value"` associations are doublets). For the
canonical synopsis the document is **37 records** and `covers_all_nine()` is `true`:

```
knowledge_base
  id "tale:fisherman-and-fish"
  primitive_scheme "concept entity predicate assertion procedure context temporal modal annotation"
  …
assertion
  id "a:0"
  subject "ent:old_man"
  subject_kind "entity"
  predicate "pred:catch"
  object "ent:golden_fish"
  object_kind "entity"
  time "temporal:в-начале-сказки"
  context "ctx:seaside"
  annotation "ann:0"
  provenance "tale:fisherman-and-fish@0:28"
```

The assertion is the atomic block: its `subject`/`predicate`/`object` associations
are links into the entity/predicate catalogue, and its qualifiers (`time`, `modal`,
`context`) are ordinary associations pointing at the temporal/modal/context links —
no reification, no nested record. The full primitive-by-primitive mapping is in
[`formal-protocol-mapping.md`](formal-protocol-mapping.md).

### 4.6 Closed-class extraction, honest fallback (R313)

The article's §7 general pipeline (tokenization → POS → dependency parse → SRL →
NER → coreference) is a chain of **learned-model** problems, and `formal-ai`
performs **no neural inference** by design (`NON-GOALS.md`). Faking it with a
brittle hand-rolled parser masquerading as general understanding would be the
dishonest path. Instead the formalizer is split by confidence:

- **Annotations are fully general.** Every sentence of *any* input becomes an
  `annotation` link with real **character** offsets (Cyrillic-safe) — never guessed.
- **Assertions are grounded or honest.** A closed-class lexicon (stored as data in
  `data/agentic-coding/fisherman-lexicon.lino`) recognises subject-predicate-object
  triples; recognised triples become structured assertions. An *unrecognised*
  sentence still produces an assertion — `subject "—"`, `predicate "pred:states"`,
  the raw sentence as the object, plus a `natural_language` association — so the
  fact is recorded with its span but **no relation is invented**.

The general capability is named as future work (§7), not buried.

### 4.7 What the online research adds

[`raw-data/online-research.md`](raw-data/online-research.md) grounds the example
task's design in established practice: the assertion ≈ AMR/OpenIE **triple**;
statement-level modality/time/context/confidence/provenance ≈ **RDF reification /
RDF-star / named graphs** and **Wikidata qualifiers + references**; the Predicate
directory ≈ **FrameNet/PropBank** predicate inventories; and the realization as
doublets ≈ the associative *"everything is a link"* model the project already runs
on. Nothing in the literature contradicts the protocol; the qualifier set it names
is precisely what a deployed knowledge graph (Wikidata) found necessary in practice.

---

## 5. Agentic Mode — the surfaces, the constraint, the boundary (R314)

This is the core task, and it ships. The intended mechanism from the comment —
stand up the OpenAI-compatible server, point an agentic CLI at it, and let Formal
AI call tools to complete the task — is realized as the loop of §4. Here is how an
**external** CLI is pointed at it, and where the boundary sits.

The server already exposes the three OpenAI-shaped surfaces an agentic CLI targets
as its model backend; [`docs/desktop/server-api.md`](../../desktop/server-api.md)
documents how to point each CLI at `formal-ai serve`:

- **`codex`** → the Responses API (`/v1/responses`); see server-api.md §4a.
- **`opencode`** and **[`link-assistant/agent`](https://github.com/link-assistant/agent)**
  → Chat Completions (`/v1/chat/completions`); see §4b / §4c.
- **`claude`** → Anthropic Messages (`/v1/messages`); see §4d.

In every case the CLI runs the loop: it advertises its tools, the server emits the
next tool call, the CLI executes it (its own real web/file/command tools) and feeds
the result back, until the server returns the finished knowledge base. The in-repo
driver (§4.1) is the offline, deterministic stand-in for exactly this client so the
whole thing runs in CI.

**The boundary.** The comment is explicit:

> *"Also don't use claude or codex to connect them to our Formal AI, as that may
> interrupt your own process, and also break execution of other tasks."*

So external CLIs are front-ends *against* the server — they drive it; they are
never imported, spawned, or embedded by the engine. The driver that exercises the
loop in CI is the project's *own* code, not a wrapper around `claude`/`codex`.

**Reconstruction from saved traces (reference only).** The comment also notes that
if the loop fails one *"can use locally available claude and codex (ask them to
output JSON for reference, or take their saved json sessions) … to actually
reconstruct all reasoning steps."* That is a *reference* workflow: a saved agent
session is a sequence of tool calls that can be read back to reconstruct the
reasoning, and — because each call maps onto the same `web_search`/`web_fetch`/
`write_file`/`run_command` capabilities the planner already models — it can be
replayed as a transcript for comparison. It is explicitly **not** a runtime
dependency: nothing in `src/agentic_coding/` calls an external model, by design and
per the constraint.

---

## 6. Solution Plans (per requirement)

Each plan names the chosen approach and the existing component it reuses, per
R318. The survey those choices draw on is §7.

### R314 — Agentic mode (the core task)
**Approach (chosen, done):** a deterministic planner (`plan_chat_step`) wired into
the shared `agentic_outcome` decision so all three OpenAI-shaped surfaces emit and
consume tool calls identically, plus an in-repo driver + offline corpus that
exercise the full `search → fetch → write → run → final` loop offline. **Existing
components reused:** the crate's `AgentWorkspace` (sandboxed command/file
execution), the existing OpenAI-compatible server (`src/protocol.rs`,
`src/anthropic.rs`), and the associative-package permission model
(`pkg_agentic_coding`). **Constraint honored:** external CLIs drive the server;
none is embedded. **Alternative considered:** spawning `claude`/`codex` as a
subprocess — rejected outright by the maintainer's boundary.

### R306 / R307 / R310 / R311 — The example task as links
**Approach (chosen, done):** a deterministic `formalize_text_to_links` that emits
the nine primitives **directly** as Links Notation records via the crate's own
`format_lino_record` — the same helper the memory/seed subsystems use, so the
output is consistent with the rest of the project. **Alternative considered:** the
typed-struct + JSON module a previous draft hand-coded — rejected because it
contradicted *"everything is a link"* and *"meta-language already in our code
base"*; it was removed in favor of direct Links Notation emission.

### R308 — Assertion-centric representation
**Approach (chosen, done):** the assertion is the atomic record; its
subject/predicate/object are links into the catalogue and its qualifiers are
associations. Search/inference/aggregation run over the emitted Links Notation
store with the crate's general links tooling rather than a bespoke query engine,
keeping the formalizer focused on producing the store.

### R309 — Operational format vs ontology schema
**Approach (chosen, done):** a structural `source` vs `provenance` distinction —
lexicon-sourced catalogue records are tagged `source "lexicon:<doc>"`, text-derived
assertions carry `provenance "<doc>@<start>:<end>"` — so the fact-free reference
directory is separable from the facts without a schema layer.

### R312 / R313 — Demonstrate on the Tale; scope general extraction
**Approach (chosen, done):** formalize the canonical synopsis (37 records, all nine
primitives, pinned by test) as the worked demonstration; split extraction by
confidence (general annotations, grounded-or-honest assertions) so nothing is
faked. **Existing components surveyed as the future reuse path:** spaCy/Stanza,
AllenNLP, AMR parsers, OpenIE systems — named in §7 as what a learned loop would
call.

### R315 / R316 / R317 / R318 / R319 — The case-study brief
**Approach (chosen, done):** §2 collects the data; §4 + the research file do the
deep analysis with online research; §3 enumerates R306–R319; §6 + §7 give the plans
and survey; and every artifact lands in the single PR #469. **Existing component
reused:** the case-study layout and the `raw-data/online-research.md` convention
established by issue-451 / issue-408.

---

## 7. Existing Components / Prior Art Surveyed (R318)

What the field already built, and what `formal-ai` reuses, re-expresses, or names
as a future reuse target. Full citations are in
[`raw-data/online-research.md`](raw-data/online-research.md).

### Agentic-mode front-ends (the core task)
- [`link-assistant/agent`](https://github.com/link-assistant/agent),
  [`opencode`](https://github.com/sst/opencode), `codex`, and `claude` — agentic
  CLIs that target an OpenAI-compatible backend. They are the front-end reuse target
  for R314, invoked against `formal-ai serve` (the three surfaces of §5), *not*
  wired into the engine (constraint).

### Agentic-coding benchmarks (the optional "test on benchmarks" note)
- The repository's central [`docs/benchmarks.md`](../../benchmarks.md) already
  catalogs the agentic-coding-shaped suites — **BFCL** (tool/function calling),
  **SWE-bench** (repository task completion), **LiveCodeBench**, **CanItEdit**, and
  **HumanEvalFix**. The issue's optional "test on AI benchmarks" sentence maps onto
  these existing entries; this case study cross-references them rather than
  fabricating a new suite, consistent with the benchmarks doc's strict-catalog rule.

### Meaning representations (the assertion)
- **Abstract Meaning Representation (AMR)** — rooted, labeled sentence graphs with
  PropBank predicate senses. The assertion (subject/predicate/object over a
  predicate directory) is the same triple idea; AMR is the richer, learned target a
  neural loop would emit. *Surveyed as the future extraction target, not embedded.*
- **Open Information Extraction (OpenIE / Stanford OpenIE)** — schema-free
  `(arg1, relation, arg2)` triples straight from text — precisely the assertion
  shape, and the closest existing *general* extractor; named as the reuse path for
  R313's future work.

### Statement-level metadata (the qualifiers)
- **RDF reification, RDF-star (RDF 1.2), named graphs** — the standard ways to
  attach metadata (time, source, certainty) to a statement; the assertion's
  qualifier associations are the same need, and the Links Notation realization is
  isomorphic to a reified-triple encoding (a natural future bridge, noted not
  shipped).
- **Wikidata qualifiers + references** — the largest deployed knowledge graph
  attaches `point in time`, `determination method`, and `reference` to statements:
  direct empirical support that the protocol's qualifier set is the *right* set.

### Predicate inventories (the directory)
- **FrameNet, PropBank, VerbNet** — curated predicate/role inventories; the
  predicate catalogue plays the same reference-directory role, and these are the
  reuse targets if it is ever grounded against a standard inventory.

### The associative substrate (everything is a link)
- [`linksplatform/doublets-rs`](https://github.com/linksplatform/doublets-rs) — the
  native doublet store the project runs on; Links Notation records *are* this
  `source → target` model.
- **RDF triple stores / property graphs (Neo4j)** — general realizations of
  "knowledge as a graph"; the protocol maps cleanly onto either, neither embedded
  (dependency-light).

**Net conclusion:** for every requirement, either a project component already
realizes it (and is now cited), or a specific, named external component is the
documented reuse target for the scoped future work — no requirement is left both
unimplemented and unplanned, and nothing learned-model is claimed as already
shipped.

---

## 8. Risks

| Risk | Why it matters here | Mitigation in this PR |
|---|---|---|
| **Violating the external-CLI constraint** | Wiring claude/codex into Formal AI could "interrupt your own process". | The loop is driven by the project's own deterministic driver; external CLIs are documented as front-ends *against* the server (§5), never embedded — no `src/agentic_coding/` code calls an external model. |
| **Hidden autonomous action** | Tool execution without an explicit switch would be unacceptable (NON-GOAL). | Two gates: `agent_mode` is the real guard (tools refused without it) and a per-tool permission gate; granting capabilities by default enables nothing while agent mode is off (§4.3). |
| **Unbounded reasoning loops** | An agentic loop that never terminates is a NON-GOAL. | A hard `MAX_TURNS = 12` cap (the recipe needs five); the driver returns `hit_turn_cap` and the CLI surfaces it as an error rather than spinning. |
| **Overclaiming general extraction** | A hand-rolled parser could be mistaken for general NL understanding. | Extraction is split by confidence: general annotations, grounded-or-honest assertions that never guess (§4.6); the closed-class lexicon is data, and the discipline is pinned by tests. |
| **"As is" vs "everything is a link" read as contradictory** | The maintainer disagrees with entities/ontologies but asked to implement them anyway. | Resolved by emitting the primitives *as* Links Notation records — they never exist as anything but links (§4.5), so the protocol is honored in the project's own meta-language. |
| **Docs drifting from the code** | A case study describing a deleted module would mislead. | A grounded recipe (`data/meta/agentic-coding-recipe.lino`) and a traceability test pin every constant, tool, stage, function, primitive, bound, and surface to the live source. |
| **Russian-language brittleness** | Cyrillic offsets/tokenization can break on byte vs char boundaries. | Annotations use **character** offsets; tests use the Cyrillic synopsis directly. |

Documenting these honestly is itself the practice `NON-GOALS.md` demands:
*"Case studies should not become marketing pages."*

---

## 9. Files

```
docs/case-studies/issue-468/
├── README.md                     # this analysis
├── formal-protocol-mapping.md    # nine primitives → Links Notation records (R311)
└── raw-data/                     # third-party captures (lint-exempt)
    ├── issue-468.json            # the issue as filed
    ├── issue-468-comments.json   # the maintainer comment (the reframing + brief + constraints)
    ├── pr-469.json               # the pull request
    ├── article-summary.md        # summarized + cited source protocol (the example-task spec)
    └── online-research.md        # summarized + cited online research (R316)
```

Wired into the rest of the repository by:

- `src/agentic_coding/` — the planner (server brain), the driver + offline corpus
  (client), the closed-class lexicon, and the Links Notation formalizer
  (R306–R314).
- `src/protocol.rs` / `src/anthropic.rs` — the shared agentic decision across the
  Chat Completions, Responses, and Anthropic Messages surfaces (R314).
- `src/associative_package.rs` — `pkg_agentic_coding`, the per-tool permission gate
  (R314).
- `src/main.rs` — the `formal-ai agent` subcommand (R306/R314).
- `data/meta/agentic-coding-recipe.lino` — the grounded meta-algorithm recipe.
- `examples/issue_468_agentic_loop.rs` / `examples/issue_468_formalize_text.rs` —
  the worked end-to-end loop and the formalizer tour.
- `REQUIREMENTS.md` — rows **R306–R319** (R317).
- `tests/unit/agentic_coding.rs`, `tests/unit/agentic_surfaces.rs`,
  `tests/unit/specification/agentic_meta_algorithm.rs` — pin the loop on every
  surface, the formalizer's record count and coverage-of-nine, and the recipe's
  fidelity to the code.
- `changelog.d/` — fragments recording the new capability.
