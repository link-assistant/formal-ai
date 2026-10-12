import test from 'node:test';
import assert from 'node:assert/strict';
import { mkdtempSync, writeFileSync, readFileSync, rmSync, existsSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { createHash } from 'node:crypto';
import { inputSnapshots, createCoverageCheckpoints, assertFreshCoverageOutputs } from '../../../scripts/lib/coverage-checkpoints.mjs';

function fixture() {
  const directory = mkdtempSync(join(tmpdir(), 'coverage-capture-contract-'));
  const rows = ['unit\tprogram\n', 'unit\ta\nunit\tb\n', 'unit\ta\nunit\tb\n', 'native\n', 'profile\n'];
  const paths = rows.map((text, index) => {
    const path = join(directory, `input-${index}`);
    writeFileSync(path, text);
    return path;
  });
  const executable = join(directory, 'binary');
  writeFileSync(executable, Buffer.from([0, 10, 255, 13, 0]));
  const batches = [{target:'unit',names:['a'],seconds:2}, {target:'unit',names:['b'],seconds:1}];
  const executables = new Map([['unit', executable]]);
  const output = join(directory, 'capture');
  const create = () => createCoverageCheckpoints(output, inputSnapshots(paths), {source:'observed'}, batches, executables);
  return {directory, paths, rows, executable, batches, executables, output, create,
    close:() => rmSync(directory,{recursive:true,force:true})};
}
const start = (index, name) => ({index,target:'unit',names:[name],argv:['--exact',name],executable:'observed',weight:1});
const finish = (index,name) => ({...start(index,name),selected:1,completed:1,elapsedSeconds:1,
  exitCode:0,signal:null,summary:{passed:1,failed:0,ignored:0},problems:[],succeeded:true});

test('capture precedes execution and keeps complete raw inventory and batch order', () => {
  const f=fixture();
  try {
    const capture=f.create();
    assert.deepEqual(readFileSync(join(f.output,'tests.tsv')),Buffer.from(f.rows[1]));
    const plan=JSON.parse(readFileSync(join(f.output,'capture.json')));
    assert.deepEqual(plan.batches.map(x=>x.names),[['a'],['b']]);
    const expected=createHash('sha256').update(readFileSync(f.executable)).digest('hex');
    assert.equal(plan.context.executableInputs[0].sha256,expected);
    assert.equal(plan.context.executableInputs[0].bytes,5);
    assert.equal(plan.executionSourceAuthentication,'Unknown');
    capture.start(start(0,'a'));
    capture.finish(finish(0,'a'));
    capture.start(start(1,'b'));
    assert(existsSync(join(f.output,'completed-0.json')));
    assert(existsSync(join(f.output,'start-1.json')));
    assert(!existsSync(join(f.output,'completed-1.json')));
    assert.equal(JSON.parse(readFileSync(join(f.output,'completed-0.json'))).batch.names[0],'a');
  } finally {f.close();}
});
test('source drift and selected case loss refuse before output creation', () => {
  const f=fixture();
  try {
    const inputs=inputSnapshots(f.paths);
    writeFileSync(f.paths[1],'changed');
    assert.throws(()=>createCoverageCheckpoints(f.output,inputs,{},f.batches,f.executables),/changed/);
    assert(!existsSync(f.output));
    writeFileSync(f.paths[1],f.rows[1]);
    assert.throws(()=>createCoverageCheckpoints(f.output,inputSnapshots(f.paths),{},f.batches.slice(0,1),f.executables),/conserve/);
    assert(!existsSync(f.output));
  } finally {f.close();}
});
test('duplicate starts, foreign names and completion without start refuse', () => {
  const f=fixture();
  try {
    const capture=f.create();
    assert.throws(()=>capture.finish(finish(0,'a')),/start/);
    assert.throws(()=>capture.start(start(0,'foreign')),/captured/);
    capture.start(start(0,'a'));
    assert.throws(()=>capture.start(start(0,'a')),/captured/);
    assert.throws(()=>capture.finish(finish(0,'foreign')),/names/);
    capture.finish(finish(0,'a'));
    assert.throws(()=>capture.finish(finish(0,'a')),/unique/);
  } finally {f.close();}
});
test('existing capture namespace refuses without overwrite', () => {
  const f=fixture();
  try {
    f.create();
    const before=readFileSync(join(f.output,'capture.json'));
    assert.throws(()=>f.create());
    assert.deepEqual(readFileSync(join(f.output,'capture.json')),before);
  } finally {f.close();}
});

test('result input collision and reused outputs refuse before execution', () => {
  const f=fixture();
  try {
    const inputs=inputSnapshots(f.paths);
    assert.throws(()=>assertFreshCoverageOutputs(f.paths[2],f.output,inputs),/input/);
    assert(!existsSync(f.output));
    const result=join(f.directory,'existing-result.json');
    writeFileSync(result,'original');
    assert.throws(()=>assertFreshCoverageOutputs(result,f.output,inputs),/exists/);
    assert.equal(readFileSync(result,'utf8'),'original');
    assertFreshCoverageOutputs(join(f.directory,'fresh-result.json'),f.output,inputs);
  } finally {f.close();}
});

test('overlapping result and receipt namespaces refuse before outputs', () => {
  const f=fixture();
  try {
    const inputs=inputSnapshots(f.paths);
    assert.throws(()=>assertFreshCoverageOutputs(join(f.output,'capture.json'),f.output,inputs),/overlap/);
    assert.throws(()=>assertFreshCoverageOutputs(f.output,join(f.output,'receipts'),inputs),/overlap/);
    assert(!existsSync(f.output));
  } finally {f.close();}
});
test('unavailable executable observation does not suppress valid peer dispatch', () => {
  const f=fixture();
  try {
    f.executables.set('unselected',join(f.directory,'missing-executable'));
    const capture=f.create();
    const missing=capture.context.executableInputs.find(x=>x.target==='unselected');
    assert.equal(missing.status,'Missing');
    assert.equal(missing.observationError.code,'ENOENT');
    assert.equal(missing.sha256,undefined);
    capture.start(start(0,'a'));
    capture.finish(finish(0,'a'));
    assert(existsSync(join(f.output,'completed-0.json')));
  } finally {f.close();}
});
