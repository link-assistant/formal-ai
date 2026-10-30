import test from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {createHash} from 'node:crypto';
import {createReleaseWithRecovery} from '../../../scripts/create-release-with-recovery.mjs';
import {verifySourceDescriptor} from '../../../scripts/release-stage-transfer.mjs';

const environment = {GITHUB_REPOSITORY: 'owner/project', RELEASE_VERSION: '0.355.24',
  EXPECT_SOURCE: 'a'.repeat(40), GH_TOKEN: 'fixture-secret'};
const result = (status, stdout = '', stderr = '') => ({status, stdout: Buffer.from(stdout),
  stderr: Buffer.from(stderr), signal: null, timed_out: false, cleanup_complete: true});
const lostResponse = result(1, '', 'Error creating release: unexpected end of JSON input\n');
const absent = result(1, 'HTTP/2.0 404 Not Found\r\n\r\n{"message":"Not Found","status":"404"}', 'gh: Not Found (HTTP 404)');
const release = {id: 123, tag_name: 'v0.355.24', draft: false, prerelease: false,
  html_url: 'https://github.com/owner/project/releases/tag/v0.355.24'};
const present = record => result(0, 'HTTP/2.0 200 OK\ncontent-type: application/json\n\n' + JSON.stringify(record));
function scenario(responses, selectedEnvironment = environment) {
  const calls = [], pauses = [], reports = [];
  return {calls, pauses, reports, run: () => createReleaseWithRecovery(['--release-version', '0.355.24'], {
    environment: selectedEnvironment,
    transport: async (command, args, options) => {
      assert(options.timeout > 0 && options.timeout <= 180000);
      assert.equal(options.stdoutLimitBytes, 1048576); assert.equal(options.stderrLimitBytes, 1048576);
      assert.equal(options.env, selectedEnvironment);
      assert(!args.some(arg => arg.includes(environment.GH_TOKEN)));
      calls.push({command, args});
      assert(responses.length > 0, 'unexpected extra action');
      return responses.shift();
    }, pause: async value => pauses.push(value), report: value => reports.push(value),
  })};
}

test('ordinary success invokes the unchanged Rust publisher once without probing or retrying', async () => {
  const fixture = scenario([result(0, 'Created GitHub release: v0.355.24')]);
  assert.deepEqual(await fixture.run(), {complete: true, attempt: 1, reconciled: false});
  assert.deepEqual(fixture.calls.map(call => call.command), ['rust-script']);
  assert.equal(fixture.calls[0].args[0], 'scripts/create-github-release.rs');
  assert.deepEqual(fixture.pauses, []);
});
test('captured truncated JSON failure retries only after exact authenticated absence', async () => {
  const fixture = scenario([lostResponse, absent, result(0)]);
  assert.deepEqual(await fixture.run(), {complete: true, attempt: 2, reconciled: false});
  assert.deepEqual(fixture.calls.map(call => call.command), ['rust-script', 'gh', 'rust-script']);
  assert.deepEqual(fixture.calls[1].args, ['api', '--include', 'repos/owner/project/releases/tags/v0.355.24']);
  assert.deepEqual(fixture.calls[0].args, fixture.calls[2].args);
  assert.deepEqual(fixture.pauses, [2000]);
});
test('lost response after a committed release reconciles tag and source without a second write', async () => {
  const fixture = scenario([lostResponse, present(release), result(0, JSON.stringify({sha: environment.EXPECT_SOURCE}))]);
  assert.deepEqual(await fixture.run(), {complete: true, attempt: 1, reconciled: true});
  assert.equal(fixture.calls.filter(call => call.command === 'rust-script').length, 1);
  assert.deepEqual(fixture.calls.at(-1).args, ['api', 'repos/owner/project/commits/v0.355.24']);
});
test('repeated captured failures exhaust exactly three writes and preserve all observations', async () => {
  const fixture = scenario([lostResponse, absent, lostResponse, absent, lostResponse, absent]);
  await assert.rejects(fixture.run(), /exhausted three attempts/u);
  assert.equal(fixture.calls.length, 6); assert.deepEqual(fixture.pauses, [2000, 4000]);
  assert.equal(fixture.reports.length, 6);
});
test('permission, validation, process and cleanup failures do not start another action', async () => {
  for (const failure of [result(1, '', 'Error creating release: HTTP 403 Forbidden'),
    result(1, '', 'Error creating release: HTTP 422 Validation Failed'),
    {...lostResponse, error: {code: 'ETIMEDOUT'}}, {...lostResponse, timed_out: true},
    {...lostResponse, cleanup_complete: false}, {...lostResponse, signal: 'SIGKILL'}]) {
    const fixture = scenario([failure]);
    await assert.rejects(fixture.run(), /without retryable transport evidence/u);
    assert.equal(fixture.calls.length, 1);
  }
});
test('unknown absence, malformed responses and uncertain read cleanup fail closed', async () => {
  for (const observation of [result(1, '', 'gh: Not Found'),
    result(1, 'HTTP/2.0 403 Forbidden\n\n{"status":"403"}'),
    result(1, 'HTTP/2.0 404 Not Found\n\n{"message":"Not Found"}'),
    result(0, 'HTTP/2.0 200 OK\n\nnot-json'), {...absent, cleanup_complete: false}]) {
    const fixture = scenario([lostResponse, observation]);
    await assert.rejects(fixture.run()); assert.equal(fixture.calls.length, 2);
  }
});
test('wrong tag, draft, prerelease, repository or source never earns reconciliation credit', async () => {
  for (const change of [{tag_name: 'v0.355.23'}, {draft: true}, {prerelease: true},
    {html_url: 'https://github.com/foreign/project/releases/tag/v0.355.24'}, {id: 0}]) {
    const fixture = scenario([lostResponse, present({...release, ...change})]);
    await assert.rejects(fixture.run(), /identity or publication state/u);
    assert.equal(fixture.calls.length, 2);
  }
  const fixture = scenario([lostResponse, present(release), result(0, JSON.stringify({sha: 'b'.repeat(40)}))]);
  await assert.rejects(fixture.run(), /source differs/u); assert.equal(fixture.calls.length, 3);
});
test('diagnostics retain command results and redact the selected credential', async () => {
  const fixture = scenario([result(1, '', 'HTTP 403 fixture-secret')]);
  await assert.rejects(fixture.run(), error => !error.message.includes(environment.GH_TOKEN));
  assert(!JSON.stringify(fixture.reports).includes(environment.GH_TOKEN));
  assert.match(fixture.reports[0].stderr, /HTTP 403 \*\*\*/u);
});
test('legacy source fallback authenticates Git status and the shared wrapper selects recovery', async () => {
  const legacy = {...environment}; delete legacy.EXPECT_SOURCE;
  const fixture = scenario([result(0, 'a'.repeat(40) + '\n'), result(0)], legacy);
  assert.equal((await fixture.run()).complete, true); assert.equal(fixture.calls[0].command, 'git');
  await assert.rejects(scenario([result(1, 'a'.repeat(40))], legacy).run(), /source could not be authenticated/u);
  const wrapper = readFileSync(new URL('../../../scripts/invoke-create-github-release.sh', import.meta.url), 'utf8');
  assert.match(wrapper, /node scripts\/create-release-with-recovery.mjs "\$\{release_args\[@\]\}"/u);
});
test('captured failed-job rerun refuses source artifacts from a previous attempt', () => {
  const archive = Buffer.from('fixture source; no production archive'), compiler = 'fixture compiler';
  const hash = bytes => createHash('sha256').update(bytes).digest('hex');
  const identity = {repository: 'owner/project', run: '41045510144', attempt: '1',
    protocol: 'b'.repeat(40), source: environment.EXPECT_SOURCE, tree: 'c'.repeat(40),
    version: environment.RELEASE_VERSION, compilerSha256: hash(compiler)};
  const record = {schema: 'release-stage-source/v1', producerJob: 'auto_prepare-source',
    identity, archiveSha256: hash(archive), compiler};
  const expected = {...identity, producerJob: record.producerJob, archiveSha256: hash(archive)};
  verifySourceDescriptor(record, expected, archive, archive);
  assert.throws(() => verifySourceDescriptor(record, {...expected, attempt: '2'}, archive, archive),
    /source independent identity differs attempt/u);
});
