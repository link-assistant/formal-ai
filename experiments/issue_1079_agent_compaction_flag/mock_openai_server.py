#!/usr/bin/env python3
"""A minimal OpenAI-compatible server, enough for one Agent CLI turn.

Issue #1079 needs to show *which model* the Agent CLI reaches for, not what it
says, so this answers every completion with the same fixed sentence. Standard
library only: the reproduction has to run anywhere the CLI does, including in
the upstream repository, which is not a Rust project.
"""

import json
import os
import sys
import time
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

MODEL = "mock-model"
REPLY = "ok"

# The agent's turn streams; the session summarizer does not. Failing only the
# non-streaming request therefore fails *only* the summary, which is the exact
# shape of the upstream defect: a turn that succeeded, reported as a failure.
FAIL_NON_STREAMING = os.environ.get("MOCK_FAIL_NON_STREAMING") == "1"

# Seconds to hold the stream open between the first token and the last. Real
# agent turns run for minutes, so the process is still alive when the summary
# gives up after its retries; a one-second mock turn exits first and the
# rejection never lands. The delay restores the ordering CI actually sees.
STREAM_DELAY = float(os.environ.get("MOCK_STREAM_DELAY") or 0)


class Handler(BaseHTTPRequestHandler):
    protocol_version = "HTTP/1.1"

    def _send(self, payload, status=200):
        body = json.dumps(payload).encode()
        self.send_response(status)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def do_GET(self):
        if self.path.endswith("/models"):
            self._send(
                {
                    "object": "list",
                    "data": [{"id": MODEL, "object": "model", "owned_by": "mock"}],
                }
            )
            return
        self._send({"error": "not found"}, status=404)

    def _send_stream(self):
        """Answer `"stream": true` as SSE.

        The AI SDK rejects a plain JSON body for a streaming request with
        `AI_InvalidResponseDataError`, and that error would sit in the log next
        to the one this reproduction is about.
        """
        self.send_response(200)
        self.send_header("Content-Type", "text/event-stream")
        self.send_header("Cache-Control", "no-cache")
        # An SSE body has no Content-Length, so on a keep-alive HTTP/1.1
        # connection the client waits for bytes that never come. Closing the
        # connection is what ends the stream, and without it the turn hangs
        # until the harness timeout -- which would hide the very thing this
        # reproduction measures.
        self.send_header("Connection", "close")
        self.close_connection = True
        self.end_headers()
        chunks = [
            {"delta": {"role": "assistant", "content": REPLY}, "finish_reason": None},
            {"delta": {}, "finish_reason": "stop"},
        ]
        for index, choice in enumerate(chunks):
            if index and STREAM_DELAY:
                time.sleep(STREAM_DELAY)
            payload = {
                "id": "chatcmpl-mock",
                "object": "chat.completion.chunk",
                "created": 0,
                "model": MODEL,
                "choices": [dict(choice, index=0)],
            }
            self.wfile.write(f"data: {json.dumps(payload)}\n\n".encode())
        self.wfile.write(b"data: [DONE]\n\n")
        self.wfile.flush()

    def do_POST(self):
        length = int(self.headers.get("Content-Length") or 0)
        raw = self.rfile.read(length)
        try:
            streaming = bool(json.loads(raw or b"{}").get("stream"))
        except ValueError:
            streaming = False
        if streaming:
            self._send_stream()
            return
        if FAIL_NON_STREAMING:
            # 400, not 500: the CLI retries server errors with exponential
            # backoff, and three rounds of that outlast the turn. A client
            # error is not retried, so the rejection lands while the session is
            # still running -- which is the ordering the real failure had, an
            # `AI_APICallError` from the gateway rather than a retry timeout.
            self._send(
                {"error": {"message": "mock summary failure", "type": "MockError"}},
                status=400,
            )
            return
        self._send(
            {
                "id": "chatcmpl-mock",
                "object": "chat.completion",
                "created": 0,
                "model": MODEL,
                "choices": [
                    {
                        "index": 0,
                        "message": {"role": "assistant", "content": REPLY},
                        "finish_reason": "stop",
                    }
                ],
                "usage": {
                    "prompt_tokens": 1,
                    "completion_tokens": 1,
                    "total_tokens": 2,
                },
            }
        )

    def log_message(self, *_args):
        pass


if __name__ == "__main__":
    port = int(sys.argv[1]) if len(sys.argv) > 1 else 8123
    # Threading matters: the CLI summarizes a turn *while* the turn is still
    # streaming, so a single-threaded server deadlocks the very interleaving
    # this reproduction is about.
    ThreadingHTTPServer(("127.0.0.1", port), Handler).serve_forever()
