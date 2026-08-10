---
bump: patch
---

### Fixed

- Harness and server log exports from the agentic `Report` flow are written
  into a surviving temporary directory and print their final path, instead of
  dropping `formal-ai-*.lino` session dumps into the caller's working
  directory — a repository checkout root stays clean (#945).
- Report-target answers that use machine values now select every target:
  `formal_ai` was silently dropped because prompt normalization turned the
  underscore into a space before matching (#996).
- Final answers that inline machine text — the general-change plan event and
  the formalized knowledge base — wrap it in a fenced `lino` code block, so
  the text survives GitHub-comment markdown rendering instead of collapsing
  into flowing prose (#996, hive-mind #2146).
- Formalization tasks that quote their own source text («…», “…”, 「…」, 《…》)
  now formalize that text instead of silently substituting the seeded
  «Сказка о рыбаке и рыбке» tale; a quoted *title* of the tale still selects
  the full canonical text, and `FORMAL_AI_TRACE_REQUESTS=1` now also traces
  how the planner routed the received task (#956).
- The Russian liveness probes «ты тут?», «вы тут» and «я тут» are routed to
  the `test_status` intent instead of falling through to a web search (#979).
- The 22 duplicate requirement IDs in `REQUIREMENTS.md` are renumbered to
  fresh unique IDs (issue-540 block → R537–R548, issue-657 R480 → R549,
  issue-674 block → R550–R558) with every cross-reference in
  `docs/requirements-traceability.md`, the issue-540 case study, and its
  guard test updated (#964).
