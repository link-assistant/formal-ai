# Issue 687 Case Study — Agentic mode does not act on simple requests

> "Asking formal AI to report the issue on GitHub does not work (as well as other
> simple prompts/requests)."

## Collected data

Raw evidence is preserved under [`raw-data/`](raw-data/):

- [`issue-687.json`](raw-data/issue-687.json) — the issue as filed.
- [`issue-687-comments.json`](raw-data/issue-687-comments.json) — issue comments
  (empty at capture time).
- [`image-urls.txt`](raw-data/image-urls.txt) — the attachment URLs in the issue
  body.
- [`pr-688.json`](raw-data/pr-688.json),
  [`pr-688-conversation-comments.json`](raw-data/pr-688-conversation-comments.json),
  [`pr-688-review-comments.json`](raw-data/pr-688-review-comments.json) — the
  prepared pull request.
- [`recent-runs.json`](raw-data/recent-runs.json) — CI runs on the branch.

The screenshot from the issue is saved at
[`images/01-opencode-session.png`](images/01-opencode-session.png).

## Timeline / sequence of events

1. The maintainer ran **OpenCode 1.17.18** as an agentic CLI, pointed at Formal
   AI's OpenAI-compatible server as the model backend.
2. Four ordinary prompts were sent in one session (see the screenshot):
   - **"When next elections in the USA?"** → the assistant replied it *"could not
     determine … cannot infer a verified answer"* (the unknown-reasoning blurb).
   - **"Report"** → the assistant described a *web-search plan* instead of
     actually filing the issue on GitHub.
   - **"What we were talking about?"** → the unknown-reasoning blurb again.
   - **"Learn about it."** → the unknown-reasoning blurb again.
3. Each of the four is a *simple* request that a human, or a neural assistant,
   would act on immediately. Formal AI, being deterministic and symbolic,
   dead-ended on all four.

## Requirements

The full, itemised requirement list extracted from the issue body is in
[`requirements.md`](requirements.md). In short:

1. Solve by **generalization** (auto-learning + contributing guidelines), not by
   hardcoding each phrase in production code.
2. Be able to **talk about the conversation** (meta / recall questions).
3. **Factual questions** → go to the internet (web search + web fetch), find
   official sources, answer the user.
4. In **agentic mode**, rely on the tools of OpenCode CLI and any other supported
   harness (Formal AI itself has no HTTP client).
5. Ability to **report an issue** to the Formal AI repository in natural language
   (agentic mode has no Formal AI web UI, so the "Report issue" button is
   unreachable).
6. **Everything in the web UI** (button, action, setting) must be actionable /
   configurable with natural language in **all** environments.
7. Download logs/data here and produce this **deep case study**.
8. Add **debug output + verbose mode** if there is not enough data to find a root
   cause.
9. If the issue relates to another repository, **report it there** with
   reproducible examples, workarounds, and fix suggestions.
10. Apply the fix across the **entire codebase** (fix in all places).
11. Plan and execute everything in the **single PR #688**.

## Root cause

Root-cause analysis is in [`root-cause.md`](root-cause.md). In short: the
deterministic agentic planner, [`plan_chat_step`](../../../src/agentic_coding/planner.rs),
had no recipe for any of the four request classes. It returned `None`, the loop
fell through to the symbolic solver, and the solver — unable to answer a factual,
report, research, or recall prompt from local Links Notation rules — emitted the
unknown-reasoning blurb. OpenCode faithfully displayed that blurb. This mirrors
the predecessor investigation for issue #676, which likewise concluded the
harness (OpenCode) was **not** at fault.

## Implemented path

Three new deterministic planner recipes, each emitting the tool calls the harness
executes, or a final answer read from the conversation:

| Request class | Module | Behaviour |
| --- | --- | --- |
| Factual / research question | [`web_research.rs`](../../../src/agentic_coding/web_research.rs) | `websearch` → `webfetch` the surfaced source → answer from it. Fires only when the symbolic engine cannot resolve the prompt locally, so it is a genuine generalization, not a phrase list. |
| "Report [this] on GitHub" | [`report_issue.rs`](../../../src/agentic_coding/report_issue.rs) | `gh issue create --repo link-assistant/formal-ai …`, then surface the created issue URL. |
| "What were we talking about?" | [`conversation_recall.rs`](../../../src/agentic_coding/conversation_recall.rs) | Final answer built from the message history; no tool call. |

The recipes are wired into `plan_chat_step` in priority order (report → recall →
… → web research), disjoint from the existing recipes. Because the browser worker
is **WASM-compiled Rust** (`mode = "wasm worker"`), the same logic reaches the web
environment automatically; the web UI additionally already recognises "report
issue" and recall via `src/web/app/main.jsx`.

## Reproduction test

[`tests/unit/issue_687.rs`](../../../tests/unit/issue_687.rs) drives
`plan_chat_step` exactly as an agentic CLI would, one test per reported prompt
class. Every test fails on `main` (planner returns `None` → no tool call) and
passes with the fix.

## Online research

Corroboration of the factual example (the next US general election is **November
3, 2026**) and a survey of related components is in
[`online-research.md`](online-research.md).

## Upstream

Whether any other repository (OpenCode) is at fault is assessed in
[`upstream.md`](upstream.md). Conclusion: no upstream bug; the gap was in Formal
AI's own agentic planner.
