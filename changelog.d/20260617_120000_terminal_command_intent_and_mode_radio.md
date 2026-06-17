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
