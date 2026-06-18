# One-click services: Telegram bot and OpenAI-compatible server

formal-ai ships two long-running services as a single prepared Docker image
(`ghcr.io/link-assistant/formal-ai`). This document shows the two ways to run
them — **one click in the desktop app** and **one line on a server** — and how
the two paths drive the *same* containers so behaviour is identical everywhere.

The two managed services are:

| Service | Container | What it runs | Network |
| ------- | --------- | ------------ | ------- |
| **Telegram bot** | `formal-ai-telegram` | the image's default `formal-ai telegram --mode polling` | none (polls Telegram) |
| **OpenAI-compatible server** | `formal-ai-server` | `formal-ai serve` for agentic mode | `127.0.0.1:8080` |

Both run from the **single prepared image** and use the in-container Docker
daemon (Docker-in-Docker) for the agentic sandbox, so neither bind-mounts the
host's `docker.sock`. Each service gets its **own** inner-Docker volume
(`formal-ai-telegram-docker`, `formal-ai-server-docker`) because two DinD daemons
cannot share one `/var/lib/docker`.

---

## 1. Desktop app — one click

The desktop shell ([`desktop/`](../../desktop)) renders a **Services** panel in
the sidebar. Each managed service has:

- a status dot (green = running, grey = stopped) and a state label,
- a **Start** and a **Stop** button,
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
```

### How it is wired

The lifecycle logic lives in a dependency-injected module so it is unit-testable
without a live daemon:

- [`desktop/lib/service-control.cjs`](../../desktop/lib/service-control.cjs) —
  owns the `docker run` argument vectors for each service, the running-state
  probe (`docker inspect -f '{{.State.Running}}'`), stale-container reaping
  (`docker rm -f` before a fresh `run`), and required-config checks (the Telegram
  token). All Docker access goes through an injected `runDocker(args)` runner.
- [`desktop/main.cjs`](../../desktop/main.cjs) — supplies a real `docker`
  child-process runner and exposes the lifecycle over IPC:
  `formalAiDesktop:serviceStatus`, `formalAiDesktop:startService`,
  `formalAiDesktop:stopService`.
- [`desktop/preload.cjs`](../../desktop/preload.cjs) — bridges those handlers to
  the renderer as `serviceStatus()`, `startService()`, `stopService()` through
  `contextBridge` (with `contextIsolation: true` / `nodeIntegration: false`).
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

# both at once
TELEGRAM_BOT_TOKEN=123:abc docker compose --profile all up -d
```

Stop them the same way:

```bash
docker compose down                       # telegram bot
docker compose --profile server down      # server
docker compose --profile all down         # both
```

### Without Compose

The desktop app and the docs deliberately use the same arguments, so a plain
`docker run` reproduces either container:

```bash
# Telegram bot
docker run -d --name formal-ai-telegram --restart unless-stopped --privileged \
  -e TELEGRAM_BOT_TOKEN=123:abc \
  -v formal-ai-telegram-docker:/var/lib/docker \
  ghcr.io/link-assistant/formal-ai:latest

# OpenAI-compatible server on 127.0.0.1:8080
docker run -d --name formal-ai-server --restart unless-stopped --privileged \
  -p 127.0.0.1:8080:8080 \
  -v formal-ai-server-docker:/var/lib/docker \
  ghcr.io/link-assistant/formal-ai:latest \
  formal-ai serve --host 0.0.0.0 --port 8080
```

---

## 3. Configuration

| Variable | Default | Applies to | Purpose |
| -------- | ------- | ---------- | ------- |
| `TELEGRAM_BOT_TOKEN` | — (required) | Telegram bot | bot token from [@BotFather](https://core.telegram.org/bots#botfather) |
| `FORMAL_AI_DOCKER_IMAGE` | `ghcr.io/link-assistant/formal-ai:latest` | both | override the image (local build or Docker Hub mirror) |
| `FORMAL_AI_SERVER_PORT` | `8080` | server | published loopback port for the OpenAI API |
| `FORMAL_AI_TELEGRAM_ALLOWED_UPDATES` | `message,edited_message` | Telegram bot | Telegram update types to poll |

The server binds `0.0.0.0` *inside* the container but is published only to
`127.0.0.1` on the host, so it is reachable from the local machine only. See
[server-api.md](server-api.md) for the API surface, authentication, and how to
point coding CLIs (`codex`, `claude`, `opencode`, `agent`) at it.

---

## 4. Why these defaults

- **Single image, two commands.** The Telegram bot keeps the image's default
  command; the server overrides it with `formal-ai serve`. One image to build,
  pull, and pin.
- **Inner Docker, never the host socket.** The agentic sandbox uses the
  container's own daemon (DinD), so the host's `docker.sock` is never exposed.
- **Per-service volume.** Two DinD daemons cannot share one `/var/lib/docker`, so
  the bot and the server each get their own named volume and can run together.
- **Loopback-only server.** Publishing on `127.0.0.1` keeps the unauthenticated
  local API off the network by default.

---

## Related

- [Desktop runtime: in-process agent and the optional server API](server-api.md)
- [Issue #438 case study](../case-studies/issue-438/README.md) — requirements and design for the prepared image
- [Root `compose.yaml`](../../compose.yaml) — the server-side service definitions
- [`desktop/lib/service-control.cjs`](../../desktop/lib/service-control.cjs) — the lifecycle module
