#!/usr/bin/env bash
# Reproduces D19: a mid-transfer truncation fails a CI install step outright,
# because the download carries no retry.
#
# Run 33967170904, leg `E2E (opencode-vscode)`, failed with
#
#   -- installing opencode-vscode via tarball (1.135.0), command 'code'
#   curl: (18) transfer closed with 344439862 bytes remaining to read
#   !! downloading VS Code 1.135.0 failed
#
# on a commit that touched no shell script -- the 345 MB vendor tarball simply
# stopped arriving. `curl -fsSL` treats that as final, so one dropped connection
# is a red build: a false positive in the sense issue #1076 uses the word.
#
# This serves the same shape locally: the first request gets a `Content-Length`
# it never fulfils and a closed socket, every later request gets the whole body.
# Nothing here touches the network.
#
# Usage: experiments/issue-1076/repro-curl-truncated-download.sh
set -uo pipefail

work="$(mktemp -d -t repro-curl-truncated-XXXXXX)"
server_pid=""
cleanup() {
  [ -n "$server_pid" ] && kill "$server_pid" 2> /dev/null
  rm -rf "$work"
}
trap cleanup EXIT

cat > "$work/server.py" <<'PY'
"""Serve one truncated response, then serve honestly.

The failure this imitates is curl exit 18 (`CURLE_PARTIAL_FILE`): the server
promised more bytes than it delivered and then hung up. Declaring a
`Content-Length` of twice the body and writing half of it is that promise,
broken the same way.
"""

import socket

BODY = b"vendor-tarball-bytes\n" * 64
attempts = 0


def serve(sock):
    global attempts
    while True:
        connection, _ = sock.accept()
        with connection:
            connection.recv(4096)
            attempts += 1
            truncated = attempts == 1
            length = len(BODY) * 2 if truncated else len(BODY)
            connection.sendall(
                b"HTTP/1.1 200 OK\r\n"
                b"Content-Length: " + str(length).encode() + b"\r\n"
                b"Connection: close\r\n\r\n"
            )
            connection.sendall(BODY if not truncated else BODY[: len(BODY) // 2])


listener = socket.socket()
listener.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
listener.bind(("127.0.0.1", 0))
listener.listen(8)
print(listener.getsockname()[1], flush=True)
serve(listener)  # blocks until the shell's cleanup kills this process
PY

python3 "$work/server.py" > "$work/port" 2>&1 < /dev/null &
server_pid=$!

port=""
for _ in $(seq 1 50); do
  port="$(head -1 "$work/port" 2> /dev/null)"
  [ -n "$port" ] && break
  python3 -c 'import time; time.sleep(0.1)'
done
[ -n "$port" ] || { echo "server did not start" >&2; exit 1; }

url="http://127.0.0.1:$port/vscode.tar.gz"

echo "curl without retry (what CI ran):"
curl -fsSL "$url" -o "$work/a.tar.gz"
without=$?
echo "  exit=$without   $(curl --version | head -1 | cut -d' ' -f1-2)"

echo
echo "curl with the repository's retry idiom:"
curl -fsSL --retry 3 --retry-delay 2 --retry-all-errors "$url" -o "$work/b.tar.gz"
with=$?
echo "  exit=$with"

echo
if [ "$without" -eq 18 ] && [ "$with" -eq 0 ]; then
  echo "REPRODUCED: a truncated transfer is fatal without --retry (exit 18) and"
  echo "recovered with --retry --retry-all-errors, which is what the CI download"
  echo "was missing. --retry alone does not cover exit 18; --retry-all-errors does."
  exit 0
fi
echo "NOT REPRODUCED: without=$without with=$with (expected 18 and 0)"
exit 1
