#!/usr/bin/env python3
"""Finite Linux-only process/pipe lifetime observation; reap all fixture children."""
import argparse
import ctypes
import json
import os
from pathlib import Path
import resource
import signal
import subprocess
import tempfile
import time


def observe(wrapper, case="expired-parent-success"):
    # Become the reaper before launching the wrapper; never leave fixture zombies to PID 1.
    libc = ctypes.CDLL(None, use_errno=True)
    if libc.prctl(36, 1, 0, 0, 0):
        raise OSError(ctypes.get_errno(), 'PR_SET_CHILD_SUBREAPER failed')
    resource.setrlimit(resource.RLIMIT_AS, (192 * 1024 * 1024,) * 2)
    resource.setrlimit(resource.RLIMIT_STACK, (8 * 1024 * 1024,) * 2)
    with tempfile.TemporaryDirectory(prefix='issue-1580-budget-lifetime-') as temporary:
        pid_file = Path(temporary) / 'worker.pid'
        environment = dict(os.environ, BUDGET_FIXTURE_PID=str(pid_file),
                           TEST_BUDGET_POLL_SECONDS='0.05', TEST_BUDGET_GRACE_SECONDS='0',
                           BUDGET_POLL_SECONDS='0.05', BUDGET_GRACE_SECONDS='0',
                           RUSTC_WRAPPER='', BASH_ENV='/dev/null', ENV='/dev/null')
        cases = {
            'expired-parent-success': ('sleep 4', 0),
            'expired-parent-failure': ('sleep 4', 7),
            'term-resistant': ("bash -c 'trap \"\" TERM; exec sleep 4'", 0),
            'delayed-kill': ("bash -c 'trap \"\" TERM; exec sleep 4'", 0),
            'short-success': ('sleep 0.2', 0),
            'short-failure': ('sleep 0.2', 7),
            'warning-only': ('sleep 4', 0),
            'inspection-unavailable': ('sleep 4', 0),
            'inspection-unavailable-delayed-kill': ("bash -c 'trap \"\" TERM; exec sleep 4'", 0),
        }
        if case in ('inspection-unavailable', 'inspection-unavailable-delayed-kill'):
            bin_directory = Path(temporary) / 'bin'
            bin_directory.mkdir()
            unavailable = bin_directory / 'ps'
            unavailable.write_text('#!/bin/sh\nexit 9\n')
            unavailable.chmod(0o700)
            environment['PATH'] = str(bin_directory) + os.pathsep + environment['PATH']
        if case in ('delayed-kill', 'inspection-unavailable-delayed-kill'):
            # Model delayed kernel delivery without CPU pressure or unbounded workers.
            # The signaler and worker both have finite lifetimes and are reaped below.
            delayed_kill = Path(temporary) / 'delayed-kill.bash'
            delayed_kill.write_text('kill() {\n'
                                    '  if [ "$1" = "-KILL" ]; then\n'
                                    '    (sleep 0.3; builtin kill "$@" 2>/dev/null) &\n'
                                    '    return 0\n'
                                    '  fi\n'
                                    '  builtin kill "$@"\n'
                                    '}\n')
            environment['BASH_ENV'] = str(delayed_kill)
        child, status = cases[case]
        environment['TEST_BUDGET_ENFORCE'] = 'false' if case == 'warning-only' else 'true'
        command = ['bash', str(Path(wrapper).resolve()), '1', 'finite child fixture',
                   'bash', '-c', child + ' & printf "%s\\n" "$!" > "$BUDGET_FIXTURE_PID"; '
                   + 'printf "fixture stdout\\n"; printf "fixture stderr\\n" >&2; exit ' + str(status)]
        started = time.monotonic()
        process = subprocess.Popen(command, env=environment, stdout=subprocess.PIPE,
                                   stderr=subprocess.PIPE, start_new_session=True)
        worker = None
        try:
            # Both launched commands have finite inputs. This is also a 10s observation ceiling.
            for _ in range(1000):
                if process.poll() is not None:
                    break
                time.sleep(0.01)
            else:
                raise RuntimeError('Finite fixture exceeded its observation ceiling')
            wrapper_seconds = time.monotonic() - started
            worker = int(pid_file.read_text().strip())
            stat_file = Path('/proc') / str(worker) / 'stat'
            worker_state = stat_file.read_text().split(') ', 1)[1].split()[0] if stat_file.exists() else None
            stdout, stderr = process.communicate()
            result = {'case': case, 'wrapper': str(Path(wrapper).resolve()), 'argv': command,
                      'exitCode': process.returncode, 'wrapperExitSeconds': wrapper_seconds,
                      'pipeClosedSeconds': time.monotonic() - started,
                      'workerRunningWhenWrapperExited': worker_state not in (None, 'Z'),
                      'workerStateWhenWrapperExited': worker_state,
                      'stdout': stdout.decode(), 'stderr': stderr.decode(),
                      'fixtureWorkerMaximumSeconds': 4, 'budgetSeconds': 1}
        finally:
            if worker is None and pid_file.exists():
                worker = int(pid_file.read_text().strip())
            if worker:
                try:
                    os.kill(worker, signal.SIGKILL)
                except ProcessLookupError:
                    pass
            if process.poll() is None:
                os.killpg(process.pid, signal.SIGKILL)
            process.wait()
            # The only adopted descendants are this fixture's finite sleep workers.
            reaped = []
            while True:
                try:
                    child, status = os.waitpid(-1, 0)
                    reaped.append({'pid': child, 'waitStatus': status})
                except ChildProcessError:
                    break
        result['fixtureDescendantsReaped'] = reaped
        return result


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('wrapper')
    parser.add_argument('output', type=Path)
    parser.add_argument('--case', default='expired-parent-success')
    options = parser.parse_args()
    observation = observe(options.wrapper, options.case)
    options.output.write_text(json.dumps(observation, indent=2) + '\n')
    print(json.dumps(observation))
