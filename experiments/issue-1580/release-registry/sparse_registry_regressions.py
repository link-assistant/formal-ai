"""Native waiter controls for sparse-index priority and uncertain probes."""
import gzip
import hashlib
import json


def add_sparse_registry_controls(harness, scripts):
    fixtures = harness / 'fixtures'
    original = gzip.decompress((fixtures / 'wait-for-crate-before-sparse.rs.gz').read_bytes())
    metadata = json.loads((fixtures / 'wait-for-crate-before-sparse.json').read_text())
    if len(original) != metadata['bytes'] or hashlib.sha256(original).hexdigest() != metadata['sha256']:
        raise RuntimeError('the original API-only waiter control changed')
    (scripts / 'wait-for-crate-before-sparse.rs').write_bytes(original)
    return ('\n[[bin]]\nname = "wait-for-crate-before-sparse"\n'
            'path = "../scripts/wait-for-crate-before-sparse.rs"\ntest = false\n')


class SparseRegistryRegressions:
    def index_reply(self, body, status=200):
        self.__class__.sparse_reply = (status, body)

    def wait_call(self, *args, before=False):
        self.__class__.requests = []
        self.output.unlink(missing_ok=True)
        return self.script('wait-for-crate-before-sparse.rs' if before else 'wait-for-crate.rs',
                           '--max-attempts', '2', '--sleep-seconds', '0', *args)

    def test_sparse_waiter_resolves_a_version_while_the_api_is_unavailable(self):
        self.__class__.mode = '503'
        self.index_reply('{"name":"fixture","vers":"1.2.3"}\n')
        old = self.wait_call(before=True)
        self.assertNotEqual(old.returncode, 0)
        result = self.wait_call()
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertEqual(self.requests, ['/_sparse/fi/xt/fixture'])
        self.assertEqual(self.output.read_text(), 'crate_available=true\n')

    def test_sparse_waiter_uses_registry_layout_for_short_and_mixed_case_names(self):
        for name, path in [('a', '1/a'), ('ab', '2/ab'), ('abc', '3/a/abc'), ('Serde', 'se/rd/serde')]:
            with self.subTest(name=name):
                self.index_reply(json.dumps({'name': name.lower(), 'vers': '1.2.3'}))
                result = self.wait_call('--crate-name', name)
                self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
                self.assertEqual(self.requests, ['/_sparse/' + path])

    def test_sparse_waiter_falls_back_to_a_validated_api_answer(self):
        for status, body in [(403, ''), (429, ''), (503, ''), (404, ''), (200, 'not json'),
                             (200, '{"name":"fixture","vers":"0.9.0"}')]:
            with self.subTest(status=status, body=body):
                self.index_reply(body, status)
                result = self.wait_call()
                self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
                self.assertEqual(self.requests, ['/_sparse/fi/xt/fixture', '/api/v1/crates/fixture/1.2.3'])

    def test_sparse_waiter_rejects_spoofed_or_incomplete_index_records(self):
        self.__class__.mode = '403'
        for body in ['{"name":"other","vers":"1.2.3"}',
                     '{"name":"fixture","vers":"1.2.30"}',
                     '{"name":"fixture","note":"\\"vers\\":\\"1.2.3\\"","vers":"0.9.0"}',
                     '{"name":"fixture","vers":"1.2.3"}\nnot json', '', '{}']:
            with self.subTest(body=body):
                self.index_reply(body)
                result = self.wait_call()
                self.assertNotEqual(result.returncode, 0)
                self.assertFalse(self.output.exists())
                self.assertLessEqual(len(self.requests), 4)

    def test_sparse_waiter_distinguishes_both_unknown_from_definitive_absence(self):
        self.__class__.mode = '503'
        self.index_reply('', 429)
        result = self.wait_call()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn('could not determine', result.stderr.lower())
        self.assertNotIn('was not visible', result.stderr.lower())
        self.assertEqual(len(self.requests), 4)
        self.index_reply('', 404)
        result = self.wait_call()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn('was not visible', result.stderr.lower())
        self.assertNotIn('could not determine', result.stderr.lower())

    def test_sparse_waiter_keeps_output_write_failure_fatal(self):
        self.index_reply('{"name":"fixture","vers":"1.2.3"}')
        self.output.mkdir()
        self.__class__.requests = []
        result = self.script('wait-for-crate.rs', '--max-attempts', '1')
        self.assertNotEqual(result.returncode, 0)
        self.assertTrue(self.output.is_dir())
