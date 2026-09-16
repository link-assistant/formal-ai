# Plan 04 — Formalization depth: concepts and procedures, not stored sentences (bottleneck B4 of #1138)

Status: planned, nothing implemented. Depends on plan 01 (B1 live concept lookup)
for the retrieval it asks from. Written before the code so the work can be resumed
from any point. A box is ticked in the same commit that lands its leaf; a leaf that
turns out wrong is struck through with the reason, never deleted.

## Issues addressed

- **#1138 B4** — "Formalization is shallow: sentences are preserved, concepts and
  procedures are not extracted": asks that the formalizer emit an explicit need for
  every unresolved concept, that B1 satisfy it, and that the result be concepts and
  procedures the B2 composer can consume. Proof: an unfamiliar requirement in
  en/ru/hi/zh/es formalizes to the same concept graph. This plan delivers all of it.
- **#1138 B1** — plan 01 builds `concept_lookup::lookup_surface`. This plan is its
  first non-coding consumer and its first *recursive* consumer: a retrieved gloss is
  itself formalized, and its own unresolved surfaces become new needs, bounded by
  depth.
- **#710** (maintainer instruction, `docs/case-studies/issue-710/plans/README.md:14-36`)
  — "reconstruct from the formalized knowledge collected in the internet the step by
  step guide/algorithm. **Formalization itself may need recursive knowledge
  collection from trusted sources.**" This plan implements that sentence literally.
- **#468** (agentic coding mode) — R314's manual-audit note
  (`docs/requirements-traceability.md:376`): "falls back to the seeded fairy-tale KB
  rather than reflecting custom --task". This plan removes the fallback's cause, not
  only its symptom.
- **#956** — delivered inline-source selection so a quoted `--task` is formalized
  instead of the canonical tale
  (`src/agentic_coding/formalization_recipe.rs:44-56`). It did not make the
  *extraction* of that text any deeper. This plan does.
- **#873** — established that "not knowing is not the end" for an unresolved
  *prompt*. This plan extends it to an unresolved *concept inside a document the
  system is formalizing*.
- **#919** — PR #992's ledger accepts only the repository's own
  `formal_ai_coding_procedure_v1` shape (`docs/case-studies/issue-919/README.md`
  § Dependency and scope: "new operation kinds must add their own bounded executor
  and verification oracle instead of interpreting arbitrary fetched code"). This
  plan supplies a *typed extracted procedure* with its own verification obligation,
  so a procedure extracted from a trusted source has a legitimate route in.
- **#1085 / R1085-9** — concepts and procedures as links with provenance is the
  links-network-as-system-of-record step for the formalizer.
- **#559 / R333, R340-R344** — `meta_frame::NeedLedger` already has the need
  vocabulary; this plan makes the formalizer a producer of rows in it rather than a
  parallel universe.
- **Issues plan 13's coverage table names this plan as a deliverer of** (added by
  the 2026-09-16 reconciliation): **#869** (`по Грузии` is a concept the
  formalizer must ground before plan 10 can schedule anything); **#942** (the
  redaction skill reasons over formalized concepts, never a `try_*` handler);
  **#722** (the greeting clause must stop consuming the request — this plan's
  clause-splitting leaf is what plan 10 leaf 14 waits on); **#1063** (a magnitude
  with a unit is a concept, not a sentence); **#453** (a moonshot's approaches
  are formalized before they are split).
- **Plan 00 of this batch** (`00-root-causes-and-integration.md` §4.1, §4.3) fixes
  the `Need` record and the `evidence` record. This plan gives `Need` its single
  Rust definition (`src/formalization/needs.rs`), makes the formalizer its first
  producer, and leaves `evidence` to plan 05 — `ProcedureStep::verified` is
  introduced here and set only there, so the two plans do not both invent a
  satisfaction rule. Every plan-00 name is adopted verbatim; plan 14's
  reconciliation leaf has nothing to rename here.

## Current state

### The formalizer's own documentation says it is shallow

`src/agentic_coding/formalize.rs:13-16`:

> Current extraction is shallow: preserving a source sentence does not mean
> its concepts or procedures have been understood. Open-domain recursive
> discovery and grounded semantic extraction remain implementation work.

`src/agentic_coding/lexicon.rs:1-11`:

> The current extractor recognizes a closed lexicon stored as data
> (`LEXICON_LINO`) and preserves unrecognized text without inventing relations.
> Recursive source discovery and semantic extraction beyond this catalogue
> remain implementation work.

### The closed catalogue is one Russian fairy tale, and it contains one concept

`src/agentic_coding/lexicon.rs:14` binds `LEXICON_LINO` to
`data/agentic-coding/fisherman-lexicon.lino`. That file is 102 lines and contains,
counted by record kind:

| Record | Count | Lines |
| --- | ---: | --- |
| `work` | 1 | `:1-4` |
| `lexeme` | 15 | `:5-85` |
| `concept` | **1** | `:86-89` (`concept:greed`, `label "жадность"`, `type "trait"`) |
| `procedure` | **1** | `:90-94` (`proc:escalate`, `signature "escalate(wish) -> larger_wish"`) |
| `context` | 2 | `:95-102` |

The source text it describes is 395 bytes:
`data/agentic-coding/fisherman-synopsis.txt`, seven Russian sentences. **The entire
semantic depth of the formalizer is one trait and one procedure, both about one
document.**

### For any other document, the formalizer stores sentences

`src/agentic_coding/formalize.rs:116` — `let work = lexicon.best_work_for(text);`.
For a text that is not the tale, `work` is `None`. Then:

- `:142` — `let extracted = work.and_then(|work| work.extract(&sentence.text));` is
  `None` for every sentence.
- `:192-203` — every sentence takes the `None` branch:
  `subject: Term::literal("—")`, `predicate: PredicateUse { id: "pred:states" }`,
  `object: Term::literal(&sentence.text)`, `natural_language: Some(sentence.text)`.
  The sentence is stored, verbatim, as the object of a generic relation.
- `:215-235` — `concept_records`, `procedure_records` and `context_records` are
  populated only inside `if let Some(work) = work`, plus text-derived concepts drawn
  from `used_concepts`, which is populated at `:143-158` only from a successful
  `work.extract`. With `work = None`, all three are empty.

So the observed counts are: `annotations` ≥ 1, `assertions` ≥ 1, everything else 0.
`covered` (`:425-429`) is therefore exactly `["assertion", "annotation"]`, and the
honest report is **2 of 9 protocol primitives** — the number recorded in
`docs/case-studies/issue-710/plans/06:308` and `:07:93,364`.

### The measured probes say the same thing twice

`docs/case-studies/issue-710/plans/06:247-253` (the memory-contract probe):

> Agent session `ses_f5a131ed8ffe4pk0sDxINN0ZSn` completed a three-round
> requirement-formalization run. Its reviewed knowledge base is under
> `agent-cli-evidence/memory-contract/`; all five statements survive, but concept
> and procedure counts are zero. The generic `pred:states` fallback is source
> preservation, not resolved semantics. Its final report incorrectly says all
> nine primitives are realized.

`docs/case-studies/issue-710/plans/07:26-28` (observed limits):

> The live memory-contract formalizer preserves source sentences, but produces
> no concepts or procedures. Deep formalization therefore remains an open
> prerequisite itself.

`docs/case-studies/issue-710/plans/07:361-367` (the source-identity probe):

> Independent artifact inspection finds both original sentences, two annotations,
> two literal assertions, no tale identity, and zero concepts or procedures.

### The formalizer cannot say what it lacks

`FormalizationSummary` (`src/agentic_coding/formalize.rs:68-82`) has nine count
fields and `covered: Vec<String>`. There is **no** field for an unresolved surface,
an unrecognised relation, or a concept the formalizer could not ground. A sentence
it does not understand is indistinguishable, in the summary, from a sentence it
understood perfectly — both contribute one assertion and one annotation. There is
therefore no signal that a lookup could satisfy, which is exactly why B4 says "the
maintainer's recursion cannot start".

Two need vocabularies already exist elsewhere and neither is used here:

- `src/meta_frame.rs:44-61` — `NeedStatus { Pending, Planned, Satisfied, Deferred,
  Blocked, Rejected }` with `NeedLedger` at `:643-700`.
- `src/coding/concept_discovery.rs:110-116` — `ConceptNeed { phrase, structures,
  candidates, status: String }` with `"blocked"` / `"satisfied"` string statuses
  (`:222-230`).

The formalizer produces rows in neither.

### The recipe pins one query and one URL as constants

`src/agentic_coding/formalization_recipe.rs:22-29`:

```rust
pub const SEARCH_QUERY: &str = "Пушкин Сказка о рыбаке и рыбке полный текст";
pub const CANONICAL_SOURCE_URL: &str =
    "https://ru.wikisource.org/wiki/Сказка_о_рыбаке_и_рыбке_(Пушкин)";
pub const KB_PATH: &str = "knowledge-base.lino";
```

`docs/meta-algorithm.md:216-218` records this as a *feature* of the recipe —

> 2. **Pin the canonical plan as named constants** (`SEARCH_QUERY`,
>    `CANONICAL_SOURCE_URL`, `KB_PATH`) so the recipe is data, not scattered
>    literals.

— and `:214-215` records the recognizer as

> 1. **Recognise the agentic task** from the latest user turn against a small
>    closed keyword set

`plan_formalization_step` (`:60-116`) uses `SEARCH_QUERY` at `:84` and
`CANONICAL_SOURCE_URL` at `:89` whenever the task does not quote its own source. A
custom `--task` that *names* a document instead of quoting it therefore searches for
Pushkin. #956 added the inline-quote route (`:44-56`) but left the un-quoted route
exactly as it was; `docs/case-studies/issue-710/plans/07:371-377` confirms:
"Unknown unquoted sources still reach the legacy fallback; general source-role
resolution/discovery is open."

### Downstream, an extracted procedure has no consumer

`src/coding_research_learning.rs:29` defines `SOURCE_HEADER = "Formal AI coding
procedure"` and the loop accepts only that versioned shape (the #919 case study:
"accept only the versioned `Formal AI coding procedure` source shape with an
explicit SPDX license expression, task, language, operation, and operands"). A
procedure extracted from wikiHow or a Stack Exchange answer has no route into the
procedure ledger, so even a perfect extractor would produce records nothing reads.
`src/how_to_guide.rs:127-152` (`GuideStep`) is the one structure in the tree that
*does* carry an ordered, provenance-bearing, licensed procedure step — and it is
rendered to markdown (`src/how_to_guide/render.rs`), never formalized to links.

### Five-language state: the formalizer recognises two languages and one sentence terminator

`src/agentic_coding/formalize.rs:120`:

```rust
let language = if has_cyrillic(text) { "ru" } else { "en" };
```

Hindi, Chinese and Spanish text are all labelled `en`. And `segment_sentences`
(`:471-498`) splits at `:477` on `matches!(character, '.' | '!' | '?')` only — not
`。` (U+3002, Chinese), not `।` (U+0964, Devanagari danda). A Chinese or Hindi
document is therefore **one single sentence** spanning the whole text, producing one
annotation and one assertion regardless of length.

### What happens today for one concrete unfamiliar requirement in each language

Held-out concept: **isogram** (verified absent from `src`, `data`, `docs`, `tests` by
grep — see plan 01). The task is a *requirement to formalize*, driven through
`formal-ai agent --task`:

| Language | Task (verbatim) | What happens today |
| --- | --- | --- |
| en | `Formalize this requirement: an isogram check must reject any word that repeats a letter.` | `is_formalization_task` matches the formalize verb (`formalization_recipe.rs:32-35`); no quoted span, so `inline_formalization_source` returns `None` (`:44-56`); the planner searches `SEARCH_QUERY` (Pushkin) and fetches `CANONICAL_SOURCE_URL`; on failure it formalizes `CANONICAL_FISHERMAN_SYNOPSIS` (`:96-99`). The user's requirement never reaches the formalizer. |
| ru | `Формализуй требование: проверка на изограмму должна отклонять слово с повторяющейся буквой.` | Same, and the fallback text is in the same language so the substitution is least visible. |
| hi | `इस आवश्यकता को औपचारिक करें: आइसोग्राम जाँच किसी भी दोहराए गए अक्षर वाले शब्द को अस्वीकार करे।` | Same; and were the text to reach the formalizer, `has_cyrillic` labels it `en` and `segment_sentences` finds no `.`/`!`/`?` before the `।`, so the whole task is one annotation. |
| zh | `请把这个需求形式化:isogram 检查必须拒绝任何含有重复字母的单词。` | Same; `。` is not a terminator, so one annotation, one `pred:states` assertion, language `en`. |
| es | `Formaliza este requisito: una comprobación de isograma debe rechazar cualquier palabra que repita una letra.` | Same; language `en`. |

With the text quoted (the #956 route), all five reach `formalize_text_to_links` and
all five produce: `concepts 0`, `entities 0`, `predicates 1` (`pred:states`),
`assertions` = sentence count, `procedures 0`, `contexts 0`, `temporals 0`,
`modals 0`, `annotations` = sentence count, **`2 of 9`** — and for hi/zh,
`assertions 1`, `annotations 1`, because the segmenter found no terminator.

### Ledgers and tests that record the gap

- `data/seed/meanings-formalization-report.lino:1-54` — six records whose whole
  purpose is to make the report honest: "Report the observed covered count, total
  primitive count and knowledge base; **do not infer coverage from the scheme
  declaration**" and "Two observed primitive kinds out of nine declared kinds are
  reported as 2 of 9." The seed exists to stop the system lying about a number it
  cannot raise.
- `docs/requirements-traceability.md:376` — R314, "manually confirmed 2026-08-04
  (audit): … (see audit finding: falls back to the seeded fairy-tale KB rather than
  reflecting custom --task)".
- `docs/case-studies/issue-710/plans/07:280-286` — the source-identity defect:
  "treats any occurrence of `рыбак`, `fisherman` or `сказк` as a reference to the
  cached fairy tale", repaired at `:311-314` by exact normalized title matching.
- `docs/case-studies/issue-710/plans/07:416-421` — the open list still names
  "recursive semantic extraction".
- `tests/unit/agentic_coding.rs` and `tests/unit/specification/agentic_meta_algorithm.rs`
  pin the current behaviour, including `meta_primitive` count 9 against
  `PRIMITIVE_KINDS` — a declaration of nine kinds, two of which are ever produced.

## Root causes

1. **Concepts and procedures are a per-document hand-authored catalogue, not an
   extraction.** Evidence: `src/agentic_coding/formalize.rs:215-235`;
   `data/agentic-coding/fisherman-lexicon.lino:86-102` (one concept, one procedure).
   Mechanism: the formalizer's semantic vocabulary is exactly what a human already
   wrote about one text. Every new document is, by construction, semantically empty —
   which is memoization in its purest form: knowledge is present only where it was
   pre-placed.

2. **Unrecognised text is stored rather than analysed, and storage is reported as a
   covered primitive.** Evidence: `src/agentic_coding/formalize.rs:192-203`
   (`pred:states` + whole sentence as a literal object); `:425-429` (`covered`
   counts any non-zero kind).
   Mechanism: "assertion" is satisfied by preservation, so the coverage number
   cannot distinguish understanding from transcription. The system's own quality
   signal is blind to the defect it has.

3. **The formalizer has no representation for what it does not know.** Evidence:
   `FormalizationSummary` at `src/agentic_coding/formalize.rs:68-82` — nine counts,
   no unresolved field; no `NeedLedger` row is ever produced.
   Mechanism: the B1 lookup is a service with no caller, because nothing in the
   formalizer can phrase a question. This is precisely the recursion B4 says cannot
   start.

4. **Two need vocabularies exist and the formalizer belongs to neither.** Evidence:
   `src/meta_frame.rs:44-61` versus `src/coding/concept_discovery.rs:110-116`
   (`status: String`).
   Mechanism: even if the formalizer emitted needs, there is no single ledger the
   loop, the coding path and the formalizer all write to, so a need raised in one
   place cannot be satisfied by evidence gathered in another.

5. **The agentic recipe pins one query and one source URL as constants.** Evidence:
   `src/agentic_coding/formalization_recipe.rs:22-29,84,89`;
   `docs/meta-algorithm.md:216-218` documents the pinning as intended design.
   Mechanism: the retrieval half of "recursive knowledge collection from trusted
   sources" is hard-wired to one document, so a requirement can only be formalized
   against a text it happens to quote.

6. **An extracted procedure has no typed consumer.** Evidence:
   `src/coding_research_learning.rs:29` and the #919 scope statement;
   `src/how_to_guide.rs:127-152` (`GuideStep`) renders to markdown only.
   Mechanism: extraction without a consumer is another proposal-only loop (B7).
   Doing the extraction first and the consumer never is how the repository arrived
   at 48 pending handler migrations.

7. **Language handling is two-valued and sentence segmentation is Latin-only.**
   Evidence: `src/agentic_coding/formalize.rs:120,471-498`.
   Mechanism: the doctrine's five-language proof is unreachable — hi and zh
   documents cannot even be segmented, so "the same concept graph in five languages"
   is not merely unmet, it is not currently expressible.

## Solution options

### Option A — Grow the closed lexicon into an open one, still hand-authored

**Description.** Keep the architecture; replace one fairy-tale lexicon with a large,
multi-domain `.lino` lexicon of concepts, predicates and procedures, seeded from
WordNet/Wikidata exports, with per-language lexemes.

**Architecture sketch.** `data/seed/lexicon-open-*.lino` (sharded under the
1,500-line cap), `Lexicon::standard()` loads all shards, `best_work_for` becomes
`best_domain_for`, extraction unchanged.

**Pros.** No new machinery; extraction code already works; deterministic and
offline; the five-language surface problem is solved by adding lexemes.

**Cons.** This is the memoization the doctrine forbids, at scale. A word not in the
shipped lexicon is still invisible, so generality is bounded by the size of a
committed table rather than by the reach of a lookup. It contradicts the maintainer
instruction directly ("the goal is not to know everything in advance"). It also
cannot honour the forget-and-rediscover rule: a deleted lexicon shard is not
rediscoverable, it is simply lost capability.

**Doctrine fit.** Generalization: fails. No hard-coding: fails. Honesty: fine.
Associative stack: fine. Forget/rediscover: fails.
**Effort.** ~4 leaves, very large data. **Risk.** Low technical, unacceptable
doctrinal.

### Option B — Need-emitting formalizer over the B1 lookup, with typed extraction and one shared need ledger

**Description.** Make the formalizer a *producer of needs* and a *consumer of
senses*. Every surface it cannot ground becomes a `Need` row in one shared
ledger; plan 01's `concept_lookup::lookup_surface` satisfies it; the retrieved sense
is formalized in turn (bounded recursion); grounded results become `concept`,
`predicate` and `procedure` links with provenance; ordered procedural text (from
`GuideStep`, from a documentation page, from a numbered list in any captured source)
becomes a typed `ExtractedProcedure` the B2 composer and the #919 ledger can read.

**Architecture sketch.**

```
 task text  ─────────────────────────────┐
                                         ▼
                 src/formalization/segment.rs   (script-aware sentences)
                                         │
                                         ▼
                 src/formalization/needs.rs     Need rows → NeedLedger
                                         │  unresolved surfaces
                                         ▼
       plan 01: concept_lookup::lookup_surface  →  ConceptSense (provenance)
                                         │
                      ┌──────────────────┴───────────────────┐
                      ▼                                      ▼
   src/formalization/concepts.rs            src/formalization/procedures.rs
   gloss → concept / predicate links        ordered steps → ExtractedProcedure
                      │                                      │
                      └──────────────────┬───────────────────┘
                                         ▼
                 src/formalization/graph.rs   ConceptGraph (links + provenance)
                                         │
             ┌───────────────────────────┼────────────────────────────┐
             ▼                           ▼                            ▼
  agentic_coding::formalize     coding/concept_discovery      coding_research_learning
  (KB document, 9 primitives)   (structures + candidates)     (typed procedure route)
```

Recursion is bounded by the *same* `LookupBounds` plan 01 declares, plus a
`max_concept_depth`: a gloss's own unresolved surfaces become needs at depth+1 and
stop at the bound, reported as `blocked`, never guessed.

**Pros.** Implements the maintainer's recursion sentence literally. One need
vocabulary across the loop, the coding path and the formalizer. Extraction has real
consumers on day one. Provenance is carried end to end, so a concept can be
forgotten and rediscovered to the same content id. Five-language proof becomes
expressible: the same requirement in five languages must produce one
`ConceptGraph::identity()`.

**Cons.** Largest surface: a new module tree, changes to a fixture-pinned agentic
recipe, changes to `concept_discovery`, and a new typed shape crossing into #919's
review-gated ledger. Hard dependency on plan 01. The "same graph in five languages"
invariant is demanding and may fail honestly at first.

**Doctrine fit.** Generalization: strong. No hard-coding: strong (every gloss comes
from a source). Honesty: strong (unresolved is a first-class status with a count).
Associative stack: strong (the graph is links; the extraction rules are `.lino`).
Forget/rediscover: strong.
**Effort.** ~14 leaves. **Risk.** Medium-high, but decomposable — each leaf has its
own failing test first.

### Option C — Formalize by translation into the existing structural-meaning vocabulary only

**Description.** Do not build a general concept graph. Instead, make the formalizer
map every requirement onto the 82 seeded structural meanings of
`data/seed/meanings-coding-structure.lino`, using B1 lookups purely as a bridge: a
retrieved gloss is re-run through `structures_for` and the *only* output is a set of
structural meaning ids.

**Architecture sketch.** `formalize_requirement(text) -> Vec<StructuralMeaning>`;
no concepts, no procedures, no entities; `FormalizationSummary` replaced by a
structure-id set.

**Pros.** Small, and directly feeds B2 — the composer already consumes structure
ids. Five-language identity is trivially checkable (`ConceptMap::identity()` already
exists, `src/coding/concept_discovery.rs:142-153`). No new primitive vocabulary.

**Cons.** Answers only the coding half of B4 and abandons the other half: the
memory-contract probe's "zero concepts or procedures" stays zero, because concepts
and procedures are never represented. The nine protocol primitives stay at 2 of 9
forever, and `PRIMITIVE_KINDS` would have to be honestly reduced or deleted. A
non-coding requirement (a policy, a contract, a lab procedure) has no representation
at all. It also makes generality bounded by the 82 meanings — a smaller version of
Option A's failure.

**Doctrine fit.** Generalization: partial. No hard-coding: partial (the 82 meanings
are the ceiling). Honesty: good, if the primitive count is honestly reduced.
Associative stack: neutral. Forget/rediscover: weak (structure ids are seed, not
discovered).
**Effort.** ~5 leaves. **Risk.** Low, with a permanent ceiling.

### Option D — Delegate extraction to the agentic CLI loop and keep Formal AI as the verifier

**Description.** Accept that open-domain extraction is what an external agent CLI is
good at. Have `formal-ai agent` ask the driving CLI for a concept/procedure
proposal, and make Formal AI's job to *verify* it against the captured source bytes:
every proposed concept must be traceable to a span of a capture with a digest, or it
is rejected.

**Architecture sketch.** A new tool capability `Propose`, a strict verifier
`verify_proposal(proposal, captures) -> Result<ConceptGraph, Rejection>`.

**Pros.** Reaches open-domain quality immediately. The verifier is small and is the
part that carries the honesty contract. It fits the existing agentic recipe shape.

**Cons.** Neural inference is a NON-GOAL (`NON-GOALS.md`, restated at
`docs/meta-algorithm.md:198-200`: "no sampling, no hidden state, no neural
inference"). The result would be non-deterministic, so "same prompt, same cache,
same answer" (`docs/case-studies/issue-710/plans/02:238-239`) fails, offline replay
is not byte-identical, and the five-language identity invariant becomes a property
of a model rather than of the algorithm. It also makes the capability unavailable in
the browser and in the library.

**Doctrine fit.** Deterministic: fails. Generalization: it is the model's, not ours.
**Effort.** ~6 leaves. **Risk.** Fails the standing non-goal outright.

## Decision

**Selected: Option B**, with Option C adopted as its *first consumer* rather than as
an alternative: the `ConceptGraph` projects into structural-meaning ids for the
coding path (which is all B2 needs today) while also carrying the concept,
predicate, procedure and entity links the memory-contract probe found missing.

Reasons:

- B4's "fixed means" has three clauses — the formalizer emits an explicit need, B1
  satisfies it, and the result is concepts and procedures the composer can consume.
  Option B is the only option that delivers all three; C delivers the third in a
  reduced form and none of the first two; A and D deliver none.
- The maintainer instruction names the recursion explicitly ("Formalization itself
  may need recursive knowledge collection from trusted sources"). A plan that does
  not make the formalizer a *caller* of retrieval has not addressed the instruction,
  whatever else it improves.
- The repository has already paid twice for the alternative: the fairy-tale lexicon
  is Option A at n=1, and the two probes recorded in
  `docs/case-studies/issue-710/plans/06:247-253` and `:07:361-367` are the
  measurement of its ceiling.
- Option C's ceiling is the same ceiling in a different place. Choosing it would
  guarantee a third probe with the same finding.
- Option D is excluded by a standing non-goal, and its determinism cost would
  invalidate the offline-replay and forget-rediscover proofs this repository's
  honesty rests on.

Rejected, with reasons:

- **Option A** — contradicts "the goal is not to know everything in advance" and
  makes capability non-rediscoverable. A larger lexicon is still a lexicon.
- **Option C** — kept as the coding-path projection, rejected as the whole plan: it
  leaves "zero concepts or procedures" true, and would require honestly deleting
  seven of the nine declared primitives rather than earning them.
- **Option D** — violates the deterministic-projection invariant and the neural-inference
  non-goal; not available in the browser or the library.

## Architecture

### Files added

| Path | Contents |
| --- | --- |
| `src/formalization/mod.rs` | Module root; re-exports the public surface. |
| `src/formalization/segment.rs` | Script-aware sentence and clause segmentation with exact character spans. |
| `src/formalization/needs.rs` | `Need`, `NeedKind`, `NeedState`, `NeedOrigin`, `emit_needs`, the bridge to `meta_frame::NeedLedger`. |
| `src/formalization/concepts.rs` | Sense → concept/predicate/entity links with provenance. |
| `src/formalization/procedures.rs` | Ordered source text → `ExtractedProcedure`. |
| `src/formalization/graph.rs` | `ConceptGraph`, its Links Notation projection, its content identity. |
| `data/seed/formalization-relations.lino` | The relation vocabulary and the cues that evidence each relation, in five languages. |
| `data/seed/meanings-formalization-needs.lino` | Five-language response meanings for need/unresolved reporting. |
| `data/meta/formalization-depth-recipe.lino` | The grounded meta-recipe for this step. |
| `tests/fixtures/issue-1138-b4/` | Committed captures + `expected-graphs.json`. |
| `examples/issue_1138_formalization_parity.rs` | Writes `expected-graphs.json` from the Rust path. |

Names checked free by grep over `src data scripts tests`: `formalization_depth`,
`DeepFormalizer`, `ConceptNeedLedger`, `ExtractedConcept`, `ProcedureExtract`.
`ConceptNeed` is **taken** (`src/coding/concept_discovery.rs:110`) — this plan
*moves* that type to `src/formalization/needs.rs`, renames it to plan 00 §4.1's
`Need`, and leaves `ConceptNeed` behind as a re-export, so there is one definition
rather than two. `Need`, `NeedKind`, `NeedState` and `SourceLookup` are plan-00
contract names, adopted verbatim. `Procedure` is **taken**
(`src/agentic_coding/lexicon.rs:123`), hence `ExtractedProcedure`.
`FormalizationSummary` is taken (`src/agentic_coding/formalize.rs:68`) and is
extended, not duplicated.

### Files changed

| Path | Change |
| --- | --- |
| `src/agentic_coding/formalize.rs` | Delegates segmentation, need emission and concept/procedure extraction to `src/formalization/*`; keeps the nine-primitive KB rendering. Target ≤ 450 lines. |
| `src/agentic_coding/formalization_recipe.rs` | `SEARCH_QUERY`/`CANONICAL_SOURCE_URL` become *fallbacks of last resort* behind a derived query; the tale stays reachable, stops being the default. |
| `src/agentic_coding/lexicon.rs` | Stays as the *known-work* catalogue; its results become one evidence source among several, ranked below retrieved senses for unknown text. |
| `src/coding/concept_discovery.rs` | `ConceptNeed` becomes a re-export of `formalization::needs::Need`; `status: String` becomes `NeedState`. |
| `src/coding_research_learning.rs` | Accepts `ExtractedProcedure` as a second typed input shape, under the same execution + review gate. |
| `src/how_to_guide.rs` | `GuideStep` gains `to_extracted_procedure()`, so a synthesised guide is formalizable. |
| `src/meta_frame.rs` | `NeedLedger::extend_from_formalization(&ConceptGraph)`. |
| `data/agentic-coding/fisherman-lexicon.lino` | Unchanged; it becomes a regression corpus, not the semantic model. |

### Rust signatures

```rust
// src/formalization/segment.rs
/// One segmented unit of source text with exact character offsets into the
/// original string.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Segment {
    pub text: String,
    pub start: usize,
    pub end: usize,
    pub script: Script,
}

/// The writing system a segment is in, decided from its characters, never from a
/// language flag supplied by a caller.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Script { Latin, Cyrillic, Devanagari, Han, Other }

/// Segment `text` into sentences at the terminators the seed declares for each
/// script (`.` `!` `?` `。` `！` `？` `।` `॥` `¿` `¡` pairs), keeping exact spans.
#[must_use]
pub fn sentences(text: &str) -> Vec<Segment>;

/// Segment one sentence into clauses at the seeded clause separators, so a
/// requirement carrying several obligations emits several needs.
#[must_use]
pub fn clauses(sentence: &Segment) -> Vec<Segment>;
```

> **reconciled: was `Need`, `NeedKind` and `NeedState` given their single Rust
> definition in `src/formalization/needs.rs` by this plan and `NeedKind` given a
> second one in `src/seed/sources.rs` by plan 01; now all three live in
> `src/needs.rs`, landed by plan 00's contract leaf C1 before plan 01 L3, and
> this module owns only what sits *beside* the record — `NeedOrigin`, the
> retrieved senses, `emit_needs` and `satisfy_needs` (plan 00 §9 R1).**

```rust
// src/formalization/needs.rs
//
// `Need`, `NeedKind` and `NeedState` are plan 00 §4.1's contract, defined once
// in `src/needs.rs` and re-exported here. `data/seed/…` projects the same
// record as links:
//
//   need <id>
//     kind      concept | procedure | part | prerequisite | evidence | decision
//     subject   "<surface text as written by the user or the failing tool>"
//     language  en | ru | hi | zh | es | …
//     raised_by <obligation id | need id | tool result id>
//     state     open | planned | satisfied | unsatisfiable
//     satisfied_by <evidence id>

pub use crate::needs::{Need, NeedKind, NeedState};   // plan 00 leaf C1

// `NeedState` is plan 00 §4.1's `state` and the single need-status vocabulary in
// the tree: `meta_frame::NeedStatus` (`src/meta_frame.rs:44-61`) maps onto it as
// Pending→Open, Planned→Planned, Satisfied→Satisfied, Blocked→Unsatisfiable;
// `Deferred` and `Rejected` have no producer today and are removed with their
// own test (this plan's L3).

/// A finer reason *inside* a `NeedKind::Concept` or `NeedKind::Procedure` need,
/// so a reader can tell an unknown word from an unknown relation from an
/// exhausted recursion. Carried beside the ledger row, never inside it, so
/// `recipe_interpreter`'s event-for-event parity obligation (R343) is untouched.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NeedOrigin {
    /// A surface no seeded meaning, memory link or retrieved sense accounts for.
    UnresolvedSurface,
    /// A clause whose relation between two grounded terms is not recognised.
    UnresolvedRelation,
    /// An imperative clause with no ordered procedure behind it.
    UnresolvedProcedure,
    /// A retrieved gloss whose own surfaces are unresolved at depth+1.
    RecursiveGloss,
}

/// What this plan carries *beside* a `Need`, never inside it, so
/// `recipe_interpreter`'s event-for-event parity obligation (R343) is untouched.
/// `src/coding/concept_discovery.rs::ConceptNeed` re-exports `needs::Need`;
/// there is one definition in the tree, and plan 01's
/// `SourceLookup::lookup(&Need, &LookupBounds)` takes it directly.
///
/// **reconciled: was a second `Need` struct with `origin` and `evidence` fields
/// inline; now the contract record plus this sidecar, because adding a field to
/// a recorded ledger row changes the event stream R343 pins (plan 00 §9 R1,
/// and this plan's own risk 8).**
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NeedContext {
    pub need_id: String,                // joins to `Need::need_id`
    pub origin: NeedOrigin,
    pub evidence: Vec<ConceptSense>,    // from plan 01
}

/// Every unresolved surface, relation and procedure in `segments`, at `depth`.
#[must_use]
pub fn emit_needs(doc_id: &str, segments: &[Segment], graph: &ConceptGraph, depth: usize)
    -> Vec<Need>;

/// Ask plan 01's `SourceLookup` for every `Open` need, bounded by `bounds` and
/// `max_concept_depth`; each satisfied need's gloss is itself segmented and may
/// emit needs at `depth + 1`. Returns the needs with updated states.
pub fn satisfy_needs<L: SourceLookup>(
    needs: Vec<Need>,
    lookup: &mut L,
    bounds: &LookupBounds,
    max_concept_depth: usize,
) -> Vec<Need>;
```

```rust
// src/formalization/concepts.rs
/// A concept the formalizer grounded, and the exact bytes that ground it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtractedConcept {
    pub id: String,              // "concept:<slug of lemma>"
    pub label: String,
    pub language: String,
    pub gloss: String,
    pub genus: Option<String>,   // the "is a" term the gloss names, when it names one
    pub differentiae: Vec<String>,
    pub structures: Vec<String>, // seeded structural meaning ids the gloss evidences
    pub source_id: String,
    pub source_url: String,
    pub sha256: String,
    pub license_name: String,
    pub depth: usize,
}

/// Ground one retrieved sense as a concept: split the gloss into genus and
/// differentiae using the seeded relation cues, then re-run the differentiae
/// through the structural lexicon so the concept reaches seeded idioms.
#[must_use]
pub fn concept_from_sense(sense: &ConceptSense) -> Option<ExtractedConcept>;

/// Relations evidenced between two grounded terms in one clause, from the cues
/// declared in `data/seed/formalization-relations.lino`. Never invents a relation
/// the cues do not evidence.
#[must_use]
pub fn relations_in(clause: &Segment, graph: &ConceptGraph) -> Vec<ExtractedRelation>;
```

```rust
// src/formalization/procedures.rs
/// An ordered, provenance-bearing procedure extracted from a trusted source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtractedProcedure {
    pub id: String,
    pub goal: String,
    pub language: String,
    pub steps: Vec<ProcedureStep>,
    pub preconditions: Vec<String>,
    pub postconditions: Vec<String>,
    pub source_id: String,
    pub source_url: String,
    pub sha256: String,
    pub license_name: String,
    pub license_url: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcedureStep {
    pub position: usize,
    pub imperative: String,
    pub object: Option<String>,
    pub source_span: String,
    pub verified: bool,          // set only by an execution record, never by extraction
}

/// Extract an ordered procedure from any captured ordered text: a how-to guide's
/// steps, a documentation page's numbered list, an answer's ordered block.
///
/// **reconciled: was `procedure_from_steps(goal, &[GuideStep], language)`, while
/// plan 02 declared its own `ProcedureStepRecord` in
/// `src/coding/procedure_text.rs`; now this plan's L9 declares
/// `src/procedure_text.rs::ProcedureStepRecord` (one "ordered step with
/// provenance" record for the whole tree) and plan 02 L10 extends that module
/// with `StepShape`, `steps_from_capture` and `retrieve_procedure`. This plan
/// lands first in plan 00 §5's order, so it owns the record (plan 00 §9 R10).**
#[must_use]
pub fn procedure_from_steps(
    goal: &str,
    steps: &[crate::procedure_text::ProcedureStepRecord],
    language: &str,
) -> Option<ExtractedProcedure>;

impl ExtractedProcedure {
    /// The only constructor. `GuideStep::to_step_record()` in
    /// `src/how_to_guide.rs` is how a synthesised guide reaches it.
    #[must_use]
    pub fn from_step_records(
        goal: &str,
        steps: &[crate::procedure_text::ProcedureStepRecord],
        language: &str,
    ) -> Option<Self>;
}

impl ExtractedProcedure {
    /// Render into the versioned shape `coding_research_learning` already gates,
    /// so an extracted procedure enters the ledger through the existing execution
    /// and review boundary rather than beside it.
    #[must_use]
    pub fn to_coding_procedure_source(&self) -> String;
    #[must_use]
    pub fn to_links_notation(&self) -> String;
}
```

```rust
// src/formalization/graph.rs
/// Everything one formalization grounded, plus everything it could not.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ConceptGraph {
    pub doc_id: String,
    pub concepts: Vec<ExtractedConcept>,
    pub relations: Vec<ExtractedRelation>,
    pub procedures: Vec<ExtractedProcedure>,
    pub entities: Vec<ExtractedEntity>,
    pub needs: Vec<Need>,
    pub segments: Vec<Segment>,
}

impl ConceptGraph {
    /// The five-language invariant: language-independent identity over grounded
    /// concept ids, relation kinds and structural meaning ids — never over surface
    /// text, never over the source language.
    #[must_use] pub fn identity(&self) -> String;
    #[must_use] pub fn to_links_notation(&self) -> String;
    /// The projection the B2 composer consumes.
    #[must_use] pub fn structure_ids(&self) -> Vec<String>;
    #[must_use] pub fn unresolved(&self) -> Vec<&Need>;
    #[must_use] pub fn grounded_ratio(&self) -> (usize, usize); // (grounded, total needs)
}

/// The entry point. Deterministic for a given text, lookup and bounds. It takes
/// plan 01's `SourceLookup` rather than a transport, so the formalizer owns no
/// cache policy, no settings reading and no source list of its own; an offline
/// caller passes an offline lookup and every unmet need becomes
/// `NeedState::Unsatisfiable` with its consulted-source rows intact.
pub fn formalize_deeply<L: SourceLookup>(
    text: &str,
    doc_id: &str,
    lookup: &mut L,
    bounds: &LookupBounds,
    max_concept_depth: usize,
) -> ConceptGraph;
```

```rust
// src/agentic_coding/formalize.rs — the summary stops being blind.
pub struct FormalizationSummary {
    /* … the nine existing counts … */
    /// Needs the formalizer raised, and how many it grounded. A document with
    /// unresolved needs can never be reported as fully covered.
    pub needs_raised: usize,
    pub needs_grounded: usize,
    pub max_depth_reached: usize,
}
```

### Seed schema changes, shown as records

`data/seed/formalization-relations.lino` — the relation vocabulary and the cues that
*evidence* each relation, one lexeme block per language, in the shape
`data/seed/meanings-coding-structure.lino:148-215` already uses:

```lino
relations
  is_a
    kind taxonomic
    inverse subsumes
    grounding "https://www.w3.org/TR/rdf-schema/#ch_subclassof"
    lexeme en
      surface
        text "is a"
      surface
        text "is any"
      surface
        text "denotes a"
    lexeme ru
      surface
        text "это"
      surface
        text "является"
    lexeme hi
      surface
        text "एक प्रकार है"
    lexeme zh
      surface
        text 是一种
    lexeme es
      surface
        text "es un"
      surface
        text "es una"
  has_property
    kind attributive
    inverse property_of
    grounding "https://www.w3.org/TR/rdf-schema/#ch_property"
    lexeme en
      surface
        text "that has"
      surface
        text "with no"
      surface
        text containing
    ...
  requires
    kind precondition
    inverse required_by
    grounding "https://plato.stanford.edu/entries/necessary-sufficient/"
    ...
  precedes
    kind ordering
    inverse follows
    grounding "https://docs.python.org/3.12/library/functions.html#sorted"
    ...
```

Initial vocabulary: `is_a`, `has_property`, `part_of`, `requires`, `produces`,
`precedes`, `excludes`, `measured_in`. Eight relations × five languages. **This is
the one piece of seeded vocabulary this plan adds, and it is deliberately about
*relations*, not about *topics*: it says how a gloss is read, never what any
particular word means.** The no-memorization gate must be able to distinguish the
two, so the gate is extended to forbid any domain noun in this file.

`data/seed/meanings-formalization-needs.lino` — five-language reporting for needs,
extending the honesty pattern of `data/seed/meanings-formalization-report.lino:1-9`:

```lino
meanings
  formalization_unresolved_need
    defined-by output
    role output_display_request
    precondition "A formalization raised a need that no consulted source grounded."
    effect "Name the surface, its exact source span, its origin, the depth reached and every consulted source outcome; never report a document as covered while a need is unresolved."
    unit "count"
    example "3 of 7 needs grounded; \"isogram\" unresolved at depth 2."
    grounding "src/formalization/graph.rs::ConceptGraph::unresolved"
  response_formalization_unresolved_need_en
    defined-by formalization_unresolved_need
    language en
    ...
```

Intents: `formalization_unresolved_need`, `formalization_grounded_concept`,
`formalization_extracted_procedure`, `formalization_depth_exhausted`,
`formalization_offline_need` — five intents × five languages = 25 responses plus
five meanings.

`data/meta/formalization-depth-recipe.lino`, in the shape of
`data/meta/coding-discovery-recipe.lino:1-12`:

```lino
formalization_depth_meta_recipe
  record_type meta_recipe
  topic deep_formalization
  issue 1138
  segmenter src/formalization/segment.rs
  needs src/formalization/needs.rs
  lookup src/concept_lookup.rs
  concepts src/formalization/concepts.rs
  procedures src/formalization/procedures.rs
  graph src/formalization/graph.rs
  consumer src/agentic_coding/formalize.rs
  consumer src/coding/concept_discovery.rs
  consumer src/coding_research_learning.rs
formalization_depth_step_segment
  record_type meta_step
  order 1
  function sentences
formalization_depth_step_emit_needs
  record_type meta_step
  order 2
  function emit_needs
formalization_depth_step_satisfy
  record_type meta_step
  order 3
  function satisfy_needs
formalization_depth_step_ground_concepts
  record_type meta_step
  order 4
  function concept_from_sense
formalization_depth_step_extract_procedures
  record_type meta_step
  order 5
  function procedure_from_steps
formalization_depth_step_project
  record_type meta_step
  order 6
  function to_links_notation
```

### Cache layout and content addressing

No new cache. Every byte still arrives through `CachedSourceClient::fetch`
(`src/source_fetch.rs:201-230`) into
`<cache_dir>/source-cache/objects/<sha256>.body`, and every `ExtractedConcept` and
`ExtractedProcedure` carries the `sha256` of the capture it was read from. The
derived layer is one `concept-graphs.lino` in `FORMAL_AI_CACHE_DIR`, keyed by
`stable_id(doc_id ‖ ConceptGraph::identity())`, modelled on
`src/coding/discovered_procedures.rs:133-197`, storing the locator and identity and
never the source payload — the retention rule from
`docs/case-studies/issue-710/plans/06:232-239`. Plan 01's `concept-senses.lino`
remains the sense-level ledger; a graph references sense content ids, so deleting
either file loses nothing that the same captures cannot rebuild.

### Offline replay

`formalize_deeply` takes the client as a parameter and never constructs one, so an
offline caller passes an offline client and every unmet need becomes
`NeedState::Unsatisfiable` with a per-source `offline_cache_miss` outcome. The committed
`tests/fixtures/issue-1138-b4/` captures make the five held-out requirements replay
byte-identically: the test asserts every `sha256` in every `ExtractedConcept` matches
`capture-manifest.lino` and that the fixture transport's `get` was never called.
`ConceptGraph::identity()` is asserted equal between a live run and an offline replay
of the same captures.

### Settings keys and opt-outs

None added. Deep formalization consults sources only through
`concept_lookup::lookup_surface`, which honours `ServicePreferences` exactly as plan
01 specifies, so a user who has opted a dictionary out sees needs reported
`disabled` for that source rather than silently unresolved. One new **config** knob,
not a settings toggle: `max_concept_depth`, defaulting to 2, sharing
`SolverConfig::max_decomposition_depth`'s semantics — a bound, not a budget: reaching
it produces a `NeedState::Unsatisfiable` row with `origin: RecursiveGloss`, never a
truncated answer presented as complete.

### Licensing

`ExtractedConcept::gloss` is quoted text from a licensed source and carries
`source_url` + `license_name`; it is rendered with attribution and is never inlined
into generated code — the same rule plan 01 sets for `ConceptSense`, enforced by the
same test. `ExtractedProcedure` steps extracted from wikiHow are CC BY-NC-SA 3.0:
**non-commercial**, so a procedure with that license may be *shown* with attribution
but may not be promoted into `data/seed/` and may not be relicensed into generated
output. `to_coding_procedure_source()` therefore emits the SPDX expression of the
source verbatim and `coding_research_learning`'s existing review gate refuses a
procedure whose license forbids the intended use — a refusal that is tested, not
assumed.

### Browser / WASM worker parity

`src/formalization/*` is pure computation over captured bytes plus plan 01's walk, so
the browser mirror is `src/web/worker/formal_ai_worker_formalization.js` over plan
01's shared `formal_ai_worker_source_walk.js`. Both runtimes are held to one
recorded expectation, `tests/fixtures/issue-1138-b4/expected-graphs.json`, written
from the Rust path by `examples/issue_1138_formalization_parity.rs` and asserted by
`tests/unit/issue_1138_formalization_depth.rs` and
`tests/web/issue-1138-formalization-depth.test.mjs`. One new
`data/meta/worker-line-budget/` file. The browser has no Python runtime, so a
`ProcedureStep::verified` flag is always `false` there and the guide is marked
unverified — the plan-02 §7 boundary, respected.

### Failure and honesty behaviour — what the user sees when a need is not grounded

`FormalizationSummary` gains `needs_raised` / `needs_grounded` / `max_depth_reached`,
and the report seeded at `data/seed/meanings-formalization-report.lino:6` —
"Report the observed covered count … do not infer coverage from the scheme
declaration" — is extended by the same principle: **a document with an unresolved
need is never reported as covered**, regardless of how many primitive kinds happen
to be non-empty. The user sees, in the prompt's language:

- the covered count over nine, as today (`2 of 9` stays available and stays honest);
- **and** `needs_grounded of needs_raised`, with each unresolved need named by its
  surface and exact source span;
- for each unresolved need, its `origin` (unknown word / unknown relation / unknown
  procedure / depth exhausted) and every consulted source with its observed outcome.

There is no floor. A run that grounds nothing reports `0 of 7 needs grounded` and
lists seven surfaces. The generic `pred:states` assertion is *kept* — preserving the
sentence is real and useful — but it is relabelled in the projection as
`preserved_span`, so the coverage number can no longer be satisfied by transcription.
That relabelling changes an existing recorded number and must be landed with its own
failing test and an amended ledger row rather than quietly.

### How the universal loop, the coding path and the agentic recipe share one implementation

`formalize_deeply` has one definition and three callers:

- **Agentic recipe** — `plan_formalization_step`
  (`src/agentic_coding/formalization_recipe.rs:60-116`) calls it on the bound source
  text; `final_answer` (`:121-140`) renders the nine-primitive KB *plus* the need
  report. `SEARCH_QUERY`/`CANONICAL_SOURCE_URL` stop being step 1: the search query
  is derived from the task's own unresolved surfaces via
  `ConceptGraph::unresolved()`, and the canonical tale URL remains only as the
  regression fixture's source.
- **Coding path** — `src/coding/concept_discovery.rs::discover_with_lookup` consumes
  `ConceptGraph::structure_ids()` in addition to `structures_for`, and
  `ConceptGraph::needs` *is* the `Need` list it already builds — one type, one
  ledger.
- **Universal loop** — `src/meta_frame.rs` gains
  `NeedLedger::extend_from_formalization(&ConceptGraph)`, so a need raised while
  formalizing appears in the same ledger the loop already reports (R333), with
  `NeedState` rather than a second string vocabulary.

One graph type, one need type, one status enum, one ledger, three callers.

## Tests first

Every test below is written and observed failing before its implementation leaf.

### Held-out five-language cases (new corpus)

`data/benchmarks/formalization-depth-requirements.lino`, same record shape as
`data/benchmarks/coding-discovery-paraphrases.lino:1-6`. Two families. Held-out
vocabulary verified absent from `src`, `data`, `docs`, `tests` by grep before the
corpus is written, and covered by the extended `held_out_sentences_are_not_seed_lexemes`
gate (`tests/unit/coding_discovery/multilingual.rs:70-88`).

**Family `isogram_requirement`** — a requirement whose key concept must be
retrieved:

- en — `Formalize this requirement: an isogram check must reject any word that repeats a letter.`
- ru — `Формализуй требование: проверка на изограмму должна отклонять слово с повторяющейся буквой.`
- hi — `इस आवश्यकता को औपचारिक करें: आइसोग्राम जाँच किसी भी दोहराए गए अक्षर वाले शब्द को अस्वीकार करे।`
- zh — `请把这个需求形式化:isogram 检查必须拒绝任何含有重复字母的单词。`
- es — `Formaliza este requisito: una comprobación de isograma debe rechazar cualquier palabra que repita una letra.`

**Family `lipogram_procedure`** — a requirement that names a *procedure*, so the
procedure extractor is exercised:

- en — `Formalize this requirement: to check a lipogram, read the text, drop spacing and punctuation, then confirm the forbidden letter never appears.`
- ru — `Формализуй требование: чтобы проверить липограмму, прочитай текст, убери пробелы и знаки препинания, затем убедись, что запрещённая буква не встречается.`
- hi — `इस आवश्यकता को औपचारिक करें: लिपोग्राम जाँचने के लिए पाठ पढ़ें, रिक्ति और विराम चिह्न हटाएँ, फिर पुष्टि करें कि वर्जित अक्षर कभी नहीं आता।`
- zh — `请把这个需求形式化:要检查 lipogram,先读取文本,去掉空格与标点,然后确认被禁止的字母从不出现。`
- es — `Formaliza este requisito: para comprobar un lipograma, lee el texto, elimina espacios y puntuación, y confirma que la letra prohibida nunca aparece.`

No prompt explains what `isogram` or `lipogram` means. The concept must be retrieved
or the case honestly fails.

### Unit tests

`tests/unit/issue_1138_formalization_depth.rs`:

- `an_unfamiliar_requirement_raises_a_need_for_every_unresolved_surface`
- `a_need_is_satisfied_by_the_registry_lookup_and_becomes_a_grounded_concept`
- `a_grounded_gloss_raises_its_own_needs_at_the_next_depth_and_stops_at_the_bound`
- `an_ungrounded_need_is_reported_with_its_origin_span_and_consulted_sources`
- `a_document_with_an_unresolved_need_is_never_reported_as_covered`
- `preserved_sentences_no_longer_satisfy_the_assertion_primitive`
- `the_same_requirement_in_five_languages_produces_one_concept_graph_identity`
- `an_imperative_clause_sequence_becomes_an_ordered_extracted_procedure`
- `an_extracted_procedure_enters_the_ledger_only_through_execution_and_review`
- `a_non_commercial_licensed_procedure_is_shown_but_refused_for_promotion`
- `an_offline_run_replays_the_committed_captures_and_reproduces_the_graph_identity`
- `the_native_and_browser_runtimes_produce_the_same_graph`

`tests/unit/issue_1138_segmentation.rs`:

- `chinese_text_segments_at_the_ideographic_full_stop`
- `hindi_text_segments_at_the_danda`
- `spanish_inverted_punctuation_does_not_split_a_sentence`
- `every_segment_span_selects_exactly_its_own_text`
  — the exact-span defect recorded at
  `docs/case-studies/issue-710/plans/07:371-377` ("its character span includes the
  preceding separator whitespace (`42:125` instead of the exact text start at 43)"),
  fixed here with its own red test rather than by rewriting the historical artifact.

`tests/unit/agentic_coding.rs` (extended):

- `a_custom_task_is_formalized_instead_of_the_seeded_fairy_tale`
  — the R314 audit finding (`docs/requirements-traceability.md:376`), turned into a
  regression.
- `the_canonical_tale_still_formalizes_to_nine_primitives`
  — the existing behaviour must not regress; the fairy-tale lexicon becomes a
  regression corpus.

`tests/unit/coding_discovery/concepts.rs` (extended):

- `the_coding_path_and_the_formalizer_share_one_need_type_and_one_status_enum`

### Integration tests

`tests/integration/issue_1138_formalization_agent.rs` — a real `formal-ai agent`
process, mirroring the probes in
`docs/case-studies/issue-710/plans/06:307-309` and `07:361-367`:

- `an_agent_run_over_an_unfamiliar_requirement_reports_grounded_concepts_not_only_spans`
- `an_agent_run_reports_its_ungrounded_needs_rather_than_claiming_coverage`

### Specification tests

`tests/unit/specification/formalization_depth_meta_algorithm.rs` — grounds
`data/meta/formalization-depth-recipe.lino` against the live source: every `function`
exists as `fn <name>` in the named file, `order` is contiguous 1..6, every `consumer`
path exists — the `coding_discovery_meta_algorithm.rs` pattern.

`tests/unit/specification/agentic_meta_algorithm.rs` (amended) — the `meta_constant`
count drops from 3 as `SEARCH_QUERY`/`CANONICAL_SOURCE_URL` stop being the plan, and
step 1's "small closed keyword set" wording is replaced; the test must be amended in
the same commit as the recipe and `docs/meta-algorithm.md`.

`tests/unit/docs_requirements/issue_1138.rs` — the requirement shard's rows exist and
each names a test that exists.

### Web tests

`tests/web/issue-1138-formalization-depth.test.mjs` — the browser worker produces the
same `ConceptGraph::identity()` from the same fixtures and marks every
`ProcedureStep::verified` false.

### Gates and ratchets

- **No-memorization gate.** `tests/unit/coding_discovery/no_memorization.rs:11`
  extended to forbid the held-out vocabulary (`isogram`, `изограмма`, `आइसोग्राम`,
  `lipogram`, `липограмма`, `lipograma`) in `src/**` and `data/seed/**`, and
  additionally to forbid any *domain noun* in
  `data/seed/formalization-relations.lino` — the relation seed may declare how a
  gloss is read, never what a word means.
- **Coverage-honesty gate.** No code path may compute `covered` without also
  computing `needs_raised`/`needs_grounded`; pinned by
  `a_document_with_an_unresolved_need_is_never_reported_as_covered`.
- **Gloss-not-code gate** (shared with plan 01): no substring of any
  `ExtractedConcept::gloss` may appear in a `VerifiedDraft::source`.
- **Handler ratchet (#959).** Zero `try_*` arms added; the
  `data/meta/handler-migration-ledger.lino` count must not rise.
- **File-size gate.** `src/agentic_coding/formalize.rs` drops from 601 to ≤ 450
  lines; every new module stays under 900 (`scripts/check-file-size.rs:21-27`).
- **Need-vocabulary ratchet.** The count of distinct need-status vocabularies in the
  tree must fall from 2 to 1; pinned by
  `the_coding_path_and_the_formalizer_share_one_need_type_and_one_status_enum`.
- **Capture-drift check.** `FORMAL_AI_LIVE_FETCH=1 cargo run --example
  issue_1138_formalization_parity` re-fetches through the production path and
  reports drift against `tests/fixtures/issue-1138-b4/capture-manifest.lino`.

### Benchmark commands and the honest numbers expected

```sh
# The five-language deep-formalization corpus (the number this plan is judged by)
cargo test --test unit issue_1138_formalization_depth
cargo test --test unit issue_1138_segmentation

# Agent-process probes, replacing the two hand-run probes of plans 06 and 07
cargo test --test integration issue_1138_formalization_agent

# Grounded recipes
cargo test --test unit specification::formalization_depth_meta_algorithm
cargo test --test unit specification::agentic_meta_algorithm

# Live refresh / drift (never in the default suite)
FORMAL_AI_LIVE_FETCH=1 cargo run --example issue_1138_formalization_parity

# Unchanged suites, re-run to prove no regression
cargo run --bin formal-ai -- benchmark run --suite humaneval --slice 20 --append
cargo run --bin formal-ai -- benchmark run --suite mbpp --slice 20 --online --append
cargo run --bin formal-ai -- benchmark ratchet
```

Honest expectations, recorded whatever they are:

- **Concepts extracted from an unfamiliar requirement:** today **0** in all five
  languages (`docs/case-studies/issue-710/plans/06:249-250`, `07:366`). Exit
  condition: ≥ 1 grounded concept per requirement, online, in all five languages; and
  a graph identity equal across all five. If Wiktionary serves only `en`, the honest
  intermediate is `1/5 grounded, 4/5 unbound_template` — recorded as such and
  answered by routing ru/hi/zh/es to Wikipedia's per-language endpoint, never by
  copying the English gloss.
- **Procedures extracted:** today **0**. Exit condition ≥ 1 for the
  `lipogram_procedure` family, with `verified: false` until an execution record sets
  it.
- **Protocol primitives on an unfamiliar document:** today **2 of 9**. Expected after
  this plan: **5 of 9** for the `isogram_requirement` family (`concept`, `predicate`,
  `assertion`, `annotation`, plus `entity` where a term is named) and **6 of 9** for
  `lipogram_procedure` (adding `procedure`). `temporal`, `modal` and `context` are
  *not* claimed: the requirement corpus contains no temporal or modal expression, and
  claiming them would be exactly the "incorrectly says all nine primitives are
  realized" defect of `docs/case-studies/issue-710/plans/06:251-252`.
- **Sentence segmentation on hi/zh:** today 1 segment for any length of text.
  Expected: the true sentence count, asserted against the corpus.
- **HumanEval / MBPP first 20:** expected unchanged at 20/20 and 20/20. Any fall is a
  regression the ratchet fails on.
- **GSM8K / MATH / object counting / CoEdIT:** unchanged at 2/20, 0/20, 0/20, 0/20.
  This plan gives those suites a formalizer that can ask; B8 is the plan that routes
  them through it.
- **`needs_raised` with `needs_grounded == 0` on the canonical tale:** must stay 0
  raised — the known work is still fully grounded from its lexicon, and a regression
  there means the closed catalogue was broken while generalising.

## Implementation leaves

Ordered; each individually verifiable and commit-sized.

- [ ] **L1 — Segmentation, red first.** `tests/unit/issue_1138_segmentation.rs` with
      four failing tests (zh `。`, hi `।`, es inverted punctuation, exact spans).
- [ ] **L2 — `src/formalization/segment.rs`.** Script-aware `sentences`/`clauses`
      with exact spans; terminators declared in seed, not literals in Rust.
      `src/agentic_coding/formalize.rs:471-498` deleted in favour of it. L1 green.
- [ ] **L3 — One need type.** Move `ConceptNeed` to `src/formalization/needs.rs` as plan 00 §4.1's `Need`,
      replace `status: String` with `NeedState`, map `meta_frame::NeedStatus` onto it, add `NeedOrigin` and
      `source_span`, re-export from `src/coding/concept_discovery.rs`. Test:
      `the_coding_path_and_the_formalizer_share_one_need_type_and_one_status_enum`.
- [ ] **L4 — Need emission.** `emit_needs`; `FormalizationSummary` gains
      `needs_raised`/`needs_grounded`/`max_depth_reached`. Tests:
      `an_unfamiliar_requirement_raises_a_need_for_every_unresolved_surface`,
      `a_document_with_an_unresolved_need_is_never_reported_as_covered`.
- [ ] **L5 — Relation vocabulary.** `data/seed/formalization-relations.lino` (8
      relations × 5 languages) and the extended no-memorization gate forbidding
      domain nouns in it.
- [ ] **L6 — Concept grounding.** `src/formalization/concepts.rs`:
      `concept_from_sense` (genus/differentiae from the seeded cues) and
      `relations_in`. Depends on plan 01 L7. Tests:
      `a_need_is_satisfied_by_the_registry_lookup_and_becomes_a_grounded_concept`,
      plus the gloss-not-code gate.
- [ ] **L7 — Bounded recursion.** `satisfy_needs` with `max_concept_depth`; a gloss's
      own surfaces become needs at depth+1. Test:
      `a_grounded_gloss_raises_its_own_needs_at_the_next_depth_and_stops_at_the_bound`.
- [ ] **L8 — The graph.** `src/formalization/graph.rs`: `ConceptGraph`,
      `identity()`, `to_links_notation()`, `structure_ids()`, `unresolved()`,
      `grounded_ratio()`, `formalize_deeply`. Test:
      `the_same_requirement_in_five_languages_produces_one_concept_graph_identity`.
- [ ] **L9 — Procedure extraction.** `src/procedure_text.rs` with
      `ProcedureStepRecord` (the one "ordered step with provenance" record; plan
      02 L10 extends this module with `StepShape`, `steps_from_capture` and
      `retrieve_procedure`); `src/formalization/procedures.rs` with
      `ExtractedProcedure::from_step_records`; `GuideStep::to_step_record()` in
      `src/how_to_guide.rs`. Test:
      `an_imperative_clause_sequence_becomes_an_ordered_extracted_procedure`
      and `a_guide_step_and_a_captured_step_produce_the_same_record_shape`.
- [ ] **L10 — A typed route into the #919 ledger.**
      `ExtractedProcedure::to_coding_procedure_source()`;
      `src/coding_research_learning.rs` accepts it under the existing execution +
      review gate. Tests:
      `an_extracted_procedure_enters_the_ledger_only_through_execution_and_review`,
      `a_non_commercial_licensed_procedure_is_shown_but_refused_for_promotion`.
- [ ] **L11 — Rewire the agentic formalizer.** `src/agentic_coding/formalize.rs`
      delegates to `src/formalization/*`; `pred:states` output relabelled
      `preserved_span`; the tale keeps its nine primitives. Tests:
      `preserved_sentences_no_longer_satisfy_the_assertion_primitive`,
      `the_canonical_tale_still_formalizes_to_nine_primitives`.
- [ ] **L12 — Unpin the recipe.** `SEARCH_QUERY`/`CANONICAL_SOURCE_URL` demoted to
      last-resort fallbacks; the query derives from `ConceptGraph::unresolved()`.
      **This leaf solely owns the `docs/meta-algorithm.md:214-218` rewrite that
      plans 02 and 08 also proposed; they now cite plan 11 row D181 instead
      (plan 00 §9 X9).**
      Test: `a_custom_task_is_formalized_instead_of_the_seeded_fairy_tale`. Amend
      `tests/unit/specification/agentic_meta_algorithm.rs` and
      `docs/meta-algorithm.md` in the same commit.
- [ ] **L13 — Loop ledger bridge.** `NeedLedger::extend_from_formalization`.
- [ ] **L14 — Fixtures and parity.** `tests/fixtures/issue-1138-b4/`,
      `examples/issue_1138_formalization_parity.rs`,
      `src/web/worker/formal_ai_worker_formalization.js`, its worker-line-budget
      file, `tests/web/issue-1138-formalization-depth.test.mjs`.
- [ ] **L15 — Five-language reporting prose.**
      `data/seed/meanings-formalization-needs.lino` (5 meanings + 25 responses);
      the language-coverage gate must report `OK … en, ru, hi, zh, es`.
- [ ] **L16 — Agent-process probes.**
      `tests/integration/issue_1138_formalization_agent.rs`, replacing the two
      hand-run probes of plans 06 and 07 with always-run coverage.
- [ ] **L17 — Grounded recipe.** `data/meta/formalization-depth-recipe.lino` and
      `tests/unit/specification/formalization_depth_meta_algorithm.rs`.
- [ ] **L18 — Ledgers and docs.** Requirement shard
      `docs/requirements/issue-1138-formalization-depth.md`,
      `rust-script scripts/assemble-requirements.rs --write`, traceability rows and
      the R314 correction, `docs/benchmarks.md`, `docs/meta-algorithm.md`,
      `VISION.md`, `ROADMAP.md`, and the honest measured numbers.

## Docs to update

The exact quoted statements and their replacement text moved to plan 11's
findings table on 2026-09-16, so there is one docs authority and no document
is described in two places (plan 00 §8). This plan's entries are rows
**D181-D275** of
[`11-docs-consistency-audit.md`](11-docs-consistency-audit.md) §"Issue #1138
plan doc replacements", and plan 11's leaves apply them after the ledger rows
they cite exist (plan 00 §7).

| row | document |
| --- | --- |
| D181 | `docs/meta-algorithm.md` |
| D182 | `VISION.md` |
| D183 | `ROADMAP.md` |
| D184 | `docs/requirements/issue-1138-formalization-depth.md` |
| D274 | `docs/requirements-traceability.md` |
| D275 | `docs/benchmarks.md` |

Any further document this plan's implementation touches is added as a new
plan 11 row, never as a second copy here.

## Risks and open questions

1. **Hard dependency on plan 01.** Leaves L6 onward cannot start before plan 01's L7.
   If plan 01's five-language coverage lands as "en only, four honest refusals", this
   plan's headline invariant — one graph identity across five languages — is
   unreachable for the `isogram` family and the honest result is four `blocked` needs.
   The plan must record that as the measured number rather than weakening the
   invariant to something four refusals can pass.
2. **Genus/differentiae extraction from a gloss is itself an extraction problem.**
   `"a word in which no letter is repeated"` parses cleanly against the `is_a` and
   `has_property` cues; many real glosses will not. The honest failure mode is a
   grounded *sense* with no grounded *concept* — a need that moves from
   `UnresolvedSurface` to `UnresolvedRelation` rather than to `Satisfied`. That is
   progress and must be reported as such, not rounded up.
3. **Relabelling `pred:states` output changes a committed number.** Coverage on the
   canonical tale is asserted at 9 of 9 today
   (`docs/case-studies/issue-710/plans/06:265-267`). L11 must show that the tale's
   nine come from the lexicon and not from the fallback, or the relabelling has broken
   something real. If the tale's assertion count depends on the fallback, the honest
   response is to record a reduced number for the tale, not to keep the fallback
   counting.
4. **`ConceptGraph::identity()` may be too strict or too loose.** Too strict: two
   languages whose dictionaries gloss the same word slightly differently produce
   different genus terms and therefore different ids. Too loose: identity over
   structural meaning ids alone would make two different concepts identical. The plan
   proposes identity over grounded concept ids + relation kinds + structure ids; the
   first measurement will say whether that is right, and the test must be allowed to
   fail honestly before the definition is tuned.
5. **Non-commercial licenses in the procedure path.** wikiHow is CC BY-NC-SA 3.0. A
   procedure extracted from it can be shown but must never be promoted into seed data
   or generated output. The refusal is tested, but the *classification* — which uses
   count as commercial — is a judgement this plan encodes once, in
   `to_coding_procedure_source()`, and which deserves legal review before the first
   promotion attempt.
6. **Depth bound versus the "no budgets" doctrine.** `max_concept_depth` is a bound
   on recursion, in the same family as `SolverConfig::max_decomposition_depth` and
   `LookupBounds::max_depth`, not a compute budget: reaching it produces an explicit
   `Blocked` need rather than a truncated answer. The open question is whether a
   requirement ever legitimately needs depth > 2, and the corpus should be extended
   with one case that does, so the bound is measured rather than assumed.
7. **The fairy-tale lexicon's fate.** This plan keeps it as a regression corpus. It is
   also, honestly, 102 lines of memorised content about one document sitting in
   `data/agentic-coding/`. Once the general path grounds the tale from Wikisource +
   Wiktionary at the same or better coverage, the lexicon should be deleted and the
   tale rediscovered — the forget-and-rediscover proof applied to the formalizer's own
   seed. That is a follow-up leaf, not part of this plan, and it should be filed
   rather than assumed.
8. **Two need vocabularies becoming one touches `meta_frame`.** `NeedStatus` is used
   by `src/solution_evidence.rs:21,90,184` and `src/recipe_interpreter.rs:35,260,368`,
   the latter under the event-for-event parity obligation R343. L3 must prove parity
   is preserved; if adding `NeedOrigin` to a row changes the recorded event stream,
   the origin belongs beside the ledger rather than inside it.
9. **B5 overlaps here.** `ProcedureStep::verified` is set only by an execution record,
   which is exactly B5's "every obligation node carries an execution record before it
   may be satisfied". This plan introduces the field and leaves it `false`; B5 owns
   making it true. Introducing the field without B5 risks a third half-finished
   lifecycle, so L9's test must assert the field is `false` on every extraction path
   rather than leaving the invariant to a later plan.
