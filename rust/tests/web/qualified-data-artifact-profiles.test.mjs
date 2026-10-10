import test from 'node:test';
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {nativeCaptureProfile, nativeCensusProfile, verifyDataArtifactProfile} from '../../../scripts/qualified-data-artifact-profiles.mjs';
import {collectNativeResponseRecords} from '../../../scripts/lib/native-response-capture.mjs';
const sha = bytes => createHash('sha256').update(bytes).digest('hex');
function fixture() {
  const identity = {'source-commit': 'a'.repeat(40), 'source-tree': 'b'.repeat(40),
    'cargo-lock-sha256': 'c'.repeat(64), 'native-inputs-sha256': 'd'.repeat(64),
    'test-inputs-sha256': 'e'.repeat(64), compiler: 'actual compiler witness', target: 'x86_64-unknown-linux-gnu',
    profile: 'release', features: 'all', 'rust-flags': '-Dwarnings', 'encoded-rust-flags': '', 'cargo-profile-overrides': {}};
  const caller = 'example::original_case';
  const execution = {processId: 42, exitCode: 0, signal: null, succeeded: true, problems: [],
    completed: 1, summary: {passed: 1, failed: 0, ignored: 0}};
  const raw = Buffer.from(JSON.stringify({schema: 'native-response-observation/v1',
    entrypoint: 'FormalAiEngine::answer_with_memory', 'process-id': 42, sequence: 1,
    'caller-thread': caller, prompt: 'an original input', 'config-debug': 'source config',
    history: [], context: {}, response: {answer: 'полезный ответ', intent: 'answer', confidence: 0.9,
      evidence_links: [], links_notation: 'answer', derivation_id: 'derived'}}) + '\n');
  const streams = {stdout: Buffer.from('1 passed\n'), stderr: Buffer.alloc(0)};
  const observation = {id: 'rust/tests/unit/example.rs::original_case', caller,
    execution: {succeeded: true, completed: 1, batches: [execution]}, 'raw-file': 'native-42.jsonl',
    'stdout-sha256': sha(streams.stdout), 'stderr-sha256': sha(streams.stderr),
    captured: collectNativeResponseRecords(raw, {caller, processId: 42, execution, identity, expectedIdentity: identity})};
  const report = {schema: 'native-original-response-capture/v1', originalAssertionsUnchanged: true,
    expectedAnswersUpdated: false, selected: 1, accepted: 1, observations: [observation], identity};
  const selection = {schema: 'native-response-capture-selection/v1', cases: [observation.id]};
  const selectionBytes = Buffer.from(JSON.stringify(selection)), reportBytes = Buffer.from(JSON.stringify(report));
  const values = {'report.json': reportBytes, 'producer-receipt.json': Buffer.from(JSON.stringify({identity})),
    '000/report.json': Buffer.from(JSON.stringify(observation)), '000/stdout.bin': streams.stdout,
    '000/stderr.bin': streams.stderr, '000/native-42.jsonl': raw};
  return {selection, report, selectionBytes, reportBytes,
    entries: Object.entries(values).map(([name, bytes]) => ({name, bytes, directory: false})), archive: Buffer.from('external CRC-safe ZIP bytes')};
}
function changedProfile(mutator) {const value = fixture(); mutator(value); return () => nativeCaptureProfile(Buffer.from(JSON.stringify(value.selection)), Buffer.from(JSON.stringify(value.report)));}
test('selection-derived 4N+2 profile verifies Unicode original records but grants no authority', () => {
  const value = fixture(), profile = nativeCaptureProfile(value.selectionBytes, value.reportBytes);
  assert.equal(profile.entryLimit, 6); assert.equal(profile.limits.archiveBytes, 16 * 1024 * 1024);
  assert.equal(profile.limits.inflatedBytes, 64 * 1024 * 1024);
  const manifest = verifyDataArtifactProfile(profile, value.entries, value.archive);
  assert.equal(manifest.members.length, 6); assert.equal(manifest.authority, 'not-granted');
  assert.equal(manifest.zipSha256, sha(value.archive)); assert.ok(Object.isFrozen(profile.members));
});
for (const [name, mutator] of [
  ['duplicate selected original', v => v.selection.cases.push(v.selection.cases[0])],
  ['unsafe original source path', v => v.selection.cases[0] = 'rust/tests/unit/../bad.rs::case'],
  ['foreign original observation', v => v.report.observations[0].id = 'rust/tests/unit/foreign.rs::case'],
  ['expected answer replacement', v => v.report.expectedAnswersUpdated = true],
  ['failed original execution', v => v.report.observations[0].execution.batches[0].exitCode = 1],
  ['unobserved raw process', v => v.report.observations[0]['raw-file'] = 'native-99.jsonl'],
  ['missing original acceptance', v => v.report.accepted = 0]
]) test(name + ' refuses', () => assert.throws(changedProfile(mutator)));
for (const [name, mutate] of [
  ['missing member', v => v.entries.pop()],
  ['foreign member', v => v.entries[0].name = 'foreign.json'],
  ['case-colliding member', v => v.entries[1].name = 'REPORT.JSON'],
  ['traversal member', v => v.entries[0].name = '../report.json'],
  ['directory member', v => v.entries[0].directory = true],
  ['changed report bytes', v => v.entries[0].bytes = Buffer.from('{}')],
  ['changed stdout bytes', v => v.entries.find(e => e.name.endsWith('stdout.bin')).bytes = Buffer.from('fake')],
  ['changed raw answer bytes', v => v.entries.find(e => e.name.endsWith('.jsonl')).bytes = Buffer.from('{}\n')],
  ['foreign producer identity', v => v.entries.find(e => e.name === 'producer-receipt.json').bytes = Buffer.from('{"identity":{}}')]
]) test(name + ' refuses', () => {
  const value = fixture(), profile = nativeCaptureProfile(value.selectionBytes, value.reportBytes); mutate(value);
  assert.throws(() => verifyDataArtifactProfile(profile, value.entries, value.archive));
});
test('serialized or forged profile cannot confer validation authority', () => {
  const value = fixture(), profile = nativeCaptureProfile(value.selectionBytes, value.reportBytes);
  assert.throws(() => verifyDataArtifactProfile({...profile}, value.entries, value.archive));
});
test('archive, member and inflated budgets remain finite', () => {
  const value = fixture(), profile = nativeCaptureProfile(value.selectionBytes, value.reportBytes);
  assert.throws(() => verifyDataArtifactProfile(profile, value.entries, Buffer.alloc(16 * 1024 * 1024 + 1)));
  const oversized = value.entries.map(entry => ({...entry})); oversized[0].bytes = Buffer.alloc(32 * 1024 * 1024 + 1);
  assert.throws(() => verifyDataArtifactProfile(profile, oversized, value.archive));
  const repeated = Buffer.alloc(23 * 1024 * 1024);
  const inflated = value.entries.map((entry, index) => ({...entry, bytes: index < 3 ? repeated : entry.bytes}));
  assert.throws(() => verifyDataArtifactProfile(profile, inflated, value.archive));
});
test('census profile derives exact source inventory and preserves unknown source proof status', () => {
  const uploads = ['data/meta/self-ast', 'data/meta/self-ast.lino', 'data/meta/self-healing-case.lino',
    'docs/case-studies/issue-538/agent-cli-session-self-ast.json'];
  const profile = nativeCensusProfile(['rust/src/agent.rs', 'rust/src/nested/source.rs'], uploads);
  assert.equal(profile.entryLimit, 6); assert.ok(profile.members.includes('data/meta/self-ast/src/nested/source.lino'));
  assert.equal(profile.authority, 'not-granted');
  assert.throws(() => nativeCensusProfile(['rust/src/agent.rs', 'rust/src/agent.rs'], uploads));
  assert.throws(() => nativeCensusProfile(['rust/src/../foreign.rs'], uploads));
  assert.throws(() => nativeCensusProfile(['rust/src/agent.rs'], [...uploads, 'unknown/path']));
});
