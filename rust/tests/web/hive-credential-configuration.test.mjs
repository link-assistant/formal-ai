import test from 'node:test';
import assert from 'node:assert/strict';
import {requireTaskCredentialConfiguration} from '../../../scripts/lib/hive-delivery-credentials.mjs';

test('external tasks fail before builds when configured credentials are absent or incomplete', () => {
  for (const environment of [{}, {GH_TOKEN: 'default-repository-token'}, {AUTOMATION_APP_ID: '123'},
    {AUTOMATION_APP_PRIVATE_KEY: 'private-placeholder'}, {AUTOMATION_TOKEN: ' \n'}]) {
    assert.throws(() => requireTaskCredentialConfiguration(environment), /External Hive tasks require AUTOMATION_TOKEN/u);
  }
});
test('configured token or complete App pair passes presence check without claiming target access or echoing secrets', () => {
  for (const environment of [{AUTOMATION_TOKEN: 'private-placeholder'},
    {AUTOMATION_APP_ID: '123', AUTOMATION_APP_PRIVATE_KEY: 'private-placeholder'}]) {
    assert.deepEqual(requireTaskCredentialConfiguration(environment), {configured: true, deliveryVerified: false});
    assert(!JSON.stringify(requireTaskCredentialConfiguration(environment)).includes('private-placeholder'));
  }
});
