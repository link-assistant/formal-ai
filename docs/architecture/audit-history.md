# Architecture: Audit History and Current Gaps

Part of the [architecture overview](../../ARCHITECTURE.md) (§16). Section
numbers match the overview's table of contents; paths are relative to the
repository root.

## 16. Audit History And Current Gaps

The original issue #244 architecture questions, the E1-E20 follow-up batches,
and the reasoning batch E21-E27 are merged (PRs #305-#311). Every message is now
formalized into a Links-Notation intent before routing (`rust/src/intent_formalization.rs`),
unmatched prompts run a reasoning-under-unknowns loop (`rust/src/solver_unknown_reasoning.rs`)
instead of a canned opener, per-language program intents collapse into a parametric
`SelectedRule::WriteProgram`, substitution rules run over link CRUD (`rust/src/substitution.rs`),
natural language can query memory / call APIs / execute code under the permission
model (`rust/src/solver_handlers/`), a bounded isolated agent runs allowlisted commands
(`rust/src/agent.rs`), and a permissive industry benchmark slice is imported
(`data/benchmarks/industry-suite.lino`).

The 2026-05-27 audit (issue #244, fourth pass) found that the largest remaining
gap was the **generality of the synthesis step**. That gap is now closed: the
synthesis batch **E28-E32** ([#313](https://github.com/link-assistant/formal-ai/issues/313)-[#317](https://github.com/link-assistant/formal-ai/issues/317))
is merged (PRs #319-#323). The universal 11-step loop is still the main path for
every prompt (`rust/src/solver.rs::solve_with_history_probability_store_and_intent_cache`),
but the synthesis step (`record_candidates`) now **derives** answers by composing
decomposed sub-results over the links network instead of returning a seed keyed
on the prompt: arithmetic/word-problem and counting answers are computed, Python
functions are synthesized from spec + tests and verified in the bounded agent
workspace (`rust/src/solver_handlers/program_synthesis.rs`), text manipulation is
generalized over arbitrary input, and the imported benchmark suite grew to a
10-case slice that passed **10/10** with a `minimum_pass_count` ratchet (13 cases / 13-floor today — see `data/benchmarks/industry-suite.lino`)
(`rust/tests/unit/specification/benchmarks.rs`). The upstream rows of the same
suites are HumanEval 14/164 on the full slice (`--online`, 2026-09-17) and
MBPP 49/500 cold-offline (2026-09-18), with the 2026-09-15 first-20 rows
(HumanEval 20/20, MBPP 20/20) kept as regression controls; `docs/status.md`
renders every upstream suite row from the external-results ledger, and
`docs/benchmarks.md` publishes the honest current numbers per slice.

The 2026-05-29 audit (issue #244, fifth pass) found the next gap is **parity**,
per the PR #245 feedback ("all Rust and JavaScript logic are in sync", "all
languages are supported equally"). The sixth-pass audit (also 2026-05-29) records
that the parity batch is **now closed and merged**:

1. **Universal multilingual operation vocabulary (E33, [#326](https://github.com/link-assistant/formal-ai/issues/326), PR #328).**
   `rust/src/solver_handlers/text_manipulation.rs` no longer triggers on English
   literals. Every operation is recognised by canonicalising the prompt against
   one shared data-driven vocabulary (`data/seed/operation-vocabulary.lino`) that
   lists each operation's surface forms per supported language (`en|ru|hi|zh`),
   mirroring how `intent-routing.lino` already works — general, not per-handler
   literals. Adding a surface form or a whole language is a seed-data edit, not a
   code change. The Rust core loads it via `seed::operation_vocabulary()`; the
   browser worker loads the same file via `js/seed_loader.js`.
2. **Cross-runtime parity (E34, [#327](https://github.com/link-assistant/formal-ai/issues/327), PR #329).**
   The JavaScript browser worker (`js/worker/formal_ai_worker.js`) now routes
   synthesis prompts through `tryLinkNativeSynthesis`, `tryProgramSynthesis`, and
   `tryTextManipulation`, deriving the same synthesis/numeric/program/text answers
   as the Rust core, verified by the shared fixture
   `data/parity/cross-runtime-synthesis.json`, the Rust test
   `shared_cross_runtime_synthesis_fixture_matches_rust_solver`, and
   `rust/tests/e2e/tests/issue-327.spec.js`. Mirrors the E19 [#282](https://github.com/link-assistant/formal-ai/issues/282)
   browser-worker parity precedent; WebAssembly stays the bridge for shared
   primitives and JavaScript stays UI/glue per pillar 18.

With E1-E34 all merged, no vision-planning epic remains open **for issue #244
specifically**. That statement does not mean planning is finished: the later
batches **E37-E55** ([#656](https://github.com/link-assistant/formal-ai/issues/656)-[#674](https://github.com/link-assistant/formal-ai/issues/674),
from the issue [#651](https://github.com/link-assistant/formal-ai/issues/651)
gap analysis) and **E56-E68** ([#698](https://github.com/link-assistant/formal-ai/issues/698)-[#710](https://github.com/link-assistant/formal-ai/issues/710),
from the 2026-07-14 audit) are closed, as is the E69-E77 planning batch
([#916](https://github.com/link-assistant/formal-ai/issues/916)-[#924](https://github.com/link-assistant/formal-ai/issues/924),
delivered by PRs #966, #984, #986, #992, and
[#1003](https://github.com/link-assistant/formal-ai/pull/1003)-[#1007](https://github.com/link-assistant/formal-ai/pull/1007)); `ROADMAP.md` records their requirement-level
status and the later E78-E117 items per issue, and planning continues through
the live open set:

- The #559 handler-migration mandate and its ratchet
  ([#959](https://github.com/link-assistant/formal-ai/issues/959), tracked in
  `data/meta/core-boundary-ledger.lino`).
- Benchmark-frontier learning
  ([#1087](https://github.com/link-assistant/formal-ai/issues/1087)),
  requirement-derived editing
  ([#1088](https://github.com/link-assistant/formal-ai/issues/1088)), gate
  collapse ([#1089](https://github.com/link-assistant/formal-ai/issues/1089)),
  and traceability
  ([#1090](https://github.com/link-assistant/formal-ai/issues/1090)) — the
  follow-ups #1085 left open.
- Dynamic coding discovery ([#710](https://github.com/link-assistant/formal-ai/issues/710),
  PR #888 merged, plans 06/07 active) and anticipatory learning
  ([#705](https://github.com/link-assistant/formal-ai/issues/705)).
- Delivery breadth: PWA/npm
  ([#665](https://github.com/link-assistant/formal-ai/issues/665)),
  Marketplace ([#666](https://github.com/link-assistant/formal-ai/issues/666)),
  debugger ([#667](https://github.com/link-assistant/formal-ai/issues/667)),
  shareable packages ([#668](https://github.com/link-assistant/formal-ai/issues/668)),
  cloud sync ([#669](https://github.com/link-assistant/formal-ai/issues/669)),
  WebVM ([#670](https://github.com/link-assistant/formal-ai/issues/670)).

Issue #703's controller is rooted at `rust/src/orchestration/`. A deny-by-default
workspace capability gates every run; the seed client registry supplies six
structured editing and native-resume contracts; an explicitly supplied custom
argv requires an independent executable grant even when it borrows a registered
CLI label. `dispatch` copies candidates, decomposes or compares in parallel,
verifies them and composes only passing non-conflicting effects. `runner`
records canonical hash-chained sessions, native client ids and parent-digest
continuations, while `analysis` extracts answer events, formalizes them into
the meta-language, deduplicates/ranks/rechecks claims, reports contradictions,
requests same-session correction, and verifies any separately recorded
translation. Completed sessions can enter the existing client-contract learner
only as evidence-linked, human-review-gated proposals. Model agreement is
labelled a cross-agent preflight, never external factual proof.

A later issue #349 roadmap closed the concrete program-modification gap that the
parity batch exposed in a user dialog: after an active program artifact exists,
bare follow-ups such as "sort the results in reverse order" are rewritten
against conversation history, decomposed into program modifiers, lowered through
`rust/src/program_plan.rs`, and traced through default-off diagnostics. The #349 flow
is guarded by `rust/tests/integration/issue_349_reverse_sort.rs`, the browser-worker
parity harness `experiments/issue-361-cross-runtime-parity.mjs`, the
coding-modification benchmark ratchet, and the self-improvement specs.

Issue #408 text/code editing path extends that active-artifact behavior to
literal user edits such as "replace Hello World with Bye world". The solver
extracts the prior assistant artifact from conversation history when the prompt
omits an explicit input, applies deterministic text operations from
`rust/src/solver_handlers/text_manipulation.rs`, and mirrors the supported operations
in `js/worker/formal_ai_worker.js`. The issue #408 benchmark matrix uses
self-authored benchmark-family examples plus
`data/benchmarks/text-manipulation-suite.lino`, which records 48 researched
sources and drives 30 deterministic local variations per source through a
1,440/1,440 pass-count ratchet — while the upstream instructed-editing
analogue, CoEdIT, scores 0/20. The benchmark gate reports per-source totals:
each source has a 3-check repository-local 10% floor and must pass the stronger
30/30 local ratchet.

Arbitrary natural-language programming beyond the trigger/response subset of
`rust/src/skill_compiler.rs` is closed by E55 (issue #674): the shared intent
formalizer decomposes freely phrased procedures into ordered source-grounded
requirements; `rust/src/skill_procedure.rs` lowers all of them into typed operations,
persists an integrity-checked executable artifact, and interprets it through a
permissioned host. Canonical slugs give equivalent en/ru/hi/zh procedures one
set of skill links. Unknown steps produce a named gap and an inert,
human-gated learning proposal with no partial compile. Observations pair each
unsupported multilingual surface with a successful seeded paraphrase, allowing
the learner to infer one typed candidate while conflicting meanings fail
closed; promotion still requires green tests and explicit human approval. The
solver, explanation handler, public conformance CLI, and Agent CLI route all use
the same persisted artifact, and Agent verifies the interpreter's execution
record before it reports success.

Pull requests that close any of these should update the corresponding row in
the table in Section 2 and link the new module.
