// Replay the template's two reset phases through the real Node fetch transport.
import assert from 'node:assert/strict';
import {once} from 'node:events';
import net from 'node:net';
import test from 'node:test';
import {recheckAll} from '../../scripts/recheck-broken-links.mjs';

async function observeReset({phase, recover = false}) {
  const sockets = new Set();
  const observations = [];
  const server = net.createServer(socket => {
    sockets.add(socket);
    socket.on('error', () => {});
    socket.on('close', () => sockets.delete(socket));
    socket.setTimeout(1000, () => socket.destroy());
    if (phase === 'before TLS request') {
      observations.push({phase, request: null});
      socket.resetAndDestroy();
      return;
    }
    let request = '';
    socket.on('data', chunk => {
      request += chunk.toString('latin1');
      // This finite fixture never accepts an unbounded request header.
      if (request.length > 8192) {
        socket.destroy();
        return;
      }
      const end = request.indexOf('\r\n');
      if (end < 0) return;
      socket.removeAllListeners('data');
      observations.push({phase, request: request.slice(0, end)});
      if (recover && observations.length > 1) {
        socket.end('HTTP/1.1 200 OK\r\nContent-Length: 0\r\nConnection: close\r\n\r\n');
      } else {
        socket.resetAndDestroy();
      }
    });
  });
  server.maxConnections = 12;
  server.listen(0, '127.0.0.1');
  await once(server, 'listening');
  const protocol = phase === 'before TLS request' ? 'https' : 'http';
  const url = protocol + '://127.0.0.1:' + server.address().port + '/reset';
  try {
    const results = await recheckAll([url], {
      attempts: 3, waitMs: 1, timeoutMs: 500, budgetMs: 3000,
    });
    return {result: results.get(url), observations};
  } finally {
    for (const socket of sockets) socket.destroy();
    await new Promise(resolve => server.close(resolve));
  }
}

for (const phase of ['before TLS request', 'after HTTP request']) {
  test('real connection reset ' + phase + ' exhausts the finite retry allowance', {timeout: 5000}, async () => {
    const {result, observations} = await observeReset({phase});
    assert.equal(result.outcome, 'unreachable');
    assert.equal(result.attempts, 3);
    assert.equal(observations.length, 3, 'one physical connection for each actual probe');
    for (const observation of observations) {
      assert.equal(observation.phase, phase);
      if (phase === 'after HTTP request') assert.equal(observation.request, 'GET /reset HTTP/1.1');
      else assert.equal(observation.request, null, 'the reset precedes an HTTP request');
    }
  });
}

test('a real post-request reset recovers only after a later HTTP success', {timeout: 5000}, async () => {
  const {result, observations} = await observeReset({phase: 'after HTTP request', recover: true});
  assert.equal(result.outcome, 'alive');
  assert.equal(result.detail, 'HTTP 200');
  assert.equal(result.attempts, 2);
  assert.deepEqual(observations.map(row => row.request), ['GET /reset HTTP/1.1', 'GET /reset HTTP/1.1']);
});
