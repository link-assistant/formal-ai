---
bump: minor
---

### Added

- Grammatical detail for the tomato **and potato** meanings (issue #538): every
  surface (`tomato`/`tomatoes`, `помидор`/`помидоры`, `томат`/`томаты`,
  `potato`/`potatoes`) now pins its part of speech and grammatical number
  (singular/plural) in the seed data, and the previously missing plurals `томаты`
  and `potatoes` were added.
- New `grammatical_number` semantic facet kind plus `WordForm::grammatical_number()`,
  `WordForm::part_of_speech()`, and `WordForm::denotations()` accessors.
- Grounded, multilingual `grammatical_number` / `singular` / `plural` meanings
  (Wikidata `Q104083` / `Q110786` / `Q146786`) lexicalised in en/ru/hi/zh, with
  cached Wikidata data for offline grounding-closure tests.
- The meaning-detail change is produced by **driving Formal AI through its own
  in-repo Agent CLI** (`src/agentic_coding/`), with the committed seed asserted
  byte-for-byte equal to the driver output. A concept registry generalises the
  recipe, proven by driving tomato and potato with two *differently worded*
  requests. The Agent-CLI sessions that solved the task are committed
  (`docs/case-studies/issue-538/agent-cli-session*.json`), and
  `scripts/reproduce-issue-538.sh` regenerates the change on a clean checkout.
- Generated agentic-recipe **mermaid diagrams**, split into parts
  (`docs/diagrams/agentic-recipes.md`), rendered from the planner's own recipe
  table by `src/agentic_coding/diagram.rs` — a non-lexeme axis (issue #538
  R15/R16) proving the Agent-CLI method generalises beyond meaning data. The Agent
  CLI writes the document from a *third* differently worded request; the document
  and its session JSON are reproduced byte-for-byte under test.
- `formal-ai agent --session-json <path>` to capture a replayable Agent-CLI
  session as JSON.
- Case study `docs/case-studies/issue-538` with a requirements decomposition,
  per-requirement solution plan, online research, and a `refusal-anti-pattern.md`
  recording the rejected "ship a slice, defer the rest" reasoning.

### Changed

- Made the Agent-CLI-driven, no-deferral development workflow the **standing
  rule** in `CONTRIBUTING.md`: from this task forward Formal AI changes are
  produced by driving the Agent CLI (never hand-editing, never deferring to
  follow-ups), with the tool extended when it cannot yet do the work.
- Fixed a TOCTOU race in `AgentWorkspace::for_prompt` (parallel runs with the
  same prompt shared a deterministic temp dir) via a per-instance unique
  workspace id.
- Split the meaning-lexicon seed parser into `src/seed/meanings/parse.rs` so both
  `meanings.rs` and the new module stay under the Rust file-size guard after the
  grammatical-detail additions (mirrors the existing `roles.rs` split).
