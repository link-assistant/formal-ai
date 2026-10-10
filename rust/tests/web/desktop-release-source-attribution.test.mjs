// Run the actual Desktop Release resolver against source-bound GitHub replies.
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { chmodSync, mkdtempSync, mkdirSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import test from 'node:test';
import { fileURLToPath } from 'node:url';

const script = fileURLToPath(new URL('../../../scripts/desktop-release-resolve.sh', import.meta.url));
const creator = new URL('../fixtures/native-release-evidence/observations.mjs', import.meta.url).href;
const HEAD = 'a'.repeat(40);
const release = (tag_name) => ({ tag_name, draft: false, prerelease: false });
const child = { sha: 'b'.repeat(40), parents: [{ sha: HEAD }] };
const foreign = { sha: 'c'.repeat(40), parents: [{ sha: 'c'.repeat(40) }] };

function resolve(context, scenario, extra = {}) {
  const root = mkdtempSync(join(tmpdir(), 'formal-ai-desktop-source-'));
  context.after(() => rmSync(root, { recursive: true, force: true }));
  const bin = join(root, 'bin');
  mkdirSync(bin);
  const mock = '#!/usr/bin/env node\nconst scenario=' + JSON.stringify(scenario) + ';\n'
    + String.raw`(async()=>{
const creator = CREATOR_PLACEHOLDER;
const {materializePublishedFixture}=await import(creator);
const fs=require('node:fs');const os=require('node:os');const path=require('node:path');
const args = process.argv.slice(2);
const write = (value) => process.stdout.write(String(value ?? '') + '\n');
if (args[0] === 'api') {
  const endpoint = args[1];
  if (endpoint.includes('/actions/runs/')) write(JSON.stringify(scenario.actualRun ?? ({id:42,
    run_attempt:1,
    workflow_id:7,
    path:'.github/workflows/release.yml',
    status:'completed',
    conclusion:process.env.WORKFLOW_RUN_CONCLUSION,
    head_sha:process.env.WORKFLOW_RUN_HEAD_SHA,
    head_branch:'main',
    repository:{full_name:process.env.REPO},
    head_repository:{full_name:process.env.REPO}})));
  else if (endpoint.includes('/actions/workflows/')) write(JSON.stringify(scenario.actualWorkflow ?? ({id:7,path:'.github/workflows/release.yml'})));
  else if (endpoint.includes('/tags?')) write(scenario.exact ?? '');
  else if (endpoint.includes('/releases?')) {
    if (scenario.releaseFailure) { console.error('HTTP 401'); process.exit(1); }
    write(JSON.stringify(scenario.releases ?? []));
  } else if (endpoint.includes('/releases/tags/')) {
    const directory=fs.mkdtempSync(path.join(os.tmpdir(),'release-metadata-fixture-'));
    const tag=endpoint.split('/').at(-1);
    const result=materializePublishedFixture(directory,{version:tag.replace(/^v/,''),sourceCommit:scenario.commits?.[tag]?.sha??'b'.repeat(40),sourceTree:'d'.repeat(40),expectedAssets:scenario.assets??[]});
    write(JSON.stringify(result.metadata));fs.rmSync(directory,{recursive:true,force:true});
  } else if (endpoint.includes('/commits/')) {
    const commit = scenario.commits?.[endpoint.split('/').at(-1)] ?? { sha: '', parents: [] };
    if (args.includes('--jq')) write(args.at(-1) === '.parents[0].sha' ? commit.parents[0]?.sha : args.at(-1)==='.commit.tree.sha'?'d'.repeat(40):commit.sha);
    else write(JSON.stringify(commit));
  } else process.exit(2);
} else if (args[0] === 'release' && args[1] === 'view') {
  const tag = args[2]?.startsWith('--') ? null : args[2];
  if (!tag) write(scenario.latest ?? '');
  else if (args.includes('assets')) write((scenario.assets ?? []).join('\n'));
  else write(JSON.stringify({ tagName: tag }));
} else if (args[0]==='release'&&args[1]==='download') {
  const directory=args[args.indexOf('--dir')+1],tag=args[2];
  materializePublishedFixture(directory,{version:tag.replace(/^v/,''),sourceCommit:scenario.commits?.[tag]?.sha??'b'.repeat(40),sourceTree:'d'.repeat(40),expectedAssets:scenario.assets??[]});
  if(scenario.evidenceFault==='corrupt')fs.writeFileSync(path.join(directory,'formal-ai-native-source-'+tag.replace(/^v/,'')+'.json'),'{}');
  if(scenario.evidenceFault==='manifest')fs.writeFileSync(path.join(directory,'SHA256SUMS.txt'),'');
} else if(args[0]==='attestation'&&args[1]==='verify') {
  if(scenario.evidenceFault==='attestation')process.exit(1);
} else process.exit(2);
})().catch(error=>{console.error(error);process.exit(1)});
`.replace('CREATOR_PLACEHOLDER',JSON.stringify(creator));
  const executable = join(bin, 'gh');
  writeFileSync(executable, mock);
  chmodSync(executable, 0o755);
  const outputFile = join(root, 'outputs');
  writeFileSync(outputFile, '');
  const result = spawnSync('bash', [script], {
    encoding: 'utf8',
    env: { ...process.env, PATH: bin + ':' + process.env.PATH, REPO: 'owner/repository',
      EVENT: 'workflow_run', WORKFLOW_RUN_HEAD_SHA: HEAD, INPUT_TAG: '', RELEASE_TAG: '',
      WORKFLOW_RUN_ID: '42',
         WORKFLOW_RUN_ATTEMPT: '1',
         WORKFLOW_RUN_WORKFLOW_ID: '7',
         WORKFLOW_RUN_BRANCH: 'main',
         WORKFLOW_RUN_HEAD_REPOSITORY: 'owner/repository',
         WORKFLOW_RUN_CONCLUSION: 'success',

      GITHUB_OUTPUT: outputFile, ...extra },
  });
  return { ...result, outputs: Object.fromEntries(readFileSync(outputFile, 'utf8').trim().split('\n')
    .filter(Boolean).map((line) => line.split('='))) };
}

test('an unrelated latest release cannot stand in for the completed run', (context) => {
  const result = resolve(context, { latest: 'v9.9.9', releases: [release('v9.9.9')],
    commits: { 'v9.9.9': foreign } });
  assert.equal(result.status, 0, result.stderr);
  assert.equal(result.outputs.tag, '');
  assert.equal(result.outputs.should_build, 'false');
});

test('a delayed run selects its matching older published child', (context) => {
  const result = resolve(context, { latest: 'v9.9.9',
    releases: [release('v9.9.9'), release('v1.2.3')],
    commits: { 'v9.9.9': foreign, 'v1.2.3': child } });
  assert.equal(result.status, 0, result.stderr);
  assert.equal(result.outputs.tag, 'v1.2.3');
  assert.equal(result.outputs.should_build, 'true');
});

test('the issue479 child-release path still heals missing assets', (context) => {
  const result = resolve(context, { latest: 'v1.2.3', commits: { 'v1.2.3': child } });
  assert.equal(result.status, 0, result.stderr);
  assert.equal(result.outputs.tag, 'v1.2.3');
  assert.equal(result.outputs.should_build, 'true');
});

test('an exact-head release retains the defensive resolution tier', (context) => {
  const result = resolve(context, { exact: 'v1.2.3', latest: 'v9.9.9',
    commits: { 'v1.2.3': { sha: HEAD, parents: [] }, 'v9.9.9': foreign } });
  assert.equal(result.status, 0, result.stderr);
  assert.equal(result.outputs.tag, 'v1.2.3');
  assert.equal(result.outputs.should_build, 'true');
});

test('an explicit manual rebuild may still select an older release', (context) => {
  const result = resolve(context, { latest: 'v9.9.9', commits: { 'v9.9.9': foreign } },
    { EVENT: 'workflow_dispatch', INPUT_TAG: 'v9.9.9' });
  assert.equal(result.status, 0, result.stderr);
  assert.equal(result.outputs.tag, 'v9.9.9');
  assert.equal(result.outputs.should_build, 'true');
});

test('a GitHub transport failure cannot authorize an unrelated release', (context) => {
  const result = resolve(context, { latest: 'v9.9.9', commits: { 'v9.9.9': foreign },
    releaseFailure: true });
  assert.notEqual(result.status, 0);
  assert.equal(result.outputs.should_build, undefined);
});


// Model the concrete assets emitted by the actual packaging/finalize jobs.
function completeAssets(version) {
  const desktop = [
    ...['arm64', 'x64'].flatMap((arch) => ['dmg', 'zip'].map((format) =>
      `formal-ai-desktop-macos-${arch}-${version}.${format}`)),
    ...['installer', 'portable'].flatMap((kind) => ['x64', 'arm64'].map((arch) =>
      `formal-ai-desktop-windows-${kind}-${arch}-${version}.exe`)),
    ...['x64', 'arm64'].flatMap((arch) => ['AppImage', 'deb', 'tar.gz'].map((format) =>
      `formal-ai-desktop-linux-${arch}-${version}.${format}`)),
    'latest.yml', 'latest-mac.yml', 'latest-linux.yml',
  ];
  const workflow = readFileSync(new URL('../../../.github/workflows/desktop-release.yml', import.meta.url), 'utf8');
  const cli = [...workflow.matchAll(/target: *([A-Za-z0-9_.-]+), *label: *cli-[^,]+,.*archive: *([a-z.]+),/gu)]
    .map((match) => `formal-ai-cli-${match[1]}.${match[2]}`);
  assert.equal(cli.length, 5);
  const native=workflow.slice(workflow.indexOf('\n  native:'),workflow.indexOf('\n  build:'));
  const targets=[...native.matchAll(/target: *([A-Za-z0-9_.-]+), *binext:/gu)].map(match=>match[1]);
  assert.equal(targets.length,8);
  const evidence=[`formal-ai-native-source-${version}.json`,`formal-ai-native-protocol-${version}.json`,
    ...targets.map(target=>`formal-ai-native-${target}-${version}.json`),
    ...['macos-arm64','macos-x64'].map(label=>`formal-ai-signing-${label}-${version}.json`)];
  return [...desktop, ...cli, ...evidence, `formal-ai-vscode-${version}.vsix`, 'SHA256SUMS.txt', 'BUILD-PROVENANCE.txt'];
}

for (const omitted of ['formal-ai-vscode-1.2.3.vsix', 'SHA256SUMS.txt', 'BUILD-PROVENANCE.txt']) {
  test(`the matching release heals a missing ${omitted} after desktop and CLI success`, (context) => {
    const assets = completeAssets('1.2.3').filter((name) => name !== omitted);
    const result = resolve(context, { latest: 'v1.2.3', commits: { 'v1.2.3': child }, assets });
    assert.equal(result.status, 0, result.stderr);
    assert.equal(result.outputs.tag, 'v1.2.3');
    assert.equal(result.outputs.should_build, 'true');
    assert.ok(result.stdout.includes(omitted));
  });
}

test('only a release with the complete package and manifest set skips an automatic build', (context) => {
  const result = resolve(context, { latest: 'v1.2.3', commits: { 'v1.2.3': child }, assets: completeAssets('1.2.3') });
  assert.equal(result.status, 0, result.stderr);
  assert.equal(result.outputs.tag, 'v1.2.3');
  assert.equal(result.outputs.should_build, 'false');
});

test('a VSIX from another version cannot complete the matching release', (context) => {
  const assets = completeAssets('1.2.3').map((name) => name === 'formal-ai-vscode-1.2.3.vsix'
    ? 'formal-ai-vscode-9.9.9.vsix' : name);
  const result = resolve(context, { latest: 'v1.2.3', commits: { 'v1.2.3': child }, assets });
  assert.equal(result.status, 0, result.stderr);
  assert.equal(result.outputs.tag, 'v1.2.3');
  assert.equal(result.outputs.should_build, 'true');
});

test('an explicit manual rebuild retains the complete-assets override', (context) => {
  const result = resolve(context, { latest: 'v1.2.3', commits: { 'v1.2.3': child }, assets: completeAssets('1.2.3') },
    { EVENT: 'workflow_dispatch', INPUT_TAG: 'v1.2.3' });
  assert.equal(result.status, 0, result.stderr);
  assert.equal(result.outputs.tag, 'v1.2.3');
  assert.equal(result.outputs.should_build, 'true');
});

for(const fault of ['corrupt','manifest','attestation']) {
  test('complete names still heal invalid durable evidence: '+fault,context=>{
    const result=resolve(context,{latest:'v1.2.3',commits:{'v1.2.3':child},assets:completeAssets('1.2.3'),evidenceFault:fault});
    assert.equal(result.status,0,result.stderr);assert.equal(result.outputs.tag,'v1.2.3');assert.equal(result.outputs.should_build,'true');
    assert.match(result.stdout,/Durable release evidence did not verify/);
  });
}

// Producer groups preserve PR cancellation while isolating immutable stable runs.
import YAML from 'yaml';
import {createHash as workflowSourceHash} from 'node:crypto';
function concurrencyExpression(source,context){
 const re=/\s*(?:('(?:[^'\\]|\\.)*')|(==|!=|&&|\|\||[(),])|([A-Za-z_][A-Za-z0-9_.]*)|(\d+))/y,tokens=[];let pos=0;
 while(pos<source.length){re.lastIndex=pos;const m=re.exec(source);assert.ok(m,'unsupported expression token');tokens.push(m[1]??m[2]??m[3]??m[4]);pos=re.lastIndex;}let i=0;
 const precedence={'||':1,'&&':2,'==':3,'!=':3};
 function primary(){const t=tokens[i++];assert.ok(t,'missing operand');
  if(t==='('){const v=parse(1);assert.equal(tokens[i++],')');return v;}
  if(t.startsWith("'"))return t.slice(1,-1).replace(/\\(['\\])/gu,'$1');
  if(/^\d+$/u.test(t))return Number(t);
  if(t==='true'||t==='false')return t==='true';
  if(t==='format'&&tokens[i]==='('){
   i++;
   const values=[parse(1)];
   while(tokens[i]===','){
    i++;
    values.push(parse(1));
   }
   assert.equal(tokens[i++],')');
   const [pattern,...args]=values;
   assert.equal(typeof pattern,'string');
   return pattern.replace(/\{(\d+)\}/gu,(_,n)=>{
    assert.ok(Number(n)<args.length);
    return String(args[n]);
   });
  }
  let value=context;for(const key of t.split('.')){assert.ok(value!==null&&typeof value==='object'&&Object.hasOwn(value,key),'unbound expression field:'+t);value=value[key];}return value;
 }
 function parse(min){let left=primary();while(precedence[tokens[i]]>=min){const op=tokens[i++],right=parse(precedence[op]+1);left=op==='||'?(left||right):op==='&&'?(left&&right):op==='=='?left===right:left!==right;}return left;}
 const value=parse(1);assert.equal(i,tokens.length,'expression tail');return value;
}
function concurrencyTemplate(source,context){return source.replace(/\$\{\{\s*(.*?)\s*\}\}/gu,(_,body)=>String(concurrencyExpression(body.trim(),context)));}

const desktopSourceUrl=new URL('../../../.github/workflows/desktop-release.yml',import.meta.url);
const desktopGroupSuffix="${{ github.event_name != 'pull_request' && format('-{0}', github.run_id) || '' }}";
const originalProducerGroups={resolve:'${{ github.workflow }}-${{ github.ref }}-resolve',base:'${{ github.workflow }}-${{ github.ref }}-base','native-source':'${{ github.workflow }}-${{ github.ref }}-native-source',native:'${{ github.workflow }}-native-${{ github.event.pull_request.number || github.ref }}-${{ matrix.target }}'};
const producerContext=(event,run,target)=>({github:{workflow:'Desktop Release',ref:event==='pull_request'?'refs/pull/81/merge':'refs/heads/main',event_name:event,run_id:run,event:{pull_request:{number:event==='pull_request'?81:null}}},matrix:{target}});
function verifyDesktopProducerGroups(workflow){
 let assertions=0;
 for(const [id,originalGroup]of Object.entries(originalProducerGroups)){
  const concurrency=workflow.jobs[id].concurrency;
  assert.equal(concurrency.group,originalGroup+desktopGroupSuffix);
  assert.equal(concurrency['cancel-in-progress'],"${{ github.event_name == 'pull_request' }}");assert.equal(concurrency.queue,undefined);
  for(const target of ['x86_64-unknown-linux-gnu','aarch64-apple-darwin']){
   for(const run of [11,12,13]){const context=producerContext('pull_request',run,target);assert.equal(concurrencyTemplate(concurrency.group,context),concurrencyTemplate(originalGroup,context));assert.equal(concurrencyExpression("github.event_name == 'pull_request'",context),true);assertions+=2;}
   for(const event of ['workflow_run','workflow_dispatch','release']){
    const contexts=[11,12,13].map(run=>producerContext(event,run,target));
    assert.equal(new Set(contexts.map(context=>concurrencyTemplate(originalGroup,context))).size,1);
    assert.equal(new Set(contexts.map(context=>concurrencyTemplate(concurrency.group,context))).size,3);
    for(const context of contexts)assert.equal(concurrencyExpression("github.event_name == 'pull_request'",context),false);assertions+=5;
   }
  }
 }
 assert.notEqual(concurrencyTemplate(workflow.jobs.native.concurrency.group,producerContext('release',11,'amd64')),concurrencyTemplate(workflow.jobs.native.concurrency.group,producerContext('release',11,'arm64')));assertions++;
 return assertions;
}
test('immutable desktop producers retain PR identity and isolate three stable main runs',()=>{
 const bytes=readFileSync(desktopSourceUrl),sha=workflowSourceHash('sha256').update(bytes).digest('hex');
 const workflow=YAML.parse(bytes.toString());assert.equal(verifyDesktopProducerGroups(workflow),169);
 assert.equal(workflowSourceHash('sha256').update(readFileSync(desktopSourceUrl)).digest('hex'),sha,'actual source changed during observer');
});
test('producer group loss cancellation changes and unknown expression bindings refuse',()=>{
 const workflow=YAML.parse(readFileSync(desktopSourceUrl,'utf8'));
 // Independent positive fixture comes from the original declared groups plus the reviewed suffix.
 for(const [id,group]of Object.entries(originalProducerGroups))workflow.jobs[id].concurrency={group:group+desktopGroupSuffix,'cancel-in-progress':"${{ github.event_name == 'pull_request' }}"};
 assert.equal(verifyDesktopProducerGroups(workflow),169);
 for(const id of Object.keys(originalProducerGroups))for(const change of [c=>c.group=originalProducerGroups[id],c=>c.group+='-foreign',c=>c['cancel-in-progress']=false,c=>c.queue='max']){
  const altered=structuredClone(workflow);change(altered.jobs[id].concurrency);assert.throws(()=>verifyDesktopProducerGroups(altered));
 }
 for(const source of ['unknown.field',"process.exit(1)","format('-{2}',github.run_id)","github.run_id trailing"])assert.throws(()=>concurrencyExpression(source,producerContext('release',11,'amd64')));
});
// Real process/fixture transport: provider identities remain independent of the event fields.
for (const [field,
   value] of [['id',
   43],
   ['run_attempt',
   2],
   ['status',
   'in_progress'],
   ['conclusion',
   'cancelled'],
   ['conclusion',
   'neutral'],
   ['head_sha',
   'c'.repeat(40)],
   ['head_branch',
   'feature'],
   ['repository',
   {
  full_name: 'foreign/repository'
}], ['head_repository', {
  full_name: 'fork/repository'
}], ['workflow_id', 8], ['path', '.github/workflows/foreign.yml']]) test('an exact published tag refuses mismatched authenticated run ' + field, context => {
  const actualRun = {
    id: 42,
    run_attempt: 1,
    workflow_id: 7,
    status: 'completed',
    conclusion: 'success',
    head_sha: HEAD,
    head_branch: 'main',
    repository: {
      full_name: 'owner/repository'
    },
    head_repository: {
      full_name: 'owner/repository'
    },
    path: '.github/workflows/release.yml',
    [field]: value
  };
  const result = resolve(context, {
    exact: 'v1.2.3',
    actualRun,
    commits: {
      'v1.2.3': {
        sha: HEAD,
        parents: []
      }
    }
  });
  assert.equal(result.status, 0, result.stderr);
  assert.equal(result.outputs.tag, '');
  assert.equal(result.outputs.should_build, 'false');
});
test('an exact published tag remains available after an authenticated later job failure', context => {
  const actualRun = {
    id: 42,
    run_attempt: 1,
    workflow_id: 7,
    status: 'completed',
    conclusion: 'failure',
    head_sha: HEAD,
    head_branch: 'main',
    repository: {
      full_name: 'owner/repository'
    },
    head_repository: {
      full_name: 'owner/repository'
    },
    path: '.github/workflows/release.yml'
  };
  const result = resolve(context, {
    exact: 'v1.2.3',
    actualRun,
    commits: {
      'v1.2.3': {
        sha: HEAD,
        parents: []
      }
    }
  }, {
    WORKFLOW_RUN_CONCLUSION: 'failure'
  });
  assert.equal(result.status, 0, result.stderr);
  assert.equal(result.outputs.tag, 'v1.2.3');
  assert.equal(result.outputs.should_build, 'true');
});
