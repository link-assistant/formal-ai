# Architecture: Translation Between Languages

Part of the [architecture overview](../../ARCHITECTURE.md) (§10). Section
numbers match the overview's table of contents; paths are relative to the
repository root.

## 10. Translation Between Languages

Because formalization is language-independent (a Wikidata Q-id is the same
whether it is named in English, Russian, Hindi, or Chinese), translation is
not a separate model — it is a re-rendering of the same formalized links network
into the target language's labels.

```text
formalization_007
  subject_q "wikidata:Q170978"
  predicate_p "wikidata:P31"
  object_q "wikidata:Q14660"

render_en  -> "QuickSort is an instance of sorting algorithm"
render_ru  -> "Быстрая сортировка — это разновидность алгоритма сортировки"
render_hi  -> "क्विकसॉर्ट एक छँटाई एल्गोरिथ्म का उदाहरण है"
render_zh  -> "快速排序是一种排序算法"
```

The same machinery translates between natural and programming languages.
When the formalizer recognizes the input as a programming-language
construct (Rust function, Python class, SQL query), it lifts the construct
to a formalized links network the same way and re-renders into any other language
the renderer supports. The renderer is a transformation rule (Section 9):
the input is `(links_network, target_language)`; the output is rendered text.

### 10.1 Formalize → Meaning → Deformalize Pipeline

Translations flow through a generalized pipeline that resolves any
surface pair via existing public knowledge bases — Wiktionary's
translation tables and Wikidata's lexeme/sense graph — instead of a
hand-written list of phrase pairs:

```text
formalize(surface, source_lang)
  -> Wiktionary translation_blocks(source_edition, page)
  -> sense_blocks ⨯ candidates(target_lang)
meaning(source_lexeme, target_lexeme)
  -> Wikidata SPARQL: ?lexeme ontolex:sense ?sense .
                      ?sense  wdt:P5137     ?meaning .
  -> MeaningId (priority: Q-item > sense > Wiktionary page)
deformalize(meaning, target_lang)
  -> winning candidate by round-trip confirmation
match_source_formatting(target, source)
  -> mirrors the source fragment's leading capitalization
     and terminal punctuation
```

The pipeline lives under `rust/src/translation/`:

- `rust/src/translation/http.rs` — `HttpClient` trait. The default
  transport shells out to `curl` so the crate has no TLS dependency.
- `rust/src/translation/cache.rs` — `CachedHttpClient` persists raw
  response bodies under `data/translation-cache/<fnv1a>.body` with a
  sibling `.url` file. Online mode is gated by `FORMAL_AI_LIVE_API`;
  offline mode reads only from the committed cache, so every test
  runs deterministically.
- `rust/src/translation/wiktionary.rs` — parses `{{t|...}}` / `{{t+|...}}`
  / `{{tt|...}}` / `{{перев-блок|...}}` / `{{翻譯-頂}}...{{翻譯-底}}`
  templates and splits polysemous entries by `{{trans-top|gloss}}`
  blocks.
- `rust/src/translation/wikidata.rs` — runs the canonical lexeme join
  (`ontolex:sense` / `wdt:P5137`) so two surfaces share a stable
  `meaning:` id regardless of which language we observe first.
- `rust/src/translation/meaning.rs` — `MeaningId` selector.
- `rust/src/translation/pipeline.rs` —
  `TranslationPipeline::translate(surface, source, target)`.
- `rust/src/translation/formatting.rs` — `match_source_formatting` keeps
  lowercase phrases lowercase, capitalizes targets when the source
  fragment is capitalized, and only emits a terminal `? ! .` (or the
  Chinese full-width equivalents `？ ！ ．`) when the source carried
  one.

The meaning ID, source language, and target language remain in
`evidence_links` so the Links Notation trace is still inspectable;
the user-facing body is just the deformalized surface.

### 10.2 Resolution Order and Browser Fallback

`TranslationPipeline::translate` resolves any surface pair by:

1. Fetching the source-edition Wiktionary page and parsing its
   `{{trans-top}}` blocks for `target_lang` candidates.
2. Falling back to the `/translations` subpage when the main page
   omits translations (common for high-traffic English entries).
3. Falling back to the target-edition Wiktionary page in reverse when
   the source edition is sparse (typical for ru → en).
4. Generating phrasal variants (e.g. dropping Russian "у тебя",
   "у вас", "у меня" infixes) when the literal page does not exist.
5. Selecting the best sense block by round-trip confirmation rate —
   for each candidate, count how many target-edition pages list the
   source surface as a translation. The block with the most confirms
   wins.
6. Upgrading the meaning id to a Wikidata Q-item or sense id when the
   lexeme join returns one.

Issue #526 promotes that round-trip confirmation from a ranking heuristic into
the testable quality contract. The active matrix requires
language-to-meta-to-same-language survival for every supported natural language
and a directed pair round trip across en, ru, hi, and zh. Code translation uses
the same shape and the same anti-`N * N` rule: `translate_program` never matches
on a `(source, target)` pair. It formalizes source code into a language-neutral
`CodeMeaning` (`rust/src/solver_helpers/code.rs::formalize_code_meaning`) and renders that
meaning into the target (`render_code_meaning`), so adding a language is one
formalizer plus one renderer, not a new pair. Because the source language never
enters the formalizer, any pair — including ones with no hardcoded arm, such as
Python -> JavaScript — shares one meaning, and Rust <-> JavaScript returns to the
same `meaning:` link.

Issue #890 extends that meta-language path from simple functions to formal
proofs. `FormalProof` (`rust/src/proof_program.rs`) owns interval bounds, inclusive
flags, satisfiability, and the integer witness without owning any prose or
program syntax. The number-constraint solver emits its canonical statement;
`formalize_code_meaning` recovers `CodeMeaning::FormalProof`, and
`render_code_meaning` projects the same value into a complete Rust or Python
program. The presentation templates live in
`data/seed/proof-program-templates.lino`; native and browser renderers only
bind proof fields into the selected target template. Each program checks the
witness at runtime before printing it. This keeps the architecture at
one proof formalizer plus one data-defined projection per target, never one
implementation per natural-language/programming-language pair. The browser
compiles the same presentation-independent core into
`js/wasm-worker/src/proof_translation_worker.rs`; JavaScript only extracts
the quoted statement and target before crossing the WASM boundary. Worker 13
supplies the script-aware target-language alias match used by Chinese requests.

Issue #917 makes formal languages first-class projections of the same semantic
layer. `rust/src/translation/formal_statement.rs` formalizes a natural statement as
a Wikidata-grounded predicate plus role-qualified subject and object meanings,
then renders that value through `data/seed/formal-language-projections.lino`.
The catalog owns formal aliases and statement templates as well as each natural
language's word order and canonical relation surface. Consequently a new
syntax adds one projection rather than translators for every natural/formal
pair. The inverse lookup uses both Wikidata ID and semantic role because one
identifier can legitimately ground multiple lexicon meanings.

The browser compiles the corresponding catalog interpreter into
`js/wasm-worker/src/formal_statement_worker.rs` and loads the same
projection and Wikidata seed files. JavaScript only recognizes the translation
request and crosses the WASM boundary. Both surfaces expose the stable meaning
`statement:P31(Q89,Q3314483)`, so the issue #526 round-trip contract now covers
natural -> FOL -> natural without a direct pair path. Formal output is not
executed as code, leaving issue #917 independent of E69's execution gate.

Issue #921 closes the external orchestration loop without introducing another
adapter. In one direction, Hive Mind's public `solve` parser selects its shipped
Agent executor, which starts the native Agent CLI against the candidate Formal
AI server. In the other, Formal AI's existing `agent run` adapter starts that
same external CLI and records its exit, verified workspace delta, native
session, and hash-chained orchestration events. The release gate saves exact
fixture commits and replays the canonical session; a child-process failure is a
failed gate in both directions. CI prepares the public Hive Mind command but
suppresses the GitHub-writing portion, then calls the same production executor
directly, keeping the integration real without mutating issue state. A scoped
permission-response shim lets Hive Mind 2.12.2 reach its no-write preparation
exit under the workflow's read-only token; it is absent from execution. Lazy
Hive Mind helper installs use an isolated, writable npm prefix owned by the
gate, rather than relying on machine-global package permissions. Hive Mind's
identity preflight is satisfied with repository-local config in the disposable
prepare clone plus process-local config on the prepare-only command, never with
runner-global Git config. The disposable clone materializes its candidate HEAD
as a local branch as well, making Hive Mind's current-branch preflight
independent of GitHub Actions' detached checkout shape. A matching local
`origin/main` ref lets Hive Mind create its temporary solution branch without
fetching or changing the candidate commit.

The Rust pipeline is the canonical implementation. The browser worker
(`js/worker/formal_ai_worker.js`) cannot reach Wiktionary or Wikidata
directly because of browser CORS restrictions, so it keeps a small
offline phrase table as a CORS-safe fallback for the GitHub Pages
demo. The fallback returns the same `[<lang>] <surface>` placeholder
the Rust pipeline uses when a lookup misses, so the contract stays
identical across surfaces.
