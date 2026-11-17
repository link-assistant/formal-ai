import test from 'node:test';
import assert from 'node:assert/strict';
import {mkdtempSync,mkdirSync,writeFileSync,rmSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join,dirname} from 'node:path';
import {createHash} from 'node:crypto';
import {traceTree} from './trace-template-ci.mjs';

function fixture(run){
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
