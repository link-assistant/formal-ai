import test from 'node:test';
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {readFileSync} from 'node:fs';
import {declaredPolicyRoots, policyIdentity} from '../../../scripts/lib/hive-delivery-source.mjs';
import {cleanupPolicyIdentity} from '../../../scripts/lib/generated-task-cleanup-journal.mjs';
import {admissionPolicy} from '../../../scripts/lib/hive-delivery-admission.mjs';

const repository = 'link-assistant/formal-ai', protocol = 'a'.repeat(40);
const deliveryPath = 'scripts/lib/hive-delivery-source.mjs';
const cleanupPath = 'scripts/lib/generated-task-cleanup-journal.mjs';
const inputPath = 'data/benchmarks/issue-1364-client-replay-inputs.json';
const hash = value => createHash('sha256').update(value).digest('hex');
const inputs = readFileSync(new URL('../../../data/benchmarks/issue-1364-client-replay-inputs.json', import.meta.url));

function historicalReader() {
  const deliveryRoots = [deliveryPath, '.github/workflows/hive-original-delivery.yml'];
  const cleanupRoots = [cleanupPath, '.github/workflows/hive-original-delivery-finalize.yml'];
  const members = new Map([
    [deliveryPath, Buffer.from('export const policyRoots = ' + JSON.stringify(deliveryRoots) + ';\n')],
    [cleanupPath, Buffer.from('const cleanupRoots = ' + JSON.stringify(cleanupRoots) + ';\n')],
    [deliveryRoots[1], Buffer.from('original delivery workflow\n')],
    [cleanupRoots[1], Buffer.from('original cleanup workflow\n')],
    [inputPath, inputs],
  ]);
  const calls = [];
  const reader = {async read(endpoint) {
    const prefix = 'repos/' + repository + '/contents/', suffix = '?ref=' + protocol;
    assert(endpoint.startsWith(prefix) && endpoint.endsWith(suffix));
    const name = endpoint.slice(prefix.length, -suffix.length); calls.push(name);
    const bytes = members.get(name);
    assert(bytes, 'the old commit has no newly introduced source files: ' + name);
    return {type: 'file', encoding: 'base64', size: bytes.length, content: bytes.toString('base64'),
      sha: createHash('sha1').update('blob ' + bytes.length + '\0').update(bytes).digest('hex')};
  }};
  const roots = paths => paths.map(path => ({path, sha256: hash(members.get(path))}));
  return {reader, calls, members, expectedDelivery: hash(JSON.stringify({schema: admissionPolicy,
    inputHash: hash(inputs), roots: roots(deliveryRoots)})),
  expectedCleanup: hash(JSON.stringify({schema: 'formal-ai-hive-cleanup-policy/v1', roots: roots(cleanupRoots)}))};
}

test('historical policies retain their original hashes when newer source files did not exist yet', async () => {
  const fixture = historicalReader();
  const delivery = await policyIdentity(fixture.reader, repository, protocol);
  assert.equal(delivery.policyHash, fixture.expectedDelivery);
  assert.equal((await cleanupPolicyIdentity(fixture.reader, repository, protocol)).policyHash, fixture.expectedCleanup);
  assert(!fixture.calls.includes('scripts/lib/hive-isolated-task.mjs'));
  assert(!fixture.calls.includes('.github/actions/automation-token/action.yml'));
  const count = fixture.calls.length;
  await policyIdentity(fixture.reader, repository, protocol);
  await cleanupPolicyIdentity(fixture.reader, repository, protocol);
  assert.equal(fixture.calls.length, count, 'successful identities reuse immutable source observations');
});

test('failed historical identity reads are cached and never repeatedly request the same missing source', async () => {
  for (const identify of [policyIdentity, cleanupPolicyIdentity]) {
    let calls = 0;
    const reader = {async read() {calls++; throw new Error('observed HTTP 404');}};
    for (let index = 0; index < 5; index++) {
      await assert.rejects(identify(reader, repository, protocol), /HTTP 404/u);
    }
    assert.equal(calls, 1);
  }
});

test('source declarations cannot execute expressions, omit their own source, or select unsafe paths', () => {
  const valid = "export const policyRoots = ['" + deliveryPath + "', 'scripts/example.mjs',];\n";
  assert.deepEqual(declaredPolicyRoots(Buffer.from(valid), 'policyRoots', deliveryPath),
    [deliveryPath, 'scripts/example.mjs']);
  for (const body of [valid + valid, valid.replace("'scripts/example.mjs'", 'process.exit(1)'),
    valid.replace('scripts/example.mjs', '../outside'), valid.replace('scripts/example.mjs', deliveryPath),
    "const policyRoots = ['scripts/example.mjs'];", "const policyRoots = [...other];"]) {
    assert.throws(() => declaredPolicyRoots(Buffer.from(body), 'policyRoots', deliveryPath));
  }
});

test('a changed historical member or forged Git blob identity cannot authenticate the original policy', async () => {
  const fixture = historicalReader();
  fixture.members.set('.github/workflows/hive-original-delivery-finalize.yml', Buffer.from('changed workflow\n'));
  assert.notEqual((await cleanupPolicyIdentity(fixture.reader, repository, protocol)).policyHash, fixture.expectedCleanup);
  const forged = historicalReader(), read = forged.reader.read;
  forged.reader.read = async endpoint => ({...await read(endpoint), sha: '0'.repeat(40)});
  await assert.rejects(cleanupPolicyIdentity(forged.reader, repository, protocol), /source Git blob/u);
});
