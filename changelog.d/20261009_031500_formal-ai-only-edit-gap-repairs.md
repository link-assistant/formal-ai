## Fixed

- Repair quoted edit payloads through the JavaScript Formal AI planner: scalar code spans preserve comma-separated backticks, copy/move clauses precede quoted edits, file paths inside literal content do not become targets, and exact quoted content retains indentation and its final newline.
- Keep objective labels inside quoted source as data, and preserve new-only source newline escapes. Both JavaScript and Rust roots share these general repairs, with focused JavaScript and native regression tests.
- Record the limits of the Formal AI only development experiment: feature-level repair synthesis still needs a parent-supplied implementation; the earlier multi-line answer truncation was not reproduced. The JavaScript source-effect replay reaches 31 of 32 Issue 1028 leaves; native compilation and the full CLI ladder remain CI checks.
