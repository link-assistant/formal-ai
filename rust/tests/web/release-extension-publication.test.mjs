import test from 'node:test';
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { publicationIdentity, publicationReceipt, qualifyPublicationOutcomes }
  from '../../../scripts/release-extension-publication.mjs';

const bytes = Buffer.from('data-only private fixture, not a VSIX');
const environment = {
  GITHUB_REPOSITORY: 'link-assistant/formal-ai', PACKAGE_RELEASE_TAG: 'v1.2.3',
  EXTENSION_SOURCE_COMMIT: 'a'.repeat(40), EXTENSION_SOURCE_TREE: 'b'.repeat(40),
  GITHUB_WORKFLOW_SHA: 'c'.repeat(40), EXTENSION_PRODUCER_RUN: '11',
  EXTENSION_PRODUCER_ATTEMPT: '1', GITHUB_RUN_ID: '22', GITHUB_RUN_ATTEMPT: '2',
  EXTENSION_PACKAGE_SHA256: createHash('sha256').update(bytes).digest('hex')
};
const manifest = { version: '0.345.0', publisher: 'link-assistant', name: 'formal-ai-vscode' };
const identity = publicationIdentity(environment, bytes, manifest);
const receipts = () => ['marketplace', 'open-vsx']
  .map(registry => publicationReceipt(identity, registry, 'skipped-unconfigured'));

test('release and extension versions are distinct source identities', () => {
  assert.equal(identity.tag, 'v1.2.3');
  assert.equal(identity.packageVersion, '0.345.0');
});
test('unconfigured registries remain unknown', () => {
  const result = qualifyPublicationOutcomes(identity, receipts());
  assert.equal(result.delivery, 'Unknown');
  assert.equal(result.publicationComplete, false);
  assert.equal(result.authenticated, false);
});
test('successful publish command alone does not establish registry delivery', () => {
  const rows = ['marketplace', 'open-vsx'].map(registry =>
    publicationReceipt(identity, registry, 'publication-command-completed', 0));
  assert.equal(qualifyPublicationOutcomes(identity, rows).registryEvidence, 'Missing');
});
test('unchanged package remains unverified', () => {
  assert.equal(publicationReceipt(identity, 'marketplace',
    'existing-publication-unverified').publicationComplete, false);
});
test('already-published cannot be asserted without independent registry evidence', () => {
  assert.throws(() => publicationReceipt(identity, 'marketplace', 'already-published'));
});
test('nonzero command cannot claim completed publication', () => {
  assert.throws(() => publicationReceipt(identity, 'marketplace', 'publication-command-completed', 1));
});
test('tampered authoritative package digest refuses', () => {
  assert.throws(() => publicationIdentity(environment, Buffer.from('different'), manifest));
});
for (const key of Object.keys(identity)) {
  test('cross-producer identity refuses: ' + key, () => {
    const rows = receipts();
    rows[0][key] = String(rows[0][key]) + 'foreign';
    assert.throws(() => qualifyPublicationOutcomes(identity, rows));
  });
}
test('missing registry refuses', () => {
  assert.throws(() => qualifyPublicationOutcomes(identity, receipts().slice(0, 1)));
});
test('duplicate registry refuses', () => {
  assert.throws(() => qualifyPublicationOutcomes(identity, [receipts()[0], receipts()[0]]));
});
test('caller authority booleans refuse', () => {
  const rows = receipts();
  rows[0].authenticated = true;
  rows[0].publicationComplete = true;
  assert.throws(() => qualifyPublicationOutcomes(identity, rows));
});

test('prepared publication binds packaged metadata, not stale source manifest', async () => {
  const {preparedPublicationIdentity} = await import('../../../scripts/release-extension-publication.mjs');
  const selected = {...environment, PACKAGE_RELEASE_TAG: 'v0.352.1', EXTENSION_BUILT_VERSION: '0.352.1'};
  const cargo = '[package]\nname = "formal-ai"\nversion = "0.352.1"\n[dependencies]\n';
  const packaged = {...manifest, version: '0.352.1'};
  const result = preparedPublicationIdentity(selected, bytes, manifest, cargo, packaged);
  assert.equal(result.packageVersion, '0.352.1');
  assert.equal(result.sourceManifestVersion, '0.345.0');
  for (const wrong of [manifest, {...packaged, name: 'foreign'}, {...packaged, publisher: 'foreign'}]) {
    assert.throws(() => preparedPublicationIdentity(selected, bytes, manifest, cargo, wrong));
  }
  assert.throws(() => preparedPublicationIdentity({...selected, EXTENSION_BUILT_VERSION: '0.345.0'},
    bytes, manifest, cargo, packaged));
  assert.throws(() => preparedPublicationIdentity({...selected, PACKAGE_RELEASE_TAG: 'v0.345.0'},
    bytes, manifest, cargo, packaged));
  assert.throws(() => preparedPublicationIdentity(selected, bytes, manifest,
    cargo + '[package]\nversion = "0.352.1"\n', packaged));
});

test('original stale manifest counterexample now refuses before receipt construction', () => {
  assert.throws(() => publicationIdentity({...environment, EXTENSION_BUILT_VERSION: '0.352.1'}, bytes, manifest));
});
