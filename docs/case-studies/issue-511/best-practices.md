# Agent CLI + agent-commander best practices

Issue #520 closes the upstream-feedback and best-practices loop for the #511
desktop agent epic. This write-up records the practices that Formal AI now
applies when it drives Agent CLI through `agent-commander`.

## Current upstream baseline

Verified on 2026-06-19:

- `@link-assistant/agent` latest npm/GitHub release is **0.24.0**. It has native,
  enforceable permission modes (`auto`, `plan`, `readonly`, `ask`) and a JSON
  `permission_request` / `permission_response` protocol.
- `agent-commander` latest npm release is **0.8.0**; latest Rust release is
  **0.2.6**. It maps `--read-only` / `--plan-only` to the `agent` backend's
  native permission modes and exposes the per-command approval relay as
  `--approve-each` / `--permission-mode ask`.
- `link-assistant/agent-commander` has no open GitHub issues. The previously
  filed gaps are closed:
  [`#20`](https://github.com/link-assistant/agent-commander/issues/20),
  [`#39`](https://github.com/link-assistant/agent-commander/issues/39), and
  [`#40`](https://github.com/link-assistant/agent-commander/issues/40).

## Practices

### 1. Run autonomous tools only inside the Formal-AI container

Hive-mind's relevant guidance is isolation: autonomous coding tools should run in
dedicated Docker/VM environments that are easy to discard and restore. Formal AI
implements that as the `formal-ai-agent` container:

- the desktop app installs an idle, health-checkable container from the prepared
  Formal-AI image;
- the image bundles `formal-ai`, `@link-assistant/agent`, and `agent-commander`;
- the container has its own Docker-in-Docker daemon and named
  `/var/lib/docker` volume;
- the host Docker socket is never mounted into the container.

This satisfies the issue's non-negotiable constraint: do not use a developer's
local `claude`, `codex`, or other logged-in host tools.

### 2. Always go through `agent-commander`

The desktop app must never spawn host `agent`, `claude`, or `codex` binaries
directly. Its commander provider launches `start-agent` and defaults to
`--tool agent`, the org-owned backend with the cleanest session-scoped
`once` / `always` / `reject` approval semantics.

The desktop test suite includes a static guard that scans desktop JavaScript
sources for direct host `agent` / `claude` / `codex` process spawns. The allowed
boundary is:

```text
desktop app -> start-agent (agent-commander) -> formal-ai-agent container -> agent
```

### 3. Point tools at the local Formal AI server, not host subscriptions

The provider scrubs host subscription environment variables such as
`ANTHROPIC_API_KEY`, `CLAUDE_API_KEY`, `CODEX_API_KEY`, `GEMINI_API_KEY`, and the
host `OPENAI_*` values before launching the commander path. It then injects the
local OpenAI-compatible Formal AI endpoint (`OPENAI_BASE_URL` /
`FORMAL_AI_OPENAI_BASE_URL`) and a local placeholder API key.

The server container publishes only to `127.0.0.1` by default.

### 4. Map Formal AI permissions to native commander modes

Use upstream enforcement whenever it exists:

- no desktop grants: `--plan-only`;
- read-only request or read-only shell command (`ls`, `pwd`, `cat`, `git status`,
  etc.): `--read-only`;
- mutating agent-mode request: `--approve-each`;
- full-auto paths stay grant-gated by the desktop permission model and must not
  bypass the tool router.

`agent-commander#39` makes `--tool agent --read-only` enforceable, so the
desktop provider now uses `--read-only` for the default `agent` backend instead
of the older `--approve-each` workaround.

### 5. Render and test the NDJSON stream, keep raw events

Agent output is an event stream, not plain text. The desktop adapter preserves
raw events and maps assistant text, tool start/result, permission events,
diagnostics, and errors onto the existing chat answer contract. Tests should use
recorded NDJSON fixtures for deterministic coverage, then gate live
container-backed coverage behind an explicit environment flag.

### 6. File upstream issues only for real `agent-commander` gaps

E4-E7 did not reveal a new `agent-commander` defect after #39/#40 shipped. The
remaining approval limitation is upstream-CLI capability, not an
`agent-commander` bug: per-command approve-each relays are available for
`agent` and `claude`; `codex`, `gemini`, `qwen`, and `opencode` do not currently
offer a relayable headless approval handshake for commander to normalize.

If one of those CLIs later exposes a JSON approval handshake and
`agent-commander` does not map it, file a new `link-assistant/agent-commander`
issue with:

- the exact backend, version, and command;
- the expected commander flag and event shape;
- a minimal repro;
- the Formal AI downstream use case and workaround, if any.

## E4-E7 findings

| Slice | Finding | Upstream action |
| --- | --- | --- |
| E4 / PR #532 | Provider seam and commander path work, but the implementation kept an obsolete `agent --read-only` workaround from before #39. | No new upstream issue; fixed downstream to use shipped `--read-only`. |
| E5 / PR #533 | The prepared image needed Node.js so `start-agent --help` works in the container. | Downstream packaging fix; no upstream issue. |
| E6 / PR #536 | NDJSON rendering works from recorded fixtures; a stale CI failure came from old coverage state. | No upstream issue. |
| E7 / PR #537 | Cold-start `ls ~` journey passes hermetically and has a commander-provider gated variant. | No upstream issue. |

## Verification checklist

- `node --test desktop/scripts/agent-provider.test.mjs`
- `npm --prefix desktop test`
- `npm run --prefix tests/e2e check:web-hardcoded-ui`
- `npm run --prefix tests/e2e check:i18n`
- `npm run --prefix tests/e2e test:local -- tests/issue-511-cold-start.spec.js`

Run the commander-provider E2E with
`FORMAL_AI_E2E_AGENT_COMMANDER=1` only when a ready `formal-ai-agent` container is
available.
