#!/usr/bin/env node
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import {
  pathToFileURL,
  fileURLToPath}
from 'node:url';
import {
  routes,
  digest,
  exactHead,
  authenticateRun,
  completePage,
  authenticateProducerJob,
  authenticateArtifact,
  sourceCheckout,
  authenticateSourceCommit,
  authenticateRootTree,
  sourceUploadProducer,
  gitBytes,
  runRead,
  immutableSourceWitness,
  qualifyNativeData,
  guardedImportMembers,
  verifyPhysicalDerivationInputs,
  nativeDerivationRows}
from './lib/qualified-native-data-collector.mjs';
export async function collectQualifiedNativeData({
  head,
  destination,
  kinds=['census',
  'sessions',
  'answers'],
  repository='link-assistant/formal-ai',
  cwd=process.cwd(),
  api,
  auditOnly=false}
) {
  exactHead(head);
  assert.match(repository,
  /^[A-Za-z0-9_.-]+\/[A-Za-z0-9_.-]+$/u);
  assert.ok(kinds.length>0&&new Set(kinds).size===kinds.length&&kinds.every(k=>routes[k]));
  assert.equal(typeof auditOnly,
  'boolean');
  const target=path.resolve(destination);
  assert.ok(destination && !fs.existsSync(target),
  'fresh explicit scratch destination required');
  assert.ok(target!==path.resolve(cwd)&&!target.startsWith(path.resolve(cwd)+path.sep),
  'repository destination refused');
  let ancestor=path.dirname(target);
  while(ancestor!==path.dirname(ancestor)){
    if(fs.existsSync(ancestor))assert.ok(fs.lstatSync(ancestor).isDirectory()&&!fs.lstatSync(ancestor).isSymbolicLink(),
    'symlink scratch parent refused');
    ancestor=path.dirname(ancestor);
  }
  const implementationPaths=['collect-qualified-native-data-artifacts.mjs',
  'lib/qualified-native-data-collector.mjs',
  'qualified-data-artifact-profiles.mjs',
  'github-artifact-by-identifier.mjs',
  'lib/native-session-projections.mjs',
  'lib/native-response-capture.mjs',
  'lib/native-test-module-graph.mjs',
  'lib/rust-specification-cases.mjs',
  'lib/rust-specification-values.mjs'];
  const implementationRoot=path.dirname(fileURLToPath(import.meta.url));
  const implementation=implementationPaths.map(file=>({
    path:file,
    sha256:digest(fs.readFileSync(path.join(implementationRoot,
    file)))}
  ));
  const snapshots=[],
  request=api??(endpoint=>runRead('gh',
  ['api',
  '--hostname',
  'github.com',
  endpoint],
  cwd,
  32*1024*1024));
  const raw=endpoint=>{
    const bytes=request('repos/'+repository+'/'+endpoint);
    assert.ok(Buffer.isBuffer(bytes));
    snapshots.push({
      endpoint,
      bytes}
    );
    return bytes;
  }
  ;
  const json=endpoint=>JSON.parse(raw(endpoint));
  const headCommit=json('commits/'+head);
  assert.equal(headCommit.sha,
  head);
  const localHeadTree=runRead('git',
  ['rev-parse',
  head+'^{tree}'],
  cwd).toString().trim();
  assert.equal(localHeadTree,
  headCommit.commit.tree.sha,
  'local exact-head Git objects differ from authenticated API');
  authenticateRootTree(json('git/trees/'+localHeadTree),
  localHeadTree);
  const qualified=[],
  witnesses=new Map();
  for(const kind of kinds){
    const route=routes[kind],
    sourceWorkflow=gitBytes(cwd,
    head,
    route.producerWorkflow??route.workflow).toString();
    const producer=sourceUploadProducer(sourceWorkflow,
    route.artifact);
    let jobName=producer.jobName;
    const expectedPaths=kind==='census'?['data/meta/self-ast',
    'data/meta/self-ast.lino',
    'data/meta/self-healing-case.lino',
    'docs/case-studies/issue-538/agent-cli-session-self-ast.json']:['${{ runner.temp }}/'+(kind==='sessions'?'native-session-projections':'native-response-observations')];
    assert.deepEqual(producer.paths,
    expectedPaths,
    'unknown upload source contract');
    if(route.producerWorkflow){
      const caller=gitBytes(cwd,
      head,
      route.workflow).toString();
      const jobs=[...caller.matchAll(/^  ([A-Za-z_][\w-]*):\n((?:^(?: {4,}.*|\s*)\n)*)/gmu)].filter(m=>m[2].includes('uses: ./'+route.producerWorkflow));
      assert.equal(jobs.length,
      1,
      'unique source-owned reusable producer caller required');
      assert.ok(!/^    name:/mu.test(jobs[0][2]),
      'unsupported caller display-name expression');
      jobName=jobs[0][1]+' / '+jobName;
    }
    const page=json('actions/workflows/'+path.basename(route.workflow)+'/runs?head_sha='+head+'&per_page=100');
    const runs=completePage(page,
    'workflow_runs').filter(r=>r.head_sha===head&&r.status==='completed'&&['success',
    'failure'].includes(r.conclusion));
    assert.ok(runs.length>0,
    'no completed exact-head producer run');
    runs.sort((a,
    b)=>b.id-a.id);
    const run=authenticateRun(json('actions/runs/'+runs[0].id),
    {
      head,
      workflow:route.workflow,
      repository}
    );
    const jobs=completePage(json('actions/runs/'+run.id+'/attempts/'+run.run_attempt+'/jobs?per_page=100'),
    'jobs').filter(job=>job.name===jobName);
    assert.equal(jobs.length,
    1,
    'unique source-owned producer job required');
    const job=authenticateProducerJob(jobs[0],
    run,
    jobName);
    const log=raw('actions/jobs/'+job.id+'/logs');
    const commit=sourceCheckout(log.toString());
    const sourceCommit=json('commits/'+commit);
    const tree=sourceCommit.commit.tree.sha;
    authenticateSourceCommit(sourceCommit,
    head,
    tree);
    authenticateRootTree(json('git/trees/'+tree),
    tree);
    assert.equal(runRead('git',
    ['rev-parse',
    tree+'^{tree}'],
    cwd).toString().trim(),
    tree,
    'producer objects unavailable or source drift');
    assert.equal(digest(gitBytes(cwd,
    tree,
    route.producerWorkflow??route.workflow)),
    digest(Buffer.from(sourceWorkflow)),
    'producer workflow drift between selected head and compiled checkout');
    const artifacts=completePage(json('actions/runs/'+run.id+'/artifacts?per_page=100'),
    'artifacts').filter(a=>a.name===route.artifact);
    assert.equal(artifacts.length,
    1,
    'missing or ambiguous official producer artifact');
    const metadata=json('actions/artifacts/'+artifacts[0].id);
    assert.equal(metadata.expired,
    false);
    assert.ok(metadata.size_in_bytes<=16*1024*1024,
    'archive byte cap');
    const archive=raw('actions/artifacts/'+metadata.id+'/zip'),
    repeated=json('actions/artifacts/'+metadata.id);
    authenticateArtifact(metadata,
    run,
    route.artifact,
    archive,
    repeated);
    if(!witnesses.has(tree))witnesses.set(tree,
    await immutableSourceWitness(cwd,
    tree));
    const witness=witnesses.get(tree);
    const data=qualifyNativeData({
      kind,
      archive,
      commit,
      tree,
      cwd,
      witness,
      run,
      producerJob:producer.job,
      checkoutLog:log.toString()}
    );
    let sourceGuards=[];
    if(!auditOnly&&kind!=='answers'){
      const producerFile=kind==='sessions'?'rust/examples/regenerate_agent_cli_sessions.rs':null;
      const fixtureInputs=kind==='sessions'?JSON.parse(data.entries.find(entry=>entry.name==='producer-receipt.json').bytes).inputs.map(input=>input.path):[];
      const exampleSources=kind==='census'?[...sourceWorkflow.matchAll(/--example ([A-Za-z_][A-Za-z_0-9]*)/gu)].map(match=>'rust/examples/'+match[1]+'.rs'):[];
      sourceGuards=verifyPhysicalDerivationInputs(cwd,
      nativeDerivationRows(witness.rows,
      [...(producerFile?[producerFile]:[]),
      ...fixtureInputs,
      ...exampleSources]));
    }
    qualified.push({
      kind,
      run,
      job,
      metadata,
      archive,
      entries:data.entries,
      proof:data.proof,
      sourceGuards}
    );
  }
 // All requested cohorts qualify before a destination or import recipe is written.
 assert.deepEqual(implementationPaths.map(file=>({
    path:file,
    sha256:digest(fs.readFileSync(path.join(implementationRoot,
    file)))}
  )),
  implementation,
  'collector implementation drift');
  guardedImportMembers(qualified);
  fs.mkdirSync(target,
  {
    recursive:true}
  );
  const sources=snapshots.map((s,
  index)=>{
    const file='api/'+String(index).padStart(3,
    '0')+'.bin';
    fs.mkdirSync(path.dirname(path.join(target,
    file)),
    {
      recursive:true}
    );
    fs.writeFileSync(path.join(target,
    file),
    s.bytes,
    {
      flag:'wx',
      mode:0o600}
    );
    return {
      endpoint:s.endpoint,
      path:file,
      bytes:s.bytes.length,
      sha256:digest(s.bytes)}
    ;
  }
  );
  const imports=[];
  for(const item of qualified){
    const folder=path.join(target,
    item.kind);
    fs.mkdirSync(folder);
    fs.writeFileSync(path.join(folder,
    'original.zip'),
    item.archive,
    {
      flag:'wx',
      mode:0o600}
    );
    fs.writeFileSync(path.join(folder,
    'qualification.json'),
    JSON.stringify(item.proof,
    null,
    2)+'\n',
    {
      flag:'wx'}
    );
    for(const entry of item.entries){
      assert.equal(entry.directory,
      false);
      const output=path.join(folder,
      'members',
      entry.name);
      fs.mkdirSync(path.dirname(output),
      {
        recursive:true}
      );
      fs.writeFileSync(output,
      entry.bytes,
      {
        flag:'wx',
        mode:0o600}
      );
      if(!auditOnly&&item.kind!=='answers'&&(item.kind==='census'||entry.name.startsWith('generated/'))){
        const file=item.kind==='sessions'?entry.name.slice(10):entry.name;
        if(fs.existsSync(path.join(cwd,
        file)))assert.ok(fs.lstatSync(path.join(cwd,
        file)).isFile(),
        'nonregular repository import preimage');
        const before=fs.existsSync(path.join(cwd,
        file))?digest(fs.readFileSync(path.join(cwd,
        file))):null;
        imports.push({
          path:file,
          preimageSha256:before,
          afterSha256:digest(entry.bytes),
          bytes:entry.bytes.length,
          payload:path.relative(target,
          output),
          sourceGuard:{
            expectedHead:head,
            compiledTree:item.proof.identity.tree,
            manifestKey:'derivationSourceGuards.'+item.kind}
          ,
          apply:'root actual Formal AI guarded transformation only',
          authority:'not-granted'}
        );
      }
    }
  }
  const manifest={
    schema:'exact-head-qualified-native-data-collection/v1',
    expectedHead:head,
    repository,
    authority:'not-granted',
    semanticPassCredit:false,
    publicationAuthority:'not-granted',
    localNativeExecution:false,
    auditOnly,
    importEligibility:auditOnly?'not-granted-audit-only':'physical-derivation-inputs-verified-requires-root-FA-guards',
    collectorImplementation:{
      root:implementationRoot,
      files:implementation,
      authority:'not-granted'}
    ,
    sourceSnapshots:sources,
    cohorts:qualified.map(x=>({
      kind:x.kind,
      run:x.run.id,
      attempt:x.run.run_attempt,
      runConclusion:x.run.conclusion,
      job:x.job.id,
      artifact:x.metadata.id,
      digest:x.metadata.digest,
      source:x.proof.identity}
    )),
    derivationSourceGuards:Object.fromEntries(qualified.filter(item=>item.sourceGuards.length>0).map(item=>[item.kind,
    {
      expectedHead:head,
      compiledTree:item.proof.identity.tree,
      physicalInputs:item.sourceGuards}
    ])),
    guardedImportProposal:[...new Map(imports.map(row=>[row.path,
    row])).values()]}
  ;
  fs.writeFileSync(path.join(target,
  'collection-manifest.json'),
  JSON.stringify(manifest,
  null,
  2)+'\n',
  {
    flag:'wx'}
  );
  return manifest;
}
if(import.meta.url===pathToFileURL(process.argv[1]??'').href){
  const ceiling=setTimeout(()=>process.exit(124),
  1800000);
  ceiling.unref();
  const args=process.argv.slice(2);
  assert.equal(args.length%2,
  0,
  'explicit flag/value pairs required');
  const options={
  }
  ;
  for(let i=0;
  i<args.length;
  i+=2){
    assert.ok(['--expected-head',
    '--dest',
    '--kind',
    '--source-root',
    '--audit-only'].includes(args[i]));
    assert.ok(!Object.hasOwn(options,
    args[i]));
    options[args[i]]=args[i+1];
  }
  console.log(JSON.stringify(await collectQualifiedNativeData({
    auditOnly:options['--audit-only']===undefined?false:(assert.equal(options['--audit-only'],
    'true'),
    true),
    head:options['--expected-head'],
    destination:options['--dest'],
    cwd:options['--source-root']?path.resolve(options['--source-root']):process.cwd(),
    kinds:options['--kind']?[options['--kind']]:undefined}
  )));
}
