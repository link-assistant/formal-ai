# Issue 917 Online Research

## Abstract And Concrete Syntax

- Grammatical Framework: <https://www.grammaticalframework.org/>

  GF separates shared abstract syntax from language-specific concrete syntax.
  Issue #917 applies that boundary to Formal AI's link substrate: one semantic
  statement is projected into natural or formal concrete syntax.

- GF Informath: <https://www.grammaticalframework.org/lib/doc/informath/>

  Informath is the issue's primary example of multilingual mathematical text
  sharing an abstract representation with formal proof-language notation.

## Controlled Natural Language

- Attempto Parsing Engine: <https://github.com/Attempto/APE>

  APE maps Attempto Controlled English into a discourse representation that
  can be translated to first-order logic. Its unambiguous-entry principle
  informed the exact seeded statement slice; no APE code was imported.

## Multilingual Grammar And Lexical Grounding

- Universal Dependencies: <https://universaldependencies.org/>
- Open English WordNet: <https://github.com/globalwordnet/english-wordnet>
- FrameNet: <https://framenet.icsi.berkeley.edu/>

  These projects are the issue's references for word-order, sense, and semantic
  role metadata. The implementation uses the repository's existing
  multilingual/Wikidata seed instead of copying those datasets.

## Repository Prior Art

- Issue [#526](https://github.com/link-assistant/formal-ai/issues/526) and PR
  [#635](https://github.com/link-assistant/formal-ai/pull/635) established
  round-trip survival and the prohibition on direct N-by-N translators.
- Issue [#890](https://github.com/link-assistant/formal-ai/issues/890) and PR
  [#911](https://github.com/link-assistant/formal-ai/pull/911) established
  seed-defined projection of a formal proof into program syntax and native/WASM
  parity.
- Issue [#914](https://github.com/link-assistant/formal-ai/issues/914) and PR
  [#915](https://github.com/link-assistant/formal-ai/pull/915) identified E70
  and compiled the broader formal-reasoning research inventory.

The repository study inspected the most recent relevant merged work (#911,
#915, #880, #794, and #635). All external references above are official project
sites or repositories; no external source code or prose was copied.
