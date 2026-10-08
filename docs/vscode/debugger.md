# Formal AI Debugger (#667)

Use **Formal AI: Open Debugger** in either extension host. The dedicated
webview reuses the chat's CSP, Worker origin shim and permission bridge. Web
diagnostics mode renders the same four panes: conversation, public event-log
links, recorded Mermaid source, and recorded executing-source location. Missing
diagrams/source locations are reported as not recorded. No fictitious stages
are generated. Local memory is polled; server mode also reads the existing
`GET /v1/memory/since?event=<id>` Links Notation delta.

## Step-through debug session (R383)

The **Pause**, **Next stage** and **Continue** controls work against a server
started with `--debug-session`. Without one they stay disabled.

### What a stage is

A stage is one thinking step of a solved turn. The solver's event log turns
into these steps the same way for `thinking_steps` in every answer. The solver
runs the whole turn first. The debug session then holds the answer and hands out
its stages one by one. Only after the last stage is advanced does the HTTP
response go out. Stepping never runs a stage again. Each advance shows the next
recorded step of a turn that is already solved.

### Starting a session

```sh
formal-ai serve --debug-session                  # Rust server
node js/server/main.mjs --debug-session          # JavaScript server
```

- **Loopback only.** The flag works only when the server binds to
  `127.0.0.0/8`, `::1` or `localhost`. For any other host the server prints
  `debug_session_requires_loopback:<host>` and refuses to start (exit code 2
  for the JavaScript server, an error from the Rust CLI). The Rust CLI also
  refuses the flag together with `--ws` or `--webrtc`.
- **Token handoff.** The session token comes from
  `FORMAL_AI_DEBUG_SESSION_TOKEN` when the launcher sets it. Otherwise the
  server makes a random 48-hex-digit token. The server prints one stderr line:
  `debug_session=<session id>`, plus `;token=<token>` only when it made the
  token itself. A launcher that set the variable already knows the token, so
  it is not echoed.
- **VS Code.** The setting `formal-ai.debugger.stepping` needs
  `formal-ai.server.enabled` and the desktop (Node) host. With it on, the host
  makes a fresh token and starts `formal-ai serve --debug-session` with the
  token in `FORMAL_AI_DEBUG_SESSION_TOKEN`. Once the health check passes, the
  host adds the token to the webview status as `debugToken`. The token never
  appears on the command line.

### Endpoint

There is one manifest route, `debug_session`, in
`data/meta/server-routes.lino`: `POST /v1/debug/{action}`, where `action` is
`session`, `pause`, `advance` or `release`. It sits behind the server's usual
bearer gate. Every request body is JSON and must carry the session token:

| action | body | effect |
| --- | --- | --- |
| `session` | `{token, since?}` | Reads the state; changes nothing. |
| `pause` | `{token, since?}` | Turns stepping on. The next solved turn pauses at its first stage. |
| `advance` | `{token, turn, stage, since?}` | Advances exactly the paused `stage` of `turn` and pauses at the next one. After the last stage, the turn is released. |
| `release` | `{token, turn?, since?}` | With `turn`, releases that turn. Without it, releases every paused turn and turns stepping off. |

A success answers `200` with the session snapshot:

```json
{
  "events": [ ...events after the first `since` ... ],
  "next": 7,
  "object": "debug.session",
  "paused": [{"detail": "4", "source": "calculation", "stage": 1, "stages": 3, "step": "compute", "turn": "turn_1"}],
  "session": "debug_session_<16 hex>",
  "stepping": true
}
```

Errors use the server's usual `{"error":{"message","type":"formal_ai_error"}}`
envelope:

- `404 debug_session_disabled`: the server runs without `--debug-session`.
- `400 debug_request_invalid`: the body is not a JSON object.
- `401 debug_session_token_invalid`: the token is missing or wrong. The
  comparison takes constant time.
- `409 debug_stage_not_paused`: the `advance` names a stage that is not the
  paused one, or an unknown or released turn. This covers a repeated advance
  of the same stage. Nothing is recorded, so a stage is never advanced twice.

### Events

The session keeps an append-only event log, separate from the shared memory
log. Event ids are `debug_event_<n>`. Every event carries `session`, `turn`
(`turn_<n>`), `kind` and `stage`:

- `stage_paused`: the turn is held at `stage`. It also carries `stages`,
  `step`, `detail` and `source`, where `source` is the solver event kind the
  step came from.
- `stage_advanced`: the paused `stage` was advanced. It carries the same
  fields.
- `turn_released`: the turn's response is free to go. `reason` is one of:
  - `completed`: the last stage was advanced.
  - `released`: a `release` request freed it.
  - `disconnected`: the client that asked for the turn hung up.

### Release semantics

A held turn never outlives the ability to advance it:

- `release` without a turn frees every held turn and turns stepping off.
- A client that disconnects while its turn is held releases that turn. The
  JavaScript server sees the socket close. The Rust server checks the socket
  every 100 ms with a zero-byte peek.
- Only solves made inside an HTTP request are held. Background work never
  pauses, such as the dreaming worker or CLI solves in the same process.
  Stepping starts off, so a debug-session server answers normally until
  someone presses **Pause**.

### Implementation

- JavaScript: `js/server/debug-session.mjs`, with the gate in
  `js/server/solve.mjs` and the request scope in `js/server/http.mjs`.
- Rust: `rust/src/server/debug_session.rs`, with the gate at the end of
  `derivation::finalize_answer` and the connection scope in
  `rust/src/server/transport.rs`.
- Webview: `js/app/debugger-view.jsx`, which polls `session` with a `since`
  cursor and merges the stage events into the event pane.
- Tests: `rust/tests/web/server-debug-session.test.mjs`,
  `rust/tests/unit/specification/debug_session.rs`,
  `vscode/scripts/config.test.mjs` and
  `vscode/scripts/server-process.test.mjs`.

## Remaining acceptance

- A rendered Mermaid graph instead of the source preview.
- Recipe Mermaid output and method-registry `path:symbol` locations recorded
  as public events, so the diagram and source panes fill in during a step.
- Live screenshots or a GIF of a debugger session.
