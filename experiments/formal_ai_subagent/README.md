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
| `local-gates.mjs` | Runs the CI gates that need no Rust build, one at a time: `--list`, `--only a,b`, `--match js`. Logs go to `sandboxes/gates/`. |
| `apply-census.sh` | Applies the self-AST census CI regenerated for HEAD, a commit or a run id, with the paths read from the census workflow, for a census-only commit. |
| `translation-scope.mjs` | Measures how much of any JS root the meta-language translator would translate, by module and refusal construct, without changing the committed scope. |
| `sandboxes/` | Throwaway copies of files a delegated task edits (git-ignored). |

## Routine commands

- Requirement status after a row changes: `node scripts/assemble-requirements.mjs --write && node scripts/generate-requirement-status.mjs --write && node scripts/render-status.mjs --write && node scripts/check-requirement-status.mjs`
- JS to Rust translation after a JS change: `node scripts/translate-es.mjs --write && node scripts/translate-js-rust.mjs --verify`
- Checks without compiling Rust (JavaScript twins of the rust-script gates; CI diffs each against its original): `node scripts/check-file-size.mjs`, `node scripts/check-hardcoded-language.mjs`, `node scripts/check-worker-line-budget.mjs`, `node scripts/check-minimal-core-boundary.mjs`, `node scripts/check-debt-ratchet.mjs --base origin/main`
- Local gates: `node experiments/formal_ai_subagent/local-gates.mjs`
- Building the web app (`bun run build:web`, the e2e specs) needs the bun named in `.bun-version`: an older bun bundles Mermaid with a bare `__require`, and every diagram silently falls back to its source text. Unpack that release under `sandboxes/bun-<version>/` and put it first in PATH.
- After CI regenerates the census for a pushed commit: `bash experiments/formal_ai_subagent/apply-census.sh`, then a census-only commit.

## Probe

```sh
node experiments/formal_ai_subagent/probe.mjs plan "Find all usages of add." --tools read,write,edit,bash
node experiments/formal_ai_subagent/probe.mjs edit "Replace the heading '# Title' with '# Project' in README.md."
node experiments/formal_ai_subagent/probe.mjs path "Insert … in r.lino."
node experiments/formal_ai_subagent/probe.mjs quotes "Insert the line «a 'b'» after …"
```

Formal AI can run these itself: `Run node experiments/formal_ai_subagent/probe.mjs path "…"`.
