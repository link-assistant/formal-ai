# Issue 538 — Online Research

External facts gathered while planning and implementing issue #538
("Make our meanings and words more detailed"). Each entry records what was
looked up, what was found, and how it informed the implementation in this PR.

## 1. Wikidata grammatical-number vocabulary

The issue asks whether `помидор`, `помидоры`, `томат` are *singular or plural*.
Wikidata already models exactly this distinction on lexeme **forms** via the
`grammatical feature` slot, so we reuse its item ids rather than invent our own.

- **grammatical number — `Q104083`**: "use of grammar in a language to express
  number" — the *category*. We ground the new `grammatical_number` facet-kind
  meaning in this item.
- **singular — `Q110786`**: the grammatical feature attached to a form that
  names one instance.
- **plural — `Q146786`**: the grammatical feature attached to a form that names
  more than one instance ("used when a word refers to multiple people/things").

Wikidata's lexicographical-data documentation describes *grammatical features*
as "a set of items describing a Form's grammatical role, such as plural
(Q146786) and singular (Q110786)", used on forms when a word has different
surfaces by number. This is the precedent our seed follows: every tomato
surface pins a `grammatical_number` facet whose target is a grounded
`singular`/`plural` meaning.

Sources:
- <https://www.wikidata.org/wiki/Q104083> (grammatical number)
- <https://www.wikidata.org/wiki/Q110786> (singular)
- <https://www.wikidata.org/wiki/Q146786> (plural)
- <https://www.wikidata.org/wiki/Wikidata:Lexicographical_data/Documentation/Lexeme_statements>
- <https://www.wikidata.org/wiki/Wikidata:Lexicographical_data/Glossary>

## 2. Wikidata lexemes and forms for the tomato synonyms

The issue's asymmetry ("for `помидор` we have plural, and for `томат` we
don't") maps onto two distinct Russian lexemes, each with its own inflection
table of forms (`L…-F…`), plus the English lemma:

- **English `tomato` — `L7993`** (noun): form `L7993-F1` (singular *tomato*,
  feature `Q110786`), form `L7993-F2` (plural *tomatoes*, feature `Q146786`),
  sense `L7993-S1`.
- **Russian `помидор` — `L3526`** (noun): form `L3526-F1` (nominative singular
  *помидор*), form `L3526-F3` (nominative plural *помидоры*), sense `L3526-S1`.
- **Russian `томат` — `L170542`** (noun): form `L170542-F1` (nominative
  singular *томат*), form `L170542-F7` (nominative plural *томаты*). This lexeme
  carries no published sense, which is why the `томат` surfaces are grounded to
  forms only, not to a sense id.

Wikidata documentation confirms the general model: "Forms are the grammatical
realization of a lexeme, including inflections and declensions, with the role of
each form denoted by grammatical features", and Russian is the language with the
most lexemes on Wikidata. The specific form ids above were read from the cached
lexeme JSON (`data/cache/wikidata/lexeme/L3526.json`,
`L170542.json`, `L7993.json`), which is the authoritative local copy this repo
tests against offline.

Sources:
- <https://www.wikidata.org/wiki/Wikidata:Lexicographical_data/Documentation/Languages/ru/modeling>
- <https://www.wikidata.org/wiki/Wikidata:Lexicographical_data/Documentation>
- <https://www.wikidata.org/wiki/Lexeme:L3526> (помидор)
- <https://www.wikidata.org/wiki/Lexeme:L170542> (томат)
- <https://www.wikidata.org/wiki/Lexeme:L7993> (tomato)

## 3. Reverse vs direct dictionary (meaning ⇄ word)

The issue frames the current data as a *reverse dictionary* (meaning → ways to
express it) and asks that words also *reference possible meanings* — i.e. a
*direct dictionary* — so the reference is bidirectional. This is the classic
onomasiological (concept→word) vs semasiological (word→concept) pair in
lexicography. The repository already auto-attaches a `denotation` facet on each
parsed word form (word → meaning), so the direct direction exists in the model;
this PR makes it explicit and *tested* for every tomato surface via
`WordForm::denotations()` and the `every_tomato_surface_denotes_the_tomato_meaning`
test, and lets a surface additionally denote grammatical values
(`singular`/`plural`) that are themselves grounded meanings.

## 4. Rust → WebAssembly (aspirational follow-up R7)

The issue's larger vision includes moving formal-worker logic out of hand-written
JavaScript into a WebAssembly worker compiled from Rust at build time, keeping JS
to UI interfacing only. Research confirms the standard toolchain:

- `wasm-pack build` compiles Rust to WebAssembly and runs `wasm-bindgen` to
  generate the JS wrapper module the browser loads; it can package the result as
  an npm module.
- `wasm-bindgen` bridges JS and Rust types (pass strings, catch exceptions,
  share slices — with a copy across the JS/Rust memory boundary for arrays).
- WASM is single-threaded by default; **Web Workers** provide parallelism, each
  worker owning its own WASM instance — matching the issue's "WebAssembly web
  worker" phrasing.

This confirms the follow-up is feasible with a mainstream toolchain and no
neural runtime. Note the repo **already** has a Rust→WASM demo worker
(`src/web/wasm-worker/src/lib.rs` → `src/web/formal_ai_worker.wasm`, issue #1
R16), so the remaining work is widening that surface to absorb the hand-written
JS workers under `src/web/worker/` rather than introducing the toolchain from
scratch. It stays a build-system programme tracked on the roadmap rather than
part of this data-focused PR.

Sources:
- <https://developer.mozilla.org/en-US/docs/WebAssembly/Guides/Rust_to_Wasm>
- <https://www.shuttle.dev/blog/2024/03/06/writing-wasm-rust>
- <https://surma.dev/things/rust-to-webassembly/>

## 5. code-architecture-principles

The issue asks to double-check we apply
<https://github.com/link-foundation/code-architecture-principles>. The principles
most relevant to this change are: prefer data over code for knowledge, avoid
duplication (DRY), keep a single normalized representation per link type, and
ground knowledge in real, verifiable sources. This PR follows them by (a) adding
grammatical detail as *data* in `.lino` seed rather than as Rust branches, (b)
reusing the existing `SemanticFacet` mechanism and the closed `FACET_KINDS`
vocabulary instead of a parallel structure, and (c) grounding every new meaning
and every surface in cached Wikidata items/lexemes/forms.
