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
- Self-inspection **CST/AST census** recipe (`src/agentic_coding/self_ast.rs`,
  issue #538 R13): the meta algorithm parses one of its own Rust modules (the
  deterministic planner) through the repo's sole CST/AST engine — the
  link-foundation `meta-language` links network — and stores the abstract-syntax
  node census in our data as Links Notation (`data/meta/self-ast.lino`). The
  census logic is general (works on any Rust source, proven by tests over several
  sources), the Agent CLI drives it from a *fourth* differently worded request
  (`docs/case-studies/issue-538/agent-cli-session-self-ast.json`), and the
  committed artifact is reproduced byte-for-byte under test.
- `formal-ai agent --session-json <path>` to capture a replayable Agent-CLI
  session as JSON.
- Case study `docs/case-studies/issue-538` with a requirements decomposition,
  per-requirement solution plan, online research, and a `refusal-anti-pattern.md`
  recording the rejected "ship a slice, defer the rest" reasoning.
- Real Agent CLI ↔ formal-ai E2E round-trip test
  (`experiments/agent_cli_e2e/run_agent_cli.sh`) that boots `formal-ai serve`
  and drives it with the **external** `@link-assistant/agent` CLI over the
  OpenAI-compatible endpoint — no mocks. Wired as the new `test-agent-cli-e2e`
  CI job in `.github/workflows/release.yml` (running all four recipe axes —
  tomato, potato, diagrams, and the self-AST census — against the real server),
  and the real captured console log is committed at
  `docs/case-studies/issue-538/agent-cli-e2e-run.log` so the round-trip evidence
  is inspectable, not synthesised.

### Changed

- Made the Agent-CLI-driven, no-deferral development workflow the **standing
  rule** in `CONTRIBUTING.md`: from this task forward Formal AI changes are
  produced by driving the Agent CLI (never hand-editing, never deferring to
  follow-ups), with the tool extended when it cannot yet do the work. Added
  four further standing rules covering the real Agent-CLI E2E requirement,
  hardcoded cases only in tests, real captured logs in case studies, and small
  atomic commits.
- Fixed a TOCTOU race in `AgentWorkspace::for_prompt` (parallel runs with the
  same prompt shared a deterministic temp dir) via a per-instance unique
  workspace id.
- Split the meaning-lexicon seed parser into `src/seed/meanings/parse.rs` so both
  `meanings.rs` and the new module stay under the Rust file-size guard after the
  grammatical-detail additions (mirrors the existing `roles.rs` split).
- Reworked the meaning-detail recipe (`src/agentic_coding/meaning_detail.rs`) to
  **derive** every enriched surface from real, checked-in Wikidata lexeme JSON
  (parsed by a general serde_json algorithm) instead of hardcoded answer tables:
  the singular form is anchored to the lexeme's lemma and the plural is paired by
  matching non-number grammatical features, so the logic is general for any case
  paradigm and references no hardcoded case id. Hardcoded strings now live only in
  tests; the four seed blocks are reproduced byte-for-byte from the source JSON.
