**Problem statement.**
Source: #107 comment (https://github.com/link-assistant/formal-ai/issues/107#issuecomment-4481573815). konard: "make sure we have local server, that will be available at localhost at WebSocket and WebRTC protocols ... So CLI should be a server and client" — i.e. fully local agent-storage server reachable the same way the CLI itself is used, giving one simple interface to Formal AI.
Current-code evidence: no WebSocket or WebRTC code exists anywhere in `src/` (grep clean); `src/network_endpoint.rs` only implements plain HTTP. No delivery evidenced in #114 or later.

**What to do.**
1. Add a local WebSocket server mode to `formal-ai serve` (or a new subcommand) that speaks the same request/response shape as the existing HTTP/OpenAI-compatible surface, bound to localhost by default.
2. Add a WebRTC data-channel mode for the same local-first use case (peer-to-peer agent storage access without a central relay).
3. Make the CLI itself capable of acting as both server (spin up the local WS/WebRTC endpoint) and client (connect to a running local endpoint) through one binary, so `formal-ai` is "a simple interface to Formal AI" per the requirement.
4. Reuse the existing permission/memory model — this is a transport addition, not a new storage engine.

**How to test.**
- Automated: integration test that starts the WS server, connects a WS client (Rust test harness), round-trips a chat request, and asserts parity with the HTTP path on the same prompt; a WebRTC data-channel smoke test (loopback offer/answer) exercising one full request/response.
- Manual: `formal-ai serve --ws` then connect with a generic WebSocket client (e.g. websocat) and confirm a working chat exchange; confirm CLI-as-client mode connects to a separately-running server instance.
- Multilingual: run one en/ru/hi/zh prompt each over the WS transport and confirm answers match the HTTP-transport answers byte-for-byte (determinism doctrine).
- Standing clauses: docs/case-studies/issue-{id} with full requirement list, WebRTC-in-Rust library survey (e.g. webrtc-rs), and solution plan; verbose logging on connection lifecycle if debugging is needed; single PR.

**Source refs:** #107. **Dedup:** none.

