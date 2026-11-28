# Secretlint 14 built-in preset migration

The registry marks `@secretlint/secretlint-rule-preset-recommend@13.0.5`
deprecated because Secretlint 14 includes this preset. Keeping the external
package creates an avoidable installation warning. The existing scanner adapter
also resolves that external package, so it fails against an otherwise complete
Secretlint 14 installation.

The installer now requests only `secretlint@14.0.0`. The adapter verifies the
exact Secretlint, engine, built-in core and profiler versions. It retains masked
output, disabled profiling, sequential input handling, bounded archive decoding
and the existing workflow-command boundary. The preset identifier in the
repository configuration remains supported by the built-in resolver.

Reproduction: install only Secretlint 14 into a temporary prefix, place its
`node_modules/.bin` directory on PATH, and run the previous adapter. It fails
resolving the removed external preset. The mock regression likewise fails before
the production change. The evidence directory retains both failures.

Verification: `compressed-secrets.test.mjs` passes all 12 cases; the maintained
`bounded-subprocess-boundaries.test.mjs` adapter passes 37 cases (including those
12), and `fast-pull-request-production-profile.test.mjs` passes 18. Three source
gates, shell syntax and whitespace checks pass. With `SECRETLINT_TEST_BIN` set
to the temporary binary directory, run
`node experiments/issue-1580/secretlint14-live-check.mjs`: the actual engine
accepts clean content and rejects masked GitHub/Slack fixtures in plain and gzip
files. Its CLI rejects an actual finding while disabling workflow commands
around the untrusted filename. The final probe also rejects inventory/file
errors as substitutes for a finding.

Registry manifests and bounded tarballs were integrity checked before API
inspection. The committed evidence omits installed dependencies and tarballs.
A scratch fixture hash assertion failed after starting verification; its
corrected fixture and original failure are retained without rerunning the
finished test worker. These focused checks do not complete issue #1580, the
full JavaScript template comparison, or hosted checks for this new change.
