# One-click services and agent environment

formal-ai ships its long-running services and installable Agent CLI environment
as a single prepared Docker image (`ghcr.io/link-assistant/formal-ai`). This
document shows the two ways to run them — **one click in the desktop app** and
**one line on a server** — and how the two paths drive the *same* containers so
behaviour is identical everywhere.

The managed containers are:

| Service | Container | What it runs | Network |
| ------- | --------- | ------------ | ------- |
| **Telegram bot** | `formal-ai-telegram` | the image's default `formal-ai telegram --mode polling` | none (polls Telegram) |
| **OpenAI-compatible server** | `formal-ai-server` | `formal-ai serve` for agentic mode | `127.0.0.1:8080` |
| **Agent environment** | `formal-ai-agent` | idle container with `formal-ai`, `agent`, and `start-agent` installed | none |

Both run from the **single prepared image** and use the in-container Docker
daemon (Docker-in-Docker) for the agentic sandbox, so neither bind-mounts the
host's `docker.sock`. Each container gets its **own** inner-Docker volume
(`formal-ai-telegram-docker`, `formal-ai-server-docker`,
`formal-ai-agent-docker`) because two DinD daemons cannot share one
`/var/lib/docker`.

---

## 1. Desktop app — one click

The desktop shell ([`desktop/`](../../desktop)) renders a **Services** panel in
the sidebar. Each managed service has:

- a status dot (green = running, grey = stopped) and a state label,
- a **Start** and a **Stop** button for services,
- an **Install agent environment** action for the Agent CLI container,
- for the Telegram bot, an inline `TELEGRAM_BOT_TOKEN` field (the bot will not
  start without it),
- for the server, the loopback URL (`http://127.0.0.1:8080`) once it is up.

The panel polls status every few seconds, so the indicators reflect
`docker`-reported state without a manual refresh.

```text
Services
  ● Telegram bot                running   [Stop]
  ○ OpenAI-compatible server    stopped   [Start]
      http://127.0.0.1:8080
  ○ Agent environment           stopped   [Install agent environment]
```

The install action for the Agent environment:

1. pulls `FORMAL_AI_DOCKER_IMAGE` (default:
   `ghcr.io/link-assistant/formal-ai:latest`), falling back to a locally built tag
   when `docker image inspect` can see it,
2. recreates `formal-ai-agent` from that image,
3. health-checks the required commands inside the container:
   `formal-ai --version`, `agent --version`, and `start-agent --help`.

### How it is wired

The lifecycle logic lives in a dependency-injected module so it is unit-testable
without a live daemon:

- [`desktop/lib/service-control.cjs`](../../desktop/lib/service-control.cjs) —
  owns the `docker run` argument vectors for each service, the running-state
  probe (`docker inspect -f '{{.State.Running}}'`), stale-container reaping
  (`docker rm -f` before a fresh `run`), and required-config checks (the Telegram
  token), and the Agent environment install/health-check flow. All Docker access
  goes through an injected `runDocker(args)` runner.
- [`desktop/main.cjs`](../../desktop/main.cjs) — supplies a real `docker`
  child-process runner and exposes the lifecycle over IPC:
  `formalAiDesktop:serviceStatus`, `formalAiDesktop:startService`,
  `formalAiDesktop:installAgentEnvironment`, `formalAiDesktop:stopService`.
- [`desktop/preload.cjs`](../../desktop/preload.cjs) — bridges those handlers to
  the renderer as `serviceStatus()`, `startService()`,
  `installAgentEnvironment()`, `stopService()` through `contextBridge` (with
  `contextIsolation: true` / `nodeIntegration: false`).
- [`src/web/app.js`](../../src/web/app.js) — renders the Services panel, polls
  status, and calls the bridge from the **Start**/**Stop** buttons.

Docker is required for this panel; if it is not installed the panel shows a
clear "Docker is not available" note and the buttons stay disabled.

---

## 2. Server — one line

The same containers run from the root [`compose.yaml`](../../compose.yaml) using
Compose **profiles**, so `docker compose up` keeps starting only the Telegram
bot (the documented quick start) while the server stays opt-in.

```bash
# Telegram bot (default profile)
TELEGRAM_BOT_TOKEN=123:abc docker compose up -d

# OpenAI-compatible API server (agentic mode), opt-in profile
docker compose --profile server up -d

# Agent CLI + agent-commander environment, opt-in profile
docker compose --profile agent up -d

# all at once
TELEGRAM_BOT_TOKEN=123:abc docker compose --profile all up -d
```

Stop them the same way:

```bash
docker compose down                       # telegram bot
docker compose --profile server down      # server
docker compose --profile agent down       # agent environment
docker compose --profile all down         # all
```

### Without Compose

The desktop app and the docs deliberately use the same arguments, so plain
`docker run` commands reproduce any container:

```bash
# Telegram bot
docker run -d --name formal-ai-telegram --restart unless-stopped --privileged \
  -e TELEGRAM_BOT_TOKEN=123:abc \
  -e FORMAL_AI_MEMORY_PATH=/root/.formal-ai/memory.lino \
  -v "$HOME/.formal-ai:/root/.formal-ai" \
  -v formal-ai-telegram-docker:/var/lib/docker \
  ghcr.io/link-assistant/formal-ai:latest

# OpenAI-compatible server on 127.0.0.1:8080
docker run -d --name formal-ai-server --restart unless-stopped --privileged \
  -p 127.0.0.1:8080:8080 \
  -e FORMAL_AI_MEMORY_PATH=/root/.formal-ai/memory.lino \
  -v "$HOME/.formal-ai:/root/.formal-ai" \
  -v formal-ai-server-docker:/var/lib/docker \
  ghcr.io/link-assistant/formal-ai:latest \
  formal-ai serve --host 0.0.0.0 --port 8080

# Agent CLI environment
docker pull ghcr.io/link-assistant/formal-ai:latest
docker rm -f formal-ai-agent 2>/dev/null || true
docker run -d --name formal-ai-agent --restart unless-stopped --privileged \
  -e FORMAL_AI_MEMORY_PATH=/root/.formal-ai/memory.lino \
  -v "$HOME/.formal-ai:/root/.formal-ai" \
  -v formal-ai-agent-docker:/var/lib/docker \
  ghcr.io/link-assistant/formal-ai:latest \
  sleep infinity
docker exec formal-ai-agent sh -lc \
  'formal-ai --version && agent --version && start-agent --help >/dev/null'
```

---

## 3. Configuration

| Variable | Default | Applies to | Purpose |
| -------- | ------- | ---------- | ------- |
| `TELEGRAM_BOT_TOKEN` | — (required) | Telegram bot | bot token from [@BotFather](https://core.telegram.org/bots#botfather) |
| `FORMAL_AI_DOCKER_IMAGE` | `ghcr.io/link-assistant/formal-ai:latest` | all containers | override the image (local build or Docker Hub mirror) |
| `FORMAL_AI_SERVER_PORT` | `8080` | server | published loopback port for the OpenAI API |
| `FORMAL_AI_MEMORY_PATH` | `/root/.formal-ai/memory.lino` in containers | all containers | shared persistent Links-Notation memory file |
| `FORMAL_AI_TELEGRAM_ALLOWED_UPDATES` | `message,edited_message` | Telegram bot | Telegram update types to poll |

The server binds `0.0.0.0` *inside* the container but is published only to
`127.0.0.1` on the host, so it is reachable from the local machine only. See
[server-api.md](server-api.md) for the API surface, authentication, and how to
point coding CLIs (`codex`, `claude`, `opencode`, `agent`) at it.

---

## 4. Why these defaults

- **Single image, three commands.** The Telegram bot keeps the image's default
  command; the server overrides it with `formal-ai serve`; the Agent environment
  overrides it with `sleep infinity` so the desktop can health-check and target a
  stable container. The image includes Node.js for `agent-commander` plus Bun for
  the installed CLIs. One image to build, pull, and pin.
- **Inner Docker, never the host socket.** The agentic sandbox uses the
  container's own daemon (DinD), so the host's `docker.sock` is never exposed.
- **Per-service volume.** Two DinD daemons cannot share one `/var/lib/docker`, so
  the bot, server, and Agent environment each get their own named volume and can
  run together.
- **One host memory directory.** All three services bind the host's
  `~/.formal-ai` to `/root/.formal-ai`. This is intentionally shared: Telegram,
  API, Agent CLI, desktop, VS Code, and host CLI all read and append the same
  `memory.lino`. Set `FORMAL_AI_MEMORY_PATH` for a non-default host file.
- **Loopback-only server.** Publishing on `127.0.0.1` keeps the unauthenticated
  local API off the network by default.
- **No host agent subscriptions.** The desktop provider is guarded against direct
  host `agent`, `claude`, or `codex` process spawns. The prepared image bundles
  `@link-assistant/agent` and `agent-commander`; autonomous execution is routed to
  the Formal-AI container path instead of the developer's logged-in host CLIs.

## 5. Applied isolation practices

Issue #517 asks for the hive-mind isolation guidance to be applied to the Agent
CLI + agent-commander setup. The implemented desktop/container contract applies
the relevant practices directly:

- **Docker/VM boundary for autonomous tools.** The Agent environment is a
  Formal-AI-owned Docker container derived from `konard/box-dind:2.1.1`.
- **No host Docker socket.** Containers use their own DinD daemon and named
  `/var/lib/docker` volumes; the host `/var/run/docker.sock` is never mounted.
- **No host agent binaries.** Desktop tests statically guard against direct
  `agent`, `claude`, or `codex` spawns from the desktop code.
- **Health before use.** One-click install verifies `formal-ai`, `agent`, and
  `start-agent` inside `formal-ai-agent` before the environment is reported
  ready.
- **Local model endpoint.** The server container publishes only loopback by
  default, and the commander provider points tools at the Formal-AI
  OpenAI-compatible base URL rather than host subscription defaults.

---

## Related

- [Desktop runtime: in-process agent and the optional server API](server-api.md)
- [Issue #438 case study](../case-studies/issue-438/README.md) — requirements and design for the prepared image
- [Root `compose.yaml`](../../compose.yaml) — the server-side service definitions
- [`desktop/lib/service-control.cjs`](../../desktop/lib/service-control.cjs) — the lifecycle module
