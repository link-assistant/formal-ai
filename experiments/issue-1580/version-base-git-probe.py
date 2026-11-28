#!/usr/bin/env python3
"""Finite Git reproduction; this does not compile or execute the Rust guard."""
from pathlib import Path
import json
import os
import subprocess
import tempfile


def run_probe():
    with tempfile.TemporaryDirectory(prefix='formal-version-fetch-') as directory:
        root = Path(directory)
        origin, work, other = root / 'origin.git', root / 'work', root / 'other'
        env = {**os.environ, 'GIT_CONFIG_NOSYSTEM': '1',
               'GIT_CONFIG_GLOBAL': os.devnull, 'GIT_TERMINAL_PROMPT': '0'}
        def git(cwd, *args):
            return subprocess.run(['git', *args], cwd=cwd, env=env, capture_output=True,
                                  text=True, check=True).stdout.strip()
        git(root, 'init', '--bare', '-q', str(origin))
        work.mkdir()
        git(work, 'init', '-q', '-b', 'main')
        git(work, 'config', 'user.name', 'Fixture')
        git(work, 'config', 'user.email', 'fixture@example.invalid')
        manifest = '[package]\nname = "fixture"\nversion = "1.2.3"\n'
        (work / 'Cargo.toml').write_text(manifest)
        git(work, 'add', 'Cargo.toml')
        git(work, 'commit', '-qm', 'initial base')
        git(work, 'remote', 'add', 'origin', str(origin))
        git(work, 'push', '-q', '-u', 'origin', 'main')
        stale = git(work, 'rev-parse', 'origin/main')
        git(root, 'clone', '-q', '--branch', 'main', str(origin), str(other))
        git(other, 'config', 'user.name', 'Fixture')
        git(other, 'config', 'user.email', 'fixture@example.invalid')
        (other / 'Cargo.toml').write_text(manifest.replace('1.2.3', '1.2.4'))
        git(other, 'add', 'Cargo.toml')
        git(other, 'commit', '-qm', 'new remote base')
        git(other, 'push', '-q', 'origin', 'main')
        current = git(other, 'rev-parse', 'HEAD')
        assert current != stale
        git(work, 'config', 'remote.origin.fetch', '+refs/heads/*:refs/remotes/alternate/*')
        # The original production command succeeds but leaves origin/main stale.
        git(work, 'fetch', 'origin', 'main', '--depth=1')
        assert git(work, 'rev-parse', 'origin/main') == stale
        assert git(work, 'rev-parse', 'FETCH_HEAD') == current
        assert '1.2.3' in git(work, 'show', 'origin/main:Cargo.toml')
        sentinel = 'another fetch owns this observation\n'
        fetched_head = work / '.git/FETCH_HEAD'
        fetched_head.write_text(sentinel)
        destination = 'refs/formal-ai/version-base/probe'
        git(work, 'check-ref-format', 'refs/heads/main')
        git(work, 'fetch', '--atomic', '--no-tags', '--no-write-fetch-head', '--refmap=',
            '--depth=1', '--', 'origin', 'refs/heads/main:' + destination)
        commit = git(work, 'rev-parse', '--verify', destination + '^{commit}')
        git(work, 'update-ref', '-d', destination)
        assert commit == current
        assert fetched_head.read_text() == sentinel
        # Replacing both shared observations after fetch cannot change commit identity.
        git(work, 'update-ref', 'refs/remotes/origin/main', stale)
        fetched_head.write_text('concurrent observation\n')
        assert '1.2.4' in git(work, 'show', commit + ':Cargo.toml')
        assert git(work, 'for-each-ref', 'refs/formal-ai/version-base') == ''
        assert git(work, 'status', '--porcelain') == ''
        return {'oldFetchSucceeded': True, 'oldTrackingRefStayedStale': True,
                'isolatedCommitMatchesRemote': True, 'sharedObservationChangesIgnored': True,
                'temporaryRefRemoved': True, 'workTreePreserved': True,
                'nativeExecution': False, 'independentAuthorshipCredit': 0}


if __name__ == '__main__':
    print(json.dumps(run_probe(), indent=2))
