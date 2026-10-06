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
