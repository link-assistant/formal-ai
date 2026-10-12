// Pure data archive profiles. They confer no source, import, or publication authority.
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {projectionRegistry} from './lib/native-session-projections.mjs';
import {collectNativeResponseRecords} from './lib/native-response-capture.mjs';
const sha = bytes => createHash('sha256').update(bytes).digest('hex');
const compiled = new WeakSet();
const mebibyte = 1024 * 1024;
const limits = Object.freeze({archiveBytes: 16 * mebibyte, memberBytes: 32 * mebibyte, inflatedBytes: 64 * mebibyte});
function regularPath(name) {
  assert.equal(typeof name, 'string');
  assert.match(name, /^[A-Za-z0-9_./-]+$/u);
  assert.ok(name.length <= 512 && !name.startsWith('/') && !name.endsWith('/'));
  assert.ok(name.split('/').every(part => part && part !== '.' && part !== '..' && !/[ .]$/u.test(part)));
  return name;
}
function profile(kind, names, inputs) {
  assert.ok(names.length > 0 && names.length <= 4096);
  names.forEach(regularPath);
  assert.equal(new Set(names.map(name => name.toLowerCase())).size, names.length, 'duplicate or case-colliding member');
  const result = Object.freeze({schema: 'qualified-data-artifact-profile/v1', kind,
    authority: 'not-granted', members: Object.freeze([...names].sort()),
    entryLimit: names.length, limits, inputs: Object.freeze(inputs)});
  compiled.add(result);
  return result;
}
/** Selection bytes come from the independently pinned committed selection file. */
export function nativeCaptureProfile(selectionBytes, reportBytes) {
  assert.ok(Buffer.isBuffer(selectionBytes) && Buffer.isBuffer(reportBytes));
  assert.ok(selectionBytes.length <= mebibyte && reportBytes.length <= limits.memberBytes);
  const selection = JSON.parse(selectionBytes), report = JSON.parse(reportBytes);
  assert.equal(selection.schema, 'native-response-capture-selection/v1');
  assert.ok(Array.isArray(selection.cases) && selection.cases.length > 0 && selection.cases.length <= 512);
  const cases = selection.cases;
  assert.equal(new Set(cases).size, cases.length, 'duplicate original selection');
  for (const id of cases) { regularPath(id.slice(0, id.lastIndexOf('::'))); assert.match(id, /^rust\/tests\/(unit|integration|source)\/[\w/.-]+\.rs::[A-Za-z_][A-Za-z_0-9]*$/u); }
  assert.equal(report.schema, 'native-original-response-capture/v1');
  assert.equal(report.originalAssertionsUnchanged, true);
  assert.equal(report.expectedAnswersUpdated, false);
  assert.equal(report.selected, cases.length); assert.equal(report.accepted, cases.length);
  assert.ok(Array.isArray(report.observations)); assert.equal(report.observations.length, cases.length);
  assert.deepEqual(report.observations.map(row => row.id).sort(), [...cases].sort());
  const names = ['report.json', 'producer-receipt.json'];
  const processes = new Set();
  for (const [index, row] of report.observations.entries()) {
    const batches = row.execution?.batches;
    assert.equal(row.execution?.succeeded, true); assert.equal(row.execution?.completed, 1);
    assert.ok(Array.isArray(batches)); assert.equal(batches.length, 1);
    const batch = batches[0];
    assert.ok(Number.isSafeInteger(batch.processId) && batch.processId > 0);
    assert.ok(!processes.has(batch.processId), 'duplicate native process'); processes.add(batch.processId);
    assert.equal(batch.exitCode, 0); assert.equal(batch.signal, null);
    assert.equal(batch.succeeded, true); assert.deepEqual(batch.problems, []);
    assert.deepEqual(batch.summary, {passed: 1, failed: 0, ignored: 0});
    assert.equal(row['raw-file'], 'native-' + batch.processId + '.jsonl');
    const folder = String(index).padStart(3, '0') + '/';
    names.push(folder + 'report.json', folder + 'stdout.bin', folder + 'stderr.bin', folder + row['raw-file']);
  }
  assert.equal(names.length, 4 * cases.length + 2);
  return profile('native-capture', names, {selectionSha256: sha(selectionBytes), reportSha256: sha(reportBytes), selected: cases.length});
}
/** Pure policy generation requires an already complete, independently authenticated source inventory. */
export function nativeCensusProfile(sourcePaths, uploadPaths) {
  assert.ok(Array.isArray(sourcePaths) && sourcePaths.length > 0);
  assert.equal(new Set(sourcePaths).size, sourcePaths.length);
  assert.deepEqual(uploadPaths, ['data/meta/self-ast', 'data/meta/self-ast.lino',
    'data/meta/self-healing-case.lino', 'docs/case-studies/issue-538/agent-cli-session-self-ast.json']);
  const names = sourcePaths.map(source => {
    regularPath(source); assert.match(source, /^rust\/src\/.+\.rs$/u);
    return 'data/meta/self-ast/' + source.slice('rust/'.length).replace(/\.rs$/u, '.lino');
  });
  names.push('data/meta/self-ast/README.md', ...uploadPaths.slice(1));
  return profile('native-census', names, {sourcePathsSha256: sha(Buffer.from([...sourcePaths].sort().join('\n') + '\n')), modules: sourcePaths.length});
}
/** Entries must come from a CRC/path-safe ZIP parser; this checks exact generated membership and bounded bytes only. */
export function verifyDataArtifactProfile(profileValue, entries, archiveBytes) {
  assert.ok(compiled.has(profileValue), 'source-derived compiled data profile required');
  assert.ok(Buffer.isBuffer(archiveBytes) && archiveBytes.length <= profileValue.limits.archiveBytes);
  assert.ok(Array.isArray(entries)); assert.equal(entries.length, profileValue.entryLimit);
  let total = 0;
  const rows = entries.map(entry => {
    regularPath(entry.name); assert.equal(entry.directory, false); assert.ok(Buffer.isBuffer(entry.bytes));
    assert.ok(entry.bytes.length <= profileValue.limits.memberBytes, 'member byte budget exceeded');
    total += entry.bytes.length; assert.ok(total <= profileValue.limits.inflatedBytes, 'inflated archive budget exceeded');
    return {path: entry.name, sha256: sha(entry.bytes), bytes: entry.bytes.length};
  }).sort((a, b) => a.path.localeCompare(b.path, 'en'));
  assert.equal(new Set(rows.map(row => row.path.toLowerCase())).size, rows.length, 'duplicate archive member');
  assert.deepEqual(rows.map(row => row.path).sort(), [...profileValue.members]);
  if (profileValue.kind === 'native-capture') {
    const byName = new Map(entries.map(entry => [entry.name, entry.bytes]));
    const reportBytes = byName.get('report.json');
    assert.equal(sha(reportBytes), profileValue.inputs.reportSha256, 'profile report bytes changed');
    const report = JSON.parse(reportBytes), receipt = JSON.parse(byName.get('producer-receipt.json'));
    assert.deepEqual(receipt.identity, report.identity, 'producer receipt identity differs');
    for (const [index, observation] of report.observations.entries()) {
      const folder = String(index).padStart(3, '0') + '/';
      assert.deepEqual(JSON.parse(byName.get(folder + 'report.json')), observation, 'per-case report differs');
      for (const stream of ['stdout', 'stderr']) assert.equal(sha(byName.get(folder + stream + '.bin')), observation[stream + '-sha256']);
      const execution = observation.execution.batches[0];
      const captured = collectNativeResponseRecords(byName.get(folder + observation['raw-file']), {
        caller: observation.caller, processId: execution.processId, execution,
        identity: receipt.identity, expectedIdentity: report.identity});
      assert.deepEqual(captured, observation.captured, 'maintained original record verification differs');
    }
  }
  return Object.freeze({schema: 'qualified-data-artifact-members/v1', kind: profileValue.kind,
    authority: 'not-granted', zipSha256: sha(archiveBytes), inflatedBytes: total,
    members: Object.freeze(rows.map(Object.freeze)), profileInputs: profileValue.inputs});
}

/** Separate finite session archive policy, derived from the pinned native producer registry. */
export function nativeSessionArchiveProfile(producerBytes) {
 assert.ok(Buffer.isBuffer(producerBytes));
 const paths=projectionRegistry(producerBytes.toString('utf8'));
 return profile('native-session', [...paths.map(path=>'generated/'+path),'producer-receipt.json','build.stdout.bin','build.stderr.bin','producer.stdout.bin','producer.stderr.bin'], {producerSha256:sha(producerBytes),selected:paths.length});
}
/** Only profiles compiled by this source module may select decoder limits. */
export function dataArtifactDecodePolicy(profileValue) {
 assert.ok(compiled.has(profileValue),'source-derived compiled data profile required');
 return Object.freeze({entryLimit:profileValue.entryLimit,...profileValue.limits});
}

/** Source-selected count-only probe permits CRC-safe decoding before a captured report chooses exact process names. It cannot verify final membership. */
export function nativeCaptureArchiveEnvelope(selectionBytes) {
 assert.ok(Buffer.isBuffer(selectionBytes) && selectionBytes.length<=1024*1024);
 const selection=JSON.parse(selectionBytes);assert.equal(selection.schema,'native-response-capture-selection/v1');
 assert.ok(Array.isArray(selection.cases)&&selection.cases.length>0&&selection.cases.length<=512);
 assert.equal(new Set(selection.cases).size,selection.cases.length);
 for(const id of selection.cases)assert.match(id,/^rust\/tests\/(unit|integration|source)\/[\w/.-]+\.rs::[A-Za-z_][A-Za-z_0-9]*$/u);
 return profile('native-capture-envelope',Array.from({length:4*selection.cases.length+2},(_,index)=>'probe/'+index),{selectionSha256:sha(selectionBytes),selected:selection.cases.length});
}
