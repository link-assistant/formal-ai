# Issue 931: local WebSocket and WebRTC transports

Source: [issue #931](https://github.com/link-assistant/formal-ai/issues/931) and
the referenced maintainer comment on issue #107.

| ID | Requirement | Status and evidence |
| --- | --- | --- |
| R931-1 | Serve the HTTP/OpenAI-compatible request and response shape over WebSocket, bound to localhost by default. | Implemented by `formal-ai serve --ws`, `src/local_transport.rs`, and transport-parity integration tests. |
| R931-2 | Serve the same API over a local-first WebRTC data channel with peer-to-peer access and no central relay. | Implemented with host-only ICE, loopback offer/answer signaling, no STUN/TURN configuration, and a real data-channel smoke test. |
| R931-3 | Use one CLI binary as WebSocket/WebRTC server and client. | Implemented by `formal-ai serve --ws|--webrtc` and `formal-ai connect --transport websocket|webrtc`; tested as subprocesses. |
| R931-4 | Reuse the permission and memory model rather than creating storage. | Both transports call `handle_api_request_with_headers`; tests prove authentication failures and memory writes. |
| R931-5 | Compare a Rust WebSocket client's full chat round trip with HTTP. | `issue_931_websocket_and_webrtc_answers_match_http_byte_for_byte` compares status, content type, and answer bytes. |
| R931-6 | Exercise one WebRTC loopback offer/answer and full request/response. | The integration suite opens a real peer connection and exchanges the complete transport envelope. |
| R931-7 | Document manual `websocat` and separately running CLI-client verification. | `docs/local-transports.md` contains both workflows and the complete wire contract. |
| R931-8 | Confirm en/ru/hi/zh WebSocket answers match HTTP byte-for-byte. | The integration matrix exercises `Hi`, `Привет`, `नमस्ते`, and `你好`; WebRTC is checked against the same bytes. |
| R931-9 | Preserve issue data, every requirement, a WebRTC Rust library survey, root cause, and solution plan in an issue case study. | `docs/case-studies/issue-931/` contains collector output, primary-source research, full analysis, and this generated-requirements shard. |
| R931-10 | Keep opt-in verbose transport lifecycle diagnostics if needed for root-cause work. | `--transport-trace` covers WebSocket connection and WebRTC signaling/channel/state/errors; default is off. |
| R931-11 | Deliver the issue in one PR with release metadata. | PR #1008 contains code, tests, docs, evidence, and a minor changelog fragment. |
| R931-12 | Produce at least 20% of the smallest task leaves through the real Formal AI Agent CLI and preserve proof. | One of five leaves is Agent-authored; session and exact-byte verification are under `docs/case-studies/issue-931/self-hosting-authorship/`. |
