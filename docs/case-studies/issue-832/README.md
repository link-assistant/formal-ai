# Issue 832: reliable agentic reports after an upgrade

- Issue: <https://github.com/link-assistant/formal-ai/issues/832>
- Pull request: <https://github.com/link-assistant/formal-ai/pull/833>
- Referenced requirement:
  <https://github.com/link-assistant/formal-ai/issues/819#issuecomment-5059277610>

## Reported journey

The reporter ran Formal AI 0.302.2 from OpenCode and asked it to find a folder
on the Desktop. The visible server still used the old command-echoing,
template-like narration that issue #819 had removed. After the local search
returned no matches, the reporter selected Harness log, Server log, and GitHub
issue. The combined command exported the harness log, then tried to retrieve
the other context from `127.0.0.1:3000` with `curl`. That connection failed,
`set -e` stopped the command before `gh issue create`, and Formal AI nevertheless
answered, “Filed the issue on GitHub.”

![Original OpenCode reproduction](screenshots/reproduction.png)

The raw issue, referenced requirement, prepared pull request surfaces, related
PR #831, and pre-change CI run are preserved under [`raw-data/`](raw-data/).

## Root causes

Three independent assumptions combined into the observed failure.

1. The report planner generated HTTP requests for server context even though
   the shell tool runs in the client process. The earlier TUI test injected
   `FORMAL_AI_BASE_URL` and provided a fake `curl`; real OpenCode did neither.
   The command therefore fell back to a hard-coded port unrelated to the
   configured server.
2. Report progress treated any completed shell tool call as a completed report.
   The final renderer claimed GitHub success even when the output had no issue
   URL and contained a non-zero command failure.
3. `formal-ai with` considered any listener on the selected port reusable. It
   never checked the listener's identity or version. A long-running pre-#831
   Formal AI server could therefore survive a CLI upgrade and continue serving
   the narration that the on-disk 0.302.2 binary no longer contained.

The previous report tests encoded the first assumption: they asserted `curl`
URLs and supplied the missing environment variable. Fresh in-process narration
tests also could not expose the third assumption because they never reused an
older server.

## Requirements and implementation

| Requirement | Implementation and evidence |
| --- | --- |
| Keep pre-tool narration natural and command-free | The #819 HTTP journey remains covered by `issue_819_report_flow`; the launcher now prevents an older server from silently bypassing that released behavior. |
| Execute every selected report destination | Harness, server, and combined GitHub context all use `formal-ai context export --source harness/server/both`, so the tool process reads the shared local dialog log without a server URL. |
| Keep Formal AI learning available offline | `formal-ai context learn --session …` uses the same shared learning function as the HTTP endpoint and accepts an explicit dialog-log directory for diagnostics and tests. |
| Never report false GitHub success | A GitHub report is acknowledged only when tool output contains an issue URL. Missing URLs and command errors receive a failure status with the original diagnostic. |
| Do not reuse stale server behavior | `/health` now exposes the crate version. Auto-starting wrappers validate an existing listener and reject missing or mismatched versions before starting the client. Explicit `--no-start-server` endpoints remain usable for user-managed compatible services. |
| Test the real client boundary | The OpenCode report TUI harness no longer injects `FORMAL_AI_BASE_URL` or installs a fake `curl`; it requires three local exports and the final `gh issue create`. |

## Regression strategy

`tests/unit/issue_832.rs` directly reproduces the two report failures and
asserts the health-version contract. The existing #819 and #822 tests were
updated to require local exports rather than the defective HTTP command.

`tests/integration/issue_832_server_compatibility.rs` serves controlled health
responses from a real loopback listener. A matching server is reused, while
version 0.301.0 and a legacy health response with no version are both rejected;
the wrapped agent is never invoked. The context integration suite also invokes
`formal-ai context learn` as a separate process with only a local dialog log
and memory file.

Focused post-fix logs are preserved in [`test-logs/`](test-logs/). The final
verification matrix was:

- The detached pre-fix worktree failed all three issue #832 regression tests:
  local context export, truthful GitHub completion, and health-version
  identification. The exact failures are in
  [`reproduction-before-fix.log`](test-logs/reproduction-before-fix.log).
- The new unit regressions, three real-listener compatibility cases, local
  learning process test, and updated #714/#771 report suites all pass.
- `cargo test --all-features` passes every test binary. Its main integration
  suite reports 188 passed; its main unit suite reports 1,990 passed and two
  intentionally ignored exhaustive/network cases.
- Formatting, Clippy with `-D warnings`, both public API documentation profiles,
  locked dependency metadata, file-size, associative-terminology,
  hardcoded-language, and shell syntax checks pass. The release build also
  succeeds.
- The real OpenCode TUI runs against the release Formal AI server and executes
  `--source harness`, `--source server`, `--source both`, and `gh issue create`.
  The harness supplies neither `FORMAL_AI_BASE_URL` nor a fake `curl`. Its
  transcript, server trace, dialog logs, and action record are preserved in
  [`real-client-report/`](real-client-report/).

## Self-coding attempt

The required live self-coding entry point was attempted before implementation.
The configured external solver rejected the `formal-ai` model identifier before
it could propose a patch, so none of the issue fix or its manually prepared
regressions claim self-authored attribution. The full output is preserved in
[`self-coding-live.log`](raw-data/self-coding-live.log), and the runner's
automatic diagnostic is recorded in
[issue comment 5063145457](https://github.com/link-assistant/formal-ai/issues/832#issuecomment-5063145457).

After the pull request's differential self-hosting check identified that the
branch would lower the repository-wide metric, the established source-links
harness ran a separate real Agent CLI to Formal AI session,
`ses_06f008aa0ffeshFEmHa8iHyYJM`. That session authored a representative
source-links specification and projected all 293 owned modules into two
round-trip-verified shards. Its generated artifacts are preserved under
[`self-hosting-evidence/`](self-hosting-evidence/). Only the isolated generated
artifact commit carries that session's `Formal-AI-Session` and
`Formal-AI-Evidence` trailers.
