import assert from 'node:assert/strict';
import test from 'node:test';
import http from 'node:http';
import {once} from 'node:events';
import {spawnSync} from 'node:child_process';
import {mkdtempSync, readFileSync, rmSync, writeFileSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import {fileURLToPath} from 'node:url';
import {classifyFailures, isAccepted, parseAcceptList, recheckAll} from '../../scripts/recheck-broken-links.mjs';

const retryable = [408, 429, 500, 503, 599];

test('the deployed link checker requires a successful response rather than a timeout or server failure', () => {
  const workflow = readFileSync(new URL('../../.github/workflows/links.yml', import.meta.url), 'utf8');
  const spec = /^\s*--accept\s+'([^']+)'\s*$/mu.exec(workflow)?.[1];
  assert.ok(spec, 'read the deployed lychee acceptance argument');
  const ranges = parseAcceptList(spec);
  for (const status of [200, 201, 204]) assert.equal(isAccepted(status, ranges), true);
  for (const status of [...retryable, 404, 410]) assert.equal(isAccepted(status, ranges), false, String(status));
  assert.doesNotMatch(workflow, /^\s*--accept-timeouts\s*$/mu);
  assert.equal([...workflow.matchAll(/LYCHEE_EXIT_CODE: \$\{\{ steps\.lychee\.outputs\.exit_code \}\}/gu)].length, 2);
  assert.equal([...workflow.matchAll(/steps\.lychee\.outputs\.exit_code == '2'/gu)].length, 2);
  assert.match(workflow, /steps\.lychee\.outputs\.exit_code != '2' \|\| steps\.recheck\.outputs\.all_recovered != 'true'/u);
});

test('only transient HTTP verdicts join transport errors in the bounded retry queue', () => {
  const failures = [...retryable, 400, 401, 403, 404, 410].map(status => ({
    url: `https://response.invalid/${status}`, marker: String(status), detail: `Rejected status code: ${status}`,
  }));
  const classified = classifyFailures(failures);
  assert.deepEqual(classified.unanswered.map(row => Number(row.marker)), retryable);
  assert.deepEqual(classified.answered.map(row => Number(row.marker)), [400, 401, 403, 404, 410]);
  assert.equal(classifyFailures([{url: 'https://response.invalid/', marker: '404', detail: 'Rejected status code: 408'}]).answered.length, 1);
});

for (const status of retryable) {
  test(`real HTTP ${status} requires later success and cannot exhaust retries as healthy`, {timeout: 10000}, async () => {
    const calls = new Map();
    const server = http.createServer((request, response) => {
      const count = (calls.get(request.url) ?? 0) + 1;
      calls.set(request.url, count);
      response.writeHead(request.url === '/recover' && count > 1 ? 200 :
        request.url === '/missing' ? 404 : request.url === '/gone' ? 410 : status);
      response.end();
    });
    server.listen(0, '127.0.0.1');
    await once(server, 'listening');
    try {
      const base = `http://127.0.0.1:${server.address().port}`;
      const results = await recheckAll(['/recover', '/persist', '/missing', '/gone'].map(path => base + path), {waitMs: 1});
      assert.deepEqual([results.get(base + '/recover').outcome, results.get(base + '/recover').attempts], ['alive', 2]);
      assert.deepEqual([results.get(base + '/persist').outcome, results.get(base + '/persist').attempts], ['retryable', 3]);
      for (const path of ['/missing', '/gone']) {
        assert.deepEqual([results.get(base + path).outcome, results.get(base + path).attempts], ['rejected', 1]);
      }
      assert.deepEqual(Object.fromEntries(calls), {'/recover': 2, '/persist': 3, '/missing': 1, '/gone': 1});
    } finally {
      server.closeAllConnections();
      await new Promise(resolve => server.close(resolve));
    }
  });
}

const archiveScript = fileURLToPath(new URL('../../scripts/check-web-archive.mjs', import.meta.url));
const report = '## Errors per input\n\n* [404] <https://missing.invalid/document> | Rejected status code: 404\n';
const archiveCases = [
  {name: 'confirmed snapshot', mode: 'archived', content: report, code: 0, archived: 'true', fetches: 1},
  {name: 'empty report', mode: 'absent', content: '', code: 1, archived: 'false', fetches: 0},
  {name: 'missing failure entries', mode: 'absent', content: '# Summary\n\n| Errors | 1 |\n', code: 1, archived: 'false', fetches: 0},
  {name: 'incomplete report', mode: 'archived', content: '| Errors | 2 |\n' + report, code: 1, archived: 'false', fetches: 0},
  {name: 'malformed failure entry', mode: 'absent', content: '## Errors per input\n\n* [404] | Missing target\n', code: 1, archived: 'false', fetches: 0},
  {name: 'checker runtime failure', mode: 'archived', content: report, exit: '1', code: 1, archived: 'false', fetches: 0},
  {name: 'checker-confirmed link failure', mode: 'archived', content: report, exit: '2', code: 0, archived: 'true', fetches: 1},
  {name: 'failed checker without failures', mode: 'absent', content: '| Errors | 0 |\n', exit: '2', code: 1, archived: 'false', fetches: 0},
  {name: 'zero-failure summary', mode: 'absent', content: '# Summary\n\n| Errors | 0 |\n| Timeouts | 0 |\n', code: 0, archived: 'true', fetches: 0},
  {name: 'novel local failure marker', mode: 'absent', content: '## Novel outcome per input\n\n* [NOVEL] <file:///missing.md> | Unexpected failure\n', code: 1, archived: 'false', fetches: 0},
  {name: 'confirmed absence', mode: 'absent', content: report, code: 1, archived: 'false', fetches: 1},
  {name: 'Wayback outage', mode: 'outage', content: report, code: 0, archived: 'false', fetches: 1, unknown: '1'},
  {name: 'unreadable Wayback response', mode: 'unreadable', content: report, code: 0, archived: 'false', fetches: 1, unknown: '1'},
  {name: 'missing report', mode: 'absent', content: null, code: 1, archived: 'false', fetches: 0},
  {name: 'failed local link', mode: 'absent', content: '## Errors per input\n\n* [ERROR] <file:///missing.md> | File not found\n', code: 1, archived: 'false', fetches: 0},
  {name: 'healthy report', mode: 'absent', content: '## Successes per input\n\n* [200] <https://working.invalid/document>\n', code: 0, archived: 'true', fetches: 0},
  {name: 'failed relative link', mode: 'absent', content: '## Errors per input\n\n* [ERROR] ./missing.md | File not found\n', code: 1, archived: 'false', fetches: 0},
  {name: 'failed local HTTP-style status', mode: 'absent', content: '## Errors per input\n\n* [404] <file:///missing.md> | File not found\n', code: 1, archived: 'false', fetches: 0},
  {name: 'excluded local failure marker', mode: 'absent', content: '## Excluded per input\n\n* [ERROR] <file:///ignored.md> | Excluded\n', code: 0, archived: 'true', fetches: 0},
  {name: 'mixed archived HTTP and failed local links', mode: 'archived', content: report + '* [ERROR] <file:///missing.md> | File not found\n', code: 1, archived: 'false', fetches: 1},
];

for (const fixture of archiveCases) {
  test(`actual archive CLI preserves the ${fixture.name} observation`, {timeout: 10000}, () => {
    const directory = mkdtempSync(join(tmpdir(), 'formal-link-observation-'));
    try {
      const source = join(directory, 'report.md');
      const output = join(directory, 'outputs');
      const calls = join(directory, 'calls');
      const preload = join(directory, 'fetch-control.mjs');
      if (fixture.content !== null) writeFileSync(source, fixture.content);
      writeFileSync(output, '');
      writeFileSync(calls, '');
      writeFileSync(preload, `import {appendFileSync} from 'node:fs';
globalThis.fetch = async url => {
  if (!String(url).startsWith('https://archive.org/wayback/available?url=')) throw new Error('Unexpected request');
  appendFileSync(process.env.LINK_CONTROL_CALLS, 'request\\n');
  const mode = process.env.LINK_CONTROL_MODE;
  return {ok: mode !== 'outage', status: mode === 'outage' ? 503 : 200, json: async () => {
    if (mode === 'unreadable') throw new Error('Invalid JSON');
    return mode === 'archived' ? {archived_snapshots: {closest: {available: true, timestamp: '20260101000000', url: 'https://web.archive.org/web/20260101000000/https://missing.invalid/document'}}} : {archived_snapshots: {}};
  }};
};
`);
      const env = {...process.env, NODE_OPTIONS: '--max-old-space-size=64',
        LYCHEE_OUTPUT: source, GITHUB_OUTPUT: output, RECOVERED_URLS: join(directory, 'not-recovered'),
        LINK_CONTROL_MODE: fixture.mode, LINK_CONTROL_CALLS: calls};
      delete env.LYCHEE_EXIT_CODE;
      if (fixture.exit !== undefined) env.LYCHEE_EXIT_CODE = fixture.exit;
      const result = spawnSync(process.execPath, ['--import', preload, archiveScript], {
        cwd: directory, env, encoding: 'utf8', maxBuffer: 65536, timeout: 5000,
      });
      assert.ifError(result.error);
      assert.equal(result.status, fixture.code, result.stdout + result.stderr);
      const outputs = Object.fromEntries(readFileSync(output, 'utf8').trim().split('\n').filter(Boolean).map(line => line.split('=')));
      assert.equal(outputs.all_archived, fixture.archived, result.stdout + result.stderr);
      if (fixture.unknown) assert.equal(outputs.undetermined, fixture.unknown);
      assert.equal(readFileSync(calls, 'utf8').split('\n').filter(Boolean).length, fixture.fetches);
    } finally {
      rmSync(directory, {recursive: true, force: true});
    }
  });
}

const recheckScript = fileURLToPath(new URL('../../scripts/recheck-broken-links.mjs', import.meta.url));
const transportReport = '## Errors per input\n\n* [ERROR] <https://response.invalid/recover> | Connection reset\n';
const recoveryCases = [
  {name: 'HTTP-only recovery', content: transportReport, recovered: '1', remaining: '0', all: 'true', fetches: 1},
  {name: 'local failure beside recovered HTTP', content: transportReport + '* [ERROR] <file:///missing.md> | File not found\n', recovered: '1', remaining: '1', all: 'false', fetches: 1},
  {name: 'runtime failure', content: transportReport, exit: '1', recovered: '0', remaining: '0', all: 'false', fetches: 0},
  {name: 'empty report', content: '', recovered: '0', remaining: '0', all: 'false', fetches: 0},
  {name: 'incomplete report', content: '| Errors | 2 |\n' + transportReport, recovered: '0', remaining: '0', all: 'false', fetches: 0},
  {name: 'failed checker without failures', content: '| Errors | 0 |\n', exit: '2', recovered: '0', remaining: '0', all: 'false', fetches: 0},
];
for (const fixture of recoveryCases) {
  test(`actual re-check CLI preserves ${fixture.name}`, {timeout: 10000}, () => {
    const directory = mkdtempSync(join(tmpdir(), 'formal-recovery-observation-'));
    try {
      const source = join(directory, 'report.md');
      const output = join(directory, 'outputs');
      const calls = join(directory, 'calls');
      const recovered = join(directory, 'recovered');
      const preload = join(directory, 'fetch-control.mjs');
      writeFileSync(source, fixture.content);
      writeFileSync(output, '');
      writeFileSync(calls, '');
      writeFileSync(preload, `import {appendFileSync} from 'node:fs';
globalThis.fetch = async url => {
  if (url !== 'https://response.invalid/recover') throw new Error('Unexpected request');
  appendFileSync(process.env.LINK_CONTROL_CALLS, 'request\\n');
  return {status: 200};
};
`);
      const env = {...process.env, NODE_OPTIONS: '--max-old-space-size=64', LYCHEE_OUTPUT: source,
        GITHUB_OUTPUT: output, RECOVERED_OUTPUT: recovered, LINK_CONTROL_CALLS: calls};
      delete env.LYCHEE_EXIT_CODE;
      if (fixture.exit !== undefined) env.LYCHEE_EXIT_CODE = fixture.exit;
      const result = spawnSync(process.execPath, ['--import', preload, recheckScript], {
        cwd: directory, env, encoding: 'utf8', maxBuffer: 65536, timeout: 5000,
      });
      assert.ifError(result.error);
      // Re-check observations use outputs; the workflow remains the failure authority.
      assert.equal(result.status, 0, result.stdout + result.stderr);
      const outputs = Object.fromEntries(readFileSync(output, 'utf8').trim().split('\n').filter(Boolean).map(line => line.split('=')));
      assert.equal(outputs.all_recovered, fixture.all, result.stdout + result.stderr);
      assert.equal(outputs.recovered, fixture.recovered);
      assert.equal(outputs.remaining, fixture.remaining);
      assert.equal(readFileSync(calls, 'utf8').split('\n').filter(Boolean).length, fixture.fetches);
    } finally {
      rmSync(directory, {recursive: true, force: true});
    }
  });
}
