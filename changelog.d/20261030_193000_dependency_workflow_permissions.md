### Fixed

- Request workflow-file permission for the dependency updater's scoped App
  token. With only the default token, retain complete workflow-changing updates
  as review patches, and dispatch checks for ordinary dependency PRs it can create.
- Preserve repository branch protections with immutable per-run update branches;
  retain review data rather than rewriting an existing branch or duplicating a PR.
- Mint optional write credentials after validation so long jobs do not exhaust
  their lifetime before publication; use the default token for registry reads.
- Bound recovery from a truncated GitHub release-create response. Authenticate
  the exact release tag and source before crediting an uncertain write, and
  require an authenticated absence before retrying, at most three times.
