import assert from 'node:assert/strict';
import {execFile} from 'node:child_process';
import {createServer} from 'node:http';
import {mkdtemp, readFile, rm} from 'node:fs/promises';
import {tmpdir} from 'node:os';
import path from 'node:path';
import test from 'node:test';
import {fileURLToPath} from 'node:url';
import {promisify} from 'node:util';

const execute = promisify(execFile);
const repository = fileURLToPath(new URL('../../', import.meta.url));

test('actual npm diagnoses the maintained mismatched lock with an empty cache before installation',
  {timeout: 20000}, async t => {
    const output = await mkdtemp(path.join(tmpdir(), 'formal-ai-secretlint-cache-control-'));
    t.after(() => rm(output, {recursive: true, force: true}));
    const lock = JSON.parse(await readFile(new URL('../../.github/secretlint/package-lock.json', import.meta.url)));
    const version = lock.packages['node_modules/secretlint'].version;
    const requests = [];
    const server = createServer((request, response) => {
      requests.push(request.url);
      if (request.url !== '/secretlint') {
        response.writeHead(404); response.end('Unexpected registry request'); return;
      }
      response.writeHead(200, {'content-type': 'application/json'});
      response.end(JSON.stringify({
        name: 'secretlint', 'dist-tags': {latest: version},
        versions: {[version]: {name: 'secretlint', version,
          dist: {tarball: registry + 'must-not-download.tgz'}}},
      }));
    });
    server.maxRequestsPerSocket = 10;
    await new Promise(resolve => server.listen(0, '127.0.0.1', resolve));
    t.after(() => new Promise(resolve => server.close(resolve)));
    const registry = 'http://127.0.0.1:' + server.address().port + '/';
    const environment = {...process.env};
    delete environment.NODE_OPTIONS;
    await execute('python3', ['experiments/issue-1580/secretlint-cold-cache-control.py', output, registry], {
      cwd: repository, env: environment, timeout: 15000, maxBuffer: 256 * 1024,
    });
    const result = JSON.parse(await readFile(path.join(output, 'control-result.json'), 'utf8'));
    const log = await readFile(path.join(output, 'mismatched-lock.log'), 'utf8');
    assert.equal(result.records.length, 1);
    assert.equal(result.records[0].label, 'mismatched-lock');
    assert.equal(result.cacheInitiallyAbsent, true);
    assert.equal(result.nodeModulesCreated, false);
    assert.equal(result.records[0].passed, true, log);
    assert.match(log, /EUSAGE/);
    assert.doesNotMatch(log, /ENOTCACHED/);
    assert.ok(requests.includes('/secretlint'), JSON.stringify(requests));
    assert.ok(requests.every(request => request === '/secretlint'), JSON.stringify(requests));
  });
