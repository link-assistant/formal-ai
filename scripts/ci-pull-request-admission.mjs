// Fast, conservative PR admission. Production retains its full Rust classifier.
import assert from 'node:assert/strict';
import {appendFileSync, existsSync, readFileSync} from 'node:fs';
import {execFileSync} from 'node:child_process';
import {pathToFileURL} from 'node:url';
import {createHash} from 'node:crypto';
import {isDocumentationPath} from './lib/documentation-paths.mjs';

export const isDocumentation = isDocumentationPath;
// Classify conservatively: extra lightweight checks are cheaper than missed changes.
export function admission(paths) {
  assert(Array.isArray(paths) && paths.every(path => typeof path === 'string'));
  const relevant = paths.filter(path => !['experiments/', 'dev/log/', 'docs/case-studies/'].some(prefix => path.startsWith(prefix)));
  const code = relevant.some(path => !isDocumentation(path));
  return Object.fromEntries(['rs-changed', 'toml-changed', 'docs-changed', 'workflow-changed',
    'pipeline-changed', 'agentic-routing-changed', 'any-code-changed', 'js-changed',
    'ts-changed', 'rust-changed', 'release-pending'].map(key => [key,
      key === 'release-pending' ? false : key === 'docs-changed' ? relevant.some(isDocumentation) : code]));
}
export function versionIn(text) {
  const section = /^\[package\]\s*\n([\s\S]*?)(?=^\[|$(?![\s\S]))/mu.exec(text);
  const version = /^version\s*=\s*"([^"]+)"/mu.exec(section?.[1] ?? '');
  assert(version, 'readable package version required; refusing unknown manifest');
  return version[1];
}
export function checkVersions(head, base) {
  assert.equal(versionIn(head), versionIn(base), 'Use a changelog fragment instead of changing the crate version');
}
function git(args) {
  return execFileSync('git', args, {encoding: 'utf8', timeout: 15000, maxBuffer: 4 * 1024 * 1024});
}
export function changedPaths(event, before, invoke = git) {
  if (event === 'pull_request') {
    const parents = invoke(['rev-list', '--parents', '-n', '1', 'HEAD']).trim().split(/\s+/u);
    assert(parents.length === 3, 'PR admission requires the checked-out synthetic merge commit');
    return invoke(['diff', '--no-renames', '--name-only', '-z', parents[1], parents[2]]).split('\0').filter(Boolean);
  }
  // Bot dispatch checks the whole branch against the latest default branch.
  const base = event === 'workflow_dispatch' ? 'origin/main' : before;
  assert(base && !/^0+$/u.test(base), 'known comparison base required');
  return invoke(['diff', '--no-renames', '--name-only', '-z', base, 'HEAD']).split('\0').filter(Boolean);
}

const SOURCE_READ_LIMIT = 4 * 1024 * 1024;
const GIT_SHA = /^[a-f0-9]{40}$/u;
function sourceIdentity(value, repository, suffix) {
  const url = new URL(value);
  assert(url.origin === 'https://api.github.com' && !url.username && !url.password
    && !url.search && !url.hash
    && decodeURIComponent(url.pathname) === '/repos/' + repository + '/' + suffix,
  'base source API identity mismatch');
}
async function boundedGithubJson(fetcher, endpoint, token, signal, available) {
  available();
  let response;
  try {
    response = await fetcher('https://api.github.com/' + endpoint, {method:'GET', redirect:'error', signal,
      headers:{Accept:'application/vnd.github+json',Authorization:'Bearer '+token,
        'X-GitHub-Api-Version':'2022-11-28'}});
  } catch { throw new Error('base source API transport failed'); }
  available();
  assert(response.status === 200 && !response.redirected, 'base source API read denied or unavailable');
  assert(response.body && typeof response.body.getReader === 'function', 'base source body unavailable');
  const reader = response.body.getReader(), chunks = [];
  let bytes = 0;
  try {
    for (;;) {
      const chunk = await reader.read();
      available();
      if (chunk.done) break;
      assert(chunk.value instanceof Uint8Array, 'base source response bytes required');
      bytes += chunk.value.byteLength;
      assert(bytes <= SOURCE_READ_LIMIT, 'base source response exceeds byte bound');
      chunks.push(chunk.value);
    }
  } finally { reader.releaseLock(); }
  const text = new TextDecoder('utf-8',{fatal:true,ignoreBOM:true}).decode(Buffer.concat(chunks,bytes));
  return JSON.parse(text);
}
async function completeSourceTree(read, repository, sha) {
  assert(GIT_SHA.test(sha), 'base source tree SHA required');
  const value = await read('git/trees/' + sha);
  assert(value.sha === sha && value.truncated === false && Array.isArray(value.tree),
    'complete exact base source tree required');
  sourceIdentity(value.url,repository,'git/trees/'+sha);
  const names = new Set();
  for (const entry of value.tree) {
    assert(typeof entry.path === 'string' && entry.path.length > 0
      && !entry.path.includes('/') && !['.','..'].includes(entry.path)
      && !names.has(entry.path), 'unique nonrecursive source tree paths required');
    names.add(entry.path);
  }
  return value.tree;
}
async function baseManifestEntry(read, repository, tree) {
  const root = await completeSourceTree(read,repository,tree);
  const cargo = root.find(entry=>entry.path==='Cargo.toml');
  if (cargo !== undefined) return {entry:cargo,path:'Cargo.toml'};
  const rust = root.find(entry=>entry.path==='rust');
  assert(rust?.type==='tree' && rust.mode==='040000' && GIT_SHA.test(rust.sha),
    'base Cargo manifest missing');
  sourceIdentity(rust.url,repository,'git/trees/'+rust.sha);
  const entries = await completeSourceTree(read,repository,rust.sha);
  const entry = entries.find(value=>value.path==='Cargo.toml');
  assert(entry, 'base Cargo manifest missing');
  return {entry,path:'rust/Cargo.toml'};
}
async function sourceManifestBlob(read, repository, entry) {
  assert(entry.type==='blob' && ['100644','100755'].includes(entry.mode)
    && GIT_SHA.test(entry.sha), 'regular base Cargo manifest blob required');
  assert(Number.isSafeInteger(entry.size) && entry.size > 0 && entry.size <= SOURCE_READ_LIMIT,
    'bounded base Cargo manifest size required');
  sourceIdentity(entry.url,repository,'git/blobs/'+entry.sha);
  const value = await read('git/blobs/'+entry.sha);
  sourceIdentity(value.url,repository,'git/blobs/'+entry.sha);
  assert(value.sha===entry.sha && value.encoding==='base64' && value.size===entry.size
    && typeof value.content==='string', 'exact base Cargo blob required');
  const content=value.content.replace(/[\r\n]/gu,'');
  assert(/^[A-Za-z0-9+/]*={0,2}$/u.test(content), 'base Cargo blob encoding required');
  const bytes=Buffer.from(content,'base64');
  assert(bytes.toString('base64')===content && bytes.length===value.size,
    'canonical base Cargo blob encoding and size required');
  assert(createHash('sha1').update('blob '+bytes.length+'\0').update(bytes).digest('hex')===entry.sha,
    'base Cargo Git blob identity mismatch');
  return new TextDecoder('utf-8',{fatal:true,ignoreBOM:true}).decode(bytes);
}
/** Read one latest-base manifest through small immutable Git objects, never a Git fetch. */
export async function readBaseManifest({repository,base,token,fetcher=globalThis.fetch,now=Date.now,report=()=>{}}) {
  assert(typeof repository==='string' && /^[A-Za-z0-9_.-]+\/[A-Za-z0-9_.-]+$/u.test(repository)
    && repository.split('/').every(part=>!['.','..'].includes(part)), 'valid repository required');
  assert(typeof base==='string' && /^[a-zA-Z0-9_./-]+$/u.test(base) && !base.startsWith('-'),
    'valid base branch required');
  assert(typeof token==='string' && token.trim().length>0, 'explicit GitHub read token required');
  const started=now(), deadline=started+15000, controller=new AbortController();
  const observation={schema:'base-manifest-observation/v1',repository,base,phase:'ref',endpoint:null,
    status:null,baseCommit:null,path:null,blobSHA:null,requests:0,deadlineMilliseconds:15000};
  const summary=outcome=>({...observation,outcome,elapsedMs:Math.max(0,now()-started)});
  const available=()=>assert(Number.isFinite(now()) && now()<deadline, 'base source aggregate deadline exceeded');
  let timer;
  const expired=new Promise((_,reject)=>{
    timer=setTimeout(()=>{controller.abort();reject(new Error('base source aggregate deadline exceeded'));},15000);
  });
  const observe=async()=>{
    const read=suffix=>{
      observation.phase=suffix.split('/')[1];observation.endpoint='repos/'+repository+'/'+suffix;
      observation.status=null;
      const transport=async(url,options)=>{
        observation.requests++;
        const response=await fetcher(url,options);
        observation.status=Number.isInteger(response.status)?response.status:null;
        return response;
      };
      return boundedGithubJson(transport,observation.endpoint,token,controller.signal,available);
    };
    const ref=await read('git/ref/heads/'+encodeURIComponent(base));
    assert(ref.ref==='refs/heads/'+base && ref.object?.type==='commit' && GIT_SHA.test(ref.object.sha),
      'exact base commit ref required');
    sourceIdentity(ref.url,repository,'git/refs/heads/'+base);
    sourceIdentity(ref.object.url,repository,'git/commits/'+ref.object.sha);
    const commit=await read('git/commits/'+ref.object.sha);
    assert(commit.sha===ref.object.sha && GIT_SHA.test(commit.tree?.sha), 'exact base commit required');
    sourceIdentity(commit.url,repository,'git/commits/'+commit.sha);
    sourceIdentity(commit.tree.url,repository,'git/trees/'+commit.tree.sha);
    observation.baseCommit=commit.sha;
    const selected=await baseManifestEntry(read,repository,commit.tree.sha);
    observation.path=selected.path;
    observation.blobSHA=GIT_SHA.test(selected.entry.sha)?selected.entry.sha:null;
    const manifest=await sourceManifestBlob(read,repository,selected.entry);
    available();
    return manifest;
  };
  try {
    const manifest=await Promise.race([observe(),expired]);
    report(summary('observed'));
    return manifest;
  } catch(error) {
    const reason=error instanceof assert.AssertionError?error.message
      : error.message==='base source aggregate deadline exceeded'?error.message
      : error.name==='SyntaxError'?'invalid base source JSON'
      : error.code==='ERR_ENCODING_INVALID_ENCODED_DATA'?'invalid base source UTF8'
      : 'base source transport or body unavailable';
    const diagnostic={...summary('unavailable'),reason};
    report(diagnostic);
    throw new Error('base source observation failed: '+JSON.stringify(diagnostic));
  } finally { clearTimeout(timer); controller.abort(); }
}

async function main(mode) {
  if (mode === 'detect') {
    const result = admission(changedPaths(process.env.GITHUB_EVENT_NAME, process.env.GITHUB_EVENT_BEFORE));
    for (const [key, value] of Object.entries(result)) {
      const line = `${key}=${value}\n`;
      if (process.env.GITHUB_OUTPUT) appendFileSync(process.env.GITHUB_OUTPUT, line);
      process.stdout.write(line);
    }
  } else {
    assert.equal(mode, 'version');
    if (process.env.GITHUB_EVENT_NAME !== 'pull_request') return;
    if (['changelog-manual-release-', 'changeset-release/', 'release/', 'automated-release/']
      .some(prefix => process.env.GITHUB_HEAD_REF?.startsWith(prefix))) return;
    const base = process.env.GITHUB_BASE_REF || 'main';
    assert(/^[a-zA-Z0-9_./-]+$/u.test(base) && !base.startsWith('-'), 'valid base branch required');
    const manifest = await readBaseManifest({repository:process.env.GITHUB_REPOSITORY,base,
      token:process.env.GH_TOKEN || process.env.GITHUB_TOKEN,report:record=>console.log(JSON.stringify(record))});
    const head = ['Cargo.toml', 'rust/Cargo.toml'].find(existsSync);
    assert(head, 'head manifest required');
    checkVersions(readFileSync(head, 'utf8'), manifest);
    console.log('Package version unchanged.');
  }
}
if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  main(process.argv[2]).catch(error=>{console.error(error.message);process.exitCode=1;});
}
