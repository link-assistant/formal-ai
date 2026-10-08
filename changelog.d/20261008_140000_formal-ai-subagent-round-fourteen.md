---
bump: minor
---

### Added
- A browser end-to-end test steps a held turn through the debugger view in the built app: the question goes through the composer, the SVG diagram and both handler panes are checked, the answer arrives only after the last stage, and the view is saved as the live session capture (R383).
- `experiments/formal_ai_subagent/local-gates.mjs` runs every CI gate that needs no Rust build, one at a time. It reads the gate registry (`data/meta/ci-gates/`) and the checker steps of the standalone workflows, so coding agents and Formal AI share one gate list with CI.
- `experiments/formal_ai_subagent/apply-census.sh` applies the self-AST census that CI regenerated for a commit or a run. It reads the census paths from the workflow's upload step.

- Formal AI searches file contents ("Where is add used in this project?", "Find TODO in the repository"), runs a named test file with the runner its extension needs, checks a reported bug against its stated expectation before changing anything, lists a file's exported functions, summarizes a named file, and lists the directory a request names.
- Formal AI edits: indented blocks keep their indentation relative to the anchor; ordinal lines ("the second line", "the last line"); "with these lines:" block replacements; several quoted lines in one insert or append; "the contents of a.txt" as the text to insert; "the last 2 lines of f" and "lines 3-5 of f" answer only those lines; inserts at the end or start of a Markdown section; empty-line inserts next to an anchor; quoted file renames.
- Formal AI answers a Spanish request in Spanish when the script detector would fall back to English.
- The debugger names the code that emitted each stage in both runtimes (`data/meta/debug-stage-sources.lino`, generated from the source with a drift gate), and a turn is held before it is solved (R383 delivered).
- Self-authoring, SWE-bench and the coding ladder record the same nine-stage trace of `data/meta/repository-workspace-protocol.lino`; the authoring loop runs the document's stages (R1138-3-5, Rust; the JS runners are still missing).
- Routing: a call to an undefined function is answered as an execution failure, JSON-to-JSON conversion re-emits the parsed document, a product request with no marketplace names the catalogued ones, and the browser learns from a named source (R1173-3: browser misroutes 9 → 2).

### Changed
- Three more handlers read their vocabulary and sentences from seed data: `text_manipulation`, `document_originality_check` and `software_project_followup` (R918-2: 51 migrated, 20 pending).
- `scripts/author-change-with-formal-ai.sh` refuses to commit unless it is given `--commit`, like `formal-ai solve` (R1138-3-7). The self-authoring action passes it, and `--no-commit` still means the default.

### Fixed
- Destructive Formal AI edits: "Delete the line 'x'" removes only the line that is `x` (all lines containing it only when the request says so), a nested quote inside a quoted line no longer turns an append into a move, a list of quoted lines is appended line by line instead of overwriting the file, and a different existing file is kept unless the request consents to overwrite it.
- Shell intent cues match whole words: the `move` cue inside "removed" no longer plans `mv`.
- A quoted line in an edit request no longer chooses a shell command: "Add the line '- run tests' …" used to run the workspace tests.
- Diagnostics mode no longer crashes the web app and the VS Code debugger view: the debugger view binds the JSX factory, and a test now fails on any app file that writes JSX without it.
- The debugger view gets its own row in the chat panel, so the message list no longer covers its buttons.
- An answer that echoes a literal holding backticks now fences it with a longer CommonMark code span, so the Markdown shows it verbatim.
- The file-size gate measures only the files a commit can carry, so a git-ignored local file no longer fails it. Claude Code session exports at the repository root are ignored.
