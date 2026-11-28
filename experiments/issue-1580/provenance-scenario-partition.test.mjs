import assert from 'node:assert/strict';
import test from 'node:test';
import {readFileSync} from 'node:fs';
import {gunzipSync} from 'node:zlib';
import {createHash} from 'node:crypto';
const read = path => readFileSync(new URL('../../' + path, import.meta.url), 'utf8');
const fixture = new URL('./fixtures/hosted-javascript-de79e6ec/', import.meta.url);

test('the over-budget provenance scenario is divided without losing any original setup or assertions', () => {
  const original = gunzipSync(readFileSync(new URL('full-provenance-cli.before.mjs.gz', fixture))).toString('utf8');
  const manifest = JSON.parse(readFileSync(new URL('scenario-partition.json', fixture), 'utf8'));
  assert.equal(createHash('sha256').update(original).digest('hex'), manifest.originalSha256);
  assert(manifest.originalObservedPrimaryTestSeconds > 270);
  const setupStart = original.indexOf(' const directory=');
  const installedStart = original.indexOf('  const before=fileSystem.statSync(delivered,{bigint:true}),positive=invoke(requestFile);');
  const archivedStart = original.indexOf('  const archivedRequest=');
  const guardsStart = original.indexOf("  assert.throws(()=>verifyFullRepositoryProof({...request,sourceMode:");
  const end = original.indexOf(' }finally{fileSystem.rmSync(directory', guardsStart);
  const identity = "  assert.equal(fileSystem.statSync(delivered,{bigint:true}).ctimeNs,before.ctimeNs);assert.equal(git(workspace,['rev-parse','HEAD']),head);\n";
  const expected = [
    original.slice(installedStart, archivedStart).replace('  const before=fileSystem.statSync(delivered,{bigint:true}),positive=invoke(requestFile);', '  const positive=invoke(requestFile);') + identity,
    original.slice(archivedStart, guardsStart) + identity,
    original.slice(guardsStart, end).replace(identity, ''),
  ];
  const helper = read(manifest.sharedFixture);
  assert(helper.includes(original.slice(setupStart, installedStart).replace("new URL('../../../js/", "new URL('../../../../js/")));
  assert(helper.trimEnd().endsWith(original.slice(end, original.indexOf("test('runtime request sequences", end)).replace('\n});', '\n}').trimEnd()));
  assert.equal(manifest.scenarios.length, 3);
  for (const [i, scenario] of manifest.scenarios.entries()) {
    const source = read(scenario.path);
    assert(source.includes(expected[i]), scenario.role);
    assert.equal(createHash('sha256').update(expected[i]).digest('hex'), scenario.bodySha256);
    assert(source.includes('withFullCliFixture(({'));
    assert.equal(scenario.assertionCount, expected[i].split('assert').length - 1);
  }
  const tail = original.slice(original.indexOf("test('runtime request sequences", end));
  assert(read(manifest.scenarios[0].path).endsWith(tail), 'all remaining request/append controls stay byte-identical');
});
