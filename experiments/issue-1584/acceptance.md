# Issue 1584 requirement-to-evidence inventory

Source: [issue 1584](https://github.com/link-assistant/formal-ai/issues/1584) and the
maintainer's PR comment. That comment asks for a full implementation, a generalized
("meta") check of inputs and outputs in every configuration, and repository-wide tests.
A01–A07 are the seven acceptance checkboxes, in order. Every row names the code and
the test that verify it. Commands are in [README.md](README.md).

The domain is the finite controlled fragment declared in
[`data/seed/cnl-output.lino`](../../data/seed/cnl-output.lino):

- 8 facts with their converses, both polarities and both moods (64 graphs), plus 4
  dialogue graphs;
- one declarative grammar per language in `data/seed/cnl-language-<id>.lino` (en, ru,
  hi, zh, es).

Text outside the fragment never fails. It stays opaque and goes to the ordinary solver
unchanged.

| Acceptance | Requirement | Implementation | Verification |
| --- | --- | --- | --- |
| A01 | Accept any free-form prose, slang, typos and CNL dialects | `utteranceGraphs` parses each grammar's input styles (plain, basic, ste, ace, gellish, sbvr, eprime, primes, dense, colloquial) with declared fillers and typo maps. Unparsed text stays opaque. | Matrix property *inversion*: every style × register realization parses back to its graph in all five languages. The dialogue suite checks that the solver's earlier #962 prompts keep their answers. |
| A01 | Everyday and CNL variants map to the same Links Notation | One graph key per meaning. `outputLinksNotation` serializes the graph, not the words. | Matrix properties *inversion* and *dialogue*: every phrasing of one question gets one answer. |
| A02 | `cnl-default` in every language is the ASD-STE100 / Plain Language / ACE / Minimal English intersection | The `cnl-default` profile uses styles `ace\|plain\|ste` and registers `ste\|minimal\|forms`. It requires every check those standards share. | Matrix *compliance*: all 68 graphs × 5 languages are compliant under `cnl-default`, at standard and max-precision. |
| A02 | Simple unambiguous words, short active SVO sentences, no pronouns, idioms or filler | Checks `unambiguous-vocabulary`, `explicit-references`, `active-voice`, `plain-language-rules` and `ste-writing-rules`. Pronoun and filler lists are declared per language. | *Uniqueness*: no form names two concepts in one register. *Compliance* reports every check outcome, so none goes unchecked. |
| A02 | English, Russian, Hindi, Chinese and localized equivalents (Spanish) | Five grammars with the same 65 NSM prime ids, every term and every output style. Russian adds clause-close punctuation, Chinese drops sentence spacing, Spanish puts the verb first in questions. | Matrix *inventory* for every language. `check-language.mjs en,ru,zh,es,hi` passes with all five loaded together. |
| A03 | `asd-ste100`, `ace`, `gellish`, `sbvr`, `plain-language`, `minimal-english`, `e-prime`, `basic-english`, `special-english`, `globish` | One profile per standard. Each has its styles, registers, word lists and checks: ACE grammar and FOL interpretation, Gellish relations, SBVR rule forms, E-Prime copula filter, and others. | Matrix *compliance*: every profile is compliant for every graph at standard density, in every language. |
| A03 | `primes`, `top-100/200/500/1000`, any `top-N`, `standard`, `dense`/`max-precision` | `frequencyTier` accepts any positive N. NSM primes are the floor below every tier: a concept outside the tier is decomposed into primes. | Matrix rows for 7 densities. A named standard whose own grammar cannot be written in primes under a primes or top-N density is listed in the reviewed [infeasibility ledger](../../data/parity/issue-1584-cnl-infeasible.json). The response names the failing checks; it does not claim compliance. |
| A03 | CLI, lino-arguments, HTTP API, desktop and browser UI, WASM worker | Native CLI flags and env defaults (`rust/src/main.rs`, `rust/src/protocol/cnl.rs`). JS server fields (`js/server/cnl-output.mjs`). UI settings (`js/app/cnl-settings.jsx`, `js/preferences/controlled-output.js`). Solver hook `controlledSolve` in `js/agentic/crate/cnl_output.mjs` (twin of `controlled_solve` in `rust/src/cnl_output/controlled.rs`), called from `js/worker/formal_ai_worker_solve.js`. Its draft (answer, clarification or paraphrase) always goes through the recursive procedure (`metaRunProcedure` / `run_meta_procedure`). | `cnl-output-http.test.mjs` covers the HTTP dialogue, including a clarification and its numbered reply. `cnl-solver-dialogue.test.mjs` runs the worker solver on every fixture case. Rust protocol unit tests cover the CLI and API. `issue-1550-procedure-census.test.mjs` checks that neither solver entry can exit before the procedure. |
| A03 | Apply the policy in formalize → summarize → deformalize | `formalizeOutput` → `outputLinksNotation` → `renderOutput`. A meaning re-renders from its graph. | Matrix *round trip* for every graph in every language. |
| A04 | `max-precision` uses the fewest exact words and shows a dictionary notice | `precisionPairs` searches every style with the precise register. `lexical-minimum` compares the word count with the minimum over the whole candidate domain. A localized notice comes first. | Matrix *compliance* (`lexical-minimum` is verified, not assumed). The HTTP test checks that the notice is not empty. |
| A05 | Explicit requests: "Answer in ACE", "Use ASD-STE100", "Respond in E-Prime" | Localized `directive` prefixes per language in the shared policy. | `cnl-output.test.mjs` directive tests. |
| A05 | Simplify, primes only, top 200, fewest exact words, max-precision | Localized `switch` and `frequency` phrases re-render the previous answer from its graph. | `cnl-output.test.mjs`. `cnl-solver-dialogue.test.mjs` and its Rust twin in `rust/tests/unit/cnl_output.rs` check that "Use max-precision." and "Rewrite in max-precision mode" re-render the previous answer to the same exact text every time. A bare `max-precision` is not a directive and goes to the ordinary solver. |
| A05 | Deterministic switching and paraphrasing | Stable candidate order and pure functions of (graph, language, configuration). | Matrix test "in-dialog switching is deterministic for every configuration" (11 profiles × 7 densities). |
| A06 | Ambiguous prompts get a CNL clarification restating each candidate | When a prompt parses to more than one graph (for example a bare polysemous "mouse"), the engine renders each candidate precisely. It asks for a number, and a numbered reply answers that reading. | Matrix *dialogue*: every ambiguous phrasing must clarify, and the clarification must be compliant. The fixtures hold 249 clarification cases (en 52, ru 52, hi 52, es 48, zh 45). The HTTP test covers this end to end in English and Russian. |
| A06 | Ask before acting on an ambiguous task | A task paraphrase has a canonical prompt. The worker solves it when the original prompt reached no intent or a different one. | `cnl-solver-dialogue.test.mjs` covers the arithmetic and coding task shapes (sum, difference, product, reverse-text, list-maximum, list-sum, factorial) in every language. |
| A07 | Free-form, colloquial, primes, STE, ACE, Gellish, SBVR, plain, Basic, E-Prime and dense input corpus | The grammar templates per style are the corpus. The fixtures enumerate each question in every style and in the precise, forms and input registers. | Per-language fixtures `data/parity/issue-1584-cnl/<id>.json` hold 2,259 dialogue cases. |
| A07 | Every input style × every output standard and tier | `complianceMatrix` enumerates 5 languages × 68 graphs × 11 profiles × 7 densities = 26,180 projections. It also validates the whole candidate domain as an oracle. | 17,076 projections are feasible and compliant. The other 9,104 match the reviewed ledger exactly (test "compliance … matches the ledger"). |
| A07 | Same internal graph for every input style | `utteranceGraphs` + `graphKey`. | *Inversion* and *dialogue*: one answer per question across all styles. |
| A07 | free-form → Lino → dense → primes → cnl-default round trip | `languageProperties().roundTrip`. | Zero drift for all 64 fact graphs in all five languages. |
| A07 | Rust and JS/WASM parity | Native grammar engine in `rust/src/cnl_output/grammar.rs` with the same seeds. The generated worker modules are the JS/WASM side. | `cnl-parity.test.mjs` regenerates every fixture from the JS engine. `rust/tests/unit/cnl_parity.rs` checks that the Rust engine reproduces the same projections and dialogue. |

## Recorded limits

- **Word lists are project-authored.** The full ASD-STE100 dictionary, the Special
  English list and the Globish list are not redistributable here. Each grammar holds a
  small subset that covers its fragment, as the grammar headers state. Basic English
  uses Ogden's public-domain 850 words. The NSM exponents follow the published
  65-prime tables. Frequency tiers are upper rank bounds, not a corpus.
- **The fragment is finite.** Equivalence and compliance are proven exhaustively for
  the declared graphs. Prompts outside the fragment are not rewritten; they reach the
  solver unchanged, so input is never refused.
- **Some combinations are infeasible.** For example, ACE grammar written only in NSM
  primes cannot hold for every graph. The ledger lists these combinations with the
  checks that still fail, and the output reports them.
