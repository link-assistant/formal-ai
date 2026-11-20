#!/usr/bin/env python3
"""Bounded wall-second alignment fixture for the production budget wrapper."""
import argparse
import json
import os
from pathlib import Path
import resource
import signal
import subprocess
import time


def observe(wrapper, case):
    resource.setrlimit(resource.RLIMIT_AS, (192 * 1024 * 1024,) * 2)
    resource.setrlimit(resource.RLIMIT_STACK, (8 * 1024 * 1024,) * 2)
    # Sleep to the late part of the next second; never busy-spin to align.
    target = int(time.time()) + 1 + 0.82
    time.sleep(max(0, target - time.time()))
    environment = dict(os.environ, TEST_BUDGET_POLL_SECONDS='0.05',
                       TEST_BUDGET_GRACE_SECONDS='0', RUSTC_WRAPPER='',
                       BASH_ENV='/dev/null', ENV='/dev/null')
    command = ['bash', str(Path(wrapper).resolve()), '2', 'aligned finite fixture',
               'python3', '-c', 'import time; time.sleep(1.4); print("completed inside budget")']
    if case == 'term-grace':
        environment['TEST_BUDGET_POLL_SECONDS'] = '0.8'
        environment['TEST_BUDGET_GRACE_SECONDS'] = '1'
        command[2] = '1'
        command[-1] = ('import signal,time; '
                       'signal.signal(signal.SIGTERM, lambda *_: '
                       '(time.sleep(0.9), print("cooperative cleanup completed", flush=True), exit(0))); '
                       'time.sleep(5)')
    wall_fraction = time.time() % 1
    started = time.monotonic()
    process = subprocess.Popen(command, env=environment, stdout=subprocess.PIPE,
                               stderr=subprocess.PIPE, start_new_session=True)
    try:
        stdout, stderr = process.communicate(timeout=8)
    finally:
        if process.poll() is None:
            os.killpg(process.pid, signal.SIGKILL)
            process.communicate()
    return {'case': case, 'argv': command, 'exitCode': process.returncode,
            'elapsedSeconds': time.monotonic() - started,
            'startedWallFraction': wall_fraction,
            'stdout': stdout.decode(), 'stderr': stderr.decode(),
            'maximumChildSeconds': 5, 'observationCeilingSeconds': 8}


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('wrapper')
    parser.add_argument('case', choices=['within-budget', 'term-grace'])
    parser.add_argument('output', type=Path)
    args = parser.parse_args()
    observed = observe(args.wrapper, args.case)
    args.output.write_text(json.dumps(observed, indent=2) + '\n')
    print(json.dumps(observed))
