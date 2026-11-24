# Complete c55 hosted round and portable artifact repair

PR: https://github.com/link-assistant/formal-ai/pull/1581.

All 33 workflows on `c55a5f9c50ebe58d9bb6103ca46c685e355fcd0d` finished:
30 succeed, one explicit scope gate skips and two fail. The complete retained
logs bind 163 checks, 25 annotations and 43 diagnostic candidates to that source.
Every candidate and annotation has an explicit classification. This is not a
claim that every complete log line has been read literally.

The ten native contract groups pass. The actual Rust compile-input job also
executes the physical workspace-manifest and file-size I/O regressions; the
exact passing names, lines and raw-log digest are in `new-native-io-passes.json`.
The newly drafted production changelog parser is absent from this published
head and still requires native execution on the next head.

## Windows preload reproduction

[Workflows 44988162542](https://github.com/link-assistant/formal-ai/actions/runs/44988162542)
fails its Windows artifact round trip. Lines 2387, 2444 and 2501 report
`Cannot find module` before each actual transport attempt starts. The source
path contains backslashes in the quoted `NODE_OPTIONS` preload; Node's option
parser removes those backslashes, including the drive-path separators.

The adapter now prepares its options in Bash, changes Windows backslashes to
forward slashes and keeps the preload quoted for spaces. All three attempts
consume the same prepared output and retain their distinct attempt identities,
immutable action revision, strict digest policy and original exit outcomes.

Run `node --test --test-concurrency=1
experiments/issue-1580/artifact-download-portable-options.test.mjs`.
The original adapter produces two failures and one pass. After repair all three
cases pass: the actual Bash output and Node option parser preserve a Windows
path with spaces, an actual POSIX path with spaces loads the actual hook, and
every bounded attempt consumes the prepared options. A resolver fixture
supplies the Windows filesystem boundary locally; it is not actual Windows
execution. The next hosted Windows round trip remains required.

The unchanged eight annotation controls and eleven transport controls pass.
The maintained artifact entry passes all 20 top-level cases. Preserve the first
maintained-entry failure caused by one stale expected options string, then its
passing correction. Preserve the mistaken `file-size` gate invocation too;
the actual `check-file-size` gate passes afterward. Workflow, abbreviation,
readability and timing gates pass without compiling Rust locally.

## Other collected observations

[Layered CI 44988162770](https://github.com/link-assistant/formal-ai/actions/runs/44988162770)
contains 63 stale file-size result assertions and two actual 270-second
headroom warnings. The complete consumer repair and raise-only measured shard
weights are described in [the focused reproduction](c55-file-size-and-shard-repair.md).
All 11 local consumer suites pass; 913 complete hosted cost observations raise
55 weights and lower none. The new plan still needs actual hosted wall times.

Pipeline line 196 still records UNKNOWN crates.io publish-token authority.
This remains unresolved; a public read cannot prove publication permission.
Printed conditional source, deliberate failing probes and passing test names
are classified separately from actual emitted failures. Scope notices do not
qualify deferred production builds or heavy integrations.

The complete local inventory passes 89 entries and explicitly excludes 53
hosted-only entries, 142 total; the subsequent portable-options checks cover
its additional source changes. New hosted execution, original full-suite
budgets, applicable template practices, complete aggregate review and the
external identity/delivery/authorship criteria remain unfinished. Supplied
patches, probes and conclusions earn zero independent authorship credit.

Evidence: `experiments/issue-1580/evidence/ci/terminal-c55a5f9c/` and
`experiments/issue-1580/evidence/checks/c55-portable-artifact-options/`.
