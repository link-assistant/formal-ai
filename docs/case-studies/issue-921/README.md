# Issue 921: Hive-Mind Full-Circle Integration Gate

Issue [#921](https://github.com/link-assistant/formal-ai/issues/921) closes
E74 with a replayable integration gate in both directions. It builds on E69's
now-merged coding-harness foundation rather than weakening that dependency.

## 1. Collected Data

`raw-data/github/` preserves issue #921, every issue comment, prepared PR #1004
and all three PR feedback surfaces. It also contains the predecessor issues
#655 and #703, dependency E69 (#916), upstream Hive Mind issue #2059, and the
two most recent upstream implementation PRs (#2108 and #2147). There were no
screenshots or image attachments to download.

The live evidence was captured on 2026-08-13 with the versions in
`versions.txt`. Review-sized artifacts are committed under the two direction
folders. Full stdout, stderr, and server traces remain CI artifacts on failure
because a single run produces roughly 850 KiB of diagnostic output.

## 2. Requirements

`requirements.md` and root `REQUIREMENTS.md` map R921-1 through R921-5 to the
public Hive Mind invocation, the production executor, Formal AI's public
orchestrator, committed workspace effects, canonical replay, failure
propagation, and release CI.

## 3. Reproduction And Root Cause

Issue #655 proved only the inner Agent-CLI-to-Formal-AI loop. Its intended
outer command was blocked because Hive Mind rejected `formal-ai` as an Agent
model. Upstream issue hive-mind#2059 and PR #2108 added the dispatch, while PR
#2147 replaced the intermediate path with the native Agent CLI backed by a
Formal AI runtime. Hive Mind 2.12.2 now prepares this command:

```text
agent --model formalai/formal-ai --verbose
```

Formal AI already had the complementary public `agent run` and `agent replay`
surfaces from #703. The remaining defect was therefore structural: neither
repository continuously crossed the now-working outer Hive Mind boundary, and
no one gate proved the reverse direction, an observable commit, replay
integrity, and nonzero failure propagation together. A future regression could
silently restore #655's unsupported-model gap while every inner test stayed
green.

## 4. Implemented Gate

`experiments/issue_921_hive_mind_full_circle/run.sh` creates isolated Git
workspaces and boots the candidate Formal AI server.

**Hive Mind -> Agent CLI -> Formal AI.** The gate first runs the exact public
shape
`solve ISSUE_URL --tool agent --model formal-ai --attach-logs --verbose` in
command-preparation mode. This proves that Hive Mind accepts the public
arguments and selects the native Agent/Formal-AI command without mutating an
issue or pull request. It then imports Hive Mind's shipped
`executeAgentCommand` implementation and runs that same production executor
against the actual Agent CLI and candidate Formal AI server. The requested
file is byte-compared and committed in the fixture repository.

This split is deliberate and explicit: executing the top-level `solve` command
past preparation would write comments and possibly branches or pull requests
to the live issue on every CI run. The public parser/selection boundary and the
production execution boundary are both real; only the external GitHub writes
between them are suppressed.

Hive Mind 2.12.2 performs a write-permission preflight before reaching its
prepare-only exit even though that route cannot write. The release job retains
`contents: read`. A narrowly scoped `gh` wrapper returns write permission only
for that one preflight request and delegates every issue, repository, and user
read to the authenticated GitHub CLI. It is not present for the production
executor or either live Agent process.

**Formal AI -> external Agent CLI.** The gate sends a bounded acceptance payload
extracted from the committed hive-mind-shaped issue fixture through
`formal-ai agent run --cli agent --target formal-ai`. The installed Agent CLI
calls the live Formal AI server, creates the requested file, and passes an
allowlisted verification command. Formal AI saves a canonical session and
`formal-ai agent replay` validates its hash-chained event stream before the
effect is committed.

Both directions run an Agent process that exits 23. The Hive Mind driver
returns 23 and makes no effect commit; Formal AI exits nonzero and records a
failed orchestration session with the original exit code. No retry or wrapper
turns either probe green.

## 5. Verification

- `cargo test --test unit issue_921 -- --nocapture` validates every committed
  artifact, replays the canonical session with the production Rust replay
  function, and pins the workflow wiring.
- `experiments/issue_921_hive_mind_full_circle/run.sh OUTPUT` regenerates both
  directions using installed Hive Mind and Agent CLIs. It refuses a non-empty
  output directory so stale evidence cannot produce a false pass.
- `.github/workflows/release.yml` installs pinned Hive Mind 2.12.2, runs the
  full-circle harness after the E69-dependent invariant gates, and uploads its
  raw logs when any step fails.

The committed success artifacts include exact task/result bytes, full fixture
commit IDs and patches, the Hive Mind-native Agent session ID, the prepared
command, the canonical Formal AI orchestration session, replay result, and both
failure summaries. The fifth clean local capture passed both directions and
both failure probes.

## Related Work

- [formal-ai#655](https://github.com/link-assistant/formal-ai/issues/655) — the
  earlier inner-loop proof and the recorded upstream block.
- [formal-ai#703](https://github.com/link-assistant/formal-ai/issues/703) — the
  external-CLI orchestration and replay substrate.
- [formal-ai#916](https://github.com/link-assistant/formal-ai/issues/916) — E69,
  the required coding-harness foundation.
- [hive-mind#2059](https://github.com/link-assistant/hive-mind/issues/2059),
  [PR #2108](https://github.com/link-assistant/hive-mind/pull/2108), and
  [PR #2147](https://github.com/link-assistant/hive-mind/pull/2147) — upstream
  model acceptance and native dispatch implementation.
