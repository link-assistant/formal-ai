# A truly recursive meta algorithm, and Formal AI coding itself

Source: the architect's 2026-10-06 instructions while continuing
[PR #1188](https://github.com/link-assistant/formal-ai/pull/1188). The
requirements they became are the rows of the 2026-10-06 recursive
meta-algorithm doctrine in `REQUIREMENTS.md` (R1001-R1012).

> we should reduce number of hardcoded logic in the system, it should be able
> to reason by discovering data, as human programmer would do usually, meaning
> each symbol and word meaning is unknown, each phrase and so on, we must
> discover meaning or possible meanings from the request or problem or task
> description and the internet by searching and formalizing requests, crafting
> hypotheses on how to search and so on, so no solution is hardcoded, yet we
> can pre-cache source data to improve speed of tests, but we should use real
> meta algorithm that will reason about the unknown, our goal is to make trully
> recursive meta algorithm, that is able to understand everything it didn't
> understand before, reason about the requirements, and plan the end result,
> and transition current state of code base or any folder or place to the end
> result by reducing difference between goal. [...] with fully transparent
> recursive reasoning steps/traces and so on.

> if we have some critical blockers on requirements we should first atack the
> most critical blockers for our system to be able to do coding in general, so
> it can code itself as fast as possible [...] try to use Formal AI on tasks in
> what you do of diffrent sized, first try big task and see how it operate,
> after that try smaller task and continue make it smaller until you find the
> smallest task describable - if it also fails fix it, so Formal AI is able to
> do such small task, and better not by hardcoding, but by discovering it, and
> caching it in memory, and reusing when needed. If you find small task that
> is able to do check next bigger task and make it work and so on. One step at
> a time.

## Audit that preceded the work

Before this change, in both engines the 11-step universal algorithm
(VISION.md) was a fallback, not the main path:
- `solve()` recorded impulse, formalization and decomposition, then
  dispatched first-match-wins over the 90 handler rows of
  `data/seed/handler-precedence.lino`.
- The unknown-reasoning path ran only after every handler declined.
- Steps 6, 7 and 10 (tests first, drafts with selection, least-action
  simplification) were not on the solve path at all. `least_action.rs` was
  called only from its own tests.
- The JavaScript worker had no derivation record.
- 514 multi-word natural-language literals were counted in the JavaScript
  modules this branch added.

## Design (best practices adopted)

| Practice | Where it lives |
| --- | --- |
| Soar universal subgoaling: an impasse becomes a subgoal | unknown word → gloss grounding one level deeper; handler gap intent → general loop (`metaResolveImpasse`) |
| Explanation-based learning / chunking | `metaLearn`: a capture-grounded word used by a derived program becomes a chunk |
| GPS means-ends analysis | the difference is the number of failing examples; candidates are judged by it |
| CEGIS | the first failing example is the counterexample that rejects a candidate; the rendered source is re-verified |
| MDL / Occam | shortest typed program first; ungrounded operations cost |
| Lesk-style word-sense grounding with IDF | documentation tokens weighted by inverse frequency over the instruction set; an operation's own name outweighs its prose |
| Lazy grounding (expected-information-gain proxy) | a lookup is spent only on a word that blocks; gloss words only when no gloss reached an operation |
| Anytime with honest partial results | an unsolved goal answers with the open unknowns and the deciding question |
| W3C PROV-style trace | numbered events; derivation in links notation on every answer |
| Record/replay sources | real captures with sha256 under `rust/tests/fixtures/meta-reasoner` |

## Comparison with published approaches (R1025)

The practices above are adopted; this table states, per family of published
work, what the recursive meta algorithm does differently, where that is an
advantage, and where it is still weaker. It is re-checked whenever the loop
changes.

| Approach | What it does | Where Formal AI does better | Where Formal AI is weaker today |
| --- | --- | --- | --- |
| DreamCoder (Ellis et al., 2021): wake-sleep library learning | Grows a DSL of reusable abstractions by compressing solved programs; a neural recognizer guides search | Every learned chunk carries its grounding (dictionary capture, sha256, derivation in links notation), so a learned operation can be audited and retracted; no opaque recognizer | No compression pass over solved programs yet — chunks are learned per word, not refactored into shared abstractions |
| FlashFill / PROSE (Gulwani, 2011; Polozov and Gulwani, 2015): programming by example over a DSL | Version-space algebra over a fixed DSL, ranked by hand-tuned scores | The instruction set is grounded from documentation rather than fixed, so a request outside the starting DSL can still be read; examples verify instead of being the only specification | Version-space search is far faster and more complete within its DSL; the loop enumerates shortest-first without a shared version space |
| SyGuS / CEGIS (Alur et al., 2013; Solar-Lezama, 2008) | Syntax-guided synthesis against a logical specification, refined by counterexamples | Specifications are formalized from natural language in four languages, and every rejected candidate is recorded in the trace | No SMT solver checks a candidate against a full logical specification; verification is by examples and seeded probes |
| Large-language-model code generation (Codex, AlphaCode and successors) | Samples programs from a model trained on code, filtered by tests | Answers are derived, never recited (the no-memorization gate pins it), deterministic, offline-reproducible and explained step by step; an unsolved goal says what is unknown instead of guessing | Coverage of everyday programming tasks is much narrower; the ladder measures how far it reaches (`data/meta/ladder-ratchet.lino`) |
| Cyc and other hand-built knowledge bases | Encode common sense as hand-written axioms | Meanings are discovered from cached public sources with provenance rather than authored axiom by axiom; the closure audit keeps the seed honest | The discovered network is shallow next to decades of curated axioms; many words still close only through a gloss |
| Soar / ACT-R cognitive architectures | Impasse-driven subgoaling and chunking over production rules | The same impasse→subgoal→chunk cycle is applied to language grounding and program derivation together, with a human-readable derivation | No learned utility or decay; chunks are never forgotten or re-weighted |

The comparison keeps two obligations open: a compression pass that refactors
learned chunks into shared abstractions (DreamCoder's sleep phase), and a
solver-backed verifier for formalized specifications beyond examples.

## The ladder

Real requests put to Formal AI's own JavaScript engine
(`node scripts/formal-ai-js.mjs --trace "<prompt>"`) were tried from the
biggest down.

The biggest, "Implement a new CI gate script that fails when any js/worker
module exceeds 1000 lines", was answered as `calculation_error` by the
arithmetic handler. It now gets an honest open-unknowns answer.

Smaller requests were tried until one failed for a reason the general loop
could fix. Each fix was general, and the next size up was retried:
- a threshold filter combinator;
- request-number binding;
- structural vocabulary;
- argument-type evidence from data words;
- head-word result type;
- coverage that excludes filter measures;
- multi-sample probes.

`rust/tests/fixtures/meta-reasoner/ladder.lino` holds every rung reached.
