# Controlled natural language output — issue 1584

Implementation evidence for [PR 1585](https://github.com/link-assistant/formal-ai/pull/1585).
The [requirement-to-evidence table](acceptance.md) maps each acceptance criterion to its
code and tests.

## Design

The engine is generalized, not a set of fixed answer strings. Each language has one
declarative grammar in `data/seed/cnl-language-<id>.lino`. The same description drives
three directions:

- **realization**: graph → text, for each template style and term register;
- **parsing**: text → graphs, the exact inverse of realization, plus declared fillers,
  typos and input-only words;
- **validation**: the profile checks that judge a text.

The shared policy in `data/seed/cnl-output.lino` declares the rest:

- the profiles and their checks;
- the frames, including first-order logic for ACE;
- the facts;
- the localized directives, switches and dictionary notices.

The JavaScript engine is `js/agentic/crate/cnl_grammar.mjs` plus `cnl_output.mjs`. The
browser and desktop worker use generated copies of it. The native port in
`rust/src/cnl_output/` reads the same seeds. Both solvers enter the engine through one hook:
`controlledSolve` in `cnl_output.mjs` and its twin `controlled_solve` in
`rust/src/cnl_output/controlled.rs`.

Grammar keys that a language needs only for itself:

| Key | Example | Effect |
| --- | --- | --- |
| `clause-close` | Russian `","` | Closes a multi-word slot form before the next word |
| `sentence-space` | Chinese `""` | Removes the space after a sentence end |
| `input` register | Chinese 鼠, Hindi मूषक | Shared polysemous words that are parsed but never produced. They make a question ambiguous, so the engine asks. |

Spanish matches question templates even without "?". Its grammar therefore puts the
verb first in questions, so a question never reads as a statement.

## The generalized meta test

`rust/tests/web/cnl-grammar-matrix.test.mjs`, with the enumeration in
`rust/tests/web/support/cnl-matrix.mjs`, checks properties instead of examples. Each
property must hold for every graph, style, register, language and configuration:

1. **inventory**: 65 NSM primes, every term and every output style in each language;
2. **uniqueness**: no form names two concepts in one register;
3. **coverage**: every output style can realize every graph;
4. **inversion**: every realization parses back to its own graph;
5. **compliance**: 26,180 projections (5 languages × 68 graphs × 11 profiles ×
   7 densities), each validated against an oracle over the whole candidate domain;
6. **round trip**: free-form → Links Notation → max-precision → primes →
   cnl-default keeps the graph;
7. **dialogue**: every phrasing of a question gets one compliant answer, and an
   ambiguous phrasing asks a compliant numbered clarification.

Compliance results:

- 17,076 projections are compliant.
- The other 9,104 are combinations where no candidate can satisfy both the named
  standard and a primes or top-N vocabulary.
- Those 9,104 must equal the reviewed ledger `data/parity/issue-1584-cnl-infeasible.json`
  exactly.
- Standard and max-precision density are feasible for every profile in every language.
  `minimal-english` is feasible under every density, and `cnl-default` under top-200
  and above.

Frequency tiers use the NSM primes as their floor, so any `top-N`, including `top-1`,
decomposes into primes.

## Parity and runtime surfaces

| Surface | Test | What it checks |
| --- | --- | --- |
| JS engine (fixture generator) | `rust/tests/web/cnl-parity.test.mjs` | Regenerates each `data/parity/issue-1584-cnl/<id>.json` and compares it byte for byte |
| Rust engine | `rust/tests/unit/cnl_parity.rs` | Reproduces every fixture projection digest and dialogue turn |
| Browser/desktop worker | `rust/tests/web/cnl-solver-dialogue.test.mjs` | Every question case and one paraphrase per task shape in each language; #962 prompts keep their answers; deterministic density switch |
| HTTP API | `rust/tests/web/cnl-output-http.test.mjs` | Request options and directives, then "Is a mouse an animal?" (en and ru) gets a clarification, and "1" picks a reading |

The fixtures hold 2,259 dialogue cases:

| Language | Cases | Clarifications |
| --- | --- | --- |
| en | 432 | 52 |
| ru | 443 | 52 |
| hi | 485 | 52 |
| zh | 489 | 45 |
| es | 410 | 48 |

## Reproduction

Run each file on its own, as AGENTS.md requires:

```sh
timeout 600 node --test --test-concurrency=1 rust/tests/web/cnl-grammar-matrix.test.mjs
timeout 600 node --test --test-concurrency=1 rust/tests/web/cnl-parity.test.mjs
timeout 600 node --test --test-concurrency=1 rust/tests/web/cnl-solver-dialogue.test.mjs
timeout 600 node --test --test-concurrency=1 rust/tests/web/cnl-output.test.mjs
timeout 600 node --test --test-concurrency=1 rust/tests/web/cnl-output-http.test.mjs
cargo test --manifest-path rust/Cargo.toml --test unit cnl_
```

Experiment scripts:

| Script | Purpose |
| --- | --- |
| `check-language.mjs en,ru,hi,zh,es [--ledger]` | Property counterexample counts for a grammar draft, optionally with only some languages loaded |
| `feasibility.mjs --write` | Regenerates the infeasibility ledger |
| `parity-fixture.mjs [--write]` | Checks or rewrites the per-language fixtures |
| `dialogue-probe.mjs "<prompt>" ...` | Prints the controlled turns of a dialogue |
| `probe-tiers.mjs` | Prints answers and compliance reports under several standards and densities |
| `rendered-answers.mjs` | Prints the exact engine answers the solver tests pin (first answer, max-precision switch, task canonical prompt) |
| `worker-answers.mjs` | Prints the browser-worker answers for the same prompts, including that a bare `max-precision` is not a directive |

## Sources and limits

- **NSM exponents** follow the published 65-prime tables (English, Russian, Chinese,
  Spanish and Hindi; [NSM resources](https://nsm-approach.net/resources)).
- **Basic English** uses Ogden's public-domain 850-word list.
- **STE, Special English and Globish** lists are small project subsets for this
  fragment, because their full dictionaries are not redistributable here.
- **ACE** follows the [ACE 6.6 construction rules](https://attempto.ifi.uzh.ch/site/docs/ace/6.6/ace_constructionrules.html).
  Every ACE frame declares its first-order reading.
- **Plain Language** follows the reader-oriented rules of
  [ISO 24495-1](https://www.iso.org/standard/78907.html) as concrete sentence checks.
- **One grammar drives both directions**, following the multilingual precedent of
  Grammatical Framework.
- **The fragment is finite.** Text outside it is never rejected or rewritten; it goes
  to the ordinary solver unchanged.
