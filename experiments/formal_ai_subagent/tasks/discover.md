TASK (tag DISCOVER): move the R1165 rows forward. They live in `docs/requirements/issue-1165*.md`; grep for `R1165-`. The aim is that writing a program comes from discovered procedures, not memorized programs.

Open rows:
- **R1165-1:** a `WriteProgram` cache miss runs `research_coding_skill_gap`, so the answer is discovery-backed.
- **R1165-2:** `plan_work_item_execution` uses the same miss path and never emits `planned_not_executed` for a recognised pair.
- **R1165-4:** delete `hello-world-programs.lino` and `ORACLE_SNAPSHOTS`; `knows_language` becomes "a grammar exists and discovery found a procedure".
- **R1165-6:** no language row hard-codes a compile command once a verified procedure reproduces it.
- **R1165-8:** the no-memorization gate fails on any verbatim per-language Hello World string in `data/`. Today it is a ratchet in `rust/tests/unit/coding_discovery/no_memorization.rs`.

Do, JS first:
1. Find the JS twins of the `WriteProgram` path and of the procedure cache (`js/worker`, `js/server`, `js/agentic`). Where none exists, write one, so the browser and the server answer from the same procedure cache data.
2. Seed a verified procedure cache: data rows with the source shape, the run/compile command and provenance, for the languages whose Hello World answers today come from the snapshot or memorized seed. Each row must be reproducible from its recorded discovery, not hand-copied output. Then retire the snapshots: delete `hello-world-programs.lino` and `ORACLE_SNAPSHOTS` once nothing reads them. Lower the R1165-8 ratchet toward 0, and make it a hard gate if it reaches 0.
3. Make the compile commands in `rust/src/coding/catalog/languages.rs` come from the verified procedure rows where one exists (R1165-6), with the JS twin reading the same data.
4. Add tests in both roots: `rust/tests/web/*.test.mjs` with `node --test`, and Rust tests checked with rustfmt only.
5. Update each R1165 row honestly. Mark a row implemented only when tests pin it in both roots. Ledger rows T260–T269.

Rules:
- **No cargo or any Rust build locally.** Check with `rustfmt --edition 2024 --check` only, and run `node experiments/formal_ai_subagent/local-gates.mjs --js-only` before you report.
- Files must stay ≤1500 lines, Rust ≤1000. No hardcoding: vocabulary and data go in `data/seed` or `data/meta`, mirrored to `rust/embedded` where the mirror check requires it.
- Use Formal AI for small edits: `node experiments/js_dogfood/drive.mjs --dir <dir> --steps 8 "<prompt>"`, with «» quotes and never escaped quotes. Log every failure in `experiments/formal_ai_subagent/gaps.md`.
- Claim files in `experiments/formal_ai_subagent/claims.md` first.
- Other agents are working:
  - ROUTE3 owns the routing probes, `program_requests` and `code_examples` worker modules.
  - MIGRATE4 owns the `handler-rules` / `nl_tool` / translation handlers.
  - Coordinate through claims and never revert their changes.
- Do not commit.
