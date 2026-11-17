import test from 'node:test';
import assert from 'node:assert/strict';
import {mkdtempSync,mkdirSync,writeFileSync,rmSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join,dirname} from 'node:path';
import {createHash} from 'node:crypto';
import {traceTree} from './trace-template-ci.mjs';

function fixture(run, additions={}){
 const directory=mkdtempSync(join(tmpdir(),'formal-ai-ci-graph-'));
 const inputs={
  '.github/workflows/check.yml':'name: Check\non: push\njobs:\n  check:\n    steps:\n      - uses: ./.github/actions/check\n      - run: node scripts/check.mjs\n      - run: node scripts/${DYNAMIC}.mjs\n',
  '.github/actions/check/action.yml':'name: Local\nruns:\n  using: node24\n  main: index.mjs\n',
  '.github/actions/check/index.mjs':"import './helper.mjs';\n",
  '.github/actions/check/helper.mjs':'export const valid=true;\n',
  'scripts/check.mjs':"import './lib/guard.mjs';\nconsole.log('bounded fixture');\n",
  'scripts/lib/guard.mjs':'export const guard=true;\n',
  'unrelated/private.txt':'No reference from a workflow.\n',
 };
 Object.assign(inputs,additions);
 const inventory=Object.entries(inputs).map(([path,text])=>{
  const bytes=Buffer.from(text);mkdirSync(dirname(join(directory,path)),{recursive:true});writeFileSync(join(directory,path),bytes);
  return {path,gitBlob:createHash('sha1').update(`blob ${bytes.length}\0`).update(bytes).digest('hex')};
 });
 try{run(directory,inventory);}finally{rmSync(directory,{recursive:true,force:true});}
}

test('reference inventory follows nested local action/module helpers and preserves unresolved paths',{timeout:5000},()=>{
 fixture((directory,inventory)=>{
  const graph=traceTree(directory,inventory);
  assert.deepEqual(graph.roots,['.github/workflows/check.yml']);
  for(const file of ['.github/actions/check/index.mjs','.github/actions/check/helper.mjs','scripts/check.mjs','scripts/lib/guard.mjs'])assert.ok(graph.files.includes(file),file);
  assert.equal(graph.files.includes('unrelated/private.txt'),false);
  assert.ok(graph.unresolved.some(row=>row.value.includes('${DYNAMIC}')));
  assert.match(graph.coverageClaim,/not complete execution closure/u);
  assert.equal(graph.commands.length,2);
 });
});

test('a changed transitive helper cannot silently retain its pinned source identity',{timeout:5000},()=>{
 fixture((directory,inventory)=>{
  writeFileSync(join(directory,'scripts/lib/guard.mjs'),'export const guard=false;\n');
  assert.throws(()=>traceTree(directory,inventory),/source drift: scripts\/lib\/guard.mjs/u);
 });
});

const selfRepositoryInputs={
 '.github/workflows/check.yml':`name: Check
on: push
jobs:
  reusable:
    uses: $/.github/workflows/reusable.yml
  check:
    steps:
      - uses: $/.github/actions/check
      - uses: external/action@0123456789012345678901234567890123456789
`,
 '.github/workflows/reusable.yml':'name: Reusable\non: workflow_call\njobs:\n  pass:\n    steps:\n      - run: echo bounded\n',
 '.github/actions/check/action.yml':'name: Composite\nruns:\n  using: composite\n  steps:\n    - uses: $/.github/actions/leaf\n',
 '.github/actions/leaf/action.yml':'name: Leaf\nruns:\n  using: node24\n  main: main.mjs\n  pre: pre.mjs\n  post: post.mjs\n',
 '.github/actions/leaf/main.mjs':"import './helper.mjs';\n",
 '.github/actions/leaf/pre.mjs':'export const before=true;\n',
 '.github/actions/leaf/post.mjs':'export const after=true;\n',
 '.github/actions/leaf/helper.mjs':'export const checked=true;\n',
};

test('self-repository uses follows reusable workflows, nested composites and JavaScript action phases',{timeout:5000},()=>{
 fixture((directory,inventory)=>{
  const graph=traceTree(directory,inventory);
  assert.ok(graph.edges.some(edge=>edge.from==='.github/workflows/check.yml'&&edge.to==='.github/workflows/reusable.yml'&&edge.reason==='self-repository reusable workflow'));
  assert.ok(graph.edges.some(edge=>edge.from==='.github/workflows/check.yml'&&edge.to==='.github/actions/check/action.yml'&&edge.reason==='self-repository action'));
  assert.ok(graph.edges.some(edge=>edge.from==='.github/actions/check/action.yml'&&edge.to==='.github/actions/leaf/action.yml'&&edge.reason==='self-repository action'));
  for(const file of ['main.mjs','pre.mjs','post.mjs','helper.mjs']) assert.ok(graph.files.includes('.github/actions/leaf/'+file),file);
  assert.equal(graph.files.includes('unrelated/private.txt'),false);
  assert.equal(graph.unresolved.some(row=>row.value.startsWith('external/action@')),false);
  assert.match(graph.coverageClaim,/not complete execution closure/u);
 },selfRepositoryInputs);
});

test('missing, dynamic and escaping self-repository paths remain explicit unresolved evidence',{timeout:5000},()=>{
 fixture((directory,inventory)=>{
  const graph=traceTree(directory,inventory);
  assert.ok(graph.unresolved.some(row=>row.value==='$/.github/actions/missing'&&row.reason.startsWith('no static file match:')));
  for(const value of ['$/../outside','$/..']) assert.ok(graph.unresolved.some(row=>row.value===value&&row.reason==='reference leaves the source tree'),value);
  assert.ok(graph.unresolved.some(row=>row.value==='$/.github/actions/${{ inputs.action }}'&&row.reason.includes('dynamic')));
 },{'.github/workflows/check.yml':'name: Check\non: push\njobs:\n  check:\n    steps:\n      - uses: $/.github/actions/missing\n      - uses: $/../outside\n      - uses: $/..\n      - uses: $/.github/actions/${{ inputs.action }}\n'});
});

test('self-repository transitive files retain pinned-blob verification',{timeout:5000},()=>{
 fixture((directory,inventory)=>{
  writeFileSync(join(directory,'.github/actions/leaf/helper.mjs'),'export const checked=false;\n');
  assert.throws(()=>traceTree(directory,inventory),/source drift: \.github\/actions\/leaf\/helper.mjs/u);
 },selfRepositoryInputs);
});
