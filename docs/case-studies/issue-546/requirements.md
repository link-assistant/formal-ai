# Issue 546 Requirements

## R1. Run normal terminal commands on the host by default

**Source:** the issue states that `ls ~` is expected to execute on the host
machine, not in Docker.

**Acceptance criteria:**

- A granted `shell` tool call with `command: "ls ~"` uses the host runner.
- The result identifies the target as `servedBy: "host-shell"` and
  `isolation: "host"`.
- The host default does not probe Docker, pull `konard/box-dind`, or require
  Docker credentials.

## R2. Keep Docker available as a second execution target

**Source:** the issue asks to support both host and Docker execution.

**Acceptance criteria:**

- `code_exec` and `eval_js` remain Docker-sandboxed.
- `shell` can request Docker isolation explicitly with
  `input.isolation = "docker"`.
- Docker-isolated calls still refuse with `sandbox_unavailable` when Docker is
  unavailable, rather than falling back to unsandboxed execution.

## R3. Preserve explicit permissions and default-deny behavior

**Source:** the screenshot shows the existing explicit tool permission flow.

**Acceptance criteria:**

- No tool side effect runs before the matching grant is present.
- Granting `shell` only permits the `shell` tool.
- The permission panel text accurately describes host shell behavior.

## R4. Fix all local host surfaces that share the router

**Source:** the desktop and VS Code extension both reuse the shared router.

**Acceptance criteria:**

- Electron injects a host-shell runner.
- VS Code desktop injects a host-shell runner.
- VS Code web remains in-process only and does not import Node builtins.

## R5. Keep user-facing descriptions and environment seed data consistent

**Source:** the issue asks for variants to be supported through meanings and
words, and the previous permission copy said shell ran inside Docker.

**Acceptance criteria:**

- The permission catalog says `shell` runs on the host machine in all supported
  locales.
- `data/seed/environments.lino` describes local shell and Docker code execution
  separately.
- Existing terminal-command vocabulary remains seed-backed, not hardcoded in a
  new language-specific branch.

## R6. Verify the failure mode with tests

**Source:** the issue is a regression-prone routing bug.

**Acceptance criteria:**

- A test fails on the old router because `shell` goes to Docker by default.
- A test covers the agent-provider path used by the first terminal-command
  journey.
- A test covers explicit Docker isolation for shell so both targets remain
  available.

## R7. Compile issue data and case-study analysis

**Source:** the issue explicitly requests raw logs/data, timeline, requirements,
root causes, and solution plans under `docs/case-studies/issue-{id}`.

**Acceptance criteria:**

- The issue JSON, comments, PR discussion captures, screenshot, and related
  repository metadata live under `docs/case-studies/issue-546`.
- The case study reconstructs the event sequence and maps each requirement to a
  root cause, solution, and verification path.

## R8. Evaluate link-foundation/start and command-stream

**Source:** the issue asks to use or evaluate `link-foundation/start` and
`link-foundation/command-stream`, and to file upstream issues only for confirmed
missing features.

**Acceptance criteria:**

- Metadata for both repositories and the `command-stream` package is captured.
- The PR records why the direct shell fix uses a temporary Node host runner.
- No upstream issue is filed without a reproducible upstream defect.
