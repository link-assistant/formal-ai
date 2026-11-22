#!/usr/bin/env python3
"""Bounded OS-pipe reproduction; this does not execute the Rust release CLI."""
import hashlib
import json
import os
from pathlib import Path
import resource
import signal
import subprocess
import sys
import threading
import time

PAYLOAD_BYTES = 120_000
DIAGNOSTIC_BYTES = 131_072
MEMORY_BYTES = 96 * 1024 * 1024
DEADLINE_SECONDS = 2


def worker(mode, channel):
    resource.setrlimit(resource.RLIMIT_AS, (MEMORY_BYTES, MEMORY_BYTES))
    code = ('import os,sys; '
            f'os.write({1 if channel == "stdout" else 2}, b"x" * {DIAGNOSTIC_BYTES}); '
            f'assert len(sys.stdin.buffer.read()) == {PAYLOAD_BYTES}')
    child = subprocess.Popen([sys.executable, '-c', code], stdin=subprocess.PIPE,
                             stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    stream = child.stdin
    child.stdin = None

    def write():
        stream.write(b'p' * PAYLOAD_BYTES)
        stream.close()

    if mode == 'sequential':
        write()
        output, errors = child.communicate()
    else:
        writer = threading.Thread(target=write)
        writer.start()
        output, errors = child.communicate()
        writer.join()
    assert child.returncode == 0
    assert len(output if channel == 'stdout' else errors) == DIAGNOSTIC_BYTES
    print(json.dumps({'completed': True, 'stdoutBytes': len(output), 'stderrBytes': len(errors)}))


def main():
    if len(sys.argv) == 4 and sys.argv[1] == '--worker':
        worker(sys.argv[2], sys.argv[3])
        return
    source = Path('scripts/create-github-release.rs').read_bytes()
    rows = []
    for mode in ['sequential', 'concurrent']:
        for channel in ['stdout', 'stderr']:
            start = time.monotonic()
            process = subprocess.Popen([sys.executable, __file__, '--worker', mode, channel],
                                       stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                                       start_new_session=True)
            expired = False
            try:
                stdout, stderr = process.communicate(timeout=DEADLINE_SECONDS)
            except subprocess.TimeoutExpired:
                expired = True
                os.killpg(process.pid, signal.SIGKILL)
                stdout, stderr = process.communicate()
            assert expired == (mode == 'sequential'), (mode, channel, process.returncode, stderr)
            row = {'mode': mode, 'channel': channel, 'deadlineReached': expired,
                   'exitCode': process.returncode, 'seconds': round(time.monotonic()-start, 3)}
            if not expired:
                assert process.returncode == 0, stderr
                row['result'] = json.loads(stdout)
            rows.append(row)
    report = {'productionSourceSha256': hashlib.sha256(source).hexdigest(),
              'payloadBytes': PAYLOAD_BYTES, 'diagnosticBytes': DIAGNOSTIC_BYTES,
              'memoryLimitBytesPerProcess': MEMORY_BYTES, 'deadlineSecondsPerCase': DEADLINE_SECONDS,
              'rustExecuted': False, 'autonomousAuthorshipCredit': 0, 'controls': rows}
    print(json.dumps(report, indent=2))


if __name__ == '__main__':
    main()
