# Issue 687 — root-cause analysis

## Observed control flow

An OpenAI-compatible request enters the server's agentic loop. The planner can
return tool calls, a final answer, or no plan. Before this work, the four issue
prompts followed one of two bad paths:

```text
user prompt
  -> no specific agentic recipe
  -> general planning text or symbolic solver
  -> unknown-answer response
  -> client has no tool call to execute
```

The client was functioning correctly: it can only execute a tool when Formal AI
emits one.

## Root cause 1: language policy was duplicated in Rust

The first implementation added per-feature arrays for words such as `report`,
`issue`, `recall`, and `research`. That made the screenshot pass, but contradicted
the repository's generalization goal: each new paraphrase or supported language
would require a code change and another ordering decision.

The deeper fix introduces semantic seed records in
`data/seed/meanings-agent-actions.lino`. Rust code asks the shared seed registry
whether a text expresses a report verb, report subject, conversation recall, or
research imperative. Multilingual behavior is therefore knowledge, not control
flow. Repository identity and report templates likewise live in
`data/seed/agent-info.lino`.

## Root cause 2: follow-ups did not share history semantics

`What were we talking about?` and `Learn about it.` are not isolated intents.
Both require resolving the current user text against prior turns. Separate
feature-specific extraction would drift and could not benefit from the
associative-memory work merged in issue #686.

Recall now delegates to `solve_with_history`, the same history-aware solver used
for contextual learning and pronoun resolution. The planner's research role can
therefore turn `it` back into the prior election topic instead of searching for
the literal pronoun or returning unknown.

## Root cause 3: progress was global instead of turn-scoped

The real Agent CLI run exposed a bug that unit-only simulation missed. Planner
`Progress::scan` searched every tool result in the session. After the initial
election research, the later `Learn about it.` turn found the old fetch result
and treated the new task as already complete.

Progress is now scanned only after the latest user message. This preserves the
multi-round state machine within one request while preventing a previous
request's search or fetch from satisfying a later request. A regression test
models two research turns in one conversation.

## Root cause 4: UI state had no complete message-command model

The web shell contained a large imperative recognizer and many direct state
setters. Several visible settings had no message route at all, including Full
Auto, thinking detail, message animation, follow-up probability, and toolbar icon
pack. Sharing the Rust planner does not automatically invoke React state setters,
so the earlier claim that WASM compilation fixed every UI action was incorrect.

`data/seed/interface-capabilities.lino` now declares preference keys, types,
phrases, enum aliases, and numeric scales. The browser loader parses that catalog
and one generic recognizer emits `set_preference` commands. Existing specialized
commands remain compatible, while the uncovered settings use the declarative
path. Playwright verifies the actual controls change.

The browser investigation found a second test-infrastructure root cause:
`bun --cwd ../.. run build:web` prints help and exits successfully with the
installed Bun version. Playwright consequently served a stale committed bundle.
Changing the command to `bun run --cwd ../.. build:web` makes every run rebuild
the application before serving it.

## Why the solution generalizes

| Concern | Previous extension point | New extension point |
| --- | --- | --- |
| Report/recall/research wording | Rust arrays and branches | Links Notation meanings |
| Repository/report metadata | Code literals | `agent-info.lino` |
| Contextual recall and `it` | Separate feature logic | Shared `solve_with_history` |
| Research source choice | First URL found | Deterministic official-domain ranking |
| Repeated tool workflows | Whole-session scan | Latest-user-turn progress scope |
| UI preference commands | One branch per phrase | Typed seed capability catalog |

The code still performs deterministic orchestration: it binds seed meanings to
advertised client capabilities, quotes shell data safely, and advances search →
fetch → cited answer states. The linguistic and configurable parts are data.

## Environment coverage

| Surface | Verification |
| --- | --- |
| Native OpenAI-compatible server | Unit planner tests and release-server Agent CLI E2E |
| Agent/OpenCode-compatible client | Four continued real `agent` invocations; shell action observed |
| Browser/desktop solver worker | Shared Rust planner and embedded seed bundle |
| Browser/desktop React shell | Chromium test against actual settings controls |
| CI/release | Agent CLI script is a release-workflow step; Playwright spec is in the local/CI matrix |

No upstream defect remained after these paths were exercised, so no external
issue was warranted.
