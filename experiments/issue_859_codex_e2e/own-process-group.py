#!/usr/bin/env python3
"""Wait for a client, then stop its remaining helpers before releasing its home."""
import json
import os
import signal
import subprocess
import sys
import time


def signal_group(group, value):
    try:
        os.killpg(group, value)
    except ProcessLookupError:
        pass


def live_group(group):
    result = subprocess.run(["ps", "-eo", "pgid=,stat="], check=True,
                            stdout=subprocess.PIPE, encoding="utf-8", timeout=2)
    return any(len(fields) == 2 and fields[0] == str(group)
               and not fields[1].startswith(("Z", "X"))
               for fields in (line.split() for line in result.stdout.splitlines()))


def stop_group(group):
    # The client can exit while a plugin clone still writes in CODEX_HOME.
    # A group leader's exit alone therefore cannot certify safe directory removal.
    signal_group(group, signal.SIGTERM)
    try:
        for _ in range(10):
            if not live_group(group):
                return
            time.sleep(0.05)
        signal_group(group, signal.SIGKILL)
        for _ in range(40):
            if not live_group(group):
                return
            time.sleep(0.05)
        raise RuntimeError("client process group still has live descendants")
    except BaseException:
        signal_group(group, signal.SIGKILL)
        raise


def main(arguments):
    if not arguments:
        raise ValueError("a client command is required")
    process = subprocess.Popen(arguments, start_new_session=True)
    def cancel(value, _frame):
        signal_group(process.pid, value)
        raise SystemExit(128 + value)
    signal.signal(signal.SIGTERM, cancel)
    signal.signal(signal.SIGINT, cancel)
    try:
        status = process.wait()
    finally:
        stop_group(process.pid)
        process.wait()
    if os.environ.get("FORMAL_AI_TRACE_CODEX_LIFETIME") == "1":
        print(json.dumps({"client_group": process.pid, "cleanup_complete": True,
                          "status": status}), file=sys.stderr)
    return status if status >= 0 else 128 - status


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
