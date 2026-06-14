# Issue 468 Case Study

> **Status:** Protocol implemented "as is" and reduced to links in PR #469.
> **Type:** Documentation + enhancement (new `text_formalization` module + case study).
> **Primary source:** [Формальный протокол для перевода текстов в базу знаний](https://telegra.ph/Formalnyj-protokol-dlya-perevoda-tekstov-v-bazu-znanij-06-10) (Igor Martynov, January 2026)

- **Issue:** <https://github.com/link-assistant/formal-ai/issues/468>
- **Pull request:** <https://github.com/link-assistant/formal-ai/pull/469>
- **Protocol → links mapping:** [`formal-protocol-mapping.md`](formal-protocol-mapping.md)
- **Source-protocol summary (cited):** [`raw-data/article-summary.md`](raw-data/article-summary.md)
- **Online research (cited):** [`raw-data/online-research.md`](raw-data/online-research.md)
- **Worked example:** [`examples/issue_468_text_formalization.rs`](../../../examples/issue_468_text_formalization.rs)
- **Module:** [`src/text_formalization/`](../../../src/text_formalization/)

All raw artifacts referenced below live in [`raw-data/`](raw-data/).

---

## 1. Summary

Issue #468 asks the project to **code, into the application, the capability to
translate texts into a knowledge base** following a concrete protocol — Igor
Martynov's *"Formal protocol for translating texts into a knowledge base"* — with
the canonical input being Pushkin's **«Сказка о рыбаке и рыбке»** ("The Tale of
the Fisherman and the Fish"). The protocol defines **nine primitives** (Concept,
Entity, Predicate/Relation, Assertion, Procedure, Context, Temporal, Modal,
Annotation), an **assertion-centric** representation, a **JSON wire format**, a
**declarative query**, and a deliberate **operational-format-vs-ontology-schema**
distinction. The maintainer's comment adds the case-study deliverable (collect
data → deep analysis + online research → enumerate every requirement → solution
plans with a component survey → land it all in the single PR #469) and one
binding instruction:

> *"I personally [do] not agree with entities and ontologies (because for us
> everything is a link), while we don't agree with requirements we still should
> by default implement them as is."*

The central finding is that the two halves of that instruction are **not in
tension** — they are the same artifact viewed twice. We implement the nine
primitives **exactly as specified** (typed Rust structs, the article's JSON
field-for-field), *and* we show that **every one of them reduces to plain
links/doublets** — a `source → target` edge stream — so the protocol is honored
"as is" while the project's standing position that *everything is a link* is
demonstrated rather than asserted. The two views are byte-checked against each
other in tests.

The deliverables are therefore code **and** documentation, fully traceable:

1. A new [`src/text_formalization/`](../../../src/text_formalization/) module: the
   nine [primitive types](../../../src/text_formalization/primitives.rs), the
   [`KnowledgeBase`](../../../src/text_formalization/knowledge_base.rs) aggregate
   with its canonical JSON wire `ProtocolDocument`, a structured
   [Links-Notation](../../../src/text_formalization/lino.rs) view, the fully
   reduced [doublet stream](../../../src/text_formalization/links.rs), a
   declarative conjunctive [`Query`](../../../src/text_formalization/query.rs)
   (article §9), a curated [tale](../../../src/text_formalization/tale.rs)
   knowledge base exercising all nine primitives, and a constrained, closed-class
   [`Extractor`](../../../src/text_formalization/extract.rs).
2. A `formal-ai formalize` CLI subcommand (`tale` / `extract`) and a worked
   [example](../../../examples/issue_468_text_formalization.rs).
3. [`formal-protocol-mapping.md`](formal-protocol-mapping.md): every primitive
   and every assertion qualifier mapped to its link/doublet realization, with the
   `source → target` shape spelled out per primitive.
4. This case study: collected data, deep analysis, online research, the
   exhaustive requirement list **R306–R319**, and a solution plan plus prior-art
   survey for each.
5. Regression tests (`tests/unit/text_formalization.rs`) pinning the JSON
   round-trip against the article's own example, the doublet count, the
   coverage-of-nine, and the extractor's *never-guess* discipline, so none of the
   above can silently regress.

What is **out of current scope, stated honestly** (not hidden): general
open-domain natural-language extraction — the article's §7 learned pipeline (POS
tagging, dependency parsing, semantic role labeling, NER, coreference) — requires
neural inference, which is a project NON-GOAL. The deterministic extractor
therefore covers exactly one explicit sentence template and **never guesses**;
the general capability is scoped as future work in §6 (R313) with the prior art
surveyed in §7.

---

## 2. Collected Data

The raw, third-party captures (exempt from authored-prose lints) are archived
under [`raw-data/`](raw-data/):

| File | What it is |
|---|---|
| [`raw-data/issue-468.json`](raw-data/issue-468.json) | The issue as filed (`gh issue view 468 --json …`). Labels: `documentation`, `enhancement`. Body (Russian) specifies the nine primitives and links the source article. |
| [`raw-data/issue-468-comments.json`](raw-data/issue-468-comments.json) | The single maintainer comment — the case-study deliverable, the "everything is a link" position, the "implement as is" instruction, and the "don't wire external CLIs into Formal AI" constraint. |
| [`raw-data/pr-469.json`](raw-data/pr-469.json) | The draft pull request this work lands in. |
| [`raw-data/article-summary.md`](raw-data/article-summary.md) | A paraphrased-and-cited summary of the source telegra.ph protocol: the nine primitives with their exact fields, the authoritative JSON example (quoted), the assertion BNF, the worked-example sentence, the §9 declarative query, the §12 nested-assertion form, the §10 metrics, and the §7 NL pipeline. |
| [`raw-data/online-research.md`](raw-data/online-research.md) | Summarized-and-cited research grounding each primitive in established practice: AMR, OpenIE, RDF reification / RDF-star / named graphs, Wikidata qualifiers, FrameNet/PropBank, the general NL→KB pipeline, the associative "everything is a link" model, and the Tale's own provenance. |

Per [NON-GOALS.md](../../../NON-GOALS.md) (*"Research notes should not copy large
external texts; they should summarize and cite sources"*), both research files
quote only short definitional phrases and link every claim to its source. The
Tale text itself is public-domain (Pushkin, 1833) and is referenced, not bulk-copied.

---

## 3. Holistic Requirements

Every requirement extracted from the issue body **and** the maintainer comment
(the two together are the complete specification). These are recorded in
[`REQUIREMENTS.md`](../../../REQUIREMENTS.md) as **R306–R319** under *"Issue #468
Text-To-Knowledge Formalization"*.

| ID | Requirement (intent) | Status |
|---|---|---|
| **R306** | Code into the application the capability to translate texts into a knowledge base per the protocol, with «Сказка о рыбаке и рыбке» as the canonical input. | Done — `src/text_formalization/` + the curated `tale_knowledge_base()`; surfaced via `formal-ai formalize tale`. |
| **R307** | Realize the **nine primitives** with their exact fields (Concept, Entity, Predicate, Assertion, Procedure, Context, Temporal, Modal, Annotation). | Done — `src/text_formalization/primitives.rs`; field-for-field with the article (§4.1 below). |
| **R308** | Make the representation **assertion-centric**: assertions are the units for search, inference, and aggregation — including a declarative query. | Done — `Assertion` is the atomic block; `Query` realizes the article §9 declarative form (`src/text_formalization/query.rs`). |
| **R309** | Keep the **operational format distinct from an ontology schema**: assertions + procedures + modality + provenance carry the facts; Concept/Predicate declarations remain a fact-free reference directory. | Done — `Directory` (catalogue) vs `annotations` (assertions) in `ProtocolDocument`; the directory is omitted from JSON when empty. |
| **R310** | Carry the per-assertion qualifiers the protocol names: **modality, time, context, confidence, provenance**. | Done — `Assertion` fields + `Modal`/`Temporal`/`Context`/`Provenance` types; exercised by the tale (§4.3). |
| **R311** | Honor *"everything is a link"*: implement the primitives **as is**, and additionally **demonstrate every primitive reduces to plain links/doublets**. | Done — `to_lino()` (structured) + `to_links()` (reduced doublet stream) + JSON, all three byte-checked to agree; [`formal-protocol-mapping.md`](formal-protocol-mapping.md). |
| **R312** | Provide a **curated knowledge base for the Tale** that exercises all nine primitives end to end. | Done — `tale_knowledge_base()`; `coverage().covers_all_nine() == true` (3 concepts, 5 entities, 7 predicates, 7 assertions, 1 procedure, 4 contexts, 2 temporals, 7 modals, 1 annotation). |
| **R313** | Provide a **deterministic extractor** for the article's worked example, and scope **general open-domain extraction** honestly (it needs neural inference — a project NON-GOAL). | Done — closed-class `Extractor` reproduces «Пётр открыл магазин в Москве в 2019 году.» exactly and returns `None` on anything outside its template/lexicon; general extraction scoped as future work (§6) with prior art surveyed (§7). |
| **R314** | Address the title's **"agentic mode"**: document how the system would drive an agentic CLI / OpenAI-compatible server to solve such tasks, while honoring the explicit constraint *not to wire external CLIs (claude/codex) into Formal AI*. | Done — §5 documents the agentic-mode flow and its boundary; the deterministic protocol core ships now, the autonomous end-to-end loop is scoped as future work without violating the constraint. |
| **R315** | **Collect issue data** into `docs/case-studies/issue-468/`. | Done — §2 and [`raw-data/`](raw-data/). |
| **R316** | **Deep case-study analysis**, including **online research** for additional facts. | Done — §4 (analysis) + [`raw-data/online-research.md`](raw-data/online-research.md) (8 cited sources). |
| **R317** | **List each and all requirements** from the issue. | Done — this table (R306–R319) plus the prose enumeration below. |
| **R318** | Propose **solutions and solution plans per requirement**, checking **existing components/libraries**. | Done — §6 (per-requirement plans) + §7 (prior-art survey). |
| **R319** | **Plan and execute everything in the single PR #469.** | Done — the module, CLI, example, tests, this case study, the mapping doc, the `raw-data/` captures, the `REQUIREMENTS.md` rows, and the changelog fragment all land in PR #469. |

### Why these fourteen and not more

The issue body is one specification paragraph plus a primitive list; the comment
is the case-study brief plus constraints. R306 is the literal "code it into the
app" ask; R307 is the primitive list; R308 is *"Assertions — единицы, по которым
делаем поиск, выводы, агрегации"* ("assertions are the units for search,
inference, aggregation"); R309 is the *"Чем отличается от онтологии"* ("how it
differs from an ontology") paragraph; R310 is the explicit `Assertion` field set
(modality/time/context/confidence/provenance). R311 is the comment's
everything-is-a-link position combined with "implement as is". R312–R313 are the
two concrete demonstrations the body asks to *"проверить работоспособность"*
("check that the scheme works") — one curated, one extracted. R314 is the issue
**title** ("agentic mode") read against the comment's constraint. R315–R319 are
the comment's five-part case-study brief. No requirement is implied beyond these
without over-reading the text. The optional *"test on AI benchmarks"* sentence is
recorded here as explicitly deferred (it depends on the agentic loop of R314) and
is not promoted to a requirement, to avoid overclaiming.

---

## 4. Deep Analysis

### 4.1 The nine primitives, field-for-field

The article specifies each primitive's fields; the module mirrors them exactly.
The mapping is one-to-one (see `src/text_formalization/primitives.rs`):

| Protocol primitive | Article fields | Rust type | Notes |
|---|---|---|---|
| **Concept** | `id, label, type, attributes` | `Concept` | `type` → `concept_type` (Rust keyword); attributes are ordered key/values. |
| **Entity** | `id, label, canonical_forms, attributes` | `Entity` | `canonical_forms` is the surface-form list (synonyms/aliases). |
| **Predicate / Relation** | `id, name, arity, semantics` | `Predicate` | `semantics` is the optional formula/type string. |
| **Assertion** | `id, subject, predicate, object(s), modality, time, context, confidence, provenance` | `Assertion` | the atomic block; `object` is a list of `Term`s (entity / concept / literal / **assertion reference** for nesting). |
| **Procedure** | `id, signature, body, triggers` | `Procedure` | `triggers` are predicate ids that fire the rule. |
| **Context** | situation / validity bounds | `Context` | id + label + description + ordered properties (e.g. `location = Москва`). |
| **Temporal** | `instant / interval / relative` | `Temporal` enum | three variants; `calendar_year()` projects an `Instant` year for querying. |
| **Modal** | `belief / obligation / possibility / …` | `Modal` | `kind` + `confidence`; default is a plain `assertion` at confidence `1.0`. |
| **Annotation** | source ref + `offsets` + `language` + `tokenization` | `Annotation` | grounds an assertion in a `[start, end]` character span of the source. |

The JSON serialization (`ProtocolDocument`) is field-for-field the article's own
example. The clinching evidence is a **round-trip test**: the article's verbatim
JSON example parses into `ProtocolDocument` and re-serializes to the same value,
so the wire format is conformant, not merely inspired by the article.

### 4.2 Assertion-centric, not ontology-centric (R308 / R309)

The article is explicit that this is **not** an ontology: *"Онтология — это набор
типов и связей (schema). Наш формат — операционный: Assertions + процедуры +
модальность + provenance. Онтология остаётся как справочник."* The module
encodes that split structurally: `ProtocolDocument` carries a `directory` (the
fact-free catalogue of Concept/Entity/Predicate/Procedure/Context/Annotation
declarations) and an `annotations` array (the assertions — the facts). The
directory is **omitted from the JSON when empty**, so a pure-assertion document
is exactly `{doc_id, annotations}` — the operational format with no schema
overhead. Search/inference/aggregation run over assertions: the `Query` type is a
conjunctive filter over the assertion stream, realizing the article's §9
declarative form (`SELECT ?x WHERE subject = … AND predicate = … AND
ctx.location = … AND time.year = …`).

### 4.3 The Tale exercises every primitive (R312)

`tale_knowledge_base()` is the curated proof that the representation is
expressive enough for the issue's canonical input. It encodes the moral arc of
«Сказка о рыбаке и рыбке» as 7 assertions over 5 entities (old man, old woman,
golden fish, sea, trough), 7 predicates, and 3 abstract concepts (greed, wish,
ransom), and it deliberately reaches for the harder primitives:

- **Temporal** — `a:catch` and `a:release` carry *relative* time ("at the start
  of the tale", "after he caught it").
- **Context** — `a:catch` is bound to `ctx:seaside`; `a:final` to `ctx:final`
  ("return to the initial state"), capturing the validity-bounds idea.
- **Modal** — `b:make_ruler` is a *possibility* (confidence 0.5); `a:demand_sea`
  is a *desire* (0.9); the rest are plain assertions. This is exactly the
  belief/obligation/possibility spectrum the article names.
- **Nested assertion (§12)** — `a:demand_sea`'s object is **not** an entity but a
  reference to another assertion (`b:make_ruler`): "the old woman demanded *[that
  the fish make her a ruler]*". Higher-order assertions fall out of `Term` having
  an `AssertionRef` variant.
- **Procedure** — `proc:escalate` is triggered by `pred:grant`, modeling the
  tale's escalation rule.
- **Annotation + Provenance** — `a:catch` is grounded in a source span and
  records the curating extractor in its provenance.

`coverage().covers_all_nine()` returns `true`, and the exact counts are pinned by
a test so the curated KB cannot lose a primitive unnoticed.

### 4.4 Everything is a link (R311)

The maintainer's position — *"for us everything is a link"* — is demonstrated, not
argued. `KnowledgeBase::to_links()` reduces the **entire** knowledge base to a
flat stream of `Link { id, source, target }` doublets: each concept becomes
`concept → Concept` / `concept → label` edges; each assertion becomes
`assertion → subject`, `assertion → predicate`, `assertion → object:i`,
`assertion → time`, `assertion → context`, `assertion → modal`,
`assertion → confidence`, `assertion → provenance` edges; literals become
`lit:<datatype>:<value>` nodes. For the Tale this is **115 doublets**, and
`to_links_lino()` serializes them in the same Links Notation the rest of the
crate emits. The three views — structured `.lino`, protocol JSON, and the reduced
doublet stream — are all derived from the same `KnowledgeBase`, and the JSON view
round-trips losslessly, so "implement the primitives as is" and "everything is a
link" are the **same** object rendered three ways. The per-primitive reduction is
tabulated in [`formal-protocol-mapping.md`](formal-protocol-mapping.md).

### 4.5 Why general extraction is out of scope, and what is in scope (R313)

The article's §7 describes the general pipeline: tokenization → POS → dependency
parse → semantic role labeling → NER → coreference → assertion assembly. Every
step after tokenization is a **learned-model** problem; doing it well is exactly
what large language models are for, and `formal-ai` performs **no neural-network
inference** by design (`NON-GOALS.md`). Pretending otherwise would mean a brittle
hand-rolled parser masquerading as general NL understanding. Instead, the module
ships a **constrained, closed-class** extractor: it recognizes one explicit
sentence template (`<Subject> <Predicate> <Object> в <Location> в <Year>`) over a
fixed lexicon, reproduces the article's worked example exactly, and returns
`None` for **anything** outside that template or lexicon. It never guesses. This
is the honest deterministic core; the general capability is the agentic-mode work
of §5/R314 and the neuro-symbolic future work of §7 — named, not buried.

### 4.6 What the online research adds

[`raw-data/online-research.md`](raw-data/online-research.md) grounds each design
choice in established practice and confirms the protocol is a reasonable synthesis
rather than an outlier: the assertion ≈ AMR/OpenIE **triple**; statement-level
modality/time/context/confidence/provenance ≈ **RDF reification / RDF-star /
named graphs** and **Wikidata qualifiers + references**; the Predicate directory ≈
**FrameNet/PropBank** predicate inventories; and the reduction to doublets ≈ the
associative *"everything is a link"* model the project already runs on. Nothing in
the literature contradicts the protocol; the qualifier set it names is precisely
what deployed knowledge graphs (Wikidata) found necessary in practice.

---

## 5. Agentic Mode — the title, the constraint, and the boundary (R314)

The issue **title** is *"Our system should be able to solve such tasks in agentic
mode."* The comment sketches the intended mechanism: stand up the project's
OpenAI-compatible server, point an agentic CLI ([`link-assistant/agent`](https://github.com/link-assistant/agent),
falling back to `gemini-cli`) at it, and have Formal AI *"call all the tools from
any agentic CLI, understand errors from tools, … call bash commands, … web fetch
and web search, to actually complete the task"* — i.e. autonomously produce the
knowledge base from the raw text.

This case study honors the title while respecting the comment's **explicit
boundary**:

> *"Also don't use claude or codex to connect them to our Formal AI, as that may
> interrupt your own process, and also break execution of other tasks."*

So the autonomous end-to-end loop is **documented and scoped, not wired up**. The
agentic-mode flow, once built, is:

1. **Serve** — `formal-ai serve` already exposes the OpenAI-compatible endpoints
   (`src/server.rs`, `src/protocol.rs`); an agentic CLI can target it as its
   model backend.
2. **Tool surface** — the project already has the symbolic tool primitives the
   loop needs: bash/command execution (`src/agent.rs` `AgentWorkspace`), web
   fetch/search (`src/web_engine_core.rs`, `src/web_search_core.rs`), and GitHub
   evidence collection (`src/github_logs.rs`). The agentic loop is *orchestration*
   over these, not new capability.
3. **Formalize** — each agent turn's text is reduced to assertions by the very
   `text_formalization` module shipped here; the curated/extracted KB is the
   reference target a learned loop would be scored against.
4. **Reconstruct** — the comment's fallback ("use claude/codex JSON sessions to
   reconstruct reasoning steps") becomes a *replay* problem: an agent's saved JSON
   session is a sequence of tool calls that can be re-expressed as assertions +
   procedures in exactly this format, so Formal AI can learn the *flow* without
   being *driven* by the external model at runtime — preserving the boundary.

What ships **now** is the deterministic substrate that makes (3) and (4)
well-defined: a concrete, testable target format. The learned orchestration of
(1)–(2) that would make the loop fully autonomous is the future work, named so it
is not mistaken for already-done.

---

## 6. Solution Plans (per requirement)

Each plan names the chosen approach and the existing component it reuses, per
R318. The survey those choices draw on is §7.

### R306 / R307 / R310 — Code the protocol with its nine primitives and qualifiers
**Approach (chosen, done):** a dedicated `src/text_formalization/` module of typed
structs, one per primitive, with builder methods and `serde` derives matching the
article's JSON field names. **Existing components reused:** `serde`/`serde_json`
(already crate dependencies) for the wire format; the crate's own
`links_format::format_lino_record` for Links-Notation rendering — the same helper
the memory/seed subsystems use, so the output is consistent with the rest of the
project. **Alternative considered:** a generic property-graph crate (e.g.
`petgraph`) — rejected because the protocol's value is its *named, typed* fields,
which a generic graph would erase; the reduction to a generic graph is provided
*on top* (`to_links()`) rather than *instead*.

### R308 — Assertion-centric representation + declarative query
**Approach (chosen, done):** `Assertion` is the atomic unit; `Query` is a
conjunctive filter with a small parser for the article's §9 textual form. **Existing
component reused:** the crate's existing `links_query` module is the precedent for
"parse a tiny query language over a link store"; `Query` follows its shape
(builder + `parse` + `Error` enum) rather than inventing a new idiom. **Alternative
considered:** embedding a full SPARQL engine — rejected as wildly disproportionate
to a conjunctive equality/threshold filter and contrary to the dependency-light,
WASM-safe posture.

### R309 — Operational format vs ontology schema
**Approach (chosen, done):** structural separation in `ProtocolDocument`
(`directory` catalogue vs `annotations` facts), with the directory `skip`-ped from
JSON when empty so the operational format carries no schema weight. No external
component needed; this is a serialization-shape decision.

### R311 — Everything is a link
**Approach (chosen, done):** a `to_links()` reduction emitting `Link{id,source,
target}` doublets for every primitive and every assertion slot, plus
`to_links_lino()` to serialize them. The structured `.lino`, the JSON, and the
doublet stream are three renderings of one `KnowledgeBase`, byte-checked to agree.
**Existing component reused:** the project's own doublet model (`DoubletLink`,
`link_store`) is the conceptual target; the reduction maps the protocol onto it.
**Alternative considered:** an RDF/RDF-star export — noted in
[`formal-protocol-mapping.md`](formal-protocol-mapping.md) as a natural future
bridge (the doublet stream is isomorphic to reified RDF triples) but not shipped,
to avoid an `rdf` dependency for a demonstration.

### R312 — Curated Tale knowledge base
**Approach (chosen, done):** `tale_knowledge_base()` hand-curates the Tale to
exercise all nine primitives, with a coverage assertion pinned by test. This is
the "does the scheme actually work" demonstration the issue body asks for, on the
issue's own canonical input.

### R313 — Deterministic extractor + general extraction scoped
**Approach (chosen, done):** a closed-class, template-bound `Extractor` that
reproduces the article's worked example and never guesses (returns `None` off-
template). **Why not general extraction:** it requires neural inference (a project
NON-GOAL); §4.5 and §7 explain and survey the alternative. **Existing components
surveyed as the future reuse path:** spaCy/Stanza (POS/dependency/NER), AllenNLP
(SRL/coreference), the AMR parsers, and OpenIE systems — all named in §7 as what a
learned loop would call, consistent with the agentic-mode design of §5.

### R314 — Agentic mode, within the constraint
**Approach (chosen, done):** document the flow and its boundary (§5); ship the
deterministic target format now; scope the autonomous loop as future work without
wiring external CLIs into Formal AI. **Existing components reused (already in the
repo):** `src/server.rs` (OpenAI-compatible serving), `src/agent.rs`
(workspace/command execution), `src/web_*_core.rs` (fetch/search). **External
reuse target named:** `link-assistant/agent` as the agentic CLI front-end, with
`gemini-cli` as the documented fallback — invoked *against* the server, not
*embedded into* Formal AI, per the constraint.

### R315 / R316 / R317 / R318 / R319 — The case-study brief
**Approach (chosen, done):** §2 collects the data; §4 + the research file do the
deep analysis with online research; §3 enumerates R306–R319; §6 + §7 give the
plans and survey; and every artifact lands in the single PR #469. **Existing
component reused:** the case-study layout and the `raw-data/online-research.md`
convention established by issue-451 / issue-408, copied here for consistency.

---

## 7. Existing Components / Prior Art Surveyed (R318)

What the field already built for text-to-knowledge formalization, and what
`formal-ai` reuses, re-expresses, or names as a future reuse target. Full
citations are in [`raw-data/online-research.md`](raw-data/online-research.md).

### Meaning representations (the assertion)
- **Abstract Meaning Representation (AMR)** — rooted, labeled sentence graphs with
  PropBank predicate senses. The `Assertion` (subject/predicate/object over a
  predicate directory) is the same triple idea; AMR is the richer, learned target
  a neural loop would emit. *Surveyed as the future extraction target, not embedded.*
- **Open Information Extraction (OpenIE / Stanford OpenIE)** — schema-free
  `(arg1, relation, arg2)` triples straight from text. This is precisely the
  assertion shape; OpenIE is the closest existing *general* extractor and is named
  as the reuse path for R313's future work.

### Statement-level metadata (the qualifiers)
- **RDF reification, RDF-star (RDF 1.2), named graphs** — the standard ways to
  attach metadata (time, source, certainty) to a statement. The `Assertion`'s
  modality/time/context/confidence/provenance fields are the same need; the
  `to_links()` doublet stream is isomorphic to a reified-triple encoding, so an
  RDF-star export is a natural future bridge (noted, not shipped).
- **Wikidata qualifiers + references** — the largest deployed knowledge graph
  attaches `point in time`, `determination method`, and `reference` to statements.
  This is direct empirical support that the protocol's qualifier set is the
  *right* set: a production KB independently found the same fields necessary.

### Predicate inventories (the directory)
- **FrameNet, PropBank, VerbNet** — curated predicate/role inventories. The
  `Predicate` directory (fact-free declarations with `semantics`) is the same
  reference-catalogue role; these are the reuse targets if the predicate set is
  ever grounded against a standard inventory.

### The associative substrate (everything is a link)
- [`linksplatform/doublets-rs`](https://github.com/linksplatform/doublets-rs) —
  the native doublet store the project already runs on; `to_links()` targets this
  exact `source → target` model.
- **RDF triple stores / property graphs (Neo4j)** — the general-purpose
  realizations of "knowledge as a graph". The doublet reduction shows the protocol
  maps cleanly onto either; neither is embedded, to stay dependency-light.

### Agentic-mode front-ends (the title)
- [`link-assistant/agent`](https://github.com/link-assistant/agent) and
  [`gemini-cli`](https://github.com/google-gemini/gemini-cli) — agentic CLIs that
  can target an OpenAI-compatible backend. Named as the front-end reuse target for
  R314, invoked against `formal-ai serve`, *not* wired into the engine (constraint).

**Net conclusion:** for every requirement, either a project component already
realizes it (and is now cited), or a specific, named external component is the
documented reuse target for the scoped future work — no requirement is left both
unimplemented and unplanned, and nothing learned-model is claimed as already
shipped.

---

## 8. Risks

| Risk | Why it matters here | Mitigation in this PR |
|---|---|---|
| **Overclaiming general extraction** | A hand-rolled parser could be mistaken for general NL understanding. | The extractor is explicitly closed-class and returns `None` off-template; §4.5 + R313 state the boundary; a test pins the never-guess behavior. |
| **"As is" vs "everything is a link" read as contradictory** | The maintainer disagrees with entities/ontologies but asked to implement them anyway. | Both are shipped as **one** object: typed primitives *and* their doublet reduction, byte-checked to agree, so neither side is compromised. |
| **Violating the external-CLI constraint** | Wiring claude/codex into Formal AI could "interrupt your own process". | The agentic loop is documented and scoped (§5), not wired; external CLIs are named as front-ends *against the server*, not embedded. |
| **Curated KB drifting from the protocol** | A hand-curated Tale could quietly diverge from the article's JSON shape. | The JSON wire format round-trips the article's **own** example in a test; the Tale serializes through the same `ProtocolDocument`. |
| **Russian-language brittleness** | Cyrillic offsets/tokenization can break on byte vs char boundaries. | Annotations use **character** offsets; the extractor tokenizes on Unicode alphanumerics; tests use the article's Cyrillic example directly. |

Documenting these honestly is itself the practice `NON-GOALS.md` demands:
*"Case studies should not become marketing pages."*

---

## 9. Files

```
docs/case-studies/issue-468/
├── README.md                     # this analysis
├── formal-protocol-mapping.md    # nine primitives → links/doublets mapping (R311)
└── raw-data/                     # third-party captures (lint-exempt)
    ├── issue-468.json            # the issue as filed
    ├── issue-468-comments.json   # the maintainer comment (the case-study brief + constraints)
    ├── pr-469.json               # the pull request
    ├── article-summary.md        # summarized + cited source protocol
    └── online-research.md        # summarized + cited online research (R316)
```

Wired into the rest of the repository by:

- `src/text_formalization/` — the nine primitives, JSON/lino/doublet codecs, the
  query, the curated tale, and the extractor (R306–R313).
- `src/main.rs` — the `formal-ai formalize` subcommand (R306).
- `examples/issue_468_text_formalization.rs` — the worked end-to-end tour.
- `REQUIREMENTS.md` — rows **R306–R319** (R317).
- `tests/unit/text_formalization.rs` — pins the JSON round-trip against the
  article example, the doublet count, coverage-of-nine, the query, and the
  extractor's never-guess discipline.
- `changelog.d/` — a `minor` fragment recording the new module and capability.
```
