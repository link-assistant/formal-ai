# Dependency update publication

The daily updater refreshes the registry snapshot, rewrites unblocked pins,
re-resolves lockfiles and runs all three gate stages before publication. Its
scoped App token must explicitly request workflow-file write permission because
the validated update can include `.github/workflows/` changes. Contents write
permission alone cannot publish those changes.
Mint optional write credentials after the long validation stages and only when
publication is requested. Installation tokens expire after one hour; public
registry discovery uses the job's default token instead.

Optional credentials remain optional. With only `github.token`, an update that
does not change workflows may still create one PR and dispatch its checks. If
workflows changed, retain the complete patch and its digest in the
`dependency-update-review` artifact. Report publication as deferred; do not push
an unauthorized workflow edit, discard the workflow changes, publish an
unvalidated partial update, or claim that dependency updates were delivered.
This review artifact is the default-token fallback, not publication authority.

Use one immutable `dependencies/latest/<workflow-run-id>` branch per update run.
Never delete it or rewrite a previous update branch: repository rules prohibit
both operations. A bounded read-only ref check detects an existing branch on a
retry and defers publication with the full review patch. This prevents duplicate
PRs and preserves uncertain earlier publication state for inspection.

The [observed failure](https://github.com/link-assistant/formal-ai/actions/runs/41048151603/job/124037268862)
validated its dependency changes but failed to push them because its App token
had not requested workflow permission. The shared credential resolver already
supported that optional scope; this caller omitted it. Keep its existing
fallback order and do not make a new secret mandatory.
