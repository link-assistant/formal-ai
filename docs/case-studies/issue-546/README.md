# Issue 546 Case Study - Host shell default for terminal commands

> **Issue:** <https://github.com/link-assistant/formal-ai/issues/546>
> **Pull request:** <https://github.com/link-assistant/formal-ai/pull/547>
> **Case study date:** 2026-06-20
> **Status:** Implemented in this PR.

## Artifacts

| Artifact | Path |
|---|---|
| Issue JSON | [`raw-data/issue-546.json`](raw-data/issue-546.json) |
| Issue comments | [`raw-data/issue-546-comments.json`](raw-data/issue-546-comments.json) |
| PR JSON and discussion captures | [`raw-data/pr-547.json`](raw-data/pr-547.json), [`raw-data/pr-547-review-comments.json`](raw-data/pr-547-review-comments.json), [`raw-data/pr-547-reviews.json`](raw-data/pr-547-reviews.json) |
| Screenshot from the issue | [`assets/issue-screenshot.png`](assets/issue-screenshot.png) |
| Related repository/package metadata | [`raw-data/link-foundation-start.json`](raw-data/link-foundation-start.json), [`raw-data/link-foundation-command-stream.json`](raw-data/link-foundation-command-stream.json), [`raw-data/npm-command-stream.json`](raw-data/npm-command-stream.json) |
| Online/code-search notes | [`raw-data/online-research.md`](raw-data/online-research.md) |
| Requirement inventory | [`requirements.md`](requirements.md) |
| Root cause and solution plan | [`solution-plans.md`](solution-plans.md) |

## Summary

The user asked the desktop app in Russian to run `ls ~` in a terminal. The app
correctly recognized the prompt as a terminal-command request, switched into the
permission flow, and asked for the `shell` grant. After the grant, the command
was routed through the Docker sandbox instead of the host shell. Docker then tried
to pull `konard/box-dind:2.1.1` and failed because the GUI process could not find
`docker-credential-desktop`.

The expected behavior for this issue is narrower: a normal terminal request such
as `ls ~` should run on the host machine by default, while Docker must remain
available as an explicit sandbox target. The root bug was not the multilingual
terminal-command recognizer. It was the shared desktop tool router classifying
`shell` as a sandboxed tool.

## Timeline

1. The user entered "Выполни в терминале `ls ~`" in the Electron desktop app.
2. The web worker classified the request as a terminal command using the seeded
   terminal vocabulary from `data/seed/terminal-commands.lino`.
3. Chat mode produced an agent suggestion and the permission panel asked for
   explicit desktop tool grants.
4. After `shell` was granted, the in-process agent provider invoked the shared
   `shell` tool through `desktop/lib/tool-router.cjs`.
5. The router treated `shell` as part of `SANDBOXED_TOOLS`, so
   `desktop/main.cjs` called the Docker runner.
6. Docker failed before the command executed, so the user saw a Docker pull /
   credential helper error instead of their home directory listing.

## Current-state inventory

| Capability | Current component | Finding |
|---|---|---|
| Multilingual terminal-command intent | `data/seed/terminal-commands.lino`, `src/solver_terminal.rs`, `src/web/formal_ai_worker.js` | Already covers Russian run verbs and "in terminal" phrasing. No vocabulary change was needed for this exact prompt. |
| Permission-gated tools | `desktop/lib/tool-router.cjs`, `src/web/app.js` | Default-deny policy was correct and preserved. The target selected after grant was wrong. |
| Desktop host effects | `desktop/main.cjs` | Local process already served fetch/file tools and Docker code execution. It lacked a host shell executor for the `shell` tool. |
| VS Code host effects | `vscode/src/extension.node.cjs` | Reuses the desktop router, so the same host-shell default must be wired there too. |
| Docker sandbox | `runInSandbox`, `konard/box-dind:2.1.1` | Still required for `code_exec`, `eval_js`, and explicit Docker-isolated shell calls. |
| Agent backend | `agent-commander` / `start-agent` | Already used for the broader agent-provider path. It is separate from the direct `shell` tool route fixed here. |

## Implementation

The shared router now has two execution targets for `shell`:

- default `shell`: `runOnHost({ tool, command })`, served as `host-shell` with
  `isolation: "host"`;
- explicit Docker shell: `input.isolation = "docker"`, served by the existing
  `box-dind` sandbox with `isolation: "docker"`.

`code_exec` and `eval_js` continue to require Docker and still refuse with
`sandbox_unavailable` instead of running unsandboxed when Docker is missing.

Both Electron and VS Code inject a host-shell runner based on `child_process`.
The runner uses the desktop user's home directory as the Electron working
directory and the first VS Code workspace folder, when available, for VS Code.
Output is captured into the same structured tool result shape used by the
existing tool bridge.

## link-foundation/start and command-stream

The issue asked to evaluate `link-foundation/start` and
`link-foundation/command-stream`. The captured metadata shows:

- `link-foundation/start`: a command execution/gamification project with GitHub
  auto-reporting support.
- `link-foundation/command-stream`: the `command-stream` package, version
  `0.14.0`, described as a streaming shell utility optimized for Bun runtime.

The direct desktop and VS Code host-shell path in this repository is CommonJS
Node/Electron code. Adding a Bun-optimized shell package to both extension hosts
would increase packaging risk for this bug fix. This PR therefore uses the
temporary compatibility path that the repo already uses elsewhere:
`child_process.spawn(..., { shell: true })`, injected as `runOnHost` so the
router policy remains testable. No upstream issue was filed because the research
did not identify a reproducible missing feature in `command-stream` or `start`;
the gap is integration policy in this repo, not a confirmed defect upstream.

## Verification

The reproducing tests are:

- `desktop/scripts/tool-router.test.mjs`: a granted `shell` call for `ls ~` must
  run through `host-shell` without probing Docker.
- `desktop/scripts/tool-router.test.mjs`: `shell` can still opt into Docker with
  `input.isolation = "docker"`.
- `desktop/scripts/agent-provider.test.mjs`: the in-process agent path for a
  read-only `ls ~` command must return `servedBy: "host-shell"`.

The static surface checks were also updated so Electron and VS Code keep wiring
the host runner while preserving the Docker sandbox for code execution.
