import assert from 'node:assert/strict';
import test from 'node:test';
import fs from 'node:fs';
import {
  join}
from 'node:path';
import {
  tmpdir}
from 'node:os';
import {
  createHash}
from 'node:crypto';
import {
  deflateRawSync}
from 'node:zlib';
import {
  readArtifactZip,
  readQualifiedDataArtifactZip}
from '../../../scripts/github-artifact-by-identifier.mjs';
import {
  nativeCensusProfile,
  nativeCaptureArchiveEnvelope,
  nativeSessionArchiveProfile}
from '../../../scripts/qualified-data-artifact-profiles.mjs';
import {
  authenticateRun,
  completePage,
  authenticateProducerJob,
  authenticateArtifact,
  sourceCheckout,
  authenticateSourceCommit,
  authenticateRootTree,
  sourceUploadProducer,
  digest,
  exactHead,
  runRead,
  guardedImportMembers,
  normalizeUnsupportedEdges,
  verifyPhysicalDerivationInputs,
  nativeDerivationRows}
from '../../../scripts/lib/qualified-native-data-collector.mjs';
const head='a'.repeat(40),
tree='b'.repeat(40),
run={
  id:1,
  run_attempt:1,
  head_sha:head,
  path:'.github/workflows/test.yml',
  status:'completed',
  conclusion:'success'}
;
const crc=b=>{
  let n=0xffffffff;
  for(const v of b){
    n^=v;
    for(let i=0;
    i<8;
    i++)n=(n>>>1)^((n&1)?0xedb88320:0);
  }
  return (n^0xffffffff)>>>0;
}
;
function zip(rows){
  const local=[],
  central=[];
  let pos=0;
  for(const [name,
  body] of rows){
    const raw=Buffer.from(body),
    compressed=deflateRawSync(raw),
    n=Buffer.from(name),
    l=Buffer.alloc(30),
    c=Buffer.alloc(46);
    l.writeUInt32LE(0x04034b50);
    l.writeUInt16LE(20,
    4);
    l.writeUInt16LE(8,
    8);
    l.writeUInt32LE(crc(raw),
    14);
    l.writeUInt32LE(compressed.length,
    18);
    l.writeUInt32LE(raw.length,
    22);
    l.writeUInt16LE(n.length,
    26);
    c.writeUInt32LE(0x02014b50);
    c.writeUInt16LE(20,
    4);
    c.writeUInt16LE(20,
    6);
    c.writeUInt16LE(8,
    10);
    c.writeUInt32LE(crc(raw),
    16);
    c.writeUInt32LE(compressed.length,
    20);
    c.writeUInt32LE(raw.length,
    24);
    c.writeUInt16LE(n.length,
    28);
    c.writeUInt32LE(pos,
    42);
    local.push(l,
    n,
    compressed);
    central.push(c,
    n);
    pos+=l.length+n.length+compressed.length;
  }
  const c=Buffer.concat(central),
  end=Buffer.alloc(22);
  end.writeUInt32LE(0x06054b50);
  end.writeUInt16LE(rows.length,
  8);
  end.writeUInt16LE(rows.length,
  10);
  end.writeUInt32LE(c.length,
  12);
  end.writeUInt32LE(pos,
  16);
  return Buffer.concat([...local,
  c,
  end]);
}
const paths=['data/meta/self-ast',
'data/meta/self-ast.lino',
'data/meta/self-healing-case.lino',
'docs/case-studies/issue-538/agent-cli-session-self-ast.json'];
const largeProfile=nativeCensusProfile(Array.from({
  length:260}
,
(_,
i)=>'rust/src/m'+i+'.rs'),
paths);
test('default256 policy remains; separate source-derived census accepts264 safe CRC entries',
()=>{
  const bytes=zip(largeProfile.members.map(n=>[n,
  'native data']));
  assert.throws(()=>readArtifactZip(bytes),
  /ZIP entry limit/u);
  assert.equal(readQualifiedDataArtifactZip(bytes,
  largeProfile).length,
  264);
}
);
test('copied and forged decoder profiles refuse',
()=>{
  for(const p of [{
    ...largeProfile}
  ,
  {
    entryLimit:65535,
    limits:{
      archiveBytes:Infinity,
      memberBytes:Infinity,
      inflatedBytes:Infinity}
  }
  ])assert.throws(()=>readQualifiedDataArtifactZip(zip([['x',
  'y']]),
  p),
  /compiled data profile/u);
}
);
test('profile decoder refuses excess count, unsafe names, collisions and corrupt CRC',
()=>{
  const p=nativeCensusProfile(['rust/src/a.rs'],
  paths);
  for(const rows of [[['../x',
  'y']],
  [['a',
  'x'],
  ['A',
  'y']],
  Array.from({
    length:6}
  ,
  (_,
  i)=>['x'+i,
  ''])])assert.throws(()=>readQualifiedDataArtifactZip(zip(rows),
  p));
  const corrupt=zip([['safe',
  'data']]);
  corrupt[30+'safe'.length]^=1;
  assert.throws(()=>readQualifiedDataArtifactZip(corrupt,
  p));
}
);
test('zip bomb declared member and inflated budgets refuse before unbounded allocation',
()=>{
  const p=nativeCensusProfile(['rust/src/a.rs'],
  paths),
  b=zip([['x',
  'tiny']]);
  const central=b.indexOf(Buffer.from([0x50,
  0x4b,
  0x01,
  0x02]));
  b.writeUInt32LE(33*1024*1024,
  22);
  b.writeUInt32LE(33*1024*1024,
  central+24);
  assert.throws(()=>readQualifiedDataArtifactZip(b,
  p));
}
);
test('source-selected capture envelope count and session members remain finite',
()=>{
  const p=nativeCaptureArchiveEnvelope(Buffer.from(JSON.stringify({
    schema:'native-response-capture-selection/v1',
    cases:['rust/tests/unit/x.rs::case']}
  )));
  assert.equal(p.entryLimit,
  6);
  const s=nativeSessionArchiveProfile(Buffer.from('let sessions: [(&str, &str); 1] = [(TASK, "docs/case-studies/x/session.json")];'));
  assert.equal(s.entryLimit,
  6);
}
);
test('exact-head run authenticates failed overall run separately from successful producer',
()=>{
  assert.equal(authenticateRun({
    ...run,
    conclusion:'failure'}
  ,
  {
    head,
    workflow:run.path}
  ).conclusion,
  'failure');
  for(const change of [{
    head_sha:tree}
  ,
  {
    path:'.github/workflows/foreign.yml'}
  ,
  {
    status:'in_progress'}
  ,
  {
    conclusion:'cancelled'}
  ])assert.throws(()=>authenticateRun({
    ...run,
    ...change}
  ,
  {
    head,
    workflow:run.path}
  ));
}
);
test('incomplete or truncated API pages and root Git trees refuse',
()=>{
  assert.throws(()=>completePage({
    total_count:2,
    jobs:[{
    }
    ]}
  ,
  'jobs'));
  assert.throws(()=>authenticateRootTree({
    sha:tree,
    truncated:true,
    tree:[]}
  ,
  tree));
  assert.throws(()=>authenticateRootTree({
    sha:head,
    truncated:false,
    tree:[]}
  ,
  tree));
}
);
test('official producer must succeed with exact head/run/name',
()=>{
  const j={
    run_id:1,
    head_sha:head,
    status:'completed',
    conclusion:'success',
    name:'native',
    steps:[{
      conclusion:'success'}
    ]}
  ;
  authenticateProducerJob(j,
  run,
  'native');
  for(const change of [{
    run_id:2}
  ,
  {
    head_sha:tree}
  ,
  {
    conclusion:'failure'}
  ,
  {
    name:'foreign'}
  ])assert.throws(()=>authenticateProducerJob({
    ...j,
    ...change}
  ,
  run,
  'native'));
}
);
test('archive full metadata/digest binding and repeated metadata refuses expiry foreign and drift',
()=>{
  const archive=zip([['x',
  'y']]),
  m={
    id:2,
    name:'data',
    expired:false,
    size_in_bytes:archive.length,
    digest:'sha256:'+digest(archive),
    workflow_run:{
      id:1,
      head_sha:head}
  }
  ;
  authenticateArtifact(m,
  run,
  'data',
  archive,
  structuredClone(m));
  for(const change of [{
    expired:true}
  ,
  {
    name:'foreign'}
  ,
  {
    digest:'sha256:'+'0'.repeat(64)}
  ,
  {
    workflow_run:{
      id:2,
      head_sha:head}
  }
  ])assert.throws(()=>authenticateArtifact({
    ...m,
    ...change}
  ,
  run,
  'data',
  archive,
  structuredClone(m)));
  assert.throws(()=>authenticateArtifact(m,
  run,
  'data',
  archive,
  {
    ...m,
    updated_at:'drift'}
  ));
}
);
test('checkout must come from unique official full Git log witness',
()=>{
  const line='timestamp [command]/usr/bin/git log -1 --format=%H\ntimestamp '+head+'\n';
  assert.equal(sourceCheckout(line),
  head);
  assert.throws(()=>sourceCheckout('HEAD is now '+head));
  assert.throws(()=>sourceCheckout(line+'timestamp [command]/usr/bin/git log -1 --format=%H\ntimestamp '+tree+'\n'));
}
);
test('synthetic merge source binds exact second parent and source tree',
()=>{
  authenticateSourceCommit({
    sha:tree,
    commit:{
      tree:{
        sha:tree}
    }
    ,
    parents:[{
      sha:'c'.repeat(40)}
    ,
    {
      sha:head}
    ]}
  ,
  head,
  tree);
  assert.throws(()=>authenticateSourceCommit({
    sha:tree,
    commit:{
      tree:{
        sha:tree}
    }
    ,
    parents:[{
      sha:head}
    ,
    {
      sha:'c'.repeat(40)}
    ]}
  ,
  head,
  tree));
  assert.throws(()=>exactHead('main'));
}
);
test('upload producer must be unique literal maintained source',
()=>{
  const s='jobs:\n  regenerate:\n    name: Native data\n    steps:\n      - uses: actions/upload-artifact@v7\n        with:\n          name: data\n          path: output\n';
  assert.equal(sourceUploadProducer(s,
  'data').jobName,
  'Native data');
  assert.throws(()=>sourceUploadProducer(s+s,
  'data'));
  assert.throws(()=>sourceUploadProducer(s.replace('name: data',
  'name: ${{ env.NAME }}'),
  'data'));
}
);
test('inflated aggregate64MiB cap remains even when each member is below32MiB',
()=>{
  const p=nativeCensusProfile(['rust/src/a.rs'],
  paths);
  const bytes=zip([['a',
  Buffer.alloc(23*1024*1024)],
  ['b',
  Buffer.alloc(23*1024*1024)],
  ['c',
  Buffer.alloc(23*1024*1024)]]);
  assert.throws(()=>readQualifiedDataArtifactZip(bytes,
  p),
  /ZIP inflated limit/u);
}
);
test('collector read transport refuses unknown Git mutation and GitHub write routes before invocation',
()=>{
  assert.throws(()=>runRead('git',
  ['checkout',
  'main'],
  process.cwd()),
  /unknown Git operation/u);
  assert.throws(()=>runRead('gh',
  ['api',
  '--method',
  'POST',
  'repos/x/y/issues'],
  process.cwd()),
  /unknown GitHub operation/u);
  assert.throws(()=>runRead('cargo',
  ['test'],
  process.cwd()),
  /read-only/u);
}
);
test('source cohort overlap deduplicates equal bytes and refuses conflicting projections',
()=>{
  const entry={
    name:'docs/case-studies/x/session.json',
    directory:false,
    bytes:Buffer.from('same')}
  ;
  const census={
    kind:'census',
    entries:[entry]}
  ,
  session={
    kind:'sessions',
    entries:[{
      ...entry,
      name:'generated/'+entry.name}
    ]}
  ;
  assert.equal(guardedImportMembers([census,
  session]).length,
  1);
  assert.throws(()=>guardedImportMembers([census,
  {
    ...session,
    entries:[{
      ...session.entries[0],
      bytes:Buffer.from('foreign')}
    ]}
  ]),
  /conflicting/u);
  assert.deepEqual(guardedImportMembers([{
    kind:'answers',
    entries:[entry]}
  ]),
  []);
}
);
test('unsupported registration normalizes only independently declared source suffixes',
()=>{
  const sources=[{
    path:'rust/tests/unit/x.rs'}
  ],
  edge={
    path:'/host/work/repo/rust/tests/unit/x.rs',
    reason:'opaque macro',
    trace:[]}
  ;
  assert.equal(normalizeUnsupportedEdges([edge],
  sources)[0].path,
  sources[0].path);
  assert.throws(()=>normalizeUnsupportedEdges([{
    ...edge,
    path:'/host/unknown.rs'}
  ],
  sources),
  /declared source/u);
  assert.throws(()=>normalizeUnsupportedEdges([edge],
  [...sources,
  ...sources]),
  /declared source/u);
}
);
test('import source guard refuses physical source drift and undeclared derivation input',
()=>{
  const root=fs.mkdtempSync(join(fs.realpathSync(tmpdir()),
  'native-collector-source-'));
  try{
    fs.mkdirSync(join(root,
    'rust/src'),
    {
      recursive:true}
    );
    const bytes=Buffer.from('maintained source'),
    row={
      path:'rust/src/a.rs',
      sha:createHash('sha1').update('blob '+bytes.length+'\0').update(bytes).digest('hex')}
    ;
    fs.writeFileSync(join(root,
    row.path),
    bytes);
    assert.equal(verifyPhysicalDerivationInputs(root,
    [row])[0].sha256,
    digest(bytes));
    fs.writeFileSync(join(root,
    row.path),
    'drift');
    assert.throws(()=>verifyPhysicalDerivationInputs(root,
    [row]),
    /physical derivation source drift/u);
    assert.throws(()=>nativeDerivationRows([row],
    ['rust/examples/missing.rs']),
    /declared derivation/u);
  }
  finally{
    fs.rmSync(root,
    {
      recursive:true,
      force:true}
    );
  }
}
);
