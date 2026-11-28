"""Bounded, local screenshot fixture; supplied code earns no authorship credit.

Serve the browser regression harness, production worker and seed corpus, substituting only the pre-fix meaning
hydrator on port 8048. Port 8049 reads the current working tree. Native/WASM
execution is deliberately unavailable: this experiment checks the JS worker.
External network access is blocked separately in the Playwright context.
"""
import hashlib
import gzip
import http.server
import json
import pathlib
import subprocess
import threading
import time
import urllib.parse

ROOT = pathlib.Path(__file__).resolve().parents[2]
SOURCE = "js/worker/formal_ai_worker_seed_responses_and_language.js"
BASE = "7f19154a21d79f268f48af6b0938f17e6fb32ba3"
BEFORE = subprocess.check_output(["git", "show", BASE + ":" + SOURCE], cwd=ROOT)
AFTER = (ROOT / SOURCE).read_bytes()
DEADLINE = time.monotonic() + 600


class Handler(http.server.SimpleHTTPRequestHandler):
    def __init__(self, *args, **kwargs):
        super().__init__(*args, directory=str(ROOT / "js"), **kwargs)

    def translate_path(self, address):
        decoded = urllib.parse.unquote(urllib.parse.urlparse(address).path)
        if decoded == "/meaning-regression/":
            return str(ROOT / "experiments/issue-1580/meaning-hydration-browser.html")
        if decoded.startswith("/seed/"):
            base, relative = ROOT / "data/seed", decoded[len("/seed/"):]
        else:
            base, relative = ROOT / "js", decoded.lstrip("/")
        path = (base / relative).resolve()
        if not path.is_relative_to(base.resolve()):
            return str(base / "__not_found__")
        return str(path)

    def do_POST(self):
        # Keep observed browser data separate from production routes and source.
        if self.path != "/__experiment_observation" or self.server.saved_observation:
            self.send_error(404)
            return
        size = int(self.headers.get("Content-Length", "0"))
        if not 0 < size <= 524288:
            self.send_error(413)
            return
        payload = self.rfile.read(size)
        observation = json.loads(payload)
        phase = "before" if self.server.server_port == 8048 else "after"
        assert observation["phase"] == phase and observation["complete"] is True
        out = ROOT / "experiments/issue-1580/evidence/checks/browser-meaning-hydration/browser"
        out.mkdir(parents=True, exist_ok=True)
        (out / (phase + ".json.gz")).write_bytes(gzip.compress(payload, mtime=0))
        receipt = {"phase": phase, "sha256": hashlib.sha256(payload).hexdigest(),
                   "bytes": len(payload), "port": self.server.server_port}
        (out / (phase + "-receipt.json")).write_text(json.dumps(receipt, indent=2) + "\n")
        self.server.saved_observation = True
        response = json.dumps(receipt).encode()
        self.send_response(200)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(response)))
        self.end_headers()
        self.wfile.write(response)

    def do_GET(self):
        self.server.request_count += 1
        if self.server.request_count > 2000 or time.monotonic() >= DEADLINE:
            self.send_error(503, "Bounded screenshot experiment finished")
            return
        target = urllib.parse.urlparse(self.path).path
        if target.endswith(".wasm"):
            self.send_error(404, "JavaScript worker experiment")
            return
        if target == "/worker/" + pathlib.Path(SOURCE).name:
            payload = BEFORE if self.server.server_port == 8048 else AFTER
            self.send_response(200)
            self.send_header("Content-Type", "text/javascript; charset=utf-8")
            self.send_header("Cache-Control", "no-store")
            self.send_header("Content-Length", str(len(payload)))
            self.end_headers()
            self.wfile.write(payload)
            return
        super().do_GET()


servers = [http.server.ThreadingHTTPServer(("0.0.0.0", port), Handler) for port in [8048, 8049]]
for server in servers:
    server.request_count = 0
    server.saved_observation = False
    server.daemon_threads = True
    threading.Thread(target=server.serve_forever, daemon=True).start()
print(json.dumps({"beforeCommit": BASE, "source": SOURCE, "beforeSHA256": hashlib.sha256(BEFORE).hexdigest(),
                  "afterSHA256": hashlib.sha256(AFTER).hexdigest(), "ports": [8048, 8049],
                  "maximumSeconds": 600, "maximumRequestsPerPort": 2000, "nativeBehaviorVerified": False}), flush=True)
try:
    while time.monotonic() < DEADLINE:
        time.sleep(0.5)
finally:
    for server in servers:
        server.shutdown()
        server.server_close()
