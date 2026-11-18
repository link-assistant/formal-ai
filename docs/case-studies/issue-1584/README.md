# Controlled natural language output and clarification

Issue [#1584](https://github.com/link-assistant/formal-ai/issues/1584), implemented in
[PR 1585](https://github.com/link-assistant/formal-ai/pull/1585).

## Problem

Formal AI should answer in a selectable controlled natural language (CNL) and accept
any input unchanged.

- **Languages:** English, Russian, Hindi, Chinese and Spanish.
- **Standards:** the `cnl-default` intersection, ASD-STE100, ACE, Gellish, SBVR, Plain
  Language, Minimal English, E-Prime, Basic English, Special English and Globish.
- **Densities:** NSM primes, top-N word tiers, standard vocabulary, and max-precision
  with a dictionary notice.
- **Dialogue:** switching inside a dialogue must be deterministic, and an ambiguous
  prompt must get a CNL clarification.
- **Verification:** a cross-standard test matrix in Rust and in JS/WASM.

## Timeline

1. **First attempt.** It added configuration, request options, UI settings and
   localized directives. It then stopped at a lexical preview: one proposition with
   declared paraphrases, and no checks that could certify any standard. The earlier
   evidence archive in `raw-data/` keeps that state. It includes two E-Prime
   certificates that were withdrawn because they lacked independent evidence.
2. **Maintainer review.** The review asked for a full implementation, checked by
   generalization over every configuration.
3. **Generalized engine.** One declarative grammar per language now drives
   realization, parsing and validation. A property matrix replaces hand-written
   examples.
4. **Whole-prompt dialogue.** Fact questions get controlled answers, ambiguous
   questions get numbered clarifications, and task paraphrases are solved through
   their canonical prompt. This works in the worker, over HTTP and on desktop.
5. **Rust port.** The native engine reads the same seeds. JS-generated per-language
   fixtures pin the parity.

## Root causes of the earlier gaps

- **Fixed strings.** Realizations were declared strings, not generated from a
  meaning. Nothing could prove inversion, coverage or minimality.
- **Unbacked labels.** A profile label stood in for evidence. The fix is to name
  every check, and to compare each projection with an oracle that validates the whole
  candidate domain.
- **Hidden infeasibility.** Combinations that cannot be satisfied, such as ACE
  grammar in NSM primes only, were hidden behind a ratio. They are now listed in a
  reviewed ledger that the test compares exactly.
- **The top-1 conflict.** The primes are the floor below every frequency tier. A
  concept outside a tier is decomposed into primes, so every `top-N` is satisfiable.
- **No ambiguous surface in some languages.** Chinese and Hindi had no polysemous
  surface word. An input-only register adds the shared words 鼠 and मूषक. They are
  parsed but never produced, so these languages can ask for clarification too.

## Evidence

The [experiment README](../../../experiments/issue-1584/README.md) has the design,
the reproduction commands and the measured results. The
[acceptance table](../../../experiments/issue-1584/acceptance.md) maps every
criterion to code and tests.

Summary:

- **Matrix:** 26,180 projections. 17,076 are compliant, and the remaining 9,104
  match the infeasibility ledger exactly.
- **Language properties:** inventory, uniqueness, coverage, inversion, round trip
  and dialogue hold with zero counterexamples in all five languages.
- **Fixtures:** 2,259 dialogue cases, 249 of them clarifications. The JS engine, the
  worker solver and the Rust engine all reproduce them.

Screenshots of the settings and a rewrite turn are in
[`docs/screenshots/issue-1584/`](../../screenshots/issue-1584/). CI evidence is in
the [PR case study](../pull-request-1585/README.md).

## Limits

- The full STE, Special English and Globish dictionaries are not redistributable.
  The grammars use small project subsets that cover the fragment.
- Compliance is proven exhaustively over the declared fragment. Text outside it
  goes to the ordinary solver unchanged.
