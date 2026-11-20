// Fast, conservative PR admission. Production retains its full Rust classifier.
import assert from 'node:assert/strict';
import {appendFileSync, existsSync, readFileSync} from 'node:fs';
import {execFileSync} from 'node:child_process';
import {pathToFileURL} from 'node:url';
import {createHash} from 'node:crypto';
import {isDocumentationPath} from './lib/documentation-paths.mjs';

export const isDocumentation = isDocumentationPath;
// Issue #1191: PR #1497 passed every PR check, then main failed with E0308 in
// rust/src, because no PR job compiles Rust. Clippy on the library and
// binaries reads rust/ (src, build.rs, manifests, the embedded mirror),
// build.rs reads data/seed/api-cache, Cargo reads .cargo/, the toolchain files
// and clippy.toml, and the job runs scripts/ci-clippy-profile.sh. The prefixes
// below are proven NOT to be inputs, so a path outside them (unknown included)
// selects the native check: missing it costs a broken main.
const NON_NATIVE_PREFIXES = ['docs/', 'changelog.d/', 'experiments/', 'dev/', 'examples/', 'analysis/',
  'coverage/', 'js/', 'ts/', 'packages/', 'desktop/', 'vscode/', 'docker/', 'scripts/', 'rust/tests/',
  'rust/examples/', '.formal-ai/', '.githooks/'];
const NON_NATIVE_FILES = new Set(['package.json', 'bun.lock', '.bun-version', 'tsconfig.json', 'Dockerfile',
  'Dockerfile.slim', 'compose.yaml', '.dockerignore', '.gitignore', '.gitattributes', '.gitkeep',
  '.lycheeignore', '.pre-commit-config.yaml', '.secretlintrc.json', 'LICENSE']);
// The focused native lane also executes isolated integration targets and an
// HTTP replay. Changes to their sources or consumed fixtures must run that lane.
const NATIVE_REGRESSION_INPUTS = new Set([
  "experiments/agent_cli_e2e/mock-meaning-mcp.mjs",
  "experiments/formal_ai_subagent/evidence/specification-delivery-1188/dormant-staged-release/candidate-output-bindings.json",
  "experiments/formal_ai_subagent/evidence/specification-delivery-1188/dormant-staged-release/candidate-release-caller.yml",
  "experiments/formal_ai_subagent/evidence/specification-delivery-1188/dormant-staged-release/original-release-workflow.yml",
  "experiments/formal_ai_subagent/evidence/specification-delivery-1188/dormant-staged-release/stage-source-coverage.json",
  "rust/tests/e2e/playwright.pages.config.js",
  "rust/tests/e2e/tests/connectivity.spec.js",
  "rust/tests/e2e/tests/demo.spec.js",
  "rust/tests/e2e/tests/multilingual-chat.spec.js",
  "rust/tests/e2e/tests/multilingual-issue-27.spec.js",
  "rust/tests/e2e/tests/multilingual-memory-settings.spec.js",
  "rust/tests/e2e/tests/multilingual-wikipedia.spec.js",
  "rust/tests/unit/ci-cd/staged_release_operations.rs",
  "rust/tests/unit/ci-cd/workflow_fixtures.rs",
  "rust/tests/unit/ci-cd/workflow_release.rs",
  "rust/tests/unit/ci-cd/workflow_release_operations.rs",
  "rust/tests/unit/ci_gates.rs",
  "rust/tests/unit/workflow_contracts.rs",
  "scripts/check-disk-usage-policy.rs",
  "scripts/checked-release-operation-view.mjs",
  "scripts/documentation-policy-dependencies/package-lock.json",
  "scripts/documentation-policy-dependencies/package.json",
  "scripts/generate-staged-release.mjs",
  "scripts/governed-github-command-provider.mjs",
  "scripts/lib/ci-speed-workflows.mjs",
  "scripts/maintained-staged-authority.mjs",
  "scripts/native-staged-release-operation-view.mjs",
  "scripts/refresh-release-specification-witness.mjs",
  "scripts/release-source-freshness.mjs",
  "scripts/run-ci-gates.rs",
  "scripts/staged-caller-authority.mjs",
  'rust/tests/integration/interface_contracts.rs',
  'rust/tests/integration/formal_ai_cli.rs',
  'rust/tests/integration/http_server.rs',
  'rust/tests/integration/pty.rs',
  'rust/tests/support/structured_stderr.rs',
  'rust/tests/integration/with_formal_ai.rs',
  'rust/tests/integration/with_formal_ai/fixture_env.rs',
  'rust/tests/integration/with_formal_ai/issue_1582.rs',
  'rust/tests/integration/with_formal_ai/model_metadata.rs',
  'rust/tests/integration/with_formal_ai/verbosity.rs',
  'rust/tests/integration/with_formal_ai_argv.rs',
  'rust/tests/integration/issue_819_tui_isolation.rs',
  'rust/tests/integration/issue_1138_family_behaviour.rs',
  'rust/tests/unit/formal_ai/seed_and_memory.rs',
  'rust/tests/unit/seed/issue_699_handler_migration.rs',
  'docs/case-studies/issue-552/raw-data/chatgpt-share-6a3825b9.html',
  'docs/case-studies/issue-781/raw-data/chatgpt-web-capture.json',
  'docs/case-studies/issue-699/agent-cli-evidence/handler-migration-batch-report.lino',
  'rust/tests/unit/calendar-currency.rs',
  'rust/tests/unit/openai_forced_functions.rs',
  'rust/tests/unit/cnl_output.rs',
  'rust/tests/unit/cnl_parity.rs',
  'rust/tests/unit/meta_executor_parity.rs',
  'rust/tests/unit/specification/synthesis.rs',
  'rust/tests/unit/coding_contracts.rs',
  'rust/tests/unit/coding_discovery/composition.rs',
  'rust/tests/unit/coding_discovery/composition_search.rs',
  'rust/tests/unit/coding_discovery/composition/rejected_ir.rs',
  'rust/tests/unit/specification/unknown_reasoning.rs',
  'rust/tests/unit/data_files.rs',
  'rust/tests/unit/data_files/technical_scalar.rs',
  'rust/tests/unit/scheduling_contracts.rs',
  'rust/tests/unit/operational_contracts.rs',
  'rust/tests/unit/reasoning_contracts.rs',
  'rust/tests/unit/issue_1157_idempotent_commit.rs',
  'rust/tests/unit/issue_1158_pull_request_steps.rs',
  'rust/tests/unit/issue_1160_regular_prompt.rs',
  'rust/tests/unit/hive_delivery_prerequisites.rs',
  'rust/tests/fixtures/hive-mind/offline_delivery.rs',
  'rust/tests/fixtures/hive-mind/fake-gh.cjs',
  'rust/tests/fixtures/hive-mind/formal-ai-prompt.txt',
  'rust/tests/fixtures/hive-mind/regular-prompt.txt',
  'rust/tests/fixtures/hive-mind/restart-uncommitted-main-jar.txt',
  'rust/tests/fixtures/issue-1162/hello-world-kotlin-issue.md',
  'rust/tests/unit/offline_instruction_research.rs',
  'rust/tests/unit/issue_1133_hive_mind_three_runs.rs',
  'rust/tests/unit/issue_1133_hive_mind_three_runs/extended.rs',
  'rust/tests/unit/issue_1066_ladder_capability/tool_workspace.rs',
  'rust/tests/fixtures/atomic-record-append.rs',
  'rust/tests/fixtures/observed-plan-tools.rs',
  'rust/tests/fixtures/commit-cue-masking.rs',
  'rust/tests/fixtures/hive-workflow-original-task.json',
  'rust/tests/unit/specification/reasoning_paths_procedures.rs',
  'rust/tests/unit/solver/issue_1138_self_use_concept_lookup.rs',
  'rust/tests/unit/issue_1138_retrieval_method.rs',
  'rust/tests/unit/issue_498_google_trends.rs',
  'rust/tests/unit/agentic-coding/issue_498_google_trends_catalog.rs',
  'rust/examples/dump_google_trends_catalog.rs',
  'docs/case-studies/issue-498/agent-cli-session-google-trends.json',
  'rust/tests/unit/specification/reasoning_loop.rs',
  'rust/tests/unit/specification/benchmarks.rs',
  'data/benchmarks/LICENSES.md',
  'docs/case-studies/issue-244/raw-data/online-research.md',
  'rust/tests/unit/specification/formalization.rs',
  'rust/tests/unit/specification/pronoun_topic_follow_up.rs',
  'rust/tests/unit/specification/debug_session.rs',
  'rust/tests/unit/specification/event_source_origin.rs',
  'rust/tests/unit/web_requests.rs',
  'rust/tests/unit/web_requests/extended.rs',
  'rust/tests/unit/page_formalization_route.rs',
  'rust/tests/unit/seed/issue_1172_live_fact_answer.rs',
  'rust/tests/unit/issue_1173_fallback_executes_search.rs',
  'rust/tests/unit/docs_requirements/agent_cli_shell.rs',
  'rust/tests/unit/ci-cd/network_download_retry.rs',
  'README.md',
  'docs/desktop/server-api.md',
  'rust/tests/unit/issue_1138_place_timezone.rs',
  'rust/tests/unit/solver/issue_869_meeting_scheduling.rs',
  'rust/tests/unit/specification/spoken_hour_scheduling.rs',
  'rust/tests/unit/capability-routing/issue_1138_self_use_intent_routing.rs',
  'rust/tests/fixtures/issue-1580-calendar-clock.json',
  'rust/tests/unit/issue_706_any_language.rs',
  'rust/tests/unit/solver_unknown_reasoning_captured_sources.rs',
  'rust/tests/fixtures/issue-1138-b1/expected-senses.json',
  'rust/tests/fixtures/meta-executor-parity.json',
  'rust/tests/fixtures/meta-executor-status-parity.json',
  'rust/tests/fixtures/meta-executor-cue-parity.json',
  'data/parity/issue-1589-calendar-currency.json',
  'data/parity/issue-1590-forced-functions.json',
  'data/parity/cross-runtime-synthesis.json',
  'rust/tests/web/issue-1590-forced-functions.test.mjs',
  'rust/tests/web/issue-1586-structured-output.test.mjs',
  'rust/tests/web/issue-1396-native.test.mjs',
  'rust/tests/fixtures/issue-1396/wiki-how-ru.json',
  'rust/tests/web/support/structured-log-fixture.mjs',
  'js/server/structured-log.mjs',
]);
export function nativeCompileInput(path) {
  // The download contract discovers every shell script, including new paths.
  if (path.endsWith('.sh')) return true;
  if (path.startsWith('data/parity/issue-1584-cnl/') || NATIVE_REGRESSION_INPUTS.has(path) || ['scripts/ci-clippy-profile.sh',
    'scripts/ci-pull-request-admission.mjs', 'scripts/lib/documentation-paths.mjs'].includes(path)) return true;
  if (isDocumentation(path) || NON_NATIVE_FILES.has(path)) return false;
  // The isolated data-file target parses all LiNo and cached source JSON.
  if (path.startsWith('data/')) return path.endsWith('.lino')
    || (path.startsWith('data/cache/') && path.endsWith('.json'))
    || path.startsWith('data/seed/api-cache/');
  // The native workflow contracts discover every workflow and traverse local calls.
  if (path.startsWith('.github/workflows/')) return true;
  return !NON_NATIVE_PREFIXES.some(prefix => path.startsWith(prefix));
}
// Classify conservatively: extra lightweight checks are cheaper than missed changes.
export function admission(paths) {
  assert(Array.isArray(paths) && paths.every(path => typeof path === 'string'));
  const relevant = paths.filter(path => !['experiments/', 'dev/log/', 'docs/case-studies/'].some(prefix => path.startsWith(prefix)));
  const code = relevant.some(path => !isDocumentation(path));
  return Object.fromEntries(['rs-changed', 'toml-changed', 'docs-changed', 'workflow-changed',
    'pipeline-changed', 'agentic-routing-changed', 'any-code-changed', 'js-changed',
    'ts-changed', 'rust-changed', 'release-pending', 'native-type-check'].map(key => [key,
      key === 'release-pending' ? false : key === 'docs-changed' ? relevant.some(isDocumentation)
        : key === 'native-type-check' ? paths.length === 0 || paths.some(nativeCompileInput) : code]));
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
