# Formal AI Debugger (#667)

Use **Formal AI: Open Debugger** in either extension host. The dedicated
webview reuses the chat's CSP, Worker origin shim and permission bridge. Web
diagnostics mode renders the same four panes: conversation, public event-log
links, recorded Mermaid source, and recorded executing-source location. Missing
diagrams/source locations are reported as not recorded. No fictitious stages
are generated. Local memory is polled; server mode also reads the existing
`GET /v1/memory/since?event=<id>` Links Notation delta.

Pause/Next stage controls are disabled in this draft. Server integration needs:

1. An off-by-default `SolverConfig` debug flag, accepted only for explicitly
   opted-in loopback sessions; reject the flag for remote addresses.
2. A solve session id, durable stage-paused/stage-advanced events, a cancellation
   token, and an authenticated session advance endpoint. Cancellation/disconnect
   must release a paused solve. Advancing must not execute a stage twice.
3. Store recipe Mermaid output and method-registry `path:symbol` locations as
   public links/events. The projection currently reads `mermaid`/`diagram` and
   `sourceLocation`/`source_location` fields or matching event kinds. Align the
   final backend vocabulary before enabling controls.

Remaining acceptance: real rendered Mermaid graph rather than source preview,
three-stage solve stepping, loopback refusal tests, and actual debugger session
screenshots/GIF. No builds/tests or live-session capture were run.
