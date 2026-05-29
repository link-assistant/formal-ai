# Example — Syntax highlighting & copy buttons (issue #330)

This example accompanies [issue #330](https://github.com/link-assistant/formal-ai/issues/330)
and its [case study](../../docs/case-studies/issue-330/README.md). It shows the
two pieces the feature adds to the chat UI and **how to run and test them**:

1. `list_files.rs` — the exact Rust program the agent returns for *"list the
   files in the current directory"*, carrying inline run/test instructions.
2. `highlight-demo.mjs` — a runnable Node script that drives the same
   dependency-free highlighter the browser uses (`src/web/syntax-highlight.js`)
   and prints the token HTML for several languages.

## 1. Run & test the generated program

The generated program is plain Rust — no crate, no `Cargo.toml` required:

```sh
# Compile and run it against the current directory:
rustc examples/issue-330-code-highlighting/list_files.rs -o /tmp/list_files
/tmp/list_files

# Run it against any other directory by passing a path argument:
/tmp/list_files /etc

# Quick test: it should print this file's name when pointed at this folder.
/tmp/list_files examples/issue-330-code-highlighting | grep list_files.rs
```

These are exactly the "Run command" / "Check command" lines the engine appends
to the localized answer (see `src/engine.rs` → `execution_command_lines`), so
the example stays in sync with what the agent tells the user.

## 2. Run the highlighter demo

The highlighter is a plain script that attaches `FormalAiHighlight` to the
global object, so Node can load it directly:

```sh
node examples/issue-330-code-highlighting/highlight-demo.mjs
```

Expected output: for each language it prints the resolved grammar name and the
HTML token spans (e.g. `<span class="hljs-keyword">fn</span>`), proving the same
highlighting the browser renders is produced deterministically and with no
dependencies.

## 3. Exercise the feature in the real browser (e2e)

The end-to-end tests build `src/web`, serve it, send the Russian Rust prompt,
and assert the rendered block is highlighted and both copy buttons work:

```sh
cd tests/e2e
npm install            # first time only — installs Playwright
npx playwright test --config playwright.local.config.js issue-330
```

All three tests should pass:

- *renders a syntax-highlighted, copyable code block*
- *copies the whole message as Markdown*
- *highlights every seeded program language* (python / go / ruby)
