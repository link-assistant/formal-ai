#!/usr/bin/env bash
# Reproduces the js template's variant of D19, for the upstream report against
# link-foundation/js-ai-driven-development-pipeline-template.
#
# `scripts/setup-npm.mjs` installs npm by piping a tarball straight into tar:
#
#   await $`curl -fsSL "${npmRelease.tarballUrl}" | tar xz --strip-components=1 -C "${tempNpmDir}" && ...`
#
# with no retry. This serves the same shape locally -- one truncated response,
# then honest ones -- and measures three variants:
#
#   1. the upstream command: a truncation abandons the strategy;
#   2. the same command with `--retry --retry-all-errors` added in place: curl
#      restarts the transfer and re-emits the head of the file into the pipe,
#      so tar is handed a corrupt stream. Retrying a *piped* download is not a
#      fix;
#   3. download to a file, then extract: recovers cleanly.
#
# Nothing here touches the network.
#
# Usage: experiments/issue-1076/repro-npm-tarball-truncation.sh
set -uo pipefail

work="$(mktemp -d -t repro-npm-tarball-XXXXXX)"
server_pid=""
cleanup() {
  [ -n "$server_pid" ] && kill "$server_pid" 2> /dev/null
  rm -rf "$work"
}
trap cleanup EXIT

# A real gzipped tar, so `tar xz` fails on the truncation rather than on shape.
mkdir -p "$work/pkg/package"
printf '{"name":"npm","version":"11.0.0"}\n' > "$work/pkg/package/package.json"
for i in $(seq 1 200); do
  printf 'lib line %s padding padding padding padding padding\n' "$i"
done > "$work/pkg/package/index.js"
tar czf "$work/npm.tgz" -C "$work/pkg" package

cat > "$work/server.py" <<'PY'
import socket
import sys

BODY = open(sys.argv[1], "rb").read()
attempts = 0

listener = socket.socket()
listener.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
listener.bind(("127.0.0.1", 0))
listener.listen(8)
print(listener.getsockname()[1], flush=True)

while True:
    connection, _ = listener.accept()
    with connection:
        connection.recv(4096)
        attempts += 1
        truncated = attempts == 1
        length = len(BODY) if not truncated else len(BODY) * 2
        connection.sendall(
            b"HTTP/1.1 200 OK\r\nContent-Length: "
            + str(length).encode()
            + b"\r\nConnection: close\r\n\r\n"
        )
        connection.sendall(BODY if not truncated else BODY[: len(BODY) // 2])
PY

python3 "$work/server.py" "$work/npm.tgz" > "$work/port" 2>&1 < /dev/null &
server_pid=$!

port=""
for _ in $(seq 1 50); do
  port="$(head -1 "$work/port" 2> /dev/null)"
  [ -n "$port" ] && break
  python3 -c 'import time; time.sleep(0.1)'
done
[ -n "$port" ] || { echo "server did not start" >&2; exit 1; }
url="http://127.0.0.1:$port/npm-11.0.0.tgz"

run_case() {
  local label="$1"
  shift
  local dest="$work/out-$label"
  rm -rf "$dest"
  mkdir -p "$dest"
  echo "$label:"
  ( "$@" "$dest" ) > "$work/log-$label" 2>&1
  local status=$?
  sed 's/^/    /' "$work/log-$label"
  echo "  exit=$status  files=$(find "$dest" -type f | wc -l)"
  return $status
}

upstream() { curl -fsSL "$url" | tar xz --strip-components=1 -C "$1"; }
piped_retry() {
  curl -fsSL --retry 3 --retry-delay 1 --retry-all-errors "$url" \
    | tar xz --strip-components=1 -C "$1"
}
file_first() {
  local tmp="$work/downloaded.tgz"
  curl -fsSL --retry 3 --retry-delay 1 --retry-all-errors "$url" -o "$tmp" \
    && tar xz --strip-components=1 -C "$1" -f "$tmp"
}

run_case upstream upstream
first=$?
echo
run_case piped-retry piped_retry
second=$?
echo
run_case file-first file_first
third=$?

echo
if [ "$first" -ne 0 ] && [ "$third" -eq 0 ]; then
  echo "REPRODUCED: the upstream command fails on a truncation (exit $first)."
  if [ "$second" -ne 0 ]; then
    echo "Adding --retry to the piped form does not fix it (exit $second): curl"
    echo "restarts the transfer and replays the head of the file into tar."
  else
    echo "NOTE: the piped retry happened to succeed here (exit 0); the file-first"
    echo "form is still the form that cannot replay bytes into the extractor."
  fi
  echo "Downloading to a file and extracting afterwards recovers (exit $third)."
  exit 0
fi
echo "NOT REPRODUCED: upstream=$first piped-retry=$second file-first=$third"
exit 1
