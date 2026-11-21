# Repository audit and solution options

The scope includes decisions that look up a known prompt or named subject and
return a preset solution. It also includes pre-acquired external content shipped
with a runtime. The audit distinguishes these mechanisms from reusable grammar,
standards identifiers, source configuration, and algorithms that operate on
arguments. It examined seed policy and response tables, browser workers,
native handlers and adapters, compile-time includes, bundle assembly, and
release-image inputs. The earlier audit traced the larger synthesis problem to
the generic loop recording discoveries and tests while the selected handler
still supplies the answer.
[Vision audit](../issue-1548/README.md) ·
[Recorded-answer issue #1554](https://github.com/link-assistant/formal-ai/issues/1554) ·
[Meta-algorithm issue #1550](https://github.com/link-assistant/formal-ai/issues/1550)

| Location | Mechanism and evidence | Solution and verification |
| --- | --- | --- |
| [How-to policies](../../../data/seed/handler-rules.lino) and [browser worker](../../../js/worker/formal_ai_worker_how_to_guide.js) | Two specific tasks previously selected and renamed a known wikiHow title before discovery. | Replaced by bounded publisher exploration, applicability checking, and provenance-backed extraction. Verify an exact match, two searches with different topics, missing or mismatched evidence, source disable, offline behavior, and archive identity. |
| [Agentic corpus](../../../rust/src/agentic_coding/corpus.rs), [benchmark sources](../../../data/benchmarks/issue-requirements), and [test replay](../../../rust/tests/unit/agentic-coding/agentic_general_planner.rs) | A static table compiled 139 captured issue bodies into the executable. The benchmark corpus records publicly retrievable issue documents. | The Rust package no longer mirrors the corpus. The CLI uses the normal bounded capture client for public GitHub issues and the test injects its capture. Verify that a default offline corpus has no benchmark body and the same task reads the injected document before doing workspace work. |
| [Practical-help policy](../../../data/seed/handler-rules.lino) and [Rust practical help](../../../rust/src/solver_handler_how/practical_help.rs) | The goal and an NHS quote select a fixed response. One copy of the quote is used as the acceptance test and another copy as bundled evidence. Browser and native engines accept the latter without retrieving the page. | Extract guidance relevant to the stated goal from trusted source content and make the proof an observed capture. Exercise missing and mismatched sources, follow-up re-grounding, and equivalent prompts about two genuinely distinct goals. This still requires work. |
| [Documentation-method and sorting rules](../../../data/seed/handler-rules.lino), [multilingual output](../../../data/seed/multilingual-responses.lino), and [coding runtime](../../../rust/src/coding/synthesis_runtime.rs) | One pandas method has a pinned documentation answer. Sorting rules pick seed snippets by canonical direction and language. The generic coding path already has composition and verification for some shapes. | Resolve method identifiers through official documentation and derive examples from documented calls. Route sortable-sequence requirements through composition and test with varying shapes, languages, directions, and held-out methods. This still requires work. |
| [Meaning details](../../../rust/src/agentic_coding/meaning_detail.rs), [JavaScript twin](../../../js/agentic/meaning_detail.mjs), [document recipe](../../../rust/src/agentic_coding/document_recipe.rs), and [offline corpus](../../../rust/src/agentic_coding/corpus.rs) | The agent driver still carries a fisherman synopsis and two named lexeme bundles. An unrecognized document-recipe topic can fall back to tomato. | Resolve the requested subject through the existing Wikidata/Wiktionary discovery providers, build the lemma bundle from matched results, and move text used for determinism to the test transport. Test unseen concepts and no cross-topic substitution. This still requires work. |
| [Browser arithmetic](../../../js/worker/formal_ai_worker_concept_queries_and_arithmetic.js) and [compound interest](../../../rust/src/solver_handlers/compound_interest.rs) | Fixed undated currency rates and seed-linked reference quotes can supply answers. Currency codes used to parse amounts are vocabulary. | Resolve rates with the existing fact providers, require source date and currency identity, and put concrete values in replay inputs. Cover bidirectional and cross-currency pairs plus unavailable or stale rates. This still requires work. |
| [Native recurrence cache](../../../rust/src/coding/recurrence/cache.rs), [browser recurrence](../../../js/worker/formal_ai_worker_recurrence.js), and [web packaging script](../../../scripts/generate-web-distribution.py) | A recoverable Wikifunctions capture is embedded in the executable and shipped in the worker distribution. Both runtimes rely on its aliases and recurrence programs. | Use the existing formalizer with runtime source discovery and feed the committed harvest to offline tests. Verify a clean deployment against an unrelated captured recurrence and inspect package contents independently of checkout data. This still requires work. |
| [Seed embeds](../../../rust/src/seed/embedded_registry.rs), [source client](../../../rust/src/source_fetch.rs), [build script](../../../rust/build.rs), and [slim Docker artifact path](../../../Dockerfile.slim) | The binary carries fact captures and seed source bodies. The build script also supports preloading an API cache. The slim prebuilt variant copies the release binary. Browser distribution copies seed data. | Trace each capture to its existing parser and make them available through source transports or test injection. Verify package absence against the actual executable and web artifacts. This still requires work. |

Literal-language checks and the benchmark specialization gate catch limited
classes of duplication. The identified mechanisms often use short identifiers
or derived table keys, so a clean result from those checks cannot establish the
absence of hardcoded solutions. Useful verification is to give the same
capability several unrelated subjects, supply different evidence for each,
and prove the installed package can produce the results without an attached
fixture tree. Source unavailability must yield a visible evidence gap.
