# Pull request #985 — persisted-memory compatibility contract

Pull request: <https://github.com/link-assistant/formal-ai/pull/985>
Issue: <https://github.com/link-assistant/formal-ai/issues/982>

## Provenance

`raw-data/github/` is the machine-generated capture from
`formal-ai github-logs collect`: PR metadata, diff at capture time, all three
GitHub comment/review surfaces, recent repository context, and `manifest.json`.
The issue-side research, root-cause analysis, requirements map, operator
protocol, and Agent-CLI evidence live in `docs/case-studies/issue-982/`.

## Review scope

The PR introduces an additive schema-2 root marker and an explicit migration
transaction. Reviewers should verify these boundaries:

- `upgrade-status` performs reads only and returns nonzero structured JSON for
  incompatible data;
- normal startup/read/write paths retain a schema-1 representation rather than
  upgrading it implicitly;
- migration holds the shared writer lock across backup, validation, staging,
  atomic rename, parent sync, and receipt;
- the byte-exact backup exists and verifies before commit, and interruption
  leaves the source unchanged and retryable;
- unknown event metadata, identifiers, ordering, and history survive;
- the Docker CI leg really uses the last released and candidate images with one
  named volume and proves rollback/reopen;
- `/health` exposes the stable compatibility contract without migrating.

The complete requirement-to-test mapping is in `REQUIREMENTS.md` rows
R982-1..R982-11. No UI layout changed, so before/after screenshots do not apply.
