# Issue 687 — root-cause analysis

## The symptom

Four ordinary prompts, sent through OpenCode driving Formal AI's
OpenAI-compatible server, each produced either the unknown-reasoning blurb ("I
could not determine … cannot infer a verified answer") or, for "Report", a
*description of a plan* rather than the action itself.

## The code path

In agentic mode the server calls
[`plan_chat_step`](../../../src/agentic_coding/planner.rs):

```
plan_chat_step(messages, tool_names) -> Option<AgenticPlan>
```

It returns:

- `Some(AgenticPlan::ToolCalls(..))` — emit tool calls the harness executes;
- `Some(AgenticPlan::Final(..))` — a final answer; or
- `None` — no agentic recipe matched.

On `None`, the loop falls through to the symbolic solver. For a factual, report,
research, or recall prompt, the solver has no local Links Notation rule that
answers it, so it emits the unknown-reasoning blurb. OpenCode then faithfully
renders that blurb — the harness did nothing wrong.

**Root cause:** `plan_chat_step` had no recipe for any of the four reported
request classes, so all four returned `None` and dead-ended.

Note the "Report" case is subtly different: the existing `compose_general_change_plan`
path *did* sometimes match and produced a *plan description* (a web-search plan),
which is why the screenshot shows a plan instead of a filed issue — the planner
recognised "do something" but had no recipe that actually files an issue.

## The fix

Add three deterministic recipes and wire them into `plan_chat_step` in priority
order, each disjoint from the others and from the pre-existing recipes:

1. **Report** (`report_issue.rs`) — checked first: a report verb + issue noun /
   repo reference ⇒ `gh issue create --repo link-assistant/formal-ai …`. Once the
   shell tool returns the created URL, surface it as the final answer.
2. **Conversation recall** (`conversation_recall.rs`) — a recall intent ⇒ a final
   answer summarising prior topics from history; no tool call.
3. **Web research** (`web_research.rs`) — checked just before the general-change
   fallthrough: a research imperative, or an answer-seeking question the symbolic
   engine cannot resolve locally ⇒ `websearch` → `webfetch` the surfaced source →
   answer from it.

### Why this is a generalization, not a phrase list (R1)

The web-research gate asks the deterministic engine itself:

```rust
fn engine_cannot_resolve_locally(task: &str) -> bool {
    let intent = FormalAiEngine.answer(task).intent;
    matches!(intent.as_str(), "unknown" | "web_search")
}
```

So we web-search *precisely* what the engine cannot answer from its own knowledge
base, and nothing it can. "What is the capital of France?" (intent `fact_lookup`)
still resolves locally and is **not** sent to the web — preserving the existing
`planner_ignores_non_formalization_tasks` behaviour. The report and recall
recognisers use verb+noun and topic recognisers with word-boundary matching (so
"report the file sizes" does not trip the repo reference inside "report").

## Environments (R6, R10)

| Environment | Path | Status |
| --- | --- | --- |
| Agentic CLI (OpenCode, `@link-assistant/agent`) | `plan_chat_step` (native Rust) | **Fixed here.** |
| Web / desktop browser worker | `src/web/worker/*` — `mode = "wasm worker"`, WASM-compiled from the same Rust planner | **Fixed here** automatically (shares the planner). |
| Web / desktop UI shell | `src/web/app/main.jsx` — `recognizeInterfaceCommand` (report_issue), `buildRecallReport` (recall) | Already handled these classes before this PR. |

Because the agentic planner is the single choke point and the browser worker
compiles from it, the fix applies across every environment that routes through the
planner; the JS UI shell already covered the same classes independently.

## Reproduction

[`tests/unit/issue_687.rs`](../../../tests/unit/issue_687.rs) exercises
`plan_chat_step` the way an agentic CLI does — one test per reported prompt class.
All fail on `main` (planner returns `None`) and pass with the fix.
