# Non-Goals

These boundaries keep the project aligned with the symbolic, link-native direction.

## Runtime Non-Goals

- GPU-required neural inference is not a project target. The only sanctioned exception is the experimental, strictly opt-in small-model formalization fallback of issue [#483](https://github.com/link-assistant/formal-ai/issues/483): off by default, never loaded unless requested, downloaded on demand, limited to choosing among symbolically generated options that unit tests confirm — LLMs are never in control, never at the steering wheel.
- A memoized answer cache is not a substitute for reasoning from source data and traceable steps.
- Hidden autonomous actions are not acceptable in chat mode.
- Unbounded reasoning loops are not acceptable; long-running agent work must expose actions and logs.
- Unverified generated code should not be presented as tested.
- Silent execution failures should not be hidden from users.
- Browser-only mode should not claim host-level execution capabilities.

## Universal Solver Non-Goals

- The universal solver should not skip steps for "easy" prompts to look fast; every prompt walks the same loop so the trace is comparable across requests.
- Faking the evidence trail is not acceptable. Each `impulse:`, `search:local`, `search:external`, `sub_impulse:`, `candidate:`, `validation:`, `trace:`, `cache_hit:`, `source:`, `policy:`, `agent_mode:`, and `error:` link must correspond to a real recorded event.
- Memoizing answers from a static table is not a substitute for re-running formalization, decomposition, and validation; cache hits must be recorded explicitly and link to the prior trace.
- Hiding decomposition behind an opaque rule is not acceptable; every sub-impulse must be a first-class event the user can inspect.
- The append-only log must not be rewritten or pruned silently. Retractions append new events that supersede earlier ones.
- Bypassing `SolverConfig` for hard-coded behavior is not acceptable; new knobs are added to the config first, then consumed by the engine.
- Randomized candidate generation must not become a hidden source of non-determinism; the same prompt with the same config must produce the same answer.

## Append-Only Event Log Non-Goals

- The event log is not a debug-only stream. It is the system of record, and the user-facing answer is a projection of it.
- The event log is not a place for unbounded growth: events are content-addressed, deduplicated, and bounded by `max_decomposition_depth` and the source-cache TTL.
- The event log should not leak secrets. Bearer tokens, API keys, and personal data must be redacted before they reach a link.
- Reading the event log should not require a separate database; it must remain inspectable from CLI, HTTP, and chat surfaces.

## Data Non-Goals

- A large preloaded database is not the first objective.
- Vendoring massive public datasets into the repository is not a goal.
- Opaque binary knowledge stores are not enough unless paired with reviewable Links Notation exports.
- Destructive memory updates should not erase history by default.
- External web/API access should not become untracked context.
- Duplicate names should not be forced into one meaning when evidence shows different concepts.

## Product Non-Goals

- The visual graph is not meant to replace chat as the primary interface.
- The GitHub Pages demo is not expected to become a full production backend.
- Telegram support is not meant to hide environment limits or require unsupported execution features.
- The desktop app path is not a separate product until the library, API, and local data boundaries are stable.
- Agent mode is not intended for unsafe use on personal or production systems without isolation.

## Documentation Non-Goals

- Case studies should not become marketing pages.
- Vision documents should not imply that all long-term architecture exists today.
- Requirements should not be marked complete until there is implementation evidence or an explicit scope boundary.
- Research notes should not copy large external texts; they should summarize and cite sources.
