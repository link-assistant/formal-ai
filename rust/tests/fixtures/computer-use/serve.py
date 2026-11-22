"""Loopback publisher for the archived computer-use replay."""
from http.server import BaseHTTPRequestHandler, HTTPServer
from pathlib import Path

ROOT = Path(__file__).resolve().parent
RESPONSES = {
    "/orders.json": ("orders.json", "application/json"),
    "/form.html": ("form.html", "text/html"),
    "/status.html": ("status.html", "text/html"),
    "/inventory.csv": ("inventory.csv", "text/csv"),
    "/submit": ("submission.json", "application/json"),
}


class Handler(BaseHTTPRequestHandler):
    def log_message(self, *_):
        pass

    def respond(self, status, content_type, data):
        self.send_response(status)
        self.send_header("Content-Type", content_type)
        self.send_header("Content-Length", str(len(data)))
        self.end_headers()
        self.wfile.write(data)

    def replay(self):
        filename, content_type = RESPONSES[self.path]
        self.respond(200, content_type, (ROOT / filename).read_bytes())

    def do_GET(self):
        if self.path == "/redirect":
            self.send_response(302)
            self.send_header("Location", "/status.html")
            self.end_headers()
        elif self.path == "/caller-data":
            self.respond(200, "text/plain; charset=utf-8", "caller α\n".encode())
        elif self.path in RESPONSES and self.path != "/submit":
            self.replay()
        else:
            self.respond(404, "text/plain", b"missing")

    def do_POST(self):
        length = int(self.headers.get("Content-Length", "0"))
        if self.path != "/submit" or length != len(b"token=fixture-token"):
            self.respond(400, "text/plain", b"invalid request")
        elif self.rfile.read(length) == b"token=fixture-token":
            self.replay()
        else:
            self.respond(400, "text/plain", b"invalid request")


with HTTPServer(("127.0.0.1", 0), Handler) as server:
    print(f"http://127.0.0.1:{server.server_port}", flush=True)
    server.serve_forever()
