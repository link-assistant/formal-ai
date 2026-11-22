#!/usr/bin/env python3
"""Exercise release and PR version CLIs; registry and Git origins are local only."""
import http.client
import http.server
import json
import os
import re
from pathlib import Path
import shutil
import socket
import subprocess
import tempfile
import threading
import unittest
import sys

sys.dont_write_bytecode = True
from release_process_regressions import ReleaseProcessRegressions, add_release_process_controls

ROOT = Path(__file__).resolve().parents[3]
HARNESS = Path(__file__).resolve().parent


class RegistryRegressions(ReleaseProcessRegressions, unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.requests = []
        cls.mode = 'success'
        cls.scope = ''
        cls.maximum = '1.2.3'

        class Registry(http.server.BaseHTTPRequestHandler):
            def log_message(self, *_args):
                pass

            def do_GET(self):
                cls.requests.append(self.path)
                affected = cls.scope in self.path
                if affected and (cls.mode == 'drop' or (cls.mode == 'drop-once' and len(cls.requests) == 2)):
                    self.connection.shutdown(socket.SHUT_RDWR)
                    self.connection.close()
                    return
                if affected and cls.mode in ['429', '503', '403', '404']:
                    status, data = int(cls.mode), {}
                elif affected and cls.mode == 'malformed':
                    status, data = 200, {}
                else:
                    status = 200
                    if self.path.endswith('/fixture'):
                        data = {'versions': [{'num': cls.maximum, 'yanked': False}]}
                    elif '/releases/tags/' in self.path:
                        data = {'tag_name': self.path.rsplit('/', 1)[1]}
                    elif '/tags/' in self.path:
                        data = {'name': self.path.rsplit('/', 1)[1]}
                    elif self.path.endswith('/1.2.3') or (affected and cls.mode == 'wrong-identity'
                                                                  and '/crates/fixture/' in self.path):
                        data = {'version': {'num': '1.2.3'}}
                    else:
                        status, data = 404, {}
                if affected and cls.mode == 'wrong-identity' and status == 200:
                    for field in ['tag_name', 'name']:
                        if field in data:
                            data[field] = 'unrelated'
                    if 'version' in data:
                        data['version']['num'] = '0.0.0'
                body = json.dumps(data).encode()
                self.send_response(status)
                self.send_header('Content-Length', str(len(body)))
                self.end_headers()
                self.wfile.write(body)

        cls.server = http.server.ThreadingHTTPServer(('127.0.0.1', 0), Registry)
        cls.thread = threading.Thread(target=cls.server.serve_forever, daemon=True)
        cls.thread.start()
        cls.temp = tempfile.TemporaryDirectory(prefix='registry-scripts-')
        cls.scripts = Path(cls.temp.name) / 'scripts'
        shutil.copytree(ROOT / 'scripts', cls.scripts)
        release_controls = add_release_process_controls(HARNESS, cls.scripts)
        address = f'http://127.0.0.1:{cls.server.server_port}'
        for path in cls.scripts.rglob('*.rs'):
            source = path.read_text()
            for url in ['https://crates.io', 'https://api.github.com', 'https://hub.docker.com']:
                source = source.replace(url, address)
            source = source.replace('Duration::from_secs(1)', 'Duration::ZERO')
            path.write_text(source)
        cls.env = {**os.environ, 'RUST_LOG': 'error'}
        cls.env.pop('GITHUB_ACTIONS', None)
        cls.target = Path(cls.env.get('CARGO_TARGET_DIR', str(Path(cls.temp.name) / 'target'))).resolve()
        cls.env['CARGO_TARGET_DIR'] = str(cls.target)
        harness = Path(cls.temp.name) / 'harness'
        harness.mkdir()
        (harness / 'Cargo.toml').write_text((HARNESS / 'Cargo.toml').read_text()
                                           .replace('../../../scripts/', '../scripts/') + release_controls)
        shutil.copyfile(HARNESS / 'Cargo.lock', harness / 'Cargo.lock')
        result = subprocess.run(['cargo', 'build', '--manifest-path', str(harness / 'Cargo.toml'),
                                 '--locked', '--bins'], env=cls.env, capture_output=True, text=True)
        if result.returncode:
            raise RuntimeError(result.stdout + result.stderr)

    @classmethod
    def tearDownClass(cls):
        cls.server.shutdown()
        cls.server.server_close()
        cls.thread.join()
        cls.temp.cleanup()

    def setUp(self):
        self.__class__.requests = []
        self.__class__.mode = 'success'
        self.__class__.scope = ''
        self.__class__.maximum = '1.2.3'
        temp = tempfile.TemporaryDirectory(prefix='registry-cli-')
        self.addCleanup(temp.cleanup)
        root = Path(temp.name)
        self.work = root / 'work'
        self.work.mkdir()
        self.output = root / 'outputs'
        self.env = {**self.__class__.env, 'GITHUB_REF_NAME': 'main', 'GITHUB_OUTPUT': str(self.output),
                    'GITHUB_REPOSITORY': 'fixture/repo', 'HAS_FRAGMENTS': 'false'}
        for key in ['RUST_ROOT', 'GITHUB_TOKEN', 'DOCKERHUB_IMAGE', 'RELEASE_RECOVERY_PROTOCOL_ROOT',
                    'GITHUB_EVENT_NAME', 'GITHUB_EVENT_BEFORE', 'GITHUB_REF', 'GITHUB_BASE_REF',
                    'RELEASE_PIPELINE', 'BUMP_TYPE', 'DESCRIPTION', 'DEFAULT']:
            self.env.pop(key, None)
        (self.work / 'data/meta').mkdir(parents=True)
        (self.work / 'data/meta/self-hosting-ledger.lino').write_text('')
        (self.work / 'Cargo.toml').write_text('[package]\nname="fixture"\nversion="1.2.3"\n')
        (self.work / 'src').mkdir()
        (self.work / 'src/lib.rs').write_text('pub fn fixture() {}\n')
        (self.work / 'CHANGELOG.md').write_text('## [1.2.3]\n\nOriginal notes.\n')
        (self.work / 'docs').mkdir()
        for path in ['docs/status.md', 'docs/benchmarks.md', 'README.md']:
            (self.work / path).write_text('# Fixture status surface\n')
        self.git('init', '-q', '-b', 'main')
        self.git('config', 'user.name', 'Fixture')
        self.git('config', 'user.email', 'fixture@example.invalid')
        self.git('add', '.')
        self.git('commit', '-qm', 'fixture')
        self.git('tag', 'v1.2.3')
        origin = root / 'origin.git'
        subprocess.run(['git', 'init', '--bare', '-q', str(origin)], check=True)
        self.git('remote', 'add', 'origin', str(origin))
        self.git('push', '-q', '-u', 'origin', 'main')

    def git(self, *args):
        return subprocess.run(['git', *args], cwd=self.work, capture_output=True, text=True, check=True).stdout

    def script(self, name, *extra):
        args = ['--bump-type', 'patch', '--branch', 'main'] if name == 'version-and-commit.rs' else []
        return subprocess.run([str(self.target / 'debug' / name.removesuffix('.rs')), *args, *extra], cwd=self.work,
                              env=self.env, capture_output=True, text=True, timeout=40)

    def version_base(self, content):
        (self.work / 'Cargo.toml').write_text(content)
        self.git('add', 'Cargo.toml')
        self.git('commit', '-qm', 'version comparison base')
        self.git('push', '-q')
        self.env.update(GITHUB_EVENT_NAME='pull_request', GITHUB_BASE_REF='main')

    OUTPUT_CLIS = {
        'get-version.rs': 'version=1.2.3',
        'get-bump-type.rs': 'bump_type=patch',
        'detect-code-changes.rs': 'release-pending=false',
        'publish-crate.rs': 'publish_result=skipped',
        'wait-for-crate.rs': 'crate_available=skipped',
    }

    def output_cli_fixture(self):
        # The template identity takes the real publish/wait skip paths. No cargo
        # publication or external registry request can be part of this matrix.
        manifest = self.work / 'Cargo.toml'
        manifest.write_text(manifest.read_text().replace('name="fixture"',
                                                       'name="example-sum-package-name"'))
        self.git('add', 'Cargo.toml')
        self.git('commit', '-qm', 'template identity for output-only controls')
        self.env.update(GITHUB_EVENT_NAME='push', GITHUB_REF='refs/heads/main')

    def test_each_output_cli_refuses_directory_and_empty_output_paths(self):
        self.output_cli_fixture()
        for path in [str(self.work), '']:
            self.env['GITHUB_OUTPUT'] = path
            for script in self.OUTPUT_CLIS:
                with self.subTest(script=script, path=path):
                    result = self.script(script)
                    self.assertNotEqual(result.returncode, 0, result.stdout + result.stderr)
                    self.assertIn('GITHUB_OUTPUT', result.stderr)
        self.assertEqual(self.requests, [])

    @unittest.skipUnless(Path('/dev/full').exists(), 'requires the finite Linux full device')
    def test_each_output_cli_refuses_write_failure_after_open(self):
        self.output_cli_fixture()
        self.env['GITHUB_OUTPUT'] = '/dev/full'
        for script in self.OUTPUT_CLIS:
            with self.subTest(script=script):
                result = self.script(script)
                self.assertNotEqual(result.returncode, 0, result.stdout + result.stderr)
                self.assertIn('GITHUB_OUTPUT', result.stderr)
        self.assertEqual(self.requests, [])

    def test_each_output_cli_appends_and_supports_unset_local_output(self):
        self.output_cli_fixture()
        for script, expected in self.OUTPUT_CLIS.items():
            with self.subTest(script=script):
                self.env['GITHUB_OUTPUT'] = str(self.output)
                self.output.write_text('existing=value\n')
                result = self.script(script)
                self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
                output = self.output.read_text()
                self.assertTrue(output.startswith('existing=value\n'))
                self.assertIn(expected, output.splitlines())
                del self.env['GITHUB_OUTPUT']
                result = self.script(script)
                self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
                self.assertIn(expected, result.stdout)
                self.assertEqual(self.output.read_text(), output)
        self.assertEqual(self.requests, [])

    @unittest.skipUnless(os.name == 'posix', 'Unix path-byte contract')
    def test_each_output_cli_preserves_non_utf8_output_paths(self):
        self.output_cli_fixture()
        path = os.fsdecode(os.fsencode(self.output) + b'-\xff')
        self.env['GITHUB_OUTPUT'] = path
        for script, expected in self.OUTPUT_CLIS.items():
            with self.subTest(script=script):
                result = self.script(script)
                self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
                self.assertIn(expected, Path(path).read_text().splitlines())
        self.assertEqual(self.requests, [])

    def test_waiter_requires_the_requested_identity_and_a_complete_registry_answer(self):
        for mode in ['malformed', 'wrong-identity', '503', '429', '403', 'drop']:
            with self.subTest(mode=mode):
                self.__class__.mode = mode
                self.__class__.requests = []
                self.output.unlink(missing_ok=True)
                result = self.script('wait-for-crate.rs', '--max-attempts', '2', '--sleep-seconds', '0')
                self.assertNotEqual(result.returncode, 0, result.stdout + result.stderr)
                self.assertIn('availability', result.stderr.lower())
                self.assertNotIn('crate_available=true', self.output.read_text() if self.output.exists() else '')
                self.assertLessEqual(len(self.requests), 4, 'one bounded shared lookup fails closed')
                self.assertEqual(self.git('status', '--porcelain'), '')

    def test_waiter_distinguishes_expected_absence_from_confirmed_visibility(self):
        self.__class__.mode = '404'
        result = self.script('wait-for-crate.rs', '--max-attempts', '2', '--sleep-seconds', '0')
        self.assertNotEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertEqual(len(self.requests), 2)
        self.assertFalse(self.output.exists())
        self.__class__.mode = 'success'
        result = self.script('wait-for-crate.rs', '--max-attempts', '2', '--sleep-seconds', '0')
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertEqual(self.output.read_text(), 'crate_available=true\n')

    def test_release_change_detection_refuses_an_unreadable_fragment_inventory(self):
        (self.work / 'src/lib.rs').write_text('pub fn changed_fixture() {}\n')
        self.git('add', 'src/lib.rs')
        self.git('commit', '-qm', 'code change requiring release classification')
        self.env.update(GITHUB_EVENT_NAME='push', GITHUB_REF='refs/heads/main', RELEASE_PIPELINE='true')
        fragments = self.work / 'changelog.d'
        fragments.write_text('not a directory')
        result = self.script('detect-code-changes.rs')
        self.assertNotEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertIn('changelog.d', result.stderr)
        self.assertFalse(self.output.exists(), 'do not emit job decisions from an incomplete inventory')
        self.assertEqual(fragments.read_text(), 'not a directory')
        self.env['RELEASE_PIPELINE'] = 'false'
        result = self.script('detect-code-changes.rs')
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertIn('release-pending=false', self.output.read_text().splitlines())
        self.output.unlink()
        self.env['RELEASE_PIPELINE'] = 'true'
        fragments.unlink()
        result = self.script('detect-code-changes.rs')
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertIn('release-pending=false', self.output.read_text().splitlines())
        self.output.unlink()
        fragments.mkdir()
        (fragments / 'fixture.MD').write_text('A release note.\n')
        result = self.script('detect-code-changes.rs')
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertIn('release-pending=true', self.output.read_text().splitlines())
        if os.name == 'posix':
            invalid = Path(os.fsdecode(os.fsencode(fragments) + b'/invalid-\xff.md'))
            invalid.write_text('Preserve this unclassifiable name.\n')
            self.output.unlink()
            result = self.script('detect-code-changes.rs')
            self.assertNotEqual(result.returncode, 0, result.stdout + result.stderr)
            self.assertIn('Non-UTF-8', result.stderr)
            self.assertFalse(self.output.exists())
            self.assertTrue(invalid.exists())

    def test_collect_and_bump_refuse_inventory_and_fragment_read_failures(self):
        fragments = self.work / 'changelog.d'
        fragments.write_text('not a directory')
        original = (self.work / 'CHANGELOG.md').read_bytes()
        for script in ['collect-changelog.rs', 'get-bump-type.rs']:
            with self.subTest(script=script, mode='inventory'):
                result = self.script(script)
                self.assertNotEqual(result.returncode, 0, result.stdout + result.stderr)
                self.assertIn('changelog.d', result.stderr)
                self.assertEqual((self.work / 'CHANGELOG.md').read_bytes(), original)
        fragments.unlink()
        fragments.mkdir()
        good = fragments / 'good.md'
        good.write_text('---\nbump: patch\n---\n\n### Fixed\n- Keep this note.\n')
        broken = fragments / 'unreadable.md'
        broken.mkdir()
        for script in ['collect-changelog.rs', 'get-bump-type.rs']:
            with self.subTest(script=script, mode='fragment'):
                result = self.script(script)
                self.assertNotEqual(result.returncode, 0, result.stdout + result.stderr)
                self.assertIn('unreadable.md', result.stderr)
                self.assertEqual((self.work / 'CHANGELOG.md').read_bytes(), original)
                self.assertTrue(good.is_file() and broken.is_dir())
        broken.rmdir()
        (fragments / 'README.md').write_text('Retain instructions.\n')
        (fragments / 'notes.txt').write_text('Retain other files.\n')
        result = self.script('collect-changelog.rs')
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertFalse(good.exists())
        self.assertEqual((fragments / 'README.md').read_text(), 'Retain instructions.\n')
        self.assertEqual((fragments / 'notes.txt').read_text(), 'Retain other files.\n')
        self.assertIn('### Fixed\n- Keep this note.', (self.work / 'CHANGELOG.md').read_text())

    def test_fragment_content_cannot_inject_actions_commands_into_the_log(self):
        description = 'Keep λ bytes.\n::warning::forged notice\n##[error]forged legacy error'
        self.env.update(GITHUB_ACTIONS='true', DESCRIPTION=description)
        result = self.script('create-changelog-fragment.rs')
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        fragments = list((self.work / 'changelog.d').glob('*.md'))
        self.assertEqual(len(fragments), 1)
        content = fragments[0].read_bytes()
        self.assertEqual(content, ('---\nbump: patch\n---\n\n### Fixed\n\n- ' + description + '\n').encode('utf-8'))
        self.assertNotIn('::warning::', result.stdout + result.stderr)
        self.assertNotIn('##[error]', result.stdout + result.stderr)
        logged = [line for line in result.stdout.splitlines() if line.startswith('"---')]
        self.assertEqual(len(logged), 1)
        self.assertEqual(json.loads(logged[0]).encode('utf-8'), content)

    def test_invalid_fragment_bump_cannot_inject_actions_commands_into_the_log(self):
        value = 'invalid\n::warning::forged notice\n##[error]forged legacy error'
        self.env.update(GITHUB_ACTIONS='true', BUMP_TYPE=value)
        result = self.script('create-changelog-fragment.rs')
        self.assertNotEqual(result.returncode, 0)
        self.assertNotIn('::warning::', result.stdout + result.stderr)
        self.assertNotIn('##[error]', result.stdout + result.stderr)
        self.assertEqual(json.loads(result.stderr.strip()),
                         f'Invalid bump type: {value}. Must be major, minor, or patch.')
        self.assertFalse((self.work / 'changelog.d').exists())

    def test_version_check_rejects_a_package_change_after_a_dependency_table(self):
        base = ('[dependencies.decoy]\nversion = "9.9.9"\n'
                '[package]\nname = "fixture"\nversion = "1.2.3"\n')
        self.version_base(base)
        changed = base.replace('"1.2.3"', '"1.2.4"')
        (self.work / 'Cargo.toml').write_text(changed)
        result = self.script('check-version-modification.rs')
        self.assertNotEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertIn('Manual version change detected', result.stderr)
        self.assertEqual((self.work / 'Cargo.toml').read_text(), changed)

    def test_version_check_never_authorizes_release_like_branch_names(self):
        base = '[package]\nname = "fixture"\nversion = "1.2.3"\n'
        self.version_base(base)
        for branch in ['feature/check', 'changelog-manual-release-1.2.4',
                       'changeset-release/main', 'release/1.2.4', 'automated-release/1.2.4']:
            for version in ['1.2.2', '1.2.4']:
                with self.subTest(branch=branch, version=version):
                    self.env['GITHUB_HEAD_REF'] = branch
                    changed = base.replace('1.2.3', version)
                    (self.work / 'Cargo.toml').write_text(changed)
                    result = self.script('check-version-modification.rs')
                    self.assertNotEqual(result.returncode, 0, result.stdout + result.stderr)
                    self.assertIn('Manual version change detected', result.stderr)
                    self.assertEqual((self.work / 'Cargo.toml').read_text(), changed)

    def test_version_check_accepts_a_version_received_from_a_base_merge(self):
        self.version_base('[package]\nname = "fixture"\nversion = "1.2.3"\n')
        self.git('checkout', '-qb', 'feature/check')
        (self.work / 'README.md').write_text('# Feature branch change\n')
        self.git('add', 'README.md')
        self.git('commit', '-qm', 'feature change')
        self.git('checkout', '-q', 'main')
        current = '[package]\nname = "fixture"\nversion = "1.2.4"\n'
        (self.work / 'Cargo.toml').write_text(current)
        self.git('add', 'Cargo.toml')
        self.git('commit', '-qm', 'base version update')
        self.git('push', '-q')
        self.git('checkout', '-q', 'feature/check')
        self.git('merge', '--no-edit', 'main')
        self.env['GITHUB_HEAD_REF'] = 'feature/check'
        result = self.script('check-version-modification.rs')
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertIn('Version check passed', result.stdout)
        self.assertEqual((self.work / 'Cargo.toml').read_text(), current)
        self.assertEqual(self.git('status', '--porcelain'), '')

    def test_version_check_accepts_dependency_only_changes(self):
        base = ('[dependencies.decoy]\nversion = "9.9.9"\n'
                '[package]\nname = "fixture"\nversion = "1.2.3"\n')
        self.version_base(base)
        changed = base.replace('"9.9.9"', '"9.9.10"')
        (self.work / 'Cargo.toml').write_text(changed)
        result = self.script('check-version-modification.rs')
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertIn('Version check passed', result.stdout)
        self.assertEqual((self.work / 'Cargo.toml').read_text(), changed)

    def test_version_check_compares_parsed_identity_across_manifest_layouts(self):
        base = "[package]\nname = 'fixture'\n  version = '1.2.3' # literal\n"
        self.version_base(base)
        nested = self.work / 'rust'
        nested.mkdir()
        (self.work / 'Cargo.toml').rename(nested / 'Cargo.toml')
        result = self.script('check-version-modification.rs')
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertIn('Version check passed', result.stdout)

    def advance_remote_version_base(self):
        stale = self.git('rev-parse', 'origin/main').strip()
        other = self.work.parent / 'other'
        origin = self.git('remote', 'get-url', 'origin').strip()
        subprocess.run(['git', 'clone', '-q', '--branch', 'main', origin, str(other)], check=True)
        def git(*args):
            return subprocess.run(['git', *args], cwd=other, capture_output=True, text=True, check=True).stdout
        git('config', 'user.name', 'Fixture')
        git('config', 'user.email', 'fixture@example.invalid')
        current = '[package]\nname = "fixture"\nversion = "1.2.4"\n'
        (other / 'Cargo.toml').write_text(current)
        git('add', 'Cargo.toml')
        git('commit', '-qm', 'advance remote base')
        git('push', '-q', 'origin', 'main')
        return stale, git('rev-parse', 'HEAD').strip(), current

    def test_version_check_uses_fetched_commit_under_a_custom_refmap(self):
        self.version_base('[package]\nname = "fixture"\nversion = "1.2.3"\n')
        stale, current_commit, current = self.advance_remote_version_base()
        self.git('config', 'remote.origin.fetch', '+refs/heads/*:refs/remotes/alternate/*')
        fetched_head = self.work / '.git/FETCH_HEAD'
        fetched_head.write_text('unrelated fetch observation\n')
        for content, expected in [((self.work / 'Cargo.toml').read_text(), False), (current, True)]:
            with self.subTest(matches_current_base=expected):
                (self.work / 'Cargo.toml').write_text(content)
                result = self.script('check-version-modification.rs')
                self.assertEqual(result.returncode == 0, expected, result.stdout + result.stderr)
                self.assertIn(current_commit, result.stdout)
                self.assertEqual(self.git('rev-parse', 'origin/main').strip(), stale)
                self.assertEqual(fetched_head.read_text(), 'unrelated fetch observation\n')
                self.assertEqual(self.git('for-each-ref', 'refs/formal-ai/version-base'), '')
                self.assertEqual((self.work / 'Cargo.toml').read_text(), content)

    @unittest.skipUnless(os.name == 'posix', 'executable local Git interposer')
    def test_version_check_ignores_a_concurrent_tracking_ref_and_fetch_head_change(self):
        self.version_base('[package]\nname = "fixture"\nversion = "1.2.3"\n')
        stale, current_commit, current = self.advance_remote_version_base()
        real_git = shutil.which('git')
        binary = self.work.parent / 'bin'
        binary.mkdir()
        wrapper = binary / 'git'
        wrapper.write_text('#!/usr/bin/env python3\n'
                           'import os,subprocess,sys\n'
                           f'real={real_git!r}\n'
                           'args=sys.argv[1:]\n'
                           'result=subprocess.run([real,*args])\n'
                           'if result.returncode == 0 and args and args[0] == "fetch":\n'
                           f' subprocess.run([real,"update-ref","refs/remotes/origin/main",{stale!r}],check=True)\n'
                           ' with open(".git/FETCH_HEAD","w") as stream: stream.write("concurrent observation\\n")\n'
                           'sys.exit(result.returncode)\n')
        wrapper.chmod(0o755)
        self.env['PATH'] = str(binary) + os.pathsep + self.env['PATH']
        for content, expected in [((self.work / 'Cargo.toml').read_text(), False), (current, True)]:
            with self.subTest(matches_current_base=expected):
                (self.work / 'Cargo.toml').write_text(content)
                result = self.script('check-version-modification.rs')
                self.assertEqual(result.returncode == 0, expected, result.stdout + result.stderr)
                self.assertIn(current_commit, result.stdout)
                self.assertEqual(self.git('rev-parse', 'origin/main').strip(), stale)
                self.assertEqual(self.git('for-each-ref', 'refs/formal-ai/version-base'), '')

    def test_version_check_refuses_invalid_base_branch_names(self):
        self.version_base('[package]\nname = "fixture"\nversion = "1.2.3"\n')
        for branch in ['main..invalid', 'main:refs/heads/other', 'main\nother', '']:
            with self.subTest(branch=branch):
                self.env['GITHUB_BASE_REF'] = branch
                result = self.script('check-version-modification.rs')
                self.assertNotEqual(result.returncode, 0, result.stdout + result.stderr)
                self.assertIn('Could not fetch the current base branch', result.stderr)
                self.assertNotIn('Version check passed', result.stdout)
                self.assertEqual(self.git('for-each-ref', 'refs/formal-ai/version-base'), '')

    def test_version_check_refuses_cached_base_after_failed_fetch(self):
        self.version_base("[package]\nname = 'fixture'\nversion = '1.2.3'\n")
        cached = self.git('show', 'origin/main:Cargo.toml')
        self.assertIn("version = '1.2.3'", cached)
        self.git('remote', 'set-url', 'origin', str(self.work / 'unavailable-origin.git'))
        result = self.script('check-version-modification.rs')
        self.assertNotEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertIn('Could not fetch the current base branch', result.stderr)
        self.assertNotIn('Version check passed', result.stdout)
        self.assertEqual(self.git('show', 'origin/main:Cargo.toml'), cached)

    def test_version_check_refuses_missing_inherited_and_malformed_versions(self):
        self.version_base("[package]\nname = 'fixture'\nversion = '1.2.3'\n")
        for version in ['', 'version.workspace = true\n', 'version = { workspace = true }\n',
                        'version = true\n', 'version = ""\n', 'version = "1.2.3"\nversion = "2.0.0"\n']:
            with self.subTest(version=version):
                content = ('[dependencies.decoy]\nversion = "1.2.3"\n'
                           "[package]\nname = 'fixture'\n" + version)
                (self.work / 'Cargo.toml').write_text(content)
                result = self.script('check-version-modification.rs')
                self.assertNotEqual(result.returncode, 0, result.stdout + result.stderr)
                self.assertIn('Could not read a crate version', result.stderr)
                self.assertEqual((self.work / 'Cargo.toml').read_text(), content)

    def test_repository_contains_the_writer_status_staging_inputs(self):
        source = (ROOT / 'scripts/version-and-commit.rs').read_text()
        declaration = re.search(r'const STATUS_SURFACES:.*?= &\[(.*?)\];', source, re.DOTALL)
        self.assertIsNotNone(declaration, 'the actual writer declares its status surfaces')
        paths = re.findall(r'"([^"]+)"', declaration.group(1))
        self.assertTrue(paths, 'the fixture must exercise the real staging contract')
        self.git('ls-files', '--error-unmatch', '--', *paths)
        self.assertEqual(self.git('status', '--porcelain'), '')

    def test_fixture_can_return_wrong_identity_for_an_unpublished_candidate(self):
        self.__class__.scope = '/crates/fixture/'
        for version in ['1.2.4', '2.4.10']:
            self.__class__.mode = 'success'
            connection = http.client.HTTPConnection('127.0.0.1', self.server.server_port, timeout=5)
            try:
                connection.request('GET', f'/api/v1/crates/fixture/{version}')
                response = connection.getresponse()
                self.assertEqual(response.status, 404)
                response.read()
                self.__class__.mode = 'wrong-identity'
                connection.request('GET', f'/api/v1/crates/fixture/{version}')
                response = connection.getresponse()
                self.assertEqual(response.status, 200)
                self.assertEqual(json.loads(response.read()), {'version': {'num': '0.0.0'}})
            finally:
                connection.close()

    def test_failed_lookup_never_decides_release_or_mutates_version(self):
        for mode in ['503', '429', '403', 'malformed', 'drop']:
            for script in ['check-release-needed.rs', 'version-and-commit.rs']:
                with self.subTest(mode=mode, script=script):
                    self.__class__.mode = mode
                    self.__class__.requests = []
                    self.output.unlink(missing_ok=True)
                    before = (self.work / 'Cargo.toml').read_bytes()
                    result = self.script(script)
                    self.assertNotEqual(result.returncode, 0, result.stdout + result.stderr)
                    self.assertNotIn('should_release=true', self.output.read_text() if self.output.exists() else '')
                    self.assertEqual((self.work / 'Cargo.toml').read_bytes(), before)
                    self.assertEqual(self.git('status', '--porcelain'), '')

    def test_individual_artifact_and_candidate_lookup_failures_stop_the_clis(self):
        (self.work / 'Dockerfile').write_text('FROM scratch\n')
        self.git('add', 'Dockerfile')
        self.git('commit', '-qm', 'Docker fixture')
        self.git('push', '-q')
        self.env['DOCKERHUB_IMAGE'] = 'fixture/image'
        for scope, scripts in [('/crates/fixture/', ['check-release-needed.rs', 'version-and-commit.rs']),
                               ('/repos/', ['check-release-needed.rs']),
                               ('/tags/', ['check-release-needed.rs'])]:
            for mode in ['503', '429', '403', 'malformed', 'wrong-identity', 'drop']:
                for script in scripts:
                    with self.subTest(scope=scope, mode=mode, script=script):
                        self.__class__.scope = scope
                        self.__class__.mode = mode
                        self.output.unlink(missing_ok=True)
                        result = self.script(script)
                        self.assertNotEqual(result.returncode, 0, result.stdout + result.stderr)
                        self.assertNotIn('should_release=true', self.output.read_text() if self.output.exists() else '')
                        self.assertIn('version="1.2.3"', (self.work / 'Cargo.toml').read_text())
                        self.assertEqual(self.git('status', '--porcelain'), '')

    def test_missing_repository_and_invalid_docker_image_are_errors(self):
        del self.env['GITHUB_REPOSITORY']
        result = self.script('check-release-needed.rs')
        self.assertNotEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertNotIn('should_release=true', self.output.read_text())
        self.env['GITHUB_REPOSITORY'] = 'fixture/repo'
        self.env['DOCKERHUB_IMAGE'] = 'invalid/image/name'
        (self.work / 'Dockerfile').write_text('FROM scratch\n')
        self.output.unlink()
        result = self.script('check-release-needed.rs')
        self.assertNotEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertNotIn('should_release=true', self.output.read_text())

    def test_writer_uses_highest_published_version_as_its_floor(self):
        self.__class__.maximum = '2.4.9'
        result = self.script('version-and-commit.rs')
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertIn('version="2.4.10"', (self.work / 'Cargo.toml').read_text())
        self.assertTrue(any(path.endswith('/2.4.10') for path in self.requests))
        self.assertEqual(self.git('status', '--porcelain'), '')

    def test_dropped_version_request_recovers_without_rerelease(self):
        self.__class__.mode = 'drop-once'
        result = self.script('check-release-needed.rs')
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertIn('should_release=false', self.output.read_text())
        self.assertGreaterEqual(len(self.requests), 4)

    def test_only_not_found_makes_a_missing_artifact_decision(self):
        self.__class__.mode = '404'
        result = self.script('check-release-needed.rs')
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertIn('should_release=true', self.output.read_text())
        self.assertIn('skip_bump=true', self.output.read_text())


if __name__ == '__main__':
    unittest.main()
