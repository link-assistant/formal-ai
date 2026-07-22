# Goals

This project should build a practical, inspectable symbolic assistant before it grows into a larger associative intelligence system.

## Architecture Goals

- Maintain the smallest useful seed dataset, then learn or import missing knowledge on demand.
- Store meanings, events, traces, requirements, handlers, and answer evidence in a link-native form.
- Use doublet links and Links Notation as the preferred representation for reviewable knowledge.
- Keep the dynamic type system small enough to understand and flexible enough to grow.
- Make transparent reasoning a default property of the system, not a diagnostics afterthought.
- Preserve add-only history for user messages, assistant actions, external data accesses, edits, and state projections.
- Support reusable associative packages for datasets, skills, substitution rules, and executable handlers.

## Product Goals

- Keep chat-first interaction as the default user experience.
- Support both ordinary chat mode and explicit agent mode.
- Keep all public interfaces backed by the same symbolic core: Rust library, CLI, HTTP API, Docker service, GitHub Pages demo, Telegram, and future desktop mode.
- Provide clear feedback when generated code was compiled, run, not run, or cannot be run in the current environment.
- Support isolated execution for generated code and agent actions through Docker, server sandboxes, or browser-local VM technology where practical.
- Let users inspect relevant links visually when chat is not enough.
- Let users report, correct, add, update, and delete knowledge through natural language while preserving history.

## Reasoning Goals

- Convert each user message into traceable requirements, source events, and candidate meanings.
- Search local associative knowledge first and external sources only when local knowledge is insufficient.
- Cache external source access with provenance and refresh policy rather than treating the web as untracked context.
- Split hard tasks recursively into smaller tasks that can be tested, executed, or answered.
- Learn from failed code generation, failed tests, timeouts, and review comments.
- Translate between natural languages, programming languages, and Links Notation as a language of meaning.
- Reduce contradictions by splitting overloaded names into distinct meanings when needed.

## Universal Solver Goals

- Run the same universal problem-solving algorithm for every request: greetings, identity questions, code generation, translation, source checking, math, and agent actions. The shape of the loop should not branch by domain.
- Treat formalization, decomposition, candidate generation, validation, and simplification as first-class steps that always run, even when the answer is trivially known.
- Generate at least one executable test or constraint check per requirement before committing to an answer, following test-driven development.
- Reuse known parts of prior solutions before generating new ones; record reuse as `cache_hit:` evidence with a link back to the prior trace.
- When no reusable part exists, combine reasoning, random search, and evolutionary search according to the available compute budget instead of giving up.
- After verification, apply meaning-preserving transformation rules to shorten the answer and pick the smallest sufficient form.
- Surface execution failures as `trace:execution_failure` links instead of retrying silently.

## Configurable Solver Goals

- Expose every solver behavior through a small `SolverConfig` so the same engine can be tuned per surface or per user.
- Make formalization preference configurable: how often to guess the formalization vs. ask the smallest clarifying question.
- Make context sensitivity configurable: how aggressively earlier messages and recent events influence the current formalization.
- Make questioning rigor configurable: how strict clarifying questions are before accepting a requirement as formal.
- Make decomposition depth configurable so the loop does not spin forever on unsolvable problems.
- Make agent mode opt-in by configuration, off by default, and only active in environments that declare isolation.
- Make diagnostic visibility, offline mode, and source cache TTL configurable from environment variables and CLI flags.
- Keep the algorithm deterministic for a given config: "random guessing" must be seeded from the impulse content hash so the same prompt produces the same answer.

## Append-Only Event Log Goals

- Record every step of the universal solver as an append-only event in the in-process log before the answer is built.
- Tag each event with a stable, content-addressed id so users can reference and inspect it from chat.
- Treat the user-facing answer as a projection of the event log, not the system of record.
- Let users chat over the log: "why did you answer that?", "what do you know about X?", "list facts I have contributed", and "export the network" should all be answerable from the log.
- Refuse "forget X" requests by default; require an explicit retraction protocol that itself appends a `retraction` event.
- Tolerate concurrent reasoning steps by guaranteeing that the projected prefix never shrinks.

## Chat-Over-Experience Goals

- Make the recorded experience the assistant's memory: any link in the network should be addressable from chat by id or natural-language description.
- Allow users to query reasoning by step: "what was step 4 of your last answer?" must return the corresponding event.
- Allow users to query reasoning by domain: "show me every external source you fetched today" must filter the event log.
- Allow users to contribute facts that become first-class events with the user as the source, attributable for later review.
- Keep diagnostic output off by default; require an explicit opt-in flag or message prefix before flooding the chat with internal links.

## World-Model Goals

- Maintain a current-state context and a target-state context as links networks throughout every dialogue, and expose their difference on request.
- Keep the user and the agent synchronized on the target state through an explicit, append-only confirmation loop.
- Support merging and splitting context models, where every context is a links network — never embeddings.
- Treat statements as dependent under relative-meta-logic: any change recalculates the probabilities of dependent statements, visibly.
- Predict the consequences of a candidate action as a hypothetical context before executing it, and compare that context against the target state.
- Persist meta-language expressions with usage (read) and change (write) counting derived from incoming and outgoing links, so frequently used or changed knowledge persists longer.

## Agent Orchestration Goals

These are target capabilities. The current implementation can serve Agent CLIs
and execute a bounded caller-supplied recursive task tree, but it does not yet
dispatch arbitrary decompositions to every external CLI or run parallel
portfolios autonomously.

- Serve as an OpenAI-compatible backend that any agentic CLI (codex, opencode, gemini, qwen, claude, agent) can drive, with tools selected by formalized intent rather than phrasing.
- Act as an orchestrator that drives those same agent CLIs as permissioned, isolated tools: dispatch decomposed sub-tasks, capture full sessions as append-only evidence, and verify results with generated tests.
- Dispatch the same sub-task to multiple agents in parallel when configured, compare the verified results in a recorded ledger, and select the winner deterministically.
- Complete the self-coding chain: Formal AI codes itself via Agent CLI, directed by Hive Mind, with every change landing as a reviewed pull request.
- Keep every UI action, setting, and capability reachable through natural language in every environment, including agentic mode where no Formal AI UI exists.

## Self-Evolution Goals

These are ratcheted targets, not assertions of present autonomy. Report upload
currently stages verified traces for review, and approved exact lessons can be
recalled live; promotion remains explicitly benchmark- and human-gated.

- Grow capabilities through a closed learning loop: frontier detection (unknown intents, failed benchmarks, trending questions) → candidate knowledge/rules with generated tests → benchmark-gated promotion as reviewed seed edits — never silent self-modification.
- Prove every adopted item with a before/after capability pair, including held-out paraphrases, so learning is generalization rather than memorization.
- Predict likely next user requests per topic from symbolic transition records and pre-learn what they need while idle, under the existing consent and priority rules.
- Generate multiple independent candidate drafts per hard task, select by test oracle with a least-action tie-break, and record why the winner won.
- Measure the share of each release authored by Formal AI itself, starting honestly at 0% and ratcheting upward.

## Documentation Goals

- Keep issue requirements in `REQUIREMENTS.md` (alongside `VISION.md`, `GOALS.md`, `NON-GOALS.md` at the repository root).
- Keep deep issue analyses in `docs/case-studies/issue-{id}`.
- Preserve raw GitHub data and relevant logs beside each case study.
- Use case studies to explain root causes, alternatives, implementation choices, and follow-up boundaries.
- Keep vision, goals, and non-goals explicit so future PRs can decide scope consistently.

## Near-Term Goals

- Expand the link-native dataset from current greetings, identity, demo dialogs, and hello-world examples into richer requirement, trace, and source records.
- Introduce first-class trace links for commands, outputs, failures, and generated answers.
- Implement a local link-store-backed reasoning loop that can read and write the same knowledge used by CLI, API, web, and Telegram surfaces.
- Add a network visualization mode that starts from the links most relevant to the current dialog.
- Keep every new behavior covered by focused unit, integration, or e2e tests before expanding scope.
- Implement the universal solver and append-only event log inside the existing Rust engine so every chat answer surfaces the full evidence-link namespace expected by the full-scope test suite.
- Promote knobs from `SolverConfig` to environment variables and CLI flags so the same engine can be operated in chat, agent, and offline modes without code changes.
- Cover each step of the universal solver with explicit unit tests, including the failing tracked requirement tests under `tests/unit/specification/`, and graduate them out of `#[ignore]` as the implementation lands.
