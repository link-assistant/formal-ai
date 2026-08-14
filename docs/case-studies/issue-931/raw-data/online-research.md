# Online research for issue 931

Consulted 2026-08-14. Primary project and standards sources were preferred.

## WebSocket

- [RFC 6455](https://datatracker.ietf.org/doc/html/rfc6455) defines the
  WebSocket protocol, including text/binary messages and control frames.
- [`tungstenite` 0.30.0 documentation](https://docs.rs/tungstenite/0.30.0/tungstenite/)
  describes a lightweight Rust WebSocket implementation with client and server
  handshakes and no mandatory async runtime.
- [tungstenite-rs source](https://github.com/snapview/tungstenite-rs) records
  MIT or Apache-2.0 licensing. The resolved local crate metadata is version
  0.30.0 with the same license expression.

Decision: use `tungstenite` directly because Formal AI's native HTTP server is
also synchronous. `tokio-tungstenite` and framework-integrated WebSocket
extractors solve async integration that this adapter does not need.

## WebRTC

- [WebRTC for the Curious: data communication](https://webrtcforthecurious.com/docs/04-data-communication/)
  explains that WebRTC data channels use SCTP over DTLS over ICE/UDP and can be
  configured for ordered reliable delivery.
- [webrtc-rs source](https://github.com/webrtc-rs/webrtc) describes a pure Rust
  WebRTC stack based on Pion. Its resolved crate metadata is version 0.20.2,
  repository `webrtc-rs/webrtc`, licensed MIT or Apache-2.0.
- [`webrtc` crate documentation](https://docs.rs/webrtc/0.20.2/webrtc/) exposes
  peer connection, ICE, SDP, and data-channel APIs used by this implementation.
- [`str0m` source](https://github.com/algesten/str0m) offers a Sans-I/O WebRTC
  design. It gives the application more control but also requires it to drive
  protocol inputs, outputs, and timing.
- [`datachannel-rs` source](https://github.com/lerouxrgd/datachannel-rs) exposes
  WebRTC data channels through libdatachannel bindings, introducing a C++
  native dependency.

Decision: use `webrtc-rs` for an in-process Rust peer connection with explicit
host addresses and no configured STUN/TURN service. The application still owns
small local signaling because WebRTC intentionally does not prescribe a
signaling transport. Keeping the offer/answer socket on the CLI's selected
address avoids adding any central service.

## Repository prior art

- The source issue #107 and related PR #114 are captured under `github/`. PR
  #114 delivered browser web-search work but no WebSocket/WebRTC transport.
- `src/server.rs` already centralizes request routing, API permissions, and
  shared-memory behavior. Reuse of that handler is preferable to adding a
  protocol-specific router or storage path.
