# Vision

Formal AI should become a symbolic assistant whose live state is an associative operational space. The assistant should not be defined by hidden neural weights or by a large memoized answer table. It should be defined by an inspectable associative network of links: user messages, source data, inferred meanings, commands, test results, generated code, failures, permissions, and final answers.

The associative network is the AI. The runtime should activate, extend, query, and simplify that network as work happens.

## Core Idea

The system should prefer deep understanding of user needs, intent, context, and available evidence over answer memoization. A prompt should trigger enough data collection and reasoning to justify the response for that prompt. What the system learns along the way should remain available as reviewable knowledge, with source links and execution traces attached.

The long-term store should be link-native:

- Links Data Store is the operational database for meanings, history, rules, traces, and executable associations.
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

These knobs are deterministic: the same prompt with the same config produces the same answer. "Random guessing" is seeded from the impulse content hash so reproducibility is preserved.

## Append-Only Event Log

Every action the algorithm takes is appended to an in-process event log before the answer is built. Each event carries a kind (`impulse`, `language_detected`, `local_search`, `external_search`, `sub_impulse`, `candidate`, `validation`, `policy`, `agent_action`, `cache_hit`, `source`, `trace`, `error`) and is identified by a content-addressed id. The event log is the system of record; the answer and the evidence links are projections of it. The user can chat over this log:

- "Why did you answer that?" returns a meta-explanation built from the most recent trace.
- "What do you know about X?" returns the links involving X.
- "List the facts I have contributed" filters by user.
- "Forget X" is refused unless the explicit retraction protocol is used, because the log is append-only.
- "Export the network" returns the Links Notation snapshot of the seed dataset plus the visible event log.

## Computation Model

Formal AI should use trigger-style computation over links. A trigger can react to insertion, update, deletion, or a matched pattern in the network. Substitution rules should be first-class knowledge and should be able to express reads, writes, transformations, and simplification passes. These rules may be implemented as Rust code, as external handlers, or as link-native template substitutions.

Deep.Foundation is a useful reference for associative packages, handlers, permissions, and code stored inside associative memory. This project should adapt those ideas to local Rust, browser, and CLI modes using Link Foundation doublets instead of triplet links.

## Product Shape

The same symbolic core should be available through:

- Rust library API.
- CLI chat and dataset commands.
- OpenAI-compatible HTTP API surfaces.
- Docker-ready microservice.
- GitHub Pages chat demo backed by a Rust WebAssembly worker.
- Telegram private and public chat surfaces.
- Future desktop and embedded agent modes.

Code-generation tasks should be a first focus area. The assistant should generate algorithms in popular languages, compile or run generated code when the environment supports it, report execution limits honestly, and preserve logs for failed reasoning or failed execution. Browser-only mode can start with JavaScript evaluation and later experiment with WebVM.

## Meaning And Identity

For every uniquely defined concept, the system should converge on one meaning link. If the same name points to two different meanings, the system should split them into separate concepts and record why. The network should remain dynamically growing and incomplete, but it should actively reduce contradictions as new evidence arrives.

Natural languages and programming languages should be translated through link-native meanings rather than through one-off text rewrites. Links Notation should act as an intermediate language of meaning for explanations, code generation, data imports, and cross-language translation.

## Current Direction

The current repository is a proof of concept. It already has deterministic rules, Links Notation seed files, OpenAI-shaped API responses, a static web demo, Telegram support, execution metadata for simple code examples, and case-study documentation.

The next step is to keep the implemented surfaces small while moving more of the assistant's behavior into explicit links: requirements, source facts, traces, prompts, handlers, permissions, tests, and reusable problem-solving procedures.
