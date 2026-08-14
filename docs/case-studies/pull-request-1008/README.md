# Pull Request 1008 Case Study

Pull request [#1008](https://github.com/link-assistant/formal-ai/pull/1008)
implements issue #931 on branch `issue-931-93934954e1d0` without merging
directly to `main`.

## Review scope

Review covers the transport-neutral envelope, synchronous WebSocket adapter,
host-only WebRTC peer connection and local signaling, bounded chunk framing,
shared API permission/memory routing, server/client CLI surfaces, multilingual
HTTP parity, self-hosted authorship evidence, user documentation, dependency
licenses, and release metadata.

This is a native protocol and CLI change rather than visual UI work. The issue
and PR contain no screenshot attachments, and visual before/after evidence is
not applicable.

## Review channels

At the implementation baseline, the PR had no conversation comments, inline
review comments, submitted reviews, or requested changes. All three endpoints
are captured independently under `raw-data/github/`; they are queried again
before finalization so an empty conversation-comments result is not mistaken
for an absence of inline or submitted feedback.

## CI history

The prepared SHA `fd705b57c08fa0b164df907e48d173150ac67e8e` completed CI/CD,
Coverage, External Benchmarks, Security, and Stock Rust Install successfully on
2026-08-14. Those runs predate the implementation. Their IDs, timestamps,
SHAs, conclusions, and URLs are preserved in
`raw-data/github/actions-runs-recent.json`. Fresh runs must match the final
pushed SHA before this PR is marked ready; logs from any non-passing run are
downloaded and read rather than treating the baseline as current evidence.

## Decisions

- Keep HTTP as the unchanged `serve` default and make WS/WebRTC explicit,
  mutually exclusive flags.
- Reuse `handle_api_request_with_headers`; protocol adapters must not own
  permissions, routes, response generation, or memory.
- Use direct `tungstenite` to fit the existing synchronous server model.
- Use `webrtc-rs` host candidates without STUN/TURN and own only a bounded
  loopback offer/answer exchange.
- Chunk the identical JSON envelope instead of inventing a WebRTC-only API.
- Keep diagnostics behind `--transport-trace` so normal CLI output stays quiet.
- Treat the protocol contract as the one-of-five self-hosted leaf and preserve
  both the Agent CLI session and byte-equivalent canonical artifact.

## Verification

The red reproducer demonstrated that both server flags and the client command
were absent. Focused green tests prove real server/client processes, shared
authentication and memory, en/ru/hi/zh parity, WebRTC offer/answer, and
multi-chunk request/response delivery. Final repository-wide checks and fresh
GitHub Actions conclusions are added to the PR description after the last
merge-from-main and push.
