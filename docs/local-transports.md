# Local WebSocket and WebRTC transports

`formal-ai` can expose the same OpenAI-compatible router over HTTP,
WebSocket, or a WebRTC data channel. All modes bind to `127.0.0.1:8080` by
default, use the same shared memory, and apply the same bearer-token checks.
No additional storage engine is involved.

## Start a server

HTTP remains the default, so existing commands do not change:

```bash
formal-ai serve
formal-ai serve --ws
formal-ai serve --webrtc
```

`--ws` and `--webrtc` are mutually exclusive. `--host` and `--port` work for
all three modes. Binding to a non-loopback host exposes an unauthenticated
server unless a token is configured, so set one before doing that:

```bash
FORMAL_AI_API_BEARER_TOKEN=local-secret formal-ai serve --ws
```

The legacy `FORMAL_AI_HTTP_BEARER_TOKEN` and `FORMAL_AI_API_TOKEN` names remain
accepted by the shared permission layer. Add `--transport-trace` to print
connection, WebRTC signaling, data-channel, and error lifecycle events. It is
off by default.

## Use the same binary as a client

Start a server in one terminal and connect from another:

```bash
formal-ai connect \
  --transport websocket \
  --endpoint 127.0.0.1:8080 \
  --prompt 'Hi'

formal-ai connect \
  --transport webrtc \
  --endpoint 127.0.0.1:8080 \
  --prompt 'Привет'
```

Pass `--api-key local-secret` when the server requires authentication. The
client also reads the three token environment variables listed above. Text is
printed by default; `--format json` prints the complete OpenAI-compatible
response body.

## WebSocket envelope

Each RFC 6455 text message is one JSON request. The response is another text
message with the router's complete status, content type, body, and deprecation
marker:

```json
{
  "method": "POST",
  "path": "/v1/chat/completions",
  "headers": {"content-type": "application/json"},
  "body": "{\"model\":\"formal-ai\",\"messages\":[{\"role\":\"user\",\"content\":\"Hi\"}]}"
}
```

The same request can be checked with a generic client such as `websocat`:

```bash
printf '%s\n' '{"method":"POST","path":"/v1/chat/completions","headers":{"content-type":"application/json"},"body":"{\"model\":\"formal-ai\",\"messages\":[{\"role\":\"user\",\"content\":\"Hi\"}]}"}' \
  | websocat -n1 ws://127.0.0.1:8080
```

Add `"authorization":"Bearer local-secret"` to `headers` when authentication
is enabled. Invalid envelopes receive status `400` in the same response shape.

## WebRTC protocol

WebRTC uses host ICE candidates only: Formal AI configures no STUN server,
TURN server, or central relay. The listening TCP address performs local
offer/answer signaling and does not carry API requests:

1. The client opens TCP and writes a four-byte big-endian length followed by a
   JSON `RTCSessionDescription` offer.
2. The server replies with the same framing for its JSON answer.
3. The reliable, ordered `formal-ai` data channel carries the JSON envelope
   documented above.
4. Each binary channel message begins with `0` when more chunks follow or `1`
   for the final chunk. Chunks contain at most 12 KiB; a reassembled request or
   response is limited to 16 MiB.

This deliberately small signaling protocol makes two local processes establish
the peer connection without operating a signaling or relay service. The Rust
client is the interoperable reference implementation. A browser client can use
the same offer/answer and envelope contract, but browser integration is not
bundled in this native CLI change.

## Shared behavior

The adapters call `handle_api_request_with_headers`, the same transport-neutral
router used by HTTP. Consequently model aliases, `/v1/*` and `/api/*` routes,
permission checks, deterministic multilingual answers, and memory writes have
one implementation. Responses are currently returned as complete envelope
bodies; a requested HTTP SSE body is not split into incremental WebSocket or
data-channel events.
