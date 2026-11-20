// Finite offline signal probe; called in a 64 MiB child by the maintained test.
import {main} from '../../.github/actions/deploy-pages/deploy.mjs';
const signal = process.argv[2];
if (!['SIGINT', 'SIGTERM'].includes(signal)) throw new Error('Expected a termination signal');
globalThis.fetch = async (url, init) => {
  console.log(JSON.stringify({url: String(url), method: init?.method || 'GET'}));
  if (String(url).startsWith('https://oidc.invalid/')) return Response.json({value: 'fixture-oidc'});
  if (String(url).endsWith('/cancel')) return new Response(null, {status: 204});
  return Response.json({id: 5, page_url: 'https://site.invalid/'});
};
const guard = setTimeout(() => {throw new Error('Signal probe did not finish within two seconds');}, 2000);
await main({
  INPUT_TOKEN: 'fixture-token', INPUT_ARTIFACT_ID: '42', INPUT_SOURCE_SHA: 'a'.repeat(40),
  GITHUB_REPOSITORY: 'owner/repo', GITHUB_API_URL: 'https://api.invalid',
  ACTIONS_ID_TOKEN_REQUEST_URL: 'https://oidc.invalid/', ACTIONS_ID_TOKEN_REQUEST_TOKEN: 'fixture-request',
}, {sleep: () => new Promise(() => setImmediate(() => process.kill(process.pid, signal)))}).finally(() => clearTimeout(guard));
