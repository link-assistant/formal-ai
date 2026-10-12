# Formal AI Debugger (#667)

Use **Formal AI: Open Debugger** in either extension host. The dedicated
webview reuses the chat's CSP, Worker origin shim and permission bridge. Web
diagnostics mode renders the same five panes:

1. **Conversation**: the chat messages.
2. **Event links**: the public event-log links and the selected event's record.
3. **Recipe diagram**: the selected event's Mermaid source, rendered as a graph.
4. **Rust source**: the `path:line`, symbol and excerpt of the Rust code that
   emits the selected stage.
5. **JavaScript source**: the same for the JavaScript server's solver chain.

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

- **Suspended (stage 0, `impulse`):** the solve itself. A turn opens before
  the solver runs, with only its first stage known: the prompt, `stages: 1`,
  no `method`. Until stage 0 is advanced, nothing is solved: no language
  detection, formalization, routing, handler or trace. The JavaScript server
  waits in `solveSymbolic` before `ctx.worker.solve` (`beginTurn`). The Rust
  server waits at the start of
  `solve_with_history_probability_store_and_intent_cache` (`begin_turn`), for
  the outermost solve of a connection only.
- **Running:** after stage 0 is advanced, the turn has no paused stage
  (`paused` omits it, and an `advance` answers 409) while the solver runs.
- **Revealed (stages 1 and on):** the rest of the turn. Both solvers compute
  these stages in one uninterrupted call, so they are held, not suspended:
  `ctx.worker.solve` runs the browser worker's whole pipeline synchronously in
  its realm, and the Rust solve runs its handlers inside one call. The turn
  resumes paused at stage 1 with every stage known (`stages` is now the full
  count), and each advance reveals the next one. Both servers gate before the
  turn's derivation record is persisted
  (`data/cache/derivations/<answer id>.lino`). Nothing that consumes the
  answer has happened yet: no HTTP response, no memory exchange, no
  dialog-log entry. The JavaScript gate sits in `solveSymbolic` before
  `finalizeServerAnswer`. The Rust gate sits in `derivation::finalize_answer`
  before `Derivation::persist`. Stepping never runs a stage again.
- **Not suspended:** the work between stages 1 and on, as above. The agentic
  planner is not gated: it plans one step per request (`planChatStep`), and
  its tool loop spans requests rather than running inside one. A nested Rust
  solve (a replay or a synthesis sub-solve) is held as its own turn when it
  finishes, as before.

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
| `pause` | `{token, since?}` | Turns stepping on. The next turn pauses at its first stage, before it is solved. |
| `advance` | `{token, turn, stage, since?}` | Advances exactly the paused `stage` of `turn` and pauses at the next one. Advancing stage 0 lets the solve run; the turn pauses again at stage 1 once it is solved. After the last stage, the turn is released. |
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
  paused one, or an unknown, released or running turn. This covers a repeated
  advance of the same stage. Nothing is recorded, so a stage is never advanced twice.

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
  - `abandoned`: the solve ended (it threw) without resuming the turn.

The stage fields, also carried by each `paused` entry of the snapshot:

| field | value |
| --- | --- |
| `stages` | how many stages the turn has: 1 while it waits to be solved |
| `step`, `detail` | the thinking step |
| `source` | the solver event kind the step came from |
| `mermaid` | the turn's recipe as a Mermaid `flowchart TD`: top-level stages chained in order, a sub-stage joined to its parent by a dotted edge, and this stage given the `current` class |
| `rust_source`, `rust_line`, `rust_excerpt` | the Rust function that emits this stage, as `path:symbol`, the 1-based line of the definition and the definition through its closing brace (at most 40 lines) |
| `js_source`, `js_line`, `js_excerpt` | the same in the JavaScript server's solver chain (the worker's native event log, `js/agentic/crate`, `js/server`) |
| `method` | the method-registry method the turn's route resolves to (`MethodRegistry::method_for_route` over the `formalize` stage's detail, else the `dispatch_handler` stage's), or empty |
| `method_rust_*`, `method_js_*` | the same three fields for that method's handler in Rust and in the browser worker |

The locations come from the live source tree, nothing is recorded by hand.

A stage's emitter is the function that appends the stage's `source` event:

- **The table.** `data/meta/debug-stage-sources.lino` is written by
  `node scripts/generate-debug-stage-sources.mjs --write`. The CI gate
  `check-debug-stage-sources` runs it with `--check`. For every event kind
  that becomes a thinking step, it lists the functions that append it in each
  runtime: Rust `<log>.append("<kind>", …)`, JavaScript
  `solverEvent("<kind>", …)` and `<log>.push({ kind: "<kind>", … })`. A kind
  named by a string constant of the same file counts. It also lists each
  runtime's source trees and solver entry.
- **One appender:** it is the emitter.
- **Several appenders:** the emitter is the one reachable in the fewest
  calls, at most 2, from the turn's routed handler (below). Failing that, it
  is the one reachable from the solver entry
  (`solve_with_history_probability_store_and_intent_cache`, and
  `nativeSolverLog` in JavaScript). Calls are read by name from the function
  bodies of the runtime's trees. When none is reached, the stage records no
  location; a guess is never recorded.

The routed method's handler:

- **Rust:** the method's row in the `HANDLER_FUNCTIONS` table of
  `rust/src/solver_dispatch.rs`. A method that `data/seed/handler-rules.lino`
  implements resolves to `rust/src/rule_interpreter.rs:run_handler`.
- **JavaScript:** the handler of `data/seed/browser-handler-precedence.lino`
  named `try` plus the method, ignoring case and underscores. A rule-set
  method without one resolves to `runHandlerRuleSet`.

How the lookup works:

- **Lookup:** each symbol is found by its definition line, under the file the
  table names for an emitter, or in sorted file order under `rust/src` or
  `js/worker` for a handler. Each location is looked up once per process.
- **Empty fields:** a route the registry does not resolve (such as
  `greeting`), an event no function appends, or a server without its source
  tree, leaves those `*_source` fields empty and the lines at 0.

The Rust server looks for its source tree at the crate's parent directory,
then at the working directory. The served `What is 2 + 2?` turn, routed to
`arithmetic` (`method_rust_source`
`rust/src/solver_dispatch.rs:handle_arithmetic`, `method_js_source`
`js/worker/formal_ai_worker_arithmetic_and_numeric_lists.js:tryArithmetic`), names these emitters:

| stage | step | Rust emitter | JavaScript emitter |
| --- | --- | --- | --- |
| 0 | `impulse` | `rust/src/solver.rs:solve_with_history_probability_store_and_intent_cache` | `js/worker/formal_ai_worker_solver_events.js:solverEventLog` |
| 1 | `detect_language` | the same | the same |
| 2 | `formalize` | `rust/src/intent_formalization.rs:record_intent_formalization` | `js/agentic/crate/intent_formalization.mjs:recordIntentFormalization` |
| 3–6 | `compute`, `compute_engine`, `compute_expression`, `compute_steps` | `rust/src/solver_handlers/mod.rs:try_arithmetic` | `js/worker/formal_ai_worker_solver_events.js:solverCalculationEvents` |
| 7–9 | `dispatch_handler`, `rule_verification`, `deformalize` | `rust/src/solver_handlers/mod.rs:finalize_simple` | `js/worker/formal_ai_worker_solver_events.js:solverEventLog` |

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

- JavaScript: `js/server/debug-session.mjs`, the stage views in
  `js/server/debug-stage.mjs` and the emitter lookup in
  `js/server/debug-stage-sources.mjs`. The begin and the gate are in
  `js/server/solve.mjs`, and the request scope is in `js/server/http.mjs`.
- Rust: `rust/src/server/debug_session.rs`, `rust/src/server/debug_stage.rs`
  and `rust/src/server/debug_stage_sources.rs`. The begin is at the start of
  the solver entry in `rust/src/solver.rs`, the gate is in
  `derivation::finalize_answer`, and the connection scope is in
  `rust/src/server/transport.rs`.
- Generator: `scripts/generate-debug-stage-sources.mjs` writes
  `data/meta/debug-stage-sources.lino`.
- Webview: `js/app/debugger-view.jsx` over `js/debugger-client.js`, which
  polls `session` with a `since` cursor, merges the stage events into the
  event pane and renders the diagram through `js/mermaid-entry.js`.
- Tests: `rust/tests/web/server-debug-session.test.mjs` (which also checks
  that the table is current),
  `rust/tests/unit/specification/debug_session.rs`,
  `rust/tests/web/debugger-client.test.mjs`,
  `vscode/scripts/debugger-panes.test.mjs`,
  `vscode/scripts/config.test.mjs`,
  `vscode/scripts/server-process.test.mjs`, and the browser test
  `rust/tests/e2e/tests/issue-667-debugger-view.spec.js`. In the built app,
  that test sees stage 0 wait before the solve, and the source panes change
  from stage to stage.

## Remaining acceptance

- Suspending the solver between stages 1 and on. Today these stages are
  revealed from the solved turn. True suspension needs the browser worker's
  `solve` and the Rust handlers to run as resumable steps, not as one call.
- Emitters in JavaScript exist only where the worker mirrors the native
  solver's event log: the prelude, the calculator, the finalize tail, the
  write-program events and the server's formalization records. A stage from
  a Rust-only event kind, such as a concept lookup or a tool call, has no
  JavaScript location.
