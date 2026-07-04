# Vision

Formal AI should become a symbolic assistant whose live state is an associative operational space. The assistant should not be defined by hidden neural weights or by a large memoized answer table. It should be defined by an inspectable associative network of links: user messages, source data, inferred meanings, commands, test results, generated code, failures, permissions, and final answers.

The associative network is the AI. The runtime should activate, extend, query, and simplify that network as work happens.

This places Formal AI squarely in the tradition of [symbolic artificial intelligence](https://en.wikipedia.org/wiki/Symbolic_artificial_intelligence) (GOFAI): the link network is a [semantic network](https://en.wikipedia.org/wiki/Semantic_network) in the classical sense, and intelligence is the rule-driven manipulation of human-readable symbols ([physical symbol system](https://en.wikipedia.org/wiki/Physical_symbol_system) hypothesis, Newell & Simon 1976) rather than numeric optimization of hidden weights. The audit in [`docs/case-studies/issue-451/symbolic-ai-best-practices.md`](docs/case-studies/issue-451/symbolic-ai-best-practices.md) maps each of the field's best practices onto this associative stack.

## Who This Is For And What Pain It Closes

Before the architecture, the people. Mainstream AI assistants share a cluster of
pains: you cannot see *why* an answer was produced, the same prompt is not
guaranteed to return the same answer, useful answers require a GPU and a vendor's
cloud, the assistant's memory is an opaque store you cannot export or move, and
you cannot teach it by editing data instead of retraining weights.

Formal AI is for the person who needs the opposite of each: a **defensible,
reproducible, ownable, editable** answer. The auditor who must cite reasoning,
the multilingual user who wants `en | ru | hi | zh` treated equally, the learner
who wants honest execution notes on generated code, the analyst who needs math
and currency with shown work, the integrator who wants a drop-in
OpenAI-compatible endpoint with no GPU bill, the privacy-conscious owner who
exports everything the agent saw and did as one file, and the tinkerer who
reconfigures behavior by editing seed data.

A concrete example user journey — Dana the auditor asks `What is 8% of $50?`,
gets `$4`, then asks `Why did you answer that?` and receives the recorded
impulse, formalization, decomposition, validation, and simplification that
produced it — is walked end to end in
[`docs/USER-JOURNEYS.md`](docs/USER-JOURNEYS.md). That document enumerates
**every user journey supported today and every journey we could support next**,
mapped to the surfaces below and to the principles in this file, so the vision
makes a concrete promise rather than only describing the machine.

## Core Idea

The system should prefer deep understanding of user needs, intent, context, and available evidence over answer memoization. A prompt should trigger enough data collection and reasoning to justify the response for that prompt. What the system learns along the way should remain available as reviewable knowledge, with source links and execution traces attached.

The default native store is link-native:

- `doublets-rs` is the default native Links Data Store for meanings, history, rules, traces, and executable associations.
- `doublets-web` / IndexedDB is the browser-side storage shape.
- Links Notation is the reviewable text format for seed data, portable packages, traces, and repository data.
- Doublet links are the primitive storage model for this project.
- `lino-objects-codec` remains useful for converting structured objects into reviewable Links Notation during the transition.

The dynamic type system should be expressible with doublets:

```text
Type -> SubType
SubType -> SubType
SubType -> Value
```

This keeps the seed model small while letting the network define new concepts, instances, relations, handlers, and rules as they are needed.

## Operating Principles

- Small seeds, dynamic growth: start with the smallest useful seed dataset, then construct missing knowledge on demand.
- Transparent reasoning: every step, decision, command, data access, and answer should be traceable to links and source events.
- Add-only history: changes requested by users or made by the assistant should be appended as events first, then projected into the current state.
- User-queryable memory: the user should be able to ask about any relevant detail in the associative network and receive a traceable answer.
- Chat-first interface: chat should be the default interface for English, Russian, Hindi, Chinese, and later other languages.
- Visual network on demand: the link graph should be available side by side with chat when the user wants deeper inspection.
- Bounded chat autonomy: chat mode should do only enough work to answer the current message, including compiling or running code when appropriate.
- Explicit agent autonomy: agent mode should expose actions and run them in an isolated environment such as a Docker image, a server sandbox, or a browser VM where practical.

## Reasoning Model

The assistant should use a universal problem-solving loop:

1. Record the user message as an impulse event.
2. Identify unknown concepts, requirements, constraints, and missing context.
3. Search local links first, then external sources if local data is insufficient.
4. Convert findings into link-native meanings with source metadata.
5. Split the problem into smaller tasks until each task is executable or answerable.
6. Generate candidate solutions, tests, and traces.
7. Execute or validate candidates where the environment allows it.
8. Record results, failures, and learned procedures.
9. Return the smallest sufficient answer plus links to the relevant trace.

This loop can be implemented first as ordinary Rust logic, then increasingly represented as link substitutions, triggers, and reusable associative packages.

## Universal Problem-Solving Algorithm

The reasoning loop above is the outer skeleton. The inner mechanics should be a single universal algorithm that does not assume prior knowledge of the task and works in the same shape for greetings, code generation, translation, source-checking, agent actions, or any future request:

1. **Impulse**: append the raw user message as an `impulse` event in the append-only log.
2. **Formalization**: turn the impulse into a `requirement` record. The algorithm decides — controlled by a configurable knob — whether to guess the formalization from context or to ask the user the smallest possible clarifying question.
3. **Context and domain data**: derive the language, surface, mode flags (chat vs agent, diagnostic on/off), and the domain (greeting, code, translation, math, agent action) from the formalized requirement.
4. **History lookup**: check whether the same or a similar requirement has been solved before; if so, reuse the prior solution and record a `cache_hit` link.
5. **Decomposition**: when the requirement is composite (multiple clauses, "and", "with tests", "with benchmarks"), split it into sub-impulses and recursively formalize each one until every sub-requirement is small enough to be solved directly.
6. **TDD-style test generation**: derive at least one executable check or assertion that any candidate solution must pass.
7. **Solution synthesis**: build candidate solutions by (a) reusing known parts, (b) reasoning from rules, (c) random or evolutionary search where the structure allows, picking the strategy by compute budget.
8. **Combination**: combine the partial solutions back into a full solution that addresses the original requirement.
9. **Verification**: run the candidate against the generated tests; on failure, surface the failure as a `trace:execution_failure` link instead of silently retrying.
10. **Simplification**: apply transformation rules that preserve meaning to shorten the answer and the reasoning trace. Pick the smallest sufficient form.
11. **Documentation and presentation**: produce the user-facing reply, the Links Notation trace, and the visible evidence links. If the user asks for execution, run the code in the appropriate isolation level.

Every step writes its own event to the append-only log so the user can ask the chat why the assistant did what it did and get a traceable answer from the recorded experience.

## Configurable Solver Knobs

The universal algorithm should be controlled by a small, explicit, persistable `SolverConfig` so the same engine can be tuned per surface or per user:

- `guess_probability` — how often the algorithm guesses a formalization vs. asking a clarifying question (`0.0` = always ask, `1.0` = always guess).
- `context_sensitivity` — how aggressively the algorithm uses surrounding context (previous messages, recent events) when formalizing.
- `questioning_rigor` — how strict the clarifying questions are (`0.0` = accept almost anything, `1.0` = ask until the requirement is fully formal).
- `max_decomposition_depth` — how deep the recursive decomposition is allowed to go.
- `agent_mode` — whether agent mode is opted in. Off by default.
- `diagnostic_mode` — whether diagnostic links are echoed in the user-facing reply.
- `offline` — whether external lookups are allowed (also honored from the `FORMAL_AI_OFFLINE` environment variable).
- `cache_ttl_seconds` — TTL for cached external sources (default ≈ two months).
- `temperature` — controls how aggressively the formalization step collapses multiple candidate interpretations into one. Lower values bias the solver toward the highest-scoring interpretation; higher values keep more interpretations alive and trigger clarifying questions or guesses, depending on `guess_probability`. Modelled on the softmax-temperature pattern used by neural networks, but applied to discrete Wikidata-anchored interpretations and symbolic probability evidence rather than neural logits.

These knobs are deterministic: the same prompt with the same config produces the same answer. "Random guessing" is seeded from the impulse content hash so reproducibility is preserved.

## Formalization And Temperature

The reasoning loop is built around an explicit formalization layer rather than memoized prompt → answer pairs. Every input message is first translated into Links Notation as a sequence of statements or questions and appended to memory in its original form. The translated statement is then formalized: each verb phrase is mapped to a Wikidata **P-id** (property), each noun phrase to a Wikidata **Q-id** (item), with a fallback chain to Wikipedia article links and Wiktionary entries when no Wikidata anchor exists. A candidate formalization is only accepted when the targeted concept actually mentions the surface form and matches it semantically — otherwise the candidate is recorded as `formalization_unresolved` and the next candidate is tried.

Multiple plausible formalizations are scored, then a temperature-controlled selector picks among them in the same way neural networks pick among logits: lower temperature collapses onto the highest-scoring interpretation; higher temperature keeps competing interpretations alive. When two or more candidates have probabilities that are equal or close, the solver either asks the smallest clarifying question that distinguishes them or guesses according to `guess_probability` — the choice is config-driven and visible in the trace. Every interpretation, every clarifying question, every guess, every accepted formalization, and every fallback is appended to the event log so the user can ask "why did you read it that way?" and get a complete answer.

Probability evidence must remain part of the same symbolic network. Bayesian-style evidence records and Markov-style transition records are Links Notation data with provenance, timestamps, cached-source fingerprints, and deterministic replay. They can bias candidate formalizations or answer candidates, but they never modify neural weights and never call neural inference; they only adjust symbolic posterior scores before the existing temperature and clarify-vs-guess policy runs.

The full pipeline is documented in [`ARCHITECTURE.md`](ARCHITECTURE.md).

## Growable Memory And Public Knowledge As Cache

Memory should grow with use, not just with prompts. Every reasoning step, every internal decision, every external request, and every response is appended to the same log so the next similar request can reuse the prior work in part or in full. The default native store is doublets-rs in the library, CLI, server, and Telegram surfaces, with doublets-web (IndexedDB / `localStorage`) on the browser; backups are written as `.lino` files representing Links Notation, both to disk and to additional persistent storage (browser IndexedDB, future cloud sync).

Treating the internet (Wikipedia, Wikidata, Wiktionary, Wikifunctions, Rosetta Code, public APIs) as a public database and the local doublets store as a cache for that database substitutes deterministic reasoning over reviewable links for opaque GPU-backed inference. The same caching pattern carries `source:`, `fetched_at`, and `sha256` metadata per the existing `cache_ttl_seconds` policy; offline mode refuses external lookups and emits a `policy:offline` event instead of synthesizing facts.

## Append-Only Event Log

Every action the algorithm takes is appended to an in-process event log before the answer is built. Each event carries a kind (`impulse`, `language_detected`, `local_search`, `external_search`, `sub_impulse`, `candidate`, `validation`, `policy`, `agent_action`, `cache_hit`, `source`, `trace`, `error`) and is identified by a content-addressed id. The event log is the system of record; the answer and the evidence links are projections of it. The user can chat over this log:

- "Why did you answer that?" returns a meta-explanation built from the most recent trace.
- "What do you know about X?" returns the links involving X.
- "List the facts I have contributed" filters by user.
- "Forget X" is refused unless the explicit retraction protocol is used, because the log is append-only.
- "Export the network" returns the Links Notation snapshot of the seed dataset plus the visible event log.

## Computation Model

Formal AI should use trigger-style computation over links. A trigger can react to insertion, update, deletion, or a matched pattern in the network. Substitution rules should be first-class knowledge and should be able to express reads, writes, transformations, and simplification passes. The associative store supports five rule shapes, ranked from most reviewable to most flexible:

1. **Pure data rules** — `when x do y` substitutions stored directly as doublets, executable by a simple matcher in any environment.
2. **Compiled Rust handlers** — registered through `solver_handlers` and addressable as data so the rule index can point at them.
3. **Compiled JS handlers** — registered through the browser worker and addressable the same way, so the same rule can have language-specific implementations.
4. **Dynamically compiled Rust/JS code stored as data** — source text is itself a link payload; the runtime can compile-on-demand and cache the resulting handler.
5. **Natural-language skills / instructions** — stored as `.lino` text, executed either by an interpreter that walks them one step at a time or by an on-demand compiler that translates them into rules in any of the four shapes above (and ultimately into native binaries when the platform allows).

Deep.Foundation is a useful reference for associative packages, handlers, permissions, and code stored inside associative memory. This project adapts those ideas to local Rust, browser, and CLI modes using Link Foundation doublets instead of triplet links. `ARCHITECTURE.md` documents the data shape of each rule kind and how the trigger loop dispatches across them.

## Product Shape

The same symbolic core should be available through:

- Rust library API.
- CLI chat and dataset commands.
- OpenAI-compatible HTTP API surfaces.
- Docker-ready microservice.
- GitHub Pages chat demo backed by a Rust WebAssembly worker.
- Telegram private and public chat surfaces.
- Desktop and embedded agent modes share the same library boundary; the desktop wrapper is tracked by issue [#280](https://github.com/link-assistant/formal-ai/issues/280).

Code-generation tasks should be a first focus area. The assistant should generate algorithms in popular languages, compile or run generated code when the environment supports it, report execution limits honestly, and preserve logs for failed reasoning or failed execution. Browser-only mode can start with JavaScript evaluation and later experiment with WebVM.

## Meaning And Identity

For every uniquely defined concept, the system should converge on one meaning link. If the same name points to two different meanings, the system should split them into separate concepts and record why. The network should remain dynamically growing and incomplete, but it should actively reduce contradictions as new evidence arrives.

Every meaning should be explainable through other meanings. Human-readable
glosses and source labels are useful annotations, but the primary semantic
structure should be recursive links: `defined_by` links for ontology reduction
and semantic facets for notation, annotation, denotation, and connotation. The
facet kinds themselves are meanings in the seed, so adding a new semantic view
is data growth rather than a new hardcoded language constant.

The semantic root should also stay compatible with Links Theory: `reference`,
`link_action`, defined connectives, quantity primitives, and self-equations are
seed meanings, and ambiguous symbols are split into distinct meanings such as
`bank_river` and `bank_money` instead of one overloaded surface.

Natural languages and programming languages should be translated through link-native meanings rather than through one-off text rewrites. Once a phrase has been fully formalized — verbs mapped to Wikidata P-IDs, nouns mapped to Q-IDs, fallbacks documented to Wikipedia or Wiktionary entries — translating it to another language reduces to looking up the destination-language label on the same P/Q anchor. The same mechanism translates between natural language and programming languages: formalized statements can be rendered into Rust, JavaScript, or any other target whose syntactic forms have been linked into the doublet store. Links Notation acts as the intermediate language of meaning for explanations, code generation, data imports, cross-language translation, and on-demand compilation of natural-language skills into executable code.

Cross-language translation should also preserve the source's surface signal: the leading capitalization and the terminal punctuation a user typed are part of the meaning, not noise. A lowercase `как у тебя дела?` should round-trip to lowercase `how are you?`, and a source fragment with no terminal mark should not gain one in the target language. The pipeline is `formalize → meaning → deformalize → match_source_formatting`, and the meaning ID, source language, and target language remain in the Links Notation trace so the translation stays inspectable.

Issue #526 makes round-trip survival the translation quality contract. Every
supported language must pass a language-to-meta-to-same-language check with no
data loss, and every supported language pair must translate through the shared
meaning before rendering a target surface. Direct pair-specific translation can
only be an implementation detail below that contract; it is not acceptable as a
quality path when it bypasses the meta language. The same rule applies to code:
a Rust <-> JavaScript translation must preserve the code meaning link, not only
produce syntax that looks plausible.

## Data Is The Interface

The shell code should be about *interfacing* (rendering chat, dispatching tools, persisting events) and not about *logic*. The agent's identity, its multilingual responses, its concept table, and its registered tools should all live in seeded Links Notation files inside an associative store, so a user can fully reconfigure the agent — add a new language, retire an answer, register a new tool, change a rule — by editing data, not by rewriting code. The same principle applies to executable knowledge: precompiled handlers can be seeded, and dynamically-compiled Rust, JavaScript, and WebAssembly snippets can be linked into the store on demand, so the data graph itself defines what the agent can do.

Concretely, `data/seed/` is the canonical knowledge surface for every interface in this repository. The browser worker fetches the files at runtime through `src/web/seed_loader.js`; the Rust library, CLI binary, HTTP server, and Telegram webhook read the same files through `src/seed.rs`, which `include_str!`-embeds each `.lino` at compile time so even offline builds expose the same data. `scripts/sync-seed.sh` keeps `src/web/seed/` mirrored from `data/seed/` for the GitHub Pages deploy. The seed currently covers multilingual responses, the concept table, the tool registry (HTTP fetch, web search, Wikipedia lookup, JS execution, local file read, memory append, memory export — each tagged `thinking` or `agent`), language-detection rules, prompt-question patterns, identity metadata, greetings, hello-world programs, demo dialogs, and the **intent-routing rule book** (`keyword` / `phrase` / `token` / `combo` semantics) that decides which handler runs for a given prompt. Any environment-specific tool (axios, file I/O, bash, docker, container actions) is registered through the same data shape.

## Single-File Reproducibility

Every interface should offer a one-click way to capture the full agent state — the seed, the modified data, the UI preferences, and the entire append-only memory log — as one Links Notation document, and every default-labelled export action should produce that **full** document. The web demo's **Export memory** topbar button writes `formal-ai-memory.lino` as a complete `formal_ai_bundle` (seed + events + preferences + environment metadata + version), so a single click is enough to reconstitute the session. The "Report issue" link instructs the user to wrap that same file in a `.zip` (GitHub's issue uploader does not yet accept `.lino`) and to redact sensitive content before attaching. On the Rust side, `seed::merged_bundle()` produces the equivalent `formal_ai_seed_bundle` document, `seed::parse_bundle()` (mirrored in JS by `FormalAiSeed.parseBundle` / `FormalAiSeed.loadFromBundle`) recovers the per-category split files from it, and `memory::export_full_memory` / `memory::import_full_memory` round-trip the full `formal_ai_bundle` including the embedded `demo_memory` log. The CLI matches: `formal-ai memory export` defaults to the full bundle (with `--events-only` opting back into the legacy `demo_memory` shape), `formal-ai memory import` / `formal-ai bundle import` auto-detect either format, and both print `Migration: <message>` notices when the imported seed version differs from the running app's. A user can always say "here is everything my agent saw and did" in a single file, regardless of which surface they were on.

## Self-Aware Environments

The seed must describe every environment the agent can run in. `data/seed/environments.lino` (loaded as `seed::environment_directory` in Rust and `FormalAiSeed.extractEnvironmentDirectory` in the browser) lists each interface (`browser`, `rust_library`, `cli`, `http_server`, `telegram`, `docker_microservice`) along with the runtime that hosts it, the path it reads seed data from, where it persists memory, and the exact command to export/import that memory. The same file enumerates supported migration flows (browser ↔ CLI, browser ↔ browser, CLI ↔ CLI) and the Links Notation format that carries them. Running `formal-ai environments` prints this directory from the embedded seed, so the agent can answer "where can you run?" and "how do I move your memory?" from data instead of code.

## Library-First Availability

Every capability the CLI or HTTP server exposes is also reachable from the `formal_ai` library crate root. Embedders writing their own surface (a desktop app, a bot for another platform, a research notebook) get the same `MemoryStore`, `MemoryEvent`, `export_memory_links_notation`, `parse_memory_links_notation`, `export_memory_bundle`, `extract_memory_from_bundle`, `seed_files`, `merged_bundle`, `parse_bundle`, `environment_directory`, `environment_records`, `agent_info`, `intent_routing`, `multilingual_responses`, `language_rules`, and `prompt_patterns` accessors that the bundled binaries rely on. The shell code is interface; the library is logic.

## Current Direction

The current repository is a deterministic symbolic implementation. It already has deterministic rules, Links Notation seed files, OpenAI-shaped API responses, a static web demo, Telegram support, execution metadata for simple code examples, and case-study documentation. Every interface now reads its multilingual responses, concept table, tool registry, language-detection rules, prompt patterns, and intent-routing rule book from the shared `data/seed/` directory through `src/seed.rs` (Rust) and `src/web/seed_loader.js` (browser). Reasoning steps and tool invocations land in the append-only memory log on the web side; the merged seed bundle round-trips through one `formal_ai_seed_bundle` Links Notation file via `seed::merged_bundle()` / `seed::parse_bundle()`.

The next step is to keep the implemented surfaces small while moving more of the assistant's behavior into explicit links: requirements, source facts, traces, prompts, handlers, permissions, tests, and reusable problem-solving procedures. The CLI, server, and Telegram bot should expose the same bundle-export and simplified-issue-reporting actions the web demo offers while preserving the unified doublets-rs/doublets-web store and Links Notation migration surface across interfaces.

The foundation batches E1-E20, the reasoning batch E21-E27, the synthesis batch E28-E32, and the parity batch E33-E34 are merged (PRs #305-#311, #319-#323, #328-#329). Every user message is now formalized into a Links Notation intent before routing, unmatched prompts run a reasoning-under-unknowns loop instead of falling through to "I can't answer that", narrow per-language intents are collapsed into a parametric `write a program` intent, behavior can be expressed as substitution rules (`replace x y`, `when n do m`) over link CRUD, natural language can query memory / call APIs / execute code under the permission model, a bounded isolated agent runs allowlisted commands, and progress is measured against an imported industry benchmark slice (HumanEval, MBPP, GSM8K, MATH, BIG-bench).

The synthesis step is now **general**: instead of resolving answers from seeded handlers, the universal 11-step loop **derives** them by composing decomposed sub-results over the links network. The benchmark suite makes this concrete — it grew to a 10-case slice and passes **10/10** with a `minimum_pass_count` ratchet: the solver writes the HumanEval/MBPP Python functions (synthesized from spec + tests, verified in the bounded agent workspace) and computes the GSM8K (`18`), MATH (`11`), and BIG-bench object-counting (`3`) answers, all **without per-case memorization** (each source carries a held-out paraphrased variant).

The **parity** gap surfaced by the issue [#244](https://github.com/link-assistant/formal-ai/issues/244) PR feedback — "all Rust and JavaScript logic are in sync" and "all languages are supported equally" — is now **closed** by the merged parity batch (E33-E34): the text-manipulation handler triggers from a single shared, data-driven multilingual operation vocabulary (`data/seed/operation-vocabulary.lino`) so every operation is recognised equally in `en|ru|hi|zh`, and the JavaScript browser worker derives the same synthesis/numeric/program/text answers as the Rust core, pinned by the shared fixture `data/parity/cross-runtime-synthesis.json`. With E1-E34 all merged, no vision-planning epic remains open for issue #244. See [`ROADMAP.md`](ROADMAP.md) for the gap-by-gap record.

The issue [#349](https://github.com/link-assistant/formal-ai/issues/349)
reverse-sort roadmap is also closed: issues #355-#364 implemented the
reproduction, rule-synthesis design, active-program coreference, composable
program modifiers, rule construction for unknown program follow-ups, default-off
diagnostics, Rust/browser-worker parity, a multilingual coding-modification
ratchet, reasoning-first report behavior, and the white-box self-improvement
loop. The final epic #365 records that the original Russian dialog now produces
a `write_program` answer with reverse-sorted output instead of `unknown`, and
that the behavior is covered across runtime and benchmark surfaces.

Issue [#408](https://github.com/link-assistant/formal-ai/issues/408) extends
the same deterministic path to user-requested text and code edits: follow-up
replacement requests can target the previous assistant artifact, including a
generated code block, and the supported edit operations use the shared
multilingual operation vocabulary. The benchmark claim is manifest-backed:
PR #416 lists 48 researched sources in
`data/benchmarks/text-manipulation-suite.lino`, generates 30 deterministic local
edit variations per source, and requires every source to pass both the explicit
3-check repository-local 10% floor and the stronger 30/30 ratchet, for 1,440 of
1,440 passing checks before the branch is complete.
