# Production release verification for PR #1188

This is observed evidence for the current integration session, not a guarantee about an unobserved future merge. The exact final PR head must repeat its required checks successfully.

| Surface | Mechanism and observed result | Remaining observation |
| --- | --- | --- |
| CLI | All five Linux, Windows and macOS CLI release jobs succeeded on remote head 81e3e25eb. The release manifest includes their checksums and provenance. | Repeat on the final head; verify the published next-version archives. |
| Desktop | Linux x64/ARM64 and macOS x64/ARM64 builds succeeded on that head; Windows jobs remain under monitoring. | Complete the final full platform matrix and published artifacts. |
| Full Docker | Actual GHCR and Docker Hub credential preflight succeeded with two credentials verified. Docker build and runtime job 113547862237 succeeded. | Final-head build/runtime and actual next-version registry tags. |
| Slim Docker | The required PR job builds the prebuilt native binary, exercises real server/agent writes and checks a 2 GiB limit. Its first run exposed nonexistent oven/bun:1.2.28; Formal AI repaired it to official Bun1.4.2, whose manifest contains Linux amd64/arm64. Both automated and manual release paths publish and verify the versioned slim tag before completing the GitHub release. | Final-head Docker build/runtime and next-version published slim tag. |
| Browser engine | The PR package job succeeded and produces an installable TGZ. Completed trusted main CI resolves its matching stable release and attaches this package, including when the GitHub-token release event is suppressed. | Final-head package and next-version release attachment. Registry publication requires configured npm authorization. |
| VS Code | The PR package job succeeded and produces an installable VSIX. Release attachment happens after extension checks; configured Marketplace/Open VSX publication and clean-profile install run afterward. | Final-head VSIX and actual release attachment; configured marketplace observations. |

Registry authorization is distinct from building usable artifacts. This session observed no repository-scoped Actions secrets, an unauthenticated local npm session and no npm trusted-publishing configuration variable. Organization-secret visibility is unavailable, so their existence is not inferred. An inherited NPM_TOKEN enables publishing; otherwise set FORMAL_AI_NPM_TRUSTED_PUBLISHING=true only after the package is configured to trust publish-engine.yml. Without either, the GitHub release still receives its installable TGZ and the workflow explicitly reports npm publication as unconfigured. Configured publication errors remain failures. Marketplace steps likewise report missing credentials while retaining the installable VSIX.

The [npm trusted publishing documentation](https://docs.npmjs.com/trusted-publishers/) requires an authorized package/workflow relationship; an Actions id-token permission alone does not establish it. No credential values were printed or invented.

The coordinator preserves post-merge-only observations as pending, imports the CI-generated census for the final source head, then checks every workflow on the exact remote PR head. The PR is not mergeable evidence merely because local tests or an older head pass.
