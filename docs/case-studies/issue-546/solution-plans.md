# Issue 546 Root Cause and Solution Plan

## Root-cause map

| Req | Root cause | Fix in this PR | Verification |
|---|---|---|---|
| R1 | `desktop/lib/tool-router.cjs` included `shell` in `SANDBOXED_TOOLS`, so the granted `ls ~` call went to Docker by default. | Remove `shell` from `SANDBOXED_TOOLS`; add `hostShell()` and route default `shell` calls to injected `runOnHost`. | `desktop/scripts/tool-router.test.mjs` asserts `ls ~` returns `servedBy: "host-shell"` and never probes Docker. |
| R2 | There was no way to request host shell and Docker shell separately. The only shell target was Docker. | Keep the existing sandbox path and add `input.isolation = "docker"` as the opt-in for Docker-isolated shell. | `desktop/scripts/tool-router.test.mjs` covers explicit Docker shell isolation and `code_exec` still refusing when Docker is unavailable. |
| R3 | The permission gate was correct, but the shell grant led to the wrong executor and the permission description said Docker. | Preserve `isPermitted()` and default-deny order; update shell permission descriptions to host machine. | Default-deny tests include `runOnHost` as a possible side effect and assert no side effects run before a grant. |
| R4 | VS Code desktop reused the shared router but had no host runner injection, so changing only Electron would leave a second host surface inconsistent. | Add `runOnHost` to `vscode/src/extension.node.cjs`; leave `extension.web.cjs` unchanged. | `tests/unit/specification/vscode_surface.rs` and `vscode/scripts/smoke.mjs` assert the Node host wires `runOnHost`. |
| R5 | Environment seed data described shell as Docker-routed and permission strings reinforced the wrong model. | Update `data/seed/environments.lino`, `src/web/i18n-catalog-permissions.lino`, `vscode/package.json`, and docs. | I18n/static checks validate catalog shape and smoke tests validate routing markers. |
| R6 | Existing tests asserted the old Docker behavior for shell, so the bug was locked in. | Change the shell tests to require host execution and add Docker opt-in coverage. | `node --test desktop/scripts/tool-router.test.mjs desktop/scripts/agent-provider.test.mjs`. |
| R7 | The issue requested a case study and raw data capture; the PR initially only had code changes. | Add `docs/case-studies/issue-546` with raw captures, screenshot, requirements, and analysis. | Repository review plus file-size/check-changelog guards. |
| R8 | `command-stream` and `start` were not part of the current direct shell tool path, and adopting them blindly would introduce packaging risk. | Capture metadata and document the temporary injected Node runner. Keep `agent-commander`/`start-agent` for the separate agent-provider path. | Case-study notes cite captured metadata and explain why no upstream issue was filed. |

## Solution design

### Router contract

The router now distinguishes execution target from permission:

- Permission remains per tool: `shell`, `code_exec`, `eval_js`, and so on.
- Target defaults are tool-specific: `shell` defaults to host, while
  `code_exec` and `eval_js` default to Docker.
- A shell request can override its target with `input.isolation = "docker"`.

This keeps the UI permission model stable. The user grants "shell"; the request
body decides whether this particular shell command uses the default host runner
or the explicit Docker runner.

### Host shell runner

Electron and VS Code both inject:

```js
runOnHost({ tool, command })
```

The implementation uses `child_process.spawn(command, { shell: true })` so
ordinary shell syntax such as `~` expansion works. Output is captured as
`stdout`, `stderr`, combined `body`, `exitCode`, and `logPath`.

For Electron, the working directory is the user's home directory, which matches
the expectation for `ls ~`. For VS Code, the working directory is the first
workspace folder when present, otherwise the user's home directory.

### Docker target

The existing sandbox path remains unchanged for code tools:

```js
runInSandbox({ image: "konard/box-dind:2.1.1", tool, command })
```

The only router change is that `shell` reaches this path only when
`input.isolation` is exactly `"docker"`. This means Docker credential failures
can no longer block the default host terminal use case.

### Meanings and words

No new natural-language branch was added for the Russian prompt. The prompt is
already covered by `data/seed/terminal-commands.lino` and mirrored into
`src/web/formal_ai_worker.js` through the existing seed-sync convention. This PR
keeps that design intact and updates environment/permission data that describe
the execution target.

### Temporary command execution bridge

`command-stream` is a good candidate for a future shared command runner, but the
current metadata describes it as Bun-optimized and the desktop/VS Code host code
is CommonJS Node. For this single routing bug, the lower-risk path is an injected
`child_process` runner with tests around the router policy. A future migration
can replace the `runOnHost` implementation without changing the router contract.

## Test plan

1. Run the focused desktop tests:

   ```bash
   node --test desktop/scripts/tool-router.test.mjs desktop/scripts/agent-provider.test.mjs
   ```

2. Run package-level Electron checks:

   ```bash
   npm --prefix desktop test
   ```

3. Run package-level VS Code checks:

   ```bash
   npm --prefix vscode test
   ```

4. Run repository static and Rust checks used by CI, including changelog and
   file-size guards.

5. After pushing, verify GitHub Actions runs on the new branch head and download
   logs for any non-passing run into `ci-logs/`.
