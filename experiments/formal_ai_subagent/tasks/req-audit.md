TASK (tag REQ-AUDIT): make sure every requirement the user stated is a drafted row in `docs/requirements/` and has an honest status. Bulk-draft the missing ones.

Sources, all of them:
1. **The user's own messages in the Claude Code sessions for this repository:**
   - `~/.claude/projects/-Users-konard-Code-Archive-link-assistant-formal-ai/*.jsonl`, six files; the two largest are 40 MB and 38 MB.
   - Stream them with a small node script under `experiments/formal_ai_subagent/` (e.g. `collect-user-messages.mjs`). Keep only entries with `type == "user"` whose content is human text: not tool results, not `<system-reminder>`, not `<task-notification>`, not agent hand-backs, not "This session is being continued" summaries.
   - Deduplicate, then write `docs/case-studies/pull-request-1188/user-messages.md`: one entry per distinct message, verbatim, with its session id and timestamp.
   - Make the script reusable: `--check` regenerates and compares.
2. **The issues and the pull request:**
   - `gh pr view 1188 --comments`;
   - every issue the PR body says it fixes (`gh issue view <n> --comments`);
   - the PR's review comments (`gh api repos/link-assistant/formal-ai/pulls/1188/comments`).
3. **The latest vision message.** It is the last entry of `user-messages.md`; its rules are summarised at the end of `experiments/formal_ai_subagent/preamble.md`. It covers:
   - generalization over specialization;
   - code-architecture-principles (`sandboxes/refs/code-architecture-principles.md`);
   - meaningful file and directory names instead of numbered parts;
   - concise, deduplicated, human-readable links notation, with `-` preferred over `_`;
   - full English words;
   - automation by rules;
   - CI jobs and steps of 15–30 minutes, long-running first, parallel at job and test level, automatically enforced;
   - docs and vision kept in sync.

Do:
1. **Extract each distinct requirement** from these sources into a table. For each, find the row(s) in `docs/requirements/*.md` that cover it (grep), and record the mapping in `docs/case-studies/pull-request-1188/requirement-coverage.md`: requirement, source (message, issue or comment), row ids, and the row's status. The script can compute the status part; the matching is your judgement.
2. **Bulk-draft the requirements no row covers.** Use one new file, `docs/requirements/issue-1188-user-requirements.md`, in the same table format as the others: ids R1188-U1 and up, status "Not delivered" or "Partial: …" with honest evidence. Group them into meaningful sections: architecture, naming, notation, CI speed, Formal AI delegation, docs sync, JS-first parity, translation, safety.
   - Where a row already exists but its text is narrower than what the user asked, add a sub-row instead of editing the old one.
3. **Re-check every row marked implemented** that maps to a user requirement. Its evidence must point at a test or gate that exists. Use grep for the test name. If the test is gone or never existed, downgrade the row to Partial with the reason.
4. **Regenerate** with the JS twins (no rust-script):
   `node scripts/assemble-requirements.mjs --write && node scripts/generate-requirement-status.mjs --write && node scripts/render-status.mjs --write && node scripts/check-requirement-status.mjs`
5. **Docs sync:** list the places in `README.md`, `VISION.md` (or whatever holds the vision), `docs/**/*.md` and `ARCHITECTURE*` that contradict the latest vision: numbered files described as the design, abbreviations recommended, `_` names prescribed in links notation, CI described as one long job, and so on. Fix the wording where it is just text. Where the code must change first, add a requirement row instead.

Rules:
- **No cargo and no rust-script locally.** Use `node experiments/formal_ai_subagent/local-gates.mjs` for gates.
- Use Formal AI (`node experiments/js_dogfood/drive.mjs --dir <dir> --steps 8 "<prompt>"`, with «» quotes) for small edits: single rows, single replaces. Ledger rows T320–T339; log its failures in `gaps.md`.
- Claim your files in `claims.md`.
- Other agents are working:
  - TEACH-F: `js/agentic` and its Rust twins.
  - CIFIX2: routing fixes.
  - CI-SPEED: `.github/workflows` and `data/meta/ci-gates`.
  - Do not edit their files.
- Do not commit.
- Report: how many distinct requirements you found, how many were already covered, how many you drafted, the downgrades, and the docs contradictions fixed or turned into rows.
