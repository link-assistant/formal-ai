#!/usr/bin/env python3
"""Independently compile and execute the delivered source on a hosted runner."""
import base64
import json
import os
import pathlib
import shutil
import subprocess
import sys
import signal
import time

def independent_environment(workspace, inherited, *, metadata_read=False):
    """Keep program execution private; authorize metadata reads only for the owned installer."""
    allowed = ('PATH', 'JAVA_HOME', 'LANG', 'LC_ALL', 'TMPDIR')
    if metadata_read:
        allowed += ('GH_TOKEN', 'GITHUB_TOKEN')
    environment = {key: inherited[key] for key in allowed if key in inherited}
    environment['HOME'] = str(workspace / 'home')
    return environment


def require_process_success(result, phase):
    """Retain both bounded output streams and termination identity on failure."""
    if result['exitCode'] != 0:
        diagnostic = {key: result[key] for key in ('exitCode', 'signal')}
        diagnostic['phase'] = phase
        for stream in ('stdout', 'stderr'):
            diagnostic[stream + 'Tail'] = base64.b64decode(
                result[stream + 'Base64']).decode('utf-8', 'replace')[-4096:]
        raise AssertionError(json.dumps(diagnostic, ensure_ascii=False))


def main():
    if os.environ.get('GITHUB_ACTIONS') != 'true':
        raise SystemExit('hosted-only: no local native/compiler execution')
    workspace = pathlib.Path(sys.argv[1]).resolve()
    source = pathlib.Path(sys.argv[2]).resolve()
    language = sys.argv[3]
    repository = pathlib.Path(__file__).resolve().parents[1]
    assert source.is_relative_to(workspace) and source.is_file()
    environment = independent_environment(workspace, os.environ)
    pathlib.Path(environment['HOME']).mkdir()
    log_directory = pathlib.Path(os.environ['E2E_LOG_DIR']).resolve()
    log_directory.mkdir(parents=True, exist_ok=True)
    process_index = 0

    def run_bounded(command, timeout, process_environment=None):
        nonlocal process_index
        process_index += 1
        prefix = log_directory / ('independent-process-' + str(process_index))
        with prefix.with_suffix('.stdout').open('wb') as output, prefix.with_suffix('.stderr').open('wb') as errors:
            child = subprocess.Popen(command, cwd=workspace, env=environment if process_environment is None else process_environment,
                                     stdout=output, stderr=errors, start_new_session=True)
            deadline = time.monotonic() + timeout
            reason = None
            while child.poll() is None:
                if time.monotonic() > deadline:
                    reason = 'independent process timeout'
                if max(os.fstat(output.fileno()).st_size, os.fstat(errors.fileno()).st_size) > 1048576:
                    reason = 'independent process output limit'
                if reason:
                    os.killpg(child.pid, signal.SIGKILL)
                    child.wait()
                    raise AssertionError(reason)
                time.sleep(0.05)
            code = child.wait()
        assert max(prefix.with_suffix('.stdout').stat().st_size,
                   prefix.with_suffix('.stderr').stat().st_size) <= 1048576, 'independent process output limit before allocation'
        stdout = prefix.with_suffix('.stdout').read_bytes()
        stderr = prefix.with_suffix('.stderr').read_bytes()
        assert max(len(stdout), len(stderr)) <= 1048576, 'independent process output limit'
        return {'exitCode': code, 'signal': signal.Signals(-code).name if code < 0 else None,
                'stdoutBase64': base64.b64encode(stdout).decode(),
                'stderrBase64': base64.b64encode(stderr).decode()}

    compile_commands = []
    if language == 'Kotlin':
        grant = json.loads((repository / 'data/meta/workspace-prerequisite-grant.json').read_text())
        request = {'program': 'kotlinc', 'publisher': {
            'program': 'kotlinc', 'source_id': 'kotlin_official', 'host': 'kotlinlang.org',
            'documentation': 'https://kotlinlang.org/docs/command-line.html'},
            'dependencies': ['java'], 'grant': grant}
        compile_commands.append([shutil.which('python3'),
            str(repository / 'data/meta/prerequisite-installer.py'), json.dumps(request)])
        for command in compile_commands:
            completed = run_bounded(command, 480,
                independent_environment(workspace, os.environ, metadata_read=True))
            require_process_success(completed, 'compiler discovery')
        prefix = (workspace / '.formal-ai/toolchains/active-kotlinc.bin').read_text().strip()
        command = [prefix + '/kotlinc', str(source), '-include-runtime', '-d', str(workspace / 'program.jar')]
        executable = ['java', '-jar', str(workspace / 'program.jar')]
    elif language == 'Scala':
        command = ['scala-cli', 'compile', str(source), '--server=false']
        executable = ['scala-cli', 'run', str(source), '--server=false', '--quiet']
    elif language == 'Rust':
        command = ['rustc', str(source), '-o', str(workspace / 'program')]
        executable = [str(workspace / 'program')]
    else:
        raise AssertionError('unsupported original language')
    compiled = run_bounded(command, 240)
    require_process_success(compiled, 'compilation')
    observed = run_bounded(executable, 180)
    print(json.dumps({**observed, 'independentCompilation': True, 'language': language}))


if __name__ == '__main__':
    main()
