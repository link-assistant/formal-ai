import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import {tmpdir} from 'node:os';
import {execFileSync} from 'node:child_process';
import {test} from 'node:test';
import {runInNewContext} from 'node:vm';
import {fileURLToPath} from 'node:url';
import {inputs, digestFile, validateAppObservation, validateRendererProbe, rendererReadinessExpression}
  from '../../../scripts/verify-public-macos-release.mjs';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../../..');
const env = {CI: 'true', GITHUB_ACTIONS: 'true', MACOS_ARCH: 'arm64',
  RELEASE_TAG: 'v2.7.11', RELEASE_VERSION: '2.7.11',
  RELEASE_SOURCE_SHA: 'a'.repeat(40), RELEASE_SOURCE_TREE: 'b'.repeat(40),
  RELEASE_DOWNLOAD_DIRECTORY: '/tmp/downloads'};
const expected = {version: '2.7.11', mode: 'adhoc', nativeSha256: 'c'.repeat(64), nativeBytes: 91};
const app = {bundleId: 'ai.formal.desktop', version: '2.7.11', envelope: true,
  signatureStatus: 0, display: 'Signature=adhoc\n', nativeSha256: 'c'.repeat(64),
  nativeBytes: 91, nativeVersion: 'formal-ai 2.7.11\n'};

test('public package execution requires exact hosted platform architecture and release identity', () => {
  assert.equal(inputs(env, 'darwin', 'arm64').label, 'macos-arm64');
  for (const change of [{CI: 'false'}, {GITHUB_ACTIONS: ''}, {MACOS_ARCH: 'x64'},
    {RELEASE_TAG: 'v2.7.12'}, {RELEASE_SOURCE_SHA: 'not-a-commit'},
    {RELEASE_SOURCE_TREE: 'A'.repeat(40)}, {RELEASE_DOWNLOAD_DIRECTORY: 'relative'}]) {
    assert.throws(() => inputs({...env, ...change}, 'darwin', 'arm64'));
  }
  assert.throws(() => inputs(env, 'linux', 'arm64'));
});

test('both self-signed and Developer ID app observations retain exact signed native identity', () => {
  validateAppObservation(app, expected);
  validateAppObservation({...app, display: 'Authority=Developer ID Application: Example\n'},
    {...expected, mode: 'signed'});
  for (const change of [{bundleId: 'other'}, {version: '2.7.12'}, {envelope: false},
    {signatureStatus: 1}, {display: ''}, {display: 'Signature=adhoc\nAuthority=Developer ID Application: X\n'},
    {nativeSha256: 'd'.repeat(64)}, {nativeBytes: 90}, {nativeVersion: 'formal-ai 2.7.12'}]) {
    assert.throws(() => validateAppObservation({...app, ...change}, expected));
  }
});

test('an open process or arbitrary debugging page cannot substitute for the real renderer bridge', () => {
  const probe = {readyState: 'complete', pathname: '/app/index.html', desktop: '1',
    status: {appVersion: '2.7.11'}, ui: {rootChildren: 1, rootTextLength: 20,
      composerCount: 1, composerPlaceholderLength: 8, submitCount: 1,
      submitTextLength: 4, messageListCount: 1}};
  validateRendererProbe(probe, '2.7.11');
  for (const change of [{readyState: 'loading'}, {pathname: '/'}, {desktop: null},
    {status: {}}, {status: {appVersion: '2.7.12'}},
    {ui: {...probe.ui, rootChildren: 0}}, {ui: {...probe.ui, rootTextLength: 0}},
    {ui: {...probe.ui, composerCount: 0}}, {ui: {...probe.ui, submitCount: 0}},
    {ui: {...probe.ui, submitTextLength: 0}}, {ui: {...probe.ui, messageListCount: 0}}]) {
    assert.throws(() => validateRendererProbe({...probe, ...change}, '2.7.11'));
  }
});

test('archive hashing reads actual complete bytes with bounded chunks', context => {
  const directory = fs.mkdtempSync(path.join(tmpdir(), 'public-macos-hash-'));
  context.after(() => fs.rmSync(directory, {recursive: true, force: true}));
  const file = path.join(directory, 'input'); fs.writeFileSync(file, 'abc');
  assert.equal(digestFile(file), 'ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad');
  fs.appendFileSync(file, 'changed');
  assert.notEqual(digestFile(file), 'ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad');
});

test('real ZIP metadata preflight accepts contained framework aliases and refuses escape or cycles', () => {
  const python = String.raw`
import importlib.util, io, stat, zipfile
spec = importlib.util.spec_from_file_location('guard', ${JSON.stringify(path.join(root, 'scripts/verify-public-macos-zip.py'))})
guard = importlib.util.module_from_spec(spec)
spec.loader.exec_module(guard)
def archive(link=None, extra=None):
    data = io.BytesIO()
    with zipfile.ZipFile(data, 'w') as output:
        output.writestr('formal-ai Desktop.app/Contents/Info.plist', 'fixture')
        if link:
            info = zipfile.ZipInfo('formal-ai Desktop.app/Contents/link')
            info.external_attr = (stat.S_IFLNK | 0o777) << 16
            output.writestr(info, link)
        if extra:
            output.writestr(extra, 'fixture')
    return zipfile.ZipFile(io.BytesIO(data.getvalue()))
guard.validate_archive(archive('Frameworks/Versions/Current'))
for link, extra in [('/tmp/outside', None), ('../../outside', None), ('link', None),
                    (None, '../outside'), (None, '/absolute'), (None, 'other.app/file')]:
    try:
        guard.validate_archive(archive(link, extra))
    except ValueError:
        continue
    raise AssertionError((link, extra))
`;
  execFileSync('python3', ['-B', '-c', python], {timeout: 10000});
});

test('the hosted entry and consumer preserve public signatures and independent DMG and ZIP paths', () => {
  const shell = fs.readFileSync(path.join(root, 'scripts/verify-public-macos-release.sh'), 'utf8');
  execFileSync('bash', ['-n'], {input: shell});
  const source = fs.readFileSync(path.join(root, 'scripts/verify-public-macos-release.mjs'), 'utf8');
  assert.ok(source.includes("for (const format of ['dmg', 'zip'])"));
  assert.ok(source.includes("'-nobrowse', '-readonly'"));
  assert.ok(source.includes('verify-public-macos-zip.py'));
  assert.ok(source.includes("['--verify', '--deep', '--strict', '--verbose=2', application]"));
  assert.ok(source.includes('validateSigningEvidence(signing, source'));
  assert.ok(source.includes('signing.native_component.sha256'));
  assert.ok(!/--sign|--remove-signature|--noqtn/u.test(source));
});

test('the actual CDP expression refuses an empty or broken React surface despite a live preload bridge', async () => {
  const model = {
    'textarea[data-testid="chat-composer-input"]': 1,
    'button[data-testid="chat-composer-submit"]': 1,
    '[data-testid="message-list"]': 1
  };
  const root = {childElementCount: 1, innerText: 'Conversations Send',
    querySelector: selector => selector.includes('input')
      ? {getAttribute: () => 'Ask anything'} : {innerText: 'Send'},
    querySelectorAll: selector => ({length: model[selector] ?? 0})};
  const probe = async surface => await runInNewContext(rendererReadinessExpression, {
    document: {readyState: 'complete', getElementById: id => id === 'root' ? surface : null},
    location: {pathname: '/app/index.html', search: '?desktop=1'}, URLSearchParams,
    window: {FormalAiDesktop: {getStatus: async () => ({appVersion: '2.7.11'})}}
  });
  validateRendererProbe(await probe(root), '2.7.11');
  for (const broken of [null, {...root, childElementCount: 0},
    {...root, innerText: 'Application crashed', querySelectorAll: () => ({length: 0})}]) {
    const actual = await probe(broken);
    assert.throws(() => validateRendererProbe(actual, '2.7.11'), /actual rendered desktop UI/u);
  }
});
