---
bump: minor
---

### Added
- `experiments/formal_ai_subagent/`: the coding agents' shared preamble, claims, gap list, task prompts, planner probe (`probe.mjs`) and git-ignored sandboxes now live in the repository, so Formal AI can read, edit and run them as a subagent on its own requirements (R1026).
- Formal AI line operations in both runtimes: numbered lines and ranges ("delete lines 2-3", "insert … after line 4"), a quoted line together with the line directly above or below it, moving a line (top, bottom, after or before an anchor), and swapping two lines.
- Formal AI replace and setting semantics: "change K from A to B" (including declaration lines such as `const K: T = v;`), "bump the version to …", whole-line replacement for "replace the line …", several lines or several insert clauses in one request, and "after the line Y that follows Z".
- The debug session records a Mermaid flowchart and the Rust and JavaScript source location of each stage's handler. The VS Code view renders the diagram (sanitized) and shows both source panes, and a held turn is neither persisted nor answered until it is released (R383, partial).

### Changed
- Two more handlers (`calendar_reasoning`, `calendar_create_event`) read their weekday names, parser vocabulary and sentences from seed data (R918-2: 48 migrated, 23 pending).

### Fixed
- Destructive Formal AI edits: an "Add … to <file>" description, a routed write in a removal request, an append of a block, and a replace scoped by "the line it follows" no longer overwrite the whole file. A computed change that would drop most of a file is refused.
- Diagram routing words inside a quoted payload no longer send an edit to the diagram recipe.
