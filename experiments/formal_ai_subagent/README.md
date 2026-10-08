# Formal AI as a subagent on its own requirements

This folder is where coding agents working on Formal AI delegate work to
Formal AI itself. Everything they share lives here, in the repository, so
Formal AI can read and edit it like any other workspace file. That
includes the gaps found, the task prompts, the claims and the probe tool.
The aim is recursive self-improvement. Each task Formal AI fails becomes a
general fix with a regression test, and the next task goes to Formal AI too.

## The loop

1. **Delegate.** Give Formal AI one plain instruction and run it in-process
   through the dogfood driver:

   ```sh
   node experiments/js_dogfood/drive.mjs --dir <dir> --steps 12 "<instruction>"
   ```

   - `<dir>` is either a sandbox under `sandboxes/` (git-ignored) holding
     copies of the files the task edits, or the repository root (`.`).
   - Use the root only for small edits to coordination files, such as a
     claim line, a gap entry or a ledger row. Running there also writes
     Formal AI's plan event to `.formal-ai/`; see gap G15.
2. **Check** the result, then copy sandbox files back.
3. **When it fails, teach it.**
   - Find the cause with `probe.mjs`.
   - Fix it generally in the JS planner (`js/agentic/`), then in its Rust
     twin (`rust/src/agentic_coding/`). Vocabulary goes in `data/seed/`,
     never hardcoded.
   - Add a regression test in both roots.
   - Add a row to
     `docs/case-studies/pull-request-1188/formal-ai-dogfood.md`, using
     Formal AI itself.
4. **Probe ahead.** Try new task shapes on sandbox copies. Record each
   failure in `gaps.md`, again by asking Formal AI (`Append the line '- G…'
   to experiments/formal_ai_subagent/gaps.md.`).

## Files

| Path | What it holds |
|---|---|
| `preamble.md` | The shared instructions every delegated coding agent reads first. |
| `tasks/*.md` | Bulk task prompts, one per agent tag. |
| `claims.md` | Who is editing what right now. Append a line before editing a file. |
| `gaps.md` | Task shapes Formal AI failed, open or FIXED with their ledger row. |
| `probe.mjs` | Shows how the planner reads a request: `plan`, `edit`, `path`, `quotes`, `read-parse`. |
| `sandboxes/` | Throwaway copies of files a delegated task edits (git-ignored). |

## Probe

```sh
node experiments/formal_ai_subagent/probe.mjs plan "Find all usages of add." --tools read,write,edit,bash
node experiments/formal_ai_subagent/probe.mjs edit "Replace the heading '# Title' with '# Project' in README.md."
node experiments/formal_ai_subagent/probe.mjs path "Insert … in r.lino."
node experiments/formal_ai_subagent/probe.mjs quotes "Insert the line «a 'b'» after …"
```

Formal AI can run these itself: `Run node experiments/formal_ai_subagent/probe.mjs path "…"`.
