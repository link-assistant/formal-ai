import assert from 'node:assert/strict';
import {
  createHash}
from 'node:crypto';
import {
  spawnSync,
  spawn}
from 'node:child_process';
import {
  readFileSync,
  lstatSync}
from 'node:fs';
import {
  join}
from 'node:path';
import {
  nativeCaptureProfile,
  nativeCaptureArchiveEnvelope,
  nativeCensusProfile,
  nativeSessionArchiveProfile,
  verifyDataArtifactProfile}
from '../qualified-data-artifact-profiles.mjs';
import {
  readQualifiedDataArtifactZip}
from '../github-artifact-by-identifier.mjs';
import {
  projectionRegistry,
  projectionDataProfile,
  verifyProjectionDataEntries}
from './native-session-projections.mjs';
import {
  readNativeTestModuleGraph,
  bindOriginalSourceCase}
from './native-test-module-graph.mjs';
export const digest=b=>createHash('sha256').update(b).digest('hex');
const nativeRoots=['rust/src',
'rust/embedded',
'rust/Cargo.toml',
'rust/Cargo.lock',
'rust/build.rs',
'data/seed/api-cache',
'.cargo',
'rust/.cargo'];
export const routes=Object.freeze({
  census:{
    workflow:'.github/workflows/regenerate-self-ast-census.yml',
    artifact:'self-ast-census'}
  ,
  sessions:{
    workflow:'.github/workflows/regenerate-native-session-projections.yml',
    artifact:'native-session-projections'}
  ,
  answers:{
    workflow:'.github/workflows/release.yml',
    producerWorkflow:'.github/workflows/native-response-observations.yml',
    artifact:'native-response-observations'}
}
);
export function exactHead(value){
  assert.match(value,
  /^[a-f0-9]{40}$/u);
  return value;
}
export function authenticateRun(run,
{
  head,
  workflow,
  repository}
){
  exactHead(head);
  assert.equal(run.head_sha,
  head);
  assert.equal(run.status,
  'completed');
  assert.ok(['success',
  'failure'].includes(run.conclusion),
  'completed authentic run required; producer success is checked separately');
  assert.equal(run.path,
  workflow);
  if(repository!==undefined)assert.equal(run.repository.full_name,
  repository);
  assert.match(String(run.id),
  /^[1-9]\d*$/u);
  assert.match(String(run.run_attempt),
  /^[1-9]\d*$/u);
  return run;
}
export function completePage(value,
key){
  assert.ok(Array.isArray(value[key]));
  assert.equal(value.total_count,
  value[key].length,
  'truncated or incomplete API enumeration');
  return value[key];
}
export function authenticateProducerJob(job,
run,
name){
  assert.equal(job.run_id,
  run.id);
  assert.equal(job.head_sha,
  run.head_sha);
  assert.equal(job.status,
  'completed');
  assert.equal(job.conclusion,
  'success');
  assert.equal(job.name,
  name);
  assert.ok(job.steps.some(step=>step.conclusion==='success'),
  'successful producer steps required');
  return job;
}
export function authenticateArtifact(metadata,
run,
name,
archive,
repeated){
  assert.equal(metadata.name,
  name);
  assert.equal(metadata.expired,
  false);
  assert.match(String(metadata.id),
  /^[1-9]\d*$/u);
  assert.equal(metadata.workflow_run.id,
  run.id);
  assert.equal(metadata.workflow_run.head_sha,
  run.head_sha);
  assert.equal(metadata.size_in_bytes,
  archive.length);
  assert.ok(archive.length<=16*1024*1024);
  assert.equal(metadata.digest,
  'sha256:'+digest(archive));
  assert.deepEqual(repeated,
  metadata,
  'artifact metadata changed');
}
export function sourceCheckout(log){
  const rows=[...log.matchAll(/\[command\][^\n]*git log -1 --format=%H\r?\n[^\n]*? ([a-f0-9]{40})(?:\r?\n|$)/gu)].map(m=>m[1]);
  assert.equal(new Set(rows).size,
  1,
  'unique official checkout commit log witness required');
  return exactHead(rows[0]);
}
export function authenticateSourceCommit(commit,
head,
tree){
  exactHead(commit.sha);
  assert.equal(commit.commit.tree.sha,
  tree);
  if(commit.sha!==head){
    assert.equal(commit.parents.length,
    2,
    'only source-owned exact-head synthetic merge supported');
    assert.equal(commit.parents[1].sha,
    head,
    'foreign merge parent');
  }
  return commit;
}
/** Complete nonrecursive root API binds immutable local Git tree, never a truncated recursive response. */
export function authenticateRootTree(root,
expected){
  assert.equal(root.sha,
  expected);
  assert.equal(root.truncated,
  false);
  assert.ok(Array.isArray(root.tree));
  assert.equal(new Set(root.tree.map(x=>x.path)).size,
  root.tree.length);
  for(const row of root.tree){
    assert.ok(['blob',
    'tree'].includes(row.type),
    'unsupported Git entry');
    exactHead(row.sha);
  }
  return root;
}
export function sourceUploadProducer(source,
name){
  const lines=source.split('\n');
  const hits=[];
  let job=null,
  jobName=null;
  for(let i=0;
  i<lines.length;
  i++){
    let m=/^  ([A-Za-z_][\w-]*):\s*$/u.exec(lines[i]);
    if(m){
      job=m[1];
      jobName=null;
      continue;
    }
    m=/^    name: (.+)$/u.exec(lines[i]);
    if(m){
      jobName=m[1].trim();
      continue;
    }
    if(/^\s+(?:- )?uses: actions\/upload-artifact@\S+\s*$/u.test(lines[i])){
      let end=i+1;
      while(end<lines.length&&!/^      - /u.test(lines[end])&&!/^  [A-Za-z_][\w-]*:/u.test(lines[end]))end++;
      const block=lines.slice(i,
      end).join('\n');
      const literal=/^\s+name: ([A-Za-z_][\w-]*)\s*$/mu.exec(block)?.[1];
      if(literal===name){
        assert.ok(job);
        const multiline=/^ {10}path: \|\n((?:^ {12}.+(?:\n|$))*)/gmu.exec(block);
        const single=/^ {10}path: (?!\|)(.+)$/mu.exec(block);
        assert.ok(multiline||single,
        'source-owned upload path required');
        const paths=multiline?multiline[1].trim().split('\n').map(row=>row.trim()):[single[1].trim()];
        assert.ok(paths.length>0&&paths.every(row=>row.length>0));
        hits.push({
          job,
          jobName:jobName??job,
          paths}
        );
      }
    }
  }
  assert.equal(hits.length,
  1,
  'unique literal source-owned upload producer required');
  return hits[0];
}
export function gitBytes(cwd,
commit,
file){
  exactHead(commit);
  assert.match(file,
  /^[A-Za-z0-9_./-]+$/u);
  assert.ok(file.split('/').every(p=>p&&p!=='.'&&p!=='..'));
  return runRead('git',
  ['show',
  commit+':'+file],
  cwd,
  32*1024*1024);
}
export function runRead(command,
args,
cwd,
maxBuffer=32*1024*1024){
  assert.ok(['git',
  'gh'].includes(command),
  'read-only Git/GitHub transport only');
  if(command==='git'){
    assert.ok((args.length===2&&args[0]==='show'&&/^[a-f0-9]{40}:[A-Za-z0-9_./-]+$/u.test(args[1]))||(args.length===2&&args[0]==='rev-parse'&&/^[a-f0-9]{40}\^\{tree\}$/u.test(args[1]))||(args.length===5&&args[0]==='ls-tree'&&args[1]==='-r'&&args[2]==='-t'&&args[3]==='-z'&&/^[a-f0-9]{40}$/u.test(args[4])),
    'unknown Git operation refused');
  }
  else assert.ok(args.length===4&&args[0]==='api'&&args[1]==='--hostname'&&args[2]==='github.com'&&/^repos\/[A-Za-z0-9_.-]+\/[A-Za-z0-9_.-]+\/(actions|commits|git)\/[^\s]+$/u.test(args[3]),
  'unknown GitHub operation refused');
  const result=spawnSync(command,
  args,
  {
    cwd,
    timeout:30000,
    maxBuffer}
  );
  if(result.error)throw result.error;
  assert.equal(result.status,
  0,
  result.stderr?.toString());
  assert.equal(result.signal,
  null);
  return result.stdout;
}
/** Stream all immutable Git blobs without exporting a repository or running source programs. */
export async function immutableSourceWitness(cwd,
commit){
  exactHead(commit);
  const rootTree=runRead('git',
  ['rev-parse',
  commit+'^{tree}'],
  cwd).toString().trim();
  exactHead(rootTree);
  const text=runRead('git',
  ['ls-tree',
  '-r',
  '-t',
  '-z',
  commit],
  cwd).toString();
  const records=[{
    path:'',
    sha:rootTree,
    type:'tree'}
  ,
  ...text.split('\0').filter(Boolean).map(line=>{
    const m=/^(\d+) (blob|tree) ([a-f0-9]{40})\t([^\r\n]+)$/u.exec(line);
    assert.ok(m,
    'unsupported Git object or source path');
    assert.notEqual(m[1],
    '120000',
    'symlink input');
    return {
      path:m[4],
      sha:m[3],
      type:m[2]}
    ;
  }
  )].sort((a,
  b)=>a.path<b.path?-1:a.path>b.path?1:0);
  const rows=records.filter(row=>row.type==='blob');
  assert.ok(rows.length>0&&rows.length<=100000&&records.length<=150000);
  const whole=createHash('sha256'),
  native=createHash('sha256');
  let nativeFiles=0,
  total=0,
  index=0,
  buffer=Buffer.alloc(0),
  state=null;
  const child=spawn('git',
  ['cat-file',
  '--batch'],
  {
    cwd,
    stdio:['pipe',
    'pipe',
    'pipe']}
  );
  let error='';
  child.stderr.on('data',
  b=>{
    error+=b.toString();
    if(error.length>1048576)child.kill('SIGTERM');
  }
  );
  const timer=setTimeout(()=>child.kill('SIGTERM'),
  120000);
  const completion=new Promise((resolve,
  reject)=>{
    child.once('error',
    reject);
    child.once('close',
    (code,
    signal)=>code===0&&signal===null?resolve():reject(Error('Git object stream failed '+error)));
  }
  );
  completion.catch(()=>{
  }
  );
  child.stdin.end(records.map(row=>row.sha+'\n').join(''));
  try{
    for await(const part of child.stdout){
      buffer=Buffer.concat([buffer,
      part]);
      assert.ok(buffer.length<=1024*1024,
      'source stream chunk budget');
      while(buffer.length){
        if(state===null){
          const lineEnd=buffer.indexOf(10);
          if(lineEnd<0){
            assert.ok(buffer.length<=128);
            break;
          }
          assert.ok(index<records.length);
          const m=/^([a-f0-9]{40}) (blob|tree) (\d+)$/u.exec(buffer.subarray(0,
          lineEnd).toString());
          assert.ok(m,
          'missing immutable Git object');
          const row=records[index];
          assert.equal(m[1],
          row.sha);
          assert.equal(m[2],
          row.type);
          const size=Number(m[3]);
          assert.ok(size<=64*1024*1024,
          'immutable source object budget exceeded');
          total+=size;
          assert.ok(total<=2*1024*1024*1024,
          'immutable source total budget exceeded');
          const nativeRow=row.type==='blob'&&nativeRoots.some(root=>row.path===root||row.path.startsWith(root+'/'));
          state={
            row,
            remaining:size,
            hash:createHash('sha1').update(row.type+' '+size+'\0'),
            nativeRow}
          ;
          if(row.type==='blob'){
            whole.update(row.path).update('\0').update(String(size)).update('\0');
            if(nativeRow){
              native.update(row.path).update('\0').update(String(size)).update('\0');
              nativeFiles++;
            }
          }
          buffer=buffer.subarray(lineEnd+1);
        }
        if(state.remaining>0){
          const take=Math.min(state.remaining,
          buffer.length);
          if(take===0)break;
          const bytes=buffer.subarray(0,
          take);
          state.hash.update(bytes);
          if(state.row.type==='blob'){
            whole.update(bytes);
            if(state.nativeRow)native.update(bytes);
          }
          state.remaining-=take;
          buffer=buffer.subarray(take);
        }
        if(state.remaining===0){
          if(buffer.length===0)break;
          assert.equal(buffer[0],
          10);
          assert.equal(state.hash.digest('hex'),
          state.row.sha,
          'immutable Git object content hash differs');
          buffer=buffer.subarray(1);
          state=null;
          index++;
        }
      }
    }
    await completion;
    assert.equal(index,
    records.length);
    assert.equal(state,
    null);
    assert.equal(buffer.length,
    0);
    return {
      whole:{
        sha256:whole.digest('hex'),
        files:rows.length}
      ,
      native:{
        sha256:native.digest('hex'),
        files:nativeFiles}
      ,
      rows}
    ;
  }
  finally{
    clearTimeout(timer);
    child.kill();
  }
}
export function qualifyNativeData({
  kind,
  archive,
  commit,
  tree,
  cwd,
  witness,
  run,
  producerJob,
  checkoutLog}
){
  assert.ok(routes[kind],
  'unknown native data cohort');
  assert.equal(sourceCheckout(checkoutLog),
  commit);
  const rows=witness.rows;
  const lookup=new Map(rows.map(row=>[row.path,
  row]));
  const source=file=>{
    assert.ok(lookup.has(file),
    'undeclared source');
    const b=gitBytes(cwd,
    tree,
    file);
    assert.equal(createHash('sha1').update('blob '+b.length+'\0').update(b).digest('hex'),
    lookup.get(file).sha);
    return b;
  }
  ;
  let profile,
  entries,
  identity;
  const censusPaths=['data/meta/self-ast',
  'data/meta/self-ast.lino',
  'data/meta/self-healing-case.lino',
  'docs/case-studies/issue-538/agent-cli-session-self-ast.json'];
  if(kind==='census'){
    profile=nativeCensusProfile(rows.filter(r=>/^rust\/src\/.+\.rs$/u.test(r.path)).map(r=>r.path),
    censusPaths);
    entries=readQualifiedDataArtifactZip(archive,
    profile);
    const byName=new Map(entries.map(e=>[e.name,
    e.bytes]));
    for(const row of rows.filter(r=>/^rust\/src\/.+\.rs$/u.test(r.path))){
      const b=source(row.path),
      text=byName.get('data/meta/self-ast/'+row.path.slice(5).replace(/\.rs$/u,
      '.lino')).toString('utf8');
      const target=/^  target (.+)$/mu.exec(text)?.[1];
      assert.equal('rust/'+target,
      row.path);
      assert.equal(Number(/^  byte_len (\d+)$/mu.exec(text)?.[1]),
      b.length);
      let h=0xcbf29ce484222325n;
      for(const byte of b)h=((h^BigInt(byte))*0x100000001b3n)&0xffffffffffffffffn;
      assert.equal(/^  content_id (.+)$/mu.exec(text)?.[1],
      'source_module_'+h.toString(16).padStart(16,
      '0'));
    }
    identity={
      commit,
      tree}
    ;
  }
  else if(kind==='sessions'){
    const producer=source('rust/examples/regenerate_agent_cli_sessions.rs');
    profile=nativeSessionArchiveProfile(producer);
    entries=readQualifiedDataArtifactZip(archive,
    profile);
    const receipt=JSON.parse(entries.find(e=>e.name==='producer-receipt.json').bytes);
    identity=receipt.identity;
    assert.equal(identity.commit,
    commit);
    assert.equal(identity.tree,
    tree);
    assert.equal(identity.run,
    String(run.id));
    assert.equal(identity.attempt,
    String(run.run_attempt));
    assert.equal(identity.job,
    producerJob);
    assert.deepEqual(identity.physicalInputs,
    witness.whole);
    assert.deepEqual(identity.nativeInputs,
    witness.native);
    const inputs=projectionRegistry(producer.toString()).map(path=>{
      const bytes=source(path),
      value=JSON.parse(bytes);
      return {
        path,
        sha256:digest(bytes),
        task:value.task,
        tools:value.tools_advertised}
      ;
    }
    );
    verifyProjectionDataEntries(projectionDataProfile(producer,
    inputs,
    identity,
    source('rust/src/agentic_coding/driver.rs')),
    entries);
  }
  else {
    const selection=source('data/meta/native-response-capture-cases.json');
    const preliminary=readQualifiedDataArtifactZip(archive,
    nativeCaptureArchiveEnvelope(selection));
    profile= nativeCaptureProfile(selection,
    preliminary.find(e=>e.name==='report.json').bytes);
    entries=preliminary;
    const report=JSON.parse(entries.find(e=>e.name==='report.json').bytes);
    identity=report.identity;
    assert.equal(identity['source-commit'],
    commit);
    assert.equal(identity['source-tree'],
    tree);
    assert.equal(identity['cargo-lock-sha256'],
    digest(source('rust/Cargo.lock')));
    assert.equal(identity['test-inputs-sha256'],
    witness.whole.sha256);
    assert.equal(identity['test-input-files'],
    witness.whole.files);
    assert.equal(identity['native-inputs-sha256'],
    witness.native.sha256);
    assert.equal(identity['native-input-files'],
    witness.native.files);
    const graph=readNativeTestModuleGraph(cwd);
    for(const target of graph.targets)assert.equal(target.cargoSHA,
    digest(source('rust/Cargo.toml')),
    'current Cargo registration drift');
    for(const row of graph.sources){
      const p=row.path;
      assert.equal(digest(source(p)),
      row.sourceSHA,
      'current source registration drift');
    }
    assert.deepEqual(graph.targets,
    report.sourceRegistration.targets);
    assert.deepEqual(graph.sources,
    report.sourceRegistration.sourceFiles);
    assert.deepEqual(normalizeUnsupportedEdges(graph.unsupported,
    graph.sources),
    normalizeUnsupportedEdges(report.sourceRegistration.unsupportedEdges,
    graph.sources),
    'unsupported source registration differs');
    for(const observation of report.observations)for(const [key,
    value] of Object.entries(bindOriginalSourceCase(graph,
    observation.id)))assert.deepEqual(value,
    observation[key]);
  }
  const members=verifyDataArtifactProfile(profile,
  entries,
  archive);
  return {
    entries,
    proof:{
      schema:'authenticated-native-data-qualification/v1',
      kind,
      authority:'not-granted',
      semanticPassCredit:false,
      identity,
      sourceWitness:{
        whole:witness.whole,
        native:witness.native}
      ,
      ...members}
  }
  ;
}

/** Cohort overlap may share identical generated bytes; conflicting projections refuse. */
export function guardedImportMembers(cohorts) {
  const seen=new Map();
  for(const cohort of cohorts){
    assert.ok(routes[cohort.kind]);
    if(cohort.kind==='answers')continue;
    for(const entry of cohort.entries){
      assert.equal(entry.directory,
      false);
      if(cohort.kind==='sessions'&&!entry.name.startsWith('generated/'))continue;
      const name=cohort.kind==='sessions'?entry.name.slice(10):entry.name;
      const after=digest(entry.bytes);
      if(seen.has(name)){
        assert.equal(seen.get(name),
        after,
        'conflicting source-derived cohort projections');
      }
      else seen.set(name,
      after);
    }
  }
  return [...seen].map(([path,
  sha256])=>({
    path,
    sha256}
  ));
}

/** Absolute hosted workspace prefixes grant no path authority; only one declared Git source suffix may normalize. */
export function normalizeUnsupportedEdges(edges,
sources) {
  assert.ok(Array.isArray(edges)&&Array.isArray(sources));
  return edges.map(row=>{
    assert.equal(typeof row.path,
    'string');
    const matches=sources.filter(source=>row.path.endsWith('/'+source.path));
    assert.equal(matches.length,
    1,
    'unsupported edge requires one declared source path');
    return {
      ...row,
      path:matches[0].path}
    ;
  }
  );
}

/** Physical application inputs must still be the independently witnessed producer blobs. */
export function verifyPhysicalDerivationInputs(cwd,
rows) {
  assert.ok(Array.isArray(rows)&&rows.length>0);
  return rows.map(row=>{
    assert.match(row.path,
    /^[A-Za-z0-9_./-]+$/u);
    assert.ok(row.path.split('/').every(part=>part&&part!=='.'&&part!=='..'));
    const file=join(cwd,
    row.path);
    assert.ok(lstatSync(file).isFile(),
    'nonregular physical derivation source');
    const bytes=readFileSync(file);
    assert.equal(createHash('sha1').update('blob '+bytes.length+'\0').update(bytes).digest('hex'),
    row.sha,
    'current physical derivation source drift: '+row.path);
    return {
      path:row.path,
      gitBlob:row.sha,
      sha256:digest(bytes),
      bytes:bytes.length}
    ;
  }
  );
}
export function nativeDerivationRows(rows,
extraPaths) {
  assert.ok(extraPaths.every(path=>rows.some(row=>row.path===path)),
  'declared derivation source missing');
  return rows.filter(row=>nativeRoots.some(root=>row.path===root||row.path.startsWith(root+'/'))||extraPaths.includes(row.path));
}
