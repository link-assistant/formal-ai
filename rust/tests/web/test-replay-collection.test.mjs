import test from 'node:test';
import assert from 'node:assert/strict';
import {parseJson, toCompactJson, isFloat} from '../../../js/server/json.mjs';
import {createEvidenceClient, collectReplayEvidence, collectReplayFixture, sanitizeEvidence, replayOriginalDelivery}
  from '../../../scripts/collect-test-replays.mjs';
import {digest} from '../../../scripts/lib/generated-task-custody.mjs';

test('non-finite request and transfer bounds are rejected before any IO', async () => {
  let calls = 0;
  for (const options of [{bounds: {timeoutMs: Infinity}}, {bounds: {runs: NaN}},
    {aggregateBytes: Infinity}, {aggregateBytes: 268435457}, {maxRequests: 0}]) {
    assert.throws(() => createEvidenceClient({...options, transport: async () => { calls++; }}));
  }
  assert.equal(calls, 0);
});
test('read retries are finite; uncertain writes are never automatically repeated', async () => {
  let calls = 0;
  const client = createEvidenceClient({bounds: {retries: 2},
    transport: async () => { calls++; return {ok: false, text: '', timedOut: true}; }});
  await client.request('repos/a/b/actions/runs'); assert.equal(calls, 3);
  await client.request('repos/a/b/issues/1', {mutation: ['--method', 'PATCH']});
  assert.equal(calls, 4); assert.equal(client.attempts.length, 4);
});
test('aggregate request/byte budgets fail closed instead of endlessly repeating a transport', async () => {
  let calls = 0;
  const client = createEvidenceClient({maxRequests: 1, transport: async () => {
    calls++; return {ok: true, text: '{}', bytesReceived: 2}; }});
  await client.json('repos/a/b/actions/runs');
  await assert.rejects(client.json('repos/a/b/actions/runs'), /budget/u); assert.equal(calls, 1);
  const bytes = createEvidenceClient({aggregateBytes: 1,
    transport: async () => ({ok: true, text: '{}', bytesReceived: 2})});
  await assert.rejects(bytes.json('repos/a/b/actions/runs'), /byte bound/u);
});
test('every success, failed and cancelled client attempt survives diagnostic deduplication', async () => {
  const conclusions = ['success', 'failure', 'cancelled'];
  const transport = async ([, endpoint]) => {
    if (endpoint.includes('/workflows/')) return {ok: true, text: JSON.stringify({workflow_runs:
      conclusions.map((conclusion, index) => ({id: index + 1, status: 'completed', conclusion,
        head_sha: String(index + 1).repeat(40), html_url: `https://github.com/a/b/actions/runs/${index + 1}`}))})};
    if (endpoint.includes('/jobs?')) {
      const id = Number(/runs\/([0-9]+)/u.exec(endpoint)[1]);
      return {ok: true, text: JSON.stringify({total_count: 1, jobs: [{id: id + 10,
        name: 'Actual Hive agent original delivery', status: 'completed', conclusion: conclusions[id - 1], steps: []}]})};
    }
    return {ok: true, text: 'Error invalid arguments EINVAL --command token=ghp_privatevalue',
      bytesReceived: 80, streamComplete: true, tailComplete: true, cleanupComplete: true};
  };
  const result = await collectReplayEvidence({repository: 'a/b', transport});
  assert.equal(result.runs.length, 3); assert.equal(result.occurrences.length, 3);
  assert.equal(result.groups[0].occurrences.length, 3);
  assert.deepEqual(result.runs.map(run => run.conclusion), conclusions);
  assert(result.runs.every(run => run.jobs[0].structuredReplayAvailable === false));
  assert.doesNotMatch(JSON.stringify(result), /privatevalue|token=|invalid arguments/iu);
});
test('partial log observations cannot become complete structured replay evidence', async () => {
  const transport = async ([, endpoint]) => {
    if (endpoint.includes('/workflows/')) return {ok: true, text: JSON.stringify({workflow_runs: [{id: 1}]})};
    if (endpoint.includes('/jobs?')) return {ok: true, text: JSON.stringify({total_count: 1,
      jobs: [{id: 2, status: 'completed', conclusion: 'failure'}]})};
    return {ok: false, text: 'Error EACCES --file', truncated: true,
      downloadTruncated: true, timedOut: true, tailComplete: false};
  };
  const result = await collectReplayEvidence({repository: 'a/b', bounds: {retries: 0}, transport});
  const job = result.runs[0].jobs[0];
  assert.equal(job.logsUnavailable, false); assert.equal(job.transferTruncated, true);
  assert.equal(job.transferComplete, false); assert.equal(job.structuredReplayAvailable, false);
  assert.equal(job.tailComplete, false); assert.equal(job.timedOut, true);
});
test('account objects and credential values are removed recursively', () => {
  const value = sanitizeEvidence({actor: {login: 'private-person'}, user: {email: 'person@example.com'},
    nested: [{token: 'private-token', message: '/Users/private-person/log password=private-pass'}],
    apiResponse: '{"login":"private-person","name":"Private Person"}'});
  assert.doesNotMatch(JSON.stringify(value), /private-person|Private Person|example.com|private-token|private-pass/u);
});
function syntheticOracleFixture() {
  return {schema: 'original-hive-delivery-replay/v1', complete: true, tool: 'codex', language: 'Rust',
    provenance: {sourceHead: '49f6dc896129a6c247ac13b122da0d41d8808933',
      sourceBinarySha256: 'b'.repeat(64), originalPromptSha256: 'c'.repeat(64)},
    harness: {tool: 'codex', model: 'formal-ai', passed: true, solveExitCode: 0,
      harness: [{ok: true}], evaluation: {results: [{ok: true}]}, verdict: {harness: true, model: true},
      pullRequestUrl: 'https://github.com/a/b/pull/42'},
    pullRequest: {number: 42, state: 'open', draft: false,
      base: {repo: {full_name: 'a/b'}, ref: 'e2e/hello-world/synthetic'},
      head: {sha: 'd'.repeat(40), ref: 'issue-42-synthetic'}}, files: [{filename: 'main.rs'}],
    expectedStdout: 'Hello, World!\n', independentProgram: {exitCode: 0, signal: null,
      stdoutBase64: Buffer.from('Hello, World!\n').toString('base64'), stderrBase64: ''}};
}
test('pure emulation reuses the unchanged real E2E oracles, with synthetic fixtures labelled honestly', () => {
  assert.equal(replayOriginalDelivery(syntheticOracleFixture()).passed, true);
  for (const change of [value => { value.independentProgram.exitCode = 7; },
    value => { value.independentProgram.stdoutBase64 = Buffer.from('Hello, World!\n\n').toString('base64'); },
    value => { value.harness.harness[0].ok = false; }, value => { value.pullRequest.draft = true; },
    value => { value.expectedStdout = 'Changed oracle\n'; },
    value => { delete value.provenance.originalPromptSha256; }, value => { value.complete = false; }]) {
    const value = syntheticOracleFixture(); change(value); assert.throws(() => replayOriginalDelivery(value));
  }
});
function sealedSynthetic() {
  const fixture = syntheticOracleFixture(), documents = Object.fromEntries(
    ['harness', 'pullRequest', 'files', 'independentProgram'].map(name => [name, JSON.stringify(fixture[name])]));
  documents.toolEvents = JSON.stringify([{kind: 'tool-result', account: 'private-account', token: 'private-token',
    output: 'Error EINVAL --command'}]);
  const expected = {...fixture.provenance, run: 42, job: 7};
  const manifest = {schema: 'sealed-public-e2e-replay/v1', complete: true,
    workload: 'public-generated-hello-world', outcome: 'success', tool: 'codex', ...expected,
    members: Object.entries(documents).map(([name, text]) => ({name, bytes: Buffer.byteLength(text), sha256: digest(text)}))};
  return {manifest, documents, expected};
}
test('sealed public evidence becomes sanitized delivery replay with source digests and explicit semantic gaps', () => {
  const result = collectReplayFixture(sealedSynthetic());
  assert.equal(result.originalOracle.passed, true);
  assert.equal(result.toolTraceSemanticEquivalence, 'unverified');
  assert.equal(result.provenance.sourceMembers.length, 5);
  assert.doesNotMatch(JSON.stringify(result), /private-account|private-token/u);
});
test('missing, changed, unbound and unproved source documents never promote to replay', () => {
  for (const change of [value => { value.documents.toolEvents += ' '; },
    value => { value.expected.sourceHead = 'e'.repeat(40); },
    value => { value.manifest.complete = false; }, value => { delete value.documents.files; },
    value => { value.manifest.members[0].bytes = Infinity; }]) {
    const value = sealedSynthetic(); change(value); assert.throws(() => collectReplayFixture(value));
  }
});
test('failed original attempts remain collected rather than disappearing behind successful fingerprints', () => {
  const value = sealedSynthetic(); value.manifest.outcome = 'failure';
  const harness = JSON.parse(value.documents.harness); harness.solveExitCode = 7;
  value.documents.harness = JSON.stringify(harness);
  Object.assign(value.manifest.members.find(row => row.name === 'harness'),
    {bytes: Buffer.byteLength(value.documents.harness), sha256: digest(value.documents.harness)});
  const result = collectReplayFixture(value);
  assert.equal(result.observedOutcome, 'failure'); assert.equal(result.originalOracle.passed, false);
  assert.match(result.originalOracle.errorFingerprint, /^[a-f0-9]{64}$/u);
});

test('provider metadata spelling variants and recursively encoded credentials cannot leak', () => {
  const privateFields = {account_id: 'private-a', accountId: 'private-b', user_id: 'private-c',
    userId: 'private-d', actorId: 'private-e', displayName: 'Private Display', full_name: 'Private Full',
    API_key: 'private-f', access_token: 'private-g', Cookie: 'private-h', private_key: 'private-i'};
  const value = sanitizeEvidence({response: JSON.stringify({nested: JSON.stringify(privateFields)}),
    encodedAgain: JSON.stringify(JSON.stringify(privateFields)),
    call_id: 'call-public-17', sourceHash: 'a'.repeat(64), repositoryUrl: 'https://github.com/a/b'});
  assert.doesNotMatch(JSON.stringify(value), /private-[a-i]|Private Display|Private Full/u);
  assert.equal(value.call_id, 'call-public-17'); assert.equal(value.sourceHash, 'a'.repeat(64));
  assert.equal(value.repositoryUrl, 'https://github.com/a/b');
});
test('personal display records lose identity while task and source identifiers survive', () => {
  const value = sanitizeEvidence({record: {userId: 'private-user', id: 'private-provider-id',
    name: 'Private Person', call_id: 'call-19', source_sha256: 'b'.repeat(64)},
    headers: {'Set-Cookie': 'private-cookie', Authorization: 'private-auth'}});
  assert.doesNotMatch(JSON.stringify(value), /private-|Private Person/u);
  assert.equal(value.record.call_id, 'call-19'); assert.equal(value.record.source_sha256, 'b'.repeat(64));
});
test('sanitization preserves the exact original delivery oracle and public repository identity', () => {
  const original = syntheticOracleFixture();
  const sanitized = sanitizeEvidence(original);
  assert.deepEqual(sanitized, original);
  assert.equal(sanitized.pullRequest.base.repo.full_name, 'a/b');
  assert.equal(replayOriginalDelivery(sanitized).passed, true);
  original.independentProgram.exitCode = 7;
  assert.throws(() => replayOriginalDelivery(sanitizeEvidence(original)));
});

test('typed float and large integer values round-trip without losing numeric identity', () => {
  const numeric = '{"large":9007199254740993,"fraction":1.25,"integerFloat":1.0,"negativeZero":-0.0}';
  const expected = parseJson(numeric);
  const observed = parseJson(numeric.slice(0, -1) + ',"access_token":"private-numeric-token"}');
  const sanitized = sanitizeEvidence(observed);
  assert.equal(typeof sanitized.large, 'bigint'); assert.equal(sanitized.large, 9007199254740993n);
  assert.equal(isFloat(sanitized.fraction), true); assert.strictEqual(sanitized.fraction, observed.fraction);
  assert.equal(isFloat(sanitized.integerFloat), true);
  assert.strictEqual(sanitized.integerFloat, observed.integerFloat);
  assert.equal(toCompactJson(sanitized), toCompactJson(expected));
  const encoded = sanitizeEvidence(JSON.stringify(numeric.slice(0, -1)
    + ',"accountId":"private-num-account","access_token":"private-num-token"}'));
  assert.doesNotMatch(encoded, /private-num/u);
  assert.equal(toCompactJson(parseJson(parseJson(encoded))), toCompactJson(expected));
});
