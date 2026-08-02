# Docker setup

The released `ghcr.io/link-assistant/formal-ai:latest` image contains the same
binary used by Telegram, the API server, Desktop-managed services, and the
Agent environment. It runs an inner Docker daemon; use `--privileged` or Sysbox
and do not mount the host `/var/run/docker.sock`.

```bash
TELEGRAM_BOT_TOKEN=123:abc docker compose up -d
docker compose --profile server up -d
docker compose --profile agent up -d
docker compose --profile all up -d
```

The profiles start Telegram, the loopback server, the idle Agent environment,
or all three. Their shared memory bind is equivalent to:

```bash
docker run --rm --privileged \
  -e FORMAL_AI_MEMORY_PATH=/root/.formal-ai/memory.lino \
  -v "$HOME/.formal-ai:/root/.formal-ai" \
  ghcr.io/link-assistant/formal-ai:latest
```

On Windows use `${env:APPDATA}\formal-ai:/root/.formal-ai`. The inner Docker
state uses a separate named volume; it is not the conversation memory.

Verify the runtime and isolation contract:

```bash
docker run --rm --privileged ghcr.io/link-assistant/formal-ai:latest verify-formal-ai-dind
curl -fsS http://127.0.0.1:8080/health
```

Desktop's Services panel manages these same container names and mounts, so
Desktop, API, Telegram, and Agent containers converge on one shared memory.
