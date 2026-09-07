#!/usr/bin/env python3
"""A server that answers every connection with a TCP reset.

Issue #1081: lychee reported `Network error: Connection reset by peer (os error
104)` for a healthy URL and did not retry it, although the run set
`--max-retries 6 --retry-wait-time 2`. This server reproduces that class of
failure deterministically: it accepts a connection and closes it with SO_LINGER
set to zero, which sends RST instead of FIN, so the client's connect/TLS phase
fails exactly the way the CI runner's did.

It prints one line per accepted connection, which is the measurement: the number
of lines is the number of attempts lychee actually made.

    python3 reset-server.py [port] [immediate|after-request]

`immediate` resets on accept, so a TLS client fails during the handshake.
`after-request` reads the request line first, so the reset lands on an
established connection instead.
"""

import socket
import struct
import sys
import time

port = int(sys.argv[1]) if len(sys.argv) > 1 else 8443
mode = sys.argv[2] if len(sys.argv) > 2 else "immediate"

server = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
server.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
server.bind(("127.0.0.1", port))
server.listen(16)
print(f"listening on 127.0.0.1:{port}", flush=True)

started = time.monotonic()
attempts = 0
while True:
    connection, _ = server.accept()
    attempts += 1
    print(f"attempt {attempts} at +{time.monotonic() - started:.2f}s", flush=True)
    if mode == "after-request":
        connection.settimeout(5)
        try:
            connection.recv(4096)
        except OSError:
            pass
    # l_onoff = 1, l_linger = 0: close() sends RST rather than FIN.
    connection.setsockopt(
        socket.SOL_SOCKET, socket.SO_LINGER, struct.pack("ii", 1, 0)
    )
    connection.close()
