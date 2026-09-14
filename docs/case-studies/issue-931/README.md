# Issue 931 Case Study

Issue [#931](https://github.com/link-assistant/formal-ai/issues/931) asks for
one `formal-ai` binary to act as a localhost server and client over WebSocket
and WebRTC while retaining the existing OpenAI-compatible behavior, permission
checks, and memory. The implementation adds transport adapters around the
existing router; it does not fork application behavior or add storage.

## 1. Collected evidence and timeline

The collector manifest under `raw-data/github/` records the source issue,
empty issue-comment stream, source issue #107 and its maintainer comment,
related PR #114, and prepared PR #1008 with all three review channels.
There are no screenshots or image attachments in the source material.

- 2026-05-17: issue #107 reported the Russian web request and asked in a
  maintainer comment for localhost WebSocket and WebRTC access with the CLI as
  both server and client.
- 2026-05-18: PR #114 fixed Russian URL requests and browser web search, but
  contained no local transport delivery.
- 2026-08-04: issue #931 separated the unimplemented transport requirement into
  acceptance criteria and explicit tests.
- 2026-08-14: PR #1008 was prepared. Before implementation, `serve --ws`,
  `serve --webrtc`, and `connect` all failed Clap argument parsing. The saved
  reproducer is described below.

## 2. Complete requirement matrix

| ID | Requirement | Implemented evidence |
| --- | --- | --- |
| R931-1 | Add a localhost-default WebSocket server mode with the HTTP/OpenAI-compatible request and response shape. | `formal-ai serve --ws`, `TransportRequest`, `TransportResponse`, and the shared dispatch parity integration test. |
| R931-2 | Add local-first WebRTC data-channel access without a central relay. | `formal-ai serve --webrtc`; host-only ICE, no STUN/TURN configuration, loopback TCP offer/answer, and the loopback smoke test. |
| R931-3 | Make the same CLI binary a server and a client for both transports. | `formal-ai connect --transport websocket|webrtc` and the subprocess whole-task test. |
| R931-4 | Reuse existing permissions and memory; do not introduce storage. | Both adapters call `handle_api_request_with_headers`; tests prove 401 parity and memory writes. |
| R931-5 | Round-trip WebSocket chat from a Rust client and compare it with HTTP. | `issue_931_websocket_and_webrtc_answers_match_http_byte_for_byte`. |
| R931-6 | Exercise one full WebRTC loopback offer/answer and request/response. | The same integration test starts a real server and completes ICE, data-channel open, request, and response. |
| R931-7 | Document and manually verify a generic WebSocket client and a separately running CLI client. | `docs/local-transports.md` includes `websocat` and two-terminal CLI commands; the CLI path is automated too. |
| R931-8 | Prove English, Russian, Hindi, and Chinese WebSocket answers are byte-identical to HTTP. | The integration test enumerates `en`, `ru`, `hi`, and `zh`; it also proves WebRTC parity. |
| R931-9 | Preserve a full case study, all requirements, library research, and a solution plan. | This document, `raw-data/online-research.md`, the collector manifest, and `docs/requirements/issue-0931-local-transports.md`. |
| R931-10 | Retain opt-in verbose lifecycle diagnostics if transport debugging needs them. | `--transport-trace` logs connections, signaling, state, channel lifecycle, and errors; default is off. |
| R931-11 | Deliver everything in one PR with release metadata. | PR #1008 and the minor changelog fragment. |
| R931-12 | Use the real Formal AI Agent CLI for at least 20% of the smallest leaves and preserve evidence. | One of five leaves authored the canonical protocol contract; session `ses_0010ce72effeGX9asCkWbyLGd9` and byte-parity evidence live in `self-hosting-authorship/`. |

## 3. Reproduction and root cause

The pre-fix release binary produced exit status 2 for every new surface; the
complete output is preserved in `raw-data/repro-before.log`:

```text
formal-ai --silent serve --ws       # unexpected argument '--ws'
formal-ai --silent serve --webrtc   # unexpected argument '--webrtc'
formal-ai --silent connect          # unrecognized subcommand 'connect'
```

The server already had the correct architectural seam:
`handle_api_request_with_headers` accepts a method, path, headers, and body and
returns status, content type, body, and deprecation metadata. The missing piece
was not application logic or persistence. `src/server/transport.rs` was the
only network adapter and projected that router exclusively onto HTTP; Clap's
`Serve` command exposed only host, port, and agent mode; no client command
existed. This is why changing the network endpoint or memory engine would have
duplicated behavior instead of fixing the actual gap.

## 4. Existing components and library survey

The detailed source notes are in `raw-data/online-research.md`.

- `tungstenite` was selected for the RFC 6455 adapter. The native server is
  deliberately blocking and thread-per-connection like the existing server,
  so adding `tokio-tungstenite` or an async HTTP framework would add a second
  execution model without improving the local protocol.
- `webrtc-rs/webrtc` was selected because it implements peer connections,
  ICE, SDP, SCTP, and data channels in Rust and permits explicit host UDP
  addresses. It fits the no-relay requirement without a browser or C library.
- `str0m` is a credible Sans-I/O alternative, but its application would need to
  own more ICE, timing, and UDP orchestration. That is useful for a custom event
  loop, not the smallest adapter around this synchronous CLI.
- `datachannel-rs` wraps libdatachannel and would add a native C++ build/runtime
  dependency. That tradeoff is unnecessary for the required Rust loopback.

No external implementation was copied. The transport envelope is derived from
Formal AI's existing router fields. The one-byte chunk marker and length-
prefixed local signaling are repository-specific glue described in
`docs/local-transports.md`.

## 5. Implemented design

`src/local_transport.rs` owns the transport-neutral serializable envelope and
both adapters. A WebSocket message contains one envelope. WebRTC exchanges SDP
over a bounded local TCP signaling connection, then sends that identical
envelope through a reliable ordered channel named `formal-ai`. It advertises
host candidates only and chunks payloads at 12 KiB, with a 16 MiB reassembly
limit, so responses do not depend on one SCTP message accepting the entire API
body.

`src/cli_local_transport.rs` keeps `serve`'s HTTP default and adds mutually
exclusive `--ws`/`--webrtc` flags plus the `connect` client. Authentication
headers and `--agent-mode` reach the existing global permission/agent policy.
All modes continue to announce the existing shared-memory path and start the
same dreaming runtime.

## 6. Requirement-by-requirement solution plan

1. Freeze the common envelope in Links Notation through the real Agent CLI and
   guard the exact bytes.
2. Add a red whole-task test that invokes the missing server flags and client
   command.
3. Project the transport-neutral router through WebSocket, then prove auth,
   memory, and HTTP response parity.
4. Add host-only WebRTC offer/answer and reliable chunked data-channel framing,
   then reuse the same request test.
5. Cover all four supported languages, both CLI roles, and bodies larger than a
   single WebRTC chunk.
6. Document the protocol and library tradeoffs, publish release metadata, run
   repository-wide policy checks, and validate fresh PR CI at the pushed SHA.

## 7. Verification

The red compiler output and focused green run are preserved as
`raw-data/red-regression.log` and `raw-data/green-focused-tests.log`. The
focused integration run passes two tests: full HTTP/WS/WebRTC parity and
same-binary CLI operation. It covers authorization failures, memory writes,
English/Russian/Hindi/Chinese answer bytes, a 20 KiB request, and a response
larger than 16 KiB. Three focused contract/documentation tests also pass and
hold the Agent CLI artifact byte-for-byte against its canonical copy.
The documented generic-client command was also executed with `websocat 1.14.1`;
`raw-data/manual-websocat.log` records its status-200 greeting and resulting
shared-memory write.

This is a native protocol change, not visual UI work, so before/after
screenshots and visual regression tests are not applicable. Final full-suite
and CI results are recorded in PR #1008 after the implementation SHA is pushed.
