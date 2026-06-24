---
bump: minor
---

### Added
- Terminal-command intent (`tryTerminalCommand`) in both the Rust solver
  (`src/solver_terminal.rs`) and the JS worker (`src/web/formal_ai_worker.js`).
  Prompts that ask to run a shell command — fenced/backtick commands, "run … in
  terminal" / «выполни … в терминале» phrasings, or an explicit leading shell
  token like `ls`/`git status` — now resolve to an `agent_suggestion` response
  that names the detected command, explains Agent mode, and offers to switch and
  grant the `shell` capability, instead of falling through to `unknown`
  (visible fix for #511, issue #513). Localized for en/ru/hi/zh.
- Three-way `Chat` / `Agent` / `Full Auto` mode radio group in the web toolbar
  and drawer, replacing the binary agent toggle. A new `mode` preference is
  persisted and the legacy `agentMode` boolean is derived from it
  (`mode !== "chat"`) for back-compat. The topbar status label now reflects the
  active mode.

### Changed
- Toolbar/drawer mode controls expose `data-testid="mode-radio"` /
  `mode-option-<mode>` and a `mode-status` label; existing e2e specs were
  updated from the old `agent-toggle` selector accordingly.
- The terminal-command response prose is no longer hardcoded in either engine.
  The four-language bodies now live in `data/seed/multilingual-responses.lino`
  under the `agent_suggestion` (Chat mode) and `agent_suggestion_active` (Agent
  mode on) intents, with a `{command}` placeholder. Both `src/solver_terminal.rs`
  (via `seed::response_for`) and the JS worker (via `answerFor`) look the
  template up and fill in the detected command, so the natural-language wording
  is sourced from seed data rather than living in code (addresses #513 review
  feedback).
- The terminal-command *trigger* vocabulary is no longer hardcoded either. The
  terminal/shell phrases, run verbs, Chinese run verbs, and leading shell tokens
  now live in the new `data/seed/terminal-commands.lino`. The Rust solver parses
  it via `src/seed/terminal_commands.rs`
  (`seed::terminal_command_vocabulary`), and the JS worker embeds a
  byte-identical inline mirror kept in lockstep by
  `experiments/issue-513-sync-worker-terminal.mjs` (the same convention as the
  operation vocabulary, #386). A `--check` mode guards the parity in CI.
- Every new terminal-command vocabulary token (shell tokens, `command-line`,
  the `agent_suggestion*` intents and their `response_*` templates) is grounded
  as a first-class meaning so the total reference-closure audit
  (`scripts/audit-total-closure.py`) stays at zero. The
  `data/seed/closure-generated-*.lino` shards were regenerated via
  `scripts/close-total.py`; the generation is idempotent.
- E2E Playwright configs now set reasonable per-test, whole-suite
  (`globalTimeout`), assertion (`expect.timeout`), and navigation/action caps in
  both `tests/e2e/playwright.local.config.js` and `playwright.pages.config.js`
  so a hung worker, server, or deployment aborts promptly instead of wedging CI
  (addresses #513 review feedback on iterating faster).
