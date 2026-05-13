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

## Documentation Goals

- Keep issue requirements in `docs/REQUIREMENTS.md`.
- Keep deep issue analyses in `docs/case-studies/issue-{id}`.
- Preserve raw GitHub data and relevant logs beside each case study.
- Use case studies to explain root causes, alternatives, implementation choices, and follow-up boundaries.
- Keep vision, goals, and non-goals explicit so future PRs can decide scope consistently.

## Near-Term Goals

- Expand the link-native dataset from current greetings, identity, demo dialogs, and hello-world examples into richer requirement, trace, and source records.
- Introduce first-class trace links for commands, outputs, failures, and generated answers.
- Prototype a local link-store-backed reasoning loop that can read and write the same knowledge used by CLI, API, web, and Telegram surfaces.
- Add a network visualization mode that starts from the links most relevant to the current dialog.
- Keep every new behavior covered by focused unit, integration, or e2e tests before expanding scope.
