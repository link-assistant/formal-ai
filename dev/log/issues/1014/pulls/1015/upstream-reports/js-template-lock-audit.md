## Reproduction

At commit `77b8f1b520fde96f9a65a0fd7b5e5a5c9d1046d3`, the template has committed
npm locks and `actions/dependency-review-action@v5`, but no required audit of
the complete current locks. Dependency review only evaluates dependency
changes in a pull request; it does not make an advisory disclosed after merge
fail a later build.

To demonstrate the gap, check out the commit and run:

```bash
npm audit --package-lock-only --audit-level=high
```

If the current lock contains a high advisory, this command exits non-zero while
the existing install/test workflow can still succeed (npm's install-time audit
summary is not a fail-closed gate).

## Workaround

Run the command locally and from a scheduled workflow until the template owns a
required gate.

## Suggested code fix

Add one explicit `npm audit --package-lock-only --audit-level=high` gate for
every committed npm lock, run it on pull requests, pushes, manual dispatch, and
a schedule, and add a workflow test proving every lock is covered. Keep normal
`npm ci` steps on `--no-audit --no-fund` to avoid duplicate non-gating output.

