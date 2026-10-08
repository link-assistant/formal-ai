TASK (tag RENAME): meaningful file and directory names, applied by rule, not by hand.

The user's rule (vision section of preamble.md):
- A nice directory and file tree, split into meaningful categories instead of parts or pages wherever possible.
- Fewer file names that contain numbers or anything that is not self-explanatory.
- Full English words.
- Follow https://github.com/link-foundation/code-architecture-principles (cached at `sandboxes/refs/code-architecture-principles.md`): modularity, high cohesion, clear naming, bounded contexts.

Out of scope:
- Evidence logs (`dev/log/**`, `docs/case-studies/**` raw data) are historical records, not names we design.
- Changelog fragments keep their date names.

Do, in this order, one tree at a time, each as a separate rule application:
1. **The rename tool.**
   - `experiments/formal_ai_subagent/rename-by-rule.mjs` reads a rename map in links notation, `data/meta/rename-map.lino` (one `rename` link per old → new path, with a `reason`).
   - It moves the files with `git mv`.
   - It rewrites every reference across the repository: import and require paths, `mod` declarations, `include_str!`/`include_bytes!`, workflow paths, seed lists, budgets, docs links and test fixtures. It matches the exact path, or the module name in a Rust `mod` or `use` path.
   - `--check` verifies that no old path is still referenced anywhere outside the map, and that every new path exists.
   - It must handle the JS ↔ TS mirrors (`ts/`), the `rust/embedded` mirrors, the worker module list (`js/worker-modules.js`), the line-budget shards (`data/meta/worker-line-budget/`) and the self-AST census paths (`data/meta/self-ast/`).
2. **Browser worker modules:**
   - `js/worker/formal_ai_worker_00.js` … `_21.js` and their `ts/` twins: name each by what it holds (read the module), e.g. `seed-hydration.js`, `answer-composition.js`.
   - Prefer folders by category, e.g. `js/worker/routing/`, `js/worker/answers/`, `js/worker/seed/`, where a cluster exists.
   - The load order of the classic scripts must be kept; it is recorded in `js/worker-modules.js`.
3. **Requirement outputs:**
   - `docs/requirements/assembled/part-01..03.md`, and `data/meta/requirement-status-ledger/requirements-01..17.lino`: split by meaningful category, e.g. the requirement's area or issue family, instead of by size.
   - The generators (`scripts/assemble-requirements.mjs` and its Rust original, `generate-requirement-status`) must produce the new layout.
   - Both roots must change together. The Rust scripts are checked only by CI; write them carefully and keep them rustfmt-clean.
4. **Test files named only by an issue number:**
   - Examples: `rust/tests/unit/issue_745.rs`, `rust/tests/web/issue-0918-*.test.mjs` where the suffix is not descriptive, and `rust/tests/e2e/tests/issue-*.spec.js`.
   - Name each by the behaviour it pins, e.g. `file_write_routing.rs`. Keep the issue number inside the file, in its doc comment, for traceability. Group into folders by area where a cluster exists.
   - Generate proposals from each file's first doc comment and test names into `rename-map.lino`, review them, then apply.
   - Do not rename files that other agents have claimed.
5. **Other numbered names** in `rust/examples`, `data/meta` and `docs/` that are ours: apply the same rule.

Rules:
- **No cargo and no rust-script locally.** Rust module renames must update `mod` lines and paths exactly; CI compiles them. Validate with `rustfmt --edition 2024 --check` on the touched Rust files, `node experiments/formal_ai_subagent/local-gates.mjs`, and the full web suite (`node --test rust/tests/web/`).
- LEAD commits after each tree (2, 3, 4, 5) passes, so stop and report after each one.
- Ledger rows T370–T389. Delegate the map entries' review notes and ledger rows to Formal AI.
- Do not commit.
