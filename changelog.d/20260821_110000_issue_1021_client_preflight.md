---
bump: patch
---

### Fixed

- The server answers Anthropic's `/api/hello` reachability probe, so a Claude
  Code session no longer opens with a `404`. `@anthropic-ai/claude-code`
  2.1.238 added `HEAD <base-url>/api/hello` to the `HEAD <base-url>` probe it
  already made, which against the `/api/anthropic` base URL our wrapper writes
  arrives as `/api/anthropic/api/hello`. The doubled `/api` belongs to neither
  side: `https://api.anthropic.com/api/hello` is Anthropic's own endpoint and
  answers `200 {"message": "hello"}`, so an Anthropic-compatible surface answers
  it too -- `GET` with that payload, `HEAD` with an empty body.

### Changed

- `t3code`'s recorded launch contract in `data/seed/client-integrations.lino`
  now lists the `pair` and `service` subcommands that t3 0.0.33 added. The
  matrix leg asserts t3's subcommand list verbatim so that a new *prompt* path
  makes the leg fail instead of silently going unexercised; both additions were
  read from the shipped CLI and neither is one -- `pair` mints a pairing token
  and prints it as a QR code, and `service` installs, updates or reports on the
  same server as a background service.
