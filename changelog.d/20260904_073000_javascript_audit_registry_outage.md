---
bump: patch
---

### Fixed
- `check_javascript_dependencies` no longer reports a registry outage as a dependency finding. In run 100928011479 `Lint and Format Check` went red on a branch that touched no lockfile: `bun audit` spent five minutes inside one request and exited with `error: POST https://registry.npmjs.org/-/npm/v1/security/advisories/bulk - 503`, so npmjs.org had said nothing at all about `bun.lock` and the branch wore the result anyway. An unanswered registry is now retried (`FORMAL_AI_AUDIT_ATTEMPTS`, `FORMAL_AI_AUDIT_RETRY_DELAY_SECONDS`), while a registry that answers -- with an advisory, or with anything the script does not recognise as a transport fault -- still ends the gate on its first attempt, because retrying an answer only spends three times as long arriving at the same red. Retrying is not passing: an outage that outlasts every attempt leaves the lockfiles unaudited, and an unaudited lockfile is what this gate exists to refuse, so it stays closed and annotates the job with why.
