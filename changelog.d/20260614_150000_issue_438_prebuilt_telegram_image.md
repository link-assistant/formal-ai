---
bump: minor
---

### Added

- Publish the Telegram Docker-in-Docker image to GHCR on release and document the one-line `docker run` / `docker compose up` startup path.
- Add root `compose.yaml` for the prebuilt Telegram bot image with `TELEGRAM_BOT_TOKEN` as the only required setting.
- One-click start/stop of both prepared services — the Telegram bot and the OpenAI-compatible API server — from the desktop app, with live Docker status polling (`desktop/lib/service-control.cjs` over IPC).
- Opt-in `server` profile in `compose.yaml` so a server reproduces the identical containers with one line (`docker compose --profile all up -d`); each Docker-in-Docker service gets its own inner-Docker volume so the bot and server can run together.
- New `docs/desktop/service-control.md` documenting both the one-click desktop and one-line server paths in detail.
