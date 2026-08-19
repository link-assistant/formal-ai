"""Standalone reproduction for the Codex first-run trust dialog regression.

Issue #1021 hit this as a red `E2E Tests (agent CLI <-> formal-ai)` job on a
branch that changed nothing in the Codex startup path. The probe removes
Formal AI from the picture entirely: it runs a bare `codex` in a pseudo
terminal under a throwaway HOME, answers the first-run
"Press enter to continue" dialog with one of several strategies, and reports
whether the dialog is still on screen at the end.

Usage:
    python3 codex_trust_dialog_probe.py <version> [strategy]

`version` is an @openai/codex version; the probe installs it into
/tmp/formal-ai-codex-versions/<version> when it is not already there.
`strategy` defaults to `enter-now`, which is what a terminal-driving harness
does: press ENTER as soon as the marker renders.
"""

import fcntl
import os
import pty
import re
import select
import struct
import subprocess
import sys
import tempfile
import termios
import time

# Every strategy is a list of (seconds-after-the-marker-first-rendered, bytes).
STRATEGIES = {
    "enter-now": [(0.0, b"\r")],
    "enter-after-3s": [(3.0, b"\r")],
    "enter-twice": [(0.0, b"\r"), (3.0, b"\r")],
    "digit-then-enter": [(3.0, b"1"), (3.5, b"\r")],
    "down-up-enter": [(3.0, b"\x1b[B"), (3.2, b"\x1b[A"), (3.4, b"\r")],
}

MARKER = b"Press enter to continue"
WINDOW_SECONDS = 20


def install(version):
    prefix = f"/tmp/formal-ai-codex-versions/{version}"
    binary = f"{prefix}/node_modules/.bin/codex"
    if not os.path.exists(binary):
        os.makedirs(prefix, exist_ok=True)
        subprocess.run(
            ["npm", "install", "--prefix", prefix, f"@openai/codex@{version}"],
            check=True,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
        )
    return binary


def run(binary, plan):
    home = tempfile.mkdtemp(prefix="codex-probe-home-")
    work = tempfile.mkdtemp(prefix="codex-probe-work-")
    pid, fd = pty.fork()
    if pid == 0:
        os.chdir(work)
        os.environ["TERM"] = "xterm-256color"
        os.environ["HOME"] = home
        os.execv("/bin/sh", ["/bin/sh", "-c", binary])
    # pty.fork() leaves the slave at 0x0, and Codex renders nothing into a
    # zero-sized terminal, so the window size has to be set explicitly.
    fcntl.ioctl(fd, termios.TIOCSWINSZ, struct.pack("HHHH", 30, 80, 0, 0))
    pending = list(plan)
    buffer, marker_at, started = b"", None, time.time()
    while time.time() - started < WINDOW_SECONDS:
        readable, _, _ = select.select([fd], [], [], 0.1)
        if readable:
            try:
                chunk = os.read(fd, 65536)
            except OSError:
                break
            if not chunk:
                break
            buffer += chunk
        if marker_at is None and MARKER in buffer:
            marker_at = time.time()
        if marker_at is not None and pending:
            if time.time() - marker_at >= pending[0][0]:
                os.write(fd, pending.pop(0)[1])
    os.kill(pid, 15)
    return buffer, marker_at is not None


def main():
    version = sys.argv[1]
    strategy = sys.argv[2] if len(sys.argv) > 2 else "enter-now"
    buffer, saw_marker = run(install(version), STRATEGIES[strategy])
    text = buffer.decode("utf8", "replace")
    plain = re.sub(r"\x1b\[[0-9;?>< ]*[a-zA-Z]", "", text).replace("\x1b", "")
    stuck = MARKER.decode() in plain[-600:]
    print(
        f"[probe] codex {version} strategy={strategy:<17} "
        f"bytes={len(buffer):>7} marker_rendered={saw_marker} "
        f"still_on_dialog={stuck}"
    )
    return 1 if stuck else 0


if __name__ == "__main__":
    sys.exit(main())
