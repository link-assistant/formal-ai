# Online research

Research was performed on 2026-07-15. Primary documentation was preferred
because this behavior depends on current CLI and API contracts.

## OpenCode tool contract

Source: <https://opencode.ai/docs/tools/>

OpenCode documents separate `read`, `write`, and `edit` tools. `read` returns
file contents, `write` creates or overwrites a file, and `edit` performs exact
string replacement. It also places those tools behind the client's permission
configuration. This supports Formal AI's chosen boundary: emit capability-based
tool calls and let the CLI own filesystem execution and authorization.

The locally installed `opencode run --help` additionally confirmed that `run`
supports `--continue`/`--session`, a custom provider model, JSON event output,
and automatic tool approval. The durable replay uses `--continue` to preserve
the first turn's tool history for the contextual second turn.

## OpenAI tool-calling contract

Source: <https://platform.openai.com/docs/quickstart/make-your-first-api-request>

OpenAI's current quickstart describes tools/functions as the mechanism through
which a model takes application-defined actions. Formal AI therefore returns
ordinary OpenAI-compatible tool calls rather than adding a CLI-specific side
channel. The same planner is reached by API consumers and by OpenCode's generic
OpenAI-compatible provider.

## Related repository evidence

The raw snapshots for issues 680, 681, 712, 714, and 716 are stored beside this
document. They show that explicit write/read intent, stale routing regressions,
agentic mode, and shell tools were already separate concerns. Issue 715 adds the
missing invariant: an artifact established in an earlier turn must remain
addressable without forcing the user to repeat its path and old contents.
