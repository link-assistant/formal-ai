# Formal AI Debugger (#667)

Use **Formal AI: Open Debugger** in either extension host. The dedicated
webview reuses the chat's CSP, Worker origin shim and permission bridge. Web
diagnostics mode renders the same five panes:

1. **Conversation**: the chat messages.
2. **Event links**: the public event-log links and the selected event's record.
3. **Recipe diagram**: the selected event's Mermaid source, rendered as a graph.
4. **Rust source**: the `path:line`, symbol and excerpt of the Rust handler.
5. **JavaScript source**: the same for the browser worker's handler.

While a turn is paused, the event of the paused stage is selected, so the
diagram and both source panes follow **Next stage**. Missing diagrams or source
locations are reported as not recorded. No fictitious stages are generated.
Local memory is polled; server mode also reads the existing
`GET /v1/memory/since?event=<id>` Links Notation delta.

### Rendering the diagram

The view loads `mermaid.bundle.js` only when it has a diagram to show.
`bun run build:web` bundles it from `js/mermaid-entry.js`, which pins `mermaid`
in `package.json` and runs it at the `strict` security level with SVG text
labels. The SVG then goes through DOMPurify's SVG profile. In the VS Code
webview the bundle resolves through the injected `<base>` onto the resource
origin that the CSP already allows. When the bundle is missing (a checkout
without `build:web`) or the source does not parse, the pane shows the Mermaid
source text instead.

The view's non-UI logic is a plain module, `js/debugger-client.js`: the
debug-session client, the poll, the event-to-pane projection and the renderer.
`js/app/debugger-view.jsx` only renders what it returns.

## Step-through debug session (R383)

The **Pause**, **Next stage** and **Continue** controls work against a server
started with `--debug-session`. Without one they stay disabled.

### What a stage is

A stage is one thinking step of a solved turn. The solver's event log turns
into these steps the same way for `thinking_steps` in every answer.

### What is and is not paused

- **Held:** the turn, from the moment its stages are known until the last one
  is advanced or the turn is released. Both servers gate before the turn's
  derivation record is persisted (`data/cache/derivations/<answer id>.lino`).
  Nothing that consumes the answer has happened yet: no HTTP response, no
  memory exchange, no dialog-log entry. The JavaScript gate sits in
  `solveSymbolic` before `finalizeServerAnswer`. The Rust gate sits in
  `derivation::finalize_answer` before `Derivation::persist`.
- **Not paused:** the solver computation itself. The handler dispatch, the
  handler and the trace projection have already run when the first stage
  pauses, so stepping reveals stages one by one without suspending work
  between them. Stepping never runs a stage again.

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

- `stage_paused`: the turn is held at `stage`. It also carries the stage
  fields below.
- `stage_advanced`: the paused `stage` was advanced. It carries the same
  fields.
- `turn_released`: the turn's response is free to go. `reason` is one of:
  - `completed`: the last stage was advanced.
  - `released`: a `release` request freed it.
  - `disconnected`: the client that asked for the turn hung up.

The stage fields, also carried by each `paused` entry of the snapshot:

| field | value |
| --- | --- |
| `stages` | how many stages the turn has |
| `step`, `detail` | the thinking step |
| `source` | the solver event kind the step came from |
| `mermaid` | the turn's recipe as a Mermaid `flowchart TD`: top-level stages chained in order, a sub-stage joined to its parent by a dotted edge, and this stage given the `current` class |
| `method` | the method-registry method the turn's route resolves to (`MethodRegistry::method_for_route` over the `formalize` stage's detail, else the `dispatch_handler` stage's), or empty |
| `rust_source`, `rust_line`, `rust_excerpt` | where that method runs in Rust, as `path:symbol`, the 1-based line of the definition and the definition through its closing brace (at most 40 lines) |
| `js_source`, `js_line`, `js_excerpt` | the same for the browser worker |

The locations come from the live source tree, nothing is recorded by hand:

- **Rust:** the method's row in the `HANDLER_FUNCTIONS` table of
  `rust/src/solver_dispatch.rs`. A method that `data/seed/handler-rules.lino`
  implements resolves to `rust/src/rule_interpreter.rs:run_handler`.
- **JavaScript:** the handler of `data/seed/browser-handler-precedence.lino`
  named `try` plus the method, ignoring case and underscores. A rule-set
  method without one resolves to `runHandlerRuleSet`.
- **Lookup:** each symbol is found by its definition line, in sorted file
  order, under `rust/src` or `js/worker`. Each method is looked up once per
  process.
- **Empty fields:** a route the registry does not resolve (such as
  `greeting`), or a server without its source tree, leaves the `*_source`
  fields empty and the lines at 0.

The Rust server looks for its source tree at the crate's parent directory,
then at the working directory. Example, for a turn routed to `arithmetic`:

```json
{"method": "arithmetic", "rust_source": "rust/src/solver_dispatch.rs:handle_arithmetic",
 "js_source": "js/worker/formal_ai_worker_06.js:tryArithmetic", "rust_line": 78, "...": "..."}
```

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

- JavaScript: `js/server/debug-session.mjs` and the stage views in
  `js/server/debug-stage.mjs`, with the gate in `js/server/solve.mjs` and the
  request scope in `js/server/http.mjs`.
- Rust: `rust/src/server/debug_session.rs` and
  `rust/src/server/debug_stage.rs`, with the gate in
  `derivation::finalize_answer` and the connection scope in
  `rust/src/server/transport.rs`.
- Webview: `js/app/debugger-view.jsx` over `js/debugger-client.js`, which
  polls `session` with a `since` cursor, merges the stage events into the
  event pane and renders the diagram through `js/mermaid-entry.js`.
- Tests: `rust/tests/web/server-debug-session.test.mjs`,
  `rust/tests/unit/specification/debug_session.rs`,
  `rust/tests/web/debugger-client.test.mjs`,
  `vscode/scripts/debugger-panes.test.mjs`,
  `vscode/scripts/config.test.mjs` and
  `vscode/scripts/server-process.test.mjs`.

## Remaining acceptance

- Suspending the solver between its stages, not only holding the solved turn
  before it is persisted and answered.
- Per-stage source locations: every stage of a turn names the method its
  route resolves to, not the code that emitted that particular step.
- A browser test of the view inside the built app. The server payloads, the
  event-to-pane projection, the renderer (with an injected bundle) and the
  webview CSP are pinned by tests. The real bundle was checked by hand in
  headless Chromium: five nodes, four edges, the current stage highlighted.
- Live screenshots or a GIF of a debugger session.
