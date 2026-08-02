# Telegram setup

Create a bot with `@BotFather`, then install and run polling mode:

```bash
curl -fsSL https://raw.githubusercontent.com/link-assistant/formal-ai/main/scripts/install.sh | sh -s -- telegram
TELEGRAM_BOT_TOKEN=123:abc formal-ai telegram
```

Polling is the default. For a public HTTPS reverse proxy or test harness, use
webhook mode:

```bash
TELEGRAM_BOT_TOKEN=123:abc formal-ai telegram \
  --mode webhook --host 127.0.0.1 --port 8080
```

On Windows PowerShell set `$env:TELEGRAM_BOT_TOKEN='123:abc'` before the same
command. Never commit the token.

The native bot resolves the shared memory file, so a bot started alongside the
Desktop/API with the same environment sees the same append-only history at
`~/.formal-ai/memory.lino` or `%APPDATA%\formal-ai\memory.lino`. Docker must
mount that directory as described on the [Docker page](docker.md).

Send `/start` and a short prompt to verify polling. For webhook debugging, POST
a fixture update to `/telegram/webhook` and inspect the HTTP status and bot log.
