"""Real release CLI controls; every gh effect is a bounded local subprocess."""
import gzip
import hashlib
import json
import os
from pathlib import Path
import signal
import subprocess
import sys
import unittest


def add_release_process_controls(harness, scripts):
    fixtures = harness / 'fixtures'
    metadata = json.loads((fixtures / 'create-github-release-before-fix.json').read_text())
    source = gzip.decompress((fixtures / 'create-github-release-before-fix.rs.gz').read_bytes())
    if hashlib.sha256(source).hexdigest() != metadata['sha256']:
        raise RuntimeError('the original release CLI negative control changed')
    (scripts / 'create-github-release-before-fix.rs').write_bytes(source)
    return ('\n[[bin]]\nname = "create-github-release-before-fix"\n'
            'path = "../scripts/create-github-release-before-fix.rs"\ntest = false\n')


class ReleaseProcessRegressions:
    def release_stub(self, scenario, channel='stderr'):
        folder = self.work.parent / 'release-stub'
        folder.mkdir(exist_ok=True)
        script = folder / 'gh'
        script.write_text('''#!/usr/bin/env python3
import json,os,sys
from pathlib import Path
assert sys.argv[1:] == ['api','repos/fixture/repo/releases','-X','POST','--input','-']
scenario=os.environ['RELEASE_STUB_SCENARIO']
stream=sys.stdout.buffer if os.environ['RELEASE_STUB_CHANNEL']=='stdout' else sys.stderr.buffer
duplicate={'resource':'Release','code':'already_exists','field':'tag_name'}
invalid={'resource':'Release','code':'custom','field':'body'}
if scenario=='closed-input':
    os.close(0)
    stream.write(json.dumps({'errors':[duplicate]}).encode());stream.flush()
    raise SystemExit(1)
if scenario=='diagnostic':
    stream.write(b'x'*131072);stream.flush()
payload=sys.stdin.buffer.read()
json.loads(payload)
Path(os.environ['RELEASE_STUB_PAYLOAD']).write_bytes(payload)
if scenario=='duplicate':
    stream.write(json.dumps({'errors':[duplicate]}).encode())
elif scenario in ['mixed','mixed-prefix']:
    if scenario=='mixed-prefix':stream.write(b'Release already exists\\n')
    stream.write(json.dumps({'errors':[duplicate,invalid]}).encode())
elif scenario=='malformed':
    stream.write(b'{"errors":[{"resource":"Release","code":"already_exists","field":"tag_name"}],')
elif scenario=='negated':
    stream.write(b'not a release already exists\\n')
elif scenario=='legacy':
    stream.write(b'Release already exists\\n')
elif scenario in ['success','diagnostic']:
    raise SystemExit(0)
else:raise AssertionError(scenario)
stream.flush()
raise SystemExit(1)
''')
        script.chmod(0o755)
        self.release_payload = self.work.parent / 'release-payload.json'
        self.release_payload.unlink(missing_ok=True)
        self.env.update(PATH=str(folder) + os.pathsep + os.environ['PATH'],
                        RELEASE_STUB_SCENARIO=scenario, RELEASE_STUB_CHANNEL=channel,
                        RELEASE_STUB_PAYLOAD=str(self.release_payload))
        for key in ['GH_TOKEN', 'GITHUB_TOKEN', 'RELEASE_VERSION', 'REPOSITORY', 'TAG_PREFIX',
                    'LANGUAGE', 'RELEASE_LABEL', 'CRATES_IO_URL', 'GHCR_URL', 'DOCKER_HUB_URL',
                    'SELF_HOSTING_LEDGER']:
            self.env.pop(key, None)

    def run_release(self, before=False, deadline=8, *extra):
        binary = self.target / 'debug' / ('create-github-release-before-fix' if before else 'create-github-release')
        # Apply the address-space boundary before exec, without preexec_fn in
        # this threaded HTTP fixture. The CLI and its gh child inherit it.
        launcher = ('import os,resource,sys; '
                    'resource.setrlimit(resource.RLIMIT_AS,(201326592,201326592)); '
                    'os.execv(sys.argv[1],sys.argv[1:])')
        args = [sys.executable, '-c', launcher, str(binary), '--release-version', '1.2.3',
                '--repository', 'fixture/repo', *extra]
        with subprocess.Popen(args, cwd=self.work, env=self.env, stdout=subprocess.PIPE,
                              stderr=subprocess.PIPE, text=True, start_new_session=True) as process:
            expired = False
            try:
                stdout, stderr = process.communicate(timeout=deadline)
            except subprocess.TimeoutExpired:
                expired = True
                os.killpg(process.pid, signal.SIGKILL)
                stdout, stderr = process.communicate()
        return subprocess.CompletedProcess(args, process.returncode, stdout, stderr), expired

    def long_release_notes(self):
        (self.work / 'CHANGELOG.md').write_text('## [1.2.3]\n\n' + 'λ雪🙂' * 20_000 + '\n')

    @unittest.skipUnless(os.name == 'posix', 'bounded process-group controls require POSIX')
    def test_release_cli_drains_both_outputs_while_writing_request(self):
        self.long_release_notes()
        for channel in ['stdout', 'stderr']:
            with self.subTest(channel=channel):
                self.release_stub('diagnostic', channel)
                before, expired = self.run_release(before=True, deadline=3)
                self.assertTrue(expired, (before.returncode, before.stderr[:1000]))
                self.assertFalse(self.release_payload.exists())
                after, expired = self.run_release()
                self.assertFalse(expired)
                self.assertEqual(after.returncode, 0, after.stderr[:1000])
                self.assertIn('Created GitHub release: v1.2.3', after.stdout)
                self.assertEqual(json.loads(self.release_payload.read_bytes())['tag_name'], 'v1.2.3')
                print(json.dumps({'releasePipeControl': channel, 'originalDeadlineReached': True,
                                  'repairedExitCode': after.returncode, 'diagnosticBytes': 131072}))

    @unittest.skipUnless(os.name == 'posix', 'bounded process-group controls require POSIX')
    def test_release_cli_preserves_unicode_byte_cap_and_metric_suffix(self):
        self.long_release_notes()
        ledger = self.work / 'fixture-ledger.lino'
        ledger.write_text('''self_hosting_ledger
  release
    tag "v1.2.3"
    since "v1.2.2"
    until "fixture"
    metric_version "3"
    changed_lines "4"
    self_authored_lines "0"
    commits "1"
    self_authored_commits "0"
    percentage_basis_points "0"
    trailing_window "3"
    trailing_percentage_basis_points "0"
''')
        note = ('## Self-hosting\n\nFormal AI authored **0.00%** of this release '
                '(0 of 4 changed lines). The 3-release trailing share is **0.00%**.')
        for before in [True, False]:
            with self.subTest(before=before):
                self.release_stub('success')
                result, expired = self.run_release(before, 8, '--self-hosting-ledger', str(ledger))
                self.assertFalse(expired)
                self.assertEqual(result.returncode, 0, result.stderr[:1000])
                body = json.loads(self.release_payload.read_bytes())['body']
                self.assertLessEqual(len(body.encode('utf-8')), 120000)
                self.assertLess(120000-len(body.encode('utf-8')), 4)
                self.assertIn('λ雪🙂' * 100, body)
                self.assertNotIn('\ufffd', body)
                self.assertIn('https://github.com/fixture/repo/blob/v1.2.3/CHANGELOG.md', body)
                self.assertTrue(body.endswith(note))

    @unittest.skipUnless(os.name == 'posix', 'bounded process-group controls require POSIX')
    def test_release_cli_requires_complete_duplicate_error_evidence(self):
        for scenario in ['mixed', 'mixed-prefix', 'malformed', 'negated', 'duplicate', 'legacy']:
            with self.subTest(scenario=scenario):
                self.release_stub(scenario)
                if scenario not in ['duplicate', 'legacy']:
                    before, expired = self.run_release(before=True)
                    self.assertFalse(expired)
                    self.assertEqual(before.returncode, 0, before.stderr[:1000])
                result, expired = self.run_release()
                self.assertFalse(expired)
                if scenario in ['duplicate', 'legacy']:
                    self.assertEqual(result.returncode, 0, result.stderr[:1000])
                    self.assertIn('already exists, skipping', result.stdout)
                else:
                    self.assertNotEqual(result.returncode, 0)
                    self.assertIn('Error creating release', result.stderr)
                    self.assertNotIn('already exists, skipping', result.stdout)

    @unittest.skipUnless(os.name == 'posix', 'bounded process-group controls require POSIX')
    def test_release_cli_keeps_failed_request_write_a_failure(self):
        self.long_release_notes()
        self.release_stub('closed-input')
        result, expired = self.run_release()
        self.assertFalse(expired)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn('Failed to write to stdin', result.stderr)
        self.assertNotIn('already exists, skipping', result.stdout)

    @unittest.skipUnless(os.name == 'posix', 'bounded process-group controls require POSIX')
    def test_release_cli_keeps_short_notes_and_release_identity(self):
        self.release_stub('success')
        result, expired = self.run_release()
        self.assertFalse(expired)
        self.assertEqual(result.returncode, 0, result.stderr[:1000])
        payload = json.loads(self.release_payload.read_bytes())
        self.assertEqual(payload['tag_name'], 'v1.2.3')
        self.assertEqual(payload['name'], '[Rust] 1.2.3')
        self.assertIn('Original notes.', payload['body'])
        self.assertNotIn('Release notes were shortened', payload['body'])
