---
bump: minor
---

### Added

- **Agentic mode now acts on simple natural-language requests** instead of
  falling to the "I could not determine…" blurb (issue #687). When Formal AI is
  driven as an agentic backend (e.g. OpenCode over the OpenAI-compatible server),
  the deterministic planner now recognises three new request classes and emits
  the appropriate tool calls for the harness to run:
  - **Factual / research questions** the symbolic engine cannot answer locally
    ("When are the next elections in the USA?", "What is the current population of
    Japan?", "Learn about it.") are routed to the client's **web-search** tool,
    then the surfaced source is **fetched** and the answer read from it
    (`src/agentic_coding/web_research.rs`). Whether a prompt warrants web research
    is decided by *asking the engine* — we search precisely what it cannot resolve
    from its own knowledge base — so it generalises across phrasings rather than
    matching fixed strings.
  - **"Report [this] on GitHub"** in natural language is turned into a real
    `gh issue create` shell tool call against the Formal AI repository, and the
    created issue URL is surfaced back to the user
    (`src/agentic_coding/report_issue.rs`). Agentic mode has no Formal AI web UI,
    so the top-bar "Report issue" button was previously unreachable.
  - **Conversational / meta questions** ("What we were talking about?") are
    answered from the message history with no tool call
    (`src/agentic_coding/conversation_recall.rs`).

### Changed

- The agentic `Progress` scan now also captures **web-search output**
  (`Progress::search_output`) so the research recipe can pick the source URL the
  search surfaced and fetch it before answering.
