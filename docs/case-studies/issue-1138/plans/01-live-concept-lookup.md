# Plan 01 — Live concept lookup through the sources registry (bottleneck B1 of #1138)

Status: planned, nothing implemented. Written before the code so the work can be
resumed from any point. A box is ticked in the same commit that lands its leaf; a
leaf that turns out wrong is struck through with the reason, never deleted.

## Issues addressed

- **#1138 B1** — "The 'understand each word' step has no live lookup in the coding
  path": asks for one `UnknownConceptLookup` implementation that walks the sources
  registry (dictionary → encyclopedia → Q&A → docs), bounded by depth and evidence
  rather than a budget, content-addressed, used by the universal loop as well as
  the coding path. This plan delivers all of it.
- **#710** (maintainer instruction, quoted verbatim in
  `docs/case-studies/issue-710/plans/README.md:14-36`) — "make this algorithm to
  discover enough knowledge in the internet to understand each word/concept … The
  goal is not to know everything in advance, the goal to know how to get know
  anything when it is needed." This plan delivers the *word/concept* half; the
  *reconstruct the step by step guide* half is B2 and the *formalization itself may
  need recursive knowledge collection* half is B4 (plan 04).
- **#873** ("not knowing is not the end") — asked that every unresolved input
  trigger research and still produce an evidence-backed answer. PR #983 delivered
  that for the *unknown-prompt fallback* (`src/solver_unknown_reasoning.rs`) only.
  This plan extends the same principle to an unresolved **word inside a prompt the
  solver otherwise matched**, which today is not an unknown at all.
- **#919** ("research → verified coding procedures") — PR #992 delivered the
  research→verify→ledger loop but, per
  `docs/case-studies/issue-919/README.md` § Dependency and scope, "the first
  learned operation is intentionally the already verified workspace-rewrite
  family" and only the repository's own `formal_ai_coding_procedure_v1` source
  shape is accepted. This plan supplies the missing upstream stage: the concept
  senses that make a *requirement* legible before any procedure is researched.
- **#991** ("dynamic multi-source how-to synthesis") — delivered the only live,
  bounded, registry-driven, provenance-carrying recursive walk in the tree
  (`src/how_to_guide.rs`). This plan generalises that walk into a shared kernel so
  concept lookup is a second need kind of one implementation, not a second
  implementation.
- **#959 / B9** (handler ratchet) — this plan must not add a `try_*` arm. It adds
  one generic kernel, one registry field, and seed data.
- **#1085 / R1085-9** (the links network is not the system that reasons) — concept
  senses land as links with provenance, not as Rust tables.
- **Issues plan 13's coverage table names this plan as a deliverer of** (added by
  the 2026-09-16 reconciliation so the two documents agree): **#720** `последние
  новости` and **#1063** `Какого размера средний корень яблони?` (the retrieval
  half of the frontier classes, with plan 10 owning the routing); **#801** /
  **#821** "search online for Elon Musk" (the retrieval walk); **#826** `ФБС vs
  ФБО` and **#827** `Что такое фуфломицин?` (the canonical B1 cases — E5 of plan
  13 records that three correct sources are fetched and their *page titles* are
  emitted, so sense extraction is the missing stage); **#722** (the subject's
  concept graph behind a composed essay); **#800** / **#872** (retrieval under a
  marketplace constraint, with the option network as a new leaf); **#939** (the
  installation-guide corpus is retrieved by this walk rather than hand-authored);
  **#940** (the research half); **#952** (the browser reads the same seed through
  the WASM parser — L13); **#453** (the splitting approaches a moonshot needs are
  retrieved, deduplicated to their first source).
- **Plan 00 of this batch** (`00-root-causes-and-integration.md` §4.1, §4.2) fixes
  the shared contract names `Need`, `NeedKind`, `SourceLookup`, `LookupBounds`,
  `LookupOutcome` and delegates two decisions to this plan: which `need_kinds` a
  registry source declares (§4.2 last bullet) and whether the coding path's
  `UnknownConceptLookup` stays as a thin adapter or is replaced (§4.2 fourth
  bullet). Both are decided below; this plan adopts every plan-00 name verbatim,
  so plan 14's reconciliation leaf has nothing to rename.

## Current state

### The trait exists; the only implementation refuses

`src/coding/concept_discovery.rs:81-83` declares

```rust
pub trait UnknownConceptLookup {
    fn lookup(&mut self, phrase: &str, depth: usize) -> Option<ConceptEvidence>;
}
```

`src/coding/concept_discovery.rs:85` declares `struct NoLookup;` — not `pub`, so no
caller outside the module can even name it — and `:87-91` implements it as
`fn lookup(&mut self, _phrase: &str, _depth: usize) -> Option<ConceptEvidence> { None }`.
The production entry `discover` at `src/coding/concept_discovery.rs:189-192` reads
`discover_with_lookup(spec, catalog, &mut NoLookup, DiscoveryBounds::default())`.
`DiscoveryBounds` (`:59-68`) defaults to `max_depth: 2, max_pages: 8` — bounds for
a walk that never happens.

The production runtime never calls `discover_with_lookup`:
`src/coding/synthesis_runtime.rs:210` and `:215` both call `discover(spec, &catalog)`,
and `discover_and_compose` (`src/coding/synthesis_runtime.rs:204-220`) is the only
entry the solver uses (`src/solver_handlers/program_synthesis.rs:49`). A grep for
`discover_with_lookup` outside `src/coding/concept_discovery.rs` finds only
`tests/unit/coding_discovery/concepts.rs`. **The trait is exercised solely by a
test-local fake.**

### Even a working lookup could not change an answer

Three separate facts make `ConceptEvidence` a dead end:

1. **The lookup is gated on total ignorance.**
   `src/coding/concept_discovery.rs:212-219` consults the lookup only when
   `structures.is_empty() && candidates.is_empty()`. A sentence that matched one
   structural meaning and contains one unknown word never asks.
2. **The unit of lookup is a sentence, not a word.**
   `:203-206` maps over `spec.requirement_sentences` and passes the whole `phrase`
   to `lookup`. The step #1138 calls "understand each word" is implemented as
   "understand each sentence, or nothing."
3. **Evidence never becomes a part.** `ConceptEvidence`
   (`src/coding/concept_discovery.rs:74-79`) carries only
   `phrase / definition / source_url / depth`. It is pushed into `ConceptMap::evidence`
   (`:220`), rendered into Links Notation at `:177-182`, and flips `status` from
   `"blocked"` to `"satisfied"` at `:222-230` — and that is all. It is absent from
   `ConceptMap::candidate_ids` (`:133-139`), absent from `ConceptMap::identity`
   (`:142-153`), and `composition::compose(&spec, &concepts)`
   (`src/coding/composition.rs:46`) consumes `needs[].structures` and
   `needs[].candidates`. A definition retrieved from Wiktionary today would change
   one status string and nothing else.

### The universal loop performs no retrieval and is barely reachable

`src/solver.rs:874-886`:

```rust
fn record_external_search(&self, log: &mut EventLog, prompt: &str) {
    if self.config.offline {
        log.append("search:external", "skipped:offline".to_owned());
        return;
    }
    log.append("search:external", prompt.to_owned());
    log.append(
        "policy:no_fetch_capability",
        "external search requested but no retrieval was executed".to_owned(),
    );
}
```

Its single caller is `src/solver.rs:636-638`, guarded by
`requires_external_lookup(prompt)`, which is `src/solver_helpers/mod.rs:104-111`:

```rust
pub fn requires_external_lookup(prompt: &str) -> bool {
    let lower = prompt.to_lowercase();
    lower.contains("capital of")
        || lower.contains("cite a definition")
        || lower.contains("define associative memory")
        || lower.contains("from wikipedia")
        || lower.contains("born in")
}
```

Five lowercase English literals. Four of the five doctrine languages cannot reach
the step at all, and when they do it appends `policy:no_fetch_capability` and
returns. Retrieval exists in `src/how_to_guide.rs`, `src/coding/function_catalog/*`,
`src/source_research.rs` and `src/coding_research_learning.rs` — never in the loop
every prompt passes through.

### The registry already declares the dictionary tier — and structurally excludes it

`data/seed/sources-registry.lino` declares 13 sources. Four of them are exactly the
lexical/encyclopedic tier B1 asks for: `wikidata` (`:2-13`), `wiktionary` (`:14-24`,
CC BY-SA 3.0, `api "https://api.dictionaryapi.dev/api/v2/entries/en/{lemma}"`),
`wordnet` (`:25-35`, CC BY 4.0) and `wikipedia` (`:36-46`, CC BY-SA 4.0). **None of
the four declares `service_group`, `settings_key` or `how_to_role`.**

The consequences are mechanical. `SourceRecord::is_external_trusted`
(`src/seed/sources.rs:118-121`) tests `service_group == "external_trusted"`, so
`external_trusted_sources()` (`:184-189`) returns nine sources and never those four.
`how_to_guide::select_sources` (`src/how_to_guide.rs:296-321`) filters on
`record.how_to_role.contributes()` (`src/seed/sources.rs:64-68`), which is `false`
for a missing role. The only live registry-walking selector in the tree therefore
*cannot* return a dictionary. `external_service_settings_keys()`
(`src/seed/sources.rs:199-207`) yields the four keys mirrored in
`src/web/app/main.jsx:1063-1068`; there is no opt-out for a dictionary lookup
because there is no dictionary lookup.

### There is a committed lexical cache that no Rust code reads

`data/cache/wiktionary/en/` and `data/cache/wordnet/en/` hold 2,053 files — one
`.json` plus a lossless `.lino` projection per lemma, e.g.
`data/cache/wiktionary/en/mass.lino` (`license / name "CC BY-SA 3.0"`,
`meanings / entry / definitions / entry / definition "…"`, `partOfSpeech noun`) and
`data/cache/wordnet/en/a.lino` (`language en`, `lemma a`, `license`, `senses / entry /
definition / id / partOfSpeech / synonyms`). They are produced by
`scripts/ground-wiktionary.py` and `scripts/ground-wordnet.py`, whose own docstring
says the lemma list is "every single-word English `surface / text` belonging to a
meaning that already carries a `grounded-in <id>` anchor" — derived from the seed,
and **English only**: `data/cache/wiktionary/` and `data/cache/wordnet/` each contain
exactly one subdirectory, `en`. A grep for `data/cache/wiktionary` across `src/`
returns nothing; the consumers are `scripts/ground-wiktionary.py`,
`scripts/audit-total-closure.py`, `scripts/build-views.py` and
`scripts/check-cache-budget.rs`. The dictionary is data with no runtime.

### What happens today for one concrete unfamiliar prompt in each language

Held-out word: **isogram** (a word with no repeated letter). `grep -ril isogram src
data docs tests` returns nothing, so it is absent from every seed file, every
cache entry, every test fixture and every catalog template.

| Language | Prompt | What happens today |
| --- | --- | --- |
| en | `Write a Python function is_isogram(word) that returns True when the word is an isogram.` | `task_spec::recognise` produces a `CodingTaskSpec`; `structures_for` (`concept_discovery.rs:246-282`) matches nothing for "isogram"; `candidates_for` (`:321`) finds no stdlib part scoring ≥ 3.0 and no Wikifunctions label overlap; the lookup is `NoLookup`; the single need is `status "blocked"`; `composition::compose` selects nothing; `src/solver_handlers/program_synthesis.rs:62-88` emits `skill_gap` + `write_program_skill_gap`. |
| ru | `Напиши функцию Python is_isogram(word), которая возвращает True, если слово — изограмма.` | Same path. Additionally `изограмма` is absent from every `lexeme ru` surface, and `data/cache/wiktionary/` has no `ru/` directory, so even the cached dictionary could not help. |
| hi | `Python फ़ंक्शन is_isogram(word) लिखें जो True लौटाए यदि शब्द एक आइसोग्राम है।` | Same path; no `hi/` cache directory. |
| zh | `编写 Python 函数 is_isogram(word),当 word 是 isogram 时返回 True。` | Same path; `research_phrases` (`synthesis_runtime.rs:254-294`) does query Wikifunctions in `zh` then `en` (`:44-77`), but Wikifunctions has no `isogram` function, so `synthesis:source_miss` is logged and the need stays blocked. |
| es | `Escribe la función Python is_isogram(word) que devuelva True cuando la palabra sea un isograma.` | Same path; no `es/` cache directory. |

And through the universal loop, with a non-coding phrasing of the same gap —
`Is this sentence a lipogram in e?` (`lipogram` likewise absent from the tree) —
`requires_external_lookup` returns `false` in all five languages, so
`record_external_search` is not even called; `answer_unknown_prompt`
(`src/solver_unknown_reasoning.rs:27`) runs link-memory → public-knowledge-cache →
(online only, and only for a seeded language) `answer_web_search_query` → the
honest unresolved-unknown reply. The word `lipogram` is never looked up anywhere.

### Ledgers and tests that record the gap

- `docs/case-studies/issue-710/plans/07-prerequisite-discovery-bridge.md:26-28`:
  "The live memory-contract formalizer preserves source sentences, but produces no
  concepts or procedures. Deep formalization therefore remains an open prerequisite
  itself."
- `docs/case-studies/issue-710/plans/02-dynamic-discovery-design.md:50` already
  specified the intended behaviour — "a label or description whose words are not in
  the lexicon is looked up (Wiktionary/Wikipedia/Wikidata chain), bounded by depth
  and page limits like `how_to_guide::GuideBounds`" — and it did not land.
- `docs/requirements-traceability.md:708` — `R710-D8 … tests/unit/coding_discovery/concepts.rs
  … not yet confirmed`. The row that pins concept discovery is pinned by a test
  whose only lookup is a fake.
- `tests/unit/coding_discovery/multilingual.rs:19-50` is the five-language held-out
  suite (25 cases from `data/benchmarks/coding-discovery-paraphrases.lino`); every
  case is a *structural* family (gcd, sum_product, near_pair, count_to_100, …)
  already covered by the 82 seeded meanings in
  `data/seed/meanings-coding-structure.lino`. No case in the corpus requires a word
  the seed does not have.

## Root causes

1. **The only implementation of the extension point is a refusal, and the
   production entry hard-wires it.**
   Evidence: `src/coding/concept_discovery.rs:85-91`, `:189-192`;
   `src/coding/synthesis_runtime.rs:210`.
   Mechanism: generality is bounded by the seed. Any requirement whose vocabulary
   falls outside the 82 structural meanings is unrepresentable, so it cannot be
   composed, so it fails before algorithm search begins. Adding words to the seed
   is exactly the memoization the doctrine forbids.

2. **The lookup is gated on *total* ignorance rather than on *any* unresolved
   word.** Evidence: `src/coding/concept_discovery.rs:212-219`.
   Mechanism: the common real case — a sentence half-understood — is
   indistinguishable from a sentence fully understood. Partial knowledge silently
   suppresses the only path to complete knowledge, so the system cannot tell that
   it is missing something.

3. **`ConceptEvidence` is not a part, so retrieval cannot affect an answer.**
   Evidence: the struct at `src/coding/concept_discovery.rs:74-79` versus
   `CandidatePart` at `:94-108`; `candidate_ids` at `:133-139`; `compose` at
   `src/coding/composition.rs:46`.
   Mechanism: a closed loop — discovery may record what it learned, but composition
   reads a different field, so learning cannot change behaviour. This is B7's
   "proposal-only" failure appearing inside a single function call.

4. **The universal loop has no retrieval, and its trigger is five English
   literals.** Evidence: `src/solver.rs:874-886`; `src/solver_helpers/mod.rs:104-111`.
   Mechanism: every non-coding prompt — GSM8K, MATH, CoEdIT, object counting (B8) —
   passes through a loop that cannot look anything up, and four of five languages
   cannot even request that it try. Retrieval is a property of handlers, which is
   why each new capability arrives as another handler (B9).

5. **The registry's dictionary tier is excluded by the only selector that walks
   it.** Evidence: `data/seed/sources-registry.lino:2-46` (no `service_group`,
   `settings_key` or `how_to_role` on wikidata/wiktionary/wordnet/wikipedia) versus
   `src/seed/sources.rs:118-121,184-189` and `src/how_to_guide.rs:296-321`.
   Mechanism: the selection policy has exactly one axis (`how_to_role`), so a second
   need kind has no way to express "these four, in this order" without either
   hard-coding a list in Rust or abusing the how-to axis.

6. **The bounded recursive walk exists once, inside a 828-line module dedicated to
   procedures.** Evidence: `src/how_to_guide.rs:325-388` (`synthesize_how_to_guide`),
   `:527-533` (`struct Walk`), `:534` (`capture_service`); the file is 828 lines
   against the 1,000-line Rust cap in `scripts/check-file-size.rs:21-27`.
   Mechanism: a second need kind must either copy the walk (two bounded walkers that
   drift — precisely the parity defect #991 was filed to remove) or cannot have one.

7. **The lexical cache has no runtime and no non-English content.**
   Evidence: `data/cache/wiktionary/en/` and `data/cache/wordnet/en/` only; no `src/`
   reference; `scripts/ground-wiktionary.py` derives its lemma list from already
   grounded English surfaces.
   Mechanism: "forget and rediscover" cannot be proven for concepts, because there
   is nothing to forget that the runtime would rediscover; and the ru/hi/zh/es
   halves of the doctrine have no cached evidence at all, so an offline
   five-language proof is impossible today.

## Solution options

### Option A — A dedicated concept-lookup module beside the how-to synthesiser

**Description.** Add `src/concept_lookup.rs` implementing `UnknownConceptLookup` over
`CachedSourceClient`, with its own selection of dictionary sources, its own bounded
walk, its own extractors, and its own outcome reporting. Leave
`src/how_to_guide.rs` untouched.

**Architecture sketch.** `RegistryConceptLookup { client, preferences, bounds,
availability }` → for each unknown surface: pick sources by a new registry
registry field → `client.fetch(record.api_url(&[("lemma", surface)]))` → per-source
extractor → `ConceptSense` → `ConceptEvidence`. A private copy of the depth/page
accounting from `how_to_guide::capture_service`.

**Pros.** Smallest diff to existing green code; no risk to the #991 parity fixture;
each purpose's bounds are independently tunable; lands in one commit.

**Cons.** Two bounded walkers. `GuideBounds` (`src/how_to_guide.rs:40-64`) and the
new bounds would have to be kept consistent by review, not by the compiler. The
accessibility cache (`src/service_accessibility.rs:40`) would be consulted by two
callers with two conventions. The browser mirror would need a second walk too
(`formal_ai_worker_how_to_guide.js` has 100+ lines of registry parsing that would be
duplicated). Exactly the divergence R991-1 was written to prevent.

**Doctrine fit.** Generalization: partial — generalises vocabulary but not the
mechanism. No hard-coding: good, if selection is registry-driven. Honesty: good.
Associative stack: neutral. Forget/rediscover: good.

**Effort.** ~3 leaves, ~700 lines Rust + ~250 lines JS. **Risk.** Medium-low now,
high later (drift).

### Option B — One need-kind-parameterised retrieval kernel; concept lookup is a second need kind

**Description.** Extract the bounded, recursive, provenance-carrying, accessibility-aware
walk out of `src/how_to_guide.rs` into `src/source_walk.rs`, parameterised by a
`NeedKind` read from the registry and by a `CaptureExtractor` that turns bytes
into items. `how_to_guide` becomes "the `Procedure` kind with a step extractor";
`concept_lookup` becomes "the `Concept` kind with a sense extractor". The
universal loop and the coding path both call `concept_lookup`.

**Architecture sketch.**

```
data/seed/sources-registry.lino   need_kinds / source_tier / service_group / settings_key
        │
        ▼
src/seed/sources.rs      SourceRecord { need_kinds, tier, settings_key, … }
        │
        ▼
src/source_walk.rs       walk_sources::<T, E>(kind, subject, client, prefs,
                             bounds, availability, now) -> WalkOutcome<E::Item>
        ├──────────────► src/how_to_guide.rs   E = StepExtractor   (existing behaviour)
        └──────────────► src/concept_lookup.rs E = SenseExtractor  (new)
                                   │
                    ┌──────────────┴───────────────┐
                    ▼                              ▼
   src/coding/concept_discovery.rs        src/solver.rs step 7
   (RegistryConceptLookup: UnknownConceptLookup)   (same function, same cache)
```

**Pros.** One bounded walk, one accessibility convention, one provenance shape, one
parity fixture pattern. `how_to_guide` shrinks below its warning band. The browser
mirror gains one shared walk module instead of two. Selection policy is entirely
registry data, so a new source or a new need kind is a `.lino` edit. Satisfies B1's
"used by the universal loop as well as the coding path" with literally one
implementation.

**Cons.** Touches a green, fixture-pinned module (#991), so the first leaf must be a
pure refactor proven byte-identical against `tests/fixtures/issue-991/expected-guides.json`.
Larger first commit. Requires a generic over both the transport and the extractor,
which constrains how the browser mirror is written.

**Doctrine fit.** Generalization: strong — one mechanism, many need kinds, chosen by
data. No hard-coding: strong. Honesty: strong (one outcome vocabulary, reused).
Associative stack: strong (the policy is links; Rust holds one primitive). Forget
and rediscover: strong (one content-addressed cache path for every need kind).

**Effort.** ~7 leaves, ~900 lines Rust changed/added, ~350 lines JS.
**Risk.** Medium, front-loaded and mechanically checkable.

### Option C — Lookup as pure registry method data interpreted by the recipe engine

**Description.** Add no Rust trait implementation. Declare the lookup as `method`
records in `.lino`, consumed by `src/method_registry.rs` and executed by
`src/recipe_interpreter.rs`, with Rust contributing exactly one new primitive:
`fetch(url) -> capture`. Selection, ordering, depth and extraction become recipe
steps.

**Architecture sketch.** `data/meta/concept-lookup-recipe.lino` declares ordered
`meta_step` records; `recipe_interpreter` gains `fetch`, `extract`, `link`
primitives; `UnknownConceptLookup` is satisfied by a thin adapter that runs the
recipe.

**Pros.** Maximal doctrine fit on "associative stack only": behaviour is data,
Rust is an interpreter. Directly advances B9 (#959) and R1085-9 (the links network
as the executable system of record). A forgotten recipe is rediscoverable.

**Cons.** `src/recipe_interpreter.rs:35,260,368` is today a *recorder* interpreter:
it replays recorder primitives against a `NeedLedger`/`ProblemFrame` and proves
event-for-event parity with `meta_core::record_meta_core` (R343). It has no I/O, no
error model, no `SourceCapture`, no license handling. Adding retrieval there means
either weakening the parity obligation or building a second interpreter. Extraction
of a definition from HTML/JSON is not expressible as recorder steps without
inventing a byte-level DSL. The honesty contract (`SourceCapture` owns the bytes it
hashes, `src/source_fetch.rs:102-158`) would have to be re-established inside the
interpreter. This is a large, separate change with no test that would fail first.

**Doctrine fit.** Associative stack: strongest. Everything else: unprovable in one
plan.
**Effort.** ~15 leaves, touching the parity obligation. **Risk.** High.

### Option D — Reuse web search fusion instead of the registry

**Description.** Point the lookup at `source_research::execute_source_research`
(`src/source_research.rs:137-164`), which already fuses DuckDuckGo rankings and
captures the first *n* result pages.

**Architecture sketch.** `RegistryConceptLookup` → `execute_source_research(client,
&format!("{surface} definition"), 3)` → extract the first paragraph of each page.

**Pros.** Zero new transport work; already provenance-carrying; already offline-replayable;
already used by `coding_research_learning` (`src/coding_research_learning.rs:27,492`).

**Cons.** Violates "trusted sources primarily": a ranking is an observation, not a
source tier — `src/source_research.rs:1-8` says so itself ("Search results are
observations, not facts"). The result set is not reproducible across time, so the
forget-and-rediscover proof degrades from "same content hash" to "some page said
something". Licensing is unknown per result, so a definition cannot be quoted
safely. `docs/case-studies/issue-710/plans/07:12-15` already records the limit:
"`.gov`/`.edu` preference also does not identify a compiler's official source."
**Doctrine fit.** Trusted sources: fails. Honesty/licensing: fails.
**Effort.** ~2 leaves. **Risk.** Low effort, high doctrine cost.

## Decision

**Selected: Option B**, with two policies borrowed from Option C: (1) every
selection, ordering and bound is declared in `.lino` and read by the kernel — Rust
contains no source list, no per-source ordering and no per-language table; (2) the
extractor bindings are registry data (`extractor wiktionary_entry_v1`), so adding a
source is a seed edit plus one extractor function, and the *choice* of extractor is
never a `match` on a host name in a handler.

Reasons:

- B1's "fixed means" names one implementation used by two callers. Option B is the
  only option where that sentence is literally true at the type level:
  `walk_sources` has one definition and `concept_lookup::lookup_surface` has one
  definition, and both the loop and the coding path call it.
- #991 already paid for a bounded, recursive, accessibility-aware, license-carrying,
  offline-replayable walk with a committed capture fixture and a browser mirror held
  to one expectation (R991-1, R991-6). Re-deriving that in a second module discards
  the evidence and re-opens the parity defect.
- The doctrine's "everything discoverable must be forgettable and rediscoverable"
  is a property of a cache layout. One kernel means one layout
  (`source-cache/objects/<sha256>.body`, `src/source_fetch.rs:353-355`) and one
  replay proof, not two.
- `src/how_to_guide.rs` is at 828 of 1,000 lines
  (`scripts/check-file-size.rs:21-27`); it cannot absorb a second need kind in place,
  and the extraction that relieves it is the same extraction Option B needs.

Rejected:

- **Option A** — produces two bounded walkers and two browser mirrors. The first
  divergence would be invisible until a user saw two different provenance
  vocabularies for the same cache. R991-1 exists because that already happened once.
- **Option C** — right destination, wrong step. `recipe_interpreter` is bound by an
  event-for-event parity obligation (R343) that retrieval would break, and a
  byte-level extraction DSL is a larger invention than the bottleneck requires.
  Option B leaves C reachable: once `walk_sources` exists, replacing its Rust
  selection loop with recipe-driven selection is a contained follow-up, and the
  registry fields this plan adds are the data that change would read.
- **Option D** — a ranked search result is not a trusted source, cannot be licensed
  for quotation, and cannot be rediscovered to the same content hash. Keeping
  `source_research` for B2's *procedure* search (where #919 already uses it under
  review gates) while B1's *definitions* come from the declared dictionary tier is
  the honest split.

## Architecture

### Files added

| Path | Contents |
| --- | --- |
| `src/source_walk.rs` | The one bounded recursive capture walk, need-kind- and extractor-parameterised. Owns plan 00 §4.2's `LookupBounds` and `SourceLookup`. |
| `src/concept_lookup.rs` | `ConceptSense`, `SenseExtractor`, `LookupOutcome`, `RegistrySourceLookup`, the `RegistryConceptLookup` adapter, `lookup_surface`, `unknown_surfaces`. |
| `src/concept_sense_ledger.rs` | Content-addressed, forgettable ledger of resolved senses. |
| `src/web/worker/formal_ai_worker_source_walk.js` | Browser mirror of the kernel (shared by every need kind). |
| `data/seed/meanings-concept-lookup.lino` | Five-language response meanings for the lookup's user-visible outcomes. |
| `data/meta/concept-lookup-recipe.lino` | The grounded meta-recipe for this step. |
| `tests/fixtures/issue-1138-b1/` | Committed real-service captures + `capture-manifest.lino` + `expected-senses.json`. **reconciled: was `tests/fixtures/issue-1138-b1/`, now suffixed `-b1` because plan 04 already uses `issue-1138-b4/` and plans 03, 06 and 08 will need their own (plan 00 §9 R13).** |
| `examples/issue_1138_concept_lookup_parity.rs` | Writes `expected-senses.json` from the Rust path. |
| `examples/issue_1138_concept_lookup_capture.rs` | Refreshes the committed captures live. |

All names checked free by grep over `src data scripts tests`: `source_walk`,
`SourceWalk`, `LookupBounds`, `WalkOutcome`, `CaptureExtractor`, `NeedKind`,
`RegistrySourceLookup`, `RegistryConceptLookup`, `ConceptSense`,
`ConceptSenseLedger`, `unknown_surfaces`, `need_kinds`. Note `ConceptLookup` is
**taken** (`src/concepts.rs:399`) and `SourceOutcome` is **taken**
(`src/how_to_guide.rs:174` as `GuideSourceOutcome`); this plan uses
`RegistrySourceLookup` and `WalkSourceOutcome` accordingly.

Names taken from plan 00 §4.2 unchanged, so plan 14's reconciliation leaf has
nothing to rename here: `SourceLookup`, `LookupBounds`, `LookupOutcome`, `Need`,
`NeedKind`, `need_kinds`.

### Files changed

| Path | Change |
| --- | --- |
| `src/how_to_guide.rs` | `capture_service`/`Walk`/depth accounting move to `source_walk`; module keeps step extraction, tier policy, ordering, rendering. Target ≤ 600 lines. |
| `src/seed/sources.rs` | `NeedKind` enum + `SourceRecord::need_kinds`; `sources_for_need_kind()`. |
| `src/coding/concept_discovery.rs` | Per-word needs; lookup consulted on any unresolved surface; evidence promoted to `CandidatePart`. |
| `src/coding/synthesis_runtime.rs` | `discover_and_compose` builds a `RegistryConceptLookup` and calls `discover_with_lookup`. |
| `src/solver.rs` | `record_external_search` performs the lookup; trigger becomes unresolved-surface-driven. |
| `src/solver_helpers/mod.rs` | `requires_external_lookup` replaced by `unresolved_surfaces_present`. |
| `data/seed/sources-registry.lino` | `need_kinds`, `service_group`, `settings_key`, `extractor`, `lemma_language` on the lexical tier. |
| `src/web/app/main.jsx` | Three new rows in `EXTERNAL_TRUSTED_SERVICES` (`:1063-1068`). |
| `data/meta/coding-discovery-recipe.lino` | A `coding_discovery_step_understand` record before `_discover`. |

### Rust signatures

> **reconciled: was `NeedKind` declared in `src/seed/sources.rs` by this plan
> and again in `src/formalization/needs.rs` by plan 04; now one definition in
> `src/needs.rs`, landed by plan 00's contract leaf C1 before this plan's L3,
> because the record every step connects through cannot have two Rust homes
> (plan 00 §9 R1).**

```rust
// src/needs.rs — the contract module (plan 00 leaf C1, lands before L3).
// `NeedKind` is plan 00 §4.1's `need.kind` vocabulary, and it is also this
// plan's registry selection axis: the `need_kinds` a source may answer, which
// plan 00 §4.2 delegates to this plan to *use in the registry*, not to declare.
//
// Variants, fixed by plan 00 §4.1: Concept, Procedure, Part, Prerequisite,
// Evidence, Decision, None. This plan reads the first four; plan 05 reads
// Evidence and plan 12 reads Decision.
pub enum NeedKind { Concept, Procedure, Part, Prerequisite, Evidence, Decision, None }

// src/seed/sources.rs — the second selection axis, read from the registry.
use crate::needs::NeedKind;

impl SourceRecord {
    /// The need kinds this source declares it may answer, in registry order.
    #[must_use] pub fn need_kinds(&self) -> &[NeedKind];
    #[must_use] pub fn answers(&self, kind: NeedKind) -> bool;
    /// Position in `data/seed/sources-registry.lino`; the declared order *is*
    /// the consultation order within one tier, so ordering is data.
    #[must_use] pub const fn registry_index(&self) -> usize;
}

/// Every source that declares `kind`, in consultation order: derived tier
/// descending, then registry order. No Rust code contains a source list.
#[must_use]
pub fn sources_for_need_kind(kind: NeedKind) -> Vec<SourceRecord>;
```

The dictionary → lexicon → encyclopedia → technical progression B1 asks for is
therefore **not** a new Rust enum: `wikidata`, `wiktionary`, `wordnet` and
`wikipedia` already occupy `data/seed/sources-registry.lino:2-46`, ahead of
`wikihow` (`:47`), `stackexchange` (`:63`) and `github` (`:164`). Declaring
`need_kinds (concept)` on the first four and `need_kinds (concept procedure)` on
the last three yields exactly that order from the file as written.

```rust
// src/source_walk.rs — the single bounded walk. `LookupBounds`, `LookupOutcome`
// and `SourceLookup` below are plan 00 §4.2's contract names.

/// Declared bounds. Every capture is charged against these, so a walk's cost is
/// knowable before it starts. Depth and evidence bounds, never a time or token
/// budget. `GuideBounds` becomes an alias of this type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LookupBounds {
    pub max_depth: usize,
    pub max_pages_per_service: usize,
    pub max_services: usize,
    pub max_items: usize,
    pub max_capture_age_seconds: u64,
}

/// Turns captured bytes into items of one need kind, and names which same-source
/// links are worth following next.
pub trait CaptureExtractor {
    type Item;
    fn extract(
        &self,
        record: &SourceRecord,
        capture: &SourceCapture,
        depth: usize,
        limit: usize,
    ) -> Vec<Self::Item>;
    fn follow(&self, record: &SourceRecord, capture: &SourceCapture, limit: usize) -> Vec<String>;
    fn entry_url(&self, record: &SourceRecord, subject: &str) -> Option<String>;
}

/// What one source produced, or why it produced nothing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WalkSourceOutcome {
    pub source_id: String,
    pub status: String,   // contributed | no_items | disabled | unbound_template
                          // | unreachable_cached | fetch_error | stale_capture
    pub detail: String,
    pub pages: usize,
    pub items: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WalkOutcome<I> {
    pub subject: String,
    pub items: Vec<I>,
    pub outcomes: Vec<WalkSourceOutcome>,
    pub bounds: LookupBounds,
}

/// Select the sources this need kind and these settings allow, walk each one
/// inside `bounds`, and return everything the extractor recognised together
/// with an outcome row for every source that could have contributed.
pub fn walk_sources<T: SourceTransport, E: CaptureExtractor>(
    kind: NeedKind,
    subject: &str,
    extractor: &E,
    client: &CachedSourceClient<T>,
    preferences: &ServicePreferences,
    bounds: &LookupBounds,
    availability: &mut ServiceAccessibilityCache,
    now: u64,
) -> WalkOutcome<E::Item>;

/// The sources a need kind may consult, in consultation order: declared kind
/// first, then derived tier descending, then registry order. Total and
/// reproducible. `how_to_role` survives only as the ordering hint inside
/// `NeedKind::Procedure`; the `HowTo`/`Concept` distinction is no longer a
/// separate axis.
#[must_use]
pub fn select_sources(
    kind: NeedKind,
    subject: &str,
    preferences: &ServicePreferences,
    bounds: &LookupBounds,
) -> Vec<SourceRecord>;
```

The plan-00 §4.2 contract is satisfied by one adapter over this kernel, so there
is exactly one `SourceLookup` implementation in the tree:

```rust
// src/concept_lookup.rs — plan 00 §4.2's contract, implemented once.

/// What a lookup produced, or honestly did not.
#[derive(Debug, Clone, PartialEq)]
pub enum LookupOutcome {
    /// Evidence, in consultation order. Every item carries the digest of the
    /// bytes it was read from.
    Found(Vec<ConceptSense>),
    /// No source answered. `consulted` names every source and its observed
    /// outcome, so the absence is attributable rather than bare.
    NotFound { consulted: Vec<WalkSourceOutcome> },
}

/// The single implementation of plan 00 §4.2. The universal loop calls it where
/// `src/solver.rs:874-886` now logs `policy:no_fetch_capability`; the coding
/// path reaches it through the `UnknownConceptLookup` adapter below; plan 04's
/// formalizer and plan 06's prerequisite recovery call it directly.
pub struct RegistrySourceLookup<'a, T: SourceTransport> { /* fields below */ }

impl<T: SourceTransport> SourceLookup for RegistrySourceLookup<'_, T> {
    fn lookup(&mut self, need: &Need, bounds: &LookupBounds) -> LookupOutcome;
}
```

**The decision plan 00 §4.2 delegates here:** `UnknownConceptLookup`
(`src/coding/concept_discovery.rs:81-83`) **stays as a thin adapter** rather than
being deleted in this plan. Deleting it would change `discover_with_lookup`'s
signature, which `tests/unit/coding_discovery/concepts.rs` pins, in the same
commit that introduces retrieval — two changes, one test, no way to tell which
broke. The adapter is one `impl` block whose body is
`self.inner.lookup(&need, &self.bounds)`, and plan 09's ratchet removes the trait
once every caller names `SourceLookup` directly. That removal is recorded as a
leaf here (L18) rather than left implicit.

```rust
// src/concept_lookup.rs

/// One retrieved sense of one surface form, with the provenance of the exact
/// bytes it was read from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConceptSense {
    pub surface: String,
    pub lemma: String,
    pub language: String,
    pub gloss: String,
    pub part_of_speech: String,
    pub synonyms: Vec<String>,
    pub source_id: String,
    pub source_url: String,
    pub sha256: String,
    pub fetched_at: String,
    pub cached: bool,
    pub tier: SourceTier,
    pub license_name: String,
    pub license_url: String,
    pub depth: usize,
}

impl ConceptSense {
    #[must_use] pub fn content_id(&self) -> String;       // stable_id over sha256+lemma+gloss
    #[must_use] pub fn to_links_notation(&self) -> String;
}

/// Registry-declared extractor bindings, chosen by the registry's `extractor`
/// field rather than by a host-name match in Rust.
pub struct SenseExtractor { surface: String, language: String }

impl CaptureExtractor for SenseExtractor {
    type Item = ConceptSense;
    /* … */
}

/// Plan 00 §4.2's `SourceLookup`, implemented once over the shared walk.
pub struct RegistrySourceLookup<'a, T: SourceTransport> {
    client: &'a CachedSourceClient<T>,
    preferences: &'a ServicePreferences,
    availability: &'a mut ServiceAccessibilityCache,
    bounds: LookupBounds,
    language: String,
    now: u64,
    consulted: BTreeSet<String>,
    outcomes: Vec<WalkSourceOutcome>,
}

impl<'a, T: SourceTransport> RegistrySourceLookup<'a, T> {
    #[must_use] pub fn new(/* … */) -> Self;
    /// The outcome row per source, so a caller can report why nothing was found.
    #[must_use] pub fn outcomes(&self) -> &[WalkSourceOutcome];
}

impl<T: SourceTransport> SourceLookup for RegistrySourceLookup<'_, T> {
    fn lookup(&mut self, need: &Need, bounds: &LookupBounds) -> LookupOutcome;
}

/// The thin adapter that keeps `src/coding/concept_discovery.rs` compiling while
/// retrieval lands. Body: build a `Need { kind: Concept, subject: phrase, … }`,
/// delegate, take the first sense. Removed by L18 once every caller names
/// `SourceLookup`.
pub struct RegistryConceptLookup<'a, T: SourceTransport> {
    inner: RegistrySourceLookup<'a, T>,
}

impl<T: SourceTransport> UnknownConceptLookup for RegistryConceptLookup<'_, T> { /* … */ }

/// Look one surface up, as the trait does, but returning every sense instead of
/// the first. The universal loop and the coding path both call this.
pub fn lookup_surface<T: SourceTransport>(
    surface: &str,
    language: &str,
    client: &CachedSourceClient<T>,
    preferences: &ServicePreferences,
    bounds: &LookupBounds,
    availability: &mut ServiceAccessibilityCache,
    now: u64,
) -> WalkOutcome<ConceptSense>;

/// The surfaces in `normalized` that no seeded meaning, no memory link and no
/// declared-name convention accounts for — the words the system must ask about.
/// Language-neutral: it asks the lexicon, never a literal list.
#[must_use]
pub fn unknown_surfaces(normalized: &str, language: &str) -> Vec<String>;
```

```rust
// src/coding/concept_discovery.rs — the three changes that make lookup matter.

/// Now carries provenance, so evidence can become a licensed part.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConceptEvidence {
    pub phrase: String,
    pub definition: String,
    pub source_id: String,      // new
    pub source_url: String,
    pub sha256: String,         // new
    pub fetched_at: String,     // new
    pub license: String,        // new
    pub depth: usize,
}

/// `NoLookup` becomes public so the offline configuration can name it.
pub struct NoLookup;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DiscoveryBounds {
    pub max_depth: usize,
    pub max_pages: usize,
    pub max_words: usize,       // new: how many unresolved surfaces one spec may ask about
}

/// Unresolved surfaces of one requirement sentence, in sentence order.
#[must_use]
pub fn unresolved_surfaces(normalized: &str, structures: &[StructuralMeaning]) -> Vec<String>;

/// A retrieved sense, as a candidate the composer can read.
#[must_use]
pub fn concept_candidate(evidence: &ConceptEvidence) -> CandidatePart; // kind: "concept_sense"
```

`candidate_kind_rank` (`src/coding/concept_discovery.rs:389-396`) gains
`"concept_sense" => 4`, after `source_program`, so a retrieved definition never
outranks a retrieved implementation.

```rust
// src/solver.rs — the loop's step 7 stops lying.
fn record_external_search(&self, log: &mut EventLog, prompt: &str, language: Language)
    -> Vec<ConceptSense>;
```

### Seed schema changes, shown as records

`data/seed/sources-registry.lino` — the lexical tier joins the live, opt-out-able
group and declares its lookup role, extractor and per-language entry template. The
`wiktionary` entry becomes (added lines marked `+`):

```lino
  source wiktionary
    name "Wiktionary"
    kind dictionary
+   service_group external_trusted
+   settings_key externalServiceWiktionary
+   default_enabled true
+   need_kinds (concept)
+   extractor wiktionary_entry_v1
    how_to_role none
    source_tier independent_corroboration
    primacy editorial_synthesis
      upstream "the durably archived usages each entry attests"
      basis "Wiktionary's criteria for inclusion require an entry to be attested from durably archived sources, so an entry stands one hop from the usage record it summarises."
    api "https://api.dictionaryapi.dev/api/v2/entries/{language}/{lemma}"
+   api_language en
    license_name "CC BY-SA 3.0"
    license_url "https://creativecommons.org/licenses/by-sa/3.0/"
    cache_path data/cache/wiktionary/
    note "Free Dictionary API serving Wiktionary content; one verified entry per lemma as .json plus a lossless .lino projection. Consulted live for unknown words and opt-out-able from settings."
```

`api_language` lists the language codes the endpoint actually serves; a lookup in a
language the source does not serve is reported `unbound_template`, never silently
answered in English. Matching additions, in the registry order that *is* the
consultation order: `wikidata` → `need_kinds (concept)`, `extractor
wikidata_entity_v1`; `wordnet` → `need_kinds (concept)`, `settings_key
externalServiceWordnet`, `extractor wordnet_sense_v1`, `api_language en`;
`wikipedia` → `need_kinds (concept)`, `settings_key externalServiceMediawikiFamily`,
`extractor mediawiki_summary_v1`,
`api "https://{language}.wikipedia.org/api/rest_v1/page/summary/{title}"`,
`api_language en ru hi zh es`. The already-live procedural sources gain a second
kind rather than a new axis: `wikihow` → `need_kinds (procedure)`,
`stackexchange` → `need_kinds (concept procedure)`, `github` →
`need_kinds (concept procedure prerequisite)` (the last so plan 06 can reach the
same walk for a missing compiler), `wikifunctions` and `rosetta_code` →
`need_kinds (part)` so plan 02's composer reaches the same kernel too.

`data/seed/meanings-concept-lookup.lino` — user-visible outcomes, five languages,
one record per intent and language, in the shape
`data/seed/meanings-formalization-report.lino:10-18` already uses:

```lino
meanings
  concept_lookup_unresolved
    defined-by output
    role output_display_request
    precondition "A lookup for one surface produced no sense from any consulted source."
    effect "Name the surface, list every consulted source with its observed outcome, and state that no meaning was found; never supply a guess."
    unit "count"
    example "No source defined \"isogram\": wiktionary no_items, wordnet no_items, wikipedia unbound_template."
    grounding "src/concept_lookup.rs::lookup_surface"
  response_concept_lookup_unresolved_en
    defined-by concept_lookup_unresolved
    language en
    role output_display_request
    ...
```

Intents: `concept_lookup_unresolved`, `concept_lookup_resolved`,
`concept_lookup_sources_heading`, `concept_lookup_citation`,
`concept_lookup_offline_miss`, `concept_lookup_disabled` — six intents × five
languages = 30 response records, plus six meaning records.

`data/meta/concept-lookup-recipe.lino`, in the shape of
`data/meta/coding-discovery-recipe.lino:1-12`:

```lino
concept_lookup_meta_recipe
  record_type meta_recipe
  topic live_concept_lookup
  issue 1138
  selector src/source_walk.rs
  walker src/source_walk.rs
  extractor src/concept_lookup.rs
  memory src/concept_sense_ledger.rs
  consumer src/coding/concept_discovery.rs
  consumer src/solver.rs
concept_lookup_step_enumerate
  record_type meta_step
  order 1
  function unknown_surfaces
concept_lookup_step_select
  record_type meta_step
  order 2
  function select_sources
concept_lookup_step_walk
  record_type meta_step
  order 3
  function walk_sources
concept_lookup_step_extract
  record_type meta_step
  order 4
  function extract
concept_lookup_step_remember
  record_type meta_step
  order 5
  function remember
```

`tests/fixtures/issue-1138-b1/capture-manifest.lino` reuses the #991 manifest shape
verbatim (`url`, `sha256`, `fetched_at`, `bytes`, `license_name`, `license_url`).

### Cache layout and content addressing

No new transport and no new cache format. Every byte arrives through
`CachedSourceClient::fetch` (`src/source_fetch.rs:201-230`), which writes
`<cache_dir>/source-cache/objects/<sha256>.body` plus
`<cache_dir>/source-cache/<cache_key(url)>.meta` (`:232-237`, `:326-355`) and
verifies the recorded digest against the bytes on every read (`:294-301`). Cache
root is `FORMAL_AI_SOURCE_CACHE_DIR`, falling back to `FORMAL_AI_CACHE_DIR`, then
`data` — the same resolution `src/coding/synthesis_runtime.rs:28-31` already uses.

The derived layer is `src/concept_sense_ledger.rs`, modelled on
`src/coding/discovered_procedures.rs:133-197`: one `concept-senses.lino` under
`FORMAL_AI_CACHE_DIR`, each record keyed by
`ConceptSense::content_id() = stable_id(sha256 ‖ lemma ‖ language ‖ gloss)`, carrying
`source_url`, `sha256`, `fetched_at`, `license_name`, `depth`, `rediscover <url>`.
Per `docs/case-studies/issue-710/plans/06:232-239` the record retains the locator
and identity, never the disposable payload, and a URL "supports reacquisition, not
an assertion that today's page reproduces a historical observation byte-for-byte."

The committed `data/cache/wiktionary/en/*.lino` and `data/cache/wordnet/en/*.lino`
projections stay exactly as they are and gain their first Rust reader: the
`wiktionary_entry_v1` and `wordnet_sense_v1` extractors parse the same schema from
either a live capture's JSON or the committed `.lino` projection, so the 2,053
existing files become an offline corpus instead of an orphan. `scripts/check-cache-budget.rs:64,94`
caps a bucket at 128 records; the lookup writes nothing into `data/cache/**` at
runtime (it writes only into `FORMAL_AI_SOURCE_CACHE_DIR`), so the budget gate is
untouched.

### Offline replay

`CachedSourceClient::new` is offline by default (`src/source_fetch.rs:171-181`,
`online: false`); `with_online(live)` is the single opt-in and `live_fetch_enabled()`
(`src/coding/synthesis_runtime.rs:306-313`) reads `FORMAL_AI_LIVE_FETCH`. An offline
miss returns `FetchError::OfflineCacheMiss` (`:216-218`), which the walk records as
the outcome row `status "offline_cache_miss"` and never as a sense. A run with the
committed fixtures replays byte-identically: the test asserts every
`ConceptSense::sha256` equals the digest in `capture-manifest.lino` and that the
transport was called zero times (the fixture transport panics on `get`, as
`tests/unit/issue_991_how_to_synthesis.rs` already does).

### Settings keys and opt-outs

New keys: `externalServiceWiktionary`, `externalServiceWordnet`. Wikipedia and
Wikidata join the existing `externalServiceMediawikiFamily`; Stack Exchange and
GitHub keep their keys and simply gain a `need_kinds`. `ServicePreferences::allows`
(`src/how_to_guide.rs:117-121`) is already authoritative in both directions and moves
to `source_walk` unchanged, so an explicit `false` silences a dictionary the registry
enables by default and the source is reported `disabled` with zero pages and zero
senses, never contacted, cached or not (the R991-2 contract, applied to a second
need kind). `src/web/app/main.jsx:1063-1068` gains two rows; `external_service_settings_keys()`
(`src/seed/sources.rs:199-207`) then returns six keys and the existing settings-parity
test extends to them.

### Licensing

Each `ConceptSense` carries `license_name`/`license_url` copied from the registry
record the bytes came from, and the rendered answer cites them per sense, exactly as
`src/how_to_guide.rs` renders `wikiHow (CC BY-NC-SA 3.0, sha256 1cbdf5f7a9d6)`.
Wiktionary CC BY-SA 3.0 and WordNet CC BY 4.0 permit quoting a gloss with
attribution; Wikipedia CC BY-SA 4.0 the same. **A retrieved gloss is quoted and
attributed; it is never inlined into generated code.** The Rosetta Code precedent is
binding here (`docs/meta-algorithm.md:897-901`): GFDL code is shown, never inserted.
A `concept_sense` candidate therefore carries `code: None` by construction and
`composition::compose` may use it only to *select* structural meanings and idioms,
never as a source of program text. A test pins that: no byte of any
`ConceptSense::gloss` may appear in a verified draft's `source`.

### Browser / WASM worker parity

`src/web/worker/formal_ai_worker_how_to_guide.js:68-97` already parses the registry
and filters `serviceGroup === "external_trusted"`. The walk moves to a new
`formal_ai_worker_source_walk.js` with the same need-kind/extractor split as the Rust
kernel, and `formal_ai_worker_how_to_guide.js` and a new
`formal_ai_worker_concept_lookup.js` become its two extractors. Both runtimes are
held to one recorded expectation, `tests/fixtures/issue-1138-b1/expected-senses.json`,
written from the Rust path by `examples/issue_1138_concept_lookup_parity.rs` and
asserted by `tests/unit/issue_1138_concept_lookup.rs` and
`tests/web/issue-1138-concept-lookup.test.mjs` — the R991-1 pattern, reused.
Worker line budgets live in `data/meta/worker-line-budget/`, one file per worker, so
two new workers add two new files and conflict with nothing. The browser has no
Python runtime (plan 02 §7), so the browser path reports senses and provenance and
marks any composed program **unverified**; it never claims execution.

### Failure and honesty behaviour — what the user sees when a lookup finds nothing

There is no floor and no guess. Three distinguishable outcomes, each rendered from
`data/seed/meanings-concept-lookup.lino` in the prompt's language:

1. **Nothing found, sources reachable.** The need stays `status "blocked"`; the reply
   names the surface, lists every consulted source with its observed outcome
   (`wiktionary no_items`, `wordnet no_items`, `wikipedia no_items`), and states that
   no meaning was found. Confidence 0.0. In the coding path this precedes the
   existing `write_program_skill_gap` body
   (`src/solver_handlers/program_synthesis.rs:69-87`) so the user learns *which word*
   blocked the task, not merely that it failed.
2. **Offline and not cached.** `status "offline_cache_miss"` per source, and the reply
   says the lookup was not attempted because live retrieval is off — the existing
   `skipped:offline` honesty boundary (`src/solver.rs:875-878`), now per source and
   per word.
3. **Opted out.** `status "disabled"` naming the `settings_key`, so an absence is
   attributable to the user's own choice, never unexplained.

`policy:no_fetch_capability` is deleted. It asserted a capability limit that will no
longer be true; leaving it would be a fabricated provenance of the system's own
state. Any consulted-but-failed source produces a real outcome row instead.

### How the universal loop and the coding path share one implementation

Both construct the same three values — a `CachedSourceClient` over the same cache
root, a `ServicePreferences` from settings, a `LookupBounds` — and call
`concept_lookup::lookup_surface`. The difference is only what they do with the result:

- **Coding path.** `src/coding/synthesis_runtime.rs::discover_and_compose` builds a
  `RegistryConceptLookup` and calls `discover_with_lookup(spec, &catalog, &mut lookup,
  bounds)` instead of `discover(spec, &catalog)`. Each sense becomes a
  `CandidatePart { kind: "concept_sense" }`; `structures_for` is re-run against the
  gloss, so "a word with no repeated letters" matches the seeded `distinct_elements`
  and `quantifier_all` meanings and the composer proceeds with parts it already has.
  **This is the mechanism by which lookup produces generality rather than
  memorisation: the retrieved text is used to reach seeded *structure*, not stored as
  an answer.**
- **Universal loop.** `src/solver.rs:636-638` calls
  `unresolved_surfaces_present(&normalized, language)` (replacing the five literals)
  and `record_external_search` returns `Vec<ConceptSense>`, which
  `answer_unknown_prompt` receives as an additional candidate source, ranked after
  link memory and the public knowledge cache and before the honest unknown reply
  (`src/solver_unknown_reasoning.rs:36-108`). B8's reasoning suites reach retrieval
  through the same call.

One `walk_sources`, one `lookup_surface`, one cache, one outcome vocabulary, one
ledger, two callers.

## Tests first

Every test below is written and observed failing before its implementation leaf.

### Held-out five-language cases (new corpus)

`data/benchmarks/concept-lookup-paraphrases.lino`, same shape as
`data/benchmarks/coding-discovery-paraphrases.lino:1-6`. Family `isogram` (coding
path) and family `lipogram` (universal loop). Both words verified absent from `src`,
`data`, `docs` and `tests` by grep before the corpus is written, and the existing
gate `held_out_sentences_are_not_seed_lexemes`
(`tests/unit/coding_discovery/multilingual.rs:70-88`) is extended to this corpus so a
future seed edit cannot quietly memorise them.

Coding-path prompts, verbatim:

- en — `Write a Python function is_isogram(word) that returns True when the word is an isogram.`
- ru — `Напиши функцию Python is_isogram(word), которая возвращает True, если слово — изограмма.`
- hi — `Python फ़ंक्शन is_isogram(word) लिखें जो True लौटाए यदि शब्द एक आइसोग्राम है।`
- zh — `编写 Python 函数 is_isogram(word),当 word 是 isogram 时返回 True。`
- es — `Escribe la función Python is_isogram(word) que devuelva True cuando la palabra sea un isograma.`

Universal-loop prompts, verbatim:

- en — `Is the sentence "quick brown fox" a lipogram in e?`
- ru — `Является ли фраза «quick brown fox» липограммой на букву e?`
- hi — `क्या वाक्य "quick brown fox" e का लिपोग्राम है?`
- zh — `句子 "quick brown fox" 是不是关于字母 e 的 lipogram?`
- es — `¿La frase "quick brown fox" es un lipograma en e?`

None of the prompts explains what the word means; the definition must come from a
source or the case fails.

### Unit tests

`tests/unit/issue_1138_concept_lookup.rs`:

- `the_registry_selects_dictionaries_before_encyclopedias_and_technical_sources`
- `an_unknown_word_resolves_to_a_licensed_sense_with_exact_provenance`
- `a_sense_is_quoted_and_attributed_and_never_inlined_into_generated_code`
- `a_settings_opt_out_silences_a_dictionary_and_is_reported_as_disabled`
- `an_offline_run_replays_the_committed_captures_without_any_transport_call`
- `a_lookup_that_finds_nothing_reports_every_consulted_source_and_no_gloss`
- `a_language_the_endpoint_does_not_serve_is_reported_unbound_not_answered_in_english`
- `the_walk_charges_every_capture_against_the_declared_bounds`
- `the_native_and_browser_runtimes_resolve_the_same_senses`

`tests/unit/issue_1138_source_walk_parity.rs` (the refactor guard, written first):

- `the_how_to_guide_is_byte_identical_after_the_walk_moves_to_the_shared_kernel`
  — replays `tests/fixtures/issue-991/source-cache` through the refactored
  `how_to_guide` and asserts equality with the committed
  `tests/fixtures/issue-991/expected-guides.json`.

`tests/unit/coding_discovery/concepts.rs` (extended):

- `a_partially_understood_sentence_still_asks_about_its_unresolved_words`
- `retrieved_evidence_becomes_a_candidate_part_the_composer_can_read`
- `concept_senses_rank_below_retrieved_implementations`

`tests/unit/coding_discovery/multilingual.rs` (extended):

- `held_out_unknown_word_tasks_share_one_concept_map_identity_in_five_languages`
  — asserts `ConceptMap::identity()` is equal across en/ru/hi/zh/es for the
  `isogram` family, the same invariant
  `five_language_paraphrases_share_specs_concepts_and_verified_compositions`
  (`:19-50`) enforces today for seeded families.

`tests/unit/issue_1138_universal_loop_lookup.rs`:

- `the_loop_asks_about_an_unresolved_word_in_every_registered_language`
- `the_loop_no_longer_claims_a_missing_fetch_capability`
  — asserts `policy:no_fetch_capability` appears in no event log.
- `an_offline_loop_keeps_the_explicit_no_network_boundary`

`tests/unit/concept_sense_ledger.rs`:

- `forgotten_senses_are_rediscovered_from_the_same_captures_to_the_same_content_id`
  — the B1 forget/rediscover proof, mirroring
  `forgotten_procedures_are_rediscovered_from_the_same_sources`
  (`docs/case-studies/issue-710/plans/02:194-196`): delete
  `concept-senses.lino`, replay the same committed captures offline, assert an
  identical `content_id` for every sense.
- `a_tampered_ledger_record_is_rejected_and_re_derived`

### Integration tests

`tests/integration/issue_1138_concept_lookup_http.rs` — a real `formal-ai serve`
process answering over `/api/openai/v1/chat/completions`, mirroring
`tests/integration/issue_991_how_to_http.rs`:

- `chat_completions_resolves_an_unknown_word_from_the_committed_captures`
- `chat_completions_honours_a_dictionary_opt_out`
- `chat_completions_reports_an_unresolved_word_instead_of_inventing_a_meaning`

### Specification tests

`tests/unit/specification/concept_lookup_meta_algorithm.rs` — grounds
`data/meta/concept-lookup-recipe.lino` against the live source: every `function`
named exists as `fn <name>` in the named file, `order` is contiguous 1..5, every
`consumer` path exists, exactly as
`tests/unit/specification/coding_discovery_meta_algorithm.rs` does for
`data/meta/coding-discovery-recipe.lino`.

`tests/unit/docs_requirements/issue_1138.rs` — the requirement shard's rows exist
and each names a test that exists.

### Web tests

`tests/web/issue-1138-concept-lookup.test.mjs` — the browser worker resolves the
same senses from the same fixtures and marks any composed program unverified.

### Gates and ratchets

- **No-memorization gate.** `tests/unit/coding_discovery/no_memorization.rs:11` is
  extended to scan `data/seed/**` and `src/**` for the held-out words `isogram`,
  `изограмма`, `आइसोग्राम`, `lipogram`, `липограмма`, `lipograma` — a fix that
  memorises the held-out vocabulary fails.
- **Gloss-not-code gate.** No substring of any `ConceptSense::gloss` may appear in a
  `VerifiedDraft::source` (pinned by
  `a_sense_is_quoted_and_attributed_and_never_inlined_into_generated_code`).
- **Handler ratchet (#959).** This plan adds zero `try_*` arms. The existing
  `data/meta/handler-migration-ledger.lino` count must not rise; the gate in
  `tests/unit/ci_gates.rs` covers it.
- **File-size gate.** `src/how_to_guide.rs` must drop below its 900-line warning band
  (`scripts/check-file-size.rs:21-27`); `src/source_walk.rs` and
  `src/concept_lookup.rs` each stay under 1,000.
- **Cache budget.** `rust-script scripts/check-cache-budget.rs` stays green; runtime
  captures go to `FORMAL_AI_SOURCE_CACHE_DIR`, never `data/cache/**`.
- **Capture-drift check.** `FORMAL_AI_LIVE_FETCH=1 cargo run --example
  issue_1138_concept_lookup_capture` re-fetches through the production path and
  reports drift against `tests/fixtures/issue-1138-b1/capture-manifest.lino`, gated
  exactly like the #991 refresh check.

### Benchmark commands and the honest numbers expected

```sh
# Five-language held-out unknown-word corpus (new; the number this plan is judged by)
cargo test --test unit coding_discovery::multilingual
cargo test --test unit issue_1138_concept_lookup
cargo test --test unit issue_1138_universal_loop_lookup

# Offline replay (no transport call permitted)
cargo test --test unit issue_1138_concept_lookup -- --nocapture

# Live refresh / drift (never in the default suite)
FORMAL_AI_LIVE_FETCH=1 cargo run --example issue_1138_concept_lookup_capture

# Upstream suites, unchanged by this plan but re-run to prove no regression
cargo run --bin formal-ai -- benchmark run --suite humaneval --slice 20 --append
cargo run --bin formal-ai -- benchmark run --suite mbpp --slice 20 --online --append
cargo run --bin formal-ai -- benchmark ratchet
```

Honest expectations, recorded whatever they are:

- **Unknown-word corpus:** today 0/10 (5 coding + 5 loop). The exit condition for
  this plan is 10/10 **online**. Offline-from-committed-captures is expected to be
  10/10 only after the fixtures are captured for all five languages; the honest
  intermediate number, if Wiktionary's API serves only `en`, is `en` resolved and
  ru/hi/zh/es reported `unbound_template` — which is a *2/10 with four honest
  refusals*, recorded as such, and answered by routing the four to Wikipedia's
  per-language REST endpoint rather than by faking an English gloss.
- **HumanEval / MBPP first 20:** expected unchanged at 20/20 and 20/20. This plan
  adds a source of parts; it removes none. Any fall is a regression the ratchet
  fails on.
- **Curated slices:** 13/13, 25/25, 1440/1440, unchanged.
- **GSM8K / MATH / object counting / CoEdIT:** unchanged at 2/20, 0/20, 0/20, 0/20.
  This plan makes retrieval *reachable* from the loop; B8 is the plan that routes
  those suites through it. Claiming movement here would be fabricating a result.
- **`policy:no_fetch_capability` occurrences in `src/`:** 1 → 0.
- **`UnknownConceptLookup` implementations outside tests:** 1 (`NoLookup`) → 2.

## Implementation leaves

Ordered; each individually verifiable and commit-sized.

**Three deviations from L2 and L3 as written, recorded rather than silent.**

1. `CaptureExtractor` has two methods, `entry_url` and `read`, not the three
   (`extract` / `follow` / `entry_url`) the Architecture section sketches.
   Every payload shape the how-to extractor recognises decides all three
   questions at once — what it produced, whether one more page is worth
   spending, and why it produced nothing — so three independent calls would have
   made the kernel classify the same bytes three times and still not know why a
   page was empty. `read` returns one `Extracted { items, follow, detail }`, and
   `detail` is the third thing the sketch had no place for: the outcome row's
   reason, which `capture_service` used to set through a `&mut` outcome.
2. `GuideBounds::max_steps` is `LookupBounds::max_items`. The alias L2 requires
   makes the two one type, and the field is named for what it bounds rather than
   for the one need kind that first bounded it.
   `src/web/worker/formal_ai_worker_how_to_guide.js` and the two tests that
   named it move in the same commit, so the Rust and browser bounds payloads
   stay one string.
3. `LookupBounds::default()` is the how-to default (`max_services 4`,
   `max_items 12`, 60-day staleness), not the wave T skeleton's invented
   numbers. The guard requires the shared kernel and the how-to path to select
   the same sources; two different defaults would be two different walks again.

**L3 declares `need_kinds` on every registry source, not only the seven the
plan lists.** `how_to_role` was the only axis a selector had, so
`select_sources(NeedKind::Procedure, …)` can only reproduce today's how-to
selection if every source with a contributing `how_to_role` also declares
`procedure`. The lexical tier joins `external_trusted` with
`how_to_role none`, which is what keeps it out of the procedure walk while
making it reachable for a concept need.

- [x] **L1 — Refactor guard first.** Add `tests/unit/issue_1138_source_walk_parity.rs`
      asserting today's `how_to_guide` output equals
      `tests/fixtures/issue-991/expected-guides.json` when replayed from
      `tests/fixtures/issue-991/source-cache`. Green before any refactor.
- [x] **L2 — Extract the kernel.** Create `src/source_walk.rs` with `NeedKind`,
      `LookupBounds`, `CaptureExtractor`, `WalkSourceOutcome`, `WalkOutcome`,
      `select_sources`, `walk_sources`, moving `Walk`, `capture_service`,
      `skipped_sources` and the depth/page accounting out of `src/how_to_guide.rs:296-388,527-…`.
      `GuideBounds` becomes `pub use source_walk::LookupBounds as GuideBounds`.
      L1 must stay green byte-for-byte; `src/how_to_guide.rs` drops below 900 lines.
- [x] **L3 — Second selection axis in the registry.** Add `NeedKind` and
      `SourceRecord::need_kinds` to `src/seed/sources.rs`; add `sources_for_need_kind()`;
      add `need_kinds`, `service_group`, `settings_key`, `extractor`, `api_language`
      to the lexical tier in `data/seed/sources-registry.lino`; `select_sources`
      dispatches on `NeedKind`. Test: dictionary-before-encyclopedia ordering.
- [ ] **L4 — Settings surface.** Two rows in
      `src/web/app/main.jsx:1063-1068`, i18n labels, and the settings-parity test
      extended to six keys. Test: `a_settings_opt_out_silences_a_dictionary_and_is_reported_as_disabled`.
- [x] **L5 — Extractors.** `src/concept_lookup.rs` with `ConceptSense`,
      `SenseExtractor` and the four registry-bound extractors
      (`wiktionary_entry_v1`, `wordnet_sense_v1`, `mediawiki_summary_v1`,
      `wikidata_entity_v1`), each reading either live JSON or the committed
      `data/cache/**/*.lino` projection. Tests: one per extractor against a committed
      capture.
- [x] **L6 — Capture the fixtures.** `examples/issue_1138_concept_lookup_capture.rs`;
      commit `tests/fixtures/issue-1138-b1/source-cache/` and `capture-manifest.lino`
      for both held-out words in every language the sources actually serve. Record
      the honest coverage in the manifest — a language with no capture gets no row.

      **The measured coverage, from the live run on 2026-09-16.** Ten
      `(language, surface)` pairs asked, **five answered**, 7 senses:

      | language | isogram | lipogram |
      | --- | --- | --- |
      | en | 2 (wordnet, wikipedia) | 2 (wordnet, wikipedia) |
      | ru | 0 — `изограмма`: wikipedia `no_entry` (HTTP 404) | 1 (wikipedia) |
      | hi | 0 — unserved | 0 — unserved |
      | zh | 0 — unserved | 0 — unserved |
      | es | 1 (wikipedia) | 1 (wikipedia) |

      No row was written for an unserved language and no gloss was fabricated.
      The offline replay reproduces exactly these five pairs and these seven
      senses, so `expected-senses.json` is a record of what services published
      and not of what the plan hoped for.

      **The decision this leaf carried, and its reason.** `wiktionary` — the
      registry's Free Dictionary API endpoint — answers `HTTP 522` for *both*
      held-out words while answering `mass` and the lemmas the committed
      `data/cache/wiktionary/en/` corpus was built from. The word this plan is
      judged by is a word that endpoint does not have. Rather than change the
      held-out words (which would destroy the "absent from every seed file"
      property the corpus depends on),
      `an_unknown_word_resolves_to_a_licensed_sense_with_exact_provenance` now
      asserts **the first source the registry declares that actually answered**,
      with that source's exact provenance — id, url, license and content id —
      pinned. Registry ordering is what decides it, so changing the order
      changes the expectation with it. The reason is written at the test.

      **Three changes this leaf forced, recorded rather than silent.**
      1. `wordnet`'s registry `api` is now `https://en-word.net/api/lemma/{lemma}`.
         The declared `/lemma/` path 303-redirects to a 33 KB HTML page whose
         synsets only a browser can read; `/api/lemma/` is the same site's own
         machine-readable endpoint for the same data under the same licence.
      2. `wikinews` declares `need_kinds (evidence)`, not `(concept)`. A news
         wiki records what happened; it does not say what a word means. Because
         original journalism outranks every dictionary on the trust axis it had
         taken the first of the four consultation slots for *every* concept
         need and pushed a dictionary out.
      3. `FetchError::HttpStatus` exists, and a 404 no longer speaks for a
         service. The accessibility cache treats an unreachable service as a
         seven-day fact; one absent Russian article was therefore blanking
         Wikipedia for every language and every subject, and the first capture
         run measured exactly that. `page_title` was cutting `लिपोग्राम` into
         two words for the same class of reason — a Devanagari virama is not
         `char::is_alphanumeric` — and now splits only on whitespace and ASCII
         punctuation.
- [x] **L7 — `lookup_surface` and `RegistryConceptLookup`.** The public entry plus the
      `UnknownConceptLookup` impl, over `walk_sources`. Tests:
      `an_unknown_word_resolves_to_a_licensed_sense_with_exact_provenance`,
      `a_lookup_that_finds_nothing_reports_every_consulted_source_and_no_gloss`,
      `an_offline_run_replays_the_committed_captures_without_any_transport_call`.
- [ ] **L8 — Per-word needs.** `unresolved_surfaces` in
      `src/coding/concept_discovery.rs`; the lookup is consulted for every unresolved
      surface, not only when the whole sentence is unmatched; `DiscoveryBounds::max_words`.
      Test: `a_partially_understood_sentence_still_asks_about_its_unresolved_words`.
- [ ] **L9 — Evidence becomes a part.** Provenance fields on `ConceptEvidence`;
      `concept_candidate`; `candidate_kind_rank` gains `"concept_sense"`; `NoLookup`
      made `pub`. Tests: `retrieved_evidence_becomes_a_candidate_part_the_composer_can_read`,
      `concept_senses_rank_below_retrieved_implementations`,
      `a_sense_is_quoted_and_attributed_and_never_inlined_into_generated_code`.
- [ ] **L10 — Wire the coding path.** `discover_and_compose` builds the lookup and
      calls `discover_with_lookup`. Test:
      `held_out_unknown_word_tasks_share_one_concept_map_identity_in_five_languages`.
- [ ] **L11 — Wire the universal loop.** Replace `requires_external_lookup`
      (`src/solver_helpers/mod.rs:104-111`) with `unresolved_surfaces_present`;
      `record_external_search` performs the lookup and returns senses; delete
      `policy:no_fetch_capability`; rank senses in `answer_unknown_prompt`. Tests:
      `tests/unit/issue_1138_universal_loop_lookup.rs` (all three).
- [x] **L12 — Sense ledger.** `src/concept_sense_ledger.rs`; forget/rediscover and
      tamper-rejection tests.
- [ ] **L13 — Browser parity.** `formal_ai_worker_source_walk.js`,
      `formal_ai_worker_concept_lookup.js`, two `data/meta/worker-line-budget/` files,
      `examples/issue_1138_concept_lookup_parity.rs`,
      `tests/fixtures/issue-1138-b1/expected-senses.json`,
      `tests/web/issue-1138-concept-lookup.test.mjs`.
- [ ] **L14 — Five-language outcome prose.** `data/seed/meanings-concept-lookup.lino`
      (6 meanings + 30 responses); the language-coverage gate must report
      `OK … en, ru, hi, zh, es`.
- [ ] **L15 — HTTP surface.** `tests/integration/issue_1138_concept_lookup_http.rs`,
      three tests through a real server process.
- [ ] **L16 — Grounded recipe.** `data/meta/concept-lookup-recipe.lino`,
      `tests/unit/specification/concept_lookup_meta_algorithm.rs`, and the
      `coding_discovery_step_understand` record in
      `data/meta/coding-discovery-recipe.lino`.
- [ ] **L17 — Ledgers and docs.** Requirement shard
      `docs/requirements/issue-1138-live-concept-lookup.md`,
      `rust-script scripts/assemble-requirements.rs --write`, traceability rows,
      `docs/benchmarks.md` corpus section, `docs/meta-algorithm.md` section, VISION
      and ROADMAP edits from the next section, and the honest measured numbers.
- [ ] **L18 — Retire the adapter.** Once plan 02 and plan 04 call `SourceLookup`
      directly, delete `UnknownConceptLookup`, `NoLookup` and
      `RegistryConceptLookup`; `discover_with_lookup` takes `&mut dyn SourceLookup`.
      **reconciled (plan 00 §9 X2): this is the last leaf of the whole plan set.
      Plan 02 L14 was amended to take the lookup from its caller as
      `&mut dyn SourceLookup` rather than constructing a second implementation,
      so nothing blocks this leaf except its own ordering.**
      One trait for retrieval in the whole tree. Coordinated with plan 09's ratchet;
      deferred to last on purpose, because it changes a signature
      `tests/unit/coding_discovery/concepts.rs` pins and must not share a commit
      with a behaviour change.

## Docs to update

The exact quoted statements and their replacement text moved to plan 11's
findings table on 2026-09-16, so there is one docs authority and no document
is described in two places (plan 00 §8). This plan's entries are rows
**D156-D273** of
[`11-docs-consistency-audit.md`](11-docs-consistency-audit.md) §"Issue #1138
plan doc replacements", and plan 11's leaves apply them after the ledger rows
they cite exist (plan 00 §7).

| row | document |
| --- | --- |
| D156 | `docs/meta-algorithm.md` |
| D157 | `VISION.md` |
| D158 | `ROADMAP.md` |
| D159 | `docs/requirements/issue-1138-live-concept-lookup.md` |
| D272 | `docs/requirements-traceability.md` |
| D273 | `docs/benchmarks.md` |

Any further document this plan's implementation touches is added as a new
plan 11 row, never as a second copy here.

## Risks and open questions

1. **The refactor in L2 touches a fixture-pinned, merged feature.** Mitigated by
   writing L1 first and requiring byte-identical output against the committed
   `expected-guides.json`. If the extraction cannot be made byte-identical, the
   honest response is to stop and record why, not to regenerate the fixture.
2. **The Free Dictionary API serves English well and other languages unevenly.** The
   plan's answer is `api_language` plus an honest `unbound_template` outcome, and
   routing ru/hi/zh/es to Wikipedia's per-language REST summary endpoint. The open
   question is whether a Wikipedia summary is a *definition* for these purposes; the
   tier derivation (`PrimacyChain::derive_tier`, `src/seed/sources.rs:168`) says it is
   editorial synthesis one hop from cited sources, which is weaker than a dictionary
   and must be reported as such per sense, not silently equated.
3. **Wiktionary CC BY-SA 3.0 is share-alike.** Quoting a gloss with attribution in an
   answer is fine; the open question is whether the *sense ledger* — a derived,
   committed-adjacent artifact — is a derivative work. This plan keeps the ledger in
   the ignored cache directory and stores the locator and identity rather than the
   gloss, per `docs/case-studies/issue-710/plans/06:232-239`, which side-steps the
   question. If a future change wants glosses in `data/`, legal review first.
4. **Tokenising "each word" in zh and hi.** `engine::normalize_prompt` is
   whitespace-oriented; Chinese has no spaces. `unknown_surfaces` must therefore fall
   back to the longest seeded-lexeme match plus the residual span, and the residual
   span may be a phrase rather than a word. The honest position is that the unit is a
   *surface*, not a word, and the zh case is expected to produce a longer surface —
   which the test must assert rather than paper over.
5. **Lexical relevance, again.** #991's recorded limitation ("Relevance is lexical:
   `matches_task` accepts a … question whose title literally names the topic") applies
   to the `technical` role here too. This plan does not solve ranking; it inherits the
   limitation and records it. Tightening belongs with #709.
6. **A lookup can succeed and still not help.** Resolving "isogram" to "a word with no
   repeated letter" only produces a program if the gloss re-enters `structures_for`
   and matches seeded meanings. If it does not, the need is `satisfied` for
   understanding and `blocked` for composition — two different statuses that the
   `ConceptNeed` type currently conflates into one `status: String`. Open question:
   split it into `understood` and `composable`, or let B2 own that split. This plan
   leaves it as one field and records the ambiguity rather than inventing a second
   field B2 may want shaped differently.
7. **Does the loop's new trigger fire too often?** `unresolved_surfaces_present` will
   be true for many ordinary prompts containing a proper noun. The bound is
   `LookupBounds::max_services` and the per-service accessibility cache
   (`src/service_accessibility.rs:40`, seven-day floor), not a request budget. Whether
   that is sufficient in practice is measurable only after L11; the first measurement
   must be recorded even if it is embarrassing.
8. **Option C remains the destination.** Nothing here forecloses moving selection and
   ordering into recipe data; the registry fields this plan adds are exactly the data
   such a move would read. The open question is whether `recipe_interpreter`'s
   event-parity obligation (R343) can absorb retrieval at all, which needs its own
   analysis before B9 is closed.
