## Standing Doctrine: The Recursive Meta Algorithm Is the Main Path (2026-10-06)

Stated by the project owner while continuing PR #1188 (quoted in
[the 2026-10-06 meta-algorithm architect note](../architect-notes/2026-10-06-recursive-meta-algorithm.md)).

The system reasons about the unknown instead of recognizing known tasks. Every
word starts unknown; meanings are discovered from the request and trusted
sources, with captures pre-cached for tests. Solutions are composed and
verified, not written per task. Formal AI is put to work on real tasks of
decreasing size: the smallest failing task is made to work by discovery, then
the next bigger one.

| ID | Requirement | Status / Evidence |
| --- | --- | --- |
| R1001 | The general loop runs on every turn ahead of the handler table, not as a fallback: it opens each request word as an unknown and grounds it without network before any handler is consulted. | Delivered in JavaScript 2026-10-06: `metaReason` at the top of `solveImpl` (`js/worker/formal_ai_worker_20.js`). Rust twin pending (R1012). |
| R1002 | Grounding order is request definition → learned chunk → instruction-set documentation → dictionary capture; a capture's glosses are grounded by the same function one level deeper, with depth and cycle bounds. A gloss word is looked up only when no gloss of its parent reached an operation. | Delivered: `metaGround` (`js/worker/formal_ai_worker_meta_reasoner.js`); `rust/tests/web/meta-reasoner.test.mjs` replays real captures. |
| R1003 | A request's own examples are its goal state: programs are enumerated over the typed instruction set shortest first, each candidate's difference is its failing examples, the first failing example is the counterexample, and parameters/thresholds are inferred from the examples. | Delivered: `formal_ai_worker_meta_synthesis.js`. |
| R1004 | The program an answer shows is the exact source the verifier (examples) or the probe (no examples) ran; an unverified program says so. | Delivered: `metaRender` + `metaVerify`; `meta_reasoned_program_unverified` intent. |
| R1005 | The instruction set, cue vocabulary, grammatical words, affixes, probes, impasse intents and response templates are seed data (`data/seed/meta-reasoning.lino`); the reasoner's code holds no natural-language literal. Operation documentation is written as the operation's reference documentation, never as a list of request words. | Delivered 2026-10-06. Ratchet over all of `js/worker`: open (R1010). |
| R1006 | A specialized handler's admission that it could not do the task (`*_skill_gap`, `*_unspecified`, `calculation_error`, `unknown`) is an impasse handed to the general loop; when nothing is derivable the answer lists the open unknowns and the question that would decide them. | Delivered: `metaResolveImpasse`. |
| R1007 | A word grounded through a source and used by a derived program is learned (a chunk keyed by the word), so a paraphrase is answered without a lookup; chunks serialise to links notation for persistence. | Delivered in memory (`metaLearnedLino`); persistence into the user's memory store: open. |
| R1008 | Every answer carries its derivation in links notation (goal, unknowns with origin, program, verification, numbered events); the reasoning shown to the reader is generated from the same events. | Delivered in JavaScript: `result.derivation`. |
| R1009 | Formal AI is exercised on real tasks of decreasing size (`scripts/formal-ai-js.mjs`); each rung reached is kept in `rust/tests/fixtures/meta-reasoner/ladder.lino` and must keep being derived, never handled. | Delivered: 16 rungs, up to "Write a Node.js script that prints every file in a folder with more than 1000 lines". The derived program, run on `js/worker`, lists exactly the files `wc -l` does. |
| R1010 | The natural-language literal count in `js/worker` handler code only falls (ratchet like R998), as vocabulary and templates move to seed and handlers are replaced by derivations. | Open. Baseline from the audit: 514 multi-word literals in the modules this branch added. |
| R1011 | Critical blockers for general coding, attacked first: (a) a file/repository instruction set (list, read, write, run) so the loop composes repository edits; (b) a handler that claims prose it cannot parse (the arithmetic handler answering a coding request containing a number) defers to the loop; (c) multi-clause decomposition into sub-goals the loop solves one by one. | (a) started: `list_files` / `read_file` (environment node) and composed filter measures; write and run are open. (b) mitigated by R1006. (c) coordinated clauses compose in order (`metaClauses`); sub-goal decomposition of whole requests is open. |
| R1012 | The meta reasoner reaches Rust by translation (R1000) once the translator lands; until then the Rust engine's derivation and handler-impasse behavior are ported by hand, naming the JavaScript originals. | Open. |
