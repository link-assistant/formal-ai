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
folders. The real Hive Mind executor and public preparation logs are preserved
there as deterministic gzip files, alongside the canonical Formal AI success
and failure sessions. The high-volume server trace remains a CI artifact on
failure because a single run produces roughly 400 KiB of server diagnostics.

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
executor or either live Agent process. Hive Mind also resolves pinned helper
packages lazily; the harness gives npm a disposable prefix inside its scratch
directory so an unprivileged CI runner never tries to write `/usr/local`. The
disposable prepare clone also receives a repository-local Git identity, which
satisfies Git operations without changing the runner's global config. Because
Hive Mind checks identity before entering `--working-directory`, the same
identity is passed as process-local Git config only to the prepare command.
GitHub Actions checks out pull requests at a detached HEAD, so the disposable
clone also materializes a local `main` branch at that exact candidate commit
and a matching disposable `origin/main` ref before Hive Mind asks for the
current branch and creates its temporary solution branch.

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
- `.github/workflows/release.yml` installs pinned Hive Mind 2.12.5, runs the
  full-circle harness after the E69-dependent invariant gates, and uploads its
  raw logs when any step fails. The committed evidence below was captured
  against 2.12.2; the pin moved to 2.12.5 because that is the first release
  carrying [hive-mind#2159](https://github.com/link-assistant/hive-mind/pull/2159),
  the boundary fix this gate must run against (see section 6).

The committed artifacts include exact task/result bytes, full fixture commit
IDs and patches, the Hive Mind-native Agent session ID, compressed executor and
public preparation logs, the prepared command, the canonical Formal AI success
and failure sessions, replay result, and both failure summaries. The sixth
clean local capture passed both directions and both failure probes.

## 6. The Two Defects The Gate Was Not Enough To Catch

A gate proves the *transport* works: Hive Mind selects Formal AI, the Agent CLI
runs, a workspace effect lands, and a failure propagates. Everything in sections
3 to 5 passed while Formal AI still could not do the work Hive Mind dispatched
to it. That is the gap [hive-mind#2158](https://github.com/link-assistant/hive-mind/issues/2158)
reported, with a 21-log production bundle preserved in
[hive-mind#2159](https://github.com/link-assistant/hive-mind/pull/2159):

| Client / language | Observed | Requested artifact |
| --- | --- | --- |
| Agent + Scala | every attempt ended `planned_not_executed` | never created |
| Claude + Kotlin | first attempt ran `pwd`, later attempts `planned_not_executed` | never created |
| Codex + Rust | every attempt ran bare `sudo` | never created |

All three pull requests were still empty after five automatic restarts each.
Hive Mind fixed its own side in #2159 — a bounded repository objective, and
`planned_not_executed` classified as a terminal tool failure — and reported the
two remaining Formal AI defects upstream. Both are fixed on this branch.

### 6.1 The objective boundary (#907, follow-up)

The shipped #907 fix separates the caller from the user by the *markup* a client
wraps its framing in: `<session_context>`, `<environment_context>`, `<env>`.
Hive Mind's adapters wrapped theirs in nothing — workflow policy and objective
arrived concatenated into one untagged `user` message — so there was no markup
left to key on, and the whole message reached intent routing:

| Preamble line | Matched | Planned |
| --- | --- | --- |
| `Your prepared working directory: /tmp/example` | the `pwd` intent cue *working directory* | `pwd` |
| `When running sudo commands, run them in the background.` | a run verb plus the `sudo` shell token | `sudo` |

Neither sentence asks for anything. The tell that replaces the markup is the
delimiter the caller itself wrote: `objective_text` already cut a line-anchored
`Issue to solve:` / `Task:` / `Goal:` lead for the general planner (that is the
original #904 fix), but the router reached `shell_command_for_task` with the raw
message. `plan_chat_step` now narrows the task the same way, so one boundary
serves every route rather than one recipe.

The second row survives even with no delimiter present, so it needed its own
rule: a seed-declared `policy_lead` (`when`, `if`, `never`, `когда`, `如果`, …)
marks a clause as *governing* a class of commands rather than requesting one,
and `named_shell_command` reads one sentence at a time so run context can only
license the command sharing its sentence.

Rungs `R916-09` and `R916-10` in the write-effect ladder judge both from a real
workspace, and `tests/unit/issue_907.rs` pins the boundary, the seed data, and
the inverse case where an imperative naming `sudo` still selects it.

### 6.2 Reading the work item (#904, follow-up)

The `planned_not_executed` terminal state introduced by the original #904 fix is
truthful and stayed truthful. It was also every repository run's outcome, because
`finish_general_change` returned it unconditionally for
`GeneralPlanMode::RepositoryWorkItem`.

The reason is in the plan: a work item names an *issue*, and an issue URL names
no artifact. Recording the reference was the only honest end available, because
the run never read the one document that says what to build. So the fix is not to
loosen the terminal state — it is to read the work item first. The composed plan
opens with a `Fetch` step for the URL it named; once that text arrives, the plan
is re-composed from what the issue actually says and the existing literal-file
and source routes execute it against the artifact the issue names.

`planned_not_executed` survives exactly where #2158's point 3 reserves it: the
client advertises no fetch capability, the fetch came back empty, or the work
item names no artifact — which is never invented into one.
`tests/unit/issue_904.rs` pins all four cases.

### 6.3 What this branch does not close: language coverage

The three reproduction issues linked from #2158 ask for a Hello World program in
a named language. The coding catalog
(`src/coding/catalog/languages.rs`) covers ten: Rust, Python, JavaScript,
TypeScript, Go, C, C++, Java, C#, and Ruby. The production matrix asked for
**Scala** (Agent leg) and **Kotlin** (Claude leg), and covers neither. Only the
Codex leg's Rust was in the catalog — which is consistent with it being the leg
that got furthest before the separate `sudo` hijack stopped it.

So the two fixes above close the *routing* defects: the objective now reaches
the router intact, and the work item is read before its execution is judged
impossible. A Scala or Kotlin work item still ends `planned_not_executed`,
because reading an issue cannot invent a language the catalog does not have.
That is the truthful outcome and the reason #2158's point 3 keeps the state, but
it is not the same as being able to solve those three issues.

`an_uncovered_language_is_not_invented_into_an_artifact` in
`tests/unit/issue_904.rs` records the boundary rather than assuming it: the test
fails the day the catalog gains Scala or Kotlin, which is exactly when this
section stops being accurate. Growing that coverage is separate work and is not
claimed here.

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
- [hive-mind#2158](https://github.com/link-assistant/hive-mind/issues/2158) and
  [PR #2159](https://github.com/link-assistant/hive-mind/pull/2159) — the
  production evidence that the transport worked while the work did not, the
  Hive Mind-side boundary fix, and the two upstream reports section 6 closes.
- [formal-ai#904](https://github.com/link-assistant/formal-ai/issues/904) and
  [formal-ai#907](https://github.com/link-assistant/formal-ai/issues/907) — the
  two reopened defects, fixed in section 6.
