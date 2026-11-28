// Supplied probe: execute the exact pinned helpers against a finite mock API.
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {readFileSync} from 'node:fs';
import {pathToFileURL} from 'node:url';

export async function observePagesAdoption(language, {artifactBound = false, acceptedBeforeDisconnect = false} = {}) {
  const root = new URL(`./fixtures/pages-templates/${language}/`, import.meta.url);
  const manifest = JSON.parse(readFileSync(new URL('./fixtures/pages-templates/source.json', import.meta.url)));
  const source = manifest.find(row => row.language === language);
  assert(source, language);
  for (const item of source.files) {
    const raw = readFileSync(new URL(item.path, root));
    assert.equal(createHash('sha1').update(`blob ${raw.length}\0`).update(raw).digest('hex'), item.gitBlob);
    assert.equal(createHash('sha256').update(raw).digest('hex'), item.sha256);
  }
  const module = await import(new URL(language === 'js' ? 'scripts/deploy-pages.mjs' : '.github/actions/deploy-pages/deploy.mjs', root));
  const sha = 'a'.repeat(40);
  const key = artifactBound ? createHash('sha256').update(JSON.stringify(['owner/repo', '42', sha])).digest('hex') : sha;
  const base = '/repos/owner/repo/pages';
  const deployments = new Map([[sha, {artifactId: 41, status: 'succeed'}]]);
  const calls = [];
  let posts = 0;
  const request = async (method, path, body) => {
    assert(calls.length < 20, 'bounded request count');
    calls.push({method, path, ...(body ? {body} : {})});
    if (method === 'POST' && path === `${base}/deployments`) {
      posts++;
      if (posts === 1) {
        if (acceptedBeforeDisconnect) deployments.set(body.pages_build_version, {artifactId: body.artifact_id, status: 'succeed'});
        throw Object.assign(new TypeError('fetch failed'), {cause: Object.assign(new Error('reset before response'), {code: 'ECONNRESET'})});
      }
      deployments.set(body.pages_build_version, {artifactId: body.artifact_id, status: 'succeed'});
      return {id: body.pages_build_version, page_url: 'https://owner.github.io/repo/'};
    }
    if (method === 'GET' && path === base) return {html_url: 'https://owner.github.io/repo/'};
    if (method === 'GET' && path.startsWith(`${base}/deployments/`)) {
      const value = deployments.get(path.slice(`${base}/deployments/`.length));
      if (!value) throw new module.HttpError(404, 'Not Found');
      return {status: value.status}; // GitHub's documented response contains no artifact ID.
    }
    throw new Error(`Unexpected request ${method} ${path}`);
  };
  let clock = 0;
  const result = await module.deployPages({request, getIdToken: async () => 'fixture-oidc', repository: 'owner/repo', artifactId: '42', buildVersion: key,
    sleep: async ms => {clock += ms;}, now: () => clock, log: () => {}});
  return {language, source: source.commit, artifactBound, acceptedBeforeDisconnect, requestedArtifactId: 42,
    observedArtifactId: deployments.get(String(result.id))?.artifactId, posts, result, calls};
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  const observations = [];
  for (const language of ['js', 'python']) for (const artifactBound of [false, true]) for (const acceptedBeforeDisconnect of [false, true])
    observations.push(await observePagesAdoption(language, {artifactBound, acceptedBeforeDisconnect}));
  console.log(JSON.stringify(observations, null, 2));
}
