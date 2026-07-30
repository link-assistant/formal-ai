# Multi-agent orchestration

`formal-ai agent` is the explicit controller for external coding agents. It
keeps the conservative `formal-ai with` defaults unchanged while providing a
separate permission-gated path for editing, verification, parallel comparison,
bounded decomposition, composition, and byte-replayable provenance.

## Run one agent

Choose one seed-registered adapter and grant an existing workspace:

```bash
formal-ai agent run \
  --cli codex \
  --task "add a README badge" \
  --workspace /tmp/example-repository \
  --session /tmp/codex-session.json
```

The library API denies execution by default. The CLI canonicalizes
`--workspace` and creates a capability for that exact path. The controller
records the child process output, status, wall time, and file effects. Client
editing controls still apply: for example, the Codex adapter selects its
`workspace-write` sandbox. The runner sets both the process current directory
and `PWD` to the canonical workspace so clients cannot accidentally select the
controller's repository as their project. A timeout kills the external process
and its process group on Unix, then records one failed attempt; the controller
never retries implicitly.

The six orchestration adapters are `agent`, `claude`, `codex`, `gemini`,
`qwen`, and `opencode`. Their non-interactive editing and structured-output
arguments live in `data/seed/client-integrations.lino`.

The capability grant limits what Formal AI will launch and compose; filesystem
containment is supplied by each client policy or by the environment running
Formal AI. Codex uses `workspace-write`; Agent, Claude, Gemini, Qwen, and
OpenCode use their noninteractive edit policies rooted at the selected
workspace. Run untrusted client binaries in a container or equivalent host
sandbox as an additional boundary.

By default, the selected client talks to the local Formal AI model:

```bash
formal-ai serve --host 127.0.0.1 --port 8080
formal-ai agent run \
  --cli agent \
  --task "create release-notes.md" \
  --workspace /tmp/example-repository
```

Use `--model` or `--base-url` to override that target. `--target vendor` skips
the Formal AI wrapper and uses the client's existing vendor authentication;
pair it with `--model` to select a model that vendor provides. The permission,
timeout, effects, verification, and provenance boundaries remain active.

## Require verification

Verification is argv-based and default-denied. Each executable must be
allowlisted separately:

```bash
formal-ai agent run \
  --cli codex \
  --task "fix the failing parser test" \
  --workspace /tmp/example-repository \
  --allow-command cargo \
  --verify '["cargo","test","parser"]'
```

Repeat `--allow-command` and `--verify` for additional checks. Commands run
without a shell, use the same hard timeout, and retain stdout, stderr, exit
status, and timeout state in the session. A process or verification failure is
visible to the caller and is never treated as a passing result.

## Compare the same task

Comparison copies the workspace once per CLI, runs all candidates in parallel,
and composes only a passing winner:

```bash
formal-ai agent dispatch \
  --cli codex,claude,opencode \
  --compare \
  --task "add a README badge" \
  --workspace /tmp/example-repository \
  --allow-command cargo \
  --verify '["cargo","test","--workspace"]'
```

The recorded selection order is passing status, smaller diff, shorter wall
time, CLI id, then session path. Candidate workspaces, canonical sessions, and
`comparison-ledger.json` are written below
`.formal-ai-orchestration/`. A custom `--output-dir` must remain strictly
inside the granted workspace and is excluded from candidate copies.

## Decompose and compose a task

Omit `--compare` to use the universal task decomposer:

```bash
formal-ai agent dispatch \
  --cli codex,claude \
  --task "add a README badge and write a release note" \
  --workspace /tmp/example-repository
```

Bounded leaves are assigned round-robin to isolated candidate copies.
Composition preflights overlapping effects and applies changes only from
passing sessions. Identical overlapping effects are deduplicated and
conflicting effects are rejected. Immediately before composition, the
controller revalidates both the original pre-run hashes and the candidate
post-run hashes, preventing workspace or candidate drift from being composed.
Failed candidates remain isolated. Duplicate CLI ids are rejected before any
worker starts. `--max-depth` controls the decomposition bound.

## Replay a session

```bash
formal-ai agent replay /tmp/codex-session.json
```

Sessions use canonical pretty JSON with a terminal newline. Replay validates
the schema, event sequence, previous-event links, SHA-256 event digests, and
exact serialization bytes before printing the record. This detects reordered,
edited, truncated, or noncanonical evidence.

For an end-to-end controller run and a committed two-client comparison, see
the [issue 703 case study](../case-studies/issue-703/README.md).

## Real-client compatibility gate

The six installed CLIs can be exercised with the exact acceptance task:

```bash
FORMAL_AI_ISSUE_703_LIVE=1 \
experiments/issue_703_orchestration/run_live_cli_matrix.sh
```

This gate is opt-in because it launches authenticated clients and may consume
vendor quota. Each client receives a fresh workspace; the gate requires a
successful canonical session, a recorded README effect, and the actual badge
in that workspace.

Codex's native sandbox requires user namespaces. If the surrounding CI job is
already externally sandboxed but blocks those namespaces, the harness alone
supports `FORMAL_AI_ISSUE_703_CODEX_UNSANDBOXED=1`. This does not change the
product default and must not be used on an unsandboxed host.
