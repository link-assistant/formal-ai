### Fixed

- Request workflow-file permission for the dependency updater's scoped App
  token. With only the default token, retain complete workflow-changing updates
  as review patches, and dispatch checks for ordinary dependency PRs it can create.
