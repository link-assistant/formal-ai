---
bump: minor
---

### Added
- Added a `formal-ai telegram` CLI subcommand that defaults to Telegram long polling (`getUpdates`) and keeps the existing webhook server as `--mode=webhook`, configured through `lino-arguments` so flags, environment variables, and `.lenv`/`.env` files all feed the same parser.
- Introduced `TelegramPollingConfig`, `parse_get_updates_response`, the `TelegramTransport` trait, and a curl-backed default transport so the polling loop is fully unit-tested without a network.
