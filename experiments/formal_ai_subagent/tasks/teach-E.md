TASK (tag TEACH-E): teach Formal AI the open gaps in `experiments/formal_ai_subagent/gaps.md` that sit in the edit composer and line operations. TEACH-D finished that lane in round 14: `positional_edit`, `write_request`, `workspace_change`, `workspace_line_operation`, `tool_result` and their Rust twins are free. Claim them in claims.md first.

Order: destructive first, then the rest.

1. **G60 (destructive):** "Delete the line 'x' from f.md." — see the entry.
2. **G61 (destructive):** an append whose quoted line itself holds a single quote.
3. **G53, G54:** several quoted line literals in one insert or append, each its own line.
4. **G50, G51:** "Append/Insert the contents of a.txt …" reads the source file and places its bytes; it never runs `cat a b`. G51 also covers a colon-introduced multi-line insert with a "that follows" anchor.
5. **G33:** "Show the last/first N lines of f" and "lines A-B of f" answer only those lines (read with offset/limit).
6. **G35:** a section-scoped insert ("at the end of the section '## Usage' in README.md"). The section ends at the line before the next heading of the same or higher level. Since G32 was fixed, this request plans nothing; it must plan read, edit, check.
7. **G37:** a rename with both paths quoted moves the file like the unquoted form (TEACH-D's G24 guard in `capability_router` is the place to look).
8. **G52:** an intermittent "Verification failed … observed bytes differ" on a 7000-character line. Find the cause (digest of the wrong bytes? a race in the driver?) and fix it, or pin it as not reproduced.
9. **G62:** a backticked compound `a && b` command runs as written.

Each fix:
- generic, with vocabulary in `data/seed`, JS first and then the Rust twin;
- a regression test in both roots;
- a ledger row in T150–T169 (Formal AI writes it);
- the gap marked FIXED by Formal AI.

TEACH-C still owns `planner.mjs` / `planner.rs` (workspace-search arm), `workspace_search`, `shell_command` (test-file hook) and `test_file_runner`; coordinate through claims before touching those. LEAD changed `intentShellCommand` / `intent_shell_command` for G32. `rust/src/agentic_coding/shell_command.rs` is at 997 of 1000 lines, so move code out before adding any.
