### Fixed

- Request workflow-file permission for the dependency updater's scoped App
  token. With only the default token, retain complete workflow-changing updates
  as review patches, and dispatch checks for ordinary dependency PRs it can create.
- Preserve repository branch protections with immutable per-run update branches;
  retain review data rather than rewriting an existing branch or duplicating a PR.
