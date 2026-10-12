import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import {
  createHash}
from 'node:crypto';
import {
  prepareSourcePackets,
  integrateSourcePackets}
from '../../../experiments/formal_ai_subagent/source-bootstrap-integration.mjs';
const h=s=>createHash('sha256').update(s).digest('hex');
function fixture(){
  const root=fs.mkdtempSync(path.join(os.tmpdir(),
  'bootstrap-packet-controls-'));
  fs.mkdirSync(path.join(root,
  'js'));
  const before='export const value=0;',
  after='export const value=1;';
  fs.writeFileSync(path.join(root,
  'js/item.mjs'),
  before);
  const packet=path.join(root,
  'packet.json');
  const row={
    owner:'js/item.mjs',
    before,
    after,
    beforeSHA256:h(before),
    afterSHA256:h(after)}
  ;
  const save=rows=>fs.writeFileSync(packet,
  JSON.stringify({
    sources:rows}
  ));
  save([row]);
  return {
    root,
    packet,
    row,
    save}
  ;
}
test('exact physical preimages admit generic owners without authoring authority',
()=>{
  const f=fixture();
  try{
    const p=prepareSourcePackets(f.root,
    [f.packet]);
    assert.equal(p.ready,
    true);
    assert.equal(p.sources[0].status,
    'ready');
    assert.equal(p.suppliedPatchAutonomousCredit,
    0);
    assert.equal(p.usageTokens,
    null);
  }
  finally{
    fs.rmSync(f.root,
    {
      recursive:true}
    );
    fs.rmSync(f.root+'-output',
    {
      recursive:true,
      force:true}
    );
  }
}
);
test('root drift refuses apply before runtime or source effects',
async()=>{
  const f=fixture();
  try{
    fs.writeFileSync(path.join(f.root,
    'js/item.mjs'),
    'export const drift=2;');
    const out=f.root+'-output';
    const r=await integrateSourcePackets(f.root,
    [f.packet],
    out,
    {
      apply:true}
    );
    assert.equal(r.applied,
    false);
    assert.deepEqual(r.refusedOwners,
    ['js/item.mjs']);
    assert.equal(fs.existsSync(path.join(out,
    'runtime')),
    false);
    assert.equal(JSON.parse(fs.readFileSync(path.join(out,
    'admission.json'))).ready,
    false);
  }
  finally{
    fs.rmSync(f.root,
    {
      recursive:true}
    );
    fs.rmSync(f.root+'-output',
    {
      recursive:true,
      force:true}
    );
  }
}
);
test('body identity mismatch cannot borrow a frozen hash',
()=>{
  const f=fixture();
  try{
    f.save([{
      ...f.row,
      after:'export const forged=9;'}
    ]);
    assert.throws(()=>prepareSourcePackets(f.root,
    [f.packet]),
    /PacketSourceIdentityMismatch/);
  }
  finally{
    fs.rmSync(f.root,
    {
      recursive:true}
    );
    fs.rmSync(f.root+'-output',
    {
      recursive:true,
      force:true}
    );
  }
}
);
test('unsafe owner paths cannot direct runtime writes',
()=>{
  const f=fixture();
  try{
    for(const owner of ['../escape.mjs',
    '/outside.mjs',
    'js//item.mjs',
    'js/../../escape.mjs',
    'js/item.txt']){
      f.save([{
        ...f.row,
        owner}
      ]);
      assert.throws(()=>prepareSourcePackets(f.root,
      [f.packet]),
      /UnboundSourceRow/);
    }
  }
  finally{
    fs.rmSync(f.root,
    {
      recursive:true}
    );
    fs.rmSync(f.root+'-output',
    {
      recursive:true,
      force:true}
    );
  }
}
);
test('different packet bodies cannot silently share source ownership',
()=>{
  const f=fixture();
  try{
    const other=path.join(f.root,
    'other.json');
    const after='export const other=3;';
    fs.writeFileSync(other,
    JSON.stringify({
      sources:[{
        ...f.row,
        after,
        afterSHA256:h(after)}
      ]}
    ));
    assert.throws(()=>prepareSourcePackets(f.root,
    [f.packet,
    other]),
    /ConflictingSourceOwners/);
  }
  finally{
    fs.rmSync(f.root,
    {
      recursive:true}
    );
    fs.rmSync(f.root+'-output',
    {
      recursive:true,
      force:true}
    );
  }
}
);
test('physical candidate drift is independently checked',
()=>{
  const f=fixture();
  try{
    const candidate=path.join(f.root,
    'candidate.mjs');
    fs.writeFileSync(candidate,
    'export const unrelated=4;');
    f.save([{
      ...f.row,
      candidate}
    ]);
    assert.throws(()=>prepareSourcePackets(f.root,
    [f.packet]),
    /PhysicalCandidateDrift/);
  }
  finally{
    fs.rmSync(f.root,
    {
      recursive:true}
    );
    fs.rmSync(f.root+'-output',
    {
      recursive:true,
      force:true}
    );
  }
}
);
test('existing exact target is classified without duplicate synthesis credit',
()=>{
  const f=fixture();
  try{
    fs.writeFileSync(path.join(f.root,
    'js/item.mjs'),
    f.row.after);
    const p=prepareSourcePackets(f.root,
    [f.packet]);
    assert.equal(p.sources[0].status,
    'already-current');
    assert.equal(p.suppliedPatchAutonomousCredit,
    0);
  }
  finally{
    fs.rmSync(f.root,
    {
      recursive:true}
    );
    fs.rmSync(f.root+'-output',
    {
      recursive:true,
      force:true}
    );
  }
}
);
test('same output cannot silently replace a different declared preimage',
()=>{
  const f=fixture();
  try{
    const other=path.join(f.root,
    'other.json');
    const before='export const alternative=5;';
    fs.writeFileSync(other,
    JSON.stringify({
      sources:[{
        ...f.row,
        before,
        beforeSHA256:h(before)}
      ]}
    ));
    assert.throws(()=>prepareSourcePackets(f.root,
    [f.packet,
    other]),
    /ConflictingSourceOwners/);
  }
  finally{
    fs.rmSync(f.root,
    {
      recursive:true}
    );
    fs.rmSync(f.root+'-output',
    {
      recursive:true,
      force:true}
    );
  }
}
);
test('symlink parent of a new source is not a write scope',
()=>{
  const f=fixture();
  try{
    fs.symlinkSync(f.root,
    path.join(f.root,
    'js/linked'),
    'dir');
    f.save([{
      ...f.row,
      owner:'js/linked/new.mjs',
      before:null,
      beforeSHA256:null}
    ]);
    assert.throws(()=>prepareSourcePackets(f.root,
    [f.packet]),
    /UnownedSourceParent/);
  }
  finally{
    fs.rmSync(f.root,
    {
      recursive:true}
    );
  }
}
);
test('repository outputs and script symlink owners are refused',
async()=>{
  const f=fixture();
  try{
    await assert.rejects(integrateSourcePackets(f.root,
    [f.packet],
    path.join(f.root,
    'output')),
    /OutputMustBeOutsideRepository/);
    f.save([{
      ...f.row,
      owner:'scripts/item.mjs'}
    ]);
    assert.throws(()=>prepareSourcePackets(f.root,
    [f.packet]),
    /UnboundSourceRow/);
  }
  finally{
    fs.rmSync(f.root,
    {
      recursive:true}
    );
  }
}
);


// Sparse fixture collection proves byte membership, never runtime/operation authority.
import {collectSourceRuntimeClosure, stageSourceRuntimeClosure}
  from '../../../experiments/formal_ai_subagent/source-bootstrap-integration.mjs';
function closureFixture(files, entrypoints) {
  const root = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), 'sparse-runtime-')));
  const rows = [];
  for (const [name, text, kind = 'module'] of files) {
    fs.mkdirSync(path.dirname(path.join(root, name)), {recursive: true});
    fs.writeFileSync(path.join(root, name), text);
    rows.push({path: name, sha256: h(text), kind});
  }
  fs.mkdirSync(path.join(root, 'unrelated'));
  fs.writeFileSync(path.join(root, 'unrelated/private.txt'), 'excluded');
  return {root, declaration: {schema: 'source-runtime-closure/v1', files: rows, entrypoints},
    clean: () => fs.rmSync(root, {recursive: true})};
}
for (const [name, files, entrypoints] of [
  ['static module edge', [['js/a.mjs', "import {x} from './b.mjs'; export {x};"],
    ['js/b.mjs', 'export const x=1;']], ['js/a.mjs']],
  ['literal dynamic module edge', [['js/a.mjs', "export const load=()=>import('./b.mjs');"],
    ['js/b.mjs', 'export const x=1;']], ['js/a.mjs']],
  ['literal source-owned binary asset', [['js/a.mjs', "export const url=new URL('./asset.bin',import.meta.url);"],
    ['js/asset.bin', Buffer.from([0, 255, 10]), 'asset']], ['js/a.mjs']],
  ['cyclic finite module edges', [['js/a.mjs', "import './b.mjs'; export const a=1;"],
    ['js/b.mjs', "import './a.mjs'; export const b=2;"]], ['js/a.mjs']],
]) test('sparse closure preserves ' + name + ' without copying unrelated folders', () => {
  const f = closureFixture(files, entrypoints), output = f.root + '-runtime';
  try {
    const closure = collectSourceRuntimeClosure(f.root, f.declaration);
    assert.equal(closure.authority, 'not-granted');
    assert.equal(closure.ordinaryInstalledAcceptance, false);
    assert.equal(stageSourceRuntimeClosure(closure, output).files, files.length);
    assert.equal(fs.existsSync(path.join(output, 'unrelated')), false);
    for (const [name, text] of files) {
      assert.deepEqual(fs.readFileSync(path.join(output, name)), Buffer.from(text));
      assert.equal(fs.lstatSync(path.join(output, name)).isSymbolicLink(), false);
    }
  } finally { f.clean(); fs.rmSync(output, {recursive: true, force: true}); }
});
test('unclosed runtime refuses before directory copies and native or module execution', async () => {
  const f = fixture(), out = fs.realpathSync(f.root) + '-unsupported';
  try {
    const result = await integrateSourcePackets(f.root, [f.packet], out, {apply: true});
    assert.equal(result.applied, false);
    assert.equal(result.runtimeClosure, 'Unknown');
    assert.equal(fs.existsSync(path.join(out, 'runtime')), false);
    assert.equal(fs.existsSync(path.join(out, 'request-0.json')), false);
    assert.equal(JSON.parse(fs.readFileSync(path.join(out, 'runtime-refusal.json'))).authority, 'not-granted');
  } finally { fs.rmSync(f.root, {recursive: true}); fs.rmSync(out, {recursive: true, force: true}); }
});
test('sparse closure refuses unknown dynamic imports resource APIs and stale source declarations', () => {
  for (const source of ["export const x=()=>import(name);", "import fs from 'node:fs';",
    "export const x=()=>fetch('https://example.invalid');", "export const x=new URL(name,import.meta.url);"]) {
    const f = closureFixture([['js/a.mjs', source]], ['js/a.mjs']);
    try { assert.throws(() => collectSourceRuntimeClosure(f.root, f.declaration), /Unsupported/u); }
    finally { f.clean(); }
  }
  const f = closureFixture([['js/a.mjs', 'export const x=1;']], ['js/a.mjs']);
  try {
    assert.throws(() => stageSourceRuntimeClosure({schema: 'qualified-source-runtime-closure/v1'}, f.root+'-runtime'), /Unqualified/u);
    const closure = collectSourceRuntimeClosure(f.root, f.declaration);
    assert.throws(() => stageSourceRuntimeClosure(closure, path.join(f.root, 'runtime')), /Unowned/u);
    fs.writeFileSync(path.join(f.root, 'js/a.mjs'), 'export const x=2;');
    assert.throws(() => stageSourceRuntimeClosure(closure, f.root+'-runtime'), /RuntimeSourceDrift/u);
    assert.equal(fs.existsSync(f.root+'-runtime'), false);
    assert.throws(() => collectSourceRuntimeClosure(f.root, f.declaration), /RuntimeSourceDrift/u);
  } finally { f.clean(); }
});
test('sparse closure conserves candidate edges and refuses missing duplicate extra and symlink members', () => {
  const f = closureFixture([['js/a.mjs', "import './b.mjs';"], ['js/b.mjs', 'export const x=1;']], ['js/a.mjs']);
  try {
    assert.throws(() => collectSourceRuntimeClosure(f.root, {...f.declaration, files: f.declaration.files.slice(0, 1)}), /Unclosed/u);
    assert.throws(() => collectSourceRuntimeClosure(f.root, {...f.declaration, files: [...f.declaration.files, f.declaration.files[0]]}), /Unbound/u);
    assert.throws(() => collectSourceRuntimeClosure(f.root, {...f.declaration, entrypoints: ['js/b.mjs']}), /Reachability/u);
    assert.throws(() => collectSourceRuntimeClosure(f.root, f.declaration, [{owner: 'js/c.mjs', after: "import './missing.mjs';"}]), /Unclosed/u);
    fs.unlinkSync(path.join(f.root, 'js/b.mjs'));
    fs.symlinkSync('/etc/hosts', path.join(f.root, 'js/b.mjs'));
    assert.throws(() => collectSourceRuntimeClosure(f.root, f.declaration), /Unowned/u);
  } finally { f.clean(); }
});
