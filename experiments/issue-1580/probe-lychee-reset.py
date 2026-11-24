"""Observe the published lychee binary's retries at two real TCP phases."""

import json
from pathlib import Path
import socket
import struct
import subprocess
import sys
import tempfile
import threading


def probe(binary, work, scheme):
    request_lines = []
    server_failures = []
    stopped = threading.Event()

    with socket.socket() as listener:
        listener.bind(("127.0.0.1", 0))
        listener.listen(3)
        listener.settimeout(0.2)
        port = listener.getsockname()[1]

        def reset_connections():
            try:
                while not stopped.is_set() and len(request_lines) < 3:
                    try:
                        connection, _ = listener.accept()
                    except TimeoutError:
                        continue
                    with connection:
                        line = b""
                        if scheme == "http":
                            connection.settimeout(1)
                            with connection.makefile("rb") as request:
                                line = request.readline(8192).strip()
                        request_lines.append(line.decode("ascii"))
                        connection.setsockopt(
                            socket.SOL_SOCKET, socket.SO_LINGER, struct.pack("ii", 1, 0)
                        )
            except (OSError, UnicodeError) as error:
                server_failures.append(error)

        source = Path(work) / f"{scheme}.md"
        source.write_text(
            f"[finite probe]({scheme}://127.0.0.1:{port}/finite-probe)\n",
            encoding="utf-8",
        )
        server = threading.Thread(target=reset_connections)
        server.start()
        try:
            result = subprocess.run(
                [
                    binary, "--no-progress", "--verbose", "--max-retries", "2",
                    "--retry-wait-time", "1", "--timeout", "2",
                    source.name,
                ],
                cwd=work, text=True, capture_output=True, timeout=10, check=False,
            )
        finally:
            stopped.set()
            server.join()

    if server_failures:
        raise RuntimeError(f"{scheme} fixture failed") from server_failures[0]
    print(result.stdout, end="")
    print(result.stderr, end="")
    print(json.dumps({
        "scheme": scheme, "configured_retries": 2, "request_lines": request_lines,
        "exit_code": result.returncode,
    }), flush=True)
    assert result.returncode == 2 and request_lines
    if scheme == "http":
        assert request_lines == ["GET /finite-probe HTTP/1.1"] * 3
    else:
        assert request_lines == [""]


def main():
    binary = str(Path(sys.argv[1]).resolve(strict=True))
    with tempfile.TemporaryDirectory() as work:
        subprocess.run([binary, "--version"], cwd=work, timeout=5, check=True)
        for scheme in ("https", "http"):
            probe(binary, work, scheme)


if __name__ == "__main__":
    main()
