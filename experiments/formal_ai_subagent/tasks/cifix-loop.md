TASK (tag CIFIX-LOOP): you are the CI fixer. You commit and push CI fixes as soon as each run reports (owner, 2026-10-08, R1188-U26). Two other agents bulk-draft requirements in the same working tree and never commit; LEAD integrates their work.

Loop until every workflow on the branch head is green, or until LEAD stops you:
1. **Find the head and its runs:**
   - `git rev-parse --short HEAD`
   - `gh run list --branch qa-reasoning-coding-bulk-fixes --limit 40 --json headSha,status,conclusion,workflowName,databaseId`, filtered to that head.

   Wait with a background poll (`sleep 120` between checks), never a busy loop.
2. **As each failed run completes, read its failed jobs:**
   - `gh run view <id> --log-failed`, or `gh api repos/link-assistant/formal-ai/actions/jobs/<job>/logs`.
   - Save the logs under `experiments/formal_ai_subagent/sandboxes/ci-<sha>/`; delete them once they are fixed.
   - Group the failures by root cause: a compile error shows up in every Rust job, so fix it once.
3. **Fix each root cause generally.** No test-specific hacks.
   - Fix JS first, then the Rust twin.
   - Check Rust with `rustfmt --edition 2024 --check` only.
   - Run only the tests next to your change (`node --test <file>`) plus the relevant `node experiments/formal_ai_subagent/local-gates.mjs --only <gate>`.
   - **Never run cargo or rust-script locally.**
4. **Commit only the files you changed for the fix**, through a private index, so that the drafting agents' uncommitted work stays out:
   ```
   export GIT_INDEX_FILE=experiments/formal_ai_subagent/sandboxes/cifix-index
   git read-tree HEAD
   git add <your files>
   git commit -F <message file>
   unset GIT_INDEX_FILE
   git reset -q
   ```
   - The message ends with the line `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`.
   - Then `git push origin HEAD:qa-reasoning-coding-bulk-fixes`.
   - `git reset -q` only moves the index back to the new HEAD, so the drafting agents' working-tree files are untouched. Never use `git stash`, `checkout --` on others' files, or `reset --hard`.
5. **Self-AST census:** when only the census tests fail, apply the census that CI regenerated:
   - run `bash experiments/formal_ai_subagent/apply-census.sh <run-id>`;
   - make a census-only commit through the private index (the paths are `data/meta/self-ast`, `data/meta/self-ast.lino`, `data/meta/self-healing-case.lino` and `docs/case-studies/issue-538/agent-cli-session-self-ast.json`).
6. **Native routing probe values:** if the probe test prints corrected `rust_misroute` lines, apply them and lower the ceilings honestly.
7. **Log each fix cycle** as a row in `docs/case-studies/pull-request-1188/formal-ai-dogfood.md` (T470–T499). Use Formal AI for the small edits where it can do them.

Report to LEAD after each push: the head, what failed, what you fixed and what is still running. If a failure is in a file a drafting agent has claimed in `claims.md`, describe it to LEAD instead of editing that file.

Disk: delete the CI logs once fixed, and never copy the repository.
