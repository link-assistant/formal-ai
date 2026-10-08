TASK (tag TEST-AREAS): group the unit test files into area folders (R1188-U5, the last "Missing" clause).

The measure:
- `rust/tests/unit/` holds 408 files at its top level; 264 of them start with `issue_` and 26 with `pull_request_1188_`.
- Only `ci-cd`, `coding_discovery`, `docs_requirements`, `formal_ai`, `installation_conversion`, `specification`, `verifiable_task`, `web_requests` and two issue folders group them.
- R1188-U5 asks that files and directories say what they hold. A reader should find a test by the area it pins (routing, translation, memory, agentic coding, computer use, calculation, …), not by scrolling an issue-numbered list.

Run it only when no other agent is adding unit tests: the moves touch `rust/tests/unit/mod.rs` and every file at the top level.

1. **Areas by rule.** Derive each file's area from what it tests: the `formal_ai::<module>` paths it imports and its doc comment. Write the assignment as a new tree of `data/meta/rename-map.lino` (one `rename` per file with its reason), so it is reviewable data. Don't hand-pick.
   - An area needs at least 5 files; a smaller group stays where it is.
   - Name areas with full words.
2. **Apply.** Run `node experiments/formal_ai_subagent/rename-by-rule.mjs --tree <name> --git-mv`. Each area folder gets its own `mod.rs`, with `#[path]` entries as `docs_requirements/issues.rs` does.
   - The test module paths change (`unit::x` becomes `unit::area::x`). Rewrite the names in `data/meta/test-durations.lino` and in any gate that names a test by path (`grep -rn "unit::" data/meta .github scripts`). The shard plan reads those durations.
3. **Check.** Run `check_rename_map`, `check-ci-speed`, `check-file-size` and the local gates. CI compiles the result, so nothing may be renamed in the Rust items themselves, only moved.
4. **Rows.** Update the R1188-U5 row, then run the requirement pipeline and `node scripts/render-progressive-plan.mjs --write`.

Rules: the same as `tasks/generalize.md`.
- No cargo, rustc or rust-script locally.
- Claim the folder in `claims.md`.
- Log any Formal AI edits as rows T900-T919.
- Don't commit. LEAD integrates.
