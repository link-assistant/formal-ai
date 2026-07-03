### Fixed

- Covered the legacy `/v1/responses` route with a real loopback HTTP streaming
  regression test so Codex-style Responses SSE clients must receive
  `response.completed`.

### Documentation

- Documented a copy-paste Codex 0.142+ configuration and `codex exec "hi"`
  command for driving Formal AI through the Responses wire API.
