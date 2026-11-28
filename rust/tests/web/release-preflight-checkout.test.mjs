import {readCanonicalSource} from '../../../scripts/canonical-release-source.mjs';
import { once } from 'node:events';
import { execFile } from 'node:child_process';
import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { createServer } from 'node:http';
import { promisify } from 'node:util';
import YAML from 'yaml';
const root=new URL('../../../',import.meta.url);
const evidence='experiments/formal_ai_subagent/evidence/specification-delivery-1188/dormant-staged-release/';
const read=file=>readFileSync(new URL(file,root),'utf8');
function validatePreflight(workflow){
 const job=workflow.jobs['release-preflight'],checkout=job.steps[0],probe=job.steps[1];
 assert.equal(job['timeout-minutes'],5);assert.equal(checkout['timeout-minutes'],2);
 assert.equal(checkout.uses,'actions/checkout@v7');
 assert.deepEqual(checkout.with,{'persist-credentials':false,'fetch-depth':1,filter:'blob:none',
  'sparse-checkout':'/scripts/preflight-credentials.sh','sparse-checkout-cone-mode':false});
 assert.equal(checkout.with.ref,undefined,'preserve actual triggering merge/event source');
 assert.deepEqual(checkout.env,{GIT_HTTP_LOW_SPEED_LIMIT:'1024',GIT_HTTP_LOW_SPEED_TIME:'20'});
 assert.equal(workflow.env.GIT_CONFIG_COUNT,'1');assert.equal(workflow.env.GIT_CONFIG_KEY_0,'init.defaultBranch');
 assert.equal(workflow.env.GIT_CONFIG_VALUE_0,'main');
 assert.equal(job.steps.length,2);assert.equal(probe.run,'bash scripts/preflight-credentials.sh');
 assert.equal(probe.if,undefined);assert.equal(job.if,undefined);assert.equal(probe['continue-on-error'],undefined);
 assert.deepEqual(job.permissions,{contents:'read',packages:'write'});
 return job;
}
test('canonical and deployed preflight retain the real credential gate with bounded sparse source checkout',()=>{
 const canonical=YAML.parse(readCanonicalSource(root).text);
 const expected=validatePreflight(canonical);
 for(const file of ['.github/workflows/release.yml',evidence+'candidate-release-caller.yml']){
  assert.deepEqual(validatePreflight(YAML.parse(read(file))),expected);
 }
 const script=read('scripts/preflight-credentials.sh');
 assert.ok(script.startsWith('#!/usr/bin/env bash\n'));
 assert.doesNotMatch(script,/^\s*(?:source|\.)\s+[^#\n]+/mu,'sparse closure must remain standalone');
});
test('broad checkout, lost transfer bounds, changed event source and bypassed probes are rejected',()=>{
 const original=YAML.parse(readCanonicalSource(root).text);
 const mutations=[
  x=>delete x.jobs['release-preflight'].steps[0].with['sparse-checkout'],
  x=>x.jobs['release-preflight'].steps[0].with['sparse-checkout']='/',
  x=>x.jobs['release-preflight'].steps[0].with['sparse-checkout-cone-mode']=true,
  x=>delete x.jobs['release-preflight'].steps[0].with.filter,
  x=>x.jobs['release-preflight'].steps[0].with.ref='main',
  x=>x.jobs['release-preflight'].steps[0].with['persist-credentials']=true,
  x=>delete x.jobs['release-preflight'].steps[0].env.GIT_HTTP_LOW_SPEED_TIME,
  x=>x.jobs['release-preflight'].steps[0].env.GIT_HTTP_LOW_SPEED_LIMIT='0',
  x=>x.jobs['release-preflight'].steps[0]['timeout-minutes']=5,
  x=>x.jobs['release-preflight'].steps[1].if='false',
  x=>x.jobs['release-preflight'].steps[1].run='true',
  x=>x.jobs['release-preflight']['timeout-minutes']=6
 ];
 for(const mutate of mutations){const changed=structuredClone(original);mutate(changed);assert.throws(()=>validatePreflight(changed));}
});

// Issue #1639: the crates.io check is a no-archive publish probe (PUT /crates/new),
// not the public crate lookup that could only ever say `unknown`.
test('a stalled crates.io publish probe is retried and all credential verdicts are reported', { timeout: 15_000 }, async (t) => {
  const requests = [];
  const server = createServer((request, response) => {
    const route = `${request.method} ${request.url.split('?')[0]}`;
    requests.push(route);
    if (route === 'PUT /crates/new' && requests.length === 1) {
      const timer = setTimeout(() => response.end('{}'), 7000);
      response.once('close', () => clearTimeout(timer));
      return;
    }
    const replies = {
      'PUT /crates/new': [503],
      'GET /ghcr-token': [200, {}, '{"token":"fixture-bearer"}'],
      'POST /v2/test/ghcr/blobs/uploads/': [202, { location: '/ghcr-upload' }],
      'DELETE /ghcr-upload': [204],
      'GET /docker-token': [200, {}, '{"token":"fixture-bearer"}'],
      'POST /v2/test/docker/blobs/uploads/': [403],
    };
    const [status, headers, body] = replies[route] ?? [404];
    response.writeHead(status, headers).end(body);
  });
  server.listen(0, '127.0.0.1');
  t.after(() => new Promise((resolve) => {
    server.close(resolve);
    server.closeAllConnections();
  }));
  await once(server, 'listening');
  const endpoint = `http://127.0.0.1:${server.address().port}`;
  await assert.rejects(promisify(execFile)('bash', ['scripts/preflight-credentials.sh', '--mode', 'release'], {
    cwd: root,
    timeout: 12_000,
    env: {
      PATH: process.env.PATH,
      CARGO_REGISTRY_TOKEN: 'fixture-crate-token',
      GHCR_IMAGE: 'ghcr.io/test/ghcr',
      GITHUB_TOKEN: 'fixture-github-token',
      DOCKERHUB_IMAGE: 'test/docker',
      DOCKERHUB_USERNAME: 'fixture-user',
      DOCKERHUB_TOKEN: 'fixture-docker-token',
      CRATES_IO_API: endpoint,
      GHCR_REGISTRY: endpoint,
      GHCR_TOKEN_ENDPOINT: `${endpoint}/ghcr-token`,
      DOCKERHUB_REGISTRY: endpoint,
      DOCKERHUB_TOKEN_ENDPOINT: `${endpoint}/docker-token`,
      PREFLIGHT_RETRIES: '2',
      PREFLIGHT_RETRY_DELAY: '0',
    },
  }), (error) => {
    assert.equal(error.code, 1);
    assert.match(error.stdout, /Release preflight \(release mode\): 1 verified, 1 failed, 1 unknown/);
    assert.match(error.stdout, /UNKNOWN: crates\.io publish token -- the no-archive publish probe answered HTTP 503/);
    assert.match(error.stdout, /OK: GHCR push -- .*opened a blob upload session/);
    assert.match(error.stdout, /BLOCKED: Docker Hub push -- .*HTTP 403/);
    return true;
  });
  assert.deepEqual(requests, [
    'PUT /crates/new', 'PUT /crates/new',
    'GET /ghcr-token', 'POST /v2/test/ghcr/blobs/uploads/', 'DELETE /ghcr-upload',
    'GET /docker-token', 'POST /v2/test/docker/blobs/uploads/',
  ]);
});
