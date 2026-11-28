// Reproduce the transport failure shared by Formal AI and the C++ template.
import test from 'node:test';
import assert from 'node:assert/strict';
import http from 'node:http';
import {execFile} from 'node:child_process';
import {promisify} from 'node:util';
const execute = promisify(execFile);

test('retry-all-errors retries a truncated successful HTTP response; retry alone stops', {timeout: 15000}, async () => {
  let requests = 0;
  const server = http.createServer((_, response) => {
    requests++;
    response.writeHead(200, {'Content-Length': requests === 1 ? '100' : '2', Connection: 'close'});
    response.end('ok');
  });
  await new Promise(resolve => server.listen(0, '127.0.0.1', resolve));
  const url = `http://127.0.0.1:${server.address().port}/archive`;
  const args = ['--fail', '--location', '--retry', '3', '--retry-delay', '0', '--max-time', '2', '--output', '/dev/null'];
  try {
    await assert.rejects(execute('curl', [...args, url], {timeout: 10000, maxBuffer: 16384}), error => error.code === 18);
    assert.equal(requests, 1, 'the template stops after the first truncated transfer');
    requests = 0;
    await execute('curl', [...args, '--retry-all-errors', url], {timeout: 10000, maxBuffer: 16384});
    assert.equal(requests, 2, 'one bounded retry reaches the complete response');
  } finally {
    server.closeAllConnections();
    await new Promise(resolve => server.close(resolve));
  }
});
