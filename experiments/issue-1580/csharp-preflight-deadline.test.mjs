// Run against a disposable checkout of the pinned C# template.
import assert from 'node:assert/strict';
import { once } from 'node:events';
import { createServer } from 'node:http';
import { resolve } from 'node:path';
import test from 'node:test';
import { pathToFileURL } from 'node:url';

const checkout = process.env.CSHARP_TEMPLATE_ROOT;
assert.ok(checkout, 'provide the pinned template checkout in CSHARP_TEMPLATE_ROOT');
const root = resolve(checkout);
const { main } = await import(pathToFileURL(resolve(root, 'scripts/preflight-credentials.mjs')));
process.chdir(root);

// The production job has a five-minute deadline. Use a finite response to
// prove that the probe waits through a stall even when a later probe can run.
for (const stall of ['none', 'headers', 'body']) {
  test(`C# preflight reports NuGet visibility after ${stall === 'none' ? 'a healthy GitHub response' : `stalled GitHub ${stall}`}`, {
    timeout: 4000,
  }, async (t) => {
    let waitExpired = false;
    const requests = [];
    const server = createServer((request, response) => {
      requests.push(request.url);
      if (request.url !== '/repos/test/fixture') {
        response.writeHead(404).end();
        return;
      }
      if (stall === 'none') {
        response.end('{"full_name":"test/fixture"}');
        return;
      }
      if (stall === 'body') {
        response.writeHead(200);
        response.write('{"full_name":');
      }
      const timer = setTimeout(() => {
        waitExpired = true;
        if (stall === 'headers') response.writeHead(503);
        response.end(stall === 'body' ? '"test/fixture"}' : '{}');
      }, 900);
      response.once('close', () => clearTimeout(timer));
    });
    server.listen(0, '127.0.0.1');
    t.after(() => new Promise((done) => {
      server.close(done);
      server.closeAllConnections();
    }));
    await once(server, 'listening');
    const origin = `http://127.0.0.1:${server.address().port}`;
    Object.assign(process.env, {
      GITHUB_API_URL: origin,
      GITHUB_REPOSITORY: 'test/fixture',
      GITHUB_TOKEN: '',
      GITHUB_OUTPUT: '',
      GITHUB_STEP_SUMMARY: '',
      NUGET_INDEX_URL: `${origin}/nuget`,
      NUGET_API_KEY: '',
      NUGET_OIDC_OUTCOME: 'skipped',
      NUGET_OIDC_KEY: '',
      PREFLIGHT_MODE: 'report',
    });
    const decision = await main({
      configurationCheck: () => ({ status: 'ok', detail: 'fixture identity' }),
    });
    assert.equal(decision.passed, true, 'report mode still reports rather than blocking');
    assert.equal(requests[0], '/repos/test/fixture');
    assert.match(requests[1], /^\/nuget\/[^/]+\/index\.json$/);
    assert.equal(requests.length, 2);
    assert.equal(waitExpired, false, 'the GitHub probe should time out before the stall ends');
  });
}
