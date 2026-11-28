"""Exercise the actual publishing CLI with a recording Cargo subprocess."""
import gzip
import hashlib
import json
import os
from pathlib import Path


def add_publish_process_controls(harness, scripts):
    fixtures = harness / 'fixtures'
    original = gzip.decompress((fixtures / 'publish-crate-before-token-fix.rs.gz').read_bytes())
    metadata = json.loads((fixtures / 'publish-crate-before-token-fix.json').read_text())
    if len(original) != metadata['bytes'] or hashlib.sha256(original).hexdigest() != metadata['sha256']:
        raise RuntimeError('the original publishing credential control changed')
    (scripts / 'publish-crate-before-token-fix.rs').write_bytes(original)
    return ('\n[[bin]]\nname = "publish-crate-before-token-fix"\n'
            'path = "../scripts/publish-crate-before-token-fix.rs"\ntest = false\n')


class PublishProcessRegressions:
    def publish_stub(self, result='success'):
        folder = self.work.parent / 'publish-stub'
        folder.mkdir(exist_ok=True)
        script = folder / 'cargo'
        script.write_text('''#!/usr/bin/env python3
import json,os,sys
from pathlib import Path
Path(os.environ['PUBLISH_STUB_RECEIPT']).write_text(json.dumps({
    'arguments':sys.argv[1:],
    'token':os.environ.get('CARGO_REGISTRY_TOKEN'),
    'cwd':os.getcwd(),
}))
if os.environ['PUBLISH_STUB_RESULT']=='auth-failed':
    print('error: unauthorized: fixture token rejected',file=sys.stderr)
    raise SystemExit(101)
''')
        script.chmod(0o755)
        self.publish_receipt = self.work.parent / 'publish-receipt.json'
        self.env.update(PATH=str(folder) + os.pathsep + os.environ['PATH'],
                        PUBLISH_STUB_RECEIPT=str(self.publish_receipt),
                        PUBLISH_STUB_RESULT=result)
        for key in ['CARGO_REGISTRY_TOKEN', 'CARGO_TOKEN']:
            self.env.pop(key, None)

    def publish_call(self, *args, before=False):
        self.publish_receipt.unlink(missing_ok=True)
        self.output.unlink(missing_ok=True)
        result = self.script('publish-crate-before-token-fix.rs' if before else 'publish-crate.rs', *args)
        receipt = json.loads(self.publish_receipt.read_text())
        return result, receipt

    def assert_private_token(self, result, receipt, token):
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertEqual(receipt['arguments'], ['publish', '--allow-dirty', '-p', 'fixture'])
        self.assertEqual(receipt['token'], token)
        self.assertNotIn(token, result.stdout + result.stderr)
        self.assertEqual(self.output.read_text(), 'publish_result=success\n')

    def test_publish_cli_keeps_selected_token_out_of_cargo_arguments(self):
        self.publish_stub()
        self.env.update(CARGO_REGISTRY_TOKEN='fixture-native-token', CARGO_TOKEN='fixture-alternate-token')
        old, old_receipt = self.publish_call('--token', 'fixture-cli-token', before=True)
        self.assertEqual(old.returncode, 0)
        self.assertIn('fixture-cli-token', old_receipt['arguments'])
        result, receipt = self.publish_call('--token', 'fixture-cli-token')
        self.assert_private_token(result, receipt, 'fixture-cli-token')

    def test_publish_cli_preserves_native_and_legacy_environment_precedence(self):
        self.publish_stub()
        self.env.update(CARGO_REGISTRY_TOKEN='fixture-native-token', CARGO_TOKEN='fixture-alternate-token')
        result, receipt = self.publish_call()
        self.assert_private_token(result, receipt, 'fixture-native-token')
        self.env['CARGO_REGISTRY_TOKEN'] = ''
        result, receipt = self.publish_call()
        self.assert_private_token(result, receipt, 'fixture-alternate-token')

    def test_publish_cli_empty_override_falls_back_to_configured_environment(self):
        self.publish_stub()
        self.env['CARGO_REGISTRY_TOKEN'] = 'fixture-native-token'
        result, receipt = self.publish_call('--token', '')
        self.assert_private_token(result, receipt, 'fixture-native-token')

    def test_publish_cli_does_not_warn_before_cargo_credential_provider_runs(self):
        self.publish_stub()
        old, _ = self.publish_call(before=True)
        self.assertEqual(old.returncode, 0)
        self.assertIn('::warning::', old.stdout)
        result, receipt = self.publish_call()
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertEqual(receipt['arguments'], ['publish', '--allow-dirty', '-p', 'fixture'])
        self.assertIsNone(receipt['token'])
        self.assertNotIn('::warning::', result.stdout + result.stderr)
        self.assertEqual(self.output.read_text(), 'publish_result=success\n')

    def test_publish_cli_keeps_rejected_credentials_a_failed_publication(self):
        self.publish_stub('auth-failed')
        result, receipt = self.publish_call()
        self.assertEqual(result.returncode, 1)
        self.assertIsNone(receipt['token'])
        self.assertEqual(self.output.read_text(), 'publish_result=auth_failed\n')
